---
document_type: story
story_id: PLUGIN-MIGRATION-001-C
title: "prism-ocsf: Merge 4 Hardcoded Mappers → SpecDrivenMapper + .prx WASM Transformers"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: merged
merged_at: "2026-05-27T10:53:03Z"
merged_via_pr: 158
merged_via_sha: "282013a6"
version: "v1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-27T00:00:00Z"
modified: "2026-05-27"
tdd_mode: strict
subsystems: [SS-02, SS-16, SS-17]
# Subsystem anchor justifications:
#   SS-02 (OCSF Normalization, prism-ocsf) owns the SensorMapper trait, OcsfNormalizer,
#   and all four mapper modules being deleted. SpecDrivenMapper replaces these.
#   SS-16 (Spec Engine, prism-spec-engine) owns the spec-catalog that SpecDrivenMapper
#   reads for column-level ocsf_field annotations. This story's core read path is into
#   the spec-catalog types delivered by PREREQ-B/C.
#   SS-17 (WASM Plugin Runtime) owns the PluginRuntime that SpecDrivenMapper dispatches
#   to for WASM-required mapping patterns. The dispatch hook (ocsf_transform) is the
#   plug point between SS-02 and SS-17 per ADR-023 Rule 1.
crates_touched: [prism-ocsf, prism-spec-engine]
target_module: prism-ocsf
capabilities: [CAP-005, CAP-029]
behavioral_contracts:
  - BC-2.02.002  # OcsfNormalizer — normalize_with_mappers delegates to SensorMapper;
                 #   SpecDrivenMapper is the sole registered implementation after this story
  - BC-2.02.007  # Vendor extension preservation — unmapped fields in raw_extensions; must
                 #   survive the mapper refactor unchanged (VP-017 is the gate)
verification_properties:
  - VP-151  # VP-PLUGIN-006 alias — OCSF column mapping fixture catalog, 6 representative
            # cases, byte-equal post-canonicalization per TS-PLUGIN-PARITY-001;
            # primary VP authored in this story
  - VP-016  # OCSF normalization: output is valid protobuf — proptest anti-regression
  - VP-017  # OCSF normalization: unmapped fields preserved — proptest anti-regression
depends_on:
  - S-PLUGIN-PREREQ-C  # TOML grammar extensions: ocsf_field column annotation must be
                       #   parseable from the column spec struct (already present per
                       #   spec_parser.rs line 199); virtual_field_aliases also needed
  - S-PLUGIN-PREREQ-D  # PluginRuntime boot wiring: ocsf_transform hook dispatch requires
                       #   PluginRuntime to be live at boot; without this, WASM-required
                       #   pattern dispatch is a non-functional stub
  - PLUGIN-MIGRATION-001-A  # Auth modules deleted + clean module boundary: SpecDrivenMapper
                             # must not import from the hardcoded mapper modules; 001-A
                             # establishes the clean sensor namespace before 001-C wires
                             # the new dispatch path
blocks:
  - PLUGIN-MIGRATION-001-G  # BC/ADR/doc sweep; PLUGIN-MIGRATION-001-G amends BC-2.02.003/004/005/006
                             # body text — must come after 001-C deletes those mappers and makes
                             # the sensor-named BCs structurally obsolete
points: 13
# Points justification:
#   - SpecDrivenMapper implementation (5 TOML patterns + 8 WASM dispatch patterns): ~3 pts
#   - Delete 4 mapper modules + update mod.rs + lib.rs exports: ~1 pt
#   - Wire SpecDrivenMapper into OcsfNormalizer construction at boot: ~1 pt
#   - VP-PLUGIN-006 fixture catalog (6+ test cases, byte-equal assertions): ~3 pts
#   - BC-2.02.002 amendment (reference SpecDrivenMapper explicitly): ~0.5 pt
#   - WASM ocsf_transform plugin scaffold (in-repo .prx stub for complex patterns): ~3 pts
#   - Adversary passes + parity validation against DTU fixtures: ~1.5 pts
#   Total: 13 points. Highest-complexity story in Wave 1; touches 2 crates + WASM plugin scaffold.
estimated_days: 5
risk: HIGH
# Risk justification: This story replaces the entire OCSF normalization dispatch mechanism.
# SpecDrivenMapper must produce byte-equal output to the deleted mappers for all DTU fixture data
# (TS-PLUGIN-PARITY-001 parity canon). Any regression in raw_extensions coverage (BC-2.02.007,
# VP-017) or protobuf validity (BC-2.02.002, VP-016) is a P0 defect. The WASM dispatch path
# adds a new execution layer that did not previously exist in prism-ocsf.
acceptance_criteria_count: 10
red_gate_tests: 7
estimated_passes: "3-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Parity gate (INV-PARITY-001 extension): SpecDrivenMapper must produce byte-equal OCSF output
    to the deleted hardcoded mappers for all DTU fixture data (TS-PLUGIN-PARITY-001 canonicalization).
    VP-PLUGIN-003 (VP-148) parity tests authored in PLUGIN-MIGRATION-001-D are the gate; they must
    remain GREEN after this story deletes the hardcoded mappers."
  - "VP-017 regression guard: BC-2.02.007 raw_extensions preservation is tested by VP-017 proptest.
    SpecDrivenMapper MUST populate raw_extensions for every field not covered by ocsf_field
    annotation — same contract as the deleted mappers. Adversary SAP-1 probe applies."
  - "WASM dispatch isolation: WASM-required patterns must degrade gracefully if PluginRuntime
    has no loaded plugin for a given (sensor_id, table) pair. SpecDrivenMapper returns
    PrismError::OcsfNormalizationFailed with reason 'no ocsf_transform plugin registered for
    sensor X table Y' — never panics, never silently drops fields (VP-022 invariant)."
inputs:
  - "crates/prism-ocsf/src/mappers/mod.rs"
  - "crates/prism-ocsf/src/mappers/crowdstrike.rs"
  - "crates/prism-ocsf/src/mappers/cyberint.rs"
  - "crates/prism-ocsf/src/mappers/claroty.rs"
  - "crates/prism-ocsf/src/mappers/armis.rs"
  - "crates/prism-ocsf/src/normalizer.rs"
  - "crates/prism-ocsf/src/lib.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-spec-engine/src/plugin/mod.rs"
  - ".factory/specs/behavioral-contracts/BC-2.02.002-dynamic-message-creation.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.007-raw-extensions-preservation.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-D-author-4-production-toml-sensor-specs.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-C: prism-ocsf — Merge 4 Hardcoded Mappers → SpecDrivenMapper + .prx WASM Transformers

**Story ID:** PLUGIN-MIGRATION-001-C
**Status:** draft
**Version:** v1.0
**Wave:** 1 (ordered after PLUGIN-MIGRATION-001-A; 001-C deletes the same mapper modules
that 001-A established clean namespace for)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 1 of the PLUGIN-MIGRATION saga.
This is the most architecturally significant story in Wave 1: it replaces the entire
OCSF normalization dispatch mechanism.

