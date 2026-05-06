//! `virtual_fields` — virtual field injection into Arrow RecordBatches.
//!
//! Injects three Prism-specific provenance columns into every materialized
//! RecordBatch. These columns are not part of the OCSF schema — they are
//! Prism metadata distinguishable by their underscore prefix.
//!
//! The engine MUST overwrite any sensor-emitted columns with these names to
//! prevent spoofing. (BC-2.11.012, EC-005)
//!
//! # Virtual Fields
//! - `_sensor`       — sensor type string (e.g. `"crowdstrike"`)
//! - `_client`       — OrgSlug for the client that owns the sensor instance
//! - `_source_table` — source table name (e.g. `"crowdstrike.detections"`)
//!
//! # BC References
//! - BC-2.11.012 — Virtual Fields in Queries
//! - BC-2.11.005 — injected during materialization pipeline Step 5
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
#![allow(dead_code)]

use arrow::record_batch::RecordBatch;
use prism_core::{types::SensorType, OrgSlug};

// ---------------------------------------------------------------------------
// VIRTUAL_FIELD_NAMES
// ---------------------------------------------------------------------------

/// Reserved virtual field column names. These names cannot be used by sensors.
///
/// Validated at build time against the OCSF proto schema to prevent
/// collisions. (BC-2.11.012 invariant)
pub const VIRTUAL_FIELD_SENSOR: &str = "_sensor";
pub const VIRTUAL_FIELD_CLIENT: &str = "_client";
pub const VIRTUAL_FIELD_SOURCE_TABLE: &str = "_source_table";

// ---------------------------------------------------------------------------
// inject_virtual_fields
// ---------------------------------------------------------------------------

/// Inject `_sensor`, `_client`, and `_source_table` columns into a RecordBatch.
///
/// If the batch already contains any of these column names (sensor spoofing
/// attempt), the existing column is overwritten with the canonical value.
/// (BC-2.11.012, EC-005)
///
/// All three virtual fields are `Utf8` (string) typed Arrow columns.
/// Numeric comparisons on virtual fields are type errors at the query layer.
/// (BC-2.11.012)
///
/// # BC-2.11.012
/// Virtual fields are available in all PrismQL modes:
/// - Filter: `_sensor = "crowdstrike" AND severity >= "high"`
/// - SQL: `SELECT _sensor, count(*) FROM events GROUP BY _sensor`
/// - Pipe: `| where _sensor = "claroty" | stats count by _client`
pub fn inject_virtual_fields(
    _batch: RecordBatch,
    _sensor: &SensorType,
    _client_id: &OrgSlug,
    _source_table: &str,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    todo!("S-3.02 — inject_virtual_fields")
}

// ---------------------------------------------------------------------------
// remove_spoofed_virtual_columns
// ---------------------------------------------------------------------------

/// Remove any existing columns with virtual field names from a RecordBatch.
///
/// Called before injecting canonical values to ensure sensor-emitted columns
/// with reserved names are completely replaced. (BC-2.11.012 spoofing prevention)
pub(crate) fn remove_spoofed_virtual_columns(
    _batch: RecordBatch,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    todo!("S-3.02 — remove_spoofed_virtual_columns")
}

// ---------------------------------------------------------------------------
// sensor_type_to_string
// ---------------------------------------------------------------------------

/// Convert a `SensorType` enum to the canonical virtual field string value.
///
/// Returns the lowercase sensor name used in `_sensor` column values.
/// (BC-2.11.012 `_sensor` field values)
///
/// # GREEN-BY-DESIGN candidate (self-check per BC-5.38.005)
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?"
/// Answer: Yes — the match is trivial pattern dispatch that would make tests
/// pass without implementer effort. Therefore: `todo!()`.
pub(crate) fn sensor_type_to_string(_sensor: &SensorType) -> &'static str {
    todo!("S-3.02 — sensor_type_to_string")
}
