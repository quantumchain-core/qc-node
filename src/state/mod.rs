// src/state/mod.rs
// QTC M6: Account model, StateDB, state root
//
// AUDIT-010 FIX: set_account() now prunes zero-state accounts (balance=0,
// nonce=0, no code, no storage). This ensures two states with the same
// effective balances always produce the same state_root, regardless of
// whether zero accounts were explicitly written or never touched.
// Without this, a state with explicit zero-balance accounts could produce
// a different root than a state with no entry for those addresses —
// breaking consensus-critical state root verification.

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

    /// Returns true if this account has no state — same as a missing account.
    pub fn is_empty(&self) -> bool {
        self.balance == 0
            && self.nonce == 0
            && self.code.is_empty()
            && self.storage_root == [0u8; 32]
    }
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

    /// Store an account. AUDIT-010: if the account is effectively empty
    /// (zero balance, zero nonce, no code, no storage), remove it from
    /// the map entirely. This keeps state_root deterministic — an explicit
    /// zero account and a missing account are identical.
    pub fn set_account(&mut self, addr: Address, account: Account) {
        if account.is_empty() {
            self.accounts.remove(&addr);
        } else {
            self.accounts.insert(addr, account);
        }
    }

    /// SHA256 over sorted (address, balance, nonce) tuples.
    /// Deterministic because zero-state accounts are pruned before this runs.
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

    // AUDIT-010: zero account pruning tests
    #[test]
    fn test_zero_account_pruned_on_set() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        state.set_account(alice, Account { balance: 0, nonce: 0, ..Default::default() });
        // zero account must NOT be stored
        assert_eq!(state.accounts.len(), 0);
    }

    #[test]
    fn test_state_root_same_with_or_without_zero_account() {
        let mut state_a = StateDB::new();
        let mut state_b = StateDB::new();
        let alice: Address = [1u8; 32];
        let bob: Address = [2u8; 32];

        // state_a: alice has balance, bob never touched
        state_a.set_account(alice, Account { balance: 500, nonce: 1, ..Default::default() });

        // state_b: alice has balance, bob explicitly set to zero
        state_b.set_account(alice, Account { balance: 500, nonce: 1, ..Default::default() });
        state_b.set_account(bob, Account { balance: 0, nonce: 0, ..Default::default() });

        // Both must produce identical state roots (AUDIT-010)
        assert_eq!(state_a.state_root(), state_b.state_root());
    }

    #[test]
    fn test_nonzero_account_not_pruned() {
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        state.set_account(alice, Account { balance: 1, nonce: 0, ..Default::default() });
        assert_eq!(state.accounts.len(), 1);
        // nonce only, no balance
        let bob: Address = [2u8; 32];
        state.set_account(bob, Account { balance: 0, nonce: 1, ..Default::default() });
        assert_eq!(state.accounts.len(), 2);
    }

    // AUDIT-008 regression tests: coinbase overlap cases.
    // These exercise the exact bug class AUDIT-008 fixed — a naive
    // sequential read-modify-write of sender/recipient/coinbase would
    // silently lose balance when addresses overlap. Verifying via total
    // supply conservation: total balance across all accounts must be
    // unchanged after a transfer (gas fees + value just move between
    // existing accounts, nothing is created or destroyed).

    fn total_balance(state: &StateDB, addrs: &[Address]) -> u128 {
        addrs.iter().map(|a| state.get_account(a).balance).sum()
    }

    #[test]
    fn test_coinbase_is_sender_balance_conserved() {
        // tx.from == coinbase: sender pays gas to themselves (net: only
        // value moves to recipient, gas_cost should return to sender).
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32]; // sender AND coinbase
        let bob: Address = [2u8; 32];   // recipient
        state.set_account(alice, Account { balance: 1_000_000, nonce: 0, ..Default::default() });

        let tx = make_tx(alice, bob, 100, 0);
        let block = make_block(vec![tx], 21);
        let before = total_balance(&state, &[alice, bob]);

        Executor::execute_block(&mut state, &block, &alice).unwrap();

        let after = total_balance(&state, &[alice, bob]);
        assert_eq!(before, after, "total supply must be conserved when sender == coinbase");
        // alice paid gas_cost (21) but it came right back as coinbase fee,
        // net effect: alice only loses `value` (100) to bob
        assert_eq!(state.get_account(&alice).balance, 1_000_000 - 100);
        assert_eq!(state.get_account(&bob).balance, 100);
    }

    #[test]
    fn test_coinbase_is_recipient_balance_conserved() {
        // tx.to == coinbase: recipient is also the fee beneficiary
        // (receives both the transferred value AND the gas fee).
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32]; // sender
        let bob: Address = [2u8; 32];   // recipient AND coinbase
        state.set_account(alice, Account { balance: 1_000_000, nonce: 0, ..Default::default() });

        let tx = make_tx(alice, bob, 100, 0);
        let block = make_block(vec![tx], 21);
        let before = total_balance(&state, &[alice, bob]);

        Executor::execute_block(&mut state, &block, &bob).unwrap();

        let after = total_balance(&state, &[alice, bob]);
        assert_eq!(before, after, "total supply must be conserved when recipient == coinbase");
        // bob should receive value (100) + gas fee (21) = 121
        assert_eq!(state.get_account(&bob).balance, 121);
        assert_eq!(state.get_account(&alice).balance, 1_000_000 - 100 - 21);
    }

    #[test]
    fn test_self_transfer_balance_conserved() {
        // tx.from == tx.to: alice sends to herself. Only gas_cost should
        // leave her balance (value transfers to herself, net zero).
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        let coinbase: Address = [9u8; 32];
        state.set_account(alice, Account { balance: 1_000_000, nonce: 0, ..Default::default() });

        let tx = make_tx(alice, alice, 100, 0); // from == to == alice
        let block = make_block(vec![tx], 21);

        Executor::execute_block(&mut state, &block, &coinbase).unwrap();

        // alice only loses gas_cost; value transfer to self nets to zero
        assert_eq!(state.get_account(&alice).balance, 1_000_000 - 21);
        assert_eq!(state.get_account(&coinbase).balance, 21);
    }

    #[test]
    fn test_self_transfer_and_self_coinbase_balance_conserved() {
        // The fully degenerate case: tx.from == tx.to == coinbase.
        // Alice sends to herself AND collects her own gas fee.
        // Net effect: balance unchanged.
        let mut state = StateDB::new();
        let alice: Address = [1u8; 32];
        state.set_account(alice, Account { balance: 1_000_000, nonce: 0, ..Default::default() });

        let tx = make_tx(alice, alice, 100, 0); // from == to == alice
        let block = make_block(vec![tx], 21);

        Executor::execute_block(&mut state, &block, &alice).unwrap();

        // value returns to self, gas fee returns to self: net zero change
        assert_eq!(state.get_account(&alice).balance, 1_000_000);
        assert_eq!(state.get_account(&alice).nonce, 1);
    }
}
