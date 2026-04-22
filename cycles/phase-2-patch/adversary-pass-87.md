---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/STATE.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/stories/STORY-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/specs/architecture/purity-boundary-map.md
  - .factory/specs/verification-properties/vp-021-prismql-parser-no-panic.md
  - .factory/specs/verification-properties/vp-025-cache-key-deterministic.md
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
  - .factory/specs/behavioral-contracts/BC-2.15.003-buffered-audit-log-persistence.md
  - .factory/stories/S-1.02-entity-types.md
  - .factory/stories/S-1.11-spec-loading.md
  - .factory/stories/S-1.14-infusion-specs.md
  - .factory/stories/S-1.15-wasm-runtime.md
  - .factory/stories/S-2.02-audit-buffer-watchdog.md
  - .factory/stories/S-3.04-alias-system.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/stories/S-5.03-resources-prompts.md
  - .factory/stories/S-5.10-audit-trail-forwarding.md
  - .factory/policies.yaml
input-hash: "4e3184f"
traces_to: ""
pass: 87
counter_before: 0
counter_after: 0
findings_total: 6
findings_critical: 0
findings_high: 3
findings_medium: 3
findings_low: 0
observations: 0
convergence_recommendation: RESET
---

# Adversarial Review — Pass 87 (Phase 2 Patch)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 3 |
| LOW | 0 |

Pass-87 deeper audit: 1 direct pass-86 regression, 1 cross-subsystem semantic mis-anchor, 1 systematic frontmatter-body drift (18 VPs × 9 stories), 3 secondary patterns.

## Findings

### F87-001 — HIGH — VP-021 method mis-labeled as `proptest` in BC-2.11.006 (DIRECT pass-86 regression)

- Files: BC-2.11.006-query-security-limits.md line 86 row `| VP-021 | ... | proptest |`. VP-INDEX, VP-021 file frontmatter, verification-architecture.md, verification-coverage-matrix.md all say `fuzz`.
- Pass-86 F86-007 fix added the row but used wrong method.
- Remediation (architect): change `proptest` → `fuzz` on BC-2.11.006 line 86.

### F87-002 — HIGH — VP-025 cross-subsystem semantic mis-anchor (cache_key vs aliases.toml)

- Files: vp-025-cache-key-deterministic.md (body: cache_key for BC-2.07.005 / prism_query::cache::cache_key). Anchor story S-3.04 (alias system) line 316-321 redefines: "VP-025 previously referenced RocksDB key derivation — this proof now targets aliases.toml TOML table key construction".
- Two different functions in two different subsystems. Implementer confusion guaranteed.
- Remediation (architect decision): Decide whether VP-025 is cache_key (BC-2.07.005, S-3.05) or alias_key (alias BC, S-3.04). Orchestrator recommends OPTION A: VP-025 IS cache_key (matches body + source_bc); re-anchor VP-025 from S-3.04 → S-3.05 in VP-INDEX + VP file + S-3.04 frontmatter; remove the "note" redefinition from S-3.04. If a new alias-key VP is needed, file VP-063 separately.

### F87-003 — HIGH (pattern flag) — 18 VPs across 9 stories in frontmatter but absent from body

- 9 stories: S-1.02 (3 VPs missing), S-1.11 (1), S-1.14 (2), S-1.15 (4), S-2.02 (1), S-4.06 (3), S-4.08 (4), S-5.03 (1), S-5.10 (1). Total 18+ VPs not materialized as body tasks/ACs/table rows.
- Pattern originated in pass-70/77 frontmatter propagation that did not body-propagate.
- Remediation (story-writer): For each story × missing VP: add Verification Properties table row + Task + AC citing the VP. Follow S-1.08/S-5.09 canonical template.

### F87-004 — MEDIUM — Module naming: `prism-persistence` (VP/arch docs) vs `prism-storage` (everything else)

- Files: VP-INDEX, vp-055/057/058, verification-architecture, verification-coverage-matrix use `prism-persistence`. ARCH-INDEX, module-decomposition, purity-boundary-map, all storage stories use `prism-storage`.
- Remediation (architect): replace `prism-persistence` with `prism-storage` across VP/arch docs.

### F87-005 — LOW-MEDIUM — BC-2.15.003 VP-033 uses placeholder "(none)" row rather than proper row

- File: BC-2.15.003 lines 96-98 VP table has `| (none) | Persist-before-forward ... transitively by VP-033 ...`.
- Remediation (architect): replace with `| VP-033 | Audit buffer: RocksDB write completes before delivery attempt | integration_test |`.

### F87-006 — MEDIUM (pattern) — Corpus-wide VP body "Source BC" label drift vs canonical BC-INDEX titles

- 11+ VPs have paraphrased/truncated labels. Examples: VP-005/006 ("," vs "with"); VP-016/017/022 (paraphrased); VP-029 (truncated); VP-034/035 (paraphrased); VP-047 (paraphrased); VP-051/058 (truncated).
- Pass-86 F86-008 fixed VP-026 but did not sweep the corpus.
- Remediation (architect): corpus-wide sweep — update each VP body "Source BC" label to match canonical BC-INDEX H1 exactly.

## Observations

None (all issues elevated to findings).

## Pass-86 Verification

| Item | Status |
|------|--------|
| F86-001 VP-018 re-anchor | VERIFIED |
| F86-002 VP-025 re-anchor | VERIFIED (but F87-002 shows deeper semantic issue remains) |
| F86-003 VP-031 re-anchor | VERIFIED |
| F86-004 VP-005/006 back-refs in BC-2.14.002 | VERIFIED |
| F86-005 VP-051 matrix propagation | VERIFIED |
| F86-006 STATE.md arithmetic 213→208 | VERIFIED |
| F86-007 VP-021 added to BC-2.11.006 | **REGRESSED** (F87-001: wrong method `proptest`) |
| F86-008 VP-026 body label | VERIFIED (but F87-006 shows wider pattern) |

## Arithmetic Consistency

All PASS. VP-INDEX 62 = 26K+28P+6F+2I = 43 P0 + 19 P1. BC-INDEX 200 + 6 + 2 = 208. STORY-INDEX 75 stories. STATE pins match actual versions.

## Policy Rubric

| Policy | Verdict |
|--------|---------|
| 4. semantic_anchoring_integrity | FAIL — F87-002 |
| 8. bc_array_changes_propagate_to_body_and_acs | FAIL — F87-003 (verification_properties analog) |
| 9. vp_index_is_vp_catalog_source_of_truth | FAIL — F87-001 (pass-86 regression) |

## Novelty Assessment

| **Pass** | 87 |
|----------|------|
| New findings | 6 |
| Duplicate/variant findings | 0 |
| Novelty score | HIGH |
| Median severity | HIGH |
| Trajectory | p80=9 → p81=10 → p82=7 → p83=6 → p84=3 → p85=4 → p86=8 → **p87=6** (1 pass-86 regression + 2 new HIGH patterns) |
| Verdict | FINDINGS_REMAIN |

## Counter Recommendation

**RESET 0/3.** 3 HIGH + 3 MED, including 1 direct pass-86 regression.
