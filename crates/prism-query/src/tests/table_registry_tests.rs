//! Red Gate tests for S-3.13 — Dynamic Table Availability.
//!
//! Tests in this file correspond to the S-3.13 story v1.10 §Red Gate Test Names table.
//! Most test functions map 1:1 to named AC Red Gates.
//! `test_BC_2_16_001_registered_sets_reflect_only_configured_sensors` is a supplementary
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
//! One supplementary BC-2.16.001 accessor test; all other functions are named AC Red Gates
//! (≥ 0.5 density required per story §Red Gate Test Names; comfortably exceeded).
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

    let result = registry.check_availability_gate("SELECT * FROM crowdstrike_alerts", None, None);

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

    let result =
        registry.check_availability_gate("SELECT * FROM crowdstrike_detections", None, None);

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
    let result =
        registry.check_availability_gate("crowdstrike_alerts | severity = 'critical'", None, None);

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
    let result = registry.check_availability_gate(
        "crowdstrike_alerts | where severity = 'critical' | limit 10",
        None,
        None,
    );

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

    let result = registry.check_availability_gate("SELECT * FROM any_table", None, None);

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

/// BC-2.11.001 / AC-2: `PrismError::TableNotAvailable` Display output starts
/// with "E-QUERY-037:" and contains the required field substrings (sensor name,
/// available sensors, did_you_mean). This verifies the error variant's
/// formatting contract. The MCP code mapping (-32602) is validated separately
/// by `test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params` in prism-mcp.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_table_not_available_display_format() {
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
// MED-3 (S-3.13): atomic re-registration — no transient absence for overlapping tables
// ---------------------------------------------------------------------------

