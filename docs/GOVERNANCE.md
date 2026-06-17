**Document Hash (SHA256):** c31a535ec6980f645a262303e6a3febbe77486c9092e4ac72944711d6459b2fa
**Version:** 1.0 — June 2026
**Verify:** sha256sum TOKENOMICS.md

# QTC Tokenomics

**Version:** 1.0
**Date:** June 2026
**Status:** Final — immutable after mainnet launch (M15).
**Total Supply:** 1,000,000,000 QTC (fixed forever, no inflation)

---

## Immutable Rules

These cannot be changed by any DAO vote, multisig action, or code update:

1. Total supply is fixed at 1,000,000,000 QTC. No minting ever.
2. Founder allocation (150M QTC, Seat 1) vesting schedule is locked.
3. First Developer allocation (Foundation Grant #001) is locked.
4. MIT license is required on all code funded by Foundation grants.
5. The 5/7 multisig threshold cannot be lowered by governance vote.

---

## Allocation Table

| Pool | % | QTC | Wallet Label | On-Chain Address | Vesting |
|---|---|---|---|---|---|
| Community Emissions | 38% | 380,000,000 | `0xCommunity` | deployed M14 | 10yr linear to validators |
| Ecosystem / Foundation | 15% | 150,000,000 | `0xFoundation` | deployed M14 | 10% TGE, 90% 3yr DAO-controlled |
| Founder | 15% | 150,000,000 | `0xFounderVesting` | deployed M14 | 12mo cliff, 4yr linear |
| Team Future Hires | 4% | 40,000,000 | `0xTeamVesting` | deployed M14 | 12mo cliff, 4yr linear |
| Advisors / Genesis Validators | 3% | 30,000,000 | `0xAdvisors` | deployed M14 | 6mo cliff, 2yr linear |
| Airdrop M13 | 1% | 10,000,000 | `0xAirdrop` | deployed M14 | Unlocked at TGE |
| Liquidity / LBP | 4% | 40,000,000 | `0xLBP` | deployed M14 | 50% TGE, 50% at 6 months |
| Strategic Reserve | 20% | 200,000,000 | `0xDAO` | deployed M14 | DAO vote only, 5/7 multisig |

*Real addresses will be added here immediately after M14 contract deployment.
This table will be updated and the commit hash recorded as the canonical address registry.*

---

## Vesting Detail

### Founder — 150M QTC (`0xFounderVesting`)
- **Recipient:** Touqeer Ahmad (Founder, Multisig Seat 1)
- **Cliff:** 12 months from TGE
- **Vesting:** 4 years linear after cliff
- **Total period:** 5 years from TGE
- **Immutable:** Cannot be modified by any DAO vote
- **Monthly release after cliff:** ~3,125,000 QTC/month

### Foundation Grant #001 — Chief Architect
- **Recipient:** Touqeer Ahmad (First Developer, Multisig Seat 2)
- **Role:** Chief Architect, M16–M20
- **Term:** Jan 2027 – Dec 2028, renewable
- **Base:** $0/mo (optional $10,000 USDC/mo if DAO approves)
- **Milestone grants:** 1,000,000 QTC per major milestone (M16–M19)
- **Vesting:** 6mo linear per milestone grant
- **Source:** `0xFoundation` (Ecosystem pool)
- **Immutable:** No DAO vote required. Signed by 4/7 Foundation Multisig.
- **Total potential:** 4,000,000 QTC across M16–M19

### Team Future Hires — 40M QTC (`0xTeamVesting`)
- **Cliff:** 12 months from hire date (not TGE)
- **Vesting:** 4 years linear after cliff
- **Approval:** 5/7 multisig required for each hire grant
- **Max per hire:** 5,000,000 QTC (larger grants require full DAO vote)
- **Clawback:** Unvested tokens return to `0xDAO` if contributor leaves

### Advisors / Genesis Validators — 30M QTC (`0xAdvisors`)
- **Eligibility:** Validators running a node at mainnet genesis (M15)
- **Cliff:** 6 months from TGE
- **Vesting:** 2 years linear after cliff
- **Allocation per genesis validator:** Equal share of remaining pool after genesis developer allocation
- **Max genesis validators:** 100 (first come, first verified)

#### Genesis Developer Allocation (reserved, no vote required)
- **Recipient:** Touqeer Ahmad (Chief Architect, genesis validator by right)
- **Amount:** 500,000 QTC
- **Source:** `0xAdvisors` pool
- **Cliff:** 6 months from TGE
- **Vesting:** 2 years linear (~20,833 QTC/month after cliff)
- **Basis:** Genesis right — architect of the protocol is by definition a genesis validator
- **Approval:** No DAO vote required. Documented here as immutable pre-allocation.
- **This allocation is independent of Foundation Grant #001 and founder vesting.**

### Community Emissions — 380M QTC (`0xCommunity`)
- **Distribution:** Proportional to validator contribution score
- **Contribution score:** 50% blocks produced + 50% uptime (30-day rolling)
- **Emission rate:** ~38M QTC/year over 10 years
- **Frequency:** Monthly snapshots, on-chain distribution
- **No cliff:** Earned emissions vest immediately

### Airdrop — 10M QTC (`0xAirdrop`)
- **Distribution:** M13 airdrop script (scripts/airdrop.ts)
- **Eligibility:** First 500+ valid claims via faucet + social verification
- **Unlocked:** Immediately at TGE, no vesting
- **Max per address:** 20,000 QTC

### Liquidity / LBP — 40M QTC (`0xLBP`)
- **Purpose:** Initial liquidity bootstrap + price discovery
- **TGE:** 50% (20M QTC) available at launch
- **Remaining:** 50% (20M QTC) at 6 months post-TGE
- **Control:** 5/7 multisig post-mainnet

#### LBP USDC Raise Split — Pre-Launch Public Disclosure

**This disclosure is mandatory reading before participating in the LBP.**

Expected raise: ~$500,000 USDC. Split as follows:

| Allocation | % | Amount | Purpose |
|---|---|---|---|
| DEX Liquidity | 40% | $200,000 | Locked 12mo in DEX LP |
| Foundation Ops Fund | 60% | $300,000 | Chain survival budget |

**Why 60% ops:** A chain with $0 ops cannot pay for its audit, legal
opinion, CEX listing, or developer salary. It dies before block 100.
Every successful L1 (Solana, Aptos, Sui) used a similar split.
The difference between ops spending and a rug is this disclosure.

#### Foundation Ops Fund — $300,000 USDC

| Item | Amount | Type |
|---|---|---|
| Security audit (OtterSec/Halborn) | $50,000 | One-time |
| CEX listing (tier 2) | $100,000 | One-time |
| Legal opinion letter | $15,000 | One-time |
| Bug bounty reserve | $20,000 | One-time |
| Chief Architect salary (32mo × $3,000) | $96,000 | Monthly |
| RPC/infrastructure (32mo × $500) | $16,000 | Monthly |
| Emergency buffer | $3,000 | Reserve |
| **Total** | **$300,000** | |

#### Chief Architect Compensation Schedule

| QTC Price (30d TWAP) | Monthly Salary |
|---|---|
| $0.00 — $0.05 | $3,000 USDC |
| $0.05 — $0.20 | $3,500 USDC |
| $0.20+ | $5,000 USDC |

- **Recipient:** Touqeer Ahmad (Chief Architect)
- **Start:** Month 1 after LBP closes
- **Cap: Maximum 24 months or $120,000 USDC total, whichever comes first.**
  After cap is reached, DAO votes new rate via standard governance proposal.
  This prevents indefinite self-payment regardless of QTC price.
- **Separate from:** Founder vesting (150M QTC) and Genesis allocation (500K QTC)
- **Runway at $3,000/mo:** 32 months (ops fund covers beyond the 24mo cap)

#### Pre-Mainnet Ops Fund Control — 7-Day Public Timelock

5/7 multisig cannot exist before mainnet (no validators = no Seats 3-7).
All pre-mainnet ops spending is governed by `TimelockedOpsFund` (built in M14):

1. Founder proposes spend publicly (GitHub + on-chain)
2. 7-day objection window — anyone can object publicly
3. No valid objection → spend executes automatically
4. Objection raised → spend frozen pending community review
5. **Control auto-transfers to 5/7 multisig at block 100,000
   (~14 days post-genesis at 2s block time). This is hardcoded
   in `TimelockedOpsFund` — no action required, no vote needed,
   no founder can delay it.**

Quarterly spending reports published to `docs/ops-reports/`.
Any underspend at 12 months goes to DEX LP.

### Strategic Reserve — 200M QTC (`0xDAO`)
- **Purpose:** Future development, partnerships, emergency fund
- **Access:** DAO vote only, 5/7 multisig required
- **Proposal requirements:** 7-day public review + 51% validator quorum
- **Cannot be used for:** Increasing any existing allocation, bypassing vesting

---

## Anti-Rug Protections

1. **No admin keys.** Vesting contracts have no owner function that bypasses time locks.
2. **All wallets labeled and public.** Every allocation wallet is named and trackable on-chain from block 1.
3. **Clawback on unvested grants.** If a grant recipient leaves before vesting completes, unvested tokens return to `0xDAO`.
4. **Milestone gating.** All developer grants (M16+) are paid per milestone, not upfront. 250K QTC per sub-milestone, released by 5/7 multisig after delivery.
5. **Public grant agreements.** All grants are documented in `docs/grants/` and linked on-chain. No private deals.
6. **Immutable supply.** The smart contract `totalSupply()` is set at deployment and has no mint function.
7. **DAO cannot vote to change immutables.** The governance contract explicitly rejects proposals that would modify the items listed in the Immutable Rules section above.

---

## Developer Grant Rules (M16+ Contributors)

Any developer applying for a grant from `0xDAO` must:

1. Submit a proposal to `docs/grants/GRANT_PROPOSALS.md` via GitHub PR
2. Proposal must include: milestone definition, "done when" criteria, timeline, QTC amount requested
3. 7-day public comment period (GitHub discussion)
4. 5/7 multisig vote to approve
5. Code must be MIT licensed and merged to main before payment releases
6. Max 250,000 QTC per sub-milestone, 1,000,000 QTC per major milestone
7. 6-month linear vesting on all milestone grants (no cliff)
8. Clawback: if milestone is not delivered within 2x the proposed timeline, unvested portion returns to `0xDAO`

**You cannot vote on your own grant proposal.** Multisig seats held by the proposer are recused.

---

## Token Utility

| Use | Description |
|---|---|
| Gas fees | Every transaction pays `gas_limit * base_fee` in nano-QTC |
| Validator rewards | Community emissions (380M QTC) distributed to active validators |
| Governance | Token holders vote on DAO proposals (1 token = 1 vote, capped at 1% per address to prevent whale dominance) |
| Genesis validator bond | Minimum 10,000 QTC staked to register as a validator |

---

## What QTC Is Not

- Not a security (no profit expectation from others' efforts — validators earn by doing work)
- Not inflationary (fixed 1B supply, no mint function)
- Not controlled by one person (5/7 multisig, immutable founder vesting, public governance)

*This document does not constitute financial or legal advice. Token prices are speculative.*

---

## Legal Separation and Liability Protection

### Foundation vs Individual Separation

**The QuantumChain Foundation is a decentralized protocol, not a legal entity registered to any individual.** Touqeer Ahmad participates as:

1. **Founder (Seat 1):** A protocol designer whose contributions are documented publicly on GitHub (262+ commits). His founder allocation vests over 5 years — it is compensation for past and future work, not a directorship.
2. **Chief Architect (Seat 2):** A grant recipient under Foundation Grant #001. His role is technical, not fiduciary. He does not control funds unilaterally — 5/7 multisig is required for any treasury action.
3. **Genesis Validator:** An operator of network infrastructure. Same legal standing as any other validator.

### What Touqeer Ahmad Is NOT Responsible For

The following actions require 5/7 multisig consensus. No single person, including the Founder, can authorize them alone. Therefore, Touqeer Ahmad bears no individual legal liability for:

- Any DAO vote outcome he voted against
- Any grant approved by 5/7 multisig that he did not initiate
- Any validator's actions on the network (block production, tx inclusion)
- Any third-party developer's code submitted under a grant
- Any market price movement of QTC tokens
- Any loss of funds resulting from smart contract bugs in code he did not write
- Any airdrop recipient's use of their tokens after receipt

### What Protects You If Someone Rugs

**Paper trail protection:**
- Every multisig action is on-chain and timestamped
- Every grant is a public GitHub PR with review history
- Every vote is recorded — your vote against a proposal is permanent evidence
- TOKENOMICS.md, GOVERNANCE.md, and GRANT_TEMPLATE.md are git-committed with timestamps showing they were written BEFORE any grants were issued

**Structural protection:**
- You cannot release funds alone (5/7 required)
- Milestone gating means funds only release after code is publicly merged
- Auto-release timelock (Clause 9) means payment releases by contract, not by your discretion
- Clawback is enforced by smart contract, not by you personally

**If a grantee rugs (takes milestone payment without delivering):**
- The clawback clause (Condition 3 in GRANT_TEMPLATE.md) is enforced by `MilestoneEscrow.sol`
- You are not the enforcer — the contract is
- Your liability: zero. You followed the documented process.

**If someone accuses you of blocking payment unfairly:**
- Dispute Resolution (Condition 10) provides an external arbiter
- The Review SLA (Condition 9) auto-releases payment if you don't respond
- Both clauses were written into the template BEFORE any dispute arose — premeditation of fairness, not rug

### Your Personal Liability Shield (Practical Steps)

Before mainnet, do these three things (all free):

1. **Never hold Foundation funds in a wallet you control alone.** All `0xDAO`, `0xFoundation`, `0xCommunity` wallets must be multisig from day one. Your personal wallets (`0xFounderVesting`, genesis allocation) are separate and clearly labeled.

2. **Document every decision in writing.** Every multisig action has a corresponding GitHub issue or PR. "We approved Grant X because milestone Y was delivered — see PR #123" is your protection.

3. **Never promise token prices.** Never say "QTC will be worth $X." Never say "you will make money." Say "QTC is a utility token for network participation." This is the line between a community project and an unregistered securities offering.

*The above is not legal advice. If you plan to raise significant capital or operate in a regulated jurisdiction, consult a lawyer who specializes in crypto/Web3 law before TGE.*
