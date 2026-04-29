// CrowdStrike schema types derived from:
//   - gofalcon v0.18.0 (github.com/crowdstrike/gofalcon)
//   - poller-cobra internal/crowdstrike/api.go + source.go
//   - crates/prism-dtu-crowdstrike/fixtures/ (static fixture JSON — authoritative
//     for empirically observed fields not in Go structs, per EC-004)
//   - crates/prism-dtu-crowdstrike/src/state.rs (stateful schema — authoritative
//     for containment_store and detection_status_store field names, per ADR-009 §1.2)
//
// These are REFERENCE ARTIFACTS only — not compiled into any production crate.
// Consumed by S-3.7.05 fixture generator implementation.
//
// default_page_size: 100
// (source: poller-cobra internal/crowdstrike/source.go — `limit` defaults to 100
//  when config.Limit <= 0; api.go FetchAlerts/FetchDetections/FetchHosts apply the
//  same `if limit <= 0 { limit = 100 }` guard)

#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// OAuth2TokenResponse — CrowdStrike OAuth2 token exchange response
//
// Source: gofalcon handles token exchange transparently (the SDK calls
// /oauth2/token internally). This struct represents the raw API response shape
// for fixture generation purposes.
// ---------------------------------------------------------------------------

/// CrowdStrike OAuth2 token exchange response (`POST /oauth2/token`).
///
/// gofalcon's SDK handles this transparently; this struct is used by S-3.7.05 to
/// produce fixture responses for the DTU auth endpoint.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OAuth2TokenResponse {
    /// Bearer access token.
    pub access_token: Option<String>,

    /// Token type (always "bearer" for CrowdStrike OAuth2).
    pub token_type: Option<String>,

    /// Token lifetime in seconds.
    pub expires_in: Option<i64>,

    /// Error code if token exchange failed.
    pub error: Option<String>,

    /// Human-readable error description.
    pub error_description: Option<String>,
}

// ---------------------------------------------------------------------------
// IdPage — Step-1 response for the 2-step IDs→detail pattern (EC-002)
//
// Source: poller-cobra/api.go FetchAlerts Step 1 — `QueryV2` returns a list of
// alert IDs (`queryResp.Payload.Resources`), then Step 2 fetches details.
// FetchDetections and FetchHosts follow the same pattern.
// ---------------------------------------------------------------------------

/// Step-1 response for the CrowdStrike 2-step query pattern.
///
/// Pattern (EC-002):
///   Step 1: `GET /queries/<resource>/v1` → `IdPage` (list of string IDs)
///   Step 2: `POST /entities/<resource>/v1` with IDs → `[Detail]` (full records)
///
/// Used by `FetchAlerts` (alerts.QueryV2), `FetchDetections`, and `FetchHosts`
/// in poller-cobra/internal/crowdstrike/api.go.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct IdPage {
    /// Array of resource IDs returned by the query step.
    /// Source: `queryResp.Payload.Resources` in api.go.
    pub resources: Option<Vec<String>>,

    /// API errors (if any).
    pub errors: Option<Vec<ApiError>>,

    /// Response metadata.
    pub meta: Option<ResponseMeta>,
}

/// CrowdStrike API error object embedded in responses.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    /// Error code integer.
    pub code: Option<i32>,

    /// Human-readable error message.
    pub message: Option<String>,

    /// Additional error context.
    pub id: Option<String>,
}

/// CrowdStrike response metadata.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ResponseMeta {
    /// Request query ID for tracing.
    pub query_time: Option<f64>,

    /// Pagination information.
    pub pagination: Option<Pagination>,

    /// Trace ID for support requests.
    pub trace_id: Option<String>,
}

/// Pagination metadata in CrowdStrike responses.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Pagination {
    /// Total number of matching records.
    pub total: Option<i64>,

    /// Number of records returned in this page.
    pub count: Option<i32>,

    /// FQL offset for the next page.
    pub offset: Option<String>,

    /// Maximum records per page for this query.
    pub limit: Option<i32>,
}

