---
document_type: story
story_id: S-SPEC-TYPE-UNIFICATION-001
title: "Retire types::SensorSpec — Unify on spec_parser::SensorSpec as Canonical"
wave: 4
epic_id: wave-4-operations
priority: P1
status: draft
version: "v1.1"
level: "L4"
producer: story-writer
timestamp: "2026-05-27T00:00:00Z"
created: "2026-05-23"
modified: "2026-05-27"
tdd_mode: strict
subsystems: [SS-06, SS-16]
# Subsystem anchor justifications:
#   SS-06 (Client Configuration): owns ConfigSnapshot and the ArcSwap hot-reload
#   infrastructure in prism-spec-engine. Changing ConfigSnapshot::sensor_specs field
#   type is an SS-06 change.
#   SS-16 (Sensor Adapter Engine): owns spec_parser::SensorSpec (canonical type)
#   and parse_spec_directory. Retiring types::SensorSpec from this module is SS-16.
crates_touched: [prism-spec-engine, prism-bin, prism-ocsf]
# prism-ocsf is touched by Task 8 (test fixture updates):
#   - crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs: update
#     `types::SensorSpec as HotReloadSensorSpec` import + new_hot_reload construction
#   - crates/prism-ocsf/tests/proptest_extensions.rs: update Arbitrary impl to use
#     spec_parser::SensorSpec
# Note: prism-core is NOT touched by Approach D — types::ConfigSnapshot is in
# prism-spec-engine, not prism-core. The prism-core shell struct (config.rs) is
# unaffected. See ADR-030 §Critical Finding.
target_module: prism-spec-engine
behavioral_contracts:
  - BC-2.16.001  # Sensor Spec File Loading — ConfigSnapshot::sensor_specs is the
                 # output of parse_spec_directory; changing its value type touches
                 # the loading contract.
  - BC-2.06.012  # Per-Tenant Overlay Loading — OverlayLoader::load_overlays expects
                 # spec_parser::SensorSpec; after unification build_type_spec_map_for_overlay
                 # is deleted and replaced by reuse of ConfigSnapshot::sensor_specs.
verification_properties: []
# Expected VPs: 1 regression VP — ConfigSnapshot::sensor_specs contains no
# SensorSpec with stringly-typed auth_type field after unification.
depends_on:
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Introduces build_type_spec_map_for_overlay
                                        # (the duplicate parse helper) and documents
                                        # the type mismatch at boot.rs. Must be merged
                                        # before this story ships.
blocks: []
# No current story blocks on this; unification is a structural cleanup that
# unlocks cleaner boot-path reasoning but does not gate any Wave 4 feature.
estimated_days: 3
points: 5
# Estimate rationale (Approach D — no ADR work required; ADR-030 already ACCEPTED):
#   Day 1: Add 3 fields to spec_parser::SensorSpec (#[serde(default)]); update
#           parse_and_validate_spec_toml return type; update config_manager.rs + hot_reload.rs
#           + add_sensor_spec.rs (6 sites).
#   Day 2: Update ConfigSnapshot::sensor_specs field type; delete
#           build_type_spec_map_for_overlay from boot.rs; refactor
#           step4_load_sensor_specs_with_overlays; update validation.rs.
#   Day 3: Update prism-bin test helpers (plugin_boot_tests.rs),
#           prism-ocsf fixtures (proptest_extensions.rs +
#           spec_driven_mapper_fixtures.rs); update EXPECTED in ci.yml (§D6);
#           implement on-demand SensorTableDescriptor conversion (§D7);
#           verify 3-CLEAN cascade.
# Approach D reduces scope vs original estimate: no new crate, no dep-cycle surgery,
# all changes within prism-spec-engine and its callers.
traces_to:
  - F-LP2-LOW-001  # Adversary finding from S-CONFIG-MULTI-TENANT-OVERRIDE-001
                   # cascade pass-2: per-boot duplicate parsing exposure;
                   # type proliferation root cause.
  - plugin-system-audit-2026-05-08.md:115  # Original finding: "Unify SensorSpec types
                                           # — pick spec_parser::SensorSpec as canonical,
                                           # retire types::SensorSpec" (item 4 of ordered
                                           # remediation list; predates current cascade).
  - adversary-pass-2-S-PLUGIN-PREREQ-C:115  # First adversarial flagging; same recommendation.
