---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/STATE.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/behavioral-contracts/BC-2.20.001-log-forwarder-recursive-prevention.md
  - .factory/specs/behavioral-contracts/BC-2.20.002-log-forwarder-min-level-filter.md
  - .factory/specs/behavioral-contracts/BC-2.20.003-log-forwarder-queue-cap.md
  - .factory/specs/behavioral-contracts/BC-2.20.004-log-forwarder-credential-resolution.md
  - .factory/specs/behavioral-contracts/BC-2.20.005-log-forwarder-destination-isolation.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/S-5.09-external-log-forwarding.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/specs/domain-spec/capabilities.md
  - .factory/specs/prd.md
  - .factory/specs/prd-supplements/test-vectors.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/holdout-scenarios/HOLDOUT-INDEX.md
  - .factory/policies.yaml
input-hash: "b645ac4"
traces_to: ""
pass: 81
counter_before: 0
counter_after: 0
findings_total: 10
findings_critical: 1
findings_high: 4
findings_medium: 4
findings_low: 1
observations: 4
convergence_recommendation: RESET
---

# Adversarial Review — Pass 81 (Phase 2 Patch)

**Scope:** Regression review of pass-80 remediation (F80-001..006 + CAP-035 follow-on)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 4 |
| MEDIUM | 4 |
| LOW | 1 |
| OBSERVATIONS | 4 |

Pass-81 found **10 findings**, all attributable to incomplete propagation of the pass-80 remediation into secondary indexes and adjacent artifacts. Primary remediation (BCs, CAP-035, NFRs, test-vectors) landed correctly; secondary-index propagation failed.

## Findings

### F81-001 — CRITICAL — S-5.09 anchor_capabilities is still CAP-025 (mis-anchor)

- File: `.factory/stories/S-5.09-external-log-forwarding.md` line 28
- Evidence: `anchor_capabilities: [CAP-025]` but 5 bound BCs (BC-2.20.001..005) anchor to CAP-035
- Remediation (story-writer): `anchor_capabilities: [CAP-025]` → `[CAP-035]`; add v1.7 changelog row.

### F81-002 — HIGH — module-decomposition.md SS-20 BC count stale (0, should be 5)

- File: `.factory/specs/architecture/module-decomposition.md` line 498
- Evidence: Narrative "SS-20 has 0 BCs at introduction" + arithmetic prism-mcp sum = 28 (actually 33 post-pass-80).
- Remediation (architect): update SS-20 BC count 0→5, prism-mcp sum 28→33, remove stale "at introduction" narrative.

### F81-003 — HIGH — CAP-035 priority mismatch (P1 cap with 5 P0 BCs)

- Files: capabilities.md line 56 (P1); L2-INDEX.md line 108 (P1 bucket); BC-INDEX.md lines 232-236 (5 P0); prd.md lines 423-427 (5 P0); BC-2.20.001..005 frontmatter (all P0).
- Policy 5 violation: capability priority must be consistent with its enforcing BCs.
- Remediation (business-analyst — decision): PROMOTE CAP-035 P1 → P0 (diagnostic log forwarding is launch-critical for MSSP operator workflow, not optional post-MVP). Update capabilities.md, L2-INDEX Priority Distribution.

### F81-004 — HIGH — E-FWD-001 referenced but not defined in error-taxonomy.md

- Files: S-5.09 lines 118/231/241; BC-2.20.004; error-taxonomy.md (no E-FWD-* entries)
- Remediation (product-owner): add E-FWD-001 (inline credential literal rejected) and companion codes E-FWD-002 (delivery timeout) + E-FWD-003 (destination quarantined) to error-taxonomy.md. Cross-ref from BC-2.20.004 and S-5.09.

### F81-005 — HIGH — PRD §4 "18 NFRs" stale

- File: prd.md line 473: "18 non-functional requirements..."
- Reality: NFR-001..023 now exist (F80-005 added NFR-019..023).
- Remediation (product-owner): "18" → "23"; add to changelog.

### F81-006 — MEDIUM — HOLDOUT-INDEX body/state-checkpoint inconsistencies

- File: HOLDOUT-INDEX.md
- Evidence: frontmatter `total_scenarios: 52` (correct) vs line 19 body "Total Scenarios: 53" (stale); HS-001 row count 6 (stale→5); state-checkpoint YAML line 196 total_scenarios 53 (stale→52); line 198 p0_scenarios 37 (needs 37→36 if HS-001-05 was P0).
- Remediation (product-owner): sync 4 inline references.

