//! VP-059 proptest harness: Spec Validator — All Errors Collected, No Fail-Fast.
//!
//! Property statement (VP-059):
//!   For any SensorSpec with N distinct validation errors (N >= 1),
//!   `validate_sensor_spec(spec)` returns `Err(errors)` where `errors.len() == N`.
//!   The validator never returns early on the first error.
//!   For a spec with only warnings and no errors, returns `Ok(warnings)`.
//!   The function is deterministic: same input -> same output.
//!
//! Source BC: BC-2.16.009
//! Method: proptest
//! Priority: P1
//!
//! NOTE: These tests are Red Gate tests — they MUST FAIL until the implementation
//! of `validate_sensor_spec` is complete.

#![cfg(test)]
#![allow(non_snake_case)]

use proptest::prelude::*;

use crate::spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, SensorSpec, TableSpec,
};
use crate::validation::validate_sensor_spec;
use prism_core::ColumnType;

// ---------------------------------------------------------------------------
// Test vector builders: construct SensorSpec values with exactly N known errors.
//
// Each error is sourced from a distinct validation rule in BC-2.16.009.
// Rule index:
//   1 — invalid sensor_id (starts with digit, violates ^[a-z][a-z0-9_-]*$)
//   2 — forward variable reference (step 1 references step 2's output)
//   3 — duplicate column name within a table
//   4 — empty table (no columns)
//   5 — empty table (no steps)
//   6 — invalid auth_type — not representable via enum; use invalid base_url instead
//   6 — invalid base_url (not a valid URL)
//   7 — empty sensor name
//   8 — empty table name
//   9 — column type mismatch (handled via invalid version — not semver)
// ---------------------------------------------------------------------------

