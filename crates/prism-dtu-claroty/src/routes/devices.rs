//! Route handlers for the device inventory endpoint and DTU control endpoints.
//!
//! `POST /api/v1/devices` — device list with optional POST-body filtering,
//! `group_by` semantics, pagination, and tag state merge from `ClarotyState`.
//!
//! `POST /dtu/configure` — runtime reconfiguration (auth_mode, rate_limit_after).
//! `POST /dtu/reset` — clears tag store and counters.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use prism_core::OrgId;
use prism_dtu_common::FailureMode;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::ClarotyState;
use crate::types::{DtuConfigureBody, GetDevicesBody};

/// Load the devices fixture as a `Vec<serde_json::Value>`.
fn load_devices_fixture() -> Vec<Value> {
    // SAFETY: fixture files are bundled at build time; missing fixture is a build error, not runtime condition.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "devices")
        .expect("fixtures/devices.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    raw.as_array()
        .expect("devices fixture must be a JSON array")
        .clone()
}

/// Given the current failure mode and the 1-indexed request number, produce an
/// optional error response. Returns `None` if the request should proceed normally.
fn apply_failure_mode(mode: FailureMode, n: u32) -> Option<axum::response::Response> {
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "auth rejected by failure mode", "code": 401})),
            )
                .into_response(),
        ),
        FailureMode::InternalError { at_request_n } => {
            if n == at_request_n {
                Some(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "internal server error (injected)", "code": 500})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => {
            if n > after_n_requests {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "rate limit exceeded", "code": 429})),
                )
                    .into_response();
                resp.headers_mut().insert(
                    "retry-after",
                    // SAFETY: retry_after_secs is a u32 stringified — always a valid header value.
                    #[allow(clippy::expect_used)]
                    retry_after_secs
                        .to_string()
                        .parse()
                        .expect("retry_after_secs is a valid header value"),
                );
                Some(resp)
            } else {
                None
            }
        }
        FailureMode::NetworkTimeout { after_ms: _ } => {
            // Network timeout: hang the request. For tests this is simulated by
            // an extremely long sleep; the caller's timeout fires first.
            // We do not implement the sleep here — NetworkTimeout is not used in S-6.08 tests.
            None
        }
        FailureMode::Unprocessable { at_request_n } => {
            if n == at_request_n {
                Some(
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"error": "unprocessable entity (injected)", "code": 422})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
        FailureMode::MalformedResponse => {
            // Return a non-JSON body to exercise Prism's parse-error path (EC-006).
            Some(
                // SAFETY: static response builder with hardcoded status and header — cannot fail.
                #[allow(clippy::expect_used)]
                axum::response::Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        b"\xff\xfe{not valid json!@#$%^&*(" as &[u8],
                    ))
                    .expect("build malformed response"),
            )
        }
    }
}

/// Extract `OrgId` from the `X-Org-Id` header (UUID string).
///
/// Falls back to a stable sentinel UUID when the header is absent. The sentinel
/// preserves backward compatibility for single-org callers that do not supply an
/// `X-Org-Id` header — they all share the same implicit org bucket and continue to
/// see each other's tag state, matching pre-S-3.2.01 behaviour.
///
/// # Stub note (S-3.2.01)
///
/// Structural placeholder until auth middleware wires validated `OrgId` into
/// request extensions (S-3.2.02). Sentinel UUID must NOT be relied upon in
/// production multi-tenant deployments.
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

