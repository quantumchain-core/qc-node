# QTC — Project Master Tracker

**Last Updated:** June 2026
**Founder:** Touqeer Ahmad
**Location:** Dunga Bunga, Punjab, Pakistan
**Degree:** BSc Physics (attestation pending — HEC Pakistan)
**Email:** touqeerahmadofficial896@gmail.com
**GitHub:** github.com/quantumchain-core
**Project:** Post-quantum Layer 1 blockchain — CRYSTALS-Dilithium2

---

## Public Good Statement

**QTC is MIT-licensed public infrastructure.**
All code, docs, and audit reports are free forever.
No patents. No proprietary components. No closed source.
Grant funds are used only for: professional security audit + legal
opinion to protect users. No founder profit from any grant.
Founder compensation is fully disclosed in TOKENOMICS.md.
I am not anonymous. I am publicly accountable.

---

## Founder

**Touqeer Ahmad**
BSc Physics — Dunga Bunga, Punjab, Pakistan
Email: touqeerahmadofficial896@gmail.com
GitHub: github.com/quantumchain-core
262+ commits. Solo developer. Zero funding. Zero team.
I am public. MIT license covers liability. No anon team.

---

## Mainnet Proof

**Status:** ⏳ Pending — update after block 1 is mined

Once live, fill in:
- Block 1 hash: `0x...`
- Block 1 timestamp: `...`
- Explorer: `http://[oracle-ip]:3000`
- RPC endpoint: `http://[oracle-ip]:8545`
- RUN_VALIDATOR.md: [link]
- Validators at launch: 1 (founder node)

*Grants fund live networks 10x more than testnets. Mine block 1 first, then apply.*

---

## Quick Status

| Item | Status |
|---|---|
| Core node (M1-M10) | ✅ Complete — 39 tests green |
| Security audit fixes | 🔄 6/10 done |
| qtc-client | ✅ Complete |
| qtc-faucet | ✅ Complete |
| qtc-wallet | ✅ Complete |
| qtc-explorer | ✅ Complete |
| Docs (README, Whitepaper v3, Architecture) | ✅ Complete |
| Tokenomics | ✅ Locked + hashed |
| Governance | ✅ Complete |
| Grant Template | ✅ Complete |
| Mainnet (M15) | ⏳ Pending M13-M14 |
| LBP | ⏳ Pending audit + legal |

---

---

## Daily Tasks — Community Building

**Rule: 3 posts every day without exception. Consistency beats quality.**

### Post 1 — Binance Square (daily)
Topics to rotate:

| Day | Topic |
|---|---|
| Mon | Technical — explain one QTC feature in simple words |
| Tue | Story — your journey, struggles, why you are building |
| Wed | Education — what is post-quantum cryptography? |
| Thu | Progress — what milestone you completed this week |
| Fri | Community — ask a question, start a discussion |
| Sat | News tie-in — connect QTC to something in crypto news |
| Sun | Motivation — why post-quantum matters for everyone |

### Post 2 — Twitter/X (daily)
- Under 280 characters
- Always end with #QTC #PostQuantum #BuildInPublic
- Tag: @ethereum @web3foundation @binance @polkadot
- Template: "Built a post-quantum blockchain with $0 and zero team.
  262 commits. 39 tests green. Block 1 coming soon.
  The quantum threat is real. #QTC #PostQuantum #BuildInPublic"

### Post 3 — Reddit (daily, rotate subreddits)
- r/CryptoCurrency
- r/rust
- r/blockchain
- r/cryptography
- r/Pakistan
- r/programming
- r/cscareerquestions

---

## Content Bank

| # | Title | Platform | Status |
|---|---|---|---|
| 1 | I built a post-quantum blockchain alone with $0 | Binance Square | ✅ Ready |
| 2 | What happens to Bitcoin when quantum computers arrive? | Binance Square | ⏳ Write next |
| 3 | Why I chose Rust to build a blockchain in Pakistan | Binance Square | ⏳ Write next |
| 4 | NIST standardized post-quantum crypto in 2024. Is your chain ready? | Twitter/X | ⏳ Write next |
| 5 | QTC mainnet is live. Here is block 1. | All platforms | ⏳ After block 1 |

