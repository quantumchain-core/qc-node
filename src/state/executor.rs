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

use crate::chain::Block;
use crate::state::StateDB;
use crate::mempool::Transaction;
use crate::state::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("insufficient balance: have {0}, need {1}")]
    InsufficientBalance(u128, u128),
    #[error("nonce mismatch: expected {0}, got {1}")]
    NonceMismatch(u64, u64),
    #[error("gas limit exceeded")]
    GasLimitExceeded,
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

        for tx in &block.transactions {
            let gas_used = Self::execute_tx(state, tx, base_fee, coinbase)?;

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

        let value    = tx.value as u128;
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

        Ok(tx.gas_limit)
    }
}
