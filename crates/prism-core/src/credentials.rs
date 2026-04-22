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
/// - Does not contain `/`, `\`, `..`, or null bytes (`\0`)
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
    /// - `s` contains `/`, `\`, `..`, or `\0`
    ///
    /// AC-4: `"../../passwd"` → `Err`
    /// AC-5: `"key\0value"` → `Err`
    pub fn new(_s: &str) -> Result<Self, PrismError> {
        unimplemented!("implement in S-1.02 — stub for Red Gate")
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
