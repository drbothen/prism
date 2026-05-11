---
document_type: story
story_id: S-PLUGIN-PREREQ-A
title: "SensorId(Arc<str>) Open Newtype Replaces SensorType Closed Enum (Keystone Migration)"
wave: 0
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: ready
depends_on: [S-PLUGIN-PREREQ-F]
blocks: [S-PLUGIN-PREREQ-B, S-PLUGIN-PREREQ-C, S-PLUGIN-PREREQ-D, S-PLUGIN-PREREQ-E]
points: 13
estimated_days: 4
risk: HIGH
tdd_mode: strict
crates_touched: [prism-core, prism-sensors, prism-query, prism-spec-engine]
target_module: prism-core
subsystems: [SS-01, SS-02, SS-08, SS-16]
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-05-11T00:00:00Z"
input-hash: "7d38067"
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
behavioral_contracts:
  - BC-2.01.013  # DataSource trait: spec-driven adapter pattern + open SensorId dispatch
  - BC-2.16.004  # Rust escape hatch (DEPRECATED — noted only; not retired in this story)
verification_properties:
  - VP-PLUGIN-001  # SensorId open-newtype replaces SensorType closed enum; zero non-test references post-PREREQ-A
  - VP-PLUGIN-007  # Zero hardcoded CustomAdapter Rust adapters in production code paths post-PREREQ-A
anchor_bcs: [BC-2.01.013, BC-2.16.004]
anchor_vps: [VP-PLUGIN-001, VP-PLUGIN-007]
anchor_capabilities: [CAP-001]
anchor_subsystem: [SS-01, SS-02, SS-08, SS-16]
assumption_validations: []
risk_mitigations: []
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/domain-spec/invariants.md"
---

# S-PLUGIN-PREREQ-A — SensorId(Arc<str>) Open Newtype Replaces SensorType Closed Enum (Keystone Migration)

## Narrative

As the Prism platform, I want the closed `SensorType` enum deleted from `prism-core` and
replaced with an open `SensorId(Arc<str>)` newtype, so that sensor identity is determined
by string-keyed spec declarations at runtime instead of compile-time enum variants,
unblocking the plugin-only sensor architecture where external and user-authored sensors
can be added without recompiling the platform.

---

## Behavioral Contracts

| BC ID | Title | Subsystem | Role in This Story |
|-------|-------|-----------|-------------------|
| BC-2.01.013 | DataSource Trait: Spec-Driven Adapter Pattern | SS-01 | Drives the open dispatch requirement — adapter implementations are produced from TOML SensorSpec declarations at runtime; `SensorAdapter::sensor_type()` must return `SensorId` so the registry can be keyed by sensor identity string, not enum variant |
| BC-2.16.004 | Rust Escape Hatch for Custom Adapters (DEPRECATED v1.4) | SS-16 | Awareness only — `CustomAdapter` Rust trait is NOT retired in this story; trait calls remain dispatchable through the new SensorId-keyed registry; full trait retirement is PREREQ-E scope |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `prism-core/src/types.rs` (enum deletion + newtype module) | ~1,200 |
| `prism-core/src/sensor_id.rs` (new file — full newtype impl) | ~2,000 |
| `prism-core/src/lib.rs` (re-export update) | ~400 |
| `prism-core/src/error.rs` (UnknownSensorType variant update) | ~500 |
| `prism-sensors/src/adapter.rs` (sensor_type return type change) | ~800 |
| `prism-sensors/src/registry.rs` (HashMap key type + get/register signatures) | ~1,200 |
| `prism-sensors/src/auth/crowdstrike.rs` (impl update) | ~400 |
| `prism-sensors/src/auth/cyberint.rs` (impl update) | ~400 |
| `prism-sensors/src/auth/claroty.rs` (impl update) | ~400 |
| `prism-sensors/src/auth/armis.rs` (impl update) | ~400 |
| `prism-query/src/virtual_fields.rs` (dispatch site 1) | ~600 |
| `prism-query/src/explain.rs` (dispatch sites 2 + 3) | ~1,000 |
| `prism-query/src/write_dispatch.rs` (dispatch site 4) | ~600 |
| `prism-query/src/materialization.rs` (dispatch site 5) | ~600 |
| `prism-query/src/invalidation.rs` (dispatch site 6) | ~800 |
| `prism-sensors/src/fanout.rs` (dispatch site 7) | ~600 |
| BC files (2 BCs) | ~3,000 |
| Test files (unit + integration, estimated 5–10 files updated) | ~4,000 |
| `perimeter-violation` compile-fail test (AC-6) | ~800 |
| Total | ~23,300 |