### F81-007 — MEDIUM — 5 new SS-20 BCs have literal [md5] placeholder

- Files: BC-2.20.001..005 line 25 frontmatter `input-hash: "[md5]"`.
- Remediation (state-manager): run `compute-input-hash --update` on each.

### F81-008 — MEDIUM — 5 new SS-20 BCs cite zero DI invariants

- Files: BC-2.20.001..005 Traceability tables (no DI-NNN rows).
- Policy 2 violation.
- Remediation (product-owner): add DI citations. BC-2.20.004 at minimum must cite DI-002 (credential isolation) + DI-014 (credential sanitization). Others may require new DI filings.

### F81-009 — MEDIUM — 5 new SS-20 BCs have unresolved VP-TBD-20-NNN placeholders

- Files: BC-2.20.001..005 VP sections.
- VP-INDEX v1.8 contains VP-001..060; VP-TBD-20-NNN are NOT registered.
- Policy 9 violation + regression of pass-74 VP-TBD closure policy.
- Remediation (architect): apply pass-74 decision matrix. Either file VP-061..065 (integration_test VPs) and update VP-INDEX/verification-coverage-matrix/verification-architecture, or MARK-NONE per canonical pattern.

### F81-010 — LOW — BC-INDEX v4.11 changelog paragraph superseded by v4.12

- File: BC-INDEX.md lines 480-483: v4.11 paragraph references CAP-025 fallback "A dedicated CAP-035... is recommended for a future capabilities.md update" — which v4.12 did.
- Remediation (product-owner, optional): annotate with "(superseded by v4.12)".

## Observations

- **OBS-1 (cluster drift):** SS-20 cluster — 7 of 10 findings trace to incomplete SS-20 propagation. Flag subsystem for content review.
- **OBS-2:** Pass-80 closed primary findings but introduced secondary-index drift (error-taxonomy, PRD NFR count, module-decomposition BC counts, story anchor_capabilities). Pass-82 should include explicit "secondary index propagation" audit.
- **OBS-3:** Architecture docs do not cite CAP-035 but also don't cite most CAPs — architecture uses SS-NN references; not drift.
- **OBS-4:** BC-2.20.003 description "This is a v1 best-effort delivery model" — release-version language may become stale. Consider removing "v1" qualifier.

## Semantic Anchoring Audit

| Anchor | Status |
|--------|--------|
| S-5.09 anchor_capabilities: [CAP-025] | MIS-ANCHORED (F81-001) |
| BC-2.20.001..005 subsystem: SS-20 | correct |
| BC-2.20.001..005 capability: CAP-035 | correct |
| S-5.09 subsystems: [SS-20] | correct |
| NFR-023 Traces to | correct |

## Policy Rubric Results

| Policy | Verdict |
|--------|---------|
| 1. append_only_numbering | PASS |
| 2. lift_invariants_to_bcs | FAIL — F81-008 |
| 3. state_manager_runs_last | cannot verify read-only |
| 4. semantic_anchoring_integrity | FAIL — F81-001 |
| 5. creators_justify_anchors | PASS (with CAP-035 caveat) |
| 6. architecture_is_subsystem_name_source_of_truth | PASS |
| 7. bc_h1_is_title_source_of_truth | PASS |
| 8. bc_array_changes_propagate_to_body_and_acs | PASS |
| 9. vp_index_is_vp_catalog_source_of_truth | FAIL — F81-009 |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 81 |
| **New findings** | 10 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (10/10) |
| **Median severity** | 3.0 (HIGH) |
| **Trajectory** | →p80:9(1C+4H+3M+1L)→p81:10(1C+4H+4M+1L) |
| **Verdict** | FINDINGS_REMAIN |

8 of 10 findings are genuinely new content (not refinements). Pass-80 remediation closed its stated scope but introduced new wave of secondary-index drift. F81-001 (critical mis-anchor) only becomes findable after CAP-035 was created; F81-002/005 are cascading count-staleness from pass-80 additions; F81-003 is a priority mismatch that only surfaces once SS-20 BCs exist; F81-004 is first-time E-FWD-001 introduction; F81-007/008/009 are quality issues in the 5 new BCs. Not retreading — the spec has NOT converged.

## Counter recommendation

**RESET TO 0/3.** Ten findings including 1 CRITICAL block advancement. User directive: "No pragmatic convergence. No shortcuts." Clean pass required.