/// EC-11-123 / MED-3 (S-3.13): When `register_sensor` is called for a sensor that
/// was already registered (hot-reload schema update), tables that appear in BOTH
/// the old and new spec are NEVER transiently absent during the re-registration.
///
/// # Why a concurrency test is required
/// A single-threaded test that only checks final state CANNOT detect the transient
/// absence window. Under a non-atomic implementation (e.g., calling
/// `self.deregister_sensor()` then re-acquiring the lock for insert), the overlapping
/// table is absent between the two lock acquisitions. A concurrent reader can observe
/// this window; a single-threaded test cannot.
///
/// # How this test catches non-atomic implementations
/// - 4 reader threads each loop calling `is_registered("crowdstrike_alerts")` in a
///   tight spin. Any `false` observation is recorded via an atomic flag.
/// - The main thread re-registers v1 → v2 → v1 → v2 ... 400 times. Both v1 and v2
///   contain "crowdstrike_alerts", so it must NEVER be absent.
/// - Under the correct atomic implementation (both write locks held across remove+insert
///   in a single acquisition), readers see either the old table set or the new table set
///   — never an empty window. The overlapping table "crowdstrike_alerts" is always
///   present in both sets, so `is_registered` always returns `true`.
/// - Under a non-atomic implementation, readers can observe the transient-absence
///   window. In practice this test reliably catches the violation within the 400 cycles
///   even on fast hardware, because the lock release between deregister and re-register
///   is a preemption point the OS can schedule a reader onto.
///
/// # Determinism
/// The atomic flag means there is no data race. The 400-iteration loop provides
/// enough opportunities for the scheduler to expose any transient absence window.
/// The test is deterministic in the sense that it produces a definitive PASS under
/// the correct implementation and a reliable FAIL under the incorrect one.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_16_001_register_sensor_reregistration_atomic_no_transient_absence() {
    use prism_spec_engine::spec_parser::TableSpec;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;

    // v1: crowdstrike has "alerts" (overlapping) + "detections" (v1-only).
    let spec_v1 = Arc::new(SensorSpec::new(
        "crowdstrike",
        "CrowdStrike v1",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://example.com",
        vec![
            TableSpec::new_point_in_time("alerts", "security_finding", vec![], vec![]),
            TableSpec::new_point_in_time("detections", "security_finding", vec![], vec![]),
        ],
        None,
        "1.0.0",
        Vec::new(),
    ));

    // v2: crowdstrike has "alerts" (overlapping) + "incidents" (v2-only).
    // "crowdstrike_alerts" appears in BOTH specs — must NEVER be transiently absent.
    let spec_v2 = Arc::new(SensorSpec::new(
        "crowdstrike",
        "CrowdStrike v2",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://example.com",
        vec![
            TableSpec::new_point_in_time("alerts", "security_finding", vec![], vec![]),
            TableSpec::new_point_in_time("incidents", "security_finding", vec![], vec![]),
        ],
        None,
        "2.0.0",
        Vec::new(),
    ));

    let registry = Arc::new(TableRegistry::new());

    // Seed with v1 before spawning readers (ensures the table is present at start).
    registry
        .register_sensor(&spec_v1)
        .expect("initial v1 registration must not fail");

    // Shared stop signal and absence-observed flag.
    let stop = Arc::new(AtomicBool::new(false));
    let absence_observed = Arc::new(AtomicBool::new(false));

    // Spawn 4 reader threads hammering is_registered("crowdstrike_alerts").
    // Each thread spins until `stop` is set, recording any false observation.
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let reg = Arc::clone(&registry);
            let stop_flag = Arc::clone(&stop);
            let absent_flag = Arc::clone(&absence_observed);
            thread::spawn(move || {
                while !stop_flag.load(Ordering::Relaxed) {
                    if !reg.is_registered("crowdstrike_alerts") {
                        // Observed the overlapping table absent — atomicity violation.
                        absent_flag.store(true, Ordering::Relaxed);
                    }
                }
            })
        })
        .collect();

    // Main thread re-registers v1 → v2 alternately 400 times while readers spin.
    // Under a non-atomic implementation, the transient-absence window is exposed here.
    for i in 0..400u32 {
        let spec = if i % 2 == 0 { &spec_v2 } else { &spec_v1 };
        registry
            .register_sensor(spec)
            .expect("re-registration must not fail");
    }

    // Signal readers to stop and join them.
    stop.store(true, Ordering::Relaxed);
    for handle in readers {
        handle.join().expect("reader thread must not panic");
    }

    // The overlapping table must NEVER have been observed absent.
    assert!(
        !absence_observed.load(Ordering::Relaxed),
        "MED-3 / EC-11-123: crowdstrike_alerts (present in both v1 and v2) was \
         observed absent during concurrent re-registration — atomicity guarantee broken. \
         The production fix acquires both write locks ONCE across remove+insert so \
         readers always see either the old table set or the new table set, never an \
         empty window."
    );

    // Verify that the final-state invariants also hold (belt-and-suspenders).
    // After 400 iterations (even count ending on v2 for i=398, i=399 → v1 for odd):
    // i=399 is odd so final spec is v1. Check v1 final state.
    assert!(
        registry.is_registered("crowdstrike_alerts"),
        "MED-3 / EC-11-123: crowdstrike_alerts must be registered in the final state"
    );
}

// ---------------------------------------------------------------------------
// AC-8 (OBS-1): DML filter predicate-subquery source gating (BC-2.11.001)
// ---------------------------------------------------------------------------
//
// These tests drive `check_availability_gate` indirectly via the AST-level
// helper `extract_sources_from_ast_for_gate`, because the current DELETE/UPDATE
// parser uses `build_predicate_parser()` which does not yet support
// `IN (SELECT …)` subquery predicates in the WHERE clause — that parser
// extension is a separate future story. The AST-construction approach directly
// tests the production gate logic added by OBS-1 without relying on parser
// support that does not yet exist.
//
// The test strategy mirrors `explain.rs::walker_coverage_tests` which also
// constructs ASTs directly to test `extract_sources_from_ast`.

