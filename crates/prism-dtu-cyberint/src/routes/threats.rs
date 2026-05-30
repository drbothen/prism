//! Threat intelligence route handler for the Cyberint DTU clone.
//!
//! Routes:
//! - `GET /api/v1/threat-intel` — threat intelligence feed with cursor pagination
//!
//! All routes require cookie auth — validated via `extract_access_token` (ADR-031 §D3-a).

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    routes::alerts::extract_access_token,
    state::{AuthMode, CyberintState},
};

/// Query parameters for the threat-intel endpoint.
#[derive(Debug, Deserialize, Default)]
pub struct ThreatListParams {
    pub cursor: Option<String>,
}

/// `GET /api/v1/threat-intel`
///
/// Returns paginated threat intelligence feed from `fixtures/threats.json`.
/// Requires valid `access_token` cookie (ADR-031 §D3-a; AC-003).
pub async fn get_threat_intel(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Query(params): Query<ThreatListParams>,
) -> impl IntoResponse {
    // Auth check — use access_token (not session cookie; ADR-031 §D3-a).
    if state.auth_mode() == AuthMode::Reject {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized", "code": 401})),
        )
            .into_response();
    }
    let token = match extract_access_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "unauthorized", "code": 401})),
            )
                .into_response()
        }
    };
    if !state.is_valid_access_token(&token) {
        todo!(
            "AC-003 (threats): return 401 when access_token cookie is present but not in allowlist; \
             access_token validation is org-agnostic (ADR-031 §D3-a)"
        )
    }
    if state.check_and_increment_rate_limit() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "rate limit exceeded", "code": 429})),
        )
            .into_response();
    }

    // Simple pagination: cursor present means page 2 (empty for threats fixture).
    let (data, next_cursor) = if params.cursor.is_some() {
        (Vec::<serde_json::Value>::new(), serde_json::Value::Null)
    } else {
        (state.threat_fixture.clone(), serde_json::Value::Null)
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data": data,
            "next_cursor": next_cursor,
        })),
    )
        .into_response()
}
