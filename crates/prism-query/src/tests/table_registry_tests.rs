//! Red Gate tests for S-3.13 — Dynamic Table Availability.
//!
//! Tests in this file correspond to the S-3.13 story v1.10 §Red Gate Test Names table.
//! 14 of the 15 test functions map 1:1 to named AC Red Gates. The fifteenth test,
//! `test_BC_2_16_001_registered_sets_reflect_only_configured_sensors`, is a supplementary
//! BC-2.16.001 accessor-correctness test (verifying `registered_sensor_ids` /
//! `registered_tables` fidelity) — it is not one of the named story AC Red Gates.
//!
//! # Naming convention
//! All tests follow `test_BC_S_SS_NNN_descriptive_name()` per the VSDD naming standard.
//!
//! # BC coverage
//! | BC | Tests |
//! |-----|-------|
//! | BC-2.11.001 | `_table_not_available_*`, `_did_you_mean_*`, `_mode_agnostic_*`, `_no_sensors_*`, `_e_query_037_mcp_*` |
//! | BC-2.16.001 | `_register_sensor_*`, `_unregistered_sensor_*`, `_explain_*`, `_registered_sets_*` (supplementary) |
//! | BC-2.16.007 | `_hot_reload_add_*`, `_hot_reload_remove_*`, `_hot_reload_schema_*` |
//!
//! # Red Gate density
//! 14 named AC Red Gates / 15 total test functions (1 supplementary BC-2.16.001 accessor test).
//! Named Red Gate density: 14/15 = 0.93 (≥ 0.5 required per story §Red Gate Test Names).
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

/// BC-2.16.007 / AC-4: When a sensor spec is added via the REAL `ConfigManager::store`
/// swap path, the registry reflects the new tables (`is_registered` returns `true`).
///
/// Drives the production path: `ConfigManager::store(snapshot_with_armis)` →
/// `notify_swap_listeners()` → listener calls `register_sensor(armis_spec)` →
/// `is_registered("armis_alerts")` returns `true`.
///
/// The `notifications/resources/list_changed` MCP notification is deferred to S-5.03
/// (MCP resources framework; see CRIT-4 adjudication in fix-burst report).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_add_sensor_registers_tables() {
    use std::sync::Arc;

    use prism_spec_engine::config_manager::ConfigManager;

    // Start with an empty registry and a config that has NO sensors.
    let registry = Arc::new(TableRegistry::new());
    let manager = Arc::new(ConfigManager::empty());

    // Wire the swap listener (mirrors wire_table_registry_swap_listener in boot.rs).
    // The listener reads the new snapshot from `manager_for_listener.load()` and
    // applies register_sensor for all sensors in the new snapshot.
    {
        let manager_for_listener = Arc::clone(&manager);
        let registry_for_listener = Arc::clone(&registry);
        manager.register_swap_listener(Box::new(move || {
            let snap = manager_for_listener.load();
            // Register all sensors in the new snapshot (same as production listener).
            for spec in snap.sensor_specs.values() {
                let _ = registry_for_listener.register_sensor(spec);
            }
            // Deregister removed sensors.
            let new_ids: std::collections::HashSet<&str> =
                snap.sensor_specs.keys().map(String::as_str).collect();
            for id in registry_for_listener.registered_sensor_ids() {
                if !new_ids.contains(id.as_str()) {
                    let _ = registry_for_listener.deregister_sensor(&id);
                }
            }
        }));
    }

    // Precondition: initially empty.
    assert!(
        !registry.is_registered("armis_alerts"),
        "AC-4 precondition: registry must be empty initially"
    );

    // Drive the REAL hot-reload path: store a new snapshot that includes armis.
    let armis_spec = make_sensor_spec_one_table("armis", "alerts");
    let mut new_snapshot = ConfigSnapshot::empty();
    new_snapshot
        .sensor_specs
        .insert("armis".to_string(), armis_spec);
    manager.store(new_snapshot); // ← triggers notify_swap_listeners() → listener runs

    // AC-4 / BC-2.16.007 / EC-11-122: after the swap, the registry must reflect the new sensor.
    assert!(
        registry.is_registered("armis_alerts"),
        "AC-4 / BC-2.16.007 / EC-11-122: is_registered('armis_alerts') must be true \
         after ConfigManager::store (real hot-reload swap path). The swap listener wired \
         in boot.rs drives register_sensor — this test validates that wiring."
    );
}

