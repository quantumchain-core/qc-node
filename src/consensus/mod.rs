// src/consensus/mod.rs
// QTC - M5: Consensus Engine
// Per whitepaper: PoS, 2s block time, EIP-1559 base_fee adjustment

use std::time::{SystemTime, UNIX_EPOCH};

use crate::chain::{Block, BlockHeader};
use crate::mempool::Transaction;
use crate::mempool::Mempool;
// use crate::crypto; // M1 - Dilithium2 - REMOVED: unused in M5.1, will re-add in M7

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Target block time in seconds
pub const BLOCK_TIME_SECS: u64 = 2;

/// Block gas limit - must match mempool
pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;

/// Max transactions per block before gas limit
pub const MAX_TXS_PER_BLOCK: usize = 10_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// PoS validator identity - M1 Dilithium2 pubkey
pub type ValidatorId = [u8; 32];

/// Block proposer selected for a slot
#[derive(Debug, Clone)]
pub struct SlotProposer {
    pub validator: ValidatorId,
    pub slot: u64, // unix_time / BLOCK_TIME_SECS
}

/// Current chain state view for consensus
#[derive(Debug)]
pub struct ChainState {
    /// Latest block height
    pub height: u64,
    /// Latest block hash
    pub head_hash: [u8; 32],
    /// Current EIP-1559 base fee
    pub base_fee: u64,
    /// Genesis timestamp
    pub genesis_time: u64,
}

