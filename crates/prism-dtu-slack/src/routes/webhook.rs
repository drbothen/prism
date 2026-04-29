//! Slack Incoming Webhook route handler.
//!
//! Endpoints:
//! - `POST /services/{token}` — Slack Incoming Webhook endpoint
//!
//! Behavior per story AC and dtu-assessment.md §3.5.1:
//!
//! 1. Increment `request_count`.
//! 2. Check failure mode:
//!    - `FailureMode::RateLimit { after_n_requests, retry_after_secs }`:
//!      If `request_count > after_n_requests` → HTTP 429 `"ratelimited"` with `Retry-After` header.
//!    - `FailureMode::InternalError { at_request_n }`:
//!      If `request_count == at_request_n` → HTTP 500.
//! 3. Validate payload:
//!    - Must be a JSON object; otherwise HTTP 400 `"invalid_payload"`.
//!    - All top-level keys must be in the Block Kit allow-list; unknown keys → HTTP 400 `"unknown_field"`.
//!    - Must contain `blocks` or `text`; otherwise HTTP 400 `"invalid_payload"`.
//! 4. Capture payload in `received_payloads`.
//! 5. Return HTTP 200 `{"ok": true, "message_ts": "1234567890.123456"}`.
//!
//! The `token` path parameter is accepted verbatim — not validated — matching real Slack
//! webhook behavior (the token is embedded in the webhook URL, not separately authenticated).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::Value;

#[cfg(feature = "dtu")]
use prism_core::OrgId;
use prism_dtu_common::FailureMode;

use crate::state::SlackState;
use crate::types::WebhookOkResponse;

/// Allowed top-level keys in a Block Kit payload per `fixtures/block-kit-schema.json`.
///
/// Per story Task 4 and AC-3: any top-level key outside this set → HTTP 400 `"unknown_field"`.
const ALLOWED_BLOCK_KIT_KEYS: &[&str] = &[
    "blocks",
    "text",
    "username",
    "icon_emoji",
    "icon_url",
    "attachments",
];

/// Infer the originating `OrgId` from the webhook auth context or routing metadata.
///
/// In the shared-mode Slack DTU, the `OrgId` is not embedded in the URL path — it
/// must be resolved from the webhook token, request headers, or an out-of-band
/// routing table. This stub captures the ingress tagging contract per BC-3.2.004.
///
/// # Constraints (BC-3.2.004 invariant 1)
/// - `OrgId` MUST be resolved at ingress, before `capture_payload_tagged` is called.
/// - The resolved UUID MUST NOT be placed in the Slack webhook URL path, query
///   parameters, or `X-` headers forwarded to the upstream Slack API.
///
/// # Implementation Status
/// STUB — full implementation in S-3.2.05 (Red Gate prep).
#[cfg(feature = "dtu")]
#[allow(dead_code)]
fn infer_org_id_from_webhook_token(_token: &str) -> OrgId {
    todo!(
        "S-3.2.05: resolve OrgId from webhook token or auth context for shared-mode ingress tagging"
    )
}

/// `POST /services/*token`
///
/// Slack Incoming Webhook endpoint. Real Slack URLs are multi-segment paths such as
/// `/services/T00000000/B00000000/XXXXXXXXXXXX` — the wildcard route captures all segments.
/// The token path parameter is accepted verbatim and not validated.
///
/// Returns:
/// - HTTP 429 `"ratelimited"` with `Retry-After` header when rate-limit threshold exceeded (AC-4).
/// - HTTP 400 `"invalid_payload"` when payload lacks both `blocks` and `text` (AC-2, EC-001).
/// - HTTP 400 `"unknown_field"` when payload contains unknown top-level keys (AC-3).
/// - HTTP 200 `{"ok": true, "message_ts": "1234567890.123456"}` on valid payload (AC-1).
pub async fn post_webhook(
    Path(_token): Path<String>,
    State(state): State<Arc<SlackState>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Step 1: increment request count (before any other processing).
    let count = state.increment_request_count();

    // Step 2: check failure mode.
    match state.current_failure_mode() {
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } if count > after_n_requests => {
            let retry_after_str = retry_after_secs.to_string();
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after_str.as_str())],
                "\"ratelimited\"",
            )
                .into_response();
        }
        FailureMode::InternalError { at_request_n } if count == at_request_n => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "\"internal_error\"").into_response();
        }
        _ => {}
    }

    // Step 3: parse and validate payload.
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
        }
    };

    let obj = match payload.as_object() {
        Some(o) => o,
        None => {
            return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
        }
    };

    // Check for unknown top-level fields (AC-3).
    for key in obj.keys() {
        if !ALLOWED_BLOCK_KIT_KEYS.contains(&key.as_str()) {
            return (StatusCode::BAD_REQUEST, "\"unknown_field\"").into_response();
        }
    }

    // Check for presence of `blocks` or `text` (AC-2, EC-001).
    if !obj.contains_key("blocks") && !obj.contains_key("text") {
        return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
    }

    // Step 4: capture validated payload.
    state.capture_payload(payload);

    // Step 5: return success response with stable fake message_ts (AC-1, EC-004).
    let response = WebhookOkResponse {
        ok: true,
        message_ts: "1234567890.123456".to_string(),
    };
    (StatusCode::OK, Json(response)).into_response()
}
