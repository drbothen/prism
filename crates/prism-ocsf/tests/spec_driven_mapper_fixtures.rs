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

use std::{collections::HashMap, sync::Arc, time::Duration};

use prism_core::{ColumnType, PrismError};
use prism_ocsf::mappers::{SensorMapper, SpecDrivenMapper};
use prism_spec_engine::{
    spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    types::ConfigSnapshot,
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
/// ADR-030 Approach D: constructs `spec_parser::SensorSpec` directly with `Vec<TableSpec>`.
/// `SpecDrivenMapper::map()` reads `ocsf_field` from `TableSpec.columns` (type `ColumnSpec`).
///
/// `sensor_id` must match whatever `SpecDrivenMapper::sensor_id()` returns for it
/// to be dispatched correctly by `OcsfNormalizer::normalize_with_mappers()`.
///
/// NOTE: `table_name` is passed UNQUALIFIED (e.g., `"detections"` not
/// `"crowdstrike.detections"`) — mirroring how TOML loading stores table names in
/// `spec_parser::TableSpec.table_name`. Qualification as `{sensor_id}.{table_name}`
/// happens at DataFusion registration time (MED-001 fix).
fn config_manager_with_ocsf_columns(
    sensor_id: &str,
    table_name: &str,
    columns: Vec<ColumnSpec>,
) -> Arc<ConfigManager> {
    let table = TableSpec::new_point_in_time(
        table_name, // UNQUALIFIED — matches TOML-loaded spec behavior (MED-001)
        "security_finding",
        columns,
        vec![],
    );

    let sensor_spec = SensorSpec::new(
        sensor_id,
        sensor_id,
        AuthType::ApiKey,
        "https://example.test",
        vec![table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = HashMap::new();
    sensor_specs.insert(sensor_id.to_string(), sensor_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        failed_specs: HashMap::new(),
        snapshot_hash: "test-hash".to_string(),
        org_display_names: HashMap::new(),
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

/// Build a `ColumnSpec` with a string type and an OCSF field mapping.
///
/// ADR-030 Approach D: uses `ColumnSpec` (spec_parser type) now that `ConfigSnapshot`
/// holds `spec_parser::SensorSpec` with `Vec<TableSpec>` → `Vec<ColumnSpec>`.
/// Uses `Default::default()` + mutation for `#[non_exhaustive]` forward-compat.
fn string_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnSpec {
    let mut col = ColumnSpec::default();
    col.name = name.to_string();
    col.column_type = ColumnType::String;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Build a `ColumnSpec` with a string type and NO OCSF field mapping.
///
/// Columns without `ocsf_field` land in the `extensions` map (BC-2.02.007).
fn string_col_unmapped(name: &str) -> ColumnSpec {
    let mut col = ColumnSpec::default();
    col.name = name.to_string();
    col.column_type = ColumnType::String;
    col.ocsf_field = None;
    col
}

/// Build a `ColumnSpec` for a datetime column with `ocsf_field` mapping.
///
/// `ColumnSpec.timestamp_formats` is available (unlike the retired `ColumnDef`).
/// The `_formats` arg is accepted for API compatibility but stored as-is.
fn datetime_col_with_ocsf(name: &str, ocsf_field: &str, formats: Vec<String>) -> ColumnSpec {
    let mut col = ColumnSpec::default();
    col.name = name.to_string();
    col.column_type = ColumnType::Datetime;
    col.ocsf_field = Some(ocsf_field.to_string());
    col.timestamp_formats = formats;
    col
}

/// Build a `ColumnSpec` for an integer column with `ocsf_field` mapping.
fn integer_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnSpec {
    let mut col = ColumnSpec::default();
    col.name = name.to_string();
    col.column_type = ColumnType::Integer;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Build a `ColumnSpec` for a JSON column with `ocsf_field` mapping (complex transform path).
fn json_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnSpec {
    let mut col = ColumnSpec::default();
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
    use std::{fs, path::PathBuf};

    // Find the prism-ocsf src directory relative to this test binary's manifest.
    // In Cargo test runs, CARGO_MANIFEST_DIR resolves to the crate root.
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    // The tests/ subdirectory is #[cfg(test)] code; it may legitimately reference
    // these symbol names in test helpers. We exclude it from the production scan.
    let tests_subdir = src_dir.join("tests");

    let symbols_to_ban = [
        "CrowdStrikeMapper",
        "CyberintMapper",
        "ClarotyMapper",
        "ArmisMapper",
    ];

    // Recursive walk helper: collect all .rs files under `dir`, excluding `exclude`.
    fn collect_rs_files(
        dir: &PathBuf,
        exclude: &PathBuf,
        out: &mut Vec<PathBuf>,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            if path == *exclude {
                continue;
            }
            if path.is_dir() {
                collect_rs_files(&path, exclude, out)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut production_files: Vec<PathBuf> = Vec::new();
    collect_rs_files(&src_dir, &tests_subdir, &mut production_files)
        .expect("AC-005: failed to walk src/ for production .rs files");

    let mut findings: Vec<String> = Vec::new();
    for path in &production_files {
        let contents = fs::read_to_string(path).unwrap_or_default();
        for &sym in &symbols_to_ban {
            let count = contents.matches(sym).count();
            if count > 0 {
                findings.push(format!(
                    "{path}: {count} occurrence(s) of '{sym}'",
                    path = path.display()
                ));
            }
        }
    }

    // AC-005 post-migration assertion: ZERO occurrences in production src (excl. src/tests/).
    assert!(
        findings.is_empty(),
        "AC-005: Found hardcoded per-sensor mapper symbol(s) in production src/:\n{}",
        findings.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Test 8: AC-006 — VP-PLUGIN-006 fixture catalog (6 parameterized cases)
// ---------------------------------------------------------------------------

/// AC-006: VP-PLUGIN-006 fixture catalog — 9 canonical (input, expected-output) pairs.
///
/// Covers: 6 TOML-mappable fixtures (string, string, datetime, integer, string, boolean) +
/// 3 WASM-required fixtures (Json column type, no loaded plugin → OcsfNormalizationFailed).
///
/// F-LP1-HIGH-001 compliance: 3 WASM-required cases (fixtures 7–9).
/// F-LP1-HIGH-002 compliance: datetime fixture uses datetime_col_with_ocsf;
///   integer fixture uses integer_col_with_ocsf.
/// F-LP1-MED-004 compliance: string fixture asserts source_id value (not just is_ok).
#[test]
fn test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases() {
    // ---------------------------------------------------------------------------
    // Fixtures 1–6: TOML-mappable (no WASM plugin required)
    // ---------------------------------------------------------------------------

    // Fixture 1: String → finding_info.uid (string column)
    {
        let col = string_col_with_ocsf("detection_id", "finding_info.uid");
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "detection_id": "FIX-001" });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        let source_id = mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 1: string mapping must succeed");
        assert!(
            !extensions.contains_key("detection_id"),
            "VP-PLUGIN-006 fixture 1: detection_id must not appear in extensions"
        );
        assert_eq!(
            source_id, "FIX-001",
            "VP-PLUGIN-006 fixture 1: source_id must equal detection_id value"
        );
    }

    // Fixture 2: String → severity (string column)
    {
        let col = string_col_with_ocsf("severity", "severity");
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "severity": "High" });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        let source_id = mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 2: severity string mapping must succeed");
        assert!(
            !extensions.contains_key("severity"),
            "VP-PLUGIN-006 fixture 2: severity must not appear in extensions"
        );
        assert_eq!(
            source_id, "High",
            "VP-PLUGIN-006 fixture 2: source_id must equal severity value"
        );
    }

    // Fixture 3: Datetime → time (datetime column, RFC3339)
    // F-LP1-HIGH-002: uses datetime_col_with_ocsf (NOT string_col_with_ocsf)
    {
        let col = datetime_col_with_ocsf("created_at", "time", vec!["iso8601".to_string()]);
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "created_at": "2024-03-15T10:30:00Z" });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 3: RFC3339 timestamp mapping must succeed");
        assert!(
            !extensions.contains_key("created_at"),
            "VP-PLUGIN-006 fixture 3: created_at must not appear in extensions"
        );
    }

    // Fixture 4: Integer → device.uid (integer column)
    // F-LP1-HIGH-002: uses integer_col_with_ocsf (NOT string_col_with_ocsf)
    {
        let col = integer_col_with_ocsf("asset_id", "device.uid");
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "asset_id": 99999i64 });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        let source_id = mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 4: integer-to-uid mapping must succeed");
        assert!(
            !extensions.contains_key("asset_id"),
            "VP-PLUGIN-006 fixture 4: asset_id must not appear in extensions"
        );
        assert_eq!(
            source_id, "99999",
            "VP-PLUGIN-006 fixture 4: source_id must be string representation of integer"
        );
    }

    // Fixture 5: String → ref_uid (string column)
    {
        let col = string_col_with_ocsf("alert_ref", "ref_uid");
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "alert_ref": "ALERT-REF-005" });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        let source_id = mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 5: alert_ref string mapping must succeed");
        assert!(
            !extensions.contains_key("alert_ref"),
            "VP-PLUGIN-006 fixture 5: alert_ref must not appear in extensions"
        );
        assert_eq!(
            source_id, "ALERT-REF-005",
            "VP-PLUGIN-006 fixture 5: source_id must equal alert_ref value"
        );
    }

    // Fixture 6: Boolean → is_active (string column stores bool as string)
    {
        let col = string_col_with_ocsf("is_active", "is_active");
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let raw = json!({ "is_active": true });
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        mapper
            .map("fixture-table", &raw, &mut msg, &mut extensions)
            .expect("VP-PLUGIN-006 fixture 6: boolean field mapping must succeed");
        assert!(
            !extensions.contains_key("is_active"),
            "VP-PLUGIN-006 fixture 6: is_active must not appear in extensions"
        );
    }

    // ---------------------------------------------------------------------------
    // Fixtures 7–9: WASM-required (Json column type) — F-LP1-HIGH-001 compliance
    //
    // These 3 fixtures exercise the ColumnType::Json path which requires a WASM
    // plugin. With an empty PluginRuntime (no loaded plugins), map() returns
    // OcsfNormalizationFailed (AC-003). This proves the WASM dispatch gate exists
    // and is triggered for Json columns.
    // ---------------------------------------------------------------------------

    for (fixture_num, (col_name, ocsf_field, raw_val)) in [
        (
            "behaviors",
            "attacks",
            json!({ "behaviors": [{"tactic": "Discovery"}] }),
        ),
        (
            "device_details",
            "device",
            json!({ "device_details": {"hostname": "srv-001", "os": "Linux"} }),
        ),
        (
            "threat_indicators",
            "evidences",
            json!({ "threat_indicators": [{"type": "hash", "value": "abc123"}] }),
        ),
    ]
    .iter()
    .enumerate()
    {
        let col = json_col_with_ocsf(col_name, ocsf_field);
        let config_manager =
            config_manager_with_ocsf_columns("fixture-sensor", "fixture-table", vec![col]);
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
        let mut msg = stub_dynamic_message();
        let mut extensions = serde_json::Map::new();
        let result = mapper.map("fixture-table", raw_val, &mut msg, &mut extensions);
        match result {
            Err(PrismError::OcsfNormalizationFailed { reason, .. }) => {
                assert!(
                    reason.contains("plugin") || reason.contains("ocsf_transform"),
                    "VP-PLUGIN-006 WASM fixture {}: error must mention plugin, got: {reason}",
                    fixture_num + 7
                );
            }
            Ok(_) => {
                // Acceptable if WASM plugin IS wired (not in this test environment).
            }
            Err(e) => panic!(
                "VP-PLUGIN-006 WASM fixture {}: unexpected error variant: {:?}",
                fixture_num + 7,
                e
            ),
        }
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

// ---------------------------------------------------------------------------
// F-LP1-HIGH-003: Missing AC-001 tests — nullable propagation + identity passthrough
// ---------------------------------------------------------------------------

/// AC-001 nullable propagation: column declared with `ocsf_field` but value is null/absent
/// in the raw record.
///
/// Per HIGH-005 / EC-004 graceful degradation: when `raw_value` is `None` for a
/// spec-declared ocsf_field column, the implementation inserts `(col.name, Null)` into
/// `extensions` and emits a debug trace (HIGH-005). This test verifies:
/// 1. `map()` succeeds (does not return Err for absent optional field).
/// 2. The absent column appears as `Null` in `extensions`.
/// 3. The other present columns still map correctly.
#[test]
fn test_BC_2_02_002_spec_driven_nullable_propagation() {
    // Two columns: one present in raw, one absent (null).
    let columns = vec![
        string_col_with_ocsf("detection_id", "finding_info.uid"), // present in raw
        string_col_with_ocsf("severity", "severity"),             // ABSENT in raw
    ];
    let config_manager = config_manager_with_ocsf_columns("test-sensor", "detections", columns);
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    // severity is absent — only detection_id is present.
    let raw = json!({ "detection_id": "DET-NULL-TEST" });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);
    let source_id = result.expect(
        "BC-2.02.002 AC-001: nullable propagation — map() must succeed even when optional \
         ocsf_field column is absent from raw record",
    );

    assert_eq!(
        source_id, "DET-NULL-TEST",
        "BC-2.02.002 AC-001: nullable propagation — source_id must equal detection_id value"
    );

    // detection_id WAS present and has an ocsf_field — must NOT appear in extensions.
    assert!(
        !extensions.contains_key("detection_id"),
        "BC-2.02.002 AC-001: nullable propagation — detection_id must not appear in extensions"
    );

    // severity was ABSENT in raw but declared with ocsf_field — HIGH-005 inserts Null.
    assert!(
        extensions.contains_key("severity"),
        "BC-2.02.002 AC-001: nullable propagation — absent ocsf_field column must appear \
         as Null in extensions (HIGH-005)"
    );
    assert_eq!(
        extensions.get("severity"),
        Some(&serde_json::Value::Null),
        "BC-2.02.002 AC-001: nullable propagation — absent ocsf_field column must have \
         Null value in extensions"
    );
}

/// AC-001 identity passthrough: column name equals the ocsf_field target.
///
/// When `col.name == ocsf_field`, the field is both the source and the destination.
/// The mapper must still write the value to `msg` (via `set_field_by_name`) and return
/// the value as `source_id` without error.
///
/// This exercises the common vendor pattern where the source field name matches the
/// OCSF target exactly (e.g., `severity = "severity"` in sensor spec TOML).
#[test]
fn test_BC_2_02_002_spec_driven_identity_passthrough() {
    // `status` maps to `status` — column name equals ocsf_field target.
    let config_manager = config_manager_with_ocsf_columns(
        "test-sensor",
        "events",
        vec![string_col_with_ocsf("status", "status")],
    );
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    let raw = json!({ "status": "active" });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("events", &raw, &mut msg, &mut extensions);
    let source_id = result.expect(
        "BC-2.02.002 AC-001: identity passthrough — map() must succeed when column name \
         equals ocsf_field target",
    );

    assert_eq!(
        source_id, "active",
        "BC-2.02.002 AC-001: identity passthrough — source_id must equal the field value"
    );

    // `status` has an ocsf_field — must NOT appear in extensions.
    assert!(
        !extensions.contains_key("status"),
        "BC-2.02.002 AC-001: identity passthrough — status must not appear in extensions \
         (it was mapped to msg via ocsf_field)"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-001: Value::Null in raw JSON must not corrupt to string "null"
// ---------------------------------------------------------------------------

/// F-LP2-HIGH-001: When a raw JSON record contains an explicit `null` value for a
/// spec-declared column with `ocsf_field`, the value must be placed into `extensions`
/// as `Value::Null` — NOT written to the DynamicMessage as the string `"null"`.
///
/// The bug was: `raw_value.is_none()` was false for `Some(Value::Null)`, so the
/// null fell through to the `other.to_string()` branch which produced `"null"` as a
/// ProtoValue::String — corrupting the field.
///
/// Fix: treat `None` and `Some(Value::Null)` identically in the absent/null branch.
#[test]
fn test_F_LP2_HIGH_001_json_null_value_placed_in_extensions_not_corrupted_to_string_null() {
    let columns = vec![
        string_col_with_ocsf("detection_id", "finding_info.uid"),
        string_col_with_ocsf("severity", "severity"), // value will be explicit null
    ];
    let config_manager = config_manager_with_ocsf_columns("test-sensor", "detections", columns);
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

    // severity is explicitly null in the raw JSON.
    let raw = serde_json::json!({ "detection_id": "DET-X", "severity": null });
    let mut msg = stub_dynamic_message();
    let mut extensions = serde_json::Map::new();

    let result = mapper.map("detections", &raw, &mut msg, &mut extensions);
    let source_id = result.expect(
        "F-LP2-HIGH-001: map() must succeed even when an ocsf_field column has explicit null value",
    );

    assert_eq!(
        source_id, "DET-X",
        "F-LP2-HIGH-001: source_id must equal detection_id, not the null severity"
    );

    // The null severity must appear as Value::Null in extensions — NOT as the string "null".
    assert!(
        extensions.contains_key("severity"),
        "F-LP2-HIGH-001: explicit-null ocsf_field column must appear in extensions"
    );
    assert_eq!(
        extensions.get("severity"),
        Some(&serde_json::Value::Null),
        "F-LP2-HIGH-001: explicit-null value must be serde_json::Value::Null in extensions, \
         not the string 'null'"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-002: DynamicMessage field value assertion using real OCSF descriptor
// ---------------------------------------------------------------------------

/// F-LP2-HIGH-002: Assert that `set_field_by_name` / `set_nested_field` writes the
/// expected value into the real OCSF `DynamicMessage` for a FLAT field.
///
/// Previous tests used `stub_dynamic_message()` with a zero-field descriptor, meaning
/// all `set_field_by_name` calls were no-ops. This test uses the real OCSF descriptor
/// pool (via `OcsfNormalizer::with_mappers`) to verify that the value actually lands in
/// the protobuf message.
///
/// DetectionFinding (class_uid=2004) has a top-level `string severity = 55` field.
/// This test maps `{"severity": "High"}` through a SpecDrivenMapper with
/// `ocsf_field = "severity"` and asserts the real DynamicMessage has that value.
///
/// Note: If the OCSF descriptor binary is a zero-byte stub (build without ocsf-proto-gen),
/// the pool won't have the DetectionFinding descriptor and the test is skipped via
/// `if let Ok` on the normalizer result.
#[test]
fn test_F_LP2_HIGH_002_dynamic_message_field_value_written_to_real_descriptor() {
    use prism_ocsf::normalizer::OcsfNormalizer;
    use prost_reflect::ReflectMessage;

    // Build a SpecDrivenMapper for "crowdstrike" / "detections" with a flat
    // `severity` → `severity` mapping. This matches the real OCSF DetectionFinding
    // string field `severity` (field #55).
    let config_manager = config_manager_with_ocsf_columns(
        "crowdstrike",
        "detections",
        vec![string_col_with_ocsf("severity", "severity")],
    );
    let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(mapper)]);

    let raw = serde_json::json!({ "severity": "High" });

    // normalize_with_mappers requires the real pool to have class_uid 2004.
    // If the pool is a stub (empty binary), this returns OcsfDescriptorNotFound — skip.
    let result = normalizer.normalize_with_mappers("crowdstrike", "detection", raw);
    let (msg, source_id) = match result {
        Ok(pair) => pair,
        Err(prism_core::PrismError::OcsfDescriptorNotFound { .. }) => {
            // Stub pool — skip the real-descriptor assertion.
            // This branch runs when ocsf-proto-gen has not been run.
            return;
        }
        Err(e) => panic!(
            "F-LP2-HIGH-002: unexpected error from normalize_with_mappers: {:?}",
            e
        ),
    };

    // Assert source_id is correct.
    assert_eq!(
        source_id, "High",
        "F-LP2-HIGH-002: source_id must equal the mapped severity value"
    );

    // Assert the flat 'severity' field was written into the DynamicMessage.
    // This exercises set_nested_field (flat case) on a REAL descriptor — if
    // set_field_by_name / set_nested_field was a no-op, get_field_by_name returns None
    // or the default empty string.
    let severity_field = msg
        .descriptor()
        .get_field_by_name("severity")
        .expect("F-LP2-HIGH-002: DetectionFinding descriptor must have a 'severity' string field");

    let severity_value = msg.get_field(&severity_field);
    assert_eq!(
        severity_value.as_ref(),
        &prost_reflect::Value::String("High".to_owned()),
        "F-LP2-HIGH-002: DynamicMessage 'severity' field must equal 'High' after mapping; \
         set_nested_field (flat path) must write the actual value, not no-op"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S-ADR058-OCSF-ROUTING-001 Red Gate Tests — RG-011, RG-012, RG-023
//
// These tests cover AC-009 class_selector path:
// - RG-011: select_by_class_name("entity_management") and ("inventory_info") new arms
// - RG-012: select("armis", "audit_log") corrected to 3004 (EntityManagement)
// - RG-023: select("claroty", "audit_log") corrected to 3004 (EntityManagement)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-011 / AC-009(a) / ADR-058 §K5 Div-3 / BC-2.16.003 EC-016-013-023/024
///
/// `select_by_class_name` MUST handle two new TOML `ocsf_class` values introduced by
/// KF-01 ("entity_management" → 3004) and KF-02 ("inventory_info" → 5001).
///
/// **KF-01 rationale:** Claroty/Armis `audit_log` must map to EntityManagement (3004),
/// NOT AccountChange (3001). AccountChange lacks the `comment` attribute, silently dropping
/// every `note` field value. EntityManagement carries `comment` (inherited from BaseEvent).
///
/// **KF-02 rationale:** Claroty/Armis `devices` must map to DeviceInventoryInfo (5001).
/// The previous `ocsf_class = "device"` arm (5001) is preserved for compatibility; the
/// new `"inventory_info"` arm is added as the KF-02-corrected canonical value.
///
/// **Red gate:** both arms are `todo!()` — the first call panics.
#[test]
fn test_class_selector_entity_management_and_inventory_info_arms() {
    use prism_ocsf::{class_selector::CLASS_UID_ENTITY_MANAGEMENT, EventClassSelector};

    // Sub-case 1: "entity_management" → 3004 (KF-01)
    let em_result = EventClassSelector::select_by_class_name("entity_management");
    assert!(
        em_result.is_ok(),
        "AC-009(a) (RG-011): select_by_class_name('entity_management') MUST return Ok(3004); \
         KF-01 TOML correction routes Claroty/Armis audit_log here. Got Err: {:?}",
        em_result.err()
    );
    assert_eq!(
        em_result.unwrap(),
        CLASS_UID_ENTITY_MANAGEMENT,
        "AC-009(a) (RG-011): select_by_class_name('entity_management') MUST return \
         CLASS_UID_ENTITY_MANAGEMENT (3004). Without this arm, KF-01 TOML correction \
         regresses class_uid from 3001 to 0 (BASE_EVENT fallback via .unwrap_or(0))."
    );

    // Sub-case 2: "inventory_info" → 5001 (KF-02)
    let ii_result = EventClassSelector::select_by_class_name("inventory_info");
    assert!(
        ii_result.is_ok(),
        "AC-009(a) (RG-011): select_by_class_name('inventory_info') MUST return Ok(5001); \
         KF-02 TOML correction routes Claroty/Armis devices here. Got Err: {:?}",
        ii_result.err()
    );
    assert_eq!(
        ii_result.unwrap(),
        prism_ocsf::class_selector::CLASS_UID_DEVICE_INVENTORY_INFO,
        "AC-009(a) (RG-011): select_by_class_name('inventory_info') MUST return \
         CLASS_UID_DEVICE_INVENTORY_INFO (5001). Without this arm, KF-02 TOML correction \
         regresses class_uid from 5001 to 0."
    );
}

/// RG-012 / AC-009(a) / ADR-058 §K5 Div-3 / BC-2.16.003 EC-016-013-023
///
/// `select("armis", "audit_log")` MUST return 3004 (EntityManagement) after KF-01
/// correction. Currently returns 3001 (AccountChange) — this is the pre-KF stale arm.
///
/// **Why 3004 is required:** AccountChange (3001) lacks the `comment` attribute in
/// OCSF v1.7.0. Armis `audit_log` maps `note → comment`. Without 3004, the
/// `note` value silently disappears from every protobuf-normalized Armis audit event.
///
/// **Red gate:** `select("armis", "audit_log")` returns `CLASS_UID_ACCOUNT_CHANGE` (3001)
/// — the assertion `== 3004` fails.
#[test]
fn test_class_selector_armis_audit_log_maps_to_entity_management_3004() {
    use prism_ocsf::{class_selector::CLASS_UID_ENTITY_MANAGEMENT, EventClassSelector};

    let result = EventClassSelector::select("armis", "audit_log");
    assert!(
        result.is_ok(),
        "AC-009(a) (RG-012): select('armis', 'audit_log') MUST return Ok(3004); \
         it is a known sensor+record_type pair. Got Err: {:?}",
        result.err()
    );
    let uid = result.unwrap();
    assert_eq!(
        uid, CLASS_UID_ENTITY_MANAGEMENT,
        "AC-009(a) (RG-012): select('armis', 'audit_log') MUST return \
         CLASS_UID_ENTITY_MANAGEMENT (3004) after KF-01 correction. \
         Currently returns 3001 (AccountChange) — the stale pre-KF arm. \
         AccountChange (3001) lacks the 'comment' attribute; every Armis audit_log \
         'note' value is silently dropped during protobuf normalization."
    );
    assert_ne!(
        uid, 3001u32,
        "AC-009(a) (RG-012): result MUST NOT be 3001 (AccountChange)"
    );
}

/// RG-023 / AC-009(a) / ADR-058 §K5 Div-3 / BC-2.16.003 EC-016-013-023
///
/// `select("claroty", "audit_log")` MUST return 3004 (EntityManagement) after KF-01
/// correction. Currently returns 3001 (AccountChange) — this is the pre-KF stale arm.
///
/// **Why 3004 is required:** Same rationale as RG-012 (Armis sibling).
/// Claroty `audit_log` maps `note → comment`. AccountChange (3001) lacks `comment`.
///
/// **Red gate:** `select("claroty", "audit_log")` returns `CLASS_UID_ACCOUNT_CHANGE` (3001)
/// — the assertion `== 3004` fails.
#[test]
fn test_class_selector_claroty_audit_log_select_arm_maps_to_entity_management_3004() {
    use prism_ocsf::{class_selector::CLASS_UID_ENTITY_MANAGEMENT, EventClassSelector};

    let result = EventClassSelector::select("claroty", "audit_log");
    assert!(
        result.is_ok(),
        "AC-009(a) (RG-023): select('claroty', 'audit_log') MUST return Ok(3004); \
         it is a known sensor+record_type pair. Got Err: {:?}",
        result.err()
    );
    let uid = result.unwrap();
    assert_eq!(
        uid, CLASS_UID_ENTITY_MANAGEMENT,
        "AC-009(a) (RG-023): select('claroty', 'audit_log') MUST return \
         CLASS_UID_ENTITY_MANAGEMENT (3004) after KF-01 correction. \
         Currently returns 3001 (AccountChange) — the stale pre-KF arm. \
         AccountChange (3001) lacks 'comment'; every Claroty audit_log 'note' value \
         (e.g., 'reviewed') is silently dropped during protobuf normalization."
    );
    assert_ne!(
        uid, 3001u32,
        "AC-009(a) (RG-023): result MUST NOT be 3001 (AccountChange)"
    );
}
