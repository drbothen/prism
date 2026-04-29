//! Route handlers for the device tag write endpoints.
//!
//! `POST /api/v1/devices/{device_id}/tags/` — add a tag to a device.
//! `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — remove a tag from a device.
//!
//! Both endpoints mutate `ClarotyState::tag_store`. Tag state persists across
//! requests until `reset()` is called (AC-3, AC-4, AC-8).
//!
//! # S-3.2.01 — Multi-tenant stub
//!
//! Route handlers accept an `OrgId` from request extensions. The extension is
//! populated by upstream auth middleware (implementation in S-3.2.02). In this
//! stub cycle the `OrgId` is extracted from the `X-Org-Id` header if present,
//! falling back to a per-request freshly minted `OrgId` as a safe no-op
//! placeholder until the middleware layer is wired up.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use prism_core::OrgId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::routes::devices::check_bearer_auth;
use crate::state::ClarotyState;
use crate::types::AddTagBody;

/// Extract `OrgId` from the `X-Org-Id` header (UUID string).
///
/// Falls back to a stable sentinel UUID when the header is absent. The sentinel
/// preserves backward compatibility for single-org callers that do not supply an
/// `X-Org-Id` header — they all share the same implicit org bucket and continue to
/// see each other's tag state, matching pre-S-3.2.01 behaviour.
///
/// # Stub note (S-3.2.01)
///
/// This is a structural placeholder. The definitive implementation wires
/// `OrgId` from validated auth middleware extensions (planned for S-3.2.02).
/// The sentinel UUID (`00000000-0000-7000-8000-000000000000`) is a test-harness
/// compatibility shim and must NOT be relied upon in production multi-tenant deployments.
fn extract_org_id(headers: &HeaderMap) -> OrgId {
    // STUB(S-3.2.01): sentinel fallback for header-less single-org callers.
    const SENTINEL: Uuid = uuid::uuid!("00000000-0000-7000-8000-000000000000");
    headers
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or(OrgId::from_uuid(SENTINEL))
}

/// `POST /api/v1/devices/{device_id}/tags/`
///
/// Inserts `tag_key` into `tag_store[(org_id, device_id)]`.
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

    let org_id = extract_org_id(&headers);
    state.add_tag(org_id, &device_id, &body.tag_key);

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
/// Removes `tag_key` from `tag_store[(org_id, device_id)]`.
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

    let org_id = extract_org_id(&headers);
    if state.remove_tag(org_id, &device_id, &tag_key) {
        (StatusCode::OK, Json(json!({"status": "removed"})))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tag not found"})),
        )
    }
}
