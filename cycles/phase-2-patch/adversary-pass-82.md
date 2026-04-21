---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/STATE.md
  - .factory/specs/prd.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/specs/architecture/observability.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/specs/domain-spec/capabilities.md
  - .factory/specs/domain-spec/invariants.md
  - .factory/stories/S-5.09-external-log-forwarding.md
  - .factory/policies.yaml
input-hash: "9cd4b19"
traces_to: ""
pass: 82
counter_before: 0
counter_after: 0
findings_total: 7
findings_critical: 0
findings_high: 3
findings_medium: 3
findings_low: 1
observations: 4
convergence_recommendation: RESET
---

# Adversarial Review — Pass 82 (Phase 2 Patch)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 3 |
| LOW | 1 |
| OBSERVATIONS | 4 |

7 findings; 4 of 7 are direct propagation regressions from pass-81.

## Findings

### F82-001 — HIGH — PRD §5 claims "33 error categories" but error-taxonomy.md has 34 (FWD added)

- File: prd.md line 487, table lines 489–523
- Evidence: prd.md §5 count "33 error categories"; error-taxonomy.md v1.6 added `## FWD: Log Forwarder Errors` (pass-81 F81-004). PRD table has no FWD row.
- Remediation (product-owner): update PRD §5 count 33→34; add FWD row.

### F82-002 — HIGH — module-decomposition.md prism-mcp BC count wrong (says 33, actual 35)

- File: module-decomposition.md lines 469, 500
- Evidence: prism-mcp row shows 33. BC-INDEX v4.12 shows SS-10=11 (not 10) and SS-06=10 (not 9). Correct sum: SS-10=11 + SS-06=10 + SS-08=9 + SS-20=5 = 35.
- Pass-81 F81-002 used stale SS-06/SS-10 counts.
- Remediation (architect): fix 33→35; recompute footnote SS arithmetic.

### F82-003 — HIGH — module-decomposition.md prism-security BC count wrong (says 22, actual 23)

- File: module-decomposition.md line 475
- Evidence: prism-security row shows 22. BC-INDEX: SS-04=15, SS-09=8 → sum 23.
- Pre-existing drift (not pass-81 regression) but blocks convergence.
- Remediation (architect): fix 22→23; audit grand-total crate BC sum matches BC-INDEX (should be 200).

### F82-004 — MEDIUM — STATE.md VP priority split wrong (claims "45 P0 + 17 P1"; actual 43 P0 + 19 P1)

- File: STATE.md lines 174, 178
- Evidence: STATE.md narrative claims 45 P0 + 17 P1. VP-INDEX v1.9 Summary: Total=62, P0=43, P1=19. verification-coverage-matrix v1.6 and verification-architecture v1.7 agree.
- Pass-81 state-manager pinned VP-INDEX v1.9 but did not update the P0/P1 split narrative.
- Remediation (state-manager): update STATE.md 45 P0 + 17 P1 → 43 P0 + 19 P1 (2 lines).

### F82-005 — MEDIUM — S-5.09 frontmatter verification_properties:[] empty; should reference VP-061 + VP-062

- File: S-5.09-external-log-forwarding.md line 21
- Evidence: VP-061 (line 50) and VP-062 anchor to S-5.09; VP-INDEX rows confirm. S-5.09 reciprocal frontmatter is empty.
- Precedent: S-5.10, S-1.02, S-3.04 frontmatter all carry their VP IDs.
- Policy 4 violation (bidirectional semantic anchor broken).
- Remediation (story-writer): S-5.09 `verification_properties: [VP-061, VP-062]`; bump version.

### F82-006 — MEDIUM — NFR-023 config key `max_batch_age=5s` contradicts observability.md `flush_interval_seconds=10`

- Files: nfr-catalog.md lines 237–238; observability.md lines 401–402, 511; capabilities.md CAP-035 description
- Evidence: NFR-023 names flush trigger `max_batch_age` with default 5s. observability.md and capabilities.md name it `flush_interval_seconds` with default 10s. Different key names AND different defaults for same quantity.
- Remediation (architect + product-owner): align on `flush_interval_seconds` + default 10 (architecture is source of truth). Update NFR-023 text. If a secondary age-based trigger is intentional, document as separate config key.

### F82-007 — LOW — E-FWD-001 Message Format has `\"env\"` escaped quotes that render literally as backslashes

- File: error-taxonomy.md line 413
- Evidence: Cell contains `\`{source = \"env\", key = \"...\"}\``. Backticks already protect from Markdown; the `\` is unnecessary and produces malformed TOML string.
- Remediation (product-owner): unescape quotes within the backticked TOML string.

## Observations (non-blocking but actionable)

### OBS-082-001 — STATE.md `cap_count: 35` counts max ID; active count is 34

- STATE.md line 59. Cosmetic; clarify semantic.

### OBS-082-002 — L2-INDEX DI ID Registry count 25 stale; actual 28 active DIs (range includes DI-032)

- File: L2-INDEX.md line 96. Says `DI-001 to DI-031` with count 25. Actual: 28 active DIs; DI-032 exists.
- Remediation (business-analyst): update count and range.

### OBS-082-003 — SS-20 Phase-Introduced label "Phase 1" in ARCH-INDEX; SS-20 was introduced pass-80 during Phase 2 patch

- File: ARCH-INDEX.md line 111. SS-17/18/19 labeled "Phase 3"; SS-20 should match or be "Phase 2 patch".
- Remediation (architect): correct phase label.

### OBS-082-004 — SS-20 BCs lack first-class DI anchors; L2 Invariants rows provide prose rationale instead

- 4 of 5 SS-20 BCs have "No direct DI covers..." prose. Consider filing DI-033 (forward-path purity), DI-034 (bounded forwarder queue best-effort), DI-035 (destination isolation).
- Deferred (next-burst scope question).

## Cluster Drift

- Architecture BC-count rollup (module-decomposition.md): 2 HIGH findings (F82-002, F82-003). Arithmetic off by 3 across crate table.
- Pass-81 propagation gaps: 4 findings are regressions or incomplete propagation. Pattern flag.

## Policy Rubric Results

| Policy | Verdict |
|--------|---------|
| 1. append_only_numbering | PASS |
| 2. lift_invariants_to_bcs | PASS (syntactic; OBS-082-004 semantic open) |
| 3. state_manager_runs_last | PARTIAL (F82-004 shows state-manager did not reconcile) |
| 4. semantic_anchoring_integrity | FAIL — F82-005 (S-5.09 ↔ VP-061/062 broken) |
| 5. creators_justify_anchors | PASS |
| 6. architecture_is_subsystem_name_source_of_truth | FAIL — F82-002/003 (BC count drift) |
| 7. bc_h1_is_title_source_of_truth | PASS |
| 8. bc_array_changes_propagate_to_body_and_acs | PASS |
| 9. vp_index_is_vp_catalog_source_of_truth | PASS (VP-061/062 registered) |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 82 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 4 |
| **Novelty score** | 0.43 (3/7) |
| **Median severity** | 2.0 (HIGH) |
| **Trajectory** | →p81:10(1C+4H+4M+1L)→p82:7(0C+3H+3M+1L) |
| **Verdict** | FINDINGS_REMAIN |

## Counter Recommendation

**RESET 0/3 → 0/3**. 3 HIGH findings + multiple MED. Per user directive "No pragmatic convergence", cannot advance.
