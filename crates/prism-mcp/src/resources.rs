//! MCP resource handler functions for `PrismServer` (BC-2.10.008, BC-2.08.006).
//!
//! Implements the following `prism://` URI resources:
//! - `prism://config/clients` — all configured client IDs with sensor inventory
//! - `prism://config/clients/{client_id}/sensors` — per-client sensor configs
//! - `prism://schema/{sensor_id}/{table_name}` — OCSF schema for a sensor+table
//! - `prism://sensors/health` — cached sensor health data (BC-2.08.006)
//!
//! S-DEMO-PRISMQL-ONBOARDING-001-A adds (BC-2.10.013, BC-2.10.014):
//! - `prismql://schema/{client_id}` — per-client PQL table/column/type schema catalog
//! - `prismql://reference` — runtime-assembled PQL grammar reference (built via `build_reference_content()`)
//!
//! Resources are served by overriding `list_resources`, `list_resource_templates`,
//! and `read_resource` on `impl ServerHandler for PrismServer` in `server.rs`.
//! There is NO `#[resource_handler]` macro in rmcp 1.7 — confirmed against rmcp source.
//!
//! # Credential Redaction (VP-050)
//!
//! All resource response serialization MUST redact API keys and full URL paths.
//! Only host+port components are emitted for URL fields (VP-050, BC-2.10.008 postcondition).

/// `prismql://schema/{client_id}` resource template and `prismql://reference` runtime resource.
///
/// `prismql://schema/{client_id}` (BC-2.10.013) returns the full PQL table/column/type
/// catalog for a given client as structured JSON. `prismql://reference` (BC-2.10.014)
/// serves the PQL grammar reference assembled at runtime by `build_reference_content()`.
pub mod schema;

use std::{collections::BTreeSet, sync::Arc};

use chrono::{DateTime, Utc};
use rmcp::model::{
    AnnotateAble, ErrorCode, ErrorData, ListResourceTemplatesResult, ListResourcesResult,
    RawResource, RawResourceTemplate, ReadResourceResult, ResourceContents, Role,
};
use serde::{Deserialize, Serialize};

use crate::context::PrismContext;

// ─── Public response types (BC-2.10.008, BC-2.08.006) ────────────────────────

/// Per-client summary entry in `prism://config/clients` response (BC-2.10.008 postcondition 1).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInventoryEntry {
    /// The client identifier (OrgSlug).
    pub client_id: String,
    /// Human-readable display name for the client (BC-2.10.008).
    ///
    /// Sourced from `[[orgs]].name` in `prism.toml` (`OrgEntry.name`).
    /// Serializes as JSON `null` when the org has no configured display name.
    pub display_name: Option<String>,
    /// Number of sensors configured for this client.
    pub sensor_count: usize,
    /// Sensor IDs enabled for this client.
    pub enabled_sensors: Vec<String>,
}

/// Per-sensor config entry in `prism://config/clients/{client_id}/sensors`
/// response (BC-2.10.008 postcondition 2).
///
/// BC-2.10.008: `api_base_url` MUST be present and contain only the
/// scheme+host+port component (e.g., `"https://api.crowdstrike.com"`).
/// Full URL paths, query strings, and credentials MUST NOT appear (VP-050, DI-002).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfigEntry {
    /// Sensor type identifier (e.g., "crowdstrike", "claroty").
    pub sensor_type: String,
    /// Operational status of the sensor.
    pub status: String,
    /// Credential reference (name or path — never the raw credential value).
    pub credential_ref: String,
    /// Data source identifiers (table names) for this sensor.
    pub sources: Vec<String>,
    /// API base URL — scheme+host+port ONLY (VP-050 / BC-2.10.008 postcondition 2).
    /// Full path, query string, and credentials MUST NOT appear here.
    pub api_base_url: String,
}

/// Health result for a single sensor — stored in the health cache (BC-2.08.005, BC-2.08.006).
///
/// BC-2.08.005 two-phase probe model:
/// - S-5.03 scope (`probe_level: "spec-only"`): `reachable` and `auth_valid` are `None`
///   (honest-unknown — no live probe has been performed). Hardcoding `true` is FORBIDDEN.
/// - S-5.04 scope (`probe_level: "live"`): `reachable` and `auth_valid` are `Some(bool)`
///   from actual API probe results.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHealthResult {
    /// Sensor identifier.
    pub sensor_id: String,
    /// Client identifier — always present (BC-2.08.005 postcondition).
    pub client_id: String,
    /// Probe level: `"spec-only"` (S-5.03) or `"live"` (S-5.04).
    /// Must never be absent.
    pub probe_level: String,
    /// Whether the sensor API endpoint was reachable.
    /// `None` for spec-only scope (no live probe); `Some(bool)` for live probe (S-5.04).
    /// MUST NOT be hardcoded `Some(true)` in S-5.03 scope — that is a false-positive signal.
    pub reachable: Option<bool>,
    /// Whether the provided credentials were accepted by the sensor.
    /// `None` for spec-only scope; `Some(bool)` for live probe (S-5.04).
    pub auth_valid: Option<bool>,
    /// Rate-limit information (None if not applicable or unavailable).
    pub rate_limit: Option<RateLimitInfo>,
    /// Timestamp of the last successful query to this sensor.
    /// `None` for spec-only scope (no query has run); `Some(DateTime)` after a live query.
    pub last_successful_query_at: Option<DateTime<Utc>>,
    /// Sanitised error text (prompt-injection-safe), if the health check failed.
    pub error: Option<String>,
    /// Round-trip probe latency in milliseconds (F-S504-P1-MED-001 / AC-1).
    ///
    /// `Some(ms)` for live probes where the sensor responded (connectivity Up).
    /// `None` when the sensor was unreachable (Down), not yet probed (spec-only),
    /// or when latency could not be meaningfully measured.
    pub latency_ms: Option<u64>,
    /// Remediation guidance for unhealthy, auth-invalid, or rate-limited sensors.
    ///
    /// `None` for healthy sensors. Populated by `check_one` / `check_sensor_health` when
    /// the sensor is not fully healthy (BC-2.08.007 postcondition — suggestion field).
    ///
    /// Examples (verbatim BC-2.08.007 text):
    /// - Rate-limited:  `"Rate limit in effect — wait before retrying."`
    /// - Auth-invalid:  `"Check credentials — sensor rejected authentication."`
    /// - Degraded (5xx): `"Sensor returned a server error (5xx) — service may be temporarily unavailable."`
    /// - Unreachable:   `"Sensor unreachable — verify network and endpoint configuration."`
    ///
    /// BC-2.08.001 EC-08-001 (F-S504-LP1P1-MED-001): Degraded (ConnectivityStatus::Degraded,
    /// HTTP 5xx) and Down (connection error) produce distinct suggestions.  `check_one` sets
    /// `result.error = Some("service_unavailable")` for Degraded probes; the suggestion ladder
    /// in `check_sensor_health` uses this marker to dispatch the correct string.
    pub suggestion: Option<String>,
}

impl SensorHealthResult {
    /// Create a new `SensorHealthResult` in spec-only scope (BC-2.08.005 S-5.03 contract).
    ///
    /// `probe_level` is `"spec-only"`. `reachable`, `auth_valid`, and
    /// `last_successful_query_at` are all `None` (honest-unknown — no live probe).
    pub fn new(sensor_id: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            client_id: client_id.into(),
            probe_level: "spec-only".to_string(),
            reachable: None,
            auth_valid: None,
            rate_limit: None,
            last_successful_query_at: None,
            error: None,
            latency_ms: None,
            suggestion: None,
        }
    }

    /// Builder: set `reachable` to `Some(bool)` (S-5.04 live-probe use only).
    ///
    /// NOTE: Do NOT call this in S-5.03 scope — `reachable` must remain `None` for
    /// spec-only responses (BC-2.08.005 postcondition, hardcoded-true prohibition).
    #[allow(dead_code)]
    pub fn with_reachable(mut self, reachable: bool) -> Self {
        self.reachable = Some(reachable);
        self
    }

    /// Builder: set `auth_valid` to `Some(bool)` (S-5.04 live-probe use only).
    ///
    /// NOTE: Do NOT call this in S-5.03 scope — `auth_valid` must remain `None`.
    #[allow(dead_code)]
    pub fn with_auth_valid(mut self, auth_valid: bool) -> Self {
        self.auth_valid = Some(auth_valid);
        self
    }

    /// Builder: set `last_successful_query_at`.
    pub fn with_last_successful_query_at(mut self, at: DateTime<Utc>) -> Self {
        self.last_successful_query_at = Some(at);
        self
    }

    /// Builder: set sanitised error text.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Builder: set remediation suggestion for unhealthy/rate-limited sensors.
    ///
    /// Called by `check_one` / `check_sensor_health` when the sensor is not fully
    /// healthy (BC-2.08.007 postcondition — suggestion field).
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Rate limit state for a sensor (BC-2.08.005 postcondition field).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Remaining requests in the current window (None if unavailable).
    pub remaining: Option<u32>,
    /// Total requests allowed in the window (None if unavailable).
    pub limit: Option<u32>,
    /// UTC time when the window resets (None if unavailable).
    pub reset_at: Option<DateTime<Utc>>,
}