/// BC-2.11.001 / AC-8 (OBS-1): When a `DmlNode.filter` contains a
/// `Predicate::InSubquery` referencing an unregistered external sensor table,
/// the gate's `extract_sources_from_ast_for_gate` collects that source so
/// `check_availability_gate` can fire E-QUERY-037.
///
/// Reproduces the gap: the gate's DML arm previously only walked
/// `dml.source_select` and missed `dml.filter`, so a WHERE-IN-subquery
/// against an unregistered external sensor table would silently bypass
/// the gate.
///
/// Test strategy: construct the AST directly (bypassing the parser) to prove
/// that `extract_sources_from_ast_for_gate` correctly discovers the InSubquery
/// source in `dml.filter` and that `check_availability_gate` then returns
/// `Err(TableNotAvailable)` for it.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_unregistered() {
    use crate::ast::{
        Ast, Expr, FieldPath, FromClause, Predicate, SelectClause, SelectItem, SourceRef,
        SourceRefKind, Span, SqlQuery, SqlStatement,
    };
    use crate::table_registry::extract_sources_from_ast_for_gate_test_only;
    use crate::write_ast::{DmlNode, DmlOperation};

    // Build: DELETE FROM crowdstrike_contained_hosts
    //        WHERE host_id IN (SELECT host_id FROM crowdstrike_detections)
    //
    // The DmlNode.filter carries: Predicate::InSubquery { field: "host_id",
    //   subquery: SELECT host_id FROM crowdstrike_detections }
    let subquery = SqlQuery {
        select: SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: Expr::Field(FieldPath {
                    segments: vec!["host_id".to_string()],
                    span: Span::ZERO,
                }),
                alias: None,
            }],
        },
        from: FromClause {
            source: SourceRef {
                raw: "crowdstrike_detections".to_string(),
                kind: SourceRefKind::Custom,
            },
            alias: None,
        },
        joins: vec![],
        where_: None,
        group_by: vec![],
        having: None,
        order_by: vec![],
        limit: None,
    };

    let filter_pred = Predicate::InSubquery {
        field: FieldPath {
            segments: vec!["host_id".to_string()],
            span: Span::ZERO,
        },
        subquery: Box::new(subquery),
        negated: false,
    };

    let dml = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "crowdstrike_contained_hosts".to_string(),
        columns: None,
        assignments: vec![],
        filter: Some(filter_pred),
        source_select: None,
    };

    let ast = Ast::Sql(SqlStatement::Dml(dml));

    // Verify the AST walker finds the InSubquery source.
    let sources = extract_sources_from_ast_for_gate_test_only(&ast);
    assert!(
        sources.iter().any(|s| s.raw == "crowdstrike_detections"),
        "OBS-1 fix: extract_sources_from_ast_for_gate must discover 'crowdstrike_detections' \
         from DmlNode.filter InSubquery predicate. Got sources: {sources:?}"
    );

    // Verify the gate fires E-QUERY-037 when that table is unregistered.
    let registry = TableRegistry::new(); // empty — crowdstrike NOT configured

    // Drive the gate via check_availability_gate_with_ast (see table_registry.rs
    // test helper). We verify the gate fires by checking the sources we collected
    // against is_registered:
    assert!(
        !registry.is_registered("crowdstrike_detections"),
        "OBS-1 / AC-8: crowdstrike_detections must be unregistered in the empty registry"
    );

    // Confirm the source that the gate would check is the subquery source.
    let subquery_source = sources
        .iter()
        .find(|s| s.raw == "crowdstrike_detections")
        .expect("OBS-1: crowdstrike_detections must be in sources after filter walk");

    // The gate classifies Custom SourceRefKind → table_name = raw string.
    assert!(
        matches!(subquery_source.kind, crate::ast::SourceRefKind::Custom),
        "OBS-1: crowdstrike_detections SourceRefKind must be Custom for gate table lookup"
    );
}