inputs:
  - .factory/specs/architecture/decisions/ADR-030-sensor-spec-type-unification.md
  - crates/prism-spec-engine/src/types.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-spec-engine/src/config_manager.rs
  - crates/prism-bin/src/boot.rs
input-hash: ""
---

# S-SPEC-TYPE-UNIFICATION-001 — Retire `types::SensorSpec`; Unify on `spec_parser::SensorSpec`

## Narrative

As a prism maintainer, I want a single `SensorSpec` type in the spec-engine codebase so
that the boot path parses each `.sensor.toml` file exactly once, the `ConfigSnapshot` carries
the structured `AuthType` enum rather than a stringly-typed `auth_type: String`, and future
adversarial cascades do not re-flag this type proliferation on every story that touches boot.rs.

## Background and Origin

This story exists because two parallel `SensorSpec` types accumulated during the spec-engine
evolution:

- `prism_spec_engine::types::SensorSpec` — legacy type with `auth_type: String`. Defined in
  `types.rs:156`. Stored in `ConfigSnapshot::sensor_specs`. Has hot-reload infrastructure
  fields: `file_hash: String`, `source_path: String`, `mode: DtuMode`.
- `prism_spec_engine::spec_parser::SensorSpec` — canonical type with `auth_type: AuthType`
  enum. Produced by `SpecLoader::parse`. Used by `OverlayLoader::load_overlays`, `pipeline.rs`,
  `auth_provider.rs`, `overlay.rs`. Does NOT have `file_hash`, `source_path`, or `mode`.

### Corrected Dependency Analysis (ADR-030 §Critical Finding)

The v0.1 story spec described `ConfigSnapshot` as residing in `prism-core` — this was
historically aspirational, not currently accurate:

- `prism_core::config::ConfigSnapshot` (shell struct) contains only `version: u64` and
  `raw: serde_json::Value`. Its module comment confirms: "This stub exists solely so
  downstream crates can hold a `ConfigSnapshot` reference without depending on prism-spec-engine."
- `prism_spec_engine::types::ConfigSnapshot` is the **real** `ConfigSnapshot` — it carries
  `sensor_specs: HashMap<String, SensorSpec>`, `failed_specs`, and `snapshot_hash`.

**Consequence:** No dep-cycle resolution is required. Both `types::SensorSpec` and
`spec_parser::SensorSpec` are in `prism-spec-engine`. The unification requires no crate
boundary crossing. No new dep edges are introduced by Approach D (ADR-030).

### The Concrete Runtime Problem

`boot.rs::build_type_spec_map_for_overlay` (lines 890-938 as of S-CONFIG-MULTI-TENANT-OVERRIDE-001
merge) performs a SECOND parse of every `.sensor.toml` file because `OverlayLoader::load_overlays`
requires `spec_parser::SensorSpec` (with `AuthType` enum) but `ConfigSnapshot::sensor_specs`
stores `types::SensorSpec` (with `auth_type: String`). With 4+ production sensor TOMLs
(armis, claroty, crowdstrike, cyberint), every boot incurs 8+ TOML parses instead of 4.
This is a SOUL.md §4 violation: data that exists is re-derived.

### Type 3 Is Out of Scope

`prism_sensors::adapter::SensorSpec` (per-request runtime fetch descriptor) is categorically
different — it carries `source_table`, `org_id`, and opaque `sensor_config: serde_json::Value`
for a single query execution. ADR-030 explicitly excludes it. Do NOT modify adapter.rs.

### Field Delta Resolved by Approach D

