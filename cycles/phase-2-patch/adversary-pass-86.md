---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/STATE.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/vp-005-case-state-machine.md
  - .factory/specs/verification-properties/vp-006-case-state-no-self-transitions.md
  - .factory/specs/verification-properties/vp-018-detection-rule-validation.md
  - .factory/specs/verification-properties/vp-021-prismql-parser-no-panic.md
  - .factory/specs/verification-properties/vp-025-cache-key-deterministic.md
  - .factory/specs/verification-properties/vp-026-splay-deterministic.md
  - .factory/specs/verification-properties/vp-031-required-column-enforcement.md
  - .factory/specs/verification-properties/vp-051-case-state-exhaustive.md
  - .factory/specs/behavioral-contracts/BC-2.07.005-cache-key-derivation.md
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
  - .factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md
  - .factory/specs/behavioral-contracts/BC-2.12.004-schedule-execution-loop.md
  - .factory/specs/behavioral-contracts/BC-2.13.001-detection-rule-loading.md
  - .factory/specs/behavioral-contracts/BC-2.14.002-case-state-transitions.md
  - .factory/policies.yaml
input-hash: "b645ac4"
traces_to: ""
pass: 86
counter_before: 0
counter_after: 0
findings_total: 8
findings_critical: 2
findings_high: 4
findings_medium: 2
findings_low: 0
observations: 1
convergence_recommendation: RESET
---

# Adversarial Review — Pass 86 (Phase 2 Patch)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 4 |
| MEDIUM | 2 |
| LOW | 0 |
| OBSERVATIONS | 1 |

Pass-86 went deeper than prior passes — full bidirectional anchor audit across 62 VPs × 208 BCs. Found 3 more VP source_bc mis-anchors (same bug-class as F85), 3 missing BC back-references, 1 matrix propagation gap, 1 STATE.md arithmetic error, 1 label drift.

## Findings

### F86-001 — CRITICAL — VP-018 source_bc anchored to wrong BC (Rule-to-SQL Compilation instead of Rule Loading)

- Files: vp-018-detection-rule-validation.md line 12 `source_bc: BC-2.13.009`; BC-2.13.009 title "Rule-to-SQL Compilation" (CAP-027); correct anchor is BC-2.13.001 "Detection Rule Loading — Parse PrismQL, Validate at Load Time" whose VP Anchors (line 92) already lists VP-018.
- Evidence: VP-018 property is `validate_rule(r)` returns Ok only for valid rules; BC-2.13.009 has no validation semantics (compiles predicates to DataFusion WHERE).
- Remediation (architect): VP-018 frontmatter `source_bc: BC-2.13.001`; update body "Source BC" label to canonical title.

### F86-002 — CRITICAL — VP-025 source_bc anchored to list_aliases MCP Tool (should be Cache Key Derivation)

- Files: vp-025-cache-key-deterministic.md line 12 `source_bc: BC-2.11.013`; BC-2.11.013 title "list_aliases MCP Tool"; correct anchor BC-2.07.005 "Cache Key Derivation from Push-Down Parameters" whose VP Anchors already lists VP-025.
- Evidence: VP-025 property is cache_key deterministic; BC-2.11.013 is alias listing (unrelated).
- Remediation (architect): VP-025 frontmatter `source_bc: BC-2.07.005`; update body label.

### F86-003 — HIGH — VP-031 source_bc anchored to Cross-Client Scoping (should be Sensor Filter Push-Down)

- Files: vp-031-required-column-enforcement.md line 12 `source_bc: BC-2.11.011`; BC-2.11.011 title "Cross-Client Query Scoping"; correct anchor BC-2.11.007 "Sensor Filter Push-Down" whose VP Anchors already lists VP-031.
- Remediation (architect): VP-031 frontmatter `source_bc: BC-2.11.007`; update body label.

### F86-004 — HIGH — VP-005/VP-006 missing back-refs in BC-2.14.002 (bidirectional asymmetry)

- Files: vp-005-case-state-machine.md, vp-006-case-state-no-self-transitions.md both declare `source_bc: BC-2.14.002`; BC-2.14.002 Verification Properties lists only VP-051.
- Same bug-class as F85-004.
- Remediation (architect): add VP-005 and VP-006 rows to BC-2.14.002 Verification Properties table; bump BC.

