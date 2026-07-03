//! Post-TOML-parse env var token resolver (BC-2.16.009 §Validation Rules 6 / AC-6).
//!
//! Scans all `String` fields in `SensorSpec` for `${env.VAR_NAME}` tokens
//! (where `VAR_NAME` matches `[A-Z0-9_]+`) and resolves them against
//! `std::env::var` before any downstream validation runs.
//!
//! # Ordering invariant
//! This pass MUST run:
//!   - AFTER TOML deserialization (operates on a fully-populated `SensorSpec`)
//!   - BEFORE URL-format validation (the `starts_with("http://")` check in `validation.rs`
//!     must see the resolved URL, not the raw `${env.VAR}` token string)
//!
//! # Namespace boundary
//! Only `${env.VAR_NAME}` tokens are resolved here. Tokens with other namespaces
//! (`${step.field}`, `${query.*}`) belong to the runtime interpolation engine
//! (BC-2.16.002) and are NOT touched by this pass.
//!
//! # Fail-closed semantics
//! A spec with any unresolvable token (absent var or empty string value) is REJECTED
//! ENTIRELY. Multiple unresolvable tokens produce multiple `E-SPEC-024` errors in the
//! same multi-error pass (no fail-fast). The caller MUST reject the spec if the returned
//! vec is non-empty.
//!
//! # AD-017 no-value-leak
//! `SpecEngineError::EnvVarNotSet` carries only the var NAME and TOML path.
//! The resolved VALUE is NEVER included in any error or log emitted by this module.
//!
//! BC-2.16.009 §Validation Rules 6; error-taxonomy.md §E-SPEC-024;
//! S-SPEC-ENV-VAR-001.

use std::env::VarError;
use std::sync::LazyLock;

use regex::Regex;

use crate::{error::SpecEngineError, spec_parser::SensorSpec};

/// Compiled regex that matches `${env.VAR_NAME}` tokens where VAR_NAME is `[A-Z0-9_]+`.
///
/// Only the `${env.*}` namespace is matched — `${step.*}`, `${query.*}`, etc. are
/// NOT matched and are left untouched by this pass (BC-2.16.009 §Validation Rules 6
/// namespace boundary).
///
/// Exposed as `pub(crate)` so that `validation.rs` Rule 7 can share the SAME grammar
/// to test whether a method field still contains a well-formed env token (TD-VSDD-060
/// single source of truth — no duplicated pattern string). Rule 7 skips only when this
/// regex matches; malformed pseudo-tokens like `${env.lower}` or `${env.}` do NOT match
/// and are therefore NOT skipped (F-LOCAL-P3-MED-002).
pub(crate) static ENV_TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{env\.([A-Z0-9_]+)\}").expect("ENV_TOKEN_REGEX is a valid pattern")
});

/// Resolve all `${env.VAR_NAME}` tokens in a single string field in-place.
///
/// Returns one `EnvVarNotSet` error per unresolvable token (absent or empty).
/// On success (all tokens resolved), the `field_value` is mutated with resolved values.
///
/// ## AD-017 no-value-leak
/// The error is constructed with the var NAME only. The resolved value is NEVER stored
/// in the error variant — not even in the `file_path` or `toml_path` fields.
fn resolve_field(
    field_value: &mut String,
    toml_path: &str,
    file_path: &str,
) -> Vec<SpecEngineError> {
    let mut errors = Vec::new();

    // Collect all matches first so we can iterate without borrow conflicts.
    // Each entry is (full_token_string, var_name).
    //
    // Deduplicate by var_name: when the same ${env.VAR} token appears multiple times in a
    // field, String::replace already replaces ALL occurrences in one call, so processing the
    // same token twice would produce duplicate resolved_pairs / error entries without adding
    // any correctness benefit (PR#165 M-002). We preserve the first-seen ordering via a
    // HashSet sentinel rather than sorting, so multi-error output order is deterministic.
    let mut seen_var_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let matches: Vec<(String, String)> = ENV_TOKEN_REGEX
        .captures_iter(field_value)
        .filter_map(|cap| {
            let full_token = cap[0].to_string();
            let var_name = cap[1].to_string();
            if seen_var_names.insert(var_name.clone()) {
                Some((full_token, var_name))
            } else {
                None
            }
        })
        .collect();

    if matches.is_empty() {
        return errors;
    }

    // Process each token. We apply successful replacements in a single pass at the end
    // to avoid offset issues when replacing multiple tokens in the same string.
    // For error cases, we do NOT mutate the field (fail-closed: the caller rejects on errors).
    let mut resolved_pairs: Vec<(String, String)> = Vec::new();

    for (full_token, var_name) in matches {
        match std::env::var(&var_name) {
            Ok(value) if !value.is_empty() => {
                resolved_pairs.push((full_token, value));
            }
            Ok(_empty) => {
                // Empty string is treated as missing (BC-2.16.009 §VR6 EC-009-002).
                // AD-017: do NOT include the empty value in the error.
                errors.push(SpecEngineError::EnvVarNotSet {
                    var_name,
                    toml_path: toml_path.to_string(),
                    file_path: file_path.to_string(),
                });
            }
            Err(VarError::NotPresent) => {
                // AD-017: do NOT call std::env::var again to put the value in the error.
                errors.push(SpecEngineError::EnvVarNotSet {
                    var_name,
                    toml_path: toml_path.to_string(),
                    file_path: file_path.to_string(),
                });
            }
            Err(VarError::NotUnicode(_)) => {
                // Non-UTF-8 value — treat as missing (can't use it as a URL component).
                errors.push(SpecEngineError::EnvVarNotSet {
                    var_name,
                    toml_path: toml_path.to_string(),
                    file_path: file_path.to_string(),
                });
            }
        }
    }

    // Only apply replacements when there are no errors for this field.
    // If any token in this field is unresolvable, leave the field as-is (fail-closed).
    // The caller will reject the spec on non-empty errors.
    if errors.is_empty() {
        for (token, value) in resolved_pairs {
            // String::replace replaces ALL occurrences of `token` in the field.
            // Since each token is unique by VAR_NAME (e.g., ${env.HOST}), this is correct.
            *field_value = field_value.replace(&token, &value);
        }
    }

    errors
}

