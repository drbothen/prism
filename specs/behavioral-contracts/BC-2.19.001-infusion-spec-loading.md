---
document_type: behavioral-contract
level: L3
version: "1.9"
status: active
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
modified: 2026-06-18
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
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
- **API-backed source wiring (two phases) — applies to `InfusionType::Plugin` and `InfusionType::HttpLookup`:**
  - **PARSE PHASE** (`InfusionLoader::load_all`): parses `*.infusion.toml` files and returns
    `(Vec<InfusionSpec>, Vec<InfusionError>)`. It does NOT construct `PluginInfusionSource`
    or `HttpLookupSource`, and does NOT attach anything as `descriptor.source`. At the end of
    the PARSE PHASE each plugin-type or http-lookup-type descriptor carries `Arc<NullSource>`
    as a placeholder source.
  - **RUNTIME PHASE** (`InfusionRegistry::load_spec_with_runtime`, and future boot-time wiring
    chained from it): branches on `InfusionType`:
    - `Plugin`: builds `PluginInfusionSource` — carrying `plugin_id` and `config` populated
      from the `InfusionSpec` — and attaches it as `descriptor.source` (an `Arc<dyn InfusionSource>`).
      The `plugin_id` and `config` values from the spec are NOT fields on `InfusionUdfDescriptor`
      directly; they live on `PluginInfusionSource`, reachable via `descriptor.source`.
    - `HttpLookup` (added ADR-040 v2.0 §D8.6): builds `HttpLookupSource` — carrying `http_lookup_config`
      from the `InfusionSpec` (base URL, JSONPath, credential config) — and attaches it as
      `descriptor.source`. Construction also performs SSRF validation; if `base_url` resolves
      to a private/loopback address and `PRISM_DTU_MODE` is unset, returns `E-INFUSE-011`
      and rejects the spec.
  - A plugin-type or http-lookup-type spec that reaches query execution still carrying
    `Arc<NullSource>` as `descriptor.source` — because `load_spec_with_runtime` was not invoked
    or failed silently — is a loading defect equivalent to `E-INFUSE-003`: `NullSource` returns
    `None` for all enrichment lookups, making enrichment silently inoperative.
- `prism-query` (S-3.02) consumes `udf_descriptors()` and registers each as a DataFusion `ScalarUDF`
- **`enrich_descriptor()` API (AC-3):** `InfusionRegistry::enrich_descriptor(name: &str)` returns an
  `EnrichStageDescriptor` (defined in `prism-spec-engine::infusion::enrich_descriptor`) for any loaded
  infusion. The descriptor carries:
  - `infusion_name` — the infusion name exactly as declared in the spec
  - `input_field` — the `input_field` from the spec's first `[[infusion.fields]]` entry (the join key)
  - `output_columns` — the `name` of every `[[infusion.fields]]` entry in declaration order
  - `infusion_id` — the `infusion_id` from the spec root
  This descriptor is consumed by `prism-query` (S-3.02) to execute the `| enrich` pipe stage
  transformation. Unknown name returns `E-INFUSE-001`.
