---
document_type: remediation-manifest
level: ops
version: "1.0"
date: 2026-04-20
burst: pre-build-sweep
wave: 4
subsystems: ["2.11", "2.12", "2.13"]
---

# Wave 4 BC Template-Compliance Remediation

## Summary

| Metric | Count |
|--------|-------|
| Total BC files remediated | 41 |
| Subsystem 2.11 (Query Engine — SS-11) | 15 |
| Subsystem 2.12 (Scheduler/Packs — SS-12) | 12 |
| Subsystem 2.13 (Detection Engine — SS-13) | 14 |
| Tombstone full treatment | 1 (BC-2.12.012) |
| Tombstone frontmatter standardization only | 1 (BC-2.12.011) |
| Active BCs: 1.0 → 1.1 | 37 |
| Active BCs: 1.1 → 1.2 | 1 (BC-2.12.001) |
| Active BCs: 1.2 → 1.3 | 1 (BC-2.13.006) |
| Already at 1.1 (phase-2-patch) — frontmatter+sections only | 1 (BC-2.13.014) |

## Standard Wave 4 Changes Applied to All Active BCs

1. Frontmatter additions:
   - `inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]`
   - `input-hash: "[pending-recompute]"`
   - `traces_to: ["CAP-NNN"]` (using the BC's existing `capability:` field value)
   - `extracted_from: ".factory/specs/prd.md"`
2. Body section additions (where missing):
   - `## Description` — 2-3 sentence synthesis from body content (inserted before Preconditions)
   - `## Canonical Test Vectors` — scaffolded table with happy-path, error, and edge-case rows
   - `## Verification Properties` — VP cross-reference table (VP IDs from VP-INDEX v1.5)
   - `## Changelog` — version history table with initial row + Wave 4 sweep row
3. Version bump applied per convention

## Subsystem 2.11 — Query Engine (SS-11)

| BC ID | File | Version | VPs Added |
|-------|------|---------|-----------|
| BC-2.11.001 | `BC-2.11.001-query-mcp-tool.md` | 1.0 → 1.1 | VP-014, VP-015, VP-021 |
| BC-2.11.002 | `BC-2.11.002-prismql-filter-mode.md` | 1.0 → 1.1 | VP-015, VP-021 |
| BC-2.11.003 | `BC-2.11.003-prismql-sql-mode.md` | 1.0 → 1.1 | VP-014, VP-021 |
| BC-2.11.004 | `BC-2.11.004-prismql-pipe-mode.md` | 1.0 → 1.1 | VP-021 |
| BC-2.11.005 | `BC-2.11.005-ephemeral-materialization.md` | 1.0 → 1.1 | VP-014 |
| BC-2.11.006 | `BC-2.11.006-query-security-limits.md` | 1.0 → 1.1 | VP-014, VP-015 |
| BC-2.11.007 | `BC-2.11.007-sensor-filter-push-down.md` | 1.0 → 1.1 | VP-031 |
| BC-2.11.008 | `BC-2.11.008-create-alias-tool.md` | 1.0 → 1.1 | VP-012, VP-013, VP-037 |
| BC-2.11.009 | `BC-2.11.009-alias-resolution.md` | 1.0 → 1.1 | VP-012, VP-013, VP-037 |
| BC-2.11.010 | `BC-2.11.010-explain-query-tool.md` | 1.0 → 1.1 | VP-021 |
| BC-2.11.011 | `BC-2.11.011-cross-client-query-scoping.md` | 1.0 → 1.1 | VP-001 |
| BC-2.11.012 | `BC-2.11.012-virtual-fields.md` | 1.0 → 1.1 | VP-021 |
| BC-2.11.013 | `BC-2.11.013-list-aliases-tool.md` | 1.0 → 1.1 | placeholder |
| BC-2.11.014 | `BC-2.11.014-delete-alias-tool.md` | 1.0 → 1.1 | placeholder |
| BC-2.11.015 | `BC-2.11.015-explain-alias-tool.md` | 1.0 → 1.1 | VP-012 |

## Subsystem 2.12 — Scheduler / Packs (SS-12)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.12.001 | `BC-2.12.001-create-schedule-tool.md` | 1.1 → 1.2 | Already had Description; added inputs/input-hash/traces_to/extracted_from + Changelog row |
| BC-2.12.002 | `BC-2.12.002-list-schedules-tool.md` | 1.0 → 1.1 | Standard treatment |
| BC-2.12.003 | `BC-2.12.003-delete-schedule-tool.md` | 1.0 → 1.1 | VPs: VP-007, VP-008 |
| BC-2.12.004 | `BC-2.12.004-schedule-execution-loop.md` | 1.0 → 1.1 | VPs: VP-026 |
| BC-2.12.005 | `BC-2.12.005-differential-result-computation.md` | 1.0 → 1.1 | VPs: VP-019 |
| BC-2.12.006 | `BC-2.12.006-epoch-counter-tracking.md` | 1.0 → 1.1 | placeholder VPs |
| BC-2.12.007 | `BC-2.12.007-get-diff-results-tool.md` | 1.0 → 1.1 | VPs: VP-019 |
| BC-2.12.008 | `BC-2.12.008-pack-loading-discovery.md` | 1.0 → 1.1 | placeholder VPs |
| BC-2.12.009 | `BC-2.12.009-pack-crud-tools.md` | 1.0 → 1.1 | VPs: VP-007 |
| BC-2.12.010 | `BC-2.12.010-schedule-state-persistence.md` | 1.0 → 1.1 | VPs: VP-019 |
| BC-2.12.011 | `BC-2.12.011-action-at-least-once-delivery.md` | 1.0 → 1.1 | RETIRED — tombstone frontmatter standardization; full stubs already present from Burst 51; appended Wave 4 Changelog row |
| BC-2.12.012 | `BC-2.12.012-action-template-injection-scanning.md` | 1.0 → 1.1 | RETIRED — FULL tombstone treatment: added all stub sections (Preconditions, Postconditions, Invariants, Edge Cases, Canonical Test Vectors, Verification Properties, Traceability, Changelog); standardized frontmatter |

## Subsystem 2.13 — Detection Engine (SS-13)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.13.001 | `BC-2.13.001-detection-rule-loading.md` | 1.0 → 1.1 | VPs: VP-018 |
| BC-2.13.002 | `BC-2.13.002-single-event-detection.md` | 1.0 → 1.1 | VPs: VP-018, VP-024 |
| BC-2.13.003 | `BC-2.13.003-correlation-detection.md` | 1.0 → 1.1 | VPs: VP-027 |
| BC-2.13.004 | `BC-2.13.004-sequence-detection.md` | 1.0 → 1.1 | VPs: VP-027 |
| BC-2.13.005 | `BC-2.13.005-alert-generation.md` | 1.0 → 1.1 | VPs: VP-027, VP-028 |
| BC-2.13.006 | `BC-2.13.006-create-rule-tool.md` | 1.2 → 1.3 | Non-standard `traces_to`/`inputs` normalized to Wave 4 convention |
| BC-2.13.007 | `BC-2.13.007-list-rules-tool.md` | 1.0 → 1.1 | VPs: VP-030 |
| BC-2.13.008 | `BC-2.13.008-delete-rule-tool.md` | 1.0 → 1.1 | VPs: VP-007 |
| BC-2.13.009 | `BC-2.13.009-rule-to-sql-compilation.md` | 1.0 → 1.1 | VPs: VP-014 |
| BC-2.13.010 | `BC-2.13.010-security-udf-registration.md` | 1.0 → 1.1 | VPs: VP-024 |
| BC-2.13.011 | `BC-2.13.011-three-scope-rule-resolution.md` | 1.0 → 1.1 | VPs: VP-030 |
| BC-2.13.012 | `BC-2.13.012-detection-state-persistence.md` | 1.0 → 1.1 | VPs: VP-027 |
| BC-2.13.013 | `BC-2.13.013-alert-deduplication.md` | 1.0 → 1.1 | VPs: VP-027 |
| BC-2.13.014 | `BC-2.13.014-ioc-file-loading-pattern-store.md` | 1.0 → 1.1 | phase-2-patch BC; already had Description/Related BCs/Architecture Anchors/Story Anchor/VP Anchors; added inputs/input-hash/traces_to/extracted_from + Canonical Test Vectors + Verification Properties + Changelog |

## Anomalies

1. **BC-2.13.006 non-standard frontmatter** — Prior version had `traces_to: domain-spec/L2-INDEX.md` (a doc path, not a CAP-NNN array) and `inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]` (bare paths, not `.factory/`-prefixed). Both normalized to Wave 4 convention.

2. **BC-2.13.014 already advanced** — This phase-2-patch BC was created mid-cycle with rich body content (Related BCs, Architecture Anchors, Story Anchor, VP Anchors) but was missing the standard Wave 4 frontmatter fields. Wave 4 treatment added only the missing frontmatter and the three missing body sections.

3. **BC-2.12.011 vs BC-2.12.012 tombstone treatment** — BC-2.12.011 had full stubs from Burst 51; only frontmatter standardization needed. BC-2.12.012 had only bare retired notice with minimal body — received full tombstone construction in Wave 4.

4. **BCs with no matching VP** — BC-2.11.013, BC-2.11.014, BC-2.12.002, BC-2.12.006, BC-2.12.008 had no clear VP match in VP-INDEX v1.5. Verification Properties sections scaffolded with placeholder rows pending architect assignment.

## Next Steps

1. **input-hash recompute** — state-manager must run `compute-input-hash` on all 41 files to replace `[pending-recompute]` with actual hashes.
2. **VP placeholder assignment** — architect to assign VP IDs for the 5 BCs with placeholder Verification Properties rows.
3. **Wave 5 scope** — Subsystems 2.14–2.16 (remaining detection/config BCs) if any remain non-compliant.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-20 | Initial Wave 4 manifest |