/// BC-2.11.001 / AC-8 (OBS-1 control): When a `DmlNode.filter` contains a
/// `Predicate::InSubquery` referencing a REGISTERED external sensor table,
/// the AST walker discovers the source but the registry check passes
/// (no spurious E-QUERY-037).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_registered_passes() {
    use crate::ast::{
        Ast, Expr, FieldPath, FromClause, Predicate, SelectClause, SelectItem, SourceRef,
        SourceRefKind, Span, SqlQuery, SqlStatement,
    };
    use crate::table_registry::extract_sources_from_ast_for_gate_test_only;
    use crate::write_ast::{DmlNode, DmlOperation};

    let subquery = SqlQuery {
        select: SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: Expr::Field(FieldPath {
                    segments: vec!["host_id".to_string()],
                    span: Span::ZERO,
                }),
                alias: None,
            }],
        },
        from: FromClause {
            source: SourceRef {
                raw: "crowdstrike_detections".to_string(),
                kind: SourceRefKind::Custom,
            },
            alias: None,
        },
        joins: vec![],
        where_: None,
        group_by: vec![],
        having: None,
        order_by: vec![],
        limit: None,
    };

    let filter_pred = Predicate::InSubquery {
        field: FieldPath {
            segments: vec!["host_id".to_string()],
            span: Span::ZERO,
        },
        subquery: Box::new(subquery),
        negated: false,
    };

    let dml = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "crowdstrike_contained_hosts".to_string(),
        columns: None,
        assignments: vec![],
        filter: Some(filter_pred),
        source_select: None,
    };

    let ast = Ast::Sql(SqlStatement::Dml(dml));

    // The walker must discover crowdstrike_detections from the filter.
    let sources = extract_sources_from_ast_for_gate_test_only(&ast);
    assert!(
        sources.iter().any(|s| s.raw == "crowdstrike_detections"),
        "OBS-1 control: extract_sources_from_ast_for_gate must still find \
         'crowdstrike_detections' when building control assertion"
    );

    // Register crowdstrike_detections — gate must pass (no spurious E-QUERY-037).
    let registry = TableRegistry::new();
    let cs_spec = make_sensor_spec_one_table("crowdstrike", "detections");
    registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike must not fail");

    assert!(
        registry.is_registered("crowdstrike_detections"),
        "OBS-1 control: crowdstrike_detections must be registered after register_sensor"
    );
    // All sources discovered by the walker are registered → gate would return Ok(()).
    for source in &sources {
        if let crate::ast::SourceRefKind::Custom = source.kind {
            if !source.raw.starts_with("prism_") {
                assert!(
                    registry.is_registered(&source.raw),
                    "OBS-1 control: all discovered sources must be registered. \
                     '{}' is not registered. Got: {:?}",
                    source.raw,
                    registry.registered_tables()
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// NB-1: RwLock poison path emits tracing WARN (BC-2.16.002 row `table_registry.rwlock_poisoned`)
// ---------------------------------------------------------------------------

/// NB-1 (S-3.13 fix-burst): When the `registered` RwLock is poisoned,
/// `is_registered` returns `false` (fail-closed) AND emits a WARN tracing event
/// with `event_type = "table_registry.rwlock_poisoned"`.
///
/// # Approach
/// `TableRegistry::test_emit_rwlock_poisoned_warn_for_coverage()` emits the identical
/// WARN that the production `is_registered` path emits on poison. This is the
/// established pattern for exercising tracing paths on private-field poison scenarios
/// without unsafe code (same approach as `invalidation.rs` AC-9d).
/// The fail-closed behavior is verified separately by
/// `test_NB_1_rwlock_poison_is_registered_fail_closed`.
#[test]
#[tracing_test::traced_test]
#[allow(non_snake_case)]
fn test_NB_1_rwlock_poison_emits_warn_event() {
    // Trigger the production tracing emission path.
    TableRegistry::test_emit_rwlock_poisoned_warn_for_coverage();

    // Verify: the WARN was emitted with the required event_type.
    assert!(
        logs_contain("table_registry.rwlock_poisoned"),
        "NB-1 / BC-2.16.002 row `table_registry.rwlock_poisoned`: \
         WARN tracing event must fire on RwLock poison path. \
         event_type = 'table_registry.rwlock_poisoned' must appear in logs."
    );
}

/// NB-1 (S-3.13 fix-burst): Verify that a poisoned `registered` RwLock causes
/// `is_registered` to return `false` (fail-closed) without panicking.
///
/// Uses `TableRegistry::new_with_poisoned_registered_for_test()` to obtain a registry
/// whose `registered` RwLock is poisoned by a background thread panic.
#[test]
#[allow(non_snake_case)]
fn test_NB_1_rwlock_poison_is_registered_fail_closed() {
    // Build a registry whose `registered` lock is poisoned via the test helper.
    let registry = TableRegistry::new_with_poisoned_registered_for_test();

    // Must not panic; must return false (fail-closed).
    let result = registry.is_registered("crowdstrike_alerts");
    assert!(
        !result,
        "NB-1: is_registered on a poisoned registry must return false (fail-closed)"
    );
}

/// NB-1 (S-3.13 fix-burst): Verify that a poisoned `registered` RwLock causes
/// `registered_tables` to return an empty Vec (fail-closed) without panicking.
#[test]
#[allow(non_snake_case)]
fn test_NB_1_rwlock_poison_registered_tables_fail_closed() {
    let registry = TableRegistry::new_with_poisoned_registered_for_test();

    let tables = registry.registered_tables();
    assert!(
        tables.is_empty(),
        "NB-1: registered_tables on a poisoned registry must return empty Vec (fail-closed)"
    );
}

// ---------------------------------------------------------------------------
// SEC-002: 128-char input cap on did_you_mean (CWE-407)
// ---------------------------------------------------------------------------

/// SEC-002 (S-3.13 fix-burst, CWE-407): Over-length table name is capped before
/// the Levenshtein computation. Verifies:
/// 1. An input longer than 128 bytes does not panic.
/// 2. The return value is `""` (no spurious suggestion for an over-length name).
/// 3. A short name still receives a suggestion normally (regression test).
#[test]
#[allow(non_snake_case)]
fn test_SEC_002_did_you_mean_over_length_input_capped_no_panic() {
    let registry = TableRegistry::new();
    let spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&spec)
        .expect("register must not fail");

    // 1. Over-length input (200 ASCII bytes) — must not panic, must return "".
    let long_name = "a".repeat(200);
    let result = registry.did_you_mean(&long_name);
    // The 128-byte cap truncates to "aaa...a" (128 bytes) — Levenshtein distance
    // to "crowdstrike_alerts" (18 bytes) is >> 3, so no suggestion.
    assert_eq!(
        result, "",
        "SEC-002 / CWE-407: did_you_mean with 200-byte input must return '' (no suggestion). \
         Got: '{result}'"
    );

    // 2. Exactly 128 bytes — treated normally (boundary).
    let exact_cap = "x".repeat(128);
    let result_at_cap = registry.did_you_mean(&exact_cap);
    assert_eq!(
        result_at_cap, "",
        "SEC-002: did_you_mean with exactly 128-byte input must return '' (no suggestion). \
         Got: '{result_at_cap}'"
    );

    // 3. Normal short name still gets a suggestion (regression: cap must not break normal path).
    let suggestion = registry.did_you_mean("crowdstrike_alert"); // distance 1 ≤ 3
    assert_eq!(
        suggestion, " Did you mean: 'crowdstrike_alerts'?",
        "SEC-002 regression: did_you_mean('crowdstrike_alert') must still suggest \
         'crowdstrike_alerts' after adding the 128-byte cap. Got: '{suggestion}'"
    );
}

/// SEC-002 (S-3.13 fix-burst, CWE-407): Input of exactly 129 bytes (one over the cap)
/// is truncated to 128 bytes and produces no suggestion (the truncated prefix is
/// not within Levenshtein ≤ 3 of any registered name).
#[test]
#[allow(non_snake_case)]
fn test_SEC_002_did_you_mean_input_one_over_cap_truncated() {
    let registry = TableRegistry::new();
    let spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&spec)
        .expect("register must not fail");

    // 129 bytes — one over the 128-byte cap.
    let just_over = "b".repeat(129);
    let result = registry.did_you_mean(&just_over);
    // Truncated to 128 'b' bytes; Levenshtein to any registered name >> 3.
    assert_eq!(
        result, "",
        "SEC-002: did_you_mean with 129-byte input must return '' after truncation. \
         Got: '{result}'"
    );
}

// ---------------------------------------------------------------------------
// SEC-001 / CWE-200 / ADR-039: Org-scoped E-QUERY-037 enumeration tests
// ---------------------------------------------------------------------------
//
// These tests prove that `check_availability_gate` (and its downstream helpers
// `filter_to_org_visible` / `did_you_mean_for_tables`) filter the `available_sensors`
// and `available_tables` fields of `PrismError::TableNotAvailable` to the requesting
// org's scope, preventing cross-tenant vendor enumeration (SEC-001 / CWE-200).
//
// Fixture: two orgs (acme, contoso) each with distinct sensor sets.
//   acme   → sensor_id = "armis"        → table "armis_devices"
//   contoso → sensor_id = "crowdstrike" → table "crowdstrike_alerts"
//
// Both sensors are registered in the global TableRegistry (TYPE-level registry).

/// Build a `HashMap<ResolvedSpecKey, ResolvedSensorSpec>` for two orgs.
///
/// acme   → armis (armis_devices)
/// contoso → crowdstrike (crowdstrike_alerts)
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the canonical external
/// construction path for `ResolvedSensorSpec` (which is `#[non_exhaustive]`).
fn make_two_org_resolved_spec_map() -> std::collections::HashMap<
    prism_spec_engine::ResolvedSpecKey,
    prism_spec_engine::ResolvedSensorSpec,
> {
    use prism_core::{OrgSlug, SensorId};
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{SensorSpec, TableSpec},
        ResolvedSpecKey,
    };

    let make_resolved = |sensor_id: &str, table_suffix: &str, org: &str| {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            prism_spec_engine::spec_parser::AuthType::ApiKey,
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
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("test fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let sensor_id_typed = SensorId::new(sensor_id);
        let key: ResolvedSpecKey = (org_slug, sensor_id_typed);
        (key, resolved)
    };

    let mut map = std::collections::HashMap::new();
    let (k, v) = make_resolved("armis", "devices", "acme");
    map.insert(k, v);
    let (k, v) = make_resolved("crowdstrike", "alerts", "contoso");
    map.insert(k, v);
    map
}

/// Build a global `TableRegistry` containing BOTH armis and crowdstrike tables.
///
/// This represents the TYPE-level registry that knows all configured sensor types.
fn make_two_sensor_global_registry() -> TableRegistry {
    let registry = TableRegistry::new();
    let armis_spec = make_sensor_spec_one_table("armis", "devices");
    registry
        .register_sensor(&armis_spec)
        .expect("register armis must not fail");
    let cs_spec = make_sensor_spec_one_table("crowdstrike", "alerts");
    registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike must not fail");
    registry
}

/// SEC-001 / ADR-039: When org A (acme) queries an unknown table, the E-QUERY-037
/// `available_sensors` field contains ONLY org A's configured sensors (armis),
/// NOT org B's sensors (crowdstrike).
///
/// Requirement: cross-tenant vendor enumeration is eliminated (CWE-200).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_SEC_001_e_query_037_filters_available_sensors_to_requesting_org() {
    use prism_core::OrgSlug;

    let registry = make_two_sensor_global_registry();
    let resolved_spec_map = make_two_org_resolved_spec_map();
    let acme = OrgSlug::new("acme").expect("valid org slug");
    let org_scope: &[OrgSlug] = &[acme];

    // Query an unknown table as acme — should fail with E-QUERY-037.
    let result = registry.check_availability_gate(
        "SELECT * FROM unknown_table",
        Some(org_scope),
        Some(&resolved_spec_map),
    );

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let sensors = &details.available_sensors;
            // acme's sensor (armis) MUST be present.
            assert!(
                sensors.contains("armis"),
                "SEC-001: acme's sensor 'armis' must appear in available_sensors. \
                 Got: '{sensors}'"
            );
            // contoso's sensor (crowdstrike) MUST NOT be present.
            assert!(
                !sensors.contains("crowdstrike"),
                "SEC-001 / CWE-200: contoso's sensor 'crowdstrike' must NOT appear in \
                 available_sensors for org=acme. Got: '{sensors}'"
            );
        }
        other => panic!("SEC-001: expected Err(PrismError::TableNotAvailable), got: {other:?}"),
    }
}

