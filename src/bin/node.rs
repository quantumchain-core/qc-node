// src/bin/node.rs
//
// DEDUP FIX (core-dev review): this file used to carry its own private
// copy of the entire encrypted-keystore implementation (Keystore struct,
// derive_key, require_keystore_password, load_or_generate_keypair,
// restrict_keystore_permissions) — byte-for-byte identical to
// qc_node::keystore, which was extracted from this file specifically so
// bin/keygen.rs could share it. keygen.rs was migrated to the shared
// module; this binary never was, so the real node and the shared library
// module could silently drift apart if only one got edited. Now uses
// qc_node::keystore directly, same as keygen.rs does.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use libp2p::{gossipsub, identify, request_response, swarm::SwarmEvent};

use qc_node::chain::Address;
use qc_node::consensus::{address_from_pubkey, Producer, ValidatorRegistry, BLOCK_TIME_SECS};
use qc_node::keystore::load_or_generate_keypair;
use qc_node::mempool::Mempool;
use qc_node::net::{self, GossipMsg, QcBehaviourEvent};
use qc_node::node::Node;
use qc_node::rpc::{self, AppState, ChainHead};
use qc_node::state::Storage;
use qc_node::sync;

fn load_coinbase(pk: &[u8]) -> Result<Address, Box<dyn std::error::Error>> {
    match std::env::var("QC_COINBASE") {
        Ok(hex_str) => {
            let clean = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
            let bytes = hex::decode(clean)?;
            if bytes.len() != 32 { return Err("Invalid coinbase length".into()); }
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&bytes);
            Ok(addr)
        }
        Err(_) => Ok(address_from_pubkey(pk)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = std::env::var("QC_NETWORK").unwrap_or_else(|_| "testnet".to_string());
    println!("================================================\n  QTC NODE -- {} \n================================================", network.to_uppercase());

    let storage = Storage::new()?;
    let state_db = storage.get_state()?.unwrap_or_default();

    let app_state = AppState {
        state_db: Arc::new(Mutex::new(state_db)),
        mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
        storage: Arc::new(storage),
        chain_head: Arc::new(Mutex::new(ChainHead::default())),
        outbox: Arc::new(Mutex::new(Vec::new())),
    };

    let (pk, sk) = load_or_generate_keypair()?;
    let coinbase = load_coinbase(&pk)?;
    let producer = Producer::new(sk, pk.clone(), coinbase);

    let registry = match std::env::var("QC_GENESIS_PATH") {
        Ok(path) => ValidatorRegistry::load_from_file(&path)?,
        Err(_) => ValidatorRegistry::single(&pk),
    };

    println!("Validator registry: {} validator(s)", registry.len());

    let mut node = Node::new(app_state.clone(), producer, registry);

    let rpc_app = rpc::router(app_state.clone());
    // Default to localhost-only if QC_RPC_ADDR isn't set. Previously
    // defaulted to 0.0.0.0 (all interfaces) — meaning a dropped/missing
    // env var during a restart would silently expose the RPC port to the
    // public internet instead of failing safe. Explicit opt-in to a
    // public bind (set QC_RPC_ADDR yourself) is safer than an accidental
    // public default.
    let rpc_addr = std::env::var("QC_RPC_ADDR").unwrap_or_else(|_| "127.0.0.1:8545".to_string());
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    tokio::spawn(async move { let _ = axum::serve(listener, rpc_app).await; });

    let mut swarm = net::new_swarm().await?;
    net::start_listening(&mut swarm)?;
    net::dial_bootstrap_peers(&mut swarm);
    let mut block_timer = tokio::time::interval(Duration::from_secs(BLOCK_TIME_SECS));

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(QcBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    })) => {
                        // Peek at the block number before handing to on_gossip: a
                        // gap always means on_gossip will reject it, but we still
                        // want to know the peer + number to request the missing
                        // range from. on_gossip is called either way — this is
                        // purely additive, not a replacement for its validation.
                        if let Ok(GossipMsg::NewBlock(block)) = bincode::deserialize::<GossipMsg>(&message.data) {
                            if let Some(req) = node.sync_request_for_gap(block.header.number) {
                                println!("⏳ gap detected (need {}..{}), requesting sync from {propagation_source}", req.from, req.to);
                                net::request_sync(&mut swarm, propagation_source, req);
                            }
                        }
                        let _ = node.on_gossip(&message.data);
                    }
                    SwarmEvent::Behaviour(QcBehaviourEvent::Sync(request_response::Event::Message {
                        message,
                        ..
                    })) => match message {
                        request_response::Message::Request { request, channel, .. } => {
                            let response = sync::build_sync_response(&app_state.storage, &request);
                            let _ = net::respond_sync(&mut swarm, channel, response);
                        }
                        request_response::Message::Response { response, .. } => {
                            match node.apply_sync_blocks(response.blocks) {
                                Ok(n) if n > 0 => println!("🔄 synced {n} block(s)"),
                                Ok(_) => {}
                                Err(e) => eprintln!("⚠️  sync apply failed: {e}"),
                            }
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("👂 listening on {address}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("🔗 connection established with {peer_id}");
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        println!("🔌 connection closed with {peer_id}: {cause:?}");
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        println!("⚠️  outgoing connection error to {peer_id:?}: {error}");
                    }
                    SwarmEvent::IncomingConnectionError { error, .. } => {
                        println!("⚠️  incoming connection error: {error}");
                    }
                    SwarmEvent::Behaviour(QcBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                        println!("🪪 identify: {peer_id} supports protocols: {:?}", info.protocols);
                    }
                    SwarmEvent::Behaviour(QcBehaviourEvent::Identify(identify::Event::Error { peer_id, error, .. })) => {
                        println!("⚠️  identify error with {peer_id}: {error}");
                    }
                    _ => {}
                }
            }
            _ = block_timer.tick() => {
                match node.try_produce_block() {
                    Ok(Some(_)) => {} // block produced; gossiped below via drain_outbox
                    Ok(None) => {} // not our turn, or mempool empty — not an error
                    Err(e) => eprintln!("⚠️  block production failed: {e}"),
                }
            }
        }

        for msg in node.drain_outbox() {
            let _ = net::publish(&mut swarm, &msg);
        }
    }
}

// Keystore behavior (permissions, password requirement, self-healing) is
// now exercised by qc_node::keystore's own test module — see
// src/keystore.rs — since this binary no longer has a private copy of
// that logic to test separately.
