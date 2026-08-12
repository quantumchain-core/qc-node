// src/bin/fund_and_send.rs - dev/test helper
//
// Funds a fresh account by writing DIRECTLY to the on-disk database, then
// tries to submit one transaction from it over RPC.
//
// IMPORTANT CONSTRAINT (not a bug, just how the node works): the node only
// reads its state from disk ONCE, at startup — after that it runs from an
// in-memory copy and never looks back at storage except to persist
// (write), not to re-read. That means the funding side effect here only
// actually helps if this runs BEFORE the target node process starts. If
// the node is already running when you invoke this, the RPC transaction
// submission below may or may not succeed depending on timing, but the
// funding write will NOT be visible to that already-running process.
//
// Two additive changes from the original version, both backward
// compatible (existing manual usage without these env vars is unchanged):
//   - QC_RPC_URL overrides the previously-hardcoded "http://localhost:8545"
//   - QC_PRINT_SECRET_KEY=1 also prints the secret key hex, so a separate
//     later step (e.g. send_tx with QC_TX_FROM_SK_HEX) can reuse this same
//     funded identity instead of always generating a throwaway one.
//     Dev/test tooling only — never do this with a real production key.

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
        action: qc_node::mempool::TxAction::Transfer,
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

    let rpc_url = std::env::var("QC_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());

    let resp = Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(&rpc_url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload)
        .output()
        .expect("failed to execute curl");

    println!("funded from: 0x{}", hex::encode(from));
    if std::env::var("QC_PRINT_SECRET_KEY").ok().as_deref() == Some("1") {
        println!("secret key (dev/test only): 0x{}", hex::encode(&sk));
        println!("pubkey: 0x{}", hex::encode(&pk));
    }
    println!("node response: {}", String::from_utf8_lossy(&resp.stdout));
}
