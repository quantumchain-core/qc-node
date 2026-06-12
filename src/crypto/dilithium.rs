// src/crypto/dilithium.rs
// QTC M1 — Post-Quantum Signatures
// Temporary: m1_print_sizes will reveal exact byte sizes for this platform

use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey, SecretKey};

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

pub fn sign(sk_bytes: &[u8], msg: &[u8]) -> Vec<u8> {
    let sk = SecretKey::from_bytes(sk_bytes).expect("invalid secret key");
    detached_sign(msg, &sk).as_bytes().to_vec()
}

pub fn verify(msg: &[u8], sig_bytes: &[u8], pk_bytes: &[u8]) -> bool {
    let pk = match PublicKey::from_bytes(pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let sig = match DetachedSignature::from_bytes(sig_bytes) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    verify_detached_signature(&sig, msg, &pk).is_ok()
}

#[cfg(test)]
mod m1_tests {
    use super::*;

    #[test]
    fn m1_print_sizes() {
        let (pk, sk) = generate_keypair();
        let msg = b"test";
        let sig = sign(&sk, msg);
        panic!("pk={} sk={} sig={}", pk.len(), sk.len(), sig.len());
    }

    #[test]
    fn m1_keygen_sign_verify() {
        let (pk, sk) = generate_keypair();
        let msg = b"qtc test message";
        let sig = sign(&sk, msg);
        assert!(verify(msg, &sig, &pk), "valid sig should verify");
    }

    #[test]
    fn m1_bad_sig_fails() {
        let (pk, sk) = generate_keypair();
        let msg = b"qtc test message";
        let mut sig = sign(&sk, msg);
        sig[0] ^= 0xFF;
        assert!(!verify(msg, &sig, &pk), "corrupted sig should fail");
    }

    #[test]
    fn m1_wrong_key_fails() {
        let (pk1, _) = generate_keypair();
        let (_, sk2) = generate_keypair();
        let msg = b"qtc test message";
        let sig = sign(&sk2, msg);
        assert!(!verify(msg, &sig, &pk1), "sig from wrong key should fail");
    }
}
