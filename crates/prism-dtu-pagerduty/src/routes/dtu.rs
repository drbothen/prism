//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real PagerDuty Events API v2 — they exist
//! solely to support integration test assertions and harness control:
//!
//! - `GET /dtu/incidents` — return all incidents currently in the registry
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, auth_mode, etc.)
//! - `POST /dtu/reset` — reset all mutable state (incident registry, auth mode)
//! - `GET /dtu/health` — liveness check (no state access; safe for readiness polling)
//!
//! Per ADR-002 §6: configure, reset, and health are required for every clone.
//! `GET /dtu/incidents` is a PagerDuty-specific introspection route.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::state::PagerDutyState;
use crate::types::{IncidentSummary, IncidentsResponse};

/// `GET /dtu/incidents`
///
/// Returns all incidents currently in the registry, in snapshot order.
///
/// Response: `{"incidents": [{dedup_key, status, severity, summary}, ...]}`
///
/// Integration tests assert incident lifecycle transitions and dedup idempotency
/// by checking this endpoint after each action.
pub async fn get_incidents(State(state): State<Arc<PagerDutyState>>) -> impl IntoResponse {
    let incidents = state.incidents_snapshot();
    let summaries: Vec<IncidentSummary> = incidents
        .into_iter()
        .map(|r| IncidentSummary {
            dedup_key: r.dedup_key,
            status: r.status.as_str().to_string(),
            severity: r.severity,
            summary: r.summary,
        })
        .collect();
    let body = IncidentsResponse {
        incidents: summaries,
    };
    (StatusCode::OK, Json(body)).into_response()
}

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime. Unknown keys are
/// rejected per `deny_unknown_fields` on `ConfigPayload`.
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn post_configure(
    State(state): State<Arc<PagerDutyState>>,
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
/// Resets all mutable DTU state: incident registry cleared, auth mode reset,
/// failure mode reset to None.
///
/// Per ADR-002 §4: delegates to `state.reset()`.
pub async fn post_reset(State(state): State<Arc<PagerDutyState>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health`
///
/// Liveness check — returns `HTTP 200 {"status": "ok"}` with no state access.
/// Safe for test-harness readiness polling without side effects.
///
/// Per ADR-002 §6: required for every clone.
pub async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}
