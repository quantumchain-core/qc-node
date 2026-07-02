// src/governance/mod.rs
// QTC M14: Native Rust Governance
//
// Implements GOVERNANCE.md rules:
// - 5/7 multisig with recusal rule
// - 7-day review period (14 days for governance changes)
// - 51% validator quorum (66% for governance changes)
// - Immutable rule enforcement — auto-rejects banned proposals
// - Proposer cooldown (1 proposal per 30 days per address)

use serde::{Deserialize, Serialize};
use crate::chain::Address;

pub const PROPOSAL_REVIEW_BLOCKS: u64   = 7  * 24 * 3600 / 2; // 302,400
pub const GOVERNANCE_REVIEW_BLOCKS: u64 = 14 * 24 * 3600 / 2; // 604,800
pub const EMERGENCY_REVIEW_BLOCKS: u64  = 24 * 3600 / 2;       // 43,200
pub const STANDARD_QUORUM_PCT: u64      = 51;
pub const GOVERNANCE_QUORUM_PCT: u64    = 66;
pub const MULTISIG_SEATS: usize         = 7;
pub const MULTISIG_THRESHOLD: usize     = 5;
pub const PROPOSER_COOLDOWN_BLOCKS: u64 = 30 * 24 * 3600 / 2; // 1,296,000

// ---------------------------------------------------------------------------
// Immutable rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImmutableRule {
    TotalSupply,
    FounderVesting,
    FirstDevAllocation,
    MITLicenseRequirement,
    MultisigThreshold,
}

// ---------------------------------------------------------------------------
// Seats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SeatType {
    Permanent,
    AutoStake,
    AutoContrib,
    Elected,
    DAOReserve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultisigSeat {
    pub seat_number: u8,
    pub seat_type: SeatType,
    pub holder: Option<Address>,
    pub assigned_at_block: u64,
}

impl MultisigSeat {
    pub fn permanent(seat_number: u8, holder: Address) -> Self {
        Self { seat_number, seat_type: SeatType::Permanent,
               holder: Some(holder), assigned_at_block: 0 }
    }
    pub fn is_filled(&self) -> bool { self.holder.is_some() }
}

// ---------------------------------------------------------------------------
// Proposals
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    TreasuryRelease   { amount_qtc: u128, recipient: Address },
    GrantApproval     { grantee: Address, amount_qtc: u128, milestone: String },
    ProtocolUpgrade   { description: String },
    GovernanceChange  { rule_changed: String, new_value: String },
    EmergencyPatch    { description: String },
    FillSeat7         { candidate: Address },
}

impl ProposalType {
    pub fn requires_token_vote(&self) -> bool {
        matches!(self, ProposalType::ProtocolUpgrade { .. }
                     | ProposalType::GovernanceChange { .. }
                     | ProposalType::FillSeat7 { .. })
    }

    pub fn review_blocks(&self) -> u64 {
        match self {
            ProposalType::GovernanceChange { .. } => GOVERNANCE_REVIEW_BLOCKS,
            ProposalType::EmergencyPatch   { .. } => EMERGENCY_REVIEW_BLOCKS,
            _                                      => PROPOSAL_REVIEW_BLOCKS,
        }
    }

    pub fn quorum_pct(&self) -> u64 {
        match self {
            ProposalType::GovernanceChange { .. } => GOVERNANCE_QUORUM_PCT,
            _                                      => STANDARD_QUORUM_PCT,
        }
    }