### F86-005 — HIGH — VP-051 orphaned from verification-coverage-matrix.md

- Files: verification-coverage-matrix.md line 78 DI-025 row cites VP-005, VP-006 but NOT VP-051 (which verifies DI-025 via 5×5 transition table). VP-051 is also missing from BC-level Invariant Properties table under BC-2.14.002.
- Remediation (architect): update matrix line 78 + BC-level table; bump v1.7 → v1.8.

### F86-006 — HIGH — STATE.md arithmetic error "200 + 8 = 213" (should be 208)

- File: STATE.md line 180 "200 active BCs + 8 tombstones = 213 BC files"
- Correct: 200 + 6 removed + 2 retired = 208 per BC-INDEX v4.12 frontmatter.
- Remediation (state-manager): STATE.md `= 213 BC files` → `= 208 BC files`.

### F86-007 — MEDIUM — VP-021 missing back-ref in BC-2.11.006

- Files: vp-021-prismql-parser-no-panic.md `source_bc: BC-2.11.006`; BC-2.11.006 VP Anchors lists VP-014/VP-015 but not VP-021.
- Remediation (architect): add VP-021 row to BC-2.11.006 VP Anchors.

### F86-008 — MEDIUM — VP-026 body label drift ("Schedule Splay for Load Distribution" vs canonical "Schedule Execution Loop")

- File: vp-026-splay-deterministic.md line 46 fabricates BC title.
- Remediation (architect): update body line 46 to canonical BC-2.12.004 title.

## Observations

### OBS-86-001 — Matrix "23 BC-anchored VPs" count will become 24 after F86-005 fix

- F85-003 corrected changelog from "24" to "23"; F86-005 adds VP-051 → count becomes 24 again. Architect must update the count statement in the matrix narrative when filing F86-005.

## Pass-85 Verification

| Item | Status |
|------|--------|
| F85-001 VP-027 re-anchor BC-2.13.013 | VERIFIED |
| F85-002 VP-033 re-anchor BC-2.15.003 | VERIFIED |
| F85-003 matrix changelog "24"→"23" | VERIFIED (but F86-005 will re-raise to 24) |
| F85-004 VP-036 back-ref in BC-2.15.007 | VERIFIED |
| OBS-85-001 VP priority convention note | VERIFIED (verification-architecture.md v1.10) |

## Semantic Anchoring Audit

- Total VP source_bc audit: 62 VPs × cross-check with BC-INDEX + BC VP-Anchor back-references
- Correct bidirectional: 56
- Source-BC mis-anchors: 3 (VP-018, VP-025, VP-031)
- Missing BC back-refs: 3 (VP-005, VP-006, VP-021)
- Body-label drift: 1+ (VP-026; possibly more — not exhaustively audited)

## Arithmetic Consistency

- VP-INDEX 62 = 26K+28P+6F+2I ✓
- VP-INDEX 62 = 43 P0 + 19 P1 ✓
- BC-INDEX 208 = 200 + 6 + 2 ✓
- L2-INDEX DI 28 ✓
- L2-INDEX CAP 34 ✓
- STATE.md 213 BCs ✗ (F86-006)

## Policy Rubric

| Policy | Verdict |
|--------|---------|
| 4. semantic_anchoring_integrity | FAIL — F86-001, F86-002, F86-003, F86-004, F86-007, F86-008 |
| 9. vp_index_is_vp_catalog_source_of_truth | FAIL — F86-005 |

## Novelty Assessment

| **Pass** | 86 |
|----------|------|
| New findings | 8 |
| Duplicate/variant findings | 0 |
| Novelty score | HIGH |
| Median severity | HIGH |
| Trajectory | p80=9 → p81=10 → p82=7 → p83=6 → p84=3 → p85=4 → **p86=8** (regression from deeper audit; same bug-class as F85 found in 3 more VPs + systematic bidirectional asymmetry) |
| Verdict | FINDINGS_REMAIN |

## Counter Recommendation

**RESET 0/3.** 2 CRITICAL + 4 HIGH + 2 MED. Mis-anchoring per policy 4 is never deferrable.
