use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage, DetachedSignature};
use sha3::{Digest, Sha3_256};

// M1 FUNCTION 1: KEYGEN - Makes quantum-safe keypair
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

// M1 FUNCTION 2: SIGN - Signs any message
pub fn sign(msg: &[u8], sk: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(msg);
    let hash = hasher.finalize();
    
    let sk = SecretKey::from_bytes(sk).expect("M1: bad secret key");
    let signed_msg = detached_sign(&hash, &sk);
    signed_msg.as_bytes().to_vec()
}

// M1 FUNCTION 3: VERIFY - Checks signature
pub fn verify(msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    let mut hasher = Sha3_256::new();
    hasher.update(msg);
    let hash = hasher.finalize();
    
    let pk = PublicKey::from_bytes(pk).expect("M1: bad public key");
    let sig = DetachedSignature::from_bytes(sig).expect("M1: bad signature");
    verify_detached_signature(&sig, &hash, &pk).is_ok()
}

// M1 TEST: This must pass or M1 is broken
#[cfg(test)]
mod m1_tests {
    use super::*;
    #
    fn m1_keygen_sign_verify() {
        // Test 1: Key sizes
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.len(), 1952, "M1 FAIL: pubkey wrong size");
        assert_eq!(sk.len(), 4000, "M1 FAIL: secretkey wrong size");
        
        // Test 2: Sign + Verify works
        let msg = b"QuantumChain M1 test";
        let sig = sign(msg, &sk);
        assert_eq!(sig.len(), 3293, "M1 FAIL: signature wrong size");
        assert!(verify(msg, &sig, &pk), "M1 FAIL: verify failed");
        
        // Test 3: Tamper check
        let mut bad_msg = msg.to_vec();
        bad_msg[0] ^= 1;
        assert!(!verify(&bad_msg, &sig, &pk), "M1 FAIL: accepted fake msg");
    }
}
