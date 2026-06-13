// src/consensus/mod.rs
// QTC M5: Consensus Engine
// Uses unified chain types: number (not height), proposer=[u8;32], sig on BlockHeader

pub mod producer;
pub mod validator;
pub use validator::validate_block_sig;

use std::time::{SystemTime, UNIX_EPOCH};
use crate::chain::{Block, BlockHeader};
use crate::mempool::{Transaction, Mempool};

pub const BLOCK_TIME_SECS: u64 = 2;
pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;
pub const MAX_TXS_PER_BLOCK: usize = 10_000;

pub type ValidatorId = [u8; 32];

#[derive(Debug, Clone)]
pub struct SlotProposer {
    pub validator: ValidatorId,
    pub slot: u64,
}

#[derive(Debug)]
pub struct ChainState {
    pub number: u64,
    pub head_hash: [u8; 32],
    pub base_fee: u64,
    pub genesis_time: u64,
}

impl ChainState {
    pub fn current_slot(&self) -> u64 {
        now_secs().saturating_sub(self.genesis_time) / BLOCK_TIME_SECS
    }
}

#[derive(Debug, PartialEq)]
pub enum ConsensusError {
    NotProposer,
    UnknownParent,
    FutureBlock,
    InvalidBlockSig,
    EmptyMempool,
    BlockGasExceeded,
}

pub struct Consensus {
    pub validator_id: ValidatorId,
}

impl Consensus {
    pub fn new(validator_id: ValidatorId, _validator_sk: Vec<u8>) -> Self {
        Self { validator_id }
    }

    pub fn try_propose(
        &self,
        chain: &ChainState,
        mempool: &mut Mempool,
    ) -> Result<Block, ConsensusError> {
        let slot = chain.current_slot();

        if !self.is_proposer(slot) {
            return Err(ConsensusError::NotProposer);
        }

        let mut block_gas_used = 0u64;
        let mut txs: Vec<Transaction> = Vec::new();

        for tx in mempool.peek_best(MAX_TXS_PER_BLOCK) {
            if block_gas_used + tx.gas_limit > BLOCK_GAS_LIMIT {
                break;
            }
            block_gas_used += tx.gas_limit;
            txs.push(tx.clone());
        }

        if txs.is_empty() {
            return Err(ConsensusError::EmptyMempool);
        }

        let header = BlockHeader {
            parent_hash: chain.head_hash,
            number: chain.number + 1,
            slot,
            timestamp: now_secs(),
            proposer: self.validator_id,
            tx_root: merkle_root(&txs),
            state_root: [0u8; 32],
            base_fee: chain.base_fee,
            gas_used: block_gas_used,
            gas_limit: BLOCK_GAS_LIMIT,
            signature: self.sign_header_bytes(),
        };

        let block = Block {
            header,
            transactions: txs.clone(),
        };

        for tx in &block.transactions {
            mempool.remove(&tx.hash);
        }

        let new_base_fee = calculate_next_base_fee(chain.base_fee, block_gas_used);
        mempool.update_base_fee(new_base_fee);

        Ok(block)
    }

    pub fn process_block(
        &self,
        block: &Block,
        chain: &ChainState,
    ) -> Result<(), ConsensusError> {
        if block.header.parent_hash != chain.head_hash {
            return Err(ConsensusError::UnknownParent);
        }
        if block.header.timestamp > now_secs() + 1 {
            return Err(ConsensusError::FutureBlock);
        }
        if block.header.signature.is_empty() {
            return Err(ConsensusError::InvalidBlockSig);
        }
        let gas_used: u64 = block.transactions.iter().map(|t| t.gas_limit).sum();
        if gas_used != block.header.gas_used || gas_used > BLOCK_GAS_LIMIT {
            return Err(ConsensusError::BlockGasExceeded);
        }
        Ok(())
    }

    fn is_proposer(&self, _slot: u64) -> bool {
        true // TODO: VRF in M8
    }

    fn sign_header_bytes(&self) -> Vec<u8> {
        vec![0u8; 2420] // TODO: wire M1 crypto::sign() in producer.rs
    }
}

fn calculate_next_base_fee(parent_base_fee: u64, parent_gas_used: u64) -> u64 {
    let target_gas = BLOCK_GAS_LIMIT / 2;
    if parent_gas_used == target_gas {
        return parent_base_fee;
    }
    if parent_gas_used > target_gas {
        let delta = parent_base_fee * (parent_gas_used - target_gas) / target_gas / 8;
        parent_base_fee + delta.max(1)
    } else {
        let delta = parent_base_fee * (target_gas - parent_gas_used) / target_gas / 8;
        parent_base_fee.saturating_sub(delta.max(1))
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn merkle_root(txs: &[Transaction]) -> [u8; 32] {
    if txs.is_empty() { return [0u8; 32]; }
    [1u8; 32] // TODO: real merkle in M8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_fee_increase() {
        let fee = calculate_next_base_fee(1000, BLOCK_GAS_LIMIT);
        assert!(fee > 1000);
        assert!(fee < 1150);
    }

    #[test]
    fn test_base_fee_decrease() {
        let fee = calculate_next_base_fee(1000, 0);
        assert!(fee < 1000);
        assert!(fee > 850);
    }

    #[test]
    fn test_base_fee_stable() {
        let fee = calculate_next_base_fee(1000, BLOCK_GAS_LIMIT / 2);
        assert_eq!(fee, 1000);
    }
}
