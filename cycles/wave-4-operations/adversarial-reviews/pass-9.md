---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 9
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T22:00:00Z
predecessor: pass-8.md (BLOCKED 6 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 6
severity_breakdown: { CRITICAL: 0, HIGH: 2, MEDIUM: 3, LOW: 1, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [<Stage 1 SHA>]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 9

**Verdict: BLOCKED** — 6 findings (0C / 2H / 3M / 1L / 0OBS). Convergence window reset (0/3).

Trajectory: 38→17→8→7→7→5→5→6→6 (flat at 6; all HIGH = sibling-sweep regressions).

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 9 |
| **New findings** | 6 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 6 / (6 + 0) = 1.00 |
| **Median severity** | 2.5 (HIGH=2, MEDIUM=3, LOW=1) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6 |
| **Verdict** | FINDINGS_REMAIN |

---

## HIGH Findings

### F-P9-H-001 — Retry CF Key Sibling Sweep Gap (carry-forward from Pass 8)

**Severity:** HIGH
**Location:** S-4.08 §2 + ADR-016 §2.5 retry-state row
**Finding:** Pass 8 remediation added the retry-state row to ADR-016 §2.5 (column-family table) with key pattern `{org_id}:{client_id}:{action_id}`. However, the sibling CF keys in the same table were not swept to confirm they all use the same three-segment pattern. ADR-016 §2.5 lists `action_delivery_state` CF using `{org_id}:{client_id}:{action_id}` and `dead_letter` CF using a separate key. The dead_letter CF key pattern in the Pass 8 commit uses `idempotency_key` as the discriminating field rather than `action_id`, creating a key-space mismatch. Range scans over `{org_id}:{client_id}:` prefix will not sweep the dead-letter CF correctly when the dead-letter record was written using `idempotency_key` as the trailing segment.
**Impact:** Dead-letter records may be orphaned under failure modes where `action_id` and `idempotency_key` diverge (e.g., replay scenarios where `action_id` is reassigned but `idempotency_key` is preserved). AT-LEAST-ONCE delivery guarantee (BC-2.18.001) may be silently violated.
**Required Fix:** Unify dead-letter CF key on `{org_id}:{client_id}:{action_id}` (same as retry-state CF). Add `idempotency_key` as a value field in the dead-letter record, not the key discriminant. Update ADR-016 §2.5 table and S-4.08 AC-7/AC-8 to reflect the unified key pattern.

### F-P9-H-002 — Dead-Letter Key `event_id`/`alert_id` Contradiction

**Severity:** HIGH
**Location:** ADR-016 §3 (dead-letter contract), S-4.08 §4 (dead-letter AC)
**Finding:** ADR-016 §3 defines the dead-letter record as containing `event_id` to link back to the originating alert or case event. S-4.08's AC-7 (dead-letter AC) refers to the same field as `alert_id`. These two field names identify the same conceptual value (the originating event that triggered the action) but use inconsistent terminology. The prism-operations domain model uses `AlertId` (per BC-2.18.001 §2), not `EventId`. Using `event_id` in the ADR is a terminology drift that will cause confusion during implementation; using `alert_id` in the story while the ADR uses `event_id` creates a discrepancy that constitutes a spec inconsistency under Policy 4.
**Impact:** Implementation engineer picks the wrong field name; dead-letter serialization format diverges between ADR spec and story AC. Traceability from BC-2.18.001 → ADR-016 → S-4.08 breaks on this field.
**Required Fix:** Canonicalize to `alert_id: AlertId` throughout (ADR-016 §3 dead-letter record schema + S-4.08 AC-7 dead-letter shape). Add a note in ADR-016 §3 clarifying that "action trigger event" = `AlertId` (for alert-triggered actions) or `CaseId` (for case-triggered actions); use `trigger_id: TriggerRef` as the union field if polymorphism is needed.

---

## MEDIUM Findings

### M-001 — VP-145 BC-Table Gap in Verification Coverage Matrix

**Severity:** MEDIUM
**Location:** `.factory/specs/architecture/verification-coverage-matrix.md` VP-145 row
**Finding:** VP-145 (added in Pass 1 to ADR-017 for retry-state invariant) has a BC column entry of `—` in the verification coverage matrix. VP-145 verifies that the retry-state transitions are bounded (at-most-5-attempts invariant). This invariant is normatively defined in BC-2.18.001 §3.2. The BC column for VP-145 should reference BC-2.18.001, not be empty.
**Required Fix:** Update verification-coverage-matrix.md VP-145 row: set BC column to `BC-2.18.001`.

### M-002 — Idempotency Bullets Cleanup in ADR-016 §2.3

**Severity:** MEDIUM
**Location:** ADR-016 §2.3 (SMTP delivery idempotency bullets)
**Finding:** ADR-016 §2.3 contains a bulleted list describing idempotency constraints for SMTP delivery. Following the Pass 8 retry-state row addition, two of the bullets now overlap semantically with the §2.5 CF table prose. Specifically: bullet "Retry state stored in `retry_state` CF, keyed by `{org_id}:{client_id}:{action_id}`" duplicates the §2.5 table row without cross-reference, and bullet "Idempotency key derived from action spec hash at load time" is missing a forward reference to §3 (dead-letter record), where `idempotency_key` appears as a field. The bullets should either be removed (trusting §2.5 as SoT) or annotated with cross-references to avoid duplication drift.
**Required Fix:** Remove or collapse the duplicate bullet in §2.3; add a cross-reference to §2.5 and §3 for readers who encounter §2.3 first.

### M-003 — SMTP Auth Dev Notes in S-4.08

**Severity:** MEDIUM
**Location:** S-4.08 §4 Dev Notes section
**Finding:** S-4.08 §4 Dev Notes contain a note reading "SMTP auth order: XOAUTH2 → PLAIN → E-AD-018 fallback." This note was added in Pass 8 (v1.16) to close P8-H-001. However, the note uses implementation-guidance language ("auth order") that should appear in the Task list (as a Task constraint or sub-step), not in Dev Notes. Dev Notes per VSDD policy are for non-normative context; normative ordering constraints belong in Tasks. The current placement means the constraint may be overlooked during implementation review.
**Required Fix:** Move the SMTP auth-order constraint from Dev Notes to Task 7 (SMTP delivery implementation task) as a normative sub-step: "7a. Auth negotiation order: XOAUTH2 (preferred) → PLAIN → E-AD-018 (no-auth-available error)."

---

## LOW Findings

### L-001 — Changelog Ordering in STORY-INDEX

**Severity:** LOW
**Location:** `.factory/stories/STORY-INDEX.md` § Changelog (lines ~927-928)
**Finding:** STORY-INDEX changelog shows v1.89 (Pass 8 entry, 2026-05-03) before v1.88 (Pass 7 entry, 2026-05-03). Changelog is ordered with newest-first at the top of the table, which is correct. However, v1.89 (newest) appears at row index 927 while v1.88 (older) appears at row 928 — i.e., the two entries are in reverse order within what should be a descending-version sequence. The entry for v1.89 is correctly the latest version but it was appended above v1.88 while v1.87 and v1.86 follow v1.88 correctly. The table from v1.89 downward should read v1.89, v1.88, v1.87, v1.86... which it does. This is correct ordering. On closer inspection, the issue is that v1.89 is the current version but the table row says "v1.89 | 2026-05-03 | Wave 4 Phase 4.A Pass 8 remediation" — the version label v1.89 describes Pass 8 work, yet the STORY-INDEX frontmatter `version: "v1.89"` pins to this row. This is not an ordering error; it is an expected telescoping caused by out-of-order version bumps during Passes 5-8. For audit clarity, a note should be added to the v1.89 row indicating that v1.86-v1.88 are retroactive intermediate versions documented below.
**Required Fix:** Append "(retroactive: v1.86-v1.88 below document earlier in-pass partial bumps)" to the v1.89 row description for audit clarity. LOW priority — informational only.

---

## Remediation Record

All 6 findings remediated 2026-05-03:
- **F-P9-H-001:** ADR-016 §2.5 dead-letter CF key unified to `{org_id}:{client_id}:{action_id}`; `idempotency_key` moved to value field. S-4.08 AC-7/AC-8 updated.
- **F-P9-H-002:** Canonicalized to `alert_id: AlertId` in ADR-016 §3 + S-4.08 AC-7.
- **M-001:** verification-coverage-matrix VP-145 BC column set to `BC-2.18.001`.
- **M-002:** ADR-016 §2.3 idempotency bullets cleaned up; cross-reference to §2.5/§3 added.
- **M-003:** S-4.08 SMTP auth-order constraint moved from Dev Notes to Task 7a.
- **L-001:** STORY-INDEX v1.89 row annotated for audit clarity.
