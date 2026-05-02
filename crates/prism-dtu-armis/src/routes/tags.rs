//! Stateful device tag write route handlers.
//!
//! Endpoints:
//! - `POST /api/v1/devices/{device_id}/tags/` — add a tag to a device
//! - `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — remove a tag from a device
//!
//! Auth: requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}`.
//!
//! `X-Org-Id` uses a **dual-mode** policy keyed on `instance_org_id`:
//!
//! ## Default-instance clones (`instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID`)
//! Use **validate-on-presence**:
//! - Header absent → guard skipped → request proceeds (backward compat).
//! - Header present with matching UUID → 201/200.
//! - Header present with mismatch or non-UUID → 401.
//!
//! ## Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`)
//! Use **auth model A** (same as Claroty/CrowdStrike):
//! - Header absent → 401.
//! - Header present with matching UUID → 201/200.
//! - Header present with mismatch → 401.
//!
//! (CR-017 / M-50-001; BC-3.5.002 precondition 3; BC-3.2.001 precondition 4)
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

use crate::routes::devices::validate_org_id;
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

    // CR-017 / M-50-001: dual-mode X-Org-Id policy.
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat),
    //   present header with mismatch → 401.
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
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

    // CR-017 / M-50-001: dual-mode X-Org-Id policy (see module doc).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
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
