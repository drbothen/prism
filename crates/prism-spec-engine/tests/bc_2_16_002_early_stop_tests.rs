#![allow(non_snake_case)]
//! RG-001..RG-004 for S-ENGINE-LIMIT-EARLY-STOP-001: LIMIT-aware early-stop pagination.
//! BC-2.16.002 postconditions — LIMIT-Aware Early-Stop Pagination (ADR-060 §D8).
//! OBS-1 regression guard: CursorToken early-stop arm (ADR-060 §D8.4).
//!
//! | RG  | Name                                                              | Color          | Traces to |
//! |-----|-------------------------------------------------------------------|----------------|-----------|
//! | 001 | early_stop_fetch_context_new_stores_early_stop_limit              | GREEN (stub)   | AC-001    |
//! | 002 | early_stop_pipeline_stops_without_setting_truncated               | RED            | AC-002    |
//! | 003 | early_stop_none_fetches_all_pages                                 | GREEN (sentry) | AC-003    |
//! | 004 | early_stop_di019_fires_before_early_stop_check                    | GREEN (sentry) | AC-004    |
//! | OBS-1 | early_stop_cursor_token_stops_after_first_page                | GREEN (guard)  | ADR-060 §D8.4 |
//!
//! RG-001 passes now: `FetchContext::new` stub added @9530f3478 already carries
//!   `early_stop_limit: Option<usize>` and the matching constructor parameter.
//!
//! RG-002 is RED: `execute_impl` does not yet contain the early-stop `break 'steps`
//!   immediately after the DI-019 block. Without it the pipeline fetches all pages.
//!
//! RG-003 passes now and after implementation: `None` must leave full-pagination
//!   behaviour untouched. Regression sentinel.
//!
//! RG-004 passes now and after CORRECT implementation: DI-019 fires before early-stop,
//!   so `truncated` is set by DI-019, not by the early-stop path. This test catches the
//!   ordering bug where early-stop is mistakenly placed BEFORE DI-019.

use std::collections::HashMap;

use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::{
    NullAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::{AuthType, ColumnSpec, FetchStep, PaginationConfig, SensorSpec, TableSpec},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal GET OffsetLimit sensor spec pointing at `base_url`.
fn make_get_offset_spec(base_url: &str, page_size: u32) -> SensorSpec {
    SensorSpec::new(
        "early-stop-sensor",
        "Early Stop Sensor",
        AuthType::BearerStatic,
        base_url,
        vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_items",
                "GET",
                "/items",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::OffsetLimit { page_size }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    )
}

fn make_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed")
}

// ---------------------------------------------------------------------------
// RG-001 (GREEN) — FetchContext constructor stores early_stop_limit field
// ---------------------------------------------------------------------------

/// RG-001 (GREEN): `FetchContext::new` stores `early_stop_limit` field correctly.
///
/// Passes now — stub committed @9530f3478 already adds the field and constructor param.
/// Fails only before the field and constructor were added (pre-stub phase).
///
/// Traces to AC-001 (BC-2.16.002 LIMIT-Aware Early-Stop; ADR-060 §D8.1).
#[test]
fn test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit() {
    // Some(5): stored faithfully.
    let ctx_some = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), Some(5));
    assert_eq!(
        ctx_some.early_stop_limit,
        Some(5),
        "RG-001: FetchContext::new(..., Some(5)) must store early_stop_limit = Some(5); \
         got {:?}",
        ctx_some.early_stop_limit
    );

    // None: stored faithfully.
    let ctx_none = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    assert_eq!(
        ctx_none.early_stop_limit, None,
        "RG-001: FetchContext::new(..., None) must store early_stop_limit = None; \
         got {:?}",
        ctx_none.early_stop_limit
    );

    // Some(0): degenerate boundary — store the value even though it is semantically odd.
    let ctx_zero = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), Some(0));
    assert_eq!(
        ctx_zero.early_stop_limit,
        Some(0),
        "RG-001: FetchContext::new(..., Some(0)) must store early_stop_limit = Some(0)"
    );
}

// ---------------------------------------------------------------------------
// RG-002 (RED) — pipeline stops after first complete page, truncated stays false
// ---------------------------------------------------------------------------

