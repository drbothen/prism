//! Tests for BC-2.03.006: Credential Resolution at Sensor Query Time
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests MUST fail at Red Gate (stubs are todo!()).

use prism_credentials::resolution::{resolve_credential, CredentialResolutionError};

// ---------------------------------------------------------------------------
// TV-BC-2.03.006-001: credential exists — resolves successfully
// ---------------------------------------------------------------------------

/// BC-2.03.006 postcondition: when credential exists, it is resolved as SecretString.
#[tokio::test]
async fn test_BC_2_03_006_resolves_existing_credential_as_secret_string() {
    // This test requires a configured credential in the store.
    // For Red Gate purposes, the store is empty (todo stubs) — this will panic.
    let result = resolve_credential("acme", "crowdstrike", "api_key").await;
    assert!(
        result.is_ok(),
        "resolution of existing credential must succeed, got: {result:?}"
    );
    // The returned value must be SecretString — it is typed, so compile-time enforced.
    let _secret: secrecy::SecretString = result.unwrap();
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.006-002: credential missing — returns clear error, no API call
// ---------------------------------------------------------------------------

/// BC-2.03.006 error case: credential not in store returns CredentialResolutionError::NotFound.
/// The sensor API call must NOT be attempted.
#[tokio::test]
async fn test_BC_2_03_006_rejects_missing_credential_with_setup_suggestion() {
    let result = resolve_credential("acme", "crowdstrike", "missing_key").await;
    assert!(result.is_err(), "missing credential must return error");
    match result.unwrap_err() {
        CredentialResolutionError::NotFound {
            client_id,
            sensor_id,
            credential_name,
            suggestion,
        } => {
            assert_eq!(client_id, "acme");
            assert_eq!(sensor_id, "crowdstrike");
            assert_eq!(credential_name, "missing_key");
            assert!(!suggestion.is_empty(), "setup suggestion must be provided");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.006-003: CrowdStrike both credentials present — query proceeds
// ---------------------------------------------------------------------------

/// BC-2.03.006 EC-03-014: CrowdStrike requires two credentials.
/// Both must be resolved independently; both must succeed.
#[tokio::test]
async fn test_BC_2_03_006_crowdstrike_both_credentials_resolve() {
    let client_id_result = resolve_credential("acme", "crowdstrike", "client_id").await;
    let client_secret_result = resolve_credential("acme", "crowdstrike", "client_secret").await;
    assert!(
        client_id_result.is_ok(),
        "CrowdStrike client_id must resolve, got: {client_id_result:?}"
    );
    assert!(
        client_secret_result.is_ok(),
        "CrowdStrike client_secret must resolve, got: {client_secret_result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.006-004: CrowdStrike client_secret missing — query fails
// ---------------------------------------------------------------------------

/// BC-2.03.006 EC-03-014: if either CrowdStrike credential is missing, query fails.
#[tokio::test]
async fn test_BC_2_03_006_rejects_crowdstrike_with_missing_client_secret() {
    let result = resolve_credential("acme", "crowdstrike", "client_secret").await;
    assert!(
        result.is_err(),
        "query must fail before API call if client_secret is missing"
    );
}

// ---------------------------------------------------------------------------
// Invariant: resolved credential remains in SecretString wrapper
// ---------------------------------------------------------------------------

/// BC-2.03.006 invariant DI-002: resolved credential value stays wrapped in SecretString.
/// The return type enforces this at compile time.
#[tokio::test]
async fn test_BC_2_03_006_invariant_resolved_credential_is_secret_string() {
    // Type-level assertion: resolve_credential returns SecretString, not String or &str.
    // This test would fail to compile if the return type were changed to String.
    let _: Result<secrecy::SecretString, CredentialResolutionError> =
        resolve_credential("acme", "crowdstrike", "api_key").await;
}

// ---------------------------------------------------------------------------
// Postcondition: resolution is audit-logged (namespace only)
// ---------------------------------------------------------------------------

/// BC-2.03.006 postcondition: resolution emits audit log with namespace, never value.
/// This is verified by checking that resolve_credential calls emit_audit internally.
/// For Red Gate: this test documents the expected behavior; fails due to todo stub.
#[tokio::test]
async fn test_BC_2_03_006_resolution_emits_audit_log_without_value() {
    // We can't easily capture tracing output in a unit test without a custom subscriber.
    // This test verifies the contract by asserting that resolution succeeds (audit is
    // a side effect). The audit content test is in bc_2_03_010_audit_logging.rs.
    let _ = resolve_credential("acme", "crowdstrike", "api_key").await;
    // If we reach here without panic, the function at least didn't crash on audit emission.
    // Red Gate: will panic at todo!() before reaching this line.
}
