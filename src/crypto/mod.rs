use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn from_secret_bytes(bytes: &[u8]) -> Option<Self> {
        let sk = SecretKey::from_bytes(bytes).ok()?;
        let pk = sk.public_key();
        Some(Keypair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: bytes.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen() {
        let kp = Keypair::generate();
        assert_eq!(kp.public_key.len(), 1312); // Dilithium2 pubkey size
        assert_eq!(kp.secret_key.len(), 2528); // Dilithium2 seckey size
        println!("M1 PASS: Pubkey: {}...", &kp.public_key_hex()[..16]);
    }
}
