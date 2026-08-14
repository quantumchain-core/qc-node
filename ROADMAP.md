# QTC Roadmap

**Doc accuracy note (Aug 2026):** this file previously listed several items
as "planned" that were already built (state sync, keystore encryption) and
one as planned-in-a-separate-Solidity-repo that turned out to already exist
here as native Rust (M14). Corrected below as part of a core-dev review;
see `MILESTONES.md` for the detailed per-milestone writeup.

## Phase 1: Core Node — M1–M10 ✅ COMPLETE, plus M14 + M17 (partial)

All milestones complete. CI green. See `MILESTONES.md`.

The node: generates Dilithium2 keypairs, finds peers via libp2p, gossips
blocks and transactions, executes transfers and M14 vesting/governance
actions with a working EIP-1559 fee market (base fee that actually adjusts,
priority fees that actually get paid), persists state to sled, verifies
block signatures against a genesis validator registry, serves a JSON-RPC
HTTP API, syncs new/rejoining peers up via block backfill, and produces
blocks on a 2s timer.

---

## Phase 2: Ecosystem — M11–M15 (separate repos, $0 infra)

| M | Goal | Repo | Infra | Status |
|---|---|---|---|---|
| M11.1 | TypeScript RPC client | qtc-client | npm | ✅ Done — needs an update for the M14 tx wire-format change (see MILESTONES.md M4) |
| M11.2 | Cloudflare faucet (100 QTC/24h) | qtc-faucet | Cloudflare free | ✅ Done |
| M11.3 | Tauri desktop wallet | qtc-wallet | local binary | ✅ Done |
| M12 | Next.js block explorer | qtc-explorer | Vercel free | 🔄 In progress |
| M13 | Airdrop script + docs | qtc-mainnet | GitHub Pages | Planned |
| M14 | Vesting + Governance | **qc-node (this repo)** | none — native, no separate infra | ✅ Core logic + wiring complete |
| M15 | Mainnet genesis + launch | qtc-mainnet | Oracle Cloud free | Planned |

**M14 update:** the Solidity/EVM plan below is retired. Vesting and
governance are implemented as native Rust in this repo (`src/vesting`,
`src/governance`), wired into `StateDB`, transaction execution, and
read-only RPC. No EVM-compatible execution layer was needed or built.
~~Vesting and DAO contracts are written in Solidity, which implies an
EVM-compatible execution environment separate from qc-node's native account
model. Architecture decision required before M14: app-chain, EVM extension,
or cross-chain bridge.~~ (superseded — decision made: neither; native Rust.)

**M15 infra:** Oracle Cloud Always Free (2 AMD vCPUs, 1GB RAM) is sufficient
to run qc-node 24/7. Total cost to mainnet: $0.

---

## Phase 3: Protocol Upgrades — M16–M20

These milestones significantly increase protocol value and are the basis
for the Foundation Grant #001 (M16–M20, Jan 2027 – Dec 2028).

| M | Goal | Why It Matters | Status |
|---|---|---|---|
| M16 | Light client + ZK bridge | Coinbase needs this for PQC custody. 1M QTC grant. | Not started |
| M17 | State pruning + snapshots | 1TB state kills decentralization. 500K QTC grant. | 🔄 Backfill sync done (see MILESTONES.md M17); pruning/snapshots not started |
| M18 | PoUW app-chain | Original whitepaper promise. 2M QTC + 20% app-chain token. | Not started |
| M19 | Sharding V1 | 10k TPS = Visa level. Price pump. 3M QTC grant. | Not started |
| M20 | On-chain governance V2 | DAO can fire you. Proves decentralization. 0 QTC. Legacy. | Not started — note M14 (this repo) already ships governance V1 natively; V2 scope should be re-evaluated against what's already live before treating this as greenfield. |

Total M16–M19 grants: **6.5M QTC** from Foundation 15% allocation.
At QTC = $1, that is $6.5M for 2 years of work.

---

## Protocol Upgrade Backlog (pre-M16, tracked in ARCHITECTURE.md)

These are known gaps to close before or during M16–M20. Reviewed and
corrected in a core-dev pass (Aug 2026) — several items below were marked
outstanding while already done, and vice versa.

**Still open:**
- VRF proposer rotation — still round-robin (`slot % validator_count`),
  by design for now; VRF deferred to M16+.
- EIP-1559 fee burn — no burn mechanism exists. The full effective fee
  (base_fee + priority_fee, capped at the tx's max fee) goes to the
  coinbase; this was previously true for base_fee alone before the
  priority-fee-payment fix (see MILESTONES.md M6), and remains true now
  that priority_fee is included too. Burn vs. full-payout-to-proposer is
  a tokenomics decision that hasn't been made, not an oversight.
- Merkle Patricia Trie for tx_root and state_root — state is still a flat
  map, re-serialized and re-hashed in full every block. Fine at current
  scale; will not scale.
- Slashing for invalid block proposals — not implemented.
- PQC transport layer — libp2p noise still uses classical Ed25519 for
  transport identity, unrelated to (and weaker than) the Dilithium2
  identity used for block/tx authenticity at the payload level.
- eth_getTransactionByHash — requires a tx index in storage; not built.
- Per-source RPC rate limiting — the current limiter is a single global
  counter across all callers, not per-IP/per-key. Fine for a single-
  operator testnet; needs to change before this is public-facing.
- Governance quorum wiring — `SubmitProposal` currently receives `0` for
  active validator count (executor has no registry reference), so
  validator-quorum checks trivially pass. Needs the real registry count
  threaded through to the executor (see MILESTONES.md M14).
- Ops-fund USDC custody — `TimelockedOpsFund` tracks an internal USDC
  ledger with no actual on-chain asset backing it yet; design not started.
- Transport-identity binding — libp2p `PeerId` is unrelated to validator
  identity (fresh random Ed25519 per node, ignores the Dilithium2 pubkey
  passed in). Not a correctness bug today (payload-level signatures do the
  real authentication) but worth a deliberate decision for Sybil/eclipse
  resistance before it matters.

**Done — removed from backlog (were listed as outstanding, actually complete):**
- ~~State sync for new peers~~ — done, see MILESTONES.md M17 (backfill
  sync; pruning/snapshots still open, tracked under M17 in the Phase 3
  table above, not here).
- ~~Persistent validator keystore~~ — done, `src/keystore.rs`.
- ~~Argon2 + AES-256-GCM keystore encryption~~ — done, same module.
- ~~EIP-1559 base fee adjustment~~ — this was actually marked done in
  MILESTONES.md before but wasn't true in the code; now actually
  implemented (`producer::next_base_fee`), see MILESTONES.md M5.

