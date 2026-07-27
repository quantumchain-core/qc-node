# QTC — Project Master Tracker

**Last Updated:** July 22, 2026
**Founder:** Touqeer Ahmad | touqeerahmadofficial896@gmail.com
**GitHub:** github.com/quantumchain-core
**Location:** Dunga Bunga, Punjab, Pakistan

---

## Public Good Statement

QTC is MIT-licensed public infrastructure. All code, docs, and audit
reports are free forever. No patents. No proprietary components.
Grant funds used only for security audit and legal opinion.
No founder profit from grants. Not anonymous. Publicly accountable.

---

## 🟢 TESTNET LIVE

| Item | Detail |
|---|---|
| Block 1 mined | July 20, 2026 |
| Blocks produced | 3,677+ and counting |
| Chain ID | 0x74 (testnet) |
| Validators | 1 (founder node, moving to Oracle Cloud) |
| Node running | GitHub Codespaces → Oracle Cloud this week |

---

## Completed Milestones

| M | Name | Status |
|---|---|---|
| M1 | Dilithium2 cryptography | ✅ |
| M2/M7 | libp2p + gossip | ✅ |
| M3 | Chain types + genesis | ✅ |
| M4 | EIP-1559 mempool | ✅ |
| M5/M10 | Consensus + validator registry | ✅ |
| M6 | State + sled storage | ✅ |
| M8 | JSON-RPC API | ✅ |
| M9 | Event loop | ✅ |
| M11 | TypeScript client + faucet + wallet | ✅ |
| M12 | Block explorer | ✅ |
| M13 | Testnet config + airdrop script | ✅ |
| M14 | Native Rust vesting + governance | ✅ |
| M15 | Mainnet genesis + launch script | ✅ |
| Audit | 24 findings, all fixed | ✅ |
| Testnet | Block 1 mined July 20, 2026 | ✅ |

---

## Key Decisions

| Decision | Detail | Date |
|---|---|---|
| Pure fair launch | No LBP, no token sale | July 2026 |
| Performance-based rewards | Earn only while running | July 2026 |
| No time-locked rewards | Run=earn, stop=stop | July 2026 |
| tQTC→QTC ratio | DAO vote before mainnet | July 2026 |
| Multisig | Add when validators join, no rush | July 2026 |
| Build everything on testnet first | Test before mainnet | July 2026 |
| Step-down emissions | 50M yr1 → 12M yr10 | July 2026 |

---

## Validator Reward Model

```
Monthly Reward =
  (your_blocks / total_blocks) × 60% of monthly pool
+ (your_uptime%) × 40% of monthly pool

Year 1: 50M QTC/year = ~4,166,666/month
```

Not running = $0. Best uptime = best share.

---

## Testnet Phases

```
Phase 1 (DONE):   Single validator — 3,677+ blocks
Phase 2:          2-5 real validators, test rotation
Phase 3 (M17):    State pruning + snapshots
Phase 4 (M19):    Sharding V1, 2 shards
Phase 5:          EVM on testnet (needs grant)
Phase 6:          tQTC stress testing
Phase 7:          DAO vote tQTC→QTC ratio
Phase 8:          MAINNET LAUNCH
```

---

## Next Zero-Cost Milestones

| M | Name | Cost | Status |
|---|---|---|---|
| Landing page | Validator registration UI | $0 | Building |
| M17 | State pruning | $0 | Next |
| M19 | Sharding V1 | $0 | After M17 |
| M18 | PoUW app-chain | $25 | After grant |
| M16 | ZK light client | Grant | After grant |
| M20 | EVM + governance V2 | Grant | After grant |

---

## Grant Applications

| Grant | Amount | Status |
|---|---|---|
| Outlier Ventures | $100K | ⏳ Apply now |
| ETH Foundation ESP | $10-50K | ⏳ Apply now |
| Gitcoin Grants | Variable | ⏳ Apply now |
| DoraHacks | $10K | ⏳ Apply now |
| Web3 Foundation | $10-50K | ❌ Closed |

**Pitch:** QTC Testnet live July 20 2026. 3,677+ blocks.
Post-quantum L1, Dilithium2, MIT, solo $0. Need $50K audit.

---

## Infrastructure

| Service | Provider | Cost | Status |
|---|---|---|---|
| Testnet node | Codespaces (temp) | Free 60hr | ✅ Running |
| Testnet node | Oracle Cloud | $0 forever | ⏳ This week |
| Landing page | Vercel | Free | ⏳ Building |
| Explorer | Vercel | Free | ⏳ Deploy |

---

## Financial Reality

| Item | Status |
|---|---|
| Current USDC | $0 |
| Founder vesting | 150M QTC (12mo cliff) |
| Genesis allocation | 500K QTC (6mo cliff) |
| Salary path | Grant → audit → mainnet → $3K/mo |