The four hardcoded mapper modules (`crowdstrike.rs`, `cyberint.rs`, `claroty.rs`,
`armis.rs`) encode sensor-specific OCSF field mapping directly in compiled Rust. This
contradicts ADR-023 Rule 1 (Hybrid TOML/WASM OCSF Mapping), which mandates that
column-level OCSF field mapping is declarative via `ocsf_field` TOML annotations for
the 5 TOML-mappable patterns, with `.prx` WASM transformer plugins for the 8
WASM-required patterns.

The `ocsf_field` column annotation infrastructure already exists in `prism-spec-engine`
(see `spec_parser.rs` line 199: `pub ocsf_field: Option<String>`; validated against
OCSF v1.7.0 proto schema at lines 587-603). This story connects that infrastructure to
the normalization dispatch layer.

---

## Story-Level Goal

At merge, the four hardcoded per-sensor mapper modules are deleted from
`crates/prism-ocsf/src/mappers/`. A `SpecDrivenMapper` in
`crates/prism-ocsf/src/mappers/spec_driven.rs` replaces all four, implementing the
`SensorMapper` trait by:

1. Reading column-level `ocsf_field` annotations from the spec-catalog (prism-spec-engine)
   for the 5 TOML-mappable patterns.
2. Dispatching to `.prx` WASM transformer plugins via `PluginRuntime`'s `ocsf_transform`
   hook for the 8 WASM-required patterns.

`OcsfNormalizer` is constructed with `SpecDrivenMapper` at boot. The VP-PLUGIN-006
fixture catalog (VP-151) is authored with a minimum of 6 cases (3 TOML-mappable,
3 WASM-required), located at
`crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs`.

---

## Narrative

As the Prism platform, I want the four hardcoded OCSF mapper modules replaced by a
`SpecDrivenMapper` that reads `ocsf_field` column annotations from the spec-catalog and
dispatches to `.prx` WASM transformer plugins for complex cases, so that OCSF
normalization is entirely data-driven and can handle new sensors without Rust code changes,
as mandated by ADR-023 Rule 1.

---

## Functional Summary

1. **Implement `SpecDrivenMapper`** at `crates/prism-ocsf/src/mappers/spec_driven.rs`:
   - Implements `SensorMapper` trait
   - Constructor: `SpecDrivenMapper::new(spec_catalog: Arc<SpecCatalog>, plugin_runtime: Arc<PluginRuntime>) -> Self`
   - `sensor_id()`: returns `"*"` (handles all sensors dynamically via spec-catalog lookup)
     OR is dynamically instantiated per sensor; see §Architecture Compliance Rules
   - `record_types()`: returns `&["*"]` (delegates record_type filtering to spec-catalog lookup)
   - `map()` algorithm:
     a. Look up `SensorSpec` from `spec_catalog` by `sensor_id`; error if not found
     b. Find `table_spec` matching `record_type` within the spec; error if not found
     c. For each column in `table_spec.columns`:
        - If `column.ocsf_field` is `Some` AND the column's mapping pattern is in the
          TOML-mappable set (string-to-string, nullable, integer-to-string cast,
          identity/no-op, RFC3339 timestamp): apply TOML-driven mapping directly
        - If `column.ocsf_field` is `None` OR the column requires a WASM-required
          pattern: dispatch to `plugin_runtime.call_ocsf_transform(sensor_id, record_type, raw)`
        - If the field has no `ocsf_field` annotation and no WASM transform: place in
          `extensions` (raw_extensions preservation per BC-2.02.007)
     d. Ensure every input field appears in either `msg` or `extensions` (no silent drops)

2. **Implement the 5 TOML-mappable patterns** in `spec_driven.rs`:
   - **String-to-string**: `raw[source_col]` (as string) → `msg[ocsf_field]`
   - **Nullable**: `raw[source_col]` may be `null`; propagate `null` to OCSF optional field
   - **Integer-to-string cast**: `raw[source_col]` as `i64`; call `.to_string()`;
     set on `msg[ocsf_field]` (observed in Armis `id`, `alertId`)
   - **Identity/no-op**: source column name equals `ocsf_field` target; pass through
   - **RFC3339 timestamp**: parse source string via `DateTime::parse_from_rfc3339`;
     fallback to `%Y-%m-%dT%H:%M:%S` naive parse; map to OCSF epoch-millis on `msg[ocsf_field]`

3. **Implement WASM dispatch** in `spec_driven.rs`:
   - Call `plugin_runtime.call_ocsf_transform(sensor_id, record_type, raw)` for
     WASM-required patterns
   - The WASM plugin receives the full raw record and returns a partial OCSF field set
   - Merge the WASM plugin's output into `msg`; unrecognized fields go to `extensions`
   - On `PluginRuntime` returning `Err` or no plugin registered: return
     `PrismError::OcsfNormalizationFailed { source_id, reason: "no ocsf_transform plugin ..." }`
     (never panic, never silently drop — VP-022 invariant)

4. **Author in-repo `.prx` WASM plugin scaffold** for complex-transform patterns:
   - Plugin source crate at `crates/plugins/ocsf-complex-transforms/`
   - `Cargo.toml`: `[lib] crate-type = ["cdylib"]`, targeting `wasm32-wasi`
   - Implements the `ocsf_transform` WIT interface
   - Handles the 8 WASM-required patterns (per ADR-023 Rule 1): multi-field combination,
     non-RFC3339/unix timestamp, array/list mapping, enum coercion,
     nested struct flattening, conditional mapping, timestamp fallback chain,
     unit conversion
   - Built via `just plugin-build ocsf-complex-transforms`; output artifact:
     `target/wasm32-wasi/release/ocsf_complex_transforms.prx`
   - NOT part of workspace `[workspace.members]`; excluded from main `cargo build/test`

5. **Delete the 4 hardcoded mapper modules**:
   - `crates/prism-ocsf/src/mappers/crowdstrike.rs` (120 lines)
   - `crates/prism-ocsf/src/mappers/cyberint.rs` (140 lines)
   - `crates/prism-ocsf/src/mappers/claroty.rs` (140 lines)
   - `crates/prism-ocsf/src/mappers/armis.rs` (202 lines)
   - Total: 602 lines deleted

6. **Update `mappers/mod.rs`** and `lib.rs`:
   - Remove `pub mod {crowdstrike,cyberint,claroty,armis};`
   - Remove `pub use {Crowdstrike,Armis,Claroty,Cyberint}Mapper;`
   - Add `pub mod spec_driven;`
   - Add `pub use spec_driven::SpecDrivenMapper;`
   - Update `lib.rs` re-exports accordingly

7. **Wire `SpecDrivenMapper` into `OcsfNormalizer` construction at boot**:
   - Update the boot path (wherever `OcsfNormalizer::with_mappers(...)` is called) to
     construct `SpecDrivenMapper::new(spec_catalog.clone(), plugin_runtime.clone())`
     and pass `vec![Box::new(spec_driven_mapper)]`
   - Remove the explicit per-sensor mapper construction
     (`CrowdStrikeMapper`, `CyberintMapper`, etc.) from that boot path

