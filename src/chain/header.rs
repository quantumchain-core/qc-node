use serde::{Deserialize, Serialize};
pub use crate::mempool::Transaction;
use crate::crypto::DILITHIUM2_SIG_BYTES; // ADD THIS

pub type Hash = [u8; 32];
pub type Address = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub height: u64,
    pub slot: u64,
    pub timestamp: u64,
    pub proposer: Address,
    pub tx_root: Hash,
    pub base_fee: u128,         // CHANGE: u64 -> u128 for EIP-1559
    pub gas_used: u64,
    pub gas_limit: u64,
    pub state_root: Hash,       // ADD THIS - M6 needs it
    pub signature: [u8; DILITHIUM2_SIG_BYTES], // ADD THIS - fixed size
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    // DELETE the signature field from here
}

impl BlockHeader {
    pub fn to_bytes_without_sig(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend(&self.parent_hash);
        v.extend(&self.height.to_le_bytes());
        v.extend(&self.slot.to_le_bytes());
        v.extend(&self.timestamp.to_le_bytes());
        v.extend(&self.proposer);
        v.extend(&self.tx_root);
        v.extend(&self.base_fee.to_le_bytes());
        v.extend(&self.gas_used.to_le_bytes());
        v.extend(&self.gas_limit.to_le_bytes());
        v.extend(&self.state_root);
        v // signature NOT included
    }
}

impl Block {
    pub fn hash(&self) -> Hash {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.header.to_bytes_without_sig());
        hasher.finalize().into()
    }
}