/// RG-002 (RED): pipeline stops after the first complete page when
/// `early_stop_limit = Some(1)` with `page_size = 10`.
///
/// **Mock setup** (OffsetLimit GET, page_size=10):
///   - Mocks 1–3 (`up_to_n_times(1)` each): return 10 records (full page → continue).
///   - Mock 4 (fallback): returns 0 records (short page → loop terminates naturally).
///
/// **Without** the early-stop check in `execute_impl` (current code):
///   - All 4 requests are made → 30 records.
///   - `assert_eq!(result.records.len(), 10)` FAILS with "got 30" → RED gate.
///
/// **With** the early-stop check (AC-002 implementation):
///   - Page 1 brings `all_records.len()` to 10; check: 10 ≥ 1 → `break 'steps`.
///   - 1 HTTP request, 10 records, `truncated = false` → assertions pass.
///
/// Traces to AC-002, BC-2.16.002 LIMIT-Aware Early-Stop postcondition (ADR-060 §D8.2/D8.3).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated() {
    let mock_server = MockServer::start().await;

    let page_records: Vec<serde_json::Value> =
        (0u32..10).map(|i| serde_json::json!({"id": i})).collect();

    // Pages 1–3: full pages of 10 records (served in registration order, once each).
    for _ in 0..3 {
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "items": page_records })),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
    }
    // Page 4: empty (short page → OffsetLimit loop terminates naturally after page 3).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 10);
    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("rg002-org"), HashMap::new(), Some(1));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-002: pipeline execute must return Ok (no HTTP error expected)");

    // PRIMARY RED GATE: records must be from the first page only (10 records).
    // Without early-stop: 3 full pages → 30 records; assertion FAILS → RED gate fires.
    assert_eq!(
        result.records.len(),
        10,
        "RG-002 RED GATE: early-stop must halt after the first full page. \
         `early_stop_limit=Some(1)` + page_size=10: after page 1, all_records.len()=10 ≥ 1 \
         → `break 'steps`. Without the check, all pages are fetched (got {} records).",
        result.records.len()
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    // `truncated = true` is reserved exclusively for DI-019.
    assert!(
        !result.truncated,
        "RG-002: early-stop must NOT set truncated=true (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );

    // TERTIARY: exactly 1 HTTP request must be issued (1 page fetched).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "RG-002: exactly 1 HTTP request expected (early-stop after first page). \
         Without early-stop, 4 requests are made (3 full + 1 empty). Got {}.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// RG-003 (GREEN / regression sentinel) — None fetches all available pages
// ---------------------------------------------------------------------------

/// RG-003 (GREEN / regression sentinel): `early_stop_limit = None` fetches every page.
///
/// Must pass BOTH before and after the early-stop implementation. Verifies that
/// `None` leaves the full-pagination behaviour completely unchanged.
///
/// Same mock topology as RG-002: 3 full pages + 1 empty terminal page.
/// Expected: ≥ 3 HTTP requests; `records.len() == 30`; `truncated = false`.
///
/// Traces to AC-003 None-branch (BC-2.16.002 LIMIT-Aware Early-Stop).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_none_fetches_all_pages() {
    let mock_server = MockServer::start().await;

    let page_records: Vec<serde_json::Value> =
        (0u32..10).map(|i| serde_json::json!({"id": i})).collect();

    for _ in 0..3 {
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "items": page_records })),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 10);
    let table = spec.tables[0].clone();
    // None: no early-stop; full pagination.
    let context = FetchContext::new(OrgSlug::new("rg003-org"), HashMap::new(), None);
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-003: pipeline execute must return Ok");

    // All 3 data pages fetched → 30 records.
    assert_eq!(
        result.records.len(),
        30,
        "RG-003 SENTINEL: early_stop_limit=None must fetch ALL available pages. \
         Expected 30 records (3 pages × 10 each). Got {} records. \
         If this sentinel fails, the implementation broke the None branch.",
        result.records.len()
    );
    assert!(
        !result.truncated,
        "RG-003 SENTINEL: truncated must be false (30 records is below the 10K DI-019 cap \
         and no early-stop fires when early_stop_limit=None)."
    );

    // At least 3 HTTP requests (3 data pages; 4th empty request also expected in practice).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert!(
        received.len() >= 3,
        "RG-003 SENTINEL: at least 3 HTTP requests expected (one per data page). \
         Got {} requests.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// RG-004 (GREEN / DI-019 ordering sentinel)
// ---------------------------------------------------------------------------