8. **Author VP-PLUGIN-006 fixture catalog** at
   `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs`:
   - Minimum 6 fixtures (3 TOML-mappable, 3 WASM-required)
   - Each fixture: small `RecordBatch` + expected OCSF output
   - Assertion: byte-equal post-canonicalization per TS-PLUGIN-PARITY-001

9. **Annotate BC amendments needed** (per ADR-023 §Sweep and this story):
   - `BC-2.02.002`: add a note that `SpecDrivenMapper` is now the sole `SensorMapper`
     implementation registered in `OcsfNormalizer::with_mappers`
   - `BC-2.02.003`, `BC-2.02.004`, `BC-2.02.005`, `BC-2.02.006`: full body amendment
     is scoped to PLUGIN-MIGRATION-001-G per ADR-023; in this story, add a prefix note
     to each: "This mapper behavior is now implemented by SpecDrivenMapper reading
     ocsf_field annotations from the spec-catalog per ADR-023 Rule 1. Full BC amendment
     in PLUGIN-MIGRATION-001-G."

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.02.002 | 1.3 | DynamicMessage Creation from Sensor Records | SS-02 | **Primary** — OcsfNormalizer delegates to SpecDrivenMapper; this story's wiring is the fulfillment of the delegation contract |
| BC-2.02.007 | 1.3 | Vendor Extension Preservation in raw_extensions | SS-02 | **Anti-regression** — SpecDrivenMapper MUST place all fields without an ocsf_field annotation in extensions; VP-017 is the parity gate |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~10,000 |
| BC-2.02.002 + BC-2.02.007 (read in full) | ~6,000 |
| ADR-023 §Rule 1, §Verification Properties, §Migration Plan | ~8,000 |
| 4 mapper source files (602 lines) | ~8,000 |
| normalizer.rs + mappers/mod.rs + lib.rs | ~5,000 |
| spec_parser.rs relevant sections (ocsf_field, ocsf_class) | ~4,000 |
| plugin/mod.rs (PluginRuntime call_ocsf_transform interface) | ~4,000 |
| VP-INDEX rows VP-151/VP-016/VP-017 | ~1,000 |
| PLUGIN-MIGRATION-001-D (predecessor, read §Functional Summary) | ~3,000 |
| BC files (4 to be amended) — ~500 tokens each | ~2,000 |
| **Total estimate** | **~51,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~25.5%** |

Within the 20–30% target. The story is at the upper bound — the implementer should
read spec_parser.rs and plugin/mod.rs selectively (relevant sections only) to avoid
context pressure.

---

## Acceptance Criteria

### AC-001: `SpecDrivenMapper` implements `SensorMapper` for all 5 TOML-mappable patterns (traces to BC-2.02.002 postcondition — mapped fields from sensor record set on DynamicMessage via spec-catalog ocsf_field lookup)

`SpecDrivenMapper::map()` correctly handles all five TOML-mappable patterns:

| Pattern | Source | OCSF Target | Verification |
|---------|--------|-------------|-------------|
| String-to-string | `raw["detection_id"]` (String) | `msg["finding_info.uid"]` | `test_BC_2_02_002_spec_driven_string_to_string` |
| Nullable | `raw["optional_field"]` = null | OCSF Optional field absent | `test_BC_2_02_002_spec_driven_nullable_propagation` |
| Integer-to-string cast | `raw["id"]` (i64 e.g. 12345) | `msg["device.uid"]` = "12345" | `test_BC_2_02_002_spec_driven_int_to_string_cast` |
| Identity/no-op | `raw["finding_uid"]`, `ocsf_field = "finding_uid"` | `msg["finding_uid"]` = pass-through | `test_BC_2_02_002_spec_driven_identity_passthrough` |
| RFC3339 timestamp | `raw["created_at"]` = "2024-01-15T10:30:00Z" | `msg["time"]` = epoch-millis | `test_BC_2_02_002_spec_driven_rfc3339_timestamp` |

Each test provides a synthetic `SensorSpec` with the appropriate column `ocsf_field`
annotation, passes a raw JSON record, and asserts the output `DynamicMessage` has the
expected field value.

(traces to BC-2.02.002 postcondition — Mapped fields from the sensor record are set on
the DynamicMessage via prost-reflect runtime field access; spec-driven path via
spec-catalog ocsf_field column annotations)

### AC-002: `SpecDrivenMapper` dispatches to `.prx` WASM plugin for WASM-required patterns (traces to BC-2.02.002 postcondition — SpecDrivenMapper as sole registered implementation; ADR-023 Rule 1 WASM dispatch)

When a column's mapping pattern is in the WASM-required set (multi-field combination,
unix timestamp, array mapping, enum coercion, nested struct flatten, conditional
mapping, timestamp fallback chain, unit conversion), `SpecDrivenMapper::map()` invokes
`plugin_runtime.call_ocsf_transform(sensor_id, record_type, raw)` and merges the
returned partial OCSF field set into `msg`.

Red Gate test: `test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern`

The test uses a mock `PluginRuntime` (or a `TestPluginRuntime` stub implementing the
same trait) that records calls and returns a fixture output. Assertions:
- `call_ocsf_transform` was called exactly once with the correct `(sensor_id, record_type, raw)` triple
- The returned partial OCSF fields are merged into `msg`
- Unrecognized keys from the WASM output are placed in `extensions`

(traces to BC-2.02.002 postcondition — DynamicMessage valid after SpecDrivenMapper maps
complex-transform fields via WASM plugin; ADR-023 Rule 1 §WASM-required patterns)

### AC-003: WASM plugin absence returns `OcsfNormalizationFailed`, never panics (traces to BC-2.02.002 error case — fatal record-skipped; VP-022 never-panics invariant)

When `PluginRuntime` has no loaded plugin for a given `(sensor_id, table)` pair,
`SpecDrivenMapper::map()` returns:

```
Err(PrismError::OcsfNormalizationFailed {
    source_id: <derived from raw>,
    reason: "no ocsf_transform plugin registered for sensor '<sensor_id>' table '<record_type>'",
})
```

Red Gate test: `test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed`

The test constructs `SpecDrivenMapper` with an empty `PluginRuntime` (no plugins loaded)
and calls `map()` with a record that requires WASM dispatch. Asserts the error variant
and reason string match. Asserts no panic (test completes normally).

(traces to BC-2.02.002 error case — fatal record skipped; BC-2.02.002 invariant DI-005
protobuf validity; ADR-023 Rule 1 — WASM-required patterns must fail cleanly when no
plugin is registered)

### AC-004: All unmapped fields preserved in `raw_extensions` — BC-2.02.007 parity (traces to BC-2.02.007 postconditions — all unmapped vendor-specific fields preserved in raw_extensions; VP-017 proptest gate)

