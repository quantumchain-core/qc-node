# QuantumChain (QTC) — Technical Whitepaper v3.0

**Status:** This document describes the system as implemented in M1–M10.
It supersedes Whitepaper v2.0 (PoUW design) which was simplified during
implementation. Future milestones (M16+) may reintroduce PoUW as an
app-chain layer.

**Authors:** Touqeer (Chief Architect), QuantumChain Foundation
**Date:** June 2026
**License:** MIT

---

## Abstract

QuantumChain (QTC) is a Layer 1 blockchain designed to be secure against
quantum computers. Every validator signature, every block header, and every
wallet key uses CRYSTALS-Dilithium2 — one of four algorithms selected by
NIST in their Post-Quantum Cryptography standardization process (FIPS 204,
August 2024). The system is implemented in Rust, uses libp2p for peer
discovery and block/transaction gossip, maintains an EIP-1559-style fee
market, and exposes an Ethereum-compatible JSON-RPC API. A multi-validator
registry loaded from a genesis configuration file enables permissioned
validator sets without a separate staking contract. This paper describes
the cryptographic primitives, network protocol, consensus mechanism, state
model, and RPC interface as they exist in the M1–M10 implementation.

---

## 1. Motivation

### 1.1 The Quantum Threat to Existing Blockchains

Bitcoin, Ethereum, and Solana all rely on elliptic curve cryptography
(ECDSA or EdDSA over secp256k1 or Ed25519). The security of these schemes
depends on the hardness of the elliptic curve discrete logarithm problem
(ECDLP). A quantum computer running Shor's algorithm can solve ECDLP in
polynomial time, meaning any sufficiently powerful quantum computer could
derive private keys from public keys and forge signatures on any existing
blockchain.

NIST estimates that quantum computers capable of breaking 256-bit elliptic
curve cryptography could exist within 10–15 years. When that threshold is
crossed, every wallet address and every validator signature on every
existing blockchain becomes forgeable.

### 1.2 The QTC Approach

QTC replaces all uses of elliptic curve cryptography with
CRYSTALS-Dilithium2, a lattice-based signature scheme whose security
reduces to the hardness of the Module Learning With Errors (MLWE) problem.
No known quantum algorithm provides a meaningful advantage against MLWE.
Dilithium2 is standardized as NIST FIPS 204.

The trade-off is larger key and signature sizes compared to ECDSA:

| Scheme | Public key | Secret key | Signature |
|---|---|---|---|
| ECDSA (secp256k1) | 33 bytes | 32 bytes | 64 bytes |
| Ed25519 | 32 bytes | 64 bytes | 64 bytes |
| **Dilithium2 (QTC)** | **1312 bytes** | **2560 bytes** | **2420 bytes** |

These sizes are fixed constants in the QTC implementation. All storage,
wire format, and RPC encoding decisions are built around them.

---

## 2. Cryptographic Primitives

### 2.1 Signatures — CRYSTALS-Dilithium2

QTC uses the `pqcrypto-dilithium` Rust crate, specifically the `dilithium2`
module. The variant is selected by import path, not by a Cargo feature flag
— this is a critical implementation detail that must be preserved across
dependency updates.

```
Key generation:  dilithium2::keypair() -> (PublicKey, SecretKey)
Signing:         dilithium2::detached_sign(msg, sk) -> DetachedSignature
Verification:    dilithium2::verify_detached_sign(sig, msg, pk) -> bool
```

**What gets signed:** the `to_signable_bytes()` serialization of a
`BlockHeader` — all header fields in a fixed order, excluding the
`signature` field itself. This prevents circular dependencies and ensures
the signed commitment covers all consensus-critical data.

### 2.2 Hashing — SHA2-256 and SHA3-256

Two hash functions are used, for different purposes:

**SHA2-256** (`sha2` crate): block hash computation.
```
Block::hash() = SHA256(header.to_signable_bytes())
```
This is the canonical block identifier used as `parent_hash` in child
blocks and as the chain head pointer in `ChainHead`.

**SHA3-256** (`sha3` crate, NIST FIPS 202): validator address derivation.
```
address_from_pubkey(pk) = SHA3_256(pk)   // returns [u8; 32]
```
Every validator's 32-byte address is deterministically derived from their
1312-byte Dilithium2 public key. This means the address is self-certifying:
given an address in a block header's `proposer` field, a verifier can look
up the corresponding public key in the `ValidatorRegistry` and confirm the
address matches `SHA3_256(pubkey)`.

### 2.3 State Root — SHA2-256 over Sorted Accounts

The `state_root` field in a `BlockHeader` is a deterministic commitment to
the account state after executing the block's transactions:

