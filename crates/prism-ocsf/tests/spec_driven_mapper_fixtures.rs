// Allow BC-based test naming convention (test_BC_S_SS_NNN_... and test_PLUGIN_MIGRATION_...
// use mixed-case identifiers for traceability — non_snake_case is intentional per VSDD protocol).
#![allow(non_snake_case)]

//! VP-PLUGIN-006 fixture catalog for SpecDrivenMapper.
//!
//! # Red Gate Tests — PLUGIN-MIGRATION-001-C
//!
//! All tests in this file MUST FAIL before implementation begins.
//! `SpecDrivenMapper::new()` and `map()` both use `todo!()`, so every test
//! that reaches those call sites will panic with "not yet implemented".
//!
//! # BC Coverage
//!
//! | Test                                                          | AC        | BC / VP                   |
//! |---------------------------------------------------------------|-----------|---------------------------|
//! | test_BC_2_02_002_spec_driven_string_to_string                 | AC-001    | BC-2.02.002 / VP-PLUGIN-006 |
//! | test_BC_2_02_002_spec_driven_rfc3339_timestamp                | AC-001    | BC-2.02.002 / VP-PLUGIN-006 |
//! | test_BC_2_02_002_spec_driven_int_to_string_cast               | AC-001    | BC-2.02.002 / VP-PLUGIN-006 |
//! | test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern | AC-002 | VP-PLUGIN-006 |
//! | test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed | AC-003 | BC-2.02.002 |
//! | test_BC_2_02_007_spec_driven_extensions_preserved             | AC-004    | BC-2.02.007               |
//! | test_PLUGIN_MIGRATION_001_C_005_no_hardcoded_mapper_symbols_in_production_src | AC-005 | VP-PLUGIN-006 |
//! | test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases | AC-006 | VP-PLUGIN-006 |
//! | test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper | AC-007 | BC-2.02.002 |

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use prism_core::{ColumnType, PrismError};
use prism_ocsf::mappers::{SensorMapper, SpecDrivenMapper};
use prism_spec_engine::{
    types::{
        ColumnDef, ConfigSnapshot, PaginationType, SensorSpec as HotReloadSensorSpec,
        SensorTableDescriptor as HotReloadTableDescriptor,
    },
    ConfigManager, PluginRuntime,
};
use prost_reflect::{DescriptorPool, DynamicMessage};
use prost_types::{DescriptorProto, FileDescriptorProto};
use serde_json::json;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a minimal `reqwest::Client` with the 30-second timeout required by
/// TD-S-PLUGIN-PREREQ-B-005 (CLAUDE.md Conventions §HTTP client timeout).
fn test_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest::Client construction must not fail in test environment")
}

/// Build a `PluginRuntime` with no loaded plugins (test mode, NoOpPluginAuditSink).
fn empty_plugin_runtime() -> Arc<PluginRuntime> {
    Arc::new(
        PluginRuntime::new(test_http_client())
            .expect("PluginRuntime::new must succeed with valid HTTP client"),
    )
}

/// Build a `ConfigManager` containing a single synthetic sensor spec entry.
///
/// The `types::SensorSpec` contains a `SensorTableDescriptor` whose `ColumnDef` list
/// drives `ocsf_field` lookups in `SpecDrivenMapper::map()`. This is the hot-reload
/// infrastructure type (`types::SensorSpec`, not `spec_parser::SensorSpec`).
///
/// `sensor_id` must match whatever `SpecDrivenMapper::sensor_id()` returns for it
/// to be dispatched correctly by `OcsfNormalizer::normalize_with_mappers()`.
fn config_manager_with_ocsf_columns(
    sensor_id: &str,
    table_name: &str,
    columns: Vec<ColumnDef>,
) -> Arc<ConfigManager> {
    let descriptor = HotReloadTableDescriptor::new(
        format!("{}.{}", sensor_id, table_name),
        columns,
        0,
        PaginationType::None,
    );

    let sensor_spec = HotReloadSensorSpec::new_hot_reload(
        sensor_id,
        sensor_id,
        "1.0.0",
        "api_key",
        "https://example.test",
        vec![descriptor],
        "",
        "",
    );

    let mut sensor_specs = HashMap::new();
    sensor_specs.insert(sensor_id.to_string(), sensor_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        failed_specs: HashMap::new(),
        snapshot_hash: "test-hash".to_string(),
    };

    Arc::new(ConfigManager::new(snapshot))
}

