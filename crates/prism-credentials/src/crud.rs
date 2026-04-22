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
//! - `list_credentials(client_id: null)`: returns `E-FLAG-006`.
//! - All operations accept source-type references ONLY — never raw values.
//! - All operations are audit-logged.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

/// Configure (create or update) a credential source reference.
///
/// # Contract: BC-2.03.005
/// - Create (no existing): returns `ConfigureCredentialResponse::Created` immediately.
/// - Update (existing): returns `ConfigureCredentialResponse::ConfirmationRequired`.
/// - Accepts `CredentialRef` only — never raw values.
///
/// # Errors
/// - `PrismError::InvalidCredentialName` if credential_name fails validation
/// - `PrismError::CredentialAccessDenied` if E-CRED-003 applies (raw value attempt)
pub async fn configure_credential_source(
    request: ConfigureCredentialRequest,
) -> Result<ConfigureCredentialResponse, prism_core::PrismError> {
    todo!("S-1.07: implement configure_credential_source")
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
    todo!("S-1.07: implement credential_status")
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
    todo!("S-1.07: implement delete_credential")
}

/// List credentials for a specific client/sensor (metadata only, never values).
///
/// # Contract: BC-2.03.005
/// - Requires non-null `client_id` — returns `E-FLAG-006` if null/empty.
/// - Returns metadata entries only.
pub async fn list_credentials(
    client_id: Option<&str>,
    sensor_id: Option<&str>,
) -> Result<Vec<CredentialMetadata>, prism_core::PrismError> {
    todo!("S-1.07: implement list_credentials")
}
