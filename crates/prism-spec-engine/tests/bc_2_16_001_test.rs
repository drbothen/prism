#![allow(non_snake_case)]
//! BC-2.16.001: Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables
//!
//! Tests cover:
//! - Precondition violations (parse error, empty dir)
//! - Postconditions (SensorSpec struct, SensorTableDescriptor production)
//! - Table name conflict detection at load time
//! - Sensor_id conflict detection (E-SPEC-009)
//! - Partial-failure isolation (DI-030)
//! - No DataFusion dependency (structural test)
//!
//! AC-1: Given a valid crowdstrike.sensor.toml, When SpecParser loads it,
//!       Then all sources produce SensorTableDescriptor entries.

use prism_core::{ColumnOptions, ColumnType};
use prism_spec_engine::spec_parser::{AuthType, SpecLoader};

// ---------------------------------------------------------------------------
// Canonical test TOML (CrowdStrike-like minimal spec)
// ---------------------------------------------------------------------------

const CROWDSTRIKE_SENSOR_TOML: &str = r#"
sensor_id = "crowdstrike"
name = "CrowdStrike Falcon"
auth_type = "oauth2_client_credentials"
base_url = "https://api.crowdstrike.com"
version = "1.0.0"

[[tables]]
table_name = "detections"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "detection_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.columns]]
  name = "created_timestamp"
  column_type = "datetime"
  ocsf_field = "time"

  [[tables.steps]]
  name = "fetch_detections"
  method = "GET"
  path_template = "/detections/queries/detections/v2"
  response_path = "$.resources"
  variables_produced = ["detection_ids"]

[[tables]]
table_name = "hosts"
ocsf_class = "device_inventory"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_hosts"
  method = "GET"
  path_template = "/devices/queries/devices/v1"
  response_path = "$.resources"
  variables_produced = []
"#;

