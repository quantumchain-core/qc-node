**Document Hash (SHA256):** 487781bdad2f640fda1bdd5e4259de33380219eb22c798c60e5b31d523ff80b2
**Version:** 2.0 — July 2026 (Pure Fair Launch)
**Verify:** sha256sum TOKENOMICS.md

# QTC Tokenomics

**Version:** 2.0
**Date:** July 2026
**Status:** Final — immutable after mainnet launch (M15)
**Total Supply:** 1,000,000,000 QTC (fixed forever, no inflation)
**Model:** Pure Fair Launch — no public token sale, no LBP

---

## Why Pure Fair Launch

QTC uses a pure fair launch model. No tokens are sold to the public.
All tokens are earned through work or distributed via community programs.

**Legal basis:** Running open source software and distributing free tokens
is not a securities sale in any jurisdiction. No audit or legal opinion
is required before launch. Professional audit and legal opinion will be
obtained after mainnet using grant funding (Outlier Ventures, Web3
Foundation, Ethereum Foundation ESP).

**Inspired by:** Bitcoin's original fair launch — Satoshi never sold
tokens. Miners earned them by doing work. QTC follows the same model.

---

## Immutable Rules

These cannot be changed by any DAO vote, multisig action, or code update:

1. Total supply is fixed at 1,000,000,000 QTC. No minting ever.
2. Founder allocation (150M QTC) vesting schedule is locked.
3. MIT license is required on all code funded by ecosystem grants.
4. The 5/7 multisig threshold cannot be lowered by governance vote.
5. No public token sale without professional security audit and legal opinion.

---

## Allocation Table

| Pool | % | QTC | Wallet | How Earned/Released |
|---|---|---|---|---|
| Founder | 15% | 150,000,000 | `0xFounderVesting` | 12mo cliff, 4yr linear |
| Validators (PoS) | 30% | 300,000,000 | `0xValidators` | Step-down emissions, 10yr |
| Ecosystem/Grants | 17% | 170,000,000 | `0xEcosystem` | DAO vote, 5/7 multisig |
| Strategic Reserve | 20% | 200,000,000 | `0xDAO` | DAO vote only |
| dApp Developers | 8% | 80,000,000 | `0xDevRewards` | Gas volume proportional |
| Community Airdrop | 5% | 50,000,000 | `0xAirdrop` | Faucet + testnet activity |
| Liquidity Bootstrap | 5% | 50,000,000 | `0xLiquidity` | DEX LP providers |
| **Total** | **100%** | **1,000,000,000** | | |

All wallet addresses: deployed M14. Updated after deployment.

---

## Validator Emissions — Step-Down Curve

Total validator pool: 300,000,000 QTC over 10 years.
Front-loaded to subsidize validators when token price is lowest.

| Year | QTC Emitted | % of Pool | Reasoning |
|---|---|---|---|
| 1 | 50,000,000 | 16.7% | Highest subsidy — price near zero, hardware costs real |
| 2 | 42,000,000 | 14.0% | Network growing, still needs heavy incentive |
| 3 | 36,000,000 | 12.0% | Ecosystem building, price rising |
| 4 | 30,000,000 | 10.0% | Network established |
| 5 | 25,000,000 | 8.3% | Organic usage replacing subsidies |
| 6 | 20,000,000 | 6.7% | Fee revenue supplementing rewards |
| 7 | 18,000,000 | 6.0% | Mature network |
| 8 | 16,000,000 | 5.3% | Long-term security budget |
| 9 | 14,000,000 | 4.7% | Tail emissions |
| 10 | 12,000,000 | 4.0% | Final year |
| **Total** | **263,000,000** | | Remaining 37M to DAO reserve |

**Validator reward distribution:**
- 50% blocks produced (last 30 days)
- 50% uptime percentage (last 30 days)
- Minimum stake: 10,000 QTC to register
- Monthly snapshot, on-chain distribution

---

## Founder Vesting

- **Recipient:** Touqeer Ahmad (Founder, Multisig Seat 1 + 2)
- **Amount:** 150,000,000 QTC
- **Cliff:** 12 months from mainnet genesis
- **Vesting:** 4 years linear after cliff (~3,125,000 QTC/month)
- **Immutable:** Cannot be modified by any DAO vote
- **Income pre-cliff:** Ecosystem pool salary (see below)

---

## Chief Architect Compensation

