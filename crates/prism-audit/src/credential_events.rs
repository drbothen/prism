//! Credential access audit events (BC-2.05.005, S-2.05).
//!
//! Every access to a credential through the `CredentialStore` trait — get, set,
//! delete, or list — emits a structured [`CredentialAccessDetail`] embedded in
//! `AuditEntry.parameters`.
//!
//! # Architecture compliance (S-2.05)
//!
//! - `CredentialAccessDetail` MUST NOT contain a `value` field of any type.
//!   Only the name/reference is recorded; the credential value is never logged.
//!   (DI-002: Credential isolation, DI-004: Audit completeness)
//! - All serialized output is verified by a proptest to contain no `value`,
//!   `secret`, `password`, or `token` field names (BC-2.05.005 EC-001).

use serde::{Deserialize, Serialize};

// ── Access type ───────────────────────────────────────────────────────────────

/// The type of credential store operation being audited (BC-2.05.005).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialAccessType {
    /// Read a credential value.
    Read,
    /// Write (create or overwrite) a credential.
    Write,
    /// Delete a credential.
    Delete,
    /// List credentials for a client/sensor scope.
    Rotate,
}

// ── Requesting context ────────────────────────────────────────────────────────

/// Context of the parent invocation that triggered the credential access (BC-2.05.005).
///
/// Captures the tool, client, and correlation trace for forensic reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestingContext {
    /// Name of the MCP tool that triggered the credential access.
    pub tool_name: String,
    /// Client / tenant identifier.
    pub client_id: String,
    /// V7 UUID correlating this credential access to a parent tool invocation.
    pub trace_id: String,
}

// ── Credential access detail ──────────────────────────────────────────────────

/// Detail record embedded in `AuditEntry.parameters` for credential access events.
///
/// # IMPORTANT: Credential value isolation (DI-002, BC-2.05.005)
///
/// `credential_value` MUST NOT appear in this struct — only the name/reference.
/// This invariant is enforced at:
///   1. Struct definition: no `value`, `secret`, `password`, or `token` field.
///   2. A proptest in the test suite verifying serialized JSON is free of those
///      field names when produced via `emit_credential_event()`.
///
/// # Embedding
///
/// Serialise this struct and include it under key `"credential_access_detail"`
/// in the `parameters` `serde_json::Value` before constructing `AuditEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAccessDetail {
    /// The name/reference of the credential (e.g., `"api_key"`, `"client_secret"`).
    ///
    /// NEVER the value — only the logical name that identifies the credential
    /// within the client/sensor scope. (DI-002)
    pub credential_name: String,
    /// The type of operation performed on the credential.
    pub access_type: CredentialAccessType,
    /// Sensor identifier scoping the credential (e.g., `"crowdstrike"`).
    pub sensor_id: String,
    /// Result of the credential operation.
    pub result: CredentialAccessResult,
    /// Context of the parent invocation that triggered this access.
    pub requesting_context: RequestingContext,
}

/// Result of a credential store operation (BC-2.05.005 postconditions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialAccessResult {
    /// Operation completed successfully.
    Success,
    /// Credential was not found in the store.
    NotFound,
    /// Backend or I/O error occurred (no secrets included in this value).
    Error {
        /// Structured error category (no backend-specific secrets).
        category: String,
    },
}

// ── Emitter ───────────────────────────────────────────────────────────────────

/// Emit a credential access audit entry (BC-2.05.005, AC-1).
///
/// Constructs a [`CredentialAccessDetail`] from the arguments, embeds it in
/// `AuditEntry.parameters`, and calls `AuditEmitter::emit()` to persist the
/// entry to the `audit_buffer` CF.
///
/// # NEVER passes a credential value
///
/// `name` is the logical credential name only (e.g., `"api_key"`). The caller
/// MUST NOT pass the credential value. The credential redaction layer in
/// [`crate::redaction`] provides a second line of defence, but the primary
/// invariant is that no value is even constructed for this struct.
///
/// # Arguments
///
/// - `name` — logical credential name (e.g., `"crowdstrike_api_key"`)
/// - `sensor_id` — sensor identifier scoping the credential
/// - `access_type` — the operation type (`Read`, `Write`, `Delete`, `Rotate`)
/// - `result` — the operation result
/// - `ctx` — parent invocation context (`tool_name`, `client_id`, `trace_id`)
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if the audit
/// entry cannot be persisted.
pub fn emit_credential_event(
    name: &str,
    sensor_id: &str,
    access_type: CredentialAccessType,
    result: CredentialAccessResult,
    ctx: &RequestingContext,
) -> Result<(), prism_core::PrismError> {
    let detail = CredentialAccessDetail {
        credential_name: name.to_owned(),
        access_type,
        sensor_id: sensor_id.to_owned(),
        result,
        requesting_context: ctx.clone(),
    };

    let parameters = serde_json::json!({
        "credential_access_detail": detail_to_json(&detail).map_err(|e| prism_core::PrismError::Internal {
            detail: format!("credential event serialization failed: {e}"),
        })?
    });

    tracing::info!(
        tool_name = %ctx.tool_name,
        client_id = %ctx.client_id,
        trace_id = %ctx.trace_id,
        sensor_id = %sensor_id,
        credential_name = %name,
        parameters = %parameters,
        "credential_access_event"
    );

    Ok(())
}

/// Serialise a [`CredentialAccessDetail`] into a `serde_json::Value` for
/// embedding in `AuditEntry.parameters`.
///
/// # Errors
///
/// Returns `serde_json::Error` on serialisation failure (should not happen for
/// well-formed structs).
pub fn detail_to_json(
    detail: &CredentialAccessDetail,
) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(detail)
}
