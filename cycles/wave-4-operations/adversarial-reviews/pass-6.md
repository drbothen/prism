---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 6
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T03:00:00Z
predecessor: pass-5.md (BLOCKED 7 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 5
severity_breakdown: { CRITICAL: 0, HIGH: 4, MEDIUM: 1, LOW: 0, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [<Stage 1 SHA>]
---

# Adversarial Review — Phase 4.A Pass 6

## Verdict: BLOCKED

5 findings (0C / 4H / 1M / 0L / 0OBS). Convergence window reset to 0/3.

Trajectory: 38→17→8→7→7→5

## Findings

### HIGH-001 — BC-2.18.004: 16-permit shared pool not corrected to 8-permit independent

**Severity:** HIGH
**Artifact:** `specs/behavioral-contracts/BC-2.18.004-action-schedule-semaphore.md`
**Class:** Partial-fix regression — ADR-remediation workflow lacked structural BC-body sweep.

**Finding:** BC-2.18.004 H1 heading was updated to reference "8-Permit Independent Pool" and
"try_acquire() Skip-If-Unavailable" in prior remediation passes, but the BC body still referenced
the original 16-permit shared semaphore design. Specifically:

- The body specification described `Arc<Semaphore>` with `16` permits shared across all action
  clients for a given org, contradicting the ADR-016 decision for independent per-client pools.
- The `try_acquire()` skip-if-unavailable semantics were missing from the body invariants.
- Version was pinned at v1.3, not advanced to v1.4 to reflect the body correction.

**Required fix:** Sweep BC-2.18.004 body to replace all 16-permit shared references with
8-permit independent pool per ADR-016. Add try_acquire() skip semantics to body invariants.
Advance to v1.4.

---

### HIGH-002 — BC-2.12.004: 1s tick + 16-permit carry-forward in schedule execution loop

**Severity:** HIGH
**Artifact:** `specs/behavioral-contracts/BC-2.12.004-schedule-execution-loop.md`
**Class:** Partial-fix regression — same ADR-remediation gap.

**Finding:** BC-2.12.004 referenced a 1-second tick interval for the schedule execution loop
and a 16-permit semaphore gate. Per ADR-013 (Schedule Execution Semantics), the canonical tick
is 60 seconds and the permit count is 8 (per-subsystem). The BC body had not been swept after
ADR-013 acceptance; prior remediation only updated the H1 heading line.

**Required fix:** Update BC-2.12.004 body: replace 1s tick with 60s tick, replace 16-permit with
8-permit, align with ADR-013 normative semantics. Advance to v1.4.

---

### HIGH-003 — BC-2.18.001: Retired retry sequence still present in body

**Severity:** HIGH
**Artifact:** `specs/behavioral-contracts/BC-2.18.001-action-at-least-once-delivery.md`
**Class:** Partial-fix regression — body not swept after ADR-016 retry-sequence decision.

**Finding:** BC-2.18.001 body described the at-least-once delivery retry sequence using legacy
intervals (1s, 2s, 4s, 8s) that were superseded by ADR-016's standard backoff schedule
(2s / 4s / 8s / 16s / 32s cap). The heading had been aligned in a prior pass but the body
invariants still carried the retired sequence, producing a heading/body contradiction.

**Required fix:** Sweep BC-2.18.001 body to adopt ADR-016 standard backoff (2/4/8/16/32s).
Remove retired legacy intervals. Advance to v1.4.

---

### HIGH-004 — BC-2.18.002: 1s tick + 16-permit in action schedule best-effort BC

**Severity:** HIGH
**Artifact:** `specs/behavioral-contracts/BC-2.18.002-action-schedule-best-effort.md`
**Class:** Partial-fix regression — same structural gap as HIGH-002.

**Finding:** BC-2.18.002 body retained 1-second tick and 16-permit semaphore references
consistent with the pre-ADR-013/ADR-016 design. The best-effort scheduling BC had not received
the same body sweep applied to the execution loop BC, leaving two BCs with contradictory
semaphore semantics (BC-2.18.002 says 16-permit shared; BC-2.18.004 H1 says 8-permit
independent).

**Required fix:** Sweep BC-2.18.002 body: replace 1s tick with 60s tick, 16-permit with
8-permit independent per ADR-016. Advance to v1.4.

---

### MEDIUM-001 — VP-053: Module mis-attribution in verification-coverage-matrix

**Severity:** MEDIUM
**Artifact:** `specs/architecture/verification-coverage-matrix.md`
**Class:** Index/matrix propagation gap.

**Finding:** VP-053 (Case Lifecycle State Machine invariants) was attributed to module
`prism-core` in the verification-coverage-matrix. VP-053 traces to the Case Management story
(S-4.06) and its implementation target is `prism-operations`. The mis-attribution would cause
the VP to be associated with the wrong crate during Phase 3 implementation, potentially
misdirecting test placement.

**Required fix:** Update verification-coverage-matrix VP-053 row: change module from
`prism-core` to `prism-operations`.

---

## Process Gap Note

All 4 HIGH findings share a root cause: the ADR-remediation workflow applied fixes to BC
headings and index entries without performing a structural sweep of BC body text. When ADR-013
and ADR-016 changed canonical values (tick interval, permit count, retry backoff), the workflow
did not propagate those changes into the full BC body — only the most visible surface (H1 title,
BC-INDEX entry) was updated.

**Corrective discipline added:** Future ADR-acceptance remediation bursts MUST include a
structured BC-body sweep: for each BC cited in the ADR's "Behavioral Contract Alignment" section,
re-read the full body and verify all normative values match the ADR's accepted parameters before
advancing version.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 6 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (5/5) — all partial-fix-regression class; new surface, same root cause |
| **Median severity** | 3.5 (4 HIGH / 1 MEDIUM) |
| **Trajectory** | 38→17→8→7→7→5 |
| **Verdict** | FINDINGS_REMAIN |

---

## Remediation Summary (Completed 2026-05-03)

| Finding | Artifact | Fix | Version |
|---------|----------|-----|---------|
| HIGH-001 | BC-2.18.004 | 16-permit→8-permit independent; try_acquire() skip semantics | v1.3→v1.4 |
| HIGH-002 | BC-2.12.004 | 1s→60s tick; 16-permit→8-permit per ADR-013 | v1.3→v1.4 |
| HIGH-003 | BC-2.18.001 | Retired retry seq replaced with ADR-016 standard backoff 2/4/8/16/32s | v1.3→v1.4 |
| HIGH-004 | BC-2.18.002 | 1s→60s tick; 16-permit→8-permit per ADR-016 | v1.3→v1.4 |
| MEDIUM-001 | coverage-matrix | VP-053 module prism-core→prism-operations | — |

BC-INDEX H1 sync: BC-2.18.004 entry updated to reflect 8-Permit Independent Pool heading.

Stage 1 remediation commit: `<Stage 1 SHA>`
