//! Query-time credential resolution.
//!
//! # Contract: BC-2.03.006 / BC-2.06.003
//! At sensor query time, the credential for a (client_id, sensor_id, credential_name)
//! tuple is resolved from the per-client priority chain and returned as a
//! `secrecy::SecretString`. Resolution is audit-logged (namespace only, never the value).
//! If resolution fails, returns a clear error before any API call is attempted.
//!
//! # Per-client env-var convention (ADR-032 / BC-2.06.003 v1.3)
//!
//! `{ID}` = org slug uppercased with hyphens → underscores
//! (e.g. `demo-org-a` → `DEMO_ORG_A`, `acme-corp` → `ACME_CORP`).
//!
//! Resolution chain (four-tier, per-client):
//!   1. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` — file path; if set but file
//!      missing → hard error, no fallthrough (Tier 1 highest priority)
//!   2. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` — direct env-var value (Tier 2)
//!   3. OS keyring via `CredentialStoreOrgId::get_by_org` (Tier 3)
//!   4. CRUD store `credential_status` → backend source lookup (Tier 4 lowest)
//!
//! The retired global `{SENSOR}_{REF}` format (e.g. `ARMIS_BEARER_TOKEN`) is NOT
//! used. DI-002 (credential isolation per client) requires per-client namespacing.

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

/// Derive the SCREAMING_SNAKE_CASE `{ID}` component from an org slug.
///
/// BC-2.06.003 §Slug-to-SCREAMING-SNAKE Transform:
/// 1. Take the org slug exactly as declared in `prism.toml` `[[orgs]]` `org_slug`.
/// 2. Convert to UPPERCASE.
/// 3. Replace every hyphen (`-`) with an underscore (`_`).
/// 4. No other substitution: alphanumerics and underscores pass through unchanged.
///
/// Examples:
/// - `demo-org-a` → `DEMO_ORG_A`
/// - `acme` → `ACME`
/// - `acme-corp` → `ACME_CORP`
pub fn slug_to_screaming_snake(slug: &str) -> String {
    slug.to_uppercase().replace('-', "_")
}

/// Build the per-client Tier 2 env-var name for a credential.
///
/// Format: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}`
///
/// where:
/// - `{ID}` = `slug_to_screaming_snake(org_slug)`
/// - `{SENSOR}` = sensor_id uppercased with hyphens → underscores
/// - `{REF}` = ref_name uppercased with hyphens → underscores
pub fn per_client_env_var(org_slug: &str, sensor_id: &str, ref_name: &str) -> String {
    let id = slug_to_screaming_snake(org_slug);
    let sensor_upper = sensor_id.to_uppercase().replace('-', "_");
    let ref_upper = ref_name.to_uppercase().replace('-', "_");
    format!("PRISM_CLIENTS_{id}_SENSORS_{sensor_upper}_{ref_upper}")
}

/// Build the per-client Tier 1 env-var name (file path variant).
///
/// Format: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE`
pub fn per_client_file_env_var(org_slug: &str, sensor_id: &str, ref_name: &str) -> String {
    format!("{}_FILE", per_client_env_var(org_slug, sensor_id, ref_name))
}

