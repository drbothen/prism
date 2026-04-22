//! Variable interpolation for fetch pipeline templates (BC-2.16.002).
//!
//! Resolves `${step_name.field}` references against a context of prior step
//! outputs. Applies safety escaping before substitution:
//!   - JSON body context: JSON-escape the resolved value
//!   - URL path/query context: percent-encode the resolved value
//!
//! The spec author declares which context each variable is interpolated into.

use thiserror::Error;

/// The context in which a variable is interpolated.
///
/// Determines the escaping applied before substitution (BC-2.16.002 postcondition,
/// Architecture Compliance Rule in S-1.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationContext {
    /// Variable appears in a JSON body template; value is JSON-escaped.
    JsonBody,
    /// Variable appears in a URL path segment or query parameter; value is percent-encoded.
    UrlPath,
}

/// Error produced when variable interpolation fails at parse or runtime.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterpolationError {
    /// `${step_name.field}` — step_name does not exist in current context.
    #[error("step '{step_name}' referenced in template but not defined. Available steps: {available:?}")]
    UnknownStep {
        step_name: String,
        available: Vec<String>,
    },
    /// `${step_name.field}` — field path does not match actual response structure.
    #[error("variable '${{{step_name}.{field_path}}}' not found in response: {hint}")]
    FieldNotFound {
        step_name: String,
        field_path: String,
        hint: String,
    },
    /// Forward reference: referencing step appears after the referencing step in order.
    #[error("step '{referencing_step}' references '{referenced_step}' which has not executed yet")]
    ForwardReference {
        referencing_step: String,
        referenced_step: String,
    },
}

/// Stub interpolation engine.
///
/// All methods are `unimplemented!()` — implemented in S-1.11.
pub struct Interpolator;

impl Interpolator {
    /// Interpolate all `${step_name.field}` references in `template`.
    ///
    /// - `context`: escaping mode (JsonBody or UrlPath)
    /// - `vars`: map of `"step_name.field"` → resolved string values
    ///
    /// Returns the substituted string or an `InterpolationError`.
    pub fn interpolate(
        template: &str,
        context: &InterpolationContext,
        vars: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<String, InterpolationError> {
        unimplemented!("Interpolator::interpolate — implement in S-1.11 (BC-2.16.002)")
    }

    /// Extract all variable references from a template string.
    ///
    /// Returns a list of `(step_name, field_path)` tuples in order of appearance.
    pub fn extract_references(template: &str) -> Vec<(String, String)> {
        unimplemented!("Interpolator::extract_references — implement in S-1.11 (BC-2.16.002)")
    }

    /// Apply JSON escaping to a value string (for JsonBody context).
    pub fn json_escape(value: &str) -> String {
        unimplemented!("Interpolator::json_escape — implement in S-1.11 (BC-2.16.002)")
    }

    /// Apply percent-encoding to a value string (for UrlPath context).
    pub fn percent_encode(value: &str) -> String {
        unimplemented!("Interpolator::percent_encode — implement in S-1.11 (BC-2.16.002)")
    }
}