// ---------------------------------------------------------------------------
// FalconDevice — CrowdStrike host/device
//
// Source Go type: `Host` in poller-cobra/internal/crowdstrike/api.go +
// fixture JSON in crates/prism-dtu-crowdstrike/fixtures/hosts-detail.json.
//
// Authoritative fixture fields (EC-004 — empirically observed):
//   device_id, hostname, platform_name, os_version, status, containment_status,
//   last_seen, external_ip, local_ip, agent_version
// ---------------------------------------------------------------------------

/// CrowdStrike Falcon host/device (from `/devices/entities/devices/v2`).
///
/// Field names match `crates/prism-dtu-crowdstrike/src/state.rs` `containment_store`
/// key semantics: the `device_id` field is the key in `HashMap<String, ContainmentStatus>`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FalconDevice {
    /// Unique CrowdStrike device identifier (AID).
    /// Key in `containment_store: HashMap<String, ContainmentStatus>` (state.rs).
    pub device_id: Option<String>,

    /// Device hostname.
    pub hostname: Option<String>,

    /// Platform (e.g., "Linux", "Windows", "macOS").
    pub platform_name: Option<String>,

    /// Operating system version string.
    pub os_version: Option<String>,

    /// Device status (e.g., "normal", "contained").
    /// Mirrors `ContainmentStatus.status` in state.rs.
    pub status: Option<String>,

    /// Current containment status (e.g., "normal", "contained", "lift_containment_pending").
    /// Maps to `containment_store[device_id].status` in state.rs.
    pub containment_status: Option<String>,

    /// ISO-8601 timestamp of last sensor activity.
    /// EC-004: observed in fixtures/hosts-detail.json.
    pub last_seen: Option<String>,

    /// External (public) IP address.
    /// EC-004: observed in fixtures/hosts-detail.json.
    pub external_ip: Option<String>,

    /// Internal (LAN) IP address.
    /// EC-004: observed in fixtures/hosts-detail.json.
    pub local_ip: Option<String>,

    /// Falcon sensor agent version.
    /// EC-004: observed in fixtures/hosts-detail.json.
    pub agent_version: Option<String>,

    /// CrowdStrike Customer ID (CID).
    pub cid: Option<String>,

    /// Agent ID (AID) — may duplicate `device_id` in some API responses.
    pub agent_id: Option<String>,

    /// Additional properties (EC-003: Go `map[string]any` → `serde_json::Value`).
    /// Source: `Host.Raw map[string]any` in api.go.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

// ---------------------------------------------------------------------------
// FalconDetection — CrowdStrike detection/alert
//
// Source Go type: `Detection` in api.go + `Alert` in api.go (both map to
// similar shapes in gofalcon models) + fixtures/detections-detail.json.
//
// The `alertToMap()` function in api.go is the authoritative field list for
// `Alert`; `Detection` currently returns an empty stub but shares the same
// status/ID pattern.
// ---------------------------------------------------------------------------

