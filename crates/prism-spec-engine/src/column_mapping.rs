//! Column-to-OCSF field mapping at query time (BC-2.16.003).
//!
//! After the pipeline returns raw records, columns with `ocsf_field` mappings
//! are translated to the corresponding OCSF protobuf field using the standard
//! four-tier field resolution (BC-2.02.008). Columns without mappings go to
//! `raw_extensions`. Type coercion is applied with non-fatal fallback.

use prism_core::PrismError;
use serde_json::Value;

use crate::spec_parser::{ColumnSpec, TableSpec};

/// Result of mapping a single raw record to OCSF fields.
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Fields successfully mapped to OCSF protobuf paths.
    pub mapped_fields: std::collections::HashMap<String, Value>,
    /// Fields placed in raw_extensions (no ocsf_field, or coercion failure).
    pub raw_extensions: std::collections::HashMap<String, Value>,
    /// Coercion warnings for this record.
    pub coercion_warnings: Vec<CoercionWarning>,
}

/// A non-fatal coercion warning: field placed in raw_extensions due to type mismatch.
#[derive(Debug, Clone)]
pub struct CoercionWarning {
    pub column_name: String,
    pub expected_ocsf_type: String,
    pub actual_value: String,
}

/// Maps raw records from a pipeline execution to OCSF fields (BC-2.16.003).
///
/// All methods are `unimplemented!()` — implemented in S-1.11.
pub struct ColumnMapper;

/// A single column mapping specification.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMapping {
    /// Source column name from the raw response.
    pub source_column: String,
    /// OCSF field path target (e.g., `"device.ip"`).
    pub ocsf_field_path: String,
}

impl ColumnMapper {
    /// Map a raw JSON record to `MappingResult` using the table's ColumnSpec entries.
    ///
    /// - Columns with `ocsf_field` -> OCSF protobuf field (four-tier resolution BC-2.02.008)
    /// - Columns without `ocsf_field` -> raw_extensions
    /// - Type coercion failures -> raw_extensions + CoercionWarning (non-fatal)
    /// - ocsf_class from TableSpec determines the OCSF event class
    ///
    /// Records are NEVER dropped due to mapping or coercion failures (invariant BC-2.16.003).
    pub fn map_record(_raw: &Value, _table: &TableSpec) -> Result<MappingResult, PrismError> {
        unimplemented!("ColumnMapper::map_record — implement in S-1.11 (BC-2.16.003)")
    }

    /// Apply type coercion for a single column value.
    ///
    /// Returns `Ok(coerced_value)` on success, `Err(CoercionWarning)` on failure
    /// (caller places failed value in raw_extensions and continues — never drops record).
    pub fn coerce_value(
        _value: &Value,
        _column: &ColumnSpec,
        _ocsf_field_path: &str,
    ) -> Result<Value, CoercionWarning> {
        unimplemented!("ColumnMapper::coerce_value — implement in S-1.11 (BC-2.16.003)")
    }
}