impl RateLimitInfo {
    /// Construct a `RateLimitInfo` with only the `reset_at` timestamp set.
    ///
    /// Used in tests and health-checker paths where only the retry deadline is known
    /// (e.g., parsed from `Retry-After` header). `remaining` and `limit` are `None`.
    pub fn new_with_reset_at(reset_at: DateTime<Utc>) -> Self {
        Self {
            remaining: None,
            limit: None,
            reset_at: Some(reset_at),
        }
    }
}

/// Resource pressure section in `check_sensor_health` response (BC-2.08.005 postcondition).
///
/// BC-2.08.005 two-phase resource_pressure behavior (RECONCILIATION-3 anchor):
/// - S-5.03 scope: both counts are `None` — emitted as JSON `null` so the AI consumer can
///   distinguish "not yet wired" from a genuine zero (hardcoded `0` is FORBIDDEN in S-5.03).
/// - S-5.04 scope: live counts wired via `QueryEngine::cursor_count()` / `::token_count()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePressure {
    /// Current number of non-expired cursors (out of 200 cap).
    /// `None` in S-5.03 scope (not yet wired); `Some(usize)` in S-5.04 after live wiring.
    pub active_cursor_count: Option<usize>,
    /// Current number of unexpired, unconsumed confirmation tokens (out of 100 cap).
    /// `None` in S-5.03 scope (not yet wired); `Some(usize)` in S-5.04 after live wiring.
    pub active_token_count: Option<usize>,
}

impl ResourcePressure {
    /// Construct a ResourcePressure snapshot.
    ///
    /// Pass `None` for both counts in S-5.03 scope to emit JSON `null` (not `0`).
    /// Pass `Some(n)` in S-5.04 scope with live-wired count values.
    pub fn new(active_cursor_count: Option<usize>, active_token_count: Option<usize>) -> Self {
        Self {
            active_cursor_count,
            active_token_count,
        }
    }
}

/// Summary counts object for `check_sensor_health` response (BC-2.08.007 postcondition).
///
/// Provides structured counts that enable AI consumers to quickly triage the health picture
/// without scanning individual sensor entries. Serializes into `summary_counts` in the
/// `SensorHealthStructuredContent` JSON response.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Number of sensors that are fully healthy (reachable + auth valid, not rate-limited).
    pub healthy_count: usize,
    /// Number of sensors that are NOT fully healthy (unreachable, auth-invalid, or rate-limited).
    pub unhealthy_count: usize,
    /// Total number of sensors probed.
    pub total_count: usize,
    /// Number of sensors that are rate-limited (subset of unhealthy_count).
    pub rate_limited_count: usize,
}

impl HealthSummary {
    /// Compute a `HealthSummary` from a slice of `SensorHealthResult` entries.
    pub fn from_results(results: &[SensorHealthResult]) -> Self {
        let total_count = results.len();
        let healthy_count = results
            .iter()
            .filter(|r| {
                r.reachable == Some(true) && r.auth_valid == Some(true) && r.rate_limit.is_none()
            })
            .count();
        let rate_limited_count = results.iter().filter(|r| r.rate_limit.is_some()).count();
        let unhealthy_count = total_count.saturating_sub(healthy_count);
        Self {
            healthy_count,
            unhealthy_count,
            total_count,
            rate_limited_count,
        }
    }
}

/// Top-level structured content shape for `check_sensor_health` (BC-2.08.005, BC-2.08.007).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHealthStructuredContent {
    /// Per-sensor health results.
    pub sensors: Vec<SensorHealthResult>,
    /// Aggregate resource pressure snapshot.
    pub resource_pressure: ResourcePressure,
    /// Trust level — always `"internal"` (health data is Prism-generated).
    pub trust_level: String,
    /// Prose summary text (e.g., "2 of 3 sensors healthy for client 'acme'").
    pub summary: String,
    /// Aggregate status string (BC-2.08.007 postcondition).
    ///
    /// Values (verbatim BC-2.08.007 postcondition):
    /// - `"healthy"` — all sensors reachable, auth valid, not rate-limited
    /// - `"partial"` — at least one sensor is unreachable or auth-invalid
    /// - `"rate_limited"` — ALL sensors rate-limited, none unreachable/auth-invalid (EC-08-015)
    /// - `"unhealthy"` — all sensors unreachable or auth-invalid
    pub overall_status: String,
    /// Structured summary counts (BC-2.08.007 postcondition).
    ///
    /// Contains `healthy_count`, `unhealthy_count`, `total_count`, `rate_limited_count`.
    pub summary_counts: HealthSummary,
}

impl SensorHealthStructuredContent {
    /// Construct a `SensorHealthStructuredContent` with the given sensors, summary, and aggregate status.
    ///
    /// `trust_level` is always `"internal"` — it is set unconditionally here per
    /// BC-2.08.005 postcondition 7 (health data is Prism-generated, not sensor-sourced).
    ///
    /// `overall_status` is the serialized `OverallStatus` string (e.g., `"rate_limited"`).
    /// `summary_counts` is computed from `sensors` via `HealthSummary::from_results`.
    pub fn new(
        sensors: Vec<SensorHealthResult>,
        resource_pressure: ResourcePressure,
        summary: impl Into<String>,
    ) -> Self {
        let overall_status = "healthy".to_string(); // default; callers override via new_with_status
        let summary_counts = HealthSummary::from_results(&sensors);
        Self {
            sensors,
            resource_pressure,
            trust_level: "internal".to_string(),
            summary: summary.into(),
            overall_status,
            summary_counts,
        }
    }

    /// Construct a `SensorHealthStructuredContent` with an explicit `overall_status` string.
    ///
    /// Use this constructor in the live-probe path where `OverallStatus` is computed by
    /// `HealthCheckResult::aggregate` and must be serialized into the response.
    ///
    /// `overall_status_str` MUST be one of `"healthy"`, `"partial"`, `"rate_limited"`,
    /// or `"unhealthy"` (verbatim BC-2.08.007 postcondition classification table).
    pub fn new_with_status(
        sensors: Vec<SensorHealthResult>,
        resource_pressure: ResourcePressure,
        summary: impl Into<String>,
        overall_status_str: impl Into<String>,
    ) -> Self {
        let summary_counts = HealthSummary::from_results(&sensors);
        Self {
            sensors,
            resource_pressure,
            trust_level: "internal".to_string(),
            summary: summary.into(),
            overall_status: overall_status_str.into(),
            summary_counts,
        }
    }
}

// ─── URI constants ─────────────────────────────────────────────────────────────

pub const URI_CONFIG_CLIENTS: &str = "prism://config/clients";
pub const URI_SENSORS_HEALTH: &str = "prism://sensors/health";
pub const URI_TEMPLATE_CLIENT_SENSORS: &str = "prism://config/clients/{client_id}/sensors";
pub const URI_TEMPLATE_SCHEMA: &str = "prism://schema/{sensor_id}/{table_name}";

// ─── Internal error helpers ────────────────────────────────────────────────────

fn not_found_error(msg: impl Into<String>) -> ErrorData {
    ErrorData::new(
        ErrorCode(-32602), // INVALID_PARAMS — resource not found
        msg.into(),
        None,
    )
}

fn internal_error(msg: impl Into<String>) -> ErrorData {
    ErrorData::new(
        ErrorCode(-32000), // INTERNAL_ERROR
        msg.into(),
        None,
    )
}