Within the 30% context window budget (~40k tokens for a 128k-context agent).

---

## Tasks

1. **Create `prism-core/src/sensor_id.rs`** — the new open newtype:
   ```rust
   use std::sync::Arc;
   use std::borrow::Borrow;

   #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
   pub struct SensorId(Arc<str>);

   impl SensorId {
       pub fn new(s: impl Into<Arc<str>>) -> Self { SensorId(s.into()) }
   }
   impl From<&str> for SensorId { ... }
   impl From<String> for SensorId { ... }
   impl From<Arc<str>> for SensorId { ... }
   impl std::fmt::Display for SensorId { ... }
   impl Borrow<str> for SensorId { ... }
   impl AsRef<str> for SensorId { ... }
   ```
   Use `Arc<str>` (not `String`) for cheap clone + intern-friendliness (Wave-1 prep).
   Implement `From<&'static str>` separately for static-string zero-alloc optimization.
   Do NOT add redundant `as_str()` — `Borrow<str>` + `Display` + `AsRef<str>` is sufficient.

2. **Delete `pub enum SensorType { ... }` from `prism-core/src/types.rs`**.
   Remove the `Display` impl block immediately following (lines 79–88).
   Remove the `#[serde(rename_all = "snake_case")]` derive block.

3. **Update `prism-core/src/lib.rs`**: replace `pub use types::{..., SensorType, ...}` with
   `pub use sensor_id::SensorId;`. Add `mod sensor_id;` declaration.
   Remove all `SensorType` re-exports.

4. **Update `prism-core/src/error.rs`**: change `UnknownSensorType { name: String }` variant
   to `UnknownSensorId { id: String }` (or remove if unused after migration). Update all
   format strings that reference `sensor_type`.

5. **Update `prism-sensors/src/adapter.rs` — trait method signature**:
   ```rust
   // Before:
   fn sensor_type(&self) -> SensorType;
   // After:
   fn sensor_type(&self) -> SensorId;
   ```
   Update the `AdapterNotFound` error variant field type accordingly.

6. **Update all four `impl SensorAdapter for ...` blocks** in `prism-sensors/src/auth/`:
   - `crowdstrike.rs:378`: return `SensorId::from("crowdstrike")`
   - `cyberint.rs:255`: return `SensorId::from("cyberint")`
   - `claroty.rs:248`: return `SensorId::from("claroty")`
   - `armis.rs:559`: return `SensorId::from("armis")`

7. **Update `prism-sensors/src/registry.rs`**:
   - Change `HashMap<(OrgId, SensorType), Arc<dyn SensorAdapter>>` → `HashMap<(OrgId, SensorId), Arc<dyn SensorAdapter>>`
   - Update `register`, `get`, and `get_all_for_sensor_type` signatures to accept `SensorId`
   - The `get_all_for_sensor_type` method becomes `get_all_for_sensor(sensor_id: &SensorId)` — use `Borrow<str>` so callers can pass `&str` directly

8. **Replace 7 match-site dispatch groups** (see AC-5 enumeration below). Each match arm
   against a `SensorType::X` variant becomes either a `HashMap<SensorId, _>` lookup or
   trait-object dispatch. No stringly-typed `==` comparisons against literals.

9. **Update test files** that reference `SensorType` literals. Grep estimate: ~10 test files
   (`execute_integration_tests.rs`, `bc_2_01_013.rs`, `bc_2_01_010.rs`, `bc_2_01_002.rs`,
   `org_id_binding.rs`, `cr013_fan_out_org_id_consistency.rs`, and related).
   Update all `SensorType::CrowdStrike` → `SensorId::from("crowdstrike")`, etc.

10. **Add or update compile-fail perimeter test** in `tests/external/perimeter-violation/`
    confirming that reintroducing `pub enum SensorType` is caught by the negative test
    infrastructure (VP-PLUGIN-001 enforcement).

11. **Write two unit tests in `prism-core/src/sensor_id.rs`** (or a `tests/` submodule):
    - `test_sensor_id_equality_hash_display_roundtrip`: construct `SensorId::from("crowdstrike")`,
      assert `Display` produces `"crowdstrike"`, insert in `HashSet`, assert contains.
    - `test_sensor_id_borrow_str_lookup`: insert `SensorId::from("armis")` in
      `HashMap<SensorId, u32>`, then look up via `map.get("armis" as &str)`.

12. **Write one integration test in `prism-sensors/` or `prism-spec-engine/`**:
    - `test_adapter_registry_sensorid_keyed`: register a mock adapter under `SensorId::from("crowdstrike")`,
      look it up, assert `Some`. Register under a different SensorId, assert no cross-lookup.

