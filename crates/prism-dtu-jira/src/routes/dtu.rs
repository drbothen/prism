//! DTU-internal test API routes (`/dtu/*`).
//!
//! These routes are NOT present in the real Jira Cloud REST API — they exist solely
//! to support integration test assertions and harness control:
//!
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, etc.)
//! - `POST /dtu/reset` — reset all mutable state (issue registry + counter)
//! - `GET /dtu/health` — liveness check (no state access; safe for readiness polling)
//! - `GET /dtu/issues` — returns all current issues for test assertions (AC-1)
//!
//! Per ADR-002 §6: configure, reset, and health are required for every clone.
//! `GET /dtu/issues` is Jira-specific introspection per story task #9.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use subtle::ConstantTimeEq;

use crate::state::JiraState;
use crate::types::{DtuIssueSummary, DtuIssuesResponse};

/// `POST /dtu/configure`
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if absent
/// or incorrect.
pub async fn post_configure(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    // SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
    let provided = headers
        .get("x-admin-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let provided_bytes = provided.as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }
    match state.apply_config(&body) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// `POST /dtu/reset`
///
/// Resets all mutable DTU state: issue registry cleared, next_issue_num reset to 1000.
///
/// Per ADR-002 §4: delegates to `state.reset()`.
///
/// # ADR-003 Amendment #5 (TD-WV0-08)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 if missing
/// or incorrect.
pub async fn post_reset(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
    let provided = headers
        .get("x-admin-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let provided_bytes = provided.as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health`
///
/// Liveness check — returns `HTTP 200 {"status": "ok"}` with no state access.
/// Safe for test-harness readiness polling without side effects.
///
/// Per ADR-002 §6: required for every clone.
pub async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/issues`
///
/// Returns all current issues in the registry for integration test assertions.
///
/// Response: `{"issues": [{"key": "ACME-SEC-1000", "status": "Open", "summary": "...",
/// "comment_count": 0}, ...]}`
///
/// Per story task #9: test API for AC-1 assertion (`GET /dtu/issues` shows the issue
/// with `status: "Open"` after creation).
pub async fn get_dtu_issues(State(state): State<Arc<JiraState>>) -> impl IntoResponse {
    let issues = state
        .all_issues()
        .into_iter()
        .map(|issue| DtuIssueSummary {
            key: issue.key,
            status: issue.status.display_name().to_owned(),
            summary: issue.summary,
            comment_count: issue.comment_count,
        })
        .collect();

    (StatusCode::OK, Json(DtuIssuesResponse { issues })).into_response()
}
