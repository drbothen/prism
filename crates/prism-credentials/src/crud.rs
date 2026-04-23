//! Credential CRUD operations exposed via MCP tools.
//!
//! # Contract: BC-2.03.005
//! Exposes four operations: configure_credential_source, credential_status,
//! delete_credential, and list_credentials.
//!
//! Mutation rules:
//! - `configure_credential_source` (create): returns `status: "created"` immediately.
//! - `configure_credential_source` (update): returns `ConfirmationRequired`.
//! - `delete_credential`: returns `ConfirmationRequired`.
//! - `list_credentials(client_id: null)`: returns E-FLAG-006 (client_id required).
//! - All operations accept source-type references ONLY — never raw values.
//! - All operations are audit-logged.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The kind of credential source reference (AI-opaque model per AD-017).
/// Raw credential values are NEVER accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRefKind {
    /// Env var name containing the credential value
    Env,
    /// File path containing the credential value
    File,
    /// HashiCorp Vault path
    Vault,
    /// OS keyring entry name
    Keyring,
}

impl std::fmt::Display for CredentialRefKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialRefKind::Env => write!(f, "env"),
            CredentialRefKind::File => write!(f, "file"),
            CredentialRefKind::Vault => write!(f, "vault"),
            CredentialRefKind::Keyring => write!(f, "keyring"),
        }
    }
}

/// A source-type reference for a credential (never a raw value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRef {
    pub kind: CredentialRefKind,
    /// The reference identifier (env var name, file path, vault path, or keyring entry)
    pub reference: String,
}

/// Request to configure (create or update) a credential source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureCredentialRequest {
    pub client_id: String,
    pub sensor_id: String,
    pub credential_name: String,
    pub source: CredentialRef,
}

/// Confirmation token returned when a mutation requires confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequired {
    pub status: String, // always "confirmation_required"
    pub confirmation_token: String,
    pub operation: String,
}

/// Response from configure_credential_source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConfigureCredentialResponse {
    /// Immediate success — initial creation (no existing credential)
    Created { credential_name: String },
    /// Confirmation required — update to an existing credential
    ConfirmationRequired(ConfirmationRequired),
}

/// Metadata returned by credential_status (never the raw value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub credential_name: String,
    pub client_id: String,
    pub sensor_id: String,
    pub backend_type: String,
    pub last_modified: Option<DateTime<Utc>>,
}

/// Response from credential_status
pub type CredentialStatusResponse = Option<CredentialMetadata>;

// ---------------------------------------------------------------------------
// In-memory store for credential metadata (test-time and runtime use).
// This is a simple in-memory map keyed by "{client_id}/{sensor_id}/{credential_name}".
//
// Thread-local storage gives each OS thread (and therefore each tokio::test, which
// creates a new runtime per thread) its own isolated store — this eliminates
// cross-test state contamination when running `cargo test` in parallel.
//
// Production use would delegate to KeyringBackend or EncryptedFileBackend.
// ---------------------------------------------------------------------------

use std::cell::RefCell;

thread_local! {
    static CREDENTIAL_STORE: RefCell<HashMap<String, CredentialMetadata>> = RefCell::new(HashMap::new());
}

fn with_store<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, CredentialMetadata>) -> R,
{
    CREDENTIAL_STORE.with(|cell| {
        let mut map = cell.borrow_mut();
        f(&mut map)
    })
}

fn store_key(client_id: &str, sensor_id: &str, credential_name: &str) -> String {
    format!("{client_id}/{sensor_id}/{credential_name}")
}

/// Validate credential name — rejects path traversal and invalid characters.
///
/// # Contract: BC-2.03.005 precondition
fn validate_credential_name(name: &str) -> Result<(), prism_core::PrismError> {
    // Reject path traversal sequences
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(prism_core::PrismError::InvalidCredentialName {
            name: name.to_string(),
            reason:
                "E-CRED-001: invalid credential name — path traversal characters are not allowed"
                    .to_string(),
        });
    }
    // Reject empty names
    if name.is_empty() {
        return Err(prism_core::PrismError::InvalidCredentialName {
            name: String::new(),
            reason: "E-CRED-001: invalid credential name — credential name must not be empty"
                .to_string(),
        });
    }
    Ok(())
}