---

## Community Tracker

| Platform | Followers | Target | Status |
|---|---|---|---|
| Binance Square | 0 | 1,000 | ⏳ Start today |
| Twitter/X | 0 | 5,000 | ⏳ Start today |
| Reddit karma | 0 | 1,000 | ⏳ Start today |
| GitHub stars | 2 | 100 | ⏳ Growing |
| Discord/Telegram | 0 | 500 | ⏳ Create after 100 followers |

*Update this table weekly every Sunday.*

---

## Strategic Decision — Testnet Before Mainnet (June 2026)

**Decision: Launch QTC Testnet first. Mainnet only after grant-funded
professional audit + legal opinion.**

Why: testnet tokens (tQTC) have zero monetary value by design, which means:
- Zero legal risk -- cannot be classified as a security
- No audit/legal opinion required before launch
- $0 cost, launches immediately on Oracle Cloud free tier
- Real multi-node testing before any money is at stake
- Strengthens grant applications ("live testnet with N validators"
  is far more fundable than "we plan to launch")

```
NOW        Testnet launch (replaces "M13 mainnet prep" framing)
Week 1-2   Run testnet, invite community, find + fix bugs
Week 3-4   M14 -- vesting/governance, tested on testnet first
Month 2    Apply for grants citing live testnet + community validators
Month 3    Professional audit (funded by grant)
Month 4    Mainnet launch (M15) -- real money, real LBP, audit complete
```

**Chain ID separation:** testnet and mainnet use different chain_id
values so wallets/tools never confuse them. Selected via QC_NETWORK
env var (`testnet` default, `mainnet` explicit).

---

## Financial Reality

| Item | Status |
|---|---|
| Current USDC | $0 |
| Current QTC | 0 (pre-mainnet) |
| Founder vesting | 150M QTC (locked 12mo cliff post-TGE) |
| Genesis allocation | 500K QTC (6mo cliff, 2yr linear) |
| Grant #001 | 1M QTC per milestone M16-M19 |
| Monthly salary (post-LBP) | $3,000 USDC (capped 24mo/$120K) |
| LBP target | $500K USDC (40% LP, 60% ops) |
| Ops fund | $300K USDC (after LBP) |
| Runway at $3K/mo | 32 months post-LBP |

---

## The Circular Problem + Solution

```
PROBLEM:
Need LBP to get money
Need audit + legal for LBP
Need money for audit + legal
= Circular dependency

SOLUTION (decided June 2026):
Step 1: Launch mainnet with ZERO public token sale (legal, free)
Step 2: Airdrop only — no sale = no securities risk
Step 3: Apply for grants with live mainnet as proof
Step 4: Use grant money for audit + legal opinion
Step 5: THEN do LBP safely
```

---

## Road to Mainnet — $0 Path

```
CURRENT: Finishing security audit fixes
  ↓
WEEK 1-2: Finish audit fixes 7-10 (free)
  ↓
WEEK 2-3: M13 — airdrop script + RUN_VALIDATOR.md + TOKENOMICS.md (free)
  ↓
WEEK 3-4: M14 — native Rust vesting + governance (free, NO Solidity)
  ↓
WEEK 4-5: M15 — mainnet_genesis.json + launch.sh (free)
  ↓
WEEK 5-6: Mine block 1 on Oracle Cloud Always Free (free forever)
  ↓
WEEK 6-7: Publish SECURITY_AUDIT.md + RUN_VALIDATOR.md publicly
  ↓
WEEK 7-8: Apply for grants (Web3 Foundation, ETH Foundation, Polkadot)
  ↓
MONTH 3:  Grant decision — use for professional audit + legal opinion
  ↓
MONTH 4:  LBP with audit + legal — THEN raise $500K USDC
```

---

## Road to LBP — After Mainnet

**Requirements before LBP (non-negotiable):**
- [ ] Professional security audit (OtterSec/Halborn/Sec3)
- [ ] Legal comfort letter (utility token opinion)
- [ ] "Not available to US residents" disclaimer on LBP page
- [ ] Mainnet live with real validators
- [ ] SECURITY_AUDIT.md published

