---
document_type: adr
adr_id: "ADR-030"
title: "SensorSpec Type Unification — Dep-Cycle Resolution Approach for types::SensorSpec Retirement"
status: Proposed
date: "2026-05-27"
modified: "2026-05-27"
version: "1.0"
producer: architect
subsystems_affected: [SS-06, SS-16]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-SPEC-TYPE-UNIFICATION-001]
related_adrs: [ADR-022, ADR-023, ADR-029]
related_bcs: [BC-2.16.001, BC-2.06.012]
locked_decisions: []
inputs:
  - .factory/stories/S-SPEC-TYPE-UNIFICATION-001-retire-types-SensorSpec-in-favor-of-spec-parser-SensorSpec.md
  - crates/prism-spec-engine/src/types.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-sensors/src/adapter.rs
  - crates/prism-core/src/config.rs
  - crates/prism-bin/src/boot.rs
  - crates/prism-spec-engine/src/config_manager.rs
input-hash: ""
wiring_deferred_to: null
---

# ADR-030: SensorSpec Type Unification — Dep-Cycle Resolution Approach

## Status

Proposed 2026-05-27, v1.0. Ready for human review. Implementation proceeds via story
`S-SPEC-TYPE-UNIFICATION-001` after this ADR is accepted.

---

## Context

### The Three SensorSpec Types

Three distinct `SensorSpec` structs exist in the codebase. The story spec
(S-SPEC-TYPE-UNIFICATION-001 v0.1) identified two; codebase analysis at ADR-030 authoring
time revealed a third:

#### Type 1: `prism_spec_engine::types::SensorSpec` (hot-reload infrastructure type)

Defined at `crates/prism-spec-engine/src/types.rs:156`. Key characteristics:

- `auth_type: String` — stringly-typed; no enum validation at struct level
- `file_hash: String` — per-file SHA-256 hash for change detection in `hot_reload.rs`
- `source_path: String` — filesystem path of the source TOML file
- `mode: DtuMode` — DTU deployment mode (Shared vs Client); used by `reload_config.rs`
- `tables: Vec<SensorTableDescriptor>` — lightweight table descriptors (no `FetchStep`)
- `credential_refs: Vec<CredentialRef>`, `auth_plugin: Option<String>`
- Owner: `ConfigSnapshot::sensor_specs` in `prism-spec-engine::types`

Callsites (non-test): `config_manager.rs`, `hot_reload.rs`, `add_sensor_spec.rs`,
`reload_config.rs`, `validation.rs` (via raw fields), `boot.rs` (via `sensor_spec.auth_type`
string comparison in step 5 Rule C enforcement).

#### Type 2: `prism_spec_engine::spec_parser::SensorSpec` (canonical TOML parse output)

Defined at `crates/prism-spec-engine/src/spec_parser.rs:384`. Key characteristics:

- `auth_type: AuthType` — structured enum (`ApiKey`, `BearerStatic`, `Oauth2ClientCredentials`,
  `CookieRoundtrip`) with `.as_str()` method for comparison
- `tables: Vec<TableSpec>` — full table schema including `FetchStep`, `PaginationConfig`
- `rate_limit_hints: Option<RateLimitHints>` — per-sensor rate limit
- `credential_refs`, `auth_plugin: Option<String>` — same as Type 1
- No `file_hash`, `source_path`, `mode` fields
- Owner: `OverlayLoader::load_overlays`, `pipeline.rs`, `auth_provider.rs`, `overlay.rs`,
  `validation.rs`, `plugin_auth_provider.rs`

#### Type 3: `prism_sensors::adapter::SensorSpec` (per-request runtime fetch descriptor)

Defined at `crates/prism-sensors/src/adapter.rs:41`. Key characteristics:

