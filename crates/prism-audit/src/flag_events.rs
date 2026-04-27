//! Feature flag evaluation audit events (BC-2.05.009, S-2.05).
//!
//! Provides [`FlagEvalDetail`] and [`emit_flag_eval()`] for auditing feature
//! flag evaluations on write/mutation MCP tool invocations.
//!
//! # Architecture compliance (S-2.05)
//!
//! - `FlagEvalDetail` is embedded in `AuditEntry.parameters` as JSON — it is
//!   NOT a separate RocksDB entry. It shares the `audit_buffer` CF with all
//!   other audit entries.
//! - The `resolution_trace` records the full hierarchical evaluation chain in
//!   evaluation order (most-specific to least-specific), enabling forensic
//!   analysis of why a write was permitted or denied (BC-2.05.009).
//! - `emit_flag_eval()` may be called with an empty `resolution_trace`
//!   (EC-004: no panic, entry still emitted with `resolution_trace: []`).

use prism_storage::audit_buffer;
use prism_storage::backend::RocksStorageBackend;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Resolution step ───────────────────────────────────────────────────────────

/// A single step in the feature flag hierarchical resolution chain (BC-2.05.009).
///
/// Steps are recorded in evaluation order (most-specific to least-specific,
/// ending at the global deny default). Each step records whether its rule
/// matched the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagResolutionStep {
    /// Identifier of the rule evaluated (e.g., `"R-001"`, `"global-deny-default"`).
    pub rule_id: String,
    /// Whether this rule matched the current request.
    pub matched: bool,
    /// Human-readable explanation (e.g., `"client_id matched allowlist rule R-001"`).
    ///
    /// Must be human-readable for audit entries to be useful without reference
    /// to internal rule IDs (Dev Notes).
    pub reason: String,
}

// ── Flag evaluation detail ────────────────────────────────────────────────────

/// Detail record embedded in `AuditEntry.parameters` for feature flag
/// evaluation events on write operations (BC-2.05.009, AC-3).
///
/// # Embedding
///
/// Serialise this struct and include it under key `"flag_eval_detail"`
/// in the `parameters` `serde_json::Value` before constructing `AuditEntry`.
///
/// # Empty resolution_trace (EC-004)
///
/// An empty `resolution_trace` is valid — if no rules were evaluated, the
/// slice is `[]`. Callers MUST NOT panic on this; `emit_flag_eval()` is
/// safe to call with an empty trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEvalDetail {
    /// Client / tenant for which the flag was evaluated.
    pub client_id: String,
    /// Dot-separated capability path evaluated (e.g., `"sensors.crowdstrike.write"`).
    pub capability_path: String,
    /// Final grant/deny decision after full hierarchical resolution.
    pub evaluation_result: bool,
    /// Ordered list of rules evaluated, most-specific first (BC-2.05.009).
    ///
    /// May be empty (EC-004) — still emitted, no panic.
    pub resolution_trace: Vec<FlagResolutionStep>,
}

// ── Emitter ───────────────────────────────────────────────────────────────────

/// Context of the write invocation that triggered flag evaluation (S-2.05).
///
/// Provides the `tool_name`, `client_id`, and `trace_id` needed to link the
/// flag evaluation event to the parent write tool invocation.
#[derive(Debug, Clone)]
pub struct FlagEvalContext {
    /// Name of the MCP write tool being authorized.
    pub tool_name: String,
    /// Client / tenant identifier.
    pub client_id: String,
    /// V7 UUID correlating this event to the parent tool invocation.
    pub trace_id: String,
}

/// Emit a feature flag evaluation audit entry (BC-2.05.009, AC-3).
///
/// Constructs an `AuditEntry` embedding `detail` in `parameters` under key
/// `"flag_eval_detail"`, then calls `append_audit_entry()` to persist the
/// entry to the `audit_buffer` CF.
///
/// Called from the feature flag evaluation path in `prism-flags` (story task 4).
///
/// # Empty resolution_trace (EC-004)
///
/// An empty `detail.resolution_trace` is valid — the entry is still emitted
/// with `resolution_trace: []`. This function MUST NOT panic on empty trace.
///
/// # Arguments
///
/// - `backend` — storage backend to persist the audit entry into
/// - `detail` — the populated `FlagEvalDetail` (with potentially empty trace)
/// - `ctx` — parent write invocation context
///
/// # Errors
///
/// Returns `prism_core::PrismError::AuditPersistenceFailed` if the audit
/// entry cannot be persisted.
pub fn emit_flag_eval<B: RocksStorageBackend>(
    backend: &B,
    detail: FlagEvalDetail,
    ctx: &FlagEvalContext,
) -> Result<(), prism_core::PrismError> {
    let parameters = serde_json::json!({
        "flag_eval_detail": detail_to_json(&detail).map_err(|e| prism_core::PrismError::Internal {
            detail: format!("flag eval event serialization failed: {e}"),
        })?
    });

    tracing::info!(
        tool_name = %ctx.tool_name,
        client_id = %ctx.client_id,
        trace_id = %ctx.trace_id,
        capability_path = %detail.capability_path,
        evaluation_result = %detail.evaluation_result,
        resolution_trace_len = %detail.resolution_trace.len(),
        parameters = %parameters,
        "flag_eval_event"
    );

    let timestamp_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
    let trace_id = Uuid::now_v7().to_string();
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("event_type".to_owned(), "flag_eval".to_owned());
    payload.insert("parameters".to_owned(), parameters.to_string());

    let entry = audit_buffer::AuditEntry {
        timestamp_ns,
        trace_id,
        payload,
    };

    audit_buffer::append_audit_entry(backend, &entry)
        .map_err(|_| prism_core::PrismError::AuditPersistenceFailed)
}

/// Serialise a [`FlagEvalDetail`] into a `serde_json::Value` for embedding
/// in `AuditEntry.parameters`.
///
/// # Errors
///
/// Returns `serde_json::Error` on serialisation failure (should not happen for
/// well-formed structs).
pub fn detail_to_json(detail: &FlagEvalDetail) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(detail)
}
