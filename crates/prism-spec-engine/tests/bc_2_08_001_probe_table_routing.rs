#![allow(non_snake_case)]
//! BC-2.08.001 postcondition 5 — probe_table routing / fallback chain (S-5.04 AC-9, AC-10)
//!
//! These tests are S-5.04 Red Gate tests for the unimplemented AC-9 and AC-10 behaviors:
//!
//! **AC-9**: When `probe_connectivity()` runs for a sensor, it routes the LIMIT-0 fetch
//! to `"{sensor_id}.{probe_table}"` when `probe_table` is set, OR falls back to the first
//! declared table (`spec.tables[0].table_name`) when `probe_table` is absent, OR is a
//! structural no-op when both `probe_table` is absent and `spec.tables` is empty.
//!
//! **AC-10**: Each canonical sensor TOML spec declares `probe_table` with the architect-
//! specified value (crowdstrike→"detections", cyberint→"alerts", claroty→"devices",
//! armis→"devices"), AND each value matches a declared `[[tables]]` block (Rule 8 passes).
//!
//! # Location
//! AC-9 tests live here (prism-spec-engine tests) because they test the SensorSpec shape
//! consumed by connectivity.rs — the routing logic is in prism-mcp, but the CONTRACT
//! is about what SensorSpec.probe_table contains. The prism-mcp tests verify the adapter
//! call routing; the spec-engine tests verify the TOML deserialization.
//!
//! AC-10 tests verify the canonical sensor TOML files via SpecLoader::parse() — naturally
//! lives here.
//!
//! # Red Gate status
//! - AC-9 (probe routing): The actual routing logic lives in `connectivity.rs`.
//!   The test here verifies that `probe_table` is correctly deserialized from TOML and
//!   accessible — this passes once the field exists (which it does after scaffold).
//!   The behavioral AC-9 routing tests are in prism-mcp's test file.
//!
//! - AC-10 (canonical TOML parity): The canonical sensor TOML files do NOT yet have
//!   `probe_table` declared. Tests MUST FAIL until the implementer adds `probe_table`
//!   to each of the four canonical TOML files.
//!
//! # SAP-2 compliance
//! probe_table values ("detections", "alerts", "devices") were verified against the
//! actual [[tables]] blocks in crates/prism-sensors/specs/*.sensor.toml:
//!   crowdstrike: ["detections", "devices", "incidents"] → probe_table = "detections"
//!   cyberint:    ["alerts", "incidents"]               → probe_table = "alerts"
//!   claroty:     ["alerts", "audit_logs", "devices"]   → probe_table = "devices"
//!   armis:       ["devices", "alerts"]                 → probe_table = "devices"

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// AC-9 — probe_table deserialization from TOML (spec-engine layer)
// The BEHAVIORAL routing tests (adapter call verification) are in prism-mcp.
// ---------------------------------------------------------------------------

/// AC-9 deserialization: `probe_table = "detections"` in TOML is correctly
/// read into `SensorSpec.probe_table = Some("detections")`.
///
/// GREEN-BY-DESIGN once the scaffold field exists (currently passes).
/// Included to document the deserialization contract and guard regressions.
#[test]
fn test_BC_2_08_001_probe_table_field_deserializes_from_toml() {
    let toml_with_probe_table = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"
probe_table = "detections"

[[tables]]
table_name = "detections"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/detections"
  response_path = "$.resources"
  variables_produced = []
  [tables.steps.pagination]
  type = "none"
"#;

    // NOTE: Rule 8 validation is not yet implemented, so parse() returns Ok
    // even if probe_table were invalid. Once Rule 8 ships, this spec is valid
    // anyway (probe_table="detections" matches the declared table).
    let spec = SpecLoader::parse(toml_with_probe_table)
        .expect("probe_table='detections' matching a declared [[tables]] must parse successfully");

    assert_eq!(
        spec.probe_table,
        Some("detections".to_string()),
        "AC-9: probe_table must deserialize from TOML as Some('detections'); got: {:?}",
        spec.probe_table
    );
}

/// AC-9 deserialization: absent `probe_table` in TOML → `SensorSpec.probe_table = None`.
///
/// Backward-compat guard: existing TOML files without probe_table must still parse.
/// GREEN-BY-DESIGN (currently passes).
#[test]
fn test_BC_2_08_001_probe_table_absent_in_toml_yields_none() {
    let toml_without_probe_table = r#"
sensor_id = "legacy-sensor"
name = "Legacy Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "alerts"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/alerts"
  response_path = "$.data"
  variables_produced = []
  [tables.steps.pagination]
  type = "none"
"#;

    let spec = SpecLoader::parse(toml_without_probe_table)
        .expect("TOML without probe_table must parse successfully (backward compat)");

    assert_eq!(
        spec.probe_table, None,
        "AC-9: absent probe_table MUST deserialize as None; got: {:?}",
        spec.probe_table
    );
}

// ---------------------------------------------------------------------------
// AC-10 — canonical sensor TOML parity (probe_table declared + matches tables)
// ---------------------------------------------------------------------------
//
// Each test loads the canonical sensor spec from crates/prism-sensors/specs/
// and verifies:
// 1. `probe_table` is Some(expected_value) — not None (would indicate implementer
//    has not yet added the field to the TOML file).
// 2. The probe_table value matches an actual [[tables]] block's table_name
//    (Rule 8 parity — the loaded value exists in spec.tables).
//
// RED GATE: The canonical TOML files do NOT yet have `probe_table` declared.
// ALL four tests below MUST FAIL until the implementer adds probe_table to each file.

