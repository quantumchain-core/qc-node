# QuantumChain (QTC) Benchmarks v1.0

**Version**: 1.0  
**Date**: October 2025  
**Status**: Research Phase. No Benchmarks Run.  
**Purpose**: Define test methodology for mobile performance. Contains no data.

## 0. Disclaimer

**THIS DOCUMENT CONTAINS NO BENCHMARK DATA.**

No tests have been run. No devices have been tested. No performance claims are made. All tables below are templates for future research.

**Do not use this document to estimate performance.**  
**Do not use this document for device purchasing decisions.**  
**Do not claim QTC runs on any specific phone.**

Mainnet launch is conditional on completion of benchmarks defined herein. As of October 2025, all benchmarks are pending.

For threat analysis requiring performance data, see `THREAT_MODEL.md v1.0` Attack #64, #65.

## 1. Benchmark Scope

### 1.1 Objectives
Future versions will measure:
1. **Dilithium3 Verify Time**: Milliseconds to verify 1 signature on target devices
2. **Block Processing Time**: Milliseconds to process 500KB block on 3G
3. **Storage Growth**: GB/year under varying transaction loads
4. **Battery Drain**: mAh consumed per hour of validating
5. **TEE Quote Time**: Milliseconds to generate attestation

**Current Status**: 0 of 5 completed.

### 1.2 Target Devices
Tests must be run on 3 tiers:

| **Tier** | **Example Device** | **Spec** | **Status** |
| --- | --- | --- | --- |
| High-end | iPhone 15 Pro | A17, 8GB RAM | **Not tested** |
| Mid-range | Pixel 6a | Tensor, 6GB RAM | **Not tested** |
| Low-end | Redmi 9A | Helio G25, 2GB RAM | **Not tested** |

**Pass Criteria**: All 5 benchmarks must pass on Low-end tier before mainnet.

### 1.3 Test Methodology

All tests will use:
1. **Reference Implementation**: `qc-node v0.1.0-alpha` - does not exist yet
2. **Network Conditions**: 3G, 4G, WiFi. Throttled via Network Link Conditioner
3. **Dataset**: Mainnet genesis block + 1000 synthetic transactions
4. **Repetitions**: 100 runs per test, report p50, p95, p99

**No results will be published without methodology disclosure.**

## 2. Benchmark Templates

### 2.1 Dilithium3 Verification

**Test**: Verify 1000 Dilithium3 signatures in sequence.

| **Device** | **p50 (ms)** | **p95 (ms)** | **p99 (ms)** | **Pass <50ms?** |
| --- | --- | --- | --- | --- |
| iPhone 15 Pro | TBD | TBD | TBD | TBD |
| Pixel 6a | TBD | TBD | TBD | TBD |
| Redmi 9A | TBD | TBD |

**Status**: Not run. `TBD` = To Be Determined.

### 2.2 Block Processing - 3G

**Test**: Download + verify 500KB block on 3G. 1 Mbps throttled.

| **Device** | **p50 (ms)** | **p95 (ms)** | **p99 (ms)** | **Pass <3000ms?** |
| --- | --- | --- |
| iPhone 15 Pro | TBD | TBD |
| Pixel 6a | TBD | TBD |
| Redmi 9A | TBD | TBD | TBD | TBD |

**Status**: Not run. Required to validate Whitepaper v2.0 claim of 3s block time.

### 2.3 Storage Growth

**Test**: Run node for 24h at 10 TPS. Measure disk usage.

| **Device** | **GB/day** | **GB/year** | **Pass <5GB?** |
| --- | --- | --- | --- |
| iPhone 15 Pro | TBD | TBD | TBD |
| Pixel 6a | TBD | TBD | TBD |
| Redmi 9A | TBD | TBD | TBD |

**Status**: Not run. Required to validate Whitepaper v2.0 claim of <5GB state.

### 2.4 Battery Drain

**Test**: Validate for 1 hour. Measure mAh consumed.

| **Device** | **mAh/hour** | **Hours on 4000mAh** | **Pass <5%?** |
| --- | --- | --- | --- |
| iPhone 15 Pro | TBD | TBD | TBD |
| Pixel 6a | TBD | TBD | TBD |
| Redmi 9A | TBD | TBD | TBD |

**Status**: Not run. Critical for mobile-first claim.

### 2.5 TEE Quote Generation

**Test**: Generate 100 TEE attestations via Android Keystore / Secure Enclave.

| **Device** | **p50 (ms)** | **p95 (ms)** | **Pass <500ms?** |
| --- | --- | --- | --- |
| iPhone 15 Pro | TBD | TBD | TBD |
| Pixel 6a | TBD | TBD | TBD |
| Redmi 9A | TBD | TBD | TBD |

**Status**: Not run. Required for PoUW timing.

## 3. Pending Benchmarks

| **ID** | **Benchmark** | **Blocking Mainnet** | **Status** |
| --- | --- | --- | --- |
| BENCH-01 | Dilithium3 on Low-end | Yes | Pending |
| BENCH-02 | 500KB block on 3G | Yes | Pending |
| BENCH-03 | Storage <5GB/year | Yes | Pending |
| BENCH-04 | Battery <5%/hour | Yes | Pending |
| BENCH-05 | TEE quote <500ms | Yes | Pending |

**Mainnet Launch Criteria**: All 5 must pass on Redmi 9A or equivalent.

**Current Progress**: 0 of 5 complete.

## 4. Limitations

Even after benchmarks complete:
1. **Lab vs Real World**: 3G tests use throttling. Real 3G has packet loss.
2. **Device Fragmentation**: 10,000+ Android devices exist. Cannot test all.
3. **OS Updates**: iOS 18 may change performance. Benchmarks expire.
4. **No Consensus Load**: Tests are single-node. Network consensus adds overhead.

**Conclusion**: Benchmarks inform, but do not guarantee performance.

## 5. How to Contribute

Run benchmarks on your device once `qc-node v0.1.0-alpha` releases. Open PR with results.  
**Warning**: Do not submit fake data. All results must be reproducible.

---

**This document contains zero benchmark data.**  
**All performance claims in Whitepaper v2.0 are pending validation by this document.**  
**We do not claim mobile performance. We claim intent to test.**

End of BENCHMARKS.md v1.0