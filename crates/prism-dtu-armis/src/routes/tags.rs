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

use crate::state::{ArmisState, DTU_ROUTE_ORG_ID};
use crate::types::ArmisError;

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Add a tag to a device's tag set.
/// Response: HTTP 201 `{"device_id": "...", "tag_key": "...", "status": "added"}`
///
/// OrgId: DTU clone is a single-tenant HTTP server per test instance; all route
/// calls use `DTU_ROUTE_ORG_ID` (BC-3.2.001 postcondition 2).
pub async fn post_device_tag(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    let tag_key = body
        .get("tag_key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    state.add_tag(DTU_ROUTE_ORG_ID, &device_id, &tag_key);

    let resp_body = serde_json::json!({
        "device_id": device_id,
        "tag_key": tag_key,
        "status": "added"
    });
    (StatusCode::CREATED, Json(resp_body)).into_response()
}

/// `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`
///
/// Remove a tag from a device's tag set.
/// Response: HTTP 200 `{"status": "removed"}` or HTTP 404 if tag not found (EC-003).
///
/// OrgId: DTU clone is a single-tenant HTTP server per test instance; all route
/// calls use `DTU_ROUTE_ORG_ID` (BC-3.2.001 postcondition 2).
pub async fn delete_device_tag(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path((device_id, tag_key)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    if state.remove_tag(DTU_ROUTE_ORG_ID, &device_id, &tag_key) {
        (
            StatusCode::OK,
            Json(serde_json::json!({"status": "removed"})),
        )
            .into_response()
    } else {
        let body = ArmisError {
            error: "tag not found".to_owned(),
            code: 404,
        };
        (StatusCode::NOT_FOUND, Json(body)).into_response()
    }
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
