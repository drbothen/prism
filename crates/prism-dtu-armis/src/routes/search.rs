//! AQL search route handler.
//!
//! Endpoint:
//! - `GET /api/v1/search` — AQL-forwarded unified search (devices or alerts)
//!
//! This is the primary data query path in the real Armis Centrix API (ADR-031 §D8-a).
//! The production poller (poller-coaster) uses `centrix.Search()` with an AQL query
//! string for all 7 data sources. This route closes Gap-AR-001 / DTU-EXT-003 / DTU-EXT-004.
//!
//! # Auth model
//!
//! Requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}` (AC-001 EC-004).
//!
//! # AQL treatment
//!
//! The `aql` query parameter is accepted verbatim, captured via `state.capture_aql()`,
//! and NOT parsed or validated (R-DTU-002 / ADR-005 §D1 opaque AQL model).
//! Simple string pattern matching determines whether to return devices or alerts:
//! - Contains `in:type=Device` (or device selector) → DeviceRecord results
//! - Contains `in:type=Alert` or alert keyword → AlertRecord results
//! - Absent or unrecognized → devices (safe fallback, EC-001)
//!
//! # Response envelope
//!
//! Real Armis `/api/v1/search` returns `{"data": {"results": [...], "total": N}}`.
//! Note `results` (not `devices` or `alerts`) — matches `$.data.results` in armis.sensor.toml.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    state::{ArmisState, DTU_ROUTE_ORG_ID},
    types::ArmisError,
};

/// Query parameters accepted by `GET /api/v1/search`.
///
/// Per AC-001 / AC-002 / AC-003: `aql` carries the AQL filter string.
/// `page` and `size` mirror the pagination params used by devices/alerts endpoints.
#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    /// AQL filter string — accepted verbatim, stored in AQL log (R-DTU-002).
    pub aql: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<u32>,
    /// Page size. Defaults to 25.
    pub size: Option<u32>,
}

/// Top-level search response wrapper.
///
/// Real Armis `/api/v1/search` envelope: `{"data": {"results": [...], "total": N}}`.
/// Used by `GET /api/v1/search` (the primary path per ADR-031 §D8-a).
/// `armis.sensor.toml` response_path = `"$.data.results"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: SearchData,
}

/// Inner data envelope for the search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchData {
    /// Unified results array — may contain `DeviceRecord` or `AlertRecord` entries
    /// depending on the AQL filter, serialized as JSON objects.
    pub results: Vec<serde_json::Value>,
    /// Total count of matching records (before pagination).
    pub total: u32,
}

/// `GET /api/v1/search` — AQL-forwarded unified search handler.
///
/// Accepts an `aql` query parameter, captures it via `state.capture_aql()`,
/// inspects the AQL string to determine the response type (devices or alerts),
/// and returns the appropriate paginated result in the search envelope format.
///
/// AC-001: registered in `build_router()` → returns 200 with valid Bearer, 403 without.
/// AC-002: `in:type=Device` AQL → returns DeviceRecord results.
/// AC-003: alert AQL → returns AlertRecord results.
/// EC-001: absent AQL → defaults to devices.
pub async fn get_search(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    // AC-001 EC-004: 403 (not 401) for missing/empty Bearer — matches Armis auth model.
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // R-DTU-002 / ADR-005 §D1: capture AQL verbatim, no parsing or validation.
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(25).max(1) as usize;

    // Determine whether to return alerts or devices based on AQL pattern matching.
    // R-DTU-002: AQL is opaque — only simple string pattern matching is permitted.
    // EC-002: if both Device and Alert appear, devices take precedence.
    // EC-001: absent AQL → default to devices (safe fallback).
    let return_alerts = params
        .aql
        .as_deref()
        .map(|s| {
            (s.contains("Alert") || s.contains("alert"))
                && !s.contains("Device")
                && !s.contains("device")
        })
        .unwrap_or(false);

    if return_alerts {
        // AC-003: alert AQL → paginated AlertRecord results.
        let all_alerts = &state.alert_fixture;
        let total = all_alerts.len() as u32;
        let offset = ((page - 1) as usize) * size;

        let page_alerts: Vec<serde_json::Value> = if offset >= all_alerts.len() {
            vec![]
        } else {
            all_alerts
                .iter()
                .skip(offset)
                .take(size)
                .filter_map(|a| serde_json::to_value(a).ok())
                .collect()
        };

        let body = SearchResponse {
            data: SearchData {
                results: page_alerts,
                total,
            },
        };
        (StatusCode::OK, Json(body)).into_response()
    } else {
        // AC-002: device AQL (or absent AQL per EC-001) → paginated DeviceRecord results.
        let all_devices = &state.devices_ordered;
        let total = all_devices.len() as u32;
        let offset = ((page - 1) as usize) * size;

        let page_devices: Vec<serde_json::Value> = if offset >= all_devices.len() {
            vec![]
        } else {
            all_devices
                .iter()
                .skip(offset)
                .take(size)
                .filter_map(|d| {
                    // BC-3.2.001: merge per-org tag_store entries with fixture tags.
                    let merged_tags = state.tags_for(DTU_ROUTE_ORG_ID, &d.device_id, &d.tags);
                    let merged = crate::types::DeviceRecord {
                        tags: merged_tags,
                        ..d.clone()
                    };
                    serde_json::to_value(&merged).ok()
                })
                .collect()
        };

        let body = SearchResponse {
            data: SearchData {
                results: page_devices,
                total,
            },
        };
        (StatusCode::OK, Json(body)).into_response()
    }
}

/// Validate `Authorization: Bearer {non-empty}` header.
///
/// Returns `Some(response)` on auth failure (HTTP 403) or `None` when valid.
/// Per AC-001 EC-004 / AC-5 (`dtu-assessment.md §3.4`): Armis returns 403, NOT 401.
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
