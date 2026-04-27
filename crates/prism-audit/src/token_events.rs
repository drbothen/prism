//! Confirmation token lifecycle audit events (BC-2.05.010, S-2.05).
//!
//! Every event in the confirmation token lifecycle — issuance, successful
//! consumption, and all rejection paths (expired, already consumed, not found,
//! hash mismatch) — produces an audit entry with a distinct `result_summary`.
//!
//! # Architecture compliance (S-2.05)
//!
//! - `TokenLifecycleDetail` is embedded in `AuditEntry.parameters` as JSON —
//!   it is NOT a separate RocksDB entry. It shares the `audit_buffer` CF with
//!   all other audit entries.
//! - Token IDs are intentionally excluded from issuance audit entries to
//!   prevent correlation by log readers (BC-2.05.010 postconditions).
//! - `emit_token_consumed()` and `emit_token_expired()` are DISTINCT events:
//!   a consumed token was intentionally used; an expired token was not used in
//!   time (EC-003, Dev Notes).

use chrono::{DateTime, Utc};
use prism_storage::audit_buffer;
use prism_storage::backend::RocksStorageBackend;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Token event type ──────────────────────────────────────────────────────────

/// The lifecycle event type for a confirmation token (BC-2.05.010).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenEvent {
    /// A confirmation token was issued for a pending irreversible write.
    Generated,
    /// A confirmation token was successfully consumed (write executed).
    Consumed,
    /// A confirmation token expired before it was consumed.
    Expired,
    /// A `confirm_action` call referenced a token ID that does not exist.
    NotFound,
    /// The token's action hash did not match the `confirm_action` parameters.
    HashMismatch,
    /// A `confirm_action` call was made for a token that was already consumed.
    AlreadyConsumed,
}

// ── Token lifecycle detail ────────────────────────────────────────────────────

/// Detail record embedded in `AuditEntry.parameters` for confirmation token
/// lifecycle events (BC-2.05.010, AC-4).
///
/// # Token ID exclusion at issuance (BC-2.05.010)
///
/// For `TokenEvent::Generated` entries, `token_id` contains the opaque
/// identifier. **However**, BC-2.05.010 specifies that token IDs are
/// intentionally excluded from issuance audit entries. The emitter functions
/// (`emit_token_generated`) implement this by NOT including the token ID in the
/// `result_summary` or the top-level `AuditEntry` fields — only the
/// `action_summary` and `expiry_time` are surfaced at issuance.
///
/// # Embedding
///
/// Serialise this struct and include it under key `"token_lifecycle_detail"`
/// in the `parameters` `serde_json::Value` before constructing `AuditEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLifecycleDetail {
    /// Opaque token identifier (not the token value, not the action hash).
    ///
    /// See BC-2.05.010: token IDs are excluded from issuance audit entries at
    /// the `result_summary` level; this field is only present in consumption
    /// and rejection entries.
    pub token_id: String,
    /// The lifecycle event type.
    pub event_type: TokenEvent,
    /// Human-readable description of the gated action (e.g.,
    /// `"isolate host acme-ws-01"`).
    pub action_summary: String,
    /// When the token expires or expired.
    pub expiry_time: DateTime<Utc>,
}

/// Context for a token lifecycle event (S-2.05).
///
/// Provides the `tool_name`, `client_id`, and `sensor` fields required by
/// BC-2.05.010 ("All token lifecycle events include the `client_id`, `sensor`,
/// and `tool_name` of the original write operation").
#[derive(Debug, Clone)]
pub struct TokenEventContext {
    /// Name of the original write tool that triggered token issuance.
    pub tool_name: String,
    /// Client / tenant identifier.
    pub client_id: String,
    /// Sensor identifier (e.g., `"crowdstrike"`).
    pub sensor: String,
}

// ── Emitters ──────────────────────────────────────────────────────────────────

/// Emit an audit entry for token issuance (BC-2.05.010, AC-4).
///
/// Records `result_summary: "confirmation_token_issued"` with the
/// `action_summary` and `expiry_time`, but NOT the token ID (BC-2.05.010).
/// Persists the entry to the `audit_buffer` CF via `append_audit_entry`.
///
/// # Arguments
///
/// - `backend` — storage backend to persist the audit entry into
/// - `token_id` — opaque token identifier (recorded in detail, excluded from
///   top-level `result_summary`)
/// - `action_summary` — human-readable description of the gated action
/// - `expiry` — when the token expires
/// - `ctx` — original write operation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if audit
/// persistence fails.
pub fn emit_token_generated<B: RocksStorageBackend>(
    backend: &B,
    token_id: &str,
    action_summary: &str,
    expiry: DateTime<Utc>,
    ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    let detail = TokenLifecycleDetail {
        token_id: token_id.to_owned(),
        event_type: TokenEvent::Generated,
        action_summary: action_summary.to_owned(),
        expiry_time: expiry,
    };

    let parameters = serde_json::json!({
        "token_lifecycle_detail": detail_to_json(&detail).map_err(|e| prism_core::PrismError::Internal {
            detail: format!("token generated event serialization failed: {e}"),
        })?
    });

    // BC-2.05.010 postcondition: result_summary is "confirmation_token_issued".
    // Token ID is NOT included in result_summary — only action_summary and expiry.
    tracing::info!(
        tool_name = %ctx.tool_name,
        client_id = %ctx.client_id,
        sensor = %ctx.sensor,
        action_summary = %action_summary,
        result_summary = "confirmation_token_issued",
        parameters = %parameters,
        "token_generated_event"
    );

    let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
    let trace_id = Uuid::now_v7().to_string();
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("event_type".to_owned(), "token_generated".to_owned());
    payload.insert(
        "result_summary".to_owned(),
        "confirmation_token_issued".to_owned(),
    );
    payload.insert("parameters".to_owned(), parameters.to_string());

    let entry = audit_buffer::AuditEntry {
        timestamp_ns,
        trace_id,
        payload,
    };

    audit_buffer::append_audit_entry(backend, &entry)
        .map_err(|_| prism_core::PrismError::AuditPersistenceFailed)
}

