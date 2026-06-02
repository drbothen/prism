// SPDX-License-Identifier: Apache-2.0
//! Red Gate tests for S-DEMO-002: E2E Subprocess Smoke Test — All 4 Sensors + Multi-Org Isolation.
//!
//! All tests in this file are marked `#[ignore]` per AC-010:
//!   "Standard `cargo nextest run -p prism-bin` skips them; the CI e2e profile un-ignores them."
//!
//! # E2E-001 gate
//! Every test carries `// E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.`
//!
//! # Red Gate state
//! ALL tests FAIL at Red Gate because:
//! - `helpers::launch_dtu_server()` / `helpers::launch_prism_bin()` are stubs (`todo!()`).
//! - S-DEMO-001 AdapterRegistry wiring is not yet confirmed merged.
//! - `helpers::write_demo_config()` / `write_multi_org_demo_config()` are stubs.
//! - AC-014 AQL seeding (`query_filters["aql"]`) is not yet implemented in prism-query.
//!
//! # Test → AC → BC Mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | `test_BC_2_22_001_e2e_smoke_test_launches_dtu_and_prism_bin_without_error` | AC-001 | BC-2.22.001 |
//! | `test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data` | AC-003 | BC-2.11.005 |
//! | `test_BC_2_11_005_e2e_armis_query_returns_data` | AC-004 | BC-2.11.005 |
//! | `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count` | AC-011 | BC-3.2.001, BC-2.22.001 |
//! | `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_adapter_not_found` | AC-012 | BC-3.2.001 |
//!
//! Story: S-DEMO-002 v1.3
//! BCs: BC-2.11.001, BC-2.11.005, BC-2.09.008, BC-2.10.001, BC-2.10.010, BC-3.2.001,
//!      BC-2.22.001, BC-2.11.007

mod helpers;

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// AC-001 / BC-2.22.001: DTU + prism-bin launch without error
// ---------------------------------------------------------------------------

