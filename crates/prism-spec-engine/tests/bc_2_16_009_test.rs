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
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec, TableSpec,
};
use prism_spec_engine::validation::{
    validate_ocsf_field_path, validate_sensor_id, validate_sensor_spec,
    validate_variable_references,
};

fn minimal_valid_spec() -> SensorSpec {
    SensorSpec {
        sensor_id: "valid-sensor".to_string(),
        name: "Valid Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: "https://api.example.com".to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_alerts".to_string(),
                method: "GET".to_string(),
                path_template: "/alerts".to_string(),
                body_template: None,
                response_path: "$.resources".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: None,
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
    }
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
    let steps = vec![FetchStep {
        name: "step_one".to_string(),
        method: "GET".to_string(),
        path_template: "/alerts?id=${nonexistent.field}".to_string(),
        body_template: None,
        response_path: "$.data".to_string(),
        pagination_cursor_path: None,
        variables_produced: vec![],
        fan_out_batch_size: None,
        pagination: None,
    }];

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
        FetchStep {
            name: "step_one".to_string(),
            method: "GET".to_string(),
            path_template: "/data?ids=${step_two.ids}".to_string(), // forward ref
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        },
        FetchStep {
            name: "step_two".to_string(),
            method: "GET".to_string(),
            path_template: "/ids".to_string(),
            body_template: None,
            response_path: "$.ids".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec!["ids".to_string()],
            fan_out_batch_size: None,
            pagination: None,
        },
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
        FetchStep {
            name: "get_token".to_string(),
            method: "POST".to_string(),
            path_template: "/oauth/token".to_string(),
            body_template: None,
            response_path: "$.access_token".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec!["access_token".to_string()],
            fan_out_batch_size: None,
            pagination: None,
        },
        FetchStep {
            name: "fetch_data".to_string(),
            method: "GET".to_string(),
            path_template: "/data?token=${get_token.access_token}".to_string(), // valid back-ref
            body_template: None,
            response_path: "$.resources".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        },
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
    spec.rate_limit_hints = Some(RateLimitHints {
        requests_per_second: Some(0.0), // invalid: must be > 0
        burst_size: None,
    });

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
    spec.rate_limit_hints = Some(RateLimitHints {
        requests_per_second: Some(10.0),
        burst_size: Some(0), // invalid: must be >= 1
    });

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
    spec.tables[0].steps.push(FetchStep {
        name: "step2".to_string(),
        method: "GET".to_string(),
        path_template: "/detail".to_string(),
        body_template: None,
        response_path: "$.data".to_string(),
        pagination_cursor_path: None,
        variables_produced: vec!["ids".to_string()],
        fan_out_batch_size: None,
        pagination: None,
    });
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