/// Sanitize a display_name string for safe inclusion in AI agent contexts (SEC-003 / DI-006).
///
/// Applies two transformations:
/// 1. **128-char cap**: truncates to at most 128 characters (prevents context-stuffing).
/// 2. **Printable-ASCII filter**: replaces any control character or non-printable byte
///    with a space (prevents control-char injection / ANSI escape attacks).
///
/// Called at the read site in `render_client_list_resource` before `display_name` is
/// included in the resource response forwarded to AI agent contexts.
fn sanitize_display_name(name: &str) -> String {
    // Step 1: cap at 128 chars (character boundary, not byte boundary).
    // Use char_indices enumerated by position so no manual counter is needed.
    let capped: &str = {
        let byte_end = name
            .char_indices()
            .nth(128)
            .map(|(idx, _)| idx)
            .unwrap_or(name.len());
        &name[..byte_end]
    };

    // Step 2: replace control characters and non-printable bytes with a space.
    // Printable ASCII is 0x20–0x7E. Non-ASCII (multi-byte) chars are kept as-is
    // (they are not control characters and do not pose injection risk via ANSI escapes).
    capped
        .chars()
        .map(|c| {
            if c.is_ascii() && !c.is_ascii_graphic() && c != ' ' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

// ─── list_resources implementation ──────────────────────────────────────────────

/// Build the static list of concrete (non-templated) resources.
///
/// Called from `ServerHandler::list_resources` override on `PrismServer`.
///
/// Includes `prismql://reference` (BC-2.10.014) as a static resource added by
/// S-DEMO-PRISMQL-ONBOARDING-001-A.
///
/// Content delivery is handled by `dispatch_read_resource` →
/// `build_reference_content` (runtime-assembled; ADR-045 §A / CRIT-001).
pub fn build_resource_list() -> ListResourcesResult {
    let resources = vec![
        RawResource::new(URI_CONFIG_CLIENTS, "Prism Client Inventory")
            .with_description("All configured client IDs with sensor counts and enabled sensors.")
            .with_mime_type("application/json")
            .no_annotation(),
        RawResource::new(URI_SENSORS_HEALTH, "Prism Sensor Health")
            .with_description(
                "Cached sensor health data from the last check_sensor_health invocation.",
            )
            .with_mime_type("application/json")
            .no_annotation(),
        // L3: PQL grammar reference resource (BC-2.10.014 — S-DEMO-PRISMQL-ONBOARDING-001-A).
        // Content assembled at runtime via build_reference_content (ADR-045 §A / CRIT-001).
        // No subscribe/listChanged (content is stable per session — BC-2.10.014).
        // BC-2.10.014 AC-007: annotations.priority=0.8 + audience=["assistant"] required
        // (high-value reference material targeted at LLM agents, not human users).
        RawResource::new(schema::URI_PQL_REFERENCE, "PrismQL Grammar Reference")
            .with_description(
                "Full PrismQL grammar reference — SELECT/FROM/WHERE/GROUP BY/ORDER BY/LIMIT, \
                 operators, datetime arithmetic, error quick-reference, and self-correction workflow.",
            )
            .with_mime_type("text/markdown")
            .with_priority(0.8)
            .with_audience(vec![Role::Assistant]),
    ];
    ListResourcesResult {
        resources,
        next_cursor: None,
        meta: None,
    }
}

/// Build the list of URI-template resources.
///
/// Called from `ServerHandler::list_resource_templates` override on `PrismServer`.
///
/// Includes `prismql://schema/{client_id}` (BC-2.10.013) as a URI template added by
/// S-DEMO-PRISMQL-ONBOARDING-001-A.
///
/// Content delivery is handled by `render_pql_schema_resource` in
/// `resources/schema.rs`. Subscribe/notify is dispatched via `notify_schema_updated`.
pub fn build_resource_template_list() -> ListResourceTemplatesResult {
    let resource_templates = vec![
        RawResourceTemplate::new(URI_TEMPLATE_CLIENT_SENSORS, "Prism Client Sensor Config")
            .with_description(
                "Sensor configuration for a specific client. Substitute {client_id} with \
             the target client identifier.",
            )
            .with_mime_type("application/json")
            .no_annotation(),
        RawResourceTemplate::new(URI_TEMPLATE_SCHEMA, "Prism Sensor Schema")
            .with_description(
                "OCSF schema definition for a specific sensor and table. Substitute \
                 {sensor_id} and {table_name} with the target values.",
            )
            .with_mime_type("application/json")
            .no_annotation(),
        // L2: Per-client PQL schema resource template (BC-2.10.013 — S-DEMO-PRISMQL-ONBOARDING-001-A).
        // Content is structurally identical to prism_describe(client_id) (single-source-of-truth).
        // Supports server-side subscribe/notify (NET-NEW machinery — see resources/schema.rs).
        RawResourceTemplate::new(schema::URI_TEMPLATE_PQL_SCHEMA, "PrismQL Client Schema")
            .with_description(
                "Per-client PQL table/column/type schema catalog. Substitute {client_id} with \
                 the target client identifier. Subscribe to receive notifications when the \
                 client's sensor schema changes.",
            )
            .with_mime_type("application/json")
            .no_annotation(),
    ];
    ListResourceTemplatesResult {
        resource_templates,
        next_cursor: None,
        meta: None,
    }
}

// ─── read_resource dispatch ───────────────────────────────────────────────────

/// Dispatch a `read_resource` request to the correct resource handler.
///
/// Matches the incoming `uri` against the known `prism://` URI patterns and
/// delegates to the appropriate handler function. Returns a 404-equivalent
/// `ErrorData` for unknown URIs.
///
/// Called from `ServerHandler::read_resource` override on `PrismServer`.
pub async fn dispatch_read_resource(
    uri: &str,
    context: &PrismContext,
    query_engine: Option<&Arc<prism_query::engine::QueryEngine>>,
    config_manager: Option<
        &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
) -> Result<ReadResourceResult, ErrorData> {
    // Extract org_registry and resolved_spec_map from the query engine (IMP-8).
    // These are `None` when the engine is in test / MVP mode without multi-tenant support.
    let org_registry = query_engine.and_then(|qe| qe.org_registry());
    let resolved_spec_map = query_engine.and_then(|qe| qe.resolved_spec_map());

    // Exact match: prism://config/clients
    if uri == URI_CONFIG_CLIENTS {
        match (config_manager, query_engine) {
            (Some(cm), Some(qe)) => {
                return render_client_list_resource(
                    cm,
                    qe,
                    org_registry.as_ref(),
                    resolved_spec_map.as_ref(),
                )
                .await
            }
            _ => {
                // Fallback when not fully wired (test construction): return empty array
                let text = serde_json::to_string(&Vec::<ClientInventoryEntry>::new())
                    .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    text, uri,
                )]));
            }
        }
    }

    // Exact match: prism://sensors/health
    if uri == URI_SENSORS_HEALTH {
        return render_sensors_health_resource(context);
    }

    // Template match: prism://config/clients/{client_id}/sensors
    if let Some(client_id) = extract_template_param(uri, "prism://config/clients/", "/sensors") {
        match config_manager {
            Some(cm) => {
                return render_client_sensors_resource(
                    client_id,
                    cm,
                    resolved_spec_map.as_ref(),
                    org_registry.as_ref(),
                )
                .await
            }
            None => {
                return Err(not_found_error(
                    "Client sensors resource not available (config manager not wired)",
                ))
            }
        }
    }

    // Template match: prism://schema/{sensor_id}/{table_name}
    if let Some(rest) = uri.strip_prefix("prism://schema/") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 {
            let sensor_id = parts[0];
            let table_name = parts[1];
            match config_manager {
                Some(cm) => return render_schema_resource(sensor_id, table_name, cm).await,
                None => {
                    return Err(not_found_error(
                        "Schema resource not available (config manager not wired)",
                    ))
                }
            }
        }
    }

    // Exact match: prismql://reference (BC-2.11.022, ADR-045 §A — runtime reference content).
    // build_reference_content generates the PQL grammar reference dynamically, incorporating
    // live InfusionRegistry data so the returned document reflects currently-loaded enrichment
    // UDFs at query time (CRIT-001, closed S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001).
    if uri == schema::URI_PQL_REFERENCE {
        let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());
        let content = build_reference_content(infusion_registry.as_deref());
        return Ok(ReadResourceResult::new(vec![ResourceContents::text(
            content,
            schema::URI_PQL_REFERENCE,
        )]));
    }

    // Template match: prismql://schema/{client_id} (BC-2.10.013 — per-client schema catalog)
    if let Some(client_id) = uri.strip_prefix("prismql://schema/") {
        // BC-2.10.013 EC-10-033: validate client_id via OrgSlug::new — rejects all invalid
        // formats: empty, path-traversal ('..' or '/'), and invalid chars (e.g., 'acme!').
        // Using OrgSlug::new as the SINGLE gate ensures all rejection paths return the
        // canonical EC-10-033 error at dispatch time, not a deeper different-string rejection
        // from inside render_pql_schema_resource.
        // DI-006: do NOT echo the raw client_id in the error message.
        if prism_core::OrgSlug::new(client_id).is_ok() {
            return schema::render_pql_schema_resource(client_id, query_engine, config_manager)
                .await;
        } else {
            return Err(not_found_error(
                "Invalid client_id in resource URI".to_string(),
            ));
        }
    }

    // DI-006: do NOT echo the raw `uri` in the error message — the URI is attacker-controlled
    // input that feeds into AI agent contexts. Echoing it is an injection/echo surface.
    // Use a generic, non-echoing message consistent with the pattern applied throughout
    // this file for invalid client_id, sensor_id, and table_name parameters.
    Err(not_found_error(
        "Unknown or unsupported resource URI".to_string(),
    ))
}

/// Extract a URI template parameter between a prefix and a suffix.
///
/// Returns `Some(param)` if `uri` starts with `prefix` and ends with `suffix`.
fn extract_template_param<'a>(uri: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let after_prefix = uri.strip_prefix(prefix)?;
    let param = after_prefix.strip_suffix(suffix)?;
    // Reject empty params and path-traversal attempts
    if param.is_empty() || param.contains('/') || param.contains("..") {
        return None;
    }
    Some(param)
}

