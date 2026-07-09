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
crates_touched: [prism-sensors, prism-spec-engine, prism-query, prism-bin]
target_module: prism-sensors
subsystems: [SS-01, SS-07, SS-16, SS-17, SS-22]
capabilities: [CAP-001, CAP-029]
version: "1.55"
modified: "2026-05-21"
level: "L4"
producer: product-owner
timestamp: "2026-05-16T00:00:00Z"
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
  - VP-156  # WriteToolInvalidationMap Registration Uniqueness (proptest P1) — anchors BC-2.16.012 TD-S-PLUGIN-PREREQ-A-003 closure
  - VP-PLUGIN-001  # No production hardcoded sensor references — perimeter check must remain green
  - VP-PLUGIN-007  # Plugin manifest allowlist enforcement — must remain unaffected
architectural_decisions:
  - ADR-026  # SensorAuth unsealing decision — defines runtime enforcement rules (E-SPEC-012/013/014) and VP-153
  - ADR-027  # CustomAdapter Same-Burst Removal — Perimeter Enforcement (defines compile-fail perimeter (VP-155) and WASM equivalence (VP-154))
  - ADR-023  # Plugin-only sensor architecture — §Architectural Constraints (C5 bullet) rules authoritative for this story's scope
  - ADR-022  # Production runtime wiring — §B step 7.5/8 ordering authoritative for Task 7b AtomicBool flag set-time
holdout_scenarios:
  - HS-PREREQ-E-001  # SensorAuth Open Trait — External Implementation Compiles and Loads (+ VP-153 cross-composition)
  - HS-PREREQ-E-002  # CustomAdapter Retirement — No Behavioral Regression (+ VP-154/VP-155 coverage)
  - HS-PREREQ-E-003  # PluginRegistry Dispatch — Behavioral Equivalence + WriteToolInvalidationMap extensibility
anchor_bcs: [BC-2.01.016, BC-2.16.011, BC-2.16.012, BC-2.01.013, BC-2.16.004]
anchor_vps: [VP-153, VP-154, VP-155, VP-156, VP-PLUGIN-001, VP-PLUGIN-007]
anchor_capabilities: [CAP-001, CAP-029]
anchor_subsystem: [SS-01, SS-07, SS-16, SS-17]
assumption_validations:
  - "prism-spec-engine has never been published to crates.io with CustomAdapter exposed (PLUGIN-AUDIT-001 HIGH-3 confirmed — no deprecation window required)"
  - "spec_parser.rs contains zero CustomAdapter/CustomAdapterRegistry references (ADR-023 §Architectural Constraints (C5 bullet) F-CRIT-NEW-001-PASS2-RESIDUAL verified by grep)"
  - "S-WAVE5-PREP-01 already removed custom_adapter_registry references from boot.rs — no boot.rs changes required in PREREQ-E"
  - "TD-S-PLUGIN-PREREQ-A-003 (WriteToolInvalidationMap extensibility) is routed to PREREQ-E per S-PLUGIN-PREREQ-A fix-burst-2 decision"
risk_mitigations:
  - "AC-1..3c: SensorAuth unsealing is pure deletion + per-test-fixture credential-validation coverage. Risk: E-SPEC-012/013/014 regression at credential-validation pass. Mitigation: AC-3 + AC-3b + AC-3c Red Gate tests assert error-on-invalid (Test 3, 4, 5)."
  - "AC-4..6: CustomAdapter deletion confirmed safe by PLUGIN-AUDIT-001 (zero external consumers). Risk: registry-dispatch hot-path regression vs legacy CustomAdapter::override_fetch behavior. Mitigation: CustomAdapter retirement verified via Red Gate Tests 6 (type absence: test_BC_2_16_011_001_custom_adapter_absent_post_deletion) + 7 (E-SPEC-008 not constructed by live code: test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code); AC-6 holdout HS-PREREQ-E-002-06 frontmatter verification. AC-5 mechanism: lib.rs re-export removal verified by perimeter-violation compile-fail style pattern (style reference: existing `tests/external/perimeter-violation/` crate; VP-155 CustomAdapter perimeter authored at `tests/external/no-hardcoded-sensors/` in PLUGIN-MIGRATION-001-A scope per ADR-027 D3). Behavioral-equivalence verification (CrowdStrikeAdapter registry-dispatch vs legacy CustomAdapter::override_fetch) is deferred to VP-154 (P1, PLUGIN-MIGRATION-001-A scope per ADR-027 §Verification Property Anchors)."
  - "AC-7..8: spec_parser.rs migration verified by behavioral-equivalence integration test. Risk: type-name collision with shadow enum or stale callsite. Mitigation: AC-7 Red Gate Test 8 + AC-8 Red Gate Test 9 sibling-sweep checks."
  - "AC-9: TD-S-PLUGIN-PREREQ-A-003 closure via RwLock<Vec<WriteToolInvalidationMap>> + AtomicBool query-phase flag. Risk: post-boot register_write_tool() leaks past production call-site gate. Mitigation: AC-9 third-test asserts public-API `mark_query_phase_started()` invocation (FB45 hardening) + WARN tracing event field schema per BC-2.16.002 row 33 (Red Gate Test 13)."
  - "AC-10: full build and pre-push gate, single squash-merge commit. Risk: partial commits in feature branch before final squash; lefthook hook bypass. Mitigation: AC-10 production-grade gate asserts `just check` clean pre-push (process gate, not a Red Gate test); pre-commit hook enforces fmt+clippy+layout per lefthook.yml."
  - "AC-11: E-SPEC-008 retirement annotation — two-layer enforcement model per architect adjudication FB-PR-1-error-taxonomy-test-relocation.md (Option 1). Risk: (a) E-SPEC-008 construction site reintroduced in code, (b) error-taxonomy.md annotation regresses. Mitigation: (a) Red Gate Test 14 (`test_BC_2_16_011_e_spec_008_retired_annotation`) greps `crates/*/src/` for `ESpec008`/`E-SPEC-008` construction sites — zero allowed; POL-1 exempts the variant declaration in `prism-core/src/error.rs`. (b) `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` asserts `'RETIRED in S-PLUGIN-PREREQ-E'` + `'ADR-027'` present in E-SPEC-008 row; runs in `.factory/` pre-commit chain and wave-gate check. ID preserved per append_only_numbering (POL-1). (traces to BC-2.16.011 AC-11)"
acceptance_criteria_count: 13
red_gate_tests: 14
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
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | SS-01 | Primary delivery — sealed marker removed; four built-in auth impls each add one new method body (`auth_type_name`); runtime Rule 2 enforcement confirmed |
| BC-2.16.011 | CustomAdapter Rust Trait Retirement — Removal of Trait, Registry, and All Call Sites | SS-16 | Primary delivery — `custom_adapter.rs` deleted; three call sites cleaned; BC-2.16.004 transitions to removed |
| BC-2.16.012 | PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup | SS-16 | Primary delivery — open dispatch migration; behavioral equivalence test; TD-S-PLUGIN-PREREQ-A-003 WriteToolInvalidationMap closure |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | Awareness — PREREQ-F established that SensorAuth is NOT sealed; this story is the mechanical implementation of that amendment |
| BC-2.16.004 | Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient | SS-16 | Lifecycle close (`deprecated → removed`) — `lifecycle_status: deprecated → removed`; this story is the execution of the retirement planned in PREREQ-F |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,000 |
| BC-2.01.016 (SensorAuth open trait) | ~2,500 |
| BC-2.16.011 (CustomAdapter retirement) | ~2,500 |
| BC-2.16.012 (PluginRegistry migration) | ~2,500 |
| `crates/prism-sensors/src/auth/mod.rs` (sealed marker removal) | ~400 |
| `crates/prism-sensors/src/auth/crowdstrike.rs` (add `auth_type_name` method) | ~50 |
| `crates/prism-sensors/src/auth/cyberint.rs` (add `auth_type_name` method) | ~50 |
| `crates/prism-sensors/src/auth/claroty.rs` (add `auth_type_name` method) | ~50 |
| `crates/prism-sensors/src/auth/armis.rs` (add `auth_type_name` method) | ~50 |
| `crates/prism-spec-engine/src/custom_adapter.rs` (deletion) | ~0 (deleted) |
| `crates/prism-spec-engine/src/lib.rs` (re-export removal) | ~300 |
| `crates/prism-spec-engine/examples/demo_spec_loading.rs` (cleanup/delete) | ~200 |
| `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` (deletion) | ~0 (deleted) |
| `crates/prism-spec-engine/src/spec_parser.rs` (open dispatch migration) | ~800 |
| `crates/prism-spec-engine/src/spec_parser.rs` (Task 6b: 3-validator E-SPEC-012/013/014 implementation calling the 3 new SpecEngineError variants per ADR-026 D3 + BC-2.01.016 Rule 2 / ADR-023 Rule 2) | ~250 |
| `crates/prism-query/src/invalidation.rs` (WriteToolInvalidationMap RwLock migration + AtomicBool flag + mark_query_phase_started helper) | ~700 |
| `crates/prism-spec-engine/src/error.rs` (Task 7c: WriteToolRegistrationAfterBoot variant + Task 6c: AuthTypeCrossComposition + MultipleCredentialRefs + AuthTypeCredentialMismatch variants — all with redacted Debug impls per AD-017) | ~150 |
| `crates/prism-spec-engine/src/plugin/mod.rs` (or `loader.rs`) (PluginRuntime write-tool registration wiring) | ~150 |
| `crates/prism-bin/src/boot.rs` (Task 7b: 1-line `mark_query_phase_started()` insertion) | ~30 |
| `BC-2.16.004-rust-escape-hatch.md` (frontmatter: deprecated → removed) | ~200 |
| `error-taxonomy.md` (E-SPEC-008 retired annotation) | ~100 |
| Test files (Red Gate set + behavioral equivalence) | ~2,000 |
| `crates/prism-query/Cargo.toml` (Task 7d: add `tracing-test = "0.2"` to `[dev-dependencies]`) | ~30 |
| Total | ~18,010 |

Well within the 30% context window budget (~40k tokens).

---

## Tasks

1. **Remove `private::Sealed` marker from `crates/prism-sensors/src/auth/mod.rs`**
   - Delete the `mod private { pub trait Sealed {} }` block (or equivalent sealed-marker pattern)
   - Remove `private::Sealed` from the `SensorAuth` trait's supertrait bounds (`trait SensorAuth: Sealed` → `trait SensorAuth`)
   - Verify that after Task 1 (sealed-marker removal) AND Task 1b (auth_type_name addition per ADR-026 D1/D2 Path B), the four concrete auth impls compile cleanly. The "without modification" claim from prior drafts was incorrect — ADR-026 D2 Path B mandates a new method body per impl.

1b. **Expand `SensorAuth` trait surface per ADR-026 D1/D2 Path B**
   - Step 1: In `crates/prism-sensors/src/auth/mod.rs`, add `fn auth_type_name(&self) -> &'static str;` method declaration to the `pub trait SensorAuth` body (per ADR-026 §D1 — 2-method trait surface; per §D2 Path B — no default impl).
   - Step 2: In `crates/prism-sensors/src/auth/crowdstrike.rs` (or wherever `impl SensorAuth for CrowdStrikeAuth` lives), add method body `fn auth_type_name(&self) -> &'static str { "oauth2_client_credentials" }`.
   - Step 3: In `crates/prism-sensors/src/auth/cyberint.rs`, add `fn auth_type_name(&self) -> &'static str { "bearer_static" }`.
   - Step 4: In `crates/prism-sensors/src/auth/claroty.rs`, add `fn auth_type_name(&self) -> &'static str { "cookie_roundtrip" }`.
   - Step 5: In `crates/prism-sensors/src/auth/armis.rs`, add `fn auth_type_name(&self) -> &'static str { "api_key" }`.
   - Step 6: Verify `cargo check -p prism-sensors` succeeds with the new trait method declaration + 4 impl bodies wired.

   File paths above match the four auth impl rows in §File Structure Requirements (`crowdstrike.rs`, `cyberint.rs`, `claroty.rs`, `armis.rs`). Auth-type name strings match ADR-026 §D3 canonical enumerated set (also enforced by AC-2, VP-153 Rule A, and E-SPEC-012).

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

