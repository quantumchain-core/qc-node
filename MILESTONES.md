# QTC Node — Milestones

All milestones M1–M10 are complete, plus M14 (vesting + governance, native
Rust — see below) and M17 (state sync, partial — see below). This document
reflects the implementation as built, not the original PoUW design from
Whitepaper v2.0. See `QTC_WHITEPAPER_v3.md` for the full technical
specification.

**Doc accuracy note (Aug 2026):** a full line-by-line code review found this
file and `ROADMAP.md` had drifted from the actual code in both directions —
some things marked done weren't, some things marked planned-elsewhere were
already built here. This revision corrects that; see the core-dev review
notes inline below for what changed and why.

---

## ✅ M1 — Post-Quantum Crypto `v0.1.0`
**Location:** `src/crypto/`
**Status:** LOCKED. DO NOT MODIFY.

Dilithium2 (not Dilithium3) keygen/sign/verify via `pqcrypto-dilithium`.
Variant selected by import path `dilithium2::*`, not a Cargo feature.
Byte sizes: pk=1312, sk=2560, sig=2420. All downstream code depends on these exact sizes.

---

## ✅ M2 — P2P Network `v0.2.0`
**Location:** `src/net/mod.rs`
**Status:** Complete.

libp2p 0.53 swarm over TCP + noise + yamux. Gossipsub topics `qc-blocks`
and `qc-txs`. `SwarmBuilder::with_existing_identity()` API (new_ephemeral
removed in 0.53). Fixed via `.map_err(Box::<dyn Error>::from)` clippy pattern.
`identify` added to diagnose connections that established but never carried
gossip traffic. Listener + bootstrap-peer dialing confirmed present (an
earlier gap: swarms built correctly but never actually listened or dialed
out, so peers had nothing to connect to).

Transport identity (libp2p `PeerId`) is a fresh random Ed25519 keypair,
unrelated to the node's Dilithium2 validator identity — block/tx
authenticity is verified independently at the payload level, not via the
p2p layer's identity. Worth a deliberate decision before this matters for
Sybil/eclipse resistance (see ROADMAP backlog).

---

## ✅ M3 — Chain Types `v0.3.0`
**Location:** `src/chain/mod.rs`
**Status:** Complete.

`Block`, `BlockHeader`, `Transaction`, `genesis_block()`. Field: `number`
(not `height`). `to_signable_bytes()` excludes signature. `Block::hash()`
= SHA256 of signable bytes. Deleted conflicting `src/chain/header.rs`.

---

## ✅ M4 — Mempool `v0.4.0`
**Location:** `src/mempool/mod.rs`
**Status:** Complete.

EIP-1559 fee ordering via three indexes: by_hash, by_sender (nonce BTreeMap),
fee_index (u64::MAX - fee trick). MempoolConfig defaults: global_max=10000,
per_sender_max=64, base_fee=1000, ttl=3600.

M14 wiring added `Transaction.action: TxAction` (default `Transfer`) —
committed into both `signable_bytes()` and the RPC's `compute_tx_hash()`,
so it's part of what's signed and hashed, not a bolt-on unverified field.
This was a breaking change to the wire/signing format; safe only because
there's no live chain yet — see the state_root note under M6.

---

## ✅ M5 — Consensus Engine `v0.5.0`
**Location:** `src/consensus/mod.rs`, `src/consensus/producer.rs`
**Status:** Complete.

Block production loop. BLOCK_TIME_SECS=2, BLOCK_GAS_LIMIT=30_000_000.

**EIP-1559 base fee adjustment (target = gas_limit/2, ±1/8 per block,
floored at 1) is implemented in `producer::next_base_fee()`.** This line
previously claimed the same thing while the code just carried the parent's
base_fee forward unchanged forever — caught in a core-dev review and
actually implemented, with the mempool's admission floor (`update_base_fee`)
now kept in sync via hooks in `node::on_block` and `node::try_produce_block`
(that function existed but was previously never called outside tests).

Proposer selection is round-robin by slot (`slot % validator_count ==
my_index`), implemented directly in `Node::try_produce_block`. An earlier,
separate `Consensus::is_proposer()` that always returned `true` and checked
a legacy env var instead of the real registry was identified as dead code
(nothing called it) and removed — it was never the actual proposer-turn
check. VRF-based rotation is still deferred to M16+; round-robin is the
current mechanism, not a placeholder left mid-wire.

---

