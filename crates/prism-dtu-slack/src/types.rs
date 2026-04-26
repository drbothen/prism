//! Slack Incoming Webhook API request and response types.
//!
//! Structs mirror the Slack Incoming Webhook JSON schema so that Prism's Slack
//! action-delivery adapters can serialize payloads that the DTU validates
//! identically to the live Slack API.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Webhook response types
// ---------------------------------------------------------------------------

/// Successful Slack webhook response (`HTTP 200`).
///
/// Per AC-1: `message_ts` is a stable fake string `"1234567890.123456"` — not
/// random — so regression tests can assert on the exact value (EC-004).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookOkResponse {
    pub ok: bool,
    pub message_ts: String,
}

// ---------------------------------------------------------------------------
// DTU introspection response types
// ---------------------------------------------------------------------------

/// Response body for `GET /dtu/received-payloads`.
///
/// Returns all Block Kit payloads captured since the last reset.
/// Per AC-5: ordered list, 3 deliveries before the call → 3 payloads in response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedPayloadsResponse {
    pub payloads: Vec<Value>,
}

// ---------------------------------------------------------------------------
// Configure payload type
// ---------------------------------------------------------------------------

/// Validated configuration payload for `POST /dtu/configure`.
///
/// Unknown fields are rejected to prevent silent misconfiguration.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SlackConfigPayload {
    /// `"rate_limit"`, `"internal_error"`, `"none"`.
    #[serde(default)]
    pub failure_mode: Option<String>,
    /// Companion for `"rate_limit"`: requests before triggering 429.
    #[serde(default)]
    pub after_n_requests: Option<u32>,
    /// Companion for `"rate_limit"`: seconds in `Retry-After` header.
    #[serde(default)]
    pub retry_after_secs: Option<u32>,
    /// Companion for `"internal_error"`: 1-indexed request number to fail at.
    #[serde(default)]
    pub at_request_n: Option<u32>,
    /// Shorthand: `{"rate_limit_after": N}` sets `failure_mode="rate_limit"` with `after_n_requests=N`.
    #[serde(default)]
    pub rate_limit_after: Option<u32>,
    /// Shorthand: `{"fail_with": 500}` sets `failure_mode="internal_error"`.
    #[serde(default)]
    pub fail_with: Option<u16>,
}
