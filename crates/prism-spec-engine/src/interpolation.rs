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

/// Variable interpolation engine (BC-2.16.002).
pub struct Interpolator;

/// Regex pattern for `${step_name.field_path}` variable references.
static VAR_PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

fn var_regex() -> &'static regex::Regex {
    VAR_PATTERN.get_or_init(|| {
        regex::Regex::new(r"\$\{([a-zA-Z0-9_]+)\.([a-zA-Z0-9_.]+)\}")
            .expect("var interpolation regex is valid")
    })
}

impl Interpolator {
    /// Interpolate all `${step_name.field}` references in `template`.
    ///
    /// - `context`: escaping mode (JsonBody or UrlPath)
    /// - `vars`: map of `"step_name.field"` → resolved JSON values
    ///
    /// Returns the substituted string or an `InterpolationError`.
    pub fn interpolate(
        template: &str,
        context: &InterpolationContext,
        vars: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<String, InterpolationError> {
        let re = var_regex();
        let mut result = String::with_capacity(template.len());
        let mut last_end = 0;

        for cap in re.captures_iter(template) {
            let full_match = cap.get(0).expect("full match always present");
            let step_name = cap.get(1).expect("group 1").as_str();
            let field_path = cap.get(2).expect("group 2").as_str();
            let key = format!("{step_name}.{field_path}");

            // Append the literal portion before this match
            result.push_str(&template[last_end..full_match.start()]);

            // Look up the variable value
            let value = match vars.get(&key) {
                Some(v) => v,
                None => {
                    // Check if the step exists at all (any key starting with step_name.)
                    let step_exists = vars.keys().any(|k| k.starts_with(&format!("{step_name}.")));
                    if step_exists {
                        return Err(InterpolationError::FieldNotFound {
                            step_name: step_name.to_string(),
                            field_path: field_path.to_string(),
                            hint: format!(
                                "field '{field_path}' not found in step '{step_name}' response"
                            ),
                        });
                    } else {
                        let available: Vec<String> = vars
                            .keys()
                            .filter_map(|k| k.split('.').next().map(|s| s.to_string()))
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter()
                            .collect();
                        return Err(InterpolationError::UnknownStep {
                            step_name: step_name.to_string(),
                            available,
                        });
                    }
                }
            };

            // Coerce to string and apply context-appropriate escaping
            let raw_str = value_to_string(value);
            let escaped = match context {
                InterpolationContext::UrlPath => Self::percent_encode(&raw_str),
                InterpolationContext::JsonBody => Self::json_escape(&raw_str),
            };

            result.push_str(&escaped);
            last_end = full_match.end();
        }

        // Append the remainder of the template after the last match
        result.push_str(&template[last_end..]);
        Ok(result)
    }

    /// Extract all variable references from a template string.
    ///
    /// Returns a list of `(step_name, field_path)` tuples in order of appearance.
    pub fn extract_references(template: &str) -> Vec<(String, String)> {
        let re = var_regex();
        re.captures_iter(template)
            .map(|cap| {
                let step = cap.get(1).expect("group 1").as_str().to_string();
                let field = cap.get(2).expect("group 2").as_str().to_string();
                (step, field)
            })
            .collect()
    }

    /// Apply JSON escaping to a value string (for JsonBody context).
    ///
    /// Escapes: `"` → `\"`, `\` → `\\`, control chars → `\uXXXX`.
    pub fn json_escape(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        for ch in value.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    out.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
    }

    /// Apply percent-encoding to a value string (for UrlPath context).
    ///
    /// Encodes all characters except unreserved per RFC 3986: `[A-Za-z0-9._~-]`.
    /// Spaces become `%20`; `&` becomes `%26`; `=` becomes `%3D`.
    pub fn percent_encode(value: &str) -> String {
        use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
        // RFC 3986 unreserved characters: ALPHA / DIGIT / "-" / "." / "_" / "~"
        // All other chars (including space, &, =, /, ?, #) are encoded.
        const UNRESERVED: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'!')
            .add(b'"')
            .add(b'#')
            .add(b'$')
            .add(b'%')
            .add(b'&')
            .add(b'\'')
            .add(b'(')
            .add(b')')
            .add(b'*')
            .add(b'+')
            .add(b',')
            .add(b'/')
            .add(b':')
            .add(b';')
            .add(b'<')
            .add(b'=')
            .add(b'>')
            .add(b'?')
            .add(b'@')
            .add(b'[')
            .add(b'\\')
            .add(b']')
            .add(b'^')
            .add(b'`')
            .add(b'{')
            .add(b'|')
            .add(b'}');
        utf8_percent_encode(value, UNRESERVED).to_string()
    }
}

/// Convert a `serde_json::Value` to its string representation for interpolation.
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}