- `source_table: String` — the specific table being queried
- `org_id: OrgId` — canonical per-org identity (BC-3.2.001)
- `client_id: String` — deprecated; retained for S-3.1.06 Red Gate phase
- `sensor_config: serde_json::Value` — opaque JSON blob of sensor-specific config
- No relationship to sensor-spec TOML definition; this is a per-request runtime object

**Scope decision:** Type 3 (`prism_sensors::adapter::SensorSpec`) is OUT OF SCOPE for this
unification. It serves a categorically different purpose — it is a runtime fetch descriptor
passed to `SensorAdapter::fetch()`, not a spec-definition type. Its `sensor_config` field
carries the per-request configuration blob extracted at query time. Renaming or merging it
with the spec-definition types would require coordinated changes across `prism-sensors`,
`prism-query`, and all adapter tests; the benefit is zero (the semantic distinction is
intentional). This ADR covers only the Type 1 ↔ Type 2 unification.

### Actual Dependency Graph

Cargo dep direction at `--depth 1`:

```
prism-core         (no dep on prism-spec-engine)
prism-spec-engine  → prism-core, prism-credentials, arc-swap, …
prism-sensors      → prism-core, prism-spec-engine, prism-storage, …
prism-bin          → prism-spec-engine, prism-sensors, prism-core, …
prism-query        → prism-sensors, prism-spec-engine, …
```

**Critical finding:** The story spec (S-SPEC-TYPE-UNIFICATION-001 v0.1, lines 83-88)
describes `ConfigSnapshot` as residing in `prism-core` — "Because `ConfigSnapshot` lives
in `prism-core` (to break the `prism-core` ↔ `prism-spec-engine` dependency cycle)."

Codebase inspection reveals this description is historically aspirational, not currently
accurate. The reality:

- `prism_core::config::ConfigSnapshot` (lines 14-28 of `crates/prism-core/src/config.rs`)
  is a **shell struct** containing only `version: u64` and `raw: serde_json::Value`. Its
  module-level doc comment confirms: "This stub exists solely so downstream crates can hold
  a `ConfigSnapshot` reference without depending on prism-spec-engine."

- `prism_spec_engine::types::ConfigSnapshot` (lines 343-360 of
  `crates/prism-spec-engine/src/types.rs`) is the **real** `ConfigSnapshot` — it carries
  `sensor_specs: HashMap<String, SensorSpec>`, `failed_specs`, and `snapshot_hash`.

**Consequence:** The dep cycle concern articulated in the story spec does NOT apply to this
specific unification. `types::ConfigSnapshot` is already in `prism-spec-engine`, and
`spec_parser::SensorSpec` is also in `prism-spec-engine`. Moving `sensor_specs` from
`types::SensorSpec` to `spec_parser::SensorSpec` requires no crate boundary crossing. No
new dep edges are introduced by the unification.

### The Concrete Runtime Problem

`boot.rs::build_type_spec_map_for_overlay` (lines 890-938) performs a SECOND parse of every
`.sensor.toml` file because `OverlayLoader::load_overlays` requires `spec_parser::SensorSpec`
(with `AuthType` enum) but `ConfigSnapshot::sensor_specs` stores `types::SensorSpec` (with
`auth_type: String`). The boot sequence comment at line 843-847 documents this explicitly:

> "This is a lightweight directory read, not a full re-validation."

With 4+ production sensor TOMLs, every boot incurs 8+ TOML parses instead of 4. The two
types diverged because `types::SensorSpec` accumulated hot-reload infrastructure fields
(`file_hash`, `source_path`, `mode`) that `spec_parser::SensorSpec` does not carry.

### Field Delta Between Type 1 and Type 2

