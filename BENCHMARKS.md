# QTC Benchmarks

**Status:** No benchmarks run yet. Templates only.
**Updated:** June 2026 — reflects Dilithium2 (not Dilithium3 as in v1.0),
desktop-first node (not mobile-first), and actual M1-M10 implementation.

---

## What Changed from v1.0

The original BENCHMARKS.md was written for the Whitepaper v2.0 PoUW design
(mobile phones as validators, TEE attestation, 3G block download).
The M1–M10 implementation diverged significantly:

| v2.0 assumption | Actual implementation |
|---|---|
| Dilithium3 (sig=3293 bytes) | **Dilithium2 (sig=2420 bytes)** |
| Mobile validator nodes | Desktop/server `qc-node` binary |
| TEE attestation required | Not implemented |
| 500KB block target | 30M gas limit (variable block size) |
| 3G network target | Standard TCP/IP (libp2p) |

Benchmarks below are updated for the actual implementation.
TEE and mobile benchmarks are deferred to M18 (PoUW app-chain).

---

## 1. Dilithium2 Performance (most critical)

**Why it matters:** Every incoming block requires one Dilithium2 verify call.
At 2s block time, the node must verify at least 1 signature every 2 seconds.
This is trivially fast on any modern CPU.

| Test | Target | Status |
|---|---|---|
| Sign 1 block header (2560-byte sk) | < 5ms | Not measured |
| Verify 1 block signature (1312-byte pk) | < 5ms | Not measured |
| Verify 1000 signatures sequentially | < 5000ms | Not measured |

**How to run once implemented:**
```bash
cargo bench --bench dilithium
```

---

## 2. Block Production Throughput

| Test | Target | Status |
|---|---|---|
| Produce 1 block (1000 txs, full mempool) | < 100ms | Not measured |
| Execute 1000 txs (state transitions) | < 50ms | Not measured |
| Gossip 1 block to 10 peers | < 500ms | Not measured |

---

## 3. Storage Growth

At 2s block time with 1000 txs/block:
- **TPS:** ~500 transactions/second
- **Block size (estimate):** ~1000 txs × ~2600 bytes/tx ≈ 2.6MB/block
- **Storage growth:** ~2.6MB × 43,200 blocks/day ≈ **112GB/day**

This is significant. State pruning (M17) is required before mainnet
can run sustainably on consumer hardware.

---

## 4. Memory Usage

| Scenario | Target | Status |
|---|---|---|
| Idle node (no txs) | < 100MB RSS | Not measured |
| Full mempool (10,000 txs) | < 500MB RSS | Not measured |
| 1M accounts in StateDB | < 2GB RSS | Not measured |

---

## 5. How to Contribute

Run any benchmark on your hardware and open a PR with:
- Hardware spec (CPU, RAM, OS)
- `cargo bench` output or timing methodology
- Any anomalies or regressions found

Do not submit benchmark results without disclosing methodology.
