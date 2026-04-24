//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Armis Centrix API — they exist solely
//! to support integration test assertions and harness control:
//!
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, etc.)
//! - `POST /dtu/reset` — reset all mutable state (tag store + AQL log)
//! - `GET /dtu/health` — liveness check (no state access; safe for readiness polling)
//! - `GET /dtu/aql-log` — return all AQL strings received since last reset
//!
//! Per ADR-002 §6: the first three are required for every L2 clone.
//! `GET /dtu/aql-log` is an Armis-specific introspection route (R-DTU-002 mitigation).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::state::ArmisState;
use crate::types::AqlLogResponse;

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime. Unknown keys are
/// silently ignored per ADR-002 §5.
///
/// Per ADR-002 §7: uses `Json(body)` extractor; no manual `serde_json::to_string`.
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn post_configure(
    State(state): State<Arc<ArmisState>>,
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
/// Resets all mutable DTU state: tag store cleared, AQL log cleared.
/// Fixture registries are NOT affected.
///
/// Per ADR-002 §4: delegates to `state.reset()`.
pub async fn post_reset(State(state): State<Arc<ArmisState>>) -> impl IntoResponse {
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

/// `GET /dtu/aql-log`
///
/// Returns all AQL strings received since the last reset, in order.
///
/// Response: `{"aql_strings": ["in:type=switch", ...]}`
///
/// Integration tests assert correct AQL push-down by checking this log
/// (R-DTU-002 mitigation: the real Armis API passes AQL verbatim to its
/// query engine, so the DTU must capture it without parsing).
pub async fn get_aql_log(State(state): State<Arc<ArmisState>>) -> impl IntoResponse {
    let aql_strings = state.aql_log();
    let body = AqlLogResponse { aql_strings };
    (StatusCode::OK, Json(body)).into_response()
}