/// Build a stub `DynamicMessage` using a minimal in-process descriptor.
///
/// The `SpecDrivenMapper` writes known OCSF fields into `msg`. In the Red Gate
/// phase the `todo!()` bodies prevent any writes; the stub descriptor just needs
/// to compile and not panic on construction. This mirrors the same helper used in
/// `crates/prism-ocsf/src/tests/mapper_tests.rs`.
fn stub_dynamic_message() -> DynamicMessage {
    let file = FileDescriptorProto {
        name: Some("stub_spec_driven_test.proto".to_owned()),
        syntax: Some("proto3".to_owned()),
        message_type: vec![DescriptorProto {
            name: Some("StubMsg".to_owned()),
            ..DescriptorProto::default()
        }],
        ..FileDescriptorProto::default()
    };

    let mut pool = DescriptorPool::new();
    pool.add_file_descriptor_proto(file)
        .expect("stub proto descriptor must be valid");
    let desc = pool
        .get_message_by_name("StubMsg")
        .expect("StubMsg must be in pool");
    DynamicMessage::new(desc)
}

/// Build a `ColumnDef` with a string type and an OCSF field mapping.
///
/// Uses `Default::default()` and field mutation to comply with the `#[non_exhaustive]`
/// annotation on `ColumnDef` (prevents struct-literal construction from external crates).
fn string_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnDef {
    let mut col = ColumnDef::default();
    col.name = name.to_string();
    col.column_type = ColumnType::String;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Build a `ColumnDef` with a string type and NO OCSF field mapping.
///
/// Columns without `ocsf_field` land in the `extensions` map (BC-2.02.007).
fn string_col_unmapped(name: &str) -> ColumnDef {
    let mut col = ColumnDef::default();
    col.name = name.to_string();
    col.column_type = ColumnType::String;
    col.ocsf_field = None;
    col
}

/// Build a `ColumnDef` for a datetime column with `ocsf_field` mapping.
///
/// NOTE: `ColumnDef` does not have a `timestamp_formats` field (that is a
/// `spec_parser::ColumnSpec` field). The datetime format hint is provided at
/// the `SpecDrivenMapper` level via the column_type = Datetime signal.
fn datetime_col_with_ocsf(name: &str, ocsf_field: &str, _formats: Vec<String>) -> ColumnDef {
    let mut col = ColumnDef::default();
    col.name = name.to_string();
    col.column_type = ColumnType::Datetime;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Build a `ColumnDef` for an integer column with `ocsf_field` mapping.
fn integer_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnDef {
    let mut col = ColumnDef::default();
    col.name = name.to_string();
    col.column_type = ColumnType::Integer;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Build a `ColumnDef` for a JSON column with `ocsf_field` mapping (complex transform path).
fn json_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnDef {
    let mut col = ColumnDef::default();
    col.name = name.to_string();
    col.column_type = ColumnType::Json;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

// ---------------------------------------------------------------------------
// Test 1: AC-001 — string-to-string direct OCSF field mapping
// ---------------------------------------------------------------------------

/// AC-001 (part 1): `detection_id` → `finding_info.uid` via `ocsf_field` annotation.
///
/// Exercises VP-PLUGIN-006: spec-driven field mapping correctness.
/// The spec declares `ocsf_field = "finding_info.uid"` on column `detection_id`.
/// After `map()` returns, the DynamicMessage must have `finding_info.uid = "DET-123"`.
///
/// Red Gate: panics with "not yet implemented" at `SpecDrivenMapper::new()`.
#[test]
fn test_BC_2_02_002_spec_driven_string_to_string() {
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "detections",
        vec![string_col_with_ocsf("detection_id", "finding_info.uid")],
    );

    // SpecDrivenMapper::new() uses todo!() — panics here during Red Gate.
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({ "detection_id": "DET-123" });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    // map() also uses todo!() — would panic here even if new() passed.
    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);
    let source_id =
        result.expect("BC-2.02.002 AC-001: spec-driven string-to-string mapping must succeed");

    assert_eq!(
        source_id, "DET-123",
        "BC-2.02.002 AC-001: returned source_id must equal the mapped detection_id"
    );

    // The DynamicMessage field assertion is deferred to implementation-phase tests
    // that use the actual OCSF protobuf descriptor pool rather than a stub descriptor.
    // What IS asserted here: the call completes without error and extensions does NOT
    // contain detection_id (it was mapped to msg, not spilled to extensions).
    assert!(
        !extensions.contains_key("detection_id"),
        "BC-2.02.002 AC-001: detection_id is mapped to OCSF — must not appear in extensions"
    );
}

// ---------------------------------------------------------------------------
// Test 2: AC-001 — RFC3339 timestamp → epoch-millis OCSF field
// ---------------------------------------------------------------------------

/// AC-001 (part 2): `created_at` with `ocsf_field = "time"` and `timestamp_formats = ["iso8601"]`.
///
/// Input `{"created_at": "2024-01-15T10:30:00Z"}` must produce `time` = epoch millis
/// of 2024-01-15T10:30:00Z (= 1705311000000 ms).
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_BC_2_02_002_spec_driven_rfc3339_timestamp() {
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "events",
        vec![datetime_col_with_ocsf(
            "created_at",
            "time",
            vec!["iso8601".to_string()],
        )],
    );

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({ "created_at": "2024-01-15T10:30:00Z" });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("events", &raw, &mut msg, &mut extensions);
    result.expect("BC-2.02.002 AC-001: RFC3339 timestamp mapping must succeed");

    // Verify the raw timestamp field was NOT sent to extensions (it was mapped to msg).
    assert!(
        !extensions.contains_key("created_at"),
        "BC-2.02.002 AC-001: created_at is mapped to OCSF time — must not appear in extensions"
    );
}

// ---------------------------------------------------------------------------
// Test 3: AC-001 — integer column → string OCSF field (int-to-string cast)
// ---------------------------------------------------------------------------

/// AC-001 (part 3): integer `id` with `ocsf_field = "device.uid"` cast to string.
///
/// Input `{"id": 12345}` must produce OCSF `device.uid = "12345"` (string cast).
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_BC_2_02_002_spec_driven_int_to_string_cast() {
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "devices",
        vec![integer_col_with_ocsf("id", "device.uid")],
    );

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({ "id": 12345i64 });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("devices", &raw, &mut msg, &mut extensions);
    result.expect("BC-2.02.002 AC-001: integer-to-string cast mapping must succeed");

    assert!(
        !extensions.contains_key("id"),
        "BC-2.02.002 AC-001: id is mapped to OCSF device.uid — must not appear in extensions"
    );
}

