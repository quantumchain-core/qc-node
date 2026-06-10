use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, DetachedSignature, sign, verify};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Keypair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl Keypair {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Keypair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }
    pub fn public_key_hex(&self) -> String {
        hex::encode(&self.public_key)
    }
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let sk = SecretKey::from_bytes(&self.secret_key).unwrap();
        sign(msg, &sk).as_bytes().to_vec()
    }
    pub fn verify(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> bool {
        let pk = match PublicKey::from_bytes(pubkey) {
            Ok(pk) => pk,
            Err(_) => return false,
        };
        let signature = match DetachedSignature::from_bytes(sig) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verify(&signature, msg, &pk).is_ok()
    }
}
