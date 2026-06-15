// src/bin/node.rs
// QTC M9/M10: Node binary — async event loop
//
// Wires together everything built in M1-M10:
//   - libp2p swarm (M2/M7): gossip in/out
//   - Node (M9 core): on_gossip / try_produce_block / drain_outbox
//   - ValidatorRegistry (M10): multi-validator genesis config + real sig verify
//   - RPC server (M8): runs concurrently, shares AppState via Arc<Mutex<...>>
//
// Loop:
//   1. Incoming gossip -> node.on_gossip() -> mempool/state/storage updated
//   2. Every BLOCK_TIME_SECS -> node.try_produce_block()
//   3. Anything queued (new txs from RPC, new blocks we produced)
//      -> drained and published to peers via net::publish()

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use libp2p::{gossipsub, swarm::SwarmEvent};

use qc_node::chain::Address;
use qc_node::consensus::{Producer, ValidatorRegistry, BLOCK_TIME_SECS};
use qc_node::crypto::generate_keypair;
use qc_node::mempool::Mempool;
use qc_node::net::{self, QcBehaviourEvent};
use qc_node::node::Node;
use qc_node::rpc::{self, AppState, ChainHead};
use qc_node::state::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Storage & shared state ---
    let storage = Storage::new()?;
    let state_db = storage.get_state()?.unwrap_or_default();

    let app_state = AppState {
        state_db: Arc::new(Mutex::new(state_db)),
        mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
        storage: Arc::new(storage),
        chain_head: Arc::new(Mutex::new(ChainHead::default())),
        outbox: Arc::new(Mutex::new(Vec::new())),
    };

    // --- Validator identity ---
    // TODO M11: load from a persistent keystore instead of generating fresh each run
    let (pk, sk) = generate_keypair();
    let coinbase: Address = [9u8; 32]; // fee recipient — TODO M11: make configurable
    let producer = Producer::new(sk, pk.clone(), coinbase);

    // --- Validator registry (M10) ---
    // QC_GENESIS_PATH points to a JSON file listing all known validators'
    // (address, pubkey) pairs. If unset, fall back to a single-validator
    // registry containing only this node's own pubkey (local dev / testnet-of-one).
    let registry = match std::env::var("QC_GENESIS_PATH") {
        Ok(path) => {
            println!("loading validator registry from {path}");
            ValidatorRegistry::load_from_file(&path)?
        }
        Err(_) => {
            println!("QC_GENESIS_PATH not set; using self-registered single-validator registry");
            ValidatorRegistry::single(&pk)
        }
    };
    println!("validator registry loaded: {} validator(s)", registry.len());

    let mut node = Node::new(app_state.clone(), producer, registry);

    // --- RPC server (M8) ---
    let rpc_app = rpc::router(app_state.clone());
    let rpc_addr = std::env::var("QC_RPC_ADDR").unwrap_or_else(|_| "0.0.0.0:8545".to_string());
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    println!("QTC node RPC listening on {rpc_addr}");
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, rpc_app).await {
            eprintln!("RPC server error: {e}");
        }
    });

    // --- P2P swarm (M2/M7) ---
    let mut swarm = net::new_swarm().await?;
    println!("QTC node P2P peer id: {}", swarm.local_peer_id());

    // --- Block production timer ---
    let mut block_timer = tokio::time::interval(Duration::from_secs(BLOCK_TIME_SECS));

    // --- Main event loop ---
    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                if let SwarmEvent::Behaviour(QcBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. }
                )) = event {
                    let result = node.on_gossip(&message.data);
                    println!("gossip received: {result:?}");
                }
            }
            _ = block_timer.tick() => {
                match node.try_produce_block() {
                    Ok(Some(block)) => println!("produced block #{}", block.header.number),
                    Ok(None) => {} // mempool empty, nothing to do
                    Err(e) => eprintln!("block production failed: {e}"),
                }
            }
        }

        // Publish anything queued: new txs from RPC, new blocks we just produced
        for msg in node.drain_outbox() {
            if let Err(e) = net::publish(&mut swarm, &msg) {
                eprintln!("publish failed: {e}");
            }
        }
    }
}
