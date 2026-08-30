#![allow(non_snake_case)]
//! RG-001..RG-004 + EC-002/EC-003/LOW-1 for S-ENGINE-LIMIT-EARLY-STOP-001: LIMIT-aware early-stop pagination.
//! BC-2.16.002 postconditions — LIMIT-Aware Early-Stop Pagination (ADR-060 §D8).
//! OBS-1 regression guard: CursorToken early-stop arm (ADR-060 §D8.4).
//! Round-2 coverage: EC-002 (limit>total), EC-003 (limit==page_size boundary), LOW-1 (claroty-scale truncated=false).
//!
//! | RG    | Name                                                              | Status         | Traces to              |
//! |-------|-------------------------------------------------------------------|----------------|------------------------|
//! | 001   | early_stop_fetch_context_new_stores_early_stop_limit              | GREEN (stub)   | AC-001                 |
//! | 002   | early_stop_pipeline_stops_without_setting_truncated               | GREEN (impl)   | AC-002                 |
//! | 003   | early_stop_none_fetches_all_pages                                 | GREEN (sentry) | AC-003                 |
//! | 004   | early_stop_di019_fires_before_early_stop_check                    | GREEN (sentry) | AC-004                 |
//! | OBS-1 | early_stop_cursor_token_stops_after_first_page                    | GREEN (guard)  | ADR-060 §D8.4          |
//! | EC-002 | early_stop_limit_exceeds_total_fetches_all_pages                 | GREEN (cov)    | EC-002                 |
//! | EC-003 | early_stop_limit_equals_page_size_boundary                       | GREEN (cov)    | EC-003                 |
//! | LOW-1  | early_stop_large_page_size_truncated_false                       | GREEN (cov)    | TV-BC-2.16.015-006     |
//! | MULTI-PAGE | early_stop_multi_page_stops_after_second_page                | GREEN (cov)    | AC-003 ceil(N/page_size) |
//! | PSG-039 | early_stop_partial_final_page_not_early_stopped                  | GREEN (impl)   | AC-014 / BC-2.16.002 EC-01-041 |
//! | PSG-041 | cursor_token_partial_page_conservative_early_stopped             | GREEN (guard)  | ADR-060 §D8.4 conservative-cursor |
//! | PSG-042 | cursor_token_full_final_page_is_early_stopped                    | GREEN (guard)  | AC-014 / ADR-060 §D8.4         |
//! | PSG-043 | cursor_token_no_page_size_conservative                           | GREEN (guard)  | ADR-060 §D8.4 conservative     |
//!
//! RG-001 gates `FetchContext::new`: stub added @9530f3478 carries
//!   `early_stop_limit: Option<usize>` and the matching constructor parameter.
//!
//! RG-002 gates the early-stop `break 'steps` in `execute_impl` immediately after the
//!   DI-019 block. Absent that check, the pipeline fetches all pages.
//!
//! RG-003 gates the None-branch: `None` must leave full-pagination behaviour untouched.
//!   Regression sentinel (passes both before and after the early-stop check is wired).
//!
//! RG-004 gates DI-019 ordering: DI-019 must fire before the early-stop check,
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
// RG-001 — FetchContext constructor stores early_stop_limit field
// ---------------------------------------------------------------------------

/// RG-001: `FetchContext::new` stores `early_stop_limit` field correctly.
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
// RG-002 — pipeline stops after first complete page, truncated stays false
// ---------------------------------------------------------------------------

/// RG-002: pipeline stops after the first complete page when
/// `early_stop_limit = Some(1)` with `page_size = 10`.
///
/// **Mock setup** (OffsetLimit GET, page_size=10):
///   - Mocks 1–3 (`up_to_n_times(1)` each): return 10 records (full page → continue).
///   - Mock 4 (fallback): returns 0 records (short page → loop terminates naturally).
///
/// **Gate intent (AC-002 wiring):** Absent the early-stop check in
/// `PipelineExecutor::execute_impl`, the pipeline would fetch all pages
/// (4 requests, 30 records); this test gates that check.
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

    // PRIMARY GATE: records must be from the first page only (10 records).
    // Absent the early-stop check: all pages are fetched (3 full pages → 30 records);
    // this assertion is the red-gate discriminator.
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
// RG-003 (regression sentinel) — None fetches all available pages
// ---------------------------------------------------------------------------

/// RG-003 (regression sentinel): `early_stop_limit = None` fetches every page.
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
// RG-004 (DI-019 ordering sentinel)
// ---------------------------------------------------------------------------

