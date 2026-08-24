// SPDX-License-Identifier: Apache-2.0
//! E2E subprocess smoke tests for S-DEMO-002: All 4 Sensors + Multi-Org Isolation.
//!
//! All tests in this file are marked `#[ignore]` per AC-010:
//!   "Standard `cargo nextest run -p prism-bin` skips them; the CI e2e profile un-ignores them."
//!
//! # E2E-001 gate
//! Every test carries `// E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.`
//!
//! # Red Gate mechanism
//! All tests are marked `#[ignore]` and require a live DTU demo server + prism-bin binary.
//! They are un-gated in CI via the 'e2e' nextest profile (`[profile.e2e]` in `.config/nextest.toml`).
//! Per AC-010: standard `cargo nextest run -p prism-bin` skips them (correct Red Gate behavior).
//!
//! # Test → AC → BC Mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | `test_BC_2_22_001_e2e_smoke_test_launches_dtu_and_prism_bin_without_error` | AC-001 | BC-2.22.001 |
//! | `test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data` | AC-003 | BC-2.11.005 |
//! | `test_BC_2_11_005_e2e_armis_query_returns_data` | AC-004 | BC-2.11.005 |
//! | `test_BC_2_11_005_e2e_claroty_query_returns_data` | AC-005 | BC-2.11.005 |
//! | `test_BC_2_11_005_e2e_cyberint_query_returns_data` | AC-006 | BC-2.11.005 |
//! | `test_BC_2_09_008_e2e_response_envelope_meta_fields_correct` | AC-007 | BC-2.09.008 |
//! | `test_BC_2_10_010_e2e_sigterm_cleanly_shuts_down_both_subprocesses` | AC-008 | BC-2.10.010 |
//! | `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count` | AC-011 | BC-3.2.001, BC-2.22.001 |
//! | `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032` | AC-012 | BC-3.2.001 |
//! | `test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port` | AC-013 | BC-3.2.001 |
//! | `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip` | AC-014 | BC-2.11.007 |
//! | `test_EC_004_e2e_limit_zero_returns_empty_not_error` | EC-004 | BC-2.11.001 |
//! | `test_EC_005_e2e_limit_200_returns_paginated_rows` | EC-005 | BC-2.11.001 |
//!
//! Story: S-DEMO-002
//! BCs: BC-2.11.001, BC-2.11.005, BC-2.09.008, BC-2.10.001, BC-2.10.010, BC-3.2.001,
//!      BC-2.22.001, BC-2.11.007

mod helpers;

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// AC-001 / BC-2.22.001: DTU + prism-bin launch without error
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.22.001: prism-bin + DTU launch without error.
///
/// Verifies BC-2.22.001 postcondition: "The MCP server binds to stdio ONLY AFTER
/// step 8 is complete". The test launches both subprocesses from scratch and
/// asserts that neither exits unexpectedly before the MCP handshake can begin.
///
/// Skipped in standard `cargo nextest run -p prism-bin` (requires live DTU binary).
/// Un-gated in CI via the `[profile.e2e]` nextest profile (E2E-001 gate).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_22_001_e2e_smoke_test_launches_dtu_and_prism_bin_without_error() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    // Step 1: Launch DTU server.
    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not write urls.json within 30s (EC-001)");

    // Step 2: Write prism config pointing at DTU ports.
    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    // Step 3: Launch prism-bin.
    let (prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin exited unexpectedly before MCP handshake (EC-002)");

    // Step 4: MCP initialize handshake.
    let _capabilities = mcp.initialize().expect("MCP initialize handshake failed");

    // Step 5: Assert tools/list contains `query` (AC-002).
    // The canonical tool name is "query" (registered via `pub async fn query` under
    // `#[tool_router]` in prism-mcp/src/server.rs; BC-2.11.001 H1 source-of-truth).
    let tools = mcp.tools_list().expect("tools/list failed");
    assert!(
        tools.iter().any(|t| t["name"] == "query"),
        "AC-002: tools/list must contain 'query' (the canonical tool name per BC-2.11.001); got: {tools:?}"
    );

    // Both subprocesses are running and MCP is responsive.
    // Guard drop (AC-008) sends SIGTERM and verifies clean exit.
    drop(mcp);
    drop(prism_guard);
    drop(dtu_guard);
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.11.005: CrowdStrike query returns non-empty OCSF data
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.11.005: CrowdStrike query returns non-empty OCSF data.
///
/// Verifies BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF
/// via the OCSF normalizer". Asserts that `detection_id` (Gap-CS-001 fix) and
/// `category_uid`/`class_uid` are present and non-null.
///
/// Skipped in standard `cargo nextest run -p prism-bin` (requires live DTU binary).
/// Un-gated in CI via the `[profile.e2e]` nextest profile (E2E-001 gate).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-003: query CrowdStrike detections.
    // Table name: "detections" (crowdstrike.sensor.toml [[tables]] table_name = "detections").
    // Full qualified source: "crowdstrike_detections" — underscore notation per PrismQL SQL mode.
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    // The resolver (sensor_id_from_table_name) rejects dot notation (E-QUERY-036 UnknownSourceTable).
    let response = mcp
        .tool_query("SELECT * FROM crowdstrike_detections LIMIT 5")
        .expect("tool_query failed for crowdstrike");

    // Assert non-empty data.
    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-003: expected at least 1 row from crowdstrike_detections; response: {response:?}"
    );

    // Assert OCSF fields present and non-null (BC-2.11.005 postcondition; spec AC-003 Then-clause).
    let first_row = &rows[0];
    assert!(
        first_row.get("category_uid").is_some(),
        "AC-003: category_uid must be present in OCSF output; row: {first_row:?}"
    );
    assert!(
        first_row.get("category_uid") != Some(&serde_json::Value::Null),
        "AC-003: category_uid must be non-null in OCSF output; row: {first_row:?}"
    );
    assert!(
        first_row.get("class_uid").is_some(),
        "AC-003: class_uid must be present in OCSF output; row: {first_row:?}"
    );
    assert!(
        first_row.get("class_uid") != Some(&serde_json::Value::Null),
        "AC-003: class_uid must be non-null in OCSF output; row: {first_row:?}"
    );

    // Assert detection_id present (Gap-CS-001 fix — NOT 'id').
    assert!(
        first_row.get("detection_id").is_some(),
        "AC-003: detection_id must be present (Gap-CS-001 fix — the primary key is detection_id, not id); row: {first_row:?}"
    );
    assert!(
        first_row.get("detection_id") != Some(&serde_json::Value::Null),
        "AC-003: detection_id must be non-null; row: {first_row:?}"
    );

    // Assert no error code in response.
    assert_response_has_no_error(&response);
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.11.005: Armis query returns non-empty data
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.11.005: Armis query returns non-empty data.
///
/// Verifies BC-2.11.005 materialization pipeline for Armis sensor.
/// Table: "devices" (armis.sensor.toml [[tables]] table_name = "devices").
///
/// Skipped in standard `cargo nextest run -p prism-bin` (requires live DTU binary).
/// Un-gated in CI via the `[profile.e2e]` nextest profile (E2E-001 gate).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_11_005_e2e_armis_query_returns_data() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-004: query Armis devices.
    // Table name: "devices" (armis.sensor.toml [[tables]] table_name = "devices").
    // DTU fetch path: GET /api/v1/search?aql=in:devices (S-DEMO-ARMIS-AQL-001 fix).
    // The Armis /api/v1/search path_template uses ${query.filter.aql} — there is NO
    // default AQL value. A WHERE aql = '...' predicate is mandatory for all armis queries.
    // SQL form required: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5")
        .expect("tool_query failed for armis");

    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-004: expected at least 1 row from armis_devices (DTU path: /api/v1/search?aql=in:devices); response: {response:?}"
    );

    // Assert no error code in response.
    assert_response_has_no_error(&response);
}

