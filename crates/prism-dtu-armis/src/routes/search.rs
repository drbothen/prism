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
    http::HeaderMap,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::state::ArmisState;

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
    State(_state): State<Arc<ArmisState>>,
    _headers: HeaderMap,
    Query(_params): Query<SearchQueryParams>,
) -> Response {
    todo!("S-DEMO-ARMIS-AQL-001: implement AQL search handler — capture AQL via state.capture_aql(), route by AQL pattern (in:type=Device vs alert), return SearchResponse envelope")
}
