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
//! - `prismql://reference` — static PQL grammar reference (build-time embedded)
//!
//! Resources are served by overriding `list_resources`, `list_resource_templates`,
//! and `read_resource` on `impl ServerHandler for PrismServer` in `server.rs`.
//! There is NO `#[resource_handler]` macro in rmcp 1.7 — confirmed against rmcp source.
//!
//! # Credential Redaction (VP-050)
//!
//! All resource response serialization MUST redact API keys and full URL paths.
//! Only host+port components are emitted for URL fields (VP-050, BC-2.10.008 postcondition).

/// `prismql://schema/{client_id}` resource template and `prismql://reference` static resource.
///
/// Stub module for S-DEMO-PRISMQL-ONBOARDING-001-A (BC-2.10.013, BC-2.10.014).
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
    /// Human-readable display name for the client (BC-2.10.008 v1.11).
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
/// BC-2.10.008 v1.8: `api_base_url` MUST be present and contain only the
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
/// BC-2.08.005 v1.5 two-phase probe model:
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
}

impl SensorHealthResult {
    /// Create a new `SensorHealthResult` in spec-only scope (BC-2.08.005 v1.5 S-5.03 contract).
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
        }
    }

    /// Builder: set `reachable` to `Some(bool)` (S-5.04 live-probe use only).
    ///
    /// NOTE: Do NOT call this in S-5.03 scope — `reachable` must remain `None` for
    /// spec-only responses (BC-2.08.005 v1.5 postcondition, hardcoded-true prohibition).
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

/// Resource pressure section in `check_sensor_health` response (BC-2.08.005 postcondition).
///
/// BC-2.08.005 v1.6 two-phase resource_pressure behavior (RECONCILIATION-3 anchor):
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

/// Top-level structured content shape for `check_sensor_health` (BC-2.08.005).
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
}

impl SensorHealthStructuredContent {
    /// Construct a `SensorHealthStructuredContent` with the given sensors and summary.
    ///
    /// `trust_level` is always `"internal"` — it is set unconditionally here per
    /// BC-2.08.005 postcondition 7 (health data is Prism-generated, not sensor-sourced).
    pub fn new(
        sensors: Vec<SensorHealthResult>,
        resource_pressure: ResourcePressure,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            sensors,
            resource_pressure,
            trust_level: "internal".to_string(),
            summary: summary.into(),
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
/// `render_pql_reference_resource` in `resources/schema.rs`.
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
        // L3: PQL grammar reference static resource (BC-2.10.014 — S-DEMO-PRISMQL-ONBOARDING-001-A).
        // Content embedded via include_str! in resources/schema.rs::PQL_REFERENCE_CONTENT.
        // No subscribe/listChanged (static content — BC-2.10.014).
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

    // Exact match: prismql://reference (BC-2.10.014 — static PQL grammar reference)
    if uri == schema::URI_PQL_REFERENCE {
        return schema::render_pql_reference_resource();
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
/// # Per-org scoping (IMP-8 / BC-2.10.008 v1.9)
///
/// When `org_registry` and `resolved_spec_map` are both wired (production multi-tenant
/// mode), this function enumerates all registered org slugs via `org_registry.list_slugs()`
/// and for each slug counts sensor entries from `resolved_spec_map`.  An org with zero
/// overlay entries is listed with `sensor_count=0` and `enabled_sensors=[]`
/// (BC-2.10.008 v1.9 Option B semantics: overlay = provisioned, not "customize a global
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
            // ── Per-org path (IMP-8 / BC-2.10.008 v1.9) ──────────────────────────
            // Enumerate all registered org slugs. For each org, collect the sensor IDs
            // that have an overlay entry in resolved_spec_map.
            //
            // BC-2.10.008 v1.11: source display_name from org_display_names in the snapshot.
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
                    // BC-2.10.008 v1.11: look up display_name from org_display_names.
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
/// # Per-org scoping (IMP-8 / BC-2.10.008 v1.8)
///
/// When `resolved_spec_map` is wired (production multi-tenant mode):
/// - Filters `resolved_spec_map` by `OrgSlug == client_id` to return only that
///   org's provisioned sensors.
/// - When `org_registry` is also wired, validates that `client_id` is a known org
///   and returns a 404 error for unregistered orgs (BC-2.10.008 error case).
/// - An org registered in `OrgRegistry` with zero overlay entries returns an empty
///   array (EC-10-017 / BC-2.10.008 v1.9 Option B semantics).
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
        // ── Per-org path (IMP-8 / DI-008 / BC-2.10.008 v1.8+v1.9) ───────────────
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
            // BC-2.08.006 v1.5 postcondition 2: `sensors` MUST be a JSON object keyed by
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
