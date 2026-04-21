---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "47125c0"
traces_to: ["CAP-031"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.19.001: Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF

## Description

When an `.infusion.toml` spec file is loaded by the `InfusionRegistry`, each
`[[infusion.fields]]` entry must result in exactly one `InfusionUdfDescriptor` being
exported. This descriptor is consumed by `prism-query` (S-3.02) to register a
DataFusion `ScalarUDF`. Duplicate UDF names across multiple infusion specs are detected
at load time and rejected. Missing required fields cause the entire spec to be rejected.
This is INV-INFUSE-001.

## Preconditions

- The `InfusionRegistry` loader is scanning `{config_dir}/infusions/*.infusion.toml`
- A spec file contains at least one `[[infusion.fields]]` entry with valid `name`,
  `input_field`, `input_type`, and `output_type` fields

## Postconditions

- For each `[[infusion.fields]]` entry in the spec:
  - Exactly one `InfusionUdfDescriptor` is produced with: `name`, `input_type`, `output_type`,
    and a reference to the `InfusionSource` lookup function
  - The descriptor is added to `InfusionRegistry::udf_descriptors()` output
- `prism-query` (S-3.02) consumes `udf_descriptors()` and registers each as a DataFusion `ScalarUDF`
- **Duplicate UDF name detection:** If two specs declare the same `[[infusion.fields]]` name
  (e.g., both declare `name = "geoip_country"`), the second spec is rejected with:
  `E-INFUSE-002: "Duplicate UDF name 'geoip_country' in '{path2}' — already registered from '{path1}'."`
  The first-registered spec is retained.
- **Missing required field:** Spec is rejected with actionable error per missing field

## Invariants

- INV-INFUSE-001: Each `[[infusion.fields]]` entry must register exactly one DataFusion scalar UDF
- UDF names are global within a DataFusion `SessionContext`; duplicates are a load-time error
- `prism-spec-engine` does NOT depend on DataFusion — it exports `InfusionUdfDescriptor`
  structs; `prism-query` handles actual DataFusion registration
- A spec with 3 `[[infusion.fields]]` entries produces exactly 3 `InfusionUdfDescriptor` objects

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-INFUSE-002` | Duplicate UDF name across specs | Second spec rejected; first retained; `ERROR` log |
| `E-INFUSE-003` | Missing required field in spec (`infusion_id`, `[[infusion.fields]]`) | Spec rejected with per-field error list; other specs continue |
| `E-INFUSE-004` | Source type not recognized (`type = "unknown"`) | Spec rejected; `E-INFUSE-004: "Unknown source type 'unknown'. Valid types: maxmind_mmdb, csv, json_lookup, plugin."` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-001 | Spec with 0 `[[infusion.fields]]` entries | Rejected: at least one field required per INV-INFUSE-001 |
| EC-19-002 | Spec with 10 fields, all valid | 10 `InfusionUdfDescriptor` objects exported |
| EC-19-003 | Hot reload adds a new spec with 3 fields | 3 new descriptors exported; `prism-query` notified to register new UDFs; old UDFs from other specs unchanged |
| EC-19-004 | Spec loaded but source file (MMDB, CSV) missing | Spec is registered but `InfusionSource::enrich_single` returns `None` for all lookups; spec is not rejected (source file may be mounted later) |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-19-001-happy | `geoip.infusion.toml` with 1 valid field | 1 `InfusionUdfDescriptor` exported; `geoip_country` UDF registered | AC-1 |
| TV-19-001-10fields | Spec with 10 valid fields | 10 descriptors exported exactly | EC-19-002 |
| TV-19-001-dup | Two specs both declare `geoip_country` | Second spec rejected with `E-INFUSE-002`; first retained | Error row 1 |
| TV-19-001-empty | Spec with 0 `[[infusion.fields]]` | Rejected: zero fields | EC-19-001 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-048 | `InfusionRegistry::load_spec()` with N valid, distinct field entries produces exactly N `InfusionUdfDescriptor` objects in the output; duplicate UDF names produce `Err(E-INFUSE-002)` rather than silently merging | Kani |

## Related BCs

- BC-2.19.002 — Per-Query Dedup Cache (governs how UDF calls are deduplicated)
- BC-2.19.003 — API-Backed UDF Rejection in Detection Rules (INV-INFUSE-003)
- BC-2.19.004 — Hot Reload Atomicity (CI-002 pattern applies to infusion registry)
- BC-2.13.009 — Rule-to-SQL Compilation (detection rules that reference infusion UDFs)

## Architecture Anchors

- AD-020: Infusions — enrichment framework
- `specs/architecture/infusions.md` — `InfusionUdfDescriptor`, spec structure, UDF registration
- S-1.14 Task 4: `infusion/udf.rs` — UDF descriptor export

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-001, AC-1)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Load `geoip.infusion.toml` → verify `geoip_country` UDF registered."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-031 |
| Story Invariant | INV-INFUSE-001 |
| ADR | AD-020 |
| Story | S-1.14 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-048); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