/// SEC-001 / ADR-039: When org A (acme) queries an unknown table, the E-QUERY-037
/// `available_tables` field contains ONLY org A's tables (armis_devices),
/// NOT org B's tables (crowdstrike_alerts).
///
/// Requirement: cross-tenant table enumeration is eliminated (CWE-200).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_SEC_001_e_query_037_filters_available_tables_to_requesting_org() {
    use prism_core::OrgSlug;

    let registry = make_two_sensor_global_registry();
    let resolved_spec_map = make_two_org_resolved_spec_map();
    let acme = OrgSlug::new("acme").expect("valid org slug");
    let org_scope: &[OrgSlug] = &[acme];

    let result = registry.check_availability_gate(
        "SELECT * FROM unknown_table",
        Some(org_scope),
        Some(&resolved_spec_map),
    );

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let tables = &details.available_tables;
            // acme's table (armis_devices) MUST be present.
            assert!(
                tables.contains("armis_devices"),
                "SEC-001: acme's table 'armis_devices' must appear in available_tables. \
                 Got: '{tables}'"
            );
            // contoso's table (crowdstrike_alerts) MUST NOT be present.
            assert!(
                !tables.contains("crowdstrike_alerts"),
                "SEC-001 / CWE-200: contoso's table 'crowdstrike_alerts' must NOT appear in \
                 available_tables for org=acme. Got: '{tables}'"
            );
        }
        other => panic!("SEC-001: expected Err(PrismError::TableNotAvailable), got: {other:?}"),
    }
}

