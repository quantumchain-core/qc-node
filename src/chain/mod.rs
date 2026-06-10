use crate::crypto::{Keypair, verify};
use sha3::{Digest, Sha3_256};

pub struct Block {
    pub height: u64,
    pub prev_hash: [u8; 32],
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub pubkey: Vec<u8>, // M1 Dilithium pk
    pub signature: Vec<u8>, // M1 Dilithium sig
}

impl Block {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(&self.payload);
        hasher.finalize().into()
    }

    pub fn verify_sig(&self) -> bool {
        let hash = self.hash();
        verify(&hash, &self.signature, &self.pubkey)
    }
}

pub fn genesis_block(keypair: &Keypair) -> Block {
    let mut block = Block {
        height: 0,
        prev_hash: [0u8; 32],
        timestamp: 1717910400, // Jun 9 2024 UTC
        payload: b"QuantumChain Genesis".to_vec(),
        pubkey: keypair.public.to_vec(),
        signature: vec![],
    };
    let hash = block.hash();
    block.signature = keypair.sign(&hash);
    block
}

#[cfg(test)]
mod m3_tests {
    use super::*;
    use crate::crypto::Keypair;

    #[test]
    fn m3_genesis_validates() {
        let kp = Keypair::generate();
        let genesis = genesis_block(&kp);
        assert_eq!(genesis.height, 0);
        assert!(genesis.verify_sig());
    }

    #[test]
    fn m3_bad_sig_fails() {
        let kp = Keypair::generate();
        let mut genesis = genesis_block(&kp);
        genesis.signature[0] ^= 1; // corrupt
        assert!(!genesis.verify_sig());
    }
      }
