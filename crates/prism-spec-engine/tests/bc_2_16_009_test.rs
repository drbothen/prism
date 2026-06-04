#![allow(non_snake_case)]
//! BC-2.16.009: Spec File Validation — Schema Validation, Variable Reference Resolution,
//!              OCSF Field Validation
//!
//! Tests cover all 5 validation rule categories:
//!   1. Schema: sensor_id, name, auth_type, base_url, version, table names, columns, steps
//!   2. Variable references: dangling, forward, self-reference
//!   3. OCSF field paths: invalid path -> warning (not error)
//!   4. Pagination config: cursor without path, offset with page_size=0
//!   5. Rate limit hints: requests_per_second <= 0
//!
//! Multi-error: all errors collected in single pass (VP-059 — see proofs/spec_validator.rs)
//!
//! AC-5 (S-1.11): dangling ${nonexistent.field} -> error with line number.

use prism_core::{ColumnType, SpecErrorCode};
use prism_spec_engine::{
    add_sensor_spec::parse_and_validate_spec_toml,
    spec_parser::{
        AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec, TableSpec,
    },
    validation::{
        validate_ocsf_field_path, validate_sensor_id, validate_sensor_spec,
        validate_variable_references,
    },
};

fn minimal_valid_spec() -> SensorSpec {
    SensorSpec::new(
        "valid-sensor",
        "Valid Sensor",
        AuthType::BearerStatic,
        "https://api.example.com",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_alerts",
                "GET",
                "/alerts",
                None,
                "$.resources",
                None,
                vec![],
                None,
                None,
            )],
        )],
        None,
        "1.0.0",
        Vec::new(),
    )
}

// ---------------------------------------------------------------------------
// Category 1: Schema Validation
// ---------------------------------------------------------------------------

/// BC-2.16.009 schema: valid spec -> no errors, no warnings.
/// Canonical test vector.
#[test]
fn test_BC_2_16_009_valid_spec_returns_ok_no_errors() {
    let spec = minimal_valid_spec();
    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_ok(),
        "valid spec must return Ok: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "valid spec must produce no warnings"
    );
}

/// BC-2.16.009 schema: sensor_id starts with digit -> E-SPEC-001 with sensor.sensor_id path.
/// Canonical test vector.
#[test]
fn test_BC_2_16_009_rejects_sensor_id_starting_with_digit() {
    let err = validate_sensor_id("1invalid-id", Some("test.sensor.toml"));
    assert!(err.is_some(), "sensor_id '1invalid-id' must be rejected");
    let e = err.unwrap();
    assert_eq!(e.code, SpecErrorCode::ESpec001);
    let path = e.toml_path.as_deref().unwrap_or("");
    assert!(
        path.contains("sensor_id"),
        "toml_path must reference sensor_id: got '{path}'"
    );
}

/// BC-2.16.009 schema: sensor_id with hyphen and numbers -> valid.
#[test]
fn test_BC_2_16_009_accepts_sensor_id_with_hyphens_and_digits() {
    let err = validate_sensor_id("sensor-01", None);
    assert!(err.is_none(), "sensor-01 must be a valid sensor_id");
}

/// BC-2.16.009 schema: empty sensor name -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_empty_sensor_name() {
    let mut spec = minimal_valid_spec();
    spec.name = "".to_string();
    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "empty name must produce an error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.to_lowercase().contains("name") && e.code == SpecErrorCode::ESpec001),
        "error must reference empty name: {:?}",
        errors
    );
}

/// BC-2.16.009 schema: invalid base_url (not a URL) -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_invalid_base_url() {
    let mut spec = minimal_valid_spec();
    spec.base_url = "not-a-url".to_string();
    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "invalid base_url must produce an error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.code == SpecErrorCode::ESpec001),
        "E-SPEC-001 must be produced for invalid base_url: {:?}",
        errors
    );
}

/// BC-2.16.009 schema: table with no columns -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_table_with_no_columns() {
    let mut spec = minimal_valid_spec();
    spec.tables[0].columns.clear();
    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "table with no columns must produce an error"
    );
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("column")),
        "error must mention columns: {:?}",
        errors
    );
}

/// BC-2.16.009 schema: table with no steps -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_table_with_no_steps() {
    let mut spec = minimal_valid_spec();
    spec.tables[0].steps.clear();
    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "table with no steps must produce an error");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("step")),
        "error must mention steps: {:?}",
        errors
    );
}