13. **Run `just check`** — must pass with zero `SensorType` references in non-test production
    code. Run `grep -rn "SensorType" crates/` to verify zero production hits before declaring done.

---

## Acceptance Criteria

**AC-1:** `SensorId(Arc<str>)` newtype exists in `prism-core/src/sensor_id.rs` (or `types.rs`)
with the following impl set: `From<&str>`, `From<String>`, `From<Arc<str>>`, `Display`,
`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Ord`, `PartialOrd`, `Borrow<str>`, `AsRef<str>`.
Zero-cost clone for `Arc<str>` payload. Static-string construction via `From<&'static str>`
avoids heap allocation.
(traces to BC-2.01.013 postcondition — adapter implementations are produced from TOML SensorSpec
declarations at runtime; sensor identity is a string key, not a compile-time enum variant)

**AC-2:** `pub enum SensorType { CrowdStrike, Cyberint, Claroty, Armis }` is DELETED from
`prism-core/src/types.rs`. After merge, `grep -rn "SensorType" crates/` returns ZERO matches
in non-test production code (files under `src/`, not `tests/`). The `Display` impl and serde
derive block are removed with it.
(traces to BC-2.01.013 postcondition — no hand-written adapter code outside prism-sensors is
required for TOML-expressible sensors; closed enum is the compile-time barrier to this)

**AC-3:** `SensorAdapter::sensor_type` trait method signature in `prism-sensors/src/adapter.rs`
is changed from `fn sensor_type(&self) -> SensorType` to `fn sensor_type(&self) -> SensorId`.
All four concrete implementations (`CrowdStrikeAdapter`, `CyberintAdapter`, `ClarotyAdapter`,
`ArmisAdapter`) return `SensorId::from("<name>")` (their canonical sensor name strings).
(traces to BC-2.01.013 postcondition — adapter implementations produce sensor identity as a
runtime string value, not a compile-time enum; `record_type` naming convention `<sensor>_<entity>`
is preserved via SensorId string)

**AC-4:** `AdapterRegistry` in `prism-sensors/src/registry.rs` keys its internal
`HashMap` by `(OrgId, SensorId)` instead of `(OrgId, SensorType)`. The `register(org_id, adapter)`
method derives `SensorId` from `adapter.sensor_type()` internally — adapter owns identity invariant,
preventing cross-method inconsistency. The `get(org_id, sensor_id)` and
`get_all_for_sensor_type(sensor_id)` lookup methods accept `SensorId` by value or reference.
(Adversary pass-1 F-LP1-HIGH-003 closure: this AC was originally drafted to specify explicit
SensorId/&str arguments to `register()`; ADOPTED current implementation where adapter owns
identity — better design than spec text, prevents inconsistent SensorId registration.)
(traces to BC-2.01.013 postcondition — registry stores adapters produced from SensorSpec
declarations keyed by sensor identity string; invariant: `get(org_a, "crowdstrike")` never
returns adapter registered under `org_b`)

**AC-5:** All 7 match-site dispatch groups across 4 crates are replaced with open dispatch
patterns (trait-object dispatch or `HashMap<SensorId, _>` lookup). No remaining `match ...SensorType::X`
in production code after migration.

Exact dispatch sites (verified by grep at story-write time; implementer must re-confirm at
implementation time):

| # | File | Line(s) | Pattern | Replacement Strategy |
|---|------|---------|---------|---------------------|
| 1 | `crates/prism-core/src/types.rs` | 79–88 | `match self { SensorType::X => "x" }` (Display impl) | Deleted with enum; `SensorId` Display delegates to inner `Arc<str>` |
| 2 | `crates/prism-query/src/virtual_fields.rs` | 163–166 | `match sensor_type { SensorType::X => "x" }` | Replace with `sensor_id.as_ref()` — SensorId IS the string |
| 3 | `crates/prism-query/src/explain.rs` | 661–664 | `match s { "crowdstrike" => Some(SensorType::CrowdStrike), ... }` | Replace with `Some(SensorId::from(s))` — any string is valid |
| 4 | `crates/prism-query/src/explain.rs` | 1046–1050 | `match src.sensor_type { SensorType::X => N }` (latency estimate) | Replace field type with `SensorId`; dispatch via `HashMap<SensorId, u64>` static lookup or `match sensor_id.as_ref() { "crowdstrike" => 250, ... _ => 300 }` |
| 5 | `crates/prism-query/src/write_dispatch.rs` | 279–283 | `match name { "crowdstrike" => SensorType::CrowdStrike, ... }` | Replace with `SensorId::from(name)` — remove match entirely |
| 6 | `crates/prism-query/src/materialization.rs` | 784–790 | `Some(SensorType::CrowdStrike)` per table prefix | Change field type to `SensorId`; replace with `Some(SensorId::from("crowdstrike"))` |
| 7 | `crates/prism-query/src/invalidation.rs` | 56–91, 302, 351, 356, 385 | `sensor_type: SensorType::CrowdStrike` struct literal fields | Change field type to `SensorId`; replace literals with `SensorId::from("crowdstrike")` etc. |