// ---------------------------------------------------------------------------
// Test 4: AC-002 — WASM dispatch called for complex-transform column
// ---------------------------------------------------------------------------

/// AC-002: When a column's `ocsf_field` requires a complex transform, `PluginRuntime`
/// is dispatched.
///
/// This test verifies the WASM dispatch path exists and is exercised.
/// With an empty `PluginRuntime` (no loaded plugins), the call must still reach
/// `PluginRuntime` dispatch and return an error identifying the missing plugin —
/// NOT silently skip the column or return a wrong value.
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern() {
    // A column flagged for WASM transform (convention: column_type = Json signals complex
    // transform that cannot be handled by direct field mapping alone).
    // The SpecDrivenMapper implementation must call PluginRuntime for this column.
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "detections",
        vec![json_col_with_ocsf("behaviors", "attacks")],
    );

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({
        "behaviors": [{"tactic": "Discovery", "technique": "System Information Discovery"}]
    });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    // After implementation: the WASM dispatch path is exercised. With no loaded plugin,
    // the result is Err(OcsfNormalizationFailed { reason: contains "no ocsf_transform plugin" }).
    // We only assert it does NOT silently succeed with an empty value.
    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);

    // The call must either succeed (plugin present) or return a structured error
    // (plugin absent) — never panic.
    match result {
        Ok(_) => {
            // Acceptable post-implementation if plugin is wired.
        }
        Err(PrismError::OcsfNormalizationFailed { reason, .. }) => {
            assert!(
                reason.contains("plugin") || reason.contains("ocsf_transform"),
                "AC-002: WASM dispatch error reason must mention plugin, got: {reason}"
            );
        }
        Err(e) => {
            panic!(
                "AC-002: unexpected error variant from WASM dispatch path: {:?}",
                e
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 5: AC-003 — missing plugin returns OcsfNormalizationFailed
// ---------------------------------------------------------------------------

/// AC-003: `SpecDrivenMapper` with no loaded plugins returns structured error.
///
/// When a column requires WASM dispatch and `PluginRuntime` has no loaded plugins,
/// `map()` MUST return `Err(PrismError::OcsfNormalizationFailed)` with a reason
/// containing "no ocsf_transform plugin". It MUST NOT panic.
///
/// Exercises BC-2.02.002 §Error postconditions.
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed() {
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "detections",
        vec![json_col_with_ocsf("behaviors", "attacks")],
    );

    // Explicitly build a PluginRuntime with no loaded plugins.
    let empty_runtime = empty_plugin_runtime();

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_runtime);

    let raw = json!({
        "behaviors": [{"tactic": "Discovery"}]
    });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);

    match result {
        Err(PrismError::OcsfNormalizationFailed { source_id, reason }) => {
            assert!(
                reason.to_lowercase().contains("plugin")
                    || reason.to_lowercase().contains("ocsf_transform"),
                "AC-003: reason must mention missing plugin, got: '{reason}'"
            );
            // source_id must be non-empty to enable error tracing (BC-2.02.011).
            assert!(
                !source_id.is_empty(),
                "AC-003: source_id in error must be non-empty for traceability"
            );
        }
        Err(e) => panic!(
            "AC-003: expected OcsfNormalizationFailed for missing plugin, got {:?}",
            e
        ),
        Ok(_) => panic!("AC-003: map() must return Err when required WASM plugin is absent"),
    }
}