**Task 6b** (prism-spec-engine: E-SPEC-012/013/014 runtime validators per ADR-026 D3 + BC-2.01.016 Rule 2)

In `crates/prism-spec-engine/src/spec_parser.rs` (or `pipeline.rs` per ADR-026 D3 location flexibility), implement the three runtime rejection rules for SensorAuth spec-load validation:

1. **E-SPEC-012 (Rule A — multi-valued or out-of-set auth_type):** At TOML `auth_type` parse, reject (a) array values for the auth_type field (must be scalar) AND (b) values outside the closed enumeration `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}`. Error variant: `SpecEngineError::AuthTypeCrossComposition { sensor_id, provided_value }`. Per BC-2.01.016 §Error Cases + ADR-023 §Architectural Constraints Rule 2 Rule A.

2. **E-SPEC-013 (Rule B — multiple credential_refs):** At spec-load, reject any auth method block declaring multiple `credential_refs` entries (cardinality must be exactly 1 credential reference per auth method). Error variant: `SpecEngineError::MultipleCredentialRefs { sensor_id, credential_count }`. Per BC-2.01.016 §Error Cases + ADR-023 Rule 2 Rule B.

3. **E-SPEC-014 (Rule C — auth_type/credential structural mismatch):** At credential-resolution time, reject when the resolved credential's structural shape does not match the declared `auth_type` variant (e.g., `auth_type = bearer_static` with a credential containing `client_id` + `client_secret` — wrong shape). Error variant: `SpecEngineError::AuthTypeCredentialMismatch { sensor_id, expected_shape, actual_shape }`. Per BC-2.01.016 §Error Cases + ADR-023 Rule 2 Rule C.

**Credential redaction discipline (AD-017):** All three new error variants MUST implement custom `Debug` that redacts credential values. Use the existing `RedactedDebug` derive macro pattern or manually implement `impl Debug for ... { ... credential: "<redacted>" ... }`.

**Verification (post-implementation):** Red Gate Tests 2 (E-SPEC-012), 4 (E-SPEC-013), 5 (E-SPEC-014) all transition pre-implementation (assertion fails — no Rule 2 enforcement exists) → post-implementation (assertion passes — validator returns the correct error variant). Each test uses a `SensorSpec` fixture that violates exactly one rule and asserts `result.is_err()` AND the error variant matches.

Estimated effort: ~3 hours implementation + ~1 hour test fixtures + ~30 min verification.

**Task 6c** (prism-spec-engine: add 3 new SpecEngineError variants in error.rs — definition site for Task 6b validators)