| Field | `types::SensorSpec` | `spec_parser::SensorSpec` | Resolution for unification |
|-------|--------------------|--------------------------|-----------------------------|
| `sensor_id` | `String` | `String` | Identical |
| `name` | `String` | `String` | Identical |
| `version` | `String` | `String` | Identical |
| `auth_type` | `String` (raw) | `AuthType` (enum) | Enum wins; callers using `.auth_type` string updated to `.auth_type.as_str()` |
| `base_url` | `String` | `String` | Identical |
| `tables` | `Vec<SensorTableDescriptor>` | `Vec<TableSpec>` | `TableSpec` is richer (FetchStep, PaginationConfig); see §D1 |
| `rate_limit_hints` | absent | `Option<RateLimitHints>` | Present in `spec_parser`; carry forward |
| `file_hash` | `String` | **absent** | Must be added to `spec_parser::SensorSpec` |
| `source_path` | `String` | **absent** | Must be added to `spec_parser::SensorSpec` |
| `mode` | `DtuMode` | **absent** | Must be added to `spec_parser::SensorSpec` |
| `credential_refs` | `Vec<CredentialRef>` | `Vec<CredentialRef>` | Identical (re-export from types) |
| `auth_plugin` | `Option<String>` | `Option<String>` | Identical |

### §D1 — `SensorTableDescriptor` vs `TableSpec` (table type split)

`types::SensorSpec.tables` is `Vec<SensorTableDescriptor>` (lightweight: table name,
`Vec<ColumnDef>`, steps_count, pagination_type — no fetch steps). This is the type stored
in `ConfigSnapshot` and used by the MCP `list_sensor_specs` endpoint.

`spec_parser::SensorSpec.tables` is `Vec<TableSpec>` (full: includes `Vec<FetchStep>`,
`PaginationConfig`). This is needed by `pipeline.rs` for query execution.

Under unification, `spec_parser::SensorSpec` becomes the single stored type. The lightweight
`SensorTableDescriptor` view needed by MCP listing is produced on-demand from
`TableSpec.table_name + columns + pagination` — no information is lost, and no permanent
intermediate type is needed. The `SensorSpecEntry` type used for MCP wire serialization
(`types::SensorSpecEntry`) is RETAINED as-is (it is an MCP protocol wire type excluded from
`#[non_exhaustive]` per AC-5 adjudication; it is NOT a spec-definition type).

---

## Decision Drivers

| Driver | Constraint |
|--------|------------|
| Single parse per file per boot | Eliminate `build_type_spec_map_for_overlay` double-parse |
| No dep cycle introduction | New dep edges between `prism-core` ↔ `prism-spec-engine` are forbidden |
| Structured `auth_type` in `ConfigSnapshot` | AC-004 of S-SPEC-TYPE-UNIFICATION-001: boot Rule C enforcement uses `.as_str()` not string equality |
| `#[non_exhaustive]` discipline | `spec_parser::SensorSpec` is already `#[non_exhaustive]`; unification must not break external construction discipline |
| Hot-reload integrity | `file_hash` and `source_path` are required by `hot_reload.rs` for change detection; `mode` by `reload_config.rs` for DtuMode change tracking |
| Minimal blast radius | Prefer surgical changes inside `prism-spec-engine` over crate boundary surgery |
| Production-grade correctness | No data loss at any callsite; all `types::SensorSpec` consumers updated |

---

## Options Considered

### Approach A — `SensorSpecCore` in `prism-core`

**Mechanism:** Extract a minimal `SensorSpecCore` into `prism-core`. `ConfigSnapshot::sensor_specs`
stores `HashMap<String, SensorSpecCore>`. Boot path converts `spec_parser::SensorSpec →
SensorSpecCore`. `spec_parser::SensorSpec` remains the richer parse output.

**Dependency analysis:** `prism-core` must define `SensorSpecCore`. Since `AuthType` is
currently in `prism-spec-engine::spec_parser`, either (a) `AuthType` moves to `prism-core`
too (growing the core, non-trivial), or (b) `SensorSpecCore.auth_type` remains `String`
(missing the structured-enum benefit of the unification).