For any sensor record processed by `SpecDrivenMapper::map()`:
- Every input JSON field that has NO `ocsf_field` annotation and is NOT consumed by the
  WASM plugin output is placed verbatim in `extensions` using its original vendor field name
- The union of mapped fields (in `msg`) and extension fields (in `extensions`) equals the
  set of all input field keys (no silent drops)

Red Gate test: `test_BC_2_02_007_spec_driven_extensions_preserved`

The test provides a `SensorSpec` where 3 columns have `ocsf_field` annotations and
5 columns do not. Asserts that `msg` has exactly 3 mapped fields and `extensions`
contains exactly the 5 unmapped columns with their original names.

VP-017 proptest (`crates/prism-ocsf/src/tests/`) MUST remain GREEN after this story
merges — this is the existing proptest for raw_extensions preservation and serves as
the production-grade regression gate.

(traces to BC-2.02.007 postconditions — All unmapped vendor-specific fields preserved
in raw_extensions JSON blob using original vendor field names; No vendor data is silently dropped)

### AC-005: 4 hardcoded mapper modules deleted; compile-clean (traces to BC-2.02.002 postcondition — SpecDrivenMapper as sole SensorMapper; ADR-023 Rule 1 retirement of per-sensor mapper modules)

The following files are deleted:
- `crates/prism-ocsf/src/mappers/crowdstrike.rs`
- `crates/prism-ocsf/src/mappers/cyberint.rs`
- `crates/prism-ocsf/src/mappers/claroty.rs`
- `crates/prism-ocsf/src/mappers/armis.rs`

The following declarations are removed from `crates/prism-ocsf/src/mappers/mod.rs`:
- `pub mod armis;`, `pub mod claroty;`, `pub mod crowdstrike;`, `pub mod cyberint;`
- `pub use armis::ArmisMapper;`, `pub use claroty::ClarotyMapper;`, etc.
- The `//!` module-level doc-comment entries for the four deleted implementations

The following re-exports are removed from `crates/prism-ocsf/src/lib.rs`:
- `pub use mappers::{ArmisMapper, ClarotyMapper, CrowdStrikeMapper, CyberintMapper, SensorMapper};`
  replaced with `pub use mappers::{SensorMapper, SpecDrivenMapper};`

`cargo build -p prism-ocsf` passes with zero errors and zero unexpected warnings.

Red Gate test: `test_PLUGIN_MIGRATION_001_C_005_no_hardcoded_mapper_symbols_in_production_src`

This compile-fail-style test asserts (via `grep` or a `build.rs` assertion) that no
production source file under `crates/prism-ocsf/src/` contains the strings
`CrowdStrikeMapper`, `CyberintMapper`, `ClarotyMapper`, or `ArmisMapper` outside
`#[cfg(test)]` blocks.

(traces to BC-2.02.002 postcondition — SpecDrivenMapper is the sole registered mapper;
ADR-023 Forbidden Patterns: `prism_ocsf::mappers::<sensor_name>` module paths must not
exist in production source)

### AC-006: VP-PLUGIN-006 fixture catalog authored; 6+ cases byte-equal post-canonicalization (traces to BC-2.02.002 postconditions — DynamicMessage valid, fields correctly set; VP-151/VP-PLUGIN-006 fulfillment)

`crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` is created with a minimum of
6 test cases:

| Fixture ID | Pattern Type | Source Description |
|------------|-------------|-------------------|
| FIXTURE-001 | TOML: string-to-string | `detection_id` → `finding_info.uid` |
| FIXTURE-002 | TOML: RFC3339 timestamp | `created_at` → OCSF `time` epoch-millis |
| FIXTURE-003 | TOML: integer-to-string cast | `id` (i64) → `device.uid` (String) |
| FIXTURE-004 | WASM: enum coercion | `severity` string → `severity_id` integer |
| FIXTURE-005 | WASM: timestamp fallback chain | `last_seen`/`created_at`/`timestamp` chain |
| FIXTURE-006 | WASM: multi-field combination | `ioc_type` + `ioc_value` → evidences struct |

Each fixture provides:
- A small `RecordBatch` (or raw JSON `serde_json::Value`)
- The expected OCSF output after `SpecDrivenMapper::map()`
- Assertion: byte-equal post-canonicalization per TS-PLUGIN-PARITY-001:
  - Timestamps: stripped to date+hour granularity
  - Request IDs: stripped
  - JSON key order: normalized alphabetical
  - Floating-point: ±1 ULP tolerance for f64
  - Nullable fields: `null` and absent-key treated as equal

Red Gate test file: `test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases`
(a single test that runs all 6 fixtures via a parameterized loop or individual sub-tests)

(traces to BC-2.02.002 postconditions — Mapped fields are set on DynamicMessage with
correct types; DynamicMessage is valid per OCSF protobuf schema; ADR-023 §E VP-PLUGIN-006)

### AC-007: `OcsfNormalizer` wired with `SpecDrivenMapper` at boot; no per-sensor mapper construction (traces to BC-2.02.002 postcondition — normalizer delegates to SensorMapper; ADR-023 Rule 1 SpecDrivenMapper replaces per-sensor dispatch)

The boot path that constructs `OcsfNormalizer` is updated:
- Removes explicit construction of `Box::new(CrowdStrikeMapper)`, `Box::new(CyberintMapper)`,
  `Box::new(ClarotyMapper)`, `Box::new(ArmisMapper)`
- Constructs `SpecDrivenMapper::new(spec_catalog.clone(), plugin_runtime.clone())`
- Calls `OcsfNormalizer::with_mappers(vec![Box::new(spec_driven_mapper)])`

`cargo build -p prism-bin` (or wherever the boot path resides after PREREQ-D wiring)
passes with zero errors.

Red Gate test: `test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper`

The test constructs `OcsfNormalizer::with_mappers` with a `SpecDrivenMapper` and asserts
that a call to `normalize_with_mappers("crowdstrike", "detection", sample_raw)` reaches
`SpecDrivenMapper::map()` (observable via the `SpecDrivenMapper` invoking spec-catalog
lookup for `sensor_id = "crowdstrike"`).

(traces to BC-2.02.002 postcondition — normalizer creates DynamicMessage and delegates
field population to the registered SensorMapper; SpecDrivenMapper is that implementation)

### AC-008: In-repo WASM plugin scaffold compiles to `.prx` artifact (traces to BC-2.02.002 postcondition — complex-transform fields populated via WASM plugin; ADR-023 Rule 1 first-party ocsf_transform plugin)

`crates/plugins/ocsf-complex-transforms/` exists as a valid Rust crate with:
- `Cargo.toml`: `[lib] crate-type = ["cdylib"]`, `name = "ocsf_complex_transforms"`;
  excludes from workspace `[workspace.members]`; declares `wasm32-wasi` target
- `src/lib.rs`: implements the `ocsf_transform` WIT interface for all 8 WASM-required
  patterns (may use `todo!()` stubs for patterns not yet fully implemented, but the
  `extern "C" fn ocsf_transform_call(...)` export must be present and not panic on a
  well-formed invocation)