| Field | `types::SensorSpec` | `spec_parser::SensorSpec` | Resolution |
|-------|--------------------|--------------------------|-----------------------------|
| `auth_type` | `String` | `AuthType` enum | Enum wins; callers use `.auth_type.as_str()` |
| `tables` | `Vec<SensorTableDescriptor>` | `Vec<TableSpec>` | `TableSpec` wins; on-demand conversion for MCP listing |
| `file_hash` | `String` | **absent** | Added to `spec_parser::SensorSpec` with `#[serde(default)]` |
| `source_path` | `String` | **absent** | Added to `spec_parser::SensorSpec` with `#[serde(default)]` |
| `mode` | `DtuMode` | **absent** | Added to `spec_parser::SensorSpec` with `#[serde(default)]` |
| All other fields | Present | Present | Identical; no migration needed |

## Governing ADR

**ADR-030** — "SensorSpec Type Unification — Dep-Cycle Resolution Approach for
`types::SensorSpec` Retirement" — status: **Proposed** (accepted pending human review).
Selects **Approach D: Field-Augment `spec_parser::SensorSpec`; Delete `types::SensorSpec`**.

See `.factory/specs/architecture/decisions/ADR-030-sensor-spec-type-unification.md` for
full approach comparison, dep-cycle analysis, §D2–§D7 implementation plan, and risk register.

Implementation does not begin until ADR-030 status is ACCEPTED.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.16.001 | Sensor Spec File Loading | active | `parse_spec_directory` returns `ConfigSnapshot`; value type of `sensor_specs` changes from `types::SensorSpec` to `spec_parser::SensorSpec` (implementation detail, not behavioral change) |
| BC-2.06.012 | Per-Tenant Overlay Loading | active | `build_type_spec_map_for_overlay` is deleted; `step4_load_sensor_specs_with_overlays` passes `config_snapshot.sensor_specs` directly to `OverlayLoader::load_overlays` — same behavioral outcome |

## Acceptance Criteria

### AC-001 — Zero remaining usages of `types::SensorSpec` (traces to BC-2.16.001 postcondition 1)

After this story merges, `grep -rn "types::SensorSpec" crates/` returns zero results
(excluding any `type SensorSpec =` re-export stub if semver requires one — but Approach D
deletes the struct outright; no re-export needed). Red Gate test name:
`test_S_SPEC_TYPE_UNIFICATION_001_001_no_types_sensor_spec_usages_remain`

Implementer verification command (run before declaring AC satisfied):
```
grep -rn "types::SensorSpec\|new_hot_reload" crates/
```
Expected: zero non-test lines (test helpers updated in-scope per §D5).

### AC-002 — `build_type_spec_map_for_overlay` deleted (traces to BC-2.06.012 postcondition 2)

`boot.rs::build_type_spec_map_for_overlay` function (lines 890-938 post-S-CONFIG merge) is
removed. `step4_load_sensor_specs_with_overlays` passes `config_snapshot.sensor_specs` directly
to `OverlayLoader::load_overlays` — no second directory scan, no second TOML parse. Red Gate
test name: `test_S_SPEC_TYPE_UNIFICATION_001_002_build_type_spec_map_helper_deleted`

### AC-003 — Single-parse boot verified by test (traces to BC-2.16.001 postcondition 2)

A new integration test (or extension of an existing boot integration test) asserts that
`SpecLoader::parse` is called exactly N times (where N = number of `.sensor.toml` files in
the test fixture directory) across a full boot sequence — not 2N. The test uses a mock or
counter-instrumented `SpecLoader` to count parse invocations. Red Gate test name:
`test_S_SPEC_TYPE_UNIFICATION_001_003_spec_loader_parse_called_n_not_2n_times`

### AC-004 — `ConfigSnapshot::sensor_specs` carries structured `auth_type` (traces to BC-2.16.001 postcondition 3)

After unification, accessing `snapshot.sensor_specs[sensor_id].auth_type` returns the
structured enum variant (e.g., `AuthType::ApiKey`), not a raw `String`. Existing callsites
that string-compared `auth_type` (e.g., boot.rs step 5 Rule C enforcement) are updated to
use `sensor_spec.auth_type.as_str()`. Red Gate test name:
`test_S_SPEC_TYPE_UNIFICATION_001_004_auth_type_is_enum_not_string`

