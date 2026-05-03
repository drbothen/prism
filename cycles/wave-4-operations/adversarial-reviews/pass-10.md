---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 10
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T23:45:00Z
predecessor: pass-9.md (BLOCKED 6 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 5
severity_breakdown: { CRITICAL: 0, HIGH: 2, MEDIUM: 2, LOW: 1, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [40458029]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 10

**Verdict:** BLOCKED — 5 findings (0C/2H/2M/1L/0OBS)

**Trajectory:** 38→17→8→7→7→5→5→6→6→5 (continued descent; sister-row sweep regression class)

**Window:** 0/3 (reset; pass-10 BLOCKED)

---

## Findings

### HIGH

#### F-P10-H-001 — ARCH-INDEX index drift

**ID:** F-P10-H-001
**Severity:** HIGH
**Category:** Index drift — ARCH-INDEX ADR-016 version lag

**Description:**
ARCH-INDEX records ADR-016 at v0.5 (line 83) while the canonical ADR-016 file is at v0.6
following the Pass 9 remediation. This is a sibling-row sweep gap — the ARCH-INDEX changelog
row for ADR-016 was not updated when ADR-016 body was bumped. The index is the cross-reference
surface that reviewers and consistency validators read first; stale version annotations here
propagate false confidence that v0.5 is current truth.

**Policy violated:** POL-3 (index integrity — index version pins must match file frontmatter
versions); POL-8 (sweep completeness — sibling documents must be updated atomically with primary
changes).

**Evidence:** ARCH-INDEX line 83: `ADR-016 | v0.5 | ...` vs ADR-016 frontmatter: `version: "0.6"`.

**Required fix:** ARCH-INDEX v2.7→v2.8: update line 83 ADR-016 row from v0.5 to v0.7 (note:
simultaneously ADR-016 is being remediated from v0.6→v0.7 in this burst, so the target is v0.7).

---

#### F-P10-H-002 — ADR-016 §2.5 retry-state sister-row sweep gap

**ID:** F-P10-H-002
**Severity:** HIGH
**Category:** Spec completeness — retry-state table structural omission

**Description:**
ADR-016 §2.5 retry-state CF row was amended in Pass 9 to add `{idempotency_key}` as the sort
key suffix but the sister row that defines the value schema was not updated to reflect that the
idempotency key has been promoted to the sort key. The value field still implies idempotency_key
lives in the value payload, contradicting the sort-key placement in the same row. This creates
ambiguity: a reader cannot determine whether `idempotency_key` appears only in the sort key, only
in the value, or in both. Implementations that follow the value-schema description will produce
a redundant encoding; implementations that follow the sort-key description will omit it from
value and violate the BC spec expectations.

**Policy violated:** POL-1 (non-contradiction — a spec MUST NOT assert two conflicting layouts
for the same field in the same section); POL-4 (implementability — ambiguous specs cannot be
mechanically followed by a code author).

**Evidence:** ADR-016 §2.5 retry-state table: sort key includes `{idempotency_key}` but value
schema row still describes idempotency_key as a value-field component.

**Required fix:** ADR-016 v0.6→v0.7: update value-schema description in §2.5 retry-state row
to note that `idempotency_key` is carried in the sort key (not repeated in value), and clarify
that value stores only the retry attempt count, next_retry_at, and last_error fields.

---

### MEDIUM

#### F-P10-M-001 — verification-architecture §11 reference not updated to §2.11

**ID:** F-P10-M-001
**Severity:** MEDIUM
**Category:** Cross-reference drift — section number mismatch

**Description:**
The verification-architecture document contains an internal cross-reference to "§11" for the
Holdout Evaluation annex. Following the document restructuring that created the §2.x section
hierarchy, the Holdout Evaluation material was moved to §2.11. References still citing bare
"§11" are broken: a reader navigating to §11 arrives at the wrong section or at a section that
does not exist at that level. In a 25+ section document this is a usability failure and a
correctness failure for downstream tooling that parses section anchors.

**Policy violated:** POL-3 (cross-reference integrity); POL-7 (H1/section pointer currency).

**Evidence:** verification-architecture line 272: `§11` (bare section reference without §2.
prefix).

**Required fix:** verification-architecture v1.24→v1.25: update line 272 reference from §11 to
§2.11.

---

#### F-P10-M-002 — BC-2.18.001 EC case-trigger analog missing

**ID:** F-P10-M-002
**Severity:** MEDIUM
**Category:** BC completeness — error case analog gap

**Description:**
BC-2.18.001 documents the at-least-once delivery contract with exponential backoff retry. The
error-case table covers delivery-failure scenarios but is missing an analog for the case where
the triggering case (not alert) fails to persist its action-dispatch record before the action is
queued. This creates a gap between the BC and the observable behavioral contract: if the case-
trigger path fails mid-flight, the BC does not specify whether the retry sequence begins, is
suppressed, or emits an error. This gap was introduced in a prior pass fix that added alert-
trigger error cases without adding the symmetric case-trigger analog; it is a sister-row sweep
failure.

**Policy violated:** POL-2 (completeness — all observable failure modes must be specified);
POL-8 (sweep completeness — error cases that share structural analogy must be specified in
parallel).

**Evidence:** BC-2.18.001 error case table: EC-18-005 documents alert-trigger dispatch failure;
no symmetric EC-18-005/a or EC-18-006 exists for case-trigger dispatch failure path.

**Required fix:** BC-2.18.001 v1.6→v1.7: add EC-18-005/a (case-trigger action-dispatch record
persistence failure before queue) as a symmetric analog to EC-18-005; specify that retry
sequence behavior is identical to alert-trigger path.

---

### LOW

#### F-P10-L-001 — ARCH-INDEX changelog text — stale pass reference

**ID:** F-P10-L-001
**Severity:** LOW
**Category:** Documentation accuracy — changelog narrative stale

**Description:**
The ARCH-INDEX changelog entry for the v2.7 row references "Pass 9 remediation" as the source
of change but does not enumerate which ADR was updated. Following the ADR-016 version bump in
this burst the v2.8 changelog row must identify the specific ADR (ADR-016 v0.5→v0.7) to
maintain the audit trail completeness that the ARCH-INDEX changelog is expected to provide.
This is LOW because the changelog is informational-only; the index table is the normative data.

**Policy violated:** POL-6 (audit trail — changelog rows must identify the specific artifact
changed, not just the pass number).

**Required fix:** ARCH-INDEX v2.8 changelog row: enumerate ADR-016 v0.5→v0.7 explicitly in
the change description.

---

## Remediation Summary

| Finding | Fix | Status |
|---------|-----|--------|
| F-P10-H-001 | ARCH-INDEX v2.7→v2.8 (line 83 ADR-016 v0.5→v0.7) | REMEDIATED 2026-05-03 |
| F-P10-H-002 | ADR-016 v0.6→v0.7 (§2.5 retry-state {idempotency_key} sort-key clarification) | REMEDIATED 2026-05-03 |
| F-P10-M-001 | verification-architecture v1.24→v1.25 (line 272 §11→§2.11) | REMEDIATED 2026-05-03 |
| F-P10-M-002 | BC-2.18.001 v1.6→v1.7 (line 58 + EC-18-005/a case-trigger analog) | REMEDIATED 2026-05-03 |
| F-P10-L-001 | ARCH-INDEX changelog text (ADR-016 v0.5→v0.7 enumerated) | REMEDIATED 2026-05-03 |

**All 5 findings remediated. Pass 11 queued.**

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 10 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 5/5 = 1.0 |
| **Median severity** | 2.4 (2H/2M/1L) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5 |
| **Verdict** | FINDINGS_REMAIN |

---

## Convergence Trajectory

| Pass | Findings | Delta | Notes |
|------|----------|-------|-------|
| 1 | 38 | — | Initial; 0C/11H/17M/7L/3OBS |
| 2 | 17 | -21 | |
| 3 | 8 | -9 | |
| 4 | 7 | -1 | |
| 5 | 7 | 0 | flat; arch aggregates class |
| 6 | 5 | -2 | |
| 7 | 5 | 0 | flat; BC sweep class |
| 8 | 6 | +1 | REGRESSION; sibling-row sweep |
| 9 | 6 | 0 | flat; sister-row sweep class |
| 10 | 5 | -1 | 0C/2H/2M/1L; continued descent |