- **Scope boundary — pipe-mode `| enrich` runtime execution (S-3.01 anchor, NOT this BC):**
  This BC's contract surface is fully satisfied when:
  (a) each `[[infusion.fields]]` entry registers exactly one DataFusion scalar UDF (SQL-mode
      `SELECT udf(field)` path, wired by `prism-query` S-3.02), AND
  (b) `InfusionRegistry::enrich_descriptor(name)` returns a well-formed `EnrichStageDescriptor`.
  The pipe-stage `| enrich` RUNTIME dispatch — i.e., resolving `Ast::Pipe { stage: Enrich, ... }`
  through materialization into actual RecordBatch column hydration — is universally unimplemented
  for ALL pipe stages and is owned by **S-3.01** (the PrismQL pipe-execution story). This is not
  a S-1.14-REDO gap; it is a system-wide materialization gap. Fresh-context adversaries MUST NOT
  flag the `| enrich` pipe-mode no-op as a S-1.14-REDO or BC-2.19.001 defect. Architect ruling:
  S-1.14-REDO Q1 (2026-06-18). The `enrich_descriptor()` API is the correct handoff boundary.
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
| `E-INFUSE-001` | `enrich_descriptor(name)` called with unknown infusion name | Returns `InfusionError::UnknownInfusion { name }` |
| `E-INFUSE-002` | Duplicate UDF name across specs | Second spec rejected; first retained; `ERROR` log |
| `E-INFUSE-003` | Missing required field in spec (`infusion_id`, `[[infusion.fields]]`) | Spec rejected with per-field error list; other specs continue |
| `E-INFUSE-004` | Source type not recognized (`type = "unknown"`) | Spec rejected; `E-INFUSE-004: "Unknown source type 'unknown'. Valid types: maxmind_mmdb, csv, json_lookup, plugin, http_lookup."` |

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
| TV-19-001-enrich-desc | `geoip.infusion.toml` with 4 fields loaded; call `enrich_descriptor("geoip")` | Returns `EnrichStageDescriptor { infusion_name: "geoip", input_field: "device_ip", output_columns: ["geoip_country","geoip_city","geoip_asn","geoip_is_tor"], infusion_id: "geoip" }` | AC-3 |
| TV-19-001-enrich-desc-unknown | Call `enrich_descriptor("nonexistent_infusion")` on empty registry | Returns `Err(InfusionError::UnknownInfusion { name: "nonexistent_infusion" })` | E-INFUSE-001 |

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
| 1.9 | S-1.14-REDO-Q1-scope-clarification | 2026-06-18 | product-owner | **Scope clarification per architect ruling S-1.14-REDO Q1 (2026-06-18).** (1) Added `enrich_descriptor()` API postcondition (AC-3): `InfusionRegistry::enrich_descriptor(name)` returns `EnrichStageDescriptor` carrying `infusion_name`, `input_field`, `output_columns`, and `infusion_id`; unknown name returns `E-INFUSE-001`. (2) Added explicit "Scope boundary — pipe-mode `| enrich` runtime execution" postcondition clarifying that this BC's contract is satisfied by UDF descriptor registration (SQL-mode) + `enrich_descriptor()` returning a well-formed `EnrichStageDescriptor`; `| enrich` pipe RUNTIME dispatch (Ast::Pipe arm, RecordBatch hydration) is universally unimplemented for ALL pipe stages and is owned by **S-3.01** — not a S-1.14-REDO gap; fresh-context adversaries must not flag this as BC-2.19.001 defect. (3) Added `E-INFUSE-001` to Error Conditions table (was tested but absent). (4) Added AC-3 canonical test vectors `TV-19-001-enrich-desc` and `TV-19-001-enrich-desc-unknown`. |
| 1.8 | PIVOT-002-bc-amendment-http-lookup | 2026-06-17 | product-owner | **Added `http_lookup` as valid `InfusionType` source per ADR-040 v2.0 §D8.3 and error-taxonomy.md v1.88.** (1) E-INFUSE-004 valid-types list: `maxmind_mmdb, csv, json_lookup, plugin` → `maxmind_mmdb, csv, json_lookup, plugin, http_lookup`. (2) Two-phase source wiring postcondition expanded: heading renamed from "Plugin-type source wiring" to "API-backed source wiring" to cover both `Plugin` and `HttpLookup` types; RUNTIME PHASE now explicitly branches on `InfusionType` — `Plugin` path unchanged, `HttpLookup` path (ADR-040 §D8.6) documents `HttpLookupSource` construction with SSRF validation and `E-INFUSE-011` rejection. `NullSource` defect note extended to cover both `Plugin` and `HttpLookup`. Scope confirmed: `HttpLookup` flows through the same `InfusionLoader::parse` (PARSE PHASE) + `InfusionRegistry::load_spec_with_runtime` (RUNTIME PHASE) two-phase path already specified by this BC — no sibling BC needed. |
| 1.7 | PIVOT-001-LOW-2-regression-fix | 2026-06-15 | product-owner | Regression fix (PIVOT-001 LOW-2): v1.6 reword incorrectly re-introduced `load_all` as constructor of `PluginInfusionSource`. Corrected to accurate two-phase model: PARSE PHASE (`load_all`) returns `(Vec<InfusionSpec>, Vec<InfusionError>)` and does NOT construct `PluginInfusionSource`; RUNTIME PHASE (`load_spec_with_runtime`) builds `PluginInfusionSource` (carrying `plugin_id`/`config` from the spec) and attaches it as `descriptor.source`. Reverses the v1.6 regression; restores and extends the v1.5 accuracy. |
| 1.6 | OBS-plugin-id-type-correction | 2026-06-15 | product-owner | Prose precision fix (OBS finding): `plugin_id`/`config` are NOT fields on `InfusionUdfDescriptor` — they live on `PluginInfusionSource`, reachable via `descriptor.source`. Reworded plugin-type source wiring postcondition to name `PluginInfusionSource` as the carrier struct and `descriptor.source` as the access path. Contract semantics unchanged; implementation was already correct. |
| 1.5 | PIVOT-001-LOCAL-HIGH-2 | 2026-06-14 | product-owner | Corrected plugin-type source wiring postcondition (PIVOT-001 LOCAL HIGH-2). Prior wording named `InfusionLoader::load_all` as producer of real `Arc<PluginInfusionSource>` — incorrect: `load_all` returns specs+errors, not runtime-wired descriptors. Reworded to name `InfusionRegistry::load_spec_with_runtime` (and future boot-time runtime wiring) as the step that attaches the real `PluginInfusionSource`; `load_all` role limited to parsing and populating `plugin_id`/`config` fields. Anti-NullSource defect definition retained (a plugin-type spec reaching query execution with `NullSource` is E-INFUSE-003 equivalent). No line-number pins (TD-VSDD-091). |
| 1.4 | S-DEMO-ENRICHMENT-PIVOT-001-po-sign-off | 2026-06-14 | product-owner | Closed NullSource gap: added plugin-type source wiring postcondition — plugin-type descriptors MUST carry Arc<PluginInfusionSource> (not NullSource) or loading is a defect equivalent to E-INFUSE-003. Needed for PIVOT-001 AC-003 Phase 3 / NullSource-replacement task. |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-048); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