/// Validate the `X-Org-Id` header against `instance_org_id`.
///
/// # W3-FIX-SEC-001 (AC-001..AC-003, BC-3.5.002 precondition 3)
///
/// Returns `Ok(OrgId)` when the header is present, parseable as UUID, and matches
/// `instance_org_id` byte-for-byte.
///
/// Returns `Err((401, JSON body))` when:
/// - The header is absent (AC-003)
/// - The header value is not a valid UUID (EC-001)
/// - The parsed UUID does not match `instance_org_id` (AC-002)
///
/// The sentinel UUID (`00000000-0000-7000-8000-000000000000`) is never a valid
/// `instance_org_id`, so it always fails validation (AC-003).
#[allow(dead_code)]
pub(crate) fn validate_org_id(
    _headers: &HeaderMap,
    _instance_org_id: OrgId,
) -> Result<OrgId, (StatusCode, Json<serde_json::Value>)> {
    todo!("AC-001/AC-002/AC-003: validate X-Org-Id header against instance_org_id; return 401 on mismatch or absence")
}

/// `POST /api/v1/devices`
///
/// Returns device list from `fixtures/devices.json`.
/// - Validates `Authorization: Bearer {non-empty}` header; returns 401 if absent (AC-5).
/// - When `group_by` is present: returns only grouped field values (AC-2).
/// - Merges tag state from `tag_store` into response device objects (AC-3).
/// - Pagination via `page` / `page_size` (or `offset` / `limit`); empty array
///   beyond last page (EC-004).
pub async fn list_devices(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    body: Option<Json<GetDevicesBody>>,
) -> axum::response::Response {
    // Auth check (AC-5).
    if let Err(err) = check_bearer_auth(&headers) {
        return err.into_response();
    }

    // Artificial latency (EC-006).
    let latency = state.latency_ms.load(std::sync::atomic::Ordering::SeqCst);
    if latency > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
    }

    // Failure injection — increment counter, then check mode (AC-6, AC-7, EC-005).
    let n = state.increment_counter();
    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mode = state
        .failure_mode
        .lock()
        .expect("failure_mode poisoned")
        .clone();
    if let Some(resp) = apply_failure_mode(mode, n) {
        return resp;
    }

    let params = body.map(|Json(b)| b).unwrap_or_default();
    let mut devices = load_devices_fixture();

    // Extract org_id for scoped tag lookups (S-3.2.01 — BC-3.2.001).
    let org_id = extract_org_id(&headers);

    // Merge tag state into each device (AC-3, AC-4).
    for device in &mut devices {
        if let Some(asset_id) = device.get("asset_id").and_then(|v| v.as_str()) {
            let tags: Vec<Value> = state
                .get_tags(org_id, asset_id)
                .into_iter()
                .map(Value::String)
                .collect();
            device["tags"] = Value::Array(tags);
        }
    }

    // group_by handling (AC-2, EC-003).
    if let Some(ref group_field) = params.group_by {
        let known_fields = [
            "device_type",
            "device_category",
            "device_subcategory",
            "device_type_family",
            "os_category",
            "risk_score",
        ];

        if known_fields.contains(&group_field.as_str()) {
            // Collect distinct values for the group field.
            let mut seen = std::collections::HashMap::<String, u32>::new();
            for device in &devices {
                let val = device
                    .get(group_field.as_str())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                *seen.entry(val).or_insert(0) += 1;
            }
            let groups: Vec<Value> = seen
                .into_iter()
                .map(|(value, count)| json!({"field": group_field, "value": value, "count": count}))
                .collect();
            let total = groups.len() as u32;
            return Json(json!({"groups": groups, "total": total})).into_response();
        } else {
            // EC-003: unknown group_by field — return empty groups (no error).
            return Json(json!({"groups": [], "total": 0u32})).into_response();
        }
    }

    let total = devices.len() as u32;

    // Pagination (EC-004).
    // Support both `page`/`page_size` and `offset`/`limit` styles.
    let paged: Vec<Value> = if let (Some(page), Some(page_size)) = (params.page, params.page_size) {
        let page_size = page_size as usize;
        let page = page as usize;
        // page is 1-indexed; page=9999 → start = 9998 * page_size, way beyond fixture.
        let start = page.saturating_sub(1).saturating_mul(page_size);
        if start >= devices.len() {
            vec![]
        } else {
            devices[start..std::cmp::min(start + page_size, devices.len())].to_vec()
        }
    } else if let Some(offset) = params.offset {
        let offset = offset as usize;
        let limit = params.limit.unwrap_or(u32::MAX) as usize;
        if offset >= devices.len() {
            vec![]
        } else {
            devices[offset..std::cmp::min(offset + limit, devices.len())].to_vec()
        }
    } else {
        devices
    };

    let page_num = params.page.unwrap_or(1);
    Json(json!({"devices": paged, "total": total, "page": page_num})).into_response()
}

