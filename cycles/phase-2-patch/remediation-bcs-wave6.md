---
document_type: remediation-manifest
level: ops
version: "1.0"
date: 2026-04-20
burst: pre-build-sweep
wave: 6
subsystems: ["2.17", "2.18", "2.19"]
---

# Wave 6 BC Template-Compliance Remediation

## Summary

| Metric | Count |
|--------|-------|
| Total BC files remediated | 20 |
| Subsystem 2.17 (WASM Plugin Runtime — SS-17) | 6 |
| Subsystem 2.18 (Action Delivery Framework — SS-18) | 9 |
| Subsystem 2.19 (Infusion Registry — SS-19) | 5 |
| Active BCs: 1.0 → 1.1 | 19 |
| Active BCs: 1.1 → 1.2 (BC-2.17.005, already had prior fix row) | 1 |
| Tombstones | 0 |
| BC-2.19.004 malformed `capability:` frontmatter correction | 1 |

## Standard Wave 6 Changes Applied to All Active BCs

1. Frontmatter additions (all 20 files):
   - `inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]`
   - `input-hash: "[pending-recompute]"`
   - `traces_to: ["CAP-NNN"]` (using the BC's existing `capability:` field value)
   - `extracted_from: ".factory/specs/prd.md"`
   - Lifecycle fields: `modified`, `deprecated`, `deprecated_by`, `replacement`, `retired`, `removed`, `removal_reason`
2. Body section additions (where missing):
   - `## Description` — already present in all 20 files (no changes needed)
   - `## Error Conditions` — renamed from `## Error Cases` (all 2.17, 2.18, 2.19 files used `## Error Cases`); files that had no explicit section name (2.18.002, 2.18.004, 2.18.005, 2.18.006, 2.18.008, 2.19.002) had inline error rows promoted to the standard table under `## Error Conditions`
   - `## Canonical Test Vectors` — scaffolded table with happy-path, error, and edge-case rows (all 20 files)
   - `## Verification Properties` — VP cross-reference table with placeholder rows (all 20 files)
   - `## Changelog` — version history table with initial row + Wave 6 sweep row (all 20 files)
3. Version bumps applied:
   - 1.0 → 1.1 for 19 files (all except BC-2.17.005)
   - 1.1 → 1.2 for BC-2.17.005 (already had a v1.1 row from Burst 36 error-code fix)

## Subsystem 2.17 — WASM Plugin Runtime (SS-17)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.17.001 | `BC-2.17.001-plugin-panic-isolation.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.17.002 | `BC-2.17.002-plugin-sandbox-filesystem.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.17.003 | `BC-2.17.003-plugin-memory-limit.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.17.004 | `BC-2.17.004-plugin-cpu-time-limit.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.17.005 | `BC-2.17.005-plugin-hot-reload-atomic-swap.md` | 1.1 → 1.2 | Was already at 1.1 (prior error-code fix in Burst 36); renamed Error Cases → Error Conditions; added frontmatter fields, Canonical Test Vectors, Verification Properties |
| BC-2.17.006 | `BC-2.17.006-plugin-wit-validation.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |

## Subsystem 2.18 — Action Delivery Framework (SS-18)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.18.001 | `BC-2.18.001-action-at-least-once-delivery.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.18.002 | `BC-2.18.002-action-schedule-best-effort.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |
| BC-2.18.003 | `BC-2.18.003-action-manual-fire-and-forget.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.18.004 | `BC-2.18.004-action-schedule-semaphore.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |
| BC-2.18.005 | `BC-2.18.005-action-partial-report-failure.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |
| BC-2.18.006 | `BC-2.18.006-action-template-injection-scan.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |
| BC-2.18.007 | `BC-2.18.007-action-credential-opaque-reference.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.18.008 | `BC-2.18.008-action-delivery-audit-logging.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |
| BC-2.18.009 | `BC-2.18.009-action-uuid-v7-validation.md` | 1.0 → 1.1 | No named section header for errors; promoted inline error rows to `## Error Conditions` table |

## Subsystem 2.19 — Infusion Registry (SS-19)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.19.001 | `BC-2.19.001-infusion-spec-loading.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.19.002 | `BC-2.19.002-infusion-per-query-dedup.md` | 1.0 → 1.1 | No named error section; promoted inline error rows to `## Error Conditions` |
| BC-2.19.003 | `BC-2.19.003-infusion-api-backed-rejection.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |
| BC-2.19.004 | `BC-2.19.004-infusion-hot-reload-atomicity.md` | 1.0 → 1.1 | **Malformed `capability:` fixed** (was YAML sequence `[CAP-030, CAP-031]`; normalized to string `"CAP-030"` as primary; secondary CAP-031 noted in `traces_to`). Promoted inline error rows to `## Error Conditions`. |
| BC-2.19.005 | `BC-2.19.005-infusion-credential-redaction.md` | 1.0 → 1.1 | Standard treatment; renamed Error Cases → Error Conditions |

## Anomalies

1. **BC-2.17.005 already at v1.1** — This file had a prior error-code correction applied in Burst 36 (E-PLUGIN-002 → E-PLUGIN-011). Wave 6 treatment bumped it to 1.2. Changelog row appended preserving the prior 1.1 row.

2. **BC-2.19.004 malformed capability frontmatter** — Original file had `capability: [CAP-030, CAP-031]` (YAML sequence, not a scalar string). Normalized to `capability: "CAP-030"` (primary capability) per the template requirement that `capability:` is a string field. Secondary capability CAP-031 referenced in `traces_to: ["CAP-030", "CAP-031"]`.

3. **BC-2.18 / BC-2.19 inline error entries** — Several BC-2.18 and BC-2.19 files had error information scattered as inline table rows with no `## Error Cases` or `## Error Conditions` heading. All were collected and formalized into the standard `## Error Conditions` table.

## Next Steps

1. **input-hash recompute** — state-manager must run `compute-input-hash` on all 20 files to replace `[pending-recompute]` with actual hashes.
2. **VP placeholder assignment** — architect to assign VP IDs for all 20 BCs with placeholder Verification Properties rows.
3. **BC-2.19.004 dual-capability audit** — architect to confirm `CAP-030` is the correct primary anchor for BC-2.19.004, or reassign to `CAP-031` with `traces_to` updated accordingly.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-20 | Initial Wave 6 manifest |
