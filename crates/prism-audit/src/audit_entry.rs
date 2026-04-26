//! AuditEntry — compliance audit record for every MCP tool invocation.
//!
//! Covers SOC 2 Type II (who/what/when/where/outcome/authorization) and
//! ISO 27001 (data_classification, capability_checks, trace_id) field
//! requirements per BC-2.05.002 and BC-2.05.008.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Compile-time field completeness check (Task 7 / BC-2.05.008) ─────────────
//
// Verifies that AuditEntry has the fields required by SOC 2 Type II and
// ISO 27001. This check fires at build time — if any required field is
// removed, the build fails.
//
// We use static_assertions::assert_fields! which checks that the listed field
// names exist on the struct.
static_assertions::assert_fields!(
    AuditEntry: trace_id,
    timestamp,
    tool_name,
    client_id,
    parameters,
    outcome,
    duration_ms,
    data_classification,
    system_id,
    user_identity,
    result_summary,
    capability_checks,
    safety_flags
);

/// Classification of the sensitivity of an audit event's subject data
/// (ISO 27001 field requirement, BC-2.05.008).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    /// Publicly accessible data.
    Public,
    /// Internal-use data.
    Internal,
    /// Confidential data — restricted to authorised roles.
    Confidential,
    /// Restricted — highest sensitivity (e.g., credential operations).
    Restricted,
}

impl Default for DataClassification {
    /// Defaults to `Internal` per Dev Notes: all tool invocations are at
    /// minimum internal; `credential_*` tools override to `Confidential`.
    fn default() -> Self {
        Self::Internal
    }
}

/// Outcome of a tool invocation (BC-2.05.001, BC-2.05.002).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Tool invocation completed successfully.
    Success,
    /// Tool invocation failed with a structured error code.
    Failure {
        /// Structured error code (e.g., `"E-QUERY-001"`).
        error_code: String,
    },
}

/// Single capability check record for a write operation (BC-2.05.004, BC-2.05.008).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityCheckRecord {
    /// The evaluated capability path (e.g., `sensor.crowdstrike.containment`).
    pub capability_path: String,
    /// Whether the flag was enabled at compile time.
    pub compile_time_enabled: bool,
    /// Whether the flag was enabled at runtime.
    pub runtime_enabled: bool,
    /// Final access decision.
    pub result: CapabilityCheckResult,
}

/// Final decision of a single capability check (BC-2.05.004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCheckResult {
    /// Write was permitted by capability evaluation.
    Permitted,
    /// Write was denied; includes the specific capability path and reason.
    Denied { reason: String },
}

