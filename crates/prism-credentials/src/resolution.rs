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
/// # Contract: BC-2.03.006 / BC-2.06.003 v1.4 (ADR-032, ADR-034)
///
/// Per-client four-tier resolution chain:
///   1. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` (file path; Tier 1 highest)
///      If set but file is missing/unreadable → hard error, no fallthrough.
///   2. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (direct env-var value; Tier 2)
///   3. OS keyring via `CredentialStoreOrgId::get_by_org` (Tier 3, ADR-034)
///      Active only when both `org_id` and `keyring` are `Some`.
///      Miss (Ok(None)/NoEntry) → fall through to Tier 4.
///      Backend error → hard `BackendUnavailable` (E-CRED-008, SOUL.md §4).
///   4. CRUD store `credential_status` → backend source lookup (Tier 4 lowest)
///
/// The global `{SENSOR}_{REF}` format is retired per ADR-032.
///
/// `client_id` is the org slug (e.g. `"demo-org-a"`, `"acme"`).
/// `org_id` is the pre-resolved `OrgId` for the org (callers in `prism-spec-engine` resolve
///   the slug → OrgId via `OrgRegistry`; `prism-credentials` MUST NOT import `OrgRegistry`
///   per the architecture compliance rule in `trait_.rs:84–85`).
/// `keyring` is the `CredentialStoreOrgId`-capable backend (injected by boot wiring).
/// Emits an audit log entry with namespace only (never the value).
///
/// # ADR-034 §D1 signature
pub async fn resolve_credential(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
    org_id: Option<&prism_core::OrgId>,
    keyring: Option<&std::sync::Arc<dyn crate::trait_::CredentialStoreOrgId>>,
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
            // arm returns "Internal error" with no detail field.
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

    // Tier 3: OS keyring via CredentialStoreOrgId::get_by_org (ADR-034 §D2).
    //
    // Active only when both `org_id` and `keyring` are `Some`. When either is `None`,
    // fall through silently to Tier 4 (BC-2.06.003 Tier-3 postcondition row 1).
    //
    // Error semantics (ADR-034 §D4 / BC-2.06.003 Tier-3 postcondition):
    //   - Ok(Some(secret)) → return Ok(secret) + audit "keyring"
    //   - Ok(None) / NoEntry → fall through to Tier 4 silently
    //   - Err(...) → hard BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: {reason}" }
    //     Rationale: a locked/unavailable keyring is an operator misconfiguration; silently falling
    //     through would hide the error (SOUL.md §4). AD-017: no credential value in the error.
    if let (Some(org_id), Some(keyring)) = (org_id, keyring) {
        // Build a CredentialName for the keyring lookup.
        let cred_name = match prism_core::CredentialName::new(credential_name) {
            Ok(n) => n,
            Err(e) => {
                // Invalid credential name — surface as hard error (DI-014).
                return Err(CredentialResolutionError::BackendUnavailable {
                    client_id: client_id.to_string(),
                    sensor_id: sensor_id.to_string(),
                    credential_name: credential_name.to_string(),
                    detail: format!("E-CRED-008: invalid credential name for Tier-3 lookup: {e}"),
                });
            }
        };

        // spawn_blocking is already encapsulated inside KeyringBackend::get_by_org
        // (ADR-034 §D2 implementation detail). No additional wrapping needed here.
        match keyring.get_by_org(org_id, sensor_id, &cred_name).await {
            Ok(Some(secret)) => {
                crate::audit::emit_audit(
                    crate::audit::AuditOperation::Get,
                    client_id,
                    sensor_id,
                    credential_name,
                    "keyring",
                    crate::audit::AuditOutcome::Success,
                );
                return Ok(secret);
            }
            Ok(None) => {
                // Tier-3 miss — fall through to Tier 4 silently.
                // BC-2.06.003 Tier-3 postcondition: Ok(None)/NoEntry → fall through.
            }
            Err(e) => {
                // Backend error (locked, NoStorageAccess, NoKeyringService, spawn panic).
                // Hard error — do NOT fall through to Tier 4 (ADR-034 §D4 / SOUL.md §4).
                // AD-017: the error detail from keyring-rs is a system error message
                // (e.g., "access denied"), NOT a credential value — safe to include.
                //
                // F-P6-OBS-003 fix: extract backend/reason without the inner E-CRED-004
                // code prefix so the operator-facing detail carries a SINGLE canonical code
                // (E-CRED-008). `PrismError::CredentialStoreError` Display includes
                // "E-CRED-004: credential store error (backend=...): {reason}" — using
                // `{e}` directly would produce double-prefix output.
                let inner_detail = match &e {
                    prism_core::PrismError::CredentialStoreError { backend, reason } => {
                        format!("backend={backend}: {reason}")
                    }
                    other => format!("{other}"),
                };
                crate::audit::emit_audit(
                    crate::audit::AuditOperation::Get,
                    client_id,
                    sensor_id,
                    credential_name,
                    "keyring",
                    crate::audit::AuditOutcome::Error,
                );
                return Err(CredentialResolutionError::BackendUnavailable {
                    client_id: client_id.to_string(),
                    sensor_id: sensor_id.to_string(),
                    credential_name: credential_name.to_string(),
                    detail: format!(
                        "E-CRED-008: OS keyring unavailable: {inner_detail}. \
                         Check keyring access (macOS Keychain / Linux libsecret). \
                         Use Tier 1/2 env vars as an alternative (BC-2.06.003)."
                    ),
                });
            }
        }
    }
    // Either org_id or keyring is None → Tier 3 skipped silently (Tier-3 disabled).

    // Tier 4: CRUD store lookup — check if the credential was configured
    // and then resolve through its source reference.
    let crud_result = crate::crud::credential_status(client_id, sensor_id, credential_name).await;

    dispatch_crud_tier4(
        crud_result,
        client_id,
        sensor_id,
        credential_name,
        &direct_env,
        &file_env,
    )
}

