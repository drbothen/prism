//! MCP resource handler functions for `PrismServer` (BC-2.10.008, BC-2.08.006).
//!
//! Implements the following `prism://` URI resources:
//! - `prism://config/clients` — all configured client IDs with sensor inventory
//! - `prism://config/clients/{client_id}/sensors` — per-client sensor configs
//! - `prism://schema/{sensor_id}/{table_name}` — OCSF schema for a sensor+table
//! - `prism://sensors/health` — cached sensor health data (BC-2.08.006)
//!
//! Resources are served by overriding `list_resources`, `list_resource_templates`,
//! and `read_resource` on `impl ServerHandler for PrismServer` in `server.rs`.
//! There is NO `#[resource_handler]` macro in rmcp 1.7 — confirmed against rmcp source.
//!
//! # Credential Redaction (VP-050)
//!
//! All resource response serialization MUST redact API keys and full URL paths.
//! Only host+port components are emitted for URL fields (VP-050, BC-2.10.008 postcondition).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use rmcp::model::{
    ErrorData, ListResourceTemplatesResult, ListResourcesResult, ReadResourceResult,
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
    /// Number of sensors configured for this client.
    pub sensor_count: usize,
    /// Sensor IDs enabled for this client.
    pub enabled_sensors: Vec<String>,
}

/// Per-sensor config entry in `prism://config/clients/{client_id}/sensors`
/// response (BC-2.10.008 postcondition 2).
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
}

/// Health result for a single sensor — stored in the health cache (BC-2.08.005, BC-2.08.006).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHealthResult {
    /// Sensor identifier.
    pub sensor_id: String,
    /// Client identifier — always present (BC-2.08.005 postcondition).
    pub client_id: String,
    /// Whether the sensor API endpoint was reachable.
    pub reachable: bool,
    /// Whether the provided credentials were accepted by the sensor.
    pub auth_valid: bool,
    /// Rate-limit information (None if not applicable or unavailable).
    pub rate_limit: Option<RateLimitInfo>,
    /// Timestamp of the last successful query to this sensor (None if never queried).
    pub last_successful_query_at: Option<DateTime<Utc>>,
    /// Sanitised error text (prompt-injection-safe), if the health check failed.
    pub error: Option<String>,
}

impl SensorHealthResult {
    /// Create a new `SensorHealthResult` with required fields.
    ///
    /// Optional fields (`rate_limit`, `last_successful_query_at`, `error`) default to `None`.
    pub fn new(sensor_id: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            client_id: client_id.into(),
            reachable: false,
            auth_valid: false,
            rate_limit: None,
            last_successful_query_at: None,
            error: None,
        }
    }

    /// Builder: set `reachable`.
    pub fn with_reachable(mut self, reachable: bool) -> Self {
        self.reachable = reachable;
        self
    }

    /// Builder: set `auth_valid`.
    pub fn with_auth_valid(mut self, auth_valid: bool) -> Self {
        self.auth_valid = auth_valid;
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
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePressure {
    /// Current number of non-expired cursors (out of 200 cap).
    pub active_cursor_count: usize,
    /// Current number of unexpired, unconsumed confirmation tokens (out of 100 cap).
    pub active_token_count: usize,
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
    /// Clients that failed health check (cross-client mode only).
    pub partial_failures: Vec<String>,
}

// ─── URI constants ─────────────────────────────────────────────────────────────

pub const URI_CONFIG_CLIENTS: &str = "prism://config/clients";
pub const URI_SENSORS_HEALTH: &str = "prism://sensors/health";
pub const URI_TEMPLATE_CLIENT_SENSORS: &str = "prism://config/clients/{client_id}/sensors";
pub const URI_TEMPLATE_SCHEMA: &str = "prism://schema/{sensor_id}/{table_name}";

// ─── list_resources implementation ──────────────────────────────────────────────

/// Build the static list of concrete (non-templated) resources.
///
/// Called from `ServerHandler::list_resources` override on `PrismServer`.
pub fn build_resource_list() -> ListResourcesResult {
    todo!()
}

/// Build the list of URI-template resources.
///
/// Called from `ServerHandler::list_resource_templates` override on `PrismServer`.
pub fn build_resource_template_list() -> ListResourceTemplatesResult {
    todo!()
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
    _uri: &str,
    _context: &PrismContext,
    _query_engine: Option<&Arc<prism_query::engine::QueryEngine>>,
    _config_manager: Option<
        &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
) -> Result<ReadResourceResult, ErrorData> {
    todo!()
}

// ─── prism://config/clients ───────────────────────────────────────────────────

/// Handle `prism://config/clients` resource read (BC-2.10.008 postcondition 1).
///
/// Sources sensor data from `QueryEngine::table_registry().registered_tables()`
/// (S-3.13 API), grouped by sensor_id prefix. NOT a static config snapshot.
///
/// Returns a JSON array of `ClientInventoryEntry` objects.
pub async fn render_client_list_resource(
    _config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    _query_engine: &Arc<prism_query::engine::QueryEngine>,
) -> Result<ReadResourceResult, ErrorData> {
    todo!()
}

// ─── prism://config/clients/{client_id}/sensors ───────────────────────────────

/// Handle `prism://config/clients/{client_id}/sensors` resource read
/// (BC-2.10.008 postcondition 2).
///
/// Validates `client_id` via `TenantId::new()` (same guard as tool calls).
/// Returns a 404-equivalent ErrorData on invalid or unknown `client_id`.
///
/// Returns a JSON array of `SensorConfigEntry` objects.
pub async fn render_client_sensors_resource(
    _client_id: &str,
    _config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
) -> Result<ReadResourceResult, ErrorData> {
    todo!()
}

// ─── prism://schema/{sensor_id}/{table_name} ──────────────────────────────────

/// Handle `prism://schema/{sensor_id}/{table_name}` resource read (BC-2.10.008).
///
/// Looks up the OCSF schema definition from the spec engine. Returns a
/// 404-equivalent ErrorData if the sensor+table combination is unknown.
pub async fn render_schema_resource(
    _sensor_id: &str,
    _table_name: &str,
    _config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
) -> Result<ReadResourceResult, ErrorData> {
    todo!()
}

// ─── prism://sensors/health ───────────────────────────────────────────────────

/// Handle `prism://sensors/health` resource read (BC-2.08.006).
///
/// Returns cached health data from the last `check_sensor_health` invocation.
/// If no health check has been run for any client: returns the "unknown" sentinel.
/// Stale entries (> 5 min) are returned with a `stale: true` flag (EC-003).
pub fn render_sensors_health_resource(
    _context: &PrismContext,
) -> Result<ReadResourceResult, ErrorData> {
    todo!()
}

// ─── VP-050: render_sensor_inventory_resource (redaction target) ─────────────

/// Render a sensor inventory resource entry for a single sensor.
///
/// This function is the proptest target for VP-050: its output MUST contain no
/// API key patterns and no full URL paths — only host+port components.
///
/// Called by `render_client_sensors_resource` for each sensor.
pub fn render_sensor_inventory_resource(
    _sensor_type: &str,
    _credential_ref: &str,
    _endpoint_url: &str,
    _sources: &[String],
) -> SensorConfigEntry {
    todo!()
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
    _old_tables: Vec<String>,
    _new_tables: Vec<String>,
    _peer: &rmcp::service::Peer<rmcp::RoleServer>,
) -> Result<(), ErrorData> {
    todo!()
}
