// src/rpc/mod.rs
// QTC M8: JSON-RPC HTTP server
//
// AUDIT-019 FIX: RPC rate limiting via tower ServiceBuilder + RateLimitLayer.
// Default: 100 requests per second. Override with QC_RPC_RATE_LIMIT env var.
// Note: RateLimitLayer from tower 0.4 requires ServiceBuilder wrapping
// because RateLimit<Route> does not implement Clone directly.

pub mod methods;

pub use methods::{AppState, ChainHead, RpcRequest, RpcResponse};

use axum::{routing::post, Json, Router};
use std::time::Duration;
use tower::ServiceBuilder;
use tower::limit::RateLimitLayer;

const DEFAULT_RATE_LIMIT: u64 = 100;

pub fn router(state: AppState) -> Router {
    let rate_limit = std::env::var("QC_RPC_RATE_LIMIT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT);

    Router::new()
        .route("/", post(handle_rpc))
        .layer(ServiceBuilder::new().layer(RateLimitLayer::new(rate_limit, Duration::from_secs(1))))
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
    use std::sync::{Arc, Mutex};
    use serde_json::json;
    use tower::ServiceExt;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};

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
                }