/// Validate a URI path segment used as a resource identifier (sensor_id or table_name).
///
/// Rejects:
/// - Empty strings
/// - Path-traversal sequences (`..`, `/`)
/// - ASCII control characters (0x00–0x1F, 0x7F)
/// - Non-ASCII bytes (non-printable or multi-byte sequences)
///
/// Returns `Ok(())` if the segment is safe for use in lookups and error messages.
/// Returns `Err(())` if the segment fails any check.
///
/// Used by `render_schema_resource` (DI-006 / BC-2.10.008 postcondition: attacker-controlled
/// input must never appear verbatim in MCP responses forwarded to AI agent contexts).
fn validate_resource_path_segment(segment: &str) -> Result<(), ()> {
    if segment.is_empty() {
        return Err(());
    }
    // Reject path traversal
    if segment.contains("..") || segment.contains('/') {
        return Err(());
    }
    // Reject control characters and non-ASCII bytes
    if !segment
        .chars()
        .all(|c| c.is_ascii() && !c.is_ascii_control())
    {
        return Err(());
    }
    Ok(())
}

// ─── prism://config/clients ───────────────────────────────────────────────────

/// Handle `prism://config/clients` resource read (BC-2.10.008 postcondition 1).
///
/// # Per-org scoping (IMP-8 / BC-2.10.008)
///
/// When `org_registry` and `resolved_spec_map` are both wired (production multi-tenant
/// mode), this function enumerates all registered org slugs via `org_registry.list_slugs()`
/// and for each slug counts sensor entries from `resolved_spec_map`.  An org with zero
/// overlay entries is listed with `sensor_count=0` and `enabled_sensors=[]`
/// (BC-2.10.008 Option B semantics: overlay = provisioned, not "customize a global
/// default").
///
/// When `org_registry` or `resolved_spec_map` is `None` (test / MVP mode without
/// multi-tenant support), falls back to the TableRegistry intersection logic. In this
/// path each `ClientInventoryEntry` represents one sensor, and `enabled_sensors` carries
/// `[sensor_id]` — a sensor-ID string per BC-2.10.008 postcondition 1 ("array of sensor
/// ID strings provisioned for this client"). Table names are NOT emitted in this field
/// in either path.
///
/// Returns a JSON array of `ClientInventoryEntry` objects.
pub async fn render_client_list_resource(
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    query_engine: &Arc<prism_query::engine::QueryEngine>,
    org_registry: Option<&Arc<prism_core::OrgRegistry>>,
    resolved_spec_map: Option<
        &Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    >,
) -> Result<ReadResourceResult, ErrorData> {
    let entries: Vec<ClientInventoryEntry> =
        if let (Some(org_reg), Some(spec_map)) = (org_registry, resolved_spec_map) {
            // ── Per-org path (IMP-8 / BC-2.10.008) ──────────────────────────
            // Enumerate all registered org slugs. For each org, collect the sensor IDs
            // that have an overlay entry in resolved_spec_map.
            //
            // BC-2.10.008: source display_name from org_display_names in the snapshot.
            // Populated at boot step 4b.5 from [[orgs]].name in prism.toml (OrgEntry.name).
            // No new Arc plumbing — read from config_manager snapshot directly.
            let cm_guard = config_manager.load();
            let snapshot = cm_guard.load();
            let org_display_names = &snapshot.org_display_names;

            let mut result: Vec<ClientInventoryEntry> = org_reg
                .list_slugs()
                .into_iter()
                .map(|slug_str| {
                    // Count sensors for this org: all keys where OrgSlug == slug.
                    // resolved_spec_map key = (OrgSlug, SensorId).
                    // We match by string representation to avoid needing OrgSlug constructor.
                    let sensors_for_org: Vec<String> = spec_map
                        .keys()
                        .filter(|(org, _sensor)| org.as_str() == slug_str.as_str())
                        .map(|(_org, sensor_id)| sensor_id.as_ref().to_string())
                        .collect();
                    let sensor_count = sensors_for_org.len();
                    // BC-2.10.008: look up display_name from org_display_names.
                    // None when the org has no name configured in prism.toml.
                    // SEC-003 / DI-006: sanitize before emitting to AI agent context —
                    // apply 128-char cap and control-char replacement at the read site.
                    let display_name = org_display_names
                        .get(slug_str.as_str())
                        .cloned()
                        .unwrap_or(None)
                        .map(|n| sanitize_display_name(&n));
                    ClientInventoryEntry {
                        client_id: slug_str,
                        display_name,
                        sensor_count,
                        enabled_sensors: sensors_for_org,
                    }
                })
                .collect();
            // Sort by client_id for deterministic output.
            result.sort_by(|a, b| a.client_id.cmp(&b.client_id));
            result
        } else {
            // ── TableRegistry intersection fallback (test / MVP mode) ─────────────
            // Used when org_registry or resolved_spec_map is None (e.g., existing
            // single-sensor tests, AC-1, AC-8, EC-10-014 fixture).
            let cm_guard = config_manager.load();
            let snapshot = cm_guard.load();

            let spec_sensor_ids: BTreeSet<String> = snapshot.sensor_specs.keys().cloned().collect();

            let registry_sensor_ids: BTreeSet<String> =
                if let Some(registry) = query_engine.table_registry() {
                    registry.registered_sensor_ids().into_iter().collect()
                } else {
                    BTreeSet::new()
                };

            let enabled_sensors: Vec<String> = spec_sensor_ids
                .intersection(&registry_sensor_ids)
                .cloned()
                .collect();

            // EC-10-014: if no sensors are registered, return [] (empty array).
            // BC-2.10.008 postcondition 1: `enabled_sensors` must carry sensor ID strings,
            // NOT table names. In this single-entry-per-sensor fallback model, each entry
            // represents one sensor, so `enabled_sensors = [sensor_id]`.
            enabled_sensors
                .iter()
                .map(|sensor_id| ClientInventoryEntry {
                    client_id: sensor_id.clone(),
                    // Fallback path: no org_display_names available (test/MVP mode).
                    // display_name is always null in this path — no prism.toml org context.
                    display_name: None,
                    sensor_count: 1,
                    // BC-2.10.008 postcondition 1: sensor ID strings, not table names.
                    enabled_sensors: vec![sensor_id.clone()],
                })
                .collect()
        };

    let text = serde_json::to_string(&entries)
        .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;

    Ok(ReadResourceResult::new(vec![ResourceContents::text(
        text,
        URI_CONFIG_CLIENTS,
    )]))
}

// ─── prism://config/clients/{client_id}/sensors ───────────────────────────────

/// Handle `prism://config/clients/{client_id}/sensors` resource read
/// (BC-2.10.008 postcondition 2).
///
/// Validates `client_id` via `OrgSlug::new()` (same guard as tool calls).
/// Returns a 404-equivalent `ErrorData` on invalid `client_id`.
///
/// # Per-org scoping (IMP-8 / BC-2.10.008)
///
/// When `resolved_spec_map` is wired (production multi-tenant mode):
/// - Filters `resolved_spec_map` by `OrgSlug == client_id` to return only that
///   org's provisioned sensors.
/// - When `org_registry` is also wired, validates that `client_id` is a known org
///   and returns a 404 error for unregistered orgs (BC-2.10.008 error case).
/// - An org registered in `OrgRegistry` with zero overlay entries returns an empty
///   array (EC-10-017 / BC-2.10.008 Option B semantics).
/// - Removes the `sensor_id == client_id` stopgap (DI-008 fix).
///
/// When `resolved_spec_map` is `None` (test / MVP mode without multi-tenant support):
/// - Falls back to `config_manager.sensor_specs` filtered by `sensor_id == client_id`
///   (existing single-tenant behavior retained for AC-2 fixture compatibility).
///
/// Returns a JSON array of `SensorConfigEntry` objects.
pub async fn render_client_sensors_resource(
    client_id: &str,
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    resolved_spec_map: Option<
        &Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    >,
    org_registry: Option<&Arc<prism_core::OrgRegistry>>,
) -> Result<ReadResourceResult, ErrorData> {
    // Validate client_id via OrgSlug — rejects path traversal and invalid chars.
    // BC-2.10.008 prompt-injection defense: do NOT echo the raw untrusted client_id
    // in the error message — attacker-controlled input must never appear verbatim
    // in MCP responses forwarded to AI agent contexts (BC-2.10.008 postcondition,
    // DI-006 invariant). Use a generic rejection message instead.
    let org_slug = prism_core::OrgSlug::new(client_id);
    if org_slug.is_err() {
        return Err(not_found_error(
            "Resource not found: invalid client_id (path traversal or invalid characters rejected)"
                .to_string(),
        ));
    }

    let entries: Vec<SensorConfigEntry> = if let Some(spec_map) = resolved_spec_map {
        // ── Per-org path (IMP-8 / DI-008 / BC-2.10.008) ───────────────────
        // Validate that the org is registered when org_registry is available.
        // An unregistered org returns a 404-equivalent error.
        if let Some(reg) = org_registry {
            if !reg.slug_exists(&org_slug) {
                return Err(not_found_error(
                    "Resource not found: client not found".to_string(),
                ));
            }
        }

        // Filter resolved_spec_map by OrgSlug == client_id.
        // Each entry for this org yields one SensorConfigEntry.
        // An org with zero overlay entries → empty vec (EC-10-017 / Option B semantics).
        let mut sensors: Vec<SensorConfigEntry> = spec_map
            .iter()
            .filter(|((org, _sensor), _spec)| org.as_str() == org_slug.as_str())
            .map(|((_org, sensor_id), resolved)| {
                let spec = &resolved.spec;
                let table_names: Vec<String> =
                    spec.tables.iter().map(|t| t.table_name.clone()).collect();
                let cred_ref = spec
                    .credential_refs
                    .first()
                    .map(|c| c.name.as_str())
                    .unwrap_or("");
                render_sensor_inventory_resource(
                    sensor_id.as_ref(),
                    cred_ref,
                    &spec.base_url,
                    &table_names,
                )
            })
            .collect();
        // Sort by sensor_type for deterministic output.
        sensors.sort_by(|a, b| a.sensor_type.cmp(&b.sensor_type));
        sensors
    } else {
        // ── ConfigManager fallback (test / MVP / single-tenant mode) ─────────────
        // The `sensor_id == client_id` filter remains here for backward compatibility
        // with existing tests that supply only config_manager (no resolved_spec_map).
        // This path is only reached when resolved_spec_map is None.
        let cm_guard = config_manager.load();
        let snapshot = cm_guard.load();

        snapshot
            .sensor_specs
            .values()
            .filter(|spec| spec.sensor_id == client_id)
            .map(|spec| {
                let table_names: Vec<String> =
                    spec.tables.iter().map(|t| t.table_name.clone()).collect();
                let cred_ref = spec
                    .credential_refs
                    .first()
                    .map(|c| c.name.as_str())
                    .unwrap_or("");
                render_sensor_inventory_resource(
                    &spec.sensor_id,
                    cred_ref,
                    &spec.base_url,
                    &table_names,
                )
            })
            .collect()
    };

    let uri = format!("prism://config/clients/{client_id}/sensors");
    let text = serde_json::to_string(&entries)
        .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;

    Ok(ReadResourceResult::new(vec![ResourceContents::text(
        text, uri,
    )]))
}

