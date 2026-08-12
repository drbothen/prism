//! Claroty xDome API types — translated from
//! `.references/mcp-claroty-xdome/src/types/claroty.ts`.
//!
//! These are used by route handlers for request deserialization and response
//! serialization. All field names match the canonical TypeScript interface
//! definitions exactly.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared filter / sort types
// ---------------------------------------------------------------------------

/// Arbitrary filter object passed in POST bodies (Claroty API is permissive —
/// unrecognized fields are ignored per EC-001).
pub type ApiQueryFilter = HashMap<String, serde_json::Value>;

/// Sort clause used in POST body parameters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiSortClause {
    pub field: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

// ---------------------------------------------------------------------------
// Device types
// ---------------------------------------------------------------------------

/// A single Claroty xDome device object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyDevice {
    pub asset_id: String,
    pub assignees: Vec<String>,
    pub device_category: String,
    pub device_subcategory: String,
    pub device_type: String,
    pub device_type_family: String,
    pub ip_list: Vec<String>,
    pub labels: Vec<String>,
    pub mac_list: Vec<String>,
    pub model: String,
    pub network_list: Vec<String>,
    pub os_category: String,
    pub retired: bool,
    pub risk_score: String,
    pub uid: String,
    pub vlan_list: Vec<u32>,
    /// Tag keys assigned via the write path; merged from `ClarotyState::tag_store`.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// POST body for `/api/v1/devices`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetDevicesBody {
    #[serde(default)]
    pub fields: Vec<String>,
    pub filter_by: Option<ApiQueryFilter>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
    /// When present, return only grouped field values (not full device objects).
    pub group_by: Option<String>,
    // Legacy pagination params used in some callers.
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for `/api/v1/devices` (non-group_by mode).
#[derive(Debug, Clone, Serialize)]
pub struct GetDevicesResponse {
    pub devices: Vec<ClarotyDevice>,
    pub total: u32,
    pub page: u32,
}

/// Response for `/api/v1/devices` (group_by mode).
#[derive(Debug, Clone, Serialize)]
pub struct GroupByResponse {
    pub groups: Vec<serde_json::Value>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Alert types
// ---------------------------------------------------------------------------

/// A single Claroty xDome alert object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyAlert {
    pub alert_type_name: String,
    pub category: String,
    pub description: String,
    pub detected_time: String,
    pub devices_count: u32,
    pub id: u32,
    pub iot_devices_count: u32,
    pub it_devices_count: u32,
    pub medical_devices_count: u32,
    pub mitre_technique_enterprise_ids: Vec<String>,
    pub mitre_technique_enterprise_names: Vec<String>,
    pub mitre_technique_ics_ids: Vec<String>,
    pub mitre_technique_ics_names: Vec<String>,
    pub status: String,
    pub unresolved_devices_count: u32,
    pub updated_time: String,
}

/// POST body for `/api/v1/alerts`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetAlertsBody {
    #[serde(default)]
    pub fields: Vec<String>,
    pub filter_by: Option<ApiQueryFilter>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for `/api/v1/alerts`.
#[derive(Debug, Clone, Serialize)]
pub struct GetAlertsResponse {
    pub alerts: Vec<ClarotyAlert>,
    pub total: u32,
    pub page: u32,
}

/// An alerted device (device + is_resolved field).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyAlertedDevice {
    #[serde(flatten)]
    pub device: ClarotyDevice,
    pub is_resolved: bool,
}

/// POST body for `/api/v1/alerts/{alert_id}/devices`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetAlertedDevicesBody {
    pub filter_by: Option<ApiQueryFilter>,
    #[serde(default)]
    pub fields: Vec<String>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
}

/// Response for `/api/v1/alerts/{alert_id}/devices`.
#[derive(Debug, Clone, Serialize)]
pub struct GetAlertedDevicesResponse {
    pub devices: Vec<ClarotyAlertedDevice>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Vulnerability types
// ---------------------------------------------------------------------------

/// A single Claroty xDome vulnerability object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyVulnerability {
    pub id: String,
    pub name: String,
    pub vulnerability_type: String,
    pub cve_ids: Vec<String>,
    pub cvss_v2_score: Option<f64>,
    pub cvss_v3_score: Option<f64>,
    pub cvss_v3_vector_string: Option<String>,
    pub description: Option<String>,
    pub affected_products: Option<String>,
    pub recommendations: Option<String>,
    pub is_known_exploited: Option<bool>,
    pub affected_devices_count: Option<u32>,
    pub published_date: Option<String>,
    pub adjusted_vulnerability_score: Option<f64>,
    pub adjusted_vulnerability_score_level: Option<String>,
    pub exploits_count: Option<u32>,
    pub vulnerability_labels: Option<Vec<String>>,
    pub epss_score: Option<f64>,
}

/// POST body for `/api/v1/vulnerabilities`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetVulnerabilitiesBody {
    #[serde(default)]
    pub fields: Vec<String>,
    pub filter_by: Option<ApiQueryFilter>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for `/api/v1/vulnerabilities`.