Touqeer Ahmad receives salary from Ecosystem pool (17%, 170M QTC).
Requires 5/7 multisig approval — standard governance process.
No token sale involved. In-network compensation.

| QTC Price (30d TWAP) | Monthly Salary |
|---|---|
| $0.00 — $0.05 | $3,000 USDC equivalent in QTC |
| $0.05 — $0.20 | $3,500 USDC equivalent in QTC |
| $0.20+ | $5,000 USDC equivalent in QTC |

**Cap:** 24 months or $120,000 total, then DAO votes new rate.
**Note:** USDC salary begins only after professional audit + legal
opinion obtained via grant funding. Pre-grant: QTC compensation only.

---

## Genesis Developer Allocation

- **Recipient:** Touqeer Ahmad (Chief Architect, genesis validator)
- **Amount:** 500,000 QTC from Community Airdrop pool
- **Cliff:** 6 months from mainnet genesis
- **Vesting:** 2 years linear (~20,833 QTC/month after cliff)
- **Basis:** Genesis right — protocol architect is genesis validator
- **No vote required:** Documented pre-allocation

---

## dApp Developer Rewards (8%, 80M QTC)

Developers who build applications on QTC earn proportional rewards
based on the gas volume their applications generate.

- Monthly snapshot of gas usage per application contract
- Developer earns: (their_gas / total_gas) × monthly_pool
- Monthly pool: ~666,666 QTC (80M over 10 years)
- Requires EVM (M20) for smart contract gas tracking
- Pre-EVM: allocated to ecosystem grants via DAO vote

---

## Community Airdrop (5%, 50M QTC)

- 500,000 QTC: Genesis developer allocation (Touqeer Ahmad)
- 10,000,000 QTC: Testnet validators (proportional to uptime)
- 20,000,000 QTC: Early community (faucet claims + social)
- 19,500,000 QTC: Future community programs (DAO vote)

Max per address: 20,000 QTC
No vesting on airdrop tokens — unlocked immediately

---

## Liquidity Bootstrap (5%, 50M QTC)

No LBP. No public token sale.

Instead: reward early liquidity providers on DEX.
When QTC/USDC or QTC/ETH pools form naturally after mainnet,
LP providers earn from this pool proportional to their liquidity share.

This creates organic price discovery without a token sale.

---

## Ecosystem/Grants Pool (17%, 170M QTC)

Primary uses (all require 5/7 multisig):
- Professional security audit (target: post-grant)
- Legal utility token opinion (target: post-grant)
- Chief Architect salary (Touqeer Ahmad)
- Developer grants (M16+ contributors, per GRANT_TEMPLATE.md)
- Partnership allocations
- Bug bounty payouts

Quarterly spending reports: `docs/ops-reports/`

---

## Strategic Reserve (20%, 200M QTC)

DAO vote + 5/7 multisig required for all releases.
Cannot be used without 51% validator quorum.
Primary purpose: emergency fund, future development, exchange listings.

---

## What This Model Is NOT

- Not a token sale (no money exchanged for tokens at launch)
- Not an ICO (no initial coin offering)
- Not a securities offering (no profit expectation sold to public)
- Not inflationary (fixed 1B supply, no mint function)

QTC tokens are earned through network participation, not purchased.
Price discovery happens naturally through DEX trading after mainnet.

---

## Grant Roadmap

Professional audit and legal opinion obtained via grants, not token sale:

| Grant | Amount | Purpose | Apply When |
|---|---|---|---|
| Outlier Ventures Base Camp | $100K USDC | Audit + ops | After testnet |
| Web3 Foundation | $10-50K | PQC research | After testnet |
| Ethereum Foundation ESP | $10-50K | Infrastructure | After testnet |
| Polkadot Treasury | Variable | Interoperability | After mainnet |

---

## Legal Protection

This fair launch model is legally sound because:
1. No tokens are sold — earned only through work
2. MIT licensed software — no proprietary claims
3. Founder compensation is in-network QTC, not cash raised from public
4. All allocations documented publicly before genesis block

*This document does not constitute financial or legal advice.*
*Token prices are speculative. Consult a professional before investing.*

---

## Anti-Rug Protections

1. Founder vesting: 12mo cliff, 4yr linear — code enforced
2. No admin keys that bypass time locks
3. 5/7 multisig for all treasury actions
4. Public grant agreements in docs/grants/
5. Immutable supply — no mint function
6. All wallets labeled and trackable from block 1
7. Quarterly spending reports published publicly
