#![allow(non_snake_case)]
//! BC-2.16.003: Column-to-OCSF Mapping at Query Time
//!
//! Tests cover:
//! - Happy path: all columns mapped -> all OCSF fields populated
//! - Mixed: some columns with ocsf_field, some without -> unmapped go to raw_extensions
//! - Type coercion: string "42" -> integer field succeeds
//! - Type coercion failure: "not-a-number" -> integer field -> raw_extensions + warning
//! - Record never dropped on coercion failure (invariant BC-2.16.003)
//! - Invalid ocsf_class -> base_event fallback, startup warning
//! - Cross-sensor: two specs both map device_ip -> device.ip (structural test)
//!
//! AC-3 (S-1.11): spec column "created_timestamp" -> ocsf_field "time" -> time populated

use prism_core::ColumnType;
use prism_spec_engine::column_mapping::ColumnMapper;
use prism_spec_engine::spec_parser::{ColumnSpec, FetchStep, TableSpec};

fn make_table_with_mapping(
    col_name: &str,
    col_type: ColumnType,
    ocsf_field: Option<&str>,
) -> TableSpec {
    TableSpec {
        table_name: "alerts".to_string(),
        ocsf_class: "security_finding".to_string(),
        columns: vec![ColumnSpec {
            name: col_name.to_string(),
            column_type: col_type,
            ocsf_field: ocsf_field.map(|s| s.to_string()),
            options: vec![],
        }],
        steps: vec![FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/data".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }],
        table_type: Default::default(),
        poll_interval_secs: None,
        retention_secs: None,
    }
}

/// AC-3 / BC-2.16.003 postcondition: column with ocsf_field -> mapped field populated.
/// Canonical: "created_timestamp" -> "time".
#[test]
fn test_BC_2_16_003_maps_column_to_ocsf_field_time() {
    let table = make_table_with_mapping("created_timestamp", ColumnType::Datetime, Some("time"));
    let raw = serde_json::json!({ "created_timestamp": "2024-01-15T10:30:00Z" });

    let result = ColumnMapper::map_record(&raw, &table).expect("mapping must not error");

    assert!(
        result.mapped_fields.contains_key("time"),
        "OCSF field 'time' must be populated from 'created_timestamp'"
    );
    assert!(
        !result.raw_extensions.contains_key("created_timestamp"),
        "'created_timestamp' must not appear in raw_extensions when mapped"
    );
}

/// BC-2.16.003 postcondition: column without ocsf_field -> raw_extensions.
#[test]
fn test_BC_2_16_003_unmapped_column_goes_to_raw_extensions() {
    let table = make_table_with_mapping("vendor_specific_field", ColumnType::String, None);
    let raw = serde_json::json!({ "vendor_specific_field": "some_value" });

    let result = ColumnMapper::map_record(&raw, &table).expect("mapping must not error");

    assert!(
        result.raw_extensions.contains_key("vendor_specific_field"),
        "column without ocsf_field must go to raw_extensions"
    );
    assert!(
        !result.mapped_fields.contains_key("vendor_specific_field"),
        "unmapped column must not appear in mapped_fields"
    );
}

/// BC-2.16.003 mixed mapping: some columns mapped, some not.
#[test]
fn test_BC_2_16_003_mixed_mapping_partial_ocsf_partial_raw_extensions() {
    let table = TableSpec {
        table_name: "events".to_string(),
        ocsf_class: "security_finding".to_string(),
        columns: vec![
            ColumnSpec {
                name: "event_time".to_string(),
                column_type: ColumnType::Datetime,
                ocsf_field: Some("time".to_string()),
                options: vec![],
            },
            ColumnSpec {
                name: "internal_ref".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            },
        ],
        steps: vec![FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/events".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }],
        table_type: Default::default(),
        poll_interval_secs: None,
        retention_secs: None,
    };
    let raw = serde_json::json!({
        "event_time": "2024-01-15T10:30:00Z",
        "internal_ref": "ref-001"
    });

    let result = ColumnMapper::map_record(&raw, &table).expect("mapping must not error");

    assert!(
        result.mapped_fields.contains_key("time"),
        "event_time->time must be mapped"
    );
    assert!(
        result.raw_extensions.contains_key("internal_ref"),
        "internal_ref must go to raw_extensions"
    );
}

/// BC-2.16.003 type coercion: string "42" -> OCSF integer field succeeds.
/// Canonical test vector from BC-2.16.003.
#[test]
fn test_BC_2_16_003_coerces_string_42_to_integer_field() {
    let col = ColumnSpec {
        name: "event_id".to_string(),
        column_type: ColumnType::String, // declared as string in sensor
        ocsf_field: Some("metadata.event_code".to_string()),
        options: vec![],
    };

    let result = ColumnMapper::coerce_value(&serde_json::json!("42"), &col, "metadata.event_code");

    assert!(
        result.is_ok(),
        "string '42' to int field must coerce successfully"
    );
    assert_eq!(result.unwrap(), serde_json::json!(42));
}

/// BC-2.16.003 type coercion failure: non-parseable string -> raw_extensions + warning.
/// Invariant: record is NEVER dropped due to coercion failure.
#[test]
fn test_BC_2_16_003_coercion_failure_produces_warning_record_not_dropped() {
    let col = ColumnSpec {
        name: "event_code".to_string(),
        column_type: ColumnType::String,
        ocsf_field: Some("metadata.event_code".to_string()),
        options: vec![],
    };

    let result = ColumnMapper::coerce_value(
        &serde_json::json!("not-a-number"),
        &col,
        "metadata.event_code",
    );

    assert!(
        result.is_err(),
        "non-parseable string to int must return CoercionWarning"
    );
    let warning = result.unwrap_err();
    assert_eq!(warning.column_name, "event_code");
    assert!(
        warning.actual_value.contains("not-a-number"),
        "warning must include the actual value"
    );
}

/// BC-2.16.003 invariant: full record mapping with coercion failure -> record included,
/// coercion_warnings non-empty, raw_extensions has failed field.
#[test]
fn test_BC_2_16_003_invariant_record_never_dropped_on_coercion_failure() {
    let table = TableSpec {
        table_name: "events".to_string(),
        ocsf_class: "security_finding".to_string(),
        columns: vec![
            ColumnSpec {
                name: "event_id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: Some("metadata.event_code".to_string()),
                options: vec![],
            },
            ColumnSpec {
                name: "event_name".to_string(),
                column_type: ColumnType::String,
                ocsf_field: Some("activity_name".to_string()),
                options: vec![],
            },
        ],
        steps: vec![FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/events".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }],
        table_type: Default::default(),
        poll_interval_secs: None,
        retention_secs: None,
    };
    let raw = serde_json::json!({
        "event_id": "not-a-number",  // will fail coercion
        "event_name": "Detection"    // will succeed
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("map_record must return Ok — record never dropped");

    // The record IS returned (not dropped)
    assert!(
        !result.coercion_warnings.is_empty(),
        "coercion warning must be present for event_id"
    );
    assert!(
        result.raw_extensions.contains_key("event_id"),
        "failed field must be in raw_extensions"
    );
    assert!(
        result.mapped_fields.contains_key("activity_name"),
        "successful field must still be mapped"
    );
}