**LBP Split (locked in TOKENOMICS.md):**
- 40% ($200K) → DEX LP, locked 12 months
- 60% ($300K) → Foundation Ops Fund

**Ops Fund Budget:**
- $50K — Security audit
- $100K — CEX listing (tier 2)
- $15K — Legal opinion
- $20K — Bug bounty reserve
- $96K — Founder salary (32mo × $3K)
- $16K — RPC/infrastructure
- $3K — Emergency buffer

---

## Legal Protection Status

| Protection | Status | Notes |
|---|---|---|
| MIT license on all code | ✅ Done | No warranty clause protects founder |
| Founder vesting in contract | ⏳ M14 | TimelockedOpsFund |
| "Not available to US residents" | ⏳ Add to LBP page | Before any token sale |
| Legal comfort letter | ⏳ Needs $1-2K | After grant received |
| No promises of profit anywhere | ✅ Policy | Never say QTC will be $X |
| All decisions documented publicly | ✅ Done | GitHub paper trail |
| 5/7 multisig (post-mainnet) | ⏳ M14 | Cannot be sole controller |
| TimelockedOpsFund contract | ⏳ M14 | 7-day public spend window |

**What cannot happen to you:**
- DAO cannot vote to take your 150M QTC — it's time-locked in code
- DAO cannot vote to change immutable rules — contract rejects it
- Dev cannot get paid without delivering — milestone gated
- You cannot be blamed for other validators' actions — MIT license
- You cannot be blamed for market price — never promised profit

---

## Security Audit Status

**Self-audit completed June 2026. No critical findings.**

| # | Finding | Severity | Status |
|---|---|---|---|
| AUDIT-001 | sign() panics on invalid sk | LOW | ⏳ Fix at M15 |
| AUDIT-002 | sk not zeroized on drop | INFO | ⏳ Fix at M15 |
| AUDIT-003 | No validator count limit | MEDIUM | ⏳ Fix 7 |
| AUDIT-004 | insert() overwrites silently | LOW | ⏳ Fix 7 |
| AUDIT-005 | address_from_pubkey no size check | INFO | ⏳ Acceptable |
| AUDIT-006 | Error leaks proposer address | INFO | ✅ Acceptable |
| AUDIT-007 | Gas overflow (checked_add) | HIGH | ✅ Fixed |
| AUDIT-008 | Coinbase==sender corruption | MEDIUM | ✅ Fixed |
| AUDIT-009 | tx.value u64 precision | LOW | ✅ Documented |
| AUDIT-010 | Zero accounts not pruned | MEDIUM | ⏳ Fix 5 |
| AUDIT-011 | StateDB grows unboundedly | LOW | ⏳ M17 |
| AUDIT-012 | No integrity check on storage | MEDIUM | ⏳ Fix 6 |
| AUDIT-013 | sled path unsanitized | LOW | ✅ Acceptable |
| AUDIT-014 | Double-lock in on_block | HIGH | ✅ Safe, documented |
| AUDIT-015 | Block number not validated | MEDIUM | ✅ Fixed |
| AUDIT-016 | Timestamp not validated | MEDIUM | ✅ Fixed |
| AUDIT-017 | bootstrap() silent failure | LOW | ⏳ Fix 6 |
| AUDIT-018 | tx.hash not verified | HIGH | ✅ Fixed |
| AUDIT-019 | No RPC rate limiting | MEDIUM | ⏳ Fix 8 |
| AUDIT-020 | ERR_PARSE unused | LOW | ✅ Fixed |
| AUDIT-021 | ERR_INTERNAL unused | LOW | ✅ Fixed |
| AUDIT-022 | Keypair regenerated on restart | HIGH | ✅ Fixed |
| AUDIT-023 | Coinbase hardcoded [9u8;32] | MEDIUM | ✅ Fixed |
| AUDIT-024 | Gossip logs internal state | LOW | ⏳ M16+ |

---

## Completed Milestones

