// src/rpc/mod.rs
// QTC M8: JSON-RPC HTTP server
//
// AUDIT-019 FIX: RPC rate limiting via a custom axum middleware function
// (not tower::limit::RateLimitLayer, which does not implement Clone and
// is incompatible with axum 0.7's Router::layer requirements).
//
// Approach: a simple fixed-window counter behind a Mutex, shared via
// Arc and injected as axum middleware with axum::middleware::from_fn_with_state.
// Default: 100 requests per second per node instance.
// Override with QC_RPC_RATE_LIMIT env var.

pub mod methods;

pub use methods::{AppState, ChainHead, RpcRequest, RpcResponse};

use axum::{
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::post,
    Json, Router,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DEFAULT_RATE_LIMIT: u64 = 100;

/// Simple fixed-window rate limiter: tracks request count and window start.
/// Resets every 1 second. Not as precise as a sliding window, but sufficient
/// to stop mempool-flooding DOS attacks (AUDIT-019).
#[derive(Clone)]
struct RateLimiter {
    inner: Arc<Mutex<RateLimiterState>>,
    max_per_second: u64,
}

struct RateLimiterState {
    count: u64,
    window_start: Instant,
}

impl RateLimiter {
    fn new(max_per_second: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterState {
                count: 0,
                window_start: Instant::now(),
            })),
            max_per_second,
        }
    }

    /// Returns true if the request is allowed, false if rate limited.
    fn check(&self) -> bool {
        let mut state = self.inner.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(state.window_start) >= Duration::from_secs(1) {
            state.count = 0;
            state.window_start = now;
        }

        if state.count >= self.max_per_second {
            return false;
        }

        state.count += 1;
        true
    }
}

async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if limiter.check() {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

/// Build the RPC router with rate limiting.
pub fn router(state: AppState) -> Router {
    let rate_limit = std::env::var("QC_RPC_RATE_LIMIT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT);

    let limiter = RateLimiter::new(rate_limit);

    Router::new()
        .route("/", post(handle_rpc))
        .route_layer(middleware::from_fn_with_state(limiter, rate_limit_middleware))
        .with_state(state)
}

async fn handle_rpc(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    Json(methods::dispatch(&state, req))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::Mempool;
    use crate::state::{StateDB, Storage};
    use serde_json::json;
    use tower::ServiceExt;
    use axum::body::Body;
    use axum::http::Request;

    fn test_state() -> AppState {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());
        AppState {
            state_db: Arc::new(Mutex::new(StateDB::new())),
            mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
            storage: Arc::new(Storage::new().unwrap()),
            chain_head: Arc::new(Mutex::new(ChainHead::default())),
            outbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[tokio::test]
    async fn test_rpc_chain_id_over_http() {
        let app = router(test_state());
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["result"], json!(methods::u64_to_hex(methods::QTC_CHAIN_ID)));
        assert_eq!(v["id"], json!(1));
    }

    #[tokio::test]
    async fn test_rpc_unknown_method_over_http() {
        let app = router(test_state());
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_doesNotExist",
            "params": [],
            "id": 2
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"]["code"], json!(methods::ERR_METHOD_NOT_FOUND));
    }

    // AUDIT-019: rate limiter rejects requests over the limit
    #[tokio::test]
    async fn test_rate_limiter_blocks_excess_requests() {
        std::env::set_var("QC_RPC_RATE_LIMIT", "2");
        let app = router(test_state());

        let make_req = || {
            let body = json!({"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1});
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap()
        };

        // First 2 requests succeed (limit=2)
        let r1 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r1.status(), StatusCode::OK);
        let r2 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r2.status(), StatusCode::OK);

        // 3rd request in the same window is rate limited
        let r3 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r3.status(), StatusCode::TOO_MANY_REQUESTS);

        std::env::remove_var("QC_RPC_RATE_LIMIT");
    }
}