In `crates/prism-spec-engine/src/error.rs`, add 3 new variants to the `SpecEngineError` enum (sibling to Task 7c's `WriteToolRegistrationAfterBoot` addition):

1. `AuthTypeCrossComposition { sensor_id: String, provided_value: String }` — returned by Task 6b validator for E-SPEC-012 (Rule A multi-valued or out-of-set auth_type)
2. `MultipleCredentialRefs { sensor_id: String, credential_count: usize }` — returned by Task 6b validator for E-SPEC-013 (Rule B multiple credential_refs)
3. `AuthTypeCredentialMismatch { sensor_id: String, expected_shape: String, actual_shape: String }` — returned by Task 6b validator for E-SPEC-014 (Rule C auth_type/credential structural mismatch)

All three variants MUST implement custom `Debug` that redacts credential values per AD-017 (use the existing `RedactedDebug` derive macro pattern OR manually implement `impl Debug for SpecEngineError::AuthTypeCredentialMismatch { ... actual_shape: "<redacted>" ... }`).

**Verification (post-implementation):** Codebase grep `rg "AuthTypeCrossComposition|MultipleCredentialRefs|AuthTypeCredentialMismatch" crates/prism-spec-engine/src/error.rs` returns 3 hits (variant declarations); `rg "fmt::Debug for SpecEngineError" crates/prism-spec-engine/src/error.rs` includes redacted-format branches for the 3 new variants.

Estimated effort: ~1 hour implementation + ~15 min Debug-impl tests.

7. **Migrate `WriteToolInvalidationMap` to runtime-extensible container (TD-S-PLUGIN-PREREQ-A-003)**
   - In `crates/prism-query/src/invalidation.rs`, change the `LazyLock<Vec<WriteToolInvalidationMap>>` container to `std::sync::RwLock<Vec<WriteToolInvalidationMap>>` (eager init per ADR-026 §D7 — `OnceLock<RwLock<...>>` wrapper is not needed because no initialization-race risk exists under the boot-step 7.5/8 ordering, and eager `RwLock::new(Vec::new())` is simpler than the `OnceLock::get_or_init` pattern that can panic in test contexts)
   - The `WriteToolInvalidationMap` struct carries a `plugin_name: String` field (set by PluginRuntime from the plugin manifest `name` field per ADR-026 D7 v1.23; cited in BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33). The struct fields are: `sensor_id: SensorId`, `tool_name: String`, `plugin_name: String` (at minimum; other fields per implementation).
   - Add a `pub fn register_write_tool(entry: WriteToolInvalidationMap) -> Result<(), SpecEngineError>` API that acquires a write guard, checks for a duplicate `tool_name`, and either returns `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))` on duplicate or pushes the entry on success
   - Update all read-side callers (the invalidation check function) to acquire a read guard instead of dereferencing the `LazyLock`
   - Wire `PluginRuntime` (already available via PREREQ-D boot wiring) to call `register_write_tool` for each plugin that declares write-tool capabilities in its manifest

7b. **Add `AtomicBool` query-phase flag for post-boot registration detection (ADR-026 D7)**
   - In `crates/prism-query/src/invalidation.rs`, declare a `static QUERY_PHASE_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);` module-level static (per ADR-026 §D7 runtime_deliverables item "AtomicBool query-phase flag for after-boot detection in crates/prism-query/src/invalidation.rs (D7)")
   - The flag is set to `true` as the first act of step 8 (query-engine init, ADR-022 §B) — immediately when step 8 begins, before any QueryEngine construction proceeds. All plugin registrations at step 7.5 are already complete when step 8 starts; setting the flag here closes the write window permanently at the step-8 boundary. Add a `pub fn mark_query_phase_started()` function that calls `QUERY_PHASE_STARTED.store(true, std::sync::atomic::Ordering::Release)`. The production caller is `crates/prism-bin/src/boot.rs`: add `prism_query::invalidation::mark_query_phase_started();` as the **first statement** of the step-8 init function (the function that constructs `QueryEngine::new()`), immediately before the `QueryEngine::new()` call. This is the sole permitted boot.rs modification per F-LP56-HIGH-001 adjudication (ADR-026 D7 v1.23). Add `prism-query` to the `use` declarations in boot.rs as needed to resolve the path.
   - In `register_write_tool`, before acquiring the write guard, check `QUERY_PHASE_STARTED.load(std::sync::atomic::Ordering::Acquire)`: if `true`, emit a `tracing::warn!(event_type = "write_tool_registration_after_boot", plugin_name = %entry.plugin_name, tool_name = %entry.tool_name, error = "E-PLUGIN-020")` structured event and return `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` without touching the `RwLock` (per ADR-026 §D7 fail-closed post-boot path; BC-2.16.012 EC-016-012-005; error-taxonomy.md E-PLUGIN-020)
   - The three structured event fields (`plugin_name`, `tool_name`, `error`) match the ADR-026 §D7 field source specification exactly

7c. **Add `SpecEngineError::WriteToolRegistrationAfterBoot` enum variant (ADR-026 D7)**
   - In the `SpecEngineError` enum (locate via `crates/prism-spec-engine/src/error.rs` or equivalent per current crate layout), add a unit variant: `WriteToolRegistrationAfterBoot` (per ADR-026 §D7 runtime_deliverables item "SpecEngineError::WriteToolRegistrationAfterBoot enum variant added (D7)"; cited in error-taxonomy.md E-PLUGIN-020 and BC-2.16.012 EC-016-012-005)
   - This is a unit variant (no fields) — the dynamic context is carried by the structured tracing event fields, not the error variant (E-PLUGIN-020 category: runtime, severity: broken)
   - Verify `cargo check -p prism-spec-engine` and `cargo check -p prism-query` both succeed with the new variant wired into `register_write_tool` return path

7d. **Add `tracing-test` dev-dependency**
   - In `crates/prism-query/Cargo.toml` `[dev-dependencies]`, add `tracing-test = "0.2"` (minor-band pinning per prism conservative-pinning convention).
   - Required for AC-9 third-test tracing assertion fixture (per Task 7b production call-site gate + AC-9 event field schema verification).
   - Token budget: 5 minutes / ~30 tokens.

8. **Update BC-2.16.004 frontmatter to `removed`**
   - Open `BC-2.16.004-rust-escape-hatch.md` and perform all four field mutations in one cohesive edit:
     - Update `deprecated_by: ADR-023` → `deprecated_by: ADR-027` (ADR-027 §Decision is the operational deletion mandate; ADR-023 Rule 5 is the deprecation philosophy that ADR-027 operationalizes)
     - Add `removed: "<PREREQ-E merge date>"` (substitute the actual PREREQ-E merge date at PR-create time; use ISO 8601 format YYYY-MM-DD)
     - Add `removal_reason: "PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`
     - Update `lifecycle_status: deprecated` → `lifecycle_status: removed`
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
(traces to BC-2.01.016 postcondition — sealed marker removed; trait is publicly implementable; BC-2.01.013 — this story is the mechanical delivery of the un-sealing amendment that PREREQ-F made to BC-2.01.013)

**AC-2 (Four Built-In Auth Impls Minimal Diff Post-Unsealing):**
The four concrete auth implementations (`CrowdStrikeAuth`, `CyberintAuth`, `ClarotyAuth`, `ArmisAuth`) require ONE NEW METHOD BODY each and no other changes to their `impl SensorAuth for X` blocks. Each impl adds exactly one line: `fn auth_type_name(&self) -> &'static str { "<auth_type_string>" }` returning the static auth-type name for that implementation (e.g., `"oauth2_client_credentials"` for `CrowdStrikeAuth`). No other changes are made to these impl blocks. If any impl referenced the `Sealed` marker via `impl private::Sealed for X`, that block is also removed; all other impl content is preserved verbatim.
`cargo build -p prism-sensors` after the change exits 0 with zero warnings.
(traces to BC-2.01.016 postcondition — four built-in auth impls require only one new method body each (auth_type_name); INV-AUTH-OPEN-002)

**AC-3 (Runtime Auth-Composition Rejection Active):**
A unit test confirms that a `SensorSpec` with `auth_type = ["oauth2_client_credentials", "bearer_static"]` is rejected at spec-load with `E-SPEC-012`. This verifies that the sealed-trait removal does NOT weaken the threat model — rejection moves from compile time to runtime. Note: E-SPEC-012 (not E-SPEC-010; E-SPEC-010 is reserved for variable interpolation field-path misses per error-taxonomy v2.26).
(traces to BC-2.01.016 invariant INV-AUTH-OPEN-003; ADR-023 Rule 2, Rule A; error-taxonomy v2.26)

**AC-3b (Runtime Auth-Composition Rejection — Multiple credential_refs Rejected, E-SPEC-013):**
A unit test confirms that a `SensorSpec` with multiple `credential_refs` per auth method (e.g., `[[sensor.credential_refs]]` declared twice for the same auth method) is rejected at spec-load with `E-SPEC-013`. Test name: `test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected`.
(traces to BC-2.01.016 §Error Cases E-SPEC-013; ADR-023 §Architectural Constraints Rule 2, Rule B (multiple credential_refs); error-taxonomy v2.26)

**AC-3c (Runtime Auth-Composition Rejection — Credential Type Mismatch Rejected, E-SPEC-014):**
A unit test confirms that a `SensorSpec` with structural mismatch between `auth_type` and resolved credential type (e.g., `auth_type = "oauth2_client_credentials"` paired with an API-key-shaped credential) is rejected with `E-SPEC-014`. Test name: `test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected`.
(traces to BC-2.01.016 §Error Cases E-SPEC-014; ADR-023 §Architectural Constraints Rule 2, Rule C (credential type mismatch); error-taxonomy v2.26)

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
`BC-2.16.004-rust-escape-hatch.md` frontmatter contains all four expected field states:
- `deprecated_by: ADR-027` (NOT `ADR-023` — ADR-027 §Decision is the operational deletion mandate)
- `removed:` is set to a valid ISO 8601 date matching the actual PREREQ-E merge date (format: `YYYY-MM-DD`)
- `removal_reason: "PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`
- `lifecycle_status: removed`
The file is NOT deleted (historical record preservation per DF-030 append_only_numbering).
(traces to BC-2.16.011 postcondition; BC-2.16.004 — this AC executes the lifecycle close (deprecated → removed) for BC-2.16.004 per DF-030 BC deprecation protocol)

**AC-7 (spec_parser.rs Open Dispatch — No Hardcoded Sensor Name Match Arms in Dispatch Context):**
`grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` returns ZERO matches in production dispatch match-arm contexts. Sensor name strings may still appear in doc comments or test fixture values (those are acceptable).
(traces to BC-2.16.012 postcondition — open dispatch; INV-SPEC-PARSER-OPEN-001)

**AC-8 (Behavioral Equivalence for Four Initial Sensors):**
Four integration tests (`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike`, `test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint`, `test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty`, `test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis`) parse each of the four built-in TOML sensor specs via the migrated `SpecParser` and assert that the resulting `SensorSpec` struct is identical to the pre-migration baseline. A novel sensor name (`"hypothetical_sensor"`) parses without error via the generic path, producing a valid `SensorSpec`.
(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-002 + INV-SPEC-PARSER-OPEN-003)

**AC-9 (WriteToolInvalidationMap Runtime Extensibility — TD-S-PLUGIN-PREREQ-A-003 Closed):**
`crates/prism-query/src/invalidation.rs` `WriteToolInvalidationMap` container is `RwLock<Vec<WriteToolInvalidationMap>>` (or equivalent). The `WriteToolInvalidationMap` struct includes a `plugin_name: String` field sourced from the plugin manifest `name` field (set by PluginRuntime per ADR-026 D7 v1.23); this field is the source for the `plugin_name` structured event field in the `write_tool_registration_after_boot` WARN tracing event (BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33). A `pub fn register_write_tool(entry: WriteToolInvalidationMap) -> Result<(), SpecEngineError>` API exists and is callable after startup. A unit test (`test_BC_2_16_012_003_write_tool_invalidation_runtime_register`) registers a custom write tool entry and asserts `.is_ok()` on the happy path and that the entry is present in the map on the next read-guard acquisition. A second test invocation with the same `tool_name` asserts `.is_err()` (E-PLUGIN-012). A third test verifies the production call path: call `prism_query::invalidation::mark_query_phase_started()` directly (as boot.rs will call it), then call `register_write_tool(entry)` and assert `.is_err()` with `WriteToolRegistrationAfterBoot` (E-PLUGIN-020). This is NOT a direct `QUERY_PHASE_STARTED.store(true, ...)` in the test body — the test must invoke the public `mark_query_phase_started()` function to confirm the production call site works. Additionally, the third test captures the WARN tracing event emission (via `tracing-test = "0.2"` subscriber fixture (dev-dependency added per Task 7d; pin to minor-band "0.2" per prism conservative-pinning convention)) and asserts the event carries exactly the following fields: `event_type = "write_tool_registration_after_boot"`, `plugin_name = <plugin>`, `tool_name = <tool>`, `error = "E-PLUGIN-020"` per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33.
(traces to BC-2.16.012 postcondition — TD-S-PLUGIN-PREREQ-A-003 WriteToolInvalidationMap; INV-INVALIDATION-EXT-001; EC-016-012-004; EC-016-012-005)

**AC-10 (Full Build and Pre-Push Gate):**
`cargo build --workspace --all-features` exits 0. `just check` (fmt + clippy + nextest + doctests + crate-layout) exits 0 with zero warnings. The PR contains exactly ONE squash-merge commit on `develop`.
(production-grade default — CLAUDE.md Canonical Principle Rule 1)

**AC-11 (E-SPEC-008 Retirement Annotation — Two-Layer Enforcement):**
AC-11 is enforced by two complementary layers per architect adjudication `FB-PR-1-error-taxonomy-test-relocation.md` (Option 1):

**Layer 1 — Code-side (Rust test):** `test_BC_2_16_011_e_spec_008_retired_annotation` (prism-spec-engine) greps `crates/*/src/` and asserts zero `ESpec008` / `E-SPEC-008` construction sites exist. POL-1 (append-only numbering) exempts the variant declaration in `prism-core/src/error.rs` itself. This test fails RED if any live `src/` path constructs or returns `E-SPEC-008`.

**Layer 2 — Spec-side (factory hook):** `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` asserts that the `E-SPEC-008` row in `error-taxonomy.md` contains both `"RETIRED in S-PLUGIN-PREREQ-E"` and `"ADR-027"` markers. The hook runs in the `.factory/` pre-commit chain (triggered by changes to `specs/prd-supplements/error-taxonomy.md`) and unconditionally as a wave-gate hygiene check for wave-0-plugin-prereqs. The annotation text in `error-taxonomy.md` reads: "**RETIRED in S-PLUGIN-PREREQ-E (error-taxonomy.md v1.26).** A CustomAdapter (BC-2.16.004) panicked during execution. Caught via catch_unwind. **No live code path triggers this code after CustomAdapter removal in S-PLUGIN-PREREQ-E per BC-2.16.011 §Error Cases + ADR-027 §Decision (operational deletion mandate). Plugin execution panics now surface via E-PLUGIN-001. ID preserved per append_only_numbering (DF-030).**"
(traces to BC-2.16.011 §Error Cases E-SPEC-008 (retired); Task 9; ADR-027 §Decision)

---

## Red Gate Test Set (failing tests that must exist BEFORE implementation)

The test-writer MUST produce these failing tests before the implementer writes any production code.
Tests are grouped by BC for readability (F-LP2-MED-004 correction).

**BC-2.01.016 (SensorAuth Open Trait):**

1. **`test_BC_2_01_016_001_sensor_auth_external_impl_compiles`** (prism-sensors) — attempts to call a function that accepts `Box<dyn SensorAuth>` with a type defined outside `prism-sensors`; fails RED because `SensorAuth` is sealed and the external impl cannot satisfy the sealed bound.

2. **`test_BC_2_01_016_002_auth_composition_runtime_rejection`** (prism-spec-engine) — constructs a `SensorSpec` with `auth_type = ["oauth2_client_credentials", "bearer_static"]` and attempts to load it via `SensorSpec::load`. Asserts `result.is_err() && err.code() == "E-SPEC-012"`. Pre-implementation: assertion fails because no Rule 2 enforcement exists (load succeeds, `is_err()` is false). Post-implementation: assertion passes (validator returns E-SPEC-012 error per BC-2.01.016 Rule 2 / ADR-023 Rule 2, Rule A).

3. **`test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing`** (prism-sensors) — constructs all four concrete auth types, calls their `SensorAuth` methods, and asserts each impl has exactly one new method body (`auth_type_name`) plus zero other changes vs pre-unsealing baseline; fails RED until `SensorAuth` is unsealed and each impl adds the `auth_type_name` body (because the sealed bound currently blocks external test construction, and the new method does not yet exist).

4. **`test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected`** (prism-spec-engine) — constructs a `SensorSpec` with `[[sensor.credential_refs]]` declared twice for the same auth method and attempts to load it via `SensorSpec::load`. Asserts `result.is_err() && err.code() == "E-SPEC-013"`. Pre-implementation: fails RED because no Rule 2/B enforcement exists. Post-implementation: assertion passes (validator returns E-SPEC-013 per BC-2.01.016 §Error Cases E-SPEC-013; ADR-023 §Architectural Constraints Rule 2, Rule B; error-taxonomy v2.26).

5. **`test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected`** (prism-spec-engine) — constructs a `SensorSpec` with `auth_type = "oauth2_client_credentials"` paired with an API-key-shaped credential and attempts to load it via `SensorSpec::load`. Asserts `result.is_err() && err.code() == "E-SPEC-014"`. Pre-implementation: fails RED because no Rule 2/C structural-mismatch check exists. Post-implementation: assertion passes (validator returns E-SPEC-014 per BC-2.01.016 §Error Cases E-SPEC-014; ADR-023 §Architectural Constraints Rule 2, Rule C; error-taxonomy v2.26).

**BC-2.16.011 (CustomAdapter Rust Trait Retirement):**

6. **`test_BC_2_16_011_001_custom_adapter_absent_post_deletion`** (prism-spec-engine) — attempts to import `prism_spec_engine::CustomAdapter`; fails RED at compile time because the type exists pre-migration. This is a compile-fail test in the style of `tests/external/perimeter-violation/`.

7. **`test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code`** (prism-spec-engine) — searches the workspace `src/` tree for any match arm or handler that constructs `E-SPEC-008`; fails RED if any live code path still produces that error code (all live paths must be absent post-deletion; the error taxonomy entry remains but is retired).

**BC-2.16.012 (PluginRegistry Dispatch Migration):**

8. **`test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch`** (prism-spec-engine) — calls the `SpecParser` with a novel `SensorSpec` TOML for `"hypothetical_sensor"` and asserts it parses without error; fails RED if `spec_parser.rs` still has a hardcoded match arm that rejects unknown sensors.

9. **`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike`** (prism-spec-engine) — parses `crowdstrike.sensor.toml` and compares against a snapshot; fails RED initially because the snapshot must be captured post-migration (the test infrastructure is set up in the Red Gate phase; snapshot is populated during implementation).

10. **`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint`** (prism-spec-engine) — parses `cyberint.sensor.toml` and compares against a snapshot; fails RED initially because the snapshot must be captured post-migration (the test infrastructure is set up in the Red Gate phase; snapshot is populated during implementation). Covers the Cyberint built-in sensor leg of AC-8's four-sensor breadth requirement.

11. **`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty`** (prism-spec-engine) — parses `claroty.sensor.toml` and compares against a snapshot; fails RED initially because the snapshot must be captured post-migration (the test infrastructure is set up in the Red Gate phase; snapshot is populated during implementation). Covers the Claroty built-in sensor leg of AC-8's four-sensor breadth requirement.

12. **`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis`** (prism-spec-engine) — parses `armis.sensor.toml` and compares against a snapshot; fails RED initially because the snapshot must be captured post-migration (the test infrastructure is set up in the Red Gate phase; snapshot is populated during implementation). Covers the Armis built-in sensor leg of AC-8's four-sensor breadth requirement.

13. **`test_BC_2_16_012_003_write_tool_invalidation_runtime_register`** (prism-query) — calls `register_write_tool(entry) -> Result<(), SpecEngineError>` where `entry` is a new `WriteToolInvalidationMap` struct; asserts `.is_ok()` for the happy path (entry visible on next read-guard); asserts `.is_err()` with `E-PLUGIN-012` for a duplicate `tool_name`; asserts `.is_err()` with `WriteToolRegistrationAfterBoot` (E-PLUGIN-020) for a post-boot registration attempt invoked via the public `mark_query_phase_started()` function (not direct `QUERY_PHASE_STARTED.store(true, ...)` in test body) and captures the WARN tracing event (via `tracing-test = "0.2"` subscriber fixture per Task 7d) with fields `event_type = "write_tool_registration_after_boot"`, `plugin_name = <plugin>`, `tool_name = <tool>`, `error = "E-PLUGIN-020"` per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33; fails RED because `register_write_tool` does not exist yet (the container is a `LazyLock<Vec<...>>` not an `RwLock`) and neither error variant exists.

14. **`test_BC_2_16_011_e_spec_008_retired_annotation`** (prism-spec-engine) — asserts that a grep of `crates/*/src/` paths for `ESpec008` / `E-SPEC-008` construction sites returns zero matches (code-side gate — Layer 1 of AC-11 two-layer enforcement model per architect adjudication FB-PR-1-error-taxonomy-test-relocation.md); fails RED if any live `src/` path constructs that error code. Note: the spec-governance annotation invariant (Layer 2 — `"RETIRED in S-PLUGIN-PREREQ-E"` + `"ADR-027"` present in error-taxonomy.md E-SPEC-008 row) is enforced separately by `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh`, not by this Rust test.

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
| Hardcoded sensor name dispatch in `spec_parser.rs` MUST be replaced with open path | ADR-023 §Architectural Constraints (C5 bullet); ADR-027 D5; BC-2.16.012 | `grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` = 0 hits in dispatch contexts |
| `boot.rs` MAY be modified in this story for ONE designated insertion only: add `prism_query::invalidation::mark_query_phase_started();` as the first statement of the step-8 function in `crates/prism-bin/src/boot.rs`, immediately before the `QueryEngine::new()` call (F-LP56-HIGH-001 adjudication; ADR-026 D7 v1.23). All other boot.rs changes remain forbidden. | ADR-026 D7 v1.23; ADR-023 §Architectural Constraints (C5 bullet) scope | Code review; `git diff develop...HEAD -- crates/prism-bin/src/boot.rs` contains EXACTLY ONE line added: `prism_query::invalidation::mark_query_phase_started();` and NO other diff hunks |
| VP-PLUGIN-001 perimeter test MUST remain green after sealed-trait removal | VP-PLUGIN-001 (FORBIDDEN-SYMBOLS-001) | CI grep gate; perimeter compile-fail test must still pass (no new forbidden symbols introduced) |
| Atomic commit: all file changes land in ONE squash commit | CLAUDE.md commit conventions; AC-10 | CI; `git log --oneline develop..HEAD` = 1 commit on `develop` |

---

## Error Taxonomy Additions

Five error codes are introduced or annotated in this story (see `error-taxonomy.md` v2.26 §SPEC and §PLUGIN); one existing code is annotated as retired:

| Code | Action | Purpose |
|------|--------|---------|
| `E-SPEC-012` | NEW (taxonomy v1.25) | ADR-023 Rule 2, Rule A — auth_type must be a single value from the canonical enumerated set; arrays or out-of-set values rejected at spec-load. Credential values must not appear in error message (AD-017). |
| `E-SPEC-013` | NEW (taxonomy v1.25) | ADR-023 Rule 2, Rule B — exactly one credential_ref per auth method; multiple bindings rejected at spec-load. |
| `E-SPEC-014` | NEW (taxonomy v1.25) | ADR-023 Rule 2, Rule C — credential structural type must match declared auth_type; mismatches rejected at credential-resolution time, before any HTTP request. Credential values must not appear in error message (AD-017). |
| `E-PLUGIN-012` | NEW (taxonomy v1.27) | `SpecEngineError::DuplicateWriteToolRegistration(String)` — Two plugins declared the same write tool name; second registration rejected at boot-step 7.5. Severity: broken. Category: boot. ADR-026 D7; BC-2.16.012 EC-016-012-004. |
| `E-PLUGIN-020` | NEW (taxonomy v1.27) | `SpecEngineError::WriteToolRegistrationAfterBoot` — `register_write_tool` called after query-engine init starts at step 8 (per ADR-026 §D7); the write-registration window closes at step 8 start (first act of step 8, before QueryEngine construction proceeds); rejected with WARN-level tracing event. Severity: broken. Category: runtime. ADR-026 D7; BC-2.16.012 EC-016-012-005. |
| `E-SPEC-008` | RETIRED (annotate only, do NOT delete) — error-taxonomy.md v1.26 | Annotate with `retired:` note: "Retired in S-PLUGIN-PREREQ-E. No live code path triggers this code post-CustomAdapter removal. Plugin execution panics surface via E-PLUGIN-001." |

Note: E-SPEC-010 (variable interpolation field-path miss) and E-SPEC-011 (pipe_verb reserved keyword) are pre-existing codes that are NOT related to auth-composition rejection. The erroneous reference to E-SPEC-010/011/012 in prior PREREQ-E authoring has been corrected to E-SPEC-012/013/014.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/src/auth/mod.rs` | Modify | Remove `private::Sealed` module, remove `: Sealed` supertrait from `SensorAuth` trait |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | Modify | Add `fn auth_type_name(&self) -> &'static str { "oauth2_client_credentials" }` per ADR-026 D1 Path B |
| `crates/prism-sensors/src/auth/cyberint.rs` | Modify | Add `fn auth_type_name(&self) -> &'static str { "bearer_static" }` per ADR-026 D1 Path B |
| `crates/prism-sensors/src/auth/claroty.rs` | Modify | Add `fn auth_type_name(&self) -> &'static str { "cookie_roundtrip" }` per ADR-026 D1 Path B |
| `crates/prism-sensors/src/auth/armis.rs` | Modify | Add `fn auth_type_name(&self) -> &'static str { "api_key" }` per ADR-026 D1 Path B |
| `crates/prism-spec-engine/src/custom_adapter.rs` | DELETE | Primary retirement target per ADR-023 Rule 5 |
| `crates/prism-spec-engine/src/lib.rs` | Modify | Remove `mod custom_adapter;` + all `CustomAdapter`/`CustomAdapterRegistry`/`CustomAuth` re-exports |
| `crates/prism-spec-engine/examples/demo_spec_loading.rs` | DELETE or Modify | Remove `CustomAdapter`-using sections; delete file if nothing meaningful remains |
| `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` | DELETE | BC-2.16.004 is removed; this test file is deleted with it |
| `crates/prism-spec-engine/src/spec_parser.rs` | Modify | Add E-SPEC-012/013/014 validator logic (Task 6b): three runtime rejection rules for SensorAuth spec-load validation returning `AuthTypeCrossComposition`, `MultipleCredentialRefs`, `AuthTypeCredentialMismatch` variants per ADR-026 D3 + BC-2.01.016 Rule 2; replace hardcoded sensor-name match arms with PluginRegistry dispatch (Task 6) |
| `crates/prism-query/src/invalidation.rs` | Modify | Migrate `WriteToolInvalidationMap` from `LazyLock<Vec<...>>` to `RwLock<Vec<...>>`; add `register_write_tool` API; struct gains `plugin_name: String` field (set by PluginRuntime from manifest `name` per ADR-026 D7 v1.23; BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33); add `static QUERY_PHASE_STARTED: AtomicBool` module-level static; add `pub fn mark_query_phase_started()` helper that stores `true` with `Release` ordering (called by query-engine init at step 8 start per ADR-026 D7) |
| `crates/prism-query/Cargo.toml` | Modify | Add `tracing-test = "0.2"` to `[dev-dependencies]` (Task 7d; required for AC-9 third-test tracing fixture per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.25) row 33 capture assertion) |
| `crates/prism-spec-engine/src/error.rs` | Modify | Add `SpecEngineError` variants: `WriteToolRegistrationAfterBoot` (Task 7c; ADR-026 D7; error-taxonomy.md E-PLUGIN-020) + `AuthTypeCrossComposition { sensor_id, provided_value }` (Task 6b; E-SPEC-012) + `MultipleCredentialRefs { sensor_id, credential_count }` (Task 6b; E-SPEC-013) + `AuthTypeCredentialMismatch { sensor_id, expected_shape, actual_shape }` (Task 6b; E-SPEC-014) — all three Task 6b variants implement redacted `Debug` per AD-017 |
| `.factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md` | Modify | Update frontmatter: `lifecycle_status: deprecated → removed`; add `removed:` + `removal_reason:` |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Modify | Add `retired:` annotation to E-SPEC-008 row |
| `crates/prism-spec-engine/src/plugin/mod.rs` (or `loader.rs` per current layout) | Modify | Wire PluginRuntime per-plugin write-tool registration: for each loaded plugin, iterate manifest write-tool entries and call `prism_query::invalidation::register_write_tool(entry)` during step 7.5 plugin-load (per ADR-026 §D7; ADR-022 §B step 7.5) |
| `crates/prism-bin/src/boot.rs` | Modify (1-line insertion) | Add `prism_query::invalidation::mark_query_phase_started();` as first statement of step-8 init function, immediately before `QueryEngine::new()` call (F-LP56-HIGH-001 adjudication; ADR-026 D7 v1.23; this is the SOLE permitted boot.rs modification per Architecture Compliance Rule line 365) |

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
5. All 13 ACs are verifiable with explicit grep/test evidence recorded in the PR description
6. `BC-2.16.004` frontmatter shows `lifecycle_status: removed`
7. `E-SPEC-008` in `error-taxonomy.md` has a `retired:` annotation
8. TD-S-PLUGIN-PREREQ-A-003 is closed in `tech-debt-register.md` with a pointer to this story's PR
9. Holdout scenarios HS-PREREQ-E-001/002/003 are registered in `HOLDOUT-INDEX.md` (state-manager task)
10. PR is squash-merged into `develop` as exactly ONE commit

