// Armis schema types derived from armis-sdk-go/v2 (github.com/1898andCo/armis-sdk-go v2.0.1)
// consumed by poller-coaster internal/collector.
//
// These are REFERENCE ARTIFACTS only — not compiled into any production crate.
// Consumed by S-3.7.04 fixture generator implementation.
//
// default_page_size: 100
// (source: poller-coaster internal/config/config.go — AlertLimit default, DeviceLimit default,
//  ActivityLimit default — all set to 100; this is also the Armis Search API's conservative
//  safe page size; the API supports higher values but the poller uses 100 as a conservative cap)

#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// ArmisId — polymorphic ID newtype (EC-001)
//
// The Armis API returns device/asset IDs as either JSON integers or JSON strings
// depending on the endpoint version and field context. The Go SDK exposes ID as a
// custom `DeviceID` type that implements both. We map this to a `Value` wrapper
// that accepts both via `deserialize_any` semantics.
// ---------------------------------------------------------------------------

/// Polymorphic Armis identifier — accepts JSON number or JSON string.
///
/// Rationale (EC-001): the armis-sdk-go `DeviceID` type is cast to `string` via
/// `string(result.ID)` in device_collector.go and connection_collector.go, implying
/// the underlying type is an integer at the SDK layer but string-representable.
/// Use `serde_json::Value` to absorb both until a fixture reveals which form each
/// endpoint returns.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct ArmisId(pub Value);

// ---------------------------------------------------------------------------
// SearchResult — the canonical `centrix.SearchResult` shape from armis-sdk-go/v2
//
// Fields reconstructed from collector field accesses in poller-coaster:
//   alert_collector.go:       AlertID, PolicyID, Title, LastAlertUpdateTime, Time
//   activity_collector.go:    PolicyID, ActivityUUIDs, Title, Time
//   audit_collector.go:       PolicyID, Title, Time
//   risk_factor_collector.go: PolicyID, Title, LastSeen, FirstSeen
//   connection_collector.go:  ID, Title, StartTimestamp, EndTimestamp
//   device_collector.go:      ID, Title, LastSeen, FirstSeen
//   vulnerability_collector:  ID, Title, LastDetected, FirstDetected, PublishedDate
// ---------------------------------------------------------------------------

/// Unified search result returned by the Armis AQL Search API.
///
/// The Go SDK uses a single `SearchResult` struct for all seven Armis data sources
/// (alerts, activities, audit logs, risk factors, connections, devices, vulnerabilities).
/// Fields not relevant to a given data source are zero-valued / empty.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ArmisSearchResult {
    // --- Identity ---
    /// Polymorphic device/asset ID. Cast to string in Go via `string(result.ID)`.
    /// May be absent (zero) for alert-type results where AlertID is used instead.
    #[serde(default)]
    pub id: Option<ArmisId>,

    /// Human-readable title or name. Fallback cursor ID when numeric ID is absent.
    #[serde(rename = "title", default)]
    pub title: Option<String>,

    // --- Alert-specific ---
    /// Numeric alert identifier (non-zero for alert results).
    /// Source: `result.AlertID != 0` guard in alert_collector.go.
    #[serde(rename = "alertId", default)]
    pub alert_id: Option<i64>,

    /// Policy identifier — used as cursor fallback for alerts, activities,
    /// audit logs, and risk factors.
    #[serde(rename = "policyId", default)]
    pub policy_id: Option<String>,

    /// Alert/audit/activity event time (RFC-3339 or RFC-3339Nano string).
    #[serde(rename = "time", default)]
    pub time: Option<String>,

    /// Timestamp of the last alert update — primary cursor timestamp for alerts.
    #[serde(rename = "lastAlertUpdateTime", default)]
    pub last_alert_update_time: Option<String>,

    // --- Device/Asset timestamps ---
    /// Last time the device was observed active — primary cursor for devices and
    /// risk factors.
    #[serde(rename = "lastSeen", default)]
    pub last_seen: Option<String>,

    /// First time the device was discovered — cursor fallback for devices.
    #[serde(rename = "firstSeen", default)]
    pub first_seen: Option<String>,

    // --- Activity-specific ---
    /// List of UUIDs identifying constituent activities in this result.
    #[serde(rename = "activityUUIDs", default)]
    pub activity_uuids: Option<Vec<String>>,

    // --- Connection-specific ---
    /// Connection start timestamp — primary cursor for connections.
    #[serde(rename = "startTimestamp", default)]
    pub start_timestamp: Option<String>,

    /// Connection end timestamp — cursor secondary for connections.
    #[serde(rename = "endTimestamp", default)]
    pub end_timestamp: Option<String>,

    // --- Vulnerability-specific ---
    /// Date vulnerability was last detected — primary cursor for vulnerabilities.
    #[serde(rename = "lastDetected", default)]
    pub last_detected: Option<String>,

    /// Date vulnerability was first detected.
    #[serde(rename = "firstDetected", default)]
    pub first_detected: Option<String>,

    /// CVE published date — tertiary cursor fallback for vulnerabilities.
    #[serde(rename = "publishedDate", default)]
    pub published_date: Option<String>,

    /// Additional fields not captured in the collector (extensible catch-all).
    /// Marked EC-003: Go `interface{}` → `serde_json::Value`.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