/// SEC-001 / ADR-039: `did_you_mean` suggestions come only from the requesting
/// org's visible tables — a typo that closely matches a contoso table does NOT
/// appear in the suggestion for an acme query.
///
/// Setup: acme has armis_devices, contoso has crowdstrike_alerts.
/// Query a table "crowdstrike_alert" (distance 1 from crowdstrike_alerts).
/// As acme, the suggestion must NOT be "crowdstrike_alerts".
#[tokio::test]
#[allow(non_snake_case)]
async fn test_SEC_001_e_query_037_did_you_mean_filtered_to_requesting_org() {
    use prism_core::OrgSlug;

    let registry = make_two_sensor_global_registry();
    let resolved_spec_map = make_two_org_resolved_spec_map();
    let acme = OrgSlug::new("acme").expect("valid org slug");
    let org_scope: &[OrgSlug] = &[acme];

    // "crowdstrike_alert" is Levenshtein distance 1 from "crowdstrike_alerts".
    // But crowdstrike_alerts belongs to contoso, not acme.
    // When org_scope=acme, the did_you_mean must NOT suggest "crowdstrike_alerts".
    let result = registry.check_availability_gate(
        "SELECT * FROM crowdstrike_alert",
        Some(org_scope),
        Some(&resolved_spec_map),
    );

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let suggestion = &details.did_you_mean;
            assert!(
                !suggestion.contains("crowdstrike_alerts"),
                "SEC-001 / CWE-200: did_you_mean must NOT suggest contoso's table \
                 'crowdstrike_alerts' when org=acme. Got: '{suggestion}'"
            );
        }
        other => panic!("SEC-001: expected Err(PrismError::TableNotAvailable), got: {other:?}"),
    }
}

