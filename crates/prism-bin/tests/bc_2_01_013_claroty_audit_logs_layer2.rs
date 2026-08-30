// SPDX-License-Identifier: Apache-2.0
//! Red Gate failing tests for S-CLAROTY-AUDITLOG-TIMEBOX-001.
//!
//! ADR-033 Option T1 time-filter push-down tests for the Claroty xDome
//! `audit_logs` table, plus SAP-3 reachability (FIX-3) and FIX-1 bare-predicate
//! regression (BP-001) tests.
//!
//! # Red Gate test list
//!
//! | ID     | Test name | Primary assertion |
//! |--------|-----------|-------------------|
//! | RG-001 | test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal | body.get("filter_by").is_some() |
//! | RG-002 | test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated | body.get("filter_by").is_some() |
//! | RG-003 | test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and | body.get("filter_by").is_some() |
//! | RG-005 | test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001 | body.get("filter_by").is_some() |
//! | RG-006 | test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal | body.get("filter_by").is_some() |
//! | RG-007 | test_BC_2_01_013_claroty_audit_logs_timestamp_index_option_required_for_pushdown_eligibility | filter_by.value == explicit bound (SAP-3 pipeline) |
//! | BP-001 | test_BC_2_01_013_claroty_audit_logs_layer2_bare_predicate_source_table_claroty_injection_fires | body.get("filter_by").is_some() (FIX-1 regression) |
//!
//! RG-004 (pipeline.rs backward-compat JSON parsing) lives in
//! `crates/prism-spec-engine/src/pipeline.rs` as an in-module unit test.
//!
//! # Wire-shape discipline (CLAUDE.md §Conventions)
//!
//! Every test asserts on the serialized POST body — the actual JSON bytes
//! sent over the wire to the mock xDome server — not on pre-serialization
//! Rust structures. `mock_server.received_requests()` captures the
//! outbound HTTP request body for wire-level assertions.
//!
//! # SID-1 compliance (CLAUDE.md §SID-1)
//!
//! No `#[ignore]` used. HTTP boundary mocked via `wiremock 0.6`.
//! No live DTU clone or external service required.
//!
//! # Failure mechanism (before implementation)
//!
//! Production TOML: `body_template = '{}'`. After OffsetLimit merge the body
//! is `{"offset": 0, "limit": 1000}` — no `filter_by` field. The load-bearing
//! assertion `body.get("filter_by").is_some()` fails with an assertion panic
//! (not a build error, not a `todo!()` panic).
//!
//! BCs: BC-2.01.013, BC-2.16.013
//! Story: S-CLAROTY-AUDITLOG-TIMEBOX-001

#![allow(
    dead_code,
    unused_imports,
    non_snake_case,
    clippy::unwrap_used,
    clippy::expect_used
)]
extern crate toml;

use std::sync::Arc;

use std::collections::HashMap;

use datafusion::execution::context::SessionContext;
use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    materialization::{MaterializationContext, run_materialization_pipeline},
};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{
    AdapterRegistry, BearerStaticSensorAuth, CredentialResolver, SensorAdapter, SensorError,
    adapter::{QueryParams, SensorSpec as SensorAdapterSpec},
};
use prism_spec_engine::{
    ResolvedSensorSpec, ResolvedSpecKey,
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::SpecLoader,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only valid
/// external construction path for `#[non_exhaustive]` `ResolvedSensorSpec`.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("S-CLAROTY-AUDITLOG-TIMEBOX-001: claroty.sensor.toml must be readable from CARGO_MANIFEST_DIR/../prism-sensors/specs/");

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("S-CLAROTY-AUDITLOG-TIMEBOX-001: claroty.sensor.toml must parse cleanly");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml = "extends = \"claroty\"\ninstance_id = \"claroty@claroty-layer2-test-org\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("test fixture: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-layer2-test-org"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("test fixture: reqwest::Client construction failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

/// Build a `SensorAdapterSpec` targeting the Claroty audit_logs table.
///
/// `source_table = "claroty_audit_logs"` → the adapter strips the "claroty_"
/// prefix and routes to the `audit_logs` table in the claroty sensor spec.
fn make_audit_log_adapter_spec() -> SensorAdapterSpec {
    #[allow(deprecated)]
    SensorAdapterSpec {
        source_table: "claroty_audit_logs".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-layer2-test-org".to_string(),
        sensor_config: serde_json::json!({}),
    }
}

/// Minimal audit_log response for the wiremock mock server.
///
/// Returns 1 record (well below the page_size=1000 threshold), causing
/// OffsetLimit pagination to stop after the first page.
/// Shape: `{"audit_log": [...], "total": N}` — matches `response_path = "$.audit_log"`.
fn audit_log_response_one_record() -> serde_json::Value {
    serde_json::json!({
        "audit_log": [
            {
                "id": "audit-layer2-test-001",
                "action": "login",
                "user_display_name": "Layer2 Test User",
                "category": "authentication",
                "timestamp": "2026-08-01T12:00:00Z",
                "details": "Red Gate test login event",
                "username": "layer2-tester"
            }
        ],
        "total": 1
    })
}

/// Parse the POST body bytes from a received wiremock request as JSON.
///
/// Returns the parsed `serde_json::Value`. Panics with a descriptive message
/// if the body is not UTF-8 or not valid JSON.
fn parse_received_body(body_bytes: &[u8]) -> serde_json::Value {
    let body_str = std::str::from_utf8(body_bytes).expect("received POST body must be valid UTF-8");
    serde_json::from_str(body_str).unwrap_or_else(|e| {
        panic!(
            "received POST body must be valid JSON. Parse error: {e}. \
             Raw body (first 512 bytes): {:?}",
            &body_str[..body_str.len().min(512)]
        )
    })
}

// ---------------------------------------------------------------------------
// Helpers for run_materialization_pipeline tests (RG-007 FIX-3 rewrite)
// ---------------------------------------------------------------------------

/// `CredentialResolver` that returns `BearerStaticSensorAuth` for any sensor.
/// Used by `run_materialization_pipeline` → `fan_out` → `SpecDrivenSensorAdapter::fetch`
/// where the `BearerStatic` auth strategy downcasts the auth arg to `BearerStaticSensorAuth`.
struct BearerStaticCredentialResolverForPipeline {
    token: String,
}

impl BearerStaticCredentialResolverForPipeline {
    fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl CredentialResolver for BearerStaticCredentialResolverForPipeline {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        Ok(Box::new(BearerStaticSensorAuth::new(self.token.clone())))
    }
}

/// Build a `ResolvedSensorSpec` from a parsed `SensorSpec` and `org_slug`.
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only valid external
/// construction path for `#[non_exhaustive]` `ResolvedSensorSpec`.
fn make_resolved_spec_for_pipeline(
    spec: prism_spec_engine::spec_parser::SensorSpec,
    org_slug: &str,
) -> ResolvedSensorSpec {
    let overlay_toml = format!(
        "extends = \"{}\"\ninstance_id = \"{}@{}\"",
        spec.sensor_id, spec.sensor_id, org_slug
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("make_resolved_spec_for_pipeline: SensorInstanceOverlay TOML parse failed");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, OrgSlug::new(org_slug))
}

/// Build an `OrgRegistry` with one org registered.
/// Returns `(registry, org_id, org_slug)`.
fn make_org_registry_for_pipeline(slug: &str) -> (prism_core::OrgRegistry, OrgId, OrgSlug) {
    let registry = prism_core::OrgRegistry::new();
    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new(slug);
    registry
        .register(org_slug.clone(), org_id)
        .expect("make_org_registry_for_pipeline: register failed");
    (registry, org_id, org_slug)
}

// ---------------------------------------------------------------------------
// RG-001: BC-2.01.013 §Postcondition 1 — default 7-day look-back
//
// When `QueryParams.start_time = None` and `QueryParams.end_time = None`,
// the outbound POST body MUST contain a `filter_by` with
// `operation = "greater_or_equal"` and `value ≈ now() - 7 days`.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// POST body is missing the default look-back filter.
//
// After implementation: body = {"filter_by": {"field": "timestamp",
//   "operation": "greater_or_equal", "value": "<now-7d as ISO-8601 RFC3339>"},
//   "offset": 0, "limit": 1000}
// BC-2.01.013 EC-01-030: value MUST be an ISO-8601 STRING, NOT an epoch-ms integer.
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.01.013 §Postcondition 1:
/// A Claroty `audit_logs` fetch with NO time filter produces a POST body
/// carrying `filter_by.operation = "greater_or_equal"` and a value ≈ 7 days ago.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — the `body_template = '{}'` in
/// `claroty.sensor.toml` has no filter injection.
///
/// BCs: BC-2.01.013 §Postcondition 1; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001 / RG-001.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // No time filter → MUST inject 7-day default look-back.
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg001-token");

    // Run the fetch. With the current implementation (body_template = '{}'),
    // the POST body will be {"offset": 0, "limit": 1000} — no filter_by.
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Verify the mock received the outbound POST request.
    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-001: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get. Got no requests. \
         Check that source_table=\"claroty_audit_logs\" routes to the audit_logs \
         table in claroty.sensor.toml."
    );

    // Wire-shape assertion: parse the outbound POST body.
    // With the current body_template = '{}', the OffsetLimit merge produces
    // {"offset": 0, "limit": 1000} — valid JSON, but no filter_by.
    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-001):
    // The POST body MUST contain a 'filter_by' key when no time params provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-001 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.start_time is None (7-day default look-back). \
         Got body: {}. \
         Root cause: spec_driven_adapter.rs does not inject _claroty_audit_filter_by \
         into FetchContext.query_filters; claroty.sensor.toml body_template = '{{}}' \
         has no filter slot; pipeline.rs step_vars seeding cannot see the filter. \
         Fix: (1) spec_driven_adapter.rs: when sensor_id == \"claroty\" (sensor-scoped, \
         NOT table-scoped — BP-001 pins this; see bare-predicate fan-out path), inject \
         query_filters[\"_claroty_audit_filter_by\"] = {{\"field\": \"timestamp\", \
         \"operation\": \"greater_or_equal\", \"value\": \"<now_minus_7d as ISO-8601 RFC3339 string>\"}}; \
         (2) claroty.sensor.toml: update body_template to include \
         ${{query.filter._claroty_audit_filter_by}}; \
         (3) pipeline.rs: parse JSON-string query_filters to Value::Object \
         (BC-2.16.013). S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        body
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "greater_or_equal",
        "RG-001: filter_by.operation must be 'greater_or_equal' for a single lower-bound \
         time filter. Got: {:?}. BC-2.01.013 §Postcondition 1.",
        filter_by["operation"]
    );
    assert_eq!(
        filter_by["field"].as_str().unwrap_or(""),
        "timestamp",
        "RG-001: filter_by.field must be 'timestamp' (the xDome audit_log timestamp field). \
         Got: {:?}. BC-2.01.013 §Postcondition 1.",
        filter_by["field"]
    );

    // Value must be an ISO-8601 string approximately 7 days ago (±60 seconds tolerance).
    // BC-2.01.013 EC-01-030: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-001 ISO-8601 assertion: filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String), NOT an epoch-millisecond integer. Got: {:?}. \
         BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-001 ISO-8601 assertion: filter_by.value must be a parseable RFC3339/ISO-8601 \
         string. Got: {:?}. Parse error: {:?}. BC-2.01.013 EC-01-030.",
        value_str.unwrap(),
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let now_secs = chrono::Utc::now().timestamp();
    let seven_days_secs: i64 = 7 * 24 * 3600;
    let expected_secs = now_secs - seven_days_secs;
    let tolerance_secs: i64 = 60; // 60-second window for test execution time
    assert!(
        value_secs >= expected_secs - tolerance_secs
            && value_secs <= expected_secs + tolerance_secs,
        "RG-001: filter_by.value must be ≈ now - 7 days (604,800 seconds). \
         Expected range [{}, {}] (seconds), got {} (seconds). Delta: {} s. \
         BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        expected_secs - tolerance_secs,
        expected_secs + tolerance_secs,
        value_secs,
        value_secs - expected_secs
    );
}

