# qc-node

> Post-quantum Layer 1 blockchain node — CRYSTALS-Dilithium2 signatures, libp2p gossip, EIP-1559 fee market, JSON-RPC API, multi-validator registry.

**Public Good Statement:** QTC is MIT-licensed public infrastructure. All code, docs, and audit reports are free forever. No patents. No proprietary components. Grant funds are used only for security audits and legal opinions to protect users — not for founder profit.

**Founder:** Touqeer Ahmad — touqeerahmadofficial896@gmail.com — Dunga Bunga, Punjab, Pakistan. Solo developer. Not anonymous. Publicly accountable.

[![CI](https://github.com/quantumchain-core/qc-node/actions/workflows/test.yml/badge.svg)](https://github.com/quantumchain-core/qc-node/actions)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Tests](https://img.shields.io/badge/tests-39%20passing-brightgreen)

---

## What is QTC?

QuantumChain (QTC) is a Layer 1 blockchain built from the ground up to be quantum-resistant. Every key, every block signature, and every validator identity uses **CRYSTALS-Dilithium2** — one of the four algorithms selected by NIST in their Post-Quantum Cryptography standardization (FIPS 204, August 2024).

Every other major blockchain (Bitcoin, Ethereum, Solana) uses ECDSA or EdDSA — cryptography that a sufficiently powerful quantum computer can break. QTC does not have this problem.

---

## Milestones completed

| Milestone | What it adds |
|---|---|
| M1 — Crypto | Dilithium2 keygen / sign / verify (pk=1312B, sk=2560B, sig=2420B) |
| M2 — Network | libp2p 0.53 swarm, TCP + noise + yamux, gossipsub |
| M3 — Chain types | Block, BlockHeader, Transaction, genesis_block() |
| M4 — Mempool | EIP-1559 fee ordering, per-sender nonce tracking, TTL eviction |
| M5 — Consensus | Block production loop, base-fee adjustment, slot framework |
| M6 — State + Storage | Account model, Executor (u128 math), sled persistence |
| M7 — Gossip handler | GossipMsg, handle_gossip(), publish(), dual-topic subscribe |
| M8 — JSON-RPC | axum HTTP server, 6 eth_* methods, AppState shared via Arc |
| M9 — Event loop | Node struct, tokio::select! loop, async swarm + RPC + block timer |
| M10 — Validator registry | ValidatorRegistry, SHA3-256 address derivation, real Dilithium2 block verification |

**39 tests passing. Clippy clean. CI green on every commit.**

---

## Repository structure

```
qc-node/
├── Cargo.toml
├── ARCHITECTURE.md          full internals reference
├── .github/workflows/
│   └── test.yml             CI: build + test + clippy
└── src/
    ├── lib.rs
    ├── crypto/              M1  — Dilithium2
    ├── net/
    │   ├── mod.rs           M2/M7 — swarm + publish()
    │   └── handler.rs       M7/M10 — gossip wire-format checks
    ├── chain/               M3  — Block, BlockHeader, genesis_block()
    ├── mempool/             M4  — tx pool, EIP-1559
    ├── consensus/
    │   ├── mod.rs           M5  — base-fee, ChainState
    │   ├── producer.rs      M6/M10 — builds + signs blocks
    │   ├── validator.rs     M10 — real Dilithium2 verify
    │   └── registry.rs      M10 — ValidatorRegistry, address_from_pubkey
    ├── state/               M6  — Account, StateDB, Executor, Storage
    ├── rpc/                 M8  — JSON-RPC HTTP server
    ├── node/                M9  — Node core (sync, unit-testable)
    └── bin/node.rs          M9/M10 — async binary entrypoint
```

---

## Quick start

### Prerequisites

- Rust stable (`rustup install stable`)
- A C compiler for pqcrypto (`build-essential` on Ubuntu / Xcode CLT on macOS)

### Build

```bash
git clone https://github.com/quantumchain-core/qc-node
cd qc-node
cargo build --release
```

### Run a single-validator node

```bash
# Storage path (sled database)
export QC_DB_PATH=/tmp/qtc-data

# RPC listen address (Ethereum-compatible JSON-RPC)
export QC_RPC_ADDR=0.0.0.0:8545

# Start the node (generates a fresh Dilithium2 keypair on first run)
./target/release/node
```

The node will:
1. Generate a Dilithium2 keypair and self-register as the sole validator
2. Bootstrap the genesis block (block 0)
3. Start the libp2p swarm (random port)
4. Start the JSON-RPC server on `QC_RPC_ADDR`
5. Produce a new block every 2 seconds when the mempool is non-empty

### Run with a multi-validator genesis config

```bash
export QC_GENESIS_PATH=/path/to/genesis.json
export QC_DB_PATH=/tmp/qtc-data
./target/release/node
```

Genesis file format:

```json
{
  "validators": [
    {
      "address": "0x<64 hex chars — SHA3-256 of pubkey>",
      "pubkey":  "0x<2624 hex chars — 1312-byte Dilithium2 pubkey>"
    }
  ]
}
```

Each `address` must equal `SHA3-256(pubkey)` — the node validates this at startup and rejects mismatches.

---

## JSON-RPC API

The node exposes an Ethereum-compatible JSON-RPC 2.0 HTTP endpoint.

```bash
# Check block number
curl -X POST http://localhost:8545 \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get balance (address is 0x + 64 hex chars, 32 bytes)
curl -X POST http://localhost:8545 \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x'$(python3 -c 'print("00"*32)')']","id":1}'
```

| Method | Params | Returns |
|---|---|---|
| `eth_chainId` | — | hex chain id |
| `eth_blockNumber` | — | hex block height |
| `eth_getBalance` | `[address]` | hex balance in nano-QTC |
| `eth_getTransactionCount` | `[address]` | hex nonce |
| `eth_getBlockByNumber` | `[hex number]` | block JSON or null |
| `eth_sendRawTransaction` | `[hex bincode tx]` | hex tx hash |

---

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `QC_DB_PATH` | `./qc-data` | sled storage directory |
| `QC_RPC_ADDR` | `0.0.0.0:8545` | JSON-RPC HTTP bind address |
| `QC_GENESIS_PATH` | unset | path to multi-validator genesis JSON |

---

## Run tests

```bash
QC_DB_PATH=/tmp/qtc-test cargo test -- --test-threads=1 --nocapture
```

---

## Ecosystem repos

| Repo | Purpose |
|---|---|
| [qtc-client](https://github.com/quantumchain-core/qtc-client) | TypeScript RPC client (wallet + faucet) |
| [qtc-faucet](https://github.com/quantumchain-core/qtc-faucet) | Cloudflare Worker faucet (100 QTC / 24h) |
| [qtc-wallet](https://github.com/quantumchain-core/qtc-wallet) | Tauri desktop wallet (Dilithium2 signing in Rust) |
| [qtc-explorer](https://github.com/quantumchain-core/qtc-explorer) | Next.js block explorer |

---

## Known limitations (tracked for M13+)

- No proposer rotation — `is_proposer()` always returns `true`. VRF needed before multi-validator launch.
- No state sync — new peers cannot download missing blocks from peers.
- No fee burn — base fee credited to coinbase, not burned.
- No persistent validator keystore — fresh keypair generated on every restart.
- No merkle tx_root — placeholder value.
- Keystore unencrypted — wallet saves sk in plain hex (Argon2 encryption in M13+).

---

## License

MIT — see [LICENSE](LICENSE).

---

## Whitepaper

See [QTC_WHITEPAPER_v3.md](./QTC_WHITEPAPER_v3.md) for the technical specification of the system as actually built (M1-M10). This supersedes the original v2.0 whitepaper which described a PoUW design that was intentionally simplified during implementation.