(traces to BC-2.01.013 postcondition — adapter implementations produced from TOML SensorSpec
declarations at runtime; all dispatch points must be open/string-driven, not closed-enum-driven)

**AC-6:** The compile-fail perimeter test in `tests/external/perimeter-violation/` that catches
reintroduction of `pub enum SensorType` compiles successfully AND the compile-fail assertion
fires (i.e., the test still works as a negative test after the rename). VP-PLUGIN-001 enforcement
is maintained.
(traces to VP-PLUGIN-001 — SensorId open-newtype replaces SensorType closed enum; zero non-test
references to SensorType post-PREREQ-A)

**AC-7:** No `CustomAdapter` trait calls remain in production code paths that depend on
closed-enum dispatch. The `CustomAdapter` trait itself still exists (full retirement is
PREREQ-E). Calls to the trait remain dispatchable through the new `SensorId`-keyed
`CustomAdapterRegistry` (verify: `CustomAdapterRegistry::get(sensor_id)` accepts a `&str`
or `SensorId` without requiring `SensorType`).
(traces to VP-PLUGIN-007 — zero hardcoded CustomAdapter Rust adapters in production code paths
post-PREREQ-A; trait fully retired by PREREQ-E, not this story)

**AC-8:** The PR contains exactly ONE squash-merge commit. `cargo build --workspace --all-features`
PASSES at the squash-merged commit. `cargo test` workspace passes. The squash-merge IS the
atomic unit; intermediate Red Gate commit on the feature branch is permitted to reference
yet-to-be-defined symbols (e.g., `SensorId` construction in Red Gate test files before `SensorId`
implementation exists). On the `develop` branch post-merge, this story manifests as a single
atomic commit.
(Adversary pass-1 F-LP1-LOW-001 wording clarification: the original AC-8 text said "no
intermediate broken state in commit history" which was literally violated on the feature branch;
the squash-merge model is the operative semantics.)
(traces to BC-2.01.013 invariant — each DataSource produces records of a single type; the
migration commit must be atomic to prevent type-system-broken intermediate states)

**AC-9:** At least 2 unit tests in `prism-core` cover:
- (a) `SensorId` equality, hash, Display roundtrip: `SensorId::from("crowdstrike")` displays
  as `"crowdstrike"`, equals another `SensorId::from("crowdstrike")`, and is found in a
  `HashSet<SensorId>`.
- (b) `Borrow<str>` lookup behavior: `SensorId::from("armis")` inserted into
  `HashMap<SensorId, u32>` is retrievable via `map.get("armis" as &str)`.

(traces to BC-2.01.013 postcondition — sensor identity is a runtime string value; Borrow<str>
is required for registry lookup without cloning)

**AC-10:** At least 1 integration test in `prism-sensors/` exercises `AdapterRegistry`
insert + lookup with the new `SensorId` key type: register a mock adapter under
`SensorId::from("crowdstrike")`, look it up via `registry.get(org_id, SensorId::from("crowdstrike"))`,
assert `Some`. Verify cross-sensor isolation: lookup for `"cyberint"` returns `None`.
(traces to BC-2.01.013 invariant — each DataSource produces records of a single type;
registry must not return wrong-sensor adapter)

**AC-11 (BC-2.16.004 awareness):** Story body explicitly notes — as this AC does — that the
deprecated `CustomAdapter` Rust trait is NOT removed in this story. PREREQ-E owns trait
retirement. The `CustomAdapterRegistry` in `prism-spec-engine/src/custom_adapter.rs` uses
`sensor_id: &str` for its internal key (no `SensorType` dependency); if it uses `SensorType`
as a key, migrate to `SensorId` or `String` in this story's atomic commit.
(traces to BC-2.16.004 Deprecation Notice — trait retirement effective with PREREQ-F + PREREQ-A
lands, but the trait body itself is retired by PREREQ-E)

---

## Match-Site Dispatch Inventory (AC-5 Source of Truth)