**Assessment against actual dep graph:** The premise — that `ConfigSnapshot` needs to live
in `prism-core` to break a cycle — is not realized today. `types::ConfigSnapshot` is in
`prism-spec-engine`. Adding `SensorSpecCore` to `prism-core` would be creating infrastructure
for a dep-isolation goal that has not yet been activated. This is forward-looking overhead
without present-tense benefit.

**Pros:** Dep direction explicitly managed; `prism-core` stays as the dep-isolation layer.

**Cons:**
- Adds a third spec-adjacent type (`SensorSpecCore`) when the goal is to eliminate one
- `AuthType` must either migrate to `prism-core` (grows core) or `SensorSpecCore.auth_type`
  stays `String` (fails AC-004)
- Two spec types still exist post-migration; future `prism-core::ConfigSnapshot` activation
  becomes another migration
- Conversion function (`spec_parser::SensorSpec → SensorSpecCore`) is new code surface that
  must be kept in sync

**Not chosen:** Adds complexity without eliminating the two-type problem. The dep-cycle
concern motivating this approach is not realized in the current codebase.

---

### Approach B — Move `AuthType` + canonical `SensorSpec` into `prism-core`

**Mechanism:** Move `AuthType` enum and a unified `SensorSpec` definition into `prism-core`.
Both `prism-spec-engine::types` and `prism-spec-engine::spec_parser` reference
`prism-core::SensorSpec`. `ConfigSnapshot::sensor_specs` in both crates uses the same type.

**Dependency analysis:** `prism-core` must gain `AuthType`, `SensorSpec`, `TableSpec`,
`RateLimitHints`, `CredentialRef`, `DtuMode` — these are all currently in `prism-spec-engine`.
Moving them to `prism-core` moves spec-parsing knowledge into what is intended to be a
thin identity/types-only crate.

**Assessment:** This is the highest-change approach: 6+ types migrate crates, all import
paths in `prism-spec-engine`, `prism-sensors`, `prism-bin`, `prism-query`, and test files
change. `prism-core` grows into a spec-aware crate, which contradicts its "shell struct"
design intent (per `config.rs:1` module comment). The blast radius is workspace-wide.

**Pros:** Truly single type everywhere; no conversion; dep direction explicit.

**Cons:**
- Largest blast radius of all approaches (workspace-wide import path changes)
- `prism-core` absorbs spec-engine domain knowledge, violating its thin-layer intent
- Requires semver bump on `prism-core` (public type additions)
- Migration of 6+ types is high-risk for a structural cleanup story

**Not chosen:** Correct goal, disproportionate cost given that the dep cycle is not
currently active.

---

### Approach C — Introduce `prism-sensor-types` crate

**Mechanism:** New zero-dep crate `prism-sensor-types` holds `SensorSpec`, `AuthType`,
`TableSpec`, etc. Both `prism-core` and `prism-spec-engine` depend on `prism-sensor-types`.

**Assessment:** This is the "correct" approach if the workspace grows to the point where
multiple independent crates need `SensorSpec`. Today, only `prism-spec-engine` owns spec
definition; `prism-sensors` and `prism-query` consume it as an upstream dep. Introducing
a new intermediary crate adds Cargo.toml changes across at minimum 5 crates, a new crate
to maintain, and compilation overhead — for a cross-crate isolation boundary that is not
needed given the current dep graph.

**Pros:** Clean separation; no dep cycle; single type; extensible.

**Cons:**
- New crate overhead (Cargo.toml, `Cargo.lock`, CI compile target)
- No concrete dep cycle to resolve in current graph
- Overkill for a within-`prism-spec-engine` duplication problem

**Not chosen:** Correct for a future multi-crate scenario; premature for the current problem.

---

### Approach D — Field-Augment `spec_parser::SensorSpec`; Delete `types::SensorSpec` (CHOSEN)

