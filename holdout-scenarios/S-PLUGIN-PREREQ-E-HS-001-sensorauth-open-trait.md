---
document_type: holdout-scenario
level: L3
id: "HS-PREREQ-E-001"
title: "SensorAuth Open Trait — External Implementation Compiles and Loads"
category: "plugin-migration"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
story_source: "S-PLUGIN-PREREQ-E"
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-15T00:00:00
phase: 4
inputs: []
input-hash: null
traces_to: "BC-2.01.016"
behavioral_contracts:
  - BC-2.01.016
  - BC-2.01.013
verification_properties:
  - VP-153
lifecycle_status: active
introduced: S-PLUGIN-PREREQ-E
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# HS-PREREQ-E-001: SensorAuth Open Trait — External Implementation Compiles and Loads

**Story:** S-PLUGIN-PREREQ-E
**Must Pass:** YES (P0 — gates PLUGIN-MIGRATION-001-A Wave 1 dispatch)
**BC Traced:** BC-2.01.016

---

## Scenario Description

After PREREQ-E merges, an external Rust crate (simulated in a compile-pass test) should be able
to implement `SensorAuth` without any `Sealed` marker workaround. This scenario verifies that
(a) the sealed marker is gone, (b) a new `SensorAuth` implementation compiles without errors,
and (c) the three ADR-023 Rule 2 runtime cross-sensor auth-composition rules still reject
malformed specs.

---

## HS-PREREQ-E-001-01: Sealed Marker Is Gone — External Impl Compiles

**Title:** Rust code outside `prism-sensors` implements `SensorAuth` without private marker

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged to `develop`
- `prism-sensors` is compiled as a dependency
- A new test crate or integration test file is authored that imports `SensorAuth` from `prism_sensors::auth`
- The `private` module (if it exists) no longer exports a `Sealed` marker trait

**Steps:**

1. Author a minimal struct `struct TestAuth;` in a test file or a `tests/external/sensor-auth-open/src/main.rs` crate
2. Implement `SensorAuth` for `TestAuth` — implement all required methods with stub bodies returning sensible defaults
3. `cargo build` the test crate with `prism-sensors` as a dependency
4. Verify that the build succeeds without "the trait `auth::private::Sealed` is not implemented" or equivalent sealed-trait error

**Expected Outcome:**

- Build succeeds
- No compiler error referencing `private::Sealed` or `Sealed` marker
- `dyn SensorAuth` is usable as a trait object bound; `TestAuth` can be boxed as `Box<dyn SensorAuth>`
- `grep -rn "private::Sealed\|impl Sealed\|trait Sealed" crates/prism-sensors/src/auth/` returns zero matches

**Repos Tested:** prism-sensors (auth module)

---

## HS-PREREQ-E-001-02: Runtime Auth-Composition Rejection Replaces Compile-Time Sealing

**Title:** SensorSpec with mixed auth types is rejected at spec-load, not compile time

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- `prism-spec-engine` spec validation is active (`BC-2.16.001` enforcement)
- A fixture TOML spec file is prepared with `auth_type = ["oauth2_client_credentials", "bearer_static"]` (array value)

**Steps:**

1. Load the malformed fixture TOML spec via `prism start --spec-dir <fixture_dir>` or via the `add_sensor_spec` MCP tool
2. Observe the spec-load error response

**Expected Outcome:**

- Spec load is rejected with `E-SPEC-012` (ADR-023 Rule 2, Rule A — auth_type must be single value; see error-taxonomy.md v1.27)
- Error message cites "auth_type must be a single value" or equivalent
- Process does NOT fail to compile (no compile-time sealed-trait error — the rejection is runtime)
- Other valid specs in the same directory continue to load (N-1 survivor rule)

**Repos Tested:** prism-spec-engine (spec_parser, validation), prism-sensors (auth module)

---

## HS-PREREQ-E-001-03: Known-Good Corpus — Four Built-In Sensors Load After Unsealing