/// BC-2.16.009 schema: duplicate column names within a table -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_duplicate_column_names_within_table() {
    let mut spec = minimal_valid_spec();
    let dup = spec.tables[0].columns[0].clone();
    spec.tables[0].columns.push(dup);

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "duplicate column names must produce an error"
    );
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("column")
                || e.message.to_lowercase().contains("duplicate")),
        "error must reference duplicate columns: {:?}",
        errors
    );
}

/// BC-2.16.009 schema: invalid version (not semver) -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_invalid_version_string() {
    let mut spec = minimal_valid_spec();
    spec.version = "not.semver!".to_string();
    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "non-semver version must produce an error");
}

// ---------------------------------------------------------------------------
// Category 2: Variable Reference Resolution
// ---------------------------------------------------------------------------

/// BC-2.16.009 variable ref: AC-5 — dangling ${nonexistent.field} -> E-SPEC-001 with path.
/// Canonical test vector for AC-5.
#[test]
fn test_BC_2_16_009_rejects_dangling_variable_ref_with_toml_path() {
    let steps = vec![FetchStep::new(
        "step_one",
        "GET",
        "/alerts?id=${nonexistent.field}",
        None,
        "$.data",
        None,
        vec![],
        None,
        None,
    )];

    let errors = validate_variable_references(
        "/alerts?id=${nonexistent.field}",
        "sensor.tables[0].steps[0].path_template",
        &steps,
        0,
    );

    assert!(!errors.is_empty(), "dangling ref must produce errors");
    assert!(
        errors.iter().any(|e| e.code == SpecErrorCode::ESpec001),
        "must be E-SPEC-001: {:?}",
        errors
    );
    assert!(
        errors
            .iter()
            .any(|e| e.toml_path.as_deref() == Some("sensor.tables[0].steps[0].path_template")),
        "toml_path must be included for actionable correction: {:?}",
        errors
    );
    assert!(
        errors.iter().any(|e| e.message.contains("nonexistent")),
        "error must name the undefined step: {:?}",
        errors
    );
}

/// BC-2.16.009 variable ref: forward reference -> E-SPEC-001 identifying both steps.
/// DEC-038 canonical test vector.
#[test]
fn test_BC_2_16_009_rejects_forward_variable_reference() {
    let steps = vec![
        FetchStep::new(
            "step_one",
            "GET",
            "/data?ids=${step_two.ids}", // forward ref
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        ),
        FetchStep::new(
            "step_two",
            "GET",
            "/ids",
            None,
            "$.ids",
            None,
            vec!["ids".to_string()],
            None,
            None,
        ),
    ];

    let errors = validate_variable_references(
        "/data?ids=${step_two.ids}",
        "sensor.tables[0].steps[0].path_template",
        &steps,
        0,
    );

    assert!(!errors.is_empty(), "forward reference must produce errors");
    let e = &errors[0];
    assert!(
        e.message.contains("step_two") || e.message.contains("forward"),
        "error must identify the forward-referenced step: {}",
        e.message
    );
}

/// BC-2.16.009 variable ref: valid backward reference -> no errors.
#[test]
fn test_BC_2_16_009_accepts_valid_backward_variable_reference() {
    let steps = vec![
        FetchStep::new(
            "get_token",
            "POST",
            "/oauth/token",
            None,
            "$.access_token",
            None,
            vec!["access_token".to_string()],
            None,
            None,
        ),
        FetchStep::new(
            "fetch_data",
            "GET",
            "/data?token=${get_token.access_token}", // valid back-ref
            None,
            "$.resources",
            None,
            vec![],
            None,
            None,
        ),
    ];

    let errors = validate_variable_references(
        "/data?token=${get_token.access_token}",
        "sensor.tables[0].steps[1].path_template",
        &steps,
        1, // step at index 1; get_token is at 0 -> valid backward reference
    );

    assert!(
        errors.is_empty(),
        "valid backward reference must produce no errors: {:?}",
        errors
    );
}

// ---------------------------------------------------------------------------
// Category 3: OCSF Field Validation (warnings, not errors)
// ---------------------------------------------------------------------------

/// BC-2.16.009 OCSF field: invalid path -> warning (not error). Spec loads.
#[test]
fn test_BC_2_16_009_invalid_ocsf_field_produces_warning_not_error() {
    let warning = validate_ocsf_field_path(
        "nonexistent.made.up.field",
        "some_column",
        "sensor.tables[0].columns[0].ocsf_field",
    );

    assert!(
        warning.is_some(),
        "invalid OCSF field path must produce a warning"
    );
    let w = warning.unwrap();
    assert!(
        w.message.contains("nonexistent") || w.message.contains("ocsf"),
        "warning must reference the invalid path: {}",
        w.message
    );
}

