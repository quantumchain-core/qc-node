// src/chain/mod.rs
// QTC - M3: Chain Types
// Updated for M5/M6: EIP-1559, Dilithium5, separate header/body

use serde::{Deserialize, Serialize};

pub use crate::mempool::Transaction;

/// 32-byte hash
pub type Hash = [u8; 32];
pub type Address = [u8; 32]; // Keep for account addresses
pub type Pubkey = Vec<u8>;   // NEW: Dilithium5 pubkey is 2592 bytes

/// Block header - consensus fields only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    pub parent_hash: Hash,      // was prev_hash
    pub height: u64,
    pub slot: u64,              // M5: PoS slot
    pub timestamp: u64,
    pub proposer: Pubkey,        // FIXED: was Address, now Vec<u8> for D5
    pub tx_root: Hash,           // merkle root of transactions
    pub base_fee: u64,           // M4: EIP-1559
    pub gas_used: u64,           // M4: total gas in block
    pub gas_limit: u64,          // M4: BLOCK_GAS_LIMIT
}

/// Full block with body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>, // was payload
    pub signature: Vec<u8>,             // Dilithium5 over header - 4595 bytes
}

impl Block {
    pub fn hash(&self) -> Hash {
        // TODO: M1 crypto::hash(bincode::serialize(&self.header))
        [0u8; 32] // placeholder
    }
}
