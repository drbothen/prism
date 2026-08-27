#![allow(
    non_snake_case,
    clippy::unwrap_used,
    unused_imports,
    dead_code,
    deprecated
)]
//! RG-005 and RG-006 for S-ENGINE-LIMIT-EARLY-STOP-001: prism-bin adapter layer.
//!
//! Verifies that `SpecDrivenSensorAdapter::fetch` maps `params.limit` to
//! `FetchContext.early_stop_limit`, causing the pipeline to stop at page
//! boundaries and issue only `ceil(limit / page_size)` HTTP requests.
//!
//! BC-2.16.002 postcondition — LIMIT-Aware Early-Stop (ADR-060 §D8);
//! BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006.
//!
//! | RG  | Test Name                                                              | Status        | Traces to |
//! |-----|------------------------------------------------------------------------|---------------|-----------|
//! | 005 | early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit   | GREEN (impl)  | AC-005    |
//! | 006 | early_stop_claroty_page_size_1000_limit_1_single_page                  | GREEN (impl)  | AC-005/6  |
//!
//! **Gate intent (RG-005 and RG-006):**
//! Absent the `params.limit → early_stop_limit` mapping in `SpecDrivenSensorAdapter::fetch`,
//! `FetchContext::new` receives `None` (the `early_stop_limit` field stub added at @9530f3478)
//! and the pagination loop runs to exhaustion, fetching ALL available pages.
//! Both tests gate that exactly 1 HTTP request is issued; without the mapping,
//! 3–4 requests are made instead.
//!
//! **AC-005 wiring (implemented):**
//! `params.limit > 0` maps to `Some(params.limit as usize)`; `params.limit == 0` maps to `None`.
//! With `early_stop_limit = Some(1)` and `page_size = P`, after the first
//! page `all_records.len() = P ≥ 1` → `break 'steps` → 1 HTTP request.

extern crate toml;

use std::sync::Arc;