// ---------------------------------------------------------------------------
// RG-002: BC-2.01.013 §Postcondition 2 — explicit start_time not truncated
//
// When `QueryParams.start_time = Some(t)` where t is ~45 days ago, the
// outbound POST body MUST use t as the lower bound — NOT the 7-day default.
// Proves the implementation does not silently cap explicit look-backs.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-002 / BC-2.01.013 §Postcondition 2:
/// An explicit `start_time` of ~45 days ago is honored as-is in the POST body
/// — the filter uses the explicit timestamp, NOT the 7-day default look-back.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — no time filter of any kind.
///
/// BCs: BC-2.01.013 §Postcondition 2; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002 / RG-002.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // Explicit start_time ≈ 45 days ago. Well outside the 7-day default window.
    // After implementation, the POST body must use this exact timestamp,
    // not the 7-day fallback — proving no silent truncation.
    let explicit_start = "2026-07-01T00:00:00Z"; // ~45 days before 2026-08-15
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(explicit_start.to_string()),
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg002-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-002: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-002):
    // The POST body MUST contain a 'filter_by' using the EXPLICIT 45-day start_time.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-002 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.start_time = Some(\"{explicit_start}\"). \
         Got body: {body}. \
         Root cause: same as RG-001 — spec_driven_adapter.rs does not inject \
         _claroty_audit_filter_by into FetchContext.query_filters. \
         Fix: inject the EXPLICIT start_time as the filter value, NOT the 7-day default. \
         BC-2.01.013 §Postcondition 2; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002."
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "greater_or_equal",
        "RG-002: filter_by.operation must be 'greater_or_equal' for a single lower-bound. \
         Got: {:?}.",
        filter_by["operation"]
    );

    // Value must be an ISO-8601 string equal to the explicit start_time.
    // BC-2.01.013 EC-01-031: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-002 ISO-8601 assertion: filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String), NOT an epoch-millisecond integer. Got: {:?}. \
         BC-2.01.013 EC-01-031; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-002: filter_by.value must parse as RFC3339/ISO-8601. Got: {:?}. \
         BC-2.01.013 EC-01-031.",
        value_str
    );

    // Verify the parsed timestamp matches the explicit start_time.
    let explicit_dt = chrono::DateTime::parse_from_rfc3339(explicit_start)
        .expect("test fixture: explicit_start must be a valid RFC3339 datetime");
    let value_secs = value_dt.unwrap().timestamp();
    let explicit_secs = explicit_dt.timestamp();
    assert!(
        (value_secs - explicit_secs).abs() <= 60,
        "RG-002 no-truncation assertion: filter_by.value must equal the EXPLICIT \
         start_time ({explicit_start} = {explicit_secs}s). Got {value_secs}s \
         (delta: {}s). The implementation MUST NOT substitute the 7-day fallback. \
         BC-2.01.013 EC-01-031.",
        value_secs - explicit_secs
    );

    // Confirm it is NOT the 7-day default (≈ now - 604,800 s).
    let seven_day_default_secs = chrono::Utc::now().timestamp() - 7_i64 * 24 * 3600;
    let delta_from_7day_secs = (value_secs - seven_day_default_secs).abs();
    assert!(
        delta_from_7day_secs > 30_i64 * 24 * 3600, // more than 30 days apart
        "RG-002 no-truncation: filter_by.value MUST NOT be the 7-day default. \
         Got {value_secs}s which is only {delta_from_7day_secs}s from the 7-day \
         default {seven_day_default_secs}s. BC-2.01.013 EC-01-031.",
    );
}

// ---------------------------------------------------------------------------
// RG-003: BC-2.01.013 §Postcondition 3 — both bounds → compound AND filter
//
// When both `start_time` and `end_time` are provided, the POST body MUST
// contain a compound `filter_by` with `operation = "and"` and two operands:
// `greater_or_equal` on start_time and `less_or_equal` on end_time.
// Compound key MUST be "operands" (NOT "conditions"). BC-2.01.013 EC-01-033.
// All value fields MUST be ISO-8601 strings, NOT epoch-ms integers.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.01.013 §Postcondition 3:
/// When both `start_time` and `end_time` are provided, the POST body carries
/// a compound `filter_by` with `operation = "and"` and compound key `"operands"`
/// (NOT `"conditions"`) containing two operands: `greater_or_equal` (start_time)
/// and `less_or_equal` (end_time). All value fields are ISO-8601 strings.
/// BC-2.01.013 EC-01-033.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — no filter at all.
///
/// BCs: BC-2.01.013 §Postcondition 3; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-003 / RG-003.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let start_time = "2026-07-01T00:00:00Z";
    let end_time = "2026-08-10T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(start_time.to_string()),
        end_time: Some(end_time.to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg003-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-003: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-003):
    // The POST body MUST contain 'filter_by' when both bounds are provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-003 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when both start_time and end_time are provided. \
         Got body: {body}. \
         Root cause: same as RG-001/002. Fix: build a compound AND filter \
         with greater_or_equal(start_time) and less_or_equal(end_time). \
         BC-2.01.013 §Postcondition 3; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-003."
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "RG-003: filter_by.operation must be 'and' when both start_time and end_time \
         are provided. Got: {:?}. BC-2.01.013 §Postcondition 3.",
        filter_by["operation"]
    );

    // The AND filter must have two operands. Key MUST be "operands" (NOT "conditions").
    // BC-2.01.013 EC-01-033: compound filter uses `operands`, NOT `conditions`.
    assert!(
        filter_by.get("operands").is_some(),
        "RG-003 LOAD-BEARING: compound filter MUST use key 'operands' (NOT 'conditions'). \
         filter_by keys: {:?}. BC-2.01.013 EC-01-033; AC-003.",
        filter_by.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert!(
        filter_by.get("conditions").is_none(),
        "RG-003: compound filter MUST NOT use key 'conditions' (wrong key — use 'operands'). \
         BC-2.01.013 EC-01-033; AC-003.",
    );
    let operands = filter_by["operands"].as_array().unwrap_or(&vec![]).clone();
    assert_eq!(
        operands.len(),
        2,
        "RG-003: filter_by.operands must have exactly 2 elements (start + end bound). \
         Got {} operands: {:?}. BC-2.01.013 EC-01-033.",
        operands.len(),
        operands
    );

    // One operand must be greater_or_equal (start_time lower bound).
    let has_gte = operands
        .iter()
        .any(|c| c["operation"].as_str() == Some("greater_or_equal"));
    assert!(
        has_gte,
        "RG-003: one operand must have operation = 'greater_or_equal' (lower bound). \
         Operands: {:?}. BC-2.01.013 EC-01-033.",
        operands
    );

    // One operand must be less_or_equal (end_time upper bound).
    let has_lte = operands
        .iter()
        .any(|c| c["operation"].as_str() == Some("less_or_equal"));
    assert!(
        has_lte,
        "RG-003: one operand must have operation = 'less_or_equal' (upper bound). \
         Operands: {:?}. BC-2.01.013 EC-01-033.",
        operands
    );

    // Validate the lower-bound value is an ISO-8601 string matching start_time.
    // BC-2.01.013 EC-01-033: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let gte_operand = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("greater_or_equal"))
        .expect("gte operand must exist");
    let gte_value_str = gte_operand["value"].as_str();
    assert!(
        gte_value_str.is_some(),
        "RG-003: greater_or_equal value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        gte_operand["value"]
    );
    let gte_dt = chrono::DateTime::parse_from_rfc3339(gte_value_str.unwrap())
        .expect("RG-003: gte operand value must parse as RFC3339");
    let start_secs = chrono::DateTime::parse_from_rfc3339(start_time)
        .expect("test fixture: start_time must be valid RFC3339")
        .timestamp();
    assert!(
        (gte_dt.timestamp() - start_secs).abs() <= 60,
        "RG-003: greater_or_equal value must match start_time ({start_time} = {start_secs}s). \
         Got {}s (delta: {}s). BC-2.01.013 EC-01-033.",
        gte_dt.timestamp(),
        gte_dt.timestamp() - start_secs
    );

    // Validate the upper-bound value is an ISO-8601 string matching end_time.
    let lte_operand = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("less_or_equal"))
        .expect("lte operand must exist");
    let lte_value_str = lte_operand["value"].as_str();
    assert!(
        lte_value_str.is_some(),
        "RG-003: less_or_equal value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        lte_operand["value"]
    );
    let lte_dt = chrono::DateTime::parse_from_rfc3339(lte_value_str.unwrap())
        .expect("RG-003: lte operand value must parse as RFC3339");
    let end_secs = chrono::DateTime::parse_from_rfc3339(end_time)
        .expect("test fixture: end_time must be valid RFC3339")
        .timestamp();
    assert!(
        (lte_dt.timestamp() - end_secs).abs() <= 60,
        "RG-003: less_or_equal value must match end_time ({end_time} = {end_secs}s). \
         Got {}s (delta: {}s). BC-2.01.013 EC-01-033.",
        lte_dt.timestamp(),
        lte_dt.timestamp() - end_secs
    );
}