/// Resolve a credential at sensor query time.
///
/// # Contract: BC-2.03.006 / BC-2.06.003 v1.3 (ADR-032)
///
/// Per-client four-tier resolution chain:
///   1. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` (file path; Tier 1 highest)
///      If set but file is missing/unreadable → hard error, no fallthrough.
///   2. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (direct env-var value; Tier 2)
///   3. OS keyring (Tier 3) — not implemented here, delegated to CRUD store lookup
///   4. CRUD store `credential_status` → backend source lookup (Tier 4 lowest)
///
/// The global `{SENSOR}_{REF}` format is retired per ADR-032.
///
/// `client_id` is the org slug (e.g. `"demo-org-a"`, `"acme"`).
/// Emits an audit log entry with namespace only (never the value).
pub async fn resolve_credential(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<SecretString, CredentialResolutionError> {
    // Build per-client env-var names (ADR-032 / BC-2.06.003 v1.3).
    // Tier 1: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE
    // Tier 2: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}
    let direct_env = per_client_env_var(client_id, sensor_id, credential_name);
    let file_env = per_client_file_env_var(client_id, sensor_id, credential_name);

    // Attempt resolve_secret env var chain first (Tier 1 + Tier 2).
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
            // Env chain not set — fall through to crud store lookup (Tier 4).
        }
        Err(e) => {
            // Tier 1 `_FILE` env var was set but resolution failed (file not found,
            // permission error, etc.).
            //
            // BC-2.06.003 §Error Cases: "Tier 1 `_FILE` env var set but file
            // non-existent or unreadable → Error; do NOT fall through to Tier 2 —
            // the explicit `_FILE` reference is a misconfiguration."
            //
            // SEC-002 (S-DEMO-001 PR review, AD-017 / CWE-209 determination):
            // The `{e}` interpolated into `detail` below CAN include a filesystem error
            // message (e.g., "No such file or directory: /some/path"). That `detail`
            // propagates to CredentialResolutionError::BackendUnavailable →
            // SpecEngineError::AuthAcquisitionFailed → SensorError::Internal { detail }.
            //
            // DOES NOT REACH MCP OUTPUT: In prism-query/src/materialization.rs,
            // SensorError::Internal surfaces as `fan_err.error.error_code()` → "E-SENSOR-099"
            // ONLY. The `detail` is emitted to internal `tracing::warn!` (audit log) but
            // NEVER to the MCP error response. prism-mcp/src/error_mapping.rs E-SENSOR-*
            // arm returns "Internal error; see audit log" with no detail field.
            // Full path verified: resolution.rs → auth_provider.rs → spec_driven_adapter.rs
            // → materialization.rs → error_mapping.rs (E-SENSOR-099, no detail leakage).
            // No sanitization needed per this determination.
            crate::audit::emit_audit(
                crate::audit::AuditOperation::Get,
                client_id,
                sensor_id,
                credential_name,
                "env_file",
                crate::audit::AuditOutcome::Error,
            );
            return Err(CredentialResolutionError::BackendUnavailable {
                client_id: client_id.to_string(),
                sensor_id: sensor_id.to_string(),
                credential_name: credential_name.to_string(),
                detail: format!(
                    "FILE env var resolution failed for credential '{}': {}. \
                     Check that the file path in the _FILE env var exists and is readable.",
                    credential_name, e
                ),
            });
        }
    }

    // Tier 4: CRUD store lookup — check if the credential was configured
    // and then resolve through its source reference.
    let crud_result = crate::crud::credential_status(client_id, sensor_id, credential_name).await;

    match crud_result {
        Ok(Some(meta)) => {
            // The credential has been configured. Try to resolve through its source.
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
                    // Backend configured but value not accessible.
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
                            "Credential '{credential_name}' is configured (backend: {backend_name}) \
                             but the referenced source is not accessible. \
                             Ensure the env var or file is set in the execution environment."
                        ),
                    })
                }
            }
        }
        _ => {
            // Not in crud store and not in env — NotFound.
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
                    "Set env var '{direct_env}' for org '{client_id}', sensor '{sensor_id}', \
                     credential '{credential_name}' (BC-2.06.003 Tier 2). \
                     Or set '{file_env}' to a file path containing the secret (Tier 1). \
                     Or run `configure_credential_source` to register a source in the CRUD store (Tier 4).",
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
            // Try the direct env var matching the credential name.
            let name_upper = credential_name.to_uppercase().replace('-', "_");
            std::env::var(&name_upper)
                .ok()
                .map(|v| SecretString::new(v.into()))
        }
        "file" => {
            // File path resolution (try credential_name as path).
            None // In-memory store does not persist file paths in this implementation.
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-2.06.003 §Slug-to-SCREAMING-SNAKE Transform worked examples.
    #[test]
    fn test_slug_to_screaming_snake_worked_examples() {
        assert_eq!(slug_to_screaming_snake("demo-org-a"), "DEMO_ORG_A");
        assert_eq!(slug_to_screaming_snake("acme"), "ACME");
        assert_eq!(slug_to_screaming_snake("acme-corp"), "ACME_CORP");
        assert_eq!(slug_to_screaming_snake("contoso"), "CONTOSO");
    }

    /// BC-2.06.003 §Env-var formats: Tier 2 direct env var.
    #[test]
    fn test_per_client_env_var_worked_examples() {
        assert_eq!(
            per_client_env_var("demo-org-a", "armis", "bearer_token"),
            "PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN"
        );
        assert_eq!(
            per_client_env_var("acme", "claroty", "bearer_token"),
            "PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN"
        );
        assert_eq!(
            per_client_env_var("acme", "cyberint", "api_key"),
            "PRISM_CLIENTS_ACME_SENSORS_CYBERINT_API_KEY"
        );
        assert_eq!(
            per_client_env_var("acme", "crowdstrike", "client_id"),
            "PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID"
        );
        assert_eq!(
            per_client_env_var("acme", "crowdstrike", "client_secret"),
            "PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET"
        );
    }

    /// BC-2.06.003 §Env-var formats: Tier 1 file env var.
    #[test]
    fn test_per_client_file_env_var_worked_examples() {
        assert_eq!(
            per_client_file_env_var("acme", "armis", "bearer_token"),
            "PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN_FILE"
        );
    }
}
