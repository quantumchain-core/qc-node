use crate::crypto;
use sha3::{Digest, Sha3_256};

pub struct Block {
    pub height: u64,
    pub prev_hash: [u8; 32],
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub signature: Vec<u8>,
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
        crypto::verify(&hash, &self.signature, &self.pubkey)
    }
}

pub fn genesis_block() -> Block {
    let (pk, sk) = crypto::generate_keypair();
    let mut block = Block {
        height: 0,
        prev_hash: [0u8; 32],
        timestamp: 1717910400,
        payload: b"QuantumChain Genesis".to_vec(),
        pubkey: pk,
        signature: vec![],
    };
    let hash = block.hash();
    block.signature = crypto::sign(&sk, &hash);
    block
}

#[cfg(test)]
mod m3_tests {
    use super::*;

    #[test]
    fn m3_genesis_validates() {
        let genesis = genesis_block();
        assert_eq!(genesis.height, 0);
        assert!(genesis.verify_sig());
    }

    #[test]
    fn m3_bad_sig_fails() {
        let mut genesis = genesis_block();
        genesis.signature[0] ^= 1;
        assert!(!genesis.verify_sig());
    }
    }
