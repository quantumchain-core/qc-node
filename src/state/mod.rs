use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chain::Hash;

pub mod executor;
pub mod storage;
pub use executor::{Executor, ExecError};
pub use storage::{Storage, StorageError};

pub type Address = [u8; 32];

/// Account state - like Ethereum accounts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub balance: u128, // M6: coin balance
    pub nonce: u64, // M6: tx counter per account
    pub code: Vec<u8>, // M9: smart contract code - empty for now
    pub storage_root: Hash, // M9: contract storage - zero for now
}

impl Account {
    pub fn new() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            code: Vec::new(),
            storage_root: [0u8; 32],
        }
    }
}

/// In-memory state DB - M6 keeps it simple first
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDB {
    accounts: HashMap<Address, Account>,
}

impl StateDB {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Get account, create if missing
    pub fn get_account(&self, addr: &Address) -> Account {
        self.accounts.get(addr).cloned().unwrap_or_else(Account::new)
    }

    /// Update account
    pub fn set_account(&mut self, addr: Address, account: Account) {
        self.accounts.insert(addr, account);
    }

    /// M6: Calculate state_root - hash of all accounts
    /// Simple version: hash all account data. M9 will use Merkle Patricia Trie
    pub fn state_root(&self) -> Hash {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();

        // Sort by address so root is deterministic
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
    use crate::mempool::Transaction;
    use crate::chain::{Block, BlockHeader};
    use tempfile::TempDir; // Add this import

    #
    fn test_execute_transfer() {
        // M6: Each test gets its own temp DB dir to prevent sled lock conflicts
        let tmp_dir = TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp_dir.path());

        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        let bob: Address = [2u8; 32];
        let coinbase: Address = [3u8; 32];

        state.set_account(alice, Account {
            balance: 1000,
            nonce: 0,
            code: vec![],
            storage_root: [0u8; 32]
        });

        let tx = Transaction {
            from: alice,
            to: bob,
            value: 100,
            nonce: 0,
            gas_limit: 21,
        };

        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            timestamp: 0,
            state_root: [0u8; 32],
            gas_limit: 10_000_000,
            gas_used: 21,
            base_fee: 1,
            signature: [0u8; 2420],
        };

        let block = Block { header, transactions: vec![tx] };

        let gas_used = Executor::execute_block(&mut state, &block, &coinbase).unwrap();

        assert_eq!(gas_used, 21);
        assert_eq!(state.get_account(&alice).balance, 879); // 1000 - 100 - 21
        assert_eq!(state.get_account(&alice).nonce, 1);
        assert_eq!(state.get_account(&bob).balance, 100);
        assert_eq!(state.get_account(&coinbase).balance, 21);
    }
    }
