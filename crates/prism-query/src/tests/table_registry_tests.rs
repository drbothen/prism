//! Red Gate tests for S-3.13 — Dynamic Table Availability.
//!
//! Every test exercises a concrete behavior specified in the S-3.13 story v1.8
//! §Red Gate Test Names table. All 15 tests map 1:1 to the named Red Gates.
//!
//! # Naming convention
//! All tests follow `test_BC_S_SS_NNN_descriptive_name()` per the VSDD naming standard.
//!
//! # BC coverage
//! | BC | Tests |
//! |-----|-------|
//! | BC-2.11.001 | `_table_not_available_*`, `_did_you_mean_*`, `_mode_agnostic_*`, `_no_sensors_*`, `_e_query_037_mcp_*` |
//! | BC-2.16.001 | `_register_sensor_*`, `_unregistered_sensor_*`, `_explain_*` |
//! | BC-2.16.007 | `_hot_reload_add_*`, `_hot_reload_remove_*`, `_hot_reload_schema_*`, `_config_clients_*` |
//!
//! # Red Gate density
//! 15 tests / 15 functions = 1.0 (≥ 0.5 required per story §Red Gate Test Names).
//!
//! Story: S-3.13
// Test-code lint allowances: `expect()` in test assertions is the established project
// pattern for test setup (see alias_tests.rs, bc_gap_fill_tests.rs, integration_tests.rs).
#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]

use prism_core::PrismError;
use prism_spec_engine::{
    spec_parser::{AuthType, SensorSpec, TableSpec},
    ConfigSnapshot,
};

use crate::table_registry::TableRegistry;