**Mechanism:** Add the three missing fields (`file_hash: String`, `source_path: String`,
`mode: DtuMode`) to `spec_parser::SensorSpec`. Change
`prism_spec_engine::types::ConfigSnapshot::sensor_specs` to
`HashMap<String, spec_parser::SensorSpec>`. Migrate `config_manager.rs`, `hot_reload.rs`,
`add_sensor_spec.rs`, `reload_config.rs`, `validation.rs`, and `boot.rs` to use
`spec_parser::SensorSpec` uniformly. Delete the `types::SensorSpec` struct. Retain
`types::ConfigSnapshot` unchanged (its import path is stable for callers).

**Dependency analysis:** All changes are within `prism-spec-engine` or in callers that
already import `prism-spec-engine`. No new dep edges introduced. No crate boundary surgery.

**Why Approach D is correct given the actual dep graph:**

1. Both types are in `prism-spec-engine`. The unification is within a single crate. No
   dep-cycle problem to solve at the crate boundary level.

2. `spec_parser::SensorSpec` is already the richer type (full `TableSpec` with `FetchStep`,
   structured `AuthType` enum, `rate_limit_hints`). It is consumed by more modules
   (`OverlayLoader`, `pipeline.rs`, `auth_provider.rs`) and is the direction the codebase
   is already trending.

3. The three missing fields (`file_hash`, `source_path`, `mode`) are hot-reload metadata
   that belongs alongside the spec definition. Adding them to `spec_parser::SensorSpec`
   is a natural extension: `file_hash` and `source_path` describe the spec's filesystem
   origin; `mode` describes its DTU deployment classification. These are spec attributes,
   not parsing-infrastructure concerns. They fit the type's identity.

4. The `build_type_spec_map_for_overlay` double-parse is eliminated cleanly:
   `step4_load_sensor_specs_with_overlays` can pass `config_snapshot.sensor_specs` directly
   to `OverlayLoader::load_overlays` once both use `spec_parser::SensorSpec`.

5. `add_sensor_spec::parse_and_validate_spec_toml` — currently the primary constructor of
   `types::SensorSpec` — is restructured to return `spec_parser::SensorSpec`. The
   `RawSpec`-to-`SensorSpec` conversion logic is largely retained; the output type changes.
   Alternatively, callers can migrate to `SpecLoader::parse` which already produces
   `spec_parser::SensorSpec` (with the caveat that `SpecLoader::parse` must gain `file_hash`,
   `source_path`, and `mode` injection capability).

**`#[non_exhaustive]` impact:** `spec_parser::SensorSpec` already carries `#[non_exhaustive]`.
Adding fields (`file_hash`, `source_path`, `mode`) is a source-compatible extension for
external consumers using the `..Default::default()` pattern. The `#[non_exhaustive]` gate
at `tests/external/non_exhaustive_violation/` tests for MISSING `#[non_exhaustive]` (it
enforces presence); the EXPECTED count covers `types::SensorSpec` — once deleted, the count
decrements by 1. Story S-SPEC-TYPE-UNIFICATION-001 must update `EXPECTED=N` in `ci.yml`
accordingly.

**Pros:**
- Zero new crate infrastructure
- No dep edges introduced or changed
- Single canonical type (`spec_parser::SensorSpec`) owns the complete spec definition
- `build_type_spec_map_for_overlay` deleted cleanly (AC-002 of S-SPEC-TYPE-UNIFICATION-001)
- `ConfigSnapshot::sensor_specs` carries structured `AuthType` enum (AC-004)
- Minimal blast radius: changes contained to `prism-spec-engine` internals and callers
  of `types::SensorSpec` (all within `prism-spec-engine` except one test in `prism-bin`)

**Cons:**
- `spec_parser::SensorSpec` grows infrastructure fields (`file_hash`, `source_path`, `mode`)
  that are not conceptually "parse output." This is a minor semantic friction — these are
  spec attributes set post-parse (by the caller who knows the file path), not derived from
  TOML content.
- `parse_and_validate_spec_toml` and hot-reload path require updates to set the three new
  fields post-construction (same pattern as today: `spec.file_hash = file_hash` at line 116
  of `config_manager.rs`).

