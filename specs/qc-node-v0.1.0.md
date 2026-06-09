# qc-node v0.1.0-alpha Specification

**Version**: 0.1.0-alpha 
**Date**: 2026-06-09 
**Status**: Draft. Not implemented. 
**Target**: Android 8.0+, iOS 15+, 2GB RAM, 3G network

## 1. Scope for Alpha

**In**:
1. P2P sync with libp2p
2. Block verification: Dilithium3 + SHA3-256
3. State: Account model, 500KB max blocks
4. PoUW: Accept TEE quote, no actual ML work yet
5. CLI: `qc-node start`, `qc-node status`

**Out**:
1. Wallets, RPC, staking UI
2. Real PoUW tasks
3. Slashing, governance
4. Mainnet, tokens

**Success = 2 phones on 4G sync 10 blocks in <60s.**

## 2. Architecture

### 2.1 Language & Deps
**Language**: Rust 1.78. Required for mobile + Dilithium perf.

**Core crates**:
- `libp2p = "0.53"` — networking
- `tokio = { version = "1", features = ["full"] }` — async
- `pqcrypto-dilithium = "0.17"` — Dilithium3
- `sha3 = "0.10"` — SHA3-256
- `rocksdb = "0.22"` — state DB
- `serde = "1.0"` — serialization

### 2.2 Module Structure

### 2.3 Consensus Rules Alpha
1. **Chain Selection**: Longest chain wins. No finality.
2. **Block Time**: Target 3s. If no block 10s, skip.
3. **Block Validity**:
   1. size <= 500KB
   2. height = parent.height + 1
   3. parent_hash == hash(parent)
   4. timestamp > parent.timestamp
   5. dilithium_verify(proposer, block_hash, signature) == true
   6. state_transition(parent.state, block) == Ok(new_state)
   7. new_state.root == state_root
4. **PoUW**: In alpha, accept any `tee_quote`. Log it. Don't verify.

### 2.4 Networking
**libp2p**: TCP + Noise + Yamux 
**Protocols**: `/qtc/sync/0.1.0`, `/qtc/block/0.1.0` 
**Message Limits**: 512KB max. Protects 3G users.

### 2.5 Mobile Build Targets
**Android**: `cargo ndk -t arm64-v8a build --release` → `libqc_node.so` 
**iOS**: `cargo build --target aarch64-apple-ios --release` → `libqc_node.a` 
**Storage**: All data in `<app_files>/qc-node/`. DB <100MB for alpha.

## 3. Benchmark Targets for v0.1.0

Must pass before APK release. These fill `BENCHMARKS.md v1.1`.

| **Test** | **Device** | **Target** |
| --- | --- | --- |
| Dilithium3 verify | Redmi 9A | <50ms |
| 500KB block verify | Redmi 9A | <3000ms on 3G |
| App cold start | Redmi 9A | <5s |
| Idle RAM | Redmi 9A | <150MB |
| DB size 10k blocks | Any | <100MB |

If any fail, alpha does not ship.

---

**End of spec. This is what I build to. No token. No price. Just code.** 