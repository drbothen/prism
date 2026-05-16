---
document_type: story
story_id: S-PLUGIN-PREREQ-E
title: "prism-sensors/prism-spec-engine: Un-seal SensorAuth + Remove CustomAdapter Rust Trait + WriteToolInvalidationMap Runtime Extensibility"
wave: 0
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
depends_on:
  - S-PLUGIN-PREREQ-F
  - S-PLUGIN-PREREQ-A
  - S-PLUGIN-PREREQ-D
blocks:
  - PLUGIN-MIGRATION-001-A
  - PLUGIN-MIGRATION-001-C
  - PLUGIN-MIGRATION-001-D
  - PLUGIN-MIGRATION-001-E
points: 3
estimated_days: 1
risk: MEDIUM
tdd_mode: strict
crates_touched: [prism-sensors, prism-spec-engine, prism-query]
target_module: prism-sensors
subsystems: [SS-01, SS-16]
capabilities: [CAP-001, CAP-029]
version: "1.2"
level: "L4"
producer: product-owner
timestamp: "2026-05-15T00:00:00Z"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
behavioral_contracts:
  - BC-2.01.016  # SensorAuth open trait contract (NEW — this story)
  - BC-2.16.011  # CustomAdapter Rust trait retirement (NEW — this story)
  - BC-2.16.012  # PluginRegistry dispatch migration in spec_parser.rs (NEW — this story)
  - BC-2.01.013  # DataSource trait adapter pattern (amended in PREREQ-F; this story implements the un-sealing side)
  - BC-2.16.004  # CustomAdapter escape hatch (DEPRECATED in PREREQ-F; this story retires/removes it)
verification_properties:
  - VP-153  # SensorAuth Runtime Cross-Composition Prevention (proptest) — anchors BC-2.01.016 Rule 2 rejection
  - VP-154  # CustomAdapter Behavioral Equivalence integration test — P1, authored PLUGIN-MIGRATION-001-A scope
  - VP-155  # CustomAdapter Absent from prism-spec-engine Public API (compile-fail) — P0, authored PLUGIN-MIGRATION-001-A scope
  - VP-PLUGIN-001  # No production hardcoded sensor references — perimeter check must remain green
  - VP-PLUGIN-007  # Plugin manifest allowlist enforcement — must remain unaffected
architectural_decisions:
  - ADR-026  # SensorAuth unsealing decision — defines runtime enforcement rules (E-SPEC-012/013/014) and VP-153
  - ADR-027  # CustomAdapter deprecation/removal — defines compile-fail perimeter (VP-155) and WASM equivalence (VP-154)
  - ADR-023  # Plugin-only sensor architecture — §Architectural Constraints (C5 bullet) rules authoritative for this story's scope
holdout_scenarios:
  - HS-PREREQ-E-001  # SensorAuth Open Trait — External Implementation Compiles and Loads (+ VP-153 cross-composition)
  - HS-PREREQ-E-002  # CustomAdapter Retirement — No Behavioral Regression (+ VP-154/VP-155 coverage)
  - HS-PREREQ-E-003  # PluginRegistry Dispatch — Behavioral Equivalence + WriteToolInvalidationMap extensibility
anchor_bcs: [BC-2.01.016, BC-2.16.011, BC-2.16.012, BC-2.01.013, BC-2.16.004]
anchor_vps: [VP-153, VP-154, VP-155, VP-PLUGIN-001, VP-PLUGIN-007]
anchor_capabilities: [CAP-001, CAP-029]
anchor_subsystem: [SS-01, SS-16]
assumption_validations:
  - "prism-spec-engine has never been published to crates.io with CustomAdapter exposed (PLUGIN-AUDIT-001 HIGH-3 confirmed — no deprecation window required)"
  - "spec_parser.rs contains zero CustomAdapter/CustomAdapterRegistry references (ADR-023 §Architectural Constraints (C5 bullet) F-CRIT-NEW-001-PASS2-RESIDUAL verified by grep)"
  - "S-WAVE5-PREP-01 already removed custom_adapter_registry references from boot.rs — no boot.rs changes required in PREREQ-E"
  - "TD-S-PLUGIN-PREREQ-A-003 (WriteToolInvalidationMap extensibility) is routed to PREREQ-E per S-PLUGIN-PREREQ-A fix-burst-2 decision"