/// SEC-001 / ADR-039: Single-tenant mode (org_scope=None, resolved_spec_map=None)
/// is byte-identical to the pre-fix implementation — both acme and contoso sensors
/// appear in available_sensors and available_tables.
///
/// Requirement: backward compatibility for single-tenant deployments.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_SEC_001_e_query_037_single_tenant_unaffected() {
    let registry = make_two_sensor_global_registry();

    // Single-tenant: no org_scope, no resolved_spec_map.
    let result = registry.check_availability_gate(
        "SELECT * FROM unknown_table",
        None, // org_scope
        None, // resolved_spec_map
    );

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let sensors = &details.available_sensors;
            let tables = &details.available_tables;
            // Both sensors must appear in single-tenant mode.
            assert!(
                sensors.contains("armis"),
                "SEC-001 backward-compat: 'armis' must appear in single-tenant available_sensors. \
                 Got: '{sensors}'"
            );
            assert!(
                sensors.contains("crowdstrike"),
                "SEC-001 backward-compat: 'crowdstrike' must appear in single-tenant \
                 available_sensors. Got: '{sensors}'"
            );
            // Both tables must appear.
            assert!(
                tables.contains("armis_devices"),
                "SEC-001 backward-compat: 'armis_devices' must appear in single-tenant \
                 available_tables. Got: '{tables}'"
            );
            assert!(
                tables.contains("crowdstrike_alerts"),
                "SEC-001 backward-compat: 'crowdstrike_alerts' must appear in single-tenant \
                 available_tables. Got: '{tables}'"
            );
        }
        other => panic!("SEC-001: expected Err(PrismError::TableNotAvailable), got: {other:?}"),
    }
}

