---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/vp-027-alert-dedup-key.md
  - .factory/specs/verification-properties/vp-033-audit-buffer-write-before-delivery.md
  - .factory/specs/verification-properties/vp-036-session-context-drop.md
  - .factory/specs/behavioral-contracts/BC-2.13.003-correlation-detection.md
  - .factory/specs/behavioral-contracts/BC-2.13.013-alert-deduplication.md
  - .factory/specs/behavioral-contracts/BC-2.05.011-audit-forwarding-at-least-once.md
  - .factory/specs/behavioral-contracts/BC-2.15.003-buffered-audit-log-persistence.md
  - .factory/specs/behavioral-contracts/BC-2.15.007-watchdog-query-termination.md
  - .factory/policies.yaml
input-hash: "466bc09"
traces_to: ""
pass: 85
counter_before: 0
counter_after: 0
findings_total: 4
findings_critical: 1
findings_high: 1
findings_medium: 2
findings_low: 0
observations: 1
convergence_recommendation: RESET
---

# Adversarial Review — Pass 85 (Phase 2 Patch)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |
| OBSERVATIONS | 1 |

Pass-84 remediation verified clean (arithmetic reconciles, VP-056 re-anchor correct, column header rename applied). Pass-85 fresh-context deep dive into VP source files (not examined in prior passes) surfaced 3 new mis-anchors at the VP `source_bc:` frontmatter level plus 1 changelog off-by-one.

## Findings

### F85-001 — CRITICAL — VP-027 source_bc points at wrong BC

- Files: vp-027-alert-dedup-key.md line 12 `source_bc: BC-2.13.003` + body line 46 label; BC-2.13.003 is "Correlation Detection" (not dedup); verification-architecture.md line 139 correctly anchors VP-027 to BC-2.13.013 (Alert Deduplication).
- Remediation (architect): VP-027 frontmatter `source_bc: BC-2.13.003` → `BC-2.13.013`; update body "Source BC" label to match; bump VP version.

### F85-002 — HIGH — VP-033 source_bc points at BC-2.05.011; correct is BC-2.15.003

- Files: vp-033-audit-buffer-write-before-delivery.md line 12 `source_bc: "BC-2.05.011"`; BC-2.15.003 back-references VP-033 explicitly; BC-2.05.011 does NOT.
- BC-2.15.003 title: "Buffered Audit Log Persistence — Write to RocksDB Before stderr/Vector" — exact semantic match for VP-033.
- Remediation (architect): VP-033 frontmatter `source_bc: "BC-2.05.011"` → `"BC-2.15.003"`; bump VP version. Optionally add BC-2.05.011 as secondary reference.

### F85-003 — MEDIUM — verification-coverage-matrix.md v1.7 changelog claims "24 rows" but table has 23

- File: verification-coverage-matrix.md line 123 changelog v1.7 says "BC-level Invariant Properties table expanded from 1 row to 24 rows". Actual count (lines 95-117): 23 rows.
- verification-architecture.md v1.9 changelog correctly says "23 rows".
- Remediation (architect): correct v1.7 changelog text "24 rows" → "23 rows".

### F85-004 — MEDIUM — VP-036 source_bc not reciprocated by BC-2.15.007

- Files: vp-036-session-context-drop.md line 12 `source_bc: "BC-2.15.007"`; BC-2.15.007 VP Anchors (line 102) lists ONLY VP-058, not VP-036.
- Semantic fit is defensible (BC-2.15.007 covers SessionContext cancellation via CancellationToken) but bidirectional anchor broken.
- Remediation (architect): add VP-036 to BC-2.15.007 VP Anchors section as integration_test designation; OR re-anchor VP-036 to more specific BC and update both sides.

## Observations

### OBS-85-001 — 12 VPs P1 while source BCs P0 (priority-tier mismatch cluster)

- VPs: VP-029, VP-040-043, VP-048, VP-049, VP-054, VP-055, VP-056, VP-061, VP-062.
- Source BCs: BC-2.07.002, BC-2.17.002/003/005/006, BC-2.19.001/002, BC-2.14.008, BC-2.15.002/004, BC-2.20.002/003 — all P0.
- Likely design intent: formal VPs land in hardening even for P0 behaviors. If intentional, document the convention in verification-architecture.md to prevent rediscovery.
- Remediation (architect): add one-paragraph convention note explaining VP-priority ≠ BC-priority design decision.

## Pass-84 Verification

| Pass-84 Item | Status |
|--------------|--------|
| F84-001 VP-056 re-anchor BC-2.15.004 | VERIFIED ✓ |
| F84-002 coverage-matrix BC-level table extension | VERIFIED ✓ (23 rows populated; off-by-one in changelog is F85-003) |
| F84-003 Column header "Source Invariant / BC" | VERIFIED ✓ |

## Arithmetic & Version Pins

All PASS. VP-INDEX 62 = 26K+28P+6F+2I = 43 P0 + 19 P1. STATE.md pins match. OBS-082-004 SS-20 DI anchors verified resolved.

## Policy Rubric

| Policy | Verdict |
|--------|---------|
| 1. append_only_numbering | PASS |
| 2. lift_invariants_to_bcs | PASS |
| 3. state_manager_runs_last | PASS |
| 4. semantic_anchoring_integrity | FAIL — F85-001, F85-002, F85-004 |
| 5. creators_justify_anchors | PASS |
| 6. architecture_is_subsystem_name_source_of_truth | PASS |
| 7. bc_h1_is_title_source_of_truth | PASS |
| 8. bc_array_changes_propagate_to_body_and_acs | PASS |
| 9. vp_index_is_vp_catalog_source_of_truth | PASS |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 85 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (4/4) |
| **Median severity** | HIGH |
| **Trajectory** | p80=9 → p81=10 → p82=7 → p83=6 → p84=3 → p85=4 (uptick from deep-dive into VP source file frontmatter; prior passes stayed index-level) |
| **Verdict** | FINDINGS_REMAIN |

## Counter Recommendation

**RESET 0/3.** 1 CRITICAL + 1 HIGH + 2 MED block advancement. Mis-anchoring per policy 4 is never deferrable.
