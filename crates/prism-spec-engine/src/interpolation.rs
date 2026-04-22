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
    #[error(
        "step '{step_name}' referenced in template but not defined. Available steps: {available:?}"
    )]
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

// ---------------------------------------------------------------------------
// Write-side interpolation (S-1.13)
// ---------------------------------------------------------------------------

impl Interpolator {
    /// Resolve `${record_ids}` to a JSON array (or percent-encoded list) of record id values.
    ///
    /// `record_ids` is the list of `record_id_field` values from matched query rows.
    /// Escaping obeys `context`:
    ///   - `JsonBody`: serialized as a compact JSON array literal, e.g. `["id1","id2"]`
    ///   - `UrlPath`: comma-separated percent-encoded values, e.g. `id1%2Cid2`
    pub fn interpolate_record_ids(
        template: &str,
        context: &InterpolationContext,
        record_ids: &[serde_json::Value],
    ) -> Result<String, InterpolationError> {
        let placeholder = "${record_ids}";

        // Serialize record_ids based on context.
        let replacement = match context {
            InterpolationContext::JsonBody => {
                // Produce a compact JSON array: ["val1","val2"]
                serde_json::to_string(record_ids).unwrap_or_else(|_| "[]".to_string())
            }
            InterpolationContext::UrlPath => {
                // Percent-encode each value, join with %2C (encoded comma).
                record_ids
                    .iter()
                    .map(|v| Self::percent_encode(&value_to_string(v)))
                    .collect::<Vec<_>>()
                    .join("%2C")
            }
        };

        Ok(template.replace(placeholder, &replacement))
    }

    /// Resolve `${params.KEY}` and `${params.KEY|default:VALUE}` references.
    ///
    /// `params` is the key=value map from the pipe stage's write_args.
    /// Missing keys without a default produce an `InterpolationError::FieldNotFound`.
    /// Escaping obeys `context`.
    pub fn interpolate_write_params(
        template: &str,
        context: &InterpolationContext,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String, InterpolationError> {
        // Regex: ${params.KEY} or ${params.KEY|default:VALUE}
        static PARAM_PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = PARAM_PATTERN.get_or_init(|| {
            regex::Regex::new(r"\$\{params\.([a-zA-Z0-9_]+)(?:\|default:([^}]*))?\}")
                .expect("write param regex is valid")
        });

        let mut result = String::with_capacity(template.len());
        let mut last_end = 0;

        for cap in re.captures_iter(template) {
            let full_match = cap.get(0).expect("full match");
            let key = cap.get(1).expect("key group").as_str();
            let default_val = cap.get(2).map(|m| m.as_str());

            result.push_str(&template[last_end..full_match.start()]);

            let raw_value = match params.get(key) {
                Some(v) => v.clone(),
                None => match default_val {
                    Some(d) => d.to_string(),
                    None => {
                        return Err(InterpolationError::FieldNotFound {
                            step_name: "params".to_string(),
                            field_path: key.to_string(),
                            hint: format!("param '{key}' not provided and no default specified"),
                        });
                    }
                },
            };

            let escaped = match context {
                InterpolationContext::UrlPath => Self::percent_encode(&raw_value),
                InterpolationContext::JsonBody => Self::json_escape(&raw_value),
            };

            result.push_str(&escaped);
            last_end = full_match.end();
        }

        result.push_str(&template[last_end..]);
        Ok(result)
    }
}