/// RG-004 (GREEN / ordering sentinel): DI-019 fires BEFORE the early-stop check.
///
/// A single page returns 10 001 records. `early_stop_limit = Some(5)` would stop
/// the loop very early if placed BEFORE DI-019, leaving `truncated = false`.
/// But per ADR-060 §D8.2 the early-stop check lives IMMEDIATELY AFTER the DI-019
/// block — DI-019 must fire first, set `truncated = true`, and `break 'steps`.
///
/// **Current code** (no early-stop at all): DI-019 fires → `truncated = true` → GREEN.
///
/// **Correct implementation** (early-stop after DI-019): DI-019 still fires first → GREEN.
///
/// **Wrong implementation** (early-stop before DI-019): early-stop fires at 5 records,
///   `truncated` stays false → the `assert!(result.truncated)` FAILS → ordering bug detected.
///
/// Traces to AC-004 (BC-2.16.002 DI-019 ordering; ADR-060 §D8.2).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check() {
    let mock_server = MockServer::start().await;

    // Single page: 10 001 records — exceeds the DI-019 cap of 10 000.
    // page_size = 10 001 (full page → OffsetLimit would continue, but DI-019 fires first).
    let records_10001: Vec<serde_json::Value> = (0u32..10_001)
        .map(|i| serde_json::json!({"id": i}))
        .collect();
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": records_10001 })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Fallback (should never be reached when DI-019 fires correctly after page 1):
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "rg004-sensor",
        "RG-004 DI-019 Ordering Sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "big",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_big",
                "GET",
                "/big",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::OffsetLimit { page_size: 10_001 }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    let table = spec.tables[0].clone();
    // early_stop_limit=Some(5): would fire before DI-019 if placed incorrectly.
    let context = FetchContext::new(OrgSlug::new("rg004-org"), HashMap::new(), Some(5));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-004: pipeline execute must return Ok");

    // DI-019 must fire and set truncated = true.
    // If early-stop is incorrectly placed BEFORE DI-019: truncated stays false → FAILS.
    assert!(
        result.truncated,
        "RG-004 ORDERING SENTINEL: DI-019 must fire and set truncated=true. \
         10 001 records on the first page triggers DI-019 (all_records.len() 10001 ≥ 10000). \
         If early-stop is placed BEFORE DI-019, it fires at 5 records and truncated stays \
         false → ordering bug. Got truncated=false."
    );

    // DI-019 truncates to exactly 10 000 records.
    assert_eq!(
        result.records.len(),
        10_000,
        "RG-004 ORDERING SENTINEL: DI-019 must truncate to exactly 10 000 records. \
         Got {} records.",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// OBS-1 regression guard — CursorToken early-stop arm coverage
// ---------------------------------------------------------------------------

/// OBS-1 regression guard: `early_stop_limit` fires for `PaginationConfig::CursorToken`
/// sensors (CrowdStrike/Armis pattern) just as it does for OffsetLimit sensors.
///
/// **Finding OBS-1** (BC-2.16.002 §Postconditions + ADR-060 §D8.4): the early-stop
/// check sits above the pagination-advance `match` arm, so it fires mode-agnostically.
/// Without this test, a future refactor relocating the early-stop check INTO the
/// `OffsetLimit` match arm would silently break CursorToken sensors with no failing test.
///
/// **Mock setup** (CursorToken GET, `page_size: None`):
///   - Mock 1 (`up_to_n_times(1)`): returns 5 records + `next_cursor: "cursor_page_2"`.
///   - Mock 2 (`up_to_n_times(1)`): returns 5 records + `next_cursor: "cursor_page_3"`
///     (would be reached without early-stop).
///   - Mock 3 (fallback): returns empty / no cursor (terminal page, should never be hit).
///
/// **With** `early_stop_limit = Some(1)`:
///   - Page 1 delivers 5 records; `all_records.len()=5 >= 1` fires early-stop → `break 'steps`.
///   - Pages 2+ are never fetched; exactly 1 HTTP request is issued.
///
/// Traces to BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop; ADR-060 §D8.4.
#[tokio::test]
async fn test_BC_2_16_002_early_stop_cursor_token_stops_after_first_page() {
    let mock_server = MockServer::start().await;

    let page_records: Vec<serde_json::Value> =
        (0u32..5).map(|i| serde_json::json!({"id": i})).collect();

    // Page 1: 5 records + cursor (loop would continue without early-stop).
    Mock::given(method("GET"))
        .and(path("/cursor-items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_2"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: would be reached without early-stop (5 records + advancing cursor).
    Mock::given(method("GET"))
        .and(path("/cursor-items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_3"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3: terminal fallback (no cursor, empty records — should never be reached).
    Mock::given(method("GET"))
        .and(path("/cursor-items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "cursor-early-stop-sensor",
        "Cursor Early Stop Sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "cursor_items",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_cursor_items",
                "GET",
                "/cursor-items",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next_cursor".to_string(),
                    page_size: None,
                }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    let table = spec.tables[0].clone();
    // early_stop_limit = Some(1): fires after the first page (5 records >= 1).
    let context = FetchContext::new(OrgSlug::new("obs1-cursor-org"), HashMap::new(), Some(1));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("OBS-1: pipeline execute must return Ok (no HTTP error expected)");

    // PRIMARY: only 1 page was fetched (5 records from page 1).
    // If early-stop is not mode-agnostic (e.g., guarded inside OffsetLimit arm only),
    // the cursor loop continues: 5 + 5 = 10 records → assertion fails → regression caught.
    assert_eq!(
        result.records.len(),
        5,
        "OBS-1 REGRESSION GUARD: early-stop with CursorToken must halt after the first page. \
         `early_stop_limit=Some(1)` + 5 records/page: after page 1, all_records.len()=5 >= 1 \
         → `break 'steps`. Got {} records (>5 means pages 2+ were fetched — early-stop does \
         not cover CursorToken).",
        result.records.len()
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    // `truncated = true` is reserved exclusively for DI-019.
    assert!(
        !result.truncated,
        "OBS-1: early-stop must NOT set truncated=true (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );

    // TERTIARY: exactly 1 HTTP request (page 1 only; pages 2+ suppressed by early-stop).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "OBS-1 REGRESSION GUARD: exactly 1 HTTP request expected (early-stop after first \
         CursorToken page). Got {} requests — if >1, the early-stop check does not fire for \
         CursorToken pagination (e.g., was moved inside OffsetLimit arm).",
        received.len()
    );
}
