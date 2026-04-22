//! Query-time credential resolution.
//!
//! # Contract: BC-2.03.006
//! At sensor query time, the credential for a (client_id, sensor_id, credential_name)
//! tuple is resolved from the active backend and returned as a `secrecy::SecretString`.
//! Resolution is audit-logged (namespace only, never the value).
//! If resolution fails, returns a clear error before any API call is attempted.

use secrecy::SecretString;
use thiserror::Error;

/// Error type specific to credential resolution (wraps PrismError with context).
#[derive(Debug, Error)]
pub enum CredentialResolutionError {
    #[error("Credential not found for {client_id}/{sensor_id}/{credential_name}: {suggestion}")]
    NotFound {
        client_id: String,
        sensor_id: String,
        credential_name: String,
        suggestion: String,
    },
    #[error("Backend unavailable for {client_id}/{sensor_id}/{credential_name}: {detail}")]
    BackendUnavailable {
        client_id: String,
        sensor_id: String,
        credential_name: String,
        detail: String,
    },
}

/// Resolve a credential at sensor query time.
///
/// # Contract: BC-2.03.006
/// - Resolves from the active backend using `(client_id, sensor_id, credential_name)`.
/// - Returns the credential as `SecretString` (never raw `String` or `&str`).
/// - Emits an audit log entry with namespace only (never the value).
/// - If resolution fails, returns `CredentialResolutionError` before any API call.
pub async fn resolve_credential(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<SecretString, CredentialResolutionError> {
    todo!("S-1.07: implement resolve_credential")
}
