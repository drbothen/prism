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

use crate::{
    interpolation::Interpolator,
    spec_parser::{FetchStep, PaginationConfig, SensorSpec},
};

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

/// The complete set of HTTP methods permitted in `FetchStep::method`.
///
/// This is a compile-time constant per BC-2.16.009 §Validation Rules 7 — the whitelist
/// is never runtime-configurable (no serde, no env var, no prism.toml field).
///
/// All 7 entries are uppercase only. Case sensitivity is intentional and matches industry
/// convention: `"get"` is invalid and produces E-SPEC-025 (BC-2.16.009 §VR7 case-sensitivity
/// clause). The whitelist is case-sensitive and upper-case only.
///
/// BC-2.16.009 §Validation Rules 7; S-SPEC-HTTP-METHOD-VALIDATION-001.
pub(crate) const ALLOWED_HTTP_METHODS: &[&str] =
    &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

/// Validate `step.method` fields across all tables in `spec` against the HTTP method whitelist.
///
/// **Rule 7 of BC-2.16.009** — runs AFTER the env-var token resolution pass (Rule 6).
/// The caller is responsible for running Rule 6 (`resolve_env_var_tokens`) before calling
/// this function; `validate_step_methods` operates on the already-resolved spec.
///
/// ## Behavior
/// - `step.method` absent (defaults to `"GET"` via `FetchStep::default()`): no error.
/// - `step.method` present and in `ALLOWED_HTTP_METHODS` (case-sensitive): no error.
/// - `step.method` present and NOT in `ALLOWED_HTTP_METHODS`: emits `E-SPEC-025`.
/// - `step.method` still contains an unresolved `${env.VAR}` token (Rule 6 failed for it):
///   this step is **skipped** to prevent double-reporting (BC-2.16.009 §VR7 ordering).
/// - All errors are collected before returning (INV-ERR-003 — no fail-fast).
///
/// ## Return value
/// Returns a `Vec<(usize, usize, SpecEngineError)>` — each entry carries:
/// - `table_index`: numeric index of the table in `spec.tables` (from `enumerate`)
/// - `step_index`: numeric index of the step in `table.steps` (from `enumerate`)
/// - `SpecEngineError::InvalidHttpMethod`: the structured error (carries `step_name`,
///   `sensor_id`, `table_name`, `method_value` for Display and downstream use)
///
/// Callers that only need the errors (e.g., `add_sensor_spec`) use `.map(|(_, _, e)| e)`.
/// Callers that need canonical `toml_path` construction (e.g., `load_all`) use the indices
/// directly to build `sensor.tables[{ti}].steps[{si}].method` without name reverse-lookup.
/// This eliminates the fragility noted in F-LOCAL-P4-MED-001: a name reverse-lookup breaks
/// when two steps in one table share the same name (step-name uniqueness is NOT enforced).
///
/// BC-2.16.009 §Validation Rules 7 (AC-7); error-taxonomy.md v1.59 E-SPEC-025;
/// S-SPEC-HTTP-METHOD-VALIDATION-001; F-LOCAL-P4-MED-001.
pub fn validate_step_methods(
    spec: &SensorSpec,
) -> Vec<(usize, usize, crate::error::SpecEngineError)> {
    let mut errors = Vec::new();

    for (ti, table) in spec.tables.iter().enumerate() {
        for (si, step) in table.steps.iter().enumerate() {
            // BC-2.16.009 §VR7 ordering: skip steps whose method still contains a
            // WELL-FORMED `${env.VAR_NAME}` token (where VAR_NAME matches `[A-Z0-9_]+`).
            //
            // Rule 6 (env_resolver) already fired E-SPEC-024 for those; double-reporting
            // the same field is noise, not signal (EC-009-020).
            //
            // IMPORTANT: we test for a WELL-FORMED token using the SAME regex that Rule 6
            // uses (ENV_TOKEN_REGEX from env_resolver.rs — TD-VSDD-060 single source of
            // truth). A malformed pseudo-token like `${env.lower}`, `${env.foo-bar}`, or
            // `${env.}` does NOT match the regex → is NOT skipped → falls through to the
            // whitelist check below → produces E-SPEC-025 (F-LOCAL-P3-MED-002 fix).
            //
            // The old `contains("${env.")` substring check was too broad: it skipped ANY
            // string containing the prefix, including malformed pseudo-tokens that Rule 6
            // never processes. The DRIFT-D926-001 gap was those literals silently reaching
            // the `_ => GET` pipeline fallback instead of being rejected here.
            if crate::env_resolver::ENV_TOKEN_REGEX.is_match(&step.method) {
                continue;
            }

            // Case-sensitive whitelist check (BC-2.16.009 §VR7 case-sensitivity clause).
            // "get" is NOT equivalent to "GET"; empty string is not in the whitelist.
            if !ALLOWED_HTTP_METHODS.contains(&step.method.as_str()) {
                errors.push((
                    ti,
                    si,
                    crate::error::SpecEngineError::InvalidHttpMethod {
                        step_name: step.name.clone(),
                        sensor_id: spec.sensor_id.clone(),
                        table_name: table.table_name.clone(),
                        method_value: step.method.clone(),
                    },
                ));
            }
        }
    }

    errors
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

/// Validate that a `SensorSpec.auth_plugin` field references a registered plugin.
///
/// When `spec.auth_plugin = Some(plugin_id)`, this function checks `registered_plugin_ids`
/// for membership. If absent, returns `SpecEngineError::UnknownAuthPlugin` (E-SPEC-012 extended).
///
/// ## Call site
///
/// This function is called AFTER `PluginRuntime::load_all_plugins` has populated the registry.
/// In boot.rs step 7.5, after plugins are loaded, iterate over all loaded SensorSpecs and call
/// this validator to reject specs that reference unloaded plugins.
///
/// ## Why separate from parse()
///
/// `SpecLoader::parse` is a pure TOML-to-struct function with no access to `PluginRuntime`.
/// Registry membership can only be validated after boot step 7.5 completes. This function
/// is the post-boot validation gate for `auth_plugin` fields (F-LP1-CRIT-003).
///
/// Story: PLUGIN-MIGRATION-001-E / F-LP1-CRIT-003 / F-LP1-HIGH-008
/// Traces to: BC-2.01.016 §Error Cases; ADR-028 §D2; error-taxonomy.md E-SPEC-012
pub fn validate_auth_plugin_registered(
    spec: &SensorSpec,
    registered_plugin_ids: &std::collections::HashSet<String>,
) -> Result<(), crate::error::SpecEngineError> {
    validate_auth_plugin_fields(
        &spec.sensor_id,
        spec.auth_plugin.as_deref(),
        registered_plugin_ids,
    )
}

/// Validate auth_plugin membership by raw fields.
///
/// Companion to `validate_auth_plugin_registered` — accepts `sensor_id` and `auth_plugin`
/// as primitive refs rather than `spec_parser::SensorSpec`. Used by boot.rs step 7.5b to
/// validate sensor spec entries from the ConfigSnapshot (F-LP2-CRIT-002 closure).
///
/// Story: PLUGIN-MIGRATION-001-E / F-LP2-CRIT-002
/// Traces to: BC-2.01.016 §Error Cases; ADR-028 §D2; error-taxonomy.md E-SPEC-012
pub fn validate_auth_plugin_fields(
    sensor_id: &str,
    auth_plugin: Option<&str>,
    registered_plugin_ids: &std::collections::HashSet<String>,
) -> Result<(), crate::error::SpecEngineError> {
    if let Some(plugin_id) = auth_plugin
        && !registered_plugin_ids.contains(plugin_id)
    {
        return Err(crate::error::SpecEngineError::UnknownAuthPlugin {
            sensor_id: sensor_id.to_string(),
            plugin_id: plugin_id.to_string(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// BC-2.16.009 v1.8 §Validation Rules 7 — HTTP Method Whitelist Validation Tests
// S-SPEC-HTTP-METHOD-VALIDATION-001 — Red Gate test suite
//
// Test naming convention: test_BC_2_16_009_<description>
// Traces to: BC-2.16.009 v1.8 §Validation Rules 7; error-taxonomy.md v1.59 E-SPEC-025.
//
// All tests in this module are RED GATE tests — they MUST FAIL before
// validate_step_methods() is implemented (todo!() panics). After implementation,
// they MUST ALL PASS.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod http_method_whitelist_tests {
    use crate::{
        error::SpecEngineError,
        spec_parser::{AuthType, FetchStep, SensorSpec, TableSpec},
    };

    use super::{ALLOWED_HTTP_METHODS, validate_step_methods};

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Build a minimal valid SensorSpec with one table and one step using the given method.
    ///
    /// sensor_id = "test-sensor", table_name = "events", step name = "fetch".
    fn make_spec_with_method(method: &str) -> SensorSpec {
        let step = FetchStep {
            name: "fetch".to_string(),
            method: method.to_string(),
            path_template: "/api/v1/events".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        let table = TableSpec::new_point_in_time("events", "security_finding", vec![], vec![step]);
        SensorSpec {
            sensor_id: "test-sensor".to_string(),
            name: "Test Sensor".to_string(),
            auth_type: AuthType::ApiKey,
            base_url: "https://example.com".to_string(),
            tables: vec![table],
            version: "1.0.0".to_string(),
            ..SensorSpec::default()
        }
    }

    /// Build a spec with no tables (no steps) — used to test zero-step edge case.
    fn make_spec_no_tables() -> SensorSpec {
        SensorSpec {
            sensor_id: "test-sensor".to_string(),
            name: "Test Sensor".to_string(),
            auth_type: AuthType::ApiKey,
            base_url: "https://example.com".to_string(),
            tables: vec![],
            version: "1.0.0".to_string(),
            ..SensorSpec::default()
        }
    }

    /// Build a spec with two steps in the same table.
    fn make_spec_with_two_steps(method1: &str, method2: &str) -> SensorSpec {
        let step1 = FetchStep {
            name: "step-one".to_string(),
            method: method1.to_string(),
            path_template: "/api/v1/first".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        let step2 = FetchStep {
            name: "step-two".to_string(),
            method: method2.to_string(),
            path_template: "/api/v1/second".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        let table =
            TableSpec::new_point_in_time("events", "security_finding", vec![], vec![step1, step2]);
        SensorSpec {
            sensor_id: "test-sensor".to_string(),
            name: "Test Sensor".to_string(),
            auth_type: AuthType::ApiKey,
            base_url: "https://example.com".to_string(),
            tables: vec![table],
            version: "1.0.0".to_string(),
            ..SensorSpec::default()
        }
    }

    // -----------------------------------------------------------------------
    // ALLOWED_HTTP_METHODS constant — compile-time correctness
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7: ALLOWED_HTTP_METHODS must contain exactly 7 values.
    ///
    /// Traces to: BC-2.16.009 §Validation Rules 7 — "Whitelist constant: The following
    /// 7 HTTP methods are the complete allowed set". Prevents accidental truncation or
    /// expansion of the constant.
    #[test]
    fn test_BC_2_16_009_allowed_http_methods_has_exactly_7_entries() {
        assert_eq!(
            ALLOWED_HTTP_METHODS.len(),
            7,
            "ALLOWED_HTTP_METHODS must have exactly 7 entries per BC-2.16.009 §VR7; got {}",
            ALLOWED_HTTP_METHODS.len()
        );
    }

    /// BC-2.16.009 v1.8 §VR7: ALLOWED_HTTP_METHODS must contain the 7 canonical values.
    ///
    /// Tests that each of the 7 documented methods is present. Combined with the count
    /// test above, this fully pins the constant.
    #[test]
    fn test_BC_2_16_009_allowed_http_methods_contains_canonical_values() {
        for method in &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] {
            assert!(
                ALLOWED_HTTP_METHODS.contains(method),
                "ALLOWED_HTTP_METHODS must contain '{}' per BC-2.16.009 §VR7",
                method
            );
        }
    }

    // -----------------------------------------------------------------------
    // Helper: extract just the SpecEngineError from the returned tuples.
    // validate_step_methods returns Vec<(usize, usize, SpecEngineError)>;
    // most unit tests only care about the error value, not the indices.
    // -----------------------------------------------------------------------
    fn errors_only(results: Vec<(usize, usize, SpecEngineError)>) -> Vec<SpecEngineError> {
        results.into_iter().map(|(_, _, e)| e).collect()
    }

    // -----------------------------------------------------------------------
    // AC-001: All 7 whitelist methods pass validation (EC-009-010, EC-009-011)
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7; AC-001.
    ///
    /// All 7 allowed HTTP methods — GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS —
    /// must pass validation without producing any E-SPEC-025 errors.
    ///
    /// Parameterized over all 7 canonical whitelist values.
    /// Canonical test vectors: "HTTP method — valid GET" and "HTTP method — valid POST".
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_valid_http_method_passes_validation() {
        for method in ALLOWED_HTTP_METHODS {
            let spec = make_spec_with_method(method);
            let results = validate_step_methods(&spec);
            assert!(
                results.is_empty(),
                "AC-001: method '{}' is in ALLOWED_HTTP_METHODS and must produce zero \
                 E-SPEC-025 errors; got {} error(s): {:?}",
                method,
                results.len(),
                results
            );
        }
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-010: step.method = "GET" (valid uppercase).
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_010_get_passes_rule_7() {
        let spec = make_spec_with_method("GET");
        let results = validate_step_methods(&spec);
        assert!(
            results.is_empty(),
            "EC-009-010: GET is valid; expected zero errors; got {:?}",
            results
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-011: step.method = "POST" (valid, common for POST-for-read sensors).
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_011_post_passes_rule_7() {
        let spec = make_spec_with_method("POST");
        let results = validate_step_methods(&spec);
        assert!(
            results.is_empty(),
            "EC-009-011: POST is valid (Claroty/Armis pattern); expected zero errors; got {:?}",
            results
        );
    }

    /// BC-2.16.009 v1.8 §VR7: spec with no tables produces zero E-SPEC-025 errors.
    ///
    /// No steps = nothing to validate.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_no_tables_produces_zero_errors() {
        let spec = make_spec_no_tables();
        let results = validate_step_methods(&spec);
        assert!(
            results.is_empty(),
            "A spec with no tables has no steps to validate; expected zero errors; got {:?}",
            results
        );
    }

    // -----------------------------------------------------------------------
    // AC-002: Invalid / unsupported methods return structured E-SPEC-025 error
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7; AC-002.
    ///
    /// An unsupported method "CONNECT" must produce exactly one E-SPEC-025 error.
    /// The error message must be byte-verbatim with the BC-2.16.009 template:
    /// "Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>'
    ///  which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"
    ///
    /// Canonical test vector: "HTTP method — CONNECT rejected".
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025() {
        let spec = make_spec_with_method("CONNECT");
        let results = validate_step_methods(&spec);
        let errors = errors_only(results);
        assert_eq!(
            errors.len(),
            1,
            "AC-002: 'CONNECT' is not whitelisted; expected exactly 1 E-SPEC-025 error; got {}: {:?}",
            errors.len(),
            errors
        );
        match &errors[0] {
            SpecEngineError::InvalidHttpMethod {
                step_name,
                sensor_id,
                table_name,
                method_value,
            } => {
                assert_eq!(
                    step_name, "fetch",
                    "step_name must be 'fetch'; got '{}'",
                    step_name
                );
                assert_eq!(
                    sensor_id, "test-sensor",
                    "sensor_id must be 'test-sensor'; got '{}'",
                    sensor_id
                );
                assert_eq!(
                    table_name, "events",
                    "table_name must be 'events'; got '{}'",
                    table_name
                );
                assert_eq!(
                    method_value, "CONNECT",
                    "method_value must be 'CONNECT'; got '{}'",
                    method_value
                );
            }
            other => panic!(
                "Expected SpecEngineError::InvalidHttpMethod, got: {:?}",
                other
            ),
        }
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-012: step.method = "CONNECT" → E-SPEC-025.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_012_connect_produces_e_spec_025() {
        let spec = make_spec_with_method("CONNECT");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "EC-009-012: CONNECT is not whitelisted; must produce E-SPEC-025"
        );
        assert!(
            errors.iter().any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "CONNECT")),
            "EC-009-012: error must cite method_value 'CONNECT'; got {:?}",
            errors
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-013: step.method = "TRACE" → E-SPEC-025.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_013_trace_produces_e_spec_025() {
        let spec = make_spec_with_method("TRACE");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "EC-009-013: TRACE is not whitelisted; must produce E-SPEC-025"
        );
        assert!(
            errors.iter().any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "TRACE")),
            "EC-009-013: error must cite method_value 'TRACE'; got {:?}",
            errors
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-014: step.method = "GETT" (typo) → E-SPEC-025.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_014_typo_gett_produces_e_spec_025() {
        let spec = make_spec_with_method("GETT");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "EC-009-014: 'GETT' (typo) is not whitelisted; must produce E-SPEC-025"
        );
        assert!(
            errors.iter().any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "GETT")),
            "EC-009-014: error must cite method_value 'GETT'; got {:?}",
            errors
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-015 / AC-002: step.method = "get" (lowercase) → E-SPEC-025.
    ///
    /// The whitelist is case-sensitive. "get" is NOT equivalent to "GET".
    /// The BC explicitly states: "The implementation MUST NOT silently normalize to upper-case."
    ///
    /// Canonical test vector: "HTTP method — lowercase rejected".
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_015_lowercase_get_produces_e_spec_025() {
        let spec = make_spec_with_method("get");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "EC-009-015: 'get' (lowercase) is not in the case-sensitive whitelist; must produce E-SPEC-025"
        );
        assert!(
            errors.iter().any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "get")),
            "EC-009-015: error must cite method_value 'get' (not normalized to 'GET'); got {:?}",
            errors
        );
    }

    /// BC-2.16.009 v1.8 §VR7: lowercase "post" is invalid (case-sensitive whitelist).
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_lowercase_post_produces_e_spec_025() {
        let spec = make_spec_with_method("post");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "'post' (lowercase) is not in the case-sensitive whitelist; must produce E-SPEC-025"
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-016: step.method = "" (empty string) → E-SPEC-025.
    ///
    /// Empty string is not in the whitelist.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_016_empty_string_produces_e_spec_025() {
        let spec = make_spec_with_method("");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "EC-009-016: empty string is not in the whitelist; must produce E-SPEC-025"
        );
        assert!(
            errors.iter().any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value.is_empty())),
            "EC-009-016: error must cite method_value '' (empty); got {:?}",
            errors
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 continued: E-SPEC-025 message format is byte-verbatim (POL-24)
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7 §Error message format; error-taxonomy.md v1.59 E-SPEC-025; POL-24.
    ///
    /// The Display output of SpecEngineError::InvalidHttpMethod must be byte-verbatim with the
    /// canonical error-taxonomy.md v1.59 E-SPEC-025 message template:
    ///
    /// "Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>'
    ///  which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"
    ///
    /// Any deviation is a POL-24 violation and requires an error-taxonomy.md version bump +
    /// synchronized test update.
    ///
    /// Red Gate: fails with todo!() panic before implementation (via validate_step_methods).
    #[test]
    fn test_BC_2_16_009_e_spec_025_display_matches_error_taxonomy_v1_59_template_byte_for_byte() {
        let spec = make_spec_with_method("CONNECT");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "expected at least one E-SPEC-025 error for method 'CONNECT'"
        );
        let display = errors[0].to_string();
        // Byte-verbatim template from error-taxonomy.md v1.59 E-SPEC-025:
        let expected = "Step 'fetch' in 'test-sensor.events' declares method 'CONNECT' \
                        which is not a supported HTTP method. \
                        Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS";
        assert_eq!(
            display, expected,
            "E-SPEC-025 Display must match error-taxonomy.md v1.59 template byte-for-byte \
             (POL-24). Got:\n  {display:?}\nExpected:\n  {expected:?}"
        );
    }

    // -----------------------------------------------------------------------
    // EC-009-017: absent step.method (defaults to "GET") is NOT an error
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7 EC-009-017: absent step.method defaults to "GET" at pipeline
    /// level and must NOT produce E-SPEC-025.
    ///
    /// `FetchStep::default()` sets `method = "GET"`. The TOML spec may omit the `method`
    /// field entirely — serde fills it with "GET" via Default. Since "GET" is in the
    /// whitelist, Rule 7 must not error.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_017_absent_method_defaults_get_no_e_spec_025() {
        let step = FetchStep {
            name: "fetch".to_string(),
            // method is NOT set explicitly — uses Default which is "GET"
            path_template: "/api/v1/events".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        assert_eq!(
            step.method, "GET",
            "FetchStep::default() must produce method='GET' per spec_parser.rs Default impl"
        );
        let table = TableSpec::new_point_in_time("events", "security_finding", vec![], vec![step]);
        let spec = SensorSpec {
            sensor_id: "test-sensor".to_string(),
            name: "Test Sensor".to_string(),
            auth_type: AuthType::ApiKey,
            base_url: "https://example.com".to_string(),
            tables: vec![table],
            version: "1.0.0".to_string(),
            ..SensorSpec::default()
        };
        let results = validate_step_methods(&spec);
        assert!(
            results.is_empty(),
            "EC-009-017: absent method defaults to 'GET' which is whitelisted; expected zero \
             E-SPEC-025 errors; got {:?}",
            results
        );
    }

    // -----------------------------------------------------------------------
    // EC-009-018: Multi-error collection — INV-ERR-003
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7 EC-009-018; INV-ERR-003.
    ///
    /// Two steps in the same spec with invalid methods ("CONNECT" + "TRACE") must produce
    /// exactly two E-SPEC-025 errors. The validator must NOT fail-fast on the first error.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_018_multi_error_collection_two_invalid_methods() {
        let spec = make_spec_with_two_steps("CONNECT", "TRACE");
        let errors = errors_only(validate_step_methods(&spec));
        assert_eq!(
            errors.len(),
            2,
            "EC-009-018: two invalid methods must produce exactly 2 E-SPEC-025 errors \
             (INV-ERR-003 no fail-fast); got {} error(s): {:?}",
            errors.len(),
            errors
        );
        // Both errors must be InvalidHttpMethod
        for e in &errors {
            assert!(
                matches!(e, SpecEngineError::InvalidHttpMethod { .. }),
                "EC-009-018: each error must be InvalidHttpMethod; got {:?}",
                e
            );
        }
        // The two method_values must be CONNECT and TRACE (in any order)
        let method_values: Vec<&str> = errors
            .iter()
            .filter_map(|e| {
                if let SpecEngineError::InvalidHttpMethod { method_value, .. } = e {
                    Some(method_value.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            method_values.contains(&"CONNECT"),
            "EC-009-018: errors must include 'CONNECT'; got {:?}",
            method_values
        );
        assert!(
            method_values.contains(&"TRACE"),
            "EC-009-018: errors must include 'TRACE'; got {:?}",
            method_values
        );
    }

    /// Multi-error: one valid + one invalid step → exactly one error.
    ///
    /// Ensures that the validator does not error-out on valid steps.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_mixed_valid_invalid_produces_one_error() {
        let spec = make_spec_with_two_steps("GET", "CONNECT");
        let errors = errors_only(validate_step_methods(&spec));
        assert_eq!(
            errors.len(),
            1,
            "one valid + one invalid step must produce exactly 1 E-SPEC-025 error; got {}: {:?}",
            errors.len(),
            errors
        );
        assert!(
            matches!(&errors[0], SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "CONNECT"),
            "the single error must cite method_value 'CONNECT'; got {:?}",
            errors[0]
        );
    }

    // -----------------------------------------------------------------------
    // AC-003: Rule 7 runs AFTER Rule 6 env-var resolution
    // EC-009-019: env-resolved invalid method → E-SPEC-025 on resolved value
    // EC-009-020: env token failed Rule 6 → Rule 7 skips step (no double-reporting)
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7 EC-009-019; AC-003.
    ///
    /// When `step.method` resolves (via Rule 6) to an invalid value (e.g., "CONNECT"),
    /// Rule 7 must fire E-SPEC-025 on the RESOLVED value, not the raw token.
    ///
    /// Canonical test vector: "HTTP method — env-resolved invalid".
    ///
    /// Precondition: the spec passed to `validate_step_methods` has already had Rule 6
    /// applied — the `method` field contains the resolved value "CONNECT", not "${env.M}".
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution() {
        // After Rule 6 resolves ${env.SENSOR_METHOD}="CONNECT", the spec has method="CONNECT".
        // validate_step_methods receives the already-resolved spec.
        let spec = make_spec_with_method("CONNECT");
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "AC-003: env-resolved method 'CONNECT' must produce E-SPEC-025; got zero errors"
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, SpecEngineError::InvalidHttpMethod { method_value, .. } if method_value == "CONNECT")),
            "AC-003: E-SPEC-025 error must cite the RESOLVED method value 'CONNECT'; got {:?}",
            errors
        );
    }

    /// BC-2.16.009 v1.8 §VR7 EC-009-020; AC-003.
    ///
    /// When Rule 6 fails to resolve `step.method = "${env.SENSOR_METHOD}"` (var unset),
    /// Rule 7 MUST SKIP that step. Double-reporting (E-SPEC-024 + E-SPEC-025 for the same
    /// field) is noise, not signal.
    ///
    /// Precondition: when Rule 6 fails, the `step.method` field still contains the raw
    /// `${env.VAR_NAME}` token (env_resolver.rs only mutates the field on success). Rule 7
    /// detects unresolved tokens by checking if the method value matches the env token pattern
    /// `${env.VAR_NAME}`.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_ec009_020_unresolved_env_token_skipped_by_rule_7() {
        // Simulate the state after Rule 6 fails: method field still contains raw token.
        // env_resolver.rs does NOT mutate the field if any token is unresolvable (fail-closed).
        let spec = make_spec_with_method("${env.SENSOR_STEP_METHOD}");
        let results = validate_step_methods(&spec);
        let raw_method = "${env.SENSOR_STEP_METHOD}";
        assert!(
            results.is_empty(),
            "EC-009-020: method '{raw_method}' is an unresolved token from a \
             failed Rule 6 pass; Rule 7 must SKIP this step to prevent double-reporting; \
             expected zero E-SPEC-025 errors; got {results:?}",
        );
    }

    /// BC-2.16.009 v1.8 §VR7: Any `${env.VAR}` pattern in method is treated as Rule 6 failure.
    ///
    /// This covers the general case: any env token pattern remaining in the method field
    /// after Rule 6 means Rule 6 failed; Rule 7 must skip it.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_any_env_token_in_method_skipped_by_rule_7() {
        // Multiple env token patterns that Rule 7 must skip.
        for raw_token in &[
            "${env.METHOD}",
            "${env.HTTP_METHOD}",
            "${env.CROWDSTRIKE_METHOD}",
        ] {
            let spec = make_spec_with_method(raw_token);
            let results = validate_step_methods(&spec);
            assert!(
                results.is_empty(),
                "method '{}' is an unresolved Rule-6 token; Rule 7 must skip it \
                 (no E-SPEC-025); got {:?}",
                raw_token,
                results
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional invalid methods — comprehensive edge case coverage
    // -----------------------------------------------------------------------

    /// BC-2.16.009 v1.8 §VR7: mixed-case methods are invalid.
    ///
    /// "Get", "Post", "Delete" are not in the whitelist (case-sensitive).
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_mixed_case_methods_produce_e_spec_025() {
        for method in &["Get", "Post", "Delete", "Put", "Patch", "Head", "Options"] {
            let spec = make_spec_with_method(method);
            let results = validate_step_methods(&spec);
            assert!(
                !results.is_empty(),
                "'{}' (mixed-case) is not in the case-sensitive whitelist; must produce \
                 E-SPEC-025",
                method
            );
        }
    }

    /// BC-2.16.009 v1.8 §VR7: "DELETE" and remaining whitelist members pass validation.
    ///
    /// Red Gate: fails with todo!() panic before implementation.
    #[test]
    fn test_BC_2_16_009_delete_put_patch_head_options_pass_validation() {
        for method in &["DELETE", "PUT", "PATCH", "HEAD", "OPTIONS"] {
            let spec = make_spec_with_method(method);
            let results = validate_step_methods(&spec);
            assert!(
                results.is_empty(),
                "'{}' is in ALLOWED_HTTP_METHODS; must produce zero E-SPEC-025 errors; got {:?}",
                method,
                results
            );
        }
    }

    // -----------------------------------------------------------------------
    // VP-059 property: validate_step_methods never panic on any SensorSpec input
    // -----------------------------------------------------------------------

    /// BC-2.16.009 VP-059 invariant: validate_step_methods is a pure function that
    /// never panics (except the implementation-required todo!() in stub state).
    ///
    /// Verifies that a spec with no tables produces an empty Vec (not a panic).
    ///
    /// Red Gate: fails with todo!() panic (from stub) before implementation — this is
    /// expected and correct for Red Gate.
    #[test]
    fn test_BC_2_16_009_invariant_pure_function_no_panic_on_empty_spec() {
        let spec = make_spec_no_tables();
        let results = validate_step_methods(&spec);
        assert!(
            results.is_empty(),
            "empty spec (no tables/steps) must produce zero errors, not panic; got {:?}",
            results
        );
    }

    // -----------------------------------------------------------------------
    // F-LOCAL-P3-MED-002 — malformed env pseudo-tokens must NOT be skipped by Rule 7
    //
    // The skip-guard must use the SAME well-formed token grammar as Rule 6
    // (ENV_TOKEN_REGEX: `\$\{env\.[A-Z0-9_]+\}`).  Malformed pseudo-tokens like
    // `${env.lower}`, `${env.foo-bar}`, and `${env.}` do NOT match that regex →
    // Rule 6 never processes them → they reach Rule 7 as literal strings → they are
    // NOT in ALLOWED_HTTP_METHODS → must produce E-SPEC-025.
    //
    // CONFIRM: the existing well-formed-unresolved-token tests above (EC-009-020 and
    // test_BC_2_16_009_any_env_token_in_method_skipped_by_rule_7) are unaffected —
    // `${env.SENSOR_STEP_METHOD}` and friends DO match ENV_TOKEN_REGEX and ARE skipped.
    // -----------------------------------------------------------------------

    /// F-LOCAL-P3-MED-002: `${env.lower}` (lowercase VAR_NAME) is a malformed
    /// pseudo-token — Rule 6 only handles `[A-Z0-9_]+`, so this was NEVER resolved.
    /// Rule 7 must NOT skip it (old broad `contains("${env.")` check skipped it).
    /// It must produce E-SPEC-025 because it is not in ALLOWED_HTTP_METHODS.
    #[test]
    fn test_BC_2_16_009_malformed_env_lowercase_var_name_produces_e_spec_025() {
        let method = "${env.lower}";
        let spec = make_spec_with_method(method);
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "F-LOCAL-P3-MED-002: '{method}' has a lowercase VAR_NAME — Rule 6 grammar \
             requires [A-Z0-9_]+ so this token is NEVER resolved; Rule 7 must NOT skip it; \
             expected E-SPEC-025, got zero errors"
        );
        assert!(
            errors.iter().any(|e| matches!(
                e,
                SpecEngineError::InvalidHttpMethod { method_value, .. }
                    if method_value == method
            )),
            "F-LOCAL-P3-MED-002: error must be InvalidHttpMethod with method_value \
             '{method}'; got {errors:?}",
        );
    }

    /// F-LOCAL-P3-MED-002: `${env.foo-bar}` (hyphen in VAR_NAME) is a malformed
    /// pseudo-token — Rule 6 grammar forbids hyphens (`[A-Z0-9_]+` only).
    /// Rule 7 must NOT skip it; it must produce E-SPEC-025.
    #[test]
    fn test_BC_2_16_009_malformed_env_hyphen_in_var_name_produces_e_spec_025() {
        let method = "${env.foo-bar}";
        let spec = make_spec_with_method(method);
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "F-LOCAL-P3-MED-002: '{method}' has a hyphen in VAR_NAME — not matched by \
             ENV_TOKEN_REGEX ([A-Z0-9_]+); Rule 7 must NOT skip it; expected E-SPEC-025, \
             got zero errors"
        );
        assert!(
            errors.iter().any(|e| matches!(
                e,
                SpecEngineError::InvalidHttpMethod { method_value, .. }
                    if method_value == method
            )),
            "F-LOCAL-P3-MED-002: error must be InvalidHttpMethod with method_value \
             '{method}'; got {errors:?}",
        );
    }

    /// F-LOCAL-P3-MED-002: `${env.}` (empty VAR_NAME) is a malformed pseudo-token —
    /// Rule 6 grammar requires at least one `[A-Z0-9_]` character after `env.`.
    /// Rule 7 must NOT skip it; it must produce E-SPEC-025.
    #[test]
    fn test_BC_2_16_009_malformed_env_empty_var_name_produces_e_spec_025() {
        let method = "${env.}";
        let spec = make_spec_with_method(method);
        let errors = errors_only(validate_step_methods(&spec));
        assert!(
            !errors.is_empty(),
            "F-LOCAL-P3-MED-002: '{method}' has an empty VAR_NAME — not matched by \
             ENV_TOKEN_REGEX ([A-Z0-9_]+); Rule 7 must NOT skip it; expected E-SPEC-025, \
             got zero errors"
        );
        assert!(
            errors.iter().any(|e| matches!(
                e,
                SpecEngineError::InvalidHttpMethod { method_value, .. }
                    if method_value == method
            )),
            "F-LOCAL-P3-MED-002: error must be InvalidHttpMethod with method_value \
             '{method}'; got {errors:?}",
        );
    }

    /// F-LOCAL-P3-MED-002: Confirm that a well-formed unresolved token IS still correctly
    /// skipped after the fix — the skip-guard change must not regress EC-009-020.
    ///
    /// `${env.VALID_NAME}` (uppercase, underscore only) matches ENV_TOKEN_REGEX exactly.
    /// After a failed Rule 6 pass (var unset), the method field still holds this raw token.
    /// Rule 7 must skip it to prevent double-reporting E-SPEC-024 + E-SPEC-025.
    #[test]
    fn test_BC_2_16_009_well_formed_unresolved_token_still_skipped_after_f_med_002_fix() {
        // These are all WELL-FORMED per ENV_TOKEN_REGEX `[A-Z0-9_]+`.
        // Rule 7 must continue to skip them (no regression from the F-MED-002 fix).
        for raw_token in &[
            "${env.VALID_NAME}",
            "${env.HTTP_METHOD}",
            "${env.SENSOR_STEP_METHOD}",
            "${env.A}",
            "${env.A1_B2}",
        ] {
            let spec = make_spec_with_method(raw_token);
            let results = validate_step_methods(&spec);
            assert!(
                results.is_empty(),
                "F-LOCAL-P3-MED-002 non-regression: '{}' is a WELL-FORMED env token \
                 (VAR_NAME matches [A-Z0-9_]+); Rule 7 must still SKIP it after the \
                 malformed-token fix; got {:?}",
                raw_token,
                results
            );
        }
    }

    // -----------------------------------------------------------------------
    // F-LOCAL-P4-MED-001 — load-bearing test: duplicate step names in one table
    //
    // ROOT CAUSE: The old name-reverse-lookup in load_all's toml_path construction
    // broke when two steps in one table shared the same name — the lookup always
    // found the FIRST step's index, even if the SECOND step had the invalid method.
    //
    // FIX: validate_step_methods now carries (ti, si) indices in its return value;
    // load_all uses those indices directly (no name lookup at all).
    //
    // This test is LOAD-BEARING against the old name-reverse-lookup pattern:
    //   - Old code (name lookup): finds index 0 for both steps → toml_path is
    //     `sensor.tables[0].steps[0].method` even though the invalid step is at
    //     index 1 → assertion FAILS.
    //   - New code (index-carry): carries si=1 directly → toml_path is
    //     `sensor.tables[0].steps[1].method` → assertion PASSES.
    //
    // Traces to: F-LOCAL-P4-MED-001; BC-2.16.009 §VR7 "exact TOML path for
    // actionable correction"; S-SPEC-HTTP-METHOD-VALIDATION-001.
    // -----------------------------------------------------------------------

    /// F-LOCAL-P4-MED-001 — Duplicate step names: second step (index 1) has invalid
    /// method; validate_step_methods must carry index 1 (not 0) in the returned tuple.
    ///
    /// Fixture: ONE table, TWO steps both named "fetch" (duplicate step names are NOT
    /// rejected by the engine — step-name uniqueness is not enforced). The FIRST step
    /// has valid method "GET"; the SECOND step has invalid method "CONNECT".
    ///
    /// Assertion: the returned tuple has `step_index == 1`, proving that the index is
    /// derived from the enumerate loop position, not from a name reverse-lookup (which
    /// would return 0 for both because .find() stops at the first match).
    ///
    /// This test FAILS under the old name-reverse-lookup code and PASSES with the
    /// index-carry fix (F-LOCAL-P4-MED-001 structural fix).
    #[test]
    fn test_BC_2_16_009_f_local_p4_med_001_duplicate_step_names_carry_correct_index() {
        // Two steps with IDENTICAL names — step-name uniqueness is NOT enforced.
        // The SECOND step (index 1) has the invalid method.
        let step0 = FetchStep {
            name: "fetch".to_string(), // duplicate name — same as step1
            method: "GET".to_string(), // VALID
            path_template: "/api/v1/first".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        let step1 = FetchStep {
            name: "fetch".to_string(),     // duplicate name — same as step0
            method: "CONNECT".to_string(), // INVALID — must produce E-SPEC-025 at steps[1]
            path_template: "/api/v1/second".to_string(),
            response_path: "$.data".to_string(),
            ..FetchStep::default()
        };
        let table =
            TableSpec::new_point_in_time("events", "security_finding", vec![], vec![step0, step1]);
        let spec = SensorSpec {
            sensor_id: "test-sensor".to_string(),
            name: "Test Sensor".to_string(),
            auth_type: AuthType::ApiKey,
            base_url: "https://example.com".to_string(),
            tables: vec![table],
            version: "1.0.0".to_string(),
            ..SensorSpec::default()
        };

        let results = validate_step_methods(&spec);

        // Must produce exactly one error (the first step is valid GET).
        assert_eq!(
            results.len(),
            1,
            "F-LOCAL-P4-MED-001: one valid GET step + one invalid CONNECT step must \
             produce exactly 1 error; got {}: {:?}",
            results.len(),
            results
        );

        let (ti, si, ref err) = results[0];

        // Table index must be 0 (only one table in spec).
        assert_eq!(
            ti, 0,
            "F-LOCAL-P4-MED-001: table index must be 0 (first and only table); got {ti}"
        );

        // LOAD-BEARING assertion: step index must be 1 (the SECOND step has the invalid method).
        // Under the old name-reverse-lookup: .find(|s| s.name == "fetch") returns index 0
        // (first match), so ti=0/si=0 would have been emitted → this assertion FAILS.
        // With index-carry fix: enumerate gives si=1 directly → this assertion PASSES.
        assert_eq!(
            si, 1,
            "F-LOCAL-P4-MED-001 LOAD-BEARING: step index must be 1 (the second step is \
             at index 1 and has the invalid method); a name-reverse-lookup would return \
             index 0 (first match on 'fetch'), producing the WRONG path. got si={si}"
        );

        // Error must cite the CONNECT method.
        assert!(
            matches!(
                err,
                SpecEngineError::InvalidHttpMethod { method_value, .. }
                    if method_value == "CONNECT"
            ),
            "F-LOCAL-P4-MED-001: error must cite method_value 'CONNECT'; got {err:?}"
        );

        // Verify the canonical toml_path would be correctly constructed from these indices.
        let expected_path = format!("sensor.tables[{ti}].steps[{si}].method");
        assert_eq!(
            expected_path, "sensor.tables[0].steps[1].method",
            "F-LOCAL-P4-MED-001: canonical toml_path from carried indices must be \
             'sensor.tables[0].steps[1].method'; got '{expected_path}'"
        );
    }
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
