// src/rpc/mod.rs
// QTC M8: JSON-RPC HTTP server
//
// Single POST endpoint at "/" accepts JSON-RPC 2.0 requests and dispatches
// to the method implementations in `methods.rs`. Shared node state
// (StateDB, Mempool, Storage, ChainHead) is passed via AppState.

pub mod methods;

pub use methods::{AppState, ChainHead, RpcRequest, RpcResponse};

use axum::{routing::post, Json, Router};

/// Build the RPC router. Call `.serve()` on the result from your node binary:
///
/// ```ignore
/// let app = rpc::router(state);
/// let listener = tokio::net::TcpListener::bind("0.0.0.0:8545").await?;
/// axum::serve(listener, app).await?;
/// ```
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", post(handle_rpc))
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
    use tower::ServiceExt; // for `oneshot`
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

        assert_eq!(response.status(), StatusCode::OK); // JSON-RPC errors are HTTP 200

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"]["code"], json!(methods::ERR_METHOD_NOT_FOUND));
    }
}
