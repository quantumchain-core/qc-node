// src/bin/send_tx.rs - dev/test helper
//
// Submits one signed transaction to a running node over RPC.
//
// Three additive changes from the original version, all backward
// compatible (omit any of these env vars and behavior is unchanged —
// still generates a fresh random keypair, still targets localhost:8545):
//   - QC_TX_FROM_SK_HEX / QC_TX_FROM_PK_HEX: reuse an existing keypair
//     instead of generating a throwaway one — needed to actually spend
//     from an account that was funded by a separate earlier step (e.g.
//     fund_and_send with QC_PRINT_SECRET_KEY=1), since a brand-new random
//     account always has zero balance.
//   - QC_TX_NONCE: override the transaction nonce (default 0 — correct
//     for the first transaction ever sent from a given account, but a
//     second transaction from the same account needs nonce 1, etc.)
//   - QC_RPC_URL: override the previously-hardcoded "http://localhost:8545"

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use qc_node::crypto::generate_keypair;
use qc_node::mempool::Transaction;
use qc_node::rpc::methods::compute_tx_hash;

fn main() {
    let (pk, sk) = match (std::env::var("QC_TX_FROM_SK_HEX"), std::env::var("QC_TX_FROM_PK_HEX")) {
        (Ok(sk_hex), Ok(pk_hex)) => {
            let sk = hex::decode(sk_hex.trim_start_matches("0x")).expect("invalid QC_TX_FROM_SK_HEX");
            let pk = hex::decode(pk_hex.trim_start_matches("0x")).expect("invalid QC_TX_FROM_PK_HEX");
            (pk, sk)
        }
        _ => generate_keypair(),
    };
    let from = qc_node::consensus::address_from_pubkey(&pk);

    let nonce: u64 = std::env::var("QC_TX_NONCE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut tx = Transaction {
        hash: [0u8; 32],
        from,
        to: [1u8; 32],
        value: 1_000,
        nonce,
        base_fee: 1_000,
        priority_fee: 50,
        gas_limit: 21_000,
        action: qc_node::mempool::TxAction::Transfer,
        signature: Vec::new(),
        received_at: 0,
        from_pubkey: pk.clone(),
    };

    // compute correct hash
    let hash = compute_tx_hash(&tx);
    tx.hash = hash;

    // sign (secret key first, message second)
    tx.signature = qc_node::crypto::sign(&sk, &tx.signable_bytes());

    // set received_at
    tx.received_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // serialize
    let bytes = bincode::serialize(&tx).expect("serialize tx");
    let hex = hex::encode(&bytes);

    // Build JSON-RPC payload
    let payload = format!("{{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"0x{}\"],\"id\":1}}", hex);

    let rpc_url = std::env::var("QC_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());

    // Call curl to POST to the node
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

    let out = String::from_utf8_lossy(&resp.stdout);
    println!("sent from: 0x{}", hex::encode(from));
    println!("node response: {}", out);
}