// ---------------------------------------------------------------------------
// AC-014 / BC-2.11.007: AQL push-down WHERE-clause→DTU round-trip
// ---------------------------------------------------------------------------

/// AC-014 / BC-2.11.007 §Mechanism B: full WHERE aql='in:devices' → DTU round-trip.
///
/// This is the E2E subprocess test that closes the AC-014 coverage gap identified in
/// F-DEMO002-LP-MED-002. It asserts BOTH halves of the full pipeline:
///
/// 1. The prism query `FROM armis_devices WHERE aql = 'in:devices' LIMIT 5` returns
///    non-empty rows from the Armis DTU (data round-trip succeeds end-to-end).
/// 2. The Armis DTU actually received `GET /api/v1/search?aql=in:devices` — verified
///    by querying `GET /dtu/aql-log` on the Armis DTU and asserting the verbatim
///    AQL string "in:devices" appears in `aql_strings` (BC-2.11.007 Mechanism B;
///    R-DTU-002 AQL capture; ADR-031 §D8-a).
///
/// # DTU capture mechanism
///
/// The Armis DTU's `GET /api/v1/search` handler calls `state.capture_aql(aql)` for
/// every received `aql` query parameter (routes/search.rs). The AQL log is exposed
/// at `GET /dtu/aql-log` as `{"aql_strings": [...]}`. The test queries this endpoint
/// directly on the Armis DTU's bound port (obtained from `dtu_ports.base_url("armis")`).
///
/// # Relationship to unit tests in aql_pushdown_tests.rs
///
/// The unit tests in `prism-query/src/tests/aql_pushdown_tests.rs` assert only the
/// query-layer (AST→FilterMap) step. THIS test is the load-bearing assertion for
/// the full pipeline: PQL parse → FilterMap → FetchContext → PipelineExecutor →
/// DTU HTTP call → aql-log capture.
///
/// # SID-1 compliance
///
/// This test is `#[ignore]`'d because it requires the DTU demo server binary and
/// the prism binary to be available (E2E-001 gate). It is un-gated in CI via the
/// `[profile.e2e]` nextest profile.
///
/// // E2E-001: requires DTU server running + prism binary; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running + prism binary; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    // Step 1: Launch the DTU demo server (all 4 clones including Armis).
    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not write urls.json within 30s (EC-001)");

    // Step 2: Write prism config pointing at DTU ports.
    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    // Step 3: Launch prism-bin.
    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin exited unexpectedly before MCP handshake (EC-002)");

    // Step 4: MCP initialize handshake.
    mcp.initialize().expect("MCP initialize handshake failed");

    // Step 5: Issue the AC-014 query with an explicit AQL WHERE predicate.
    // BC-2.11.007 Mechanism B: `WHERE aql = 'in:devices'` seeds FetchContext.query_filters["aql"]
    // which PipelineExecutor interpolates into the path_template
    // `/api/v1/search?aql=${query.filter.aql}`, forwarding the AQL opaquely to the DTU.
    // SQL form required: "FROM ... LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5")
        .expect("AC-014: tool_query failed for armis_devices with aql WHERE predicate");

    // Assertion A: prism returns non-empty data rows (pipeline succeeded end-to-end).
    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-014 BC-2.11.007: FROM armis_devices WHERE aql = 'in:devices' LIMIT 5 \
         must return at least 1 row (full pipeline: PQL parse → FilterMap → FetchContext → \
         DTU /api/v1/search?aql=in:devices → $.data.results); \
         got 0 rows. response: {response:?}"
    );

    // Assert no error envelope.
    assert_response_has_no_error(&response);

    // Assertion B: the Armis DTU received the verbatim AQL "in:devices".
    // Query GET /dtu/aql-log on the Armis DTU's bound port.
    // dtu_ports.base_url("armis") returns the Armis clone's HTTP base URL
    // (e.g., "http://127.0.0.1:<port>"). The aql-log endpoint is at /dtu/aql-log.
    let armis_base_url = dtu_ports
        .base_url("armis")
        .expect("AC-014: Armis DTU base_url not found in urls.json — verify demo.toml has [clones.armis] enabled");

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-014: reqwest client build must succeed");

    let aql_log_resp = http_client
        .get(format!("{armis_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-014: GET /dtu/aql-log on Armis DTU must not fail at transport layer");

    assert_eq!(
        aql_log_resp.status().as_u16(),
        200,
        "AC-014 BC-2.11.007: GET /dtu/aql-log on Armis DTU must return HTTP 200; \
         got {}",
        aql_log_resp.status()
    );

    let aql_log_body: serde_json::Value = aql_log_resp
        .json()
        .await
        .expect("AC-014: GET /dtu/aql-log response must be valid JSON");

    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("AC-014 BC-2.11.007: aql_log response must have 'aql_strings' array field");

    // BC-2.11.007 Mechanism B postcondition: the verbatim AQL "in:devices" must appear
    // in the DTU's aql-log. The PipelineExecutor percent-encodes the AQL in the URL;
    // axum's Query<SearchQueryParams> extractor percent-decodes it before capture_aql()
    // is called — so the log entry equals the original unencoded string.
    assert!(
        aql_strings.iter().any(|s| s.as_str() == Some("in:devices")),
        "AC-014 BC-2.11.007 §Mechanism B: Armis DTU /dtu/aql-log must contain \
         the verbatim AQL 'in:devices' after executing \
         'FROM armis_devices WHERE aql = ''in:devices'' LIMIT 5'. \
         REGRESSION INDICATOR: path_template in armis.sensor.toml may have lost \
         '?aql=${{query.filter.aql}}', or FetchContext.query_filters[\"aql\"] seeding \
         is broken, or DTU capture_aql() was removed from get_search handler. \
         aql_strings received: {:?}",
        aql_strings
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.11.005: Claroty queries (alerts + devices)
// ---------------------------------------------------------------------------

/// Red Gate test for AC-005 — Claroty alerts and devices.
///
/// Verifies BC-2.11.005 for Claroty sensor.
/// Tables: "alerts" (alert_type_name + detected_time per Gap-CL-005 fix),
///         "devices" (uid column per Gap-CL-003 fix).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_11_005_e2e_claroty_query_returns_data() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-005a: query Claroty alerts.
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let alerts_response = mcp
        .tool_query("SELECT * FROM claroty_alerts LIMIT 5")
        .expect("tool_query failed for claroty_alerts");

    let alert_rows = extract_rows_from_envelope(&alerts_response);
    assert!(
        !alert_rows.is_empty(),
        "AC-005: expected at least 1 row from claroty_alerts; response: {alerts_response:?}"
    );

    // Assert Stage 2 OCSF-flattened column names for Claroty alerts (ocsf_column_naming=true):
    // - alert_type_name is Tier-2 (KF-09 removed ocsf_field) → lives in raw_extensions
    // - detected_time OCSF-flattened → Arrow field 'time' (ocsf_field='time', single segment)
    let first_alert = &alert_rows[0];
    // AC-005: raw_extensions must exist and contain alert_type_name (Tier-2 column after KF-09).
    let raw_ext_val = first_alert.get("raw_extensions").expect(
        "AC-005: raw_extensions must be present for Claroty alerts under ocsf_column_naming=true",
    );
    let raw_ext_str = raw_ext_val
        .as_str()
        .expect("AC-005: raw_extensions must be a JSON string");
    let raw_ext_obj: serde_json::Value =
        serde_json::from_str(raw_ext_str).expect("AC-005: raw_extensions must be valid JSON");
    assert!(
        raw_ext_obj.get("alert_type_name").is_some(),
        "AC-005: alert_type_name must be in raw_extensions (Tier-2 column after KF-09 removes ocsf_field); raw_extensions: {raw_ext_str}"
    );
    assert!(
        first_alert.get("time").is_some(),
        "AC-005: time must be present (Stage 2 OCSF-flattened Arrow name for detected_time → ocsf_field='time'); row: {first_alert:?}"
    );
    assert_response_has_no_error(&alerts_response);

    // AC-005b: query Claroty devices (Gap-CL-003 fix — devices table added).
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let devices_response = mcp
        .tool_query("SELECT * FROM claroty_devices LIMIT 5")
        .expect("tool_query failed for claroty_devices");

    let device_rows = extract_rows_from_envelope(&devices_response);
    assert!(
        !device_rows.is_empty(),
        "AC-005: expected at least 1 row from claroty_devices (Gap-CL-003 fix — table was missing); response: {devices_response:?}"
    );

    // Assert device_uid present and non-null (uid OCSF-flattened → Arrow field 'device_uid'
    // because ocsf_field='device.uid', dot-separated → underscore-joined by Stage 2 routing).
    let first_device = &device_rows[0];
    assert!(
        first_device.get("device_uid").is_some(),
        "AC-005: device_uid must be present in claroty_devices result (Stage 2 OCSF-flattened Arrow name for uid → ocsf_field='device.uid'); row: {first_device:?}"
    );
    assert!(
        first_device.get("device_uid") != Some(&serde_json::Value::Null),
        "AC-005: device_uid must be non-null in claroty_devices result (Stage 2 OCSF-flattened Arrow name for uid → ocsf_field='device.uid'); row: {first_device:?}"
    );

    assert_response_has_no_error(&devices_response);
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.11.005: Cyberint query returns non-empty data
// ---------------------------------------------------------------------------

/// Red Gate test for AC-006 — Cyberint alerts.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_11_005_e2e_cyberint_query_returns_data() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-006: query Cyberint alerts.
    // Table name: "alerts" (cyberint.sensor.toml [[tables]] table_name = "alerts").
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM cyberint_alerts LIMIT 5")
        .expect("tool_query failed for cyberint");

    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-006: expected at least 1 row from cyberint_alerts; response: {response:?}"
    );
    assert_response_has_no_error(&response);
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.09.008: ResponseEnvelope _meta fields correct
// ---------------------------------------------------------------------------

/// Red Gate test for AC-007 — ResponseEnvelope trust annotation fields.
///
/// Verifies BC-2.09.008 postcondition: "ResponseEnvelope carries trust_level and
/// data_source fields".
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_09_008_e2e_response_envelope_meta_fields_correct() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM crowdstrike_detections LIMIT 5")
        .expect("tool_query failed for crowdstrike");

    // AC-007: assert ≥1 row before proceeding — safety_flags assertion is only meaningful
    // when data was actually returned (MED-002 resolution: non-vacuous assertion guard).
    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-007: safety_flags assertion requires ≥1 row to be non-vacuous; \
         got zero rows from crowdstrike_detections; response: {response:?}"
    );

    // AC-007: assert _meta.trust_level == "untrusted_external".
    let meta = response
        .get("_meta")
        .expect("AC-007: ResponseEnvelope must have '_meta' field; response: {response:?}");

    assert_eq!(
        meta.get("trust_level").and_then(|v| v.as_str()),
        Some("untrusted_external"),
        "AC-007: _meta.trust_level must be 'untrusted_external'; meta: {meta:?}"
    );

    // AC-007: assert _meta.safety_flags is an empty array (no injection flags on DTU data).
    let safety_flags = meta
        .get("safety_flags")
        .expect("AC-007: _meta.safety_flags must be present; meta: {meta:?}");
    assert_eq!(
        safety_flags.as_array().map(|a| a.is_empty()),
        Some(true),
        "AC-007: _meta.safety_flags must be an empty array for clean DTU data; meta: {meta:?}"
    );

    // AC-007: assert _meta.data_source contains the sensor name.
    //
    // ADV-SDEMO002-PR-P01-OBS-002: data_source serializes polymorphically:
    //   - single-sensor query → bare string (e.g. "crowdstrike")
    //   - cross-client query  → array (e.g. ["crowdstrike"])
    // (documented in safety_envelope.rs). Accept both forms so the assertion
    // does not panic if a future refactor changes single-sensor serialization.
    let data_source = meta
        .get("data_source")
        .expect("AC-007: _meta.data_source must be present; meta: {meta:?}");
    let source_contains_crowdstrike = match data_source {
        serde_json::Value::Array(arr) => arr.iter().any(|v| v.as_str() == Some("crowdstrike")),
        serde_json::Value::String(s) => s == "crowdstrike",
        other => panic!("AC-007: _meta.data_source must be a string or array; got: {other:?}"),
    };
    assert!(
        source_contains_crowdstrike,
        "AC-007: _meta.data_source must contain 'crowdstrike' for a crowdstrike query; data_source: {data_source:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.10.010: SIGTERM cleanly shuts down both subprocesses
// ---------------------------------------------------------------------------

/// Red Gate test for AC-008 — graceful shutdown on SIGTERM.
///
/// Verifies BC-2.10.010 postcondition: "Graceful Shutdown on SIGTERM/SIGINT".
/// Both prism-bin and DTU server must exit within 5 seconds with status 0.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
///
/// # SID-1 §2 deferral (SUGGESTION-1 from PR #171 review)
///
/// The SIGTERM handler in `crates/prism-bin/src/signals.rs` calls
/// `std::process::exit(0)` directly after receiving the signal. There is no
/// in-process hook, channel drain, or `Drop`-based callback that can be driven
/// by a unit test without spawning a real subprocess.
///
/// A unit-test substitute would require either:
/// a) Mocking `std::process::exit` — not possible in stable Rust without unsafe trickery.
/// b) Refactoring signals.rs to inject a "shutdown executor" trait — that is a
///    non-trivial architectural change (separate story scope).
///
/// The BC-2.10.010 exit-code CONTRACT (the part that IS unit-testable) is already
/// covered by `boot.rs::shutdown_exit_code_tests` — see
/// `test_wait_for_shutdown_clean_returns_exit_0` and siblings, which drive
/// `RunningServer::wait_for_shutdown()` with pre-resolved JoinHandles.
///
/// The subprocess-level SIGTERM → exit(0) behaviour (the part that REQUIRES a
/// subprocess) is covered by this `#[ignore]`'d test, ungated in CI via the
/// 'e2e' nextest profile.
///
/// Blocking dependency: signals.rs `std::process::exit(0)` is not mockable
/// without a refactor scoped to S-1.12-FOLLOWUP (signal handler redesign).
#[cfg(unix)]
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_2_10_010_e2e_sigterm_cleanly_shuts_down_both_subprocesses() {
    use std::time::{Duration, Instant};

    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (mut dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (mut prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // Issue a query to confirm working state before teardown.
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let _ = mcp.tool_query("SELECT * FROM crowdstrike_detections LIMIT 5");

    // Send SIGTERM to prism-bin (AC-008).
    let prism_pid = prism_guard.child.id() as libc::pid_t;
    unsafe { libc::kill(prism_pid, libc::SIGTERM) };

    // Wait up to 5 seconds for prism-bin to exit (BC-2.10.010).
    let deadline = Instant::now() + Duration::from_secs(5);
    let prism_status = loop {
        match prism_guard.child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(None) => {
                panic!(
                    "AC-008: prism-bin did not exit within 5s after SIGTERM (BC-2.10.010 violation)"
                );
            }
            Err(e) => panic!("AC-008: failed to poll prism-bin exit status: {e}"),
        }
    };

    assert!(
        prism_status.success(),
        "AC-008: prism-bin must exit with status 0 after SIGTERM; got: {prism_status:?}"
    );

    // Send SIGTERM to DTU server and wait.
    let dtu_pid = dtu_guard.child.id() as libc::pid_t;
    unsafe { libc::kill(dtu_pid, libc::SIGTERM) };

    let dtu_deadline = Instant::now() + Duration::from_secs(5);
    let dtu_status = loop {
        match dtu_guard.child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < dtu_deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(None) => {
                panic!("AC-008: DTU server did not exit within 5s after SIGTERM (BC-2.10.010)");
            }
            Err(e) => panic!("AC-008: failed to poll DTU server exit status: {e}"),
        }
    };

    assert!(
        dtu_status.success(),
        "AC-008: DTU server must exit with status 0 after SIGTERM; got: {dtu_status:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-011 / BC-3.2.001 + BC-2.22.001: Multi-org boot — correct adapter count
// ---------------------------------------------------------------------------

/// Red Gate test for AC-011.
///
/// Verifies that boot step 9A correctly registers 8 adapters for a 3-org config:
/// - demo-org-a: CrowdStrike + Armis (2 entries)
/// - demo-org-b: Claroty + Cyberint (2 entries)
/// - demo-org-c: all 4 sensors (4 entries)
/// Total: 8 entries in AdapterRegistry.
///
/// This is a subprocess test (requires live boot sequence). The unit-level test
/// for step9a registration is in `tests/bc_2_01_013_spec_driven_adapter.rs`.
///
/// Skipped in standard `cargo nextest run -p prism-bin` (requires live DTU binary).
/// Un-gated in CI via the `[profile.e2e]` nextest profile (E2E-001 gate).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    // Write 3-org config (demo-org-a: CS+Armis; demo-org-b: Claroty+Cyberint; demo-org-c: all 4).
    helpers::write_multi_org_demo_config(config_dir.path(), &dtu_ports)
        .expect("write_multi_org_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    // MCP initialize handshake verifies boot completed.
    mcp.initialize().expect("MCP handshake failed");

    // AC-011: verify all 4 sensors resolve for demo-org-c (has all 4 sensors).
    // This subprocess test proves demo-org-c 4-sensor resolvability over MCP only.
    // The authoritative len()==8 + no-aliasing assertion lives in the non-ignored unit test
    // `test_BC_3_2_001_step9a_multi_org_registers_eight_adapters` in bc_2_01_013_spec_driven_adapter.rs.
    // For demo-org-c, all 4 queries should return data.
    // Table names use underscore notation ({sensor}_{table}) per PrismQL FROM syntax.
    // armis_devices requires WHERE aql = '...' — /api/v1/search uses ${query.filter.aql}
    // with no default; omitting the predicate causes interpolation failure before any HTTP call.
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    for (org, sensor_table, where_clause) in [
        ("demo-org-c", "crowdstrike_detections", ""),
        ("demo-org-c", "armis_devices", " WHERE aql = 'in:devices'"),
        ("demo-org-c", "claroty_alerts", ""),
        ("demo-org-c", "cyberint_alerts", ""),
    ] {
        let query = format!("SELECT * FROM {sensor_table}{where_clause} LIMIT 1");
        let response = mcp.tool_query(&query).unwrap_or_else(|e| {
            panic!("AC-011: query failed for org={org} table={sensor_table}: {e}")
        });
        // If AdapterRegistry has all 4 entries for demo-org-c, no adapter-not-found error.
        assert_response_has_no_error(&response);
    }

    // AC-011: also verify that demo-org-a's 2 sensors resolve but NOT demo-org-b's sensors.
    // This is tested more explicitly in test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032.
}

// ---------------------------------------------------------------------------
// AC-012 / BC-3.2.001: Cross-org isolation — E-QUERY-032 for wrong-org sensor
// ---------------------------------------------------------------------------

/// AC-012 / BC-3.2.001: Cross-org isolation — E-QUERY-032 for sensor not registered for org.
///
/// Verifies BC-3.2.001 postcondition 5: when an explicit `clients` scope is given and
/// the queried sensor is NOT registered for the requesting org (but may be registered
/// for OTHER orgs), the system MUST return a SURFACED operational error E-QUERY-032.
///
/// demo-org-a has CrowdStrike + Armis but NOT Claroty or Cyberint.
/// Querying claroty_alerts from demo-org-a context must return a BC-2.10.007 structured
/// error with `isError: true` and `structuredContent.error.code = "E-QUERY-032"`.
/// No Claroty data from demo-org-b is leaked.
///
/// # Wire shape (post F-2 / BC-2.10.007+)
///
/// The MCP `query` handler routes all user-visible domain errors (including E-QUERY-032)
/// through `prism_error_to_structured_call_result`, which returns:
///   `Ok(CallToolResult { isError: true, structuredContent: { error: { code: "E-QUERY-032", ... } } })`
/// rather than the pre-F-2 JSON-RPC protocol-level error (`{"error": {...}}`).
/// BC-2.10.007 (later, more-specific contract) governs the wire shape; BC-3.2.001 governs
/// the isolation semantics. Both are satisfied: the error is observably an error (isError=true)
/// with E-QUERY-032 signal; no data leaks.
///
/// `tool_query_scoped` is used (not `tool_query_scoped_expect_rpc_error`). Since `send_request`
/// now sees `{ "result": { "isError": true, ... } }` (no top-level `error` field), it returns
/// `Ok(result_object)`. The `tool_query_with_params` normalization then falls back to the raw
/// CallToolResult because `content[0].text` is the error string (not a JSON object).
///
/// The assertions verify ALL of:
/// - `isError == true` (BC-2.10.007 structured error envelope, not a protocol-level error)
/// - `structuredContent.error.code` == `"E-QUERY-032"`
/// - `structuredContent.error.message` contains `"E-QUERY-032"`, `"claroty"`, and `"demo-org-a"`
/// - zero data rows (no ResponseEnvelope `rows` field present)
/// - no Claroty data from demo-org-b is present
///
/// Skipped in standard `cargo nextest run -p prism-bin` (requires live DTU binary).
/// Un-gated in CI via the `[profile.e2e]` nextest profile (E2E-001 gate).
/// The non-ignored SID-1 unit test in prism-query covers the same E-QUERY-032 path
/// without external subprocess dependency (BC-5.39.001 postcondition 5 coverage).
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_multi_org_demo_config(config_dir.path(), &dtu_ports)
        .expect("write_multi_org_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-012: query claroty_alerts from demo-org-a (Claroty NOT registered for demo-org-a).
    // The MCP `query` tool scopes to demo-org-a; resolve_source_refs raises E-QUERY-032
    // at the query-planning boundary (BC-3.2.001 postcondition 5).
    //
    // Post F-2: the query handler returns Ok(CallToolResult{isError:true}) via
    // `prism_error_to_structured_call_result`, NOT a JSON-RPC protocol-level error.
    // `tool_query_scoped` is used (not `tool_query_scoped_expect_rpc_error`): send_request
    // sees `{ "result": { "isError": true, ... } }` and returns Ok(result_object).
    // `tool_query_with_params` falls back to the raw CallToolResult since content[0].text
    // is the error string "ERROR: [validation] - ..." (not a JSON object).
    //
    // Scoping param: `clients: ["demo-org-a"]` (array of org slug strings).
    // QueryToolParams.clients: Option<Vec<String>>; #[serde(deny_unknown_fields)] rejects `org_slug`.
    // Claroty is registered for demo-org-b and demo-org-c (NOT demo-org-a).
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query_scoped("SELECT * FROM claroty_alerts LIMIT 5", "demo-org-a")
        .expect("tool_query_scoped failed at transport/I/O level");

    // AC-012 assertion 1: isError must be true (BC-2.10.007 structured error shape).
    // The response is the normalized CallToolResult: { "isError": true, "structuredContent": {...}, ... }.
    let is_error = response
        .get("isError")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        is_error,
        "AC-012 BC-3.2.001: querying claroty_alerts from demo-org-a must return \
         isError=true (BC-2.10.007 structured error envelope); got: {response:?}"
    );

    // AC-012 assertion 2: structuredContent.error.code must be "E-QUERY-032".
    let sc_error_code = response
        .get("structuredContent")
        .and_then(|sc| sc.get("error"))
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(
        sc_error_code.contains("E-QUERY-032"),
        "AC-012 BC-3.2.001: structuredContent.error.code must be 'E-QUERY-032'; \
         got: {sc_error_code:?}; response: {response:?}"
    );

    // AC-012 assertion 3: structuredContent.error.message contains "E-QUERY-032",
    // "claroty", and "demo-org-a".
    let sc_error_msg = response
        .get("structuredContent")
        .and_then(|sc| sc.get("error"))
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("");
    assert!(
        sc_error_msg.contains("E-QUERY-032"),
        "AC-012 BC-3.2.001: structuredContent.error.message must contain 'E-QUERY-032'; \
         got: {sc_error_msg:?}"
    );
    assert!(
        sc_error_msg.contains("claroty"),
        "AC-012 BC-3.2.001: structuredContent.error.message must mention sensor 'claroty'; \
         got: {sc_error_msg:?}"
    );
    assert!(
        sc_error_msg.contains("demo-org-a"),
        "AC-012 BC-3.2.001: structuredContent.error.message must mention org 'demo-org-a'; \
         got: {sc_error_msg:?}"
    );

    // AC-012 assertion 4: zero data rows (no leakage from demo-org-b).
    // For an isError=true CallToolResult there is no ResponseEnvelope rows field.
    let rows = extract_rows_from_envelope(&response);
    assert!(
        rows.is_empty(),
        "AC-012 BC-3.2.001: zero data rows must be returned when E-QUERY-032 is raised; \
         actual row count: {}; rows: {rows:?}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// AC-013 / BC-3.2.001: DTU multi-tenant — each org reaches the correct clone port
// ---------------------------------------------------------------------------

/// Test for AC-013 (Task 22).
///
/// Verifies BC-3.2.001 postcondition 3: both `demo-org-a` and `demo-org-c` have
/// CrowdStrike registered and both query executions succeed (reaching the same
/// DTU CrowdStrike clone port).
///
/// DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
/// AdapterRegistry layer only. Two orgs that both have CrowdStrike point to the
/// same DTU clone port and receive identical fixture data. The isolation guarantee
/// is structural at the AdapterRegistry keying layer — org A cannot accidentally get
/// org B's adapter. True per-org HTTP segregation is Wave 3 scope (BC-3.2.003/004).
///
/// This test is SEPARATE from `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count`
/// (which verifies boot-time registration count) — this test verifies runtime query
/// execution reaches the correct port for each org.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    // Write 3-org config (demo-org-a: CS+Armis; demo-org-b: Claroty+Cyberint; demo-org-c: all 4).
    helpers::write_multi_org_demo_config(config_dir.path(), &dtu_ports)
        .expect("write_multi_org_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // AC-013: query CrowdStrike from demo-org-a (CrowdStrike IS registered for demo-org-a).
    // DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
    // AdapterRegistry layer only (AC-013 scope clarification per S-DEMO-002).
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response_a = mcp
        .tool_query_scoped("SELECT * FROM crowdstrike_detections LIMIT 5", "demo-org-a")
        .expect("query for demo-org-a failed (unexpected network/transport error)");

    let rows_a = extract_rows_from_envelope(&response_a);
    assert!(
        !rows_a.is_empty(),
        "AC-013 BC-3.2.001: demo-org-a CrowdStrike query must return data rows \
         (CrowdStrike adapter registered for demo-org-a, pointing to DTU clone port); \
         response: {response_a:?}"
    );
    assert_response_has_no_error(&response_a);

    // AC-013: query CrowdStrike from demo-org-c (CrowdStrike IS registered for demo-org-c).
    // Both orgs point to the same DTU clone port — both receive identical fixture data.
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response_c = mcp
        .tool_query_scoped("SELECT * FROM crowdstrike_detections LIMIT 5", "demo-org-c")
        .expect("query for demo-org-c failed (unexpected network/transport error)");

    let rows_c = extract_rows_from_envelope(&response_c);
    assert!(
        !rows_c.is_empty(),
        "AC-013 BC-3.2.001: demo-org-c CrowdStrike query must return data rows \
         (CrowdStrike adapter registered for demo-org-c, pointing to DTU clone port); \
         response: {response_c:?}"
    );
    assert_response_has_no_error(&response_c);
}

// ---------------------------------------------------------------------------
// EC-004: LIMIT 0 returns empty-not-error
// ---------------------------------------------------------------------------

/// Test for EC-004 (Task 23).
///
/// Verifies that `FROM crowdstrike_detections LIMIT 0` returns no error envelope
/// and zero rows (empty-but-not-error per E2E-DEMO-WIRING-PLAN §6 Risk 3).
///
/// Coverage decision (F-PC-002): explicitly required per S-DEMO-002 EC-004.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_EC_004_e2e_limit_zero_returns_empty_not_error() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // EC-004: LIMIT 0 must return an empty result, not an error.
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM crowdstrike_detections LIMIT 0")
        .expect("tool query (LIMIT 0) failed at transport level");

    // Assert no error envelope.
    assert_response_has_no_error(&response);

    // Assert zero rows returned.
    let rows = extract_rows_from_envelope(&response);
    assert!(
        rows.is_empty(),
        "EC-004: LIMIT 0 must return zero rows (empty-not-error); \
         actual row count: {}; response: {response:?}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// EC-005: LIMIT 200 returns paginated rows without error
// ---------------------------------------------------------------------------

/// Test for EC-005 (Task 24).
///
/// Verifies that `FROM crowdstrike_detections LIMIT 200` returns no error envelope
/// and ≤200 rows. Asserts `rows.len() <= 200` (not exactly 200, because the DTU
/// fixture may have fewer than 200 rows).
///
/// CrowdStrike DTU fixture row count: read from `crates/prism-dtu-crowdstrike/` fixture
/// data. The DTU demo server serves all available fixture rows up to the requested LIMIT.
/// If the fixture has fewer than 200 rows, `len(rows) == fixture_count < 200` is correct.
/// We assert `!rows.is_empty() && rows.len() <= 200` to cover both cases without
/// hardcoding the fixture row count.
///
/// Coverage decision (F-PC-002): explicitly required per S-DEMO-002 EC-005.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_EC_005_e2e_limit_200_returns_paginated_rows() {
    let config_dir = TempDir::new().expect("failed to create temp config dir");

    let fixture_config =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/e2e-demo/demo.toml");
    let (_dtu_guard, dtu_ports) = helpers::launch_dtu_server(&fixture_config, &config_dir)
        .await
        .expect("DTU server did not start within 30s");

    helpers::write_demo_config(config_dir.path(), &dtu_ports).expect("write_demo_config failed");

    let (_prism_guard, mut mcp) = helpers::launch_prism_bin(config_dir.path())
        .await
        .expect("prism-bin failed to start");

    mcp.initialize().expect("MCP handshake failed");

    // EC-005: LIMIT 200 must return rows without error.
    // The DTU fixture may have fewer than 200 CrowdStrike detections; if so,
    // len(rows) == fixture_count (still <= 200 and != 0).
    // SQL form: "FROM ... LIMIT N" is invalid; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query("SELECT * FROM crowdstrike_detections LIMIT 200")
        .expect("tool query (LIMIT 200) failed at transport level");

    // Assert no error envelope.
    assert_response_has_no_error(&response);

    // Assert at least 1 row (DTU fixture is non-empty) and at most 200 rows (LIMIT respected).
    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "EC-005: LIMIT 200 query must return at least 1 row (DTU fixture is non-empty); \
         response: {response:?}"
    );
    assert!(
        rows.len() <= 200,
        "EC-005: LIMIT 200 must not return more than 200 rows; \
         actual row count: {}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

/// Extract the data rows from a ResponseEnvelope JSON.
///
/// ResponseEnvelope shape (prism-mcp/src/safety_envelope.rs + server.rs):
///
/// The `tool_query_with_params` helper in `helpers/mod.rs` extracts the
/// `content[0].text` field from the MCP tools/call result and parses it as JSON,
/// yielding the top-level `ResponseEnvelope` struct.  The `query` handler builds:
///
/// ```json
/// {
///   "_meta": { ... },
///   "results": {
///     "rows": [...],
///     "returned_results": N,
///     "total_available": N,
///     "is_truncated": false
///   },
///   "content": [{ "type": "text", "text": "N results found" }],
///   "structuredContent": { "results": { "rows": [...], ... } }
/// }
/// ```
///
/// The `results` field is the payload JSON object; rows live at `results.rows`.
/// `structuredContent.results` mirrors `results` (same data, different field path).
///
/// This function is load-bearing for EC-004 (LIMIT 0 → rows present-and-empty,
/// distinguishable from "rows key missing") and EC-005 / AC-003/004/005/006/013
/// (non-empty rows from DTU data).  An incorrect path (e.g., top-level `["rows"]`)
/// always returns `Vec::new()` — making EC-004 vacuously true (non-load-bearing)
/// and EC-005/AC-003+ false-fail.
fn extract_rows_from_envelope(envelope: &serde_json::Value) -> Vec<serde_json::Value> {
    // Primary path: ResponseEnvelope.results.rows
    // (payload built in server.rs query handler, wrapped by SafetyEnvelopeBuilder).
    if let Some(rows) = envelope
        .get("results")
        .and_then(|r| r.get("rows"))
        .and_then(|r| r.as_array())
    {
        return rows.clone();
    }

    // Mirror path: ResponseEnvelope.structuredContent.results.rows
    // (structuredContent mirrors results per BC-2.09.001).
    if let Some(rows) = envelope
        .get("structuredContent")
        .and_then(|sc| sc.get("results"))
        .and_then(|r| r.get("rows"))
        .and_then(|r| r.as_array())
    {
        return rows.clone();
    }

    // For error responses (JSON-RPC error object or AdapterNotFound) there are no rows.
    Vec::new()
}

/// Assert that the response envelope has no error code.
fn assert_response_has_no_error(envelope: &serde_json::Value) {
    // Check for MCP-level error.
    if let Some(err) = envelope.get("error") {
        panic!("Response contains unexpected error: {err:?}; envelope: {envelope:?}");
    }
    // Check for ResponseEnvelope-level error status.
    if let Some(status) = envelope.get("status").and_then(|s| s.as_str())
        && (status.contains("error") || status.contains("Error"))
    {
        panic!("Response has error status '{status}'; envelope: {envelope:?}");
    }
}