Verified at story-write time via:
```
grep -rn "SensorType::" crates/ | grep -v target/ | grep -v "/tests/" | grep "src/"
```

**7 logical dispatch groups in production `src/` code across 4 crates (23 total SensorType
references in production source, grouped by dispatch location):**

1. `prism-core/src/types.rs:79–88` — Display match (4 arms). Eliminated with enum deletion.
2. `prism-query/src/virtual_fields.rs:163–166` — sensor_type → string match (4 arms). Replaced by `SensorId::as_ref()`.
3. `prism-query/src/explain.rs:661–664` — str → SensorType parse match (4 arms). Replaced by `SensorId::from(s)`.
4. `prism-query/src/explain.rs:1046–1050` — SensorType → latency_ms match (4 arms). Replaced by `HashMap<SensorId, u64>` or `match sensor_id.as_ref()`.
5. `prism-query/src/write_dispatch.rs:279–283` — str → SensorType match (4 arms). Replaced by `SensorId::from(name)`.
6. `prism-query/src/materialization.rs:784–790` — table prefix → `Some(SensorType::X)` (4 arms). Field type changes to `SensorId`.
7. `prism-query/src/invalidation.rs:56–91` — `sensor_type: SensorType::CrowdStrike` struct literal fields (8 references). Field type changes to `SensorId`; literals become `SensorId::from("crowdstrike")` etc.

**Additional `sensor_type: SensorType::X` usage (not traditional match arms):**
- `prism-sensors/src/adapter.rs:303` — trait method return type (changed in AC-3)
- `prism-sensors/src/auth/{crowdstrike,cyberint,claroty,armis}.rs` — 4 impl return sites (changed in AC-3)
- `prism-sensors/src/registry.rs:40` — HashMap key type (changed in AC-4)
- `prism-sensors/src/fanout.rs:410` — `sensor_type: SensorType::CrowdStrike` struct literal (migrate to `SensorId::from("crowdstrike")`)
- `prism-dtu-*/src/generator.rs` — DTU generators use `sensor_type: SensorType::X` in test fixture structs (update to `SensorId::from(...)`)

---

## Red Gate Test Set (failing tests that must exist BEFORE implementation)

The test-writer MUST produce these failing tests before the implementer writes any production code:

1. **`test_sensor_id_exists`** (prism-core) — attempts to construct `SensorId::from("crowdstrike")`;
   fails RED because `SensorId` type does not exist yet.

2. **`test_sensor_id_hash_eq_display_roundtrip`** (prism-core) — constructs two `SensorId`s
   from the same string, asserts equality and same hash bucket; fails RED because type doesn't exist.

3. **`test_sensor_id_borrow_str_lookup`** (prism-core) — inserts `SensorId::from("armis")` in
   `HashMap<SensorId, u32>`, looks up via `map.get("armis" as &str)`; fails RED.

4. **`test_adapter_registry_sensorid_keyed`** (prism-sensors integration test) — registers a
   mock adapter under `SensorId::from("crowdstrike")`, asserts lookup succeeds; fails RED because
   `AdapterRegistry::register` still takes `SensorType`.

5. **`test_sensorttype_reintroduction_compile_fails`** (perimeter-violation compile-fail test) —
   asserts that `pub enum SensorType { CrowdStrike }` in a test crate causes a compile error;
   fails RED if the enum still exists in prism-core (it shouldn't fail yet, which is the
   Red Gate condition proving the test is correctly structured to block reintroduction).

6. **`test_prism_query_dispatch_uses_sensorid`** (prism-query unit test) — imports and calls a
   converted dispatch site function with a `SensorId`; fails RED because the function still
   accepts `SensorType`.

---

## Architecture Mapping

| Component | Module | Pure/Effectful | ADR Reference |
|-----------|--------|----------------|---------------|
| `SensorId` newtype definition | `prism-core/src/sensor_id.rs` | Pure | ADR-023 §C1 |
| Enum deletion | `prism-core/src/types.rs` | Pure (type definition) | ADR-023 §C1 |
| `SensorAdapter` trait method signature | `prism-sensors/src/adapter.rs` | Pure (trait def) | ADR-023 §C1 + BC-2.01.013 |
| 4 adapter `sensor_type()` impls | `prism-sensors/src/auth/*.rs` | Pure (return constant) | BC-2.01.013 postcondition |
| `AdapterRegistry` HashMap key change | `prism-sensors/src/registry.rs` | Pure (data structure) | ADR-023 §C1 |
| 7 dispatch site rewrites | `prism-query/src/{virtual_fields,explain,write_dispatch,materialization,invalidation}.rs` | Mixed | ADR-023 §C1 |
| Fanout struct field | `prism-sensors/src/fanout.rs` | Effectful (async HTTP) | BC-2.01.013 |
| DTU generator struct fields | `prism-dtu-*/src/generator.rs` | Pure (test data) | Consistency |
| Perimeter compile-fail test | `tests/external/perimeter-violation/` | Build-time enforcement | VP-PLUGIN-001 |