---

## References

All BCs cited in this story (frontmatter `behavioral_contracts` array and body table):

- [BC-2.01.013](../specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md) — DataSource Trait Eliminates Per-Sensor Code Duplication
- [BC-2.01.016](../specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md) — SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) (NEW — this story)
- [BC-2.16.004](../specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md) — Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient (lifecycle: deprecated → removed by this story)
- [BC-2.16.011](../specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md) — CustomAdapter Rust Trait Retirement — Removal of Trait, Registry, and All Call Sites (NEW — this story)
- [BC-2.16.012](../specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md) — PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup (NEW — this story)
- [BC-2.16.002 — Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation](../specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md) — Structured event catalog (row 33: `write_tool_registration_after_boot`); anchors AC-9 third-test event field schema (event_type, plugin_name, tool_name, error).

PRD Supplements:
- [error-taxonomy.md](../specs/prd-supplements/error-taxonomy.md) — Error taxonomy: E-SPEC-008 (retired in PREREQ-E), E-SPEC-012, E-SPEC-013, E-SPEC-014, E-PLUGIN-012, E-PLUGIN-020.

Capabilities:
- [capabilities.md](../specs/domain-spec/capabilities.md) — CAP-001 Sensor Adapter Layer (Internal); CAP-029 Config-Driven Sensor Adapters (per frontmatter `capabilities:` + `anchor_capabilities:`).