### AC-005 — `#[non_exhaustive]` EXPECTED count decremented in `ci.yml` (traces to BC-2.16.001 invariant 1)

The compile-fail gate at `tests/external/non_exhaustive_violation/` enforces an `EXPECTED=N`
count in `ci.yml`. Deleting `types::SensorSpec` (which carries `#[non_exhaustive]`) decrements
the count by 1. `spec_parser::SensorSpec` is already `#[non_exhaustive]` and already counted.
Net change: `EXPECTED` decreases by 1. The gate must pass after the update. Red Gate test name:
`test_S_SPEC_TYPE_UNIFICATION_001_005_non_exhaustive_expected_count_correct`

The implementer verifies: (a) the compile-fail gate passes with the updated EXPECTED, and
(b) `spec_parser::SensorSpec` appears exactly once in the gate's enforced type list.

### AC-006 — `SensorTableDescriptor` on-demand conversion does not break `list_sensor_specs` MCP response (traces to BC-2.16.001 postcondition 4)

After unification, `ConfigSnapshot::sensor_specs` holds `spec_parser::SensorSpec` whose
`tables` is `Vec<TableSpec>`. The `list_sensor_specs` MCP handler builds `SensorSpecEntry`
(retained MCP wire type) by converting `TableSpec → SensorTableDescriptor` on-demand.
Existing `list_sensor_specs` tests must pass without modification to their assertions.
Red Gate test name: `test_S_SPEC_TYPE_UNIFICATION_001_006_list_sensor_specs_response_unchanged`

### AC-007 — No regression in 3-CLEAN adversarial cascade (traces to BC-2.16.001 invariant 2)

The story's local adversarial cascade achieves 3-CLEAN (strict) convergence per BC-5.39.001.
Specific focus: SAP-1 (tracing emission catalog completeness) applies to any new tracing sites
added during the boot-path refactor. SAP-2 (DTU↔TOML schema parity) does not apply here
(no sensor TOML schema changes).

## Tasks

### Task 1 — Augment `spec_parser::SensorSpec` with 3 hot-reload fields (ADR-030 §D2)

**File:** `crates/prism-spec-engine/src/spec_parser.rs`

Add to the `SensorSpec` struct definition:

```rust
/// SHA-256 hash of the source file content (for hot-reload change detection).
/// Set by the file-loading caller immediately after parse.
/// Empty string indicates in-memory-constructed spec (AddSensorSpec MCP path).
#[serde(default)]
pub file_hash: String,

/// Source file path of the `.sensor.toml` file from which this spec was parsed.
/// Set by the file-loading caller. Empty string for in-memory-constructed specs.
#[serde(default)]
pub source_path: String,

/// DTU deployment mode — set at parse time from `[sensor]` TOML table.
/// Defaults to `DtuMode::Shared` for backward compatibility (BC-3.2.005).
#[serde(default)]
pub mode: DtuMode,
```

Add import: `use crate::types::DtuMode;` (already in scope via types re-export; confirm).
All three fields are `#[serde(default)]` so existing TOML files without these fields parse
correctly (they are infrastructure metadata populated post-parse by the loading caller).

### Task 2 — Update `add_sensor_spec.rs` return type (ADR-030 §D3)

**File:** `crates/prism-spec-engine/src/add_sensor_spec.rs`

`parse_and_validate_spec_toml` currently returns `Result<types::SensorSpec, Vec<ValidationError>>`.
Change return type to `Result<spec_parser::SensorSpec, Vec<ValidationError>>`.

Recommended approach per ADR-030: route through `SpecLoader::parse(toml_content)` as the
primary parse path, then set `file_hash`, `source_path`, and `mode` post-call. This avoids
reimplementing the `RawSpec → spec_parser::SensorSpec` mapping that `SpecLoader::parse` already
owns. `parse_and_validate_spec_toml` becomes a thin wrapper.

Auth type mapping: `SpecLoader::parse` uses serde with `AuthType`'s `#[serde(rename_all = "snake_case")]`,
so `auth_type: "api_key"` in TOML → `AuthType::ApiKey` automatically. No manual enum mapping.

