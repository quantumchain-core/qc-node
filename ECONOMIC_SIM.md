# QuantumChain (QTC) Economic Simulation Framework v1.0

**Version**: 1.0  
**Date**: October 2025  
**Status**: Research Phase. No Simulations Completed.  
**Purpose**: Define methodology for future attack cost analysis. Contains no financial data.

## 0. Disclaimer

**THIS DOCUMENT CONTAINS NO ECONOMIC DATA.**

All formulas, variables, and tables below are placeholders for future research. No simulations have been run. No attack costs have been calculated. No token prices are assumed or implied.

**This is not financial advice. This is not a security guarantee.**  
**Do not use this document for investment decisions.**  
**Do not use this document to estimate attack costs.**

Mainnet launch is conditional on completion of simulations defined herein. As of October 2025, simulations are pending.

For threat analysis without costs, see `THREAT_MODEL.md v1.0`.

## 1. Simulation Framework

### 1.1 Objectives
Future versions of this document will attempt to answer:
1. **Cost to 33% Attack**: Capital required to acquire 33% of consensus stake under varying conditions
2. **Cost to 66% Attack**: Capital required to break finality
3. **Griefing Cost**: Cost to halt liveness for N blocks
4. **Sybil Cost**: Cost to operate N validators given TEE constraints

**Current Status**: Methodology defined below. No results.

### 1.2 Assumptions
All future simulations will state assumptions explicitly. Expected assumptions:
1. **Rational Adversary**: Attacker maximizes profit or minimizes cost
2. **Liquid Market**: QTC can be acquired without 100% slippage. Simulation will test multiple liquidity levels
3. **TEE Security**: Assume rate of TEE compromise = X%. X is variable, not defined
4. **No External Factors**: Ignore regulatory bans, exchange delistings, or social layer responses

**These assumptions are not validated. Results will vary materially if assumptions change.**

### 1.3 Variables

| **Variable** | **Definition** | **Status** |
| --- | --- | --- |
| `P` | Price of QTC in USD | **Undefined** - No price assumed |
| `S_total` | Total staked QTC | **Undefined** - No mainnet |
| `N_val` | Number of active validators | **Undefined** - No mainnet |
| `C_phone` | Cost to acquire + root 1 phone | **Undefined** - Varies by country |
| `R_TEE` | Rate of TEE compromise | **Undefined** - Research pending |
| `T_unbond` | Unbonding time | **Defined** - 21 days per Whitepaper v2.0 |
| `S_slash` | Slash percentage for equivocation | **Defined** - 100% per Whitepaper v2.0 |

**No variable is assigned a numeric value in v1.0.**

## 2. Attack Cost Models

### 2.1 Cost to 33% Attack

**Formula**:  

Where:
- `C_acquisition` = cost to acquire stake without moving price. Function of market depth. **Undefined**
- `C_opportunity` = forgone staking rewards during unbonding. Function of APR. **Undefined**

**Status**: Formula defined. No inputs. No outputs. Simulation pending.

**Sensitivity Analysis Required**:
1. What if `P` drops 90% during acquisition?
2. What if `S_total` = 10M vs 100M vs 500M QTC?
3. What if `C_acquisition` includes 50% slippage?

**Results**: Pending. See v1.1.

### 2.2 Cost to 66% Attack

**Formula**:  
Where:
- `C_slash_risk` = Expected value of slashed stake. Function of detection probability. **Undefined**

**Status**: Formula defined. No inputs. No outputs. Simulation pending.

### 2.3 Sybil Attack via TEE Compromise

**Formula**:  
Where:
- `N` = Number of validators attacker wants
- `C_phone` = Hardware cost per device. **Undefined**
- `C_exploit` = Cost to develop + deploy root exploit. **Undefined**
- `R_success` = Success rate of exploit on target phones. **Undefined**

**Status**: Formula defined. No inputs. No outputs. Simulation pending.

**Critical Question**: If `R_success` = 0.01, is attack viable? Analysis pending.

### 2.4 Griefing Attack - Liveness Halt

**Formula**:  
Where:
- `T_halt` = Duration of attack in days
- Attacker loses staking rewards + faces slashing risk if detected

**Status**: Formula defined. No inputs. No outputs. Simulation pending.

## 3. Pending Simulation Matrix

The following simulations must be completed before mainnet:

| **ID** | **Simulation** | **Status** | **Blocking for Mainnet** |
| --- | --- | --- | --- |
| SIM-01 | 33% attack cost, `N_val` = 1,000 | **Pending** | Yes |
| SIM-02 | 33% attack cost, `N_val` = 10,000 | **Pending** | Yes |
| SIM-03 | 66% attack cost, `S_total` = 100M | **Pending** | Yes |
| SIM-04 | Sybil cost, `R_TEE` = 1% | **Pending** | Yes |
| SIM-05 | Sybil cost, `R_TEE` = 10% | **Pending** | Yes |
| SIM-06 | Griefing cost, 7-day halt | **Pending** | No |
| SIM-07 | Price crash impact on security | **Pending** | Yes |
| SIM-08 | Stake concentration Gini > 0.8 | **Pending** | Yes |

**Mainnet Launch Criteria**: SIM-01 through SIM-05 + SIM-07 + SIM-08 must be completed and results published.

**Current Progress**: 0 of 8 complete.

## 4. Limitations of Economic Security

Even after simulations complete, economic security has fundamental limits:

1. **Reflexivity**: If attack succeeds, QTC price → 0. Slashing worthless. Security fails.
2. **External Incentives**: Nation-state may pay > profit to break chain. Cannot model.
3. **Bootstrapping**: Early chain has low `S_total`. Attack cheap. Mitigation: Centralized checkpointing Phase 0.
4. **TEE Assumptions**: If `R_TEE` > 50%, Sybil cost collapses. Mobile consensus fails.

**Conclusion**: Economic security is not cryptographic security. Do not confuse.

## 5. Next Documents

1. **ECONOMIC_SIM v1.1** - First simulation results for SIM-01. ETA: Q1 2026
2. **BENCHMARKS.md** - Hardware costs for `C_phone`, `C_exploit`. Required input for this doc.
3. **GOVERNANCE.md** - How to upgrade parameters if simulations show insecurity

## 6. How to Contribute

Economic modeling help wanted. Open issue with label `research`.  
**Warning**: Do not submit pull requests with price predictions or investment advice. Will be closed.

---

**This document contains zero financial data.**  
**All attack costs are undefined pending research.**  
**We do not claim economic security. We claim transparency about the lack thereof.**

End of ECONOMIC_SIM.md v1.0