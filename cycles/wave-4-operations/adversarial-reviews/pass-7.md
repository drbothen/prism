---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 7
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T18:00:00Z
predecessor: pass-6.md (BLOCKED 5 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 5
severity_breakdown: { CRITICAL: 0, HIGH: 1, MEDIUM: 2, LOW: 2, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [15fa97e6]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 7

**Verdict: BLOCKED** | 5 findings (0C / 1H / 2M / 2L / 0OBS)

Trajectory: 38→17→8→7→7→5→5 (descent stalled — consumer-table sweep gap pattern)

---

## P7-HIGH-001 — S-4.08 line 88 BC table title stale (partial-fix regression from Pass 6)

**Severity:** HIGH
**File:** stories/S-4.08-action-delivery.md, line 88
**Finding:** The BC Traceability table in S-4.08 still renders the old title for BC-2.18.004. Pass 6
remediated BC-2.18.004 itself (v1.3→v1.4 with H1 title change) and updated BC-INDEX to reflect the
new canonical title. However, the consumer table inside S-4.08 was not swept as part of the Pass 6
remediation burst. This is a recurrence of the partial-fix regression class: index-level fixes applied
without propagating to story-level consumer tables.

**Root cause:** Pass 6 remediation scope covered BC-INDEX H1 sync but did not include a corpus-wide
grep for the stale BC-2.18.004 title string in story BC traceability tables. The consumer-table sweep
gap (identified as a pattern in S-3.x passes) has re-manifested here.

**Required fix:** Update S-4.08 line 88 BC table title for BC-2.18.004 to match the canonical title
now in BC-2.18.004 v1.4 and BC-INDEX.

---

## P7-MEDIUM-001 — VP catalog totals inconsistency (verification-coverage-matrix.md)

**Severity:** MEDIUM
**File:** specs/architecture/verification-coverage-matrix.md
**Finding:** The verification-coverage-matrix VP count totals contain an inline comment noting
reconciliation against the VP-145 addition that occurred in Pass 1 remediation. The comment was
authored as a placeholder during Pass 5 but was not replaced with a clean numeric reconciliation.
The HTML comment form is acceptable as a process note but introduces reader ambiguity about whether
the totals are ground-truth or pending update.

**Required fix:** Reconcile VP totals in verification-coverage-matrix.md using an HTML comment that
explicitly anchors the count to the VP-145 addition, making the reconciliation unambiguous for the
next adversarial pass. Version bump to v1.28.

---

## P7-MEDIUM-002 — BC-2.12.004 modified field and EC-12-010 tick note not reflected

**Severity:** MEDIUM
**File:** specs/behavioral-contracts/BC-2.12.004-schedule-execution-loop.md
**Finding:** BC-2.12.004 body references the execution loop tick interval but does not include the
generalization note captured in EC-12-010 (tick interval generalization to configurable cadence).
The `modified` field in frontmatter also does not reflect the most recent authoring touch from the
Pass 6 sweep.

**Required fix:** Update BC-2.12.004 frontmatter `modified` field and append EC-12-010 tick
generalization note to the relevant contract section. Bump version v1.4→v1.5.

---

## P7-LOW-001 — VP-INDEX version drift (process-gap — deferred)

**Severity:** LOW
**File:** specs/verification-properties/VP-INDEX.md
**Finding:** VP-INDEX version field has drifted from the expected version derived from the VP-145
addition in Pass 1. The version drift is a bookkeeping artifact of the pass-handoff protocol not
deriving VP-INDEX version from the file's own frontmatter.

**Disposition:** DEFERRED as process-gap. The pass-handoff scope should derive versions from files
rather than carrying forward a hardcoded version pin. This is a methodology gap (TD-VSDD candidate)
not a spec correctness issue. No VP-INDEX content change required.

---

## P7-LOW-002 — EC-12-010 tick generalization note scope

**Severity:** LOW
**File:** specs/behavioral-contracts/BC-2.12.004-schedule-execution-loop.md
**Finding:** The EC-12-010 tick generalization note added in Pass 7 remediation is appropriately
scoped but the note could be misconstrued as changing the normative tick-interval constraint. The
note should be prefaced with a non-normative marker (NOTE: or INFORMATIVE:) to prevent consumers
from treating the generalization as a contract amendment.

**Required fix:** Prefix EC-12-010 note with "NOTE (non-normative):" or equivalent marker. Folded
into P7-MEDIUM-002 fix.

---

## Remediation Summary

| Finding | Fix | Story/BC/Arch Touched |
|---------|-----|----------------------|
| P7-HIGH-001 | S-4.08 line 88 BC table title sync (BC-2.18.004 title match) | S-4.08 v1.14→v1.15 |
| P7-MEDIUM-001 | verification-coverage-matrix VP totals comment reconciled with VP-145 | verification-coverage-matrix.md v1.27→v1.28 |
| P7-MEDIUM-002 + P7-LOW-002 | BC-2.12.004 modified field + EC-12-010 tick note with non-normative marker | BC-2.12.004 v1.4→v1.5 |
| P7-LOW-001 | DEFERRED — VP-INDEX version drift is process-gap (pass-handoff scope should derive from files) | None |

All non-deferred findings remediated 2026-05-03. Pass 8 queued.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 7 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 2 |
| **Novelty score** | 0.60 (3 / 5) |
| **Median severity** | 2.5 (between MEDIUM and HIGH) |
| **Trajectory** | 38→17→8→7→7→5→5 |
| **Verdict** | FINDINGS_REMAIN |