// ---------------------------------------------------------------------------
// SearchData — top-level AQL search response wrapping a Results slice
// ---------------------------------------------------------------------------

/// Top-level Armis AQL search response.
///
/// Source: `data.Results` is the field accessed throughout all collectors.
/// `client.GetSearch()` returns `centrix.SearchData`.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchData {
    /// Array of matching search result records.
    #[serde(rename = "results", default)]
    pub results: Vec<ArmisSearchResult>,

    /// Optional total count (requested via `includeTotal` flag).
    #[serde(rename = "total", default)]
    pub total: Option<i64>,

    /// Optional sample data (requested via `includeSample` flag).
    #[serde(rename = "sample", default)]
    pub sample: Option<Value>,
}

// ---------------------------------------------------------------------------
// AqlResponse<T> — generic AQL envelope used by API layer
// ---------------------------------------------------------------------------

/// Generic AQL API response envelope.
///
/// The Armis REST API wraps search results in a typed envelope.
/// `T` is typically `SearchData` or a domain-specific record type.
#[derive(Debug, Clone, Deserialize)]
pub struct AqlResponse<T> {
    /// HTTP status code echoed in the response body.
    pub status: Option<i32>,

    /// Human-readable status message.
    pub message: Option<String>,

    /// The actual response data payload.
    pub data: Option<T>,
}

// ---------------------------------------------------------------------------
// ArmisPage<T> — pagination wrapper
// ---------------------------------------------------------------------------

/// Cursor-based pagination wrapper for Armis AQL search results.
///
/// The Armis Search API is cursor-paginated; subsequent pages reference the
/// last result's timestamp+ID pair. The poller-coaster collector manages cursor
/// advancement internally; `ArmisPage` captures the response envelope shape.
#[derive(Debug, Clone, Deserialize)]
pub struct ArmisPage<T> {
    /// Paginated results for this page.
    pub results: Vec<T>,

    /// Total number of matching records (only present if `includeTotal=true`).
    pub total: Option<i64>,

    /// Cursor value for the next page — may be a timestamp string or opaque token.
    pub next: Option<String>,
}

// ---------------------------------------------------------------------------
// ArmisAsset — high-level asset (device) view
//
// Translated from armis-sdk-go/v2 consumer usage in device_collector.go.
// Maps the subset of SearchResult fields relevant to asset/device queries.
// ---------------------------------------------------------------------------