// ---------------------------------------------------------------------------
// RG-005: BC-2.01.013 §Postcondition 4 — 4xx response surfaces E-SENSOR-001
//
// When xDome returns a 4xx HTTP status, the adapter MUST surface
// `SensorError::HttpError { status }`, not panic or return empty Vec.
//
// This test has a DUAL Red Gate:
// (1) PRIMARY FAIL: body.get("filter_by").is_some() — proves filter injection works
//     before we even get to check the error type
// (2) SECONDARY (post-impl): result is Err(SensorError::HttpError { status: 400 })
//
// CURRENT FAILURE (primary): body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-006 / BC-2.01.013 §Postcondition 4:
/// A 400 response from xDome causes `fetch()` to return
/// `Err(SensorError::HttpError { status: 400 })`.
///
/// # Dual Red Gate Failure
///
/// Primary (fires first): `body.get("filter_by").is_some()` FAILS — the POST body
/// has no `filter_by` field. This failure is intentional: it ensures the FILTER
/// INJECTION is implemented before the error-path behavior is checked.
///
/// Secondary (fires after primary passes): `result.is_err()` and error type check.
/// The 4xx error path already works in the current code; the primary assertion
/// gates on filter injection being present first.
///
/// BCs: BC-2.01.013 §Postcondition 4 (error surfacing); BC-2.16.013 (filter injection)
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-006 / RG-005.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001() {
    let mock_server = MockServer::start().await;

    // Mount a 400 Bad Request response. The mock captures the request body even
    // when it returns an error status — wiremock always records received requests.
    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string(r#"{"error": "Bad Request", "message": "invalid filter"}"#),
        )
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // No time filter — 7-day default should be injected
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg005-token");

    // Execute the fetch. Expected: Err due to 4xx response.
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // The mock captured the outbound POST even though it returned 400.
    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-005: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get before receiving the 400."
    );

    let body = parse_received_body(&requests[0].body);

    // PRIMARY LOAD-BEARING Red Gate assertion (RG-005):
    // The POST body MUST contain 'filter_by' — proves filter injection works
    // (the error path is tested AFTER filter injection is verified).
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-005 PRIMARY LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST \
         contain 'filter_by' even on paths that return 4xx. Got body: {body}. \
         Root cause: filter injection not yet implemented. \
         Fix: implement filter injection in spec_driven_adapter.rs first (see RG-001). \
         This assertion gates the error-type check below — both must pass. \
         BC-2.01.013 §Postcondition 4; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-006."
    );

    // SECONDARY assertion (runs after primary passes):
    // The 4xx MUST surface as SensorError::HttpError { status: 400 }.
    assert!(
        result.is_err(),
        "RG-005 SECONDARY: fetch() must return Err when xDome returns 400. \
         Got Ok. BC-2.01.013 §Postcondition 4."
    );

    let err = result.unwrap_err();
    match &err {
        SensorError::HttpError { status, .. } => {
            assert_eq!(
                *status, 400u16,
                "RG-005 SECONDARY: SensorError::HttpError status must be 400. \
                 Got status {}. BC-2.01.013 §Postcondition 4.",
                status
            );
        }
        other => {
            panic!(
                "RG-005 SECONDARY: fetch() must return SensorError::HttpError{{400}} \
                 for a 400 xDome response. Got: {other:?}. \
                 Check map_spec_engine_error_to_sensor_error in spec_driven_adapter.rs."
            );
        }
    }
    assert_eq!(
        err.error_code(),
        "E-SENSOR-001",
        "RG-005 SID-2: error code must be E-SENSOR-001. Got: {:?}",
        err.error_code()
    );
    let rendered = err.to_string();
    assert!(
        rendered.contains("invalid filter"),
        "RG-005 SID-2 / AC-006: rendered error MUST contain the response body snippet \
         ('invalid filter'). map_spec_engine_error_to_sensor_error must thread the \
         body through the HttpError{{body}} field. Got: {rendered}"
    );
    assert_eq!(
        rendered.matches("HTTP").count(),
        1,
        "RG-005 SID-2 / F-P37-HIGH-001 double-prefix regression guard: \
         rendered error must contain exactly 1 'HTTP' occurrence. \
         Got {} occurrences in: {rendered}",
        rendered.matches("HTTP").count()
    );
}

// ---------------------------------------------------------------------------
// RG-006: BC-2.01.013 EC-01-032 — end-only filter, NO synthetic lower bound
//
// AC-007: When end_time is Some and start_time is None, the POST body MUST
// contain a SINGLE `less_or_equal` at the supplied end. NO compound `and`
// filter, NO `greater_or_equal` synthetic lower bound.
//
// Uses end_time = "2026-01-01T00:00:00Z" (>7 months before 2026-08-15) to
// prove no 7-day floor is silently injected. Adding a 7-day floor when
// end_time < now-7d produces an inverted/empty window — SOUL.md §4.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by at all.
// ---------------------------------------------------------------------------

/// AC-007 / BC-2.01.013 EC-01-032:
/// When `start_time = None` and `end_time = Some(past_date_older_than_7d)`, the
/// POST body carries a SINGLE `less_or_equal` filter at `end_time`. No compound
/// `and` filter is produced. No synthetic 7-day lower bound is injected.
///
/// Uses `end_time = "2026-01-01T00:00:00Z"` (≈7.5 months ago) to prove that
/// the 7-day default floor is NOT applied when an explicit end_time is given.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because current body is
/// `{"offset": 0, "limit": 1000}` — no filter at all.
///
/// BCs: BC-2.01.013 EC-01-032; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007 / RG-006.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // end_time = 2026-01-01 (≈7.5 months ago from 2026-08-15), well past the 7-day default
    // window. This proves the implementation does NOT inject a synthetic now-7d lower bound.
    // If a 7-day floor were added, the window would be inverted (end_time < floor), yielding
    // an empty result set — a SOUL.md §4 silent-wrong-result violation.
    let end_time = "2026-01-01T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // Explicitly None — end-only filter case (EC-01-032).
        end_time: Some(end_time.to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg006-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-006: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-006):
    // The POST body MUST contain 'filter_by' when end_time is provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-006 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.end_time = Some(\"{end_time}\"). \
         Got body: {body}. \
         Root cause: spec_driven_adapter.rs does not inject _claroty_audit_filter_by for \
         the end-only case. BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007."
    );

    let filter_by = &body["filter_by"];

    // The filter MUST be a single less_or_equal — NOT compound.
    // A compound "operation": "and" would mean a synthetic lower bound was injected.
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "less_or_equal",
        "RG-006: end-only filter MUST be a single 'less_or_equal'. \
         Got operation: {:?}. \
         If 'and', the implementation added a synthetic lower bound — FORBIDDEN by \
         BC-2.01.013 EC-01-032 (produces inverted empty window when end < now-7d). \
         BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007.",
        filter_by["operation"]
    );

    // Confirm NO greater_or_equal anywhere in the filter object.
    // Serialize the full filter_by to catch nested occurrences.
    let filter_by_str = serde_json::to_string(filter_by).unwrap_or_default();
    assert!(
        !filter_by_str.contains("\"greater_or_equal\""),
        "RG-006: filter_by MUST NOT contain 'greater_or_equal' for the end-only case. \
         Got filter_by: {filter_by}. \
         BC-2.01.013 EC-01-032: end-only means single less_or_equal, NO synthetic floor. \
         Adding a 7-day lower bound silently produces an inverted/empty window when \
         end_time ({end_time}) < now-7d. SOUL.md §4 silent-wrong-result violation. \
         S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007."
    );

    // Field must be "timestamp".
    assert_eq!(
        filter_by["field"].as_str().unwrap_or(""),
        "timestamp",
        "RG-006: filter_by.field must be 'timestamp'. Got: {:?}. \
         BC-2.01.013 EC-01-032.",
        filter_by["field"]
    );

    // Value must be an ISO-8601 string equal to end_time.
    // BC-2.01.013 EC-01-032: value is an ISO-8601 STRING, NOT an epoch integer.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-006: filter_by.value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-032.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-006: filter_by.value must parse as RFC3339. Got: {:?}. Error: {:?}. \
         BC-2.01.013 EC-01-032.",
        value_str,
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let end_secs = chrono::DateTime::parse_from_rfc3339(end_time)
        .expect("test fixture: end_time must be valid RFC3339")
        .timestamp();
    assert!(
        (value_secs - end_secs).abs() <= 60,
        "RG-006: filter_by.value must match end_time ({end_time} = {end_secs}s). \
         Got {value_secs}s (delta: {}s). BC-2.01.013 EC-01-032; AC-007.",
        value_secs - end_secs
    );
}