/// RG-004 (ordering sentinel): DI-019 fires BEFORE the early-stop check.
///
/// A single page returns 10 001 records. `early_stop_limit = Some(5)` would stop
/// the loop very early if placed BEFORE DI-019, leaving `truncated = false`.
/// But per ADR-060 §D8.2 the early-stop check lives IMMEDIATELY AFTER the DI-019
/// block — DI-019 must fire first, set `truncated = true`, and `break 'steps`.
///
/// **DI-019 precedence:** DI-019 fires first → `truncated = true` (independent of early-stop).
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

// ---------------------------------------------------------------------------
// EC-002 (coverage) — early_stop_limit > total records: all pages fetched
// ---------------------------------------------------------------------------

/// EC-002 (coverage): `early_stop_limit = Some(1000)` with ~13 total records.
///
/// Early-stop never fires because `all_records.len()` never reaches 1000.
/// The pipeline must complete via normal OffsetLimit short-page termination.
///
/// **Mock setup** (OffsetLimit GET, page_size=5):
///   - Mock 1 (`up_to_n_times(1)`): 5 records (full page → continue).
///   - Mock 2 (`up_to_n_times(1)`): 5 records (full page → continue).
///   - Mock 3 (`up_to_n_times(1)`): 3 records (partial page < page_size → OffsetLimit
///     terminates naturally; or triggers an empty confirmation request, see fallback).
///   - Fallback: empty records (terminal confirmation page, if reached).
///
/// After page 1: 5 < 1000 → no early-stop.
/// After page 2: 10 < 1000 → no early-stop.
/// After page 3: 13 < 1000 → no early-stop; loop terminates (short or empty page).
///
/// Traces to EC-002 (BC-2.16.002 LIMIT-Aware Early-Stop edge case).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_limit_exceeds_total_fetches_all_pages() {
    let mock_server = MockServer::start().await;

    let full_page: Vec<serde_json::Value> =
        (0u32..5).map(|i| serde_json::json!({"id": i})).collect();

    // Page 1: 5 records (full page → OffsetLimit continues).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": full_page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 5 records (full page → OffsetLimit continues).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": full_page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3: 3 records (partial page < page_size=5 → OffsetLimit terminates naturally).
    let partial_page: Vec<serde_json::Value> =
        (10u32..13).map(|i| serde_json::json!({"id": i})).collect();
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": partial_page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Fallback: empty terminal page (defensive — handles implementations that require
    // an explicit empty response to confirm end-of-stream after a short page).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 5);
    let table = spec.tables[0].clone();
    // early_stop_limit = Some(1000): exceeds the ~13 total records; early-stop never fires.
    let context = FetchContext::new(OrgSlug::new("ec002-org"), HashMap::new(), Some(1000));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("EC-002: pipeline execute must return Ok");

    // PRIMARY: all 13 data records must be present; early-stop with limit=1000 must never fire.
    // If early-stop fires incorrectly at < 1000 records, fewer than 13 records are returned.
    assert_eq!(
        result.records.len(),
        13,
        "EC-002: early_stop_limit=Some(1000) with ~13 total records must fetch ALL pages. \
         After pages 1 (5), 2 (5), 3 (3): all_records.len()=13 < 1000 at every check. \
         Loop terminates normally via short/empty page, not via early-stop. \
         Got {} records.",
        result.records.len()
    );

    // SECONDARY: truncated must be false (no DI-019 cap hit; no early-stop fired).
    assert!(
        !result.truncated,
        "EC-002: truncated must be false when early_stop_limit=Some(1000) \
         and total records (13) is well below DI-019 cap."
    );

    // TERTIARY: at least 3 HTTP requests must be issued (pages 1, 2, 3 each fetched).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert!(
        received.len() >= 3,
        "EC-002: at least 3 HTTP requests expected (3 data pages). \
         Got {} requests.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// EC-003 (coverage) — early_stop_limit == page_size: exact page-boundary check
// ---------------------------------------------------------------------------

