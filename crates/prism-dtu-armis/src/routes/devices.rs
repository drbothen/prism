//! Device inventory, activity log, and risk score route handlers.
//!
//! Endpoints:
//! - `GET /api/v1/devices` — AQL-forwarded device query (GET form)
//! - `POST /api/v1/devices` — AQL-forwarded device query (POST form)
//!   (Armis supports both methods per API spec — EC-005)
//! - `GET /api/v1/devices/{device_id}/activity` — device activity log
//! - `GET /api/v1/devices/{device_id}/risk` — device risk score
//!
//! Auth: all endpoints require `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}` (Armis returns
//! 403, not 401 — AC-5, per `dtu-assessment.md §3.4`).
//!
//! AQL passthrough: `aql` query parameter (or POST body field) is accepted verbatim,
//! appended to `aql_log`, and NOT parsed or validated (R-DTU-002 mitigation).

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::state::{ArmisState, DTU_ROUTE_ORG_ID};
use crate::types::{
    ActivityData, ActivityResponse, ArmisError, DeviceRecord, DevicesData, DevicesResponse,
    RiskData, RiskResponse,
};

/// Query parameters accepted by `GET /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
pub struct DeviceQueryParams {
    /// AQL string — accepted verbatim, stored in AQL log (not parsed).
    pub aql: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<u32>,
    /// Page size. Defaults to 25.
    pub size: Option<u32>,
}

/// POST body accepted by `POST /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
pub struct DeviceQueryBody {
    pub aql: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// `GET /api/v1/devices` — device inventory with AQL forwarding and pagination.
pub async fn get_or_post_devices(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // Capture AQL string verbatim (R-DTU-002).
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(25).max(1) as usize;

    paginate_devices(&state, page, size as u32)
}

/// `POST /api/v1/devices` — AQL device query via JSON body (EC-005).
///
/// Armis supports both GET (query-param AQL) and POST (JSON body AQL).
/// This handler reads AQL from the JSON body and falls back to query-param AQL.
pub async fn post_devices(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
    body: Option<Json<DeviceQueryBody>>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // AQL priority: JSON body > query param (R-DTU-002).
    let aql = body
        .as_ref()
        .and_then(|b| b.aql.clone())
        .or_else(|| params.aql.clone());

    if let Some(ref aql_str) = aql {
        state.capture_aql(aql_str);
    }

    let page = body
        .as_ref()
        .and_then(|b| b.page)
        .or(params.page)
        .unwrap_or(1)
        .max(1);
    let size = body
        .as_ref()
        .and_then(|b| b.size)
        .or(params.size)
        .unwrap_or(25)
        .max(1);

    paginate_devices(&state, page, size)
}

/// Pagination helper shared by GET and POST device queries.
fn paginate_devices(state: &ArmisState, page: u32, size: u32) -> axum::response::Response {
    let all_devices = &state.devices_ordered;
    let total = all_devices.len() as u32;
    let offset = ((page - 1) * size) as usize;

    // EC-004: page beyond last → empty devices array, correct total.
    let page_devices: Vec<DeviceRecord> = if offset >= all_devices.len() {
        vec![]
    } else {
        all_devices
            .iter()
            .skip(offset)
            .take(size as usize)
            .map(|d| {
                // BC-3.2.001: merge per-org tag_store entries with fixture tags.
                // DTU clone is a single-tenant HTTP server per test instance; use DTU_ROUTE_ORG_ID.
                let merged_tags = state.tags_for(DTU_ROUTE_ORG_ID, &d.device_id, &d.tags);
                DeviceRecord {
                    tags: merged_tags,
                    ..d.clone()
                }
            })
            .collect()
    };

    let body = DevicesResponse {
        data: DevicesData {
            devices: page_devices,
            total,
            page,
        },
    };

    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /api/v1/devices/{device_id}/activity`
///
/// Returns activity records filtered to the requested device_id.
pub async fn get_device_activity(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    let activities: Vec<_> = state
        .activity_fixture
        .iter()
        .filter(|a| a.device_id == device_id)
        .cloned()
        .collect();

    let total = activities.len() as u32;
    let body = ActivityResponse {
        data: ActivityData { activities, total },
    };

    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /api/v1/devices/{device_id}/risk`
///
/// Returns the risk score for a device.
/// EC-002: device not in fixture → HTTP 404 `{"error": "device not found", "code": 404}`.
pub async fn get_device_risk(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    match state.device_registry.get(&device_id) {
        Some(device) => {
            let body = RiskResponse {
                data: RiskData {
                    device_id: device.device_id.clone(),
                    risk_score: device.risk_score.unwrap_or(0),
                    risk_factors: device.risk_factors.clone(),
                },
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        None => {
            let body = ArmisError {
                error: "device not found".to_owned(),
                code: 404,
            };
            (StatusCode::NOT_FOUND, Json(body)).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Validate the `Authorization: Bearer {non-empty}` header.
///
/// Returns `Some(response)` if the request is unauthorized (HTTP 403), or
/// `None` if the header is present and valid.
///
/// Per AC-5 and `dtu-assessment.md §3.4`: Armis returns 403, NOT 401.
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