// ---------------------------------------------------------------------------
// BP-001 (FIX-1 regression): injection fires on bare-predicate fan-out path
// (source_table = sensor_id = "claroty").
//
// When prism-query materialization.rs Step 3b fires (bare-predicate fan-out),
// source_table = sensor_id = "claroty" (NOT "claroty_audit_logs").
// Before FIX-1: guard `sensor_id == "claroty" && source_table == "claroty_audit_logs"` failed.
// After FIX-1: guard is `sensor_id == "claroty"` only → injection always fires.
//
// BCs: BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 BLOCKING-1
// ---------------------------------------------------------------------------

/// FIX-1 regression / BLOCKING-1 — proves that the Claroty injection guard fires
/// even when `source_table = "claroty"` (bare-predicate fan-out path).
///
/// `prism-query/src/materialization.rs` Step 3b sets `source_table = sensor_id = "claroty"`
/// on the bare-predicate fan-out path. Before FIX-1, the guard had an extra conjunct
/// `&& source_table == "claroty_audit_logs"` which silently FAILED on this path,
/// leaving `${query.filter._claroty_audit_filter_by}` unseeded → invalid JSON body.
///
/// After FIX-1: guard is `sensor_id == "claroty"` only. The `_claroty_audit_filter_by`
/// key is inert for tables whose body_template does not reference it (alerts, devices,
/// device_alert_relations) and expands only in the audit_logs body_template.
///
/// BCs: BC-2.01.013 EC-01-031; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001 / BLOCKING-1
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_bare_predicate_source_table_claroty_injection_fires()
 {
    let mock_server = MockServer::start().await;

    // Catch-all POST mock for all 4 Claroty table endpoints.
    // source_table = "claroty" → queried_table_name = None → all tables execute.
    // The response MUST include all four response_path keys ($.alerts, $.audit_log,
    // $.devices, $.devices_alerts) as empty arrays: extract_at_path returns Err for
    // missing keys (not empty-array), which would short-circuit the table loop via `?`.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "alerts": [],
            "audit_log": [],
            "devices": [],
            "devices_alerts": []
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // FIX-1 SCENARIO: source_table = "claroty" (bare-predicate fan-out path).
    // strip_prefix("claroty_") on "claroty" → None → queried_table_name = None → all tables execute.
    #[allow(deprecated)]
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-layer2-test-org".to_string(),
        sensor_config: serde_json::json!({}),
    };

    // Explicit start_time far in the past to distinguish from the 7-day default.
    let explicit_start = "2025-01-01T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(explicit_start.to_string()),
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-bp001-token");

    // Run fetch with source_table = "claroty".
    // Before FIX-1: guard fails → `_claroty_audit_filter_by` unseeded → body_template
    //   produces `{"filter_by": }` (invalid JSON) → fetch error.
    // After FIX-1: guard fires → filter_by injected → valid POST body → Ok result.
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "BP-001: bare-predicate fetch must succeed (valid JSON body after FIX-1). \
         Got Err: {:?}. \
         Root cause (before FIX-1): guard failed → _claroty_audit_filter_by unseeded \
         → body_template produced invalid JSON → fetch error.",
        result.err()
    );

    let requests = mock_server.received_requests().await.unwrap_or_default();

    // Find the audit_log POST request by URL path.
    let audit_log_req = requests
        .iter()
        .find(|r| r.url.path() == "/api/v1/audit_log/get");

    assert!(
        audit_log_req.is_some(),
        "BP-001 FIX-1 REGRESSION: SpecDrivenSensorAdapter::fetch with source_table='claroty' \
         must issue a POST to /api/v1/audit_log/get. \
         Got {} total requests with paths: {:?}. \
         Check that strip_prefix(\"claroty_\") on \"claroty\" returns None (run-all-tables path).",
        requests.len(),
        requests
            .iter()
            .map(|r| r.url.path().to_string())
            .collect::<Vec<_>>()
    );

    let body = parse_received_body(&audit_log_req.unwrap().body);

    // LOAD-BEARING FIX-1 assertion: body MUST contain 'filter_by' even when source_table="claroty".
    // Before FIX-1: guard `sensor_id == "claroty" && source_table == "claroty_audit_logs"` failed
    //   → body_template expansion of `${query.filter._claroty_audit_filter_by}` produced invalid JSON.
    // After FIX-1: guard `sensor_id == "claroty"` only → injection fires → filter_by present.
    assert!(
        body.get("filter_by").is_some(),
        "BP-001 FIX-1 REGRESSION LOAD-BEARING: POST body to /api/v1/audit_log/get MUST contain \
         'filter_by' when source_table='claroty' (bare-predicate fan-out path). \
         Got body: {body}. \
         Root cause (before FIX-1): guard `sensor_id == \"claroty\" && source_table == \"claroty_audit_logs\"` \
         failed when source_table=\"claroty\". \
         Fix: guard changed to `sensor_id == \"claroty\"` only. \
         BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 BLOCKING-1."
    );

    let filter_by = &body["filter_by"];
    let value_str = filter_by["value"].as_str().unwrap_or("");

    // Verify the EXPLICIT start_time was honored (not the 7-day default).
    let filter_dt = chrono::DateTime::parse_from_rfc3339(value_str)
        .expect("BP-001: filter_by.value must be parseable RFC3339");
    let explicit_secs = chrono::DateTime::parse_from_rfc3339(explicit_start)
        .expect("test fixture: explicit_start must be valid RFC3339")
        .timestamp();

    assert!(
        (filter_dt.timestamp() - explicit_secs).abs() <= 60,
        "BP-001 FIX-1: filter_by.value must equal the explicit start_time \
         ({explicit_start} = {explicit_secs}s). \
         Got '{value_str}' = {}s (delta: {}s). \
         The explicit bound from params.start_time must be honored on the bare-predicate path. \
         BC-2.01.013 EC-01-031.",
        filter_dt.timestamp(),
        filter_dt.timestamp() - explicit_secs
    );
}

// ---------------------------------------------------------------------------
// RG-007: AC-INDEX-CLARO-001 / BC-2.01.013 EC-01-034 (SAP-3 reachability)
//
// FIX-3 REWRITE: goes through run_materialization_pipeline → build_source_column_map
// → extract_time_window_from_ast → SpecDrivenSensorAdapter::fetch → wire POST body.
//
// Previous implementation manually called extract_time_window_from_ast with a
// hand-built col_map, bypassing build_source_column_map (SAP-3 violation).
//
// BCs: BC-2.01.013 EC-01-034; BC-2.16.013
// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007
// ---------------------------------------------------------------------------

