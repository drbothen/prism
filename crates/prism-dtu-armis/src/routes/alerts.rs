//! Alert / policy violation list route handler.
//!
//! Endpoint:
//! - `GET /api/v1/alerts` — returns paginated alert list
//!
//! Auth: requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}`.
//!
//! `X-Org-Id` uses the same **dual-mode** policy as `devices.rs`:
//! - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent header → 401.
//! - Default-instance clones: absent header → 200 (backward compat).
//!
//! (CR-017 / M-50-001; BC-3.5.002 precondition 3; BC-3.2.001 precondition 4)

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::routes::devices::validate_org_id;
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

    // CR-017 / M-50-001: dual-mode X-Org-Id policy.
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
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