## ✅ M6 — State + Storage `v0.6.0`
**Location:** `src/state/`
**Status:** Complete.

Account model (balance u128, nonce u64). Executor: all arithmetic in u128.
StateDB uses `#[derive(Default)]`. sled storage via QC_DB_PATH env var.

Gas is charged (and paid to the coinbase) at `min(tx's declared max fee,
block_base_fee + tx.priority_fee)` per gas — the effective-fee formula the
mempool already used for *ordering* transactions is now also what
*execution* actually charges. Previously execution always charged exactly
`gas_limit * block_base_fee`, silently ignoring priority_fee — a real
incentive gap (validators had no on-chain reason to prefer higher-tipped
transactions), fixed in the same core-dev review pass as the M5 fee-market
work above. No burn mechanism exists — the full effective fee still goes to
the coinbase, same as before; see ROADMAP backlog.

**StateDB now also carries vesting and governance state** (see M14 below) —
`state_root()` folds in vesting claimed-amounts and governance
proposal/vote counts, so this data is consensus-critical, not read-only
side data. This changed the state_root formula; acceptable only because
there is no live chain yet.

---

## ✅ M7 — Gossip Handler `v0.7.0`
**Location:** `src/net/handler.rs`
**Status:** Complete.

GossipMsg enum {NewBlock(Block), NewTx(Transaction)}, bincode-serialized.
handle_gossip() validates parent hash + non-empty sig (lightweight check,
used for isolated unit tests). Full crypto verify lives in Node::on_block
(M9/M10). publish() helper. Subscribes to both qc-blocks and qc-txs topics.

---

## ✅ M8 — JSON-RPC API `v0.8.0`
**Location:** `src/rpc/`
**Status:** Complete.

axum HTTP server with a global rate limiter (per-node, not per-caller — see
ROADMAP backlog). AppState{state_db, mempool, storage, chain_head, outbox}.

- eth_* (6): eth_chainId, eth_blockNumber, eth_getBalance,
  eth_getTransactionCount, eth_getBlockByNumber, eth_sendRawTransaction.
  eth_sendRawTransaction verifies the declared hash matches the tx content
  before accepting, and queues GossipMsg::NewTx to the outbox.
- qtc_* dashboard (3): qtc_getValidator, qtc_getNetworkStats,
  qtc_getValidators.
- qtc_* M14 (2): qtc_getVestingSchedule, qtc_getProposal — read-only;
  claiming/voting/proposing happens via eth_sendRawTransaction with the
  matching TxAction, not a dedicated write RPC method.

One dead, unreachable `dispatch()` (defined in `rpc/methods.rs`, shadowed
by the real one in `rpc/mod.rs`) was found and removed in the core-dev
review — it had its own tests and looked live, but nothing ever called it.

---

## ✅ M9 — Live Event Loop `v0.9.0`
**Location:** `src/node/mod.rs`, `src/bin/node.rs`
**Status:** Complete.

Node struct{app:AppState, producer:Producer, registry:ValidatorRegistry}.
on_gossip() -> HandleResult, try_produce_block() -> Result<Option<Block>>,
drain_outbox() -> Vec<GossipMsg>. Bootstraps genesis_block() on first run.
tokio::select! loop: swarm gossip + 2s block timer + RPC server task, plus
sync-gap detection/request (M17) wired into the same loop.

`bin/node.rs` previously carried its own duplicate copy of the encrypted
keystore logic instead of using the shared `qc_node::keystore` module it
was extracted from — found and fixed in the core-dev review; the node
binary and the shared library module could otherwise silently drift apart.

No heartbeat/empty blocks: `try_produce_block()` returns `Ok(None)` and
skips the slot entirely when the mempool is empty, so block production
pauses (not errors) during idle periods. A deliberate simplification, not
a bug — but means block number alone can't distinguish "chain is dead"
from "chain is just idle," and anything assuming wall-clock-to-block-number
correlation (e.g. vesting timelocks, M14 below) will be off during idle
stretches.

---

## ✅ M10 — Validator Registry `v1.0.0`
**Location:** `src/consensus/registry.rs`, `src/consensus/validator.rs`
**Status:** Complete.

address_from_pubkey(pk) = SHA3-256(pk) per FIPS 202.
ValidatorRegistry: HashMap<Address, Vec<u8>> with load_from_file(path)/from_json().
Validates address == SHA3-256(pubkey) at load time — rejects mismatches,
rejects duplicates, caps at 1000 validators (genesis-file DoS protection).
validate_block_sig(block, &registry) calls real crypto::verify().
bin/node.rs loads from QC_GENESIS_PATH or falls back to single-validator self-registration.