/// AC-INDEX-CLARO-001 / BC-2.01.013 EC-01-034 — SAP-3 parser-surface reachability test (FIX-3).
///
/// Proves via `run_materialization_pipeline` that `audit_logs.timestamp` has
/// `options = ["INDEX"]` so an explicit `WHERE timestamp > '<older-than-7d>'` predicate
/// flows through the FULL production pipeline to the xDome POST body.
///
/// # Production call graph exercised (SAP-3 — full pipeline, not synthetic AST)
///
/// ```text
/// PrismQL: "SELECT * FROM claroty_audit_logs WHERE timestamp > '2025-01-01T00:00:00Z'"
///   → run_materialization_pipeline
///   → PrismQlParser::parse
///   → build_source_column_map [reads resolved_spec_map; audit_logs.timestamp options=["INDEX"]]
///   → extract_time_window_from_ast → (Some("2025-01-01T00:00:00+00:00"), None)
///   → fan_out → SpecDrivenSensorAdapter::fetch (source_table="claroty_audit_logs")
///   → build_claroty_audit_filter_by(Some("2025-01-01..."), None)
///   → POST /api/v1/audit_log/get → wiremock captures body
/// ```
///
/// # LOAD-BEARING
///
/// `filter_by.value` MUST equal the EXPLICIT user-supplied bound (2025-01-01T00:00:00Z),
/// NOT the 7-day default. Proves: (1) INDEX option present; (2) FIX-1 guard fires;
/// (3) explicit WHERE bound flows end-to-end to the xDome POST body.
///
/// BCs: BC-2.01.013 EC-01-034; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007 (FIX-3 rewrite)
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_timestamp_index_option_required_for_pushdown_eligibility()
 {
    // SAP-3 entry point: run_materialization_pipeline, NOT direct extract_time_window_from_ast.
    // This exercises build_source_column_map which reads resolved_spec_map — bypassed in
    // the previous implementation.

    // Step 1: Start wiremock for the audit_log endpoint.
    // SELECT * FROM claroty_audit_logs only queries audit_logs because
    // queried_table_name = strip_prefix("claroty_", "claroty_audit_logs") = Some("audit_logs").
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    // Step 2: Load production claroty.sensor.toml directed at wiremock.
    // Parse twice: once for resolved_spec_map (INDEX column lookup), once for the adapter.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "RG-007: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let org_slug = "claroty-rg007-pipeline-org";

    let mut spec_for_map = SpecLoader::parse(&spec_content)
        .expect("RG-007: claroty.sensor.toml must parse cleanly (spec_for_map)");
    spec_for_map.base_url = mock_server.uri();
    let resolved_for_map = make_resolved_spec_for_pipeline(spec_for_map, org_slug);

    let mut spec_for_adapter = SpecLoader::parse(&spec_content)
        .expect("RG-007: claroty.sensor.toml must parse cleanly (spec_for_adapter)");
    spec_for_adapter.base_url = mock_server.uri();
    let resolved_for_adapter = make_resolved_spec_for_pipeline(spec_for_adapter, org_slug);

    // Step 3: Build resolved_spec_map so build_source_column_map can find INDEX columns.
    // Without this, extract_time_window_from_ast cannot see audit_logs.timestamp options=["INDEX"].
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert(
        (OrgSlug::new(org_slug), SensorId::from("claroty")),
        resolved_for_map,
    );
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    // Step 4: Build OrgRegistry + AdapterRegistry.
    let (org_registry, org_id, org_slug_typed) = make_org_registry_for_pipeline(org_slug);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("RG-007: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved_for_adapter),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext with resolved_spec_map.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolverForPipeline::new("claroty-rg007-token"),
    );
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    let session_ctx = SessionContext::new();

    // Step 6: Run FULL production pipeline from a PrismQL string (SAP-3 entry point).
    // 2025-01-01T00:00:00Z is ~7.5 months before 2026-08-16, well outside the 7-day default.
    // If the explicit bound is ignored, the LOAD-BEARING assertion FAILS.
    // ADR-058 §G (ocsf_column_naming=true): audit_logs.timestamp ocsf_field="time" →
    // Arrow field is "time". LLM must query WHERE time > ..., not WHERE timestamp > ...
    // The pushdown (extract_time_window_from_ast) recognizes "time" as INDEX-eligible
    // because RG-PD-001 inserts both col.name and ocsf_field_to_arrow_name into datetime_index_cols.
    let pql_query = "SELECT * FROM claroty_audit_logs WHERE time > '2025-01-01T00:00:00Z'";

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed]),
        limit: None,
        ..QueryOptions::default()
    };

    // Pipeline path:
    //   PrismQlParser::parse → build_source_column_map [resolved_spec_map, INDEX column]
    //   → extract_time_window_from_ast(start=Some("2025-01-01T00:00:00+00:00"), end=None)
    //   → fan_out → SpecDrivenSensorAdapter::fetch
    //   → build_claroty_audit_filter_by(Some("2025-01-01..."), None) → explicit bound in body
    let output =
        run_materialization_pipeline(pql_query, &options, &mut mat_ctx, &session_ctx).await;
    // PRIMARY: pipeline must succeed (not just issue the correct POST body).
    // D-1715/D-1716 observed-output discipline: no-crash is not sufficient;
    // assert actual successful output.
    assert!(
        output.is_ok(),
        "RG-007 (SAP-3 / D-1715 observed-output): run_materialization_pipeline must return Ok. \
         Got: {:?}",
        output.err()
    );
    // Row count: mock returns exactly 1 record (audit_log_response_one_record).
    // The pipeline must surface at least 1 row to the MCP agent.
    let output_ref = output.as_ref().unwrap();
    let total_rows: usize = output_ref.batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "RG-007 (SAP-3 / D-1715 observed-output): pipeline must return at least 1 row. \
         Mock returns 1 record (audit_log_response_one_record). Got 0 rows."
    );

    // Step 7: Wire-level assertion on the POST body captured by wiremock.
    let requests = mock_server.received_requests().await.unwrap_or_default();

    let audit_log_req = requests.iter().find(|r| r.url.path().contains("audit_log"));

    assert!(
        audit_log_req.is_some(),
        "RG-007 (SAP-3): run_materialization_pipeline for 'SELECT * FROM claroty_audit_logs WHERE ...' \
         must issue a POST to /api/v1/audit_log/get. \
         Got {} total requests: {:?}.",
        requests.len(),
        requests
            .iter()
            .map(|r| r.url.path().to_string())
            .collect::<Vec<_>>()
    );

    let body = parse_received_body(&audit_log_req.unwrap().body);

    assert!(
        body.get("filter_by").is_some(),
        "RG-007 (SAP-3) PREREQUISITE: POST body to /api/v1/audit_log/get must contain \
         'filter_by'. Got body: {body}. Check FIX-1 guard in spec_driven_adapter.rs."
    );

    let filter_by = &body["filter_by"];
    let value_str = filter_by["value"].as_str().unwrap_or("");

    let filter_dt_result = chrono::DateTime::parse_from_rfc3339(value_str);
    assert!(
        filter_dt_result.is_ok(),
        "RG-007 (SAP-3): filter_by.value must be RFC3339. Got: {:?}. filter_by: {}.",
        value_str,
        filter_by
    );

    let filter_secs = filter_dt_result.unwrap().timestamp();
    let explicit_bound_secs = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
        .expect("test fixture explicit bound must be valid RFC3339")
        .timestamp();
    let seven_day_default_secs = chrono::Utc::now().timestamp() - 7_i64 * 24 * 3600;

    // LOAD-BEARING (SAP-3 + full pipeline): filter_by.value MUST equal the EXPLICIT user-supplied bound.
    // Proves the full production path:
    //   build_source_column_map reads audit_logs.timestamp options=["INDEX"]
    //   → extract_time_window_from_ast extracts start_time = Some("2025-01-01T00:00:00+00:00")
    //   → build_claroty_audit_filter_by returns explicit bound (not 7-day default)
    assert!(
        (filter_secs - explicit_bound_secs).abs() <= 60,
        "RG-007 (SAP-3) LOAD-BEARING: filter_by.value MUST equal the EXPLICIT user-supplied \
         bound (2025-01-01T00:00:00Z = {explicit_bound_secs}s), NOT the 7-day default \
         (~{seven_day_default_secs}s).\n\
         Got filter_by.value = '{value_str}' = {filter_secs}s (delta from explicit: {}s).\n\
         \n\
         Root causes:\n\
         (1) audit_logs.timestamp lacks options=[\"INDEX\"] → extract_time_window_from_ast \
             returns (None, None) → 7-day default injected.\n\
         (2) FIX-1 guard not applied → injection never fires.\n\
         \n\
         BC-2.01.013 EC-01-034; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007.",
        filter_secs - explicit_bound_secs
    );
}

// ---------------------------------------------------------------------------
// Parser-surface coverage for EC-01-033 compound AND filter (LOW-004 closure)
//
// Exercises the `SELECT ... WHERE timestamp > 'A' AND timestamp < 'B'` parser
// path end-to-end through run_materialization_pipeline to the wire-level POST body.
// Verifies `operands` (not `conditions`) and the compound `and` structure.
// Closes cycle-17 LOW-004 (SAP-3 depth gap on EC-01-033).
// BC-2.01.013 EC-01-033; S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-17 LOW-004.
// ---------------------------------------------------------------------------