// ─── prism://schema/{sensor_id}/{table_name} ──────────────────────────────────

/// Handle `prism://schema/{sensor_id}/{table_name}` resource read (BC-2.10.008).
///
/// Looks up the OCSF schema definition from the spec engine ConfigSnapshot.
/// Returns a 404-equivalent ErrorData if the sensor+table combination is unknown.
///
/// # Prompt-Injection Defense (DI-006)
///
/// Both `sensor_id` and `table_name` are attacker-controlled URI path segments.
/// This function validates them before use and MUST NOT echo the raw values in
/// error messages — doing so would allow injection of arbitrary text into AI agent
/// contexts via a crafted URI (BC-2.10.008 postcondition, DI-006 invariant).
///
/// Validation rejects: path traversal (`..`, `/`), control characters, non-ASCII.
/// Error messages are generic and contain no attacker-controlled content.
pub async fn render_schema_resource(
    sensor_id: &str,
    table_name: &str,
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
) -> Result<ReadResourceResult, ErrorData> {
    // DI-006: validate both path segments before any lookup or error message
    // construction — reject path traversal / control chars / non-ASCII.
    if validate_resource_path_segment(sensor_id).is_err() {
        return Err(not_found_error(
            "Schema not found: invalid sensor_id (path traversal or invalid characters rejected)"
                .to_string(),
        ));
    }
    if validate_resource_path_segment(table_name).is_err() {
        return Err(not_found_error(
            "Schema not found: invalid table_name (path traversal or invalid characters rejected)"
                .to_string(),
        ));
    }

    let cm_guard = config_manager.load();
    let snapshot = cm_guard.load();

    // Look up sensor spec — generic error message, no echo of sensor_id.
    let spec = match snapshot.sensor_specs.get(sensor_id) {
        Some(s) => s,
        None => {
            return Err(not_found_error(
                "Schema not found: sensor not configured".to_string(),
            ))
        }
    };

    // Find the matching table — generic error message, no echo of table_name.
    let table_spec = match spec.tables.iter().find(|t| t.table_name == table_name) {
        Some(t) => t,
        None => {
            return Err(not_found_error(
                "Schema not found: table not available for this sensor".to_string(),
            ))
        }
    };

    let uri = format!("prism://schema/{sensor_id}/{table_name}");
    let text = serde_json::to_string(table_spec)
        .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;

    Ok(ReadResourceResult::new(vec![ResourceContents::text(
        text, uri,
    )]))
}

// ─── prism://sensors/health ───────────────────────────────────────────────────

/// Handle `prism://sensors/health` resource read (BC-2.08.006).
///
/// Returns cached health data from the last `check_sensor_health` invocation.
/// If no health check has been run for any client: returns the "unknown" sentinel.
/// If any cache entry is older than 5 minutes, the response includes `"stale": true`
/// at the response root (EC-003). Staleness is a response-level flag, not per-entry.
pub fn render_sensors_health_resource(
    context: &PrismContext,
) -> Result<ReadResourceResult, ErrorData> {
    if context.health_cache.is_empty() {
        // BC-2.08.006 postcondition 2 / EC-002: no health check run yet
        let payload = serde_json::json!({
            "status": "unknown",
            "message": "Run check_sensor_health to populate this resource."
        });
        let text = serde_json::to_string(&payload)
            .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;
        return Ok(ReadResourceResult::new(vec![ResourceContents::text(
            text,
            URI_SENSORS_HEALTH,
        )]));
    }

    // Collect all cached entries
    let all_entries = context.health_cache.get_all();
    let stale = all_entries.iter().any(|e| e.is_stale());

    // Group by client
    let mut clients_map: std::collections::BTreeMap<
        String,
        Vec<&crate::context::CachedHealthEntry>,
    > = std::collections::BTreeMap::new();
    for entry in &all_entries {
        clients_map
            .entry(entry.result.client_id.clone())
            .or_default()
            .push(entry);
    }

    let clients_payload: std::collections::BTreeMap<String, serde_json::Value> = clients_map
        .iter()
        .map(|(client_id, entries)| {
            // BC-2.08.006 postcondition 2: `sensors` MUST be a JSON object keyed by
            // `sensor_id` — NOT a JSON array. AI consumers look up sensors directly by ID
            // without scanning an array.
            //
            // OBS-1: propagate serialization errors instead of silently degrading to null.
            // SensorHealthResult contains only JSON-native types (String, bool, DateTime);
            // serialization realistically cannot fail, but an explicit error surface is
            // correct per the production-grade default (CLAUDE.md §Canonical Principle).
            let sensors: std::collections::BTreeMap<String, serde_json::Value> = entries
                .iter()
                .map(|e| {
                    let value = serde_json::to_value(&e.result).map_err(|err| {
                        internal_error(format!("health result serialize error: {err}"))
                    })?;
                    Ok((e.result.sensor_id.clone(), value))
                })
                .collect::<Result<std::collections::BTreeMap<_, _>, ErrorData>>()?;
            Ok((client_id.clone(), serde_json::json!({ "sensors": sensors })))
        })
        .collect::<Result<_, ErrorData>>()?;

    let payload = serde_json::json!({
        "clients": clients_payload,
        "stale": stale,
    });

    let text = serde_json::to_string(&payload)
        .map_err(|e| internal_error(format!("JSON serialize error: {e}")))?;

    Ok(ReadResourceResult::new(vec![ResourceContents::text(
        text,
        URI_SENSORS_HEALTH,
    )]))
}

// ─── VP-050: render_sensor_inventory_resource (redaction target) ─────────────

/// Render a sensor inventory resource entry for a single sensor.
///
/// This function is the proptest target for VP-050: its output MUST contain no
/// API key patterns and no full URL paths — only host+port components.
///
/// Called by `render_client_sensors_resource` for each sensor.
pub fn render_sensor_inventory_resource(
    sensor_type: &str,
    credential_ref: &str,
    endpoint_url: &str,
    sources: &[String],
) -> SensorConfigEntry {
    // VP-050: redact credential — only keep the reference name, not the value.
    // The credential_ref is always a name/path reference (never the raw secret value).
    // Strip any pattern that looks like a raw API key: UUIDs, bearer tokens, base64 strings.
    // In practice, credential_ref passed here is ALREADY a reference name (e.g., "crowdstrike_api_key")
    // not a raw value — but we sanitize defensively.
    let safe_credential_ref = redact_credential_ref(credential_ref);

    // VP-050 / BC-2.10.008 postcondition 2: strip URL to host+port only.
    // `api_base_url` MUST contain scheme+host+port only — full paths, query strings,
    // and credentials MUST NOT appear (DI-002 invariant).
    let api_base_url = strip_url_to_host_port(endpoint_url);

    SensorConfigEntry {
        sensor_type: sensor_type.to_string(),
        status: "active".to_string(),
        credential_ref: safe_credential_ref,
        sources: sources.to_vec(),
        api_base_url,
    }
}

