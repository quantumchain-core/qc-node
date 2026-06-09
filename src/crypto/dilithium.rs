//! Dilithium3 wrapper for qc-node
//! Network: qtc-rampura-testnet-0

use pqcrypto_dilithium::dilithium3::{
    PublicKey, SecretKey, SignedMessage,
    detached_sign, verify_detached_signature,
    keypair, DetachedSignature,
};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _};
use sha3::{Digest, Sha3_256};

pub const RAMPURA_TESTNET_0_CHAIN_ID: &str = "qtc-rampura-testnet-0";

/// Dilithium3 public key - 1952 bytes
#[derive(Clone, Debug, PartialEq)]
pub struct DilithiumPubKey(pub [u8; 1952]);

/// Dilithium3 secret key - 4000 bytes 
#[derive(Clone)]
pub struct DilithiumSecretKey(pub [u8; 4000]);

/// Dilithium3 signature - 3309 bytes
#[derive(Clone, Debug, PartialEq)]
pub struct DilithiumSig(pub [u8; 3309]);

/// Generate new Dilithium3 keypair
pub fn generate_keypair() -> (DilithiumPubKey, DilithiumSecretKey) {
    let (pk, sk) = keypair();
    (
        DilithiumPubKey(pk.as_bytes().try_into().unwrap()),
        DilithiumSecretKey(sk.as_bytes().try_into().unwrap()),
    )
}

/// Sign a message with Dilithium3
pub fn sign(msg: &[u8], sk: &DilithiumSecretKey) -> DilithiumSig {
    let secret = SecretKey::from_bytes(&sk.0).unwrap();
    let sig = detached_sign(msg, &secret);
    DilithiumSig(sig.as_bytes().try_into().unwrap())
}

/// Verify Dilithium3 signature
pub fn verify(msg: &[u8], sig: &DilithiumSig, pk: &DilithiumPubKey) -> bool {
    let public = PublicKey::from_bytes(&pk.0);
    let signature = DetachedSignature::from_bytes(&sig.0);
    verify_detached_signature(&signature, msg, &public).is_ok()
}

/// Hash data with SHA3-256
pub fn hash_sha3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let (pk, sk) = generate_keypair();
        let msg = b"qtc-rampura-testnet-0-genesis";
        let sig = sign(msg, &sk);
        assert!(verify(msg, &sig, &pk));
        assert!(!verify(b"wrong", &sig, &pk));
    }

    #[test]
    fn test_key_sizes() {
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.0.len(), 1952);
        assert_eq!(sk.0.len(), 4000);
    }
} 