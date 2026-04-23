//! Credential access audit logging.
//!
//! # Contract: BC-2.03.010
//! Every credential store operation emits a structured `tracing::info!` log entry.
//! Fields: event_type, operation, client_id, sensor_id, credential_name, backend, result, timestamp.
//! Credential values are NEVER included in audit entries.
//! Failed access attempts are logged with the same detail as successful ones.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The operation type for an audit entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOperation {
    Get,
    Set,
    Delete,
    List,
}

impl std::fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditOperation::Get => "get",
            AuditOperation::Set => "set",
            AuditOperation::Delete => "delete",
            AuditOperation::List => "list",
        };
        write!(f, "{s}")
    }
}

/// The outcome of a credential access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    NotFound,
    Error,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditOutcome::Success => "success",
            AuditOutcome::NotFound => "not_found",
            AuditOutcome::Error => "error",
        };
        write!(f, "{s}")
    }
}

/// Structured audit event for a credential access.
///
/// Emitted via `tracing::info!` with `event_type = "credential_access"`.
/// The credential VALUE is NEVER included in this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: String, // always "credential_access"
    pub operation: AuditOperation,
    pub client_id: String,
    pub sensor_id: String,
    pub credential_name: String, // name only, NEVER value
    pub backend: String,
    pub result: AuditOutcome,
    pub timestamp: DateTime<Utc>,
}

impl AuditEvent {
    /// Construct a new AuditEvent with `event_type = "credential_access"`.
    pub fn new(
        operation: AuditOperation,
        client_id: impl Into<String>,
        sensor_id: impl Into<String>,
        credential_name: impl Into<String>,
        backend: impl Into<String>,
        result: AuditOutcome,
    ) -> Self {
        Self {
            event_type: "credential_access".to_string(),
            operation,
            client_id: client_id.into(),
            sensor_id: sensor_id.into(),
            credential_name: credential_name.into(),
            backend: backend.into(),
            result,
            timestamp: Utc::now(),
        }
    }

    /// Emit this audit event via `tracing::info!`.
    ///
    /// # Contract: BC-2.03.010
    /// If the tracing subscriber is unavailable, the credential operation still proceeds;
    /// this method makes a best-effort log (tracing handles no-subscriber gracefully).
    pub fn emit(&self) {
        tracing::info!(
            event_type = %self.event_type,
            operation = %self.operation,
            client_id = %self.client_id,
            sensor_id = %self.sensor_id,
            credential_name = %self.credential_name,
            backend = %self.backend,
            result = %self.result,
            timestamp = %self.timestamp,
            "credential access audit"
        );
    }
}

/// Emit a credential access audit entry.
///
/// Convenience wrapper: constructs and emits in one call.
/// Credential value is NEVER accepted as a parameter by design.
pub fn emit_audit(
    operation: AuditOperation,
    client_id: &str,
    sensor_id: &str,
    credential_name: &str,
    backend: &str,
    result: AuditOutcome,
) {
    AuditEvent::new(operation, client_id, sensor_id, credential_name, backend, result).emit();
}
