// S-1.02 — CredentialName newtype with path-traversal validation (VP-011).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::PrismError;

/// Maximum allowed byte length for a `CredentialName`.
pub const CREDENTIAL_NAME_MAX_LEN: usize = 128;

/// Validated name for a stored credential.
///
/// Invariants enforced by `CredentialName::new`:
/// - Non-empty
/// - At most 128 characters
/// - Does not contain `/`, `\`, `..`, null bytes (`\0`), or whitespace
///
/// Enforcement: VP-011 Kani proof verifies that all path-traversal patterns
/// are rejected.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialName(Arc<str>);

impl CredentialName {
    /// Construct a validated `CredentialName`.
    ///
    /// Returns `Err(PrismError::InvalidCredentialName)` if:
    /// - `s` is empty
    /// - `s` exceeds 128 characters
    /// - `s` contains `/`, `\`, `..`, `\0`, or whitespace
    ///
    /// AC-4: `"../../passwd"` → `Err`
    /// AC-5: `"key\0value"` → `Err`
    pub fn new(s: &str) -> Result<Self, PrismError> {
        if s.is_empty() {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not be empty".to_owned(),
            });
        }
        if s.len() > CREDENTIAL_NAME_MAX_LEN {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: format!(
                    "credential name exceeds maximum length of {CREDENTIAL_NAME_MAX_LEN}: got {}",
                    s.len()
                ),
            });
        }
        if s.contains('/') {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not contain '/'".to_owned(),
            });
        }
        if s.contains('\\') {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not contain '\\'".to_owned(),
            });
        }
        if s.contains("..") {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not contain '..'".to_owned(),
            });
        }
        if s.contains('\0') {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not contain null bytes".to_owned(),
            });
        }
        if s.chars().any(|c| c.is_whitespace()) {
            return Err(PrismError::InvalidCredentialName {
                name: s.to_owned(),
                reason: "credential name must not contain whitespace".to_owned(),
            });
        }
        Ok(CredentialName(Arc::from(s)))
    }

    /// Bypass validation — for test fixtures in downstream crates only.
    ///
    /// Constructs a `CredentialName` without running path-traversal checks.
    /// Used by `prism-credentials` test helpers; kept for test-writer compatibility.
    ///
    /// MUST NOT be called from production code.
    pub fn new_unchecked(s: &str) -> Self {
        CredentialName(Arc::from(s))
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CredentialName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