| M | Name | Repo | Tests | Tag |
|---|---|---|---|---|
| M1 | Post-quantum crypto (Dilithium2) | qc-node | 3 | v0.1.0 |
| M2 | libp2p swarm | qc-node | 2 | v0.2.0 |
| M3 | Chain types + genesis_block() | qc-node | 3 | v0.3.0 |
| M4 | Mempool (EIP-1559) | qc-node | 9 | v0.4.0 |
| M5 | Consensus engine | qc-node | 3 | v0.5.0 |
| M6 | State + sled storage | qc-node | 4 | v0.6.0 |
| M7 | Gossip handler | qc-node | 7 | v0.7.0 |
| M8 | JSON-RPC API (axum) | qc-node | 12 | v0.8.0 |
| M9 | Live event loop | qc-node | 6 | v0.9.0 |
| M10 | Validator registry | qc-node | 8 | v1.0.0 |
| M11.1 | TypeScript RPC client | qtc-client | — | v0.1.0 |
| M11.2 | Cloudflare faucet | qtc-faucet | — | v0.1.0 |
| M11.3 | Tauri desktop wallet | qtc-wallet | — | v0.1.0 |
| M12 | Next.js block explorer | qtc-explorer | — | v0.1.0 |

---

## Pending Milestones

| M | Name | Repo | Cost | ETA |
|---|---|---|---|---|
| Audit fixes 7-10 | Security hardening | qc-node | $0 | This week |
| M13 | Airdrop + docs | qtc-mainnet | $0 | Week 2 |
| M14 | Native Rust vesting + governance | qc-node | $0 | Week 3 |
| M15 | Mainnet genesis + launch | qtc-mainnet | $0 | Week 4 |
| M17 | State pruning + snapshots | qc-node | $0 | After mainnet |
| M19 | Sharding V1 | qc-node | $0 | After grant |
| M18 | PoUW app-chain | qc-node | $25 (Android) | After grant |
| M16 | Light client + ZK bridge | qc-node | Needs grant | After grant |
| M20 | Governance V2 + EVM | qc-node | Needs grant | After grant |

---

## Grant Applications — To Submit

**Apply immediately after block 1 is mined:**

| Grant | URL | Amount | Deadline | Status |
|---|---|---|---|---|
| Outlier Ventures Base Camp | outlierventures.io/base-camp | $100K USDC | Rolling | ⏳ Apply now |
| Web3 Foundation | grants.web3.foundation | $10-50K | Rolling | ⏳ Apply after mainnet |
| Ethereum Foundation ESP | esp.ethereum.foundation | $10-50K | Rolling | ⏳ Apply after mainnet |
| Polkadot Treasury | polkadot.network/treasury | Variable | Rolling | ⏳ Apply after mainnet |

**Your application pitch (copy-paste ready):**

> **Public Good Statement:** QTC is MIT-licensed public infrastructure.
> All code, docs, and audit reports are free forever. Grant funds are used
> only for a professional security audit + legal opinion to protect users.
> No founder profit from this grant.
>
> **Project:** QTC is the first production-ready post-quantum Layer 1
> blockchain. Built by a solo developer (Touqeer Ahmad, BSc Physics,
> Pakistan) with zero funding using CRYSTALS-Dilithium2 (NIST FIPS 204).
> Mainnet is live with [X] validators. 39 tests passing, CI green,
> full self-audit published.
>
> **Ask:** $50,000 USD for a professional security audit (OtterSec/Halborn)
> before our public token launch. All findings will be published publicly.
>
> **Founder:** Touqeer Ahmad — touqeerahmadofficial896@gmail.com
> GitHub: github.com/quantumchain-core — 262+ commits, public, not anon.
>
> **Why this is a public good:** Post-quantum cryptography protects every
> blockchain user from the coming quantum threat. QTC is MIT licensed —
> any chain can adopt our implementation. We are infrastructure, not a startup.

---

## Infrastructure Plan — $0

| Service | Provider | Cost | Purpose |
|---|---|---|---|
| qc-node 24/7 | Oracle Cloud Always Free | $0 | Run mainnet node |
| qtc-explorer | Vercel free tier | $0 | Block explorer |
| qtc-faucet | Cloudflare Workers free | $0 | Token faucet |
| Docs | GitHub Pages | $0 | Documentation |
| Domain | None at launch | $0 | Use Oracle IP |
| CI/CD | GitHub Actions free | $0 | Test + deploy |

