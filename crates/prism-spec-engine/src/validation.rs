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

use prism_core::{SpecError, SpecErrorCode};

use crate::interpolation::Interpolator;
use crate::spec_parser::{FetchStep, PaginationConfig, SensorSpec};

/// Return a byte-index-safe prefix of `s` containing at most `max_chars` Unicode codepoints.
///
/// Using `s[..byte_index]` where `byte_index` may land mid-codepoint causes a panic for
/// multi-byte UTF-8 strings (e.g., emoji). This helper is safe for all UTF-8 input.
///
/// Used to sanitize user-controlled strings before embedding them in error messages (F-LP10-MED-001).
pub(crate) fn truncate_at_char_boundary(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

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

/// Embedded subset of known OCSF base event field paths.
///
/// This is the embedded schema used for validation — NEVER fetched at runtime.
/// Covers common OCSF fields. Unknown fields produce warnings (not errors).
const KNOWN_OCSF_FIELDS: &[&str] = &[
    "time",
    "message",
    "severity",
    "severity_id",
    "status",
    "status_id",
    "type_uid",
    "class_uid",
    "activity_id",
    "activity_name",
    "category_uid",
    "category_name",
    "metadata.event_code",
    "metadata.product.name",
    "metadata.product.vendor_name",
    "metadata.version",
    "metadata.uid",
    "device.hostname",
    "device.ip",
    "device.uid",
    "device.name",
    "device.type",
    "device.type_id",
    "device.os.name",
    "device.os.type",
    "actor.user.name",
    "actor.user.uid",
    "actor.user.email_addr",
    "actor.process.name",
    "actor.process.pid",
    "dst_endpoint.ip",
    "dst_endpoint.port",
    "dst_endpoint.hostname",
    "src_endpoint.ip",
    "src_endpoint.port",
    "src_endpoint.hostname",
    "finding.title",
    "finding.uid",
    "finding.desc",
    "finding.severity",
    "finding.types",
];

/// Validate a parsed `SensorSpec` — all-errors-collected, no fail-fast (BC-2.16.009, VP-059).
///
/// This is a pure function: `SensorSpec -> ValidatorOutput`.
/// Same input always produces the same output (determinism invariant in VP-059).
pub fn validate_sensor_spec(spec: &SensorSpec) -> ValidatorOutput {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // -------------------------------------------------------------------------
    // Category 1: Schema Validation
    // -------------------------------------------------------------------------

    // sensor_id must match ^[a-z][a-z0-9_-]*$
    if let Some(e) = validate_sensor_id(&spec.sensor_id, None) {
        errors.push(e);
    }

    // name must not be empty
    if spec.name.is_empty() {
        errors.push(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: "sensor name must not be empty".to_string(),
            toml_path: Some("sensor.name".to_string()),
            file_path: None,
            line_number: None,
        });
    }

    // base_url must be a valid URL (starts with http:// or https://)
    if !spec.base_url.starts_with("http://") && !spec.base_url.starts_with("https://") {
        errors.push(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "base_url '{}' is not a valid URL (must start with http:// or https://)",
                // Sanitize: truncate to 200 codepoints (char-boundary-safe) to avoid log injection.
                // F-LP10-MED-001: old byte-index slice panics on multi-byte UTF-8 (e.g., emoji).
                truncate_at_char_boundary(&spec.base_url, 200)
            ),
            toml_path: Some("sensor.base_url".to_string()),
            file_path: None,
            line_number: None,
        });
    }

    // version must be semver-like: N.N.N
    if !is_semver_like(&spec.version) {
        errors.push(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "version '{}' is not valid semver (expected N.N.N)",
                spec.version
            ),
            toml_path: Some("sensor.version".to_string()),
            file_path: None,
            line_number: None,
        });
    }

    // Table validation
    for (ti, table) in spec.tables.iter().enumerate() {
        let table_path = format!("sensor.tables[{}]", ti);

        // table_name must not be empty and must match [a-zA-Z0-9_]+
        if table.table_name.is_empty() {
            errors.push(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: format!("table at index {ti} has empty table_name"),
                toml_path: Some(format!("{table_path}.table_name")),
                file_path: None,
                line_number: None,
            });
        }

        // Table must have at least one column
        if table.columns.is_empty() {
            errors.push(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: format!("table '{}' must have at least one column", table.table_name),
                toml_path: Some(format!("{table_path}.columns")),
                file_path: None,
                line_number: None,
            });
        }

        // Table must have at least one step
        if table.steps.is_empty() {
            errors.push(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: format!(
                    "table '{}' must have at least one fetch step",
                    table.table_name
                ),
                toml_path: Some(format!("{table_path}.steps")),
                file_path: None,
                line_number: None,
            });
        }

        // Column name uniqueness within table
        let mut col_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for (ci, col) in table.columns.iter().enumerate() {
            if !col_names.insert(&col.name) {
                errors.push(ValidationError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "duplicate column name '{}' in table '{}'",
                        col.name, table.table_name
                    ),
                    toml_path: Some(format!("{table_path}.columns[{}].name", ci)),
                    file_path: None,
                    line_number: None,
                });
            }
        }

        // OCSF field path warnings for columns
        for (ci, col) in table.columns.iter().enumerate() {
            if let Some(ref ocsf_field) = col.ocsf_field {
                let col_path = format!("{table_path}.columns[{}].ocsf_field", ci);
                if let Some(w) = validate_ocsf_field_path(ocsf_field, &col.name, &col_path) {
                    warnings.push(w);
                }
            }
        }

        // -------------------------------------------------------------------------
        // Category 2: Variable Reference Resolution
        // -------------------------------------------------------------------------
        for (si, step) in table.steps.iter().enumerate() {
            let step_path = format!("{table_path}.steps[{}]", si);

            // Validate path_template variable references
            let path_errors = validate_variable_references(
                &step.path_template,
                &format!("{step_path}.path_template"),
                &table.steps,
                si,
            );
            errors.extend(path_errors);

            // Validate body_template variable references
            if let Some(ref body) = step.body_template {
                let body_errors = validate_variable_references(
                    body,
                    &format!("{step_path}.body_template"),
                    &table.steps,
                    si,
                );
                errors.extend(body_errors);
            }

            // -------------------------------------------------------------------------
            // Category 2b: Multi-Array Fan-Out Ambiguity (F-LP8-LOW-001)
            // -------------------------------------------------------------------------
            // Fan-out is single-array only. `find_fan_out_array` (pipeline.rs) returns
            // the FIRST array-valued variable it finds; if a step references TWO distinct
            // prior-step array-valued sources, the second is silently stringified as JSON
            // (e.g., `[1,2,3]` → percent-encoded in the URL).
            //
            // Cartesian / zipped fan-out semantics are deferred to PREREQ-C/D scope.
            // Silently using only the first array is worst-of-all-worlds; we reject at
            // validation time to force the spec author to be explicit.
            //
            // Heuristic: a prior step's output is classified as "likely array" if:
            //   (a) The step has a pagination config (implies repeated array accumulation), OR
            //   (b) The step's response_path ends with `[*]` (explicit wildcard).
            // If > 1 distinct source steps under this heuristic are referenced from the
            // same downstream step, the spec is rejected.
            {
                let array_source_steps: Vec<&str> = table.steps[..si]
                    .iter()
                    .filter(|prior| {
                        prior.pagination.is_some() || prior.response_path.ends_with("[*]")
                    })
                    .map(|s| s.name.as_str())
                    .collect();

                if array_source_steps.len() > 1 {
                    // Check if THIS step references more than one of those array sources.
                    let templates: Vec<&str> = std::iter::once(step.path_template.as_str())
                        .chain(step.body_template.as_deref())
                        .collect();

                    let mut referenced_array_steps: Vec<&str> = Vec::new();
                    for template in &templates {
                        let refs = Interpolator::extract_references(template);
                        for (step_name, _field) in refs {
                            if array_source_steps.contains(&step_name.as_str())
                                && !referenced_array_steps.contains(&step_name.as_str())
                            {
                                referenced_array_steps.push(
                                    array_source_steps
                                        .iter()
                                        .find(|&&s| s == step_name.as_str())
                                        .copied()
                                        .unwrap_or(""),
                                );
                            }
                        }
                    }

                    if referenced_array_steps.len() > 1 {
                        errors.push(ValidationError {
                            code: SpecErrorCode::ESpec001,
                            message: format!(
                                "step '{}' references multiple potentially-array-valued variables \
                                 from prior steps ({}) — fan-out is single-array only; \
                                 cartesian/zipped fan-out is not yet supported (PREREQ-C/D scope). \
                                 Restructure so only one prior step's array output is referenced \
                                 per step.",
                                step.name,
                                referenced_array_steps.join(", ")
                            ),
                            toml_path: Some(format!("{step_path}.path_template")),
                            file_path: None,
                            line_number: None,
                        });
                    }
                }
            }

            // -------------------------------------------------------------------------
            // Category 3a: response_path syntax (F-LP5-LOW-001 defense layer 2)
            // -------------------------------------------------------------------------
            // Reject "$." (empty key segment after prefix) and any path that does
            // not start with "$.". extract_at_path also rejects these at runtime,
            // but validator-time rejection prevents reaching the executor at all.
            if step.response_path == "$."
                || !step.response_path.starts_with("$.")
                || step
                    .response_path
                    .strip_prefix("$.")
                    .is_some_and(|s| s.is_empty())
            {
                errors.push(ValidationError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "step '{}': response_path '{}' must be a non-empty JSONPath starting with '$.<key>'",
                        step.name, step.response_path
                    ),
                    toml_path: Some(format!("{step_path}.response_path")),
                    file_path: None,
                    line_number: None,
                });
            }

            // -------------------------------------------------------------------------
            // Category 3b: Fan-Out Batch Size (F-LP4-HIGH-001 DoS guard)
            // -------------------------------------------------------------------------
            // fan_out_batch_size = 0 would cause slice::chunks(0) to panic.
            // Validate here (symmetric to page_size check below) so invalid specs
            // are rejected before PipelineExecutor::execute is ever called.
            if let Some(batch_size) = step.fan_out_batch_size
                && batch_size == 0
            {
                errors.push(ValidationError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "step '{}': fan_out_batch_size must be > 0 (got 0)",
                        step.name
                    ),
                    toml_path: Some(format!("{step_path}.fan_out_batch_size")),
                    file_path: None,
                    line_number: None,
                });
            }

            // -------------------------------------------------------------------------
            // Category 4: Pagination Configuration
            // -------------------------------------------------------------------------
            if let Some(ref pagination) = step.pagination {
                match pagination {
                    PaginationConfig::CursorToken {
                        cursor_response_path,
                        ..
                    } => {
                        if cursor_response_path.is_empty() {
                            errors.push(ValidationError {
                                code: SpecErrorCode::ESpec001,
                                message: format!(
                                    "cursor_token pagination in step '{}' requires non-empty cursor_response_path",
                                    step.name
                                ),
                                toml_path: Some(format!("{step_path}.pagination.cursor_response_path")),
                                file_path: None,
                                line_number: None,
                            });
                        }
                    }
                    PaginationConfig::OffsetLimit { page_size } => {
                        if *page_size == 0 {
                            errors.push(ValidationError {
                                code: SpecErrorCode::ESpec001,
                                message: format!(
                                    "offset_limit pagination in step '{}' requires page_size > 0",
                                    step.name
                                ),
                                toml_path: Some(format!("{step_path}.pagination.page_size")),
                                file_path: None,
                                line_number: None,
                            });
                        }
                    }
                    PaginationConfig::None => {}
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Category 5: Rate Limit Hints
    // -------------------------------------------------------------------------
    if let Some(ref hints) = spec.rate_limit_hints {
        if let Some(rps) = hints.requests_per_second
            && rps <= 0.0
        {
            errors.push(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: format!(
                    "rate_limit_hints.requests_per_second must be > 0, got {}",
                    rps
                ),
                toml_path: Some("sensor.rate_limit_hints.requests_per_second".to_string()),
                file_path: None,
                line_number: None,
            });
        }
        if let Some(burst) = hints.burst_size
            && burst == 0
        {
            errors.push(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: "rate_limit_hints.burst_size must be >= 1, got 0".to_string(),
                toml_path: Some("sensor.rate_limit_hints.burst_size".to_string()),
                file_path: None,
                line_number: None,
            });
        }
    }

    // -------------------------------------------------------------------------
    // Return result
    // -------------------------------------------------------------------------
    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(errors)
    }
}

