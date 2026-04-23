//! Route handlers for the device tag write endpoints.
//!
//! `POST /api/v1/devices/{device_id}/tags/` — add a tag to a device.
//! `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — remove a tag from a device.
//!
//! Both endpoints mutate `ClarotyState::tag_store`. Tag state persists across
//! requests until `reset()` is called (AC-3, AC-4, AC-8).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};

use crate::routes::devices::check_bearer_auth;
use crate::state::ClarotyState;
use crate::types::AddTagBody;

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Inserts `tag_key` into `tag_store[device_id]`.
/// Response: HTTP 201 `{"device_id": "...", "tag_key": "...", "status": "added"}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn add_tag(
    State(state): State<Arc<ClarotyState>>,
    Path(device_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<AddTagBody>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    state.add_tag(&device_id, &body.tag_key);

    (
        StatusCode::CREATED,
        Json(json!({
            "device_id": device_id,
            "tag_key": body.tag_key,
            "status": "added"
        })),
    )
}

/// `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`
///
/// Removes `tag_key` from `tag_store[device_id]`.
/// Response:
/// - HTTP 200 `{"status": "removed"}` if tag existed.
/// - HTTP 404 `{"error": "tag not found"}` if tag was never added (EC-002).
///
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn remove_tag(
    State(state): State<Arc<ClarotyState>>,
    Path((device_id, tag_key)): Path<(String, String)>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    if state.remove_tag(&device_id, &tag_key) {
        (StatusCode::OK, Json(json!({"status": "removed"})))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tag not found"})),
        )
    }
}