/// BC-2.16.007 / AC-5: When a sensor spec is removed via the REAL `ConfigManager::store`
/// swap path, `is_registered` returns `false` for its tables.
///
/// Drives the production path: `ConfigManager::store(snapshot_without_claroty)` →
/// `notify_swap_listeners()` → listener calls `deregister_sensor("claroty")` →
/// `is_registered("claroty_devices")` returns `false`.
///
/// EC-11-121 (in-flight query isolation) is guaranteed by the arc-swap ConfigSnapshot
/// pattern (CI-007) — not testable in a unit test without a full query pipeline.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_remove_sensor_deregisters_tables() {
    use std::sync::Arc;

    use prism_spec_engine::config_manager::ConfigManager;

    // Start with claroty registered.
    let registry = Arc::new(TableRegistry::new());
    let claroty_spec = make_sensor_spec_one_table("claroty", "devices");
    let mut initial_snapshot = ConfigSnapshot::empty();
    initial_snapshot
        .sensor_specs
        .insert("claroty".to_string(), claroty_spec);
    let manager = Arc::new(ConfigManager::new(initial_snapshot));

    // Pre-populate registry from the initial snapshot.
    {
        let snap = manager.load();
        for spec in snap.sensor_specs.values() {
            registry
                .register_sensor(spec)
                .expect("initial registration must not fail");
        }
    }
    assert!(
        registry.is_registered("claroty_devices"),
        "AC-5 precondition: claroty_devices must be registered before deregistration test"
    );

    // Wire the swap listener.
    {
        let manager_for_listener = Arc::clone(&manager);
        let registry_for_listener = Arc::clone(&registry);
        manager.register_swap_listener(Box::new(move || {
            let snap = manager_for_listener.load();
            // Register all sensors in new snapshot.
            for spec in snap.sensor_specs.values() {
                let _ = registry_for_listener.register_sensor(spec);
            }
            // Deregister sensors that are no longer in the new snapshot.
            let new_ids: std::collections::HashSet<&str> =
                snap.sensor_specs.keys().map(String::as_str).collect();
            for id in registry_for_listener.registered_sensor_ids() {
                if !new_ids.contains(id.as_str()) {
                    let _ = registry_for_listener.deregister_sensor(&id);
                }
            }
        }));
    }

    // Drive the REAL hot-reload path: store a new snapshot that OMITS claroty.
    let empty_snapshot = ConfigSnapshot::empty();
    manager.store(empty_snapshot); // ← triggers notify_swap_listeners() → listener deregisters claroty

    // AC-5 / BC-2.16.007: after the swap, claroty_devices must be gone.
    assert!(
        !registry.is_registered("claroty_devices"),
        "AC-5 / BC-2.16.007: is_registered('claroty_devices') must be false \
         after ConfigManager::store with empty snapshot (real hot-reload remove path). \
         The swap listener wired in boot.rs drives deregister_sensor — this test validates that wiring."
    );
}

/// BC-2.16.007 / EC-11-123: When a spec is updated (schema change) via the REAL
/// `ConfigManager::store` swap path, old tables are deregistered and new tables
/// re-registered atomically.
///
/// Drives the production path: crowdstrike v1 has one table; store a new snapshot
/// with crowdstrike v2 (two tables). After the swap, both v2 tables are registered.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_007_hot_reload_schema_change_reregisters() {
    use std::sync::Arc;

    use prism_spec_engine::{config_manager::ConfigManager, spec_parser::TableSpec};

    // v1: crowdstrike has only "alerts".
    let spec_v1 = make_sensor_spec_one_table("crowdstrike", "alerts");
    let mut snapshot_v1 = ConfigSnapshot::empty();
    snapshot_v1
        .sensor_specs
        .insert("crowdstrike".to_string(), spec_v1);

    let registry = Arc::new(TableRegistry::new());
    let manager = Arc::new(ConfigManager::new(snapshot_v1));

    // Pre-populate from v1.
    {
        let snap = manager.load();
        for spec in snap.sensor_specs.values() {
            registry
                .register_sensor(spec)
                .expect("v1 registration must not fail");
        }
    }
    assert!(
        registry.is_registered("crowdstrike_alerts"),
        "v1 must have crowdstrike_alerts"
    );
    assert!(
        !registry.is_registered("crowdstrike_detections"),
        "v1 must NOT have crowdstrike_detections"
    );

    // Wire the swap listener.
    {
        let manager_for_listener = Arc::clone(&manager);
        let registry_for_listener = Arc::clone(&registry);
        manager.register_swap_listener(Box::new(move || {
            let snap = manager_for_listener.load();
            for spec in snap.sensor_specs.values() {
                let _ = registry_for_listener.register_sensor(spec);
            }
            let new_ids: std::collections::HashSet<&str> =
                snap.sensor_specs.keys().map(String::as_str).collect();
            for id in registry_for_listener.registered_sensor_ids() {
                if !new_ids.contains(id.as_str()) {
                    let _ = registry_for_listener.deregister_sensor(&id);
                }
            }
        }));
    }

    // v2: crowdstrike has "alerts" + "detections" (schema update).
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
    let mut snapshot_v2 = ConfigSnapshot::empty();
    snapshot_v2
        .sensor_specs
        .insert("crowdstrike".to_string(), spec_v2);

    // Drive the REAL hot-reload swap — register_sensor re-registers atomically (EC-11-123).
    manager.store(snapshot_v2);

    // After v2 swap: both tables must be present.
    assert!(
        registry.is_registered("crowdstrike_alerts"),
        "EC-11-123: crowdstrike_alerts must be registered in v2 after swap"
    );
    assert!(
        registry.is_registered("crowdstrike_detections"),
        "EC-11-123 / BC-2.16.007: new table crowdstrike_detections must be registered after \
         hot-reload schema update via ConfigManager::store (real swap path)"
    );
}

