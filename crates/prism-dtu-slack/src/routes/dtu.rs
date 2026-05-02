//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Slack Incoming Webhook API — they exist solely
//! to support integration test assertions and harness control:
//!
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, rate-limit threshold)
//! - `POST /dtu/reset` — reset all mutable state (payload capture store + request counter)
//! - `GET /dtu/health` — liveness check (no state access; safe for readiness polling)
//! - `GET /dtu/received-payloads` — return all Block Kit payloads received since last reset
//!
//! Per ADR-002 §6: configure, reset, and health are required for every L2 clone.
//! `GET /dtu/received-payloads` is a Slack-specific introspection route.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::state::SlackState;
use crate::types::ReceivedPayloadsResponse;

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime.
///
/// Per ADR-003 Amendment #5: requires `X-Admin-Token` header matching `state.admin_token`.
/// Returns 401 if missing or incorrect.
pub async fn post_configure(
    State(state): State<Arc<SlackState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }
    match state.apply_config(&body) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// `POST /dtu/reset`
///
/// Resets all mutable DTU state: payload capture store cleared, request counter reset to 0,
/// failure mode reset to None.
///
/// Per AC-6 and ADR-002 §4: delegates to `state.reset()`.
///
/// # ADR-003 Amendment #5 (W3-FIX-SEC-002)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 with
/// `{"error": "missing or invalid admin token"}` if the header is absent or wrong.
pub async fn post_reset(
    State(state): State<Arc<SlackState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid admin token"})),
        )
            .into_response();
    }
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health`
///
/// Liveness check — returns `HTTP 200 {"status": "ok"}` with no state access.
/// Safe for test-harness readiness polling without side effects.
///
/// Per ADR-002 §6: required for every L2 clone.
pub async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/received-payloads`
///
/// Returns all Block Kit payloads received since the last reset, in order.
///
/// Per AC-5: response is `{"payloads": [...]}` — list of all captured Block Kit payloads.
/// Integration tests assert payload shape and ordering using this endpoint.
pub async fn get_received_payloads(State(state): State<Arc<SlackState>>) -> impl IntoResponse {
    let payloads = state.all_payloads();
    let body = ReceivedPayloadsResponse { payloads };
    (StatusCode::OK, Json(body)).into_response()
}