/// SOC 2 + ISO 27001 compliant audit record for a single MCP tool invocation.
///
/// Constructed by `AuditEmitter` and persisted to the `audit_buffer` CF via
/// `prism_storage::audit_buffer::append_audit_entry`. Field requirements per
/// BC-2.05.002 and BC-2.05.008.
///
/// # Immutability
///
/// Once emitted (BC-2.05.006), this struct is never modified. If a correction
/// is needed, a new `AuditEntry` is emitted referencing the original `trace_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    // ── SOC 2 / ISO 27001: incident response correlation ─────────────────────
    /// V7 UUID correlating all events for a single invocation (BC-2.05.006).
    pub trace_id: Uuid,

    // ── SOC 2: when ───────────────────────────────────────────────────────────
    /// RFC 3339 nanosecond-precision UTC timestamp (BC-2.05.002).
    pub timestamp: DateTime<Utc>,

    // ── SOC 2: what ───────────────────────────────────────────────────────────
    /// Name of the MCP tool called (e.g., `crowdstrike_contain_host`).
    pub tool_name: String,

    // ── SOC 2: who ────────────────────────────────────────────────────────────
    /// Analyst identity (TOML `analyst_id` → env → OS username).
    /// Set to `"unknown"` with `audit_warning` if unavailable (BC-2.05.008).
    pub user_identity: String,

    // ── SOC 2: where ─────────────────────────────────────────────────────────
    /// Client / tenant scoping the action (ISO 27001: "where").
    ///
    /// Sentinels (BC-2.05.002):
    /// - `"multi_client"` — fan-out query
    /// - `"all_clients"` — cross-client query with `clients: null`
    /// - `"cross_client"` — `client_id: null` on non-query tools
    /// - `"MISSING"` — request lacked `client_id` entirely (malformed)
    pub client_id: String,

    /// Fixed string `"prism"` — identifies the system in multi-system ISO 27001
    /// environments (BC-2.05.002, BC-2.05.008).
    pub system_id: String,

    // ── SOC 2: what (params, redacted) ───────────────────────────────────────
    /// Redacted copy of tool parameters (BC-2.05.003).
    ///
    /// Credential values are replaced with `"[REDACTED]"` before this field
    /// is populated — the raw credential value NEVER appears here, even transiently.
    pub parameters: serde_json::Value,

    // ── SOC 2: outcome ────────────────────────────────────────────────────────
    /// Success or structured failure (BC-2.05.001).
    pub outcome: AuditOutcome,

    /// Human-readable outcome summary for quick scanning.
    /// For write denials: `"denied_by_capability_check"` + path.
    /// For dry-runs: `"dry_run_preview"`.
    pub result_summary: String,

    /// Wall-clock duration from tool entry to audit emission (milliseconds).
    pub duration_ms: u64,

    /// Structured error code if `outcome == Failure` (e.g., `"E-QUERY-001"`).
    pub error_code: Option<String>,

    // ── ISO 27001: data classification ───────────────────────────────────────
    /// Sensitivity of the data touched by this invocation (BC-2.05.008).
    /// Defaults to `Internal`; overridden to `Confidential` for credential tools.
    pub data_classification: DataClassification,

    // ── SOC 2 / ISO 27001: authorization evidence ────────────────────────────
    /// Capability evaluations for write operations (BC-2.05.004, BC-2.05.008).
    /// Empty array (not omitted) for read operations.
    pub capability_checks: Vec<CapabilityCheckRecord>,

    // ── Injection detection ───────────────────────────────────────────────────
    /// Prompt injection safety flags triggered during this invocation.
    /// Empty array (not omitted) when none triggered.
    pub safety_flags: Vec<String>,

    // ── Optional warning annotation (BC-2.05.008 EC-05-013) ──────────────────
    /// Set to `"audit emission failed"` on read-op audit failure (read is not
    /// fail-closed). Absent on normal entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_warning: Option<String>,
}

impl AuditEntry {
    /// Construct an `AuditEntry` with all required compliance fields.
    ///
    /// # Parameters
    ///
    /// - `trace_id` — V7 UUID generated at the start of the tool invocation
    /// - `timestamp` — captured at tool entry (before inner handler)
    /// - `tool_name` — MCP tool name from the dispatch layer
    /// - `client_id` — resolved per BC-2.05.002 sentinel rules
    /// - `user_identity` — resolved at startup; `"unknown"` if unavailable
    /// - `parameters` — already-redacted parameters (`redact()` called before this)
    /// - `outcome` — result of the inner handler call
    /// - `result_summary` — human-readable outcome description
    /// - `duration_ms` — wall-clock duration of the invocation
    /// - `error_code` — structured error code if Failure
    /// - `data_classification` — from tool manifest; default `Internal`
    /// - `capability_checks` — filled for write ops; empty vec for read ops
    /// - `safety_flags` — injection detection records
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        trace_id: Uuid,
        timestamp: DateTime<Utc>,
        tool_name: String,
        client_id: String,
        user_identity: String,
        parameters: serde_json::Value,
        outcome: AuditOutcome,
        result_summary: String,
        duration_ms: u64,
        error_code: Option<String>,
        data_classification: DataClassification,
        capability_checks: Vec<CapabilityCheckRecord>,
        safety_flags: Vec<String>,
    ) -> Self {
        Self {
            trace_id,
            timestamp,
            tool_name,
            client_id,
            user_identity,
            parameters,
            outcome,
            result_summary,
            duration_ms,
            error_code,
            data_classification,
            system_id: "prism".to_owned(),
            capability_checks,
            safety_flags,
            audit_warning: None,
        }
    }
}
