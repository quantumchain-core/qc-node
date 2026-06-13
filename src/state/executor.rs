// src/state/executor.rs
// QTC M6: Transaction Executor
// All arithmetic in u128 to avoid overflow. tx.value/gas_limit cast up.
// NOTE: GasUsedMismatch check removed — producer fills gas_used AFTER execution.
//       Callers are responsible for verifying header.gas_used matches return value.

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
            total_gas_used += gas_used;
            if total_gas_used > block.header.gas_limit {
                return Err(ExecError::GasLimitExceeded);
            }
        }

        // NOTE: we intentionally do NOT check total_gas_used == block.header.gas_used
        // here, because the producer sets gas_used AFTER calling execute_block.
        // Block validation (in consensus/validator.rs) should verify this instead.

        Ok(total_gas_used)
    }

    fn execute_tx(
        state: &mut StateDB,
        tx: &Transaction,
        base_fee: u128,
        coinbase: &Address,
    ) -> Result<u64, ExecError> {
        let mut sender = state.get_account(&tx.from);

        if sender.nonce != tx.nonce {
            return Err(ExecError::NonceMismatch(sender.nonce, tx.nonce));
        }

        let value = tx.value as u128;
        let gas_cost = (tx.gas_limit as u128) * base_fee;
        let total_cost = value + gas_cost;

        if sender.balance < total_cost {
            return Err(ExecError::InsufficientBalance(sender.balance, total_cost));
        }

        sender.balance -= total_cost;
        sender.nonce += 1;
        state.set_account(tx.from, sender);

        let mut recipient = state.get_account(&tx.to);
        recipient.balance += value;
        state.set_account(tx.to, recipient);

        let mut coinbase_acc = state.get_account(coinbase);
        coinbase_acc.balance += gas_cost;
        state.set_account(*coinbase, coinbase_acc);

        Ok(tx.gas_limit)
    }
}