impl ChainState {
    pub fn current_slot(&self) -> u64 {
        let now = now_secs();
        now.saturating_sub(self.genesis_time) / BLOCK_TIME_SECS
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum ConsensusError {
    /// Not our slot to propose
    NotProposer,
    /// Parent block not found
    UnknownParent,
    /// Block timestamp too far in future
    FutureBlock,
    /// Invalid block signature from proposer
    InvalidBlockSig,
    /// No transactions available
    EmptyMempool,
    /// Block exceeds gas limit
    BlockGasExceeded,
}

// ---------------------------------------------------------------------------
// Consensus Engine
// ---------------------------------------------------------------------------

pub struct Consensus {
    /// Our validator identity - M1 keypair
    pub validator_id: ValidatorId,
    // validator_sk: Vec<u8>, // REMOVED: unused in M5.1, M6 uses different signing path
}

impl Consensus {
    pub fn new(validator_id: ValidatorId, _validator_sk: Vec<u8>) -> Self {
        Self {
            validator_id,
        }
    }

    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    /// Try to produce a block if we're the proposer for current slot.
    /// Called every BLOCK_TIME_SECS by the node.
    pub fn try_propose(
        &self,
        chain: &ChainState,
        mempool: &mut Mempool,
    ) -> Result<Block, ConsensusError> {
        let slot = chain.current_slot();

        // 1. Check if we're proposer - TODO: VRF/random beacon in M5.2
        // For M5.1: single validator mode, always true
        if!self.is_proposer(slot) {
            return Err(ConsensusError::NotProposer);
        }

        // 2. Select transactions from mempool ordered by fee
        let mut block_gas_used = 0u64;
        let mut txs: Vec<Transaction> = Vec::new();

        for tx in mempool.peek_best(MAX_TXS_PER_BLOCK) {
            if block_gas_used + tx.gas_limit > BLOCK_GAS_LIMIT {
                break; // Block full
            }
            block_gas_used += tx.gas_limit;
            txs.push(tx.clone());
        }

        if txs.is_empty() {
            return Err(ConsensusError::EmptyMempool);
        }

        // 3. Build header
        let timestamp = now_secs();
        let header = BlockHeader {
            parent_hash: chain.head_hash,
            height: chain.height + 1,
            slot,
            timestamp,
            proposer: self.validator_id,
            tx_root: merkle_root(&txs), // TODO: real merkle in M6
            base_fee: chain.base_fee,
            gas_used: block_gas_used,
            gas_limit: BLOCK_GAS_LIMIT,
        };

        // 4. Sign block - M1 Dilithium2
        let sig = self.sign_header(&header);

        // 5. Build block
        let block = Block {
            header,
            transactions: txs,
            signature: sig,
        };

        // 6. Remove included txs from mempool
        for tx in &block.transactions {
            mempool.remove(&tx.hash);
        }

        // 7. Update mempool base_fee for next block - EIP-1559
        let new_base_fee = calculate_next_base_fee(chain.base_fee, block_gas_used);
        mempool.update_base_fee(new_base_fee);

        Ok(block)
    }

    /// Process a block received from network. M5.1: just validate header + sig.
    /// Full state transition in M6.
    pub fn process_block(&self, block: &Block, chain: &ChainState) -> Result<(), ConsensusError> {
        // 1. Check parent
        if block.header.parent_hash!= chain.head_hash {
            return Err(ConsensusError::UnknownParent);
        }

        // 2. Check timestamp
        if block.header.timestamp > now_secs() + 1 {
            return Err(ConsensusError::FutureBlock);
        }

        // 3. Check proposer signature - M1 verify
        if!self.verify_block_sig(block) {
            return Err(ConsensusError::InvalidBlockSig);
        }

        // 4. Check gas
        let gas_used: u64 = block.transactions.iter().map(|t| t.gas_limit).sum();
        if gas_used!= block.header.gas_used || gas_used > BLOCK_GAS_LIMIT {
            return Err(ConsensusError::BlockGasExceeded);
        }

        // M6: execute txs, update state, check tx_root
        Ok(())
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// M5.1: Single validator. M5.2: VRF based selection
    fn is_proposer(&self, _slot: u64) -> bool {
        true // TODO: implement validator set + VRF
    }

    /// Sign block header with M1 Dilithium2
    fn sign_header(&self, header: &BlockHeader) -> Vec<u8> {
        let _msg = bincode::serialize(header).unwrap(); // FIXED: _msg unused in M5.1
        // TODO: call M1 crypto::sign(&self.validator_sk, &msg)
        vec![0u8; 2420] // placeholder - wire M1 in M5.2
    }

    /// Verify block signature - M1
    fn verify_block_sig(&self, block: &Block) -> bool {
        let _msg = bincode::serialize(&block.header).unwrap(); // FIXED: _msg unused in M5.1
        // TODO: call M1 crypto::verify(&block.header.proposer, &msg, &block.signature)
        true // placeholder - wire M1 in M5.2
    }
}

// ---------------------------------------------------------------------------
// EIP-1559 Base Fee Calculation
// ---------------------------------------------------------------------------

/// Adjust base_fee up/down based on block fullness. Target 50% full.
fn calculate_next_base_fee(parent_base_fee: u64, parent_gas_used: u64) -> u64 {
    let target_gas = BLOCK_GAS_LIMIT / 2;

    if parent_gas_used == target_gas {
        return parent_base_fee;
    }

    if parent_gas_used > target_gas {
        // Increase: max 12.5% per block
        let delta = parent_base_fee * (parent_gas_used - target_gas) / target_gas / 8;
        parent_base_fee + delta.max(1)
    } else {
        // Decrease: max 12.5% per block
        let delta = parent_base_fee * (target_gas - parent_gas_used) / target_gas / 8;
        parent_base_fee.saturating_sub(delta.max(1))
    }
}

// ---------------------------------------------------------------------------
// Utils
// ---------------------------------------------------------------------------

fn now_secs() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs()
}

fn merkle_root(txs: &[Transaction]) -> [u8; 32] {
    // TODO: real merkle tree in M6. For M5 just hash concat
    if txs.is_empty() {
        return [0u8; 32];
    }
    let mut data = Vec::new();
    for tx in txs {
        data.extend_from_slice(&tx.hash);
    }
    // TODO: use M1 crypto::hash
    [1u8; 32] // placeholder
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::{Mempool, MempoolConfig};

    // REMOVED: setup() was unused - clippy complained
    // fn setup() -> (Consensus, ChainState, Mempool) {... }

    #[test]
    fn test_base_fee_increase() {
        let fee = calculate_next_base_fee(1000, BLOCK_GAS_LIMIT); // 100% full
        assert!(fee > 1000); // should increase ~12.5%
        assert!(fee < 1150);
    }

    #[test]
    fn test_base_fee_decrease() {
        let fee = calculate_next_base_fee(1000, 0); // 0% full
        assert!(fee < 1000); // should decrease ~12.5%
        assert!(fee > 850);
    }

    #[test]
    fn test_base_fee_stable() {
        let fee = calculate_next_base_fee(1000, BLOCK_GAS_LIMIT / 2); // 50% full
        assert_eq!(fee, 1000);
    }
}