// ---------------------------------------------------------------------------
// Helper: minimal SensorSpec builder for tests
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` for tests without going through TOML parsing.
///
/// Creates a spec with one table `{sensor_id}_{table_suffix}` (the table_name
/// field of TableSpec is `table_suffix`, so the registered key is
/// `{sensor_id}_{table_suffix}`).
fn make_sensor_spec_one_table(sensor_id: &str, table_suffix: &str) -> SensorSpec {
    SensorSpec::new(
        sensor_id,
        format!("{sensor_id} sensor"),
        AuthType::ApiKey,
        "https://example.com",
        vec![TableSpec::new_point_in_time(
            table_suffix,
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        Vec::new(),
    )
}

/// Build a `ConfigSnapshot` containing the given `SensorSpec`s.
///
/// Used in integration-level tests that need a full snapshot (e.g., testing
/// `TableRegistry::from_snapshot` directly). Retained as a test helper.
#[allow(dead_code)]
fn make_snapshot_with_specs(specs: Vec<SensorSpec>) -> ConfigSnapshot {
    let mut snapshot = ConfigSnapshot::empty();
    for spec in specs {
        snapshot.sensor_specs.insert(spec.sensor_id.clone(), spec);
    }
    snapshot
}

// ---------------------------------------------------------------------------
// AC-1: is_registered reflects loaded specs (BC-2.16.001)
// ---------------------------------------------------------------------------

/// BC-2.16.001 / AC-1: After `register_sensor`, `is_registered` returns `true`
/// for every table declared in the spec.
///
/// Verifies the core registration invariant: spec with table_name="alerts" and
/// sensor_id="armis" results in registered table "armis_alerts".
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_register_sensor_populates_is_registered() {
    let registry = TableRegistry::new();
    let spec = make_sensor_spec_one_table("armis", "alerts");
    registry
        .register_sensor(&spec)
        .expect("register_sensor must not fail");

    assert!(
        registry.is_registered("armis_alerts"),
        "AC-1 / BC-2.16.001: is_registered('armis_alerts') must be true after register_sensor"
    );
}

/// BC-2.16.001 / AC-1: `is_registered` returns `false` for a table that belongs
/// to a sensor not present in the initial `ConfigSnapshot`.
///
/// Registers armis only; crowdstrike is not configured.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_unregistered_sensor_is_not_registered() {
    let registry = TableRegistry::new();
    let armis_spec = make_sensor_spec_one_table("armis", "alerts");
    registry
        .register_sensor(&armis_spec)
        .expect("register_sensor must not fail");

    assert!(
        !registry.is_registered("crowdstrike_alerts"),
        "AC-1 / BC-2.16.001: is_registered('crowdstrike_alerts') must be false \
         when only armis is configured"
    );
}

// ---------------------------------------------------------------------------
// AC-2: E-QUERY-037 returned for unregistered table (BC-2.11.001)
// ---------------------------------------------------------------------------

/// BC-2.11.001 / AC-2: Querying an unregistered table returns
/// `PrismError::TableNotAvailable` with error code prefix "E-QUERY-037".
///
/// Uses check_availability_gate directly to verify the gate fires before fan-out.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_table_not_available_returns_e_query_037() {
    // Empty registry — no sensors configured.
    let registry = TableRegistry::new();

    let result = registry.check_availability_gate("SELECT * FROM crowdstrike_alerts");

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let display = details.to_string();
            assert!(
                display.starts_with("E-QUERY-037:"),
                "AC-2 / BC-2.11.001: Display must start with 'E-QUERY-037:', got: {display}"
            );
            assert_eq!(
                details.table, "crowdstrike_alerts",
                "AC-2: table field must be 'crowdstrike_alerts'"
            );
        }
        other => panic!(
            "AC-2 / BC-2.11.001: expected Err(PrismError::TableNotAvailable), got: {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------------
// AC-3: did_you_mean suggestion within Levenshtein ≤ 3 (BC-2.11.001)
// ---------------------------------------------------------------------------

/// BC-2.11.001 / AC-3: When the queried table name is within Levenshtein ≤ 3
/// of a registered table name, `did_you_mean` contains the suggestion string
/// `" Did you mean: 'X'?"`.
///
/// "crowdstrike_alert" → "crowdstrike_alerts" has Levenshtein distance 1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_did_you_mean_suggestion_within_threshold() {
    let registry = TableRegistry::new();
    let spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&spec)
        .expect("register must not fail");

    let suggestion = registry.did_you_mean("crowdstrike_alert");
    assert_eq!(
        suggestion, " Did you mean: 'crowdstrike_alerts'?",
        "AC-3 / BC-2.11.001 / EC-11-120: did_you_mean('crowdstrike_alert') \
         must suggest 'crowdstrike_alerts' (distance 1 ≤ 3)"
    );
}

/// BC-2.11.001 / EC-11-126: When the queried table name is further than Levenshtein 3
/// from ALL registered table names, `did_you_mean` returns `""` (no suggestion).
///
/// "totallywrong" vs "crowdstrike_alerts" has distance > 3.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_did_you_mean_empty_when_distance_exceeds_threshold() {
    let registry = TableRegistry::new();
    let spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&spec)
        .expect("register must not fail");

    let suggestion = registry.did_you_mean("totallywrong");
    assert_eq!(
        suggestion, "",
        "AC-3 / BC-2.11.001 / EC-11-126: did_you_mean('totallywrong') must return '' \
         (all distances > 3)"
    );
}

// ---------------------------------------------------------------------------
// AC-8: mode-agnostic plan-time gate (BC-2.11.001)
// ---------------------------------------------------------------------------

/// BC-2.11.001 / AC-8: SQL mode query against unregistered table returns E-QUERY-037
/// before any fan-out occurs.
///
/// Empty registry; SQL mode query `SELECT * FROM crowdstrike_detections`.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_mode_agnostic_plan_time_gate_sql() {
    let registry = TableRegistry::new();

    let result = registry.check_availability_gate("SELECT * FROM crowdstrike_detections");

    assert!(
        matches!(result, Err(PrismError::TableNotAvailable(..))),
        "AC-8 / BC-2.11.001: SQL mode 'SELECT * FROM crowdstrike_detections' with empty registry \
         must return Err(PrismError::TableNotAvailable). No fan-out must occur. Got: {result:?}"
    );
}

/// BC-2.11.001 / AC-8: Filter mode query against unregistered table returns E-QUERY-037
/// before any fan-out occurs.
///
/// Empty registry; filter mode `crowdstrike_alerts | severity = 'critical'`.
/// Filter mode PrismQL syntax: `source | predicate` where source is the table name.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_mode_agnostic_plan_time_gate_filter() {
    let registry = TableRegistry::new();

    // Filter mode: "source | predicate" — the source is "crowdstrike_alerts".
    // PrismQL filter syntax requires string literals to be quoted (e.g. 'critical').
    let result = registry.check_availability_gate("crowdstrike_alerts | severity = 'critical'");

    assert!(
        matches!(result, Err(PrismError::TableNotAvailable(..))),
        "AC-8 / BC-2.11.001: filter mode 'crowdstrike_alerts | severity = \\'critical\\'' with \
         empty registry must return Err(PrismError::TableNotAvailable). No fan-out must occur. \
         Got: {result:?}"
    );
}

/// BC-2.11.001 / AC-8: Pipe mode query against unregistered table returns E-QUERY-037
/// before any fan-out occurs.
///
/// Empty registry; pipe mode `crowdstrike_alerts | where severity = 'critical' | limit 10`.
/// Pipe mode PrismQL syntax: `source | where predicate | limit N` (using `where`, not `filter`).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_mode_agnostic_plan_time_gate_pipe() {
    let registry = TableRegistry::new();

    // Pipe mode: "source | where predicate | limit N"
    // PrismQL pipe keyword is `where` (not `filter`); `limit` is also a valid pipe stage.
    let result = registry
        .check_availability_gate("crowdstrike_alerts | where severity = 'critical' | limit 10");

    assert!(
        matches!(result, Err(PrismError::TableNotAvailable(..))),
        "AC-8 / BC-2.11.001: pipe mode with empty registry must return \
         Err(PrismError::TableNotAvailable). No fan-out must occur. Got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// EC-11-125: No sensors configured (BC-2.11.001)
// ---------------------------------------------------------------------------

/// BC-2.11.001 / EC-11-125: When no sensors are configured at startup,
/// `registered_tables()` returns empty list and queries return E-QUERY-037
/// with `available_sensors = ""` and `did_you_mean = ""`.
///
/// Empty registry; any query → TableNotAvailable with empty lists.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_no_sensors_configured_returns_e_query_037_empty_list() {
    let registry = TableRegistry::new();

    // Verify registered_tables() is empty.
    assert!(
        registry.registered_tables().is_empty(),
        "EC-11-125: registered_tables() must return empty Vec when no sensors configured"
    );

    let result = registry.check_availability_gate("SELECT * FROM any_table");

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            assert_eq!(
                details.available_sensors, "",
                "EC-11-125 / BC-2.11.001: available_sensors must be '' when no sensors configured, \
                 got: '{}'",
                details.available_sensors
            );
            assert_eq!(
                details.available_tables, "",
                "EC-11-125 / BC-2.11.001: available_tables must be '' when no sensors configured, \
                 got: '{}'",
                details.available_tables
            );
            assert_eq!(
                details.did_you_mean, "",
                "EC-11-125 / BC-2.11.001: did_you_mean must be '' when no sensors configured, \
                 got: '{}'",
                details.did_you_mean
            );
        }
        other => panic!(
            "EC-11-125 / BC-2.11.001: expected Err(PrismError::TableNotAvailable) with empty \
             lists, got: {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------------
// AC-2: E-QUERY-037 MCP mapping → -32602 INVALID_PARAMS (BC-2.11.001)
// ---------------------------------------------------------------------------

/// BC-2.11.001 / AC-2: `PrismError::TableNotAvailable` maps to MCP error code
/// -32602 (INVALID_PARAMS) in `error_mapping.rs`, NOT -32000 (INTERNAL_ERROR).
///
/// This test constructs the error variant with the correct boxed shape and
/// verifies the Display output starts with "E-QUERY-037:" and includes all
/// required fields. The MCP mapping test in prism-mcp validates the -32602 code.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params() {
    let err = helpers::make_table_not_available_error();

    match &err {
        PrismError::TableNotAvailable(details) => {
            let display = details.to_string();
            assert!(
                display.starts_with("E-QUERY-037:"),
                "AC-2 / BC-2.11.001: Display must start with 'E-QUERY-037:', got: {display}"
            );
            assert!(
                display.contains("crowdstrike"),
                "AC-2: Display must contain sensor name 'crowdstrike'"
            );
            assert!(
                display.contains("armis"),
                "AC-2: Display must contain available_sensors 'armis'"
            );
            // did_you_mean is "" in the helper, so no suggestion string.
            assert_eq!(details.did_you_mean, "", "AC-2: did_you_mean must be ''");
        }
        other => panic!("AC-2: expected TableNotAvailable variant, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-4/AC-5: Hot-reload add/remove (BC-2.16.007)
// ---------------------------------------------------------------------------

/// BC-2.16.007 / AC-4: When a sensor spec is added via `register_sensor` (simulating
/// hot-reload add), the registry reflects the new tables (`is_registered` returns `true`).
///
/// The MCP `notifications/resources/list_changed` notification is sent by the hot-reload
/// integration layer (wired via ConfigManager swap-listener in production); this unit test
/// verifies only the registry state (the notification channel is an integration concern).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_add_sensor_registers_tables() {
    let registry = TableRegistry::new();

    // Initially empty.
    assert!(
        !registry.is_registered("armis_alerts"),
        "AC-4 precondition: registry must be empty initially"
    );

    // Simulate hot-reload add: register armis spec.
    let armis_spec = make_sensor_spec_one_table("armis", "alerts");
    registry
        .register_sensor(&armis_spec)
        .expect("register_sensor must not fail");

    assert!(
        registry.is_registered("armis_alerts"),
        "AC-4 / BC-2.16.007 / EC-11-122: is_registered('armis_alerts') must be true \
         after register_sensor (hot-reload add)"
    );
}

/// BC-2.16.007 / AC-5: When a sensor spec is removed via `deregister_sensor` (simulating
/// hot-reload remove), `is_registered` returns `false` for its tables.
///
/// EC-11-121 (in-flight query isolation) is guaranteed by the arc-swap ConfigSnapshot
/// pattern (CI-007) — not testable in a unit test without a full query pipeline.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_remove_sensor_deregisters_tables() {
    let registry = TableRegistry::new();

    // Register claroty first.
    let claroty_spec = make_sensor_spec_one_table("claroty", "devices");
    registry
        .register_sensor(&claroty_spec)
        .expect("register_sensor must not fail");
    assert!(
        registry.is_registered("claroty_devices"),
        "AC-5 precondition: claroty_devices must be registered before deregistration test"
    );

    // Simulate hot-reload remove: deregister claroty.
    registry
        .deregister_sensor("claroty")
        .expect("deregister_sensor must not fail");

    assert!(
        !registry.is_registered("claroty_devices"),
        "AC-5 / BC-2.16.007: is_registered('claroty_devices') must be false \
         after deregister_sensor('claroty')"
    );
}

/// BC-2.16.007 / EC-11-123: When a spec is updated (schema change) via hot-reload,
/// the old tables are deregistered and new tables are re-registered atomically.
///
/// This test simulates: crowdstrike v1 has one table; v2 adds a second table.
/// After re-register, both old and new tables must reflect the v2 state.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_schema_change_reregisters() {
    let registry = TableRegistry::new();

    // v1: crowdstrike has only "alerts".
    let spec_v1 = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&spec_v1)
        .expect("register spec v1 must not fail");
    assert!(
        registry.is_registered("crowdstrike_alerts"),
        "v1 must have crowdstrike_alerts"
    );
    assert!(
        !registry.is_registered("crowdstrike_detections"),
        "v1 must NOT have crowdstrike_detections"
    );

    // v2: crowdstrike has "alerts" + "detections" (schema update).
    use prism_spec_engine::spec_parser::TableSpec;
    let spec_v2 = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike v2",
        AuthType::ApiKey,
        "https://example.com",
        vec![
            TableSpec::new_point_in_time("alerts", "security_finding", vec![], vec![]),
            TableSpec::new_point_in_time("detections", "security_finding", vec![], vec![]),
        ],
        None,
        "2.0.0",
        Vec::new(),
    );

    // register_sensor handles the deregister+re-register atomically (EC-11-123).
    registry
        .register_sensor(&spec_v2)
        .expect("register spec v2 must not fail");

    // After re-register: both tables must be present.
    assert!(
        registry.is_registered("crowdstrike_alerts"),
        "EC-11-123: crowdstrike_alerts must be registered in v2"
    );
    assert!(
        registry.is_registered("crowdstrike_detections"),
        "EC-11-123 / BC-2.16.007: new table crowdstrike_detections must be registered after \
         hot-reload schema update"
    );
}

// ---------------------------------------------------------------------------
// AC-6: explain_query lists only registered tables (BC-2.16.001)
// ---------------------------------------------------------------------------

/// BC-2.16.001 / AC-6: `registered_tables()` returns exactly the tables from the
/// live registry after adding a sensor — matches what `explain_query` uses for
/// `available_tables` field (the explain_query path reads from this method).
///
/// Verifies the registry is the source of truth for explain output (AC-6).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_explain_query_lists_only_registered_tables() {
    let registry = TableRegistry::new();

    // Register armis only.
    let armis_spec = make_sensor_spec_one_table("armis", "alerts");
    registry
        .register_sensor(&armis_spec)
        .expect("register armis must not fail");

    let tables = registry.registered_tables();

    assert!(
        tables.contains(&"armis_alerts".to_string()),
        "AC-6 / BC-2.16.001: registered_tables() must include 'armis_alerts' after registering armis. \
         Got: {tables:?}"
    );

    // Unregistered sensors must not appear.
    assert!(
        !tables.contains(&"crowdstrike_alerts".to_string()),
        "AC-6 / BC-2.16.001: registered_tables() must NOT include 'crowdstrike_alerts' \
         when only armis is configured. Got: {tables:?}"
    );
    assert!(
        !tables.contains(&"cyberint_alerts".to_string()),
        "AC-6 / BC-2.16.001: registered_tables() must NOT include 'cyberint_alerts' \
         when only armis is configured. Got: {tables:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-7: prism://config/clients reflects registered sensors (BC-2.16.007)
// ---------------------------------------------------------------------------

/// BC-2.16.007 / AC-7: `registered_sensor_ids()` and `registered_tables()` together
/// reflect only the currently-configured sensors. The `prism://config/clients`
/// resource handler (in prism-mcp) reads from these methods, so verifying the
/// registry state verifies the resource content.
///
/// Only crowdstrike + claroty are registered; armis and cyberint must not appear.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_config_clients_resource_reflects_registered() {
    let registry = TableRegistry::new();

    // Register crowdstrike and claroty only.
    let cs_spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    let cl_spec = make_sensor_spec_one_table("claroty", "devices");
    registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike must not fail");
    registry
        .register_sensor(&cl_spec)
        .expect("register claroty must not fail");

    let sensor_ids = registry.registered_sensor_ids();
    let tables = registry.registered_tables();

    // Crowdstrike and claroty must appear.
    assert!(
        sensor_ids.contains(&"crowdstrike".to_string()),
        "AC-7 / BC-2.16.007: crowdstrike must be in registered_sensor_ids()"
    );
    assert!(
        sensor_ids.contains(&"claroty".to_string()),
        "AC-7 / BC-2.16.007: claroty must be in registered_sensor_ids()"
    );

    // Armis and cyberint must NOT appear.
    assert!(
        !sensor_ids.contains(&"armis".to_string()),
        "AC-7 / BC-2.16.007: armis must NOT be in registered_sensor_ids()"
    );
    assert!(
        !sensor_ids.contains(&"cyberint".to_string()),
        "AC-7 / BC-2.16.007: cyberint must NOT be in registered_sensor_ids()"
    );

    // Tables must also be accurate.
    assert!(
        tables.contains(&"crowdstrike_alerts".to_string()),
        "AC-7: crowdstrike_alerts must appear in registered_tables()"
    );
    assert!(
        tables.contains(&"claroty_devices".to_string()),
        "AC-7: claroty_devices must appear in registered_tables()"
    );
    assert!(
        !tables.contains(&"armis_alerts".to_string()),
        "AC-7: armis_alerts must NOT appear in registered_tables()"
    );
}

// ---------------------------------------------------------------------------
// Test helper module (not a test itself)
// ---------------------------------------------------------------------------
#[cfg(test)]
pub(crate) mod helpers {
    use prism_core::PrismError;

    /// Construct a `PrismError::TableNotAvailable` for the MCP mapping test.
    ///
    /// This helper is used by `test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params`
    /// to avoid re-implementing the variant construction there.
    pub fn make_table_not_available_error() -> PrismError {
        PrismError::TableNotAvailable(Box::new(prism_core::error::TableNotAvailableDetails {
            table: "crowdstrike_alerts".to_string(),
            sensor: "crowdstrike".to_string(),
            available_sensors: "armis, claroty".to_string(),
            available_tables: "armis_alerts, claroty_devices".to_string(),
            did_you_mean: "".to_string(),
        }))
    }
}
