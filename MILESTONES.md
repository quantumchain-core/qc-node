# QC-Node Milestones

## M1: Post-Quantum Crypto Core `v0.1.0-m1` ✓ LOCKED
**Location:** `src/crypto/`  
**Status:** Complete + Tagged. DO NOT MODIFY.

**Scope:**
1. Dilithium3 keygen, sign, verify via `pqcrypto-dilithium`
2. SHA3-256 hashing for block/tx IDs
3. Unit tests: keypair roundtrip, signature verify, tamper detection

**Dependencies:** `pqcrypto-dilithium = "0.5"`, `sha3 = "0.10"`  
**Used by:** M3 blocks, M4 transactions, M5 consensus  
**Rules:** M1 is frozen. No libp2p code here. No changes without new major version.

---

## M2: libp2p Gossipsub Networking `v0.1.0-m2` - IN PROGRESS
**Location:** `src/net/`  
**Status:** Blocked on libp2p 0.53.0 lockfile issue

**Scope:**
1. Generate `PeerId` from ed25519 keys
2. Build `Swarm<QcBehaviour>` with gossipsub
3. Subscribe to `qc-blocks` topic
4. Tests: peer_id generation, swarm builds, topic subscribed

**Dependencies:** `libp2p = "0.53"` with features `gossipsub`, `tcp`, `noise`, `yamux`, `tokio`  
**Blocker:** `Cargo.lock` pinning `libp2p-swarm-0.44.2`. Must regenerate lockfile.  
**Used by:** M5 consensus, M6 block propagation

**Acceptance Criteria:**
- `cargo test net::m2_tests` passes
- `Swarm::new_ephemeral` compiles
- Tagged only after CI green

---

## M3: Block & State Engine `v0.1.0-m3` ✓ LOCKED
**Location:** `src/chain/`  
**Status:** Complete + Tagged. DO NOT MODIFY.

**Scope:**
1. `Block` struct with Dilithium signature from M1
2. `State` with basic account balances
3. `apply_block()` verifies M1 sig + updates state
4. Tests: valid block applies, invalid sig rejected

**Dependencies:** `src/crypto/` for Dilithium + SHA3  
**Used by:** M4 mempool validates against state, M6 fork choice  
**Rules:** No networking code. Pure state machine.

---

## M4: Transaction Pool + Mempool `v0.1.0-m4` - NEXT
**Location:** `src/mempool/`  
**Status:** Not started

**Scope:**
1. `Tx` struct with ID + data + M1 signature
2. `Mempool` HashMap for pending txs
3. Ops: `add_tx`, `get_tx`, `remove_tx`, `get_all_txs`
4. Reject duplicate tx IDs
5. Tests: add/get/remove, duplicate rejection

**Dependencies:** `src/crypto/` for tx signing, `src/chain/` for state validation  
**Used by:** M5 proposer picks txs from mempool  
**Rules:** No libp2p code. No P2P until M5 wires mempool to gossipsub.

---

## Known Issues Log

### M2 Blocker: libp2p version mismatch
**Symptom:** `error[E0599]: no associated function 'new_ephemeral' found`  
**Root Cause:** `Cargo.lock` contains `libp2p-swarm 0.44.2` despite `Cargo.toml` specifying `0.53`  
**Fix:** Delete `Cargo.lock`, run `cargo update -p libp2p --precise 0.53.0`, commit new lockfile  
**Date Found:** 2026-06-11  
**Owner:** @Touqeer