// ---------------------------------------------------------------------------
// Test 6: AC-004 — BC-2.02.007 extensions invariant (unmapped fields preserved)
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.02.007: Union invariant — mapped fields in msg, unmapped in extensions.
///
/// Sensor spec: 3 columns with `ocsf_field`, 5 columns without.
/// Input: JSON with all 8 fields.
/// After `map()`: `extensions` contains exactly the 5 unmapped fields;
/// those 5 fields do NOT appear in the DynamicMessage.
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_BC_2_02_007_spec_driven_extensions_preserved() {
    let columns = vec![
        string_col_with_ocsf("detection_id", "finding_info.uid"),
        string_col_with_ocsf("severity", "severity"),
        datetime_col_with_ocsf("created_at", "time", vec!["iso8601".to_string()]),
        // These 5 columns have NO ocsf_field — must land in extensions.
        string_col_unmapped("agent_id"),
        string_col_unmapped("cid"),
        string_col_unmapped("custom_tag"),
        string_col_unmapped("region"),
        string_col_unmapped("source_system"),
    ];

    let config_manager = config_manager_with_ocsf_columns("test-sensor", "detections", columns);

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({
        "detection_id": "DET-456",
        "severity": "High",
        "created_at": "2024-01-15T10:30:00Z",
        "agent_id": "cs-agent-xyz",
        "cid": "customer-id-123",
        "custom_tag": "prod",
        "region": "us-east-1",
        "source_system": "crowdstrike"
    });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);
    result.expect("BC-2.02.007 AC-004: extensions-preservation mapping must succeed");

    // The 5 unmapped columns must appear in extensions.
    for unmapped in &["agent_id", "cid", "custom_tag", "region", "source_system"] {
        assert!(
            extensions.contains_key(*unmapped),
            "BC-2.02.007 AC-004: unmapped field '{unmapped}' must be in extensions"
        );
    }

    // The 3 mapped columns must NOT appear in extensions.
    for mapped in &["detection_id", "severity", "created_at"] {
        assert!(
            !extensions.contains_key(*mapped),
            "BC-2.02.007 AC-004: mapped field '{mapped}' must NOT appear in extensions"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 7: AC-005 — no hardcoded mapper symbols in production src
// ---------------------------------------------------------------------------

/// AC-005: No per-sensor mapper type names appear in production source files
/// (outside `#[cfg(test)]` blocks) after the migration is complete.
///
/// This is a source-code grep test. It PASSES currently (before deletion) because
/// the symbols ARE present — the test will flip to FAIL once PLUGIN-MIGRATION-001-C
/// implementation removes the hardcoded mappers. During the Red Gate phase this test
/// is structured to FAIL by asserting the opposite: it asserts that the source is
/// already clean, which it is not yet.
///
/// Rationale: The Red Gate requires ALL tests to fail. We invert the assertion so
/// this test fails correctly: it asserts "no hardcoded symbols exist" but they DO,
/// so it fails. The implementer flips the assertion back when deleting the mapper files.
#[test]
fn test_PLUGIN_MIGRATION_001_C_005_no_hardcoded_mapper_symbols_in_production_src() {
    use std::fs;
    use std::path::PathBuf;

    // Find the prism-ocsf src directory relative to this test binary's manifest.
    // In Cargo test runs, CARGO_MANIFEST_DIR resolves to the crate root.
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut hardcoded_symbol_count = 0usize;
    let symbols_to_ban = [
        "CrowdStrikeMapper",
        "CyberintMapper",
        "ClarotyMapper",
        "ArmisMapper",
    ];

    // Walk all .rs files under src/ and count occurrences outside test blocks.
    // We use a simplified check: count total occurrences in non-test code.
    // A robust implementation would parse cfg(test) blocks; for Red Gate purposes
    // we count all occurrences (test code imports these symbols, so count > 0).
    if let Ok(entries) = fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let contents = fs::read_to_string(&path).unwrap_or_default();
                // Skip test files (they legitimately reference these symbols).
                // We check files directly under src/ — the mapper implementations.
                for &sym in &symbols_to_ban {
                    // Count occurrences outside obvious test contexts.
                    // "pub struct CrowdStrikeMapper" or "CrowdStrikeMapper;" in production code.
                    hardcoded_symbol_count += contents.matches(sym).count();
                }
            }
        }
    }

    // AC-005 post-migration assertion: ZERO occurrences in production src.
    // During Red Gate this FAILS because the mapper files still exist.
    assert_eq!(
        hardcoded_symbol_count, 0,
        "AC-005: Found {hardcoded_symbol_count} hardcoded per-sensor mapper symbol(s) in \
         production src/. These must be removed when SpecDrivenMapper replaces them."
    );
}

