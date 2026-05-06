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

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
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
    batch: RecordBatch,
    sensor: &SensorType,
    client_id: &OrgSlug,
    source_table: &str,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let num_rows = batch.num_rows();

    // Step 1: Remove any spoofed virtual columns.
    let batch = remove_spoofed_virtual_columns(batch)?;

    // Step 2: Build the three virtual field arrays.
    let sensor_val = sensor_type_to_string(sensor);
    let client_val = client_id.as_str();

    let sensor_array = Arc::new(StringArray::from(vec![sensor_val; num_rows])) as _;
    let client_array = Arc::new(StringArray::from(vec![client_val; num_rows])) as _;
    let table_array = Arc::new(StringArray::from(vec![source_table; num_rows])) as _;

    // Step 3: Build new schema by appending the three virtual fields.
    let existing_schema = batch.schema();
    let mut new_fields: Vec<Field> = existing_schema
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    new_fields.push(Field::new(VIRTUAL_FIELD_SENSOR, DataType::Utf8, false));
    new_fields.push(Field::new(VIRTUAL_FIELD_CLIENT, DataType::Utf8, false));
    new_fields.push(Field::new(
        VIRTUAL_FIELD_SOURCE_TABLE,
        DataType::Utf8,
        false,
    ));

    let new_schema = Arc::new(Schema::new(new_fields));

    // Step 4: Build new column list.
    let mut new_columns: Vec<_> = (0..batch.num_columns())
        .map(|i| batch.column(i).clone())
        .collect();
    new_columns.push(sensor_array);
    new_columns.push(client_array);
    new_columns.push(table_array);

    RecordBatch::try_new(new_schema, new_columns)
}

// ---------------------------------------------------------------------------
// remove_spoofed_virtual_columns
// ---------------------------------------------------------------------------

/// Remove any existing columns with virtual field names from a RecordBatch.
///
/// Called before injecting canonical values to ensure sensor-emitted columns
/// with reserved names are completely replaced. (BC-2.11.012 spoofing prevention)
pub(crate) fn remove_spoofed_virtual_columns(
    batch: RecordBatch,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let reserved: &[&str] = &[
        VIRTUAL_FIELD_SENSOR,
        VIRTUAL_FIELD_CLIENT,
        VIRTUAL_FIELD_SOURCE_TABLE,
    ];

    // Build list of column indices to keep.
    let keep_indices: Vec<usize> = batch
        .schema()
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| !reserved.contains(&f.name().as_str()))
        .map(|(i, _)| i)
        .collect();

    if keep_indices.len() == batch.num_columns() {
        // Nothing to remove.
        return Ok(batch);
    }

    let new_columns: Vec<_> = keep_indices
        .iter()
        .map(|&i| batch.column(i).clone())
        .collect();
    let new_fields: Vec<Field> = keep_indices
        .iter()
        .map(|&i| batch.schema().field(i).clone())
        .collect();
    let new_schema = Arc::new(Schema::new(new_fields));
    RecordBatch::try_new(new_schema, new_columns)
}

// ---------------------------------------------------------------------------
// sensor_type_to_string
// ---------------------------------------------------------------------------

/// Convert a `SensorType` enum to the canonical virtual field string value.
///
/// Returns the lowercase sensor name used in `_sensor` column values.
/// (BC-2.11.012 `_sensor` field values)
pub(crate) fn sensor_type_to_string(sensor: &SensorType) -> &'static str {
    match sensor {
        SensorType::CrowdStrike => "crowdstrike",
        SensorType::Cyberint => "cyberint",
        SensorType::Claroty => "claroty",
        SensorType::Armis => "armis",
    }
}