    pub fn touches_immutable(&self) -> Option<ImmutableRule> {
        if let ProposalType::GovernanceChange { rule_changed, .. } = self {
            let r = rule_changed.to_lowercase();
            if r.contains("total_supply") || r.contains("supply") {
                return Some(ImmutableRule::TotalSupply);
            }
            if r.contains("founder_vesting") || r.contains("seat_1") {
                return Some(ImmutableRule::FounderVesting);
            }
            if r.contains("grant_001") || r.contains("first_dev") {
                return Some(ImmutableRule::FirstDevAllocation);
            }
            if r.contains("mit_license") || r.contains("license") {
                return Some(ImmutableRule::MITLicenseRequirement);
            }
            if r.contains("multisig_threshold") || r.contains("5/7") {
                return Some(ImmutableRule::MultisigThreshold);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Vote { Yes, No, Abstain }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    UnderReview,
    Voting,
    Approved,
    Rejected,
    RejectedImmutable { rule: ImmutableRule },
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub description: String,
    pub proposed_at_block: u64,
    pub status: ProposalStatus,
    pub multisig_votes: Vec<(u8, Vote)>,
    pub validator_votes: Vec<(Address, Vote)>,
    pub active_validator_count: u64,
}

impl Proposal {
    pub fn new(id: u64, proposer: Address, proposal_type: ProposalType,
               description: String, proposed_at_block: u64,
               active_validator_count: u64) -> Self {
        Self { id, proposer, proposal_type, description, proposed_at_block,
               status: ProposalStatus::UnderReview,
               multisig_votes: Vec::new(), validator_votes: Vec::new(),
               active_validator_count }
    }

    pub fn multisig_yes_count(&self) -> usize {
        self.multisig_votes.iter().filter(|(_, v)| *v == Vote::Yes).count()
    }

    pub fn multisig_no_count(&self) -> usize {
        self.multisig_votes.iter().filter(|(_, v)| *v == Vote::No).count()
    }

    pub fn multisig_approved(&self) -> bool {
        self.multisig_yes_count() >= MULTISIG_THRESHOLD
    }

    pub fn validator_quorum_met(&self) -> bool {
        if !self.proposal_type.requires_token_vote() { return true; }
        let yes = self.validator_votes.iter()
            .filter(|(_, v)| *v == Vote::Yes).count() as u64;
        let total = self.validator_votes.len() as u64;
        let needed = self.active_validator_count
            .saturating_mul(self.proposal_type.quorum_pct()) / 100;
        total >= needed && yes > total / 2
    }

    pub fn is_fully_approved(&self) -> bool {
        self.multisig_approved() && self.validator_quorum_met()
    }
}

// ---------------------------------------------------------------------------
// Governance state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Governance {
    pub seats: Vec<MultisigSeat>,
    pub proposals: Vec<Proposal>,
    pub next_proposal_id: u64,
    pub proposer_cooldown: Vec<(Address, u64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GovernanceError {
    ProposalNotFound,
    TouchesImmutableRule(ImmutableRule),
    ReviewPeriodNotOver,
    AlreadyVoted,
    NotAMultisigHolder,
    RecusedFromOwnProposal,
    ProposerCooldownActive { blocks_remaining: u64 },
    InvalidStatus,
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::ProposalNotFound => write!(f, "proposal not found"),
            GovernanceError::TouchesImmutableRule(r) =>
                write!(f, "touches immutable rule: {r:?}"),
            GovernanceError::ReviewPeriodNotOver => write!(f, "review period not over"),
            GovernanceError::AlreadyVoted => write!(f, "already voted"),
            GovernanceError::NotAMultisigHolder => write!(f, "not a multisig seat holder"),
            GovernanceError::RecusedFromOwnProposal => write!(f, "recused from own proposal"),
            GovernanceError::ProposerCooldownActive { blocks_remaining } =>
                write!(f, "cooldown: {blocks_remaining} blocks remaining"),
            GovernanceError::InvalidStatus => write!(f, "invalid proposal status"),
        }
    }
}

impl Governance {
    pub fn new(permanent_seat_1: Address, permanent_seat_2: Address) -> Self {
        Self {
            seats: vec![
                MultisigSeat::permanent(1, permanent_seat_1),
                MultisigSeat::permanent(2, permanent_seat_2),
                MultisigSeat { seat_number: 3, seat_type: SeatType::AutoStake,
                               holder: None, assigned_at_block: 0 },
                MultisigSeat { seat_number: 4, seat_type: SeatType::AutoContrib,
                               holder: None, assigned_at_block: 0 },
                MultisigSeat { seat_number: 5, seat_type: SeatType::Elected,
                               holder: None, assigned_at_block: 0 },
                MultisigSeat { seat_number: 6, seat_type: SeatType::Elected,
                               holder: None, assigned_at_block: 0 },
                MultisigSeat { seat_number: 7, seat_type: SeatType::DAOReserve,
                               holder: None, assigned_at_block: 0 },
            ],
            proposals: Vec::new(),
            next_proposal_id: 0,
            proposer_cooldown: Vec::new(),
        }
    }

    pub fn assign_auto_seat(&mut self, seat: u8, holder: Address, current_block: u64) {
        if let Some(s) = self.seats.iter_mut().find(|s| s.seat_number == seat) {
            if matches!(s.seat_type, SeatType::AutoStake | SeatType::AutoContrib) {
                s.holder = Some(holder);
                s.assigned_at_block = current_block;
            }
        }
    }

    fn find_seat(&self, addr: &Address) -> Option<u8> {
        self.seats.iter()
            .find(|s| s.holder.as_ref() == Some(addr))
            .map(|s| s.seat_number)
    }

    pub fn submit_proposal(&mut self, proposer: Address, proposal_type: ProposalType,
                           description: String, current_block: u64,
                           active_validator_count: u64) -> Result<u64, GovernanceError> {
        if let Some(rule) = proposal_type.touches_immutable() {
            return Err(GovernanceError::TouchesImmutableRule(rule));
        }

        if let Some((_, last)) = self.proposer_cooldown.iter()
            .find(|(a, _)| a == &proposer) {
            let elapsed = current_block.saturating_sub(*last);
            if elapsed < PROPOSER_COOLDOWN_BLOCKS {
                return Err(GovernanceError::ProposerCooldownActive {
                    blocks_remaining: PROPOSER_COOLDOWN_BLOCKS - elapsed,
                });
            }
        }

        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        self.proposals.push(Proposal::new(id, proposer, proposal_type,
            description, current_block, active_validator_count));

        if let Some(e) = self.proposer_cooldown.iter_mut().find(|(a, _)| a == &proposer) {
            e.1 = current_block;
        } else {
            self.proposer_cooldown.push((proposer, current_block));
        }

        Ok(id)
    }

    pub fn cast_multisig_vote(&mut self, voter: Address, proposal_id: u64,
                              vote: Vote, current_block: u64) -> Result<(), GovernanceError> {
        let seat_number = self.find_seat(&voter)
            .ok_or(GovernanceError::NotAMultisigHolder)?;

        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.proposer == voter {
            return Err(GovernanceError::RecusedFromOwnProposal);
        }

        let elapsed = current_block.saturating_sub(proposal.proposed_at_block);
        if elapsed < proposal.proposal_type.review_blocks() {
            return Err(GovernanceError::ReviewPeriodNotOver);
        }

        if proposal.multisig_votes.iter().any(|(s, _)| *s == seat_number) {
            return Err(GovernanceError::AlreadyVoted);
        }

        if !matches!(proposal.status, ProposalStatus::UnderReview | ProposalStatus::Voting) {
            return Err(GovernanceError::InvalidStatus);
        }

        proposal.status = ProposalStatus::Voting;
        proposal.multisig_votes.push((seat_number, vote));

        if proposal.is_fully_approved() {
            proposal.status = ProposalStatus::Approved;
        } else if proposal.multisig_no_count() > MULTISIG_SEATS - MULTISIG_THRESHOLD {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(())
    }

    pub fn cast_validator_vote(&mut self, validator: Address, proposal_id: u64,
                               vote: Vote) -> Result<(), GovernanceError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if !proposal.proposal_type.requires_token_vote() {
            return Err(GovernanceError::InvalidStatus);
        }
        if proposal.validator_votes.iter().any(|(a, _)| a == &validator) {
            return Err(GovernanceError::AlreadyVoted);
        }

        proposal.validator_votes.push((validator, vote));
        if proposal.is_fully_approved() {
            proposal.status = ProposalStatus::Approved;
        }
        Ok(())
    }

    pub fn get_proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.id == id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const FOUNDER: Address = [1u8; 32];
    const CHIEF:   Address = [2u8; 32];
    const SEAT3:   Address = [3u8; 32];
    const SEAT4:   Address = [4u8; 32];
    const SEAT5:   Address = [5u8; 32];
    const SEAT6:   Address = [6u8; 32];
    const SEAT7:   Address = [7u8; 32];
    const OUTSIDER:Address = [9u8; 32];

    fn setup() -> Governance {
        let mut g = Governance::new(FOUNDER, CHIEF);
        g.assign_auto_seat(3, SEAT3, 1000);
        g.assign_auto_seat(4, SEAT4, 1000);
        g.seats[4].holder = Some(SEAT5);
        g.seats[5].holder = Some(SEAT6);
        g.seats[6].holder = Some(SEAT7);
        g
    }

    fn grant() -> ProposalType {
        ProposalType::GrantApproval {
            grantee: OUTSIDER, amount_qtc: 250_000,
            milestone: "M17".into(),
        }
    }

    #[test]
    fn test_immutable_total_supply_rejected() {
        let mut g = setup();
        let r = g.submit_proposal(OUTSIDER, ProposalType::GovernanceChange {
            rule_changed: "total_supply".into(), new_value: "2B".into(),
        }, "".into(), 1000, 10);
        assert!(matches!(r, Err(GovernanceError::TouchesImmutableRule(ImmutableRule::TotalSupply))));
    }

    #[test]
    fn test_immutable_founder_vesting_rejected() {
        let mut g = setup();
        let r = g.submit_proposal(OUTSIDER, ProposalType::GovernanceChange {
            rule_changed: "founder_vesting".into(), new_value: "no cliff".into(),
        }, "".into(), 1000, 10);
        assert!(matches!(r, Err(GovernanceError::TouchesImmutableRule(ImmutableRule::FounderVesting))));
    }

    #[test]
    fn test_immutable_multisig_threshold_rejected() {
        let mut g = setup();
        let r = g.submit_proposal(OUTSIDER, ProposalType::GovernanceChange {
            rule_changed: "multisig_threshold".into(), new_value: "3/7".into(),
        }, "".into(), 1000, 10);
        assert!(matches!(r, Err(GovernanceError::TouchesImmutableRule(ImmutableRule::MultisigThreshold))));
    }

    #[test]
    fn test_vote_before_review_period_fails() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, grant(), "".into(), 1000, 10).unwrap();
        let r = g.cast_multisig_vote(FOUNDER, id, Vote::Yes, 1001);
        assert!(matches!(r, Err(GovernanceError::ReviewPeriodNotOver)));
    }