```
state_root = SHA256(
  concat(sorted_by_address([
    (address || balance_le_bytes || nonce_le_bytes)
    for each account with non-zero balance or nonce
  ]))
)
```

This is a placeholder construction, not a Merkle Patricia Trie. It is
deterministic and collision-resistant for the current validator set size
but does not support efficient inclusion proofs. A full Merkle Patricia
Trie is planned for M16+.

---

## 3. Network Protocol

### 3.1 Transport Stack

QTC uses libp2p 0.53 with the following transport stack:

```
TCP  ->  Noise (XX handshake, Ed25519 ephemeral keys)  ->  Yamux multiplexer
```

Note that the transport-layer identity (Ed25519 for Noise) is separate from
the validator identity (Dilithium2 for block signing). Transport keys are
ephemeral and not stored; they are generated fresh on each node startup.

### 3.2 Gossip Protocol

QTC uses libp2p-gossipsub with `ValidationMode::Strict`. Two topics:

| Topic | Message type | Direction |
|---|---|---|
| `qc-blocks` | `GossipMsg::NewBlock(Block)` | Produced blocks broadcast to peers |
| `qc-txs` | `GossipMsg::NewTx(Transaction)` | Transactions relayed from RPC to peers |

`GossipMsg` is serialized with `bincode` (little-endian, fixed-width
integers, length-prefixed `Vec<u8>`). This is the same encoding used for
`eth_sendRawTransaction` in the RPC layer.

### 3.3 Message Handling

Incoming gossip messages are processed by `node::Node::on_gossip()`:

**For `NewTx`:** deserialize, call `mempool.add(tx)`. The mempool validates
nonce ordering, base fee, gas limit, and per-sender/global capacity limits.

**For `NewBlock`:** five-step validation pipeline:
1. `parent_hash == chain_head.head_hash` — chain linkage
2. `validate_block_sig(&block, &registry)` — Dilithium2 verification
3. Declared tx gas sum matches `header.gas_used` and `<= gas_limit`
4. `Executor::execute_block()` on a state clone — full tx execution
5. Returned gas used matches `header.gas_used`

Only if all five steps pass: commit state, remove included txs from
mempool, persist block + state, advance `chain_head`.

**Two-layer validation design:** `net::handler::handle_gossip()` performs a
cheap wire-format check (parent hash + non-empty signature). Full
cryptographic and state validation lives in `node::Node::on_block()`, which
has access to the `ValidatorRegistry` and `StateDB`. This separation keeps
the network handler unit-testable without a registry or state dependency.

---

## 4. Consensus

### 4.1 Block Production

QTC currently uses a **single-proposer model**: every registered validator
may produce a block, but there is no enforced rotation. The
`is_proposer()` function always returns `true`. This is intentional for the
M1–M10 implementation — proposer selection via VRF is planned for M16+.

In a single-validator deployment (the expected configuration for M15
mainnet launch), this is correct: there is one validator, it produces every
block, and all other nodes validate via the registry.

Block production (`Producer::produce_block`):

```
1. Pull up to 1000 highest-fee txs from mempool
2. Build BlockHeader skeleton:
     parent_hash = current chain head hash
     number      = parent.number + 1
     slot        = parent.slot + 1
     timestamp   = now()
     proposer    = SHA3_256(validator_pk)   // self-identifying
     gas_used    = 0                         // filled after execution
     gas_limit   = 10,000,000
     base_fee    = parent.base_fee
     signature   = []                        // filled after signing
3. Execute all txs against a StateDB clone
4. Fill header.gas_used = actual gas used
5. Fill header.state_root = state.state_root()
6. Sign header.to_signable_bytes() with validator_sk -> header.signature
7. Remove included txs from mempool
8. Persist block + state via Storage
```

### 4.2 Block Time

`BLOCK_TIME_SECS = 2`. The async event loop fires `try_produce_block()`
on a `tokio::time::interval` every 2 seconds. If the mempool is empty,
no block is produced (the chain does not produce empty blocks).

### 4.3 EIP-1559 Fee Market

QTC implements an EIP-1559-style adaptive base fee:

```
target_gas = BLOCK_GAS_LIMIT / 2   // 15,000,000

if gas_used == target_gas:
    base_fee unchanged
elif gas_used > target_gas:
    base_fee += max(1, base_fee * (gas_used - target_gas) / target_gas / 8)
else:
    base_fee -= max(1, base_fee * (target_gas - gas_used) / target_gas / 8)
```

`BLOCK_GAS_LIMIT = 30,000,000`. Default `base_fee = 1,000` nano-QTC per
gas unit. A standard transfer costs `21,000 * base_fee` nano-QTC in gas
plus `value` in transfer amount.

**Note:** The base fee is currently credited to the block proposer's
`coinbase` address, not burned. Fee burning is planned for M16+.

