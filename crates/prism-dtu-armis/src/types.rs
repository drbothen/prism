//! Armis Centrix API response types.
//!
//! Structs mirror the Armis Centrix REST API JSON schema so that Prism's
//! `armis_devices`, `armis_device_activity`, and `armis_alerts` datasource
//! adapters can deserialize DTU responses identically to live Armis responses.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Device types
// ---------------------------------------------------------------------------

/// A single Armis device record (from `fixtures/devices.json`).
///
/// Key fixture requirement per S-6.10: device `"d-001"` has `last_seen: null`
/// and `first_seen: "2024-01-15T10:00:00Z"` to exercise Prism's timestamp
/// fallback path. Device `"d-002"` has both fields populated (contrast case).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_id: String,
    pub name: String,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
    pub manufacturer: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    /// Risk score 0–100.
    pub risk_score: Option<u32>,
    /// Risk factor labels, e.g. `["unpatched_cve", "open_ports"]`.
    pub risk_factors: Vec<String>,
    /// Primary timestamp. May be `null` (timestamp-fallback fixture case).
    pub last_seen: Option<String>,
    /// Secondary timestamp — used by Prism when `last_seen` is null.
    pub first_seen: Option<String>,
    pub network_id: Option<String>,
    pub site: Option<String>,
    /// Current tags on the device (merged with in-memory `tag_store` at query time).
    pub tags: Vec<String>,
}

/// Top-level device list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesResponse {
    pub data: DevicesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesData {
    pub devices: Vec<DeviceRecord>,
    pub total: u32,
    pub page: u32,
}

/// Risk score response for `GET /api/v1/devices/{device_id}/risk`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResponse {
    pub data: RiskData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskData {
    pub device_id: String,
    pub risk_score: u32,
    pub risk_factors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Activity types
// ---------------------------------------------------------------------------

/// A single device activity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub activity_id: String,
    pub device_id: String,
    pub activity_type: String,
    pub timestamp: String,
    pub details: serde_json::Value,
}

/// Top-level device activity response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityResponse {
    pub data: ActivityData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    pub activities: Vec<ActivityRecord>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Alert types
// ---------------------------------------------------------------------------

/// A single Armis alert / policy violation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub alert_id: String,
    pub name: String,
    pub severity: String,
    pub status: String,
    pub policy_name: String,
    pub device_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Top-level alert list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub data: AlertsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsData {
    pub alerts: Vec<AlertRecord>,
    pub total: u32,
}

// ---------------------------------------------------------------------------
// Tag write response types
// ---------------------------------------------------------------------------

/// Response for `POST /api/v1/devices/{device_id}/tags/` (HTTP 201).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagAddedResponse {
    pub device_id: String,
    pub tag_key: String,
    pub status: String,
}

/// Response for `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` (HTTP 200).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRemovedResponse {
    pub status: String,
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Generic error response body returned for 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmisError {
    pub error: String,
    pub code: u32,
}

// ---------------------------------------------------------------------------
// AQL log response (DTU test API)
// ---------------------------------------------------------------------------

/// Response for `GET /dtu/aql-log`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AqlLogResponse {
    pub aql_strings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tag request body
// ---------------------------------------------------------------------------

/// Request body for `POST /api/v1/devices/{device_id}/tags/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTagBody {
    pub tag_key: String,
}
