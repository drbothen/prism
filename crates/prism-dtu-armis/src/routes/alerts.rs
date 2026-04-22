//! Alert / policy violation list route handler.
//!
//! Endpoint:
//! - `GET /api/v1/alerts` — returns paginated alert list
//!
//! Auth: requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}`.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::state::ArmisState;
use crate::types::{AlertRecord, AlertsData, AlertsResponse, ArmisError};

/// Query parameters for `GET /api/v1/alerts`.
#[derive(Debug, Deserialize, Default)]
pub struct AlertQueryParams {
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// `GET /api/v1/alerts`
///
/// Returns a paginated list of alert / policy violation records.
/// Pagination: `page` (1-based, default 1), `size` (default 25).
pub async fn get_alerts(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<AlertQueryParams>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(25).max(1) as usize;
    let offset = ((page - 1) as usize) * size;

    let all_alerts = &state.alert_fixture;
    let total = all_alerts.len() as u32;

    let page_alerts: Vec<AlertRecord> = if offset >= all_alerts.len() {
        vec![]
    } else {
        all_alerts.iter().skip(offset).take(size).cloned().collect()
    };

    let body = AlertsResponse {
        data: AlertsData {
            alerts: page_alerts,
            total,
        },
    };

    (StatusCode::OK, Json(body)).into_response()
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
