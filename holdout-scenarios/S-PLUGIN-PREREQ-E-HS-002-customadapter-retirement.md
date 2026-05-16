---
document_type: holdout-scenario
level: L3
id: "HS-PREREQ-E-002"
title: "CustomAdapter Rust Trait Retirement — No Behavioral Regression on Four Initial Sensors"
category: "plugin-migration"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
story_source: "S-PLUGIN-PREREQ-E"
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-15T00:00:00
phase: 4
inputs: []
input-hash: null
traces_to: "BC-2.16.011"
behavioral_contracts:
  - BC-2.16.011
  - BC-2.16.004
verification_properties:
  - VP-154
  - VP-155
lifecycle_status: active
introduced: S-PLUGIN-PREREQ-E
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# HS-PREREQ-E-002: CustomAdapter Rust Trait Retirement — No Behavioral Regression on Four Initial Sensors

**Story:** S-PLUGIN-PREREQ-E
**Must Pass:** YES (P0 — gates PLUGIN-MIGRATION-001-C Wave 1 stories that depend on clean spec-engine surface)
**BC Traced:** BC-2.16.011 (retirement contract)

---

## Scenario Description

The `CustomAdapter` Rust trait, `CustomAdapterRegistry`, and `CustomAuth` struct are deleted
from `prism-spec-engine`. This scenario verifies that (a) no `CustomAdapter`-related symbol
exists in the compiled workspace after deletion, (b) the four initial sensors (which never
used `CustomAdapter` per invariant in the deprecated BC-2.16.004) continue to produce
identical query results, and (c) the `bc_2_16_004_test.rs` file is absent and its absence
does not cause build failures.

---

## HS-PREREQ-E-002-01: Known-Good Corpus — Workspace Builds and Tests Green After Deletion

**Title:** `cargo build --workspace --all-features` and `just check` pass after CustomAdapter deletion

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged to `develop`
- `custom_adapter.rs` is deleted
- The three call sites in `lib.rs`, `examples/demo_spec_loading.rs`, and `tests/bc_2_16_004_test.rs` are also removed/cleaned

**Steps:**

1. Run `cargo build --workspace --all-features` from workspace root
2. Run `just check` (fmt + clippy + nextest + doctests + crate-layout)
3. Run `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` — expect zero matches

**Expected Outcome:**

- `cargo build --workspace --all-features` exits 0, zero errors
- `just check` exits 0; zero clippy warnings; all tests green
- Grep for `CustomAdapter\|CustomAdapterRegistry\|CustomAuth` returns zero matches in `src/` and `tests/` paths
- The error code `E-SPEC-008` appears ONLY in `error-taxonomy.md` with a `retired` annotation; it does NOT appear in any live match arm

**Repos Tested:** prism-spec-engine, prism-sensors, prism-query, prism-bin

---

## HS-PREREQ-E-002-02: Known-Problematic Corpus — The Deleted bc_2_16_004_test.rs Is Genuinely Absent