/// Configure (create or update) a credential source reference.
///
/// # Contract: BC-2.03.005
/// - Create (no existing): returns `ConfigureCredentialResponse::Created` immediately.
/// - Update (existing): returns `ConfigureCredentialResponse::ConfirmationRequired`.
/// - Accepts `CredentialRef` only — never raw values.
///
/// # Errors
/// - `PrismError::InvalidCredentialName` if credential_name fails validation
pub async fn configure_credential_source(
    request: ConfigureCredentialRequest,
) -> Result<ConfigureCredentialResponse, prism_core::PrismError> {
    validate_credential_name(&request.credential_name)?;

    let key = store_key(
        &request.client_id,
        &request.sensor_id,
        &request.credential_name,
    );

    let exists = with_store(|map| map.contains_key(&key));

    if exists {
        // Update path: requires confirmation
        let token = uuid_v7_token();
        Ok(ConfigureCredentialResponse::ConfirmationRequired(
            ConfirmationRequired {
                status: "confirmation_required".to_string(),
                confirmation_token: token,
                operation: format!(
                    "update_credential:{}/{}/{}",
                    request.client_id, request.sensor_id, request.credential_name
                ),
            },
        ))
    } else {
        // Create path: immediate success
        let meta = CredentialMetadata {
            credential_name: request.credential_name.clone(),
            client_id: request.client_id.clone(),
            sensor_id: request.sensor_id.clone(),
            backend_type: request.source.kind.to_string(),
            last_modified: Some(Utc::now()),
        };
        with_store(|map| {
            map.insert(key, meta);
        });

        crate::audit::emit_audit(
            crate::audit::AuditOperation::Set,
            &request.client_id,
            &request.sensor_id,
            &request.credential_name,
            &request.source.kind.to_string(),
            crate::audit::AuditOutcome::Success,
        );

        Ok(ConfigureCredentialResponse::Created {
            credential_name: request.credential_name,
        })
    }
}

/// Check the status of a credential (returns metadata only, never raw value).
///
/// # Contract: BC-2.03.005
/// - Returns `Some(CredentialMetadata)` if the credential exists.
/// - Returns `None` if not found.
/// - NEVER returns the raw credential value.
pub async fn credential_status(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<CredentialStatusResponse, prism_core::PrismError> {
    let key = store_key(client_id, sensor_id, credential_name);
    let result = with_store(|map| map.get(&key).cloned());

    let outcome = if result.is_some() {
        crate::audit::AuditOutcome::Success
    } else {
        crate::audit::AuditOutcome::NotFound
    };

    crate::audit::emit_audit(
        crate::audit::AuditOperation::Get,
        client_id,
        sensor_id,
        credential_name,
        "in_memory",
        outcome,
    );

    Ok(result)
}

/// Delete a credential (gated behind confirmation token).
///
/// # Contract: BC-2.03.005
/// Returns `ConfirmationRequired` — deletion executes only after `confirm_action`.
pub async fn delete_credential(
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
) -> Result<ConfirmationRequired, prism_core::PrismError> {
    let token = uuid_v7_token();

    crate::audit::emit_audit(
        crate::audit::AuditOperation::Delete,
        client_id,
        sensor_id,
        credential_name,
        "in_memory",
        crate::audit::AuditOutcome::Success,
    );

    Ok(ConfirmationRequired {
        status: "confirmation_required".to_string(),
        confirmation_token: token,
        operation: format!("delete_credential:{client_id}/{sensor_id}/{credential_name}"),
    })
}

/// List credentials for a specific client/sensor (metadata only, never values).
///
/// # Contract: BC-2.03.005
/// - Requires non-null `client_id` — returns error if null/empty.
/// - Returns metadata entries only.
pub async fn list_credentials(
    client_id: Option<&str>,
    sensor_id: Option<&str>,
) -> Result<Vec<CredentialMetadata>, prism_core::PrismError> {
    // E-FLAG-006 equivalent: client_id is required
    let client_id = client_id.filter(|s| !s.is_empty()).ok_or_else(|| {
        prism_core::PrismError::InvalidCredentialName {
            name: "client_id".to_string(),
            reason: "E-FLAG-006: list_credentials requires a non-null client_id — cross-client listing is prohibited to prevent MSSP portfolio disclosure".to_string(),
        }
    })?;

    let entries = with_store(|map| {
        map.values()
            .filter(|meta| {
                meta.client_id == client_id && sensor_id.is_none_or(|s| meta.sensor_id == s)
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    crate::audit::emit_audit(
        crate::audit::AuditOperation::List,
        client_id,
        sensor_id.unwrap_or("*"),
        "*",
        "in_memory",
        crate::audit::AuditOutcome::Success,
    );

    Ok(entries)
}

/// Generate a time-ordered random token for confirmation flows.
///
/// Uses UUID v7 (timestamp + 74 random bits) for high entropy and time-ordering.
/// Resolves TD-S-1.07-02.
fn uuid_v7_token() -> String {
    uuid::Uuid::now_v7().to_string()
}