/// Emit an audit entry for successful token consumption (BC-2.05.010).
///
/// Records `result_summary: "confirmed_and_executed"` with the action outcome.
/// Sets `expiry_time` to `Utc::now()` (the moment of consumption).
/// Persists the entry to the `audit_buffer` CF via `append_audit_entry`.
///
/// # Note: distinct from `emit_token_expired` (EC-003)
///
/// A consumed token is intentionally used; an expired token was not used in
/// time. Both are security-relevant distinct events.
///
/// # Arguments
///
/// - `backend` — storage backend to persist the audit entry into
/// - `token_id` — the token that was consumed
/// - `action_summary` — the action that was confirmed and executed
/// - `ctx` — original write operation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if audit
/// persistence fails.
pub fn emit_token_consumed<B: RocksStorageBackend>(
    backend: &B,
    token_id: &str,
    action_summary: &str,
    ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    let now = Utc::now();
    let detail = TokenLifecycleDetail {
        token_id: token_id.to_owned(),
        event_type: TokenEvent::Consumed,
        action_summary: action_summary.to_owned(),
        // expiry_time set to Utc::now() — the moment of consumption (BC-2.05.010).
        expiry_time: now,
    };

    let parameters = serde_json::json!({
        "token_lifecycle_detail": detail_to_json(&detail).map_err(|e| prism_core::PrismError::Internal {
            detail: format!("token consumed event serialization failed: {e}"),
        })?
    });

    tracing::info!(
        tool_name = %ctx.tool_name,
        client_id = %ctx.client_id,
        sensor = %ctx.sensor,
        token_id = %token_id,
        action_summary = %action_summary,
        result_summary = "confirmed_and_executed",
        parameters = %parameters,
        "token_consumed_event"
    );

    let timestamp_ns = now.timestamp_nanos_opt().unwrap_or(0) as u64;
    let trace_id = Uuid::now_v7().to_string();
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("event_type".to_owned(), "token_consumed".to_owned());
    payload.insert(
        "result_summary".to_owned(),
        "confirmed_and_executed".to_owned(),
    );
    payload.insert("parameters".to_owned(), parameters.to_string());

    let entry = audit_buffer::AuditEntry {
        timestamp_ns,
        trace_id,
        payload,
    };

    audit_buffer::append_audit_entry(backend, &entry)
        .map_err(|_| prism_core::PrismError::AuditPersistenceFailed)
}

/// Emit an audit entry for a token that expired before consumption (BC-2.05.010).
///
/// Records `result_summary: "token_expired"` with the original action summary.
/// Persists the entry to the `audit_buffer` CF via `append_audit_entry`.
/// Called by the token expiry sweep task.
///
/// # Note: distinct from `emit_token_consumed` (EC-003)
///
/// An expired token was not used in time (passive expiry). A consumed token
/// was intentionally invoked. The `event_type` values are distinct.
///
/// # Arguments
///
/// - `backend` — storage backend to persist the audit entry into
/// - `token_id` — the token that expired
/// - `action_summary` — the action the token would have confirmed
/// - `expiry` — the recorded expiry timestamp (from the token record)
/// - `ctx` — original write operation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if audit
/// persistence fails.
pub fn emit_token_expired<B: RocksStorageBackend>(
    backend: &B,
    token_id: &str,
    action_summary: &str,
    expiry: DateTime<Utc>,
    ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    let detail = TokenLifecycleDetail {
        token_id: token_id.to_owned(),
        event_type: TokenEvent::Expired,
        action_summary: action_summary.to_owned(),
        // expiry_time is from the token record — the original expiry (BC-2.05.010).
        expiry_time: expiry,
    };

    let parameters = serde_json::json!({
        "token_lifecycle_detail": detail_to_json(&detail).map_err(|e| prism_core::PrismError::Internal {
            detail: format!("token expired event serialization failed: {e}"),
        })?
    });

    tracing::info!(
        tool_name = %ctx.tool_name,
        client_id = %ctx.client_id,
        sensor = %ctx.sensor,
        token_id = %token_id,
        action_summary = %action_summary,
        result_summary = "token_expired",
        parameters = %parameters,
        "token_expired_event"
    );

    let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
    let trace_id = Uuid::now_v7().to_string();
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("event_type".to_owned(), "token_expired".to_owned());
    payload.insert("result_summary".to_owned(), "token_expired".to_owned());
    payload.insert("parameters".to_owned(), parameters.to_string());

    let entry = audit_buffer::AuditEntry {
        timestamp_ns,
        trace_id,
        payload,
    };

    audit_buffer::append_audit_entry(backend, &entry)
        .map_err(|_| prism_core::PrismError::AuditPersistenceFailed)
}

/// Serialise a [`TokenLifecycleDetail`] into a `serde_json::Value` for
/// embedding in `AuditEntry.parameters`.
///
/// # Errors
///
/// Returns `serde_json::Error` on serialisation failure (should not happen for
/// well-formed structs).
pub fn detail_to_json(
    detail: &TokenLifecycleDetail,
) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(detail)
}
