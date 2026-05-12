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
        use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
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

// ---------------------------------------------------------------------------
// S-PLUGIN-PREREQ-C: AC-4 Red Gate — `$${...}` literal escape mechanism
//
// BC-2.16.002 postcondition: path_template and body_template are interpolated
// against variables; the escape mechanism is a grammar extension of the
// interpolation surface.
//
// AC-4 requires: `$${var}` → literal `${var}` (no variable lookup).
//                `${var}` → interpolated value of `var` (existing, unchanged).
//                `$$${var}` → literal `$` followed by interpolated `var`.
//
// RED GATE MECHANISM: The current `Interpolator::interpolate` uses a regex that
// matches `\$\{([a-zA-Z0-9_]+)\.([a-zA-Z0-9_.]+)\}`. A double-dollar `$$` before
// the opening brace is NOT recognized as an escape — the current regex will either
// match (treating `$$` as a literal `$` before the live reference) or not match
// (if the regex is anchored differently). These tests verify the EXPECTED semantics;
// they fail until the escape mechanism is implemented.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod escape_tests {
    use std::collections::HashMap;

    use super::{InterpolationContext, Interpolator};

    /// AC-4(a): double-dollar escape (`$$` before `{`) produces literal `${var}`.
    ///
    /// Template `"$${step1.var}"` with `step1.var = "hello"` in scope must produce
    /// the literal string `"${step1.var}"` — NOT the interpolated value `"hello"`.
    ///
    /// RED GATE: the current regex matches `${step1.var}` inside the template
    /// (treating the leading `$` as literal preceding text), producing `"$hello"`.
    /// After AC-4, `$$` before `{` suppresses interpolation and the output is literal.
    /// This test MUST FAIL until AC-4 is implemented.
    #[test]
    fn test_BC_2_16_002_interpolator_escape_double_dollar() {
        let mut vars = HashMap::new();
        vars.insert(
            "step1.var".to_string(),
            serde_json::Value::String("hello".to_string()),
        );

        // Template contains "$$" before "${step1.var}" — the AC-4 escape sequence.
        let template = "$${step1.var}";
        let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

        assert!(
            result.is_ok(),
            "AC-4 RED GATE: double-dollar escape must not return an error; got: {:?}",
            result.err()
        );
        let output = result.unwrap();
        let expected_literal = "${step1.var}";
        assert_eq!(
            output, expected_literal,
            "AC-4 RED GATE: double-dollar escape must produce literal '{expected_literal}', \
             not the interpolated value. CURRENT OUTPUT (before AC-4): '{output}'\n\
             IMPLEMENTATION NEEDED: detect $$ prefix before ${{...}} and suppress interpolation."
        );
    }

    /// AC-4(b): `${var}` with `var` in scope still produces the interpolated value.
    ///
    /// Backward compat: existing interpolation behavior is unchanged after AC-4.
    /// This test MUST PASS before AND after AC-4.
    #[test]
    fn test_BC_2_16_002_interpolator_live_reference_unaffected() {
        let mut vars = HashMap::new();
        vars.insert(
            "step1.var".to_string(),
            serde_json::Value::String("hello".to_string()),
        );

        let template = "${step1.var}";
        let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

        assert!(
            result.is_ok(),
            "backward compat: dollar-brace reference must succeed"
        );
        assert_eq!(
            result.unwrap(),
            "hello",
            "backward compat: dollar-brace reference must interpolate to 'hello'"
        );
    }

    /// AC-4(c): triple-dollar before brace — one literal `$` plus interpolated value.
    ///
    /// Template `"$$${step1.var}"` (three dollars):
    ///   AC-4 semantics: `$$` escapes to literal `$`; `${step1.var}` is a live reference.
    ///   Result: `"$"` + `"hello"` = `"$hello"`.
    ///
    /// RED GATE: the current regex processes `${step1.var}` inside the template
    /// but does not handle the leading `$$` escape prefix. Before AC-4:
    ///   - The regex matches `${step1.var}` (the tail), producing `$$hello`
    ///     (two literal `$` from unprocessed prefix, then interpolated `hello`).
    /// After AC-4: the `$$` is consumed as escape → `$`, and the remaining
    /// `${step1.var}` is interpolated → `hello`. Combined: `$hello`.
    ///
    /// This test MUST FAIL until AC-4 is implemented.
    #[test]
    fn test_BC_2_16_002_interpolator_triple_dollar_escape() {
        let mut vars = HashMap::new();
        vars.insert(
            "step1.var".to_string(),
            serde_json::Value::String("hello".to_string()),
        );

        // "$$${step1.var}" — two dollar signs then a live reference.
        // AC-4: `$$` → literal `$`; `${step1.var}` → `hello`; combined: `$hello`.
        let template = "$$${step1.var}";
        let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

        assert!(
            result.is_ok(),
            "AC-4(c): triple-dollar template must not return an error; got: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert_eq!(
            output, "$hello",
            "AC-4 RED GATE: triple-dollar template must produce '$hello' \
             (literal $ from double-dollar escape plus interpolated value 'hello'). \
             CURRENT OUTPUT (before AC-4 implementation): '{}'",
            output
        );
    }
}
