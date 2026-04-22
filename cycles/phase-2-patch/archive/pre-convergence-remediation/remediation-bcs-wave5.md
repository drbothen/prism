---
document_type: remediation-manifest
level: ops
version: "1.0"
date: 2026-04-20
burst: pre-build-sweep
wave: 5
subsystems: ["2.14", "2.15", "2.16"]
---

# Wave 5 BC Template-Compliance Remediation

## Summary

| Metric | Count |
|--------|-------|
| Total BC files remediated | 33 |
| Subsystem 2.14 (Case Management — SS-14) | 12 |
| Subsystem 2.15 (Persistence / Watchdog / Decorators — SS-15) | 11 |
| Subsystem 2.16 (Config-Driven Sensors — SS-16) | 10 |
| Active BCs: 1.0 → 1.1 | 33 |
| Tombstones | 0 |
| BC-2.14.012/013 frontmatter correction (hook-caught missing fields) | 2 |

## Standard Wave 5 Changes Applied to All Active BCs

1. Frontmatter additions (all 33 files):
   - `inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]`
   - `input-hash: "[pending-recompute]"`
   - `traces_to: ["CAP-NNN"]` (using the BC's existing `capability:` field value)
   - `extracted_from: ".factory/specs/prd.md"`
   - Missing lifecycle fields added to phase-2-patch BCs (BC-2.14.012, BC-2.14.013): `modified`, `deprecated`, `deprecated_by`, `replacement`, `retired`, `removed`, `removal_reason`
2. Body section additions (where missing):
   - `## Description` — 2-3 paragraph synthesis from Preconditions/Postconditions (all 33 files)
   - `## Invariants` — added to all BC-2.16 files (where absent); BC-2.14 and BC-2.15 already had Invariants sections
   - `## Error Conditions` — renamed from `## Error Cases` (BC-2.14 files) or `## Error Handling` (BC-2.16 files), or created from inline error notes (BC-2.16.006, BC-2.16.010)
   - `## Canonical Test Vectors` — scaffolded table with happy-path, error, and edge-case rows (all 33 files)
   - `## Verification Properties` — VP cross-reference table with placeholder rows (all 33 files)
   - `## Changelog` — version history table with initial row + Wave 5 sweep row (all 33 files)
3. `## Traces` → `## Traceability` conversion for BC-2.16.002, BC-2.16.003, BC-2.16.004, BC-2.16.006, BC-2.16.010 (these used an informal `## Traces` section instead of the standard Traceability table)
4. Version bump: 1.0 → 1.1 applied to all 33 files

## Subsystem 2.14 — Case Management (SS-14)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.14.001 | `BC-2.14.001-create-case-tool.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.002 | `BC-2.14.002-case-state-transitions.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.003 | `BC-2.14.003-update-case-tool.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.004 | `BC-2.14.004-list-cases-tool.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.005 | `BC-2.14.005-get-case-tool.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.006 | `BC-2.14.006-disposition-assignment.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.007 | `BC-2.14.007-timeline-annotations.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.008 | `BC-2.14.008-mttd-mttr-computation.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.009 | `BC-2.14.009-case-persistence.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.010 | `BC-2.14.010-case-metrics-tool.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.14.012 | `BC-2.14.012-acknowledge-alert.md` | 1.0 → 1.1 | Phase-2-patch BC; already had Description, Related BCs, Architecture Anchors, Story Anchor, VP Anchors. Added frontmatter fields (hook caught 7 missing: deprecated, deprecated_by, modified, removal_reason, removed, replacement, retired). Added Canonical Test Vectors, Verification Properties, Changelog. Renamed Error Cases → Error Conditions. |
| BC-2.14.013 | `BC-2.14.013-auto-case-creation.md` | 1.0 → 1.1 | Phase-2-patch BC; already had Description, Related BCs, Architecture Anchors, Story Anchor, VP Anchors. Added full frontmatter (all lifecycle fields). Added Canonical Test Vectors, Verification Properties, Changelog. Renamed Error Cases → Error Conditions. |

## Subsystem 2.15 — Persistence / Watchdog / Decorators (SS-15)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.15.001 | `BC-2.15.001-rocksdb-initialization.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.002 | `BC-2.15.002-domain-kv-operations.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.003 | `BC-2.15.003-buffered-audit-log-persistence.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.004 | `BC-2.15.004-audit-buffer-overflow.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.005 | `BC-2.15.005-crash-recovery-dirty-bits.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.006 | `BC-2.15.006-resource-watchdog-initialization.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.007 | `BC-2.15.007-watchdog-query-termination.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.008 | `BC-2.15.008-query-denylisting.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.009 | `BC-2.15.009-context-decorator-injection.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.010 | `BC-2.15.010-decorator-three-phase-model.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |
| BC-2.15.011 | `BC-2.15.011-internal-table-registration.md` | 1.0 → 1.1 | Standard treatment; Error Cases → Error Conditions |

## Subsystem 2.16 — Config-Driven Sensors (SS-16)

| BC ID | File | Version | Notes |
|-------|------|---------|-------|
| BC-2.16.001 | `BC-2.16.001-sensor-spec-file-loading.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions (from inline error handling), Canonical Test Vectors, Verification Properties, Changelog |
| BC-2.16.002 | `BC-2.16.002-multi-step-fetch-pipeline.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability table; added test vectors, VPs, Changelog |
| BC-2.16.003 | `BC-2.16.003-column-to-ocsf-mapping.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability; added test vectors, VPs, Changelog |
| BC-2.16.004 | `BC-2.16.004-rust-escape-hatch.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability; added test vectors, VPs, Changelog |
| BC-2.16.005 | `BC-2.16.005-reload-config-tool.md` | 1.0 → 1.1 | Standard+ treatment: added Description, Invariants, Canonical Test Vectors, Verification Properties, Changelog; pre-existing Traceability table retained |
| BC-2.16.006 | `BC-2.16.006-arc-swap-config-access.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability; added test vectors, VPs, Changelog |
| BC-2.16.007 | `BC-2.16.007-sensor-spec-hot-reload.md` | 1.0 → 1.1 | Standard+ treatment: added Description, Invariants, Error Conditions (extracted from body), Canonical Test Vectors, Verification Properties, Changelog |
| BC-2.16.008 | `BC-2.16.008-add-sensor-spec-tool.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability; fixed malformed capability frontmatter (was YAML array, now string); added test vectors, VPs, Changelog |
| BC-2.16.009 | `BC-2.16.009-spec-file-validation.md` | 1.0 → 1.1 | Standard+ treatment: added Description, Invariants, Error Conditions (normalized from Error Codes section), Canonical Test Vectors, Verification Properties, Changelog |
| BC-2.16.010 | `BC-2.16.010-list-sensor-specs-tool.md` | 1.0 → 1.1 | Heavy treatment: added Description, Invariants, Error Conditions; converted ## Traces → ## Traceability; added test vectors, VPs, Changelog |

## Anomalies

1. **BC-2.14.012 hook failure** — The post-write hook caught 7 missing frontmatter fields (deprecated, deprecated_by, modified, removal_reason, removed, replacement, retired) on the first write attempt. This was a phase-2-patch BC that had a stripped frontmatter. Fixed immediately via Edit; subsequent writes to BC-2.14.013 included all fields from the start.

2. **BC-2.16.008 malformed capability frontmatter** — Original file had `capability: [CAP-029, CAP-030]` (YAML sequence). The template requires a string field. Normalized to `capability: "CAP-029"` (primary capability) with secondary reference in Traceability table.

3. **BC-2.16 `## Traces` section pattern** — Five BC-2.16 files (002, 003, 004, 006, 010) used an informal `## Traces` section with bullet-list references instead of the standard `## Traceability` markdown table. All converted to the standard table format.

4. **BC-2.16 missing Invariants** — All 10 BC-2.16 files were missing `## Invariants` sections. For BCs where no domain-invariant-specific rules applied beyond the standard DI references, the section was populated with a statement referencing the relevant DI codes and the standard invariants.md.

5. **BC-2.16 Error Handling vs Error Conditions** — BC-2.16 files used inconsistent section names: `## Error Handling` (2.16.002, 003, 004), inline error notes (2.16.005, 007), `## Error Codes` (2.16.009), or no section (2.16.006, 2.16.010). All normalized to `## Error Conditions` with the standard table format.

## Next Steps

1. **input-hash recompute** — state-manager must run `compute-input-hash` on all 33 files to replace `[pending-recompute]` with actual hashes.
2. **VP placeholder assignment** — architect to assign VP IDs for all 33 BCs with placeholder Verification Properties rows.
3. **Error Cases alias** — BC-2.14 files previously used `## Error Cases`. Renamed to `## Error Conditions` for consistency with the template and Waves 1-4 convention. If any downstream references used the old heading, they should be updated.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-04-20 | Initial Wave 5 manifest |
