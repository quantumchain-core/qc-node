// src/vesting/mod.rs
// QTC M14: Native Rust Vesting
//
// Implements all vesting schedules from TOKENOMICS.md:
// 1. CliffLinearVesting — cliff then linear (Founder, Team, Advisors)
// 2. LinearVesting      — no cliff, linear (Milestone grants)
// 3. TimelockedOpsFund  — 7-day spend window, transfers to 5/7 at block 100,000

use serde::{Deserialize, Serialize};
use crate::chain::Address;

pub const BLOCK_TIME_SECS: u64 = 2;
pub const BLOCKS_PER_MONTH: u64 = 30 * 24 * 3600 / BLOCK_TIME_SECS;
pub const BLOCKS_PER_YEAR: u64 = BLOCKS_PER_MONTH * 12;
pub const MULTISIG_TRANSFER_BLOCK: u64 = 100_000;
pub const SPEND_TIMELOCK_BLOCKS: u64 = 7 * 24 * 3600 / BLOCK_TIME_SECS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliffLinearVesting {
    pub beneficiary: Address,
    pub total_amount: u128,
    pub start_block: u64,
    pub cliff_blocks: u64,
    pub vesting_blocks: u64,
    pub claimed: u128,
}

impl CliffLinearVesting {
    pub fn founder(beneficiary: Address, total_amount: u128, tge_block: u64) -> Self {
        Self { beneficiary, total_amount, start_block: tge_block,
               cliff_blocks: BLOCKS_PER_MONTH * 12,
               vesting_blocks: BLOCKS_PER_YEAR * 4, claimed: 0 }
    }

    pub fn team_hire(beneficiary: Address, total_amount: u128, hire_block: u64) -> Self {
        Self { beneficiary, total_amount, start_block: hire_block,
               cliff_blocks: BLOCKS_PER_MONTH * 12,
               vesting_blocks: BLOCKS_PER_YEAR * 4, claimed: 0 }
    }

    pub fn genesis_advisor(beneficiary: Address, total_amount: u128, tge_block: u64) -> Self {
        Self { beneficiary, total_amount, start_block: tge_block,
               cliff_blocks: BLOCKS_PER_MONTH * 6,
               vesting_blocks: BLOCKS_PER_YEAR * 2, claimed: 0 }
    }

    pub fn vested_at(&self, current_block: u64) -> u128 {
        let elapsed = current_block.saturating_sub(self.start_block);
        if elapsed < self.cliff_blocks { return 0; }
        let post_cliff = elapsed - self.cliff_blocks;
        if post_cliff >= self.vesting_blocks { return self.total_amount; }
        self.total_amount.saturating_mul(post_cliff as u128) / (self.vesting_blocks as u128)
    }

    pub fn claimable_at(&self, current_block: u64) -> u128 {
        self.vested_at(current_block).saturating_sub(self.claimed)
    }

    pub fn claim(&mut self, current_block: u64) -> u128 {
        let c = self.claimable_at(current_block);
        self.claimed = self.claimed.saturating_add(c);
        c
    }