Architecture layer: `prism-core` is Layer 0 (foundational types). `prism-sensors` is Layer 1.
`prism-query` is Layer 2. No Layer 2+ crate may define sensor identity; `SensorId` must remain
in Layer 0 (`prism-core`) to avoid circular dependencies (the existing `SensorType` was already
in `prism-core` for this reason).

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `prism-core/src/sensor_id.rs` | Pure | No I/O; data type definition and trait impls only |
| `prism-sensors/src/registry.rs` | Pure | Read-only after initialization; `HashMap` operations are pure |
| Dispatch site rewrites in `prism-query/src/` | Pure (rewrite is pure; callers remain mixed) | Each dispatch site conversion is purely structural |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `SensorId` MUST live in `prism-core` (Layer 0), not in `prism-sensors` or higher | ADR-023 §C1 + existing layering | Circular dep check: `prism-sensors` depends on `prism-core`; if `SensorId` were in `prism-sensors`, `prism-core` could not use it |
| Closed `SensorType` enum MUST NOT be reintroduced in any crate | VP-PLUGIN-001 | Perimeter compile-fail test (AC-6); `grep -rn "SensorType" crates/` post-merge must yield zero production hits |
| All dispatch sites MUST use `HashMap<SensorId, _>` or `match id.as_ref() { ... }` — never enum variant match | ADR-023 §C1 | Code review; AC-5 enumeration verified by grep |
| Atomic commit: all 15+ file changes land in a SINGLE squash commit | ADR-023 §C1; AC-8 | CI history; no intermediate commits with broken `cargo build` |
| DTU generators are NOT removed in this story — their `sensor_type` struct fields migrate from `SensorType` to `SensorId` | ADR-023 §C1 scope | Code review; DTU retirement is Wave 1/A scope |
| `CustomAdapter` trait is NOT removed in this story | BC-2.16.004; PREREQ-E scope | Code review; trait body remains unchanged |

**Forbidden Dependencies:** After this story merges, `prism-core` MUST NOT re-export `SensorType`.
If any crate (including test crates) imports `prism_core::types::SensorType` or
`prism_core::SensorType`, the workspace build MUST fail (no such item). The perimeter test
enforces this structurally.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `std::sync::Arc` | stdlib | Arc<str> payload for SensorId — cheap clone, immutable, thread-safe |
| `std::borrow::Borrow` | stdlib | Borrow<str> impl enables HashMap<SensorId, _>::get("string") lookup |
| `prism-core` | workspace | Source of SensorId (new), OrgId |
| `prism-sensors` | workspace | AdapterRegistry, SensorAdapter trait |
| `prism-query` | workspace | Dispatch sites: virtual_fields, explain, write_dispatch, materialization, invalidation |
| Rust stable | per rust-toolchain.toml (1.85+) | edition 2024 |

