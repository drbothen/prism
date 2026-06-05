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
//! `X-Org-Id` uses the same **dual-mode** policy as `devices.rs` (CR-012/SEC-P2-001):
//!
//! - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent header → 401.
//! - Default-instance clones: absent header → 200 (backward compat).
//!
//! (BC-3.5.002 precondition 3; TD-VSDD-060 sibling-sweep of devices.rs + alerts.rs guard)
//!
//! # AQL treatment
//!
//! The `aql` query parameter is accepted verbatim, captured via `state.capture_aql()`,
//! and NOT parsed or validated (R-DTU-002 / ADR-005 §D1 opaque AQL model).
//! Simple string pattern matching determines whether to return devices or alerts using the
//! real Armis Centrix `in:<entity>` discriminator syntax (research artifact 2026-06-01):
//! - Contains `in:alerts` → AlertRecord results
//! - Contains `in:devices` (or absent AQL per EC-001) → DeviceRecord results
//! - Both present → devices take precedence (EC-002)
//!
//! Real Armis AQL examples (from 1898 production poller + Armis community):
//! - `in:devices` — all devices
//! - `in:devices type:(switch)` — devices filtered to switch type
//! - `in:alerts status:Open` — open alerts
//!
//! # Response envelope
//!
//! Real Armis `/api/v1/search` returns `{"data": {"results": [...], "total": N}}`.
//! Note `results` (not `devices` or `alerts`) — matches `$.data.results` in armis.sensor.toml.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    routes::devices::validate_org_id,
    state::{ArmisState, DTU_ROUTE_ORG_ID},
    types::ArmisError,
};