/// Resolve `${env.VAR_NAME}` tokens in a single `String` field in-place.
///
/// Thin `pub(crate)` wrapper around `resolve_field` for callers (e.g., `overlay.rs`)
/// that need to resolve env tokens in a standalone field rather than a full `SensorSpec`.
///
/// ## Ordering contract (EC-009-007)
/// Must be called AFTER TOML deserialization and BEFORE any URL-format / scheme check
/// on the field value. This matches the `resolve_env_var_tokens` ordering invariant for
/// the type-spec path and extends it to the overlay path.
///
/// ## AD-017 no-value-leak
/// Errors carry only the var NAME and `toml_path` — the resolved value is never stored.
///
/// BC-2.16.009 §VR6 EC-009-007; S-SPEC-ENV-VAR-001 F-LOCAL-P1-CRIT-001.
pub(crate) fn resolve_env_tokens_in_string_field(
    field_value: &mut String,
    toml_path: &str,
    file_path: &str,
) -> Vec<SpecEngineError> {
    resolve_field(field_value, toml_path, file_path)
}

/// Post-TOML-parse env var token resolver (BC-2.16.009 §Validation Rules 6 / AC-6).
///
/// Scans all user-facing template `String` fields in `SensorSpec` for `${env.VAR_NAME}`
/// tokens and resolves them via `std::env::var`.
///
/// ## Fields scanned
/// Top-level: `name`, `base_url`, `version`, `auth_plugin`, `source_path`.
/// Per-table: `table_name`, `ocsf_class`, column `name`/`ocsf_field`,
/// step `name`/`method`/`path_template`/`response_path`/`body_template`/`pagination_cursor_path`.
///
/// ## Fields excluded (with rationale — Canonical Principle Rule 6)
/// - `sensor_id`: format-constrained to `^[a-z][a-z0-9_-]*$` (lowercase ASCII + `0-9_-`).
///   The regex structurally excludes `${` (which contains an uppercase or bracket character
///   outside the allowed set), making env token injection impossible. Excluding it avoids
///   a pointless regex scan on a provably token-free field.
/// - `credential_refs[].name`: format-constrained to `[a-zA-Z0-9_-]+`. This character set
///   structurally excludes `$` and `{`, making `${env.VAR_NAME}` injection impossible by
///   construction (same rationale as `sensor_id`). Scanning it would be a no-op.
/// - `file_hash`: infrastructure-set SHA-256 hex string populated by the file-loading
///   caller AFTER parse. It is never user-provided TOML content and cannot contain
///   `${env.VAR}` tokens by construction.
///
/// ## Resolution rules
/// - `${env.VAR_NAME}` where `VAR_NAME` is set and non-empty → replaced with value.
/// - `${env.VAR_NAME}` where var is absent OR value is empty (`""`) →
///   `E-SPEC-024` pushed to error vec (empty string treated as missing).
/// - Non-`env` namespace tokens (`${step.*}`, `${query.*}`) → left untouched.
/// - Multiple tokens in the same field → all tokens scanned (no fail-fast).
/// - Multiple fields with unresolvable tokens → errors collected across all fields.
///
/// ## Return value
/// Returns a `Vec<SpecEngineError>` — one `EnvVarNotSet` per unresolvable token.
/// If the vec is empty, all tokens resolved successfully and `spec` has been mutated
/// in-place with resolved values. If non-empty, the caller MUST reject the spec.
///
/// ## Call-site ordering
/// Insert this call in `validate_sensor_spec` (or the spec-load path) immediately
/// after TOML deserialization and before the URL-format check:
///
/// ```rust,ignore
/// // (1) TOML deserialization → SensorSpec produced
/// // (2) resolve_env_var_tokens — BC-2.16.009 AC-6
/// let env_errors = resolve_env_var_tokens(&mut spec, &file_path);
/// if !env_errors.is_empty() {
///     // reject spec — collected alongside other validation errors
/// }
/// // (3) URL-format check (starts_with("http://") / starts_with("https://")) — sees resolved URL
/// ```
///
/// BC-2.16.009 §Validation Rules 6; S-SPEC-ENV-VAR-001; AD-017.
pub fn resolve_env_var_tokens(spec: &mut SensorSpec, file_path: &str) -> Vec<SpecEngineError> {
    let mut all_errors = Vec::new();

    // -------------------------------------------------------------------------
    // Scan user-facing template String fields of SensorSpec (TD-VSDD-060 sibling-sweep).
    //
    // Excluded fields (see pub-fn doc for rationale — Canonical Principle Rule 6):
    //   - sensor_id: format-constrained ^[a-z][a-z0-9_-]*$ → structurally excludes "${".
    //   - credential_refs[].name: format-constrained [a-zA-Z0-9_-]+ → excludes "$" and "{"
    //     making ${env.VAR} injection impossible by construction (same rationale as sensor_id).
    //   - file_hash: infrastructure-set SHA-256 hex by file-loading caller post-parse.
    //
    // source_path IS scanned: it is an infrastructure field but may contain arbitrary
    // paths from user-supplied --spec-dir flags; scanning is belt-and-suspenders.
    // -------------------------------------------------------------------------

    // sensor.name
    all_errors.extend(resolve_field(&mut spec.name, "sensor.name", file_path));

    // sensor.base_url (primary field — most likely to contain ${env.VAR} tokens)
    all_errors.extend(resolve_field(
        &mut spec.base_url,
        "sensor.base_url",
        file_path,
    ));

    // sensor.version
    all_errors.extend(resolve_field(
        &mut spec.version,
        "sensor.version",
        file_path,
    ));

    // sensor.auth_plugin (Option<String>)
    if let Some(auth_plugin) = spec.auth_plugin.as_mut() {
        all_errors.extend(resolve_field(auth_plugin, "sensor.auth_plugin", file_path));
    }

    // sensor.source_path (infrastructure metadata — scan for completeness)
    all_errors.extend(resolve_field(
        &mut spec.source_path,
        "sensor.source_path",
        file_path,
    ));

    // -------------------------------------------------------------------------
    // Scan per-table string fields.
    // TableSpec.table_name, TableSpec.ocsf_class; FetchStep.name, .method,
    // .path_template, .body_template, .response_path, .pagination_cursor_path.
    // ColumnSpec.name, .ocsf_field (Option<String>).
    // -------------------------------------------------------------------------
    for (table_idx, table) in spec.tables.iter_mut().enumerate() {
        let table_path_prefix = format!("tables[{table_idx}]");

        all_errors.extend(resolve_field(
            &mut table.table_name,
            &format!("{table_path_prefix}.table_name"),
            file_path,
        ));

        all_errors.extend(resolve_field(
            &mut table.ocsf_class,
            &format!("{table_path_prefix}.ocsf_class"),
            file_path,
        ));

        for (col_idx, col) in table.columns.iter_mut().enumerate() {
            let col_path = format!("{table_path_prefix}.columns[{col_idx}]");
            all_errors.extend(resolve_field(
                &mut col.name,
                &format!("{col_path}.name"),
                file_path,
            ));
            if let Some(ocsf_field) = col.ocsf_field.as_mut() {
                all_errors.extend(resolve_field(
                    ocsf_field,
                    &format!("{col_path}.ocsf_field"),
                    file_path,
                ));
            }
        }

        for (step_idx, step) in table.steps.iter_mut().enumerate() {
            let step_path = format!("{table_path_prefix}.steps[{step_idx}]");

            all_errors.extend(resolve_field(
                &mut step.name,
                &format!("{step_path}.name"),
                file_path,
            ));
            all_errors.extend(resolve_field(
                &mut step.method,
                &format!("{step_path}.method"),
                file_path,
            ));
            all_errors.extend(resolve_field(
                &mut step.path_template,
                &format!("{step_path}.path_template"),
                file_path,
            ));
            all_errors.extend(resolve_field(
                &mut step.response_path,
                &format!("{step_path}.response_path"),
                file_path,
            ));

            if let Some(body) = step.body_template.as_mut() {
                all_errors.extend(resolve_field(
                    body,
                    &format!("{step_path}.body_template"),
                    file_path,
                ));
            }
            if let Some(cursor_path) = step.pagination_cursor_path.as_mut() {
                all_errors.extend(resolve_field(
                    cursor_path,
                    &format!("{step_path}.pagination_cursor_path"),
                    file_path,
                ));
            }
        }
    }

    all_errors
}
