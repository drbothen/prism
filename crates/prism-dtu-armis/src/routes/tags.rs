//! Stateful device tag write route handlers.
//!
//! Endpoints:
//! - `POST /api/v1/devices/{device_id}/tags/` — add a tag to a device
//! - `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — remove a tag from a device
//!
//! Auth: requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}`.
//!
//! The tag store is maintained in `ArmisState::tag_store` and merged into
//! device records at query time in the devices route handler.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::state::ArmisState;
use crate::types::ArmisError;

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Add a tag to a device's tag set.
/// Response: HTTP 201 `{"device_id": "...", "tag_key": "...", "status": "added"}`
///
/// # S-3.2.02 Stub
///
/// Implementation is deferred: this handler must receive an `OrgId` from the
/// HTTP request context and thread it into `state.add_tag(org_id, device_id, tag_key)`.
/// See BC-3.2.001 postcondition 2.
pub async fn post_device_tag(
    State(_state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(_device_id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }
    todo!("S-3.2.02: extract OrgId from request context and call state.add_tag(org_id, device_id, tag_key) (BC-3.2.001)")
}

/// `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`
///
/// Remove a tag from a device's tag set.
/// Response: HTTP 200 `{"status": "removed"}` or HTTP 404 if tag not found (EC-003).
///
/// # S-3.2.02 Stub
///
/// Implementation is deferred: this handler must receive an `OrgId` from the
/// HTTP request context and thread it into `state.remove_tag(org_id, device_id, tag_key)`.
/// See BC-3.2.001 postcondition 2.
pub async fn delete_device_tag(
    State(_state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path((_device_id, _tag_key)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }
    todo!("S-3.2.02: extract OrgId from request context and call state.remove_tag(org_id, device_id, tag_key) (BC-3.2.001)")
}

fn check_bearer_auth(headers: &HeaderMap) -> Option<axum::response::Response> {
    let valid = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > "Bearer ".len())
        .unwrap_or(false);

    if valid {
        None
    } else {
        let body = ArmisError {
            error: "invalid or missing bearer token".to_owned(),
            code: 403,
        };
        Some((StatusCode::FORBIDDEN, Json(body)).into_response())
    }
}
