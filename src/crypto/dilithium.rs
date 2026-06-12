use pqcrypto_dilithium::dilithium5::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, DetachedSignature};

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

pub fn sign(sk: &[u8], msg: &[u8]) -> Vec<u8> {
    let sk = SecretKey::from_bytes(sk).expect("invalid sk");
    let sig = detached_sign(msg, &sk);
    sig.as_bytes().to_vec()
}

pub fn verify(msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    let pk = match PublicKey::from_bytes(pk) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let sig = match DetachedSignature::from_bytes(sig) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    verify_detached_signature(&sig, msg, &pk).is_ok()
}

#[cfg(test)]
mod m1_tests {
    use super::*;

    #[test]
    fn m1_keygen_sign_verify() {
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.len(), 2592);
        assert_eq!(sk.len(), 4896); // D5-AES
        let msg = b"test";
        let sig = sign(&sk, msg);
        assert_eq!(sig.len(), 4595);
        assert!(verify(msg, &sig, &pk));
    }

    #[test]
    fn m1_bad_sig_fails() {
        let (pk, sk) = generate_keypair();
        let msg = b"test";
        let mut sig = sign(&sk, msg);
        sig[0] ^= 1;
        assert!(!verify(msg, &sig, &pk));
    }
}