use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
use prism_core::{OrgSlug, column::ColumnType};
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter,
    adapter::{QueryParams, SensorSpec as SensorAdapterSpec},
};
use prism_spec_engine::{
    ResolvedSensorSpec,
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::{AuthType, ColumnSpec, FetchStep, PaginationConfig, SensorSpec, TableSpec},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a minimal BearerStatic sensor spec with OffsetLimit pagination (GET method).
///
/// The table is named "items" with a single String column "id". The fetch step
/// is `GET /items` with `$.items` as the response path.
fn make_offset_limit_spec(sensor_id: &str, base_url: &str, page_size: u32) -> SensorSpec {
    SensorSpec::new(
        sensor_id,
        &format!("{sensor_id} early-stop integration test sensor"),
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

/// Build a Claroty-style BearerStatic sensor spec with POST-method OffsetLimit pagination.
///
/// Mirrors the real claroty `fetch_vulnerabilities` step (TV-BC-2.16.015-006 fidelity):
/// - Method: POST (OffsetLimit injects offset/limit into the POST JSON body)
/// - Path: /vulnerabilities
/// - Response path: $.vulnerabilities
///
/// Used by RG-006 to discharge TV-BC-2.16.015-006 with faithful POST wire shape.
fn make_claroty_vulnerabilities_post_spec(
    sensor_id: &str,
    base_url: &str,
    page_size: u32,
) -> SensorSpec {
    SensorSpec::new(
        sensor_id,
        &format!("{sensor_id} claroty-scale POST early-stop test sensor"),
        AuthType::BearerStatic,
        base_url,
        vec![TableSpec::new_point_in_time(
            "vulnerabilities",
            "vulnerability_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_vulnerabilities",
                "POST",
                "/vulnerabilities",
                None,
                "$.vulnerabilities",
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

/// Build a `ResolvedSensorSpec` from a `SensorSpec` and an org slug.
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only legitimate
/// external construction path for `ResolvedSensorSpec` (#[non_exhaustive]).
fn make_resolved(spec: SensorSpec, org_slug: &str) -> ResolvedSensorSpec {
    let overlay_toml = format!(
        "extends = \"{}\"\ninstance_id = \"{}@{}\"",
        spec.sensor_id, spec.sensor_id, org_slug
    );
    let overlay: SensorInstanceOverlay =
        toml::from_str(&overlay_toml).expect("test fixture: SensorInstanceOverlay TOML parse");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, OrgSlug::new(org_slug))
}

/// Build a `reqwest::Client` with the mandatory 30s timeout (ADR-050 / CLAUDE.md).
fn make_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("test fixture: reqwest Client build must succeed")
}

/// Build a `QueryParams` with the given limit and all other fields at defaults.
fn make_query_params(limit: u64) -> QueryParams {
    QueryParams {
        cursor: None,
        limit,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    }
}

/// Build a minimal `SensorAdapterSpec` for the given sensor.
///
/// `source_table = "items"` — does not match the sensor_id_ prefix pattern, so
/// `queried_table_name = None` and all tables in the sensor spec are executed
/// (defensive fallback in spec_driven_adapter.rs strip_prefix logic).
fn make_adapter_spec(sensor_id: &str) -> SensorAdapterSpec {
    let org_id = prism_core::OrgId::from_uuid(uuid::Uuid::now_v7());
    SensorAdapterSpec {
        source_table: "items".to_string(),
        org_id,
        #[allow(deprecated)]
        client_id: format!("{sensor_id}-test"),
        sensor_config: serde_json::json!({}),
    }
}

/// Build a page of `count` JSON records `{"id": "N"}` with string id values.
///
/// String values (not integers) avoid type coercion issues when the
/// column mapper builds an Arrow `StringArray` for the `id` column.
fn make_page_records(count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| serde_json::json!({ "id": i.to_string() }))
        .collect()
}

// ---------------------------------------------------------------------------
// RG-005 — adapter maps params.limit to early_stop_limit
// ---------------------------------------------------------------------------

/// RG-005: `SpecDrivenSensorAdapter::fetch` must map `params.limit=1` to
/// `FetchContext.early_stop_limit = Some(1)`, causing the pipeline to stop after
/// exactly 1 page (page_size=10, 2 data pages + 1 terminal page available).
///
/// **Mock topology (OffsetLimit GET, page_size=10):**
/// - Page 1 (`up_to_n_times(1)`): 10 records — OffsetLimit would normally continue.
/// - Page 2 (`up_to_n_times(1)`): 10 records — OffsetLimit would normally continue.
/// - Terminal page (fallback): 0 records — OffsetLimit terminates naturally.
///
/// **Gate intent (AC-005 wiring):** This test gates the AC-005 wiring: absent the
/// `params.limit → early_stop_limit` mapping in `SpecDrivenSensorAdapter::fetch`,
/// `FetchContext::new` would receive `None` and all pages would be fetched (3 requests);
/// with the wiring, exactly 1 request is issued.
///
/// **GREEN state (AC-005 wired):**
///   `let early_stop_limit = if params.limit == 0 { None } else { Some(params.limit as usize) };`
///   `let context = FetchContext::new(..., early_stop_limit);`
/// → `early_stop_limit = Some(1)`: after page 1 (10 records ≥ 1) → `break 'steps`.
/// → 1 HTTP request, fetch returns `Ok(batches)` → GREEN.
///
/// Also includes a **`params.limit=0 → None` regression sentinel** (Part B) that
/// passes BOTH before and after implementation: `limit=0` must always map to `None`
/// so full pagination is unchanged when no LIMIT clause is present.
///
/// Traces to AC-005 (BC-2.16.002 LIMIT-Aware Early-Stop; ADR-060 §D8).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit() {
    const PAGE_SIZE: u32 = 10;

    // -----------------------------------------------------------------------
    // Part A (gate intent): params.limit=1 → early_stop_limit=Some(1) → 1 request
    // -----------------------------------------------------------------------
    {
        let mock_server = MockServer::start().await;
        let page = make_page_records(PAGE_SIZE as usize);

        // Page 1: full page (PAGE_SIZE records) — served once.
        // Without early-stop: OffsetLimit continues. With early-stop: `break 'steps`.
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page })),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Page 2: full page — served once. Only reached without early-stop.
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page })),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Terminal page: empty — OffsetLimit terminates naturally on short page.
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })),
            )
            .mount(&mock_server)
            .await;

        let spec = make_offset_limit_spec("rg005a-sensor", &mock_server.uri(), PAGE_SIZE);
        let resolved = make_resolved(spec, "rg005a-org");
        let adapter = SpecDrivenSensorAdapter::new(
            Arc::new(resolved),
            AdapterAuthStrategy::BearerStatic,
            make_http_client(),
        );
        let adapter_spec = make_adapter_spec("rg005a-sensor");
        let params = make_query_params(1);
        let sensor_auth = BearerStaticSensorAuth::new("rg005-test-token");

        let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
        assert!(
            result.is_ok(),
            "RG-005A: fetch must return Ok when mock returns valid JSON. Got Err: {:?}",
            result.err()
        );

        let received = mock_server
            .received_requests()
            .await
            .expect("wiremock: received_requests() must succeed");

        // PRIMARY GATE: exactly 1 HTTP request expected.
        // Absent AC-005: None passed to FetchContext → 3 requests (2 data + 1 terminal).
        // With AC-005: Some(1) → stop after page 1 (PAGE_SIZE records ≥ 1) → 1 request.
        assert_eq!(
            received.len(),
            1,
            "RG-005 GATE: `params.limit=1` must produce `early_stop_limit=Some(1)` in \
             FetchContext, stopping the OffsetLimit loop after 1 page ({PAGE_SIZE} records ≥ 1 → \
             `break 'steps`). Absent the AC-005 wiring, None is passed to FetchContext and \
             all pages are fetched (3 requests). \
             Got {} requests. \
             AC-005 wiring: `let early_stop_limit = if params.limit == 0 {{ None }} \
             else {{ Some(params.limit as usize) }};` passed as the third arg to FetchContext::new. \
             AC-005; BC-2.16.002 LIMIT-Aware Early-Stop; ADR-060 §D8.",
            received.len()
        );
    }

    // -----------------------------------------------------------------------
    // Part B (sentinel): params.limit=0 → None → all pages fetched
    // -----------------------------------------------------------------------
    {
        let mock_server_0 = MockServer::start().await;
        let page = make_page_records(PAGE_SIZE as usize);

        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page })),
            )
            .up_to_n_times(1)
            .mount(&mock_server_0)
            .await;
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": page })),
            )
            .up_to_n_times(1)
            .mount(&mock_server_0)
            .await;
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })),
            )
            .mount(&mock_server_0)
            .await;

        let spec_0 = make_offset_limit_spec("rg005b-sensor", &mock_server_0.uri(), PAGE_SIZE);
        let resolved_0 = make_resolved(spec_0, "rg005b-org");
        let adapter_0 = SpecDrivenSensorAdapter::new(
            Arc::new(resolved_0),
            AdapterAuthStrategy::BearerStatic,
            make_http_client(),
        );
        let adapter_spec_0 = make_adapter_spec("rg005b-sensor");
        // params.limit = 0 means "no LIMIT clause" — must map to None (full pagination).
        let params_0 = make_query_params(0);
        let sensor_auth_0 = BearerStaticSensorAuth::new("rg005b-test-token");

        let result_0 = adapter_0
            .fetch(&adapter_spec_0, &params_0, &sensor_auth_0)
            .await;
        assert!(
            result_0.is_ok(),
            "RG-005B sentinel: fetch must return Ok for limit=0 path. Got Err: {:?}",
            result_0.err()
        );

        let received_0 = mock_server_0
            .received_requests()
            .await
            .expect("wiremock: received_requests() must succeed");

        // SENTINEL (passes BOTH before and after implementation):
        // params.limit=0 → early_stop_limit=None → full pagination → 3 requests
        // (2 data pages + 1 terminal).
        // If this fails after implementation, the None branch is broken.
        assert_eq!(
            received_0.len(),
            3,
            "RG-005 SENTINEL: `params.limit=0` must map to `early_stop_limit=None` (no LIMIT \
             clause), leaving full pagination unchanged. Expected 3 HTTP requests \
             (2 data pages × {PAGE_SIZE} records + 1 empty terminal page). Got {} requests. \
             If this sentinel fails after AC-005 implementation, the `params.limit == 0 → None` \
             branch is incorrect. ADR-060 §D8; BC-2.16.002 None-branch.",
            received_0.len()
        );
    }
}

