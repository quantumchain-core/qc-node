# QTC Governance

**Version:** 1.0
**Date:** June 2026
**Multisig threshold:** 5 of 7 signatures required for all actions

---

## Governance Philosophy

QTC governance is designed around one principle: **code ships, tokens vest, community decides — in that order.** No one gets paid before delivering. No one controls the chain alone. No vote can change the rules that protect everyone.

---

## The 5/7 Multisig

All Foundation actions (grants, treasury deployments, protocol upgrades) require 5 of 7 multisig signatures.

### Seat Assignments

| Seat | Holder | How Assigned | Removable? |
|---|---|---|---|
| 1 | Touqeer Ahmad (Founder) | Permanent | No |
| 2 | Touqeer Ahmad (Chief Architect) | Permanent | No |
| 3 | Highest staked validator | Auto: most QTC staked (30d snapshot) | Rotates monthly |
| 4 | Top contributing validator | Auto: 50% blocks produced + 50% uptime (30d) | Rotates monthly |
| 5 | Community elected | Annual token holder vote | Yes, annual |
| 6 | Community elected | Annual token holder vote | Yes, annual |
| 7 | Foundation reserve | DAO vote to fill after mainnet | Yes, by DAO vote |

### Seat 3 & 4 Auto-Assignment Rules

**Seat 3 — Highest Staked:**
- Snapshot taken on the 1st of each month at block closest to 00:00 UTC
- Validator with most QTC bonded to their registry address wins the seat
- Minimum stake to be eligible: 10,000 QTC
- Tie-break: earlier genesis registration date wins

**Seat 4 — Top Contributor:**
- Score = (blocks_produced_30d / total_blocks_30d) * 50 + (uptime_30d%) * 50
- Snapshot taken same time as Seat 3
- Minimum score to be eligible: 40/100
- Tie-break: highest uptime wins

**Seat rotation:** New holders announced on-chain 7 days before taking effect. Outgoing holder's pending signatures on open proposals remain valid for 48 hours after rotation.

---

## What Requires a Vote

### Requires 5/7 Multisig Only (no token holder vote)
- Developer grant approval (≤ 1,000,000 QTC from `0xDAO`)
- Team hire approval (≤ 5,000,000 QTC from `0xTeamVesting`)
- Emergency protocol patch (critical security fix, < 24hr window)
- Monthly community emissions distribution approval

### Requires 5/7 Multisig + Token Holder Vote (51% quorum)
- Strategic Reserve deployment > 1,000,000 QTC
- Protocol upgrade (non-emergency)
- New genesis validator batch approval
- Annual election for Seats 5 & 6
- Filling Seat 7
- Any change to governance rules (except immutables)

### Cannot Be Voted On (Immutable)
- Total supply (1,000,000,000 QTC, fixed)
- Founder vesting schedule (Seat 1, 150M QTC, 12mo cliff, 4yr linear)
- Foundation Grant #001 terms (Seat 2)
- 5/7 threshold — cannot be lowered
- MIT license requirement on all funded code
- Validator registry format (breaking change to consensus)

---

## Proposal Process

### Standard Proposal (requires token holder vote)

```
Day 0:   Proposer submits PR to docs/grants/GRANT_PROPOSALS.md
         PR must include: title, description, amount, timeline,
         "done when" criteria, proposer wallet address
Day 1-7: Public comment period (GitHub Discussions)
Day 8:   5/7 multisig vote opens (48hr window)
Day 10:  If 5/7 signed AND 51% validator quorum reached → approved
Day 11+: Implementation begins. Payment gated on delivery.
```

### Emergency Proposal (critical security fix only)

```
Hour 0:  Any multisig holder declares emergency via signed message
Hour 1:  All 7 holders notified via registered contact
Hour 24: 5/7 signatures collected → patch deployed
Hour 48: Public post-mortem published in docs/
```

Emergency proposals cannot release tokens — only deploy code fixes.

### Grant Proposal Recusal Rule

**A multisig holder cannot vote on any proposal that benefits them directly.** If a holder is the proposer or a named beneficiary, their seat is recused for that vote. The threshold adjusts: if 1 seat is recused, approval requires 5/6 of remaining seats. If 2 seats recused, 5/5 (unanimous).

---

## Validator Voting

All validators registered in the `ValidatorRegistry` at the time of a snapshot can vote on governance proposals.

