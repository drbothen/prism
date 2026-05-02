//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Threat Intel API — they exist solely to
//! support integration test assertions:
//! - `GET /dtu/health` — confirms the DTU clone is reachable and responding.
//! - `POST /dtu/reset` — reset all mutable state to initial values.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use subtle::ConstantTimeEq;

use crate::state::ThreatIntelState;

/// `GET /dtu/health`
///
/// Returns `{"status": "ok"}` — no auth required. Used by test harnesses to
/// confirm the DTU clone is reachable and responding before running assertions.
pub async fn dtu_health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `POST /dtu/reset`
///
/// Resets all mutable DTU state (request counters, rate-limit threshold, fixture registry).
///
/// # ADR-003 Amendment #5 (TD-WV0-08)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn dtu_reset(
    State(state): State<Arc<ThreatIntelState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
    let provided = headers
        .get("x-admin-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let provided_bytes = provided.as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }
    state.reset();
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}
