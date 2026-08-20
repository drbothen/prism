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
use prism_spec_engine::{
    column_mapping::ColumnMapper,
    spec_parser::{ColumnSpec, FetchStep, TableSpec},
};

fn make_table_with_mapping(
    col_name: &str,
    col_type: ColumnType,
    ocsf_field: Option<&str>,
) -> TableSpec {
    TableSpec::new_point_in_time(
        "alerts",
        "security_finding",
        vec![ColumnSpec::new(
            col_name,
            col_type,
            ocsf_field.map(|s| s.to_string()),
            vec![],
        )],
        vec![FetchStep::new(
            "fetch",
            "GET",
            "/data",
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        )],
    )
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
    let table = TableSpec::new_point_in_time(
        "events",
        "security_finding",
        vec![
            ColumnSpec::new(
                "event_time",
                ColumnType::Datetime,
                Some("time".to_string()),
                vec![],
            ),
            ColumnSpec::new("internal_ref", ColumnType::String, None, vec![]),
        ],
        vec![FetchStep::new(
            "fetch",
            "GET",
            "/events",
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        )],
    );
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

/// BC-2.16.003 type coercion: when column_type is String, the String-type-first rule
/// takes precedence over the numeric ocsf_field heuristic.
///
/// Updated from the original "string '42' coerces to integer 42" assertion because the
/// String-type-first fix (LIVE-DRIFT-003) changed this behavior: if the column declares
/// `column_type = "string"`, the value is preserved as a string regardless of the OCSF
/// field suffix. This is correct for production use cases where sensors return string IDs
/// (e.g. UUID, mixed-type ID) mapped to `*.uid` fields.
///
/// For a `ColumnType::Integer` column with the same ocsf_field, coercion still applies.
#[test]
fn test_BC_2_16_003_coerces_string_42_to_integer_field() {
    // String column + numeric ocsf_field: String-type-first → preserve as string.
    let col_string = ColumnSpec::new(
        "event_id",
        ColumnType::String,
        Some("metadata.event_code".to_string()),
        vec![],
    );
    let result_string =
        ColumnMapper::coerce_value(&serde_json::json!("42"), &col_string, "metadata.event_code");
    assert!(
        result_string.is_ok(),
        "string column must coerce successfully (no warning)"
    );
    // String-type-first: column_type=String wins over numeric ocsf_field suffix.
    // The value is preserved as a JSON string, not coerced to integer.
    assert_eq!(
        result_string.unwrap(),
        serde_json::json!("42"),
        "ColumnType::String column preserves string value even for numeric-suffix ocsf_field \
         (String-type-first rule, LIVE-DRIFT-003)"
    );

    // Integer column + numeric ocsf_field: coercion still applies when column_type is Integer.
    let col_integer = ColumnSpec::new(
        "event_id",
        ColumnType::Integer,
        Some("metadata.event_code".to_string()),
        vec![],
    );
    let result_integer = ColumnMapper::coerce_value(
        &serde_json::json!("42"),
        &col_integer,
        "metadata.event_code",
    );
    assert!(
        result_integer.is_ok(),
        "integer column: string '42' to int field must coerce successfully"
    );
    assert_eq!(
        result_integer.unwrap(),
        serde_json::json!(42),
        "ColumnType::Integer column coerces string '42' to Number(42)"
    );
}

/// BC-2.16.003 coercion with String column: no CoercionWarning (String-type-first rule).
///
/// Updated from the original "non-parseable string → CoercionWarning" assertion because
/// the String-type-first fix (LIVE-DRIFT-003) changed this behavior for String columns:
/// `ColumnType::String` now bypasses the numeric-suffix heuristic. The value is returned
/// as-is without a CoercionWarning. This prevents usernames / string IDs mapped to OCSF
/// `*.uid` fields from being spuriously demoted to `raw_extensions`.
///
/// For a `ColumnType::Integer` column, the original behavior (CoercionWarning on parse
/// failure) is preserved.
#[test]
fn test_BC_2_16_003_coercion_failure_produces_warning_record_not_dropped() {
    // String column: String-type-first → no CoercionWarning even for non-parseable values.
    let col_string = ColumnSpec::new(
        "event_code",
        ColumnType::String,
        Some("metadata.event_code".to_string()),
        vec![],
    );
    let result_string = ColumnMapper::coerce_value(
        &serde_json::json!("not-a-number"),
        &col_string,
        "metadata.event_code",
    );
    assert!(
        result_string.is_ok(),
        "ColumnType::String column must NOT return CoercionWarning (String-type-first rule, \
         LIVE-DRIFT-003); got: {result_string:?}"
    );
    assert_eq!(
        result_string.unwrap(),
        serde_json::json!("not-a-number"),
        "ColumnType::String column must preserve non-parseable value as string"
    );

    // Integer column: CoercionWarning still applies for non-parseable strings.
    let col_integer = ColumnSpec::new(
        "event_code",
        ColumnType::Integer,
        Some("metadata.event_code".to_string()),
        vec![],
    );
    let result_integer = ColumnMapper::coerce_value(
        &serde_json::json!("not-a-number"),
        &col_integer,
        "metadata.event_code",
    );
    assert!(
        result_integer.is_err(),
        "ColumnType::Integer column: non-parseable string must still return CoercionWarning"
    );
    let warning = result_integer.unwrap_err();
    assert_eq!(warning.column_name, "event_code");
    assert!(
        warning.actual_value.contains("not-a-number"),
        "warning must include the actual value"
    );
}

/// BC-2.16.003 invariant: record is NEVER dropped — String-type-first variant.
///
/// Covers `map_record` integration with String-type columns (LIVE-DRIFT-003):
/// `ColumnType::String` columns produce no `coercion_warnings` because the
/// String-type-first guard bypasses the numeric-suffix heuristic. All string values
/// are preserved in `mapped_fields`. The record is returned with all columns mapped
/// and `raw_extensions` empty.
///
/// For the CoercionWarning path (`map_record` integration with Integer column +
/// non-parseable string → `coercion_warnings` non-empty + value in `raw_extensions`),
/// see `test_BC_2_16_003_invariant_record_never_dropped_integer_column_coercion_failure`.
#[test]
fn test_BC_2_16_003_invariant_record_never_dropped_string_type_first_variant() {
    // String columns: String-type-first → both columns mapped successfully, no warnings.
    let table = TableSpec::new_point_in_time(
        "events",
        "security_finding",
        vec![
            ColumnSpec::new(
                "event_id",
                ColumnType::String,
                Some("metadata.event_code".to_string()),
                vec![],
            ),
            ColumnSpec::new(
                "event_name",
                ColumnType::String,
                Some("activity_name".to_string()),
                vec![],
            ),
        ],
        vec![FetchStep::new(
            "fetch",
            "GET",
            "/events",
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        )],
    );
    let raw = serde_json::json!({
        "event_id": "not-a-number",
        "event_name": "Detection"
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("map_record must return Ok — record never dropped");

    // Record is returned (not dropped) — core invariant.
    // With String-type-first: both fields go to mapped_fields (no CoercionWarning, no raw_extensions).
    assert!(
        result.coercion_warnings.is_empty(),
        "ColumnType::String columns must NOT produce CoercionWarnings (String-type-first, LIVE-DRIFT-003)"
    );
    assert!(
        result.mapped_fields.contains_key("metadata.event_code"),
        "event_id must be in mapped_fields (not raw_extensions) with String-type-first rule"
    );
    assert!(
        result.mapped_fields.contains_key("activity_name"),
        "event_name must be in mapped_fields"
    );
    assert!(
        result.raw_extensions.is_empty(),
        "raw_extensions must be empty when all columns are mapped (String-type-first)"
    );
}

/// BC-2.16.003 invariant: record is NEVER dropped — Integer-column coercion-failure variant.
///
/// Covers the `map_record` integration for the `Err(CoercionWarning)` branch:
/// when `ColumnType::Integer` column with a numeric-suffix `ocsf_field` receives a
/// non-parseable string, `coerce_value` returns `Err(CoercionWarning)` and `map_record`
/// must:
///   1. NOT drop the record (returns `Ok(MappedRecord)`)
///   2. Push the warning to `coercion_warnings` with the correct `column_name`
///   3. Insert the raw value into `raw_extensions` (fallback; field NOT in `mapped_fields`)
///
/// This is the guard for BC-2.16.003 postcondition:
///   "Type coercion failures → raw_extensions + CoercionWarning (non-fatal, record kept)"
#[test]
fn test_BC_2_16_003_invariant_record_never_dropped_integer_column_coercion_failure() {
    let table = TableSpec::new_point_in_time(
        "events",
        "security_finding",
        vec![ColumnSpec::new(
            "event_id",
            ColumnType::Integer,
            Some("metadata.event_code".to_string()),
            vec![],
        )],
        vec![FetchStep::new(
            "fetch",
            "GET",
            "/events",
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        )],
    );
    // Non-parseable string for an Integer column on a numeric-suffix OCSF path.
    let raw = serde_json::json!({ "event_id": "not-a-number" });

    // Core invariant: record is never dropped — map_record must return Ok.
    let result = ColumnMapper::map_record(&raw, &table)
        .expect("map_record must return Ok even on coercion failure — record never dropped");

    // Coercion failure path: one warning with the correct column_name.
    assert_eq!(
        result.coercion_warnings.len(),
        1,
        "exactly one CoercionWarning expected for the Integer column parse failure"
    );
    assert_eq!(
        result.coercion_warnings[0].column_name, "event_id",
        "CoercionWarning must name the failing column"
    );
    assert!(
        result.coercion_warnings[0]
            .actual_value
            .contains("not-a-number"),
        "CoercionWarning must capture the raw value that failed parsing"
    );

    // Failed field diverted to raw_extensions, NOT mapped_fields.
    assert!(
        result.raw_extensions.contains_key("event_id"),
        "failed Integer column must be diverted to raw_extensions \
         (BC-2.16.003 non-fatal coercion contract)"
    );
    assert!(
        !result.mapped_fields.contains_key("metadata.event_code"),
        "failed Integer column must NOT appear in mapped_fields"
    );
}

/// RG-001 / AC-001 / BC-2.16.003 EC-016-013-007 (KNOWN GAP):
///
/// `coerce_value` MUST return `Err(CoercionWarning)` for `column_type = "string"` with
/// `Value::Array` input. The array value must NOT be passed through via `to_string()`.
///
/// Before this fix, the wildcard `other => other.clone()` arm in the String-type-first
/// block (`coerce_value`) passed the Array through as `Ok(Value::Array)`, allowing
/// structured JSON to reach a typed Arrow string column — silent data corruption.
///
/// SAP-3 reachability note (defense-in-depth): `coerce_value` is on Path B
/// (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
/// callers per ADR-058 §K5. This test is intentionally defense-in-depth / forward-compat
/// per SAP-3 rule 2/3. The equivalent LIVE coercion behavior on Path A is covered by
/// RG-006/RG-008/RG-009 (`build_column_array`).
#[test]
fn test_coerce_value_string_type_array_input_returns_err_coercion_warning() {
    let col = ColumnSpec::new(
        "tags",
        ColumnType::String,
        Some("finding.tags".to_string()),
        vec![],
    );
    let array_value = serde_json::json!(["tag1", "tag2"]);
    let result = ColumnMapper::coerce_value(&array_value, &col, "finding.tags");
    assert!(
        result.is_err(),
        "AC-001 / EC-016-013-007: coerce_value must return Err(CoercionWarning) for \
         String column + Array input (NOT pass-through via other.clone() or to_string()); \
         got Ok({:?})",
        result.ok()
    );
}

/// RG-002 / AC-002 / BC-2.16.003 EC-016-013-008 (KNOWN GAP):
///
/// `coerce_value` MUST return `Err(CoercionWarning)` for `column_type = "string"` with
/// `Value::Object` input. The object value must NOT be passed through via `to_string()`.
///
/// Before this fix, the wildcard `other => other.clone()` arm in the String-type-first
/// block passed the Object through as `Ok(Value::Object)`, allowing structured JSON to
/// reach a typed Arrow string column — silent data corruption.
///
/// SAP-3 reachability note (defense-in-depth): `coerce_value` is on Path B, which has
/// zero live production callers per ADR-058 §K5 — defense-in-depth / forward-compat.
/// The equivalent LIVE coercion behavior on Path A is covered by RG-006/RG-008/RG-009.
#[test]
fn test_coerce_value_string_type_object_input_returns_err_coercion_warning() {
    let col = ColumnSpec::new(
        "metadata",
        ColumnType::String,
        Some("finding.metadata".to_string()),
        vec![],
    );
    let object_value = serde_json::json!({"key": "value"});
    let result = ColumnMapper::coerce_value(&object_value, &col, "finding.metadata");
    assert!(
        result.is_err(),
        "AC-002 / EC-016-013-008: coerce_value must return Err(CoercionWarning) for \
         String column + Object input (NOT pass-through via other.clone() or to_string()); \
         got Ok({:?})",
        result.ok()
    );
}

/// RG-003 / AC-003 / BC-2.16.003 EC-016-013-009 (KNOWN GAP):
///
/// `coerce_value` for `column_type = "integer"` + `Value::String("42")` on a
/// NON-numeric-suffix OCSF path MUST return `Ok(Value::Number(42))`.
///
/// AC-003 extends Rule 2 behavior to ALL Integer+String combinations (not just
/// numeric-suffix OCSF paths). `column_type` is authoritative: when declared Integer,
/// a String input must always attempt `s.parse::<i64>()` regardless of OCSF path suffix.
///
/// Before this fix, non-numeric-suffix paths fell through to `Ok(value.clone())`
/// without a parse attempt, returning the raw `Value::String("42")` — data loss.
///
/// Non-numeric path: `"device.hostname"` — last segment "hostname" is NOT in the
/// `is_numeric_ocsf_field` suffix list (event_code, class_uid, uid, port, etc.).
///
/// SAP-3 reachability note (defense-in-depth): `coerce_value` is on Path B, zero live
/// production callers per ADR-058 §K5 — defense-in-depth / forward-compat.
#[test]
fn test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number() {
    let col = ColumnSpec::new(
        "port_number",
        ColumnType::Integer,
        Some("device.hostname".to_string()),
        vec![],
    );
    let string_value = serde_json::json!("42");
    let result = ColumnMapper::coerce_value(&string_value, &col, "device.hostname");
    assert!(
        result.is_ok(),
        "AC-003 / EC-016-013-009: coerce_value must return Ok for Integer column + \
         parseable String('42') on non-numeric OCSF path ('device.hostname'); \
         got Err({:?})",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        serde_json::json!(42),
        "AC-003 / EC-016-013-009: coerce_value must parse String('42') → Number(42) for \
         Integer column on non-numeric OCSF path; currently falls through to \
         Ok(String('42')) without parse attempt (non-numeric path skips Rule 2 block)"
    );
}

/// RG-004 / AC-003 / BC-2.16.003 EC-016-013-009 (KNOWN GAP):
///
/// `coerce_value` for `column_type = "integer"` + non-parseable `Value::String("not-a-number")`
/// on a NON-numeric-suffix OCSF path MUST return `Err(CoercionWarning)`.
///
/// Before this fix, non-numeric-suffix paths fell through to `Ok(value.clone())`
/// without a parse attempt, returning `Ok(String("not-a-number"))` instead of a
/// CoercionWarning — silently allowing the wrong type into a typed Arrow Integer column.
///
/// Same non-numeric path as RG-003: `"device.hostname"` — "hostname" is NOT in the
/// `is_numeric_ocsf_field` suffix list.
///
/// SAP-3 reachability note (defense-in-depth): `coerce_value` is on Path B, zero live
/// production callers per ADR-058 §K5 — defense-in-depth / forward-compat.
#[test]
fn test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err() {
    let col = ColumnSpec::new(
        "source_ip",
        ColumnType::Integer,
        Some("device.hostname".to_string()),
        vec![],
    );
    let string_value = serde_json::json!("not-a-number");
    let result = ColumnMapper::coerce_value(&string_value, &col, "device.hostname");
    assert!(
        result.is_err(),
        "AC-003 / EC-016-013-009: coerce_value must return Err(CoercionWarning) for \
         Integer column + non-parseable String on non-numeric OCSF path ('device.hostname'); \
         got Ok({:?}) — currently falls through to Ok(String) without parse attempt",
        result.ok()
    );
}
