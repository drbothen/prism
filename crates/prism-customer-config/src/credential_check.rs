use crate::error::ConfigError;

/// Recognized credential field name suffixes (BC-3.3.002 R-CRED-001 through R-CRED-005).
#[allow(dead_code)]
const CREDENTIAL_SUFFIXES: &[&str] = &["_token", "_secret", "_key", "_password", "_pass"];

/// Exact field names that are always treated as credential fields (BC-3.3.002 R-CRED-006).
#[allow(dead_code)]
const CREDENTIAL_EXACT: &[&str] = &["password"];

/// Recognized scheme prefixes for opaque credential references (BC-3.3.002 invariant 4).
#[allow(dead_code)]
const ALLOWED_SCHEMES: &[&str] = &["vault://", "env://", "file://", "keyring://"];

/// Recursively walk a parsed TOML value tree and collect any fields whose names
/// match credential heuristics but whose values are not scheme-prefixed references.
///
/// Returns a vec of `ConfigError::SuspectedCredentialValue` entries.
/// The error message MUST NOT include the field value (BC-3.3.002 Invariant 3).
pub fn scan_for_credentials(_file: &str, _value: &toml::Value) -> Vec<ConfigError> {
    todo!("credential_check::scan_for_credentials — implemented in Red Gate phase")
}

/// Returns true if the field name looks like a credential field.
#[allow(dead_code)]
fn is_credential_field(_name: &str) -> bool {
    todo!("credential_check::is_credential_field — implemented in Red Gate phase")
}

/// Returns true if the value string uses one of the allowed scheme prefixes.
#[allow(dead_code)]
fn has_allowed_scheme(_value: &str) -> bool {
    todo!("credential_check::has_allowed_scheme — implemented in Red Gate phase")
}
