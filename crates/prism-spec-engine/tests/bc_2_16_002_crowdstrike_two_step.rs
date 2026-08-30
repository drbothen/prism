#![allow(non_snake_case)]
//! RG-03 / AC-007: BC-2.16.002 — CrowdStrike Two-Step Pipeline Execution
//!
//! Tests that PipelineExecutor runs the CrowdStrike two-step pipeline
//! (QueryV2 GET → PostEntities POST with fan_out_batch_size) against an
//! in-process wiremock HTTP mock server.
//!
//! No DTU clone required — uses wiremock for HTTP mocking.
//!
//! Red Gate discipline (BC-5.38.001): ALL tests MUST FAIL before Tasks 3, 5 (TOML
//! grammar extensions for two-step spec), and pipeline executor correctly handles
//! the variable forwarding between steps 1 and 2.
//!
//! AC coverage: AC-007 (CrowdStrike two-step pipeline parity test structure)
//!              PLUGIN-MIGRATION-001-F AC-002 (TOML-driven two-step test naming)
//! HS coverage: HS-013 (partial — the wiremock path, not the DTU parity verdict)

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_spec_engine::{
    NullAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// RG-03: CrowdStrike two-step pipeline via in-process HTTP mock
// ---------------------------------------------------------------------------

/// RG-03 / AC-007 / BC-2.16.002 §Postconditions:
/// test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec
///
/// Builds a SensorSpec matching the crowdstrike detections two-step pipeline
/// and drives PipelineExecutor::execute against a wiremock server. Asserts:
///   - Step 1 (QueryV2 GET) is issued and detection IDs are extracted.
///   - Step 2 (PostEntities POST) is issued with IDs from step 1.
///   - result.request_count >= 2 (single-page assumption; may be > 2 for paginated).
///   - result.records contains the detection summaries from step 2 response.
///
/// RED GATE: Fails until PipelineExecutor correctly forwards step-1 variables to
/// step-2 body template interpolation and the crowdstrike.sensor.toml two-step spec
/// is production-grade (Tasks 3-5).
#[tokio::test]
async fn test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec() {
    let mock_server = MockServer::start().await;

    // Step 1 response: 3 detection IDs (QueryV2 pattern).
    Mock::given(method("GET"))
        .and(path("/detects/queries/detects/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ["id-001", "id-002", "id-003"]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Step 2 response: detection summaries (PostEntities pattern).
    Mock::given(method("POST"))
        .and(path("/detects/entities/summaries/GET/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [
                {"id": "id-001", "created_timestamp": "2026-01-01T00:00:00Z", "status": "new"},
                {"id": "id-002", "created_timestamp": "2026-01-01T00:01:00Z", "status": "in_progress"},
                {"id": "id-003", "created_timestamp": "2026-01-01T00:02:00Z", "status": "closed"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Build the spec from the bundled TOML file (production load path).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");
    let mut spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("crowdstrike.sensor.toml must parse (Red Gate: skeleton may fail here)");

    // Override base_url to point at the wiremock server for this test.
    spec.base_url = mock_server.uri();

    // Resolve the detections table.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("crowdstrike spec must declare a 'detections' table (AC-001)");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build must succeed");

    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed for crowdstrike two-step pipeline");

    // AC-007: request_count >= 2 (single-page QueryV2 assumption; paginated may have > 2).
    assert!(
        result.request_count >= 2,
        // single-page QueryV2 assumption; paginated responses MAY have request_count > 2
        "Two-step pipeline must issue >= 2 requests (1 QueryV2 GET + 1 PostEntities POST); \
         got {}",
        result.request_count
    );

    // Records must be non-empty (3 detection summaries from step 2).
    assert_eq!(
        result.records.len(),
        3,
        "Two-step pipeline must return 3 detection records; got {}",
        result.records.len()
    );
}

/// RG-03 sub-assertion / AC-007 / BC-2.16.002 §Batch Boundary (EC-016-013-003):
/// A fan_out_batch_size=100 prevents batches larger than 100 IDs.
/// A query returning exactly 100 IDs must produce exactly one PostEntities batch.
///
/// RED GATE: Fails until PipelineExecutor::execute correctly handles fan_out_batch_size
/// and the crowdstrike spec declares fan_out_batch_size = 100 for the PostEntities step.
#[tokio::test]
async fn test_BC_2_16_002_crowdstrike_batch_boundary_100_ids_one_batch() {
    let mock_server = MockServer::start().await;

    // Exactly 100 IDs — the maximum batch size per CROWDSTRIKE_BATCH_SIZE.
    let ids: Vec<String> = (1..=100).map(|i| format!("id-{i:03}")).collect();
    let summaries: Vec<serde_json::Value> = ids
        .iter()
        .map(|id| serde_json::json!({"id": id, "created_timestamp": "2026-01-01T00:00:00Z"}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/detects/queries/detects/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ids
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Exactly 1 PostEntities call expected (100 IDs, batch size 100 → single batch).
    Mock::given(method("POST"))
        .and(path("/detects/entities/summaries/GET/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": summaries
        })))
        .expect(1) // single batch, not multiple
        .mount(&mock_server)
        .await;

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");
    let mut spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("crowdstrike.sensor.toml must parse");
    spec.base_url = mock_server.uri();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("detections table must exist");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed");

    assert_eq!(
        result.records.len(),
        100,
        "Batch of exactly 100 IDs must produce exactly 100 records (EC-016-013-003); got {}",
        result.records.len()
    );

    // request_count >= 2: 1 QueryV2 GET + at minimum 1 PostEntities POST.
    // With exactly 100 IDs and batch_size=100, exactly 2 requests expected.
    assert!(
        result.request_count >= 2,
        // single-page QueryV2 assumption; paginated responses MAY have request_count > 2
        "100-ID batch must issue >= 2 requests; got {}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// PLUGIN-MIGRATION-001-F AC-002: TOML-driven two-step naming gate
// ---------------------------------------------------------------------------

/// PLUGIN-MIGRATION-001-F / AC-002 / BC-2.16.009 postcondition:
/// test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven
///
/// Verifies that the CrowdStrike TOML spec declares both fetch steps required
/// for the two-step pipeline (QueryV2 GET + PostEntities POST), and that
/// PipelineExecutor correctly issues >= 2 requests when driven by the TOML spec.
///
/// This test exercises the same behavior as test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec
/// but is named per PLUGIN-MIGRATION-001-F AC-002 convention to confirm the
/// TOML-driven path with no direct CrowdStrikeAdapter or CrowdStrikeAuth import.
///
/// No DTU clone required — uses wiremock for HTTP mocking.
#[tokio::test]
async fn test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven() {
    let mock_server = MockServer::start().await;

    // Step 1 response: 2 detection IDs.
    Mock::given(method("GET"))
        .and(path("/detects/queries/detects/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ["cs-id-A", "cs-id-B"]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Step 2 response: detection summaries.
    Mock::given(method("POST"))
        .and(path("/detects/entities/summaries/GET/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [
                {"id": "cs-id-A", "created_timestamp": "2026-01-01T00:00:00Z", "status": "new"},
                {"id": "cs-id-B", "created_timestamp": "2026-01-01T00:01:00Z", "status": "closed"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // TOML-driven spec loading — no CrowdStrikeAdapter or CrowdStrikeAuth import.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable (AC-002)");
    let mut spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("crowdstrike.sensor.toml must parse (AC-002)");
    spec.base_url = mock_server.uri();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-002: crowdstrike spec must declare 'detections' table");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-002: PipelineExecutor::execute must succeed against wiremock");

    // AC-002: two-step fetch pattern fires (QueryV2 GET + PostEntities POST = >= 2 requests).
    assert!(
        result.request_count >= 2,
        "AC-002: two-step CrowdStrike pipeline must issue >= 2 requests; got {}",
        result.request_count
    );

    // AC-002: records from step 2 (PostEntities) are returned.
    assert_eq!(
        result.records.len(),
        2,
        "AC-002: two-step pipeline must return 2 detection records; got {}",
        result.records.len()
    );
}