Two callsites in `add_sensor_spec.rs` use the return type — update both.

### Task 3 — Update `config_manager.rs` and `hot_reload.rs` (ADR-030 §D4)

**File:** `crates/prism-spec-engine/src/config_manager.rs`

All sites that construct or populate `types::SensorSpec` change to construct/populate
`spec_parser::SensorSpec`. The post-construction field assignment pattern (line 116 area:
`spec.file_hash = file_hash`) is preserved — the caller sets `file_hash`, `source_path`,
and `mode` immediately after `SpecLoader::parse` returns.

**File:** `crates/prism-spec-engine/src/hot_reload.rs`

Hot-reload change detection reads `spec.file_hash` and `spec.source_path`. These fields
are now on `spec_parser::SensorSpec` (Task 1). No behavioral change; field access syntax
is identical.

### Task 4 — Update `validation.rs` (ADR-030 §D4)

**File:** `crates/prism-spec-engine/src/validation.rs`

`validation.rs` currently accesses both types (2 callsites per ADR-030 callsite analysis).
After unification it uses only `spec_parser::SensorSpec`. The `validate_auth_plugin_registered`
function already accepts `spec_parser::SensorSpec`; no change needed there. Update the 2
callsites that reference `types::SensorSpec` directly.

### Task 5 — Change `ConfigSnapshot::sensor_specs` field type (ADR-030 §D4)

**File:** `crates/prism-spec-engine/src/types.rs`

```rust
// Before
pub sensor_specs: std::collections::HashMap<String, SensorSpec>,

// After
pub sensor_specs: std::collections::HashMap<String, crate::spec_parser::SensorSpec>,
```

Update the import and field declaration. `types::SensorSpec` struct itself is deleted in Task 7.

### Task 6 — Refactor `boot.rs` (ADR-030 §D4, primary target)

**File:** `crates/prism-bin/src/boot.rs` (6 callsites per ADR-030 callsite analysis)

Sub-tasks in order:

1. **Delete `build_type_spec_map_for_overlay`** (lines 890-938 post-S-CONFIG merge). This
   is the double-parse helper that this story exists to eliminate.

2. **Refactor `step4_load_sensor_specs_with_overlays`**: pass
   `config_snapshot.sensor_specs.clone()` (or reference) directly to
   `OverlayLoader::load_overlays`. The secondary directory scan is gone.

3. **Update boot.rs step 5 Rule C enforcement**: `sensor_spec.auth_type` (String comparison)
   → `sensor_spec.auth_type.as_str()` (enum method). `AuthType::as_str()` returns the
   canonical snake_case string identical to what the TOML `auth_type` field contains.

4. **Sweep all remaining `types::SensorSpec` references** in boot.rs and update to
   `spec_parser::SensorSpec` or remove (if the reference was to the now-deleted type).

### Task 7 — Delete `types::SensorSpec` (ADR-030 §D5)

**File:** `crates/prism-spec-engine/src/types.rs`

Delete:
- `struct SensorSpec { … }` (lines 156-213)
- `impl SensorSpec { new_hot_reload, with_auth_plugin }` methods
- Any `#[derive(…)]` and `#[non_exhaustive]` annotations on the deleted struct

Retain all other types in `types.rs`: `ConfigSnapshot`, `SensorSpecEntry`,
`SensorTableDescriptor`, `ColumnDef`, `ValidationError`, `DtuMode`, `CredentialRef`,
`SpecStatus`, `ClientStatus`, `AddSensorSpecArgs/Result`, etc.

### Task 8 — Update test helpers (ADR-030 §D5)

**File:** `crates/prism-bin/tests/plugin_boot_tests.rs`

One external test caller (`plugin_boot_tests.rs:1393`) constructs
`prism_spec_engine::types::SensorSpec::new_hot_reload(…)`. Update to construct
`prism_spec_engine::spec_parser::SensorSpec { …, ..Default::default() }`.

**File:** `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs`

