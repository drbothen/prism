//! Alert route handlers for the Cyberint DTU clone.
//!
//! Routes:
//! - `GET /api/v1/alerts` (or `POST /api/v1/alerts`) — alert list with cursor pagination
//! - `GET /api/v1/alerts/{alert_id}` — alert detail
//! - `PATCH /api/v1/alerts/{alert_id}/status` — acknowledge alert
//! - `POST /api/v1/alerts/{alert_id}/close` — close alert (irreversible in session)
//!
//! All routes require cookie auth — validated via `extract_session_token`.
//!
//! # Auth model deviation (CR-015)
//!
//! Cyberint intentionally does NOT use the instance-identity `X-Org-Id` guard
//! employed by Claroty, CrowdStrike, and Armis clones. Cyberint clones support
//! multiple orgs concurrently via per-session routing: each session cookie maps
//! to one org, and the `X-Prism-Org-Id` header selects the correct session
//! namespace at the handler level. An absent `X-Prism-Org-Id` header falls
//! through to the instance session (not a 401), which is intentional and required
//! for backward compatibility with legacy single-org Cyberint callers.
//! The upstream `validate_org_id` pattern is therefore incompatible with this
//! topology and has been removed from this module (BC-3.2.003; W3-FIX-CODE-004 AC-006).

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use prism_core::OrgId;
use serde::Deserialize;

use crate::state::{AuthMode, CyberintState};

/// Query parameters for the alert list endpoint.
#[derive(Debug, Deserialize, Default)]
pub struct AlertListParams {
    pub cursor: Option<String>,
}

/// Extract the session token from the `Cookie` header.
/// Returns `None` if the header is absent, empty, or no matching cookie is found.
pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix("cyberint_session=") {
            return Some(val.to_owned());
        }
    }
    None
}

/// Extract the `OrgId` for the current request from the `X-Prism-Org-Id` header.
///
/// # BC-3.2.001 / BC-3.2.003
///
/// The `X-Prism-Org-Id` header carries the canonical org identity minted by the
/// Prism query engine (ADR-008 §2.1).  If the header is absent or unparseable,
/// the `fallback` org is returned — callers should pass `state.instance_org_id`
/// to ensure legacy tests that do not send an org header still work against the
/// correct key namespace.
///
/// Production callers MUST supply a real `X-Prism-Org-Id` header.
pub fn extract_org_id(headers: &HeaderMap, fallback: OrgId) -> OrgId {
    headers
        .get("x-prism-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or(fallback)
}

/// Return HTTP 401 unauthorized response.
fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "unauthorized", "code": 401})),
    )
        .into_response()
}

/// Return HTTP 429 rate-limited response.
fn rate_limited() -> axum::response::Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({"error": "rate limit exceeded", "code": 429})),
    )
        .into_response()
}

/// Check auth and rate limits. Returns `Ok(())` if the request may proceed.
fn check_auth(
    state: &CyberintState,
    headers: &HeaderMap,
) -> Result<(), Box<axum::response::Response>> {
    // auth_mode=reject: always 401 regardless of cookie.
    if state.auth_mode() == AuthMode::Reject {
        return Err(Box::new(unauthorized()));
    }

    // Validate cookie.
    let token = extract_session_token(headers).ok_or_else(|| Box::new(unauthorized()))?;
    let org_id = extract_org_id(headers, state.instance_org_id);
    if !state.is_valid_session(org_id, &token) {
        // W3-FIX-SEC-001 (AC-002_body): when X-Prism-Org-Id header is present, the session
        // lookup failed because the caller supplied an org UUID that doesn't match any
        // registered session. Return "org_id mismatch" to distinguish this from a plain
        // missing-cookie 401 (which returns "unauthorized").
        if headers.get("x-prism-org-id").is_some() {
            return Err(Box::new(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "org_id mismatch: request does not match this clone instance"
                    })),
                )
                    .into_response(),
            ));
        }
        return Err(Box::new(unauthorized()));
    }

    // Rate limit check.
    if state.check_and_increment_rate_limit() {
        return Err(Box::new(rate_limited()));
    }

    Ok(())
}