/// EC-01-033 parser-surface: compound AND filter (`timestamp > A AND timestamp < B`)
/// MUST emit POST body with `operation == "and"` and `operands` (not `conditions`).
///
/// This test enters from a real PrismQL string (SAP-3 parser-surface requirement).
/// SAP-3 rule 1: at least one test must reach each BC arm end-to-end from the
/// public surface (parser input), not merely via synthetic AST or hand-built QueryParams.
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_BC_2_01_013_claroty_audit_logs_ec01033_compound_and_parser_surface() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    // Load production claroty.sensor.toml directed at wiremock.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "EC-01-033 parser-surface: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let org_slug = "claroty-ec01033-parser-org";

    let mut spec_for_map = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("EC-01-033 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_map)");
    spec_for_map.base_url = mock_server.uri();
    let resolved_for_map = make_resolved_spec_for_pipeline(spec_for_map, org_slug);

    let mut spec_for_adapter = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect(
            "EC-01-033 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_adapter)",
        );
    spec_for_adapter.base_url = mock_server.uri();
    let resolved_for_adapter = make_resolved_spec_for_pipeline(spec_for_adapter, org_slug);

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert(
        (OrgSlug::new(org_slug), SensorId::from("claroty")),
        resolved_for_map,
    );
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    let (org_registry, org_id, org_slug_typed) = make_org_registry_for_pipeline(org_slug);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("EC-01-033 parser-surface: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved_for_adapter),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolverForPipeline::new("claroty-ec01033-token"),
    );
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    let session_ctx = SessionContext::new();

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed]),
        limit: None,
        ..QueryOptions::default()
    };

    // Both-bounds PrismQL query — forces EC-01-033 compound `and` path.
    // ADR-058 §G (ocsf_column_naming=true): audit_logs.timestamp ocsf_field="time" →
    // Arrow field is "time". Both-bounds filter uses the OCSF-flattened name.
    let pql_query = "SELECT * FROM claroty_audit_logs WHERE time > '2025-01-01T00:00:00Z' AND time < '2025-02-01T00:00:00Z'";

    let output =
        run_materialization_pipeline(pql_query, &options, &mut mat_ctx, &session_ctx).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();

    let audit_log_req = requests.iter().find(|r| r.url.path().contains("audit_log"));
    assert!(
        audit_log_req.is_some(),
        "EC-01-033 (parser-surface): run_materialization_pipeline must issue a POST to \
         /api/v1/audit_log/get. Got {} total requests.",
        requests.len()
    );

    let body = parse_received_body(&audit_log_req.unwrap().body);

    assert!(
        body.get("filter_by").is_some(),
        "EC-01-033 (parser-surface): POST body must contain 'filter_by'. Got body: {body}."
    );

    let filter_by = &body["filter_by"];

    // LOAD-BEARING: compound AND → operation == "and" with `operands` (NOT `conditions`).
    // BC-2.01.013 EC-01-033 explicitly requires `operands`; `conditions` is the defect key.
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "EC-01-033 (parser-surface): filter_by.operation must be 'and' for a both-bounds filter. \
         Got: {:?}. filter_by: {filter_by}",
        filter_by["operation"]
    );

    assert!(
        filter_by.get("operands").is_some(),
        "EC-01-033 (parser-surface) LOAD-BEARING: filter_by must use 'operands' key (not 'conditions'). \
         BC-2.01.013 EC-01-033. Got filter_by: {filter_by}."
    );

    assert!(
        filter_by.get("conditions").is_none(),
        "EC-01-033 (parser-surface): filter_by must NOT use 'conditions' key — 'operands' is required \
         per BC-2.01.013 EC-01-033. Got filter_by: {filter_by}."
    );

    let operands = filter_by["operands"]
        .as_array()
        .expect("operands must be an array");
    assert_eq!(
        operands.len(),
        2,
        "EC-01-033 (parser-surface): operands must have exactly 2 elements (start + end). Got: {operands:?}"
    );

    // Verify the two operands have the exact xDome operations (greater_or_equal / less_or_equal).
    // BC-2.01.013 EC-01-033 mandates EXACTLY these — greater_than/less_than are NOT in the
    // xDome audit_log vocabulary. F-C18-003: drop the `_than` alternatives.
    let has_gte = operands
        .iter()
        .any(|o| o["operation"] == "greater_or_equal");
    let has_lte = operands.iter().any(|o| o["operation"] == "less_or_equal");
    assert!(
        has_gte,
        "EC-01-033 (parser-surface): operands MUST include exactly 'greater_or_equal' \
         for the lower bound. 'greater_than' is NOT in the xDome audit_log vocabulary. \
         BC-2.01.013 EC-01-033. Got: {operands:?}"
    );
    assert!(
        has_lte,
        "EC-01-033 (parser-surface): operands MUST include exactly 'less_or_equal' \
         for the upper bound. 'less_than' is NOT in the xDome audit_log vocabulary. \
         BC-2.01.013 EC-01-033. Got: {operands:?}"
    );

    // Assert operand VALUES against the parsed bounds (F-C18-003: pin ordering/values).
    // start_time = "2025-01-01T00:00:00Z", end_time = "2025-02-01T00:00:00Z" (from pql_query).
    let gte_operand = operands
        .iter()
        .find(|o| o["operation"] == "greater_or_equal")
        .expect("EC-01-033: greater_or_equal operand must exist after has_gte assertion");
    let lte_operand = operands
        .iter()
        .find(|o| o["operation"] == "less_or_equal")
        .expect("EC-01-033: less_or_equal operand must exist after has_lte assertion");

    let gte_value_str = gte_operand["value"].as_str();
    assert!(
        gte_value_str.is_some(),
        "EC-01-033 (parser-surface): greater_or_equal value must be an ISO-8601 STRING. \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        gte_operand["value"]
    );
    let gte_dt = chrono::DateTime::parse_from_rfc3339(gte_value_str.unwrap())
        .expect("EC-01-033: gte operand value must parse as RFC3339");
    let expected_start_secs = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
        .expect("test fixture: start bound must be valid RFC3339")
        .timestamp();
    assert!(
        (gte_dt.timestamp() - expected_start_secs).abs() <= 60,
        "EC-01-033 (parser-surface): greater_or_equal value must match the start bound \
         (2025-01-01T00:00:00Z = {expected_start_secs}s). Got {}s (delta: {}s). \
         BC-2.01.013 EC-01-033.",
        gte_dt.timestamp(),
        gte_dt.timestamp() - expected_start_secs
    );

    let lte_value_str = lte_operand["value"].as_str();
    assert!(
        lte_value_str.is_some(),
        "EC-01-033 (parser-surface): less_or_equal value must be an ISO-8601 STRING. \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        lte_operand["value"]
    );
    let lte_dt = chrono::DateTime::parse_from_rfc3339(lte_value_str.unwrap())
        .expect("EC-01-033: lte operand value must parse as RFC3339");
    let expected_end_secs = chrono::DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z")
        .expect("test fixture: end bound must be valid RFC3339")
        .timestamp();
    assert!(
        (lte_dt.timestamp() - expected_end_secs).abs() <= 60,
        "EC-01-033 (parser-surface): less_or_equal value must match the end bound \
         (2025-02-01T00:00:00Z = {expected_end_secs}s). Got {}s (delta: {}s). \
         BC-2.01.013 EC-01-033.",
        lte_dt.timestamp(),
        lte_dt.timestamp() - expected_end_secs
    );

    assert!(
        output.is_ok(),
        "EC-01-033 (parser-surface): run_materialization_pipeline must succeed. Got: {:?}",
        output.err()
    );
}

// ---------------------------------------------------------------------------
// FIX-3 / TD-VSDD-059: tests for `Some(Err(()))` inverted-window-avoidance
// in `build_claroty_audit_filter_by`.
//
// These tests cover the DEFENSE-IN-DEPTH arms (unreachable from the production
// surface, but exercised here to close TD-VSDD-059 paper-fix gate — the FIX-2
// from burst-2 changed behavior in both-bounds arm; this proves it works).
//
// BCs: BC-2.01.013 EC-01-032 (malformed start → falls through to end-only)
// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3 / TD-VSDD-059
// ---------------------------------------------------------------------------

/// FIX-3-A: malformed start_time + valid end_time → single `less_or_equal`
///
/// When `start_time = Some("not-a-timestamp")` (RFC3339 parse fails) and
/// `end_time = Some("2026-01-01T00:00:00Z")`, `build_claroty_audit_filter_by`
/// MUST produce a SINGLE `less_or_equal` at `2026-01-01T00:00:00Z` with NO
/// compound `and` filter and NO `greater_or_equal` operand.
///
/// The `Some(Err(()))` arm falls through to EC-01-032 (end-only) to avoid
/// an inverted window. Adding `now-7d` as a synthetic lower bound when
/// `end_time < now-7d` would produce an empty result set (SOUL.md §4 violation).
///
/// TD-VSDD-059: this test provides a load-bearing assertion for the
/// `Some(Err(()))` branch changed in FIX-2 of burst-2. Without this test,
/// the closure is a paper-fix (no load-bearing assertion existed).
///
/// BCs: BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-A
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_malformed_start_time_falls_through_to_end_only()
{
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // Malformed start_time: RFC3339 parse will fail → Some(Err(())).
    // Valid end_time: provides the upper bound for EC-01-032.
    let valid_end = "2026-01-01T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some("not-a-timestamp".to_string()),
        end_time: Some(valid_end.to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-fix3a-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "FIX-3-A: fetch must issue a POST to /api/v1/audit_log/get"
    );

    let body = parse_received_body(&requests[0].body);

    assert!(
        body.get("filter_by").is_some(),
        "FIX-3-A: POST body must contain 'filter_by'. Got body: {body}."
    );

    let filter_by = &body["filter_by"];

    // LOAD-BEARING (FIX-3-A / TD-VSDD-059): Must be single `less_or_equal`, NOT compound `and`.
    // If `Some(Err(()))` erroneously substituted now-7d as start, we'd get compound `and`
    // with an inverted window (end_time=2026-01-01 < now-7d → empty result set).
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "less_or_equal",
        "FIX-3-A LOAD-BEARING (TD-VSDD-059): malformed start_time must fall through to \
         EC-01-032 (single less_or_equal). Got operation: {:?}. \
         If 'and', the Some(Err(())) arm incorrectly substituted now-7d as lower bound \
         — this produces an inverted window (end_time 2026-01-01 < now-7d). \
         BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-A.",
        filter_by["operation"]
    );

    // Must NOT contain greater_or_equal anywhere.
    let filter_by_str = serde_json::to_string(filter_by).unwrap_or_default();
    assert!(
        !filter_by_str.contains("\"greater_or_equal\""),
        "FIX-3-A: filter_by MUST NOT contain 'greater_or_equal' — malformed start means \
         no lower bound (EC-01-032). Got filter_by: {filter_by}. \
         BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-A."
    );

    // Must NOT have a compound `and` operation.
    assert_ne!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "FIX-3-A: filter_by MUST NOT be compound 'and' — EC-01-032 requires single \
         less_or_equal when start_time is malformed. \
         BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-A."
    );

    // Value must equal the valid end_time.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "FIX-3-A: filter_by.value must be an ISO-8601 string. Got: {:?}.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "FIX-3-A: filter_by.value must parse as RFC3339. Got: {:?}. Error: {:?}.",
        value_str,
        value_dt.err()
    );
    let end_secs = chrono::DateTime::parse_from_rfc3339(valid_end)
        .expect("test fixture: valid_end must be RFC3339")
        .timestamp();
    let value_secs = value_dt.unwrap().timestamp();
    assert!(
        (value_secs - end_secs).abs() <= 60,
        "FIX-3-A: filter_by.value must equal end_time ({valid_end} = {end_secs}s). \
         Got {value_secs}s (delta: {}s). BC-2.01.013 EC-01-032.",
        value_secs - end_secs
    );
}