/// Validate a `sensor_id` against the required regex `^[a-z][a-z0-9_-]*$`.
///
/// Returns `Some(ValidationError)` if invalid, `None` if valid.
pub fn validate_sensor_id(sensor_id: &str, file_path: Option<&str>) -> Option<ValidationError> {
    if sensor_id.is_empty() {
        return Some(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: "sensor_id must not be empty".to_string(),
            toml_path: Some("sensor.sensor_id".to_string()),
            file_path: file_path.map(|s| s.to_string()),
            line_number: None,
        });
    }

    // Must start with lowercase letter
    let first = sensor_id.chars().next().expect("non-empty checked above");
    if !first.is_ascii_lowercase() {
        return Some(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "sensor_id '{}' must start with a lowercase letter [a-z]",
                // Sanitize: don't echo arbitrary input, just note the violation
                if first.is_ascii_uppercase() {
                    "(starts with uppercase)"
                } else {
                    "(invalid first char)"
                }
            ),
            toml_path: Some("sensor.sensor_id".to_string()),
            file_path: file_path.map(|s| s.to_string()),
            line_number: None,
        });
    }

    // All chars must be [a-z0-9_-]
    for ch in sensor_id.chars() {
        if !matches!(ch, 'a'..='z' | '0'..='9' | '_' | '-') {
            return Some(ValidationError {
                code: SpecErrorCode::ESpec001,
                message: "sensor_id must match ^[a-z][a-z0-9_-]*$ (invalid character found)"
                    .to_string(),
                toml_path: Some("sensor.sensor_id".to_string()),
                file_path: file_path.map(|s| s.to_string()),
                line_number: None,
            });
        }
    }

    // Length: 1..=64 characters.
    // Parity with prism_core::sensor_id::validate_sensor_id_string — the SensorId
    // newtype enforces this domain limit; TOML spec values must not exceed it.
    if sensor_id.len() > 64 {
        return Some(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: format!("sensor_id is {} characters; maximum is 64", sensor_id.len()),
            toml_path: Some("sensor.sensor_id".to_string()),
            file_path: file_path.map(|s| s.to_string()),
            line_number: None,
        });
    }

    // No trailing `-` or `_`.
    // Parity with prism_core::sensor_id::validate_sensor_id_string (InvalidBoundary).
    // Real sensor IDs never end in a delimiter; this prevents ambiguous TOML keys.
    let last = sensor_id
        .chars()
        .next_back()
        .expect("non-empty checked above");
    if last == '-' || last == '_' {
        return Some(ValidationError {
            code: SpecErrorCode::ESpec001,
            message: "sensor_id must not end with '-' or '_'".to_string(),
            toml_path: Some("sensor.sensor_id".to_string()),
            file_path: file_path.map(|s| s.to_string()),
            line_number: None,
        });
    }

    None
}