/// `POST /dtu/configure`
///
/// Accepts `{"auth_mode": "reject"}`, `{"rate_limit_after": N, "retry_after_secs": M}`,
/// `{"internal_error_at": N}`, `{"unprocessable_at": N}`, or `{"latency_ms": N}`.
/// Updates `ClarotyState::failure_mode` (and `latency_ms`) for subsequent requests.
///
/// Unknown fields return 400 Bad Request (TD-WV0-04: deny_unknown_fields).
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn dtu_configure(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    Json(raw): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        );
    }
    // Deserialize with deny_unknown_fields to catch typos / unknown keys (TD-WV0-04).
    let body: DtuConfigureBody = match serde_json::from_value(raw) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid /dtu/configure payload: {e}")})),
            );
        }
    };

    // Apply latency configuration (independent of failure mode).
    if let Some(latency_ms) = body.latency_ms {
        state.apply_latency(latency_ms);
    }

    // Determine the failure mode to apply (priority: unprocessable > internal_error > rate_limit > auth_mode).
    let mode = if let Some(unprocessable_at) = body.unprocessable_at {
        FailureMode::Unprocessable {
            at_request_n: unprocessable_at,
        }
    } else if let Some(internal_error_at) = body.internal_error_at {
        FailureMode::InternalError {
            at_request_n: internal_error_at,
        }
    } else if let Some(rate_limit_after) = body.rate_limit_after {
        FailureMode::RateLimit {
            after_n_requests: rate_limit_after,
            retry_after_secs: body.retry_after_secs.unwrap_or(60),
        }
    } else if body.auth_mode.as_deref() == Some("reject") {
        FailureMode::AuthReject
    } else {
        // No failure mode fields set — leave failure mode unchanged (latency-only configure).
        // If latency was the only field, preserve existing failure mode.
        if body.latency_ms.is_some() {
            return (StatusCode::OK, Json(json!({"status": "ok"})));
        }
        FailureMode::None
    };

    state.apply_config(mode);
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// `POST /dtu/reset`
///
/// Calls `state.reset()` and resets FailureLayer counters.
pub async fn dtu_reset(State(state): State<Arc<ClarotyState>>) -> (StatusCode, Json<Value>) {
    state.reset();
    (StatusCode::OK, Json(json!({"status": "reset"})))
}

/// `GET /dtu/health`
///
/// Liveness check for test-harness readiness polling. No state access.
/// Returns `HTTP 200 {"status": "ok"}` unconditionally.
pub async fn dtu_health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// `POST /dtu/reset_for/{org_id}`
///
/// Selectively evicts all tag-store entries belonging to `org_id`.
/// Entries for other orgs are preserved (BC-3.2.001 invariant 1, AC-005).
///
/// Returns:
/// - HTTP 200 `{"status": "reset_for"}` on success.
/// - HTTP 400 `{"error": "..."}` if `org_id` is not a valid UUID.
pub async fn dtu_reset_for(
    State(state): State<Arc<ClarotyState>>,
    Path(org_id_str): Path<String>,
) -> (StatusCode, Json<Value>) {
    let uuid = match Uuid::parse_str(&org_id_str) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"error": format!("invalid org_id: {org_id_str:?} is not a valid UUID")}),
                ),
            );
        }
    };
    let org_id = OrgId::from_uuid(uuid);
    state.reset_for(org_id);
    (StatusCode::OK, Json(json!({"status": "reset_for"})))
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
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid Authorization header", "code": 401})),
        ))
    }
}