/// CrowdStrike detection as returned by `/detects/entities/summaries/v1`.
///
/// Field names match `detection_status_store: HashMap<String, String>` in state.rs,
/// where the key is `detection_id` and the value is the `status` string.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FalconDetection {
    /// Detection unique identifier.
    /// Key in `detection_status_store: HashMap<String, String>` (state.rs).
    /// EC-004: observed as `"detection_id"` in fixtures/detections-detail.json.
    pub detection_id: Option<String>,

    /// Detection workflow status (e.g., "new", "in_progress", "true_positive", "false_positive").
    /// Value in `detection_status_store`.
    /// EC-004: observed in fixtures/detections-detail.json.
    pub status: Option<String>,

    /// Human-readable severity (e.g., "Low", "Medium", "High", "Critical").
    /// EC-004: observed in fixtures/detections-detail.json.
    pub severity: Option<String>,

    /// ISO-8601 creation timestamp.
    /// EC-004: `created_timestamp` in fixtures/detections-detail.json.
    pub created_timestamp: Option<String>,

    /// ISO-8601 last-updated timestamp.
    /// EC-004: `updated_timestamp` in fixtures/detections-detail.json.
    pub updated_timestamp: Option<String>,

    /// Associated device summary.
    pub device: Option<FalconDetectionDevice>,

    /// MITRE ATT&CK behaviors observed in this detection.
    pub behaviors: Option<Vec<FalconBehavior>>,

    // --- Fields from alertToMap() in api.go (Alert → Detection mapping) ---
    /// Composite ID (Alert.CompositeID).
    pub composite_id: Option<String>,

    /// Aggregate ID (Alert.AggregateID).
    pub aggregate_id: Option<String>,

    /// Alert confidence score (0-100).
    pub confidence: Option<i32>,

    /// Detection/alert display name.
    pub display_name: Option<String>,

    /// Detection/alert description.
    pub description: Option<String>,

    /// Source product (e.g., "epp", "idp").
    pub product: Option<String>,

    /// Platform (e.g., "Windows", "Linux").
    pub platform: Option<String>,

    /// MITRE tactic name.
    pub tactic: Option<String>,

    /// MITRE tactic ID.
    pub tactic_id: Option<String>,

    /// MITRE technique name.
    pub technique: Option<String>,

    /// MITRE technique ID.
    pub technique_id: Option<String>,

    /// Detection objective.
    pub objective: Option<String>,

    /// Falcon agent ID associated with the detection.
    pub agent_id: Option<String>,

    /// Process command line.
    pub cmdline: Option<String>,

    /// Process filename.
    pub filename: Option<String>,

    /// Process filepath.
    pub filepath: Option<String>,

    /// SHA-256 hash of associated file.
    pub sha256: Option<String>,

    /// MD5 hash of associated file.
    pub md5: Option<String>,

    /// Analyst assigned to this detection.
    pub assigned_to_name: Option<String>,

    /// UUID of assigned analyst.
    pub assigned_to_uuid: Option<String>,

    /// Resolution comment.
    pub resolution: Option<String>,

    /// Tags applied to the detection.
    pub tags: Option<Vec<String>>,

    /// Additional fields (EC-003).
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

/// Condensed device info embedded in a detection summary.
///
/// EC-004: observed in fixtures/detections-detail.json as `device` object.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FalconDetectionDevice {
    /// Device ID of the host where the detection occurred.
    pub device_id: Option<String>,

    /// Hostname of the device.
    pub hostname: Option<String>,
}

/// MITRE ATT&CK behavior associated with a detection.
///
/// EC-004: observed in fixtures/detections-detail.json as `behaviors` array.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FalconBehavior {
    /// MITRE ATT&CK tactic (e.g., "Initial Access").
    pub tactic: Option<String>,

    /// MITRE ATT&CK technique ID (e.g., "T1059").
    pub technique: Option<String>,
}

// ---------------------------------------------------------------------------
// ContainmentResponse — CrowdStrike device containment action response
//
// Source: state.rs `containment_store` + DTU routes for POST /devices/entities/devices-actions/v2
// The DTU stores ContainmentStatus { status, updated_at } keyed by device_id.
// ---------------------------------------------------------------------------

/// Response from a CrowdStrike device containment or lift-containment action.
///
/// Maps to state.rs `ContainmentStatus` which is stored in
/// `containment_store: Mutex<HashMap<String, ContainmentStatus>>`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ContainmentResponse {
    /// Array of resources affected by the action.
    pub resources: Option<Vec<ContainedDevice>>,

    /// Errors from the containment request.
    pub errors: Option<Vec<ApiError>>,

    /// Response metadata.
    pub meta: Option<ResponseMeta>,
}

/// Individual device record in a containment response.
///
/// Field names align with `state.rs ContainmentStatus`:
/// - `status` → `ContainmentStatus.status`
/// - device_id is the key in the HashMap
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ContainedDevice {
    /// Device ID (key in `containment_store`).
    pub device_id: Option<String>,

    /// Containment status applied (e.g., "contained", "normal").
    /// Stored as `ContainmentStatus.status`.
    pub status: Option<String>,

    /// ISO-8601 timestamp of the status change.
    /// Stored as `ContainmentStatus.updated_at`.
    pub updated_at: Option<String>,
}