risk_mitigations:
  - "AC-1..3: SensorAuth unsealing is pure deletion — no new code paths; four built-in auth impls are unchanged"
  - "AC-4..6: CustomAdapter deletion confirmed safe by PLUGIN-AUDIT-001: three call sites are isolated to lib.rs re-export + one example + one test file"
  - "AC-7..8: spec_parser.rs migration verified by behavioral-equivalence integration test (TV-BC-2.16.012-003)"
  - "AC-9: TD-S-PLUGIN-PREREQ-A-003 closure via RwLock<Vec<WriteToolInvalidationMap>> — write lock held only during registration; read-side is zero-copy"
acceptance_criteria_count: 10
red_gate_tests: 8
estimated_passes: "4-6 LOCAL adversary passes"
td_resolves:
  - TD-S-PLUGIN-PREREQ-A-003  # WriteToolInvalidationMap runtime extensibility (PREREQ-E scope)
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md"
  - ".factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md"
  - ".factory/tech-debt-register.md"
  - ".factory/cycles/wave-4-operations/forward-task-map.md"
---

# S-PLUGIN-PREREQ-E: prism-sensors/prism-spec-engine — Un-seal SensorAuth + Remove CustomAdapter + WriteToolInvalidationMap Runtime Extensibility

## Narrative

As the Prism platform, I want the `SensorAuth` sealed trait opened for plugin implementation,
the `CustomAdapter` Rust trait and all its scaffolding permanently removed, and
`WriteToolInvalidationMap` made runtime-extensible, so that `.prx` WASM plugins are the sole
non-declarative escape hatch and plugin-registered write tools participate correctly in cache
invalidation — completing the Wave 0 plugin-only sensor architecture foundation.

---

## Behavioral Contracts

| BC ID | Title | Subsystem | Role in This Story |
|-------|-------|-----------|-------------------|
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | SS-01 | Primary delivery — sealed marker removed; four built-in auth impls unchanged; runtime Rule 2 enforcement confirmed |
| BC-2.16.011 | CustomAdapter Rust Trait Retirement — Removal of Trait, Registry, and All Call Sites | SS-16 | Primary delivery — `custom_adapter.rs` deleted; three call sites cleaned; BC-2.16.004 transitions to removed |
| BC-2.16.012 | PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup | SS-16 | Primary delivery — open dispatch migration; behavioral equivalence test; TD-S-PLUGIN-PREREQ-A-003 WriteToolInvalidationMap closure |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | Awareness — PREREQ-F established that SensorAuth is NOT sealed; this story is the mechanical implementation of that amendment |
| BC-2.16.004 | Rust Escape Hatch for Custom Adapters (DEPRECATED → REMOVED) | SS-16 | Lifecycle close — `lifecycle_status: deprecated → removed`; this story is the execution of the retirement planned in PREREQ-F |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,000 |
| BC-2.01.016 (SensorAuth open trait) | ~2,500 |
| BC-2.16.011 (CustomAdapter retirement) | ~2,500 |
| BC-2.16.012 (PluginRegistry migration) | ~2,500 |
| `crates/prism-sensors/src/auth/mod.rs` (sealed marker removal) | ~400 |
| `crates/prism-spec-engine/src/custom_adapter.rs` (deletion) | ~0 (deleted) |
| `crates/prism-spec-engine/src/lib.rs` (re-export removal) | ~300 |
| `crates/prism-spec-engine/examples/demo_spec_loading.rs` (cleanup/delete) | ~200 |
| `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` (deletion) | ~0 (deleted) |
| `crates/prism-spec-engine/src/spec_parser.rs` (open dispatch migration) | ~800 |
| `crates/prism-query/src/invalidation.rs` (WriteToolInvalidationMap RwLock migration) | ~600 |
| `BC-2.16.004-rust-escape-hatch.md` (frontmatter: deprecated → removed) | ~200 |
| `error-taxonomy.md` (E-SPEC-008 retired annotation) | ~100 |
| Test files (Red Gate set + behavioral equivalence) | ~2,000 |
| Total | ~17,100 |

Well within the 30% context window budget (~40k tokens).

---

## Tasks

1. **Remove `private::Sealed` marker from `crates/prism-sensors/src/auth/mod.rs`**
   - Delete the `mod private { pub trait Sealed {} }` block (or equivalent sealed-marker pattern)
   - Remove `private::Sealed` from the `SensorAuth` trait's supertrait bounds (`trait SensorAuth: Sealed` → `trait SensorAuth`)
   - Verify that the four concrete auth impls (`CrowdStrikeAuth`, `CyberintAuth`, `ClarotyAuth`, `ArmisAuth`) compile without modification after removing the supertrait

2. **Delete `crates/prism-spec-engine/src/custom_adapter.rs`**
   - The file contains: `trait CustomAdapter`, `struct CustomAdapterRegistry`, `struct CustomAuth` (sealed-trait workaround), all impls and registrations
   - Remove all content and delete the file