### 4.4 Validator Registry

The `ValidatorRegistry` maps 32-byte validator addresses to their full
1312-byte Dilithium2 public keys. It is loaded from a genesis JSON file
at node startup:

```json
{
  "validators": [
    {
      "address": "0x<64 hex chars>",
      "pubkey":  "0x<2624 hex chars>"
    }
  ]
}
```

At load time, the registry validates that each `address == SHA3_256(pubkey)`
and that each pubkey is exactly 1312 bytes. Mismatches are rejected with
a descriptive error — this catches copy/paste errors in genesis config.

If `QC_GENESIS_PATH` is not set, the node self-registers using its own
freshly generated public key (`ValidatorRegistry::single(&pk)`). This is
the intended configuration for a single-validator testnet or local
development node.

---

## 5. State Model

### 5.1 Accounts

QTC uses an account-based (not UTXO) model. Every address maps to:

```rust
pub struct Account {
    pub balance: u128,    // nano-QTC (10^-9 QTC). u128 prevents overflow.
    pub nonce: u64,       // incremented on each outgoing transaction
    pub code: Vec<u8>,     // reserved for smart contracts (M16+)
    pub storage_root: Hash, // reserved for contract storage (M16+)
}
```

**1 QTC = 1,000,000,000 nano-QTC** (10^9). The `u128` balance type can
hold up to ~3.4 × 10^20 nano-QTC, or ~3.4 × 10^11 QTC — well above the
1,000,000,000 QTC fixed supply.

### 5.2 Transaction Execution

For each transaction in a block:

```
1. Check sender.nonce == tx.nonce         (replay protection)
2. gas_cost = tx.gas_limit * base_fee     (u128 multiplication, no overflow)
3. total_cost = tx.value + gas_cost
4. Check sender.balance >= total_cost     (solvency check)
5. sender.balance -= total_cost
6. sender.nonce += 1
7. recipient.balance += tx.value
8. coinbase.balance += gas_cost           (fee to block proposer)
```

All arithmetic is performed in `u128`. `tx.value` and `tx.gas_limit` are
cast to `u128` before any multiplication. This prevents the silent overflow
bugs that have historically led to critical vulnerabilities in blockchain
implementations.

### 5.3 Storage

QTC uses `sled` (an embedded Rust key-value store) for persistence.

| Key | Value |
|---|---|
| `"block_{number}"` | bincode-serialized `Block` |
| `"state"` | bincode-serialized `StateDB` (full account map) |

The storage path is configured via `QC_DB_PATH`. On node restart, the
persisted state is loaded and the chain resumes from the last committed
block.

---

## 6. JSON-RPC API

QTC exposes a JSON-RPC 2.0 HTTP server (axum) on `QC_RPC_ADDR`
(default `0.0.0.0:8545`). Method names follow Ethereum conventions for
wallet and tooling compatibility.

### 6.1 Methods

**`eth_chainId`** — returns the QTC chain id as a hex string.
```json
{"method":"eth_chainId","params":[]}
// result: "0x51"
```

**`eth_blockNumber`** — current block height.
```json
{"method":"eth_blockNumber","params":[]}
// result: "0x1a"
```

**`eth_getBalance`** — account balance in nano-QTC.
```json
{"method":"eth_getBalance","params":["0x<address>"]}
// result: "0x3b9aca00"   (1,000,000,000 nano-QTC = 1 QTC)
```

**`eth_getTransactionCount`** — account nonce.
```json
{"method":"eth_getTransactionCount","params":["0x<address>"]}
// result: "0x5"
```

**`eth_getBlockByNumber`** — full block with transactions.
```json
{"method":"eth_getBlockByNumber","params":["0x1"]}
```

**`eth_sendRawTransaction`** — submit a signed transaction.
```json
{"method":"eth_sendRawTransaction","params":["0x<hex bincode tx>"]}
// result: "0x<tx hash>"
```

### 6.2 Transaction Wire Format

Transactions submitted via `eth_sendRawTransaction` must be `bincode`-
serialized in the following field order (matching the Rust `Transaction`
struct):

```
hash          [u8; 32]   — 32 bytes raw (no length prefix)
from          [u8; 32]   — 32 bytes raw
to            [u8; 32]   — 32 bytes raw
value         u64        — 8 bytes little-endian
nonce         u64        — 8 bytes little-endian
base_fee      u64        — 8 bytes little-endian
priority_fee  u64        — 8 bytes little-endian
gas_limit     u64        — 8 bytes little-endian
signature     Vec<u8>    — 8-byte LE length prefix + 2420 bytes
received_at   u64        — 8 bytes little-endian
```

