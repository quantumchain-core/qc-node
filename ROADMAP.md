# QC-Node Roadmap: M1 to M10

## M1: Post-Quantum Crypto Core `v0.1.0-m1` ✓ LOCKED
**Location:** `src/crypto/`  
**Status:** Complete + Tagged. FROZEN.

**Scope:**
1. Dilithium3 keygen, sign, verify
2. SHA3-256 hashing for all IDs  
3. Unit tests: roundtrip, tamper detection

**Rules:** No changes. Used by M3, M4, M5, M8.

---

## M2: libp2p Gossipsub Networking `v0.1.0-m2` - BLOCKED
**Location:** `src/net/`  
**Status:** Blocker: `Cargo.lock` pins libp2p 0.44.2

**Scope:**
1. Generate `PeerId` from ed25519
2. Build `Swarm<QcBehaviour>` with gossipsub on 0.53.0
3. Subscribe to `qc-blocks`, `qc-txs` topics
4. Tests: swarm builds, topics subscribed

**Used by:** M5, M6, M7, M9

---

## M3: Block & State Engine `v0.1.0-m3` ✓ LOCKED
**Location:** `src/chain/`  
**Status:** Complete + Tagged. FROZEN.

**Scope:**
1. `Block` with M1 Dilithium sig
2. `State` with account balances + nonces
3. `apply_block()` verifies sig + executes txs
4. Tests: valid/invalid blocks, state transitions

**Rules:** Pure state machine. No P2P.

---

## M4: Transaction Pool + Mempool `v0.1.0-m4` - NEXT
**Location:** `src/mempool/`  
**Status:** Not started

**Scope:**
1. `Tx` struct with M1 signature
2. `Mempool` HashMap + basic ops: add/get/remove/list
3. Reject duplicate txid + invalid sig
4. Prioritization: nonce ordering per account
5. Tests: duplicate rejection, nonce gaps

**Used by:** M5 proposer, M6 propagation

---

## M5: Proof of Stake Consensus `v0.1.0-m5`
**Location:** `src/consensus/`  

**Scope:**
1. Validator set + stake tracking in `State`
2. VRF-based proposer selection each slot
3. Block proposal from M4 mempool
4. Attestation messages via M2 gossipsub `qc-attest`
5. 2/3 stake finality rule
6. Tests: proposer election, attest collection, finality

**Depends on:** M1, M2, M3, M4

---

## M6: Block Propagation + Fork Choice `v0.1.0-m6`
**Location:** `src/net/`, `src/chain/`

**Scope:**
1. Broadcast new blocks via M2 `qc-blocks` topic
2. Download + verify blocks from peers
3. LMD-GHOST fork choice rule
4. Handle reorgs, orphan blocks
5. Tests: block sync, reorg simulation

**Depends on:** M2, M3, M5

---

## M7: Light Client Headers `v0.1.0-m7`
**Location:** `src/light/`

**Scope:**
1. Light client sync protocol
2. Header chain + M5 finality proofs
3. Merkle proof verification for state
4. Gossip `qc-headers` topic via M2
5. Tests: header sync, fraud proof detection

**Depends on:** M1, M2, M5

---

## M8: Account Abstraction + Smart Contracts `v0.1.0-m8`
**Location:** `src/vm/`, `src/chain/`

**Scope:**
1. WASM VM for contracts
2. Gas metering + limits
3. Account abstraction: M1 Dilithium as default, custom auth logic
4. Contract deploy + call txs in M4
5. Tests: contract deploy, gas exhaustion, AA validation

**Depends on:** M1, M3, M4

---

## M9: RPC + Indexer `v0.1.0-m9`
**Location:** `src/rpc/`, `src/indexer/`

**Scope:**
1. JSON-RPC server: `eth_*` compatible methods
2. Subscribe to M2 gossipsub, index blocks/txs to DB
3. Endpoints: `getBalance`, `getBlock`, `sendRawTransaction`
4. WebSocket subscriptions for new heads
5. Tests: RPC calls, event subscription

**Depends on:** M2, M3, M6

---

## M10: CLI + Wallet + App `v0.1.0-m10`
**Location:** `src/cli/`, `src/wallet/`

**Scope:**
1. `qc-node` CLI: start node, key management
2. `qc-wallet` CLI: create M1 keys, sign txs, check balance
3. Config files, logging, metrics
4. Docker image + compose for easy run
5. E2E tests: spin up 3 nodes, send tx, confirm finality

**Depends on:** All M1-M9
## M11: Mobile Wallet APK `v0.1.0-m11`
**Location:** `mobile/`
**Scope:** Android/iOS app. Rust core + Tauri/Flutter UI. Light client only.
**Depends on:** M1, M7, M9
**Deliverable:** `qc-wallet.apk` on GitHub Releases

---

## Core Rules

1. **M1 LOCKED:** Never edit `src/crypto/` after tag. Post-quantum sigs are consensus critical.
2. **Module Isolation:** M2 = `net/`, M3 = `chain/`, M4 = `mempool/`. No cross-contamination.
3. **Test Before Tag:** CI must be green before `git tag v0.1.0-mX`.
4. **Document Blockers:** If a milestone blocks, update this file with root cause + fix.
5. **One Milestone at a Time:** Finish M(N) before starting M(N+1).

## Current Blocker

**M2 libp2p version:** `Cargo.lock` forces `0.44.2` but code needs `0.53.0`.  
**Fix:** Delete lockfile, `cargo update -p libp2p --precise 0.53.0`, commit.  
**Impact:** M5-M10 all blocked until M2 unblocked.