/// Redact a credential reference that might accidentally contain a raw credential value.
///
/// VP-050: the `credential_ref` field in the sensor config resource MUST NOT contain
/// raw API key values. This function strips patterns that look like raw secrets.
fn redact_credential_ref(credential_ref: &str) -> String {
    // If the credential_ref contains a UUID pattern (8-4-4-4-12 hex), it might be a raw key.
    // Replace with a redacted marker.
    // Check for UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    if looks_like_raw_credential(credential_ref) {
        "[REDACTED]".to_string()
    } else {
        credential_ref.to_string()
    }
}

/// Returns true if the string looks like a raw API key (UUID, bearer token, or long base64).
fn looks_like_raw_credential(s: &str) -> bool {
    let s_trimmed = s.trim();

    // Check Bearer token prefix
    if s_trimmed.starts_with("Bearer ") {
        let token = s_trimmed.trim_start_matches("Bearer ").trim();
        if token.len() >= 16 {
            return true;
        }
    }

    // Check UUID-format: exactly matches xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let parts: Vec<&str> = s_trimmed.split('-').collect();
    if parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
    {
        return true;
    }

    // Check base64: 32+ chars of base64 alphabet (not containing hyphens or dots)
    // A reference name like "crowdstrike_api_key" has underscores and is < 32 base64 chars.
    // A raw base64 secret of 32+ chars is suspicious.
    if s_trimmed.len() >= 32
        && s_trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return true;
    }

    false
}

/// Strip a URL to its host+port component only (VP-050).
///
/// Input: `https://api.example.com:443/v1/events?token=secret`
/// Output: `https://api.example.com:443`
///
/// Input: `https://user:secret@host:443/path` (F-OBS-1 / AD-017)
/// Output: `https://host:443` — userinfo (`user:pass@`) is stripped.
///
/// The URL authority section may contain userinfo in the form `user:pass@host:port`.
/// Userinfo is stripped because it may contain embedded credentials that must not
/// transit into MCP resource responses forwarded to AI agent contexts (AD-017 /
/// DI-002 / BC-2.19.005 credential-redaction spirit).
fn strip_url_to_host_port(url: &str) -> String {
    // Find the scheme (e.g., "https://")
    if let Some(rest) = url.strip_prefix("https://") {
        let authority = strip_path_from_authority(strip_userinfo(rest));
        format!("https://{authority}")
    } else if let Some(rest) = url.strip_prefix("http://") {
        let authority = strip_path_from_authority(strip_userinfo(rest));
        format!("http://{authority}")
    } else {
        // No known scheme — strip path, then userinfo.
        strip_path_from_authority(strip_userinfo(url)).to_string()
    }
}

/// Strip URL userinfo (`user:pass@`) from the authority section (F-OBS-1 / AD-017).
///
/// RFC 3986 §3.2.1: userinfo is the segment before the last `@` in the authority.
/// For `user:secret@host:443/path`, returns `host:443/path`.
/// For `host:443/path` (no userinfo), returns the input unchanged.
fn strip_userinfo(authority_and_rest: &str) -> &str {
    // Split the path-part off first so we don't accidentally find `@` in a query value.
    // The authority ends at the first `/` or `?`.
    let (authority_section, rest_suffix) = if let Some(slash_pos) = authority_and_rest.find('/') {
        (
            &authority_and_rest[..slash_pos],
            &authority_and_rest[slash_pos..],
        )
    } else if let Some(q_pos) = authority_and_rest.find('?') {
        (&authority_and_rest[..q_pos], &authority_and_rest[q_pos..])
    } else {
        (authority_and_rest, "")
    };

    // Find the last `@` in the authority section — everything before it is userinfo.
    if let Some(at_pos) = authority_section.rfind('@') {
        // Reconstruct: skip the userinfo prefix and re-attach any path/query suffix.
        // `rest_suffix` already starts with `/` or `?`, so direct concatenation is safe.
        // We return a subslice of the original `authority_and_rest` that starts right
        // after the `@`. The suffix offset is at_pos + 1 within authority_section,
        // which is the same byte position in authority_and_rest.
        let _ = rest_suffix; // consumed by the pointer arithmetic below
        &authority_and_rest[at_pos + 1..]
    } else {
        authority_and_rest
    }
}

/// Strip path/query/fragment from an authority (host:port) string.
fn strip_path_from_authority(authority_and_rest: &str) -> &str {
    // authority_and_rest is "host:port/path?query" or "host:port"
    // Find the first '/' that is NOT part of the authority
    if let Some(slash_pos) = authority_and_rest.find('/') {
        &authority_and_rest[..slash_pos]
    } else if let Some(question_pos) = authority_and_rest.find('?') {
        &authority_and_rest[..question_pos]
    } else {
        authority_and_rest
    }
}

// ─── PrismQL reference content (ADR-045) ────────────────────────────────────

/// Classification of a PQL usage example (ADR-045).
///
/// Used in `REFERENCE_EXAMPLES` to tag each example for the 3-tier CI gate
/// and for the `build_reference_content` runtime injector.
///
/// Variants match the BC-2.11.022 / ADR-045 D3 mandate:
/// - `Positive` → Tier 1 (positive examples that MUST parse successfully)
/// - `NegativeE040` → Tier 2 (E-QUERY-040 dual-limit examples — gate asserts RedundantRowLimit)
/// - `NegativeOther` → Tier 3 (other error quick-reference / self-correction examples)
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExampleKind {
    /// Tier 1: positive parseable examples (filter, SELECT, pipe, stats, temporal).
    /// All entries MUST round-trip through `PrismQlParser::parse` without error.
    Positive,
    /// Tier 2: E-QUERY-040 FORBID-BOTH examples.
    /// Every entry MUST produce `PrismError::RedundantRowLimit` when executed.
    NegativeE040,
    /// Tier 3: other error quick-reference entries (E-QUERY-001, E-QUERY-038, etc.).
    /// May be comment-prefixed; these are excluded from the positive round-trip gate.
    NegativeOther,
}

/// Canonical reference examples shared by `build_reference_content` and the
/// 3-tier CI gate (ADR-045 §B).
///
/// Each tuple is `(kind, title, pql_snippet)`. The CI gate asserts that at least
/// one `Positive`, one `NegativeE040`, and one `NegativeOther` example is present.
///
/// **ADR-044 INTERVAL format:** Duration strings use the `Nh` / `Nd` unit suffix
/// (e.g., `'7d'` = 7 days, `'24h'` = 24 hours). Full English words like `'7 days'`
/// are NOT accepted by the PrismQL parser — they produce E-QUERY-001.
pub const REFERENCE_EXAMPLES: &[(ExampleKind, &str, &str)] = &[
    (
        ExampleKind::Positive,
        "filter — detections with HIGH severity",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        // The canonical FROM-target syntax is `<sensor_table>` (sensor_name + "_" + table_name).
        // Dot-notation (`sensor.table`) is illegal in FROM position — returns E-QUERY-037
        // at plan time (BC-2.11.001 / EC-11-067 / N2). ADR-046 / BC-2.11.023: filter mode
        // uses `<table_name> | <predicate>` — the table name is always underscore-qualified.
        // Generic `sensor_table` placeholder satisfies BC-2.10.014 AC-008 (no vendor names).
        "sensor_table | severity = 'High'",
    ),
    (
        ExampleKind::Positive,
        "SQL — select all detections",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        // Replaces dot-notation (crowdstrike.detections) which returns E-QUERY-037 at plan time.
        // Generic `sensor_table` satisfies BC-2.10.014 AC-008 (no hardcoded vendor prefixes).
        "SELECT * FROM sensor_table WHERE severity = 'High'",
    ),
    (
        ExampleKind::Positive,
        "pipe — filter by severity",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        "FROM sensor_table | where severity = 'High'",
    ),
    (
        ExampleKind::Positive,
        "temporal — last 7 days (SQL mode) [datetime_col is sensor-specific — use prism_describe]",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        // F-PQL2-MED-001: `timestamp` is an ILLUSTRATIVE placeholder column name. Real datetime
        // column names are sensor-specific: crowdstrike_detections→created_timestamp,
        // *_devices→last_seen/first_seen, claroty_alerts→detected_time, cyberint_alerts→created_at,
        // claroty_audit_logs→timestamp (literal). Always use the column name from prism_describe.
        "SELECT * FROM sensor_table WHERE timestamp > NOW() - INTERVAL '7d'",
    ),
    (
        ExampleKind::Positive,
        "temporal — last 24 hours (pipe mode) [datetime_col is sensor-specific — use prism_describe]",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        // F-PQL2-MED-001: `timestamp` is an ILLUSTRATIVE placeholder column name. Real datetime
        // column names are sensor-specific — always take the column name from prism_describe.
        "FROM sensor_table | where timestamp > NOW() - INTERVAL '24h'",
    ),
    (
        ExampleKind::Positive,
        "SQL→Pipe — enrich with stats",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        "SELECT * FROM sensor_table | enrich threat_score(src_ip) | limit 10",
    ),
    (
        ExampleKind::Positive,
        "pipe stats — count by severity",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        "FROM sensor_table | stats count() by severity",
    ),
    (
        ExampleKind::Positive,
        "IEQ — case-insensitive equality (OCSF Title-case: HIGH/high/High all match 'High')",
        // S-PRISMQL-CASE-INSENSITIVE-001: IEQ operator — case-insensitive equality.
        // Lowered to `lower(severity) = lower('high')` in DataFusion SQL (BC-2.11.024).
        // Post-normalization: all OCSF enum values (severity, status, etc.) are Title-case at query time.
        // IEQ is case-insensitive so any input casing ('high', 'HIGH', 'High') matches 'High'.
        "sensor_table | severity IEQ 'high'",
    ),
    (
        ExampleKind::Positive,
        "INE — case-insensitive inequality (exclude any casing of 'informational')",
        // S-PRISMQL-CASE-INSENSITIVE-001: INE operator — case-insensitive inequality.
        // Lowered to `lower(severity) != lower('informational')` in DataFusion SQL (BC-2.11.024).
        "sensor_table | severity INE 'informational'",
    ),
    (
        ExampleKind::Positive,
        "IIN — case-insensitive set membership (open/OPEN/Open all match)",
        // S-PRISMQL-CASE-INSENSITIVE-001: IIN operator — case-insensitive IN.
        // Lowered to `lower(status) IN (lower('open'), lower('new'))` in DataFusion SQL (BC-2.11.024).
        "sensor_table | status IIN ('open', 'new')",
    ),
    (
        ExampleKind::NegativeE040,
        "E-QUERY-040 FORBID-BOTH — SQL LIMIT + pipe limit",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        "SELECT * FROM sensor_table LIMIT 10 | limit 5",
    ),
    // OBS-1 fix: error-taxonomy.md §E-QUERY-040 CI-gate obligation (ADR-045 D3)
    // mandates NegativeE040 examples for BOTH `| limit` AND `| tail`. The `| tail`
    // variant combines SQL LIMIT in the head with a `| tail` stage — both consume
    // the shared row-limit slot and thus violate the FORBID-BOTH invariant (ADR-043 §C).
    (
        ExampleKind::NegativeE040,
        "E-QUERY-040 FORBID-BOTH — SQL LIMIT + pipe tail",
        // CRIT-001 fix: FROM target uses generic underscore-qualified table name.
        "SELECT * FROM sensor_table LIMIT 10 | tail 5",
    ),
    (
        ExampleKind::NegativeOther,
        "E-QUERY-001 self-correction",
        "-- If you receive E-QUERY-001: check spelling and use prism_describe to list columns.",
    ),
    (
        ExampleKind::NegativeOther,
        "E-QUERY-038 column not found",
        "-- E-QUERY-038: column not found. Run prism_describe to see available columns.",
    ),
];