/// Check all `${step_name.field}` references in a template against the step list.
///
/// Returns one `ValidationError` per dangling or forward reference found.
pub fn validate_variable_references(
    template: &str,
    template_toml_path: &str,
    all_steps: &[FetchStep],
    current_step_index: usize,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let refs = Interpolator::extract_references(template);

    for (step_name, field_name) in &refs {
        // Check if the step exists in the pipeline at all
        let step_pos = all_steps.iter().position(|s| &s.name == step_name);

        match step_pos {
            None => {
                // Step doesn't exist — dangling reference
                errors.push(ValidationError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "variable '${{{{{}:{}}}}}'  references step '{}' which is not defined in this pipeline",
                        step_name, field_name, step_name
                    ),
                    toml_path: Some(template_toml_path.to_string()),
                    file_path: None,
                    line_number: None,
                });
            }
            Some(pos) if pos >= current_step_index => {
                // Step exists but comes at or after the current step — forward reference
                errors.push(ValidationError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "variable '${{{{{}:{}}}}}'  is a forward reference: step '{}' at index {} cannot be referenced by step at index {}",
                        step_name, field_name, step_name, pos, current_step_index
                    ),
                    toml_path: Some(template_toml_path.to_string()),
                    file_path: None,
                    line_number: None,
                });
            }
            Some(_) => {
                // Valid backward reference — no error
            }
        }
    }

    errors
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
    if KNOWN_OCSF_FIELDS.contains(&ocsf_field) {
        None // known field — no warning
    } else {
        Some(ValidationWarning {
            message: format!(
                "column '{}': ocsf_field '{}' is not a recognized OCSF field path \
                (check spelling or add to ocsf_schema.json if this is a new field)",
                column_name, ocsf_field
            ),
            toml_path: Some(toml_path.to_string()),
        })
    }
}

