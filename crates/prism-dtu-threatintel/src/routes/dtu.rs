//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Threat Intel API — they exist solely to
//! support integration test assertions:
//! - `GET /dtu/health` — confirms the DTU clone is reachable and responding.
//! - `POST /dtu/reset` — reset all mutable state to initial values.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

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
/// No auth required.
pub async fn dtu_reset(State(state): State<Arc<ThreatIntelState>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}