/// A minimal valid SensorSpec used as a baseline.
fn minimal_valid_spec() -> SensorSpec {
    SensorSpec {
        sensor_id: "valid-sensor".to_string(),
        name: "Valid Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: "https://api.example.com".to_string(),
        tables: vec![TableSpec {
            table_name: "alerts".to_string(),
            ocsf_class: "security_finding".to_string(),
            columns: vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            steps: vec![FetchStep {
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
        }],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
    }
}

/// Rule 1: inject invalid sensor_id (starts with digit).
fn inject_error_invalid_sensor_id(spec: &mut SensorSpec) {
    spec.sensor_id = "1invalid-id".to_string();
}

/// Rule 2: inject forward variable reference (step[0] references step[1]).
fn inject_error_forward_variable_ref(spec: &mut SensorSpec) {
    if let Some(table) = spec.tables.first_mut() {
        // Add a second step
        table.steps.push(FetchStep {
            name: "step_two".to_string(),
            method: "GET".to_string(),
            path_template: "/details".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec!["step_two_ids".to_string()],
            fan_out_batch_size: None,
            pagination: None,
        });
        // Step[0] references step[1] — forward reference error
        table.steps[0].path_template = "/alerts?ids=${step_two.step_two_ids}".to_string();
    }
}

/// Rule 3: inject duplicate column name within a table.
fn inject_error_duplicate_column_name(spec: &mut SensorSpec) {
    if let Some(table) = spec.tables.first_mut() {
        let duplicate = table.columns[0].clone();
        table.columns.push(duplicate);
    }
}

/// Rule 4: inject empty table (no columns).
fn inject_error_empty_table_no_columns(spec: &mut SensorSpec) {
    spec.tables.push(TableSpec {
        table_name: "empty_table".to_string(),
        ocsf_class: "base_event".to_string(),
        columns: vec![], // violation: must have at least one column
        steps: vec![FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/data".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }],
    });
}

/// Rule 5: inject empty table (no steps).
fn inject_error_empty_table_no_steps(spec: &mut SensorSpec) {
    spec.tables.push(TableSpec {
        table_name: "nostep_table".to_string(),
        ocsf_class: "base_event".to_string(),
        columns: vec![ColumnSpec {
            name: "col".to_string(),
            column_type: ColumnType::String,
            ocsf_field: None,
            options: vec![],
        }],
        steps: vec![], // violation: must have at least one step
    });
}

/// Rule 6: inject invalid base_url.
fn inject_error_invalid_base_url(spec: &mut SensorSpec) {
    spec.base_url = "not-a-valid-url".to_string();
}

/// Rule 7: inject empty sensor name.
fn inject_error_empty_sensor_name(spec: &mut SensorSpec) {
    spec.name = "".to_string();
}

/// Rule 8: inject empty table name.
fn inject_error_empty_table_name(spec: &mut SensorSpec) {
    spec.tables.push(TableSpec {
        table_name: "".to_string(), // violation: table_name must match [a-zA-Z0-9_]+
        ocsf_class: "base_event".to_string(),
        columns: vec![ColumnSpec {
            name: "col".to_string(),
            column_type: ColumnType::String,
            ocsf_field: None,
            options: vec![],
        }],
        steps: vec![FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/data".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }],
    });
}

/// Rule 9: inject invalid version (not semver).
fn inject_error_invalid_version(spec: &mut SensorSpec) {
    spec.version = "not-semver".to_string();
}

/// Rule 10: cursor_token pagination without cursor_response_path.
fn inject_error_cursor_pagination_missing_path(spec: &mut SensorSpec) {
    if let Some(table) = spec.tables.first_mut() {
        if let Some(step) = table.steps.first_mut() {
            step.pagination = Some(PaginationConfig::CursorToken {
                cursor_response_path: "".to_string(), // empty = invalid
            });
        }
    }
}

/// Apply error injectors in order, up to `n_errors` injectors.
/// Each injector introduces exactly one distinct validation error.
fn build_spec_with_n_errors(n_errors: usize) -> SensorSpec {
    let injectors: &[fn(&mut SensorSpec)] = &[
        inject_error_invalid_sensor_id,
        inject_error_forward_variable_ref,
        inject_error_duplicate_column_name,
        inject_error_empty_table_no_columns,
        inject_error_empty_table_no_steps,
        inject_error_invalid_base_url,
        inject_error_empty_sensor_name,
        inject_error_empty_table_name,
        inject_error_invalid_version,
        inject_error_cursor_pagination_missing_path,
    ];
    assert!(
        n_errors <= injectors.len(),
        "n_errors {} exceeds available injectors {}",
        n_errors,
        injectors.len()
    );
    let mut spec = minimal_valid_spec();
    for injector in injectors.iter().take(n_errors) {
        injector(&mut spec);
    }
    spec
}

/// Build a spec with warnings only (invalid OCSF field paths → warnings, not errors).
fn build_spec_with_n_warnings_no_errors(n_warnings: usize) -> SensorSpec {
    let mut spec = minimal_valid_spec();
    // Add columns with invalid OCSF field paths (warning, not error per BC-2.16.009 Rule 3)
    if let Some(table) = spec.tables.first_mut() {
        for i in 0..n_warnings {
            table.columns.push(ColumnSpec {
                name: format!("warn_col_{i}"),
                column_type: ColumnType::String,
                ocsf_field: Some(format!("nonexistent.field_{i}")),
                options: vec![],
            });
        }
    }
    spec
}

// ---------------------------------------------------------------------------
// VP-059: proptest — all errors collected, no fail-fast
// ---------------------------------------------------------------------------

proptest! {
    /// VP-059 variant 1: for any N in 1..=10, spec with N errors returns Err with len==N.
    ///
    /// This directly proves the "no fail-fast" invariant from BC-2.16.009 and VP-059.
    /// test_BC_2_16_009_invariant_all_errors_collected
    #[test]
    fn test_BC_2_16_009_invariant_all_errors_collected(
        n_errors in 1usize..=10usize
    ) {
        let spec = build_spec_with_n_errors(n_errors);
        let result = validate_sensor_spec(&spec);

        prop_assert!(
            result.is_err(),
            "spec with {} errors must return Err, got Ok",
            n_errors
        );

        let errors = result.unwrap_err();
        prop_assert_eq!(
            errors.len(),
            n_errors,
            "all {} errors must be collected (no fail-fast); validator returned {} errors",
            n_errors,
            errors.len()
        );
    }

    /// VP-059 variant 2: warning-only spec returns Ok (spec accepted).
    ///
    /// test_BC_2_16_009_invariant_warning_only_returns_ok
    #[test]
    fn test_BC_2_16_009_invariant_warning_only_returns_ok(
        n_warnings in 1usize..=5usize
    ) {
        let spec = build_spec_with_n_warnings_no_errors(n_warnings);
        let result = validate_sensor_spec(&spec);

        prop_assert!(
            result.is_ok(),
            "warning-only spec must return Ok (spec accepted with {} warnings), got Err",
            n_warnings
        );

        let warnings = result.unwrap();
        prop_assert_eq!(
            warnings.len(),
            n_warnings,
            "all {} warnings must be present in Ok variant; got {}",
            n_warnings,
            warnings.len()
        );
    }

    /// VP-059 variant 3: determinism — same input, same output.
    ///
    /// test_BC_2_16_009_invariant_deterministic
    #[test]
    fn test_BC_2_16_009_invariant_deterministic(
        n_errors in 0usize..=10usize
    ) {
        let spec = if n_errors == 0 {
            minimal_valid_spec()
        } else {
            build_spec_with_n_errors(n_errors)
        };

        let result1 = validate_sensor_spec(&spec);
        let result2 = validate_sensor_spec(&spec);

        match (result1, result2) {
            (Ok(w1), Ok(w2)) => prop_assert_eq!(w1.len(), w2.len(), "determinism violation: Ok results differ"),
            (Err(e1), Err(e2)) => prop_assert_eq!(e1.len(), e2.len(), "determinism violation: Err results differ"),
            _ => prop_assert!(false, "determinism violation: one call returned Ok, other returned Err"),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests for individual validation rules (BC-2.16.009)
// These are NOT proptest — they exercise specific canonical vectors.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::validation::{validate_sensor_id, validate_variable_references};

    /// BC-2.16.009 schema validation: sensor_id starting with digit -> E-SPEC-001.
    #[test]
    fn test_BC_2_16_009_rejects_sensor_id_starting_with_digit() {
        let result = validate_sensor_id("1invalid", Some("test.sensor.toml"));
        assert!(
            result.is_some(),
            "sensor_id '1invalid' must be rejected (starts with digit)"
        );
        let err = result.unwrap();
        assert!(
            err.toml_path.as_deref() == Some("sensor.sensor_id")
                || err.message.contains("sensor_id"),
            "error must reference sensor_id path: {:?}",
            err
        );
    }

    /// BC-2.16.009 schema validation: valid sensor_id accepted.
    #[test]
    fn test_BC_2_16_009_accepts_valid_sensor_id() {
        let result = validate_sensor_id("valid-sensor-01", None);
        assert!(
            result.is_none(),
            "valid sensor_id must not produce an error"
        );
    }

    /// BC-2.16.009 schema validation: sensor_id with uppercase -> E-SPEC-001.
    #[test]
    fn test_BC_2_16_009_rejects_sensor_id_with_uppercase() {
        let result = validate_sensor_id("CrowdStrike", Some("crowdstrike.sensor.toml"));
        assert!(
            result.is_some(),
            "sensor_id 'CrowdStrike' must be rejected (contains uppercase)"
        );
    }

    /// BC-2.16.009 variable reference: forward reference -> E-SPEC-001.
    #[test]
    fn test_BC_2_16_009_rejects_forward_variable_reference() {
        let steps = vec![
            FetchStep {
                name: "step_one".to_string(),
                method: "GET".to_string(),
                path_template: "/alerts?ids=${step_two.ids}".to_string(), // forward ref
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
                path_template: "/details".to_string(),
                body_template: None,
                response_path: "$.data".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec!["ids".to_string()],
                fan_out_batch_size: None,
                pagination: None,
            },
        ];

        let errors = validate_variable_references(
            "/alerts?ids=${step_two.ids}",
            "sensor.tables[0].steps[0].path_template",
            &steps,
            0, // step_one is at index 0, step_two is at index 1 -> forward ref
        );

        assert!(
            !errors.is_empty(),
            "forward variable reference must produce at least one error"
        );
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("step_two") || e.message.contains("forward")),
            "error must mention the forward-referenced step: {:?}",
            errors
        );
    }

    /// BC-2.16.009 variable reference: dangling reference -> E-SPEC-001.
    #[test]
    fn test_BC_2_16_009_rejects_dangling_variable_reference() {
        let steps = vec![FetchStep {
            name: "step_one".to_string(),
            method: "GET".to_string(),
            path_template: "/data/${nonexistent.field}".to_string(), // dangling ref
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }];

        let errors = validate_variable_references(
            "/data/${nonexistent.field}",
            "sensor.tables[0].steps[0].path_template",
            &steps,
            0,
        );

        assert!(
            !errors.is_empty(),
            "dangling variable reference must produce at least one error"
        );
        assert!(
            errors.iter().any(|e| e.message.contains("nonexistent")),
            "error must mention the undefined step: {:?}",
            errors
        );
    }

    /// BC-2.16.009 multi-error: all errors in one file reported together.
    /// Canonical test vector: invalid sensor_id + forward reference = 2 errors.
    #[test]
    fn test_BC_2_16_009_reports_multiple_errors_together() {
        let spec = build_spec_with_n_errors(2);
        let result = validate_sensor_spec(&spec);
        assert!(result.is_err(), "spec with 2 errors must return Err");
        let errors = result.unwrap_err();
        assert_eq!(
            errors.len(),
            2,
            "exactly 2 errors must be reported together (no fail-fast); got {:?}",
            errors
        );
    }

    /// BC-2.16.009 valid spec: no errors, no warnings -> Ok(empty).
    #[test]
    fn test_BC_2_16_009_accepts_valid_spec_clean() {
        let spec = minimal_valid_spec();
        let result = validate_sensor_spec(&spec);
        assert!(
            result.is_ok(),
            "valid spec must return Ok, got Err: {:?}",
            result.unwrap_err()
        );
        let warnings = result.unwrap();
        assert!(
            warnings.is_empty(),
            "valid spec with no OCSF warnings must produce empty warnings"
        );
    }
}