Imports `types::SensorSpec as HotReloadSensorSpec` and calls `new_hot_reload`. Update to
import and construct `spec_parser::SensorSpec` directly.

**File:** `crates/prism-ocsf/tests/proptest_extensions.rs`

Same pattern as fixtures file — update `Arbitrary` impl for `SensorSpec` to use
`spec_parser::SensorSpec`.

### Task 9 — Implement on-demand `SensorTableDescriptor` conversion (ADR-030 §D7)

**File:** `crates/prism-spec-engine/src/types.rs` (or adjacent `list_sensor_specs` handler)

`SensorSpecEntry::tables` is `Vec<SensorTableDescriptor>`. After unification,
`ConfigSnapshot::sensor_specs[id].tables` is `Vec<TableSpec>`. Implement a conversion:

```rust
fn sensor_table_descriptor_from_table_spec(ts: &TableSpec) -> SensorTableDescriptor {
    SensorTableDescriptor {
        table_name: ts.table_name.clone(),
        columns: ts.columns.iter().map(ColumnDef::from_column_spec).collect(),
        steps_count: ts.fetch_steps.len(),
        pagination_type: ts.pagination.pagination_type_name(),
    }
}
```

Field mapping is 1-to-1; no information is lost. `SensorTableDescriptor` is retained as
the MCP wire type (AC-006).

### Task 10 — Update `EXPECTED` in `ci.yml` (ADR-030 §D6)

**File:** `.github/workflows/ci.yml` (or equivalent CI config)

Decrement `EXPECTED=N` (currently 36 per S-PLUGIN-PREREQ-C AC-5) by 1, to `EXPECTED=35`.
`types::SensorSpec` is `#[non_exhaustive]` and was counted; deleting it removes one entry.
`spec_parser::SensorSpec` was already counted; net change is -1. Verify the compile-fail
gate passes after the change.

### Task 11 — SAP-1 sweep: tracing emission catalog check

Run `rg 'event_type\s*=' crates/ --type rust` after all code changes. For any `event_type`
value in the changed files, verify a corresponding row exists in BC-2.16.002 Structured Event
Catalog with full field schema, audit role, and recurrence policy. Any new emission site
without a catalog row is a P1 finding per CLAUDE.md §Conventions.

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| ADR-030 (full) | ~5,000 |
| BC-2.16.001 | ~2,000 |
| BC-2.06.012 | ~1,500 |
| `spec_parser.rs` (read for struct shape) | ~4,000 |
| `types.rs` (read for deletion target + ConfigSnapshot) | ~4,000 |
| `boot.rs` (read for build_type_spec_map_for_overlay + step4 + step5) | ~5,000 |
| `config_manager.rs` (read for callsites) | ~3,000 |
| `add_sensor_spec.rs` (read for return type + callsites) | ~2,500 |
| `validation.rs` (read for 2 callsites) | ~1,500 |
| `hot_reload.rs` (read for file_hash/source_path usage) | ~1,500 |
| Test files (3 files) | ~3,000 |
| BC-2.16.002 (for SAP-1 catalog check) | ~3,000 |
| Test output / compiler output | ~3,000 |
| **Total** | **~43,000** |

Well within the 20-30% agent context window threshold. No story split required.

## Previous Story Intelligence

**S-CONFIG-MULTI-TENANT-OVERRIDE-001** (blocking dependency): introduces
`build_type_spec_map_for_overlay` and documents the type mismatch at boot.rs. This story
deletes that helper. Key lesson from S-CONFIG: the dep-cycle concern in the original story
spec was based on a misunderstanding of where `ConfigSnapshot` lives. ADR-030 corrects this.

**ADR-030 authoring (2026-05-27)**: Codebase inspection revealed that `types::ConfigSnapshot`
is in `prism-spec-engine`, not `prism-core`. This eliminates all three alternative approaches
(A/B/C) as unnecessary: the unification is entirely within `prism-spec-engine`. Approach D
is strictly simpler than originally anticipated.