The `qtc-client` TypeScript package (`serializeTransaction`) handles this
encoding automatically.

---

## 7. Token Economics

**Fixed supply: 1,000,000,000 QTC** (1 billion, no inflation).

| Pool | % | QTC | Vesting |
|---|---|---|---|
| Community Emissions | 38% | 380M | 10yr linear to validators |
| Ecosystem / Foundation | 15% | 150M | 10% TGE, 90% 3yr DAO |
| Founder | 15% | 150M | 12mo cliff, 4yr linear |
| Team Future Hires | 4% | 40M | 12mo cliff, 4yr linear |
| Advisors / Genesis | 3% | 30M | 6mo cliff, 2yr linear |
| Airdrop M13 | 1% | 10M | Unlocked at TGE |
| Liquidity / LBP | 4% | 40M | 50% TGE, 50% 6mo |
| Strategic Reserve | 20% | 200M | DAO vote only |

**Base denomination:** nano-QTC (10^-9 QTC). All on-chain values are
stored and transmitted in nano-QTC. 1 QTC = 1,000,000,000 nano-QTC.

---

## 8. Roadmap

### M11–M15: Ecosystem and Mainnet

| Milestone | Goal | Status |
|---|---|---|
| M11.1 | qtc-client TypeScript RPC client | Done |
| M11.2 | qtc-faucet Cloudflare Worker | Done |
| M11.3 | qtc-wallet Tauri desktop wallet | Done |
| M12 | qtc-explorer Next.js block explorer | In progress |
| M13 | Airdrop script + docs (RUN_VALIDATOR, TOKENOMICS) | Planned |
| M14 | Vesting + DAO contracts + UI | Planned |
| M15 | Mainnet genesis config + launch script | Planned |

### M16–M20: Protocol Upgrades

| Milestone | Goal |
|---|---|
| M16 | Light client + ZK bridge |
| M17 | State pruning + snapshots |
| M18 | PoUW app-chain (original whitepaper promise) |
| M19 | Sharding V1 (10k TPS target) |
| M20 | On-chain governance V2 |

---

## 9. Known Limitations and Future Work

| Limitation | Planned fix |
|---|---|
| No proposer rotation | VRF-based selection (M16+) |
| No state sync for new peers | `sync.rs` request-response protocol (M16+) |
| No fee burn | EIP-1559 base fee burn (M16+) |
| No Merkle tx_root | Merkle Patricia Trie (M16+) |
| No slashing | On-chain penalty for invalid blocks (M16+) |
| Unencrypted keystore | Argon2 + AES-256-GCM (M13+) |
| Persistent validator keystore | Load from file instead of regenerating (M15) |
| No tx hash verification | Mempool to verify hash == SHA256(tx fields) (M13+) |
| Smart contracts | RISC-V zkVM (M18+) |

---

## 10. Security Considerations

### 10.1 Post-Quantum Security Level

Dilithium2 targets NIST security level 2 (roughly equivalent to AES-128).
For a higher security level, Dilithium3 (level 3, ~AES-192) or Dilithium5
(level 5, ~AES-256) can be substituted — all three are in the
`pqcrypto-dilithium` crate. The implementation uses Dilithium2 for its
smaller signature size (2420 vs 3293 bytes for Dilithium3), which reduces
block size and network bandwidth.

### 10.2 Transport vs Validator Identity

The libp2p transport layer uses ephemeral Ed25519 keys (not post-quantum).
This means the P2P connection handshake is not quantum-resistant. Only
block signatures and wallet keys use Dilithium2. Full PQC transport
(using ML-KEM for key exchange) is a planned upgrade for M16+.

### 10.3 State Execution Safety

All transaction arithmetic uses `u128` to prevent overflow. The executor
validates nonce and balance before modifying state. State changes are
applied to a clone and committed atomically only on full success —
a failed transaction mid-block does not leave state partially mutated.

### 10.4 Genesis Config Integrity

The `ValidatorRegistry` validates `address == SHA3_256(pubkey)` at load
time. A corrupted or maliciously crafted genesis file with a mismatched
address is rejected before the node starts. This prevents a class of
attacks where a validator registers a pubkey they do not control.

---

## References

1. NIST FIPS 204 — Module-Lattice-Based Digital Signature Standard
   https://doi.org/10.6028/NIST.FIPS.204

2. CRYSTALS-Dilithium specification
   https://pq-crystals.org/dilithium/

3. libp2p specification
   https://github.com/libp2p/specs

4. EIP-1559: Fee market change
   https://eips.ethereum.org/EIPS/eip-1559

5. pqcrypto-dilithium Rust crate
   https://crates.io/crates/pqcrypto-dilithium

6. sled embedded database
   https://github.com/spacejam/sled
