//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Cyberint API — they exist solely to
//! support integration test control:
//! - `POST /dtu/configure` — runtime reconfiguration (auth_mode, rate_limit_after).
//! - `POST /dtu/reset` — reset all mutable state to initial values.
//! - `GET /dtu/health` — liveness check (no state access; safe for readiness polling).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::routes::alerts::extract_org_id;
use crate::state::CyberintState;

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime.
/// Supported fields:
/// - `"auth_mode"`: `"accept"` | `"reject"`
/// - `"rate_limit_after"`: u32
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn post_configure(
    State(state): State<Arc<CyberintState>>,
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
/// Resets mutable DTU state.  When the `X-Prism-Org-Id` header is present,
/// only that org's `alert_store` and `session_store` entries are cleared
/// (`reset_for`).  When the header is absent, all orgs are reset (`reset_all`)
/// for backward-compatibility with integration tests that predate multi-tenancy.
pub async fn post_reset(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if headers.contains_key("x-prism-org-id") {
        let org_id = extract_org_id(&headers, state.instance_org_id);
        state.reset_for(org_id);
    } else {
        state.reset();
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health`
///
/// Liveness check. Returns `{"status": "ok"}` with no state side-effects.
/// Used by test harnesses to poll until the server is ready.
pub async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}
