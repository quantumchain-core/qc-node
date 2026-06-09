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
```toml
[dependencies]
libp2p = "0.53" # networking
tokio = { version = "1", features = ["full"] } # async
pqcrypto-dilithium = "0.17" # Dilithium3
sha3 = "0.10" # SHA3-256
rocksdb = "0.22" # state DB
serde = "1.0" # serialization
qc-node/
├── src/
│ ├── main.rs # CLI entry
│ ├── config.rs # Chain params, hard-coded for alpha
│ ├── crypto/
│ │ ├── dilithium.rs # wrap pqcrypto-dilithium
│ │ └── hash.rs # SHA3-256
│ ├── net/
│ │ ├── p2p.rs # libp2p swarm
│ │ └── sync.rs # Block sync logic
│ ├── chain/
│ │ ├── block.rs # Block struct, 500KB check
│ │ ├── state.rs # RocksDB account store
│ │ └── verify.rs # Block verification pipeline
│ └── teesig/
│ └── quote.rs # Parse TEE attestation, no verify yet
pub struct Block {
    pub height: u64,
    pub parent_hash: [u8; 32],
    pub timestamp: u64,
    pub proposer: DilithiumPublicKey, // 1952 bytes
    pub tee_quote: Vec<u8>, // Variable, ~2KB. Not verified in alpha
    pub tx_root: [u8; 32],
    pub state_root: [u8; 32],
    pub signature: DilithiumSignature, // 3309 bytes
}

const MAX_BLOCK_SIZE: usize = 500_000; // 500KB hard cap
pub struct Account {
    pub nonce: u64,
    pub balance: u128, // 0 for alpha, no transfers yet
    pub pubkey: DilithiumPublicKey,
}   1. size <= 500KB
   2. height = parent.height + 1
   3. parent_hash == hash(parent)
   4. timestamp > parent.timestamp
   5. dilithium_verify(proposer, block_hash, signature) == true
   6. state_transition(parent.state, block) == Ok(new_state)
   7. new_state.root == state_root