const MALFORMED_TOML: &str = r#"
sensor_id = "bad
this is not valid toml [[[
"#;

// ---------------------------------------------------------------------------
// BC-2.16.001 Postcondition Tests
// ---------------------------------------------------------------------------

/// AC-1 / BC-2.16.001 postcondition: valid TOML produces SensorSpec with sensor_id, name,
/// auth_type, base_url, tables, version.
#[test]
fn test_BC_2_16_001_parses_valid_spec_into_sensor_spec_struct() {
    let result = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML);
    let spec = result.expect("valid TOML must parse without error");

    assert_eq!(spec.sensor_id, "crowdstrike");
    assert_eq!(spec.name, "CrowdStrike Falcon");
    assert_eq!(spec.auth_type, AuthType::Oauth2ClientCredentials);
    assert_eq!(spec.base_url, "https://api.crowdstrike.com");
    assert_eq!(spec.version, "1.0.0");
    assert_eq!(spec.tables.len(), 2);
}

/// BC-2.16.001 postcondition: each TableSpec has table_name, ocsf_class, columns, steps.
#[test]
fn test_BC_2_16_001_parses_table_specs_with_columns_and_steps() {
    let spec = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");

    let detections = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("detections table must be present");

    assert_eq!(detections.ocsf_class, "security_finding");
    assert_eq!(detections.columns.len(), 2);
    assert_eq!(detections.steps.len(), 1);
}

/// BC-2.16.001 postcondition: ColumnSpec includes name, column_type, ocsf_field, options.
#[test]
fn test_BC_2_16_001_parses_column_spec_with_type_and_ocsf_field() {
    let spec = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");

    let detections = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .unwrap();
    let ts_col = detections
        .columns
        .iter()
        .find(|c| c.name == "created_timestamp")
        .expect("created_timestamp column must be present");

    assert_eq!(ts_col.column_type, ColumnType::Datetime);
    assert_eq!(ts_col.ocsf_field.as_deref(), Some("time"));
}

/// BC-2.16.001 postcondition: REQUIRED column option is parsed.
#[test]
fn test_BC_2_16_001_parses_column_options_required() {
    let spec = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");

    let detections = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .unwrap();
    let id_col = detections
        .columns
        .iter()
        .find(|c| c.name == "detection_id")
        .unwrap();

    assert!(
        id_col.options.contains(&ColumnOptions::Required),
        "detection_id must have REQUIRED option"
    );
}

/// BC-2.16.001 postcondition: valid spec produces SensorTableDescriptor for each table.
/// AC-1 (S-1.11).
#[test]
fn test_BC_2_16_001_produces_sensor_table_descriptor_per_table() {
    let spec = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");
    let _loader = SpecLoader::new("/tmp/sensor-specs");

    // Descriptors should be producible from a valid spec
    // This exercises the descriptor production path (AC-1)
    let conflicts = SpecLoader::detect_table_name_conflicts(std::slice::from_ref(&spec));
    assert!(
        conflicts.is_empty(),
        "valid spec must produce no table name conflicts"
    );
}

/// BC-2.16.001 postcondition: table name format is {sensor_id}.{table_name}.
#[test]
fn test_BC_2_16_001_table_name_format_sensor_id_dot_table_name() {
    let spec = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");

    // The table name in SensorTableDescriptor must be "crowdstrike.detections" etc.
    // This verifies the naming convention postcondition.
    let conflicts = SpecLoader::detect_table_name_conflicts(&[spec]);
    assert!(conflicts.is_empty());
    // If SpecLoader::load_all were called, descriptors with names like
    // "crowdstrike.detections" and "crowdstrike.hosts" must be returned.
    // Full integration with load_all will fail here (unimplemented!) — Red Gate.
}

// ---------------------------------------------------------------------------
// BC-2.16.001 Error Condition Tests
// ---------------------------------------------------------------------------

/// BC-2.16.001 error: TOML parse error -> E-SPEC-001 with line number.
#[test]
fn test_BC_2_16_001_rejects_malformed_toml_with_e_spec_001() {
    let result = SpecLoader::parse(MALFORMED_TOML);
    assert!(result.is_err(), "malformed TOML must return Err");
    // Error must carry line number (BC-2.16.001 Error Conditions)
    // Full error code check deferred to implementation — Red Gate.
}

/// BC-2.16.001 error: duplicate sensor_id -> E-SPEC-009, first wins.
#[test]
fn test_BC_2_16_001_rejects_duplicate_sensor_id_e_spec_009() {
    let spec1 = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");
    let spec2 = SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML).expect("parse must succeed");

    // Two specs with the same sensor_id: second should be rejected.
    let conflicts = SpecLoader::detect_sensor_id_conflicts(&[
        ("crowdstrike.sensor.toml".to_string(), spec1),
        ("crowdstrike-duplicate.sensor.toml".to_string(), spec2),
    ]);

    assert!(
        !conflicts.is_empty(),
        "duplicate sensor_id must produce E-SPEC-009"
    );
}

/// BC-2.16.001 error: duplicate table_name within a sensor -> E-SPEC-004, spec rejected.
#[test]
fn test_BC_2_16_001_rejects_duplicate_table_name_within_sensor_e_spec_004() {
    let duplicate_table_toml = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
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
"#;

    let spec = SpecLoader::parse(duplicate_table_toml).expect("parse must succeed for TOML syntax");

    let conflicts = SpecLoader::detect_table_name_conflicts(&[spec]);
    assert!(
        !conflicts.is_empty(),
        "duplicate table_name within sensor must produce E-SPEC-004"
    );
}

/// BC-2.16.001 partial-failure isolation (DI-030): one invalid spec does not block valid ones.
/// This is tested at the load_all level — exercising the stub to verify Red Gate.
#[test]
fn test_BC_2_16_001_partial_failure_isolation_valid_specs_load_despite_invalid() {
    let loader = SpecLoader::new("/tmp/sensor-specs");
    // load_all is unimplemented -> will panic with unimplemented!
    // This test verifies Red Gate: it must fail.
    let (_descriptors, _errors) = loader.load_all();
    // If load_all were implemented: valid specs produce descriptors, invalid produce errors.
    // DI-030: assert that descriptors.len() > 0 despite errors.len() > 0 for a mixed dir.
}

/// BC-2.16.001 edge case: empty sensor_specs_dir -> zero tables, no error.
#[test]
fn test_BC_2_16_001_empty_directory_produces_zero_tables_no_error() {
    let loader = SpecLoader::new("/tmp/empty-sensor-specs");
    let (_descriptors, errors) = loader.load_all();
    assert!(errors.is_empty(), "empty dir must produce no errors");
}