3. **Clean `crates/prism-spec-engine/src/lib.rs` re-exports**
   - Remove `mod custom_adapter;` declaration
   - Remove `pub use custom_adapter::*;` or individual item re-exports (`CustomAdapter`, `CustomAdapterRegistry`, `CustomAuth`)
   - Confirm no other `custom_adapter`-namespaced references remain in `lib.rs`

4. **Clean or delete `crates/prism-spec-engine/examples/demo_spec_loading.rs`**
   - If the file ONLY exercises `CustomAdapter`, delete it entirely
   - If it has valuable non-`CustomAdapter` spec-loading demo code, remove only the `CustomAdapter`-specific sections; preserve the rest

5. **Delete `crates/prism-spec-engine/tests/bc_2_16_004_test.rs`**
   - This file tests `CustomAdapterRegistry` behavior that no longer exists. Delete the file. Its test coverage is superseded by PLUGIN-MIGRATION-001-C WASM plugin tests.

6. **Migrate spec_parser.rs call sites to open dispatch**
   - Audit `crates/prism-spec-engine/src/spec_parser.rs` for any `match sensor_id.as_ref() { "crowdstrike" => ..., "cyberint" => ..., ... }` dispatch arms that encode sensor-specific parsing logic
   - Replace each such arm with a PluginRegistry lookup call or a generic parsing path that applies to all sensor strings
   - Behavioral output for the four initial sensors must remain identical (verified by behavioral-equivalence test in the Red Gate set)

7. **Migrate `WriteToolInvalidationMap` to runtime-extensible container (TD-S-PLUGIN-PREREQ-A-003)**
   - In `crates/prism-query/src/invalidation.rs`, change the `LazyLock<Vec<WriteToolInvalidationMap>>` container to `std::sync::RwLock<Vec<WriteToolInvalidationMap>>` (or `OnceLock<RwLock<Vec<...>>>` if lazy initialization is needed)
   - Add a `pub fn register_write_tool(entry: WriteToolInvalidationMap)` API that acquires a write guard and pushes the entry
   - Update all read-side callers (the invalidation check function) to acquire a read guard instead of dereferencing the `LazyLock`
   - Wire `PluginRuntime` (already available via PREREQ-D boot wiring) to call `register_write_tool` for each plugin that declares write-tool capabilities in its manifest

8. **Update BC-2.16.004 frontmatter to `removed`**
   - Open `BC-2.16.004-rust-escape-hatch.md` and update:
     - `lifecycle_status: deprecated` → `lifecycle_status: removed`
     - Add `removed: "2026-05-15"`
     - Add `removal_reason: "PREREQ-E retirement per ADR-023 Rule 5"`
   - Do NOT delete the file — it remains as a historical record per DF-030 protocol

9. **Update E-SPEC-008 in error-taxonomy.md**
   - Annotate the E-SPEC-008 row with a `retired:` note: "Retired in S-PLUGIN-PREREQ-E. No live code path triggers this code post-CustomAdapter removal. Plugin execution panics surface via E-PLUGIN-001."
   - Do NOT delete the E-SPEC-008 row (IDs are append-only per append_only_numbering policy)

10. **Run `just check`** — must pass with zero errors, zero warnings
    - Additionally run `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` — must return zero matches in `src/` paths
    - Run `grep -rn "private::Sealed\|impl Sealed for\|: Sealed" crates/prism-sensors/src/auth/` — must return zero matches

---

## Acceptance Criteria

**AC-1 (SensorAuth Sealed Marker Removed):**
`grep -rn "private::Sealed\|: Sealed\|impl Sealed for" crates/prism-sensors/src/auth/` returns ZERO matches after merge.
The `SensorAuth` trait definition in `crates/prism-sensors/src/auth/mod.rs` has no supertrait bound referencing a `Sealed` marker. The trait is `pub` and externally implementable.
(traces to BC-2.01.016 postcondition — sealed marker removed; trait is publicly implementable)

**AC-2 (Four Built-In Auth Impls Unchanged):**
The four concrete auth implementations (`CrowdStrikeAuth`, `CyberintAuth`, `ClarotyAuth`, `ArmisAuth`) require ZERO changes to their `impl SensorAuth for X` blocks. If any impl references the `Sealed` marker via `impl private::Sealed for X`, that one line is removed; all other impl content is preserved verbatim.
`cargo build -p prism-sensors` after the change exits 0 with zero warnings.
(traces to BC-2.01.016 postcondition — four built-in auth impls unchanged; INV-AUTH-OPEN-002)

