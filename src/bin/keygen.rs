// src/bin/keygen.rs
// QTC: prints a validator's address + pubkey without starting the full
// node (no RPC server, no networking, no block-production loop).
//
// Exists specifically for integration testing: a multi-validator genesis
// file needs to list real validator pubkeys BEFORE any node process
// starts. This uses the exact same `load_or_generate_keypair()` from
// `qc_node::keystore` that the real node uses — so whatever this prints
// is guaranteed to match what the real node loads later from the same
// QC_KEYSTORE_PATH, with zero risk of the two disagreeing.
//
// Usage:
//   QC_KEYSTORE_PATH=./validator-a.json QC_KEYSTORE_PASSWORD=... \
//     cargo run --bin keygen
//
// Output (stdout, machine-parseable, one KEY=VALUE per line):
//   ADDRESS=0x...
//   PUBKEY=0x...

use qc_node::consensus::address_from_pubkey;
use qc_node::keystore::load_or_generate_keypair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pk, _sk) = load_or_generate_keypair()?;
    let address = address_from_pubkey(&pk);
    println!("ADDRESS=0x{}", hex::encode(address));
    println!("PUBKEY=0x{}", hex::encode(&pk));
    Ok(())
}
