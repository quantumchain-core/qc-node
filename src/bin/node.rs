// src/bin/node.rs
// QTC M8: Node binary entrypoint
//
// Wires together: in-memory StateDB, Mempool, sled Storage, and the
// JSON-RPC HTTP server. The libp2p swarm + gossip event loop (M2/M7)
// is started but message pumping between RPC outbox <-> swarm
// publish/subscribe is a TODO for M9 (needs a shared async task).

use std::sync::{Arc, Mutex};
use qc_node::mempool::Mempool;
use qc_node::rpc::{self, AppState, ChainHead};
use qc_node::state::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Storage ---
    let storage = Storage::new()?;

    // --- Shared state ---
    let state = AppState {
        state_db: Arc::new(Mutex::new(
            storage.get_state()?.unwrap_or_default(),
        )),
        mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
        storage: Arc::new(storage),
        chain_head: Arc::new(Mutex::new(ChainHead::default())),
        outbox: Arc::new(Mutex::new(Vec::new())),
    };

    // --- P2P swarm (M2/M7) ---
    // TODO M9: spawn a task that:
    //   1. polls swarm.select_next_some() for incoming gossip,
    //      calls net::handle_gossip(&data, &mempool, &head_hash)
    //   2. drains state.outbox and calls net::publish(&mut swarm, &msg)
    let _swarm = qc_node::net::new_swarm().await?;

    // --- RPC server ---
    let app = rpc::router(state);
    let addr = std::env::var("QC_RPC_ADDR").unwrap_or_else(|_| "0.0.0.0:8545".to_string());
    println!("QTC node RPC listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