- `Justfile` recipe `plugin-build ocsf-complex-transforms` succeeds and produces
  `target/wasm32-wasi/release/ocsf_complex_transforms.wasm` (post-processed to `.prx`)

`just plugin-build ocsf-complex-transforms` passes without error.

(traces to BC-2.02.002 postcondition — DynamicMessage populated with complex-transform
fields via WASM plugin; ADR-023 Rule 1 in-repo .prx plugin for complex-transform patterns)

### AC-009: DTU parity GREEN for all 4 sensors after mapper deletion (traces to BC-2.02.002 postcondition — SpecDrivenMapper produces byte-equal OCSF output; VP-148/VP-PLUGIN-003 parity gate)

The DTU parity tests authored in `crates/prism-spec-engine/tests/parity/`
(PLUGIN-MIGRATION-001-D) remain GREEN (or remain `#[ignore]`-tagged per their
pre-existing DTU availability status) after this story's mapper deletion.

`SpecDrivenMapper` produces parity record outputs (TS-PLUGIN-PARITY-001 canonicalization
rules) compared to the deleted Rust mappers for all four sensors when fed identical inputs
against DTU clone fixtures.

VP-148 (VP-PLUGIN-003) is the gate: parity tests must not regress. If any parity test
was GREEN before this story and becomes RED after mapper deletion, that is a P0 blocking
defect — the implementer MUST fix the regression before declaring the story done.

(traces to BC-2.02.002 postcondition — DynamicMessage valid, fields correctly set; ADR-023
Rule 3 TOML authorship parity criterion; TS-PLUGIN-PARITY-001 canonicalization rules)

### AC-010: Workspace-wide `just check` GREEN; no VP-016/VP-017 regressions (traces to BC-2.02.002 invariant DI-005; BC-2.02.007 invariant DI-005 — no compile regression, no proptest regression)

`just check` (fmt + clippy + nextest + doctests + crate-layout) passes workspace-wide
with all pre-existing tests green after all changes. In particular:

- VP-016 proptest (`OCSF normalization: output is valid protobuf`) remains GREEN
- VP-017 proptest (`OCSF normalization: unmapped fields preserved`) remains GREEN
- VP-022 fuzz smoke (`OCSF normalizer: never panics`) remains GREEN

No test that was passing before this story is made to fail.

(traces to BC-2.02.002 invariant DI-005 — DynamicMessage conforms to compiled protobuf
descriptor; BC-2.02.007 invariant DI-005 — unknown fields preserved in raw_extensions)

---

## Tasks

Implementer: follow strict TDD discipline — write Red Gate tests first (Tasks 2–3),
confirm RED, then implement `SpecDrivenMapper` (Tasks 4–5) to drive GREEN.
Author the WASM plugin scaffold (Task 6) in parallel with TDD implementation.
Deletion (Task 7) comes AFTER SpecDrivenMapper is GREEN and parity is verified.

### Task 1: Read source files and BCs before writing any code

Read in full:
- `crates/prism-ocsf/src/mappers/{mod.rs,crowdstrike.rs,cyberint.rs,claroty.rs,armis.rs}`
- `crates/prism-ocsf/src/normalizer.rs`
- `crates/prism-ocsf/src/lib.rs`
- `crates/prism-spec-engine/src/spec_parser.rs` (lines 185-270 for ColumnSpec/ocsf_field;
  lines 580-610 for ocsf_field validation; lines 280-370 for TableSpec/ocsf_class)
- `crates/prism-spec-engine/src/plugin/mod.rs` (for `PluginRuntime::call_ocsf_transform`
  or equivalent interface — understand the exact function signature before writing dispatch code)
- BC-2.02.002, BC-2.02.007
- ADR-023 Rule 1 (§Decision Rules), §Verification Properties (VP-PLUGIN-006), §Migration Plan

Catalog the exact `ColumnSpec` struct fields (especially `ocsf_field: Option<String>`)
and the `PluginRuntime` WASM dispatch function signature.

### Task 2: Write Red Gate tests for TOML-mappable patterns (RED first)

In `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` (create the file):

Write failing tests for AC-001:
- `test_BC_2_02_002_spec_driven_string_to_string`
- `test_BC_2_02_002_spec_driven_nullable_propagation`
- `test_BC_2_02_002_spec_driven_int_to_string_cast`
- `test_BC_2_02_002_spec_driven_identity_passthrough`
- `test_BC_2_02_002_spec_driven_rfc3339_timestamp`

Also write:
- `test_BC_2_02_007_spec_driven_extensions_preserved` (AC-004)

Confirm all tests fail RED:
```
cargo nextest run -p prism-ocsf --no-fail-fast 2>&1 | head -40
```

### Task 3: Write Red Gate tests for WASM dispatch and error cases (RED)

Add to the test file:
- `test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern` (AC-002)
- `test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed` (AC-003)
- `test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper` (AC-007)
- `test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases` (AC-006)

These tests will need `TestPluginRuntime` mock or similar. Define the mock in the
test file's `#[cfg(test)]` block — do NOT put it in production code.

Confirm all new tests also fail RED.

### Task 4: Implement `SpecDrivenMapper` struct and 5 TOML-mappable patterns

Create `crates/prism-ocsf/src/mappers/spec_driven.rs`:

```rust
use std::sync::Arc;
use prism_core::PrismError;
use prost_reflect::DynamicMessage;
use prism_spec_engine::{SpecCatalog, PluginRuntime}; // adjust imports per actual crate API

pub struct SpecDrivenMapper {
    spec_catalog: Arc<SpecCatalog>,
    plugin_runtime: Arc<PluginRuntime>,
}

impl SpecDrivenMapper {
    pub fn new(spec_catalog: Arc<SpecCatalog>, plugin_runtime: Arc<PluginRuntime>) -> Self { ... }
}
```

Implement `SensorMapper::map()` with the TOML-mappable pattern logic. Run after each
pattern to drive tests GREEN:
```
just iter prism-ocsf
```

### Task 5: Implement WASM dispatch in `SpecDrivenMapper::map()`

Extend `SpecDrivenMapper::map()` with the WASM dispatch path:
- Call `plugin_runtime.call_ocsf_transform(sensor_id, record_type, raw)`
- Merge WASM output into `msg`; route unrecognized keys to `extensions`
- Return `Err(PrismError::OcsfNormalizationFailed)` when no plugin registered

Run tests:
```
just iter prism-ocsf
```
All Red Gate tests from Tasks 2–3 should now pass GREEN.

### Task 6: Author WASM plugin scaffold

Create `crates/plugins/ocsf-complex-transforms/`:
```
crates/plugins/ocsf-complex-transforms/
  Cargo.toml       (crate-type = ["cdylib"])
  src/
    lib.rs         (ocsf_transform WIT exports; 8 pattern stubs)
  ocsf-complex-transforms.wit   (WIT interface file)
```

