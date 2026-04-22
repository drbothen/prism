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

use crate::state::{NvdState, RateLimitError};
use crate::types::{CveResponse, NvdError, VulnerabilityWrapper};

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
    let api_key = params.api_key.as_deref();

    // Check rate limit (and auth_mode=reject).
    match state.check_rate_limit(api_key) {
        Ok(()) => {}
        Err(RateLimitError::UnauthenticatedExceeded) => {
            let body = NvdError {
                error: "Forbidden. Too many requests. (Unauthenticated rate limit exceeded.)"
                    .to_owned(),
                cve_id: None,
            };
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::to_value(body).expect("NvdError serialization")),
            )
                .into_response();
        }
        Err(RateLimitError::AuthenticatedExceeded) => {
            let body = NvdError {
                error: "Too many requests. Retry after current 30-second window expires."
                    .to_owned(),
                cve_id: None,
            };
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::to_value(body).expect("NvdError serialization")),
            )
                .into_response();
        }
        Err(RateLimitError::ApiKeyRejected) => {
            let body = NvdError {
                error: "Forbidden. apiKey not verified.".to_owned(),
                cve_id: None,
            };
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::to_value(body).expect("NvdError serialization")),
            )
                .into_response();
        }
    }

    // cveId takes precedence over pagination (EC-001).
    if let Some(cve_id) = params.cve_id {
        return handle_single_cve(&state, &cve_id).await.into_response();
    }

    // Bulk paginated fetch.
    handle_bulk_fetch(&state, params.start_index, params.results_per_page)
        .await
        .into_response()
}

async fn handle_single_cve(state: &NvdState, cve_id: &str) -> impl IntoResponse {
    match state.lookup_and_count(cve_id) {
        Some(record) => {
            let resp = CveResponse {
                results_per_page: 1,
                start_index: 0,
                total_results: 1,
                format: "NVD_CVE".to_owned(),
                version: "2.0".to_owned(),
                timestamp: fixture_timestamp(),
                vulnerabilities: vec![VulnerabilityWrapper { cve: record }],
            };
            (
                StatusCode::OK,
                Json(serde_json::to_value(resp).expect("CveResponse serialization")),
            )
                .into_response()
        }
        None => {
            let normalized = cve_id.to_uppercase();
            let body = NvdError {
                error: "CVE not found".to_owned(),
                cve_id: Some(normalized),
            };
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::to_value(body).expect("NvdError serialization")),
            )
                .into_response()
        }
    }
}

async fn handle_bulk_fetch(
    state: &NvdState,
    start_index: Option<u32>,
    results_per_page: Option<u32>,
) -> impl IntoResponse {
    let start = start_index.unwrap_or(0) as usize;
    // EC-002: resultsPerPage=0 treated as 1.
    let page_size = results_per_page.map(|r| r.max(1)).unwrap_or(2000) as usize;

    // Collect all CVEs in deterministic order (sorted by ID).
    let mut all_cves: Vec<_> = state.cve_registry.values().cloned().collect();
    all_cves.sort_by(|a, b| a.id.cmp(&b.id));

    let total = all_cves.len();

    // EC-003: startIndex beyond total returns empty array.
    let page: Vec<VulnerabilityWrapper> = if start >= total {
        vec![]
    } else {
        all_cves
            .into_iter()
            .skip(start)
            .take(page_size)
            .map(|cve| VulnerabilityWrapper { cve })
            .collect()
    };

    let actual_page_size = page.len();
    let resp = CveResponse {
        results_per_page: actual_page_size as u32,
        start_index: start as u32,
        total_results: total as u32,
        format: "NVD_CVE".to_owned(),
        version: "2.0".to_owned(),
        timestamp: fixture_timestamp(),
        vulnerabilities: page,
    };

    (
        StatusCode::OK,
        Json(serde_json::to_value(resp).expect("CveResponse serialization")),
    )
        .into_response()
}

fn fixture_timestamp() -> String {
    "2024-01-01T00:00:00.000".to_owned()
}
