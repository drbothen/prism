#![allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    unused_imports
)]
//! Tests for S-2.08 TableSpec new fields: `table_type`, `poll_interval_secs`, `retention_secs`.
//!
//! Also tests `SpecLoader::validate_table_spec` AC-7 / EC-002 validation.
//!
//! Story: S-2.08 | AC-7, EC-002
//!
//! # Coverage
//! - TableSpec default table_type is PointInTime (backward-compatible TOML parse)
//! - TableSpec with type = "event_stream" parses correctly
//! - validate_table_spec accepts valid EventStream with poll_interval >= 10s
//! - validate_table_spec rejects poll_interval_secs < 10 (AC-7, EC-002)
//! - validate_table_spec rejects poll_interval_secs = 0
//! - validate_table_spec rejects poll_interval_secs = 9 (boundary: one below minimum)
//! - validate_table_spec accepts poll_interval_secs = 10 (boundary: minimum)
//! - validate_table_spec rejects retention_secs > 604800 (7 days maximum)
//! - validate_table_spec accepts retention_secs = 604800 (boundary: maximum)
//! - validate_table_spec rejects poll_interval_secs on PointInTime table
//! - validate_table_spec rejects retention_secs on PointInTime table
//! - validate_table_spec accepts None poll_interval + None retention on EventStream
//! - TableSpec parses from full TOML with all new fields
//! - prism_core::TableType is importable directly (Defect 2 canonical home check)
//!
//! # RED GATE
//! Tests calling `validate_table_spec` will PANIC with "not yet implemented" — RED.
//! TOML parse tests exercising the new struct fields are GREEN-BY-DESIGN (serde).

use prism_core::TableType;
use prism_spec_engine::spec_parser::{ColumnSpec, FetchStep, SpecLoader, TableSpec};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn minimal_fetch_step() -> FetchStep {
    FetchStep {
        name: "fetch".to_string(),
        method: "GET".to_string(),
        path_template: "/data".to_string(),
        body_template: None,
        response_path: "$.data".to_string(),
        pagination_cursor_path: None,
        variables_produced: vec![],
        fan_out_batch_size: None,
        pagination: None,
    }
}

fn make_table_spec(
    table_type: TableType,
    poll_interval_secs: Option<u64>,
    retention_secs: Option<u64>,
) -> TableSpec {
    TableSpec::new(
        "test_table",
        "security_finding",
        vec![],
        vec![minimal_fetch_step()],
        table_type,
        poll_interval_secs,
        retention_secs,
    )
}

// Minimal TOML for a sensor spec with event_stream table
const EVENT_STREAM_SENSOR_TOML: &str = r#"
sensor_id = "crowdstrike"
name = "CrowdStrike Falcon"
auth_type = "oauth2_client_credentials"
base_url = "https://api.crowdstrike.com"
version = "1.0.0"

[[tables]]
table_name = "process_events"
ocsf_class = "process_activity"
table_type = "event_stream"
poll_interval_secs = 60
retention_secs = 86400
columns = []

  [[tables.steps]]
  name = "fetch_events"
  method = "GET"
  path_template = "/events/queries/events/v1"
  response_path = "$.resources"
  variables_produced = []
"#;

const POINT_IN_TIME_SENSOR_TOML: &str = r#"
sensor_id = "armis"
name = "Armis"
auth_type = "bearer_static"
base_url = "https://api.armis.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory"
columns = []

  [[tables.steps]]
  name = "fetch_devices"
  method = "GET"
  path_template = "/api/v1/devices"
  response_path = "$.data"
  variables_produced = []
"#;

// ---------------------------------------------------------------------------
// TOML parse tests — GREEN-BY-DESIGN (struct fields + serde)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_table_spec_default_table_type_is_point_in_time() {
    // GREEN-BY-DESIGN: #[serde(default)] on table_type + Default = PointInTime
    let spec =
        SpecLoader::parse(POINT_IN_TIME_SENSOR_TOML).expect("valid TOML must parse without error");
    assert_eq!(
        spec.tables[0].table_type,
        TableType::PointInTime,
        "TableSpec without explicit table_type must default to PointInTime (backward-compatible)"
    );
}

