# QuantumChain (QTC) Threat Model v1.0

**Version**: 1.0  
**Date**: October 2025  
**Status**: Research Phase. Living Document.  
**Scope**: Protocol design as described in Whitepaper v2.0. Excludes implementation bugs, social engineering, and operational security.

## 0. Disclaimer

This document identifies potential attack vectors for research purposes. It is not a security audit. No security audits have been completed. No testnet exists. All analysis is theoretical.

**We do not claim security.** We claim transparency.  
**Absence of a listed attack does not imply absence of vulnerability.**

For economic attack costs, simulation data, and mitigation benchmarks, see `ECONOMIC_SIM.md` and `BENCHMARKS.md` - pending.

## 1. Threat Model Scope & Assumptions

### 1.1 System Boundaries
In-scope: QTC L1 consensus, PoUW, Dilithium signatures, networking, state model, tokenomics, governance Phase 1.  
Out-of-scope: Mobile OS security, TEE hardware bugs, user wallet UX, exchange custody, legal risks.

### 1.2 Adversary Model
We assume adversaries are rational and financially motivated. Adversaries may control:
- Up to 33% of consensus stake
- Botnets of compromised mobile devices  
- Nation-state compute for cryptanalysis
- Forks of client software

We do NOT assume adversaries can break SHA3-256, Dilithium3, or physically clone secure elements at scale. Those are out of scope for v1.0.

### 1.3 Security Goals
1. **Safety**: No two conflicting blocks finalized
2. **Liveness**: Chain continues producing blocks under 33% Byzantine stake
3. **Censorship Resistance**: Any valid tx included within 10 blocks with 99% probability
4. **Sybil Resistance**: Cost to acquire 33% voting power >> honest participation cost

## 2. Attack Vector Taxonomy

**Status Key**: `Enumerated` = identified. `Analyzed` = impact/probability sketched. `Mitigated` = design change made. `Simulated` = ECONOMIC_SIM.md required. `Benchmarked` = BENCHMARKS.md required.

### 2.1 Consensus Layer - 18 vectors

**Attack #1: Long-Range Attack**  
**Status**: Analyzed  
**Description**: Adversary forks chain from genesis using old keys.  
**Impact**: New nodes may sync to attacker chain.  
**Current Mitigation**: Weak subjectivity checkpoints. Social consensus via Foundation.  
**Limitation**: Checkpoints reintroduce trust. See Attack #52.  
**Next Steps**: Research STARK-based history proofs. Pending.

**Attack #2: Nothing-at-Stake**  
**Status**: Mitigated  
**Description**: PoS validators sign multiple forks with no cost.  
**Mitigation**: PoUW requires TEE quote per block. Signing multiple = multiple PoUW proofs = cost.  
**Limitation**: TEEs have vulnerabilities. See Attack #7.

**Attack #3: Stake Grinding**  
**Status**: Analyzed  
**Description**: Validator manipulates VRF seed to elect self.  
**Mitigation**: VRF input includes prev block hash + epoch randomness. Single-leader per slot.  
**Limitation**: Bias possible if attacker controls epoch boundary block. Probability analysis pending ECONOMIC_SIM.md.

**Attack #4: 33% Liveness Halt**  
**Status**: Analyzed  
**Description**: 33%+ validators go offline. Chain stops.  
**Mitigation**: No slashing for downtime. Recovery via social hard fork.  
**Limitation**: Centralization risk during recovery. See Attack #55.

**Attack #5: 66% Safety Violation**  
**Status**: Enumerated  
**Description**: 66%+ validators sign conflicting blocks. Finality broken.  
**Mitigation**: Slashing 100% of stake for provable double-sign.  
**Limitation**: Slashing only works if stake has value. Bootstrapping problem. See ECONOMIC_SIM.md.

**Attack #6: Validator Eclipse**  
**Status**: Analyzed  
**Description**: Network adversary isolates validator, feeds false chain.  
**Mitigation**: libp2p Kademlia DHT, peer diversity, heartbeat to Foundation beacons.  
**Limitation**: Beacons are trust point. Phase 2 removes. See Attack #51.

**Attack #7: TEE Compromise**  
**Status**: Analyzed  
**Description**: Root exploit on Android allows fake TEE quote generation.  
**Impact**: Sybil attack. One phone = many validators.  
**Mitigation**: Rate limit 1 validator NFT per device per 90 days. Keystore attestation.  
**Limitation**: Keystore can be bypassed on rooted phones. This raises attacker cost, not eliminates. See ECONOMIC_SIM.md for break-even.

**Attack #8: PoUW Grinding**  
**Status**: Enumerated  
**Description**: Attacker pre-computes PoUW tasks to choose favorable block content.  
**Mitigation**: PoUW input includes prev block hash. Cannot predict.  
**Next Steps**: Formal analysis of bias pending.

**Attack #9: Withholding Attack**  
**Status**: Analyzed  
**Description**: Block proposer withholds block to break PoUW timing.  
**Mitigation**: 3-second slots. If no block, next proposer skips.  
**Limitation**: Can reduce throughput. See Attack #15.

**Attack #10-#18**: Enumerated. Details pending v1.1. Covers VRF bias, committee bribery, finality stalls, etc.

### 2.2 Network Layer - 12 vectors

**Attack #19: Sybil Attack on DHT**  
**Status**: Analyzed  
**Description**: Attacker spawns millions of nodes to control peer discovery.  
**Mitigation**: IP rate limiting, stake-weighted peer scoring.  
**Limitation**: Mobile IPs are NAT’d. CGNAT allows thousands per IP. See BENCHMARKS.md.