**AC-3 (Runtime Auth-Composition Rejection Active):**
A unit test confirms that a `SensorSpec` with `auth_type = ["oauth2_client_credentials", "bearer_static"]` is rejected at spec-load with `E-SPEC-012`. This verifies that the sealed-trait removal does NOT weaken the threat model — rejection moves from compile time to runtime. Note: E-SPEC-012 (not E-SPEC-010; E-SPEC-010 is reserved for variable interpolation field-path misses per error-taxonomy v1.25).
(traces to BC-2.01.016 invariant INV-AUTH-OPEN-003; ADR-023 Rule 2, Rule A; error-taxonomy v1.25)

**AC-4 (custom_adapter.rs Deleted):**
`crates/prism-spec-engine/src/custom_adapter.rs` does not exist after merge. `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/prism-spec-engine/src/` returns ZERO matches.
(traces to BC-2.16.011 postcondition — deletion)

**AC-5 (Three Call Sites Cleaned):**
The three confirmed call sites are cleaned:
- `crates/prism-spec-engine/src/lib.rs`: no `mod custom_adapter;` and no `pub use custom_adapter::*` or individual item re-exports for `CustomAdapter`, `CustomAdapterRegistry`, `CustomAuth`
- `crates/prism-spec-engine/examples/demo_spec_loading.rs`: either deleted or all `CustomAdapter`/`CustomAdapterRegistry`-using code removed
- `crates/prism-spec-engine/tests/bc_2_16_004_test.rs`: deleted (does not exist after merge)
(traces to BC-2.16.011 postconditions; ADR-023 §Architectural Constraints (C5 bullet) confirmed three sites)

**AC-6 (BC-2.16.004 Lifecycle Updated to Removed):**
`BC-2.16.004-rust-escape-hatch.md` frontmatter contains:
- `lifecycle_status: removed`
- `removed: "2026-05-15"` (or the actual merge date)
- `removal_reason: "PREREQ-E retirement per ADR-023 Rule 5"`
The file is NOT deleted (historical record preservation per DF-030 append_only_numbering).
(traces to BC-2.16.011 postcondition; DF-030 BC deprecation protocol)

**AC-7 (spec_parser.rs Open Dispatch — No Hardcoded Sensor Name Match Arms in Dispatch Context):**
`grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` returns ZERO matches in production dispatch match-arm contexts. Sensor name strings may still appear in doc comments or test fixture values (those are acceptable).
(traces to BC-2.16.012 postcondition — open dispatch; INV-SPEC-PARSER-OPEN-001)

**AC-8 (Behavioral Equivalence for Four Initial Sensors):**
An integration test (`test_BC_2_16_012_spec_parser_behavioral_equivalence`) parses each of the four built-in TOML sensor specs via the migrated `SpecParser` and asserts that the resulting `SensorSpec` struct is identical to the pre-migration baseline. A novel sensor name (`"hypothetical_sensor"`) parses without error via the generic path, producing a valid `SensorSpec`.
(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-002 + INV-SPEC-PARSER-OPEN-003)

**AC-9 (WriteToolInvalidationMap Runtime Extensibility — TD-S-PLUGIN-PREREQ-A-003 Closed):**
`crates/prism-query/src/invalidation.rs` `WriteToolInvalidationMap` container is `RwLock<Vec<WriteToolInvalidationMap>>` (or equivalent). A `pub fn register_write_tool(entry: WriteToolInvalidationMap)` API exists and is callable after startup. A unit test (`test_BC_2_16_012_write_tool_invalidation_runtime_register`) registers a custom write tool entry and asserts it is present in the map on the next read-guard acquisition.
(traces to BC-2.16.012 postcondition — TD-S-PLUGIN-PREREQ-A-003 WriteToolInvalidationMap; INV-INVALIDATION-EXT-001)

**AC-10 (Full Build and Pre-Push Gate):**
`cargo build --workspace --all-features` exits 0. `just check` (fmt + clippy + nextest + doctests + crate-layout) exits 0 with zero warnings. The PR contains exactly ONE squash-merge commit on `develop`.
(production-grade default — CLAUDE.md Canonical Principle Rule 1)

---

## Red Gate Test Set (failing tests that must exist BEFORE implementation)

The test-writer MUST produce these failing tests before the implementer writes any production code.

1. **`test_BC_2_01_016_001_sensor_auth_external_impl_compiles`** (prism-sensors) — attempts to call a function that accepts `Box<dyn SensorAuth>` with a type defined outside `prism-sensors`; fails RED because `SensorAuth` is sealed and the external impl cannot satisfy the sealed bound.