---

## Tokenomics Summary

**Total supply: 1,000,000,000 QTC (fixed forever)**

| Pool | QTC | % | Vesting |
|---|---|---|---|
| Community Emissions | 380M | 38% | 10yr to validators |
| Ecosystem/Foundation | 150M | 15% | 10% TGE, 90% 3yr |
| Founder (Touqeer Ahmad) | 150M | 15% | 12mo cliff, 4yr |
| Team Future Hires | 40M | 4% | 12mo cliff, 4yr |
| Advisors/Genesis | 30M | 3% | 6mo cliff, 2yr |
| Airdrop M13 | 10M | 1% | Unlocked at TGE |
| Liquidity/LBP | 40M | 4% | 50% TGE, 50% 6mo |
| Strategic Reserve | 200M | 20% | DAO vote only |

**Genesis Developer Allocation:** 500K QTC from Advisors pool
**Salary:** $3,000/mo USDC (capped 24mo/$120K, then DAO votes)
**TOKENOMICS.md hash:** c31a535ec6980f645a262303e6a3febbe77486c9092e4ac72944711d6459b2fa

---

## Governance Summary

**Multisig: 5/7 required**

| Seat | Holder | Type |
|---|---|---|
| 1 | Touqeer Ahmad | Permanent |
| 2 | Touqeer Ahmad (Chief Architect) | Permanent |
| 3 | Highest staked validator | Auto monthly |
| 4 | Top contributor (50% blocks + 50% uptime) | Auto monthly |
| 5 | Community elected | Annual vote |
| 6 | Community elected | Annual vote |
| 7 | Foundation reserve | DAO vote |

**Pre-mainnet control:** 7-day public timelock (no multisig possible without validators)
**Multisig activates:** Block 100,000 (~14 days post-genesis)

---

## Key Decisions Locked

| Decision | Where | Date |
|---|---|---|
| Dilithium2 not Dilithium3 | ARCHITECTURE.md | M1 |
| M7 gossip-only scope | ARCHITECTURE.md | M7 |
| Native Rust governance (not Solidity) | GOVERNANCE.md | June 2026 |
| EVM upgrade only if grant received | GOVERNANCE.md | June 2026 |
| 5/7 multisig threshold | GOVERNANCE.md | June 2026 |
| 40% LP / 60% ops LBP split | TOKENOMICS.md | June 2026 |
| $3K/mo salary, 24mo cap | TOKENOMICS.md | June 2026 |
| Block 100K multisig transfer | TOKENOMICS.md | June 2026 |
| Mainnet before LBP | PROJECT_TRACKER.md | June 2026 |
| No LBP without audit + legal | PROJECT_TRACKER.md | June 2026 |

---

## Repository Map

| Repo | Purpose | Status |
|---|---|---|
| qc-node | Core Rust blockchain node | ✅ Active |
| qtc-client | TypeScript RPC client | ✅ Complete |
| qtc-faucet | Cloudflare Worker faucet | ✅ Complete |
| qtc-wallet | Tauri desktop wallet | ✅ Complete |
| qtc-explorer | Next.js block explorer | ✅ Complete |
| qtc-mainnet | Genesis config + scripts | ⏳ M13-M15 |
| qtc-dao | Vesting + governance UI | ⏳ M14 |

---

## What Cannot Happen (Legal Protections)

1. Nobody can take your 150M QTC — time-locked in contract
2. Nobody can change immutable rules — contract rejects it
3. Nobody can get paid without delivering — milestone gated
4. You cannot be blamed for market price — never promised profit
5. You cannot be blamed for other validators' actions — MIT license
6. Nobody can spend ops fund silently — 7-day public timelock
7. Nobody can pass a proposal alone — 5/7 required
8. You cannot be outvoted — Seats 1+2 are permanent

---

## How to Update This File

Add a line to the relevant section when:
- A milestone completes → update Completed Milestones table
- A security fix ships → update Security Audit Status table
- A decision is made → add to Key Decisions Locked table
- A grant is applied for → update Grant Applications table
- Financial situation changes → update Financial Reality section
- A new repo is created → update Repository Map table

**Always add the date when updating.**