/// BC-2.16.009 OCSF field: valid standard path -> no warning.
/// (Note: "time" is a known OCSF base event field.)
#[test]
fn test_BC_2_16_009_valid_ocsf_field_produces_no_warning() {
    let warning = validate_ocsf_field_path(
        "time",
        "created_timestamp",
        "sensor.tables[0].columns[0].ocsf_field",
    );
    assert!(
        warning.is_none(),
        "valid OCSF field 'time' must produce no warning"
    );
}

// ---------------------------------------------------------------------------
// Category 4: Pagination Configuration
// ---------------------------------------------------------------------------

/// BC-2.16.009 pagination: cursor_token with empty cursor_response_path -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_cursor_pagination_with_empty_response_path() {
    let mut spec = minimal_valid_spec();
    if let Some(step) = spec.tables[0].steps.first_mut() {
        step.pagination = Some(PaginationConfig::CursorToken {
            cursor_response_path: "".to_string(), // invalid
            page_size: None,
        });
    }

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "cursor_token with empty path must produce an error"
    );
}

/// BC-2.16.009 pagination: offset_limit with page_size=0 -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_offset_pagination_with_zero_page_size() {
    let mut spec = minimal_valid_spec();
    if let Some(step) = spec.tables[0].steps.first_mut() {
        step.pagination = Some(PaginationConfig::OffsetLimit { page_size: 0 });
    }

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "offset_limit with page_size=0 must produce an error"
    );
}

// ---------------------------------------------------------------------------
// Category 5: Rate Limit Hints
// ---------------------------------------------------------------------------

/// BC-2.16.009 rate limit: requests_per_second <= 0 -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_rate_limit_requests_per_second_zero_or_negative() {
    let mut spec = minimal_valid_spec();
    spec.rate_limit_hints = Some(RateLimitHints::new(Some(0.0), None)); // invalid: must be > 0

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "requests_per_second=0 must produce an error"
    );
}

/// BC-2.16.009 rate limit: burst_size=0 -> E-SPEC-001.
#[test]
fn test_BC_2_16_009_rejects_rate_limit_burst_size_zero() {
    let mut spec = minimal_valid_spec();
    spec.rate_limit_hints = Some(RateLimitHints::new(Some(10.0), Some(0))); // invalid: must be >= 1

    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "burst_size=0 must produce an error");
}

// ---------------------------------------------------------------------------
// Multi-error reporting (BC-2.16.009 / VP-059 subset)
// ---------------------------------------------------------------------------

/// BC-2.16.009 multi-error: 3 schema errors + 1 variable error -> 4 errors together.
/// This is the canonical "multiple errors reported together" test vector.
#[test]
fn test_BC_2_16_009_reports_all_errors_together_no_fail_fast() {
    let mut spec = minimal_valid_spec();
    spec.sensor_id = "1invalid".to_string(); // error 1
    spec.name = "".to_string(); // error 2
    spec.base_url = "not-a-url".to_string(); // error 3
    // forward ref for error 4
    spec.tables[0].steps.push(FetchStep::new(
        "step2",
        "GET",
        "/detail",
        None,
        "$.data",
        None,
        vec!["ids".to_string()],
        None,
        None,
    ));
    spec.tables[0].steps[0].path_template = "/data?id=${step2.ids}".to_string(); // forward ref

    let result = validate_sensor_spec(&spec);
    assert!(result.is_err(), "spec with 4 errors must return Err");
    let errors = result.unwrap_err();
    assert!(
        errors.len() >= 4,
        "at least 4 errors must be reported together (no fail-fast); got {}",
        errors.len()
    );
}

// ---------------------------------------------------------------------------
// F-LP4-HIGH-001: fan_out_batch_size = 0 validation
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP4-HIGH-001: `fan_out_batch_size = 0` must be rejected by
/// `validate_sensor_spec` before `PipelineExecutor::execute` is ever called.
///
/// A value of 0 would cause `slice::chunks(0)` to panic at runtime (DoS via
/// TOML spec input). This validation is the primary guard; `fan_out_batches`
/// has a defense-in-depth `.max(1)` clamp as secondary protection.
///
/// The test verifies:
/// 1. The error is for `fan_out_batch_size` specifically (not some other field).
/// 2. The error message contains "fan_out_batch_size" and the offending step name.
/// 3. The error code is `E-SPEC-001`.
#[test]
fn test_BC_2_16_002_validation_rejects_fan_out_batch_size_zero() {
    let mut spec = minimal_valid_spec();
    // Set fan_out_batch_size = 0 on the first step (DoS input).
    spec.tables[0].steps[0].fan_out_batch_size = Some(0);

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "spec with fan_out_batch_size = 0 must be rejected by validation; got Ok"
    );

    let errors = result.unwrap_err();
    let fan_out_err = errors
        .iter()
        .find(|e| e.message.contains("fan_out_batch_size"));
    assert!(
        fan_out_err.is_some(),
        "error list must contain an entry mentioning 'fan_out_batch_size'; got: {errors:?}"
    );

    let e = fan_out_err.unwrap();
    assert_eq!(
        e.code,
        SpecErrorCode::ESpec001,
        "fan_out_batch_size=0 error must carry E-SPEC-001 code; got {:?}",
        e.code
    );
    // Step name must appear in the message for actionable diagnostics.
    assert!(
        e.message.contains("fetch_alerts"),
        "error message must identify the offending step name 'fetch_alerts'; got: {}",
        e.message
    );
    assert!(
        e.message.contains("> 0") || e.message.contains("must be"),
        "error message must communicate the constraint (> 0 or 'must be'); got: {}",
        e.message
    );
}