/// Build the `prismql://reference` resource content at runtime (ADR-045 §A).
///
/// Assembles the PQL grammar reference as a runtime Markdown document so that
/// infusion examples, sensor-specific tables, and the 3-tier example set can
/// be injected at query time rather than baked in at compile time.
///
/// # Parameters
/// - `infusion_registry`: optional live `InfusionRegistry` snapshot. When `Some`,
///   the returned content includes a section listing available infusions with their
///   field mappings. When `None`, the infusion section shows a placeholder.
///
/// # Contract (BC-2.11.022, ADR-045 §B)
/// - MUST include at least one example from each `ExampleKind` tier (Positive, NegativeE040, NegativeOther).
/// - MUST round-trip all `Positive` PQL snippets through the Chumsky parser.
/// - MUST include the infusion placeholder when `infusion_registry` is `None`.
/// - The CI gate (`crates/prism-mcp/tests/reference_content.rs`) asserts these invariants.
pub fn build_reference_content(
    infusion_registry: Option<&prism_spec_engine::InfusionRegistry>,
) -> String {
    let mut out = String::with_capacity(16 * 1024);

    // ── Header ──────────────────────────────────────────────────────────────
    out.push_str("# PrismQL Reference\n\n");

    // ── Section 1: What is PrismQL ───────────────────────────────────────────
    out.push_str("## What is PrismQL\n\n");
    out.push_str(
        "PrismQL (PQL) is the Prism query language for querying security sensor data. \
         It supports four modes:\n\n\
         | Mode | Syntax | Notes |\n\
         |------|--------|-------|\n\
         | **Filter** | `field op value` | Bare predicate; no FROM clause. |\n\
         | **SQL** | `SELECT ... FROM t WHERE ...` | Standard SQL SELECT; no pipe stages. |\n\
         | **Pipe** | `FROM t | where ... | stage ...` | Source + pipeline stages chained with `|`. |\n\
         | **SqlPipe** | `SELECT ... FROM t | stage ...` | SQL→Pipe composition; SQL header + pipe stages. |\n\n\
         All PrismQL keywords are case-insensitive. Convention: UPPER for SQL keywords, \
         lowercase for pipe stage names.\n\n",
    );

    // ── Section 2: Clause Grammar (BNF) ─────────────────────────────────────
    out.push_str("## Clause Grammar (BNF)\n\n");
    out.push_str("**SQL Mode:**\n```sql\n");
    out.push_str(
        "SELECT <columns>      -- * or comma-separated field list\n\
         FROM <table>          -- sensor_table or bare table name\n\
         [WHERE <predicate>]   -- filter expression\n\
         [GROUP BY <fields>]\n\
         [ORDER BY <field> [ASC|DESC]]\n\
         [LIMIT <n>]           -- trailing row cap; do NOT combine with pipe | limit\n",
    );
    out.push_str("```\n\n**Pipe Mode:**\n```\n");
    out.push_str(
        "FROM <table>\n\
         [| where <predicate>]\n\
         [| sort <field> [asc|desc]]\n\
         [| head <n>] [| tail <n>] [| limit <n>]  -- head/limit are equivalent\n\
         [| stats <agg> [by <field>]]\n\
         [| dedup <field>]\n\
         [| fields <field_list>]\n\
         [| enrich <fn>(<col>)]\n",
    );
    out.push_str(
        "```\n\n**SqlPipe Mode (SQL→Pipe composition):**\n\
         `SELECT ... FROM t | <stage> ...` — SQL header followed by one or more pipe stages. \
         **FORBID-BOTH (E-QUERY-040):** You cannot use both SQL `LIMIT` and `| limit` in the \
         same query. Use one or the other.\n\n",
    );

    // ── Section 3: Operators and Types ──────────────────────────────────────
    out.push_str("## Operators and Types\n\n");
    out.push_str(
        "| Operator | Description | Example |\n\
         |----------|-------------|--------|\n\
         | `=` | Equality | `severity = 'High'` |\n\
         | `!=` | Inequality | `status != 'closed'` |\n\
         | `>`, `>=`, `<`, `<=` | Comparison | `risk_score > 50` |\n\
         | `IN` | Set membership | `status IN ('open', 'new')` |\n\
         | `BETWEEN` | Range (inclusive) | `score BETWEEN 50 AND 90` |\n\
         | `CONTAINS` | Substring match (case-sensitive) | `hostname CONTAINS 'prod'` |\n\
         | `ICONTAINS` | Substring match (case-insensitive) | `hostname ICONTAINS 'prod'` |\n\
         | `IEQ` | Case-insensitive equality (OCSF Title-case) | `severity IEQ 'high'` |\n\
         | `INE` | Case-insensitive inequality | `severity INE 'informational'` |\n\
         | `IIN` | Case-insensitive set membership | `status IIN ('open', 'new')` |\n\
         | `=~` / `MATCHES` | Regex match | `hostname =~ '^web-'` |\n\
         | `IN CIDR` | CIDR range check | `src_ip IN CIDR '10.0.0.0/8'` |\n\
         | `HAS` | Field exists and is non-null | `HAS extra_data` |\n\
         | `MISSING` | Field is absent or null | `MISSING assigned_to` |\n\
         | `IS NULL` / `IS NOT NULL` | Null check | `resolved_at IS NULL` |\n\
         | `AND`, `OR`, `NOT` | Logical combinators | `severity = 'High' AND NOT MISSING src_ip` |\n\n\
         **OCSF Title-case and case-insensitive operators:** \
         OCSF enum labels use Title-case (e.g. `severity_id` → `High`, `Critical`). \
         Vendor sensors may emit different casing (`HIGH`, `CRITICAL`, `high`). \
         Use `IEQ`, `INE`, and `IIN` to match regardless of casing — these operators \
         lower both sides before comparison via `lower()` in DataFusion SQL. \
         The adapter boundary normalizes to OCSF canonical casing at ingest time, \
         but IEQ/IIN/INE are available for defensive querying when normalization is uncertain.\n\n\
         **Null semantics for JSON-list fields:** \
         `IS NOT NULL` on a JSON-list field returns `true` if the field is present and non-null \
         (empty list `[]` is NOT null; `null` value is null).\n\n\
         **Aggregate functions** (for `| stats` and SQL `SELECT`):\n\n\
         `count()`, `sum(field)`, `avg(field)`, `min(field)`, `max(field)`, \
         `percentile(field, p)`, `distinct_count(field)`.\n\n\
         **Virtual fields** injected into every result:\n\
         `_sensor`, `_client`, `_source_table`, `_safety_flags`.\n\n\
         Column names come verbatim from `prism_describe` — do not construct dot-path names.\n\n",
    );

    // ── Section 4: Datetime Arithmetic ───────────────────────────────────────
    out.push_str("## Datetime Arithmetic\n\n");
    out.push_str(
        "PrismQL supports temporal expressions in WHERE / `| where` predicates:\n\n\
         - `NOW()` — current timestamp at query planning time\n\
         - `INTERVAL 'Nd'` — duration literal; units: `s`=seconds, `m`=minutes, `h`=hours, `d`=days\n\
         - `NOW() - INTERVAL 'Nd'` — timestamp subtraction (subtraction only; `+` not supported)\n\n\
         **Examples** (`datetime_col` is sensor-specific — use the column name returned by \
         `prism_describe`; e.g., `created_timestamp` for CrowdStrike detections, `detected_time` \
         for Claroty alerts, `last_seen` for device tables):\n\
         ```sql\n\
         -- Last 7 days\n\
         WHERE <datetime_col> > NOW() - INTERVAL '7d'\n\
         -- Last 24 hours\n\
         WHERE <datetime_col> > NOW() - INTERVAL '24h'\n\
         ```\n\n\
         **Note:** Use `'7d'` not `'7 days'` — full English words are not accepted \
         (results in a parse error).\n\n",
    );

    // ── Section 5: Error Code Quick-Reference ────────────────────────────────
    out.push_str("## Error Code Quick-Reference\n\n");
    out.push_str(
        "| Code | Cause | Self-Correction |\n\
         |------|-------|-----------------|\n\
         | **E-QUERY-001** | Parse/syntax error — invalid syntax, bad operator, unknown keyword | Check spelling; use `prism_describe` to list valid columns |\n\
         | **E-QUERY-002** | Query planning failed — type mismatch, invalid operator for column type, or plan construction failure | Use `prism_describe` to verify column types; select a compatible operator |\n\
         | **E-QUERY-003** | Depth limit exceeded | Reduce nesting depth |\n\
         | **E-QUERY-037** | Table not available — sensor not configured for this client | Run `prism_describe(client_id)` to see available tables and sensors |\n\
         | **E-QUERY-038** | Column not found | Run `prism_describe(client_id, table)` to see available columns |\n\
         | **E-QUERY-039** | Enrichment infusion not registered | Call `list_infusions` to see available enrichment functions |\n\
         | **E-QUERY-040** | FORBID-BOTH — both SQL LIMIT and pipe `| limit` in same query | Remove one of the two LIMIT clauses |\n\n",
    );

    // ── Enrichment Section ────────────────────────────────────────────────────
    out.push_str(
        "Enrichment functions are called via `| enrich <fn>(<col>)` in pipe or SqlPipe mode.\n\n",
    );

    match infusion_registry {
        Some(registry) => {
            // Build enrichment list from live registry via udf_descriptors().
            // Collect unique infusion names (multiple UDFs can belong to one infusion).
            let descriptors = registry.udf_descriptors();
            // Deduplicate by per-field UDF name (descriptor.name), NOT by infusion_id.
            // Each UDF descriptor's `.name` field is the callable per-field name
            // (e.g., `threat_score`, `cvss_base_score`) — the name an analyst uses in
            // `| enrich threat_score(col)`. The `infusion_id` (e.g., `threat_intel`, `nvd`)
            // is the registry key for the infusion, NOT a callable name.
            // AC-N1 / BC-2.11.022 v1.1 EC-11-022-006.
            let mut seen_names = std::collections::BTreeSet::new();
            let mut infusion_names: Vec<String> = Vec::new();
            for desc in &descriptors {
                if seen_names.insert(desc.name.clone()) {
                    infusion_names.push(desc.name.clone());
                }
            }
            if infusion_names.is_empty() {
                out.push_str(
                    "No enrichment functions are currently registered for your deployment.\n\n",
                );
            } else {
                out.push_str("Available enrichment functions:\n\n");
                for name in &infusion_names {
                    out.push_str(&format!("- `enrich {name}(col)`\n"));
                }
                out.push('\n');
            }
        }
        None => {
            // BC-2.11.022 invariant — placeholder text when registry is not available.
            out.push_str(
                "Call `list_infusions` to see available enrichment functions for your deployment.\n\n",
            );
        }
    }

    // ── Section 6: Query Examples (from REFERENCE_EXAMPLES shared constant) ───
    // Collect into three buckets in a SINGLE exhaustive match pass over REFERENCE_EXAMPLES.
    // See the in-loop comment for the exhaustive-match rationale.
    out.push_str("## Query Examples\n\n");

    let mut positive_entries: Vec<(&str, &str)> = Vec::new();
    let mut negative_e040_entries: Vec<(&str, &str)> = Vec::new();
    let mut negative_other_entries: Vec<(&str, &str)> = Vec::new();

    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        // LOW-002 fix: exhaustive match (no matches!() filter) — within the defining
        // crate, all ExampleKind variants are known at compile time. Adding a new
        // ExampleKind variant will produce a compile error here (non-exhaustive match),
        // forcing the developer to add the corresponding rendering section below.
        // This replaces the previous 3× matches!() pass pattern which would silently
        // omit new variants from the rendered reference output.
        match kind {
            ExampleKind::Positive => positive_entries.push((title, snippet)),
            ExampleKind::NegativeE040 => negative_e040_entries.push((title, snippet)),
            ExampleKind::NegativeOther => negative_other_entries.push((title, snippet)),
        }
    }

    // Render positive examples section.
    out.push_str("### Positive Examples\n\n");
    for (title, snippet) in &positive_entries {
        out.push_str(&format!("**{title}**\n```\n{snippet}\n```\n\n"));
    }

    // Render E-QUERY-040 negative examples section.
    out.push_str("### E-QUERY-040 Negative Examples\n\n");
    out.push_str(
        "The following queries violate FORBID-BOTH (E-QUERY-040). \
         Do NOT use both SQL `LIMIT` and pipe `| limit` in the same query.\n\n",
    );
    for (title, snippet) in &negative_e040_entries {
        out.push_str(&format!("**{title}**\n```\n{snippet}\n```\n\n"));
    }

    // Render NegativeOther / error self-correction examples.
    out.push_str("### Error Self-Correction\n\n");
    for (title, snippet) in &negative_other_entries {
        out.push_str(&format!("**{title}**\n```\n{snippet}\n```\n\n"));
    }

    // ── Section 7: Self-Correction Workflow ──────────────────────────────────
    out.push_str("## Self-Correction Workflow\n\n");
    out.push_str(
        "When a query returns an error, follow these steps:\n\n\
         1. **E-QUERY-001 (parse error):** Check syntax against the BNF above. \
            Common causes: missing quotes, wrong operator, unsupported interval format.\n\
         2. **E-QUERY-037 (table not available):** Run `prism_describe(client_id)` to see \
            registered tables. The error message includes available alternatives.\n\
         3. **E-QUERY-038 (column not found):** Run `prism_describe(client_id, table)` to \
            see available columns. The error message includes a did-you-mean suggestion.\n\
         4. **E-QUERY-040 (FORBID-BOTH):** Remove either the SQL `LIMIT` clause or the \
            pipe `| limit` stage — not both.\n\
         5. **E-QUERY-002 (plan failed / type mismatch):** The operator may be incompatible \
            with the column type. Check the column type via `prism_describe` and select a \
            compatible operator.\n\n\
         Always call `prism_describe` before writing queries against unfamiliar tables.\n\n",
    );

    out
}

