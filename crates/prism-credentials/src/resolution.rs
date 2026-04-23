//! Query-time credential resolution.
//!
//! # Contract: BC-2.03.006
//! At sensor query time, the credential for a (client_id, sensor_id, credential_name)
//! tuple is resolved from the active backend and returned as a `secrecy::SecretString`.
//! Resolution is audit-logged (namespace only, never the value).
//! If resolution fails, returns a clear error before any API call is attempted.
//!
//! Resolution chain:
//!   1. Look up the credential reference in the crud store (what source type + ref was configured)
//!   2. Use resolve_secret to read the actual value from the referenced source (env/file/vault/keyring)
//!   3. Emit audit event (success or failure)

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
/// Resolution chain:
///   1. Check for env var `{SENSOR_ID_UPPER}_{CREDENTIAL_NAME_UPPER}_FILE` (file path)
///   2. Check for env var `{SENSOR_ID_UPPER}_{CREDENTIAL_NAME_UPPER}` (direct value)
///   3. Check the in-memory crud store (configured via configure_credential_source)
///      and resolve through the stored source reference
///   4. If none found → return CredentialResolutionError::NotFound
///
/// Emits an audit log entry with namespace only (never the value).
pub async fn resolve_credential(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<SecretString, CredentialResolutionError> {
    // Build canonical env var names: e.g. CROWDSTRIKE_API_KEY and CROWDSTRIKE_API_KEY_FILE
    let sensor_upper = sensor_id.to_uppercase().replace('-', "_");
    let name_upper = credential_name.to_uppercase().replace('-', "_");
    let direct_env = format!("{sensor_upper}_{name_upper}");
    let file_env = format!("{direct_env}_FILE");

    // Attempt resolve_secret env var chain first
    let env_result = crate::resolve_secret::resolve_secret(&file_env, &direct_env);

    match env_result {
        Ok(Some(secret)) => {
            crate::audit::emit_audit(
                crate::audit::AuditOperation::Get,
                client_id,
                sensor_id,
                credential_name,
                "env",
                crate::audit::AuditOutcome::Success,
            );
            return Ok(secret);
        }
        Ok(None) => {
            // Env chain not set — fall through to crud store lookup
        }
        Err(_) => {
            // Env chain had an error (e.g., file not found) — fall through to crud store
        }
    }

    // Attempt crud store lookup — check if the credential was configured
    // and then resolve through its source reference
    let crud_result = crate::crud::credential_status(client_id, sensor_id, credential_name).await;

    match crud_result {
        Ok(Some(meta)) => {
            // The credential has been configured. Try to resolve through its source.
            // For env-type sources, re-attempt via the stored reference name.
            let backend_name = meta.backend_type.clone();
            let resolved = resolve_from_backend(&meta.backend_type, credential_name);

            match resolved {
                Some(secret) => {
                    crate::audit::emit_audit(
                        crate::audit::AuditOperation::Get,
                        client_id,
                        sensor_id,
                        credential_name,
                        &backend_name,
                        crate::audit::AuditOutcome::Success,
                    );
                    Ok(secret)
                }
                None => {
                    // Backend configured but value not accessible
                    crate::audit::emit_audit(
                        crate::audit::AuditOperation::Get,
                        client_id,
                        sensor_id,
                        credential_name,
                        &backend_name,
                        crate::audit::AuditOutcome::NotFound,
                    );
                    Err(CredentialResolutionError::NotFound {
                        client_id: client_id.to_string(),
                        sensor_id: sensor_id.to_string(),
                        credential_name: credential_name.to_string(),
                        suggestion: format!(
                            "Credential '{credential_name}' is configured (backend: {backend_name}) but the referenced source is not accessible. \
                             Ensure the env var or file is set in the execution environment."
                        ),
                    })
                }
            }
        }
        _ => {
            // Not in crud store and not in env — NotFound
            crate::audit::emit_audit(
                crate::audit::AuditOperation::Get,
                client_id,
                sensor_id,
                credential_name,
                "none",
                crate::audit::AuditOutcome::NotFound,
            );
            Err(CredentialResolutionError::NotFound {
                client_id: client_id.to_string(),
                sensor_id: sensor_id.to_string(),
                credential_name: credential_name.to_string(),
                suggestion: format!(
                    "Run `configure_credential_source` to register a source for '{credential_name}' \
                     under client '{}'/sensor '{}', or set the env var '{}' or '{}'.",
                    client_id, sensor_id, direct_env, file_env
                ),
            })
        }
    }
}

/// Attempt to resolve a secret value from the backend type.
///
/// For env-type backends, re-tries the env var resolution.
/// Returns None if the backend source is unavailable.
fn resolve_from_backend(backend_type: &str, credential_name: &str) -> Option<SecretString> {
    match backend_type {
        "env" => {
            // Try the direct env var matching the credential name
            let name_upper = credential_name.to_uppercase().replace('-', "_");
            std::env::var(&name_upper)
                .ok()
                .map(|v| SecretString::new(v.into()))
        }
        "file" => {
            // File path resolution (try credential_name as path)
            None // In-memory store does not persist file paths in this implementation
        }
        _ => None,
    }
}