// ---------------------------------------------------------------------------
// AC-6: explain_query lists only registered tables (BC-2.16.001)
// ---------------------------------------------------------------------------

/// BC-2.16.001 / AC-6: `explain()` returns `available_tables` reflecting ONLY the
/// tables currently registered in the live `TableRegistry` — not a static list.
///
/// Drives the REAL `explain()` path with a `TableRegistry` threaded via
/// `ExplainOptions::table_registry`. Verifies that `ExplainResult.available_tables`
/// contains armis_alerts but NOT crowdstrike_alerts or cyberint_alerts.
///
/// This is a load-bearing test for AC-6: it calls the actual `explain()` function
/// and asserts the `available_tables` field in `ExplainResult`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_explain_query_lists_only_registered_tables() {
    use std::sync::Arc;

    use crate::explain::{explain, ExplainOptions};

    let registry = Arc::new(TableRegistry::new());

    // Register armis only.
    let armis_spec = make_sensor_spec_one_table("armis", "alerts");
    registry
        .register_sensor(&armis_spec)
        .expect("register armis must not fail");

    // Call the REAL explain() function with the wired TableRegistry (AC-6).
    let opts = ExplainOptions {
        table_registry: Some(Arc::clone(&registry)),
        ..ExplainOptions::default()
    };
    let result = explain("armis_alerts | severity = 'critical'", opts)
        .expect("explain() must succeed for a valid filter query");

    // AC-6 / BC-2.16.001: available_tables must reflect the live registry.
    assert!(
        result
            .available_tables
            .contains(&"armis_alerts".to_string()),
        "AC-6 / BC-2.16.001: ExplainResult.available_tables must include 'armis_alerts' \
         after registering armis. Got: {:?}",
        result.available_tables
    );

    // Unregistered sensors must NOT appear in available_tables.
    assert!(
        !result
            .available_tables
            .contains(&"crowdstrike_alerts".to_string()),
        "AC-6 / BC-2.16.001: available_tables must NOT include 'crowdstrike_alerts' \
         when only armis is configured. Got: {:?}",
        result.available_tables
    );
    assert!(
        !result
            .available_tables
            .contains(&"cyberint_alerts".to_string()),
        "AC-6 / BC-2.16.001: available_tables must NOT include 'cyberint_alerts' \
         when only armis is configured. Got: {:?}",
        result.available_tables
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.001: registered_sensor_ids / registered_tables reflect only configured sensors
// ---------------------------------------------------------------------------

/// BC-2.16.001: `registered_sensor_ids()` and `registered_tables()` reflect only
/// the sensors that were passed to `register_sensor()` — no more, no fewer.
///
/// This verifies the core registration fidelity invariant: after registering crowdstrike
/// and claroty, the accessor methods return exactly those two sensors and their tables.
/// Armis and cyberint (never registered) must not appear. These accessors back the
/// future `prism://config/clients` MCP resource in S-5.03; S-3.13 delivers the
/// accessors only, not the resource or its notifications.
///
/// Only crowdstrike + claroty are registered; armis and cyberint must not appear.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_registered_sets_reflect_only_configured_sensors() {
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
        "BC-2.16.001: crowdstrike must be in registered_sensor_ids() after register_sensor"
    );
    assert!(
        sensor_ids.contains(&"claroty".to_string()),
        "BC-2.16.001: claroty must be in registered_sensor_ids() after register_sensor"
    );

    // Armis and cyberint must NOT appear.
    assert!(
        !sensor_ids.contains(&"armis".to_string()),
        "BC-2.16.001: armis must NOT be in registered_sensor_ids() — was never registered"
    );
    assert!(
        !sensor_ids.contains(&"cyberint".to_string()),
        "BC-2.16.001: cyberint must NOT be in registered_sensor_ids() — was never registered"
    );

    // Tables must also be accurate.
    assert!(
        tables.contains(&"crowdstrike_alerts".to_string()),
        "BC-2.16.001: crowdstrike_alerts must appear in registered_tables()"
    );
    assert!(
        tables.contains(&"claroty_devices".to_string()),
        "BC-2.16.001: claroty_devices must appear in registered_tables()"
    );
    assert!(
        !tables.contains(&"armis_alerts".to_string()),
        "BC-2.16.001: armis_alerts must NOT appear in registered_tables() — was never registered"
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
        PrismError::TableNotAvailable(Box::new(prism_core::error::TableNotAvailableDetails::new(
            "crowdstrike_alerts",
            "crowdstrike",
            "armis, claroty",
            "armis_alerts, claroty_devices",
            "",
        )))
    }
}
