//! Credential redaction for audit entry parameters (BC-2.05.003).
//!
//! `redact()` recursively walks a `serde_json::Value`, replacing any string
//! value whose key matches a credential pattern with `"***REDACTED***"`.
//!
//! # Credential key patterns (case-insensitive, BC-2.05.003)
//!
//! Keys that end in `_key`, `_secret`, `_token`, `_password`, `_credential`,
//! and full-name matches for `password`, `secret`, `token`, `api_key`,
//! `credential`, `private_key`, `passphrase`, `bearer`.
//!
//! # Invariant
//!
//! Redaction MUST be called on `parameters` BEFORE constructing `AuditEntry`.
//! The `parameters` field in `AuditEntry` must never contain a raw credential
//! value, even transiently (Architecture Compliance Rule, S-2.04).

/// Sentinel value that replaces redacted credential values (BC-2.05.003).
pub const REDACTED_SENTINEL: &str = "***REDACTED***";

/// Returns `true` if `key` matches a known credential field pattern.
///
/// Matching is case-insensitive. Both full-word patterns and suffix patterns
/// are checked per BC-2.05.003 and the story's credential pattern list.
pub fn is_credential_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    // Full-name exact matches
    const EXACT: &[&str] = &[
        "password",
        "secret",
        "token",
        "api_key",
        "credential",
        "private_key",
        "passphrase",
        "bearer",
    ];
    // Suffix patterns (fields ending with these)
    const SUFFIXES: &[&str] = &[
        "_key",
        "_secret",
        "_token",
        "_password",
        "_credential",
        "_passphrase",
        "_bearer",
    ];
    if EXACT.iter().any(|&p| lower == p) {
        return true;
    }
    if SUFFIXES.iter().any(|&s| lower.ends_with(s)) {
        return true;
    }
    false
}

/// Recursively walk `value` and replace credential values with
/// [`REDACTED_SENTINEL`].
///
/// Rules (BC-2.05.003):
/// - Object keys matching credential patterns: replace the entire value
///   (regardless of type) with the string sentinel.
/// - Arrays: recurse into each element.
/// - Nested objects: recurse recursively at any depth.
/// - Non-object, non-array values at the top level: returned unchanged
///   (no key context available at this level).
///
/// Credential key names are **preserved** for traceability; only values are
/// replaced.
pub fn redact(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let redacted_map = map
                .into_iter()
                .map(|(k, v)| {
                    let new_v = if is_credential_key(&k) {
                        // Replace the entire value — any type — with the sentinel string.
                        serde_json::Value::String(REDACTED_SENTINEL.to_owned())
                    } else {
                        // Recurse into non-credential sub-values.
                        redact(v)
                    };
                    (k, new_v)
                })
                .collect();
            serde_json::Value::Object(redacted_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(redact).collect())
        }
        // Primitives with no key context: pass through unchanged.
        other => other,
    }
}

#[cfg(test)]
mod tests {}