**Voting weight:** 1 validator = 1 vote. Token-weighted voting is NOT used for validator governance (prevents whale dominance). Token-weighted voting IS used for electing Seats 5 & 6 (token holders, not just validators).

**Quorum:** 51% of active validators (defined as: produced at least 1 block in the last 30 days).

**Voting period:** 48 hours for standard proposals. Results are final when quorum is reached or voting period ends.

**Abstain option:** A validator can explicitly abstain. Abstentions count toward quorum but not toward approval.

---

## Anti-Collusion Rules

1. **No proposal can be submitted by the same address more than once per 30 days** (prevents spam governance attacks).
2. **Whale cap:** In token-weighted elections (Seats 5 & 6), no single address can cast more than 1% of total supply worth of votes (~10M QTC). Excess is ignored.
3. **Cooling period:** Any validator who receives a grant cannot vote on Foundation grants for 90 days after receipt.
4. **On-chain transparency:** Every vote, every signature, every proposal is recorded on-chain and publicly readable.

---

## What Happens If Someone Tries to Rob the Treasury

**Scenario: A dev proposes to grant themselves 50M QTC**

- Proposal goes to public GitHub PR — immediately visible to everyone
- 7-day comment period — community flags it
- Requires 5/7 multisig — Seats 1 & 2 (Founder + First Dev) vote no
- Even if somehow 5/7 sign → requires 51% validator quorum → validators reject
- Even if all that fails → Strategic Reserve is the only wallet involved, not vesting wallets
- Vesting contracts are time-locked in code — no signature can unlock early

**Scenario: A dev gets a grant, delivers nothing, keeps tokens**

- Grants are milestone-gated: 250K QTC per sub-milestone
- Payment only releases after code is merged to main
- 5/7 multisig must sign the release
- Clawback: unvested portion returns to `0xDAO` if 2x timeline exceeded
- MIT license means community can fork and continue without them

**Scenario: Seat 3 or 4 holder (auto-assigned validator) tries to collude**

- They rotate every 30 days — limited window to cause damage
- 5/7 required — they are 1 of 7, cannot pass anything alone
- Seats 1 & 2 are permanent and can block any proposal

---

## Governance Upgrade Path

The governance rules themselves can be upgraded, but only via the full process (5/7 multisig + 51% validator vote) and only for non-immutable rules. Any proposed governance change must:

1. Be proposed in `docs/GOVERNANCE_PROPOSALS.md`
2. Explicitly state which rule is being changed and why
3. Include a 14-day comment period (double the standard 7 days)
4. Receive 5/7 multisig + 66% validator quorum (higher bar than standard 51%)

Immutable rules cannot be changed. Full stop.

---

## Architecture Decision Record — June 2026

**M14 Vesting + Governance Implementation:**
Native Rust modules in `qc-node` (`src/vesting/mod.rs`, `src/governance/mod.rs`).
No Solidity, no external EVM chain dependency.

**Rationale:**
- $0 cost, no external chain dependency
- Fully post-quantum (no ECDSA keys on Ethereum)
- Same patterns already proven in M1-M10
- Fastest path to mainnet

**EVM Upgrade Path (Option B):**
Full EVM execution layer inside `qc-node` is planned when grant funding
arrives (target: Outlier Ventures Base Camp, a16z crypto, or M16-M19
Foundation grants). EVM upgrade will be backward compatible — existing
native Rust governance continues to work alongside it.

**Decision made:** June 2026
**Decided by:** Touqeer Ahmad (Founder + Chief Architect)
**Recorded here as permanent architectural record.**
---

## Architecture Decision Record — July 2026

**Pure Fair Launch Model Adopted:**
QTC switches from LBP (public token sale) to pure fair launch.
No tokens are sold to the public at any point before or during mainnet.

**Rationale:**
- Zero legal risk — no securities sale without audit/legal opinion
- Testnet and mainnet can launch immediately without $50K audit
- Audit and legal opinion obtained post-launch via grants
- Aligns with Bitcoin's original fair launch philosophy
- MIT licensed, community owned from block 1

**Chief Architect Salary Source:**
Touqeer Ahmad's salary comes from the Ecosystem/Grants pool (17%, 170M QTC)
via standard 5/7 multisig governance vote — not from a public token sale.
USDC salary begins only after grant funding is received.
Pre-grant: QTC compensation at market rate.

**Decision made:** July 2026
**Decided by:** Touqeer Ahmad (Founder + Chief Architect)
**Recorded here as permanent architectural record.**
