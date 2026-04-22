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
use crate::types::{AddTagBody, ArmisError, TagAddedResponse, TagRemovedResponse};

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Add a tag to a device's tag set.
/// Response: HTTP 201 `{"device_id": "...", "tag_key": "...", "status": "added"}`
pub async fn post_device_tag(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Json(body): Json<AddTagBody>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    state.add_tag(&device_id, &body.tag_key);

    let resp = TagAddedResponse {
        device_id,
        tag_key: body.tag_key,
        status: "added".to_owned(),
    };

    (StatusCode::CREATED, Json(resp)).into_response()
}

/// `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`
///
/// Remove a tag from a device's tag set.
/// Response: HTTP 200 `{"status": "removed"}` or HTTP 404 if tag not found (EC-003).
pub async fn delete_device_tag(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path((device_id, tag_key)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    if state.remove_tag(&device_id, &tag_key) {
        let resp = TagRemovedResponse {
            status: "removed".to_owned(),
        };
        (StatusCode::OK, Json(resp)).into_response()
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
