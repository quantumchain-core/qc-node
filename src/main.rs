mod crypto;
use crypto::Keypair;

fn main() {
    println!("Rampura QC-Node v0.1.0-m1");
    
    let keys = Keypair::generate();
    println!("Generated Dilithium2 keypair");
    println!("Public Key: {}...", &keys.public_key_hex()[..32]);
    println!("Pubkey Size: {} bytes", keys.public_key.len());
    println!("Seckey Size: {} bytes", keys.secret_key.len());
    println!("M1: Keygen complete");
}

#[test]
fn m1_integration() {
    let kp = Keypair::generate();
    assert!(kp.public_key.len() > 0);
}
