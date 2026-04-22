//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real NVD API — they exist solely to
//! support integration test assertions:
//! - `GET /dtu/request-count/{cve_id}` — returns how many times a CVE was fetched
//!   (used to assert Prism caches correctly: count should be 1 after two queries).
//! - `POST /dtu/configure` — runtime reconfiguration (auth_mode, failure injection).
//! - `POST /dtu/reset` — reset all mutable state to initial values.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::state::NvdState;

/// `GET /dtu/request-count/{cve_id}`
///
/// Returns `{"cve_id": "...", "count": N}` — request count for the given CVE.
/// Integration tests assert `count == 1` after two queries if Prism caches correctly.
pub async fn get_request_count(
    State(state): State<Arc<NvdState>>,
    Path(cve_id): Path<String>,
) -> impl IntoResponse {
    todo!() as (StatusCode, Json<serde_json::Value>)
}

/// `POST /dtu/configure`
///
/// Accepts a JSON body to reconfigure the DTU at runtime. Supported fields:
/// - `"auth_mode"`: `"accept"` | `"reject"`
/// - `"failure_mode"`: forwarded to `FailureLayer`
pub async fn post_configure(
    State(state): State<Arc<NvdState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    todo!() as (StatusCode, Json<serde_json::Value>)
}

/// `POST /dtu/reset`
///
/// Resets all mutable DTU state (request counters, rate-limit buckets, auth_mode).
pub async fn post_reset(State(state): State<Arc<NvdState>>) -> impl IntoResponse {
    todo!() as (StatusCode, Json<serde_json::Value>)
}
