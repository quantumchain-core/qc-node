// src/bin/node.rs - Corrected Keystore Encryption
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

use argon2::{Argon2, PasswordHasher, Params};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use rand::{RngCore, Rng};

#[derive(Serialize, Deserialize)]
struct Keystore {
    pk_hex: String,
    encrypted_sk: String,
    salt_hex: String,
    nonce_hex: String,
}

fn keystore_path() -> PathBuf {
    let path = std::env::var("QC_KEYSTORE_PATH")
        .unwrap_or_else(|_| "./qc-keystore.json".to_string());
    PathBuf::from(path)
}

fn load_or_generate_keypair() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let path = keystore_path();
    let password = std::env::var("QC_KEYSTORE_PASSWORD")
        .unwrap_or_else(|_| "CHANGE_THIS_IMMEDIATELY_IN_PRODUCTION".to_string());

    if path.exists() {
        let json = std::fs::read_to_string(&path)?;
        let ks: Keystore = serde_json::from_str(&json)?;

        let salt = hex::decode(&ks.salt_hex)?;
        let nonce_bytes = hex::decode(&ks.nonce_hex)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, Params::default());
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key = password_hash.hash.ok_or("key derivation failed")?.as_bytes();

        let cipher = Aes256Gcm::new_from_slice(key)?;
        let ciphertext = hex::decode(&ks.encrypted_sk)?;
        let sk = cipher.decrypt(nonce, ciphertext.as_ref())?;

        println!("✅ Loaded encrypted keystore from {}", path.display());
        Ok((hex::decode(&ks.pk_hex)?, sk))
    } else {
        let (pk, sk) = generate_keypair();
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key = password_hash.hash.ok_or("key derivation failed")?.as_bytes();

        let cipher = Aes256Gcm::new_from_slice(key)?;
        let encrypted = cipher.encrypt(&nonce, sk.as_ref())?;

        let ks = Keystore {
            pk_hex: hex::encode(&pk),
            encrypted_sk: hex::encode(encrypted),
            salt_hex: hex::encode(salt),
            nonce_hex: hex::encode(nonce.as_slice()),
        };

        std::fs::write(&path, serde_json::to_string_pretty(&ks)?)?;
        println!("✅ Created encrypted keystore at {}", path.display());
        Ok((pk, sk))
    }
}

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
    let rpc_addr = std::env::var("QC_RPC_ADDR").unwrap_or_else(|_| "0.0.0.0:8545".to_string());
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    tokio::spawn(async move { let _ = axum::serve(listener, rpc_app).await; });

    let mut swarm = net::new_swarm().await?;
    let mut block_timer = tokio::time::interval(Duration::from_secs(BLOCK_TIME_SECS));

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                if let SwarmEvent::Behaviour(QcBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) = event {
                    let _ = node.on_gossip(&message.data);
                }
            }
            _ = block_timer.tick() => {
                let _ = node.try_produce_block();
            }
        }

        for msg in node.drain_outbox() {
            let _ = net::publish(&mut swarm, &msg);
        }
    }
                }