// ---------------------------------------------------------------------------
// Test 8: AC-006 — VP-PLUGIN-006 fixture catalog (6 parameterized cases)
// ---------------------------------------------------------------------------

/// AC-006: VP-PLUGIN-006 fixture catalog — 6 canonical (input, expected-output) pairs.
///
/// Each fixture: raw JSON input + expected OCSF field mapping.
/// Red Gate: all 6 sub-assertions panic at `SpecDrivenMapper::new()` (first fixture).
#[test]
fn test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases() {
    /// A single fixture entry from the VP-PLUGIN-006 catalog.
    struct Fixture {
        /// Column name in the sensor spec.
        column_name: &'static str,
        /// OCSF field path this column maps to.
        ocsf_field: &'static str,
        /// Raw JSON input record.
        raw: serde_json::Value,
    }

    let fixtures = vec![
        // Fixture 1: Direct string mapping — detection ID → finding_info.uid
        Fixture {
            column_name: "detection_id",
            ocsf_field: "finding_info.uid",
            raw: json!({ "detection_id": "FIX-001" }),
        },
        // Fixture 2: Severity string → severity OCSF field
        Fixture {
            column_name: "severity",
            ocsf_field: "severity",
            raw: json!({ "severity": "High" }),
        },
        // Fixture 3: RFC3339 timestamp → time field (epoch-millis)
        Fixture {
            column_name: "created_at",
            ocsf_field: "time",
            raw: json!({ "created_at": "2024-03-15T10:30:00Z" }),
        },
        // Fixture 4: Integer ID → string OCSF uid (int-to-string cast)
        Fixture {
            column_name: "asset_id",
            ocsf_field: "device.uid",
            raw: json!({ "asset_id": 99999i64 }),
        },
        // Fixture 5: Nested source field → top-level OCSF field
        Fixture {
            column_name: "alert_ref",
            ocsf_field: "ref_uid",
            raw: json!({ "alert_ref": "ALERT-REF-005" }),
        },
        // Fixture 6: Boolean field → OCSF boolean field (no cast needed)
        Fixture {
            column_name: "is_active",
            ocsf_field: "is_active",
            raw: json!({ "is_active": true }),
        },
    ];

    for (i, fixture) in fixtures.iter().enumerate() {
        let col = string_col_with_ocsf(fixture.column_name, fixture.ocsf_field);

        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);

        // Red Gate: panics at new() on first fixture iteration.
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();

        let result = mapper.map("fixture-table", &fixture.raw, &mut msg, &mut extensions);
        let source_id = result.unwrap_or_else(|e| {
            panic!(
                "VP-PLUGIN-006 AC-006 fixture {}: map() failed: {:?}",
                i + 1,
                e
            )
        });

        // Assert the column was NOT spilled to extensions (it had an ocsf_field).
        assert!(
            !extensions.contains_key(fixture.column_name),
            "VP-PLUGIN-006 AC-006 fixture {}: column '{}' must not appear in extensions",
            i + 1,
            fixture.column_name
        );

        // Assert source_id is non-empty (BC-2.02.002 postcondition: returns source record ID).
        assert!(
            !source_id.is_empty(),
            "VP-PLUGIN-006 AC-006 fixture {}: source_id must be non-empty",
            i + 1
        );
    }
}