Add `just plugin-build ocsf-complex-transforms` recipe to `Justfile`.

Run:
```
just plugin-build ocsf-complex-transforms
```
Confirm `.wasm` artifact produced (and `.prx` if post-processing step is in place).

This task does NOT require the 8 patterns to be fully implemented — stubs that return
`todo!()` for unimplemented patterns are acceptable here, as long as the WIT interface
and export surface are correct. Full pattern implementation goes in follow-on stories
per wave scheduling.

### Task 7: Delete 4 hardcoded mapper modules

```
rm crates/prism-ocsf/src/mappers/crowdstrike.rs
rm crates/prism-ocsf/src/mappers/cyberint.rs
rm crates/prism-ocsf/src/mappers/claroty.rs
rm crates/prism-ocsf/src/mappers/armis.rs
```

Update `mappers/mod.rs` and `lib.rs` per AC-005. Run:
```
cargo build -p prism-ocsf 2>&1 | head -50
```
Resolve all compile errors. Do NOT re-add any deleted symbol.

Run full crate:
```
just iter prism-ocsf
```
All tests must remain GREEN.

### Task 8: Wire `SpecDrivenMapper` into boot path and update BC prefix notes

1. Update the boot path (locate via `grep -rn "CrowdStrikeMapper\|with_mappers" crates/`) to
   construct `SpecDrivenMapper` per AC-007.
2. Add prefix notes to BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006 per §Functional
   Summary item 9. These are file edits to `.factory/specs/behavioral-contracts/`.
3. Update BC-2.02.002 to note `SpecDrivenMapper` as the sole registered implementation.

Run:
```
cargo build --workspace 2>&1 | head -50
```
Resolve all compile errors.

### Task 9: Verify DTU parity and VP-016/VP-017

Run the existing parity tests:
```
cargo nextest run -p prism-spec-engine -E 'test(parity)' --no-fail-fast
```
If any parity test regresses from GREEN to RED, stop and fix before proceeding.

Run proptest regression check:
```
cargo nextest run -p prism-ocsf --no-fail-fast
```
VP-016 and VP-017 must remain GREEN.

### Task 10: Final workspace gate

```
just check
```
Must pass GREEN. Resolve any clippy warnings. Confirm no test regressions.

Run AC-005 symbol check:
```
grep -rn "CrowdStrikeMapper\|CyberintMapper\|ClarotyMapper\|ArmisMapper" \
  crates/prism-ocsf/src/ crates/prism-bin/src/
```
Must return zero matches in production source files (outside `#[cfg(test)]` blocks).

---

## Previous Story Intelligence

