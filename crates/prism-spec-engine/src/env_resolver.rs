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
//! BC-2.16.009 §Validation Rules 6; error-taxonomy.md v1.56 E-SPEC-024;
//! S-SPEC-ENV-VAR-001.

use crate::{error::SpecEngineError, spec_parser::SensorSpec};

/// Post-TOML-parse env var token resolver (BC-2.16.009 §Validation Rules 6 / AC-6).
///
/// Scans all `String` fields in `SensorSpec` for `${env.VAR_NAME}` tokens and resolves
/// them via `std::env::var`. Fields scanned include — at minimum — `base_url` on the
/// top-level `SensorSpec` and on any per-org overlay fields. Per the sibling-sweep note
/// in S-SPEC-ENV-VAR-001, the scanner MUST cover ALL `String` fields, not just `base_url`.
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
pub fn resolve_env_var_tokens(_spec: &mut SensorSpec, _file_path: &str) -> Vec<SpecEngineError> {
    todo!()
}