/// BC-2.16.002 / F-LP4-HIGH-001: `fan_out_batch_size = 1` (minimum valid) must
/// be accepted by `validate_sensor_spec`.
#[test]
fn test_BC_2_16_002_validation_accepts_fan_out_batch_size_one() {
    let mut spec = minimal_valid_spec();
    spec.tables[0].steps[0].fan_out_batch_size = Some(1);
    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_ok(),
        "fan_out_batch_size = 1 must be accepted; got: {:?}",
        result.err()
    );
}

/// BC-2.16.002 / F-LP4-HIGH-001 (defense-in-depth): `fan_out_batch_size = None`
/// (absent — uses default 100) must be accepted by `validate_sensor_spec`.
#[test]
fn test_BC_2_16_002_validation_accepts_fan_out_batch_size_none() {
    let spec = minimal_valid_spec();
    assert!(
        spec.tables[0].steps[0].fan_out_batch_size.is_none(),
        "minimal_valid_spec must leave fan_out_batch_size unset"
    );
    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_ok(),
        "spec with fan_out_batch_size absent must be valid; got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// F-LP5-LOW-001 regression: response_path "$." rejected at validator layer
// ---------------------------------------------------------------------------

/// BC-2.16.009 / F-LP5-LOW-001: `validate_sensor_spec` must reject a step whose
/// `response_path` is `"$."` — the path contains no key segment after the prefix,
/// which would cause `extract_at_path` to silently produce an empty pointer.
///
/// This is defense layer 2: the validator catches malformed paths before the
/// pipeline executor is ever invoked.
#[test]
fn test_BC_2_16_002_validation_rejects_malformed_dollar_dot_response_path() {
    let mut spec = minimal_valid_spec();
    // Inject "$." as response_path on the single step.
    spec.tables[0].steps[0].response_path = "$.".to_string();

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "F-LP5-LOW-001: response_path='$.' must produce a validation error; got Ok"
    );

    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("response_path")
            || e.message.contains("$.")
            || e.message.contains("non-empty JSONPath")),
        "F-LP5-LOW-001: error must mention response_path malformation; got errors: {:?}",
        errors
    );
}