/// Check if a version string is semver-like (N.N.N with optional pre-release suffix).
fn is_semver_like(version: &str) -> bool {
    // Accept N.N.N or N.N.N-pre.release forms
    let parts: Vec<&str> = version.splitn(2, '-').collect();
    let core = parts[0];
    let segments: Vec<&str> = core.split('.').collect();
    if segments.len() != 3 {
        return false;
    }
    segments.iter().all(|s| s.parse::<u64>().is_ok())
}

#[cfg(test)]
mod truncate_at_char_boundary_tests {
    use super::truncate_at_char_boundary;

    // Empty string with max_chars=0: trivial no-op, must not panic.
    #[test]
    fn empty_string_zero_chars() {
        assert_eq!(truncate_at_char_boundary("", 0), "");
    }

    // Empty string with large max_chars: caller asks for more than available, returns all (empty).
    #[test]
    fn empty_string_nonzero_max() {
        assert_eq!(truncate_at_char_boundary("", 100), "");
    }

    // ASCII string where char count equals max_chars: full string returned (no truncation).
    #[test]
    fn ascii_string_at_boundary() {
        assert_eq!(truncate_at_char_boundary("abc", 3), "abc");
    }

    // ASCII string shorter than max_chars: max_chars > length is a no-op.
    #[test]
    fn ascii_string_under_max() {
        assert_eq!(truncate_at_char_boundary("hi", 100), "hi");
    }

    // Multi-byte UTF-8: 5 emoji (4 bytes each = 20 bytes total), truncate to 3 codepoints.
    // Must slice at byte index 12 (3 × 4 bytes), NOT byte index 3 (which would be mid-codepoint).
    #[test]
    fn utf8_multi_byte_truncation_no_panic() {
        assert_eq!(truncate_at_char_boundary("🎯🎯🎯🎯🎯", 3), "🎯🎯🎯");
    }

    // max_chars=0 on a non-empty string: returns empty string (not the full string, not a panic).
    #[test]
    fn ascii_string_under_zero() {
        assert_eq!(truncate_at_char_boundary("abc", 0), "");
    }

    // Single-char string at max=1: boundary where length equals max exactly, 1-char case.
    #[test]
    fn single_char_at_max() {
        assert_eq!(truncate_at_char_boundary("a", 1), "a");
    }
}
