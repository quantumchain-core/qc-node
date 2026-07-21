use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use qc_node::crypto::generate_keypair;
use qc_node::mempool::Transaction;
use qc_node::rpc::methods::compute_tx_hash;

fn main() {
    // Generate a fresh keypair for this tx
    let (pk, sk) = generate_keypair();
    let from = qc_node::consensus::address_from_pubkey(&pk);

    let mut tx = Transaction {
        hash: [0u8; 32],
        from,
        to: [1u8; 32],
        value: 1_000,
        nonce: 0,
        base_fee: 1_000,
        priority_fee: 50,
        gas_limit: 21_000,
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

    // Call curl to POST to local node
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

    let out = String::from_utf8_lossy(&resp.stdout);
    println!("node response: {}", out);
}
