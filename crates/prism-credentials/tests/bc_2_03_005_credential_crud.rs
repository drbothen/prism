//! Tests for BC-2.03.005: Credential CRUD Operations via MCP Tools
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests MUST fail at Red Gate (all stubs are todo!()).

use prism_credentials::crud::{
    configure_credential_source, credential_status, delete_credential, list_credentials,
    ConfigureCredentialRequest, ConfigureCredentialResponse, CredentialRef, CredentialRefKind,
};

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-001: configure_credential_source (create) — no existing credential
// ---------------------------------------------------------------------------

/// BC-2.03.005 postcondition: initial creation returns `status: "created"` immediately.
/// No confirmation token required for the first write (non-destructive).
#[tokio::test]
async fn test_BC_2_03_005_create_returns_created_status() {
    let request = ConfigureCredentialRequest {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "api_key".to_string(),
        source: CredentialRef {
            kind: CredentialRefKind::Env,
            reference: "CROWDSTRIKE_API_KEY".to_string(),
        },
    };

    let result = configure_credential_source(request).await;
    assert!(
        result.is_ok(),
        "initial create should succeed, got: {result:?}"
    );
    match result.unwrap() {
        ConfigureCredentialResponse::Created { credential_name } => {
            assert_eq!(credential_name, "api_key");
        }
        other => panic!("expected Created, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-002: configure_credential_source (update) — existing credential
// ---------------------------------------------------------------------------

/// BC-2.03.005 postcondition: updating an existing credential returns ConfirmationRequired.
#[tokio::test]
async fn test_BC_2_03_005_update_existing_returns_confirmation_required() {
    // First, create the credential.
    let request = ConfigureCredentialRequest {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "api_key".to_string(),
        source: CredentialRef {
            kind: CredentialRefKind::Env,
            reference: "CROWDSTRIKE_API_KEY".to_string(),
        },
    };
    configure_credential_source(request.clone()).await.unwrap();

    // Second call (update) must return ConfirmationRequired.
    let update_request = ConfigureCredentialRequest {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "api_key".to_string(),
        source: CredentialRef {
            kind: CredentialRefKind::Env,
            reference: "CROWDSTRIKE_API_KEY_NEW".to_string(),
        },
    };
    let result = configure_credential_source(update_request).await;
    assert!(
        result.is_ok(),
        "update call should return Ok, got: {result:?}"
    );
    match result.unwrap() {
        ConfigureCredentialResponse::ConfirmationRequired(conf) => {
            assert_eq!(conf.status, "confirmation_required");
            assert!(
                !conf.confirmation_token.is_empty(),
                "confirmation_token must be non-empty"
            );
        }
        other => panic!("expected ConfirmationRequired, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-003: delete_credential — returns ConfirmationRequired
// ---------------------------------------------------------------------------

/// BC-2.03.005 postcondition: delete always returns ConfirmationRequired.
/// Deletion executes only after confirm_action.
#[tokio::test]
async fn test_BC_2_03_005_delete_returns_confirmation_required() {
    let result = delete_credential("acme", "crowdstrike", "api_key").await;
    assert!(
        result.is_ok(),
        "delete_credential should return Ok, got: {result:?}"
    );
    let conf = result.unwrap();
    assert_eq!(conf.status, "confirmation_required");
    assert!(!conf.confirmation_token.is_empty());
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-004: list_credentials(client_id: null) — returns E-FLAG-006
// ---------------------------------------------------------------------------

/// BC-2.03.005 error case: list_credentials with null client_id returns error.
/// Cross-client listing is prohibited to prevent MSSP portfolio disclosure.
#[tokio::test]
async fn test_BC_2_03_005_list_rejects_null_client_id() {
    let result = list_credentials(None, None).await;
    assert!(
        result.is_err(),
        "list with null client_id must fail with E-FLAG-006, got Ok"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    // Must reference E-FLAG-006 or "client_id" in the error message
    assert!(
        msg.contains("E-FLAG") || msg.contains("client_id") || msg.contains("null"),
        "error must reference E-FLAG-006 or client_id restriction, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-005: credential_status — returns metadata only, never value
// ---------------------------------------------------------------------------

/// BC-2.03.005 postcondition: credential_status returns metadata (backend type, last_modified)
/// but NEVER the raw credential value.
#[tokio::test]
async fn test_BC_2_03_005_credential_status_returns_metadata_not_value() {
    // First create a credential.
    let request = ConfigureCredentialRequest {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "api_key".to_string(),
        source: CredentialRef {
            kind: CredentialRefKind::Env,
            reference: "CROWDSTRIKE_API_KEY".to_string(),
        },
    };
    configure_credential_source(request).await.unwrap();

    let result = credential_status("acme", "crowdstrike", "api_key").await;
    assert!(
        result.is_ok(),
        "credential_status should succeed, got: {result:?}"
    );
    let metadata = result.unwrap();
    assert!(metadata.is_some(), "credential should exist");
    let meta = metadata.unwrap();
    assert_eq!(meta.credential_name, "api_key");
    assert_eq!(meta.client_id, "acme");
    assert_eq!(meta.sensor_id, "crowdstrike");
    assert!(
        !meta.backend_type.is_empty(),
        "backend_type must be non-empty"
    );
    // Critical: the metadata struct has no raw-value field — enforced by type
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.005-006: credential_status for non-existent credential — returns None
// ---------------------------------------------------------------------------

/// BC-2.03.005 edge case EC-005: non-existent credential returns None, not an error.
#[tokio::test]
async fn test_BC_2_03_005_credential_status_nonexistent_returns_none() {
    let result = credential_status("acme", "claroty", "does_not_exist").await;
    assert!(
        result.is_ok(),
        "status for missing credential should be Ok(None), got: {result:?}"
    );
    assert!(
        result.unwrap().is_none(),
        "missing credential should return None"
    );
}

// ---------------------------------------------------------------------------
// Precondition: invalid credential_name is rejected
// ---------------------------------------------------------------------------

/// BC-2.03.005 precondition: credential_name with path traversal characters is rejected.
/// Invariant VP-011: credential name sanitization.
#[tokio::test]
async fn test_BC_2_03_005_rejects_invalid_credential_name_path_traversal() {
    let request = ConfigureCredentialRequest {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "../etc/passwd".to_string(),
        source: CredentialRef {
            kind: CredentialRefKind::Env,
            reference: "SOME_ENV".to_string(),
        },
    };
    let result = configure_credential_source(request).await;
    assert!(
        result.is_err(),
        "path traversal in credential_name must be rejected"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("E-CRED-001") || msg.contains("invalid credential name"),
        "error must reference E-CRED-001, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Invariant: credential values never appear in list output
// ---------------------------------------------------------------------------

/// BC-2.03.005 invariant DI-002: list output is metadata only.
/// Verified structurally — CredentialMetadata has no value field.
#[tokio::test]
async fn test_BC_2_03_005_invariant_list_output_is_metadata_only() {
    let result = list_credentials(Some("acme"), Some("crowdstrike")).await;
    assert!(result.is_ok(), "list should succeed, got: {result:?}");
    let entries = result.unwrap();
    // Each entry is a CredentialMetadata — the type system ensures no value field.
    // Assert that we can iterate without encountering any value field (compile-time guarantee).
    for entry in &entries {
        assert!(!entry.credential_name.is_empty());
        // Structural check: backend_type is present but there is no `value` field on the type.
    }
}

// ---------------------------------------------------------------------------
// Edge case EC-03-012: no MCP tool exposes credential values
// ---------------------------------------------------------------------------

/// BC-2.03.005 EC-03-012: credential_status never returns a raw value field.
/// This is enforced by the CredentialMetadata type (no `value` field).
#[test]
fn test_BC_2_03_005_invariant_credential_metadata_has_no_value_field() {
    // Compile-time test: construct CredentialMetadata and confirm there is no `value` field.
    // If a `value` field were added to CredentialMetadata, the field access below would fail.
    let meta = prism_credentials::crud::CredentialMetadata {
        credential_name: "api_key".to_string(),
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        backend_type: "env".to_string(),
        last_modified: None,
    };
    // Only metadata fields exist — type has no `value: String` field.
    let _ = meta;
}
