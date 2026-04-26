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
use serde::{Deserialize, Serialize};

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
///
/// # Arguments
///
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
pub fn emit_token_generated(
    _token_id: &str,
    _action_summary: &str,
    _expiry: DateTime<Utc>,
    _ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    todo!(
        "AC-4 / BC-2.05.010: construct TokenLifecycleDetail with event_type: Generated, \
         embed in AuditEntry.parameters[\"token_lifecycle_detail\"], set \
         result_summary: \"confirmation_token_issued\", DO NOT expose token_id in \
         result_summary (BC-2.05.010 postcondition)."
    )
}

/// Emit an audit entry for successful token consumption (BC-2.05.010).
///
/// Records `result_summary: "confirmed_and_executed"` with the action outcome.
/// Sets `expiry_time` to `Utc::now()` (the moment of consumption).
///
/// # Note: distinct from `emit_token_expired` (EC-003)
///
/// A consumed token is intentionally used; an expired token was not used in
/// time. Both are security-relevant distinct events.
///
/// # Arguments
///
/// - `token_id` — the token that was consumed
/// - `action_summary` — the action that was confirmed and executed
/// - `ctx` — original write operation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if audit
/// persistence fails.
pub fn emit_token_consumed(
    _token_id: &str,
    _action_summary: &str,
    _ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    todo!(
        "BC-2.05.010: construct TokenLifecycleDetail with event_type: Consumed, \
         expiry_time: Utc::now(), embed in AuditEntry.parameters, set \
         result_summary: \"confirmed_and_executed\". DISTINCT from emit_token_expired."
    )
}

/// Emit an audit entry for a token that expired before consumption (BC-2.05.010).
///
/// Records `result_summary: "token_expired"` with the original action summary.
/// Called by the token expiry sweep task.
///
/// # Note: distinct from `emit_token_consumed` (EC-003)
///
/// An expired token was not used in time (passive expiry). A consumed token
/// was intentionally invoked. The `event_type` values are distinct.
///
/// # Arguments
///
/// - `token_id` — the token that expired
/// - `action_summary` — the action the token would have confirmed
/// - `expiry` — the recorded expiry timestamp (from the token record)
/// - `ctx` — original write operation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if audit
/// persistence fails.
pub fn emit_token_expired(
    _token_id: &str,
    _action_summary: &str,
    _expiry: DateTime<Utc>,
    _ctx: &TokenEventContext,
) -> Result<(), prism_core::PrismError> {
    todo!(
        "BC-2.05.010: construct TokenLifecycleDetail with event_type: Expired, \
         embed in AuditEntry.parameters, set result_summary: \"token_expired\". \
         DISTINCT from emit_token_consumed."
    )
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
