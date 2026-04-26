//! `inject_source_type` — pure-data `_source_type` virtual field injection.
//!
//! Sets `"_source_type"` on each row map in a `Vec<serde_json::Value>` slice.
//! Rows from an `EventStream` buffer receive `"buffered"`; rows from a live API
//! fetch (including cold-start fallback) receive `"live"`.
//!
//! Operates on `serde_json::Value` row maps only — **no DataFusion, no Arrow**.
//! S-3.02 wires this function into the DataFusion `TableProvider` integration
//! via the same virtual field injection path as `_sensor`, `_client`, and
//! `_source_table`.
//!
//! Story: S-2.08 | AC-9, AC-10

use crate::types::SensorQueryDescriptor;

// ---------------------------------------------------------------------------
// inject_source_type
// ---------------------------------------------------------------------------

/// Injects `"_source_type"` virtual field into every row in `rows`.
///
/// - When `descriptor.table_type == EventStream` **and** `descriptor.rows_from_buffer`:
///   sets `"_source_type": "buffered"` on every row (AC-9).
/// - Otherwise (PointInTime table, or EventStream cold-start live fallback):
///   sets `"_source_type": "live"` on every row (AC-10).
///
/// Operates on `serde_json::Value` row maps only — no DataFusion, no Arrow.
/// Non-object values in the slice are skipped without error.
///
/// S-3.02 will call this function from the DataFusion `TableProvider` integration
/// using the same virtual field injection path as `_sensor`, `_client`, and
/// `_source_table` (S-2.08 Architecture Compliance Rule 5).
///
/// # AC-9
/// Given `EventStream` rows from the buffer: every row has `"_source_type": "buffered"`.
///
/// # AC-10
/// Given `PointInTime` rows or cold-start fallback live rows:
/// every row has `"_source_type": "live"`.
// S-2.08 spec mandates &mut Vec<serde_json::Value> signature for S-3.02 wiring;
// clippy::ptr_arg is suppressed intentionally.
#[allow(clippy::ptr_arg)]
pub fn inject_source_type(rows: &mut Vec<serde_json::Value>, descriptor: &SensorQueryDescriptor) {
    use prism_core::TableType;

    let source_type =
        if descriptor.table_type == TableType::EventStream && descriptor.rows_from_buffer {
            "buffered"
        } else {
            "live"
        };

    for row in rows.iter_mut() {
        if let Some(obj) = row.as_object_mut() {
            obj.insert(
                "_source_type".to_string(),
                serde_json::Value::String(source_type.to_string()),
            );
        }
    }
}
