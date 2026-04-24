//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real NVD API — they exist solely to
//! support integration test assertions:
#![allow(clippy::expect_used)]
//! - `GET /dtu/request-count/{cve_id}` — returns how many times a CVE was fetched
//!   (used to assert Prism caches correctly: count should be 1 after two queries).
//! - `POST /dtu/configure` — runtime reconfiguration (auth_mode, failure injection).
//! - `POST /dtu/reset` — reset all mutable state to initial values.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::state::NvdState;
use crate::types::RequestCountResponse;

/// `GET /dtu/health`
///
/// Returns `{"status": "ok"}` — no auth required. Used by test harnesses to
/// confirm the DTU clone is reachable and responding before running assertions.
pub async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/request-count/{cve_id}`
///
/// Returns `{"cve_id": "...", "count": N}` — request count for the given CVE.
/// Integration tests assert `count == 1` after two queries if Prism caches correctly.
pub async fn get_request_count(
    State(state): State<Arc<NvdState>>,
    Path(cve_id): Path<String>,
) -> impl IntoResponse {
    let normalized = cve_id.to_uppercase();
    let count = state.request_count_for(&normalized);
    let body = RequestCountResponse {
        cve_id: normalized,
        count,
    };
    (
        StatusCode::OK,
        Json(serde_json::to_value(body).expect("RequestCountResponse serialization")),
    )
        .into_response()
}

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime. Supported fields:
/// - `"auth_mode"`: `"accept"` | `"reject"`
/// - `"exhaust_authenticated_bucket"`: bool — pre-exhaust authenticated bucket
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn post_configure(
    State(state): State<Arc<NvdState>>,
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
/// Resets all mutable DTU state (request counters, rate-limit buckets, auth_mode).
pub async fn post_reset(State(state): State<Arc<NvdState>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}
