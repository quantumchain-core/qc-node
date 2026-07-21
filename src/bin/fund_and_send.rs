use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use qc_node::state::{Account, Storage};

fn main() {
    // Ensure we use the same DB path as the running node (default ./qc-data)
    // Generate a fresh keypair and fund it
    let (pk, sk) = qc_node::crypto::generate_keypair();
    let from = qc_node::consensus::address_from_pubkey(&pk);

    // Open storage and set a funded account
    let storage = Storage::new().expect("open storage");
    let mut state = storage.get_state().unwrap().unwrap_or_default();
    state.set_account(from, Account { balance: 100_000_000u128, nonce: 0, ..Default::default() });
    storage.put_state(&state).expect("write state");

    // Build and sign a transaction from this funded account
    let mut tx = qc_node::mempool::Transaction {
        hash: [0u8; 32],
        from,
        to: [2u8; 32],
        value: 10,
        nonce: 0,
        base_fee: 1_000,
        priority_fee: 50,
        gas_limit: 21_000,
        signature: Vec::new(),
        received_at: 0,
        from_pubkey: pk.clone(),
    };

    // compute hash and sign
    tx.hash = qc_node::rpc::methods::compute_tx_hash(&tx);
    tx.signature = qc_node::crypto::sign(&sk, &tx.signable_bytes());
    tx.received_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let bytes = bincode::serialize(&tx).expect("serialize tx");
    let hex = hex::encode(&bytes);
    let payload = format!("{{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"0x{}\"],\"id\":1}}", hex);

    let resp = Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg("http://localhost:8545")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload)
        .output()
        .expect("failed to execute curl");

    println!("funded from: 0x{}", hex::encode(from));
    println!("node response: {}", String::from_utf8_lossy(&resp.stdout));
}