/// Dispatch the result of a Tier-4 CRUD store lookup.
///
/// Extracted as a pure(ish) function so unit tests can inject specific `crud_result` values —
/// including `Err(PrismError)` — without needing to mock the CRUD store itself.
///
/// # MED-003 (ADV-PR-P03-MED-003)
/// The prior `_ =>` arm in the inline match collapsed both `Ok(None)` (credential not
/// configured) and `Err(PrismError)` (CRUD store failure) into `NotFound`. This hid
/// genuine backend errors (lock poisoning, deserialization, etc.) behind a misleading
/// "not found / ensure env var is set" message — a SOUL.md #4 violation.
///
/// This function implements the correct split: `Ok(None)` → `NotFound` (genuinely not
/// configured), `Err(e)` → `BackendUnavailable` with the propagated error detail.
fn dispatch_crud_tier4(
    crud_result: Result<Option<crate::crud::CredentialMetadata>, prism_core::PrismError>,
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
    direct_env: &str,
    file_env: &str,
) -> Result<SecretString, CredentialResolutionError> {
    match crud_result {
        Ok(Some(meta)) => {
            // The credential has been configured. Try to resolve through its source.
            let backend_name = meta.backend_type.clone();
            // OBS-001 fix: pass the explicit source_reference so "env" backend reads the
            // NAMED env var stored at configure_credential_source time, not the per-client
            // derived name that Tier 2 already reads (which would make Tier 4 unreachable).
            let resolved = resolve_from_backend(
                &meta.backend_type,
                &meta.source_reference,
                client_id,
                sensor_id,
                credential_name,
            );

            match resolved {
                Ok(Some(secret)) => {
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
                Ok(None) => {
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
                Err(backend_err) => {
                    // Backend returned a hard error (e.g., file backend not implemented).
                    crate::audit::emit_audit(
                        crate::audit::AuditOperation::Get,
                        client_id,
                        sensor_id,
                        credential_name,
                        &backend_name,
                        crate::audit::AuditOutcome::Error,
                    );
                    Err(backend_err)
                }
            }
        }
        Ok(None) => {
            // Not in crud store and not in env — genuinely not configured.
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
        Err(crud_err) => {
            // CRUD store returned a hard error (lock poisoning, deserialization failure,
            // backend I/O error, etc.). This is NOT a "not found" condition — propagate
            // the underlying failure so the caller can distinguish a misconfigured backend
            // from a credential that was never configured. SOUL.md #4: do not swallow errors.
            //
            // MED-003 fix (ADV-PR-P03-MED-003): the prior `_ =>` arm collapsed both
            // Ok(None) and Err(PrismError) into NotFound, discarding genuine CRUD failures
            // behind a misleading "ensure the env var is set" message.
            crate::audit::emit_audit(
                crate::audit::AuditOperation::Get,
                client_id,
                sensor_id,
                credential_name,
                "crud_store",
                crate::audit::AuditOutcome::Error,
            );
            Err(CredentialResolutionError::BackendUnavailable {
                client_id: client_id.to_string(),
                sensor_id: sensor_id.to_string(),
                credential_name: credential_name.to_string(),
                detail: format!(
                    "CRUD store returned an error while looking up credential \
                     '{credential_name}' for org '{client_id}', sensor '{sensor_id}': {crud_err}. \
                     Check the credential store backend for lock contention or data corruption."
                ),
            })
        }
    }
}

/// Attempt to resolve a secret value from the backend type (Tier 4 CRUD store source).
///
/// OBS-001 fix: accepts `source_reference` — the EXPLICIT env var name (or file path, etc.)
/// that was stored in `CredentialMetadata.source_reference` at `configure_credential_source`
/// time. The "env" branch reads this explicitly named env var, making it semantically
/// distinct from Tier 2 (which reads the per-client-derived var via `per_client_env_var`).
///
/// This means Tier 4 "env" is now reachable: an operator who runs
/// `configure_credential_source(source = CredentialRef { kind: Env, reference: "MY_CUSTOM_VAR" })`
/// can have Tier 4 read `MY_CUSTOM_VAR` even when Tier 2's per-client derived var is unset.
///
/// SEC-001 (PR #171 review, CWE-668): the "env" branch reads the EXPLICITLY STORED reference
/// from the CRUD store, NOT the per-client derived name (ADR-032 Tier 2) and NOT any global
/// `{CREDENTIAL_NAME_UPPER}` format. This preserves isolation: the operator must explicitly
/// configure which env var to read — no implicit cross-tenant namespace access.
///
/// SEC-003 (PR #171 review): the "file" branch MUST return an explicit error rather
/// than silently returning `None` — silent `None` swallows the failure reason.
///
/// Returns:
/// - `Ok(Some(secret))` — credential resolved from backend
/// - `Ok(None)` — backend type unknown (no resolution attempted)
/// - `Err(BackendUnavailable)` — backend is known but unavailable (file backend, etc.)
fn resolve_from_backend(
    backend_type: &str,
    source_reference: &str,
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<Option<SecretString>, CredentialResolutionError> {
    match backend_type {
        "env" => {
            // OBS-001 fix: read the EXPLICITLY NAMED env var stored in source_reference.
            //
            // Semantics: the operator used `configure_credential_source` to register
            // a specific env var name (e.g. "MY_CUSTOM_ENV_VAR"). Tier 4 must read THAT var.
            //
            // This is distinct from Tier 2 (`per_client_env_var`), which reads the
            // derived `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format. If Tier 2's
            // derived var is set, Tier 2 succeeds and Tier 4 is never reached — so the
            // Tier 4 "env" path is only exercised when the operator configured a DIFFERENT
            // (explicitly named) env var via the CRUD store.
            //
            // SEC-001 maintained: source_reference comes from the CRUD store, which was
            // populated by the operator via configure_credential_source — no implicit global
            // namespace fallback, no cross-tenant leak risk.
            Ok(std::env::var(source_reference)
                .ok()
                .map(|v| SecretString::new(v.into())))
        }
        "file" => {
            // OBS-001 fix: now that source_reference carries the file path, we CAN attempt
            // to resolve. But the "file" backend requires reading from disk — this is an
            // I/O operation that the SEC-003 PR review identified as unimplemented.
            //
            // Production file-backend resolution uses the _FILE Tier 1 env-var path
            // (PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE) which is already handled
            // in the Tier 1 + Tier 2 `resolve_secret` call above. The CRUD "file" backend
            // in Tier 4 overlaps in semantics with Tier 1 and adds complexity without
            // additional security benefit in the current design.
            //
            // SEC-003 fix (unchanged): return an explicit error rather than silent None.
            // The "file" CRUD backend is deferred pending a production file-vault design
            // (no standalone story exists yet — this is the correct surface for the human
            // to decide the file-backend CRUD contract).
            Err(CredentialResolutionError::BackendUnavailable {
                client_id: client_id.to_string(),
                sensor_id: sensor_id.to_string(),
                credential_name: credential_name.to_string(),
                detail: format!(
                    "file backend in CRUD store is not yet implemented for runtime resolution; \
                     use ADR-032 per-client file refs \
                     (PRISM_CLIENTS_{{ID}}_SENSORS_{{SENSOR}}_{{REF}}_FILE) for file-based secrets instead. \
                     Configured source_reference: '{source_reference}'"
                ),
            })
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // OBS-001: Tier 4 "env" backend — positive success path
    // ---------------------------------------------------------------------------

    /// OBS-001: Tier 4 "env" backend resolve_from_backend reads the EXPLICITLY NAMED
    /// env var from source_reference, NOT the per-client derived name.
    ///
    /// This test verifies the Tier 4 path is live (not shadowed by Tier 2) by using:
    ///   - A source_reference that is a CUSTOM env var name (not the per-client format)
    ///   - Tier 1 file env var: NOT set
    ///   - Tier 2 per-client env var: NOT set
    ///   - Tier 4 "env" backend with a custom env var set to a known value
    ///
    /// The CUSTOM var is distinct from `per_client_env_var("obs001-test", "armis", "token")`,
    /// which would be `PRISM_CLIENTS_OBS001_TEST_SENSORS_ARMIS_TOKEN` — deliberately not set.
    /// This proves Tier 4 resolves a DIFFERENT var than Tier 2 would.
    #[test]
    fn test_obs001_tier4_env_backend_reads_explicitly_named_env_var() {
        // Use a unique env var name that won't collide with any per-client derived name.
        let custom_var = "PRISM_OBS001_TIER4_ENV_TEST_UNIQUE_VAR";
        let expected_secret = "obs001-tier4-resolved-value";

        // Ensure Tier 2 per-client derived var is NOT set (so Tier 2 would return None).
        let per_client_var = per_client_env_var("obs001-test", "armis", "token");
        assert_ne!(
            per_client_var, custom_var,
            "OBS-001 invariant: custom_var must differ from per_client_var to prove Tier 4 is distinct"
        );
        std::env::remove_var(&per_client_var);

        // Set the explicitly-named custom var (simulating what operator stored via CRUD).
        std::env::set_var(custom_var, expected_secret);

        // Exercise resolve_from_backend directly (Tier 4 "env" branch).
        let result = resolve_from_backend(
            "env",
            custom_var,    // source_reference = explicitly named var
            "obs001-test", // client_id
            "armis",       // sensor_id
            "token",       // credential_name
        );

        // Cleanup before assertions (avoid polluting other tests).
        std::env::remove_var(custom_var);

        let secret = result
            .expect("OBS-001: Tier 4 'env' backend must return Ok (env var is set)")
            .expect("OBS-001: Tier 4 'env' backend must return Some (env var value is non-empty)");

        use secrecy::ExposeSecret as _;
        assert_eq!(
            secret.expose_secret(),
            expected_secret,
            "OBS-001: Tier 4 'env' backend must return the value of the explicitly named env var"
        );
    }

    /// OBS-001 negative: when the explicitly named env var is NOT set, Tier 4 "env"
    /// returns Ok(None) — not an error, not a panic.
    #[test]
    fn test_obs001_tier4_env_backend_returns_none_when_var_absent() {
        let custom_var = "PRISM_OBS001_TIER4_ENV_TEST_ABSENT_VAR";
        std::env::remove_var(custom_var);

        let result = resolve_from_backend("env", custom_var, "obs001-test", "armis", "token");

        assert!(
            matches!(result, Ok(None)),
            "OBS-001: Tier 4 'env' backend must return Ok(None) when the explicit env var is absent"
        );
    }

    /// OBS-001 isolation: Tier 2 per-client var being set does NOT affect Tier 4 "env"
    /// reading a different, custom var.
    #[test]
    fn test_obs001_tier4_env_backend_is_distinct_from_tier2() {
        let per_client_var = per_client_env_var("acme", "armis", "bearer_token");
        let custom_var = "PRISM_OBS001_TIER4_CUSTOM_BEARER_TOKEN";

        // Tier 2 var is set to one value.
        std::env::set_var(&per_client_var, "tier2-value");
        // Custom var (Tier 4 source_reference) is set to a different value.
        std::env::set_var(custom_var, "tier4-value");

        // Tier 4 reads the custom var, NOT the per-client derived name.
        let result = resolve_from_backend("env", custom_var, "acme", "armis", "bearer_token");

        std::env::remove_var(&per_client_var);
        std::env::remove_var(custom_var);

        let secret = result
            .expect("resolve_from_backend must succeed")
            .expect("must return Some");

        use secrecy::ExposeSecret as _;
        assert_eq!(
            secret.expose_secret(),
            "tier4-value",
            "OBS-001 isolation: Tier 4 reads source_reference (custom var), not Tier 2 derived name"
        );
    }

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

    // ---------------------------------------------------------------------------
    // MED-003: Tier-4 CRUD store error must NOT be collapsed to NotFound
    // (ADV-PR-P03-MED-003 — SOUL.md #4 violation fix)
    // ---------------------------------------------------------------------------

    /// MED-003: When the CRUD store returns an `Err(PrismError)` (e.g., lock poisoning,
    /// deserialization failure, backend I/O error), `dispatch_crud_tier4` must return
    /// `CredentialResolutionError::BackendUnavailable` — NOT `NotFound`.
    ///
    /// The prior `_ =>` catch-all mapped BOTH `Ok(None)` and `Err(PrismError)` to `NotFound`,
    /// hiding genuine backend failures behind a misleading "ensure env var is set" message.
    ///
    /// This test injects a `Err(PrismError::CredentialStoreError)` directly into
    /// `dispatch_crud_tier4` (the extracted helper) and asserts the error is surfaced
    /// as `BackendUnavailable`, not `NotFound`.
    #[test]
    fn test_med003_crud_store_error_is_not_reported_as_not_found() {
        // Construct a realistic CRUD store error (e.g., RocksDB lock poisoning).
        let crud_error = prism_core::PrismError::CredentialStoreError {
            backend: "in_memory".to_string(),
            reason: "borrow_mut: already borrowed — lock poisoning in RefCell".to_string(),
        };

        let result = dispatch_crud_tier4(
            Err(crud_error),
            "acme",                                               // client_id
            "armis",                                              // sensor_id
            "bearer_token",                                       // credential_name
            "PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN",      // direct_env
            "PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN_FILE", // file_env
        );

        // Must be an Err — CRUD store errors are not recoverable at this tier.
        let err = result.expect_err(
            "MED-003: CRUD store Err must produce CredentialResolutionError, not Ok(secret)",
        );

        // Must be BackendUnavailable — NOT NotFound.
        // A genuine CRUD error is NOT the same as "credential not configured."
        assert!(
            matches!(err, CredentialResolutionError::BackendUnavailable { .. }),
            "MED-003: Err(PrismError) from CRUD store must map to BackendUnavailable, not NotFound. \
             Got: {err:?}"
        );

        // The error detail must contain the original PrismError message (not silently dropped).
        let CredentialResolutionError::BackendUnavailable { detail, .. } = err else {
            unreachable!("pattern already matched above")
        };
        assert!(
            detail.contains("borrow_mut") || detail.contains("lock poisoning"),
            "MED-003: BackendUnavailable detail must propagate the original CRUD error reason. \
             Got: {detail:?}"
        );
    }

    /// MED-003 complementary: `Ok(None)` from the CRUD store (credential genuinely not
    /// configured) still maps to `NotFound` — the fix must not change the happy-absent path.
    #[test]
    fn test_med003_crud_store_ok_none_is_still_not_found() {
        let direct_env = "PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN";
        let file_env = "PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN_FILE";

        let result = dispatch_crud_tier4(
            Ok(None), // genuinely not in CRUD store
            "acme",
            "armis",
            "bearer_token",
            direct_env,
            file_env,
        );

        let err = result.expect_err(
            "MED-003: Ok(None) from CRUD store must produce CredentialResolutionError::NotFound",
        );

        assert!(
            matches!(err, CredentialResolutionError::NotFound { .. }),
            "MED-003: Ok(None) from CRUD store must map to NotFound (credential genuinely absent). \
             Got: {err:?}"
        );

        // The suggestion must guide the operator — not claim a backend error.
        let CredentialResolutionError::NotFound { suggestion, .. } = err else {
            unreachable!("pattern already matched above")
        };
        assert!(
            suggestion.contains(direct_env),
            "MED-003: NotFound suggestion must cite the direct env var name for operator guidance. \
             Got: {suggestion:?}"
        );
    }
}
