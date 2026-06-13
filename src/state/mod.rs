// src/state/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chain::Hash;

pub mod executor;
pub mod storage;
pub use executor::{ExecError, Executor};
pub use storage::{Storage, StorageError};

pub type Address = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Account {
    pub balance: u128,
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage_root: Hash,
}

impl Account {
    pub fn new() -> Self { Self::default() }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateDB {
    accounts: HashMap<Address, Account>,
}

impl StateDB {
    pub fn new() -> Self { Self::default() }

    pub fn get_account(&self, addr: &Address) -> Account {
        self.accounts.get(addr).cloned().unwrap_or_default()
    }

    pub fn set_account(&mut self, addr: Address, account: Account) {
        self.accounts.insert(addr, account);
    }

    pub fn state_root(&self) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut accounts: Vec<_> = self.accounts.iter().collect();
        accounts.sort_by_key(|(addr, _)| *addr);
        for (addr, acc) in accounts {
            hasher.update(addr);
            hasher.update(acc.balance.to_le_bytes());
            hasher.update(acc.nonce.to_le_bytes());
        }
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{Block, BlockHeader};
    use crate::mempool::Transaction;
    use crate::state::executor::Executor;

    fn make_tx(from: [u8; 32], to: [u8; 32], value: u64, nonce: u64) -> Transaction {
        Transaction {
            hash: [0u8; 32],
            from,
            to,
            value,
            nonce,
            base_fee: 1,
            priority_fee: 0,
            gas_limit: 21,
            signature: vec![0u8; 2420],
            received_at: 0,
        }
    }

    fn make_block(txs: Vec<Transaction>, gas_used: u64) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: [0u8; 32],
                number: 1,
                slot: 0,
                timestamp: 0,
                proposer: [0u8; 32],
                tx_root: [0u8; 32],
                state_root: [0u8; 32],
                gas_limit: 10_000_000,
                gas_used,
                base_fee: 1,
                signature: vec![0u8; 2420],
            },
            transactions: txs,
        }
    }

    #[test]
    fn test_execute_transfer() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        let bob: Address = [2u8; 32];
        let coinbase: Address = [3u8; 32];

        state.set_account(alice, Account { balance: 1000, nonce: 0, ..Default::default() });

        let tx = make_tx(alice, bob, 100, 0);
        let block = make_block(vec![tx], 21);
        let gas_used = Executor::execute_block(&mut state, &block, &coinbase).unwrap();

        assert_eq!(gas_used, 21);
        assert_eq!(state.get_account(&alice).balance, 879);
        assert_eq!(state.get_account(&alice).nonce, 1);
        assert_eq!(state.get_account(&bob).balance, 100);
        assert_eq!(state.get_account(&coinbase).balance, 21);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        let bob: Address = [2u8; 32];
        let coinbase: Address = [3u8; 32];
        state.set_account(alice, Account { balance: 10, nonce: 0, ..Default::default() });
        let tx = make_tx(alice, bob, 100, 0);
        let block = make_block(vec![tx], 21);
        let result = Executor::execute_block(&mut state, &block, &coinbase);
        assert!(matches!(result, Err(ExecError::InsufficientBalance(_, _))));
    }

    #[test]
    fn test_nonce_mismatch() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        let bob: Address = [2u8; 32];
        let coinbase: Address = [3u8; 32];
        state.set_account(alice, Account { balance: 1000, nonce: 5, ..Default::default() });
        let tx = make_tx(alice, bob, 100, 0);
        let block = make_block(vec![tx], 21);
        let result = Executor::execute_block(&mut state, &block, &coinbase);
        assert!(matches!(result, Err(ExecError::NonceMismatch(5, 0))));
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        state.set_account(alice, Account { balance: 500, nonce: 1, ..Default::default() });
        assert_eq!(state.state_root(), state.state_root());
    }
}