#[derive(Debug, Clone, Serialize)]
pub struct GetVulnerabilitiesResponse {
    pub vulnerabilities: Vec<ClarotyVulnerability>,
    pub total: u32,
    pub page: u32,
}

/// A vulnerable device (device + vulnerability-specific fields).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyVulnerableDevice {
    #[serde(flatten)]
    pub device: ClarotyDevice,
    pub vulnerability_relevance: Option<String>,
    pub vulnerability_source: Option<Vec<String>>,
    pub vulnerability_last_updated: Option<String>,
    pub vulnerability_status: Option<String>,
}

/// POST body for `/api/v1/vulnerabilities/{vuln_id}/devices`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetVulnerabilityDevicesBody {
    pub filter_by: Option<ApiQueryFilter>,
    #[serde(default)]
    pub fields: Vec<String>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
}

/// Response for `/api/v1/vulnerabilities/{vuln_id}/devices`.
#[derive(Debug, Clone, Serialize)]
pub struct GetVulnerabilityDevicesResponse {
    pub devices: Vec<ClarotyVulnerableDevice>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Audit log types (Gap-CL-006 / S-DEMO-CLAROTY-AUDIT-DTU-001)
// ---------------------------------------------------------------------------

/// A single Claroty xDome audit log entry.
///
/// Field names match the real xDome API audit_log response and the `audit_logs`
/// table columns declared in `claroty.sensor.toml` — SAP-2 parity enforced:
/// id, action, user_display_name, category, timestamp, details, username, note (8 columns, 1:1).
///
/// `actor` and `resource` do NOT exist in the xDome API (LIVE-DRIFT-003) and are absent here.
///
/// Per EC-001 (permissive deserialization): unknown fields are allowed
/// in request bodies but the response struct is exact.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClarotyAuditLogEntry {
    /// String ID (column_type = "string" in claroty.sensor.toml).
    pub id: String,
    /// Action performed (column_type = "string").
    pub action: String,
    /// Display name of the user who performed the action (column_type = "string").
    pub user_display_name: String,
    /// Audit event category (column_type = "string").
    pub category: String,
    /// ISO 8601 timestamp with Z suffix (column_type = "datetime", ADR-028 §D8).
    pub timestamp: String,
    /// Details / description of what was done (column_type = "string").
    pub details: String,
    /// Login username / user identifier (column_type = "string").
    pub username: String,
    /// Optional note about the audit activity — absent from some records.
    #[serde(default)]
    pub note: Option<String>,
}

/// POST body for `POST /api/v1/audit_log/get`.
///
/// No `#[serde(deny_unknown_fields)]` — Claroty API is permissive (EC-001,
/// matches `GetAlertsBody` and `GetDevicesBody` conventions).
/// Includes `offset`/`limit` for S-DEMO-CLAROTY-PAGINATION-001 forward-compat.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetAuditLogBody {
    #[serde(default)]
    pub fields: Vec<String>,
    pub filter_by: Option<ApiQueryFilter>,
    pub sort_by: Option<Vec<ApiSortClause>>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub include_count: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for `POST /api/v1/audit_log/get`.
///
/// Envelope key `audit_log` matches `response_path = "$.audit_log"` in
/// `claroty.sensor.toml` (AC-003).
///
/// # No `page` field — by design (CR-002)
///
/// Unlike sibling response types `GetDevicesResponse`, `GetAlertsResponse`, and
/// `GetVulnerabilitiesResponse` which include a `page: u32` field, this struct
/// intentionally omits `page`. The Claroty xDome audit_log API returns no page
/// cursor in its response envelope — pagination state is not reflected back to
/// the caller. Future authors: do NOT copy-paste a `page` field here.
#[derive(Debug, Clone, Serialize)]
pub struct GetAuditLogResponse {
    pub audit_log: Vec<ClarotyAuditLogEntry>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Tag write types
// ---------------------------------------------------------------------------

/// POST body for `/api/v1/devices/{device_id}/tags/`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddTagBody {
    pub tag_key: String,
    pub tag_value: Option<String>,
}

/// Response for a successful tag add (HTTP 201).
#[derive(Debug, Clone, Serialize)]
pub struct AddTagResponse {
    pub device_id: String,
    pub tag_key: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// DTU control types
// ---------------------------------------------------------------------------

/// POST body for `/dtu/configure` (TD-WV0-04: deny_unknown_fields).
///
/// Unknown fields cause a 400 Bad Request response — prevents silent misconfiguration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DtuConfigureBody {
    pub auth_mode: Option<String>,
    pub rate_limit_after: Option<u32>,
    /// Seconds to include in `Retry-After` header when rate-limiting.
    pub retry_after_secs: Option<u32>,
    pub internal_error_at: Option<u32>,
    /// Trigger HTTP 422 on request number N (1-indexed). Maps to EC-005 / E-SENSOR-004.
    pub unprocessable_at: Option<u32>,
    pub latency_ms: Option<u64>,
}
