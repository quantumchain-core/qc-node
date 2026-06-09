# QuantumChain (QTC) Governance v1.0

**Version**: 1.0  
**Date**: October 2025  
**Status**: Research Phase. No On-Chain Governance Active.  
**Purpose**: Describe intended governance transition. Contains no guarantees.

## 0. Disclaimer

**NO ON-CHAIN GOVERNANCE EXISTS YET.**

All mechanisms described below are research proposals. No code is deployed. No votes have occurred. No tokens have voting power.

**This is not a promise to implement any specific system.**  
**Timelines are targets, not commitments.**  
**Phase transitions are conditional on research completion and security audits.**

For threat analysis of governance attacks, see `THREAT_MODEL.md v1.0` Attack #52-#63.  
For economic parameters subject to governance, see `ECONOMIC_SIM.md v1.0`.

## 1. Governance Philosophy

**Principle**: Minimize governance over time. Code is law where possible.

**Anti-Goals**:
1. No plutocracy - 1 token ≠ 1 vote long-term
2. No foundation control post-bootstrap
3. No emergency multisigs with upgrade power
4. No off-chain agreements that override on-chain rules

**Current Reality**: Phase 0 requires trusted foundation. This is a temporary bootstrap mechanism, not the end state.

## 2. Governance Phases

### 2.1 Phase 0: Foundation Bootstrap

**Status**: Active. October 2025 - TBD  
**Decision Maker**: QuantumChain Foundation, Pakistan  
**Scope**: All protocol upgrades, parameter changes, treasury

**Limitations**:
1. Centralized. Foundation can change any rule.
2. No slashing, no on-chain voting, no token utility.
3. Users must trust foundation not to rug.

**Exit Criteria**: Testnet launch + completion of `ECONOMIC_SIM.md` SIM-01 through SIM-05 + security audit.

**Note**: Phase 0 may last indefinitely if criteria not met. No timeline promised.

### 2.2 Phase 1: On-Chain Signaling

**Status**: Research. Not Implemented.  
**Target**: Q4 2026 - Q4 2027  
**Decision Maker**: QTC holders via off-chain vote, foundation executes

**Proposed Mechanism**:
1. **Proposal**: Any address with ≥ 10,000 QTC can post to `proposals.qtc.foundation`
2. **Discussion**: 7-day minimum public discussion period
3. **Vote**: 7-day voting period. 1 QTC = 1 vote. Quorum: 30% of staked QTC
4. **Execution**: If passed, foundation multisig executes within 14 days

**Limitations**:
1. Foundation retains multisig keys. Can censor or refuse execution.
2. 1-token-1-vote is plutocratic. Researching alternatives.
3. 30% quorum may never be met. Chain stagnation risk. See Attack #55.

**Exit Criteria**: Successful execution of 3 non-trivial upgrades + no critical bugs for 6 months.

### 2.3 Phase 2: Constitutional Minimization

**Status**: Research. Not Designed.  
**Target**: 2028+  
**Decision Maker**: On-chain code only

**Proposed Principles**:
1. **Immutable Core**: Consensus, cryptography, issuance formula cannot be changed except via hard fork requiring 80% social consensus
2. **Parameter DAO**: Only minor parameters tunable via vote: block size, fee multipliers. Ranges hard-coded.
3. **Time Locks**: All changes have 30-day delay. Emergency pause requires 5/7 security council, auto-expires 7 days.
4. **Fork Choice**: Users run software they trust. No "official" chain.

**Limitations**:
1. Not designed yet. May be impossible.
2. "Constitutional" implies off-chain social layer. Cannot be fully on-chain.
3. 80% social consensus threshold is arbitrary. May cause permanent splits.

**Exit Criteria**: Undefined. Research pending.

## 3. Parameters Under Governance

**The following parameters may be tunable in Phase 1+, subject to hard-coded limits:**

| **Parameter** | **Current** | **Min** | **Max** | **Status** |
| --- | --- | --- |
| Block size | 500KB | 100KB | 2MB | Research |
| Base fee multiplier | 1.0x | 0.1x | 10x | Research |
| Validator min stake | 100 QTC | 10 QTC | 10,000 QTC | Research |
| Unbonding time | 21 days | 7 days | 90 days | Research |
| Slashing % | 100% | 5% | 100% | Research |

**Note**: All values are proposals. No governance exists to change them yet. Mainnet launches with hard-coded values.

## 4. Treasury

**Status**: No treasury exists.  
**Phase 0**: Foundation may accept donations. No accounting published.  
**Phase 1 Proposal**: 10% of block rewards to on-chain treasury. Spending requires vote.  
**Phase 2 Proposal**: Treasury abolished. No protocol-level funding.

**Limitation**: No treasury = no paid development post-bootstrap. Sustainability risk.

## 5. Upgrade Process

**Phase 0**: Foundation posts announcement. Users upgrade or get forked off.  
**Phase 1**: Vote passes → 30-day delay → Node software auto-upgrades if flag set  
**Phase 2**: Hard forks only. No in-protocol upgrades.

**Emergency Process**: None. If critical bug found, chain halts. Social consensus to restart.

## 6. What Governance CANNOT Do

Hard-coded invariants that will never be subject to vote:
1. **21M Supply Cap**: Cannot be increased. Ever.
2. **No Premine**: Cannot retroactively mint to founders.
3. **Dilithium Requirement**: Cannot downgrade to ECDSA.
4. **PoUW Requirement**: Cannot switch to pure PoS or PoW.

**These are social commitments, not code. If 80%+ of users run new code, they can change anything. This document cannot prevent forks.**

## 7. Next Documents

1. **GOVERNANCE v1.1** - Formal spec for Phase 1 voting contract. ETA: Q2 2026
2. **SECURITY_COUNCIL.md** - Emergency pause membership, if implemented
3. **CONSTITUTION.md** - Phase 2 immutable rules. ETA: 2028+

## 8. How to Contribute

Governance research help wanted. Open issue with label `governance`.  
**Warning**: Do not propose changes that violate Section 6. Will be closed.

---

**This document describes research. No governance system is live.**  
**Phase transitions are targets, not promises.**  
**We do not claim decentralized governance. We claim intent to minimize trust over time.**

End of GOVERNANCE.md v1.0