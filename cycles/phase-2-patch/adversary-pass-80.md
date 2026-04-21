---
document_type: adversary-pass-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-21T00:00:00Z
inputs:
  - .factory/STATE.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/stories/STORY-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/specs/domain-spec/capabilities.md
  - .factory/specs/prd.md
  - .factory/specs/prd-supplements/test-vectors.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/holdout-scenarios/HOLDOUT-INDEX.md
input-hash: "d57d265"
traces_to: ""
pass: 80
counter_before: 0
counter_after: 0
findings_total: 9
findings_critical: 1
findings_high: 4
findings_medium: 3
findings_low: 1
observations: 3
convergence_recommendation: RESET
---

# Adversarial Review — Pass 80

**Pass:** 80 (fresh-context; first pass under v0.47.0 drift detection)
**Producer:** adversary (read-only)
**Project:** prism (Phase 2 patch cycle)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 4 |
| MEDIUM | 3 |
| LOW | 1 |
| OBSERVATIONS | 3 |

Semantic findings invisible to input-hash drift detection: (a) domain-spec ↔ capabilities.md drift (CAP-031..034 missing from L2-INDEX), (b) SS-20 has zero behavioral-contract coverage despite live consumer stories, (c) test-vectors.md subsystem headers name non-owning capabilities.

## Findings

### F80-001 — CRITICAL — L2-INDEX.md ID Registry contradicts capabilities.md by 4 capabilities

- Category: spec-drift / semantic-anchoring
- Affected: .factory/specs/domain-spec/L2-INDEX.md lines 90, 102–107; .factory/specs/domain-spec/capabilities.md lines 51–54, 61; .factory/specs/architecture/ARCH-INDEX.md lines 83–86
- Evidence: L2-INDEX line 90 registers CAP-001..030 (CAP-013 removed) = 29. capabilities.md defines CAP-031 (Infusion), CAP-032 (WASM Plugin), CAP-033 (Action Delivery), CAP-034 (MCP Server). ARCH-INDEX introduces SS-17/18/19 anchored to these. L2-INDEX Cross-References has no rows for CAP-031–034; Priority Distribution does not mention them.
- Remediation (business-analyst): Add CAP-031–034 rows to Cross-References, update ID Registry (29→33, CAP-001..034), update Priority Distribution, update Domain Summary narrative.

### F80-002 — HIGH — SS-20 has zero behavioral contracts yet owns two stories

- Category: traceability-gap / coupling
- Affected: BC-INDEX.md line 256 (SS-20 row all zeros); ARCH-INDEX.md line 111; architecture/observability.md line 16; S-5.09-external-log-forwarding.md; S-5.08-diagnostics-logs-cli.md lines 254–260.
- Evidence: No BC file has `subsystem: "SS-20"`. S-5.09 borrows BC-2.10.001 (SS-10) because no native SS-20 BCs exist.
- Remediation (product-owner): Author ≥3–5 SS-20 BCs: recursive-forwarding prevention, per-destination min_level filtering, queue cap, AD-017 credential-reference resolution, failed-destination isolation. Update BC-INDEX.

### F80-003 — HIGH — test-vectors.md per-subsystem headers cite non-owning capabilities

- Category: semantic-anchoring / Policy 6
- Affected: .factory/specs/prd-supplements/test-vectors.md lines 40, 69, 162
- Evidence:
  - Line 40 `### Subsystem: SS-05 Audit (CAP-007, CAP-024)` — CAP-024 is Resource Watchdog (SS-15). Should be CAP-025 (Buffered Audit Logging).
  - Line 69 `### Subsystem: SS-04 Feature Flags and Write Gate (CAP-005, CAP-006, CAP-014)` — CAP-014 is Response Caching (SS-07). Drop CAP-014.
  - Line 162 `### Subsystem: SS-14 Case Management (CAP-021, CAP-022)` — CAP-021 is Alert Generation (SS-13). Drop CAP-021.
- Remediation (product-owner): Fix three header triples.

### F80-004 — HIGH — S-5.09 semantically mis-anchored to BC-2.10.001

- Category: Policy 4 + Policy 5
- Affected: .factory/stories/S-5.09-external-log-forwarding.md lines 20, 27, 32, 66
- Evidence: BC-2.10.001 is a SS-10/CAP-034 rmcp ServerHandler BC with zero postconditions covering forwarder behavior, batched delivery, queue cap, recursive-forwarding, min_level, or credential resolution.
- Remediation (coordinated): product-owner authors SS-20 BCs (F80-002); story-writer re-anchors S-5.09.

### F80-005 — HIGH — NFR catalog has zero NFRs for SS-17/18/19/20

- Category: coverage-gap
- Affected: .factory/specs/prd-supplements/nfr-catalog.md
- Evidence: NFR-001..018 only; no NFRs for 4 Phase-3 subsystems.
- Remediation (product-owner): Add NFR-019 Plugin Memory Bounds (CAP-032); NFR-020 Plugin CPU Epoch Deadline (CAP-032); NFR-021 Action Delivery Latency & Retry Bounds (CAP-033); NFR-022 Infusion Dedup Cache Effectiveness (CAP-031); NFR-023 Log Forwarder Batch Flush Interval (SS-20).

### F80-006 — MEDIUM — HS-001-05 references REMOVED CAP-013 (xMP)

- Category: stale-artifact
- Affected: .factory/holdout-scenarios/HOLDOUT-INDEX.md line 50; L2-INDEX.md line 67 (CAP-013 REMOVED); capabilities.md line 33.
- Remediation (product-owner): Mark HS-001-05 title strikethrough + "REMOVED (CAP-013 out of scope)" or replace with live-CAP happy-path.

### F80-007 — MEDIUM — BC-INDEX changelog arithmetic untraceable

- Category: changelog-inconsistency
- Affected: BC-INDEX.md line 403 (v4.7 claim removed=13) vs line 11 frontmatter (removed_contracts: 6).
- Remediation (product-owner): Add explicit arithmetic note at v4.9 changelog: `removed_contracts: 13 → 8 (v4.8) → 6 (v4.9)`.

### F80-008 — MEDIUM — S-5.08 body names SS-20 ownership but frontmatter omits SS-20

- Category: frontmatter-body coherence (Policy 8)
- Affected: S-5.08-diagnostics-logs-cli.md line 6 vs lines 255, 260.
- Remediation (story-writer): Update `subsystems: [SS-08, SS-10, SS-20]` OR rewrite body to remove SS-20 ownership claim.

### F80-009 — LOW — STORY-INDEX wave-sum arithmetic history undocumented

- Category: stylistic
- Affected: STORY-INDEX.md line 63, 71
- Remediation: informational only.

## Observations (non-blocking)

- OBS-80-01 — STORY-INDEX body changelog gap between v1.21 and v1.22+
- OBS-80-02 — test-vectors.md covers only 7 of 20 subsystems; deserves preamble
- OBS-80-03 — STATE.md `phase_2_converged: 2026-04-15` conflicts with active patch cycle; rename to `phase_2_first_converged`

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 80 |
| **New findings** | 9 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 9 / (9 + 0) = 1.00 |
| **Median severity** | 3.0 |
| **Trajectory** | …→p79:3 →drift-rebaseline(v0.47.0) →p80:9 |
| **Verdict** | FINDINGS_REMAIN |

## Counter recommendation

**Reset to 0/3** pending remediation burst. 9 findings exceed zero-findings threshold. After remediation, dispatch pass-81.