// ---------------------------------------------------------------------------
// RG-006 — Claroty-scale: page_size=1000, 3 pages, limit=1 → 1 request
// ---------------------------------------------------------------------------

/// RG-006: Claroty-scale behavioral proof with faithful POST wire shape —
/// page_size=1000, 3 data pages of 1000 records each, `params.limit=1` → exactly
/// 1 HTTP POST request issued.
///
/// This is the direct test vector for **BC-2.16.015 EC-016-015-007 /
/// TV-BC-2.16.015-006** ("LIMIT 1 against claroty_vulnerabilities: 1 POST page fetched,
/// truncated=false; DataFusion trims to 1 row downstream").
///
/// Wire shape matches the real claroty `fetch_vulnerabilities` step:
/// - Method: POST (OffsetLimit injects offset/limit into the POST JSON body)
/// - Path: /vulnerabilities
/// - Response envelope: `{"vulnerabilities": [...]}`
///
/// **Mock topology (OffsetLimit POST, page_size=1000):**
/// - Pages 1–3 (`up_to_n_times(1)` each): 1000 records — OffsetLimit continues.
/// - Terminal page (fallback): 0 records — OffsetLimit terminates.
///
/// **Gate intent (AC-005 wiring — Claroty POST scale):** Absent the
/// `params.limit → early_stop_limit` mapping in `SpecDrivenSensorAdapter::fetch`,
/// `FetchContext::new` would receive `None`; the OffsetLimit POST loop would make
/// 4 requests instead of 1. With the wiring, exactly 1 POST request is issued.
///
/// **GREEN state (AC-005 wired):**
/// → `early_stop_limit = Some(1)`: after page 1 (1000 records ≥ 1) → `break 'steps`.
/// → 1 HTTP POST request; fetch returns `Ok(batches)` with 1000 pre-trim records.
/// → DataFusion applies `LIMIT 1` downstream (not exercised here).
///
/// Traces to AC-005 (BC-2.16.002), BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006.
/// F-R5-OBS-001 fix: GET → POST wire fidelity (2026-08-26).
#[tokio::test]
async fn test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page() {
    const PAGE_SIZE: u32 = 1000;

    let mock_server = MockServer::start().await;
    let page = make_page_records(PAGE_SIZE as usize);

    // Page 1 (1000 records — OffsetLimit continues without early-stop):
    Mock::given(method("POST"))
        .and(path("/vulnerabilities"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "vulnerabilities": page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2 (1000 records):
    Mock::given(method("POST"))
        .and(path("/vulnerabilities"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "vulnerabilities": page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3 (1000 records):
    Mock::given(method("POST"))
        .and(path("/vulnerabilities"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "vulnerabilities": page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Terminal page (0 records — OffsetLimit terminates on short page):
    Mock::given(method("POST"))
        .and(path("/vulnerabilities"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "vulnerabilities": [] })),
        )
        .mount(&mock_server)
        .await;

    // Use claroty POST spec mirroring real fetch_vulnerabilities step (TV-BC-2.16.015-006).
    let spec =
        make_claroty_vulnerabilities_post_spec("rg006-sensor", &mock_server.uri(), PAGE_SIZE);
    let resolved = make_resolved(spec, "rg006-org");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        make_http_client(),
    );
    let adapter_spec = make_adapter_spec("rg006-sensor");
    // params.limit=1: maps to early_stop_limit=Some(1) after AC-005.
    let params = make_query_params(1);
    let sensor_auth = BearerStaticSensorAuth::new("rg006-test-token");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "RG-006: fetch must return Ok when mock returns valid JSON. Got Err: {:?}",
        result.err()
    );

    let all_received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");

    // Filter to POST requests only for the explicit TV-BC-2.16.015-006 wire assertion.
    let post_received: Vec<_> = all_received
        .iter()
        .filter(|r| r.method == wiremock::http::Method::POST)
        .collect();

    // PRIMARY GATE: exactly 1 HTTP POST request expected.
    // Absent AC-005: None passed to FetchContext → 4 POST requests (3 × 1000 records + terminal).
    // With AC-005: Some(1) → after page 1 (1000 ≥ 1) → break → 1 POST request.
    assert_eq!(
        post_received.len(),
        1,
        "RG-006 GATE — BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006: \
         `LIMIT 1` against a claroty-scale sensor (POST /vulnerabilities, page_size=1000, \
         3 pages available) must issue exactly 1 HTTP POST request. \
         After page 1: all_records.len()=1000 ≥ 1 → `break 'steps` (ADR-060 §D8.2). \
         Absent the AC-005 wiring, None is passed to FetchContext and 4 POST requests \
         are issued (3 data + 1 terminal). \
         Got {} POST requests (total requests: {}). \
         AC-005 wiring: adapter maps params.limit > 0 to Some(params.limit as usize) \
         in spec_driven_adapter.rs (ADR-060 §D8). \
         DataFusion trims to 1 row downstream (not asserted here — pipeline returns 1000 pre-trim). \
         BC-2.16.002 LIMIT-Aware Early-Stop; BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006.",
        post_received.len(),
        all_received.len()
    );
}