/// BC-2.16.009 / F-LP5-LOW-001: `validate_sensor_spec` must accept a valid
/// response_path such as `"$.items"` — regression guard for the new validator rule.
#[test]
fn test_BC_2_16_002_validation_accepts_valid_dollar_dot_response_path() {
    let mut spec = minimal_valid_spec();
    spec.tables[0].steps[0].response_path = "$.items".to_string();

    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_ok(),
        "F-LP5-LOW-001 guard: valid response_path '$.items' must pass validation; got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// F-LP10-MED-001: byte-boundary-safe truncation of base_url in error messages
// ---------------------------------------------------------------------------

/// BC-2.16.009 / F-LP10-MED-001: `validate_sensor_spec` must NOT panic when
/// `base_url` contains multi-byte UTF-8 characters and a byte-index slice lands
/// mid-codepoint.
///
/// Constructs `base_url = "x" + "🎯".repeat(60)` = 1 + 60*4 = 241 bytes.
/// With the old `&url[..url.len().min(200)]`, byte 200 falls inside the 50th emoji
/// (bytes 197..201) — a non-char-boundary — causing a `byte index 200 is not a
/// char boundary` panic.
///
/// RED GATE (pre-fix): PANICS with byte-boundary panic.
/// GREEN (post-fix): returns `Err(ValidationError)` — no panic — and error message
///   contains a truncated prefix of the URL.
#[test]
fn test_BC_2_16_009_validation_handles_multibyte_utf8_base_url_without_panic() {
    let mut spec = minimal_valid_spec();
    // "x" + 60 × "🎯" = 1 + 240 = 241 bytes; 61 codepoints.
    // Old code: &url[..241.min(200)] = &url[..200] — byte 200 is inside emoji 50 → panic.
    spec.base_url = format!("x{}", "🎯".repeat(60));

    // Must not panic — must return a clean ValidationError.
    let result = validate_sensor_spec(&spec);
    assert!(
        result.is_err(),
        "base_url not starting with http:// must produce an error; got Ok"
    );
    let errors = result.unwrap_err();
    // The error must exist and must NOT panic to reach this assert.
    assert!(
        errors.iter().any(|e| e.code == SpecErrorCode::ESpec001),
        "error must carry E-SPEC-001: {:?}",
        errors
    );
    // Error message must contain a truncated (but char-boundary-safe) prefix of the URL.
    let msg = &errors
        .iter()
        .find(|e| e.code == SpecErrorCode::ESpec001)
        .unwrap()
        .message;
    assert!(
        msg.contains('x') || msg.contains("base_url"),
        "error message must reference the url or contain its prefix: {msg}"
    );
}

// ---------------------------------------------------------------------------
// F-LOCAL-P1-MED-001 — Regression-proof tests through the production load path
//
// These tests exercise `parse_and_validate_spec_toml` (the production spec-load
// path used by `parse_spec_directory` at boot and by the MCP add_sensor_spec
// tool). They are LOAD-BEARING: deleting the `validate_step_methods` call from
// `parse_and_validate_spec_toml` causes both tests to fail.
//
// Test naming: test_BC_2_16_009_load_path_*
// Traces to: BC-2.16.009 v1.8 §Validation Rules 7; S-SPEC-HTTP-METHOD-VALIDATION-001.
// ---------------------------------------------------------------------------

/// BC-2.16.009 v1.8 §VR7 — F-LOCAL-P1-MED-001 load-path regression test 1.
///
/// A TOML spec with `method = "CONNECT"` fed through `parse_and_validate_spec_toml`
/// must return `Err` whose error messages include the E-SPEC-025 canonical text.
///
/// This test is load-bearing on the Rule-7 wiring in `parse_and_validate_spec_toml`:
/// removing the `validate_step_methods` call from that function causes this test
/// to return `Ok` instead of `Err`, triggering an assertion failure.
///
/// Traces to: BC-2.16.009 §VR7 AC-7; F-LOCAL-P1-MED-001.
#[test]
fn test_BC_2_16_009_load_path_connect_method_produces_e_spec_025() {
    let toml = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "CONNECT"
  path_template = "/api/v1/events"
  response_path = "$.data"
  variables_produced = []
"#;

    let result = parse_and_validate_spec_toml(toml, "<test>");

    assert!(
        result.is_err(),
        "F-LOCAL-P1-MED-001: TOML spec with method='CONNECT' must be rejected by \
         parse_and_validate_spec_toml (load path); got Ok — Rule-7 wiring may be missing"
    );

    let errors = result.unwrap_err();
    let all_error_text: Vec<&str> = errors
        .iter()
        .flat_map(|e| e.errors.iter().map(|s| s.as_str()))
        .collect();
    let combined = all_error_text.join(" | ");

    // The E-SPEC-025 canonical message must appear (from SpecEngineError::InvalidHttpMethod Display).
    assert!(
        combined.contains("CONNECT"),
        "F-LOCAL-P1-MED-001: error text must cite method value 'CONNECT'; got: {combined:?}"
    );
    assert!(
        combined.contains("not a supported HTTP method"),
        "F-LOCAL-P1-MED-001: error text must include canonical E-SPEC-025 phrase \
         'not a supported HTTP method'; got: {combined:?}"
    );
}

/// BC-2.16.009 v1.8 §VR7 AC-003 — F-LOCAL-P1-MED-001 load-path regression test 2
/// (env-var ordering: Rule 6 resolution → Rule 7 validation).
///
/// Sets `SENSOR_STEP_METHOD=CONNECT` in the environment, constructs a TOML spec
/// with `method = "${env.SENSOR_STEP_METHOD}"`, runs `parse_and_validate_spec_toml`.
///
/// Expected: Rule 6 resolves the token to "CONNECT"; Rule 7 then fires E-SPEC-025
/// on the resolved value "CONNECT".
///
/// This genuinely exercises the Rule-6 → Rule-7 ordering because the token starts
/// as `${env.SENSOR_STEP_METHOD}` (unresolved), and only after env resolution does
/// Rule 7 see "CONNECT" and produce E-SPEC-025.
///
/// Env isolation: unique variable name `PRISM_TEST_HTTP_METHOD_VR7_AC003` prevents
/// collision with other tests. The variable is removed after the test regardless
/// of pass/fail (cleanup before the final assertion).
///
/// This test is load-bearing on BOTH the Rule-6 wiring (env resolution) AND the
/// Rule-7 wiring (method validation) in `parse_and_validate_spec_toml`. Removing
/// either call causes this test to fail.
///
/// Traces to: BC-2.16.009 §VR7 AC-003; F-LOCAL-P1-MED-001.
#[test]
fn test_BC_2_16_009_load_path_env_resolved_invalid_method_produces_e_spec_025() {
    const VAR: &str = "PRISM_TEST_HTTP_METHOD_VR7_AC003";

    // SAFETY: nextest runs each integration test binary in a forked process.
    // The unique variable name prevents collision with ambient env or other tests.
    unsafe {
        std::env::set_var(VAR, "CONNECT");
    }

    // Embed the env token in TOML method field — Rule 6 will resolve it to "CONNECT".
    let toml = format!(
        r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "${{env.{VAR}}}"
  path_template = "/api/v1/events"
  response_path = "$.data"
  variables_produced = []
"#
    );

    let result = parse_and_validate_spec_toml(&toml, "<test>");

    // Clean up env var before assertions — ensures cleanup even if assert panics.
    unsafe {
        std::env::remove_var(VAR);
    }

    assert!(
        result.is_err(),
        "F-LOCAL-P1-MED-001 AC-003: env-resolved method 'CONNECT' must be rejected by \
         parse_and_validate_spec_toml (load path); got Ok"
    );

    let errors = result.unwrap_err();
    let all_error_text: Vec<&str> = errors
        .iter()
        .flat_map(|e| e.errors.iter().map(|s| s.as_str()))
        .collect();
    let combined = all_error_text.join(" | ");

    // The resolved value "CONNECT" must appear in the error message (Rule 7 fires on RESOLVED value).
    assert!(
        combined.contains("CONNECT"),
        "F-LOCAL-P1-MED-001 AC-003: error text must cite resolved method value 'CONNECT'; \
         got: {combined:?}"
    );
    assert!(
        combined.contains("not a supported HTTP method"),
        "F-LOCAL-P1-MED-001 AC-003: error text must include canonical E-SPEC-025 phrase \
         'not a supported HTTP method'; got: {combined:?}"
    );
}

// ---------------------------------------------------------------------------
// F-LOCAL-P2-MED-001 / F-LOCAL-P2-MED-002 — load_all E-SPEC-025 numeric-index
// toml_path and load-bearing test coverage
//
// These tests exercise `SpecLoader::load_all` (the boot spec-load directory path)
// rather than `parse_and_validate_spec_toml` (the MCP single-file path).
//
// Both tests are LOAD-BEARING:
//   1. F-LOCAL-P2-MED-002-A: removing the `validate_step_methods` call from
//      `load_all` causes this test to return zero errors, triggering the
//      "must have at least one error" assertion.
//   2. F-LOCAL-P2-MED-001: the toml_path assertion fails if `load_all` emits
//      string names in the numeric-index slots (e.g., `tables[events]`).
//
// Test naming: test_BC_2_16_009_load_all_*
// Traces to: BC-2.16.009 v1.8 §VR7; S-SPEC-HTTP-METHOD-VALIDATION-001;
//            F-LOCAL-P2-MED-001; F-LOCAL-P2-MED-002.
// ---------------------------------------------------------------------------

/// BC-2.16.009 v1.8 §VR7 — F-LOCAL-P2-MED-002 / F-LOCAL-P2-MED-001.
///
/// `SpecLoader::load_all` pointed at a temp dir containing a single
/// `*.sensor.toml` with `method = "CONNECT"` must:
///   (a) Return at least one error in the errors vec (F-LOCAL-P2-MED-002 — load-bearing).
///   (b) The error must carry `code == ESpec025` (F-LOCAL-P2-MED-002).
///   (c) The error must carry `toml_path` using NUMERIC indices —
///       `sensor.tables[0].steps[0].method` — not string names like
///       `sensor.tables[events].steps[fetch].method` (F-LOCAL-P2-MED-001).
///   (d) The error message must contain the canonical E-SPEC-025 phrase
///       "not a supported HTTP method" (F-LOCAL-P2-MED-002).
///
/// Load-bearing guarantees:
///   - Removing the `validate_step_methods` call from `load_all` causes (a) to
///     fail (zero errors returned → assertion panics).
///   - Reverting the numeric-index fix in `load_all` causes (c) to fail (string
///     name `events` appears in the bracket, not a digit).
///
/// Traces to: BC-2.16.009 §VR7 AC-7; F-LOCAL-P2-MED-001; F-LOCAL-P2-MED-002.
#[test]
fn test_BC_2_16_009_load_all_invalid_method_produces_e_spec_025_with_numeric_toml_path() {
    // Write the spec TOML into a temp directory.
    // Filename must be `<sensor_id>.sensor.toml` (BC-2.16.001 §E-SPEC-017).
    let dir = tempfile::tempdir().expect("tempdir must succeed");

    let toml_content = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "CONNECT"
  path_template = "/api/v1/events"
  response_path = "$.data"
  variables_produced = []
"#;

    let spec_file = dir.path().join("test-sensor.sensor.toml");
    std::fs::write(&spec_file, toml_content).expect("write spec file must succeed");

    // Run the production directory-scan load path.
    let loader =
        prism_spec_engine::spec_parser::SpecLoader::new(dir.path().to_string_lossy().to_string());
    let (_descriptors, errors) = loader.load_all();

    // (a) + (d) F-LOCAL-P2-MED-002 — load-bearing: errors must be non-empty.
    assert!(
        !errors.is_empty(),
        "F-LOCAL-P2-MED-002: SpecLoader::load_all with method='CONNECT' must produce at \
         least one error; got zero errors — validate_step_methods call may be missing from load_all"
    );

    // Find the E-SPEC-025 error.
    let e_spec_025 = errors.iter().find(|e| {
        matches!(
            e,
            prism_core::PrismError::Spec(prism_core::SpecError {
                code: prism_core::SpecErrorCode::ESpec025,
                ..
            })
        )
    });

    assert!(
        e_spec_025.is_some(),
        "F-LOCAL-P2-MED-002: errors must include PrismError::Spec with code ESpec025; \
         got: {errors:?}"
    );

    let e_spec_025 = e_spec_025.unwrap();

    // (b) Message must contain the canonical E-SPEC-025 phrase.
    if let prism_core::PrismError::Spec(se) = e_spec_025 {
        assert!(
            se.message.contains("not a supported HTTP method"),
            "F-LOCAL-P2-MED-002: E-SPEC-025 message must contain canonical phrase \
             'not a supported HTTP method'; got: {}",
            se.message
        );

        // (c) F-LOCAL-P2-MED-001 — toml_path must use NUMERIC indices.
        // The spec has exactly one table (index 0) and one step (index 0).
        // The correct path is `sensor.tables[0].steps[0].method`.
        // The pre-fix bug produced `sensor.tables[events].steps[fetch].method`.
        let toml_path = se.toml_path.as_deref().unwrap_or("");

        assert!(
            !toml_path.is_empty(),
            "F-LOCAL-P2-MED-001: E-SPEC-025 toml_path must be Some(path); got None"
        );

        // Must contain `tables[0]` — numeric index, not string name.
        assert!(
            toml_path.contains("tables[0]"),
            "F-LOCAL-P2-MED-001: toml_path must use numeric index 'tables[0]', not a \
             string name like 'tables[events]'; got: '{toml_path}'"
        );

        // Must contain `steps[` followed immediately by a digit.
        // Use a simple approach: verify `steps[0]` is present (single step at index 0).
        assert!(
            toml_path.contains("steps[0]"),
            "F-LOCAL-P2-MED-001: toml_path must use numeric index 'steps[0]', not a \
             string name like 'steps[fetch]'; got: '{toml_path}'"
        );

        // Final sanity: must not contain the string table name in bracket form.
        assert!(
            !toml_path.contains("tables[events]"),
            "F-LOCAL-P2-MED-001: toml_path must NOT contain string name 'tables[events]'; \
             got: '{toml_path}' — numeric-index fix may not have landed"
        );

        // Must not contain the string step name in bracket form.
        assert!(
            !toml_path.contains("steps[fetch]"),
            "F-LOCAL-P2-MED-001: toml_path must NOT contain string name 'steps[fetch]'; \
             got: '{toml_path}' — numeric-index fix may not have landed"
        );

        // Full path assertion (the definitive form).
        assert_eq!(
            toml_path, "sensor.tables[0].steps[0].method",
            "F-LOCAL-P2-MED-001: toml_path must be exactly 'sensor.tables[0].steps[0].method'; \
             got: '{toml_path}'"
        );
    } else {
        panic!("expected PrismError::Spec but got: {e_spec_025:?}");
    }
}

// ---------------------------------------------------------------------------
// F-LOCAL-P3-MED-001 — load_all numeric-index test is not load-bearing against
// hardcoded-0 regression (adversary pass-3 finding).
//
// The original test uses a single step so `[0][0]` is the only possible index —
// the assertion passes even if load_all reverted to `tables[0].steps[0]` literals.
// This test puts the invalid method on the SECOND step (index 1) so that ANY
// hardcoded 0 in the index computation produces the wrong path and fails the assertion.
//
// Test: test_BC_2_16_009_load_all_invalid_method_on_second_step_produces_steps_1
// Traces to: F-LOCAL-P3-MED-001; BC-2.16.009 §VR7; S-SPEC-HTTP-METHOD-VALIDATION-001.
// ---------------------------------------------------------------------------

/// BC-2.16.009 §VR7 — F-LOCAL-P3-MED-001 (load-bearing index test).
///
/// `SpecLoader::load_all` pointed at a temp dir containing a spec with TWO steps
/// in the same table — the FIRST step uses valid `method = "GET"`, the SECOND step
/// uses invalid `method = "CONNECT"`.
///
/// The error must carry `toml_path = "sensor.tables[0].steps[1].method"`, proving that
/// the enumerate-based index computation uses `si = 1`, not a hardcoded 0.
///
/// This test FAILS if load_all hardcodes `steps[0]` instead of computing the index:
///   - A hardcoded path `sensor.tables[0].steps[0].method` ≠ the expected `steps[1]` → panic.
///   - The original single-step test would NOT catch this regression because both
///     hardcoded-0 and computed-0 produce the same string for a single-step fixture.
///
/// Traces to: F-LOCAL-P3-MED-001; BC-2.16.009 v1.8 §VR7; S-SPEC-HTTP-METHOD-VALIDATION-001.
#[test]
fn test_BC_2_16_009_load_all_invalid_method_on_second_step_produces_steps_1() {
    // Two-step spec: step 0 = valid GET, step 1 = invalid CONNECT.
    // The toml_path for the error must be `sensor.tables[0].steps[1].method`.
    let dir = tempfile::tempdir().expect("tempdir must succeed");

    let toml_content = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  # Step 0 — valid method (must produce NO error for this step)
  [[tables.steps]]
  name = "fetch-ids"
  method = "GET"
  path_template = "/api/v1/ids"
  response_path = "$.ids"
  variables_produced = ["id"]

  # Step 1 — INVALID method (must produce E-SPEC-025 with toml_path containing steps[1])
  [[tables.steps]]
  name = "fetch-detail"
  method = "CONNECT"
  path_template = "/api/v1/detail/{fetch-ids.id}"
  response_path = "$.data"
  variables_produced = []
"#;

    let spec_file = dir.path().join("test-sensor.sensor.toml");
    std::fs::write(&spec_file, toml_content).expect("write spec file must succeed");

    let loader =
        prism_spec_engine::spec_parser::SpecLoader::new(dir.path().to_string_lossy().to_string());
    let (_descriptors, errors) = loader.load_all();

    // Must have at least one error.
    assert!(
        !errors.is_empty(),
        "F-LOCAL-P3-MED-001: load_all with second step method='CONNECT' must produce at \
         least one error; got zero — validate_step_methods call or error conversion may be broken"
    );

    // Find the E-SPEC-025 error.
    let e_spec_025 = errors.iter().find(|e| {
        matches!(
            e,
            prism_core::PrismError::Spec(prism_core::SpecError {
                code: prism_core::SpecErrorCode::ESpec025,
                ..
            })
        )
    });
    assert!(
        e_spec_025.is_some(),
        "F-LOCAL-P3-MED-001: errors must include PrismError::Spec with code ESpec025; \
         got: {errors:?}"
    );

    let e_spec_025 = e_spec_025.unwrap();
    if let prism_core::PrismError::Spec(se) = e_spec_025 {
        let toml_path = se.toml_path.as_deref().unwrap_or("");

        // The invalid step is at index 1 — this is the load-bearing assertion.
        // If load_all hardcodes `steps[0]` instead of computing the index via enumerate,
        // this assertion fails with `got: 'sensor.tables[0].steps[0].method'`.
        assert!(
            toml_path.contains("steps[1]"),
            "F-LOCAL-P3-MED-001: toml_path must contain 'steps[1]' (the second step is \
             at index 1); a hardcoded 'steps[0]' would fail here. got: '{toml_path}'"
        );

        assert_eq!(
            toml_path, "sensor.tables[0].steps[1].method",
            "F-LOCAL-P3-MED-001: toml_path must be exactly 'sensor.tables[0].steps[1].method' \
             for the second step in the first table; got: '{toml_path}'"
        );
    } else {
        panic!("expected PrismError::Spec but got: {e_spec_025:?}");
    }
}