/// Query parameters accepted by `GET /api/v1/search`.
///
/// Per AC-001 / AC-002 / AC-003: `aql` carries the AQL filter string.
/// `page` and `size` mirror the real Armis pagination params.
/// `offset` and `limit` are the prism-spec-engine OffsetLimit pagination params
/// (sent by PipelineExecutor via `build_paged_url` — see pipeline.rs `OffsetLimit` arm).
/// Both styles are accepted; when `offset`/`limit` are present they take precedence
/// over `page`/`size`. This allows the DTU to respond correctly to the pipeline's
/// OffsetLimit pagination without requiring the real Armis API's param names.
#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    /// AQL filter string — accepted verbatim, stored in AQL log (R-DTU-002).
    pub aql: Option<String>,
    /// Page number (1-based). Defaults to 1. Ignored when `offset` is present.
    pub page: Option<u32>,
    /// Page size. Defaults to 25. Ignored when `limit` is present.
    pub size: Option<u32>,
    /// Zero-based record offset (prism OffsetLimit pagination convention).
    /// When present, `page` is derived as `offset / effective_size + 1`.
    pub offset: Option<u32>,
    /// Maximum records to return (prism OffsetLimit pagination convention).
    /// When present, overrides `size`.
    pub limit: Option<u32>,
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
/// AC-002: `in:devices` AQL → returns DeviceRecord results.
/// AC-003: `in:alerts` AQL → returns AlertRecord results.
/// EC-001: absent AQL → defaults to devices.
/// EC-002: if both `in:devices` and `in:alerts` are present, devices take precedence;
///   route to alerts only when `in:alerts` is present and `in:devices` is absent.
pub async fn get_search(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    // AC-001 EC-004: 403 (not 401) for missing/empty Bearer — matches Armis auth model.
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-012/SEC-P2-001: dual-mode X-Org-Id policy (sibling-sweep TD-VSDD-060).
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat with existing tests).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    // R-DTU-002 / ADR-005 §D1: capture AQL verbatim, no parsing or validation.
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    // Resolve pagination parameters.
    // Prefer `offset`/`limit` (prism OffsetLimit convention) over `page`/`size` (real Armis API).
    // The prism PipelineExecutor sends `?offset=N&limit=M` via build_paged_url for OffsetLimit steps.
    // The real Armis API uses `?page=N&size=M`. Both conventions are accepted for DTU fidelity.
    let (size, start_offset): (usize, usize) = if let Some(lim) = params.limit {
        // prism OffsetLimit path: `?offset=N&limit=M`
        let effective_size = (lim as usize).max(1);
        let effective_offset = params.offset.unwrap_or(0) as usize;
        (effective_size, effective_offset)
    } else {
        // Real Armis API path: `?page=N&size=M`
        let effective_size = params.size.unwrap_or(25).max(1) as usize;
        let page = params.page.unwrap_or(1).max(1) as usize;
        let effective_offset = (page - 1) * effective_size;
        (effective_size, effective_offset)
    };

    // Determine whether to return alerts or devices based on AQL pattern matching.
    // R-DTU-002: AQL is opaque — only simple string pattern matching is permitted.
    // Real Armis `in:<entity>` discriminator syntax (research-artifact 2026-06-01,
    // grounded from 1898 production poller + 3 independent external connectors):
    //   - `in:alerts` → alert entity selection
    //   - `in:devices` → device entity selection
    // EC-002: if both `in:devices` and `in:alerts` are present, devices take precedence.
    // EC-001: absent AQL → default to devices (safe fallback).
    let return_alerts = params
        .aql
        .as_deref()
        .map(|s| s.contains("in:alerts") && !s.contains("in:devices"))
        .unwrap_or(false);

    if return_alerts {
        // AC-003: alert AQL → paginated AlertRecord results.
        let all_alerts = &state.alert_fixture;
        let total = all_alerts.len() as u32;

        let page_alerts: Vec<serde_json::Value> = if start_offset >= all_alerts.len() {
            vec![]
        } else {
            all_alerts
                .iter()
                .skip(start_offset)
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

        let page_devices: Vec<serde_json::Value> = if start_offset >= all_devices.len() {
            vec![]
        } else {
            all_devices
                .iter()
                .skip(start_offset)
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

// ---------------------------------------------------------------------------
// AQL time-clause parsing (§8.3 — pushdown-redesign.md)
// ---------------------------------------------------------------------------

/// Parse `after:` and `before:` absolute time clauses from an AQL string.
///
/// Implements pushdown-redesign.md §8.3: after `capture_aql()` (R-DTU-002, opaque capture
/// is unaffected), parse the AQL string for `after:YYYY-MM-DDTHH:MM:SS` and
/// `before:YYYY-MM-DDTHH:MM:SS` clauses to produce optional time bounds.
///
/// # Canonical AQL syntax (research-confirmed HIGH confidence)
///
/// - `after:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive ISO8601.
/// - `before:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive ISO8601.
/// - Date-only form `YYYY-MM-DD` also accepted (BlinkOps source).
///
/// # Return value
///
/// Returns `(after_bound, before_bound)` as `Option<chrono::DateTime<chrono::Utc>>`.
/// `None` means the corresponding clause is absent from the AQL string.
///
/// # R-DTU-002 compliance
///
/// This function does NOT modify or validate the AQL string. It only extracts
/// time bounds for fixture filtering. The `capture_aql()` call in `get_search`
/// still captures the verbatim string first (R-DTU-002 preserved).
///
/// # Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1
/// Red Gate stub — returns `(None, None)` (no parsing) until implemented.
/// AC-ARMIS-TW-002 LOAD-BEARING test asserts filtered_count < unfiltered_count; FAILS here.
// S-DEMO-QUERY-PUSHDOWN-001 v2.1 Red Gate stub.
// Dead-code is suppressed because this function is wired by the implementer in §8.3.
// The inline unit tests in pushdown_dtu_red_gate_tests exercise this function directly.
#[allow(dead_code)]
pub(crate) fn parse_aql_time_bounds(aql: &str) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    // Red Gate stub: suppress unused-variable warning.
    let _ = aql;
    // Returns (None, None) — no AQL time-clause parsing.
    // With this stub: DTU returns full fixture regardless of after:/before: in AQL.
    // AC-ARMIS-TW-002 test assertion: filtered_count < unfiltered_count → FAILS here.
    (None, None)
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

// ---------------------------------------------------------------------------
// S-DEMO-QUERY-PUSHDOWN-001 v2.1 — DTU Red Gate tests for parse_aql_time_bounds
// ---------------------------------------------------------------------------

#[cfg(test)]
mod pushdown_dtu_red_gate_tests {
    //! Red Gate tests for S-DEMO-QUERY-PUSHDOWN-001 v2.1 — DTU side.
    //!
    //! Tests `parse_aql_time_bounds` (the stub function in this module).
    //! ALL tests in this module MUST FAIL before the implementer wires the real parser.
    //!
    //! # SID-1 Compliance
    //! These are in-process unit tests that exercise `parse_aql_time_bounds` directly
    //! without any external DTU dependency. They provide the non-#[ignore] coverage
    //! required by SID-1 §2.
    //!
    //! # Load-bearing assertion
    //! `test_ac_armis_tw_002_dtu_parse_aql_after_clause_yields_bound`:
    //! asserts `after:YYYY-MM-DDTHH:MM:SS` → `Some(DateTime<Utc>)`.
    //! The stub returns (None, None) → test FAILS (Red Gate correctly RED).

    use chrono::Datelike;

    use super::parse_aql_time_bounds;

    /// Unit test for parse_aql_time_bounds: `after:` clause → start bound.
    ///
    /// Red Gate: stub returns (None, None). Test asserts Some(start_bound) → FAILS.
    ///
    /// Canonical Armis AQL absolute time syntax (research-doc §2.2, HIGH confidence):
    /// `after:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive ISO8601.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_after_clause_yields_bound() {
        let (after_bound, before_bound) =
            parse_aql_time_bounds("in:devices after:2024-06-11T12:00:00");

        assert!(
            after_bound.is_some(),
            "parse_aql_time_bounds must parse 'after:2024-06-11T12:00:00' and return \
             Some(start_bound); got None. \
             Red Gate: stub returns (None, None). Implementer must wire the parser. \
             Canonical syntax: bare, unquoted, timezone-naive (research-doc §2.2)."
        );
        assert!(
            before_bound.is_none(),
            "parse_aql_time_bounds: no 'before:' clause → before_bound must be None; \
             got: {:?}",
            before_bound
        );

        // Validate the parsed bound's year/month to guard against off-by-one.
        if let Some(bound) = after_bound {
            assert_eq!(bound.year(), 2024, "parsed start year must be 2024");
            assert_eq!(bound.month(), 6, "parsed start month must be 6 (June)");
            assert_eq!(bound.day(), 11, "parsed start day must be 11");
        }
    }

    /// Unit test: `before:` clause → end bound.
    ///
    /// Red Gate: stub returns (None, None). Test asserts Some(end_bound) → FAILS.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_before_clause_yields_bound() {
        let (after_bound, before_bound) =
            parse_aql_time_bounds("in:devices before:2024-12-31T00:00:00");

        assert!(
            before_bound.is_some(),
            "parse_aql_time_bounds must parse 'before:2024-12-31T00:00:00' and return \
             Some(end_bound); got None. Red Gate: stub returns (None, None)."
        );
        assert!(
            after_bound.is_none(),
            "parse_aql_time_bounds: no 'after:' clause → after_bound must be None; \
             got: {:?}",
            after_bound
        );
    }

    /// Unit test: bounded range `after:T1 before:T2` → both bounds.
    ///
    /// Red Gate: stub returns (None, None). Test asserts (Some, Some) → FAILS.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_bounded_range_yields_both_bounds() {
        let (after_bound, before_bound) = parse_aql_time_bounds(
            "in:devices after:2024-01-01T00:00:00 before:2024-12-31T23:59:59",
        );

        assert!(
            after_bound.is_some(),
            "parse_aql_time_bounds bounded range: must parse 'after:' → Some(start); got None."
        );
        assert!(
            before_bound.is_some(),
            "parse_aql_time_bounds bounded range: must parse 'before:' → Some(end); got None."
        );
    }

    /// Unit test: AQL with no time clause → (None, None).
    ///
    /// PASSES with stub (returns (None, None) — correct behavior).
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_no_time_clause_returns_none() {
        let (after_bound, before_bound) = parse_aql_time_bounds("in:devices");

        assert!(
            after_bound.is_none(),
            "parse_aql_time_bounds: no time clause → after_bound must be None; got: {:?}",
            after_bound
        );
        assert!(
            before_bound.is_none(),
            "parse_aql_time_bounds: no time clause → before_bound must be None; got: {:?}",
            before_bound
        );
    }
}