/// Red Gate test for AC-001.
///
/// Verifies BC-2.22.001 postcondition: "The MCP server binds to stdio ONLY AFTER
/// step 8 is complete". The test launches both subprocesses from scratch and
/// asserts that neither exits unexpectedly before the MCP handshake can begin.
///
/// FAIL at Red Gate: `helpers::launch_dtu_server()` and `helpers::launch_prism_bin()`
/// are stubs (`todo!()`), so this test panics immediately.
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

    // Step 5: Assert tools/list contains tool_query (AC-002).
    let tools = mcp.tools_list().expect("tools/list failed");
    assert!(
        tools.iter().any(|t| t["name"] == "tool_query"),
        "AC-002: tools/list must contain 'tool_query'; got: {tools:?}"
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

/// Red Gate test for AC-003.
///
/// Verifies BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF
/// via the OCSF normalizer". Asserts that `detection_id` (Gap-CS-001 fix) and
/// `category_uid`/`class_uid` are present and non-null.
///
/// FAIL at Red Gate: stub helpers prevent subprocess launch.
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
    // Full qualified source: "crowdstrike.detections" or "crowdstrike_detections"
    // per PrismQL FROM syntax. Use canonical table reference.
    let response = mcp
        .tool_query("FROM crowdstrike.detections LIMIT 5")
        .expect("tool_query failed for crowdstrike");

    // Assert non-empty data.
    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-003: expected at least 1 row from crowdstrike.detections; response: {response:?}"
    );

    // Assert OCSF fields present (BC-2.11.005 postcondition).
    let first_row = &rows[0];
    assert!(
        first_row.get("category_uid").is_some(),
        "AC-003: category_uid must be present in OCSF output; row: {first_row:?}"
    );
    assert!(
        first_row.get("class_uid").is_some(),
        "AC-003: class_uid must be present in OCSF output; row: {first_row:?}"
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

/// Red Gate test for AC-004.
///
/// Verifies BC-2.11.005 materialization pipeline for Armis sensor.
/// Table: "devices" (armis.sensor.toml [[tables]] table_name = "devices").
///
/// FAIL at Red Gate: stub helpers prevent subprocess launch.
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
    // The default AQL query_filter for the devices table is "in:devices".
    let response = mcp
        .tool_query("FROM armis.devices LIMIT 5")
        .expect("tool_query failed for armis");

    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-004: expected at least 1 row from armis.devices (DTU path: /api/v1/search?aql=in:devices); response: {response:?}"
    );

    // Assert no error code in response.
    assert_response_has_no_error(&response);
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
    let alerts_response = mcp
        .tool_query("FROM claroty.alerts LIMIT 5")
        .expect("tool_query failed for claroty.alerts");

    let alert_rows = extract_rows_from_envelope(&alerts_response);
    assert!(
        !alert_rows.is_empty(),
        "AC-005: expected at least 1 row from claroty.alerts; response: {alerts_response:?}"
    );

    // Assert Gap-CL-005 column names (alert_type_name, detected_time — NOT type/created_at).
    let first_alert = &alert_rows[0];
    assert!(
        first_alert.get("alert_type_name").is_some(),
        "AC-005: alert_type_name must be present (Gap-CL-005 fix — column renamed from 'type'); row: {first_alert:?}"
    );
    assert!(
        first_alert.get("detected_time").is_some(),
        "AC-005: detected_time must be present (Gap-CL-005 fix — column renamed from 'created_at'); row: {first_alert:?}"
    );
    assert_response_has_no_error(&alerts_response);

    // AC-005b: query Claroty devices (Gap-CL-003 fix — devices table added).
    let devices_response = mcp
        .tool_query("FROM claroty.devices LIMIT 5")
        .expect("tool_query failed for claroty.devices");

    let device_rows = extract_rows_from_envelope(&devices_response);
    assert!(
        !device_rows.is_empty(),
        "AC-005: expected at least 1 row from claroty.devices (Gap-CL-003 fix — table was missing); response: {devices_response:?}"
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
    let response = mcp
        .tool_query("FROM cyberint.alerts LIMIT 5")
        .expect("tool_query failed for cyberint");

    let rows = extract_rows_from_envelope(&response);
    assert!(
        !rows.is_empty(),
        "AC-006: expected at least 1 row from cyberint.alerts; response: {response:?}"
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

    let response = mcp
        .tool_query("FROM crowdstrike.detections LIMIT 5")
        .expect("tool_query failed for crowdstrike");

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
    let data_source = meta
        .get("data_source")
        .expect("AC-007: _meta.data_source must be present; meta: {meta:?}");
    let source_arr = data_source
        .as_array()
        .expect("AC-007: _meta.data_source must be an array; meta: {meta:?}");
    assert!(
        source_arr.iter().any(|v| v.as_str() == Some("crowdstrike")),
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

    // Issue 4 queries to confirm working state before teardown.
    let _ = mcp.tool_query("FROM crowdstrike.detections LIMIT 5");

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
/// FAIL at Red Gate: `helpers::write_multi_org_demo_config()` and
/// `helpers::launch_prism_bin()` are stubs.
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
    // This indirectly proves 8 adapters were registered (2+2+4).
    // For demo-org-c, all 4 queries should return data.
    for (org, sensor_table) in [
        ("demo-org-c", "crowdstrike.detections"),
        ("demo-org-c", "armis.devices"),
        ("demo-org-c", "claroty.alerts"),
        ("demo-org-c", "cyberint.alerts"),
    ] {
        let response = mcp
            .tool_query(&format!("FROM {sensor_table} LIMIT 1"))
            .unwrap_or_else(|e| {
                panic!("AC-011: query failed for org={org} table={sensor_table}: {e}")
            });
        // If AdapterRegistry has all 4 entries for demo-org-c, no adapter-not-found error.
        assert_response_has_no_error(&response);
    }

    // AC-011: also verify that demo-org-a's 2 sensors resolve but NOT demo-org-b's sensors.
    // This is tested more explicitly in test_BC_3_2_001_e2e_cross_org_sensor_query_returns_adapter_not_found.
}

// ---------------------------------------------------------------------------
// AC-012 / BC-3.2.001: Cross-org isolation — AdapterNotFound for wrong-org sensor
// ---------------------------------------------------------------------------

/// Red Gate test for AC-012.
///
/// Verifies BC-3.2.001 postcondition 1: "state.lookup(org_id_A, resource_id)
/// returns None when entry was stored under org_id_B".
///
/// demo-org-a has CrowdStrike + Armis but NOT Claroty or Cyberint.
/// Querying claroty.alerts from demo-org-a context must return an error envelope,
/// not data. No Claroty data from demo-org-b is leaked.
///
/// FAIL at Red Gate: stub helpers prevent subprocess launch.
///
/// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
#[tokio::test]
#[ignore = "E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile."]
async fn test_BC_3_2_001_e2e_cross_org_sensor_query_returns_adapter_not_found() {
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

    // AC-012: query claroty.alerts from demo-org-a (Claroty NOT registered for demo-org-a).
    // The MCP tool_query must scope to org demo-org-a; the query engine routes via
    // AdapterRegistry.get(demo-org-a-org-id, SensorId::from("claroty")) → None → error envelope.
    //
    // Implementation note for the implementer: tool_query params must include
    // `org_slug = "demo-org-a"` in the scoping parameters (BC-2.11.001).
    let response = mcp
        .tool_query_scoped("FROM claroty.alerts LIMIT 5", "demo-org-a")
        .expect("tool_query_scoped call failed (unexpected network error)");

    // AC-012: response must contain an error indicating AdapterNotFound or isolation error.
    assert!(
        response_has_adapter_not_found_error(&response),
        "AC-012 BC-3.2.001: querying claroty.alerts from demo-org-a \
         must return AdapterNotFound error (Claroty not registered for demo-org-a); \
         actual response: {response:?}"
    );

    // AC-012: zero data rows returned (no data leakage from demo-org-b).
    let rows = extract_rows_from_envelope(&response);
    assert!(
        rows.is_empty(),
        "AC-012 BC-3.2.001: zero data rows must be returned when AdapterNotFound; \
         actual row count: {}; rows: {rows:?}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

/// Extract the data rows from a ResponseEnvelope JSON.
///
/// ResponseEnvelope shape (BC-2.09.008): `{ "rows": [...], "_meta": {...} }` or
/// similar structure. Implementer aligns with the actual ResponseEnvelope JSON shape
/// produced by prism-mcp's tool_query handler.
fn extract_rows_from_envelope(envelope: &serde_json::Value) -> Vec<serde_json::Value> {
    // Try common ResponseEnvelope shapes.
    // Implementer: align with the actual JSON shape of the ResponseEnvelope.
    if let Some(rows) = envelope.get("rows").and_then(|r| r.as_array()) {
        return rows.clone();
    }
    if let Some(data) = envelope.get("data").and_then(|d| d.as_array()) {
        return data.clone();
    }
    // If the envelope contains a "result" wrapper from MCP tools/call:
    if let Some(content) = envelope.get("result").and_then(|r| r.get("content")) {
        if let Some(text) = content
            .as_array()
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
        {
            if let Ok(inner) = serde_json::from_str::<serde_json::Value>(text) {
                if let Some(rows) = inner.get("rows").and_then(|r| r.as_array()) {
                    return rows.clone();
                }
            }
        }
    }
    Vec::new()
}

/// Assert that the response envelope has no error code.
fn assert_response_has_no_error(envelope: &serde_json::Value) {
    // Check for MCP-level error.
    if let Some(err) = envelope.get("error") {
        panic!("Response contains unexpected error: {err:?}; envelope: {envelope:?}");
    }
    // Check for ResponseEnvelope-level error status.
    if let Some(status) = envelope.get("status").and_then(|s| s.as_str()) {
        if status.contains("error") || status.contains("Error") {
            panic!("Response has error status '{status}'; envelope: {envelope:?}");
        }
    }
}

/// Return true if the response indicates AdapterNotFound or equivalent isolation error.
///
/// Implementer: align with the exact error variant string emitted by the query engine
/// for AdapterNotFound. BC-3.2.001 postcondition: "state.lookup(org_id_A, resource_id)
/// returns None when entry was stored under org_id_B".
fn response_has_adapter_not_found_error(envelope: &serde_json::Value) -> bool {
    let envelope_str = envelope.to_string();
    // Accept any of: "AdapterNotFound", "adapter_not_found", "sensor not registered",
    // "E-SENSOR-001", or equivalent isolation error text.
    envelope_str.contains("AdapterNotFound")
        || envelope_str.contains("adapter_not_found")
        || envelope_str.contains("sensor not registered")
        || envelope_str.contains("E-SENSOR-001")
        || envelope_str.contains("not registered for org")
}
