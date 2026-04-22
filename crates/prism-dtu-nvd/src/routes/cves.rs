//! `GET /rest/json/cves/2.0` — unified CVE endpoint (single + bulk fetch).
//!
//! Query parameters:
//! - `apiKey` (optional): if present, uses authenticated rate-limit bucket (50/30s);
//!   if absent, uses unauthenticated bucket (5/30s).
//! - `cveId` (optional): single CVE lookup; takes precedence over pagination params.
//! - `startIndex` (optional, default 0): pagination offset for bulk fetch.
//! - `resultsPerPage` (optional, default 2000): page size; minimum 1.
//! - `lastModStartDate` / `lastModEndDate` (optional): date-range filter (accepted but
//!   ignored by the DTU — always returns fixture subset).
//!
//! Rate limit errors:
//! - Unauthenticated bucket exhausted → HTTP 403 (Unauthenticated rate limit exceeded)
//! - Authenticated bucket exhausted → HTTP 429
//! - `auth_mode=reject` + `apiKey` present → HTTP 403 (apiKey not verified)
//!
//! CVE not found → HTTP 404.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::state::NvdState;

/// Query parameters accepted by `GET /rest/json/cves/2.0`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CveQueryParams {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    #[serde(rename = "cveId")]
    pub cve_id: Option<String>,
    pub start_index: Option<u32>,
    pub results_per_page: Option<u32>,
    pub last_mod_start_date: Option<String>,
    pub last_mod_end_date: Option<String>,
}

/// `GET /rest/json/cves/2.0`
///
/// Dispatches to single CVE lookup (when `cveId` is present) or bulk paginated
/// fetch (when pagination params are present or both are absent).
pub async fn get_cves(
    State(state): State<Arc<NvdState>>,
    Query(params): Query<CveQueryParams>,
) -> impl IntoResponse {
    todo!() as (StatusCode, Json<serde_json::Value>)
}
