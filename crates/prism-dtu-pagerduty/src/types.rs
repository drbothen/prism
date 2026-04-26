//! PagerDuty Events API v2 request and response types.
//!
//! Structs mirror the PagerDuty Events API v2 JSON schema so that Prism's
//! action delivery adapters can send and receive DTU responses identically
//! to live PagerDuty responses.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Event request types (POST /v2/enqueue)
// ---------------------------------------------------------------------------

/// Top-level request body for `POST /v2/enqueue`.
///
/// Required fields per PagerDuty Events API v2 spec:
/// - `routing_key` — integration key that identifies the service
/// - `event_action` — one of `trigger`, `acknowledge`, `resolve`
/// - `payload` — event payload object (required for `trigger`; optional for ack/resolve)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueRequest {
    /// Integration / routing key for the service.
    pub routing_key: Option<String>,
    /// Event action: `"trigger"`, `"acknowledge"`, or `"resolve"`.
    pub event_action: Option<String>,
    /// Deduplication key. If absent, the DTU generates a UUID.
    pub dedup_key: Option<String>,
    /// Event payload (object with `summary`, `severity`, `source`).
    pub payload: Option<EventPayload>,
}

/// Event payload object within an enqueue request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// Human-readable description of the event (required for trigger).
    pub summary: Option<String>,
    /// Severity level: `"critical"`, `"error"`, `"warning"`, or `"info"` (case-sensitive).
    pub severity: Option<String>,
    /// Component or system that generated the event.
    pub source: Option<String>,
}

// ---------------------------------------------------------------------------
// Event response types
// ---------------------------------------------------------------------------

/// Success response body for `POST /v2/enqueue` on trigger (HTTP 202).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResponse {
    /// Always `"success"`.
    pub status: String,
    /// Always `"Event processed"`.
    pub message: String,
    /// The dedup key assigned to (or generated for) this incident.
    pub dedup_key: String,
}

/// Success response body for acknowledge / resolve (HTTP 200).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    /// Always `"success"`.
    pub status: String,
}

/// Generic error response body returned for 4xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyError {
    /// Short error token (e.g. `"invalid severity"`, `"missing routing_key"`).
    pub status: String,
    /// Optional human-readable detail message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ---------------------------------------------------------------------------
// DTU test API types
// ---------------------------------------------------------------------------

/// Response for `GET /dtu/incidents`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentsResponse {
    pub incidents: Vec<IncidentSummary>,
}

/// A single incident summary returned by `GET /dtu/incidents`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentSummary {
    pub dedup_key: String,
    pub status: String,
    pub severity: String,
    pub summary: String,
}
