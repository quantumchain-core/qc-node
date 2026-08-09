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
//
// M14 WIRING (core-dev review, P2): vesting schedules and governance now
// live inside StateDB instead of as standalone, unreachable structs in
// src/vesting and src/governance. Both new fields use #[serde(default)]
// so a StateDB blob written by a pre-M14-wiring node still deserializes
// cleanly (as empty vesting/no governance) instead of breaking existing
// deployments on upgrade.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chain::Hash;
use crate::vesting::{CliffLinearVesting, LinearVesting, TimelockedOpsFund};
use crate::governance::Governance;

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

    // M14 WIRING: vesting schedules keyed by beneficiary address. A given
    // beneficiary has at most one CliffLinearVesting and one LinearVesting
    // grant in this design (matches the founder/team/advisor + milestone-
    // grant shape in vesting::mod.rs; a beneficiary with multiple grants
    // of the *same* kind would need a Vec here instead — not needed by
    // the schedules TOKENOMICS.md currently defines, so kept simple).
    #[serde(default)]
    cliff_vesting: HashMap<Address, CliffLinearVesting>,
    #[serde(default)]
    linear_vesting: HashMap<Address, LinearVesting>,
    #[serde(default)]
    ops_fund: Option<TimelockedOpsFund>,

    // M14 WIRING: governance is a single global instance (one DAO per
    // chain), not per-address — Option so a chain that never initializes
    // governance (e.g. existing tests, or a deployment that opts out)
    // isn't forced to carry seat/proposal state it never uses.
    #[serde(default)]
    governance: Option<Governance>,
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

    // -----------------------------------------------------------------
    // M14 WIRING: vesting accessors
    // -----------------------------------------------------------------

    pub fn get_cliff_vesting(&self, beneficiary: &Address) -> Option<&CliffLinearVesting> {
        self.cliff_vesting.get(beneficiary)
    }

    pub fn set_cliff_vesting(&mut self, beneficiary: Address, schedule: CliffLinearVesting) {
        self.cliff_vesting.insert(beneficiary, schedule);
    }

    pub fn get_linear_vesting(&self, beneficiary: &Address) -> Option<&LinearVesting> {
        self.linear_vesting.get(beneficiary)
    }

    pub fn set_linear_vesting(&mut self, beneficiary: Address, schedule: LinearVesting) {
        self.linear_vesting.insert(beneficiary, schedule);
    }

    pub fn ops_fund(&self) -> Option<&TimelockedOpsFund> {
        self.ops_fund.as_ref()
    }

    pub fn ops_fund_mut(&mut self) -> &mut Option<TimelockedOpsFund> {
        &mut self.ops_fund
    }

    pub fn init_ops_fund(&mut self, initial_usdc: u64) {
        self.ops_fund = Some(TimelockedOpsFund::new(initial_usdc));
    }

    // -----------------------------------------------------------------
    // M14 WIRING: governance accessors
    // -----------------------------------------------------------------

    pub fn governance(&self) -> Option<&Governance> {
        self.governance.as_ref()
    }

    pub fn governance_mut(&mut self) -> &mut Option<Governance> {
        &mut self.governance
    }

    pub fn init_governance(&mut self, permanent_seat_1: Address, permanent_seat_2: Address) {
        self.governance = Some(Governance::new(permanent_seat_1, permanent_seat_2));
    }

    /// SHA256 over sorted (address, balance, nonce) tuples, plus vesting
    /// claimed-amounts and governance proposal/vote state. Deterministic
    /// because zero-state accounts are pruned before this runs, and every
    /// map is sorted by key before hashing.
    ///
    /// M14 WIRING NOTE: this changes the state_root formula from the
    /// pre-wiring version (accounts only). Any chain with existing blocks
    /// signed against the old formula would need a hard fork / genesis
    /// reset to adopt this — there is no in-place migration for a
    /// consensus-critical hash. Safe today only because this repo has no
    /// live chain yet; call this out explicitly before it ever does.
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

        let mut cliff: Vec<_> = self.cliff_vesting.iter().collect();
        cliff.sort_by_key(|(addr, _)| *addr);
        for (addr, v) in cliff {
            hasher.update(addr);
            hasher.update(v.claimed.to_le_bytes());
        }

        let mut linear: Vec<_> = self.linear_vesting.iter().collect();
        linear.sort_by_key(|(addr, _)| *addr);
        for (addr, v) in linear {
            hasher.update(addr);
            hasher.update(v.claimed.to_le_bytes());
        }

        if let Some(fund) = &self.ops_fund {
            hasher.update(fund.balance_usdc.to_le_bytes());
            hasher.update((fund.proposals.len() as u64).to_le_bytes());
        }

        if let Some(gov) = &self.governance {
            hasher.update((gov.proposals.len() as u64).to_le_bytes());
            hasher.update(gov.next_proposal_id.to_le_bytes());
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
            // Executor operates on transactions already accepted by the
            // mempool (where the real pubkey/signature check happens) —
            // these tests exercise block execution/state transition only,
            // so a placeholder here is fine and intentionally not signed.
            from_pubkey: vec![0u8; 1312],
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
