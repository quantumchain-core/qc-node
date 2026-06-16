# QTC Roadmap

## Phase 1: Core Node — M1–M10 ✅ COMPLETE

All milestones complete. 39 tests passing. CI green. See `MILESTONES.md`.

The node: generates Dilithium2 keypairs, finds peers via libp2p, gossips
blocks and transactions, executes transfers with an EIP-1559 fee market,
persists state to sled, verifies block signatures against a genesis validator
registry, serves a JSON-RPC HTTP API, and produces blocks on a 2s timer.

---

## Phase 2: Ecosystem — M11–M15 (separate repos, $0 infra)

| M | Goal | Repo | Infra | Status |
|---|---|---|---|---|
| M11.1 | TypeScript RPC client | qtc-client | npm | ✅ Done |
| M11.2 | Cloudflare faucet (100 QTC/24h) | qtc-faucet | Cloudflare free | ✅ Done |
| M11.3 | Tauri desktop wallet | qtc-wallet | local binary | ✅ Done |
| M12 | Next.js block explorer | qtc-explorer | Vercel free | 🔄 In progress |
| M13 | Airdrop script + docs | qtc-mainnet | GitHub Pages | Planned |
| M14 | Vesting + DAO contracts + UI | qtc-dao | TBD (see note) | Planned |
| M15 | Mainnet genesis + launch | qtc-mainnet | Oracle Cloud free | Planned |

**M14 design note:** Vesting and DAO contracts are written in Solidity, which
implies an EVM-compatible execution environment separate from qc-node's native
account model. Architecture decision required before M14: app-chain, EVM
extension, or cross-chain bridge.

**M15 infra:** Oracle Cloud Always Free (2 AMD vCPUs, 1GB RAM) is sufficient
to run qc-node 24/7. Total cost to mainnet: $0.

---

## Phase 3: Protocol Upgrades — M16–M20

These milestones significantly increase protocol value and are the basis
for the Foundation Grant #001 (M16–M20, Jan 2027 – Dec 2028).

| M | Goal | Why It Matters |
|---|---|---|
| M16 | Light client + ZK bridge | Coinbase needs this for PQC custody. 1M QTC grant. |
| M17 | State pruning + snapshots | 1TB state kills decentralization. 500K QTC grant. |
| M18 | PoUW app-chain | Original whitepaper promise. 2M QTC + 20% app-chain token. |
| M19 | Sharding V1 | 10k TPS = Visa level. Price pump. 3M QTC grant. |
| M20 | On-chain governance V2 | DAO can fire you. Proves decentralization. 0 QTC. Legacy. |

Total M16–M19 grants: **6.5M QTC** from Foundation 15% allocation.
At QTC = $1, that is $6.5M for 2 years of work.

---

## Protocol Upgrade Backlog (pre-M16, tracked in ARCHITECTURE.md)

These are known gaps to close before or during M16–M20:

- VRF proposer rotation (is_proposer always returns true)
- State sync for new peers (sync.rs, deferred since M7)
- EIP-1559 fee burn (base fee currently credited to coinbase)
- Merkle Patricia Trie for tx_root and state_root
- Slashing for invalid block proposals
- Persistent validator keystore (currently regenerated on restart)
- Argon2 + AES-256-GCM keystore encryption (wallet)
- PQC transport layer (libp2p noise currently uses Ed25519)
- eth_getTransactionByHash (requires tx index in storage)

