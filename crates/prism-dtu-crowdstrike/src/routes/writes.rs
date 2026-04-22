//! Write (mutation) endpoints for the CrowdStrike DTU.
//!
//! - `POST /devices/entities/devices-actions/v2` — contain / lift_containment
//! - `PATCH /detects/entities/detects/v2` — update_status / assign
//!
//! # PATCH dispatch
//!
//! `PATCH /detects/entities/detects/v2` branches by body content:
//! - If `assigned_to_uid` is present → assign path
//! - Otherwise → update_status path
//! Both paths are handled in the single `patch_detections` handler.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use serde::Deserialize;

use crate::state::{ContainmentStatus, CrowdstrikeState};

/// Query params for device actions endpoint.
#[derive(Debug, Deserialize, Default)]
pub struct DeviceActionParams {
    pub action_name: Option<String>,
}

/// Body for device actions (contain / lift_containment).
#[derive(Debug, Deserialize)]
pub struct DeviceActionBody {
    pub ids: Vec<String>,
}

/// Body for PATCH /detects/entities/detects/v2 (update_status or assign).
///
/// Both paths share the same body shape; dispatch is based on `assigned_to_uid` presence.
#[derive(Debug, Deserialize)]
pub struct PatchDetectionsBody {
    pub ids: Vec<String>,
    /// Present → assign path.
    pub assigned_to_uid: Option<String>,
    /// Present → update_status path.
    pub status: Option<String>,
}

/// Validate the `Authorization` header.
fn check_auth(headers: &HeaderMap) -> Result<(), axum::response::Response> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "errors": [{"code": 401, "message": "access denied, authorization required"}]
            })),
        )
            .into_response());
    }
    Ok(())
}

/// `POST /devices/entities/devices-actions/v2?action_name=contain` or `lift_containment`
///
/// Routes by `action_name` query param.
pub async fn device_actions(
    State(state): State<Arc<CrowdstrikeState>>,
    Query(params): Query<DeviceActionParams>,
    headers: HeaderMap,
    Json(body): Json<DeviceActionBody>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return e;
    }

    match params.action_name.as_deref() {
        Some("contain") => contain(state, body).await,
        Some("lift_containment") => lift_containment(state, body).await,
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "unknown action_name"}]
            })),
        )
            .into_response(),
    }
}

async fn contain(state: Arc<CrowdstrikeState>, body: DeviceActionBody) -> axum::response::Response {
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    let mut store = state
        .containment_store
        .lock()
        .expect("containment_store poisoned");

    let mut resources = Vec::new();

    for device_id in &body.ids {
        // EC-002: already contained → return 400.
        if let Some(existing) = store.get(device_id) {
            if existing.status == "contained" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "errors": [{"code": 400, "message": "device already contained"}]
                    })),
                )
                    .into_response();
            }
        }

        let now = chrono_now();
        store.insert(
            device_id.clone(),
            ContainmentStatus {
                status: "contained".to_owned(),
                updated_at: now.clone(),
            },
        );

        resources.push(serde_json::json!({
            "device_id": device_id,
            "containment_status": "contained"
        }));
    }

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "resources": resources })),
    )
        .into_response()
}

async fn lift_containment(
    state: Arc<CrowdstrikeState>,
    body: DeviceActionBody,
) -> axum::response::Response {
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    let mut store = state
        .containment_store
        .lock()
        .expect("containment_store poisoned");

    let mut resources = Vec::new();

    for device_id in &body.ids {
        let now = chrono_now();
        store.insert(
            device_id.clone(),
            ContainmentStatus {
                status: "normal".to_owned(),
                updated_at: now,
            },
        );

        resources.push(serde_json::json!({
            "device_id": device_id,
            "containment_status": "normal"
        }));
    }

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "resources": resources })),
    )
        .into_response()
}

/// `PATCH /detects/entities/detects/v2`
///
/// Dispatches by presence of `assigned_to_uid` in body:
/// - `assigned_to_uid` present → assign path (records assignment)
/// - Otherwise → update_status path (updates detection_status_store)
///
/// Returns HTTP 200 `{}` on success.
pub async fn patch_detections(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
    Json(body): Json<PatchDetectionsBody>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return e;
    }

    let mut detection_store = state
        .detection_status_store
        .lock()
        .expect("detection_status_store poisoned");

    if body.assigned_to_uid.is_some() {
        // Assign path: record assignment (no persistent state needed beyond acknowledging).
        // In the DTU we just track the assignment in detection_status_store as "assigned".
        for id in &body.ids {
            detection_store.insert(id.clone(), "assigned".to_owned());
        }
    } else if let Some(status) = &body.status {
        // Update status path.
        for id in &body.ids {
            detection_store.insert(id.clone(), status.clone());
        }
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

/// Get a simple timestamp string (ISO-8601 format).
/// Uses a fixed format without chrono dependency.
fn chrono_now() -> String {
    // Produce a simple timestamp. In tests this value is not checked for
    // exact content, only used for record-keeping.
    "2026-01-01T00:00:00Z".to_owned()
}
