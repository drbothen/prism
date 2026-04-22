//! Spec file validation (BC-2.16.009).
//!
//! Performs five categories of checks in a single all-errors-collected pass:
//!   1. Schema validation (field types, regex patterns, enumerations)
//!   2. Variable reference resolution (no dangling refs, no forward refs, no self-refs)
//!   3. OCSF field validation (against embedded compiled protobuf schema)
//!   4. Pagination configuration consistency
//!   5. Rate limit hint validity
//!
//! # Key Invariant (VP-059)
//! Validation is ALWAYS a single-pass, all-errors-collected operation.
//! It NEVER returns early on the first error.
//! A spec with any errors is rejected; warnings-only specs are accepted (Ok).

use prism_core::SpecError;

use crate::spec_parser::SensorSpec;

/// A validation error that causes the spec to be rejected.
///
/// Carries an E-SPEC-* code, message, and TOML path for actionable correction.
/// Multiple errors are collected and returned together (no fail-fast).
pub type ValidationError = SpecError;

/// A validation warning that does NOT prevent the spec from loading.
///
/// Logged at startup; spec loads with warnings attached.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    /// Human-readable warning message.
    pub message: String,
    /// TOML path to the problematic field, if known.
    pub toml_path: Option<String>,
}

/// The result of `validate_sensor_spec`.
///
/// - `Ok(warnings)` — spec is valid (may have warnings); caller receives all warnings
/// - `Err(errors)` — spec is invalid; all errors collected in single pass (VP-059)
pub type ValidatorOutput = Result<Vec<ValidationWarning>, Vec<ValidationError>>;

/// Validate a parsed `SensorSpec` — all-errors-collected, no fail-fast (BC-2.16.009, VP-059).
///
/// This is a pure function: `SensorSpec -> ValidatorOutput`.
/// Same input always produces the same output (determinism invariant in VP-059).
///
/// Rules applied:
/// 1. Schema: sensor_id regex, name non-empty, auth_type enum, base_url valid URL,
///    version semver, table_name regex, at least one column per table, at least one
///    step per table, column name uniqueness, column type validity, column option validity.
/// 2. Variable references: no dangling refs, no forward refs, no self-refs.
/// 3. OCSF field paths: validated against embedded compiled schema (warning only).
/// 4. Pagination config: cursor_response_path required for cursor_token, page_size > 0 for offset_limit.
/// 5. Rate limit hints: requests_per_second > 0, burst_size >= 1.
pub fn validate_sensor_spec(spec: &SensorSpec) -> ValidatorOutput {
    unimplemented!("validate_sensor_spec — implement in S-1.11 (BC-2.16.009)")
}

/// Validate a `sensor_id` against the required regex `^[a-z][a-z0-9_-]*$`.
///
/// Returns `Some(ValidationError)` if invalid, `None` if valid.
pub fn validate_sensor_id(sensor_id: &str, file_path: Option<&str>) -> Option<ValidationError> {
    unimplemented!("validate_sensor_id — implement in S-1.11 (BC-2.16.009)")
}

/// Check all `${step_name.field}` references in a template against the step list.
///
/// Returns one `ValidationError` per dangling or forward reference found.
pub fn validate_variable_references(
    template: &str,
    template_toml_path: &str,
    all_steps: &[crate::spec_parser::FetchStep],
    current_step_index: usize,
) -> Vec<ValidationError> {
    unimplemented!("validate_variable_references — implement in S-1.11 (BC-2.16.009)")
}

/// Check `ocsf_field` paths against the embedded compiled OCSF protobuf schema.
///
/// Returns `Some(ValidationWarning)` for invalid paths — warnings do NOT reject the spec.
/// OCSF schema is embedded at compile time — NEVER fetched at runtime.
pub fn validate_ocsf_field_path(
    ocsf_field: &str,
    column_name: &str,
    toml_path: &str,
) -> Option<ValidationWarning> {
    unimplemented!("validate_ocsf_field_path — implement in S-1.11 (BC-2.16.009)")
}
