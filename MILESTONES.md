# QTC Node — Milestones

All milestones M1–M10 are complete. This document reflects the
implementation as built, not the original PoUW design from Whitepaper v2.0.
See `QTC_WHITEPAPER_v3.md` for the full technical specification.

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
per_sender_max=64, base_fee=1000, ttl=3600. 9 tests.

---

## ✅ M5 — Consensus Engine `v0.5.0`
**Location:** `src/consensus/mod.rs`
**Status:** Complete.

Block production loop. EIP-1559 base fee adjustment (target = BLOCK_GAS_LIMIT/2,
±1/8 per block). BLOCK_TIME_SECS=2, BLOCK_GAS_LIMIT=30_000_000.
Note: VRF proposer rotation deferred to M16+. is_proposer() always returns true.

---

## ✅ M6 — State + Storage `v0.6.0`
**Location:** `src/state/`
**Status:** Complete.

Account model (balance u128, nonce u64). Executor: all arithmetic in u128,
base_fee cast once at top of execute_block. Removed GasUsedMismatch check
from executor (producer fills gas_used AFTER execution). StateDB uses
`#[derive(Default)]`. sled storage via QC_DB_PATH env var.

---

## ✅ M7 — Gossip Handler `v0.7.0`
**Location:** `src/net/handler.rs`
**Status:** Complete.

GossipMsg enum {NewBlock(Block), NewTx(Transaction)}, bincode-serialized.
handle_gossip() validates parent hash + non-empty sig (lightweight check).
Full crypto verify lives in Node::on_block (M9/M10). publish() helper.
Subscribes to both qc-blocks and qc-txs topics.

---

## ✅ M8 — JSON-RPC API `v0.8.0`
**Location:** `src/rpc/`
**Status:** Complete.

axum HTTP server. AppState{state_db, mempool, storage, chain_head, outbox}.
6 methods: eth_chainId, eth_blockNumber, eth_getBalance,
eth_getTransactionCount, eth_getBlockByNumber, eth_sendRawTransaction.
eth_sendRawTransaction adds tx to mempool AND queues GossipMsg::NewTx to outbox.

---

## ✅ M9 — Live Event Loop `v0.9.0`
**Location:** `src/node/mod.rs`, `src/bin/node.rs`
**Status:** Complete.

Node struct{app:AppState, producer:Producer, registry:ValidatorRegistry}.
on_gossip() -> HandleResult, try_produce_block() -> Result<Option<Block>>,
drain_outbox() -> Vec<GossipMsg>. Bootstraps genesis_block() on first run.
tokio::select! loop: swarm gossip + 2s block timer + RPC server task.
6 tests including two-node produce→gossip→converge simulation.

---

## ✅ M10 — Validator Registry `v1.0.0`
**Location:** `src/consensus/registry.rs`, `src/consensus/validator.rs`
**Status:** Complete.

address_from_pubkey(pk) = SHA3-256(pk) per FIPS 202.
ValidatorRegistry: HashMap<Address, Vec<u8>> with load_from_file(path)/from_json().
Validates address == SHA3-256(pubkey) at load time — rejects mismatches.
validate_block_sig(block, &registry) calls real crypto::verify().
bin/node.rs loads from QC_GENESIS_PATH or falls back to single-validator self-registration.
39 tests passing. Clippy clean.

---

## ⏳ M11–M15 — Ecosystem + Mainnet (separate repos)

| Milestone | Repo | Status |
|---|---|---|
| M11.1 Shared RPC client | qtc-client | ✅ Done |
| M11.2 Cloudflare faucet | qtc-faucet | ✅ Done |
| M11.3 Tauri wallet | qtc-wallet | ✅ Done |
| M12 Block explorer | qtc-explorer | 🔄 In progress |
| M13 Airdrop + docs | qtc-mainnet | Planned |
| M14 Vesting + DAO | qtc-dao | Planned |
| M15 Mainnet launch | qtc-mainnet | Planned |

## 🔭 M16–M20 — Protocol Upgrades (see ROADMAP.md)