/// `GET /api/v1/alerts` or `POST /api/v1/alerts`
///
/// Returns a paginated list of alerts. Merges current status from `alert_store`.
pub async fn get_alerts(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Query(params): Query<AlertListParams>,
) -> impl IntoResponse {
    // W3-FIX-SEC-001 (EC-001): if X-Prism-Org-Id header is present but not a valid UUID,
    // reject with 401 "org_id mismatch" (non-UUID headers cannot be routed to any org).
    if let Some(h) = headers.get("x-prism-org-id") {
        if let Ok(s) = h.to_str() {
            if uuid::Uuid::parse_str(s).is_err() {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "org_id mismatch: request does not match this clone instance"
                    })),
                )
                    .into_response();
            }
        }
    }

    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let alert_store = state.alert_store.lock().expect("alert_store poisoned");

    // Simple cursor logic: no cursor → page 1; any cursor value == "page2" → page 2.
    let (alerts_to_return, next_cursor) = if params.cursor.as_deref() == Some("page2") {
        (&state.alert_fixture_page2, serde_json::Value::Null)
    } else {
        (&state.alert_fixture, serde_json::json!("page2"))
    };

    let org_id = extract_org_id(&headers, state.instance_org_id);
    let data: Vec<serde_json::Value> = alerts_to_return
        .iter()
        .map(|a| {
            let status = alert_store
                .get(&(org_id, a.alert_id.clone()))
                .map(|s| s.status.clone())
                .unwrap_or_else(|| "open".to_owned());
            serde_json::json!({
                "alert_id": a.alert_id,
                "title": a.title,
                "severity": a.severity,
                "status": status,
                "created_at": a.created_at,
                "source": a.source,
                "type": a.alert_type,
                "affected_assets": a.affected_assets,
            })
        })
        .collect();

    drop(alert_store);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data": data,
            "next_cursor": next_cursor,
        })),
    )
        .into_response()
}

/// `GET /api/v1/alerts/{alert_id}`
///
/// Returns single alert with current status from `alert_store`.
/// Returns 404 if the alert_id is not found.
pub async fn get_alert_by_id(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    let status_record = match alert_store.get(&(org_id, alert_id.clone())) {
        Some(s) => s.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "alert not found"})),
            )
                .into_response()
        }
    };
    drop(alert_store);

    // Find the fixture for this alert.
    let alert = state
        .alert_fixture
        .iter()
        .chain(state.alert_fixture_page2.iter())
        .find(|a| a.alert_id == alert_id);

    match alert {
        Some(a) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "alert_id": a.alert_id,
                "title": a.title,
                "severity": a.severity,
                "status": status_record.status,
                "created_at": a.created_at,
                "source": a.source,
                "type": a.alert_type,
                "affected_assets": a.affected_assets,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
    }
}

/// Body for `PATCH /api/v1/alerts/{alert_id}/status`.
#[derive(Debug, Deserialize)]
pub struct PatchStatusBody {
    pub status: String,
}

/// `PATCH /api/v1/alerts/{alert_id}/status`
///
/// Acknowledges an alert. Updates `alert_store[(org_id, alert_id)].status`.
/// Returns 400 if the alert is already closed.
pub async fn patch_alert_status(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
    Json(body): Json<PatchStatusBody>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    match alert_store.get_mut(&(org_id, alert_id.clone())) {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = body.status.clone();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "alert_id": alert_id,
                    "status": body.status,
                })),
            )
                .into_response()
        }
    }
}

/// `POST /api/v1/alerts/{alert_id}/close`
///
/// Closes an alert irreversibly within the session.
/// Only `reset_all()` can restore the alert to "open".
pub async fn post_close_alert(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    match alert_store.get_mut(&(org_id, alert_id.clone())) {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = "closed".to_owned();
            record.closed = true;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "alert_id": alert_id,
                    "status": "closed",
                })),
            )
                .into_response()
        }
    }
}
