//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-AUDIT-001 — BC-2.10.012 v1.4.
//!
//! Finding AUDIT-001: `build_tables_for_client` emits bare table names (`alerts`,
//! `devices`) instead of sensor-prefixed names (`crowdstrike_alerts`,
//! `crowdstrike_devices`). The `TableDescriptor.name` field (returned in
//! `prism_describe` response → `results.tables[*].name`) must be
//! `{sensor_id}_{table_name}` (e.g., `cyberint_alerts`), NOT `{table_name}`.
//!
//! Root cause: `build_tables_for_client` in `prism-mcp/src/tools/prism_describe.rs`:
//! - Multi-tenant path (line ~320): `name: table.table_name.clone()` → must be
//!   `name: format!("{sensor_id}_{}", table.table_name)`
//! - Single-tenant path (line ~368): `name: table.table_name.clone()` → must be
//!   `name: format!("{}_{}", sensor_spec.sensor_id, table.table_name)`
//!
//! `build_tables_for_client` is private — test routes through `handle_prism_describe`
//! which calls it internally and returns the result as a JSON SafetyEnvelope.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_10_012_audit_001_sensor_prefixed_table_names | AUDIT-001 | BC-2.10.012 v1.4 |

use prism_core::column::ColumnType;
use prism_mcp::tools::prism_describe::handle_prism_describe;
use prism_spec_engine::{
    spec_parser::{ColumnSpec, SensorSpec, TableSpec},
    types::ConfigSnapshot,
    AuthType, ConfigManager,
};
use std::sync::Arc;

// ── Test fixture ──────────────────────────────────────────────────────────────

/// Build a `ConfigManager` with sensor `crowdstrike` having tables `alerts` and `devices`.
///
/// Uses the same pattern as `make_config_manager_acme_crowdstrike()` in mcp_prism_describe.rs.
/// The single-tenant path: `client_id == sensor_id == "crowdstrike"`.
fn make_config_manager_crowdstrike_two_tables() -> Arc<arc_swap::ArcSwap<ConfigManager>> {
    let alerts_table = TableSpec::new_point_in_time(
        "alerts",
        "security_finding",
        vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            Some("severity".to_string()),
            vec![],
        )],
        vec![],
    );

    let devices_table = TableSpec::new_point_in_time(
        "devices",
        "device_inventory_info",
        vec![ColumnSpec::new(
            "hostname",
            ColumnType::String,
            None,
            vec![],
        )],
        vec![],
    );

    let cs_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike Falcon sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![alerts_table, devices_table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = std::collections::HashMap::new();
    sensor_specs.insert("crowdstrike".to_string(), cs_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        ..ConfigSnapshot::empty()
    };
    Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::new(
        snapshot,
    )))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.10.012 v1.4 AUDIT-001 — Red Gate test.
///
/// `handle_prism_describe("crowdstrike", ...)` must return table descriptors with
/// `name` values set to `crowdstrike_alerts` and `crowdstrike_devices` (sensor-prefixed),
/// NOT bare `alerts` and `devices`.
///
/// # Red Gate failure
///
/// Current `build_tables_for_client` (single-tenant path, line ~368):
///   ```rust
///   TableDescriptor { name: table.table_name.clone(), ... }
///   ```
/// produces `name: "alerts"` instead of `name: "crowdstrike_alerts"`.
/// The test asserts the prefixed form is present and the bare form is absent.
///
/// # Why this matters
///
/// A `prism_describe` response with bare table names breaks the AI agent workflow:
/// agents build `FROM alerts | ...` queries (invalid PrismQL) instead of
/// `FROM crowdstrike_alerts | ...` (the correct form). This is the root cause of
/// the E-SENSOR-030 silent failures reported in the demo session evidence.
#[tokio::test]
async fn test_bc_2_10_012_audit_001_sensor_prefixed_table_names() {
    let config_manager = make_config_manager_crowdstrike_two_tables();

    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,                  // single-tenant path: no query_engine
        Some(&config_manager), // sensor_spec source
        None,                  // no audit_writer (fail-open)
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 AUDIT-001: handle_prism_describe must return Ok for valid client_id \
         'crowdstrike'; got Err",
    );

    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AUDIT-001: prism_describe must not return is_error=true; \
         got: {:?}",
        call_result
    );

    // Extract JSON response.
    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text).expect(
        "BC-2.10.012 AUDIT-001: prism_describe response must be valid JSON; \
         got non-JSON content",
    );

    // SafetyEnvelope: domain payload is under `results`.
    let results = parsed
        .get("results")
        .expect("BC-2.10.012 AUDIT-001: SafetyEnvelope response must have 'results' field");

    let tables = results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AUDIT-001: response results must contain a 'tables' array");

    assert_eq!(
        tables.len(),
        2,
        "BC-2.10.012 AUDIT-001: expected 2 tables for crowdstrike; got {}. \
         Response: {}",
        tables.len(),
        content_text
    );

    // Collect the actual table names returned.
    let actual_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
        .collect();

    // ── Positive assertions: sensor-prefixed names must be present ────────────

    let expected_prefixed = ["crowdstrike_alerts", "crowdstrike_devices"];
    for name in &expected_prefixed {
        assert!(
            actual_names.contains(name),
            "BC-2.10.012 AUDIT-001 RED GATE: table 'name' must be sensor-prefixed '{}' \
             — current code emits bare table name. \
             Actual names: {:?}. Full response: {}",
            name,
            actual_names,
            content_text
        );
    }

    // ── Negative assertions: bare names must NOT appear ───────────────────────

    let bare_names = ["alerts", "devices"];
    for name in &bare_names {
        assert!(
            !actual_names.contains(name),
            "BC-2.10.012 AUDIT-001: bare table name '{}' must NOT appear in prism_describe \
             response — it causes AI agents to build invalid 'FROM {}' queries. \
             Actual names: {:?}",
            name,
            name,
            actual_names
        );
    }

    // ── example_query must reference the prefixed name ────────────────────────

    // After the fix, example_query must use the prefixed name (e.g., `FROM crowdstrike_alerts`).
    // This is also a guard that build_example_query is called with the corrected name.
    for table in tables {
        let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let example_query = table
            .get("example_query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Only check tables that were successfully named with the prefix.
        if name.starts_with("crowdstrike_") {
            assert!(
                example_query.contains(name),
                "BC-2.10.012 AUDIT-001: example_query must reference sensor-prefixed name '{}'. \
                 Got example_query: {:?}",
                name,
                example_query
            );
        }
    }
}