    #[test]
    fn test_recusal_proposer_cannot_vote() {
        let mut g = setup();
        let id = g.submit_proposal(FOUNDER, grant(), "".into(), 1000, 10).unwrap();
        let after = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        let r = g.cast_multisig_vote(FOUNDER, id, Vote::Yes, after);
        assert!(matches!(r, Err(GovernanceError::RecusedFromOwnProposal)));
    }

    #[test]
    fn test_5_of_7_approves() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, grant(), "".into(), 1000, 10).unwrap();
        let after = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        for voter in [FOUNDER, CHIEF, SEAT3, SEAT4, SEAT5] {
            g.cast_multisig_vote(voter, id, Vote::Yes, after).unwrap();
        }
        assert_eq!(g.get_proposal(id).unwrap().status, ProposalStatus::Approved);
    }

    #[test]
    fn test_4_of_7_not_enough() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, grant(), "".into(), 1000, 10).unwrap();
        let after = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        for voter in [FOUNDER, CHIEF, SEAT3, SEAT4] {
            g.cast_multisig_vote(voter, id, Vote::Yes, after).unwrap();
        }
        assert_eq!(g.get_proposal(id).unwrap().status, ProposalStatus::Voting);
    }

    #[test]
    fn test_cannot_vote_twice() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, grant(), "".into(), 1000, 10).unwrap();
        let after = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        g.cast_multisig_vote(FOUNDER, id, Vote::Yes, after).unwrap();
        let r = g.cast_multisig_vote(FOUNDER, id, Vote::No, after);
        assert!(matches!(r, Err(GovernanceError::AlreadyVoted)));
    }

    #[test]
    fn test_outsider_cannot_vote() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, grant(), "".into(), 1000, 10).unwrap();
        let after = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        let r = g.cast_multisig_vote([99u8;32], id, Vote::Yes, after);
        assert!(matches!(r, Err(GovernanceError::NotAMultisigHolder)));
    }

    #[test]
    fn test_proposer_cooldown() {
        let mut g = setup();
        g.submit_proposal(OUTSIDER, grant(), "first".into(), 1000, 10).unwrap();
        let r = g.submit_proposal(OUTSIDER, grant(), "second".into(), 1001, 10);
        assert!(matches!(r, Err(GovernanceError::ProposerCooldownActive { .. })));
        let after = 1000 + PROPOSER_COOLDOWN_BLOCKS + 1;
        assert!(g.submit_proposal(OUTSIDER, grant(), "third".into(), after, 10).is_ok());
    }

    #[test]
    fn test_governance_change_needs_14_day_review() {
        let mut g = setup();
        let id = g.submit_proposal(OUTSIDER, ProposalType::GovernanceChange {
            rule_changed: "quorum_pct".into(), new_value: "60".into(),
        }, "".into(), 1000, 10).unwrap();
        let after_7 = 1000 + PROPOSAL_REVIEW_BLOCKS + 1;
        let r = g.cast_multisig_vote(CHIEF, id, Vote::Yes, after_7);
        assert!(matches!(r, Err(GovernanceError::ReviewPeriodNotOver)));
        let after_14 = 1000 + GOVERNANCE_REVIEW_BLOCKS + 1;
        assert!(g.cast_multisig_vote(CHIEF, id, Vote::Yes, after_14).is_ok());
    }
}
