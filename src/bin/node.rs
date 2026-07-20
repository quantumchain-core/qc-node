// src/bin/node.rs - Simplified Working Encrypted Keystore
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

use argon2::Argon2;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use rand::RngCore;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
struct Keystore {
    pk_hex: String,
    encrypted_sk: String,
    salt_hex: String,
    nonce_hex: String,
}

const AES_KEY_LEN: usize = 32; // AES-256

fn keystore_path() -> PathBuf {
    let path = std::env::var("QC_KEYSTORE_PATH")
        .unwrap_or_else(|_| "./qc-keystore.json".to_string());
    PathBuf::from(path)
}

/// Derive a 32-byte AES key from `password` + `salt` via Argon2id.
/// Uses the low-level raw-bytes API (`hash_password_into`), NOT the
/// high-level `hash_password` (which expects a PHC-formatted `SaltString`,
/// not raw salt bytes â€” that mismatch was the original compile error here).
fn derive_key(argon2: &Argon2, password: &str, salt: &[u8]) -> Result<[u8; AES_KEY_LEN], Box<dyn std::error::Error>> {
    let mut key = [0u8; AES_KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("key derivation failed: {e}"))?;
    Ok(key)
}

/// Read `QC_KEYSTORE_PASSWORD` from the environment. Unlike the previous
/// version, this does NOT fall back to a hardcoded default â€” a default
/// baked into public source code isn't a secret, so a silent fallback here
/// would mean "encrypted" keystores are only as safe as a string anyone can
/// read on GitHub. Refusing to start is safer than starting insecurely.
fn require_keystore_password() -> Result<String, Box<dyn std::error::Error>> {
    std::env::var("QC_KEYSTORE_PASSWORD").map_err(|_| {
        "QC_KEYSTORE_PASSWORD is not set. Refusing to start: there is no safe default \
         for the keystore encryption password. Set QC_KEYSTORE_PASSWORD before launching the node."
            .into()
    })
}

fn load_or_generate_keypair() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let path = keystore_path();
    let password = require_keystore_password()?;
    let argon2 = Argon2::default();

    if path.exists() {
        let json = std::fs::read_to_string(&path)?;
        let ks: Keystore = serde_json::from_str(&json)?;

        let salt = hex::decode(&ks.salt_hex)?;
        let nonce_bytes = hex::decode(&ks.nonce_hex)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut key_bytes = derive_key(&argon2, &password, &salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        key_bytes.zeroize();

        let ciphertext = hex::decode(&ks.encrypted_sk)?;
        let sk = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "incorrect QC_KEYSTORE_PASSWORD or corrupted keystore file")?;

        println!("âœ… Loaded encrypted keystore from {}", path.display());
        Ok((hex::decode(&ks.pk_hex)?, sk))
    } else {
        let (pk, sk) = generate_keypair();

        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let mut key_bytes = derive_key(&argon2, &password, &salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        key_bytes.zeroize();

        let encrypted = cipher
            .encrypt(&nonce, sk.as_ref())
            .map_err(|_| "keystore encryption failed")?;

        let ks = Keystore {
            pk_hex: hex::encode(&pk),
            encrypted_sk: hex::encode(encrypted),
            salt_hex: hex::encode(salt),
            nonce_hex: hex::encode(nonce.as_slice()),
        };

        std::fs::write(&path, serde_json::to_string_pretty(&ks)?)?;
        println!("âœ… Created encrypted keystore at {}", path.display());
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
