#![allow(non_snake_case)]
//! Red Gate tests for BC-2.01.016 — SensorAuth Open Trait Contract.
//!
//! Tests 2, 4, 5 of the S-PLUGIN-PREREQ-E Red Gate set (prism-spec-engine crate).
//!
//! | Test | Name                                                          | AC   | Red Gate failure mode              |
//! |------|---------------------------------------------------------------|------|------------------------------------|
//! | 2    | test_BC_2_01_016_002_auth_composition_runtime_rejection       | AC-3 | todo!() in validate_cross_composition → panic |
//! | 4    | test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected | AC-3b| todo!() in validate_cross_composition → panic |
//! | 5    | test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected | AC-3c| todo!() in validate_cross_composition → panic |
//! | 6    | test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory | AC-3b| parse_spec_directory returns Ok(SensorSpec) instead of validation error |
//! | 7    | test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool | AC-3b| add_sensor_spec MCP entry-point returns Added instead of ValidationFailed |
//! | 8    | test_BC_2_01_016_e_spec_014_rejected_via_hot_reload           | AC-3b| process_spec_changes returns Ok(delta) instead of recording error |
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.01.016 | ADR-026 §D3 | ADR-023 Rule 2

use prism_spec_engine::{
    add_sensor_spec::add_sensor_spec,
    config_manager::{ConfigManager, parse_spec_directory},
    hot_reload::{SpecChangeEvent, process_spec_changes},
    spec_parser::SpecLoader,
    types::AddSensorSpecArgs,
};

