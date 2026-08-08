// src/bin/node.rs - Simplified Working Encrypted Keystore
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;

use futures::StreamExt;
use libp2p::{gossipsub, request_response, swarm::SwarmEvent};
use serde::{Deserialize, Serialize};

use qc_node::chain::Address;
use qc_node::consensus::{address_from_pubkey, Producer, ValidatorRegistry, BLOCK_TIME_SECS};
use qc_node::crypto::generate_keypair;
use qc_node::mempool::Mempool;
use qc_node::net::{self, GossipMsg, QcBehaviourEvent};
use qc_node::node::Node;
use qc_node::rpc::{self, AppState, ChainHead};
use qc_node::state::Storage;
use qc_node::sync;

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
/// not raw salt bytes — that mismatch was the original compile error here).
fn derive_key(argon2: &Argon2, password: &str, salt: &[u8]) -> Result<[u8; AES_KEY_LEN], Box<dyn std::error::Error>> {
    let mut key = [0u8; AES_KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("key derivation failed: {e}"))?;
    Ok(key)
}

/// Read `QC_KEYSTORE_PASSWORD` from the environment. Unlike the previous
/// version, this does NOT fall back to a hardcoded default — a default
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
        restrict_keystore_permissions(&path)?;
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

        println!("✅ Loaded encrypted keystore from {}", path.display());
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
        restrict_keystore_permissions(&path)?;
        println!("✅ Created encrypted keystore at {}", path.display());
        Ok((pk, sk))
    }
}

/// Restrict the keystore file to owner read/write only (0600). Without
/// this, the file inherits the process umask — commonly 0644, meaning
/// any other local user on the box can read the encrypted blob and mount
/// an offline password-guessing attack against it, with no need to
/// exploit anything else first. Unix-only; Windows ACLs would need a
/// different mechanism (not implemented here since RUN_VALIDATOR.md only
/// documents Ubuntu deployment).
#[cfg(unix)]
fn restrict_keystore_permissions(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_keystore_permissions(_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
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
                    _ => {}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_restrict_keystore_permissions_sets_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        // Start deliberately permissive, to prove this actually tightens
        // permissions rather than just happening to already be 0600.
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).unwrap();

        restrict_keystore_permissions(tmp.path()).unwrap();

        let mode = std::fs::metadata(tmp.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    #[cfg(unix)]
    fn test_load_or_generate_keypair_creates_keystore_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = tmp_dir.path().join("qc-keystore.json");

        std::env::set_var("QC_KEYSTORE_PATH", &keystore_path);
        std::env::set_var("QC_KEYSTORE_PASSWORD", "test-password-for-unit-test-only");

        let (pk, sk) = load_or_generate_keypair().unwrap();
        assert!(!pk.is_empty());
        assert!(!sk.is_empty());

        let mode = std::fs::metadata(&keystore_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        std::env::remove_var("QC_KEYSTORE_PATH");
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
    }

    #[test]
    #[cfg(unix)]
    fn test_load_or_generate_keypair_self_heals_loose_permissions_on_reload() {
        // Simulates a keystore that predates this fix, or got copied in
        // some other way that lost its restrictive permissions — loading
        // it should tighten permissions, not just leave them as-is.
        use std::os::unix::fs::PermissionsExt;
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = tmp_dir.path().join("qc-keystore.json");

        std::env::set_var("QC_KEYSTORE_PATH", &keystore_path);
        std::env::set_var("QC_KEYSTORE_PASSWORD", "test-password-for-unit-test-only");

        // First call creates it (already 0600 per the test above).
        load_or_generate_keypair().unwrap();
        // Deliberately loosen it, as if copied from elsewhere.
        std::fs::set_permissions(&keystore_path, std::fs::Permissions::from_mode(0o644)).unwrap();

        // Second call takes the "load existing" branch.
        load_or_generate_keypair().unwrap();

        let mode = std::fs::metadata(&keystore_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        std::env::remove_var("QC_KEYSTORE_PATH");
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
    }

    #[test]
    fn test_require_keystore_password_errors_when_unset() {
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
        assert!(require_keystore_password().is_err());
    }
}