Architecture Compliance:
- [ADR-022](../specs/architecture/decisions/ADR-022-production-runtime-wiring.md) — Production runtime wiring; §B step 7.5/8 ordering authoritative for Task 7b AtomicBool flag set-time
- [ADR-023](../specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md) §Architectural Constraints (C5 bullet) — SensorAuth un-sealing + CustomAdapter removal + spec_parser migration
- [ADR-026](../specs/architecture/decisions/ADR-026-sensorauth-unsealing.md) — SensorAuth unsealing architectural decision; §D3 runtime enforcement rules map to E-SPEC-012/013/014
- [ADR-027](../specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md) — CustomAdapter Same-Burst Removal — Perimeter Enforcement in Wave 1/A; §D3 compile-fail perimeter (VP-155) + §Verification Property Anchors WASM equivalence (VP-154)
- [VP-153](../specs/verification-properties/vp-153-sensorauth-runtime-cross-composition-prevention.md) — SensorAuth Runtime Cross-Composition Prevention proptest (anchors BC-2.01.016 E-SPEC-012/013/014)
- [VP-154](../specs/verification-properties/vp-154-custom-adapter-behavioral-equivalence.md) — CustomAdapter Behavioral Equivalence integration test (P1; PLUGIN-MIGRATION-001-A scope)
- [VP-155](../specs/verification-properties/vp-155-custom-adapter-no-public-api.md) — CustomAdapter Absent from prism-spec-engine Public API compile-fail perimeter (P0; PLUGIN-MIGRATION-001-A scope)
- [VP-156](../specs/verification-properties/vp-156-write-tool-registration-uniqueness.md) — WriteToolInvalidationMap Registration Uniqueness proptest (P1; uniqueness-only; anchors BC-2.16.012 TD-S-PLUGIN-PREREQ-A-003 closure)
- [VP-INDEX](../specs/verification-properties/VP-INDEX.md) — VP-PLUGIN-001 (perimeter test must remain green), VP-PLUGIN-007 (allowlist enforcement unaffected)