No new external crate dependencies are required. `Arc<str>` is stdlib. All impls use only
standard library traits.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-core/src/sensor_id.rs` | Create | New SensorId newtype with full impl set |
| `crates/prism-core/src/types.rs` | Modify | Delete `pub enum SensorType` + Display impl |
| `crates/prism-core/src/lib.rs` | Modify | Add `mod sensor_id; pub use sensor_id::SensorId;`; remove `SensorType` re-export |
| `crates/prism-core/src/error.rs` | Modify | Update `UnknownSensorType` variant to `UnknownSensorId` |
| `crates/prism-sensors/src/adapter.rs` | Modify | Change `fn sensor_type(&self) -> SensorType` → `-> SensorId`; update AdapterNotFound variant |
| `crates/prism-sensors/src/registry.rs` | Modify | HashMap key type `(OrgId, SensorType)` → `(OrgId, SensorId)`; update all method signatures |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | Modify | `sensor_type()` impl returns `SensorId::from("crowdstrike")` |
| `crates/prism-sensors/src/auth/cyberint.rs` | Modify | `sensor_type()` impl returns `SensorId::from("cyberint")` |
| `crates/prism-sensors/src/auth/claroty.rs` | Modify | `sensor_type()` impl returns `SensorId::from("claroty")` |
| `crates/prism-sensors/src/auth/armis.rs` | Modify | `sensor_type()` impl returns `SensorId::from("armis")` |
| `crates/prism-sensors/src/fanout.rs` | Modify | `sensor_type: SensorType::CrowdStrike` → `SensorId::from("crowdstrike")` |
| `crates/prism-query/src/virtual_fields.rs` | Modify | Dispatch site 2: `sensor_id.as_ref()` replaces match |
| `crates/prism-query/src/explain.rs` | Modify | Dispatch sites 3 + 4: parse match → `SensorId::from(s)`; latency match → HashMap or str match |
| `crates/prism-query/src/write_dispatch.rs` | Modify | Dispatch site 5: str → `SensorType` match → `SensorId::from(name)` |
| `crates/prism-query/src/materialization.rs` | Modify | Dispatch site 6: `Some(SensorType::X)` → `Some(SensorId::from("x"))`; field type change |
| `crates/prism-query/src/invalidation.rs` | Modify | Dispatch site 7: struct literals `sensor_type: SensorType::X` → `sensor_id: SensorId::from("x")` |
| `crates/prism-dtu-crowdstrike/src/generator.rs` | Modify | Struct field `sensor_type: SensorType::CrowdStrike` → `SensorId::from("crowdstrike")` |
| `crates/prism-dtu-cyberint/src/generator.rs` | Modify | Struct field → `SensorId::from("cyberint")` |
| `crates/prism-dtu-claroty/src/generator.rs` | Modify | Struct field → `SensorId::from("claroty")` |
| `crates/prism-dtu-armis/src/generator.rs` | Modify | Struct field → `SensorId::from("armis")` |
| `crates/prism-dtu-common/src/generator/pagination.rs` | Modify | `match sensor_type { SensorType::X => N }` → `match sensor_id.as_ref() { "x" => N, _ => 100 }` |
| `crates/prism-sensors/tests/org_id_binding.rs` | Modify | `SensorType::CrowdStrike` → `SensorId::from("crowdstrike")`; update test struct fields |
| `crates/prism-sensors/tests/cr013_fan_out_org_id_consistency.rs` | Modify | Same migration |
| `crates/prism-query/tests/execute_integration_tests.rs` | Modify | `fn sensor_type(&self) -> SensorType` → `SensorId`; `SensorType::X` literals throughout |
| Various other `crates/prism-sensors/src/tests/*.rs` | Modify | Update `fn sensor_type()` mock impls in bc_2_01_013.rs, bc_2_01_010.rs, bc_2_01_002.rs |
| `tests/external/perimeter-violation/` | Modify | Update/add compile-fail assertion that `SensorType` enum reintroduction fails |

Implementer note: run `grep -rn "SensorType" crates/` before committing and ensure zero
production-code hits. Test files referencing `SensorType` in mock impls are expected to
appear zero times after migration (they use `SensorId` too).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two SensorIds constructed from identical strings | `PartialEq` returns `true`; same `Hash` bucket; `HashMap` deduplicates |
| EC-002 | SensorId constructed from `Arc<str>` vs `&str` vs `String` | All produce the same logical id; `PartialEq` and `Hash` are content-based, not pointer-based |
| EC-003 | `AdapterRegistry::get()` called with `SensorId::from("unknown")` | Returns `None`; no panic; caller handles absent adapter |
| EC-004 | `get_all_for_sensor()` called on empty registry | Returns `Vec::new()` |
| EC-005 | Sensor TOML spec declares `sensor_id = "my-custom-sensor"` post-Wave-1 | `SensorId::from("my-custom-sensor")` is a valid key; registry stores it like any other |
| EC-006 | Intermediate build state (SensorType deleted, not all call sites updated) | `cargo build --workspace` fails with clear type errors. Atomic commit is mandatory — do not push partial states |
| EC-007 | DTU generators use `sensor_type` struct field name | Field name may remain `sensor_type: SensorId` in DTU structs (field-name migration is separate from type migration); or rename to `sensor_id` for clarity — implementer chooses but must be consistent |

---

## Previous Story Intelligence

This is the first code-changing story of the PLUGIN-MIGRATION wave. It depends on:

**S-PLUGIN-PREREQ-F (shipped at a952ffff):** Delivered the BC + DI documentation foundation:
- BC-2.01.013 amended to v1.4 (spec-driven adapter pattern; sealed-trait language removed)
- BC-2.16.004 deprecated to v1.4 (CustomAdapter Rust escape hatch → WASM; not yet retired)
- DI-012 amended (compile-time SensorAuth sealing → runtime spec-load validation)
- VP-PLUGIN-001 through VP-PLUGIN-007 registered in VP-INDEX.md

Key invariant from PREREQ-F: `BC-2.16.004` is DEPRECATED (lifecycle_status: deprecated) but
NOT retired — the `CustomAdapter` trait body still exists. This story DOES NOT remove it.
PREREQ-E owns removal.

Key lesson from Wave 5 stories: atomic commits are critical when type signatures change across
multiple crates. S-WAVE5-PREP-01 and S-3.02-FOLLOWUP-RUNTIME both required careful staging
to avoid intermediate broken states. This story has a larger blast radius (~23 files) — the
implementer MUST stage all changes before the first `cargo build` validation pass.

---

## Implementation Notes

**Arc<str> rationale:** Use `Arc<str>` (not `String`) for the newtype payload. `Arc<str>` is
immutable, provides cheap `Clone` via reference counting, and enables future interning in Wave 1
(where many sensors may share string identity). `String` would work but costs a clone-per-clone.

**From<&'static str> optimization:** The four initial sensors use static string literals
(`"crowdstrike"`, `"cyberint"`, `"claroty"`, `"armis"`). `From<&'static str>` can use
`Arc::from("crowdstrike")` which Rust may optimize to a static Arc. This is a micro-optimization;
correctness does not depend on it.

**Do NOT add `as_str()`:** The combination of `Borrow<str>` + `AsRef<str>` + `Display` provides
all necessary string-view access patterns. Adding a separate `as_str()` method is redundant and
creates a fourth way to get the same value. `Display` is for formatting; `Borrow<str>` is for
HashMap lookups; `AsRef<str>` is for generic string functions.

**Atomic commit is mandatory:** The closed enum's deletion and all call-site replacements must
land in a single commit. There is no intermediate state where `SensorType` is deleted but
`fn sensor_type(&self) -> SensorType` still exists — the workspace will not compile. Stage all
23+ file changes before issuing any `git commit`. CI will reject any non-green intermediate state.

**Latency dispatch site (explain.rs:1046):** The four sensor-specific latency estimates (250ms
CrowdStrike, 400ms Cyberint, 350ms Claroty, 300ms Armis) should migrate to either:
- A `match sensor_id.as_ref() { "crowdstrike" => 250, "cyberint" => 400, ... _ => 300 }` —
  acceptable for this story because it is feature-equivalent and not a closed-enum violation
  (str match is open — unknown sensors fall to the default case), OR
- A `HashMap<SensorId, u64>` populated from a const table.
The implementer should choose the simpler option. The `_ => 300` default is required.

**CustomAdapterRegistry key type:** `prism-spec-engine/src/custom_adapter.rs` uses `sensor_id: &str`
as its internal key (verified by reading the file). If it happens to reference `SensorType`,
migrate it in this story. If it uses `String` or `&str` already, no change needed there.

---

## Green Gate Definition of Done

The story is shipped when ALL of the following are true:
1. `cargo build --workspace --all-features` is clean (zero errors, zero warnings with `-D warnings`)
2. `just check` passes (fmt + clippy + nextest + doctests + crate-layout)
3. `grep -rn "SensorType" crates/` returns ZERO hits in non-test production code
4. All 11 ACs are verifiable with explicit grep/test evidence recorded in the PR description
5. VP-PLUGIN-001 perimeter test passes (compile-fail assertion fires on SensorType reintroduction)
6. VP-PLUGIN-007 is noted in PR as "partially verified" — CustomAdapter trait exists but no
   production code calls it via a SensorType key (SensorId-keyed lookup is the only path)
7. PR is squash-merged into `develop` as exactly ONE commit
8. STORY-INDEX row transitions to `status: merged` with PR# recorded in the Full Story List

---

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.1 | fix-burst-1-closure | 2026-05-11 | state-manager | AC-4 wording updated: adopted implementation where adapter owns identity (register() derives SensorId from adapter.sensor_type() internally); rationale recorded per F-LP1-HIGH-003 orchestrator decision. AC-8 wording clarified: squash-merge is the operative atomic unit; intermediate Red Gate commit on feature branch is permitted per F-LP1-LOW-001. Both changes record adversary pass-1 closure disposition. |
| 1.0 | prereq-a-materialization | 2026-05-10 | story-writer | Initial story creation from ADR-023 §C1 + grep-verified dispatch site inventory. All 11 ACs traced to BC-2.01.013 / VP-PLUGIN-001 / VP-PLUGIN-007. 7 dispatch groups enumerated with exact file:line from workspace grep. Red Gate set (6 failing tests) specified. Atomic commit requirement documented. |