#[test]
fn test_BC_2_08_table_spec_event_stream_parses_from_toml() {
    // GREEN-BY-DESIGN: serde parses event_stream variant correctly
    let spec = SpecLoader::parse(EVENT_STREAM_SENSOR_TOML)
        .expect("event_stream TOML must parse without error");
    assert_eq!(
        spec.tables[0].table_type,
        TableType::EventStream,
        "TableSpec with table_type = 'event_stream' must parse as EventStream"
    );
}

#[test]
fn test_BC_2_08_table_spec_event_stream_poll_interval_parsed() {
    // GREEN-BY-DESIGN: poll_interval_secs field parses correctly
    let spec =
        SpecLoader::parse(EVENT_STREAM_SENSOR_TOML).expect("valid TOML must parse without error");
    assert_eq!(
        spec.tables[0].poll_interval_secs,
        Some(60),
        "poll_interval_secs = 60 must parse as Some(60)"
    );
}

#[test]
fn test_BC_2_08_table_spec_event_stream_retention_parsed() {
    // GREEN-BY-DESIGN: retention_secs field parses correctly
    let spec =
        SpecLoader::parse(EVENT_STREAM_SENSOR_TOML).expect("valid TOML must parse without error");
    assert_eq!(
        spec.tables[0].retention_secs,
        Some(86400),
        "retention_secs = 86400 must parse as Some(86400)"
    );
}

#[test]
fn test_BC_2_08_table_spec_point_in_time_poll_interval_defaults_to_none() {
    // GREEN-BY-DESIGN: #[serde(default)] on poll_interval_secs
    let spec =
        SpecLoader::parse(POINT_IN_TIME_SENSOR_TOML).expect("valid TOML must parse without error");
    assert!(
        spec.tables[0].poll_interval_secs.is_none(),
        "PointInTime table without poll_interval_secs must default to None"
    );
}

#[test]
fn test_BC_2_08_table_spec_point_in_time_retention_defaults_to_none() {
    // GREEN-BY-DESIGN: #[serde(default)] on retention_secs
    let spec =
        SpecLoader::parse(POINT_IN_TIME_SENSOR_TOML).expect("valid TOML must parse without error");
    assert!(
        spec.tables[0].retention_secs.is_none(),
        "PointInTime table without retention_secs must default to None"
    );
}

#[test]
fn test_BC_2_08_toml_rejects_unknown_table_type_string() {
    // GREEN-BY-DESIGN: serde rejects unknown variant strings
    let bad_toml = r#"
sensor_id = "test"
name = "Test"
auth_type = "bearer_static"
base_url = "https://example.com"
version = "1.0.0"

[[tables]]
table_name = "t"
ocsf_class = "x"
table_type = "realtime"
columns = []

  [[tables.steps]]
  name = "s"
  method = "GET"
  path_template = "/p"
  response_path = "$.d"
  variables_produced = []
"#;
    let result = SpecLoader::parse(bad_toml);
    assert!(
        result.is_err(),
        "unknown table_type string 'realtime' must cause parse failure"
    );
}

// ---------------------------------------------------------------------------
// Canonical import check — Defect 2 architecture compliance
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_table_type_importable_from_prism_core_directly() {
    // GREEN-BY-DESIGN: structural test — verifies prism_core::TableType is re-exported
    // and importable without going through prism-spec-engine.
    // If the canonical home moves back to spec-engine, this import line fails to compile.
    let _t: TableType = TableType::PointInTime;
    let _s: TableType = TableType::EventStream;
}

#[test]
fn test_BC_2_08_table_type_from_spec_engine_matches_prism_core() {
    // GREEN-BY-DESIGN: prism-spec-engine re-exports prism_core::TableType;
    // the two imports must refer to the same type (compile-time check via assignment).
    use prism_spec_engine::TableType as SpecEngineTableType;
    let from_core: TableType = TableType::EventStream;
    let from_spec: SpecEngineTableType = SpecEngineTableType::EventStream;
    // If these were different types, the assert_eq! would fail to compile.
    assert_eq!(
        from_core, from_spec,
        "prism_core::TableType and prism_spec_engine::TableType must be the same type"
    );
}