**PLUGIN-MIGRATION-001-D** (direct predecessor, merged PR #153 develop@3f2de889):

- The four bundled TOML sensor specs are live at `crates/prism-sensors/specs/`. Their
  column-level `ocsf_field` annotations are the ground truth for SpecDrivenMapper's
  TOML-mappable pattern decisions. Do NOT re-derive mappings from the deleted Rust
  mappers — read the spec files directly.
- The parity tests under `crates/prism-spec-engine/tests/parity/` are live. They test
  the TOML-driven fetch path; this story's SpecDrivenMapper adds the OCSF normalization
  leg to the same pipeline. The parity tests must remain GREEN after mapper deletion.
- TS-PLUGIN-PARITY-001 canonicalization rules (authored in PREREQ-F scope) define
  byte-equal parity semantics for timestamp granularity, key ordering, and floating-point
  tolerance. VP-PLUGIN-006 fixture assertions MUST use these same rules.

**PLUGIN-MIGRATION-001-A** (direct predecessor, merged with PR gating this story's dispatch):

- The four hardcoded auth modules are deleted. The clean sensor namespace is established.
  SpecDrivenMapper can safely import from `prism-spec-engine` without encountering dead
  sensor-named symbols.
- The `init_registry_for_org` replacement wires spec-catalog dispatch at boot.
  SpecDrivenMapper's `spec_catalog` Arc is the same catalog object that `init_registry_for_org`
  reads from — they share the same loaded specs.

**S-PLUGIN-PREREQ-B** (PipelineExecutor, merged earlier in Wave 0):

- The `AuthProvider` trait and `PipelineExecutor::execute` real implementation are live.
  SpecDrivenMapper does NOT call PipelineExecutor directly — it reads from the spec-catalog
  for column metadata. The fetch pipeline is orthogonal.
- Lesson: `PluginRuntime::call_ocsf_transform` may not yet exist as a named method if
  PREREQ-D wired a different dispatch interface. Read `plugin/mod.rs` in Task 1 to confirm
  the exact function signature before writing dispatch code. Do not assume a specific name.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md`, `ADR-023`, `ADR-028`:

| Rule | Source | Enforcement |
|------|--------|-------------|
| No per-sensor match arms in SpecDrivenMapper or OcsfNormalizer | ADR-023 Forbidden Patterns — `match SensorType::` in production code | Adversary grep check |
| `prism_ocsf::mappers::<sensor_name>` module paths must not exist | ADR-023 Forbidden Patterns | compile-fail perimeter gate (AC-005 Red Gate) |
| SpecDrivenMapper reads spec-catalog for ocsf_field; NEVER hardcodes field paths | ADR-023 Rule 1 — column-level ocsf_field is declarative | Adversary review |
| WASM dispatch: call_ocsf_transform; NOT a match on field name or sensor type | ADR-023 Rule 1 §WASM-required patterns | Adversary review |
| raw_extensions preservation: every unmapped field goes to extensions | BC-2.02.007 + VP-017 | VP-017 proptest |
| `prism-ocsf` MUST NOT gain a dependency on `prism-sensors` | CLAUDE.md §Conventions (forbidden dependency pattern) | Adversary grep; Cargo.toml review |
| No `unwrap()` / `expect()` in SpecDrivenMapper::map() production path | CLAUDE.md §Conventions | Clippy + adversary |
| No `println!` in production code | CLAUDE.md §Conventions | Clippy `--deny warnings` |
| WASM plugin crate NOT in workspace `[workspace.members]` | ADR-023 Permitted Patterns — plugin source crates excluded from workspace | `cargo metadata` check |
| `PluginRuntime::call_ocsf_transform` failure returns `OcsfNormalizationFailed` | BC-2.02.002 error cases | AC-003 Red Gate test |

### Forbidden Dependencies

`prism-ocsf` MUST NOT gain a dependency on `prism-sensors`. The sensor-specific logic
is dead after mapper deletion; the spec-catalog path in `prism-spec-engine` is the
only dependency needed. If `SpecDrivenMapper` appears to require sensor-specific context,
that context must come from the spec-catalog, not from a direct `prism-sensors` import.

If the build graph would be `prism-ocsf → prism-sensors → prism-spec-engine`, that is
a circular dependency violation. Check `cargo metadata` after adding any new imports.

---

## Library and Framework Requirements

| Library | Version | Pin Source |
|---------|---------|------------|
| `prost` | 0.13.5 | `crates/prism-ocsf/Cargo.toml` line 16 |
| `prost-reflect` | 0.14.7 (features: serde) | `crates/prism-ocsf/Cargo.toml` line 19 |
| `serde_json` | 1.0.149 | `crates/prism-ocsf/Cargo.toml` line 22 |
| `serde` | 1.0.228 (features: derive) | `crates/prism-ocsf/Cargo.toml` line 23 |
| `chrono` | 0.4 (features: serde) | `crates/prism-ocsf/Cargo.toml` line 26 |
| `tracing` | 0.1.44 | `crates/prism-ocsf/Cargo.toml` line 32 |
| `wasmtime` | 44 (features: component-model) | `crates/prism-spec-engine/Cargo.toml` — do NOT add to prism-ocsf; WASM dispatch goes through prism-spec-engine's PluginRuntime |
| `prism-core` | workspace path | `crates/prism-ocsf/Cargo.toml` line 13 |
| `prism-spec-engine` | workspace path | Add to `crates/prism-ocsf/Cargo.toml` if not already present (verify first) |

Do NOT introduce `wasmtime` as a direct dependency in `prism-ocsf/Cargo.toml`. WASM
dispatch is mediated entirely by `prism-spec-engine`'s `PluginRuntime`. `prism-ocsf`
calls a Rust function on `PluginRuntime` — it does not instantiate WASM components.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-ocsf/src/mappers/crowdstrike.rs` | DELETE | After SpecDrivenMapper GREEN (Task 7) |
| `crates/prism-ocsf/src/mappers/cyberint.rs` | DELETE | After SpecDrivenMapper GREEN (Task 7) |
| `crates/prism-ocsf/src/mappers/claroty.rs` | DELETE | After SpecDrivenMapper GREEN (Task 7) |
| `crates/prism-ocsf/src/mappers/armis.rs` | DELETE | After SpecDrivenMapper GREEN (Task 7) |
| `crates/prism-ocsf/src/mappers/spec_driven.rs` | CREATE | `SpecDrivenMapper` implementation |
| `crates/prism-ocsf/src/mappers/mod.rs` | MODIFY | Remove 4 pub mod/pub use; add spec_driven |
| `crates/prism-ocsf/src/lib.rs` | MODIFY | Update re-exports; remove per-sensor mappers |
| `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` | CREATE | VP-PLUGIN-006 fixture catalog (AC-006) |
| `crates/plugins/ocsf-complex-transforms/Cargo.toml` | CREATE | WASM cdylib crate (Task 6) |
| `crates/plugins/ocsf-complex-transforms/src/lib.rs` | CREATE | ocsf_transform WIT exports |
| `crates/plugins/ocsf-complex-transforms/*.wit` | CREATE | WIT interface definition |
| `Justfile` | MODIFY | Add `plugin-build ocsf-complex-transforms` recipe |
| `.factory/specs/behavioral-contracts/BC-2.02.002-*.md` | MODIFY | Note SpecDrivenMapper as sole impl |
| `.factory/specs/behavioral-contracts/BC-2.02.003-*.md` | MODIFY | Add prefix note (see §Functional Summary item 9) |
| `.factory/specs/behavioral-contracts/BC-2.02.004-*.md` | MODIFY | Add prefix note |
| `.factory/specs/behavioral-contracts/BC-2.02.005-*.md` | MODIFY | Add prefix note |
| `.factory/specs/behavioral-contracts/BC-2.02.006-*.md` | MODIFY | Add prefix note |
| `crates/prism-sensors/specs/*.sensor.toml` | NO CHANGE | Source of ocsf_field annotations; read-only |
| `crates/prism-spec-engine/tests/parity/` | NO CHANGE | Parity tests from 001-D; must remain GREEN |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `SensorSpec` not found in spec-catalog for the given `sensor_id` | `SpecDrivenMapper::map()` returns `PrismError::OcsfNormalizationFailed { source_id: "<sensor_id>", reason: "no spec registered for sensor '<sensor_id>'" }` |
| EC-002 | `record_type` not matched by any table in the `SensorSpec` | `PrismError::OcsfUnknownRecordType` — consistent with existing `SensorMapper` trait contract |
| EC-003 | Column has `ocsf_field` annotation but the source field is absent in the raw record | Field treated as nullable (None); placed in extensions with `null` value rather than failing; debug-level log records the absent field |
| EC-004 | RFC3339 parse fails and naive-datetime fallback also fails | Field placed in `extensions` with original raw value; `tracing::warn!` emitted with field name and attempted parse error |
| EC-005 | WASM plugin output contains a key that collides with a TOML-mapped field | WASM output takes precedence; TOML-mapped value overwritten and the original placed in `extensions` with `_vendor_` prefix per BC-2.02.007 EC-02-012 |
| EC-006 | `raw_extensions` size exceeds 1MB | Truncated with `_truncated: true` marker + warning log per BC-2.02.007 error case |
| EC-007 | `PluginRuntime::call_ocsf_transform` panics inside WASM sandbox (epoch-interrupt) | Wasmtime epoch interrupt is translated to `PrismError::OcsfNormalizationFailed`; never propagates as a Rust panic to the caller (VP-022 invariant) |
| EC-008 | WASM plugin crate not yet compiled; `.prx` artifact missing at boot | `PluginRuntime::load_all_plugins` emits `tracing::warn!(event_type = "plugin_load_unsigned", ...)` per BC-2.02.016 (unsigned-plugin warning); boot continues; affected complex-transform patterns fall through to `OcsfNormalizationFailed` per AC-003 |
| EC-009 | Sensor TOML spec has a column with both `ocsf_field` annotation and a type requiring WASM (e.g., multi-field combination that cannot be expressed as a single source field) | This is a spec authorship error; `SpecDrivenMapper` logs a `tracing::warn!` and dispatches to WASM anyway (WASM takes priority for ambiguous cases) |
| EC-010 | Deleted mapper symbols referenced in integration tests outside `#[cfg(test)]` | Compile error; implementer MUST sweep all test callsites in the same burst (TD-VSDD-060 sibling-site sweep) |

---

## §Risk Assessment — Highest Architectural Impact in Wave 1

This story is the most architecturally significant in the PLUGIN-MIGRATION-001 Wave 1 because
it replaces the entire OCSF normalization dispatch mechanism. Risk mitigations:

1. **Parity gate (VP-148)**: PLUGIN-MIGRATION-001-D's parity tests (VP-PLUGIN-003) are the
   behavioral ground truth. SpecDrivenMapper must produce equivalent output. Any parity
   regression is a P0 blocker before this story can merge.

2. **Prop-test regression gate**: VP-016 and VP-017 prop-tests exercise the full normalization
   pipeline. These run in `just check` and must remain GREEN. A regression here means
   SpecDrivenMapper broke a previously-tested invariant.

3. **WASM cold-start**: The ocsf-complex-transforms plugin introduces a new WASM execution layer.
   Per ADR-023 §Negative Consequences, `InstancePre::instantiate` costs approximately 1ms per
   call. This is acceptable for the OCSF normalization pipeline (not a sub-millisecond path).
   If benchmarks show unacceptable latency, the `bench_plugin_invocation` benchmark in
   `crates/prism-spec-engine/benches/` is the diagnostic. Do NOT pre-optimize.

4. **spec-catalog availability at normalizer construction**: `SpecDrivenMapper::new` requires
   `Arc<SpecCatalog>`. The spec-catalog is loaded in the boot sequence BEFORE
   `OcsfNormalizer` is constructed. If the boot ordering is wrong, this will fail at runtime,
   not compile time. Verify the boot step ordering matches ADR-022 canonical step numbering
   after PREREQ-D wiring.

5. **BC-2.02.003/004/005/006 prefix notes**: These 4 BCs still have bodies describing
   deleted behavior. Adding the prefix note (not a full amendment) is the minimal
   correct action for this story. Full body amendment is in PLUGIN-MIGRATION-001-G scope
   per ADR-023 §Migration Plan Wave 2. Do NOT author the full body amendment here —
   that is out of scope.

---

## §Known Gaps

| Gap ID | Scope | Description | Resolution Target |
|--------|-------|-------------|-------------------|
| GAP-001-C | L3 | WASM plugin patterns 1-8 may have `todo!()` stub bodies in the ocsf-complex-transforms scaffold | PLUGIN-MIGRATION-001-F (test rewrite wave) or dedicated WASM-implementation story if needed per wave scheduling |
| GAP-002-C | L3 | VP-PLUGIN-006 WASM fixture cases (FIXTURE-004/005/006) require the ocsf-complex-transforms plugin to return fixture data; if plugin stubs return `todo!()`, WASM fixture tests will fail | SpecDrivenMapper mock path: in tests, use `TestPluginRuntime` returning hardcoded fixture data so the test does not depend on the real plugin binary |
| GAP-003-C | L2 | BC-2.02.003/004/005/006 full body amendment (replacing sensor-specific field mapping descriptions with SpecDrivenMapper semantics) is explicitly out of scope | PLUGIN-MIGRATION-001-G (Wave 2 doc sweep per ADR-023 §Migration Plan) |

---

## Dependencies

### Gating Prerequisites

| Dependency | Status | Reason Needed |
|------------|--------|--------------|
| S-PLUGIN-PREREQ-C | Must be merged | TOML grammar extensions (ocsf_field parsing) |
| S-PLUGIN-PREREQ-D | Must be merged | PluginRuntime wired at boot; call_ocsf_transform interface live |
| PLUGIN-MIGRATION-001-A | Must be merged | Clean sensor namespace; no hardcoded mapper imports remain |

### Dependency Anchor Justifications

- `depends_on: S-PLUGIN-PREREQ-C` — because `SpecDrivenMapper` reads `ColumnSpec::ocsf_field`
  from the spec-catalog; PREREQ-C delivers the grammar extensions that make this field
  correctly populated for all four sensor specs. Without PREREQ-C, the virtual_field_aliases
  and cache_ttl_secs grammar needed by the four sensor TOMLs may not parse.

- `depends_on: S-PLUGIN-PREREQ-D` — because `SpecDrivenMapper::map()` dispatches to
  `PluginRuntime::call_ocsf_transform` for WASM-required patterns. Without PREREQ-D,
  `PluginRuntime` is not wired into the boot sequence and the ocsf_transform dispatch
  path is a non-functional stub that would require `todo!()` at every WASM-required call.
  The test suite for AC-002 requires a live (or mock) PluginRuntime.

- `depends_on: PLUGIN-MIGRATION-001-A` — because 001-A establishes the clean sensor module
  boundary. After 001-A merges: (a) the four auth modules are deleted, eliminating the
  risk of naming conflicts; (b) `init_registry_for_org` uses spec-catalog dispatch, meaning
  the spec-catalog is the authoritative source for sensor identity — SpecDrivenMapper can
  trust spec-catalog sensor_id values as canonical.

- `blocks: PLUGIN-MIGRATION-001-G` — because PLUGIN-MIGRATION-001-G performs the full
  body amendment of BC-2.02.003/004/005/006. Those amendments describe what SpecDrivenMapper
  replaced. Without 001-C merging first, the amendment text would describe a system state
  that does not yet exist.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `SpecDrivenMapper` struct | `crates/prism-ocsf/src/mappers/spec_driven.rs` | Effectful (reads spec-catalog Arc; dispatches to PluginRuntime) |
| TOML-mappable pattern logic | `spec_driven.rs` (inside `map()`) | Pure (field lookup + coercion; no I/O) |
| WASM dispatch call | `spec_driven.rs` → `prism-spec-engine::plugin::PluginRuntime` | Effectful (WASM invocation via Wasmtime) |
| ocsf-complex-transforms plugin | `crates/plugins/ocsf-complex-transforms/src/lib.rs` | Effectful (WASM; receives raw record; returns partial OCSF field set) |
| `OcsfNormalizer::with_mappers` boot wiring | `crates/prism-bin/src/boot.rs` (or equivalent) | Effectful (boot-time construction) |
| VP-PLUGIN-006 fixture tests | `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` | Pure (test-only; deterministic fixtures) |
| 4 mapper module deletions | `crates/prism-ocsf/src/mappers/` | Structural (deletion) |
| BC prefix note amendments | `.factory/specs/behavioral-contracts/BC-2.02.003–006.md` | Structural (spec edit) |

---

## §Source Citations

| Artifact | Version / SHA | Authoritative Symbols |
|----------|-------------|----------------------|
| `crates/prism-ocsf/src/mappers/mod.rs` | develop current | `SensorMapper` trait; 4 pub mod/use declarations to delete |
| `crates/prism-ocsf/src/mappers/armis.rs` | develop current | `extract_armis_timestamp`; timestamp fallback chain (WASM-required pattern) |
| `crates/prism-ocsf/src/mappers/crowdstrike.rs` | develop current | `crowdstrike_severity_to_id`; enum coercion (WASM-required pattern) |
| `crates/prism-spec-engine/src/spec_parser.rs` | develop current | `ColumnSpec::ocsf_field: Option<String>` (line 199); `TableSpec::ocsf_class` (line 289) |
| `crates/prism-spec-engine/src/plugin/mod.rs` | develop current | `PluginRuntime` — verify `call_ocsf_transform` or equivalent function name |
| ADR-023 | v1.19 (2026-05-15) | §Rule 1 closed grammar; §Verification Properties VP-PLUGIN-006; §Migration Plan Wave 1/C |
| BC-2.02.002 | v1.3 | postconditions; error cases; DI-005 invariant |
| BC-2.02.007 | v1.3 | postconditions; raw_extensions preservation invariant; EC-02-012 vendor prefix |
| VP-INDEX.md | v1.33 | VP-151 (VP-PLUGIN-006 alias) anchor: PLUGIN-MIGRATION-001-C |