The semantic friction is acceptable: `file_hash` and `source_path` are spec provenance
metadata; `mode` is a spec classification attribute. All three are set immediately after
parse and never mutated thereafter (consistent with `types::SensorSpec`'s existing pattern).
The alternative — maintaining a separate "enriched spec" wrapper type — would recreate the
two-type proliferation that this story is eliminating.

---

## Decision

**Adopt Approach D: Field-Augment `spec_parser::SensorSpec`; Delete `types::SensorSpec`.**

### Implementation Plan

#### §D2 — `spec_parser::SensorSpec` augmentation

Add three fields to `spec_parser::SensorSpec` (in `crates/prism-spec-engine/src/spec_parser.rs`):

```rust
/// SHA-256 hash of the source file content (for hot-reload change detection).
///
/// Set by the file-loading caller (config_manager, hot_reload) immediately
/// after parse. Empty string indicates the spec was constructed without a
/// file source (e.g., via AddSensorSpec MCP tool from in-memory TOML).
#[serde(default)]
pub file_hash: String,

/// Source file path of the `.sensor.toml` file from which this spec was parsed.
///
/// Set by the file-loading caller. Empty string for in-memory-constructed specs.
#[serde(default)]
pub source_path: String,

/// DTU deployment mode — set at parse time from the `[sensor]` TOML table.
///
/// Defaults to `DtuMode::Shared` for backward compatibility. Governs the
/// DTU topology used for this sensor's data flow (BC-3.2.005).
#[serde(default)]
pub mode: DtuMode,
```

`Default::default()` values: `file_hash: String::new()`, `source_path: String::new()`,
`mode: DtuMode::Shared`. All three are `#[serde(default)]` so existing TOML files without
these fields continue to parse (they are infrastructure metadata, not spec grammar).

The `DtuMode` type is already in `crate::types`; import within `spec_parser.rs` is:
`use crate::types::DtuMode;`. No new dep.

#### §D3 — `parse_and_validate_spec_toml` return type change

`add_sensor_spec::parse_and_validate_spec_toml` currently returns
`Result<types::SensorSpec, Vec<ValidationError>>`. Under unification it returns
`Result<spec_parser::SensorSpec, Vec<ValidationError>>`. The construction block at
`add_sensor_spec.rs:210` changes from `types::SensorSpec { … }` to `spec_parser::SensorSpec { … }`.

Key changes in the constructor:

- `auth_type: auth_type` (raw string) → `auth_type: auth_type.parse::<AuthType>()?` or
  equivalent conversion via `AuthType::from_str` (which must return `E-SPEC-xxx` on unknown
  string). Alternatively: `SpecLoader::parse` already handles the `auth_type` TOML field
  via serde with `AuthType`'s `#[serde(rename_all = "snake_case")]` — the implementer should
  route `parse_and_validate_spec_toml`'s `RawSpec → spec_parser::SensorSpec` through serde
  deserialization to avoid reimplementing the enum mapping.
- `tables: tables` — currently builds `Vec<SensorTableDescriptor>`; must build
  `Vec<TableSpec>`. The `RawTable → TableSpec` conversion logic exists in `spec_parser.rs`
  (`SpecLoader::parse` internal logic); factor it out or call `SpecLoader::parse` as the
  primary parse path and set `file_hash`/`source_path` post-call.
- `mode`, `file_hash`, `source_path`: set by the caller post-construction (same pattern as
  today).

**Recommended implementer approach:** Call `SpecLoader::parse(toml_content)` as the primary
parse path, then augment the returned `spec_parser::SensorSpec` with `file_hash`,
`source_path`, and `mode` at the callsite. This avoids duplicating the
`RawSpec → spec_parser::SensorSpec` conversion logic that `SpecLoader::parse` already owns.
`parse_and_validate_spec_toml` becomes a thin wrapper adding the file-origin metadata.

#### §D4 — `types::ConfigSnapshot::sensor_specs` field type change

```rust
// Before
pub sensor_specs: std::collections::HashMap<String, types::SensorSpec>,

// After
pub sensor_specs: std::collections::HashMap<String, spec_parser::SensorSpec>,
```

All internal usages of `ConfigSnapshot::sensor_specs` entries are updated:
- `boot.rs` step 5 Rule C enforcement: `sensor_spec.auth_type` (String comparison) →
  `sensor_spec.auth_type.as_str()` (enum method)
- `validation.rs::validate_auth_plugin_registered`: already accepts `spec_parser::SensorSpec`;
  no change needed
- `boot.rs::build_type_spec_map_for_overlay`: **deleted** (the function becomes unnecessary
  because `config_snapshot.sensor_specs` is now already `spec_parser::SensorSpec`)
- `step4_load_sensor_specs_with_overlays`: passes `config_snapshot.sensor_specs` directly
  to `OverlayLoader::load_overlays` after deleting the secondary parse helper

#### §D5 — `types::SensorSpec` deletion

`crates/prism-spec-engine/src/types.rs` struct `SensorSpec` (lines 156-213) is deleted.
Associated `impl SensorSpec { new_hot_reload, with_auth_plugin }` methods are also deleted.

One external test caller (`crates/prism-bin/tests/plugin_boot_tests.rs:1393`) constructs
`prism_spec_engine::types::SensorSpec::new_hot_reload(…)`. This test is updated to construct
`prism_spec_engine::spec_parser::SensorSpec { … , ..Default::default() }` instead.

The `prism_ocsf` test files (`spec_driven_mapper_fixtures.rs`, `proptest_extensions.rs`)
import `types::SensorSpec as HotReloadSensorSpec` and call `new_hot_reload`. These are
updated to import and construct `spec_parser::SensorSpec` directly.

#### §D6 — `#[non_exhaustive]` compile-fail gate count update

The compile-fail gate at `tests/external/non_exhaustive_violation/` enforces
`EXPECTED=N` (currently 36 per S-PLUGIN-PREREQ-C AC-5). `types::SensorSpec` is
`#[non_exhaustive]` and counted in that gate. Deleting `types::SensorSpec` decrements
the count by 1. The implementer updates `EXPECTED` in `ci.yml` when the struct is deleted.
The gate covers `prism-core`, `prism-spec-engine`, and `prism-query` public API surface
types only; `spec_parser::SensorSpec` is already counted (it is already `#[non_exhaustive]`).
Net change: -1 to EXPECTED.

#### §D7 — `SensorSpecEntry` and `list_sensor_specs` MCP response

`types::SensorSpecEntry` (used by `list_sensor_specs` MCP tool) constructs its `tables`
field as `Vec<SensorTableDescriptor>`. After unification, `ConfigSnapshot::sensor_specs`
holds `spec_parser::SensorSpec` whose `tables` is `Vec<TableSpec>`. The conversion from
`TableSpec → SensorTableDescriptor` is done on-demand when building the `SensorSpecEntry`
response. `SensorTableDescriptor` is RETAINED as an MCP wire/response type; it is NOT
part of the spec-definition type hierarchy being unified.

Implementer note: the `from_table_spec` conversion (or equivalent) produces
`SensorTableDescriptor { table_name, columns: Vec<ColumnDef>, steps_count, pagination_type }`
from `TableSpec`. This logic already exists in `add_sensor_spec.rs` (the inverse direction);
the inverse is straightforward.

---

## Consequences

### Architecture Impact

- `spec_parser::SensorSpec` becomes the single source of truth for sensor spec definition
  within `prism-spec-engine`. The hot-reload path, config-manager path, overlay path, and
  pipeline path all converge on one type.

- `types::SensorSpec` is deleted. `types.rs` retains all other types: `ConfigSnapshot`,
  `SensorSpecEntry`, `SensorTableDescriptor`, `ColumnDef`, `ValidationError`, `DtuMode`,
  `CredentialRef`, `SpecStatus`, `ClientStatus`, `AddSensorSpecArgs/Result`, etc.

- `prism-core::config::ConfigSnapshot` (shell struct) is unaffected. The long-term plan
  to use it as a dep-isolation layer (referenced in `config.rs:1` module comment) remains
  feasible as a future migration; this ADR does not activate or block it.

- `prism_sensors::adapter::SensorSpec` (Type 3, runtime fetch descriptor) is unaffected.
  It continues to serve its distinct purpose. The naming collision between all three
  `SensorSpec` types is reduced to a two-type collision (spec definition vs. runtime
  descriptor) — an acceptable outcome. Future story S-3.1.06 addresses the
  `adapter::SensorSpec` client_id deprecation independently.

### Behavioral Contracts

- **BC-2.16.001** (Sensor Spec File Loading): unaffected. `parse_spec_directory` continues
  to return `ConfigSnapshot` with a map of loaded specs; the value type changes from
  `types::SensorSpec` to `spec_parser::SensorSpec` (implementation detail).

- **BC-2.06.012** (Per-Tenant Overlay Loading): `build_type_spec_map_for_overlay` is deleted.
  `step4_load_sensor_specs_with_overlays` passes `config_snapshot.sensor_specs` directly.
  The overlay behavior (BC-2.06.012 through BC-2.06.016) is unchanged.

### Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| `parse_and_validate_spec_toml` routing through `SpecLoader::parse` introduces new parse failure modes | MEDIUM | Both parse paths currently perform the same TOML validation; test vectors from `add_sensor_spec` tests are rerun against the new path |
| `SensorTableDescriptor` on-demand construction from `TableSpec` introduces a divergence in the MCP `list_sensor_specs` response | LOW | `SensorTableDescriptor` fields map 1-to-1 from `TableSpec` fields; conversion is deterministic; existing `list_sensor_specs` tests validate output shape |
| `#[non_exhaustive]` EXPECTED count miscounted in `ci.yml` update | LOW | Implementer verifies compile-fail gate passes after count change; CI blocks merge if wrong |
| Existing tests constructing `types::SensorSpec::new_hot_reload` missed in sweep | MEDIUM | Implementer runs `grep -rn "types::SensorSpec\|new_hot_reload" crates/` before declaring AC-001 satisfied |
| `boot.rs` step 5 Rule C enforcement breaks if `.as_str()` differs from stored string | LOW | `AuthType::as_str()` returns the canonical `snake_case` string (same as TOML `auth_type` field); tested in `spec_parser` tests |

---

## Implementation Story Handoff

**Story:** `S-SPEC-TYPE-UNIFICATION-001` (existing; see `.factory/stories/`)

**Dependency:** `S-CONFIG-MULTI-TENANT-OVERRIDE-001` must be merged first. That story
introduces `build_type_spec_map_for_overlay` and documents the type mismatch in `boot.rs`.
This story deletes it.

**AC traceability:**

| Story AC | ADR Section Governing |
|----------|-----------------------|
| AC-001 (zero `types::SensorSpec` usages) | §D5 deletion + §D3 return type change |
| AC-002 (`build_type_spec_map_for_overlay` deleted) | §D4 and §D3 OverlayLoader pass-through |
| AC-003 (single-parse boot test) | §D3 + §D4; test instruments `SpecLoader::parse` call count |
| AC-004 (`auth_type` returns enum, not String) | §D2 field addition + §D4 caller update |
| AC-005 (3-CLEAN cascade) | Governed by BC-5.39.001; SAP-1 applies to any new `event_type` sites |

---

## Changelog

| Version | Pass | Date | Author | Change |
|---------|------|------|--------|--------|
| 1.0 | D-ADR-030 | 2026-05-27 | architect | Initial proposal. Registers in ARCH-INDEX v2.103. |
