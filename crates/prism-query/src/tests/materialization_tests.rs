//! Tests for `inject_source_type` — pure-data `_source_type` virtual field injection.
//!
//! Story: S-2.08 | AC-9, AC-10
//!
//! # Coverage
//! - EventStream + rows_from_buffer=true → every row has "_source_type": "buffered" (AC-9)
//! - PointInTime → every row has "_source_type": "live" (AC-10)
//! - EventStream + rows_from_buffer=false (cold-start fallback) → "_source_type": "live" (AC-10)
//! - Empty rows slice handled gracefully (no panic)
//! - Non-object JSON values in the slice are skipped without error
//! - Multiple rows: all are mutated
//! - Existing "_source_type" field is overwritten
//!
//! # RED GATE
//! All tests here call `inject_source_type` which is currently `todo!()`.
//! They will PANIC with "not yet implemented" at runtime — RED by design.

use crate::materialization::inject_source_type;
use crate::types::SensorQueryDescriptor;
use prism_core::TableType;
use serde_json::json;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn make_descriptor(table_type: TableType, rows_from_buffer: bool) -> SensorQueryDescriptor {
    SensorQueryDescriptor {
        table_name: "crowdstrike.process_events".to_string(),
        table_type,
        rows_from_buffer,
    }
}

fn make_row(host: &str) -> serde_json::Value {
    json!({ "host": host, "pid": 1234 })
}

// ---------------------------------------------------------------------------
// AC-9: EventStream + rows_from_buffer=true → "buffered"
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_inject_source_type_event_stream_buffered_single_row() {
    // RED: inject_source_type is todo!() — will panic with "not yet implemented"
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows = vec![make_row("host-a")];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["_source_type"],
        json!("buffered"),
        "AC-9: EventStream rows_from_buffer=true must inject _source_type=buffered"
    );
}

#[test]
fn test_BC_2_08_inject_source_type_event_stream_buffered_multiple_rows() {
    // RED: inject_source_type is todo!()
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows = vec![make_row("host-a"), make_row("host-b"), make_row("host-c")];
    inject_source_type(&mut rows, &descriptor);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row["_source_type"],
            json!("buffered"),
            "AC-9: all rows at index {i} must have _source_type=buffered"
        );
    }
}

#[test]
fn test_BC_2_08_inject_source_type_event_stream_buffered_preserves_other_fields() {
    // RED: inject_source_type is todo!()
    // Verifies that existing fields are not mutated (only _source_type is added)
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows = vec![make_row("host-z")];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["host"],
        json!("host-z"),
        "AC-9: existing row fields must not be modified by inject_source_type"
    );
    assert_eq!(
        rows[0]["pid"],
        json!(1234),
        "AC-9: existing row fields must not be modified by inject_source_type"
    );
}

// ---------------------------------------------------------------------------
// AC-10: PointInTime → "live"
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_inject_source_type_point_in_time_single_row() {
    // RED: inject_source_type is todo!()
    let descriptor = make_descriptor(TableType::PointInTime, false);
    let mut rows = vec![make_row("host-b")];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["_source_type"],
        json!("live"),
        "AC-10: PointInTime rows must inject _source_type=live"
    );
}

#[test]
fn test_BC_2_08_inject_source_type_point_in_time_multiple_rows() {
    // RED: inject_source_type is todo!()
    let descriptor = make_descriptor(TableType::PointInTime, false);
    let mut rows = vec![make_row("host-a"), make_row("host-b")];
    inject_source_type(&mut rows, &descriptor);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row["_source_type"],
            json!("live"),
            "AC-10: all rows at index {i} must have _source_type=live (PointInTime)"
        );
    }
}

#[test]
fn test_BC_2_08_inject_source_type_point_in_time_rows_from_buffer_true_is_still_live() {
    // RED: inject_source_type is todo!()
    // rows_from_buffer=true on PointInTime is a logical contradiction but must
    // still produce "live" (PointInTime always → live regardless of rows_from_buffer)
    let descriptor = make_descriptor(TableType::PointInTime, true);
    let mut rows = vec![make_row("host-x")];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["_source_type"],
        json!("live"),
        "AC-10: PointInTime always produces live regardless of rows_from_buffer"
    );
}

