//! `explain` — PrismQL query plan analysis without execution.
//!
//! Implements BC-2.11.010: parse and plan a PrismQL query without issuing any
//! sensor API calls. Returns a structured `ExplainResult` containing the
//! detected query mode, alias expansions, field resolution, per-sensor push-down
//! filters in sensor-native syntax, post-fetch operations, and a cost estimate.
//!
//! # Architecture Compliance (BC-2.11.010)
//! - MUST NOT call `fan_out()` or any sensor adapter `fetch()` method.
//! - Reuses `classify_predicates()` from `pushdown.rs` — do NOT duplicate logic.
//! - Reuses `resolve_clients()` from `scoping.rs`.
//! - DataFusion logical plan obtained against schema-only MemTables.
//! - An audit entry MUST be emitted for every invocation (DI-004).
//! - Syntactic security limits apply; materialization limit produces a warning,
//!   not an error (DI-019).
//!
//! # BC References
//! - BC-2.11.010 — `explain_query` MCP Tool
//! - BC-2.11.007 — Sensor Filter Push-Down (reused push-down classification)
//! - BC-2.11.011 — Cross-Client Query Scoping (reused resolve_clients)
//!
//! Story: S-3.03

// Stub phase: dead_code / unused variable warnings are expected.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use prism_core::{OrgSlug, PrismError, SensorType};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ExplainOptions
// ---------------------------------------------------------------------------

/// Per-call options for the `explain_query` MCP tool.
///
/// Mirrors the scoping parameters of the `query` tool (BC-2.11.010 Preconditions).
#[derive(Debug, Clone, Default)]
pub struct ExplainOptions {
    /// Client scope override: `None` = all configured clients. (BC-2.11.011)
    pub clients: Option<Vec<OrgSlug>>,
    /// Sensor scope override: `None` = all sensors for resolved clients. (BC-2.11.010)
    pub sensors: Option<Vec<SensorType>>,
    /// Data source scope override: `None` = all sources for resolved sensors. (BC-2.11.010)
    pub sources: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// ExplainResult (BC-2.11.010 Postconditions)
// ---------------------------------------------------------------------------

/// The output of a successful `explain_query` invocation.
///
/// Implements BC-2.11.010 postconditions. All fields are JSON-serializable
/// using standard types — no custom serializer required (Story §Architecture
/// Compliance Rules).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    /// The detected query mode: `"filter"`, `"sql"`, or `"pipe"`. (BC-2.11.010)
    pub parsed_mode: String,

    /// The raw query string as provided by the caller. (BC-2.11.010)
    pub original_query: String,

    /// Map of alias names to their expanded definitions. Empty if no aliases
    /// were used. (BC-2.11.010)
    pub alias_expansion: std::collections::HashMap<String, String>,

    /// The query after all alias expansion. (BC-2.11.010)
    pub expanded_query: String,

    /// Map of field names used in the query to their OCSF paths and how they
    /// were resolved (direct, alias, or virtual). (BC-2.11.010)
    pub field_resolution: std::collections::HashMap<String, FieldResolution>,

    /// The execution plan showing sensors, push-down filters, and post-fetch
    /// operations. (BC-2.11.010)
    pub execution_plan: ExecutionPlan,

    /// Structured cost estimate for the query. (BC-2.11.010)
    pub estimated_cost: CostEstimate,
}

// ---------------------------------------------------------------------------
// FieldResolution
// ---------------------------------------------------------------------------

/// How a single field name in the query was resolved to an OCSF path.
///
/// Part of `ExplainResult.field_resolution`. (BC-2.11.010)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldResolution {
    /// OCSF path the field maps to (e.g., `"finding.severity_id"`).
    pub ocsf_path: String,
    /// Resolution method: `"direct"`, `"alias"`, or `"virtual"`.
    pub resolution_method: String,
}

// ---------------------------------------------------------------------------
// ExecutionPlan
// ---------------------------------------------------------------------------

/// The execution plan produced without running the query. (BC-2.11.010)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// List of sensors that would be queried. (BC-2.11.010)
    pub sensors_to_query: Vec<ExplainSource>,

    /// Per-source post-fetch operations (filter, group-by, sort, limit, etc.).
    /// (BC-2.11.010)
    pub post_fetch_operations: Vec<String>,
}

