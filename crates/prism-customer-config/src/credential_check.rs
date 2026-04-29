use crate::error::ConfigError;

/// Recognized credential field name suffixes (BC-3.3.002 R-CRED-001 through R-CRED-005).
const CREDENTIAL_SUFFIXES: &[&str] = &["_token", "_secret", "_key", "_password", "_pass"];

/// Exact field names that are always treated as credential fields (BC-3.3.002 R-CRED-006).
const CREDENTIAL_EXACT: &[&str] = &["password"];

/// Recognized scheme prefixes for opaque credential references (BC-3.3.002 invariant 4).
const ALLOWED_SCHEMES: &[&str] = &["vault://", "env://", "file://", "keyring://"];

/// Recursively walk a parsed TOML value tree and collect any fields whose names
/// match credential heuristics but whose values are not scheme-prefixed references.
///
/// Returns a vec of `ConfigError::SuspectedCredentialValue` entries.
/// The error message MUST NOT include the field value (BC-3.3.002 Invariant 3).
pub fn scan_for_credentials(file: &str, value: &toml::Value) -> Vec<ConfigError> {
    let mut errors = Vec::new();
    scan_value(file, value, &mut errors);
    errors
}

/// Recursive helper that walks a TOML value tree.
fn scan_value(file: &str, value: &toml::Value, errors: &mut Vec<ConfigError>) {
    match value {
        toml::Value::Table(table) => {
            for (key, val) in table {
                if is_credential_field(key) {
                    if let toml::Value::String(s) = val {
                        if !has_allowed_scheme(s) {
                            errors.push(ConfigError::SuspectedCredentialValue {
                                file: file.to_string(),
                                field_name: key.clone(),
                            });
                        }
                    }
                }
                // Recurse into nested values regardless of field name.
                scan_value(file, val, errors);
            }
        }
        toml::Value::Array(arr) => {
            for item in arr {
                scan_value(file, item, errors);
            }
        }
        // Scalars have no nested keys to examine.
        _ => {}
    }
}

/// Returns true if the field name looks like a credential field.
fn is_credential_field(name: &str) -> bool {
    // Exact match first.
    if CREDENTIAL_EXACT.contains(&name) {
        return true;
    }
    // Suffix match.
    CREDENTIAL_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

/// Returns true if the value string uses one of the allowed scheme prefixes.
fn has_allowed_scheme(value: &str) -> bool {
    ALLOWED_SCHEMES
        .iter()
        .any(|scheme| value.starts_with(scheme))
}
