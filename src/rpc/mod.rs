// src/rpc/mod.rs
// QTC M8: JSON-RPC HTTP server
// Includes standard eth_* methods + custom qtc_* methods for web dashboard.
//
// AUDIT-019 FIX: Custom rate limiter via axum middleware::from_fn_with_state.

pub mod methods;
pub mod qtc_methods;

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
    State(state): State<AppState>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    Json(dispatch(&state, req))
}

fn dispatch(state: &AppState, req: RpcRequest) -> RpcResponse {
    let id = req.id.clone();
    use methods::*;

    let result = match req.method.as_str() {
        // Standard eth_* methods
        "eth_chainId"             => Ok(eth_chain_id()),
        "eth_blockNumber"         => Ok(eth_block_number(state)),
        "eth_getBalance"          => eth_get_balance(state, &req.params),
        "eth_getTransactionCount" => eth_get_transaction_count(state, &req.params),
        "eth_getBlockByNumber"    => eth_get_block_by_number(state, &req.params),
        "eth_sendRawTransaction"  => eth_send_raw_transaction(state, &req.params),

        // Custom qtc_* methods for web dashboard
        "qtc_getValidator"        => qtc_methods::qtc_get_validator(state, &req.params),
        "qtc_getNetworkStats"     => Ok(qtc_methods::qtc_get_network_stats(state)),
        "qtc_getValidators"       => Ok(qtc_methods::qtc_get_validators(state)),
        // M14 WIRING (core-dev review, P2): read-only vesting/governance
        "qtc_getVestingSchedule"  => qtc_methods::qtc_get_vesting_schedule(state, &req.params),
        "qtc_getProposal"         => qtc_methods::qtc_get_proposal(state, &req.params),

        other => return RpcResponse::err(
            id, ERR_METHOD_NOT_FOUND,
            format!("method not found: {other}")
        ),
    };

    match result {
        Ok(value) => RpcResponse::ok(id, value),
        Err(msg) if msg.starts_with("__INTERNAL__") =>
            RpcResponse::err(id, ERR_INTERNAL,
                msg.trim_start_matches("__INTERNAL__")),
        Err(msg) => RpcResponse::err(id, ERR_INVALID_PARAMS, msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::Mempool;
    use crate::state::{StateDB, Storage};
    use serde_json::json;
    use tower::ServiceExt;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};

    fn test_state() -> AppState {
        let tmp = tempfile::TempDir::new().unwrap();
        AppState {
            state_db: Arc::new(Mutex::new(StateDB::new())),
            mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
            storage: Arc::new(Storage::open_at(tmp.path()).unwrap()),
            chain_head: Arc::new(Mutex::new(ChainHead::default())),
            outbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[tokio::test]
    async fn test_rpc_chain_id_over_http() {
        std::env::remove_var("QC_NETWORK");
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
        assert_eq!(v["result"], json!(methods::u64_to_hex(methods::TESTNET_CHAIN_ID)));
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

    #[tokio::test]
    async fn test_qtc_get_network_stats() {
        let app = router(test_state());
        let body = json!({
            "jsonrpc": "2.0",
            "method": "qtc_getNetworkStats",
            "params": [],
            "id": 3
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
        assert!(v["result"]["blockNumber"].is_string());
        assert_eq!(v["result"]["network"], json!("testnet"));
    }

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
        let r1 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r1.status(), StatusCode::OK);
        let r2 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r2.status(), StatusCode::OK);
        let r3 = app.clone().oneshot(make_req()).await.unwrap();
        assert_eq!(r3.status(), StatusCode::TOO_MANY_REQUESTS);
        std::env::remove_var("QC_RPC_RATE_LIMIT");
    }
}

