# QTC Foundation Grant Template

**Use this template for all M16+ contributor grants.**
Copy to `docs/grants/GRANT_[MILESTONE]_[NAME].md` and submit as a GitHub PR.

---

## Grant Agreement — [GRANT NUMBER]

**Recipient:** [Legal name or pseudonym]
**GitHub:** [@handle]
**Wallet Address:** [0x + 64 hex chars — your QTC address]
**Role:** [e.g. Protocol Engineer, M17 State Pruning]
**Term:** [Start date] – [End date]
**Signed by:** 5/7 Foundation Multisig

---

## Milestone Definition

**Milestone:** [e.g. M17 — State Pruning + Snapshots]

**Done when:**
- [ ] [Specific, verifiable criterion 1]
- [ ] [Specific, verifiable criterion 2]
- [ ] [Specific, verifiable criterion 3]
- [ ] All tests passing in CI
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Code merged to `main` branch of relevant repo

**Proposed timeline:** [X weeks/months from grant approval]
**Deadline (2x rule):** [2x proposed timeline — clawback triggers after this]

---

## Payment Schedule

All payments are milestone-gated. No upfront payments.

| Sub-milestone | Amount | Release condition |
|---|---|---|
| [Sub-milestone 1 name] | 250,000 QTC | Merged + 5/7 signed |
| [Sub-milestone 2 name] | 250,000 QTC | Merged + 5/7 signed |
| [Sub-milestone 3 name] | 250,000 QTC | Merged + 5/7 signed |
| [Sub-milestone 4 name] | 250,000 QTC | Merged + 5/7 signed |
| **Total** | **1,000,000 QTC** | |

**Vesting:** Each sub-milestone payment vests linearly over 6 months from release date.

**Source wallet:** `0xDAO` (Strategic Reserve)

---

## Conditions

1. **MIT License.** All code produced under this grant must be MIT licensed and assigned to the QuantumChain Foundation. No exceptions.

2. **No upfront payment.** Zero QTC is released before a sub-milestone is merged and approved by 5/7 multisig.

3. **Clawback.** If the full milestone is not delivered within the 2x deadline, all unvested tokens return to `0xDAO`. Partially delivered sub-milestones are paid for work delivered only.

4. **Recusal.** The grant recipient cannot vote on this grant proposal or any proposal that benefits them for 90 days after receipt.

5. **Public work.** All work must be done publicly on GitHub. No private forks that are later submitted as grant deliverables.

6. **Quit anytime.** The recipient may stop at any time. They keep all vested tokens and grants earned to date. Unvested tokens return to `0xDAO`.

7. **No exclusivity.** The recipient may work on other projects. QTC Foundation makes no claim on work done outside this grant scope.

8. **Conduct.** The recipient agrees not to use their position to manipulate governance, collude with other multisig holders, or act against the interests of the validator community.

9. **Review SLA.** Multisig must review any submitted PR within 7 days of submission. If no review or response is recorded on-chain within 7 days, payment for that sub-milestone auto-releases on Day 8 via timelock contract. Multisig cannot block payment by simply ignoring work.

10. **Dispute Resolution.** If the recipient believes a milestone is complete but multisig disputes it: any 3 of 7 multisig holders may trigger a formal dispute. The external arbiter is the highest-staked non-multisig validator at the time of dispute. Their decision is final and binding. The arbiter is paid 10,000 QTC from `0xDAO` regardless of outcome. Neither party may appeal the arbiter's decision through governance for 90 days.

---

## Recusal Declaration

*By submitting this grant for approval, the recipient acknowledges that any multisig seat they hold is automatically recused from the approval vote for this specific grant.*

---

## Approval Record

| Multisig Seat | Holder | Vote | Date |
|---|---|---|---|
| Seat 1 — Founder | Touqeer | [ ] Yes [ ] No [ ] Recused | |
| Seat 2 — First Dev | Touqeer | [ ] Yes [ ] No [ ] Recused | |
| Seat 3 — Top Staker | [auto-assigned] | [ ] Yes [ ] No [ ] Recused | |
| Seat 4 — Top Contributor | [auto-assigned] | [ ] Yes [ ] No [ ] Recused | |
| Seat 5 — Community | [elected] | [ ] Yes [ ] No [ ] Recused | |
| Seat 6 — Community | [elected] | [ ] Yes [ ] No [ ] Recused | |
| Seat 7 — Reserve | [DAO assigned] | [ ] Yes [ ] No [ ] Recused | |

**Result:** [ ] Approved (≥5/7) [ ] Rejected
**Effective date:** [date]

---

## Recipient Acknowledgment

*I have read and agree to the conditions above. I understand that payment is milestone-gated, that unvested tokens are subject to clawback, and that all code I produce under this grant is MIT licensed.*

**Signed:** [Recipient signature / wallet signature]
**Date:** [date]