/// EC-003 (coverage): `early_stop_limit = Some(P)` where P equals the page_size.
///
/// After exactly one full page, `all_records.len() = P` and `P >= P` fires early-stop.
/// This exercises the N==page_size boundary (strictly-equal branch of the `>=` check).
///
/// **Mock setup** (OffsetLimit GET, page_size=5, early_stop_limit=Some(5)):
///   - Mock 1 (`up_to_n_times(1)`): 5 records (full page; after this, 5 >= 5 → early-stop).
///   - Fallback: 5 records (page 2; must NOT be fetched if early-stop fires correctly).
///
/// **With** correct implementation:
///   - Page 1 delivers 5 records; `all_records.len()=5 >= 5` → `break 'steps`.
///   - Exactly 1 HTTP request; 5 records; `truncated = false`.
///
/// **Without** early-stop (or with off-by-one `>`):
///   - Loop continues to page 2 → records > 5 → assertion fails.
///
/// Traces to EC-003 (BC-2.16.002 LIMIT-Aware Early-Stop edge case; ADR-060 §D8 boundary).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_limit_equals_page_size_boundary() {
    let mock_server = MockServer::start().await;

    let page_records: Vec<serde_json::Value> =
        (0u32..5).map(|i| serde_json::json!({"id": i})).collect();

    // Page 1: exactly page_size=5 records (full page → loop would continue without early-stop).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2 fallback: 5 records — reached ONLY if early-stop does not fire (off-by-one bug
    // using `>` instead of `>=`). If any request reaches this mock, early-stop is broken.
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 5);
    let table = spec.tables[0].clone();
    // early_stop_limit = Some(5) = Some(page_size): fires when all_records.len() >= 5.
    // After page 1: 5 >= 5 → break.  Tests the N==page_size boundary (not just N<page_size).
    let context = FetchContext::new(OrgSlug::new("ec003-org"), HashMap::new(), Some(5));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("EC-003: pipeline execute must return Ok");

    // PRIMARY: exactly page_size=5 records from page 1 only.
    // Off-by-one (`>` instead of `>=`): loop continues to page 2 → 10 records → FAILS.
    assert_eq!(
        result.records.len(),
        5,
        "EC-003 PAGE-BOUNDARY: early_stop_limit=Some(5) with page_size=5 must fire \
         after the FIRST page (all_records.len()=5 >= 5 → break). \
         Off-by-one using `>` would fetch page 2 and return 10 records. Got {} records.",
        result.records.len()
    );

    // SECONDARY: truncated must be false (early-stop does not set truncated, per ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "EC-003: early-stop must NOT set truncated=true (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );

    // TERTIARY: exactly 1 HTTP request (page 1 only; page 2 fallback must not be reached).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "EC-003 PAGE-BOUNDARY: exactly 1 HTTP request expected. \
         Got {} — if >1, early-stop did not fire at the N==page_size boundary.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// LOW-1 (coverage) — large page_size (claroty scale), truncated=false
// Discharges TV-BC-2.16.015-006: PipelineResult.truncated=false at page_size=1000.
// ---------------------------------------------------------------------------

/// LOW-1 (coverage): `page_size=1000` with `early_stop_limit=Some(1)`.
///
/// Claroty-scale test: page_size=1000 matches the Claroty sensor's real page budget.
/// This test directly asserts `PipelineResult.truncated == false` at the
/// `PipelineExecutor::execute` layer — the surface RG-006 (adapter-layer RecordBatch
/// assertion) cannot reach. Closes TV-BC-2.16.015-006.
///
/// **Mock setup** (OffsetLimit GET, page_size=1000):
///   - Mock 1 (`up_to_n_times(1)`): 1000 records (full page).
///   - Fallback: empty records (should never be hit; defensive only).
///
/// **With** `early_stop_limit = Some(1)`:
///   - Page 1 delivers 1000 records; `all_records.len()=1000 >= 1` fires early-stop.
///   - 1 HTTP request; `records.len() = 1000` (one full pre-DataFusion-trim page);
///     `truncated = false` (early-stop never sets truncated, per ADR-060 §D8.3).
///
/// Traces to TV-BC-2.16.015-006 (BC-2.16.002 claroty-scale truncated=false assertion).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_large_page_size_truncated_false() {
    let mock_server = MockServer::start().await;

    // Page 1: 1000 records (claroty-scale full page; triggers early-stop after fetch).
    let page_records: Vec<serde_json::Value> =
        (0u32..1000).map(|i| serde_json::json!({"id": i})).collect();
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Fallback: empty (should never be reached; early-stop fires after page 1).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 1000);
    let table = spec.tables[0].clone();
    // early_stop_limit = Some(1): fires immediately after page 1 (1000 records >= 1).
    let context = FetchContext::new(OrgSlug::new("low1-claroty-org"), HashMap::new(), Some(1));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("LOW-1: pipeline execute must return Ok");

    // PRIMARY (TV-BC-2.16.015-006): truncated must be FALSE.
    // Early-stop at page_size=1000 must NOT set truncated=true — that flag is reserved
    // exclusively for DI-019 (ADR-060 §D8.3).  This is the surface RG-006 cannot cover.
    assert!(
        !result.truncated,
        "LOW-1 TV-BC-2.16.015-006: early-stop with page_size=1000 must NOT set truncated=true. \
         truncated=true is reserved for DI-019, not for early-stop. Got truncated=true."
    );

    // SECONDARY: 1000 records from the single fetched page (pre-DataFusion-trim).
    assert_eq!(
        result.records.len(),
        1000,
        "LOW-1: exactly 1000 records expected from the single fetched page. \
         early_stop_limit=Some(1) fires after page 1 (1000 >= 1 → break). \
         Got {} records.",
        result.records.len()
    );

    // TERTIARY: exactly 1 HTTP request (only page 1 fetched; early-stop prevents page 2).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "LOW-1: exactly 1 HTTP request expected (early-stop fires after page 1). \
         Got {} requests.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// MULTI-PAGE (coverage) — early-stop fires on page 2, not page 1.