2. **`test_BC_2_01_016_002_auth_composition_runtime_rejection`** (prism-spec-engine) — constructs a `SensorSpec` with `auth_type = ["oauth2_client_credentials", "bearer_static"]` and attempts to load it via `SensorSpec::load`. Asserts `result.is_err() && err.code() == "E-SPEC-012"`. Pre-implementation: assertion fails because no Rule 2 enforcement exists (load succeeds, `is_err()` is false). Post-implementation: assertion passes (validator returns E-SPEC-012 error per BC-2.01.016 Rule 2 / ADR-023 Rule 2, Rule A).

3. **`test_BC_2_16_011_001_custom_adapter_absent_post_deletion`** (prism-spec-engine) — attempts to import `prism_spec_engine::CustomAdapter`; fails RED at compile time because the type exists pre-migration. This is a compile-fail test in the style of `tests/external/perimeter-violation/`.

4. **`test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch`** (prism-spec-engine) — calls the `SpecParser` with a novel `SensorSpec` TOML for `"hypothetical_sensor"` and asserts it parses without error; fails RED if `spec_parser.rs` still has a hardcoded match arm that rejects unknown sensors.

5. **`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike`** (prism-spec-engine) — parses `crowdstrike.sensor.toml` and compares against a snapshot; fails RED initially because the snapshot must be captured post-migration (the test infrastructure is set up in the Red Gate phase; snapshot is populated during implementation).

6. **`test_BC_2_16_012_003_write_tool_invalidation_runtime_register`** (prism-query) — calls `register_write_tool(entry)` where `entry` is a new `WriteToolInvalidationMap` struct; fails RED because `register_write_tool` does not exist yet (the container is a `LazyLock<Vec<...>>` not an `RwLock`).

7. **`test_BC_2_01_016_003_four_auth_impls_unchanged_post_unsealing`** (prism-sensors) — constructs all four concrete auth types and calls their `SensorAuth` methods; fails RED until `SensorAuth` is unsealed (because the sealed bound is currently blocking external test construction).

8. **`test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code`** (prism-spec-engine) — searches the workspace `src/` tree for any match arm or handler that constructs `E-SPEC-008`; fails RED if any live code path still produces that error code (all live paths must be absent post-deletion; the error taxonomy entry remains but is retired).

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `SensorAuth` trait (sealed marker removal) | `crates/prism-sensors/src/auth/mod.rs` | Pure (type definition change) |
| `custom_adapter.rs` deletion | `crates/prism-spec-engine/src/custom_adapter.rs` | Pure (file deleted) |
| `lib.rs` re-export removal | `crates/prism-spec-engine/src/lib.rs` | Pure (API surface change) |
| `spec_parser.rs` dispatch migration | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (parsing logic; no I/O) |
| `WriteToolInvalidationMap` extensibility | `crates/prism-query/src/invalidation.rs` | Mixed (RwLock for write registration; read side on query hot path) |
| BC-2.16.004 frontmatter update | `.factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md` | Spec artifact (state-manager) |

Architecture layer: `prism-sensors` is Layer 1 (auth surface); `prism-spec-engine` is Layer 1 (spec parsing); `prism-query` is Layer 2. The Layer 2 `WriteToolInvalidationMap` extensibility depends on the `PluginRuntime` wired in Layer 1 (via PREREQ-D).

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `prism-sensors/src/auth/mod.rs` (sealed marker removal) | Pure | Trait definition change only; no runtime behavior added |
| `prism-spec-engine/src/spec_parser.rs` (open dispatch) | Pure | Parser is stateless; registry lookup is a read-only operation |
| `prism-query/src/invalidation.rs` (RwLock migration) | Mixed-at-boundary | `RwLock::write()` during registration (effectful); `RwLock::read()` during query-time invalidation check (read-only on the hot path) |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `SensorAuth` MUST NOT have a `Sealed` supertrait after PREREQ-E | ADR-023 §Architectural Constraints (C5 bullet, Rule 2); BC-2.01.016 | `grep -rn "private::Sealed\|: Sealed\|impl Sealed" crates/prism-sensors/src/auth/` = 0 hits |
| `CustomAdapter`, `CustomAdapterRegistry`, `CustomAuth` MUST NOT exist in `src/` after PREREQ-E | ADR-023 §Architectural Constraints (C5 bullet, Rule 5); BC-2.16.011 | `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` = 0 hits in `src/` |
| Hardcoded sensor name dispatch in `spec_parser.rs` MUST be replaced with open path | ADR-023 §Architectural Constraints (C5 bullet); BC-2.16.012 | `grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` = 0 hits in dispatch contexts |
| `boot.rs` MUST NOT be modified in this story | ADR-023 §Architectural Constraints (C5 bullet) scope + F-MED-NEW-005 | Code review; `git diff develop...HEAD -- crates/prism-bin/src/boot.rs` = empty |
| VP-PLUGIN-001 perimeter test MUST remain green after sealed-trait removal | VP-PLUGIN-001 (FORBIDDEN-SYMBOLS-001) | CI grep gate; perimeter compile-fail test must still pass (no new forbidden symbols introduced) |
| Atomic commit: all file changes land in ONE squash commit | CLAUDE.md commit conventions; AC-10 | CI; `git log --oneline develop..HEAD` = 1 commit on `develop` |

