//! Route handlers for the device inventory endpoint and DTU control endpoints.
//!
//! `POST /api/v1/devices` — device list with optional POST-body filtering,
//! `group_by` semantics, pagination, and tag state merge from `ClarotyState`.
//!
//! `POST /dtu/configure` — runtime reconfiguration (auth_mode, rate_limit_after).
//! `POST /dtu/reset` — clears tag store and counters.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::Value;

use crate::state::ClarotyState;
use crate::types::{DtuConfigureBody, GetDevicesBody};

/// `POST /api/v1/devices`
///
/// Returns device list from `fixtures/devices.json`.
/// - Validates `Authorization: Bearer {non-empty}` header; returns 401 if absent (AC-5).
/// - When `group_by` is present: returns only grouped field values (AC-2).
/// - Merges tag state from `tag_store` into response device objects (AC-3).
/// - Pagination via `page` / `page_size` (or `offset` / `limit`); empty array
///   beyond last page (EC-004).
pub async fn list_devices(
    State(_state): State<Arc<ClarotyState>>,
    _headers: HeaderMap,
    _body: Option<Json<GetDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("devices::list_devices")
}

/// `POST /dtu/configure`
///
/// Accepts `{"auth_mode": "reject"}` or `{"rate_limit_after": N}`.
/// Updates `ClarotyState::failure_mode` for subsequent requests.
pub async fn dtu_configure(
    State(_state): State<Arc<ClarotyState>>,
    Json(_body): Json<DtuConfigureBody>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("devices::dtu_configure")
}

/// `POST /dtu/reset`
///
/// Calls `state.reset()` and resets FailureLayer counters.
pub async fn dtu_reset(
    State(_state): State<Arc<ClarotyState>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("devices::dtu_reset")
}

/// Validate that the `Authorization: Bearer {token}` header is present and non-empty.
///
/// Returns `Ok(())` if valid, `Err((401, JSON body))` otherwise.
pub(crate) fn check_bearer_auth(headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && !v[7..].trim().is_empty())
        .unwrap_or(false);

    if has_bearer {
        Ok(())
    } else {
        use axum::Json as AxumJson;
        use serde_json::json;
        Err((
            StatusCode::UNAUTHORIZED,
            AxumJson(json!({"error": "missing or invalid Authorization header", "code": 401})),
        ))
    }
}