/// Armis asset (device) as returned by AQL device queries.
///
/// Source Go type: `centrix.SearchResult` (the SDK uses a unified result type
/// for all AQL queries; the `ArmisAsset` struct here captures the device-relevant
/// subset).
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ArmisAsset {
    /// Polymorphic device ID (integer or string — see `ArmisId` and EC-001).
    #[serde(default)]
    pub id: Option<ArmisId>,

    /// Device name / title.
    #[serde(rename = "name", default)]
    pub name: Option<String>,

    /// Device title (alternate name field used as cursor fallback).
    #[serde(rename = "title", default)]
    pub title: Option<String>,

    /// Device type (e.g., "IoT Device", "Workstation").
    #[serde(rename = "type", default)]
    pub device_type: Option<String>,

    /// Device operational status.
    #[serde(rename = "status", default)]
    pub status: Option<String>,

    /// ISO-8601 timestamp of last observed network activity.
    /// Primary cursor field for device polling.
    #[serde(rename = "lastSeen", default)]
    pub last_seen: Option<String>,

    /// ISO-8601 timestamp when device was first discovered.
    #[serde(rename = "firstSeen", default)]
    pub first_seen: Option<String>,

    /// Primary IPv4 or IPv6 address.
    #[serde(rename = "ipAddress", default)]
    pub ip_address: Option<String>,

    /// MAC address.
    #[serde(rename = "macAddress", default)]
    pub mac_address: Option<String>,

    /// Manufacturer / vendor name.
    #[serde(rename = "manufacturer", default)]
    pub manufacturer: Option<String>,

    /// Model number.
    #[serde(rename = "model", default)]
    pub model: Option<String>,

    /// Firmware version.
    #[serde(rename = "firmwareVersion", default)]
    pub firmware_version: Option<String>,

    /// Operating system.
    #[serde(rename = "operatingSystem", default)]
    pub operating_system: Option<String>,

    /// Risk level (integer 0-100 or null).
    #[serde(rename = "riskLevel", default)]
    pub risk_level: Option<i32>,

    /// Site name where the device is located.
    #[serde(rename = "site", default)]
    pub site: Option<String>,

    /// Network zone or segment.
    #[serde(rename = "zone", default)]
    pub zone: Option<String>,

    /// Additional fields (EC-003: Go `interface{}` → `serde_json::Value`).
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

// ---------------------------------------------------------------------------
// ArmisAlert — alert/event type
//
// Translated from armis-sdk-go/v2 consumer usage in alert_collector.go.
// ---------------------------------------------------------------------------

/// Armis alert as returned by AQL alert queries.
///
/// Source Go type: `centrix.SearchResult` accessed via alert_collector.go.
/// Field names match the AQL `alert` schema fields requested in the collector config
/// (poller-coaster/internal/config/config.go AlertFields default list).
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ArmisAlert {
    /// Numeric alert identifier. Non-zero for alerts; zero for non-alert results.
    /// Source: `result.AlertID != 0` guard in alert_collector.go.
    #[serde(rename = "alertId", default)]
    pub alert_id: Option<i64>,

    /// Alert policy identifier — cursor fallback when `alertId` is absent.
    #[serde(rename = "policyId", default)]
    pub policy_id: Option<String>,

    /// Alert title.
    #[serde(rename = "title", default)]
    pub title: Option<String>,

    /// Alert status (e.g., "UNHANDLED", "ACKNOWLEDGED", "RESOLVED").
    #[serde(rename = "status", default)]
    pub status: Option<String>,

    /// Alert severity (e.g., "HIGH", "MEDIUM", "LOW").
    #[serde(rename = "severity", default)]
    pub severity: Option<String>,

    /// Alert type (e.g., "Policy Violation", "Anomaly").
    #[serde(rename = "type", default)]
    pub alert_type: Option<String>,

    /// ISO-8601 alert event timestamp.
    #[serde(rename = "time", default)]
    pub time: Option<String>,

    /// ISO-8601 timestamp of last alert update — primary cursor timestamp.
    #[serde(rename = "lastAlertUpdateTime", default)]
    pub last_alert_update_time: Option<String>,

    /// Device ID associated with the alert.
    /// Polymorphic (integer or string — see ArmisId / EC-001).
    #[serde(rename = "deviceId", default)]
    pub device_id: Option<ArmisId>,

    /// Human-readable description.
    #[serde(rename = "description", default)]
    pub description: Option<String>,

    /// Recommended remediation steps.
    #[serde(rename = "remediation", default)]
    pub remediation: Option<String>,

    /// Additional fields (EC-003).
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}