// ---------------------------------------------------------------------------
// validate_table_spec — AC-7, EC-002 (RED: todo!())
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_validate_table_spec_accepts_event_stream_with_valid_poll_interval() {
    // RED: validate_table_spec is todo!()
    // AC-7: poll_interval_secs = 60 (>= minimum 10) must be accepted
    let table = make_table_spec(TableType::EventStream, Some(60), Some(86400));
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_ok(),
        "AC-7: EventStream table with poll_interval_secs=60 must pass validation; got: {result:?}"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_accepts_minimum_poll_interval() {
    // RED: validate_table_spec is todo!()
    // AC-7 boundary: poll_interval_secs = 10 is the minimum — must be accepted
    let table = make_table_spec(TableType::EventStream, Some(10), None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_ok(),
        "AC-7: poll_interval_secs=10 (minimum) must be accepted; got: {result:?}"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_poll_interval_below_minimum() {
    // RED: validate_table_spec is todo!()
    // AC-7, EC-002: poll_interval_secs = 9 is below the 10s minimum — must be rejected
    let table = make_table_spec(TableType::EventStream, Some(9), None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "AC-7: poll_interval_secs=9 (below 10s minimum) must be rejected"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_poll_interval_5s() {
    // RED: validate_table_spec is todo!()
    // EC-002 canonical case: poll_interval = "5s" in the spec means 5 seconds
    let table = make_table_spec(TableType::EventStream, Some(5), None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "EC-002: poll_interval_secs=5 must be rejected with descriptive error"
    );
    // Error message must be descriptive (not just "invalid")
    if let Err(e) = result {
        let msg = format!("{e}");
        assert!(
            msg.contains("poll_interval") || msg.contains("minimum") || msg.contains("10"),
            "AC-7: error message must mention poll_interval, minimum, or 10; got: {msg}"
        );
    }
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_poll_interval_zero() {
    // RED: validate_table_spec is todo!()
    // Boundary: 0 seconds is clearly below minimum
    let table = make_table_spec(TableType::EventStream, Some(0), None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "poll_interval_secs=0 must be rejected (below 10s minimum)"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_accepts_maximum_retention() {
    // RED: validate_table_spec is todo!()
    // Boundary: retention_secs = 604800 (7 days) is the maximum — must be accepted
    let table = make_table_spec(TableType::EventStream, Some(60), Some(604800));
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_ok(),
        "AC-7: retention_secs=604800 (7 days maximum) must be accepted; got: {result:?}"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_retention_above_maximum() {
    // RED: validate_table_spec is todo!()
    // AC-7: retention_secs = 604801 exceeds the 7-day maximum — must be rejected
    let table = make_table_spec(TableType::EventStream, Some(60), Some(604801));
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "AC-7: retention_secs=604801 (exceeds 7 day maximum) must be rejected"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_poll_interval_on_point_in_time() {
    // RED: validate_table_spec is todo!()
    // AC-7: poll_interval_secs is only valid for EventStream tables
    let table = make_table_spec(TableType::PointInTime, Some(60), None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "AC-7: poll_interval_secs on PointInTime table must be rejected"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_rejects_retention_on_point_in_time() {
    // RED: validate_table_spec is todo!()
    // AC-7: retention_secs is only valid for EventStream tables
    let table = make_table_spec(TableType::PointInTime, None, Some(86400));
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_err(),
        "AC-7: retention_secs on PointInTime table must be rejected"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_accepts_event_stream_with_no_poll_interval() {
    // RED: validate_table_spec is todo!()
    // EventStream with no poll_interval/retention is valid (uses defaults)
    let table = make_table_spec(TableType::EventStream, None, None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_ok(),
        "EventStream table without poll_interval or retention must be valid (uses defaults)"
    );
}

#[test]
fn test_BC_2_08_validate_table_spec_accepts_point_in_time_with_no_optional_fields() {
    // RED: validate_table_spec is todo!()
    // PointInTime with all None fields is the backward-compatible case — must pass
    let table = make_table_spec(TableType::PointInTime, None, None);
    let result = SpecLoader::validate_table_spec("crowdstrike", &table);
    assert!(
        result.is_ok(),
        "PointInTime table with no optional fields must pass validation (backward compat)"
    );
}

// route_table_query tests live in prism-sensors/src/tests/ to avoid adding
// prism-sensors as a dev-dep of prism-spec-engine (S-2.08 architecture compliance).