/// FIX-3-B: valid start_time + malformed end_time → compound `and` with `now` as end
///
/// When `start_time = Some("2026-01-01T00:00:00Z")` and
/// `end_time = Some("not-a-timestamp")` (RFC3339 parse fails), the end_time
/// parse fails and the function substitutes `chrono::Utc::now()` as the end bound.
/// The result MUST be a compound `and` filter with:
///  - `greater_or_equal` at `2026-01-01T00:00:00Z` (the valid start)
///  - `less_or_equal` at approximately now (the fallback end)
///
/// TD-VSDD-059: this test provides a load-bearing assertion for the end_time
/// parse-failure fallback path. Without this test, the closure is a paper-fix.
///
/// BCs: BC-2.01.013 EC-01-033; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-B
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_malformed_end_time_substitutes_now() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // Valid start_time: provides the lower bound.
    // Malformed end_time: RFC3339 parse fails → substitutes chrono::Utc::now().
    let valid_start = "2026-01-01T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(valid_start.to_string()),
        end_time: Some("not-a-timestamp".to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-fix3b-token");

    let before_fetch_secs = chrono::Utc::now().timestamp();
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    let after_fetch_secs = chrono::Utc::now().timestamp();

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "FIX-3-B: fetch must issue a POST to /api/v1/audit_log/get"
    );

    let body = parse_received_body(&requests[0].body);

    assert!(
        body.get("filter_by").is_some(),
        "FIX-3-B: POST body must contain 'filter_by'. Got body: {body}."
    );

    let filter_by = &body["filter_by"];

    // LOAD-BEARING (FIX-3-B / TD-VSDD-059): Must be compound `and` with two operands.
    // Valid start + malformed end → compound AND (start=explicit, end≈now).
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "FIX-3-B LOAD-BEARING (TD-VSDD-059): valid start + malformed end must produce \
         compound 'and' filter. Got operation: {:?}. \
         BC-2.01.013 EC-01-033; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-B.",
        filter_by["operation"]
    );

    // Must use "operands" key (NOT "conditions").
    assert!(
        filter_by.get("operands").is_some(),
        "FIX-3-B: compound filter MUST use key 'operands' (NOT 'conditions'). \
         BC-2.01.013 EC-01-033."
    );
    assert!(
        filter_by.get("conditions").is_none(),
        "FIX-3-B: compound filter MUST NOT use key 'conditions'. \
         BC-2.01.013 EC-01-033."
    );

    let operands = filter_by["operands"].as_array().unwrap_or(&vec![]).clone();
    assert_eq!(
        operands.len(),
        2,
        "FIX-3-B: operands must have exactly 2 elements. Got: {:?}.",
        operands
    );

    // Lower bound: greater_or_equal at valid_start.
    let gte_op = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("greater_or_equal"))
        .expect("FIX-3-B: must have a greater_or_equal operand");
    let gte_val_str = gte_op["value"].as_str();
    assert!(
        gte_val_str.is_some(),
        "FIX-3-B: greater_or_equal value must be a string. Got: {:?}.",
        gte_op["value"]
    );
    let gte_secs = chrono::DateTime::parse_from_rfc3339(gte_val_str.unwrap())
        .expect("FIX-3-B: gte value must parse as RFC3339")
        .timestamp();
    let expected_start_secs = chrono::DateTime::parse_from_rfc3339(valid_start)
        .expect("test fixture: valid_start must be RFC3339")
        .timestamp();
    assert!(
        (gte_secs - expected_start_secs).abs() <= 60,
        "FIX-3-B: greater_or_equal value must equal start_time ({valid_start} = {expected_start_secs}s). \
         Got {gte_secs}s (delta: {}s). BC-2.01.013 EC-01-033.",
        gte_secs - expected_start_secs
    );

    // Upper bound: less_or_equal at approximately now (malformed end_time substitutes now).
    let lte_op = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("less_or_equal"))
        .expect("FIX-3-B: must have a less_or_equal operand");
    let lte_val_str = lte_op["value"].as_str();
    assert!(
        lte_val_str.is_some(),
        "FIX-3-B: less_or_equal value must be a string. Got: {:?}.",
        lte_op["value"]
    );
    let lte_secs = chrono::DateTime::parse_from_rfc3339(lte_val_str.unwrap())
        .expect("FIX-3-B: lte value must parse as RFC3339")
        .timestamp();

    // Malformed end_time → substituted chrono::Utc::now(), so lte value ≈ now.
    // Allow 60s tolerance for test execution time.
    let tolerance = 60i64;
    assert!(
        lte_secs >= before_fetch_secs - tolerance && lte_secs <= after_fetch_secs + tolerance,
        "FIX-3-B LOAD-BEARING (TD-VSDD-059): malformed end_time must substitute \
         chrono::Utc::now() as the end bound. \
         Expected lte_secs in [{}, {}], got {lte_secs}. \
         BC-2.01.013 EC-01-033; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3-B.",
        before_fetch_secs - tolerance,
        after_fetch_secs + tolerance
    );
}

// ---------------------------------------------------------------------------
// FIX-1 regression: String-equality on datetime INDEX column does NOT cause
// Arrow type mismatch (type-gate: pseudo-column path limited to String-typed
// INDEX columns only).
//
// Before the type-gate fix: `push_down_filters["timestamp"] = "today"` caused
// the pseudo-column path to build a `StringArray` for a `Timestamp(Microsecond,
// UTC)` schema field → `RecordBatch::try_new` Arrow type mismatch error.
//
// After the type-gate fix: the datetime INDEX column bypasses the pseudo-column
// path (type gate: column_type must be String) and uses `build_column_array`
// instead, building the correct Timestamp array from actual record data.
//
// BCs: BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-1
// ---------------------------------------------------------------------------

/// FIX-1 regression — String equality on datetime INDEX column is safe.
///
/// Sends `params.filters["timestamp"] = "today"` to `SpecDrivenSensorAdapter::fetch`.
/// Before the type-gate fix, this triggered Arrow type mismatch in
/// `pipeline_result_to_record_batch` because the pseudo-column path built
/// `StringArray` for a `Timestamp(Microsecond, UTC)` schema field.
///
/// After the type-gate fix, datetime INDEX columns bypass the pseudo-column
/// path (the gate requires `column_type == String`). `build_column_array` builds
/// the correct Timestamp array from actual record data. No Arrow normalization
/// error.
///
/// This test proves the type gate is active: `fetch()` succeeds (returns Ok)
/// with the record from the mock — no `SensorError::Internal` with
/// "OCSF→Arrow normalization failed".
///
/// BCs: BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-1
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_string_equality_on_datetime_index_col_safe() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // The filter value "today" is a non-date-like string. On a datetime INDEX column,
    // the pre-fix code built StringArray for a Timestamp schema field → Arrow type mismatch.
    // Post-fix: datetime INDEX columns use build_column_array → correct Timestamp array.
    let mut filters = std::collections::HashMap::new();
    filters.insert(
        "timestamp".to_string(),
        serde_json::Value::String("today".to_string()),
    );

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters,
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-fix1-regression-token");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // LOAD-BEARING (FIX-1 regression / type-gate):
    // fetch() MUST NOT return SensorError::Internal with "normalization failed".
    // Before the fix: Arrow type mismatch caused Internal error with that message.
    // After the fix: datetime INDEX column uses build_column_array → Ok result.
    assert!(
        result.is_ok(),
        "FIX-1 REGRESSION (type-gate): String equality on a datetime INDEX column \
         must NOT cause Arrow type mismatch in pipeline_result_to_record_batch. \
         Before fix: pseudo-column path built StringArray for Timestamp(Microsecond,UTC) \
         schema field → RecordBatch::try_new type mismatch → SensorError::Internal. \
         After fix: datetime INDEX column (column_type != String) bypasses pseudo-column \
         path → build_column_array produces correct Timestamp array → Ok result. \
         Got Err: {:?}. \
         Check the type-gate condition in pipeline_result_to_record_batch: \
         `col_spec.options.contains(Index) && col_spec.column_type == String`. \
         BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-1.",
        result.err()
    );
    // Wire-shape discipline (CLAUDE.md §Conventions): verify the result has at least 1 row.
    // The mock returns audit_log_response_one_record() = 1 record.
    // A successful fetch on a datetime INDEX column must produce non-empty output.
    let batches = result.as_ref().unwrap().batches.as_slice();
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "FIX-1 REGRESSION (type-gate wire-shape): fetch() must return at least 1 row. \
         Mock returns 1 record (audit_log_response_one_record). \
         Got 0 rows — likely build_column_array produced an empty or mismatched batch. \
         BC-2.01.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-1 cycle-17 LOW-006."
    );
}

// ---------------------------------------------------------------------------
// F-C18-001: EC-01-032 parser-surface — end-only filter via run_materialization_pipeline
//
// SAP-3 requirement: each BC arm must have at least one test reaching it end-to-end
// from the public surface (parser input). RG-006 exercises EC-01-032 via hand-built
// QueryParams; this test closes the SAP-3 gap by going through the full pipeline.
//
// Query: SELECT * FROM claroty_audit_logs WHERE timestamp < '2025-02-01T00:00:00Z'
// Expected wire body: single less_or_equal at 2025-02-01, NO greater_or_equal, NO "and".
//
// BCs: BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 F-C18-001
// ---------------------------------------------------------------------------