/// Build the absolute path to a canonical sensor spec file.
///
/// Uses `CARGO_MANIFEST_DIR` (prism-spec-engine crate root) and navigates to
/// `../prism-sensors/specs/` where the canonical TOML specs live.
fn canonical_spec_path(sensor: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo nextest");
    // prism-spec-engine manifest is at crates/prism-spec-engine/
    // canonical specs are at crates/prism-sensors/specs/
    std::path::PathBuf::from(&manifest)
        .join("../prism-sensors/specs")
        .join(format!("{sensor}.sensor.toml"))
}

/// Load and parse a canonical sensor spec TOML file, returning the parsed SensorSpec.
fn load_canonical_spec(sensor: &str) -> prism_spec_engine::spec_parser::SensorSpec {
    let path = canonical_spec_path(sensor);
    let toml_content =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    SpecLoader::parse(&toml_content)
        .unwrap_or_else(|e| panic!("failed to parse {sensor}.sensor.toml: {e:?}"))
}

/// AC-10 (S-5.04): `crowdstrike.sensor.toml` declares `probe_table = "detections"`.
///
/// Expected: spec.probe_table == Some("detections"), and "detections" is declared
/// in spec.tables (Rule 8 parity).
///
/// RED GATE: crowdstrike.sensor.toml currently has NO probe_table field.
/// parse() returns spec.probe_table = None → assertion fails.
#[test]
fn test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_crowdstrike() {
    let spec = load_canonical_spec("crowdstrike");

    // AC-10: probe_table MUST be declared
    assert_eq!(
        spec.probe_table,
        Some("detections".to_string()),
        "AC-10: crowdstrike.sensor.toml MUST declare probe_table = \"detections\" \
         (architect-specified; probe-table-field-design.md §4). \
         Currently None — implementer must add 'probe_table = \"detections\"' to the TOML file"
    );

    // Rule 8 parity: "detections" must be a declared table
    let table_names: Vec<&str> = spec.tables.iter().map(|t| t.table_name.as_str()).collect();
    assert!(
        table_names.contains(&"detections"),
        "AC-10: 'detections' must be a declared [[tables]].table_name in crowdstrike.sensor.toml; \
         declared tables: {table_names:?}"
    );
}

/// AC-10 (S-5.04): `cyberint.sensor.toml` declares `probe_table = "alerts"`.
///
/// Expected: spec.probe_table == Some("alerts"), and "alerts" is declared in spec.tables.
///
/// RED GATE: cyberint.sensor.toml currently has NO probe_table field.
#[test]
fn test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_cyberint() {
    let spec = load_canonical_spec("cyberint");

    assert_eq!(
        spec.probe_table,
        Some("alerts".to_string()),
        "AC-10: cyberint.sensor.toml MUST declare probe_table = \"alerts\" \
         (architect-specified; probe-table-field-design.md §4). \
         Currently None — implementer must add 'probe_table = \"alerts\"' to the TOML file"
    );

    let table_names: Vec<&str> = spec.tables.iter().map(|t| t.table_name.as_str()).collect();
    assert!(
        table_names.contains(&"alerts"),
        "AC-10: 'alerts' must be a declared [[tables]].table_name in cyberint.sensor.toml; \
         declared tables: {table_names:?}"
    );
}

/// AC-10 (S-5.04): `claroty.sensor.toml` declares `probe_table = "devices"`.
///
/// Expected: spec.probe_table == Some("devices"), and "devices" is declared in spec.tables.
///
/// RED GATE: claroty.sensor.toml currently has NO probe_table field.
#[test]
fn test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_claroty() {
    let spec = load_canonical_spec("claroty");

    assert_eq!(
        spec.probe_table,
        Some("devices".to_string()),
        "AC-10: claroty.sensor.toml MUST declare probe_table = \"devices\" \
         (architect-specified; probe-table-field-design.md §4). \
         Currently None — implementer must add 'probe_table = \"devices\"' to the TOML file"
    );

    let table_names: Vec<&str> = spec.tables.iter().map(|t| t.table_name.as_str()).collect();
    assert!(
        table_names.contains(&"devices"),
        "AC-10: 'devices' must be a declared [[tables]].table_name in claroty.sensor.toml; \
         declared tables: {table_names:?}"
    );
}

/// AC-10 (S-5.04): `armis.sensor.toml` declares `probe_table = "devices"`.
///
/// Expected: spec.probe_table == Some("devices"), and "devices" is declared in spec.tables.
///
/// RED GATE: armis.sensor.toml currently has NO probe_table field.
#[test]
fn test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_armis() {
    let spec = load_canonical_spec("armis");

    assert_eq!(
        spec.probe_table,
        Some("devices".to_string()),
        "AC-10: armis.sensor.toml MUST declare probe_table = \"devices\" \
         (architect-specified; probe-table-field-design.md §4). \
         Currently None — implementer must add 'probe_table = \"devices\"' to the TOML file"
    );

    let table_names: Vec<&str> = spec.tables.iter().map(|t| t.table_name.as_str()).collect();
    assert!(
        table_names.contains(&"devices"),
        "AC-10: 'devices' must be a declared [[tables]].table_name in armis.sensor.toml; \
         declared tables: {table_names:?}"
    );
}
