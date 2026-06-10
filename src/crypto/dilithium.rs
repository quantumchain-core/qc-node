use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, DetachedSignature};
use sha3::{Digest, Sha3_256};

// M1 FUNCTION 1: KEYGEN
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

// M1 FUNCTION 2: SIGN
pub fn sign(msg: &[u8], sk: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(msg);
    let hash = hasher.finalize();
    
    let sk = SecretKey::from_bytes(sk).expect("bad sk");
    let sig = detached_sign(&hash, &sk);
    sig.as_bytes().to_vec()
}

// M1 FUNCTION 3: VERIFY
pub fn verify(msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    let mut hasher = Sha3_256::new();
    hasher.update(msg);
    let hash = hasher.finalize();
    
    let pk = PublicKey::from_bytes(pk).expect("bad pk");
    let sig = DetachedSignature::from_bytes(sig).expect("bad sig");
    verify_detached_signature(&sig, &hash, &pk).is_ok()
}

#[cfg(test)]
mod m1_tests {
    use super::*;
    
    #[test]
    fn m1_keygen_sign_verify() {
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.len(), 1952);
        assert_eq!(sk.len(), 4000);
        
        let msg = b"M1 test";
        let sig = sign(msg, &sk);
        assert_eq!(sig.len(), 3293);
        assert!(verify(msg, &sig, &pk));
        
        let mut bad = msg.to_vec();
        bad[0] ^= 1;
        assert!(!verify(&bad, &sig, &pk));
    }
      }