/// EC-01-032 parser-surface: end-only filter (`WHERE timestamp < 'bound'`) via
/// `run_materialization_pipeline` MUST emit POST body with a single `less_or_equal`
/// at the explicit bound — NO `"greater_or_equal"` and NO `"and"` (no synthetic floor).
///
/// SAP-3: enters from a real PrismQL string, not hand-built QueryParams.
/// EC-01-032 hazard: adding a synthetic 7-day lower bound when end_time < now-7d
/// produces an inverted/empty window (SOUL.md §4 silent-wrong-result violation).
///
/// Closes F-C18-001 from cycle-18 LOW findings.
///
/// BCs: BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_BC_2_01_013_claroty_audit_logs_ec01032_end_only_parser_surface() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "EC-01-032 parser-surface: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let org_slug = "claroty-ec01032-parser-org";

    let mut spec_for_map = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("EC-01-032 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_map)");
    spec_for_map.base_url = mock_server.uri();
    let resolved_for_map = make_resolved_spec_for_pipeline(spec_for_map, org_slug);

    let mut spec_for_adapter = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect(
            "EC-01-032 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_adapter)",
        );
    spec_for_adapter.base_url = mock_server.uri();
    let resolved_for_adapter = make_resolved_spec_for_pipeline(spec_for_adapter, org_slug);

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert(
        (OrgSlug::new(org_slug), SensorId::from("claroty")),
        resolved_for_map,
    );
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    let (org_registry, org_id, org_slug_typed) = make_org_registry_for_pipeline(org_slug);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("EC-01-032 parser-surface: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved_for_adapter),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolverForPipeline::new("claroty-ec01032-token"),
    );
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    let session_ctx = SessionContext::new();

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed]),
        limit: None,
        ..QueryOptions::default()
    };

    // End-only PrismQL query: only a < upper-bound predicate, no lower bound.
    // Uses 2025-02-01 (well before now-7d) to prove no synthetic 7-day floor is injected.
    // If a 7-day lower bound were added: end_time(2025-02-01) < floor(now-7d) → inverted window.
    let end_bound = "2025-02-01T00:00:00Z";
    // ADR-058 §G (ocsf_column_naming=true): audit_logs.timestamp ocsf_field="time" →
    // Arrow field is "time". End-only filter uses the OCSF-flattened name.
    let pql_query = "SELECT * FROM claroty_audit_logs WHERE time < '2025-02-01T00:00:00Z'";

    let _output =
        run_materialization_pipeline(pql_query, &options, &mut mat_ctx, &session_ctx).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();

    let audit_log_req = requests.iter().find(|r| r.url.path().contains("audit_log"));
    assert!(
        audit_log_req.is_some(),
        "EC-01-032 (parser-surface): run_materialization_pipeline must issue a POST to \
         /api/v1/audit_log/get. Got {} total requests.",
        requests.len()
    );

    let body = parse_received_body(&audit_log_req.unwrap().body);

    assert!(
        body.get("filter_by").is_some(),
        "EC-01-032 (parser-surface): POST body must contain 'filter_by'. Got body: {body}."
    );

    let filter_by = &body["filter_by"];

    // LOAD-BEARING (EC-01-032 / SAP-3): Must be single less_or_equal — NOT compound AND.
    // Any `"and"` operation means a synthetic lower bound was injected (EC-01-032 hazard).
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "less_or_equal",
        "EC-01-032 (parser-surface) LOAD-BEARING: end-only filter must emit single \
         'less_or_equal'. Got operation: {:?}. If 'and', a synthetic 7-day lower bound \
         was injected — FORBIDDEN (EC-01-032: inverted window when end_time < now-7d). \
         BC-2.01.013 EC-01-032.",
        filter_by["operation"]
    );

    // Must NOT contain "and" — no compound filter allowed for end-only case.
    let filter_by_str = serde_json::to_string(filter_by).unwrap_or_default();
    assert!(
        !filter_by_str.contains("\"and\""),
        "EC-01-032 (parser-surface): serialized filter_by MUST NOT contain '\"and\"'. \
         Got filter_by: {filter_by}. BC-2.01.013 EC-01-032."
    );

    // Must NOT contain "greater_or_equal" — no synthetic floor injected.
    assert!(
        !filter_by_str.contains("\"greater_or_equal\""),
        "EC-01-032 (parser-surface): serialized filter_by MUST NOT contain \
         '\"greater_or_equal\"' — end-only means single less_or_equal, no synthetic floor. \
         Got filter_by: {filter_by}. BC-2.01.013 EC-01-032 (SOUL.md §4 hazard)."
    );

    // Field must be "timestamp".
    assert_eq!(
        filter_by["field"].as_str().unwrap_or(""),
        "timestamp",
        "EC-01-032 (parser-surface): filter_by.field must be 'timestamp'. Got: {:?}. \
         BC-2.01.013 EC-01-032.",
        filter_by["field"]
    );

    // Value must be an ISO-8601 string matching the explicit end bound.
    // BC-2.01.013 EC-01-032: value is an ISO-8601 STRING, NOT an epoch integer.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "EC-01-032 (parser-surface): filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String). Got: {:?}. BC-2.01.013 EC-01-032.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "EC-01-032 (parser-surface): filter_by.value must parse as RFC3339. Got: {:?}. \
         Error: {:?}. BC-2.01.013 EC-01-032.",
        value_str,
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let end_secs = chrono::DateTime::parse_from_rfc3339(end_bound)
        .expect("test fixture: end_bound must be valid RFC3339")
        .timestamp();
    assert!(
        (value_secs - end_secs).abs() <= 60,
        "EC-01-032 (parser-surface): filter_by.value must equal the explicit end bound \
         ({end_bound} = {end_secs}s). Got {value_secs}s (delta: {}s). \
         BC-2.01.013 EC-01-032.",
        value_secs - end_secs
    );
}

// ---------------------------------------------------------------------------
// F-C18-002: EC-01-030 parser-surface — no-filter default via run_materialization_pipeline
//
// SAP-3 requirement: RG-001 exercises EC-01-030 via hand-built QueryParams (no WHERE).
// This test closes the SAP-3 gap by entering through run_materialization_pipeline
// with a bare "SELECT * FROM claroty_audit_logs" — no WHERE clause.
//
// Expected wire body: filter_by.operation == "greater_or_equal", value ≈ now − 604800s.
//
// BCs: BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001 F-C18-002
// ---------------------------------------------------------------------------

/// EC-01-030 parser-surface: no-filter query (`SELECT * FROM claroty_audit_logs`) via
/// `run_materialization_pipeline` MUST emit POST body with
/// `filter_by.operation == "greater_or_equal"` and `filter_by.value` ≈ now − 604800s
/// (7 days, ±60s tolerance).
///
/// SAP-3: enters from a real PrismQL string, not hand-built QueryParams.
/// BC-2.01.013 EC-01-030: when no time predicate is present, the 7-day default
/// look-back MUST be applied — the POST body MUST NOT be sent without a filter.
///
/// Closes F-C18-002 from cycle-18 LOW findings.
///
/// BCs: BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_BC_2_01_013_claroty_audit_logs_ec01030_no_filter_default_parser_surface() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "EC-01-030 parser-surface: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let org_slug = "claroty-ec01030-parser-org";

    let mut spec_for_map = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("EC-01-030 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_map)");
    spec_for_map.base_url = mock_server.uri();
    let resolved_for_map = make_resolved_spec_for_pipeline(spec_for_map, org_slug);

    let mut spec_for_adapter = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect(
            "EC-01-030 parser-surface: claroty.sensor.toml must parse cleanly (spec_for_adapter)",
        );
    spec_for_adapter.base_url = mock_server.uri();
    let resolved_for_adapter = make_resolved_spec_for_pipeline(spec_for_adapter, org_slug);

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert(
        (OrgSlug::new(org_slug), SensorId::from("claroty")),
        resolved_for_map,
    );
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    let (org_registry, org_id, org_slug_typed) = make_org_registry_for_pipeline(org_slug);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("EC-01-030 parser-surface: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved_for_adapter),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolverForPipeline::new("claroty-ec01030-token"),
    );
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    let session_ctx = SessionContext::new();

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed]),
        limit: None,
        ..QueryOptions::default()
    };

    // No-filter PrismQL query — no WHERE clause.
    // EC-01-030: 7-day default look-back MUST be injected automatically.
    let pql_query = "SELECT * FROM claroty_audit_logs";

    let before_pipeline_secs = chrono::Utc::now().timestamp();
    let _output =
        run_materialization_pipeline(pql_query, &options, &mut mat_ctx, &session_ctx).await;
    let after_pipeline_secs = chrono::Utc::now().timestamp();

    let requests = mock_server.received_requests().await.unwrap_or_default();

    let audit_log_req = requests.iter().find(|r| r.url.path().contains("audit_log"));
    assert!(
        audit_log_req.is_some(),
        "EC-01-030 (parser-surface): run_materialization_pipeline must issue a POST to \
         /api/v1/audit_log/get. Got {} total requests.",
        requests.len()
    );

    let body = parse_received_body(&audit_log_req.unwrap().body);

    // LOAD-BEARING (EC-01-030 / SAP-3): POST body MUST contain 'filter_by' even with no WHERE.
    // BC-2.01.013 EC-01-030: 7-day default look-back is mandatory when no time predicate exists.
    assert!(
        body.get("filter_by").is_some(),
        "EC-01-030 (parser-surface) LOAD-BEARING: POST body to /api/v1/audit_log/get MUST \
         contain 'filter_by' when the PrismQL query has no WHERE clause. \
         Got body: {body}. BC-2.01.013 EC-01-030."
    );

    let filter_by = &body["filter_by"];

    // operation must be "greater_or_equal" (single lower bound — 7-day default).
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "greater_or_equal",
        "EC-01-030 (parser-surface): filter_by.operation must be 'greater_or_equal' for \
         the 7-day default look-back. Got: {:?}. BC-2.01.013 EC-01-030.",
        filter_by["operation"]
    );

    // Value must be an ISO-8601 string ≈ now - 604800s (±60s tolerance).
    // BC-2.01.013 EC-01-030: value MUST be ISO-8601 STRING, NOT epoch integer.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "EC-01-030 (parser-surface): filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String). Got: {:?}. BC-2.01.013 EC-01-030.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "EC-01-030 (parser-surface): filter_by.value must parse as RFC3339. Got: {:?}. \
         Error: {:?}. BC-2.01.013 EC-01-030.",
        value_str,
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let seven_days_secs: i64 = 7 * 24 * 3600;
    // Expected floor and ceiling bracket the now-7d point across the pipeline execution window.
    let expected_floor = before_pipeline_secs - seven_days_secs;
    let expected_ceiling = after_pipeline_secs - seven_days_secs;
    let tolerance_secs: i64 = 60;
    assert!(
        value_secs >= expected_floor - tolerance_secs
            && value_secs <= expected_ceiling + tolerance_secs,
        "EC-01-030 (parser-surface): filter_by.value must be ≈ now - 604800s (±60s). \
         Expected range [{}, {}] (seconds), got {value_secs}s. \
         BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001.",
        expected_floor - tolerance_secs,
        expected_ceiling + tolerance_secs
    );
}