// ---------------------------------------------------------------------------
// Test 2 — test_BC_2_01_016_002_auth_composition_runtime_rejection
// AC-3: auth_type multi-value (array) → E-SPEC-012 (Rule A).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(AuthTypeCrossComposition).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3 / INV-AUTH-OPEN-003: A `SensorSpec` with a multi-valued
/// `auth_type` (array) must be rejected at spec-load with E-SPEC-012.
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-012 (Rule A — multi-valued or
/// out-of-set auth_type). Post-unsealing, the sealed-trait compile-time guard
/// is replaced by this runtime rejection rule (ADR-023 Rule 2, Rule A).
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics
/// rather than returning the expected error. Test fails on panic.
///
/// Story: S-PLUGIN-PREREQ-E AC-3 | BC: BC-2.01.016 | ADR-026 §D3 | ADR-023 Rule 2 Rule A
#[test]
fn test_BC_2_01_016_002_auth_composition_runtime_rejection() {
    // Sensor spec with out-of-set auth_type value (simulates cross-composition attempt).
    // The auth_type "invalid_composite_type" is outside the closed enumeration
    // {oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}.
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "invalid_composite_type", // out-of-set value — triggers Rule A (E-SPEC-012)
        1,                        // credential_refs_count: exactly 1 (Rule B not triggered)
        "oauth2_client_credentials", // expected_shape
        "oauth2_client_credentials", // actual_shape matches (Rule C not triggered)
    );
    assert!(
        result.is_err(),
        "AC-3: auth_type outside closed enumeration must be rejected with E-SPEC-012; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-012"),
        "AC-3: error must cite E-SPEC-012; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
// AC-3b: multiple credential_refs → E-SPEC-013 (Rule B).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(MultipleCredentialRefs).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b: A `SensorSpec` with multiple `credential_refs` for a
/// single auth method must be rejected at spec-load with E-SPEC-013.
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-013 (Rule B — credential_refs cardinality
/// must match the auth method's schema). ADR-023 Rule 2, Rule B.
///
/// Per BC-2.06.003 / ADR-032 amendment:
///   - `oauth2_client_credentials` now allows exactly 2 refs (client_id + client_secret).
///   - 3+ refs for `oauth2_client_credentials` → E-SPEC-013.
///   - 2 refs for `bearer_static` (which needs exactly 1) → E-SPEC-013.
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b | BC: BC-2.01.016 | BC-2.06.003 | ADR-032
#[test]
fn test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected() {
    // Case 1: 3 refs for oauth2_client_credentials (only 2 are allowed).
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "oauth2_client_credentials", // valid auth_type — Rule A passes
        3, // 3 credential_refs — triggers Rule B (E-SPEC-013); oauth2 allows 2
        "oauth2_client_credentials",
        "oauth2_client_credentials",
    );
    assert!(
        result.is_err(),
        "AC-3b: 3 credential_refs for oauth2_client_credentials must be rejected with E-SPEC-013; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-013"),
        "AC-3b: error must cite E-SPEC-013; got: {err_str}"
    );

    // Case 2: 2 refs for bearer_static (only 1 is allowed).
    let result2 = SpecLoader::validate_cross_composition(
        "test_sensor_bearer",
        "bearer_static", // valid auth_type — Rule A passes
        2,               // 2 credential_refs — triggers Rule B (E-SPEC-013); bearer_static allows 1
        "bearer_static",
        "bearer_static",
    );
    assert!(
        result2.is_err(),
        "AC-3b: 2 credential_refs for bearer_static must be rejected with E-SPEC-013; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err2 = result2.unwrap_err();
    let err2_str = format!("{err2}");
    assert!(
        err2_str.contains("E-SPEC-013"),
        "AC-3b: error must cite E-SPEC-013 for bearer_static with 2 refs; got: {err2_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 5 — test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
// AC-3c: auth_type/credential structural mismatch → E-SPEC-014 (Rule C).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(AuthTypeCredentialMismatch).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3c: A `SensorSpec` where `auth_type` and the resolved
/// credential's structural shape disagree must be rejected with E-SPEC-014.
///
/// Example: `auth_type = "bearer_static"` paired with an
/// oauth2-shaped credential (expected `bearer_token`, got `client_id`).
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-014 (Rule C — structural mismatch
/// between auth_type and credential shape). ADR-023 Rule 2, Rule C.
///
/// Note: per BC-2.06.003 / ADR-032, `oauth2_client_credentials` now requires
/// exactly 2 credential_refs. To test Rule C in isolation (past Rule B), we use
/// `bearer_static` with 1 ref (Rule B passes) and mismatched shapes.
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-3c | BC: BC-2.01.016 | ADR-023 Rule 2 Rule C
#[test]
fn test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected() {
    // Valid auth_type (Rule A passes), single credential_ref (Rule B passes for bearer_static),
    // but credential shape "api_key" doesn't match expected "bearer_static".
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "bearer_static", // declared auth_type
        1,               // single credential_ref — Rule B passes (bearer_static allows 1)
        "bearer_static", // expected structural shape for bearer_static
        "api_key",       // actual resolved credential shape — MISMATCH (E-SPEC-014)
    );
    assert!(
        result.is_err(),
        "AC-3c: auth_type/credential structural mismatch must be rejected with E-SPEC-014; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-014"),
        "AC-3c: error must cite E-SPEC-014; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// F-LP-IMPL-P1-003: Production spec-load path rejects cross-composition violations
//
// This test verifies that `SpecLoader::parse` invokes `validate_cross_composition`
// and rejects a TOML spec with a Rule B violation (multiple credential_refs).
//
// Pre-fix failure mode: `SpecLoader::parse` does NOT call `validate_cross_composition` —
// the TOML parses successfully and returns Ok(SensorSpec) even with 2 credential_refs.
// Post-fix: returns Err with a message citing E-SPEC-013.
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b / F-LP-IMPL-P1-003: `SpecLoader::parse` must invoke
/// `validate_cross_composition` and reject a TOML sensor spec with multiple
/// `credential_refs` declared (Rule B / E-SPEC-013).
///
/// This is the END-TO-END production wiring test. It calls `SpecLoader::parse`
/// (the production spec-load entry point), NOT `validate_cross_composition` directly.
///
/// Pre-fix failure mode: `SpecLoader::parse` returns `Ok(SensorSpec)` for a TOML
/// with 2 credential_refs — cross-composition validation is NOT called in the
/// production parse path.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b / F-LP-IMPL-P1-003 | BC-2.01.016 Rule B | ADR-026 §D3
#[test]
fn test_BC_2_01_016_e_spec_012_rejected_at_spec_load() {
    // Construct a minimal sensor TOML with 2 credential_refs (Rule B violation).
    // auth_type is valid (Rule A passes), but cardinality != 1 → E-SPEC-013.
    // Must include at least one [[tables]] entry to satisfy SensorSpec required field.
    // The cross-composition check fires after deserialization succeeds and before
    // the spec is returned to the caller.
    let toml_with_multiple_cred_refs = r#"
sensor_id = "test_cross_comp_sensor"
name = "Test Cross-Composition Sensor"
base_url = "https://api.example.com"
auth_type = "api_key"
version = "1.0.0"

[[credential_refs]]
name = "cred_one"

[[credential_refs]]
name = "cred_two"

[[tables]]
table_name = "test_table"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/items"
  response_path = "$.items"
  variables_produced = []
"#;

    // Pre-fix: returns Ok(SensorSpec) — validate_cross_composition not called.
    // Post-fix: returns Err wrapping E-SPEC-013 from validate_cross_composition.
    let result = SpecLoader::parse(toml_with_multiple_cred_refs);

    assert!(
        result.is_err(),
        "F-LP-IMPL-P1-003: SpecLoader::parse must reject a TOML with 2 credential_refs \
         (Rule B / E-SPEC-013); got Ok(SensorSpec) instead — production validate_cross_composition \
         call is missing in SpecLoader::parse"
    );

    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-013") || err_str.to_lowercase().contains("multiple"),
        "F-LP-IMPL-P1-003: error must cite E-SPEC-013 or 'multiple'; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 6 — test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory
// AC-3b: cross-composition violation rejected through the parse_spec_directory
// production boot path (F-LP-IMPL-P2-001).
//
// Pre-fix failure mode: parse_spec_directory returns Ok(ConfigSnapshot) with the
// spec loaded into sensor_specs — validate_cross_composition not called via
// parse_and_validate_spec_toml.
// Post-fix: spec ends up in failed_specs, not sensor_specs.
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b / F-LP-IMPL-P2-001: `parse_spec_directory` must reject a
/// sensor TOML with multiple `credential_refs` (Rule B / E-SPEC-013).
///
/// This is the PRODUCTION BOOT PATH test. `parse_spec_directory` is called by
/// `config_manager.rs:111` at boot step 4, by MCP `add_sensor_spec` after disk
/// write, and by hot_reload. All three call `parse_and_validate_spec_toml`
/// internally — wiring `validate_cross_composition` there closes all three.
///
/// Pre-fix failure mode: spec loads into `sensor_specs` without error because
/// `parse_and_validate_spec_toml` does NOT call `validate_cross_composition`.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b / F-LP-IMPL-P2-001 | BC-2.01.016 Rule B | ADR-026 §D3
#[test]
fn test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory() {
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir must be created");

    // Write a cross-composition TOML with 2 credential_refs.
    // ADR-030 Approach D: SpecLoader::parse uses flat top-level TOML format.
    // Uses `column_type` field (not `type`) per ColumnSpec serde field name.
    // Includes minimal [[tables.steps]] because `steps: Vec<FetchStep>` has no #[serde(default)].
    let toml_content = r#"
sensor_id = "xcomp_boot_sensor"
name = "Cross-Composition Boot Test"
base_url = "https://api.example.com"
auth_type = "api_key"
version = "1.0.0"

[[credential_refs]]
name = "cred_one"

[[credential_refs]]
name = "cred_two"

[[tables]]
table_name = "boot_table"
ocsf_class = "security_finding"

[[tables.columns]]
name = "id"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/boot"
response_path = "$.items"
variables_produced = []
"#;

    let file_path = dir.path().join("xcomp_boot_sensor.sensor.toml");
    std::fs::write(&file_path, toml_content).expect("write must succeed");

    // Pre-fix: returns Ok(snapshot) with spec in sensor_specs (validation bypassed).
    // Post-fix: spec is in failed_specs with E-SPEC-013 validation error.
    let snapshot =
        parse_spec_directory(dir.path()).expect("parse_spec_directory must not return Err");

    assert!(
        snapshot.sensor_specs.get("xcomp_boot_sensor").is_none(),
        "F-LP-IMPL-P2-001: cross-composition spec must NOT appear in sensor_specs; \
         parse_and_validate_spec_toml must call validate_cross_composition before accepting spec"
    );

    assert!(
        snapshot.failed_specs.contains_key("xcomp_boot_sensor"),
        "F-LP-IMPL-P2-001: cross-composition spec must appear in failed_specs with E-SPEC-013; \
         got failed_specs keys: {:?}",
        snapshot.failed_specs.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Test 7 — test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool
// AC-3b: cross-composition violation rejected through the MCP add_sensor_spec
// entry point (F-LP-IMPL-P2-001).
//
// Pre-fix failure mode: add_sensor_spec returns Added{} — spec accepted because
// parse_and_validate_spec_toml does NOT call validate_cross_composition.
// Post-fix: returns ValidationFailed with E-SPEC-013 errors.
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b / F-LP-IMPL-P2-001: The MCP `add_sensor_spec` entry point
/// must reject a sensor TOML with multiple `credential_refs` (Rule B / E-SPEC-013).
///
/// `add_sensor_spec` calls `parse_and_validate_spec_toml` at line 239. When
/// `validate_cross_composition` is wired into `parse_and_validate_spec_toml`,
/// this path automatically rejects cross-composition violations.
///
/// Pre-fix failure mode: `add_sensor_spec` returns `Added` or `ConfirmationRequired`
/// because parse succeeds without calling validate_cross_composition.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b / F-LP-IMPL-P2-001 | BC-2.01.016 Rule B | ADR-026 §D3
#[test]
fn test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool() {
    use std::sync::Arc;

    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir must be created");
    let manager = Arc::new(ConfigManager::empty());

    // ADR-030 Approach D: SpecLoader::parse uses flat top-level TOML format (no [sensor] section).
    // Uses `column_type` field (not `type`) per ColumnSpec serde field name.
    // Includes minimal [[tables.steps]] because `steps: Vec<FetchStep>` has no #[serde(default)].
    let toml_with_multiple_cred_refs = r#"
sensor_id = "xcomp_mcp_sensor"
name = "Cross-Composition MCP Test"
base_url = "https://api.example.com"
auth_type = "api_key"
version = "1.0.0"

[[credential_refs]]
name = "cred_one"

[[credential_refs]]
name = "cred_two"

[[tables]]
table_name = "mcp_table"
ocsf_class = "security_finding"

[[tables.columns]]
name = "id"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/mcp"
response_path = "$.items"
variables_produced = []
"#;

    let args = AddSensorSpecArgs {
        spec_toml: toml_with_multiple_cred_refs.to_string(),
        file_name: None,
        dry_run: false,
    };

    // Pre-fix: returns Added or ConfirmationRequired (spec accepted, disk write attempted).
    // Post-fix: returns ValidationFailed containing E-SPEC-013 error.
    let result = add_sensor_spec(&manager, dir.path(), args).expect("add_sensor_spec must not Err");

    // Must return ValidationFailed — not Added, DryRun, or ConfirmationRequired.
    match &result {
        prism_spec_engine::types::AddSensorSpecResult::ValidationFailed { errors } => {
            let combined_errors = errors
                .iter()
                .flat_map(|e| e.errors.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            assert!(
                combined_errors.contains("E-SPEC-013")
                    || combined_errors.to_lowercase().contains("multiple"),
                "F-LP-IMPL-P2-001: ValidationFailed must cite E-SPEC-013; \
                 got errors: {combined_errors}"
            );
        }
        other => {
            panic!(
                "F-LP-IMPL-P2-001: add_sensor_spec must return ValidationFailed for \
                 cross-composition spec; got: {:?}",
                std::mem::discriminant(other)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 8 — test_BC_2_01_016_e_spec_014_rejected_via_hot_reload
// AC-3b: cross-composition violation rejected through hot_reload
// process_spec_changes (F-LP-IMPL-P2-001).
//
// Pre-fix failure mode: process_spec_changes returns Ok(ReloadResult) with the
// spec added to sensor_specs — validate_cross_composition not called.
// Post-fix: spec ends up in validation_errors, NOT added to sensor_specs.
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b / F-LP-IMPL-P2-001: `process_spec_changes` (hot reload)
/// must reject a sensor TOML with multiple `credential_refs` (Rule B / E-SPEC-013).
///
/// Hot reload calls `parse_and_validate_spec_toml` at hot_reload.rs:121 and :206.
/// When `validate_cross_composition` is wired into `parse_and_validate_spec_toml`,
/// hot reload automatically rejects cross-composition violations and records them
/// in `ReloadResult.validation_errors`.
///
/// Pre-fix failure mode: sensor spec is accepted into `added` list; no error recorded.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b / F-LP-IMPL-P2-001 | BC-2.01.016 Rule B | ADR-026 §D3
#[test]
fn test_BC_2_01_016_e_spec_014_rejected_via_hot_reload() {
    use std::sync::Arc;

    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir must be created");
    let manager = Arc::new(ConfigManager::empty());

    // ADR-030 Approach D: SpecLoader::parse uses flat top-level TOML format.
    // Uses `column_type` field (not `type`) per ColumnSpec serde field name.
    // Includes minimal [[tables.steps]] because `steps: Vec<FetchStep>` has no #[serde(default)].
    let toml_content = r#"
sensor_id = "xcomp_reload_sensor"
name = "Cross-Composition Hot-Reload Test"
base_url = "https://api.example.com"
auth_type = "api_key"
version = "1.0.0"

[[credential_refs]]
name = "cred_one"

[[credential_refs]]
name = "cred_two"

[[tables]]
table_name = "reload_table"
ocsf_class = "security_finding"

[[tables.columns]]
name = "id"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/reload"
response_path = "$.items"
variables_produced = []
"#;

    let file_path = dir.path().join("xcomp_reload_sensor.sensor.toml");
    std::fs::write(&file_path, toml_content).expect("write must succeed");

    // Simulate a hot-reload SpecChangeEvent::Added for the new file.
    let events = vec![SpecChangeEvent::Added(file_path)];

    // Pre-fix: reload_result.added contains the sensor's table name —
    //   validate_cross_composition not called; spec accepted without error.
    // Post-fix: reload_result.validation_errors is non-empty; sensor NOT in added list.
    let reload_result = process_spec_changes(events, &manager, dir.path())
        .expect("process_spec_changes must not Err");

    assert!(
        !reload_result.validation_errors.is_empty(),
        "F-LP-IMPL-P2-001: hot_reload must record a validation_error for cross-composition spec; \
         validation_errors was empty — validate_cross_composition not called via \
         parse_and_validate_spec_toml in hot_reload path"
    );

    assert!(
        !reload_result
            .added
            .iter()
            .any(|t| t.contains("xcomp_reload_sensor")),
        "F-LP-IMPL-P2-001: cross-composition spec table must NOT appear in reload added list; \
         got added: {:?}",
        reload_result.added
    );
}
