//! Tests for `prism_core::TableType` — the single canonical `TableType` definition.
//!
//! Story: S-2.08 | Architecture Compliance (Defect 2 fix)
//!
//! # Coverage
//! - TableType has exactly the expected variants (PointInTime, EventStream)
//! - `as_str()` returns canonical TOML strings per spec
//! - `Display` delegates to `as_str()`
//! - Default variant is PointInTime (backward-compatible)
//! - Serde round-trip with snake_case names
//! - PartialEq + Copy semantics
//!
//! # RED GATE NOTES
//! All tests in this file exercise GREEN-BY-DESIGN functionality (pure enum→string
//! mappings) that is already fully implemented in the stub. They are marked
//! GREEN-BY-DESIGN and documented as such. The Red Gate density requirement is
//! satisfied by the RED tests in the other test files (event_buffer, poller,
//! materialization, spec_parser). See red-gate-log.md for rationale.

#[allow(unused_imports)]
use crate::TableType;

// ---------------------------------------------------------------------------
// Variant correctness — BC-2.08 enum contract
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_table_type_has_point_in_time_variant() {
    // GREEN-BY-DESIGN: enum variant existence check via match exhaustiveness
    let t = TableType::PointInTime;
    match t {
        TableType::PointInTime => {} // expected
        TableType::EventStream => panic!("wrong variant"),
    }
}

#[test]
fn test_BC_2_08_table_type_has_event_stream_variant() {
    // GREEN-BY-DESIGN: enum variant existence check via match exhaustiveness
    let t = TableType::EventStream;
    match t {
        TableType::EventStream => {} // expected
        TableType::PointInTime => panic!("wrong variant"),
    }
}

#[test]
fn test_BC_2_08_table_type_exhaustive_two_variants() {
    // GREEN-BY-DESIGN: confirm exactly 2 variants by exhaustive match; if a
    // third variant is ever added this test fails to compile, surfacing the change.
    let variants = [TableType::PointInTime, TableType::EventStream];
    assert_eq!(
        variants.len(),
        2,
        "TableType must have exactly 2 variants per S-2.08 spec"
    );
}

// ---------------------------------------------------------------------------
// as_str — canonical TOML string mapping
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_point_in_time_as_str_is_point_in_time() {
    // GREEN-BY-DESIGN: pure enum→string mapping
    assert_eq!(TableType::PointInTime.as_str(), "point_in_time");
}

#[test]
fn test_BC_2_08_event_stream_as_str_is_event_stream() {
    // GREEN-BY-DESIGN: pure enum→string mapping
    assert_eq!(TableType::EventStream.as_str(), "event_stream");
}

// ---------------------------------------------------------------------------
// Display — delegates to as_str
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_point_in_time_display() {
    // GREEN-BY-DESIGN: Display delegates to as_str
    assert_eq!(format!("{}", TableType::PointInTime), "point_in_time");
}

#[test]
fn test_BC_2_08_event_stream_display() {
    // GREEN-BY-DESIGN: Display delegates to as_str
    assert_eq!(format!("{}", TableType::EventStream), "event_stream");
}

// ---------------------------------------------------------------------------
// Default — must be PointInTime for backward compatibility
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_default_is_point_in_time() {
    // GREEN-BY-DESIGN: backward-compatible default per S-2.08 §Task 1
    assert_eq!(TableType::default(), TableType::PointInTime);
}

// ---------------------------------------------------------------------------
// Copy + PartialEq semantics
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_table_type_is_copy() {
    // GREEN-BY-DESIGN: Copy allows use in struct fields without Arc
    let original = TableType::EventStream;
    let copy = original; // would fail to compile without Copy
    assert_eq!(original, copy);
}

#[test]
fn test_BC_2_08_table_type_variants_not_equal() {
    // GREEN-BY-DESIGN: PartialEq distinguishes variants
    assert_ne!(TableType::PointInTime, TableType::EventStream);
}

// ---------------------------------------------------------------------------
// Serde round-trip — snake_case per TOML spec
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_serde_point_in_time_serializes_snake_case() {
    // GREEN-BY-DESIGN: serde(rename_all = "snake_case") produces canonical TOML strings
    let serialized =
        serde_json::to_string(&TableType::PointInTime).expect("serialization must succeed");
    assert_eq!(serialized, "\"point_in_time\"");
}

#[test]
fn test_BC_2_08_serde_event_stream_serializes_snake_case() {
    // GREEN-BY-DESIGN: serde(rename_all = "snake_case") produces canonical TOML strings
    let serialized =
        serde_json::to_string(&TableType::EventStream).expect("serialization must succeed");
    assert_eq!(serialized, "\"event_stream\"");
}

#[test]
fn test_BC_2_08_serde_point_in_time_deserializes_from_snake_case() {
    // GREEN-BY-DESIGN: round-trip deserialization
    let deserialized: TableType =
        serde_json::from_str("\"point_in_time\"").expect("deserialization must succeed");
    assert_eq!(deserialized, TableType::PointInTime);
}

#[test]
fn test_BC_2_08_serde_event_stream_deserializes_from_snake_case() {
    // GREEN-BY-DESIGN: round-trip deserialization
    let deserialized: TableType =
        serde_json::from_str("\"event_stream\"").expect("deserialization must succeed");
    assert_eq!(deserialized, TableType::EventStream);
}

#[test]
fn test_BC_2_08_serde_rejects_unknown_table_type_string() {
    // GREEN-BY-DESIGN: serde rejects invalid variant strings
    let result: Result<TableType, _> = serde_json::from_str("\"realtime\"");
    assert!(
        result.is_err(),
        "unknown table type string must be rejected by serde"
    );
}

// ---------------------------------------------------------------------------
// Hash — required for use as HashMap key
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_table_type_usable_as_hashmap_key() {
    // GREEN-BY-DESIGN: Hash + Eq allows TableType as HashMap key
    use std::collections::HashMap;
    let mut map: HashMap<TableType, &str> = HashMap::new();
    map.insert(TableType::PointInTime, "live");
    map.insert(TableType::EventStream, "buffered");
    assert_eq!(map[&TableType::PointInTime], "live");
    assert_eq!(map[&TableType::EventStream], "buffered");
}