---

## ✅ M14 — Vesting + Governance `v1.1.0` (native Rust, this repo)
**Location:** `src/vesting/`, `src/governance/`, wired via `src/state/mod.rs`,
`src/mempool/mod.rs` (`TxAction`), `src/state/executor.rs`, `src/rpc/qtc_methods.rs`
**Status:** Core logic + on-chain wiring complete. Design gaps noted below.

**This replaces the M14 entry that used to point at a separate `qtc-dao`
repo written in Solidity.** That plan required building an EVM-compatible
execution layer this chain doesn't have, just to run vesting/DAO logic that
was, it turned out, already fully implemented here as native Rust — tested,
but sitting completely unwired (zero references from state/RPC/node) until
this pass. The Solidity/qtc-dao plan is retired for M14; see ROADMAP.

- `CliffLinearVesting` (founder/team/advisor grants), `LinearVesting`
  (milestone grants), `TimelockedOpsFund` (spend proposals, 7-day timelock,
  auto-upgrades to 5-of-7 multisig requirement after block 100,000).
- `Governance`: 7-seat, 5-of-7 multisig, proposer recusal, 7/14-day review
  periods, 51%/66% validator quorum tiers, 30-day proposer cooldown, and an
  immutable-rule firewall that auto-rejects proposals touching total
  supply, founder vesting, the 5/7 threshold itself, license, or the
  first-dev allocation — before any voting happens.
- Wired as new `TxAction` variants (`ClaimCliffVesting`, `SubmitProposal`,
  `CastMultisigVote`, `ProposeSpend`, `ExecuteSpend`, etc.) dispatched in
  `Executor::dispatch_action`. Gas is charged even on a failing action
  (e.g. double-voting), same as any other tx — prevents free spam.
- Read-only via `qtc_getVestingSchedule` / `qtc_getProposal`.

**Known gaps, flagged not silently worked around:**
- `SubmitProposal`'s validator-quorum math currently passes `0` for active
  validator count (the executor has no registry reference) — quorum checks
  for token-vote proposals trivially pass until the real registry count is
  threaded through.
- `TimelockedOpsFund` tracks a USDC ledger entirely separate from the
  native QTC `Account.balance` — `ExecuteSpend` updates the fund's internal
  ledger only; there is no actual USDC custody/asset design yet.

---

## 🔄 M17 — State Sync `v1.1.0` (partial)
**Location:** `src/sync/mod.rs`, `src/net/sync_codec.rs`
**Status:** Gap-detection and block-backfill complete. No pruning/snapshots.

Closes what was previously a total gap: before this, a node that crashed
or joined late had no way to catch up (gossipsub only ever carries the
newest block). `Node::sync_request_for_gap()` detects a gap between an
announced block and the local head; `apply_sync_blocks()` applies the
response through the exact same validation path as gossiped blocks — no
shortcuts. `sync_codec.rs`'s hand-rolled `request_response::Codec` impl was
checked against libp2p 0.53's actual trait shape (`type Protocol: AsRef<str>
+ Send + Clone`, `#[async_trait]`-desugared methods) in the core-dev
review and confirmed structurally correct; confirmed compiling in CI since.

Not done: no state pruning or snapshotting (see M17 in the Phase 3 table
below — the full "State pruning + snapshots" milestone is still ahead;
this entry is specifically the backfill-sync piece of it).

---

## ⏳ M11–M13, M15 — Ecosystem + Mainnet (separate repos)

| Milestone | Repo | Status |
|---|---|---|
| M11.1 Shared RPC client | qtc-client | ✅ Done — **note:** needs a coordinated update for the M14 `TxAction`/signable-bytes wire format change above |
| M11.2 Cloudflare faucet | qtc-faucet | ✅ Done |
| M11.3 Tauri wallet | qtc-wallet | ✅ Done |
| M12 Block explorer | qtc-explorer | 🔄 In progress |
| M13 Airdrop + docs | qtc-mainnet | Planned |
| M15 Mainnet launch | qtc-mainnet | Planned |

(M14 moved above — see full entry; no longer tracked in qtc-dao.)

## 🔭 M16, M18–M20 — Protocol Upgrades (see ROADMAP.md)

(M17 moved above — partially complete in this repo, not purely a future item.)

