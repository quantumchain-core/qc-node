// src/bin/node.rs
// QTC M9/M10 + Security fixes (AUDIT-022, AUDIT-023):
//
// AUDIT-022 FIX: Keypair is now persisted to QC_KEYSTORE_PATH on first run
// and loaded on subsequent runs. A node restart no longer generates a new
// address, so the genesis registry remains valid across restarts.
//
// AUDIT-023 FIX: Coinbase is now derived from the validator's own address
// (SHA3-256 of pubkey) by default, or overridden via QC_COINBASE env var.
// Gas fees now go to a real address the operator controls.
//
// Environment variables:
//   QC_DB_PATH       — sled storage directory (default: ./qc-data)
//   QC_RPC_ADDR      — JSON-RPC HTTP bind address (default: 0.0.0.0:8545)
//   QC_GENESIS_PATH  — multi-validator genesis JSON (default: single-validator)
//   QC_KEYSTORE_PATH — keypair file (default: ./qc-keystore.json)
//   QC_COINBASE      — fee recipient address 0x<64 hex> (default: own address)

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;

use futures::StreamExt;
use libp2p::{gossipsub, swarm::SwarmEvent};
use serde::{Deserialize, Serialize};

use qc_node::chain::Address;
use qc_node::consensus::{address_from_pubkey, Producer, ValidatorRegistry, BLOCK_TIME_SECS};
use qc_node::crypto::generate_keypair;
use qc_node::mempool::Mempool;
use qc_node::net::{self, QcBehaviourEvent};
use qc_node::node::Node;
use qc_node::rpc::{self, AppState, ChainHead};
use qc_node::state::Storage;

// ---------------------------------------------------------------------------
// Keystore — persists Dilithium2 keypair across restarts (AUDIT-022)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct Keystore {
    /// Hex-encoded Dilithium2 public key (1312 bytes)
    pk_hex: String,
    /// Hex-encoded Dilithium2 secret key (2560 bytes)
    /// TODO M15: encrypt with Argon2 + AES-256-GCM
    sk_hex: String,
}

fn keystore_path() -> PathBuf {
    let path = std::env::var("QC_KEYSTORE_PATH")
        .unwrap_or_else(|_| "./qc-keystore.json".to_string());
    PathBuf::from(path)
}

/// Load keypair from keystore file, or generate and save a new one.
/// This is the fix for AUDIT-022: the same address is used across restarts.
fn load_or_generate_keypair() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let path = keystore_path();

    if path.exists() {
        // Load existing keypair
        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read keystore {}: {e}", path.display()))?;
        let keystore: Keystore = serde_json::from_str(&json)
            .map_err(|e| format!("keystore corrupt: {e}"))?;
        let pk = hex::decode(&keystore.pk_hex)
            .map_err(|e| format!("invalid pk in keystore: {e}"))?;
        let sk = hex::decode(&keystore.sk_hex)
            .map_err(|e| format!("invalid sk in keystore: {e}"))?;

        if pk.len() != 1312 {
            return Err(format!("keystore pk is {} bytes, expected 1312", pk.len()).into());
        }
        if sk.len() != 2560 {
            return Err(format!("keystore sk is {} bytes, expected 2560", sk.len()).into());
        }

        println!("loaded keypair from {}", path.display());
        println!("validator address: 0x{}", hex::encode(address_from_pubkey(&pk)));
        Ok((pk, sk))
    } else {
        // Generate new keypair and save
        let (pk, sk) = generate_keypair();
        let keystore = Keystore {
            pk_hex: hex::encode(&pk),
            sk_hex: hex::encode(&sk),
        };
        let json = serde_json::to_string_pretty(&keystore)?;
        std::fs::write(&path, json)
            .map_err(|e| format!("failed to write keystore {}: {e}", path.display()))?;

        println!("generated new keypair, saved to {}", path.display());
        println!("validator address: 0x{}", hex::encode(address_from_pubkey(&pk)));
        println!("IMPORTANT: back up {} — losing it means losing your validator identity", path.display());
        Ok((pk, sk))
    }
}

/// Parse coinbase from QC_COINBASE env var, or derive from validator pubkey.
/// Fix for AUDIT-023: coinbase is no longer hardcoded to [9u8;32].
fn load_coinbase(pk: &[u8]) -> Result<Address, Box<dyn std::error::Error>> {
    match std::env::var("QC_COINBASE") {
        Ok(hex_str) => {
            let clean = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
            let bytes = hex::decode(clean)
                .map_err(|e| format!("invalid QC_COINBASE hex: {e}"))?;
            if bytes.len() != 32 {
                return Err(format!(
                    "QC_COINBASE must be 32 bytes (64 hex chars), got {}",
                    bytes.len()
                ).into());
            }
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&bytes);
            println!("coinbase: 0x{} (from QC_COINBASE)", hex::encode(addr));
            Ok(addr)
        }
        Err(_) => {
            // Default: use own validator address as fee recipient (AUDIT-023 fix)
            let addr = address_from_pubkey(pk);
            println!("coinbase: 0x{} (derived from validator pubkey)", hex::encode(addr));
            Ok(addr)
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

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

    // --- Validator identity (AUDIT-022 + AUDIT-023 fixed) ---
    let (pk, sk) = load_or_generate_keypair()?;
    let coinbase = load_coinbase(&pk)?;
    let producer = Producer::new(sk, pk.clone(), coinbase);

    // --- Validator registry (M10) ---
    let registry = match std::env::var("QC_GENESIS_PATH") {
        Ok(path) => {
            println!("loading validator registry from {path}");
            ValidatorRegistry::load_from_file(&path)?
        }
        Err(_) => {
            println!("QC_GENESIS_PATH not set — single-validator mode");
            ValidatorRegistry::single(&pk)
        }
    };
    println!("validator registry: {} validator(s)", registry.len());

    let mut node = Node::new(app_state.clone(), producer, registry);

    // --- RPC server (M8) ---
    let rpc_app = rpc::router(app_state.clone());
    let rpc_addr = std::env::var("QC_RPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8545".to_string());
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
                    println!("gossip: {result:?}");
                }
            }
            _ = block_timer.tick() => {
                match node.try_produce_block() {
                    Ok(Some(block)) => println!("produced block #{}", block.header.number),
                    Ok(None) => {}
                    Err(e) => eprintln!("block production failed: {e}"),
                }
            }
        }

        for msg in node.drain_outbox() {
            if let Err(e) = net::publish(&mut swarm, &msg) {
                eprintln!("publish failed: {e}");
            }
        }
    }
}