// ---------------------------------------------------------------------------
// ExplainSource
// ---------------------------------------------------------------------------

/// Per-sensor push-down information for the explain result. (BC-2.11.010)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainSource {
    /// Source reference string, e.g. `"crowdstrike.detections"`.
    pub source_ref: String,

    /// The sensor type this source belongs to.
    pub sensor_type: SensorType,

    /// Push-down predicates translated to sensor-native syntax
    /// (e.g., FQL for CrowdStrike). (BC-2.11.010 `api_filters_pushed`)
    pub api_filters_pushed: Vec<String>,

    /// Predicates applied post-materialization by DataFusion. (BC-2.11.010)
    pub post_filter_predicates: Vec<String>,

    /// Estimated row count from sensor count hint, if available.
    /// `None` if the sensor adapter does not expose a count hint. (Story §Dev Notes)
    pub estimated_row_count: Option<u64>,
}

// ---------------------------------------------------------------------------
// CostEstimate
// ---------------------------------------------------------------------------

/// Structured cost estimate for the query. (BC-2.11.010 `estimated_cost`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Per-sensor estimated latency based on rolling historical averages.
    /// Map from sensor identifier to estimated latency in milliseconds.
    pub per_sensor_latency_ms: std::collections::HashMap<String, u64>,

    /// Estimated API call count per sensor based on expected pagination depth.
    pub per_sensor_api_call_count: std::collections::HashMap<String, u64>,

    /// Rate limit headroom per sensor (remaining calls in the current window).
    pub per_sensor_rate_limit_headroom: std::collections::HashMap<String, u64>,

    /// Human-readable summary combining latency, call count, and rate limit
    /// into an actionable estimate for the analyst or AI agent.
    pub summary: String,

    /// Warnings attached to this estimate. Non-empty when, e.g., the estimated
    /// record count exceeds the 10K materialization limit (EC-11-025, DI-019).
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// QueryEngine::explain (BC-2.11.010)
// ---------------------------------------------------------------------------

// NOTE: `QueryEngine::explain` is declared here as a free function stub
// because `QueryEngine` is defined in `engine.rs` (an `impl` block cannot
// span files in Rust without a trait). The implementer must move this into
// an `impl QueryEngine` block in `engine.rs` or expose it via a trait.
//
// Alternatively, the implementer may restructure as:
//   - `explain.rs` exports `ExplainResult` + `ExplainOptions` types.
//   - `engine.rs` `impl QueryEngine` gains `pub async fn explain(...)`.
//
// The stub is provided here per the story file-structure requirements.

/// Analyze a PrismQL query string and return an `ExplainResult` without
/// executing any sensor API calls.
///
/// # Contract (BC-2.11.010)
/// 1. Parses `query_str` via `PrismQlParser::parse` — returns parse errors
///    immediately without attempting plan generation.
/// 2. Applies syntactic security limits (DI-019); returns `E-QUERY-003` if exceeded.
/// 3. Resolves alias expansion and field resolution — returns errors if needed.
/// 4. Resolves client scope via `scoping::resolve_clients` — NO fan-out.
/// 5. Classifies predicates via `pushdown::classify_predicates` — NO sensor calls.
/// 6. Builds DataFusion `LogicalPlan` against schema-only MemTables (no data).
/// 7. Computes cost estimate from historical latency records (read-only).
/// 8. Emits an audit entry (DI-004) recording query, scoping params, and summary.
///
/// # Returns
/// - `Ok(ExplainResult)` on success (including the EC-11-025 over-limit warning).
/// - `Err(PrismError)` for parse errors, alias errors, field errors, or security
///   limit violations.
///
/// # No sensor API calls
/// This function MUST NOT call `fan_out()`, any sensor adapter `fetch()`, or
/// any I/O path that reaches a sensor API endpoint. (BC-2.11.010 Postconditions)
pub fn explain(_query_str: &str, _options: ExplainOptions) -> Result<ExplainResult, PrismError> {
    todo!("S-3.03 — QueryEngine::explain (BC-2.11.010): parse + plan without fan_out")
}