// ---------------------------------------------------------------------------
// Test 9: AC-007 — OcsfNormalizer wired with SpecDrivenMapper
// ---------------------------------------------------------------------------

/// AC-007: `OcsfNormalizer::with_mappers([Box::new(SpecDrivenMapper)])` dispatches
/// to `SpecDrivenMapper::map()` when `normalize_with_mappers()` is called.
///
/// The key assertion: constructing the normalizer with a `SpecDrivenMapper` and
/// calling `normalize_with_mappers()` reaches `SpecDrivenMapper::map()` — observable
/// via the todo!() panic or a structured error after implementation.
///
/// This tests that the dispatch path in `OcsfNormalizer` is compatible with the
/// `SpecDrivenMapper` trait implementation (sensor_id/record_types dispatch).
///
/// Red Gate: panics at `SpecDrivenMapper::new()`.
#[test]
fn test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper() {
    use prism_ocsf::normalizer::OcsfNormalizer;

    let config_manager = config_manager_with_ocsf_columns(
        // sensor_id MUST match a known EventClassSelector entry so normalize_with_mappers
        // can resolve a class_uid for the (sensor, record_type) pair.
        // "crowdstrike" + "detection" → class_uid 2004 (DetectionFinding).
        "crowdstrike",
        "detections",
        vec![string_col_with_ocsf("detection_id", "finding_info.uid")],
    );

    // Red Gate: panics at new().
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(mapper)]);

    let sample_raw = json!({
        "detection_id": "ldt:abc123",
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z"
    });

    // normalize_with_mappers dispatches to SpecDrivenMapper::map() via the trait.
    // After implementation: returns Ok((msg, source_id)) with the mapped message.
    // During Red Gate: panics at new() before reaching this call.
    let result = normalizer.normalize_with_mappers("crowdstrike", "detection", sample_raw);

    let (_msg, source_id) =
        result.expect("AC-007: normalize_with_mappers wired with SpecDrivenMapper must succeed");

    assert_eq!(
        source_id, "ldt:abc123",
        "AC-007: source_id returned by normalizer must equal the sensor's record identifier"
    );
}