    pub fn clawback(&self, current_block: u64) -> u128 {
        self.total_amount.saturating_sub(self.vested_at(current_block))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinearVesting {
    pub beneficiary: Address,
    pub total_amount: u128,
    pub start_block: u64,
    pub vesting_blocks: u64,
    pub claimed: u128,
}

impl LinearVesting {
    pub fn milestone_grant(beneficiary: Address, total_amount: u128, release_block: u64) -> Self {
        Self { beneficiary, total_amount, start_block: release_block,
               vesting_blocks: BLOCKS_PER_MONTH * 6, claimed: 0 }
    }

    pub fn vested_at(&self, current_block: u64) -> u128 {
        let elapsed = current_block.saturating_sub(self.start_block);
        if elapsed >= self.vesting_blocks { return self.total_amount; }
        self.total_amount.saturating_mul(elapsed as u128) / (self.vesting_blocks as u128)
    }

    pub fn claimable_at(&self, current_block: u64) -> u128 {
        self.vested_at(current_block).saturating_sub(self.claimed)
    }

    pub fn claim(&mut self, current_block: u64) -> u128 {
        let c = self.claimable_at(current_block);
        self.claimed = self.claimed.saturating_add(c);
        c
    }

    pub fn clawback(&self, current_block: u64) -> u128 {
        self.total_amount.saturating_sub(self.vested_at(current_block))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelockedOpsFund {
    pub balance_usdc: u64,
    pub proposals: Vec<SpendProposal>,
    pub multisig_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpendProposal {
    pub id: u64,
    pub recipient: Address,
    pub amount_usdc: u64,
    pub purpose: String,
    pub proposed_at_block: u64,
    pub status: SpendStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpendStatus { Pending, Frozen, Executed, Rejected }

#[derive(Debug, Clone, PartialEq)]
pub enum VestingError {
    InsufficientFunds,
    MultisigActive,
    ProposalNotFound,
    InvalidProposalState,
    TimelockActive { blocks_remaining: u64 },
}

impl std::fmt::Display for VestingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VestingError::InsufficientFunds => write!(f, "insufficient funds"),
            VestingError::MultisigActive => write!(f, "multisig active — use governance"),
            VestingError::ProposalNotFound => write!(f, "proposal not found"),
            VestingError::InvalidProposalState => write!(f, "proposal not in Pending state"),
            VestingError::TimelockActive { blocks_remaining } =>
                write!(f, "timelock: {blocks_remaining} blocks remaining"),
        }
    }
}

impl TimelockedOpsFund {
    pub fn new(initial_usdc: u64) -> Self {
        Self { balance_usdc: initial_usdc, proposals: Vec::new(), multisig_active: false }
    }

    pub fn check_multisig_transfer(&mut self, current_block: u64) {
        if current_block >= MULTISIG_TRANSFER_BLOCK && !self.multisig_active {
            self.multisig_active = true;
        }
    }

    pub fn propose_spend(&mut self, recipient: Address, amount_usdc: u64,
                         purpose: String, current_block: u64) -> Result<u64, VestingError> {
        self.check_multisig_transfer(current_block);
        if self.multisig_active { return Err(VestingError::MultisigActive); }
        if amount_usdc > self.balance_usdc { return Err(VestingError::InsufficientFunds); }
        let id = self.proposals.len() as u64;
        self.proposals.push(SpendProposal {
            id, recipient, amount_usdc, purpose,
            proposed_at_block: current_block, status: SpendStatus::Pending,
        });
        Ok(id)
    }

    pub fn try_execute(&mut self, proposal_id: u64, current_block: u64) -> Result<u64, VestingError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(VestingError::ProposalNotFound)?;
        if proposal.status != SpendStatus::Pending { return Err(VestingError::InvalidProposalState); }
        let elapsed = current_block.saturating_sub(proposal.proposed_at_block);
        if elapsed < SPEND_TIMELOCK_BLOCKS {
            return Err(VestingError::TimelockActive { blocks_remaining: SPEND_TIMELOCK_BLOCKS - elapsed });
        }
        let amount = proposal.amount_usdc;
        proposal.status = SpendStatus::Executed;
        self.balance_usdc = self.balance_usdc.saturating_sub(amount);
        Ok(amount)
    }

    pub fn freeze_proposal(&mut self, proposal_id: u64) -> Result<(), VestingError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(VestingError::ProposalNotFound)?;
        if proposal.status != SpendStatus::Pending { return Err(VestingError::InvalidProposalState); }
        proposal.status = SpendStatus::Frozen;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FOUNDER: Address = [1u8; 32];
    const TGE: u64 = 1000;

    #[test] fn test_nothing_before_cliff() {
        let v = CliffLinearVesting::founder(FOUNDER, 1_000_000, TGE);
        assert_eq!(v.vested_at(TGE + v.cliff_blocks - 1), 0);
    }

    #[test] fn test_fully_vested_after_period() {
        let v = CliffLinearVesting::founder(FOUNDER, 1_000_000, TGE);
        assert_eq!(v.vested_at(TGE + v.cliff_blocks + v.vesting_blocks), 1_000_000);
    }

