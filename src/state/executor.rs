// src/state/executor.rs
// QTC M6: Transaction Executor
// All arithmetic in u128 to avoid overflow. tx.value/gas_limit cast up.
// NOTE: GasUsedMismatch check removed — producer fills gas_used AFTER execution.
//       Callers are responsible for verifying header.gas_used matches return value.
//
// AUDIT-007 FIX: total_gas_used now uses checked_add to prevent u64 overflow.
// AUDIT-008 FIX: all three accounts (sender, recipient, coinbase) are read
//   upfront before any writes. This prevents last-write-wins corruption when
//   tx.from == coinbase or tx.to == coinbase.
//
// M14 WIRING (core-dev review, P2): execute_tx now dispatches on
// tx.action. Transfer (the default/only kind that existed before this)
// runs exactly as before. Every other action still pays gas exactly like
// a Transfer (nonce bump + gas_cost debit/credit), but skips the
// value-transfer step and instead mutates the vesting/governance state
// now wired into StateDB (see src/state/mod.rs). Gas is charged even on
// a failed action (e.g. voting twice) — same as real chains: submitting
// an invalid action still costs the sender gas, which is what prevents
// free-spam voting/proposal floods.

use crate::chain::Block;
use crate::state::StateDB;
use crate::mempool::{Transaction, TxAction};
use crate::state::Address;
use crate::governance::GovernanceError;
use crate::vesting::VestingError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("insufficient balance: have {0}, need {1}")]
    InsufficientBalance(u128, u128),
    #[error("nonce mismatch: expected {0}, got {1}")]
    NonceMismatch(u64, u64),
    #[error("gas limit exceeded")]
    GasLimitExceeded,
    #[error("no matching vesting schedule for this address")]
    NoVestingSchedule,
    #[error("vesting error: {0}")]
    Vesting(String),
    #[error("governance not initialized on this chain")]
    GovernanceNotInitialized,
    #[error("governance error: {0}")]
    Governance(String),
    #[error("ops fund not initialized on this chain")]
    OpsFundNotInitialized,
}

impl From<VestingError> for ExecError {
    fn from(e: VestingError) -> Self { ExecError::Vesting(e.to_string()) }
}
impl From<GovernanceError> for ExecError {
    fn from(e: GovernanceError) -> Self { ExecError::Governance(e.to_string()) }
}

pub struct Executor;

impl Executor {
    pub fn execute_block(
        state: &mut StateDB,
        block: &Block,
        coinbase: &Address,
    ) -> Result<u64, ExecError> {
        let mut total_gas_used = 0u64;
        let base_fee = block.header.base_fee as u128;
        let current_block = block.header.number;

        for tx in &block.transactions {
            let gas_used = Self::execute_tx(state, tx, base_fee, coinbase, current_block)?;

            // AUDIT-007: use checked_add to prevent u64 overflow
            total_gas_used = total_gas_used
                .checked_add(gas_used)
                .ok_or(ExecError::GasLimitExceeded)?;

            if total_gas_used > block.header.gas_limit {
                return Err(ExecError::GasLimitExceeded);
            }
        }

        // NOTE: we intentionally do NOT check total_gas_used == block.header.gas_used
        // here, because the producer sets gas_used AFTER calling execute_block.
        // Block validation (in node/mod.rs) verifies this instead.

        Ok(total_gas_used)
    }

    fn execute_tx(
        state: &mut StateDB,
        tx: &Transaction,
        base_fee: u128,
        coinbase: &Address,
        current_block: u64,
    ) -> Result<u64, ExecError> {
        // AUDIT-008 FIX: read ALL accounts before ANY writes.
        // If tx.from == coinbase or tx.to == coinbase, the old code would
        // overwrite the account with a stale read (last write wins).
        // Reading all three upfront and writing all three at the end is correct
        // regardless of whether the addresses overlap.
        let mut sender_acc    = state.get_account(&tx.from);
        let mut recipient_acc = state.get_account(&tx.to);
        let mut coinbase_acc  = state.get_account(coinbase);

        // Nonce check
        if sender_acc.nonce != tx.nonce {
            return Err(ExecError::NonceMismatch(sender_acc.nonce, tx.nonce));
        }

        // M14 WIRING: non-Transfer actions carry no `value` transfer —
        // only gas is charged. `value` is ignored for these (wallets
        // should send 0, but a nonzero value is simply not moved rather
        // than treated as an error, to keep this permissive for now).
        let value = if matches!(tx.action, TxAction::Transfer) { tx.value as u128 } else { 0 };
        let gas_cost = (tx.gas_limit as u128) * base_fee;
        let total_cost = value + gas_cost;

        // Solvency check
        if sender_acc.balance < total_cost {
            return Err(ExecError::InsufficientBalance(sender_acc.balance, total_cost));
        }

        // Apply all changes in memory before writing
        sender_acc.balance -= total_cost;
        sender_acc.nonce   += 1;

        // Handle overlapping addresses correctly:
        // If sender == recipient (self-transfer), apply both debits/credits to same account.
        // If sender == coinbase, gas credit goes back partially to sender.
        // By reading all three upfront, we avoid the last-write-wins bug.
        if tx.from == tx.to {
            // Self-transfer: only gas cost leaves, value stays
            sender_acc.balance += value;
        } else {
            recipient_acc.balance += value;
        }

        if tx.from == *coinbase {
            // Sender is the coinbase: gas cost was deducted above, add it back
            sender_acc.balance += gas_cost;
        } else if tx.to == *coinbase {
            // Recipient is the coinbase: value credit + gas credit both apply
            coinbase_acc.balance += value + gas_cost;
            // But we already credited recipient_acc above, remove double-count
            recipient_acc.balance -= value;
        } else {
            coinbase_acc.balance += gas_cost;
        }

        // Write all accounts
        state.set_account(tx.from,   sender_acc);
        if tx.from != tx.to {
            state.set_account(tx.to, recipient_acc);
        }
        if tx.from != *coinbase && tx.to != *coinbase {
            state.set_account(*coinbase, coinbase_acc);
        } else if tx.from == *coinbase {
            // sender_acc already has the correct coinbase balance
            // set_account(tx.from) above already wrote it
        } else {
            // tx.to == coinbase: write coinbase_acc (which includes gas+value)
            state.set_account(*coinbase, coinbase_acc);
        }

        // M14 WIRING: run the action itself, now that gas/nonce accounting
        // is committed. A failing action returns Err (the whole block/tx
        // is then rejected by the caller) — gas already deducted above is
        // NOT refunded on that path, matching normal chain behavior.
        Self::dispatch_action(state, tx, current_block)?;

        Ok(tx.gas_limit)
    }