**Title:** CrowdStrike, Cyberint, Claroty, Armis TOML specs load and register tables after SensorAuth unsealing

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- The four built-in sensor TOML spec files are present in the spec directory
- No `CustomAuth` type is referenced in `lib.rs`, `custom_adapter.rs`, or any production source

**Steps:**

1. Start `prism` with the default four sensor specs
2. Verify `list_sensor_specs` MCP tool lists all four sensors with `status: available`
3. Query `SHOW TABLES IN crowdstrike` — expect CrowdStrike tables to be listed
4. Run `cargo nextest run -p prism-sensors` — all sensor auth tests pass

**Expected Outcome:**

- All four sensors load; no regression from `SensorAuth` unsealing
- `CustomAuth` is absent from the codebase; no import errors or `undefined type` panics
- The four built-in auth implementations (`CrowdStrikeAuth`, `CyberintAuth`, etc.) are behaviorally unchanged

**Repos Tested:** prism-sensors, prism-spec-engine, prism-bin

---

---

## HS-PREREQ-E-001-04: VP-153 Coverage — Proptest Cross-Composition Prevention (Verification Alignment)

**Title:** All invalid (auth_type, credential_type) pairs are rejected; no error message leaks credential values (VP-153 proptest)

**Note:** This sub-scenario is the holdout-level companion to VP-153 (SensorAuth Runtime Cross-Composition Prevention proptest). The VP-153 harness covers this computationally via proptest; this sub-scenario records the evaluator's manual verification of the same property for the holdout record.

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- VP-153 proptest harness has been authored by test-writer and passes in CI
- The proptest covers all 5 × 5 (auth_type, credential_type) pairs

**Steps:**

1. Confirm `cargo nextest run -p prism-spec-engine -E 'test(vp153)'` exits 0
2. Confirm the proptest result log shows `PASSED` for both `valid_auth_type_credential_pairs_accepted` and `mismatched_auth_type_credential_rejected`
3. Spot-check: construct a spec with `auth_type = "oauth2_client_credentials"` and a `MockCredentialType::BearerToken` credential; confirm E-SPEC-014 is returned
4. Confirm no error message in the rejection path contains a credential value string (AD-017 compliance)

**Expected Outcome:**

- VP-153 proptest passes in CI (all valid pairs accepted, all invalid pairs rejected)
- E-SPEC-012 is returned for multi-valued or out-of-set auth_type
- E-SPEC-013 is returned for multiple credential_refs
- E-SPEC-014 is returned for auth_type/credential_type mismatch
- No error message contains a credential value (AD-017 AI-opaque credential model enforced)

**Repos Tested:** prism-spec-engine (spec_parser/validation pass)

**VP Traced:** VP-153

---

## Validation Evidence Required

When this holdout scenario is evaluated, the evaluator must produce:

1. Build log showing `cargo build -p prism-sensors` succeeds with zero `Sealed`-related errors
2. `grep -rn "private::Sealed"` output from `crates/prism-sensors/src/auth/` (must be empty)
3. Spec-load test log for the malformed `auth_type = [...]` fixture (must show E-SPEC-012 rejection, not E-SPEC-010)
4. `cargo nextest run -p prism-sensors` run log (must be all-green)
5. VP-153 proptest run log (`cargo nextest run -p prism-spec-engine -E 'test(vp153)'` — must be PASSED)

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | prereq-e-fix-burst-3 | 2026-05-15 | product-owner | F-LP3-MED-002 (POL-25 propagation): stale error-taxonomy.md v1.25 reference in HS-PREREQ-E-001-02 expected outcome updated to v1.27. Changelog reference to v1.25 in v1.1 row preserved as historical record per TD-VSDD-091 anti-volatile-pin. |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q4 alignment: Added HS-PREREQ-E-001-04 sub-scenario covering VP-153 proptest coverage (cross-composition prevention). Updated sub-scenario HS-PREREQ-E-001-02 validation evidence item 3 to cite E-SPEC-012 (not E-SPEC-010 — correct code per error-taxonomy v1.25). Added `verification_properties: [VP-153]` to frontmatter. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Three sub-scenarios covering sealed-marker removal, runtime auth-composition rejection, and regression guard for four built-in sensors. |