// ─── Hot-reload notification dispatch (AC-9) ────────────────────────────────

/// Dispatch MCP `notifications/resources/list_changed` and
/// `notifications/tools/list_changed` when the table set changes on hot-reload.
///
/// Uses set-comparison: fires notifications only when the set of registered
/// table names changes (tables added or removed). If the table set is unchanged
/// (e.g., a spec attribute was updated but no tables added/removed), no
/// notifications are sent (BC-2.16.007).
///
/// `old_tables` and `new_tables` are `Vec<String>` from `TableRegistry::registered_tables()`.
/// `peer` is the `Peer<RoleServer>` from the `reload_config` tool's `RequestContext`.
pub async fn dispatch_hot_reload_notifications(
    old_tables: Vec<String>,
    new_tables: Vec<String>,
    peer: &rmcp::service::Peer<rmcp::RoleServer>,
) -> Result<(), ErrorData> {
    let old_set: BTreeSet<String> = old_tables.into_iter().collect();
    let new_set: BTreeSet<String> = new_tables.into_iter().collect();

    // Only dispatch notifications if the table set actually changed.
    if old_set == new_set {
        return Ok(());
    }

    // Dispatch both notifications — AC-9 requires BOTH on any table-set change.
    peer.notify_resource_list_changed()
        .await
        .map_err(|e| internal_error(format!("Failed to send resource list changed: {e}")))?;

    peer.notify_tool_list_changed()
        .await
        .map_err(|e| internal_error(format!("Failed to send tool list changed: {e}")))?;

    Ok(())
}
