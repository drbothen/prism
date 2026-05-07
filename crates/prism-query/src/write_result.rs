//! Write result envelope types for the S-3.07 write execution pipeline.
//!
//! Defines the output types for both dry-run and live-execute paths:
//! - [`WriteResult`]              — actual execution outcome
//! - [`WritePreview`]             — dry-run preview (BC-2.04.008)
//! - [`ConfirmationTokenPreview`] — embedded token for irreversible dry-runs
//! - [`SensorWriteError`]         — per-sensor error accumulation
//!
//! Per-record types (`RecordWriteResult`, `WriteStatus`) are defined in
//! `prism-sensors` (the crate that owns `SensorAdapter::write()`) and
//! re-exported here for convenience.
//!
//! # Architecture Compliance
//! - All types are pure data — no I/O, no async, no DataFusion imports.
//! - `WritePreview.confirmation_prompt` MUST be derived from structured query
//!   plan fields only — not from analyst-supplied free-text (prompt injection
//!   defense, Dev Notes).
//!
//! Story: S-3.07 | BCs: BC-2.04.007, BC-2.04.008

// Stub module: all non-trivial bodies are todo!() pending implementation.
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use prism_core::RiskTier;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

// Re-export per-record types from prism-sensors (they live there because
// SensorAdapter::write() returns them directly).
pub use prism_sensors::write_result::{RecordWriteResult, WriteStatus};

// ---------------------------------------------------------------------------
// SensorWriteError — per-sensor error accumulation
// ---------------------------------------------------------------------------

/// An error from a single sensor write step, accumulated in `WriteResult.sensor_errors`.
///
/// A non-empty `sensor_errors` list does not constitute a top-level error return —
/// partial success is handled in `WriteResult.failed_count` (story §Partial batch failure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorWriteError {
    /// The sensor name that produced this error (e.g., `"crowdstrike"`).
    pub sensor: String,
    /// The client ID for which the error occurred.
    pub client_id: String,
    /// Structured error code (e.g., `"E-SENSOR-001"`).
    pub error_code: String,
    /// Human-readable error detail.
    pub detail: String,
}

// ---------------------------------------------------------------------------
// WriteResult — live execution outcome
// ---------------------------------------------------------------------------

/// Output of a successful (non-dry-run) write execution (Phase 6).
///
/// Returned as `WriteOutcome::Result(write_result)` by `WriteExecutor::execute`.
///
/// `dry_run` is always `false` in a `WriteResult` — see `WritePreview` for
/// the dry-run envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// Unique identifier for this write operation (ULID).
    pub operation_id: Ulid,
    /// Always `false` — `WriteResult` represents a live execution.
    pub dry_run: bool,
    /// Target write endpoint (e.g., `"crowdstrike.contained_hosts"`).
    pub write_endpoint: String,
    /// Risk tier of the executed operation (BC-2.04.007).
    pub risk_tier: RiskTier,
    /// Confirmation token ID consumed for this write, if `risk_tier = Irreversible`.
    pub confirmed_by_token: Option<String>,
    /// Wall-clock time the write execution began.
    pub execution_started_at: DateTime<Utc>,
    /// Wall-clock time the write execution completed (all records attempted).
    pub execution_completed_at: DateTime<Utc>,
    /// Audit INTENT record ID written in Phase 5a (BC-2.05.009).
    pub audit_intent_id: Ulid,
    /// Total number of records that were targeted by this write.
    pub affected_count: u32,
    /// Number of records that wrote successfully to the sensor API.
    pub succeeded_count: u32,
    /// Number of records that failed at the sensor API.
    pub failed_count: u32,
    /// Per-record outcomes (Phase 5c fan-out results).
    pub per_record_results: Vec<RecordWriteResult>,
    /// Per-sensor errors accumulated during fan-out.
    pub sensor_errors: Vec<SensorWriteError>,
}

// ---------------------------------------------------------------------------
// ConfirmationTokenPreview — embedded in WritePreview for Irreversible tier
// ---------------------------------------------------------------------------

/// Preview of the confirmation token generated for an irreversible dry-run.
///
/// Embedded in `WritePreview.confirmation_token` when `risk_tier = Irreversible`
/// and `dry_run = true`. The analyst (or calling agent) must supply the
/// `token_id` in the follow-up execute call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationTokenPreview {
    /// The token ID to supply in the follow-up execute call.
    pub token_id: String,
    /// Token expiry time (now + 300s, per BC-2.04.011 / TOKEN_TTL).
    pub expires_at: DateTime<Utc>,
    /// SHA-256 content hash of the action parameters (BC-2.04.012).
    pub action_hash: String,
    /// Prompt-injection-safe action summary (structured fields only — Dev Notes).
    pub action_summary: String,
}

// ---------------------------------------------------------------------------
// WritePreview — dry-run preview envelope
// ---------------------------------------------------------------------------

/// Output of a dry-run write execution (Phase 4, `dry_run = true`).
///
/// Returned as `WriteOutcome::Preview(write_preview)` by `WriteExecutor::execute`.
///
/// `dry_run` is always `true` in a `WritePreview` — no sensor API was contacted.
/// The `confirmation_token` field is populated only when `risk_tier = Irreversible`
/// (BC-2.04.008, BC-2.04.007).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritePreview {
    /// Query identifier (ULID).
    pub query_id: Ulid,
    /// Always `true` — `WritePreview` represents a dry-run (no sensor API contact).
    pub dry_run: bool,
    /// Target write endpoint (e.g., `"crowdstrike.contained_hosts"`).
    pub write_endpoint: String,
    /// Risk tier of the previewed operation (BC-2.04.007).
    pub risk_tier: RiskTier,
    /// Number of records that WOULD be affected if executed.
    pub would_affect_count: u32,
    /// First 5 records from the fetch phase for analyst review.
    ///
    /// Serialized as JSON arrays (Arrow `RecordBatch` is not serializable directly).
    pub sample_records: Vec<serde_json::Value>,
    /// Reversibility classification (mirrors `risk_tier`).
    pub reversibility: RiskTier,
    /// Confirmation token preview — present only when `risk_tier = Irreversible`.
    ///
    /// `None` for `Reversible` tier operations.
    pub confirmation_token: Option<ConfirmationTokenPreview>,
    /// Prompt-injection-safe confirmation prompt for the calling agent.
    ///
    /// Derived from structured query plan fields (counts, endpoint, client ID)
    /// — never from analyst free-text input (Dev Notes / prompt injection defense).
    pub confirmation_prompt: String,
}
