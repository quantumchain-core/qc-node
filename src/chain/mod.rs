// src/chain/mod.rs
// QTC - M3: Chain Types (unified, replaces header.rs)
// Dilithium2: sig=2420 bytes, pk=1312 bytes

use serde::{Deserialize, Serialize};
pub use crate::mempool::Transaction;

/// 32-byte hash (SHA3-256 / SHA2-256)
pub type Hash = [u8; 32];

/// Account address — 32-byte Dilithium2 pubkey hash
pub type Address = [u8; 32];

/// Block header — all consensus-critical fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub number: u64,
    pub slot: u64,
    pub timestamp: u64,
    pub proposer: Address,      // 32-byte hash of validator pubkey
    pub tx_root: Hash,
    pub state_root: Hash,       // M6: updated after execution
    pub base_fee: u64,          // EIP-1559
    pub gas_used: u64,
    pub gas_limit: u64,
    pub signature: Vec<u8>,     // Dilithium2 sig over header bytes (2420 bytes)
}

impl BlockHeader {
    /// Serialize header fields WITHOUT signature for signing/verification
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.parent_hash);
        v.extend_from_slice(&self.number.to_le_bytes());
        v.extend_from_slice(&self.slot.to_le_bytes());
        v.extend_from_slice(&self.timestamp.to_le_bytes());
        v.extend_from_slice(&self.proposer);
        v.extend_from_slice(&self.tx_root);
        v.extend_from_slice(&self.state_root);
        v.extend_from_slice(&self.base_fee.to_le_bytes());
        v.extend_from_slice(&self.gas_used.to_le_bytes());
        v.extend_from_slice(&self.gas_limit.to_le_bytes());
        v // signature NOT included
    }
}

/// Full block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn hash(&self) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.header.to_signable_bytes());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signable_bytes_excludes_sig() {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 1000,
            proposer: [1u8; 32],
            tx_root: [2u8; 32],
            state_root: [3u8; 32],
            base_fee: 1000,
            gas_used: 21000,
            gas_limit: 30_000_000,
            signature: vec![0u8; 2420],
        };
        let bytes = header.to_signable_bytes();
        // sig (2420 bytes) must NOT be in signable bytes
        assert!(!bytes.windows(2420).any(|w| w == vec![0u8; 2420].as_slice()));
    }

    #[test]
    fn test_block_hash_deterministic() {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 1000,
            proposer: [1u8; 32],
            tx_root: [2u8; 32],
            state_root: [3u8; 32],
            base_fee: 1000,
            gas_used: 0,
            gas_limit: 30_000_000,
            signature: vec![0u8; 2420],
        };
        let block = Block { header, transactions: vec![] };
        assert_eq!(block.hash(), block.hash());
    }
}