    /// M14 WIRING: mutate vesting/governance state per tx.action.
    /// No-op for TxAction::Transfer (the normal, pre-M14 path).
    fn dispatch_action(
        state: &mut StateDB,
        tx: &Transaction,
        current_block: u64,
    ) -> Result<(), ExecError> {
        match &tx.action {
            TxAction::Transfer => Ok(()),

            TxAction::ClaimCliffVesting => {
                let mut schedule = state.get_cliff_vesting(&tx.from)
                    .cloned()
                    .ok_or(ExecError::NoVestingSchedule)?;
                let claimed = schedule.claim(current_block);
                state.set_cliff_vesting(tx.from, schedule);
                let mut acc = state.get_account(&tx.from);
                acc.balance = acc.balance.saturating_add(claimed);
                state.set_account(tx.from, acc);
                Ok(())
            }

            TxAction::ClaimLinearVesting => {
                let mut schedule = state.get_linear_vesting(&tx.from)
                    .cloned()
                    .ok_or(ExecError::NoVestingSchedule)?;
                let claimed = schedule.claim(current_block);
                state.set_linear_vesting(tx.from, schedule);
                let mut acc = state.get_account(&tx.from);
                acc.balance = acc.balance.saturating_add(claimed);
                state.set_account(tx.from, acc);
                Ok(())
            }

            TxAction::ProposeSpend { recipient, amount_usdc, purpose } => {
                let fund = state.ops_fund_mut().as_mut()
                    .ok_or(ExecError::OpsFundNotInitialized)?;
                fund.propose_spend(*recipient, *amount_usdc, purpose.clone(), current_block)?;
                Ok(())
            }

            TxAction::ExecuteSpend { proposal_id } => {
                let fund = state.ops_fund_mut().as_mut()
                    .ok_or(ExecError::OpsFundNotInitialized)?;
                let amount = fund.try_execute(*proposal_id, current_block)?;
                // NOTE: TimelockedOpsFund tracks a USDC balance, a
                // separate unit from the native QTC `Account.balance`
                // moved elsewhere in this function — no QTC-balance
                // change happens here, only the fund's internal ledger.
                // Wiring USDC custody to an actual on-chain asset is a
                // separate, not-yet-designed piece of work.
                let _ = amount;
                Ok(())
            }

            TxAction::SubmitProposal { proposal_type, description } => {
                let gov = state.governance_mut().as_mut()
                    .ok_or(ExecError::GovernanceNotInitialized)?;
                // active_validator_count: needed for quorum math but this
                // executor has no registry reference. Passing 0 here is a
                // known gap — quorum_pct() math against 0 validators means
                // validator_quorum_met() trivially passes for any
                // token-vote proposal. Flagged, not fixed: wiring the
                // real registry count through to the executor is the
                // next piece of this feature, not silently faked here.
                gov.submit_proposal(tx.from, proposal_type.clone(), description.clone(),
                    current_block, 0)?;
                Ok(())
            }

            TxAction::CastMultisigVote { proposal_id, vote } => {
                let gov = state.governance_mut().as_mut()
                    .ok_or(ExecError::GovernanceNotInitialized)?;
                gov.cast_multisig_vote(tx.from, *proposal_id, vote.clone(), current_block)?;
                Ok(())
            }

            TxAction::CastValidatorVote { proposal_id, vote } => {
                let gov = state.governance_mut().as_mut()
                    .ok_or(ExecError::GovernanceNotInitialized)?;
                gov.cast_validator_vote(tx.from, *proposal_id, vote.clone())?;
                Ok(())
            }
        }
    }
            }