**Attack #20: Eclipse via ISP**  
**Status**: Analyzed  
**Description**: State-level adversary blocks all QTC ports at national firewall.  
**Mitigation**: libp2p AutoNAT + Circuit Relay v2. Traffic looks like WebRTC.  
**Limitation**: DPI can still flag. Tor/I2P integration research pending.

**Attack #21: 51% Network Partition**  
**Status**: Enumerated  
**Description**: Split internet in half. Both sides finalize. Reorg on reconnect.  
**Mitigation**: None at L1. Social consensus picks fork.  
**Limitation**: Violates safety. Acceptable for research phase.

**Attack #22-#30**: Enumerated. Covers bandwidth exhaustion, mempool spam, etc.

### 2.3 Cryptographic Layer - 9 vectors

**Attack #31: Quantum Break of Dilithium3**  
**Status**: Analyzed  
**Description**: CRQC breaks Dilithium before 2035. All accounts drained.  
**Probability**: NIST estimates <1% before 2030, ~50% by 2035.  
**Mitigation**: Agility to upgrade to Dilithium5 or new NIST standard via hard fork.  
**Limitation**: Requires governance consensus. See Attack #56.

**Attack #32: Hash Collision on SHA3-256**  
**Status**: Enumerated  
**Description**: Find two messages with same hash. Forge state root.  
**Mitigation**: Rely on SHA3-256 collision resistance. No defense if broken.  
**Limitation**: All blockchains fail if SHA3 breaks. Out of scope.

**Attack #33-#39**: Enumerated. Covers VRF weaknesses, signature malleability, etc.

### 2.4 Economic Layer - 15 vectors

**Attack #40: Plunge Protection Attack**  
**Status**: Analyzed  
**Description**: Whale crashes QTC price, buys stake cheap, 66% attacks.  
**Mitigation**: 21-day unbonding. Attacker locks capital during crash.  
**Limitation**: If QTC goes to zero, slashing worthless. See ECONOMIC_SIM.md.

**Attack #41: Inflation Bug**  
**Status**: Enumerated  
**Description**: Code bug mints infinite QTC.  
**Mitigation**: Formal verification of issuance rules. Pending.  
**Limitation**: No formal verification yet. Audit required pre-mainnet.

**Attack #42: Fee Market Manipulation**  
**Status**: Enumerated  
**Description**: Attacker fills blocks to drive base fee up, prices out users.  
**Mitigation**: EIP-1559 elasticity. Target 50% fullness.  
**Next Steps**: Simulation of griefing cost pending ECONOMIC_SIM.md.

**Attack #43-#54**: Enumerated. Covers MEV, flash loan attacks, oracle manipulation, etc.

### 2.5 Governance Layer - 12 vectors

**Attack #52: Foundation Capture**  
**Status**: Analyzed  
**Description**: Phase 0: Foundation is centralized. Can rug upgrades.  
**Mitigation**: Phase 1 transition to on-chain voting. Phase 2 constitutional.  
**Limitation**: Transition timeline is social, not technical. See Attack #56.

**Attack #55: Voter Apathy**  
**Status**: Analyzed  
**Description**: <5% quorum. Whale passes malicious upgrade.  
**Mitigation**: 30% quorum required. 7-day vote delay.  
**Limitation**: Quorum may never be met. Chain stagnation. See GOVERNANCE.md.

**Attack #56: Governance Deadlock**  
**Status**: Enumerated  
**Description**: Community splits on quantum upgrade path. Chain forks.  
**Mitigation**: None. Forks are feature of blockchains.  
**Limitation**: User confusion. Brand damage.

**Attack #57-#63**: Enumerated. Covers bribery, plutocracy, etc.

### 2.6 Mobile-Specific Layer - 8 vectors

**Attack #64: Battery Drain Griefing**  
**Status**: Analyzed  
**Description**: Attacker sends blocks requiring max CPU to drain validators.  
**Mitigation**: Gas limits per opcode. PoUW tasks capped at 500ms.  
**Limitation**: Benchmarks needed. See BENCHMARKS.md.

**Attack #65: 3G Network Partition**  
**Status**: Analyzed  
**Description**: 500KB blocks take 4s on 3G. Consensus fails.  
**Mitigation**: Max block 500KB. Target 3s slots. Tight but feasible.  
**Limitation**: Users on 2G excluded. Acceptable tradeoff.

**Attack #66-#71**: Enumerated. Covers storage exhaustion, state bloat, etc.

**Attack #72-#74**: Reserved for future analysis.

## 3. Risk Matrix Summary

| **Severity** | **Count** | **Analyzed** | **Mitigated** |
| --- | --- | --- | --- |
| Critical | 12 | 8 | 2 |
| High | 28 | 19 | 5 |
| Medium | 24 | 14 | 3 |
| Low | 10 | 4 | 1 |
| **Total** | **74** | **45** | **11** |

**Note**: "Mitigated" means design includes defense. Not cryptographically proven. BENCHMARKS.md + ECONOMIC_SIM.md required to validate.

## 4. Next Documents

1. **ECONOMIC_SIM.md** - Cost to 33% attack under varying validator counts
2. **BENCHMARKS.md** - Dilithium verify time on Pixel 6, iPhone 12, Redmi 9A
3. **GOVERNANCE.md** - Phase 1 voting mechanics, quorum attacks
4. **THREAT_MODEL v1.1** - Complete Attack #10-#74 analysis

## 5. How to Contribute

Found an attack we missed? Open issue with label `security`.  
**Responsible Disclosure**: Email security@quantumchain.foundation - PGP key in repo.  
**Bounty**: No bounty yet. Post-mainnet.

---

**We do not claim security. We claim transparency.**  
**Audit everything. Trust nothing. Launch when safe.**

End of THREAT_MODEL.md v1.0