// Closes AC-003 ceil(N/page_size) gap: proves the check fires at EVERY page boundary.
// Kills the first-iteration-only mutation of the early-stop check.
// ---------------------------------------------------------------------------

/// MULTI-PAGE (coverage): `early_stop_limit = Some(15)` with `page_size = 10`.
///
/// Page 1 delivers 10 records (10 < 15 → loop CONTINUES). Page 2 delivers 10 more
/// records (cumulative 20 >= 15 → `break 'steps`). Page 3 is never fetched.
///
/// A "first-iteration-only" mutation of the early-stop check (e.g., the check is placed
/// only at the start of the loop, before the first page request, or is guarded by a
/// `page_index == 0` condition) would cause one of two failure modes:
///   - Stops after page 1 → records.len()==10 ≠ 20 → PRIMARY fails.
///   - Never fires after page 1 → fetches all 3 pages → records.len()==30 ≠ 20 → PRIMARY fails.
/// Either way, the mutant is killed.
///
/// **Mock setup** (OffsetLimit GET, page_size=10, early_stop_limit=Some(15)):
///   - Mock 1 (`up_to_n_times(1)`): 10 records (page 1; full page → 10 < 15 → CONTINUE).
///   - Mock 2 (`up_to_n_times(1)`): 10 records (page 2; full page → 20 >= 15 → BREAK).
///   - Mock 3 (`up_to_n_times(1)`): 10 records (page 3; must NOT be fetched).
///   - Fallback: empty records (terminal; must NOT be reached).
///
/// **Expected outcome**:
///   - `received_requests == 2` (page 3 never fetched).
///   - `records.len() == 20` (pages 1 + 2; no truncation from early-stop).
///   - `truncated == false` (early-stop does not set truncated, per ADR-060 §D8.3).
///
/// Traces to AC-003 `ceil(N/page_size)` promise (BC-2.16.002 LIMIT-Aware Early-Stop).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_multi_page_stops_after_second_page() {
    let mock_server = MockServer::start().await;

    let page_records: Vec<serde_json::Value> =
        (0u32..10).map(|i| serde_json::json!({"id": i})).collect();

    // Page 1: 10 records (full page; 10 < 15 → OffsetLimit loop CONTINUES).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 10 records (full page; cumulative 20 >= 15 → early-stop BREAKS).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3: 10 records — must NOT be fetched if early-stop fires correctly after page 2.
    // If fetched: records.len() becomes 30 → PRIMARY assertion (==20) fails → mutant killed.
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page_records })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Terminal fallback: empty page — must never be hit (early-stop stops at page 2).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    let spec = make_get_offset_spec(&mock_server.uri(), 10);
    let table = spec.tables[0].clone();
    // early_stop_limit = Some(15): page_size=10, so page 1 (10 records) < 15 → CONTINUE;
    // page 2 (cumulative 20 records) >= 15 → BREAK.  ceil(15/10) = 2 pages fetched.
    let context = FetchContext::new(OrgSlug::new("multi-page-org"), HashMap::new(), Some(15));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("MULTI-PAGE: pipeline execute must return Ok (no HTTP error expected)");

    // PRIMARY: exactly 20 records (pages 1+2 only; page 3 must NOT be fetched).
    // First-iteration-only mutation: stops after page 1 → records.len()==10 → FAILS here.
    // No-early-stop mutation: fetches all 3 pages → records.len()==30 → FAILS here.
    // Correct implementation: stops after page 2 → records.len()==20 → PASSES.
    assert_eq!(
        result.records.len(),
        20,
        "MULTI-PAGE: early_stop_limit=Some(15) with page_size=10 must stop after page 2. \
         Page 1: 10 records (10 < 15 → continue). Page 2: 20 cumulative (20 >= 15 → break). \
         Got {} records (10=first-iteration-only bug; 30=no-early-stop bug).",
        result.records.len()
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "MULTI-PAGE: early-stop must NOT set truncated=true (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );

    // TERTIARY: exactly 2 HTTP requests (pages 1 and 2; page 3 suppressed by early-stop).
    // This assertion also requires page 2 WAS fetched (received==1 fails equally).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        2,
        "MULTI-PAGE: exactly 2 HTTP requests expected — page 1 (10 < 15 → continue) + \
         page 2 (20 >= 15 → break). Page 3 must never be fetched. Got {} requests \
         (1=first-iteration-only bug; >=3=no-early-stop bug).",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-039 — AC-014 partial-final-page NOT early-stopped
// ---------------------------------------------------------------------------

/// RG-PSG-039 — AC-014 (BC-2.16.002 EC-01-041): pipeline-level unit verifying that a
/// partial final page (page_record_count < page_size) does NOT set `early_stopped = true`,
/// even when the cumulative record count first reaches the `early_stop_limit`.
///
/// ADR-060 §D8.2 discriminator:
///   `early_stopped = page_record_count >= page_size`
///   (FULL page → true; PARTIAL page → false)
///
/// ## Setup
///
/// - `early_stop_limit = Some(5)` — pipeline breaks when cumulative >= 5.
/// - Mock server: `page_size = 1000`. Single response: 5 records
///   (PARTIAL: `page_record_count=5 < page_size=1000`).
/// - No second page needed: after 5 records, cumulative (5) >= early_stop_limit (5) → break.
///
/// ## Why this is RED (pre-discriminator implementation)
///
/// Current code (pipeline.rs early-stop block):
/// ```text
/// if let Some(limit) = context.early_stop_limit && all_records.len() >= limit {
///     early_stopped = true;   // unconditional — wrong for partial pages
///     break 'steps;
/// }
/// ```
/// With 5 records and `early_stop_limit=Some(5)`:
///   `all_records.len() (5) >= limit (5) = true` → sets `early_stopped = true` → breaks.
///   Assertion `assert!(!result.early_stopped)` → **FAILS** → RED gate.
///
/// ## Why this is GREEN (post-discriminator, ADR-060 §D8.2)
///
/// After fix: `early_stopped = (page_record_count >= page_size) = (5 >= 1000) = false`.
///   Still breaks (cumulative >= limit), but sets `early_stopped = false`.
///   Assertion `assert!(!result.early_stopped)` → **PASSES**.
///
/// ## Traces
///
/// - BC-2.16.002 EC-01-041 (PARTIAL-final-page arm)
/// - AC-014 of S-ENGINE-LIMIT-EARLY-STOP-001
/// - ADR-060 §D8.2
#[tokio::test]
async fn test_BC_2_16_002_early_stop_partial_final_page_not_early_stopped() {
    let mock_server = MockServer::start().await;

    // 5 records: PARTIAL page (page_record_count=5 < page_size=1000).
    // Cumulative after page 1: 5 >= early_stop_limit(5) → pipeline BREAKS.
    // Discriminator: page_record_count(5) >= page_size(1000) = false → early_stopped = false.
    let partial_page: Vec<serde_json::Value> = (0u32..5)
        .map(|i| serde_json::json!({"id": i.to_string()}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": partial_page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Terminal fallback (defensive — must NOT be hit: break fires before requesting page 2).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    // page_size = 1000 (Claroty-scale); early_stop_limit = 5.
    let spec = make_get_offset_spec(&mock_server.uri(), 1000);
    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("rg039-org"), HashMap::new(), Some(5));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-PSG-039: pipeline execute must return Ok (no HTTP error expected)");

    // Precondition: exactly 5 records fetched (partial page).
    assert_eq!(
        result.records.len(),
        5,
        "RG-PSG-039 precondition: must fetch exactly 5 records (the partial page). \
         Got {} records.",
        result.records.len()
    );

    // Precondition: exactly 1 HTTP request (partial page triggers break; page 2 never hit).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "RG-PSG-039 precondition: exactly 1 HTTP request expected (break fires after \
         partial page 1). Got {} requests.",
        received.len()
    );

    // PRIMARY ASSERTION (RED gate driver — AC-014 / ADR-060 §D8.2 discriminator):
    //
    // RED  (pre-discriminator, current code):
    //   `early_stopped = true` (unconditional when `all_records.len() >= limit`)
    //   → assertion FAILS.
    //
    // GREEN (post-discriminator):
    //   `early_stopped = (page_record_count(5) >= page_size(1000)) = false`
    //   → assertion PASSES.
    assert!(
        !result.early_stopped,
        "RG-PSG-039 (AC-014 RED gate — ADR-060 §D8.2 discriminator): \
         `early_stopped` must be false for a PARTIAL final page \
         (page_record_count=5 < page_size=1000). \
         Current code sets `early_stopped = true` unconditionally when \
         `all_records.len() >= early_stop_limit`. \
         Fix: set `early_stopped = (page_record_count >= page_size)` in the early-stop \
         block (ADR-060 §D8.2). \
         Got early_stopped=true."
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "RG-PSG-039 secondary: `truncated` must be false (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-041 — CursorToken partial page: conservative early_stopped = true (RED gate)
// RG-PSG-042 — CursorToken full final page: early_stopped = true (GREEN guard)
// RG-PSG-043 — CursorToken page_size=None: conservative early_stopped = true (GREEN guard)
//
// ADR-060 §D8.4 (conservative rule): the page-fill discriminator applies to
// OffsetLimit ONLY. ALL CursorToken sub-cases (page_size=Some(N) or page_size=None)
// resolve to conservative `early_stopped = true`. Page-fill is NOT a valid cursor
// exhaustion signal; precise detection is deferred to S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001.
// ---------------------------------------------------------------------------

/// RG-PSG-041 (F-FP1-LENSA-001 sentinel) — ADR-060 §D8.4 conservative-cursor rule.
///
/// CursorToken with `page_size=Some(1000)`, `early_stop_limit=Some(5)`, mock returns
/// 5 records (partial page: 5 < 1000). Asserts `early_stopped == true`.
///
/// ADR-060 §D8.4 (conservative): the page-fill discriminator applies to OffsetLimit ONLY.
/// ALL CursorToken sub-cases resolve to conservative `early_stopped = true`. Page-fill
/// is NOT a valid cursor exhaustion signal; precise detection is deferred to
/// S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001.
///
/// ## Fix (landed — ADR-060 §D8.4 conservative rule)
///
/// All `CursorToken` variants map to `active_page_size = 0` (via `_ => 0` arm).
///   `active_page_size = 0`; `early_stopped = 5 >= 0 = true`
///   assertion `assert!(result.early_stopped)` PASSES.
///
/// ## Regression sentinel (F-FP1-LENSA-001)
///
/// Any reintroduction of `CursorToken { page_size: Some(ps), .. } => *ps` causes
/// active_page_size=1000 → 5>=1000=false → this test fails immediately, preventing
/// the unsound `early_stopped=false` cursor arm from being re-added.
///
/// Traces to AC-014 (S-ENGINE-LIMIT-EARLY-STOP-001), ADR-060 §D8.4.
#[tokio::test]
async fn test_cursor_token_partial_page_conservative_early_stopped() {
    let mock_server = MockServer::start().await;

    // 5 records: PARTIAL page (page_record_count=5 < page_size=1000).
    // next_cursor included — early-stop (not cursor exhaustion) causes the break.
    // Conservative rule: CursorToken always sets early_stopped=true regardless of page fill.
    let page_records: Vec<serde_json::Value> = (0u32..5)
        .map(|i| serde_json::json!({"id": i.to_string()}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/cursor-items-conservative"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_2"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2 fallback: must NOT be reached — early-stop fires after page 1.
    Mock::given(method("GET"))
        .and(path("/cursor-items-conservative"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_3"
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "rg041-conservative-sensor",
        "RG-PSG-041 CursorToken Conservative Early-Stop Sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "cursor_items_conservative",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_cursor_items_conservative",
                "GET",
                "/cursor-items-conservative",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next_cursor".to_string(),
                    page_size: Some(1000),
                }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    let table = spec.tables[0].clone();
    // early_stop_limit = Some(5): fires when accumulated >= 5; page_size = Some(1000).
    let context = FetchContext::new(
        OrgSlug::new("rg041-conservative-org"),
        HashMap::new(),
        Some(5),
    );
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-PSG-041: pipeline execute must return Ok (no HTTP error expected)");

    // PRECONDITION: exactly 5 records from the partial page.
    assert_eq!(
        result.records.len(),
        5,
        "RG-PSG-041 precondition: must fetch exactly 5 records (the partial page). \
         Got {} records.",
        result.records.len()
    );

    // PRECONDITION: exactly 1 HTTP request (early-stop fires; page 2 never hit).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "RG-PSG-041 precondition: exactly 1 HTTP request expected (early-stop fires \
         after partial page 1). Got {} requests.",
        received.len()
    );

    // PRIMARY ASSERTION (RED gate driver — ADR-060 §D8.4 conservative-cursor rule):
    //
    // RED  (current code, CursorToken{Some(ps)} => *ps arm present):
    //   active_page_size = 1000; early_stopped = 5 >= 1000 = false
    //   → assertion FAILS.
    //
    // GREEN (post-revert, CursorToken → _ => 0):
    //   active_page_size = 0; early_stopped = 5 >= 0 = true
    //   → assertion PASSES.
    //
    // Regression sentinel (F-FP1-LENSA-001): any reintroduction of the
    // CursorToken{Some(ps)} arm causes active_page_size=1000 → 5>=1000=false
    // → this test fails immediately.
    assert!(
        result.early_stopped,
        "RG-PSG-041 (ADR-060 §D8.4 conservative-cursor RED gate — F-FP1-LENSA-001 sentinel): \
         `early_stopped` must be TRUE for CursorToken regardless of page fill. \
         page_size=Some(1000), 5 records fetched (partial page: 5 < 1000). \
         Conservative rule: ALL CursorToken sub-cases set early_stopped=true (page-fill \
         is not a valid cursor exhaustion signal per ADR-060 §D8.4). \
         Current code (CursorToken{{Some(ps)}} arm): active_page_size=1000 → 5>=1000=false \
         → FAILS. Fix: revert CursorToken arm to _ => 0 → active_page_size=0 → \
         5>=0=true → PASSES."
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "RG-PSG-041 secondary: `truncated` must be false (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );
}

/// RG-PSG-042 — AC-014 / ADR-060 §D8.2/§D8.4: CursorToken with `page_size=Some(10)`.
///
/// A full final page (10 records == page_size=10) reached via the early-stop limit must
/// set `early_stopped = true` — the page was full, so more records may exist on the sensor.
///
/// ## Setup
///
/// - `page_size = Some(10)`, `early_stop_limit = Some(10)`.
/// - Page 1: 10 records (FULL: 10 == page_size=10) + `next_cursor: "cursor_page_2"`.
///   After page 1: accumulated(10) >= limit(10) → early-stop fires.
/// - Page 2 fallback: unreachable — early-stop fires before pagination advance.
///
/// ## Current behavior (conservative CursorToken, `_ => 0` arm)
///
/// Conservative rule: all CursorToken sub-cases fall through to `active_page_size = 0`.
/// `active_page_size=0 → early_stopped = (10 >= 0) = true` → assertion PASSES.
///
/// This test is included as a regression guard: the conservative `_ => 0` arm correctly
/// returns true for a full page, and this test confirms the arm does not accidentally
/// invert full-page behavior.
///
/// Traces to AC-014 (S-ENGINE-LIMIT-EARLY-STOP-001), ADR-060 §D8.2/§D8.4.
#[tokio::test]
async fn test_cursor_token_full_final_page_is_early_stopped() {
    let mock_server = MockServer::start().await;

    // Page 1: 10 records — FULL page (10 == page_size=10).
    // next_cursor included: proves early-stop (not cursor exhaustion) triggered the break.
    let page_records: Vec<serde_json::Value> = (0u32..10)
        .map(|i| serde_json::json!({"id": i.to_string()}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/cursor-items-full"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_2"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2 fallback: must NOT be reached — early-stop fires after page 1.
    Mock::given(method("GET"))
        .and(path("/cursor-items-full"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_3"
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "rg042-cursor-sensor",
        "RG-PSG-042 CursorToken Full Page Sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "cursor_items_full",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_cursor_items_full",
                "GET",
                "/cursor-items-full",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next_cursor".to_string(),
                    page_size: Some(10),
                }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    let table = spec.tables[0].clone();
    // early_stop_limit = Some(10): fires when accumulated >= 10; page_size = Some(10).
    let context = FetchContext::new(OrgSlug::new("rg042-org"), HashMap::new(), Some(10));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-PSG-042: pipeline execute must return Ok (no HTTP error expected)");

    // PRECONDITION: exactly 10 records from the full page.
    assert_eq!(
        result.records.len(),
        10,
        "RG-PSG-042 precondition: must fetch exactly 10 records (the full page). \
         Got {} records.",
        result.records.len()
    );

    // PRECONDITION: exactly 1 HTTP request (early-stop fires; page 2 never hit).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "RG-PSG-042 precondition: exactly 1 HTTP request expected (early-stop fires \
         after full page 1). Got {} requests.",
        received.len()
    );

    // PRIMARY ASSERTION: full page (page_record_count == page_size) must set early_stopped=true.
    // Conservative CursorToken rule (`_ => 0` arm): active_page_size=0 → 10 >= 0 = true.
    // This is the regression guard: the `_ => 0` arm must not accidentally invert full-page behavior.
    assert!(
        result.early_stopped,
        "RG-PSG-042 (regression guard — ADR-060 §D8.2/§D8.4): \
         `early_stopped` must be true for a FULL CursorToken page \
         (page_record_count=10, page_size=10). \
         Conservative CursorToken rule: active_page_size=0 → (10 >= 0) = true. \
         Got early_stopped=false — the conservative arm incorrectly returned false."
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "RG-PSG-042 secondary: `truncated` must be false (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );
}

/// RG-PSG-043 — ADR-060 §D8.4: CursorToken with `page_size=None` (conservative fallback).
///
/// When `page_size = None`, the discriminator cannot determine whether the page was
/// partial or full. The conservative path (`active_page_size = 0`, per ADR-060 §D8.4)
/// sets `early_stopped = true` unconditionally — preserving pre-fix semantics for sensors
/// that declare no page_size (e.g., legacy CursorToken configs, None-typed sensors).
///
/// ## Setup
///
/// - `page_size = None`, `early_stop_limit = Some(3)`.
/// - Page 1: 3 records + `next_cursor: "cursor_page_2"`.
///   After page 1: accumulated(3) >= limit(3) → early-stop fires.
/// - Page 2 fallback: unreachable.
///
/// ## Both pre-fix and post-fix
///
/// Pre-fix:  active_page_size=0 → early_stopped = (3 >= 0) = true → assertion PASSES.
/// Post-fix: _ => 0 (conservative) → active_page_size=0 → early_stopped = (3 >= 0) = true
///           → assertion PASSES.
///
/// This test ALREADY PASSES before the fix. It is the conservative-fallback regression
/// guard: the fix must preserve `early_stopped=true` for `page_size=None` sensors.
///
/// Traces to ADR-060 §D8.4 conservative-fallback arm.
#[tokio::test]
async fn test_cursor_token_no_page_size_conservative() {
    let mock_server = MockServer::start().await;

    // Page 1: 3 records + cursor (early-stop fires after this page).
    let page_records: Vec<serde_json::Value> = (0u32..3)
        .map(|i| serde_json::json!({"id": i.to_string()}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/cursor-items-none"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_2"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2 fallback: must NOT be reached — early-stop fires after page 1.
    Mock::given(method("GET"))
        .and(path("/cursor-items-none"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": page_records,
            "next_cursor": "cursor_page_3"
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "rg043-cursor-sensor",
        "RG-PSG-043 CursorToken No PageSize Sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "cursor_items_none",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_cursor_items_none",
                "GET",
                "/cursor-items-none",
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
    // early_stop_limit = Some(3): fires when accumulated >= 3; page_size = None.
    let context = FetchContext::new(OrgSlug::new("rg043-org"), HashMap::new(), Some(3));
    let http_client = make_http_client();
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("RG-PSG-043: pipeline execute must return Ok (no HTTP error expected)");

    // PRECONDITION: exactly 3 records from page 1.
    assert_eq!(
        result.records.len(),
        3,
        "RG-PSG-043 precondition: must fetch exactly 3 records. \
         Got {} records.",
        result.records.len()
    );

    // PRECONDITION: exactly 1 HTTP request.
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert_eq!(
        received.len(),
        1,
        "RG-PSG-043 precondition: exactly 1 HTTP request expected. Got {} requests.",
        received.len()
    );

    // PRIMARY ASSERTION: conservative fallback — page_size=None → active_page_size=0 →
    // early_stopped = (3 >= 0) = true.
    // This preserves pre-fix semantics for sensors without a declared page_size.
    // After the fix, the _ => 0 arm still covers CursorToken{page_size:None},
    // so this assertion must continue to pass.
    assert!(
        result.early_stopped,
        "RG-PSG-043 (conservative fallback — ADR-060 §D8.4): \
         `early_stopped` must be true when CursorToken has `page_size=None` \
         (active_page_size=0 → page_record_count(3) >= 0 = true). \
         The conservative fallback must be preserved after the fix. \
         Got early_stopped=false."
    );

    // SECONDARY: truncated must NOT be set by the early-stop path (ADR-060 §D8.3).
    assert!(
        !result.truncated,
        "RG-PSG-043 secondary: `truncated` must be false (reserved for DI-019 per \
         ADR-060 §D8.3). Got truncated=true."
    );
}