---

## Error Taxonomy Additions

Three NEW error codes are introduced in this story (see `error-taxonomy.md` v1.25 §SPEC); one existing code is annotated as retired:

| Code | Action | Purpose |
|------|--------|---------|
| `E-SPEC-012` | NEW | ADR-023 Rule 2, Rule A — auth_type must be a single value from the canonical enumerated set; arrays or out-of-set values rejected at spec-load. Credential values must not appear in error message (AD-017). |
| `E-SPEC-013` | NEW | ADR-023 Rule 2, Rule B — exactly one credential_ref per auth method; multiple bindings rejected at spec-load. |
| `E-SPEC-014` | NEW | ADR-023 Rule 2, Rule C — credential structural type must match declared auth_type; mismatches rejected at credential-resolution time, before any HTTP request. Credential values must not appear in error message (AD-017). |
| `E-SPEC-008` | RETIRED (annotate only, do NOT delete) — error-taxonomy.md v1.26 | Annotate with `retired:` note: "Retired in S-PLUGIN-PREREQ-E. No live code path triggers this code post-CustomAdapter removal. Plugin execution panics surface via E-PLUGIN-001." |

Note: E-SPEC-010 (variable interpolation field-path miss) and E-SPEC-011 (pipe_verb reserved keyword) are pre-existing codes that are NOT related to auth-composition rejection. The erroneous reference to E-SPEC-010/011/012 in prior PREREQ-E authoring has been corrected to E-SPEC-012/013/014.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/src/auth/mod.rs` | Modify | Remove `private::Sealed` module, remove `: Sealed` supertrait from `SensorAuth` trait |
| `crates/prism-spec-engine/src/custom_adapter.rs` | DELETE | Primary retirement target per ADR-023 Rule 5 |
| `crates/prism-spec-engine/src/lib.rs` | Modify | Remove `mod custom_adapter;` + all `CustomAdapter`/`CustomAdapterRegistry`/`CustomAuth` re-exports |
| `crates/prism-spec-engine/examples/demo_spec_loading.rs` | DELETE or Modify | Remove `CustomAdapter`-using sections; delete file if nothing meaningful remains |
| `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` | DELETE | BC-2.16.004 is removed; this test file is deleted with it |
| `crates/prism-spec-engine/src/spec_parser.rs` | Modify | Replace hardcoded sensor-name match arms with PluginRegistry lookup or generic path |
| `crates/prism-query/src/invalidation.rs` | Modify | Migrate `WriteToolInvalidationMap` from `LazyLock<Vec<...>>` to `RwLock<Vec<...>>`; add `register_write_tool` API |
| `.factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md` | Modify | Update frontmatter: `lifecycle_status: deprecated → removed`; add `removed:` + `removal_reason:` |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Modify | Add `retired:` annotation to E-SPEC-008 row |

Implementer note: run `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` before committing. Expected: zero `src/` matches. Run `grep -rn "private::Sealed\|: Sealed\|impl Sealed" crates/prism-sensors/src/auth/` — expected: zero matches.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-E-001 | `demo_spec_loading.rs` contains only `CustomAdapter` code | File is deleted entirely; no partial file left |
| EC-E-002 | `demo_spec_loading.rs` has spec-loading code unrelated to `CustomAdapter` | Non-`CustomAdapter` sections preserved; file is not deleted unless it becomes empty |
| EC-E-003 | `spec_parser.rs` has no hardcoded sensor dispatch arms (all already removed by PREREQ-A) | No changes to `spec_parser.rs` needed for dispatch migration (AC-7 passes trivially); Task 6 is a no-op |
| EC-E-004 | A caller of the invalidation check function reads `WriteToolInvalidationMap` concurrently with a `register_write_tool` call | `RwLock::read()` waits for any active `write()` to complete; readers see the new entry on the next lock acquisition; no data race |
| EC-E-005 | `register_write_tool` is called during active query fan-out | Safe if `RwLock::write()` is used; query fan-out holds `RwLock::read()` which yields to the writer; WARN-level log if a query has to wait |
| EC-E-006 | An existing `LazyLock` consumer (test or production code) dereferences the old static container | Fails to compile after migration to `RwLock` — this is intentional and desirable; all callers are updated to `read().unwrap()` or equivalent |
| EC-E-007 | The perimeter compile-fail test previously checked for sealed-trait behavior | Update the test if it asserted that `SensorAuth` cannot be externally implemented; the new assertion should confirm the opposite (compilation of an external impl succeeds) |

---

## Previous Story Intelligence

**S-PLUGIN-PREREQ-F:** Deprecated BC-2.16.004; amended BC-2.01.013 and DI-012; registered VP-PLUGIN-001..007. Key lesson: `prism-spec-engine` was never published with `CustomAdapter` exposed.

**S-PLUGIN-PREREQ-A:** Removed `SensorType` closed enum; all `SensorType::X` dispatch arms migrated to `SensorId`-based open dispatch. Key lesson: atomic commit is mandatory when type signatures change across crates. PREREQ-E has a smaller blast radius (~9 files) but the same atomicity requirement.

**S-PLUGIN-PREREQ-D:** Wired `PluginRuntime` into boot sequence. Key lesson: `PluginRuntime` is now accessible at boot; its registration call surface is available for `WriteToolInvalidationMap.register_write_tool` wiring in Task 7.

**TD-S-PLUGIN-PREREQ-A-003 (routed to this story):** `WriteToolInvalidationMap` was converted from `&[...]` static slice to `LazyLock<Vec<...>>` in S-PLUGIN-PREREQ-A fix-burst-2. The `LazyLock<Vec<T>>` provides only `Deref<Target=Vec<T>>` (read-only). True runtime extensibility requires `RwLock<Vec<...>>` + a `register_write_tool` API. This story closes TD-S-PLUGIN-PREREQ-A-003 by delivering both.

---

## Implementation Notes

**Atomic commit is mandatory.** All 7-9 file changes must land in a single squash commit. There is no intermediate compile-clean state between "sealed marker exists" and "all callers updated." Stage all changes before the first `cargo build` validation pass.

**Sealed marker removal is a pure deletion.** In Rust, the `private::Sealed` pattern means: (a) a private module defines a `Sealed` trait, (b) the public trait bounds on `SensorAuth` include `: Sealed`, (c) all in-crate impls also `impl private::Sealed for X`. Removal requires: delete the private module, remove `: Sealed` from `SensorAuth` supertrait list, and remove all `impl private::Sealed for X` blocks. The four concrete auth structs' other impl blocks are untouched.

**`RwLock<Vec<WriteToolInvalidationMap>>` write contention is negligible.** `register_write_tool` is called once per plugin at boot, not on the query hot path. Readers (`read().unwrap()`) acquire in O(1) with no blocking unless a concurrent `write()` is active. This is the correct pattern for a read-heavy, write-rare structure.

**E-SPEC-008 is NOT deleted.** The error code ID is append-only. The taxonomy row is annotated as retired with a pointer to E-PLUGIN-001 for the replacement behavior (WASM plugin panics). This ensures that any old integration test or documentation referencing E-SPEC-008 finds an explanation rather than a gap.

---

## Green Gate Definition of Done

The story is shipped when ALL of the following are true:
1. `cargo build --workspace --all-features` exits 0 (zero errors, zero warnings with `-D warnings`)
2. `just check` exits 0 (fmt + clippy + nextest + doctests + crate-layout)
3. `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` returns ZERO hits in `src/` paths
4. `grep -rn "private::Sealed\|: Sealed\|impl Sealed for" crates/prism-sensors/src/auth/` returns ZERO hits
5. All 10 ACs are verifiable with explicit grep/test evidence recorded in the PR description
6. `BC-2.16.004` frontmatter shows `lifecycle_status: removed`
7. `E-SPEC-008` in `error-taxonomy.md` has a `retired:` annotation
8. TD-S-PLUGIN-PREREQ-A-003 is closed in `tech-debt-register.md` with a pointer to this story's PR
9. Holdout scenarios HS-PREREQ-E-001/002/003 are registered in `HOLDOUT-INDEX.md` (state-manager task)
10. PR is squash-merged into `develop` as exactly ONE commit

---

## References

All BCs cited in this story (frontmatter `behavioral_contracts` array and body table):

- [BC-2.01.013](../specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md) — DataSource Trait
- [BC-2.01.016](../specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md) — SensorAuth Open Trait (NEW)
- [BC-2.16.004](../specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md) — CustomAdapter Escape Hatch (DEPRECATED → REMOVED)
- [BC-2.16.011](../specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md) — CustomAdapter Retirement (NEW)
- [BC-2.16.012](../specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md) — PluginRegistry Dispatch Migration (NEW)

Architecture Compliance:
- [ADR-023](../specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md) §Architectural Constraints (C5 bullet) — SensorAuth un-sealing + CustomAdapter removal + spec_parser migration
- [ADR-026](../specs/architecture/decisions/ADR-026-sensorauth-unsealing.md) — SensorAuth unsealing architectural decision; §D3 runtime enforcement rules map to E-SPEC-012/013/014
- [ADR-027](../specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md) — CustomAdapter deprecation/removal; §D3 compile-fail perimeter (VP-155) + §D5 WASM equivalence (VP-154)
- [VP-153](../specs/verification-properties/vp-153-sensorauth-runtime-cross-composition-prevention.md) — SensorAuth Runtime Cross-Composition Prevention proptest (anchors BC-2.01.016 E-SPEC-012/013/014)
- [VP-154](../specs/verification-properties/vp-154-custom-adapter-behavioral-equivalence.md) — CustomAdapter Behavioral Equivalence integration test (P1; PLUGIN-MIGRATION-001-A scope)
- [VP-155](../specs/verification-properties/vp-155-custom-adapter-no-public-api.md) — CustomAdapter Absent from prism-spec-engine Public API compile-fail perimeter (P0; PLUGIN-MIGRATION-001-A scope)
- [VP-INDEX](../specs/verification-properties/VP-INDEX.md) — VP-PLUGIN-001 (perimeter test must remain green), VP-PLUGIN-007 (allowlist enforcement unaffected)

Prior PREREQ stories:
- [S-PLUGIN-PREREQ-A](S-PLUGIN-PREREQ-A-sensorid-newtype.md) — SensorId open newtype (merged PR #142)
- [S-PLUGIN-PREREQ-D](S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md) — PluginRuntime boot wiring (merged PR #149)

Tech debt closed:
- [TD-S-PLUGIN-PREREQ-A-003](../tech-debt-register.md) — WriteToolInvalidationMap extensibility

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | S-PLUGIN-PREREQ-E-fix-burst-1 | 2026-05-15 | product-owner | F-LP1-HIGH-001: BC-2.01.016 §Preconditions method surface updated per ADR-026 D1 (2-method trait: `as_any()` + `auth_type_name()`). F-LP1-HIGH-003: All 8 §C5 phantom-heading citations in story body corrected to `§Architectural Constraints (C5 bullet[, Rule N])` per POL-21. F-LP1-MED-001: E-SPEC-008 retirement framing updated — action now `RETIRED … error-taxonomy.md v1.26` (path (a) chosen: PO delivers retirement annotation in v1.26 spec-burst, not deferred to implementer). F-LP1-MED-004: All ~9 `TD-A-003` alias occurrences replaced with canonical `TD-S-PLUGIN-PREREQ-A-003` (frontmatter comment, assumption_validations, risk_mitigations, BC table, Task 7, AC-9, Green Gate DoD 8, Previous Story Intelligence, References). F-LP1-MED-005: Red Gate test 2 rewritten to standard Red Gate semantics (pre/post-implementation assertion states explicit). |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Cross-domain reconciliation with architect's parallel ADR-026/027/VP-153/154/155. Q1: authored E-SPEC-012/013/014; corrected BC-2.01.016 error code references (E-SPEC-010/011/012 → E-SPEC-012/013/014). Q2: pre-staged, waiting for architect framing choice (no action). Q3: BC-2.16.011 §VP-154 Fixture Acceptance Criterion added (OCSF Detection Finding 2004 schema, semantic-equality behavioral equivalence definition). Q4: HS-PREREQ-E-001 +VP-153 sub-scenario; HS-PREREQ-E-002 +VP-154/VP-155 sub-scenarios; HS-PREREQ-E-003 VP-155 note. Q5: story frontmatter updated (verification_properties: VP-153/154/155; architectural_decisions: ADR-026/027/ADR-023; holdout_scenarios: HS-PREREQ-E-001/002/003; anchor_vps updated). AC-3 error code corrected to E-SPEC-012. Error Taxonomy Additions section updated to list 3 new codes + E-SPEC-008 retirement. References updated with ADR-026/027/VP-153/154/155 links. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Authored from ADR-023 §Architectural Constraints (C5 bullet) scope + PREREQ-A/D context. Three new BCs (BC-2.01.016, BC-2.16.011, BC-2.16.012), three holdout scenarios (HS-PREREQ-E-001/002/003), 10 ACs, 8 Red Gate tests, TD-S-PLUGIN-PREREQ-A-003 closure in scope. |