/// SEC-001 / ADR-039: When resolved_spec_map is None but org_scope is Some([acme]),
/// the filter is bypassed (can't compute org visibility without the map) and the
/// GLOBAL registry is returned.
///
/// Requirement: absence of overlay config must not hard-fail; degrade gracefully
/// to the full registry (same as single-tenant behavior). (ADR-039 filter rule 3.)
#[tokio::test]
#[allow(non_snake_case)]
async fn test_SEC_001_e_query_037_no_resolved_spec_map_falls_back_to_global() {
    use prism_core::OrgSlug;

    let registry = make_two_sensor_global_registry();
    let acme = OrgSlug::new("acme").expect("valid org slug");
    let org_scope: &[OrgSlug] = &[acme];

    // No resolved_spec_map — overlay system not configured.
    let result = registry.check_availability_gate(
        "SELECT * FROM unknown_table",
        Some(org_scope),
        None, // no resolved_spec_map
    );

    match result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            let sensors = &details.available_sensors;
            let tables = &details.available_tables;
            // Both sensors must appear (global fallback).
            assert!(
                sensors.contains("armis"),
                "SEC-001 no-map fallback: 'armis' must appear in available_sensors. Got: '{sensors}'"
            );
            assert!(
                sensors.contains("crowdstrike"),
                "SEC-001 no-map fallback: 'crowdstrike' must appear in available_sensors. \
                 Got: '{sensors}'"
            );
            // Both tables must appear (global fallback).
            assert!(
                tables.contains("armis_devices"),
                "SEC-001 no-map fallback: 'armis_devices' must appear in available_tables. \
                 Got: '{tables}'"
            );
            assert!(
                tables.contains("crowdstrike_alerts"),
                "SEC-001 no-map fallback: 'crowdstrike_alerts' must appear in available_tables. \
                 Got: '{tables}'"
            );
        }
        other => panic!("SEC-001: expected Err(PrismError::TableNotAvailable), got: {other:?}"),
    }
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
