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
use serde_json::Value;

use crate::state::ClarotyState;
use crate::types::AddTagBody;

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Inserts `tag_key` into `tag_store[device_id]`.
/// Response: HTTP 201 `{"device_id": "...", "tag_key": "...", "status": "added"}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn add_tag(
    State(_state): State<Arc<ClarotyState>>,
    Path(_device_id): Path<String>,
    _headers: HeaderMap,
    Json(_body): Json<AddTagBody>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("tags::add_tag")
}

/// `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`
///
/// Removes `tag_key` from `tag_store[device_id]`.
/// Response:
/// - HTTP 200 `{"status": "removed"}` if tag existed.
/// - HTTP 404 `{"error": "tag not found"}` if tag was never added (EC-002).
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn remove_tag(
    State(_state): State<Arc<ClarotyState>>,
    Path((_device_id, _tag_key)): Path<(String, String)>,
    _headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    unimplemented!("tags::remove_tag")
}