**SID-1 §5 discipline**: This story was deferred from S-CONFIG fix-burst-3 (F-LP2-LOW-001)
with a concrete anchor: this story ID + S-CONFIG must merge first. The deferral is now
resolved — the blocking story has merged (PR #155).

## Architecture Compliance Rules

Extracted from ADR-030, ADR-022, and CLAUDE.md §Conventions:

1. **No new dep edges.** Approach D is within `prism-spec-engine`. Do NOT add any new
   `Cargo.toml` dependency on `prism-core`, `prism-sensors`, or any other crate.

2. **`#[non_exhaustive]` discipline.** `spec_parser::SensorSpec` already carries
   `#[non_exhaustive]`. Adding fields (`file_hash`, `source_path`, `mode`) is source-compatible
   for external consumers using `..Default::default()`. Do NOT remove `#[non_exhaustive]`.

3. **No silent swallow on parse errors.** `parse_and_validate_spec_toml` routed through
   `SpecLoader::parse` must propagate all parse/validation errors — no silent `Vec::new()`
   return where partial-failure data should propagate (CLAUDE.md Standing Rule 3 §2).

4. **`prism_sensors::adapter::SensorSpec` is OUT OF SCOPE.** Do not modify
   `crates/prism-sensors/src/adapter.rs`. It is a runtime fetch descriptor, not a spec type.

5. **No `reqwest::Client::new()` without timeout.** If boot.rs refactoring touches HTTP
   client construction, the 30s timeout requirement applies (CLAUDE.md §Conventions).

6. **Structured event catalog (SAP-1).** Any new `event_type=` emission site in the
   refactored boot path requires a same-commit BC-2.16.002 catalog row.

7. **Sibling-site sweep (TD-VSDD-060).** When changing `parse_and_validate_spec_toml`
   return type, grep ALL callsites in `prism-spec-engine` and `prism-bin` before committing.

8. **EXPECTED count in `ci.yml` is authoritative.** The compile-fail gate blocks CI if
   `EXPECTED` is wrong. Decrement by exactly 1 (36 → 35).

## Library & Framework Requirements

| Dependency | Version | Usage |
|------------|---------|-------|
| `serde` | workspace pin | `#[serde(default)]` on new fields |
| `prism-spec-engine` | workspace | All changes are within this crate |
| `prism-bin` | workspace | `boot.rs` refactor; test helper updates |
| `prism-ocsf` | workspace | `proptest_extensions.rs` + fixture updates |

No new dependencies are added by Approach D. The `DtuMode` import in `spec_parser.rs` is
from `crate::types` (already in scope within `prism-spec-engine`).

## File Structure Requirements

All operations are MODIFY — no new files created, no files moved.

| File | Operation | Change Description |
|------|-----------|-------------------|
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | Add `file_hash`, `source_path`, `mode` fields to `SensorSpec` struct (Task 1) |
| `crates/prism-spec-engine/src/add_sensor_spec.rs` | MODIFY | Change `parse_and_validate_spec_toml` return type to `spec_parser::SensorSpec`; route through `SpecLoader::parse` (Task 2) |
| `crates/prism-spec-engine/src/config_manager.rs` | MODIFY | Update `types::SensorSpec` construction to `spec_parser::SensorSpec`; preserve post-parse field assignment pattern (Task 3) |
| `crates/prism-spec-engine/src/hot_reload.rs` | MODIFY | Update `types::SensorSpec` refs; `file_hash`/`source_path` field access unchanged in syntax (Task 3) |
| `crates/prism-spec-engine/src/validation.rs` | MODIFY | Update 2 callsites from `types::SensorSpec` to `spec_parser::SensorSpec` (Task 4) |
| `crates/prism-spec-engine/src/types.rs` | MODIFY | Change `ConfigSnapshot::sensor_specs` field type (Task 5); delete `SensorSpec` struct + impls (Task 7); add on-demand `sensor_table_descriptor_from_table_spec` conversion fn (Task 9) |
| `crates/prism-bin/src/boot.rs` | MODIFY | Delete `build_type_spec_map_for_overlay`; refactor `step4_load_sensor_specs_with_overlays`; update step 5 Rule C enforcement to `.as_str()` (Task 6) |
| `crates/prism-bin/tests/plugin_boot_tests.rs` | MODIFY | Update `new_hot_reload` callsite to `spec_parser::SensorSpec { …, ..Default::default() }` (Task 8) |
| `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` | MODIFY | Update `types::SensorSpec as HotReloadSensorSpec` import + construction (Task 8) |
| `crates/prism-ocsf/tests/proptest_extensions.rs` | MODIFY | Update `Arbitrary` impl to use `spec_parser::SensorSpec` (Task 8) |
| `.github/workflows/ci.yml` | MODIFY | Decrement `EXPECTED=36` to `EXPECTED=35` (Task 10) |

**Forbidden dependencies:** `prism-spec-engine` must NOT gain a new dep on `prism-sensors`
or any crate not already in its `Cargo.toml`. If cargo check reports a new dependency was
added, the build MUST fail (ADR-022 §B perimeter rule).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `.sensor.toml` without `mode` field (existing files in production) | `#[serde(default)]` on `mode: DtuMode` defaults to `DtuMode::Shared`; parse succeeds |
| EC-002 | In-memory spec constructed via `AddSensorSpec` MCP tool (no file) | `file_hash: String::new()`, `source_path: String::new()` — empty strings are valid; hot-reload skips specs with empty `file_hash` (existing behavior preserved) |
| EC-003 | `AuthType::as_str()` value differs from stored string in boot step 5 Rule C | `AuthType::as_str()` returns canonical `snake_case` string identical to TOML `auth_type` field; no behavioral change. Covered by AC-004 Red Gate test. |
| EC-004 | External test that pattern-matches on `types::SensorSpec` struct fields | Compile-fail gate (`tests/external/non_exhaustive_violation/`) catches any remaining external construction; EXPECTED count update (Task 10) enforces this |
| EC-005 | `SensorTableDescriptor` conversion produces different field values than `types::SensorSpec` construction path | `steps_count` and `pagination_type` mapping must be validated by AC-006 test; fields are 1-to-1 from `TableSpec` |
| EC-006 | Hot-reload triggers during boot while `build_type_spec_map_for_overlay` is deleted | Hot-reload uses `config_snapshot.sensor_specs` directly (same as boot does post-unification); no race condition introduced |
| EC-007 | `SpecLoader::parse` fails on a TOML that `parse_and_validate_spec_toml` previously accepted (via different code path) | Both paths perform the same TOML validation (ADR-030 Risk Register R-001 MEDIUM); test vectors from existing `add_sensor_spec` tests rerun against new path per AC-003 |

## Traceability

| Source | Reference |
|--------|-----------|
| Origin finding | F-LP2-LOW-001 (adversary-pass-2 of S-CONFIG-MULTI-TENANT-OVERRIDE-001; 2026-05-23) |
| Pre-existing audit flag | `cycles/wave-4-operations/plugin-system-audit-2026-05-08.md:115` |
| ADR | ADR-030 — Approach D selected (Proposed 2026-05-27; accepted pending human review) |
| Blocking story | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (merged PR #155 develop@3e822522) |
| Deferral authorized by | Human dispatch of architect adjudication task (per Canonical Principle Rule 3) |
| v0.1 → v1.0 | story-writer 2026-05-27: fleshed out from architect draft; ADR-030 incorporated; dep-cycle premise corrected; Tasks/Edge Cases/File Structure added; Red Gate test names added to all ACs; points 6→5 (Approach D scope reduction); crates_touched corrected (prism-core removed) |
| v1.0 → v1.1 | story-writer 2026-05-27 (MED-002 fix): `crates_touched` updated — `prism-ocsf` added to frontmatter array (was missing; STORY-INDEX v2.198 changelog had already recorded the intent but the story file was not updated). prism-ocsf is touched by Task 8: spec_driven_mapper_fixtures.rs + proptest_extensions.rs test fixture updates. Explanatory comment added to frontmatter. STORY-INDEX v2.198→v2.199. |
