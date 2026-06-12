use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chain::Hash;

pub type Address = [u8; 32];

/// Account state - like Ethereum accounts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub balance: u128,      // M6: coin balance
    pub nonce: u64,         // M6: tx counter per account
    pub code: Vec<u8>,      // M9: smart contract code - empty for now
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
#[derive(Debug, Clone)]
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