**Title:** `cargo nextest run -p prism-spec-engine` does not reference bc_2_16_004_test

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` is deleted

**Steps:**

1. Run `cargo nextest run -p prism-spec-engine --no-fail-fast`
2. Check nextest output for any test referencing `bc_2_16_004` or `custom_adapter` in test names
3. Count the `prism-spec-engine` test count before and after merger (reference: count from develop before PREREQ-E)

**Expected Outcome:**

- Zero tests with `bc_2_16_004` or `custom_adapter` in their names appear in nextest output
- The test count for `prism-spec-engine` is lower than pre-PREREQ-E by exactly the number of tests that existed in `bc_2_16_004_test.rs` (implementer records this count in the PR description)
- No "test not found" or "file not found" build errors occur because of the deletion

**Repos Tested:** prism-spec-engine

---

## HS-PREREQ-E-002-03: Behavioral Regression Guard — CrowdStrike Query Output Unchanged

**Title:** CrowdStrike sensor adapter query output is byte-identical before and after CustomAdapter deletion

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- A DTU clone fixture for CrowdStrike is available (per PLUGIN-MIGRATION-001 test infrastructure)
- Pre-PREREQ-E snapshot of CrowdStrike query output is available as a fixture (or generated against `develop@<pre-merge SHA>`)

**Steps:**

1. Run the CrowdStrike sensor integration test against the DTU clone fixture
2. Capture the `SensorSpec` parsed from `crowdstrike.sensor.toml` and the normalized OCSF records produced
3. Compare against the pre-PREREQ-E snapshot

**Expected Outcome:**

- `SensorSpec` structs are byte-identical (same auth_type, same tables, same columns)
- OCSF record output against the DTU fixture is byte-identical (or within TS-PLUGIN-PARITY-001 canonicalization tolerance if DTU non-determinism applies)
- No `CustomAdapter` code path was involved in the pre-PREREQ-E path (confirmed by BC-2.16.004 invariant: four initial sensors never used escape hatch); this test confirms that absence

**Repos Tested:** prism-spec-engine, prism-sensors, prism-dtu-crowdstrike

---

---

## HS-PREREQ-E-002-04: VP-154 Coverage — WASM Dispatch Behavioral Equivalence (P1; PLUGIN-MIGRATION-001-A scope)

**Title:** WASM plugin override path returns non-empty, semantically-equivalent OCSF records (VP-154 integration test)

**Note:** VP-154 is P1 and its full harness is authored in PLUGIN-MIGRATION-001-A scope (requires PREREQ-B + PREREQ-D both merged). This sub-scenario is a holdout-evaluation checkpoint: the evaluator confirms VP-154 passes before Wave 1/A closes, per the VP's lifecycle constraint.

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged (PREREQ-E prerequisite)
- S-PLUGIN-PREREQ-B is merged (`PipelineExecutor::execute` is real, not stubbed)
- S-PLUGIN-PREREQ-D is merged (`PluginRuntime::load_all_plugins` is wired in boot)
- VP-154 integration test harness has been authored and the minimal WASM fixture (`.prx`) exists at `crates/prism-spec-engine/tests/fixtures/minimal_sensor_fetch.prx` or equivalent

**Steps:**

1. Run `cargo nextest run -p prism-spec-engine -E 'test(vp154)'`
2. Confirm `wasm_plugin_override_returns_expected_records` passes with non-empty records
3. Confirm returned record matches the canonical OCSF schema defined in BC-2.16.011 §VP-154 Fixture Acceptance Criterion:
   - `records[0]["finding_info"]["uid"]` == `"test-001"`
   - `records[0]["class_uid"]` == `2004`
   - `records[0]["severity_id"]` is a valid OCSF integer (1–5 or 99)
4. Confirm `wasm_plugin_absent_falls_through_to_toml_pipeline` passes (TOML path is not broken)

**Expected Outcome:**

- VP-154 integration test exits 0 with PASSED
- WASM plugin override path is confirmed non-empty and non-panicking (behavioral equivalence to deleted `CustomAdapter::override_fetch returning Some(records)`)
- TOML fallthrough path is unaffected (no regression from CustomAdapter deletion)
- Records conform to OCSF Detection Finding 2004 schema per BC-2.16.011 §VP-154 Fixture Acceptance Criterion

**Repos Tested:** prism-spec-engine (PipelineExecutor + PluginRuntime integration)

**VP Traced:** VP-154

---

## HS-PREREQ-E-002-05: VP-155 Coverage — Compile-Fail Perimeter Confirms CustomAdapter Absence (P0; PLUGIN-MIGRATION-001-A scope)

**Title:** `prism_spec_engine::CustomAdapter` and `prism_spec_engine::CustomAdapterRegistry` produce `error[E0432]`

**Note:** VP-155 compile-fail files are authored in PLUGIN-MIGRATION-001-A scope and MUST be added AFTER PREREQ-E merges. This sub-scenario verifies that VP-155 is correctly sequenced and active.

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged (types are deleted)
- PLUGIN-MIGRATION-001-A has added `import_custom_adapter.rs` and `import_custom_adapter_registry.rs` to `tests/external/no-hardcoded-sensors/`
- CI count assertion has been updated to reflect 11 (was 9) entries in the FORBIDDEN-SYMBOLS-001 catalog

**Steps:**

1. Confirm `tests/external/no-hardcoded-sensors/import_custom_adapter.rs` exists and contains `use prism_spec_engine::CustomAdapter; //~ ERROR unresolved import`
2. Confirm `tests/external/no-hardcoded-sensors/import_custom_adapter_registry.rs` exists and contains `use prism_spec_engine::CustomAdapterRegistry; //~ ERROR unresolved import`
3. Run the compile-fail test suite: `cargo test -p tests-external-no-hardcoded-sensors` (or equivalent)
4. Confirm CI count assertion passes with `CATALOG_SIZE=11`

**Expected Outcome:**

- Both compile-fail files exist and produce `E0432` on `cargo build`
- CI count assertion passes (11 entries, up from 9)
- `prism-spec-engine` public API is confirmed clean of `CustomAdapter` and `CustomAdapterRegistry`

**Repos Tested:** tests/external/no-hardcoded-sensors/ (perimeter crate)

**VP Traced:** VP-155

---

## Validation Evidence Required

When this holdout scenario is evaluated, the evaluator must produce:

1. `just check` exit-0 log (full workspace)
2. `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` output (must be empty for src/ and tests/)
3. `cargo nextest run -p prism-spec-engine` output showing zero `bc_2_16_004` test names
4. CrowdStrike behavioral comparison log (byte-identical or within canonicalization tolerance)
5. VP-154 integration test run log (`cargo nextest run -p prism-spec-engine -E 'test(vp154)'` — must be PASSED; evaluated in PLUGIN-MIGRATION-001-A scope before Wave 1/A closes)
6. VP-155 compile-fail evidence (both perimeter files exist; count assertion `CATALOG_SIZE=11` passes)

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q4 alignment: Added HS-PREREQ-E-002-04 (VP-154 WASM behavioral equivalence coverage — P1, PLUGIN-MIGRATION-001-A scope) and HS-PREREQ-E-002-05 (VP-155 compile-fail perimeter confirmation — P0, PLUGIN-MIGRATION-001-A scope). Both reference the BC-2.16.011 §VP-154 Fixture Acceptance Criterion for canonical OCSF record schema. Updated Validation Evidence to include VP-154 and VP-155 evidence items. Added `verification_properties: [VP-154, VP-155]` to frontmatter. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Three sub-scenarios covering build regression, test-file deletion confirmation, and behavioral parity guard for CrowdStrike. |