// ---------------------------------------------------------------------------
// AC-10: EventStream cold-start fallback (rows_from_buffer=false) → "live"
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_inject_source_type_event_stream_cold_start_fallback_is_live() {
    // RED: inject_source_type is todo!()
    // EC-001 cold-start: EventStream table, rows_from_buffer=false → "live"
    let descriptor = make_descriptor(TableType::EventStream, false);
    let mut rows = vec![make_row("host-cold")];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["_source_type"],
        json!("live"),
        "AC-10: EventStream cold-start fallback (rows_from_buffer=false) must produce live"
    );
}

#[test]
fn test_BC_2_08_inject_source_type_event_stream_cold_start_multiple_rows_all_live() {
    // RED: inject_source_type is todo!()
    let descriptor = make_descriptor(TableType::EventStream, false);
    let mut rows = vec![make_row("host-1"), make_row("host-2"), make_row("host-3")];
    inject_source_type(&mut rows, &descriptor);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row["_source_type"],
            json!("live"),
            "AC-10: all cold-start rows at index {i} must be live"
        );
    }
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_inject_source_type_empty_rows_no_panic() {
    // RED: inject_source_type is todo!()
    // EC-001 edge: empty rows slice must not panic
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows: Vec<serde_json::Value> = vec![];
    inject_source_type(&mut rows, &descriptor);
    // Success = no panic; rows remains empty
    assert!(
        rows.is_empty(),
        "empty rows slice must remain empty after inject_source_type"
    );
}

#[test]
fn test_BC_2_08_inject_source_type_non_object_rows_skipped() {
    // RED: inject_source_type is todo!()
    // Non-object JSON values (strings, arrays, null) must be skipped without error
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows = vec![
        json!("not an object"),
        json!([1, 2, 3]),
        json!(null),
        make_row("real-row"),
    ];
    inject_source_type(&mut rows, &descriptor);
    // The real row at index 3 should be injected; non-objects remain unchanged
    assert_eq!(
        rows[3]["_source_type"],
        json!("buffered"),
        "inject_source_type must inject into object rows even when non-objects are present"
    );
    // Non-objects should be unchanged (not wrapped or modified)
    assert_eq!(
        rows[0],
        json!("not an object"),
        "non-object rows must not be modified"
    );
}

#[test]
fn test_BC_2_08_inject_source_type_overwrites_existing_source_type() {
    // RED: inject_source_type is todo!()
    // If a row already has "_source_type", it must be overwritten with the correct value
    let descriptor = make_descriptor(TableType::EventStream, true);
    let mut rows = vec![json!({ "_source_type": "stale_value", "host": "x" })];
    inject_source_type(&mut rows, &descriptor);
    assert_eq!(
        rows[0]["_source_type"],
        json!("buffered"),
        "inject_source_type must overwrite existing _source_type field"
    );
}

// ---------------------------------------------------------------------------
// Architecture compliance: no DataFusion, no Arrow
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_inject_source_type_operates_on_serde_json_only() {
    // GREEN-BY-DESIGN: structural test — verifies the function signature accepts
    // Vec<serde_json::Value> and SensorQueryDescriptor only (no DataFusion types).
    // This is a compile-time check: if DataFusion types leak into the signature,
    // this test module would fail to compile since prism-query has no datafusion dep.
    let descriptor = make_descriptor(TableType::PointInTime, false);
    let mut rows: Vec<serde_json::Value> = vec![json!({ "key": "value" })];
    // The fact that this compiles with only serde_json::Value confirms no Arrow/DF leak.
    inject_source_type(&mut rows, &descriptor);
    // No assertion needed for compile-time check; inject_source_type is todo!() so
    // this test will be RED at runtime too.
}