    #[test] fn test_half_vested_at_midpoint() {
        let v = CliffLinearVesting::founder(FOUNDER, 1_000_000, TGE);
        let mid = TGE + v.cliff_blocks + v.vesting_blocks / 2;
        let vested = v.vested_at(mid);
        assert!(vested >= 499_999 && vested <= 500_001);
    }

    #[test] fn test_claim_reduces_claimable() {
        let mut v = CliffLinearVesting::genesis_advisor(FOUNDER, 1_000_000, TGE);
        let mid = TGE + v.cliff_blocks + v.vesting_blocks / 2;
        assert!(v.claim(mid) > 0);
        assert_eq!(v.claimable_at(mid), 0);
    }

    #[test] fn test_clawback_full_before_cliff() {
        let v = CliffLinearVesting::founder(FOUNDER, 1_000_000, TGE);
        assert_eq!(v.clawback(TGE), 1_000_000);
    }

    #[test] fn test_clawback_zero_after_vesting() {
        let v = CliffLinearVesting::founder(FOUNDER, 1_000_000, TGE);
        assert_eq!(v.clawback(TGE + v.cliff_blocks + v.vesting_blocks), 0);
    }

    #[test] fn test_linear_no_cliff() {
        let v = LinearVesting::milestone_grant(FOUNDER, 250_000, TGE);
        assert_eq!(v.vested_at(TGE), 0);
        assert_eq!(v.vested_at(TGE + v.vesting_blocks), 250_000);
    }

    #[test] fn test_ops_fund_executes_after_timelock() {
        let mut fund = TimelockedOpsFund::new(300_000);
        let id = fund.propose_spend([2u8;32], 50_000, "audit".into(), 1000).unwrap();
        assert!(fund.try_execute(id, 1000 + SPEND_TIMELOCK_BLOCKS - 1).is_err());
        assert_eq!(fund.try_execute(id, 1000 + SPEND_TIMELOCK_BLOCKS).unwrap(), 50_000);
        assert_eq!(fund.balance_usdc, 250_000);
    }

    #[test] fn test_ops_fund_freeze_blocks_execution() {
        let mut fund = TimelockedOpsFund::new(300_000);
        let id = fund.propose_spend([2u8;32], 50_000, "test".into(), 1000).unwrap();
        fund.freeze_proposal(id).unwrap();
        assert!(fund.try_execute(id, 1000 + SPEND_TIMELOCK_BLOCKS + 1).is_err());
    }

    #[test] fn test_multisig_transfer_at_block_100k() {
        let mut fund = TimelockedOpsFund::new(300_000);
        fund.check_multisig_transfer(MULTISIG_TRANSFER_BLOCK - 1);
        assert!(!fund.multisig_active);
        fund.check_multisig_transfer(MULTISIG_TRANSFER_BLOCK);
        assert!(fund.multisig_active);
        assert_eq!(fund.propose_spend([2u8;32], 1_000, "blocked".into(), MULTISIG_TRANSFER_BLOCK),
                   Err(VestingError::MultisigActive));
    }

    #[test] fn test_total_supply_never_exceeds_allocation() {
        let mut v = CliffLinearVesting::founder(FOUNDER, 150_000_000, TGE);
        let mut total = 0u128;
        for month in [6u64, 12, 18, 24, 36, 48, 60] {
            total += v.claim(TGE + BLOCKS_PER_MONTH * month);
        }
        assert!(total <= 150_000_000);
        total += v.claim(TGE + BLOCKS_PER_MONTH * 61);
        assert_eq!(total, 150_000_000);
    }
}