Prior PREREQ stories:
- [S-PLUGIN-PREREQ-A](S-PLUGIN-PREREQ-A-sensorid-newtype.md) — SensorId open newtype (merged PR #142)
- [S-PLUGIN-PREREQ-D](S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md) — PluginRuntime boot wiring (merged PR #149)

Holdout Scenarios:
- [HS-PREREQ-E-001](../holdout-scenarios/S-PLUGIN-PREREQ-E-HS-001-sensorauth-open-trait.md) — SensorAuth Open Trait external-implementation behavioral compile + load
- [HS-PREREQ-E-002](../holdout-scenarios/S-PLUGIN-PREREQ-E-HS-002-customadapter-retirement.md) — CustomAdapter retirement (no behavioral regression)
- [HS-PREREQ-E-003](../holdout-scenarios/S-PLUGIN-PREREQ-E-HS-003-plugin-registry-dispatch.md) — Plugin registry dispatch + WriteToolInvalidationMap extensibility

Tech debt closed:
- [TD-S-PLUGIN-PREREQ-A-003](../tech-debt-register.md) — WriteToolInvalidationMap extensibility

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| v1.55 | error-taxonomy-v2.26-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **Reconciling pin round (pass-4 closures): error-taxonomy v1.38→v2.26. Seven live version-pin cites updated: (1) AC-3 inline trace (bare); (2) AC-3 parenthetical trace (bare); (3) AC-3b parenthetical trace (bare); (4) AC-3c parenthetical trace (bare); (5) Red Gate test 4 description (bare); (6) Red Gate test 5 description (bare); (7) §Error Taxonomy Additions heading cite (backtick-quoted `error-taxonomy.md` v1.38 form). Historical changelog rows left unchanged per POL-29. Lines 323 + 418 (v1.26 historical quotes) NOT changed. Also syncing frontmatter version 1.53→1.55 to capture the existing v1.54 FB-IMPL-6 entry that was not reflected in frontmatter; both increments combined per POL-23.** |
| v1.54 | FB-IMPL-6 (PLUGIN-MIGRATION-001-E) | 2026-05-23 | implementer | POL-30 Fork B catalog bullet cite-pin sweep: `(v1.24)` → `(v1.25)` at 5 live-narrative sites — lines 219, 311, 363, 438, 439. BC-2.16.002 catalog bullet label advanced from `(v1.24)` to `(v1.25)` in FB-IMPL-6 same burst (addition of `plugin.auth_token_parse_error` row 37; F-LP7-MED-001 closure). Catalog count 36→37. Sibling-sweep per POL-29 step 8f. |
| v1.53 | FB-IMPL-3 (PLUGIN-MIGRATION-001-E) | 2026-05-22 | implementer | POL-30 Fork B catalog bullet cite-pin sweep: `(v1.23)` → `(v1.24)` at 5 live-narrative sites — lines 219, 311, 363, 438, 439. BC-2.16.002 catalog bullet label advanced from `(v1.23)` to `(v1.24)` in FB-IMPL-3 same burst (addition of `plugin_auth_provider_constructed` row 36; F-LP3-LOW-001 closure). Catalog count 35→36. Sibling-sweep per POL-29 v1.29 step 8f. |
| v1.52 | FB-IMPL-1 | 2026-05-21 | product-owner | POL-30 Fork B catalog bullet cite-pin sweep: `(v1.22)` → `(v1.23)` at 5 live-narrative sites — lines 219, 311, 359, 434, 435 (all `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.22) row 33`). BC-2.16.002 catalog bullet label advanced from `(v1.22)` to `(v1.23)` in same burst (addition of `timestamp.fallback_to_now` row 35 per ADR-028 v1.9 §D8-B). |
| v1.51 | FB-PR-1 | 2026-05-19 | product-owner | FB-PR-1 AC-11 relocation: Rust test `test_BC_2_16_011_e_spec_008_retired_annotation` scoped to code-side `ESpec008`/`E-SPEC-008` construction-site grep gate only (Layer 1); spec-governance annotation invariant (`"RETIRED in S-PLUGIN-PREREQ-E"` + `"ADR-027"` in E-SPEC-008 row) moved to `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` (Layer 2) per architect adjudication FB-PR-1-error-taxonomy-test-relocation.md (Option 1). Sites updated: frontmatter risk_mitigations AC-11 entry; §Acceptance Criteria AC-11 body (two-layer enforcement model); §Red Gate Tests Test 14 description. `modified:` synced to 2026-05-19. |
| v1.50 | FB-IMPL-10 | 2026-05-18 | product-owner | F-LP-IMPL-P13-MED-002 closure: frontmatter `modified:` field synced "2026-05-17" → "2026-05-18" per POL-27 (most-recent-change date tracking). Pre-existing since FB-IMPL-7 / pass-10-spec-hygiene (v1.49 authored 2026-05-18 but modified field not updated). ZERO-NEW-DRIFT discipline. |
| v1.49 | pass-10-spec-hygiene | 2026-05-18 | product-owner | F-LP-IMPL-P10-SUG-001 closure (POL-29 step 8h/8i sibling propagation): catalog bullet cite-pins `(v1.21)` → `(v1.22)` at 5 live-narrative sites — lines 219, 311, 359, 434, 435 (all `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.21) row 33`). BC-2.16.002 catalog bullet label advanced from `(v1.21)` to `(v1.22)` per Option B adjudication; story cites must track the new canonical label per POL-29 step 8h. |
| v1.48 | FB75 | 2026-05-17 | product-owner | F-LP87-HIGH-001 closure (PO scope): error-taxonomy v1.37→v1.38 propagation at story lines 72 (backtick variant frontmatter), 271, 272, 276, 280, 337, 339 (7 sites). NEW META-class — same-burst dependent-artifact self-bump: FB73 ADR-026 D7 v1.22→v1.23 sweep at error-taxonomy lines 459+467 caused error-taxonomy.md to bump v1.37→v1.38 as §Changelog event within same atomic burst; POL-29 v1.26 step 8g cross-value-class enumeration covered external value classes but did NOT enumerate the DEPENDENT-ARTIFACT-SELF-BUMP class. POL-29 v1.27→v1.28 step 8h amendment by state-manager. Sibling files HS-001 v1.11 + VP-153 v0.16 + ADR-026 v1.24 swept in same burst. |
| v1.47 | FB74 | 2026-05-17 | product-owner | F-LP86-MED-002 closure (PO scope): §Changelog rows v1.0-v1.30 swept to uniform `v` prefix format matching v1.31+ convention (introduced FB53). 86-pass-surviving within-table schema integrity defect — POL-26 corollary class within-table column-format uniformity. 31 rows reformatted; cell content preserved verbatim per TD-VSDD-091 (only Version-cell prefix added). |
| v1.46 | FB73 | 2026-05-17 | product-owner | F-LP85-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.22→v1.23 propagation at story lines 219, 226, 309, 397×2, 434, 440 (6 sites). 7th consecutive 1-finding cascade-restart-#4 attempt — cross-value-class side-effect bump dimension (FB71 closure bumped ADR-026 v1.22→v1.23 as side-effect of error-taxonomy advancement but step 8e didn't iterate parallel value class). Sibling files BC-2.16.011 v1.10 + BC-2.16.012 v1.26 + BC-2.16.002 v1.31 (POL-30 Fork B preserved line 74) + VP-156 v0.18 + HS-003 v1.15 + error-taxonomy v2.26 swept. POL-29 v1.25→v1.26 step 8g cross-value-class side-effect bump detection amendment by state-manager. |
| v1.45 | FB71 | 2026-05-17 | product-owner | F-LP83-HIGH-001 closure (PO scope): error-taxonomy v1.35→v1.37 propagation at story lines 72 (frontmatter backtick-quoted variant), 271, 272, 276, 280, 337, 339, 405 (8 sites). Recurrence #23+ class (a) — FB69 self-induced error-taxonomy v1.36→v1.37 within F-LP81-HIGH-002 closure but step 8d transitive closure didn't RECURSIVELY iterate to detect second-order propagation. POL-29 v1.23→v1.24 fixed-point iteration amendment by state-manager. Sibling: HS-001 v1.10 + VP-153 v0.15 (PO) + ADR-026 v1.23 (architect). |
| v1.44 | FB69 | 2026-05-17 | product-owner | F-LP81-HIGH-002 closure (PO scope): ADR-026 D7 pin v1.21→v1.22 propagation at story lines 219, 226, 311, 397, 434, 440 (6 sites). Recurrence #22+ of POL-29 step 3a class (b). Provenance: FB62 SM step 8b catch bumped ADR-026 v1.21→v1.22 but META-META gap — step 8b didn't trigger its own external-cite sweep when iteration bumped source-of-truth frontmatter. POL-29 v1.22→v1.23 step 8d META-META transitive closure amendment by state-manager. Sibling files BC-2.16.012 v1.25 + BC-2.16.002 v1.29 + error-taxonomy v1.37 + VP-156 v0.17 + HS-003 v1.14 swept in same burst; ADR-022 by architect. |
| v1.43 | FB68 | 2026-05-17 | product-owner | F-LP80-MED-001 + MED-002 + LOW-001 closure (PO scope): Task 6c added for 3 SpecEngineError variant definitions in error.rs (sibling-class to F-LP79 — variant DEFINITION site vs Task 6b validator LOGIC site; POL-29 v1.21 step 3e first-application missed definition-site discrimination) + Cargo.toml row added to §FSR (sub-dimension of F-LP78 step 3d per-file consistency between §FSR and §Token Budget) + §Token Budget vs §FSR variant-placement contradiction resolved per Option A per-FILE accounting (error.rs ~50→~150 tokens for 4 variants + redacted Debug). Sibling-sweep: all Tasks 1-10 + 6b/6c/7b/7c/7d have §FSR + §Token Budget coverage; all 4 crates_touched have rows in both tables. POL-22 + POL-23 + POL-29 v1.20 step 3d + v1.21 step 3e all closed. |
| v1.42 | FB67 | 2026-05-17 | product-owner | F-LP79-MED-001 closure (PO scope): added Task 6b instructing implementer to write E-SPEC-012/013/014 runtime validators in spec_parser.rs (3 SpecEngineError variants with redacted-Debug per AD-017: AuthTypeCrossComposition + MultipleCredentialRefs + AuthTypeCredentialMismatch; AC-3/3b/3c gates closed via Red Gate Tests 2/4/5; ADR-026 D3 + BC-2.01.016 Rule 2 + ADR-023 Rule 2 honored). §FSR error.rs row updated with 3 new error variants; spec_parser.rs row updated to enumerate Task 6b validator logic + Task 6 dispatch migration. §Token Budget gained Task 6b row (~250 tokens; total ~17,660 → ~17,910). Closes AC↔Task implementation-instruction coverage gap surviving 33+ passes; sibling-class to F-LP78 structural-table-completeness. |
| v1.41 | FB66 | 2026-05-17 | product-owner | F-LP78-MED-001 closure: §File Structure Requirements + §Token Budget Estimate tables augmented with `crates/prism-bin/src/boot.rs` row (sibling-sweep gap from FB44 D-666 surviving 33+ passes — when crates_touched gained prism-bin per F-LP56-HIGH-001 Option A adjudication, both structural tables were not updated). POL-23 sibling-sweep discipline + POL-2 bidirectional traceability closure. Cycle-close DRIFT-OBS-LP78-001 candidate: POL-29 step 3a class (d) structural-table-completeness sibling-sweep mandate when crates_touched is amended. |
| v1.40 | FB64 | 2026-05-17 | product-owner | F-LP76-HIGH-001 closure (PO scope): burst-label cell corrected FB74→FB62 in §Changelog row for v1.38. Original FB62 closure of F-LP74-HIGH-001 was labeled "FB74" derived from finding ID; canonical FB sequential counter was FB62 per state-manager records. POL-26 schema integrity + POL-29 cross-domain sibling consistency restored. |
| v1.39 | FB63 | 2026-05-17 | product-owner | F-LP75-HIGH-001 closure (PO scope): story line 373 backtick-quoted `error-taxonomy.md` v1.34 → v1.35 (single-line fix; FB62 POL-29 v1.18 step 8b first-application missed this variant — caught 11 other sites but state-manager's execution ran canonical/combined grep without explicit per-variant enumeration). Recurrence #21 of META-PATTERN at SAME line 373 site that F-LP65-HIGH-001 first surfaced 10 passes ago. POL-29 v1.18 step 8b execution discipline gap — addressed by state-manager via POL-29 v1.19 amendment (explicit per-variant grep enumeration mandate in step 8b iteration loop). POL-29 v1.18 step 8b per-variant grep evidence: variant-1-bare pre/post=0/0; variant-2-with-md pre/post=0/0; variant-3-backtick pre/post=1/0. |
| v1.38 | FB62 | 2026-05-17 | product-owner | F-LP74-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.19→v1.21 propagation at story lines 187, 194, 279, 365×2, 402 (5 sites; 6 occurrences). Recurrence #20 of POL-29 step 3a registry class (b). Provenance: FB56b swept to v1.19 but ADR-026 bumped v1.19→v1.20 in same atomic commit (SM step 8a catch error-taxonomy propagation at §D7 line 312) + FB57 v1.20→v1.21 (POL-26 bookkeeping) — both bumps did NOT cascade pins. POL-29 v1.17 step 8a META-gap revealed: single-pass enforcement misses transitively-introduced staleness within own application cycle. POL-29 v1.17→v1.18 step 8b transitive closure amendment by state-manager (in-burst META-gap closure per user strategic direction). Sibling files BC-2.16.012 v1.23 + BC-2.16.002 v1.27 + error-taxonomy v1.35 + VP-156 v0.15 + HS-003 v1.12 swept in same burst; ADR-022 by architect. POL-29 step 8a grep evidence (PO-domain): pre-grep 19 → post-grep 0. |
| v1.37 | FB57 | 2026-05-17 | product-owner | F-LP69-MED-002 closure (PO scope): AC-11 description text updated to byte-match canonical error-taxonomy.md line 380 verbatim (preserving markdown bold + `(error-taxonomy.md v1.26)` historical-origin marker). Latent since FB51 (v1.32 enriched taxonomy per AC-11 directive but AC-11 itself was not back-synced). POL-24 (error_message_template_verbatim) closure. CLAUDE.md Source-of-Truth Precedence Rule 3 honored (PRD supplements supersede PRD prose for the same surface area). |
| v1.36 | FB56+FB56b combined SM step 8a catch | 2026-05-17 | state-manager | POL-29 v1.17 step 8a FINAL EMPIRICAL VERIFICATION CATCH: error-taxonomy v1.33→v1.34 propagation incomplete in FB56+FB56b — 9 live-narrative `error-taxonomy v1.33` cites survived in story (lines 72, 239, 240, 244, 248, 305, 307, 373 in body + frontmatter AC-11 risk_mitigation). FB56b PO v1.35 §Changelog claimed error-taxonomy sweep but actual body sites were not updated. State-manager step 8a catch closes the gap in same atomic commit. POL-29 v1.17 step 8a grep evidence: pre-grep 9 story sites → post-grep 0 live-narrative. |
| v1.35 | FB56b | 2026-05-17 | product-owner | F-LP68-HIGH-001 closure cascade (FB56b PO scope expansion): ADR-026 D7 pin v1.18→v1.19 propagation at story lines 187, 194, 279, 365×2, 402 (5 sites; 6 occurrences). Triggered by FB56 architect bump of ADR-026 v1.18→v1.19 to close 1 error-taxonomy cite at line 312; POL-29 v1.17 step 8a FIRST APPLICATION CATCH — diff-derived value-class enumeration detected the side-effect D7 v1.18 staleness BEFORE commit. Sibling files BC-2.16.012 v1.22 + VP-156 v0.14 + HS-003 v1.10 + error-taxonomy v1.35 + BC-2.16.002 v1.26 swept in same burst per POL-23. ADR-022 + ADR-026 body swept by architect. POL-29 v1.17 step 8a grep evidence (combined canonical `rg "ADR-026 (§)?D7 v1\.18"`): pre-grep 19 → post-grep 0 in PO-domain. |
| v1.34 | FB56 | 2026-05-17 | product-owner | F-LP68-HIGH-001 closure (PO scope): error-taxonomy.md v1.32→v1.33 propagation sweep at story lines 72, 239, 240, 244, 248, 305, 307, 373 (8 live-narrative sites; lines 72 + 373 are backtick-quoted variant form per POL-29 v1.16 step 3a (a) registry). FB55 multi-value-class enforcement gap closure (FB55 bumped error-taxonomy as side-effect of D7 cite edits at lines 459/467 but did not enumerate (a) as value class for sweep). POL-29 v1.16 step 8 STRENGTHENED grep evidence (combined canonical `rg "(\`)?error-taxonomy(\.md)?(\`)? v1\.32"`): pre-grep 11 → post-grep 0 in PO-domain. Sibling files HS-001 v1.6 + VP-153 v0.11 swept in same burst per POL-23. ADR-026 swept by architect. POL-29 v1.17 amendment by state-manager: diff-derived value-class enumeration mandate. |
| v1.33 | FB55 | 2026-05-17 | product-owner | F-LP67-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.17→v1.18 propagation at story lines 187, 194, 279, 365, 402 (5 live-narrative sites; missed by FB52 sibling-sweep after ADR-026 v1.17→v1.18 D7 body edit at line 312). POL-29 v1.16 step 3a (b) ADR-026 D7 pin registry first-test surfaced recurrence #18 of class (b). POL-29 step 8 STRENGTHENED grep evidence (PO-domain combined canonical `rg "ADR-026 (§)?D7 v1\.17"`): pre-grep 10 → post-grep 0 in PO-domain. Sibling files BC-2.16.012 v1.21 + VP-156 v0.13 + HS-003 v1.9 + error-taxonomy v1.33 swept in same burst per POL-23. ADR-022 swept by architect. |
| v1.32 | FB54 | 2026-05-17 | product-owner | F-LP66-MED-002 closure: story §risk_mitigations AC-4..6 line 68 phantom-entity fix — `CrowdStrikeSession` → `CrowdStrikeAdapter` (canonical SensorAdapter trait impl at `prism-sensors/src/auth/crowdstrike.rs:112`) + `CustomAdapter::call_action` → `CustomAdapter::override_fetch` (canonical method at `prism-spec-engine/src/custom_adapter.rs:42`). POL-22 Phase C sibling-cite cross-check: BC-2.16.011 §Postconditions + VP-154 §Property Statement + ADR-027 references all canonical-conformant. POL-29 v1.15 grep evidence per value class: (1) `CrowdStrikeSession` — pre-grep 1 → post-grep 0 in PO-domain; (2) `CustomAdapter::call_action` — pre-grep 1 → post-grep 0 in PO-domain. |
| v1.31 | FB53 | 2026-05-17 | product-owner | F-LP65-HIGH-001 closure: story line 373 `error-taxonomy.md` v1.31 → v1.32 (markdown-backtick-quoted variant form missed by FB52 5-site sweep — sub-dimension recurrence of F-LP64-HIGH-001 class). F-LP65-MED-001 closure: story §References line 475 BC-2.16.002 citation updated to verbatim H1 "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" (POL-7 D-571 amendment 2 + 6). POL-29 v1.14 sibling-sweep grep evidence per value class: (1) error-taxonomy variant forms — pre-grep 1 (backtick form at line 373 live-narrative) → post-grep 0 in PO-domain across .factory/stories/ + .factory/specs/ + .factory/holdout-scenarios/ (live-narrative; §Changelog rows TD-VSDD-091 exempt); (2) §References BC H1 verbatim — 6 BCs verified verbatim-conformant (BC-2.01.013, BC-2.01.016, BC-2.16.002 [fixed], BC-2.16.004, BC-2.16.011, BC-2.16.012). Note: FB52 v1.30 row reported "post-grep 0" but actual post-grep was 1 (line 373 backtick-quoted form survived); FB53 corrects this residual. OBS-LP65-001 [process-gap] POL-29 v1.15 amendment + variant-form registry routed to state-manager. |
| v1.30 | FB52 | 2026-05-17 | product-owner | F-LP64-HIGH-001 closure (PO scope): error-taxonomy v1.31→v1.32 sibling-sweep at 5 story sites (lines 72, 239, 240, 244, 248); POL-29 v1.13 grep evidence: pre-grep 5 → post-grep 0 in PO-domain. OBS-LP64-001 closure: VP-154 anchor at line 68 corrected from "ADR-027 D4" to "ADR-027 §Verification Property Anchors" (canonical source). |
| v1.29 | FB50 | 2026-05-17 | product-owner | F-LP62-MED-001 closure: §risk_mitigations AC-5 lib.rs re-export removal mechanism repositioned from misplaced AC-7..8 entry to correct AC-4..6 entry. OBS-LP62-002 (PO-domain) closure: 5 story body live-narrative ADR-026 D7 pins (v1.10×3 + v1.16×2) bumped to v1.17 per Interpretation #2 (citations follow latest ADR-026 version per FB6/FB44/FB45 precedent). |
| v1.28 | FB49 | 2026-05-17 | state-manager+product-owner | F-LP61-HIGH-001 closure (state-manager): §Changelog v1.23 row repositioned to strict-descending order per POL-26-COROLLARY bookkeeping repair (6th POL-26 monotonic-ordering recurrence; D-611/D-628/D-635/D-659/D-670 precedent). F-LP61-MED-001 closure (product-owner): §risk_mitigations AC-4..6 entry rewritten Option (a) — Tests 6-7 correctly scoped to retirement absence (type absent + E-SPEC-008 not-constructed-by-live-code); behavioral-equivalence verification deferred to VP-154 (P1, PLUGIN-MIGRATION-001-A scope per ADR-027 D4). |
| v1.27 | FB48 | 2026-05-17 | product-owner | F-LP60-LOW-001 closure: §risk_mitigations AC-7..8 prose disambiguated — "lib.rs re-export removal verified by perimeter-violation compile-fail tests/external/perimeter-violation" rewritten to "...style pattern (style reference: existing `tests/external/perimeter-violation/` crate; VP-155 CustomAdapter perimeter authored at `tests/external/no-hardcoded-sensors/` in PLUGIN-MIGRATION-001-A scope per ADR-027 D3)" — production-grade default Rule 4 Option (a) chosen by orchestrator. |
| v1.26 | FB47 | 2026-05-16 | product-owner | F-LP59-HIGH-001 closure: CAP-029 label corrected to "Config-Driven Sensor Adapters" per capabilities.md canonical source (FB46 §References expansion had wrong label). F-LP59-HIGH-002 closure: risk_mitigations AC-10 entry rewritten to remove phantom Red Gate Test 10 citation (just check is process gate, not Red Gate test); AC-11 entry Red Gate Test 11 → Test 14 (test_BC_2_16_011_e_spec_008_retired_annotation per FB39 renumbering). OBS-LP59-001 closure: AC-9 mitigation appends Red Gate Test 13 citation for stylistic consistency. F-LP59-MED-001 closure (story sites): frontmatter:50 + §References:487 ADR-027 framing label updated to "Same-Burst Removal — Perimeter Enforcement" per ADR-027 v1.8 title. |
| v1.25 | FB46 | 2026-05-16 | product-owner | F-LP58-HIGH-002 closure: HS-003-05 Step 1 + Preconditions canonicalized to public-API call site per AC-9 third-test gate (FB45 hardening). F-LP58-MED-002 closure: §References expansion (BC-2.16.002 + error-taxonomy.md + capabilities). F-LP58-MED-003 closure: risk_mitigations expanded to AC-3b/3c/10/11 (recurrence of OBS-LP54-002). OBS-LP58-001 closure: Task 7d reformatted to numbered convention. POL-23 sibling-sweep ADR-027 v1.7→v1.8 (and BC-2.16.011/VP-154/VP-155 architect-parallel pins) live-narrative updates. |
| v1.24 | FB45 | 2026-05-16 | product-owner | F-LP57-MED-001 closure: append Task 7d (Cargo.toml `tracing-test = "0.2"` dev-dep wiring per architect Option α); AC-9 third-test "or equivalent fixture" replaced with verbatim "tracing-test = \"0.2\" subscriber fixture" spec. F-LP57-HIGH-002 sibling-sweep: subsystems frontmatter [+SS-22 Process Lifecycle] mirrors ADR-026 v1.16 subsystems_affected. POL-23 sibling-sweep version-pin updates: ADR-026 v1.15→v1.16; BC-2.16.012 v1.17→v1.18; VP-156 v0.9→v0.10; ADR-022 v1.3→v1.4. |
| v1.23 | FB44 | 2026-05-16 | product-owner | Architecture Compliance Rule + Task 7b + AC-9 third-test + crates_touched updated per F-LP56-HIGH-001 architect Option A adjudication (ADR-026 D7 v1.15 + BC-2.16.012 v1.17 + VP-156 v0.9). prism-bin added to crates_touched. Single designated boot.rs insertion point: `prism_query::invalidation::mark_query_phase_started();` first statement of step-8 before `QueryEngine::new()`. |
| v1.22 | FB40 | 2026-05-16 | product-owner | F-LP50-MED-001 AC-3b/AC-3c/AC-11 trace anchors corrected from phantom-anchor "§Postconditions P-NN" syntax to canonical "§Error Cases E-SPEC-NNN" form per POL-21 + POL-22 Phase A. AC-3b: `§Postconditions P4 Rule 2/B` → `§Error Cases E-SPEC-013`; AC-3c: `§Postconditions P4 Rule 2/C` → `§Error Cases E-SPEC-014`; AC-11: `§Postconditions P7` → `§Error Cases E-SPEC-008 (retired)`. Same correction applied to Red Gate test 4 + test 5 post-implementation assertion text (same phantom-anchor pattern). ADR-023 cite form aligned: `ADR-023 Rule 2` → `ADR-023 §Architectural Constraints Rule 2, Rule B/C` (full heading anchor). AC-11 ADR-027 cite appended with `§Decision`. FB39-introduced defect closed. |
| v1.21 | FB39 | 2026-05-16 | product-owner | F-LP49-HIGH-001 PO-domain sites — AC-3 narrative + trace lines updated error-taxonomy v1.30→v1.31 (13th+ POL-23 recurrence closure). F-LP49-MED-001 BC-2.01.016 Rule 2/B+2/C AC coverage gap closed via new AC-3b (E-SPEC-013; test `test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected`) + AC-3c (E-SPEC-014; test `test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected`) + 2 new Red Gate tests (tests 4+5; former BC-2.16.011 tests 4+5 renumbered to 6+7; BC-2.16.012 tests 6–11 renumbered to 8–13); `acceptance_criteria_count: 10→13`; `red_gate_tests: 11→14`. F-LP49-MED-002 BC-2.16.011 P7 E-SPEC-008 retirement annotation AC gap closed via new AC-11 + Red Gate test 14 (`test_BC_2_16_011_e_spec_008_retired_annotation`). F-LP49-MED-003 BC-2.16.012 P6 tracing event field schema AC gap closed via AC-9 extension asserting tracing-test capture of `write_tool_registration_after_boot` WARN event fields; Red Gate test 13 (renumbered from 11) updated to assert tracing-event field capture. F-LP49-MED-004 ADR-022 added to §References Architecture Compliance subsection (`§B step 7.5/8 ordering authoritative for Task 7b AtomicBool flag set-time`). F-LP49-LOW-001 §References gains new Holdout Scenarios subsection enumerating HS-PREREQ-E-001/002/003 with relative links. Green Gate DoD item 5 updated: 10→13 ACs. |
| v1.20 | FB38 | 2026-05-16 | product-owner | F-LP48-MED-001: §Error Taxonomy Additions E-PLUGIN-020 description corrected from retired "called after boot step 8 completes" to canonical "called after query-engine init starts at step 8 (per ADR-026 §D7); window closes at step 8 start (first act of step 8, before QueryEngine construction proceeds)" per FB37 architect adjudication. F-LP48-MED-003: §File Structure Requirements gains new row for `crates/prism-spec-engine/src/plugin/mod.rs` (or `loader.rs`) wiring PluginRuntime per-plugin write-tool registration during step 7.5 per ADR-026 §D7 + ADR-022 §B step 7.5. §Token Budget gains matching row (~150 tokens); total updated ~17,450 → ~17,600. POL-23 pin bump: §Error Taxonomy Additions intro cite `error-taxonomy.md v1.30` → `v1.31` (E-PLUGIN-020 was amended in v1.31). |
| v1.19 | FB37 | 2026-05-16 | product-owner | F-LP47-LOW-001 frontmatter: ADR-022 added to `architectural_decisions` (§B step 7.5/8 ordering authoritative for Task 7b AtomicBool flag set-time); SS-17 added to `subsystems` and `anchor_subsystem` (both fields) per architect adjudication. F-LP47-MED-001 Task 7b/7c TD-VSDD-091 volatile line-number cites replaced with durable semantic anchors ("error-taxonomy.md E-PLUGIN-020" without line 467; "BC-2.16.012 EC-016-012-005" without line 109). F-LP47-MED-003 §FSR + §Token Budget swept for Task 7b/7c new content: invalidation.rs row expanded to enumerate AtomicBool flag + `mark_query_phase_started()` function; `error.rs` row added for `WriteToolRegistrationAfterBoot` variant (~50 tokens); invalidation.rs budget updated ~600 → ~700; total updated ~17,300 → ~17,450. F-LP47-MED-004 Task 7b emission form corrected to canonical `event_type` idiom per BC-2.16.012:84 + CLAUDE.md Conventions (`event_type` as first structured field, not trailing static message). |
| v1.18 | FB36 | 2026-05-16 | product-owner | F-LP46-MED-001 §Tasks expanded to enumerate ADR-026 D7 runtime_deliverables not previously covered: new Task 7b adds AtomicBool query-phase flag (`QUERY_PHASE_STARTED`) + `mark_query_phase_started()` + fail-closed post-boot check in `register_write_tool`; new Task 7c adds `SpecEngineError::WriteToolRegistrationAfterBoot` unit variant. Mirrors FB34 Task 1b coverage discipline for D7 dimension. Anchors: ADR-026 §D7 runtime_deliverables items 6+5, error-taxonomy.md E-PLUGIN-020, BC-2.16.012 EC-016-012-005. |
| v1.17 | FB35 | 2026-05-16 | product-owner | F-LP45-MED-001 Task 1b epilogue volatile + factually-wrong line-range cite "(rows 343–346)" replaced with durable semantic anchor enumerating 4 file names (crowdstrike.rs / cyberint.rs / claroty.rs / armis.rs). TD-VSDD-091 compliance + factual correction. F-LP45-LOW-001 changelog cite "runtime_deliverables 22-23" adjudicated ACCEPTABLE per TD-VSDD-091 §Changelog exception (no fix dispatched). |
| v1.16 | FB34 | 2026-05-16 | product-owner | F-LP44-MED-001 §Tasks expanded to enumerate ADR-026 D1/D2 Path B auth_type_name trait surface gain + 4 impl method body additions (new Task 1b inserted between Task 1 and Task 2); Task 1 Step 3 verification claim "compile without modification" corrected — impls WILL be modified per ADR-026 D2 Path B runtime_deliverables 22-23. |
| v1.15 | FB30 | 2026-05-16 | product-owner | F-LP38-MED-001 Task 7 "explicitly forbidden" overstrong claim replaced with rationale-based language matching ADR-026 §D7 actual text (POL-22 Phase C named-entity verification; CLAUDE.md precedence rule #2 — ADR supersedes story on contract semantics); F-LP38-LOW-001 volatile line-range citation removed (TD-VSDD-091 — §D7 semantic anchor durable, line numbers decay). |
| v1.14 | FB29 | 2026-05-16 | product-owner | F-LP37-MED-001 — AC-8 test-name reference updated: singular non-existent name replaced with explicit enumeration of 4 canonical Red Gate test names (`test_BC_2_16_012_002_spec_parser_behavioral_equivalence_{crowdstrike,cyberint,claroty,armis}`), closing within-FB28 sibling-sweep gap. F-LP37-MED-002 — Task 7 OnceLock<RwLock<...>> alternative stricken; ADR-026 §D7 explicit forbiddance cited with line range (246-259); precedence rule #2 (ADR supersedes story on contract semantics) applied. Note: `_NNN_` segments in Red Gate test names (e.g., `_002_`, `_003_`) are intra-story Red Gate test-set grouping numbers — NOT identifiers in BC-2.16.012's Canonical Test Vectors (TV-001..004), Edge Cases, or Invariants body sections. |
| v1.13 | FB28 | 2026-05-16 | product-owner | F-LP36-MED-001 — AC-9 test-name canonicalized: added `_003_` segment to match Red Gate Test 11 convention (`test_BC_2_16_012_write_tool_invalidation_runtime_register` → `test_BC_2_16_012_003_write_tool_invalidation_runtime_register`). F-LP36-MED-002 — Red Gate Tests expanded to cover 4-sensor breadth per AC-8 Option A: added Cyberint (Test 8), Claroty (Test 9), Armis (Test 10) behavioral-equivalence rows (all `_002_` group, mirroring Test 7); former Test 8 renumbered to Test 11; `red_gate_tests: 8 → 11`. State-manager closes F-LP36-MED-003 + bookkeeping in same burst. |
| v1.12 | fix-burst-22-combined-D-634 | 2026-05-16 | state-manager | F-LP27-MED-001 — 11th manifestation version-pin-drift family at NEW target (error-taxonomy.md itself): 3 story sites swept `v1.27` → `v1.30` (AC-3 narrative line 207, AC-3 trace line 208, §Error Taxonomy Additions intro line 317). 4-bump window (v1.27→v1.28→v1.29→v1.30) where these sites were not swept during FB2..FB21. Pass-27 BLOCKED 1 MED; streak RESET 2/3 → 0/3 (4th reset). Pass-26→pass-27 reset BROKE the convergence pattern. Pass-28 NEXT — first of NEW 3-CLEAN sequence (4th attempt). |
| v1.11 | prereq-e-fix-burst-16 | 2026-05-16 | product-owner | F-LP17-MED-001 — 8th manifestation BC-2.16.002 citation defect family closed at NEW dimension (phrasing-form canonicalization): 3 story sites converted from no-parens form `§Postconditions Canonical Structured Event Catalog v1.20 row 33` to canonical parens-ancestry form `§Postconditions (Canonical Structured Event Catalog bullet, v1.20) row 33` (matching workspace pattern at BC-2.16.012:84/109 + error-taxonomy:467/473). FB12-era inherited inconsistency closed (4 successive bursts FB12/FB14/FB15 addressed only pin dimension; pass-17 fresh-context surfaced phrasing-form dimension). POL-25 multi-cite propagation discipline applied with explicit grep enumeration. |
| v1.10 | prereq-e-fix-burst-15 | 2026-05-16 | product-owner | F-LP16-HIGH-001 (7th OCCURRENCE POL-23 RECURRING class) — 3 variant-phrasing sites swept v1.19→v1.20: Task 7 + AC-9 + §File Structure Requirements (all using no-parens form `Canonical Structured Event Catalog v1.19 row 33`). FB14 canonical-form sweep missed these. POL-25 explicit variant-phrasing grep mandate applied this burst to prevent 8th occurrence. |
| v1.9 | prereq-e-fix-burst-12 | 2026-05-16 | product-owner | F-LP13-HIGH-003 Option A propagation — Task 7 updated: WriteToolInvalidationMap struct field enumeration gains `plugin_name: String` field (set by PluginRuntime from plugin manifest `name` per ADR-026 D7 v1.10; BC-2.16.002 §Postconditions Canonical Structured Event Catalog v1.19 row 33). AC-9 updated: struct plugin_name field and its role as the structured event field source documented. §File Structure Requirements invalidation.rs row updated with plugin_name field enumeration. |
| v1.8 | prereq-e-fix-burst-7 | 2026-05-16 | product-owner | F-LP7-HIGH-002 + F-LP7-MED-004 — implementer-facing sibling-sweep: (1) Task 8 expanded to enumerate all four BC-2.16.004 frontmatter mutations (deprecated_by: ADR-023 → ADR-027; removed date; removal_reason advanced; lifecycle_status: deprecated → removed); (2) AC-6 acceptance criteria updated to verify all four field states; (3) §References ADR-027 VP-154 anchor §D5 → §Verification Property Anchors (FB4 D5 scope expansion sibling-sweep miss). TD-VSDD-059 paper-fix detection. |
| v1.7 | prereq-e-fix-burst-6 | 2026-05-16 | story-writer | F-LP6-CRIT-001 propagation: §File Structure Requirements + §AC-2 example chain — Claroty auth_type_name() return value `cookie` → `cookie_roundtrip` to match ADR-026 v1.8 D2 corrected value + D3 canonical enumerated set + E-SPEC-012 + VP-153 Rule A. Sibling sweep verified all four built-in auth_type_name() values match D3 enumerated set. |
| v1.6 | prereq-e-fix-burst-5 | 2026-05-15 | product-owner | F-LP5-HIGH-001: `subsystems:` updated from `[SS-01, SS-16]` to `[SS-01, SS-07, SS-16]`; `anchor_subsystem:` updated identically (prism-query → SS-07 Adapter Pagination & Response Cache per ARCH-INDEX). F-LP5-HIGH-002: §References 5 BC entries corrected to H1-verbatim titles (POL-7 D-571 sweep); lifecycle/status annotations moved outside link text to description suffix. POL-7 5-surface sweep: surface 1 BC table — BC-2.01.016 Role cell "four built-in auth impls unchanged" corrected to "each add one new method body (`auth_type_name`)" (stale from pre-v1.4). Surfaces 3/4/5 verified clean. F-LP5-MED-001: File Structure Requirements table gains 4 auth impl files (crowdstrike/cyberint/claroty/armis `.rs`), each with `auth_type_name` method action + ADR-026 D1 Path B source; Token Budget Estimate table gains 4 impl file rows (~50 tokens each); Total updated ~17,100 → ~17,300. F-LP5-MED-002: Architecture Compliance Rules table — hardcoded-sensor-string dispatch rule Source column gains `ADR-027 D5` (canonical anchor post-FB4 expansion). F-LP5-MED-003 (Path B chosen): AC-1 trace-line extended with BC-2.01.013 justification (mechanical un-sealing delivery of PREREQ-F amendment); AC-6 trace-line extended with BC-2.16.004 justification (lifecycle-close execution per DF-030). Both BCs in `behavioral_contracts:` frontmatter now have AC traces. |
| v1.5 | prereq-e-fix-burst-4 | 2026-05-15 | product-owner | F-LP4-HIGH-001: VP-156 added to `verification_properties:` frontmatter array (after VP-155, before VP-PLUGIN-001). F-LP4-HIGH-002: VP-156 added to §References Architecture Compliance section with markdown link + description matching "uniqueness only" framing. F-LP4-HIGH-003: BC-2.16.004 BC table Title cell updated to H1-verbatim ("Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient"); lifecycle annotation "(deprecated → removed)" moved inline to Role column. `anchor_vps:` frontmatter updated to include VP-156. |
| v1.4 | prereq-e-fix-burst-3 | 2026-05-15 | product-owner | F-LP3-HIGH-002 (joint with architect): AC-2 rewritten — "ZERO changes" → "ONE NEW METHOD BODY per impl (fn auth_type_name returning static auth-type name); no other changes"; Red Gate Test 3 renamed `_unchanged_` → `_minimal_diff_` + updated to assert one new method body. F-LP3-HIGH-003: Task 7 register_write_tool signature updated to `-> Result<(), SpecEngineError>`; AC-9 updated with error-path assertions (E-PLUGIN-012 duplicate, E-PLUGIN-020 after-boot); Red Gate Test 8 updated to assert all three paths. F-LP3-MED-002: Error Taxonomy Additions table expanded — E-PLUGIN-012 + E-PLUGIN-020 rows added (taxonomy v1.27); v1.25 version pins in AC-3 body updated to v1.27 (2 sites in story body). Sub-fix (same burst, D-577): `risk_mitigations:` AC-1..3 entry synced to Path B — "four built-in auth impls are unchanged" → "four built-in auth impls require ONE NEW METHOD BODY each (one-line `fn auth_type_name` returning the static auth-type name string); no other changes". |
| v1.3 | prereq-e-fix-burst-2 | 2026-05-15 | product-owner | F-LP2-MED-004 closure: Red Gate Test Set reordered so tests are grouped contiguously by BC (BC-2.01.016: tests 1–3; BC-2.16.011: tests 4–5; BC-2.16.012: tests 6–8). No test name changes — `_NNN` suffixes within each BC group were already sequential. Added BC-labeled subheadings for navigability. Former interleaved order (tests 1,2,7 then 3,8 then 4,5,6 across BCs) broke the BC-grouped reading pattern. |
| v1.2 | S-PLUGIN-PREREQ-E-fix-burst-1 | 2026-05-15 | product-owner | F-LP1-HIGH-001: BC-2.01.016 §Preconditions method surface updated per ADR-026 D1 (2-method trait: `as_any()` + `auth_type_name()`). F-LP1-HIGH-003: All 8 §C5 phantom-heading citations in story body corrected to `§Architectural Constraints (C5 bullet[, Rule N])` per POL-21. F-LP1-MED-001: E-SPEC-008 retirement framing updated — action now `RETIRED … error-taxonomy.md v1.26` (path (a) chosen: PO delivers retirement annotation in v1.26 spec-burst, not deferred to implementer). F-LP1-MED-004: All ~9 `TD-A-003` alias occurrences replaced with canonical `TD-S-PLUGIN-PREREQ-A-003` (frontmatter comment, assumption_validations, risk_mitigations, BC table, Task 7, AC-9, Green Gate DoD 8, Previous Story Intelligence, References). F-LP1-MED-005: Red Gate test 2 rewritten to standard Red Gate semantics (pre/post-implementation assertion states explicit). |
| v1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Cross-domain reconciliation with architect's parallel ADR-026/027/VP-153/154/155. Q1: authored E-SPEC-012/013/014; corrected BC-2.01.016 error code references (E-SPEC-010/011/012 → E-SPEC-012/013/014). Q2: pre-staged, waiting for architect framing choice (no action). Q3: BC-2.16.011 §VP-154 Fixture Acceptance Criterion added (OCSF Detection Finding 2004 schema, semantic-equality behavioral equivalence definition). Q4: HS-PREREQ-E-001 +VP-153 sub-scenario; HS-PREREQ-E-002 +VP-154/VP-155 sub-scenarios; HS-PREREQ-E-003 VP-155 note. Q5: story frontmatter updated (verification_properties: VP-153/154/155; architectural_decisions: ADR-026/027/ADR-023; holdout_scenarios: HS-PREREQ-E-001/002/003; anchor_vps updated). AC-3 error code corrected to E-SPEC-012. Error Taxonomy Additions section updated to list 3 new codes + E-SPEC-008 retirement. References updated with ADR-026/027/VP-153/154/155 links. |
| v1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Authored from ADR-023 §Architectural Constraints (C5 bullet) scope + PREREQ-A/D context. Three new BCs (BC-2.01.016, BC-2.16.011, BC-2.16.012), three holdout scenarios (HS-PREREQ-E-001/002/003), 10 ACs, 8 Red Gate tests, TD-S-PLUGIN-PREREQ-A-003 closure in scope. |
