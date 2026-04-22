//! Column-to-OCSF field mapping at query time (BC-2.16.003).
//!
//! After the pipeline returns raw records, columns with `ocsf_field` mappings
//! are translated to the corresponding OCSF protobuf field using the standard
//! four-tier field resolution (BC-2.02.008). Columns without mappings go to
//! `raw_extensions`. Type coercion is applied with non-fatal fallback.

use prism_core::{ColumnType, PrismError};
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
    /// - Columns with `ocsf_field` -> OCSF protobuf field
    /// - Columns without `ocsf_field` -> raw_extensions
    /// - Type coercion failures -> raw_extensions + CoercionWarning (non-fatal)
    /// - Records are NEVER dropped (invariant BC-2.16.003)
    pub fn map_record(
        raw: &Value,
        table: &TableSpec,
    ) -> Result<MappingResult, PrismError> {
        let mut mapped_fields = std::collections::HashMap::new();
        let mut raw_extensions = std::collections::HashMap::new();
        let mut coercion_warnings = Vec::new();

        for col in &table.columns {
            // Extract the raw value for this column from the record
            let raw_value = match raw.get(&col.name) {
                Some(v) => v.clone(),
                None => {
                    // Column not present in record — skip (no error, no raw_extension)
                    continue;
                }
            };

            match &col.ocsf_field {
                Some(ocsf_path) => {
                    // Attempt type coercion for the OCSF field
                    match Self::coerce_value(&raw_value, col, ocsf_path) {
                        Ok(coerced) => {
                            mapped_fields.insert(ocsf_path.clone(), coerced);
                        }
                        Err(warning) => {
                            // Coercion failed: put in raw_extensions, record coercion warning
                            raw_extensions.insert(col.name.clone(), raw_value);
                            coercion_warnings.push(warning);
                        }
                    }
                }
                None => {
                    // No ocsf_field mapping -> raw_extensions
                    raw_extensions.insert(col.name.clone(), raw_value);
                }
            }
        }

        Ok(MappingResult {
            mapped_fields,
            raw_extensions,
            coercion_warnings,
        })
    }

    /// Apply type coercion for a single column value.
    ///
    /// Returns `Ok(coerced_value)` on success, `Err(CoercionWarning)` on failure.
    /// The caller places failed values in raw_extensions and continues (never drops record).
    pub fn coerce_value(
        value: &Value,
        column: &ColumnSpec,
        ocsf_field_path: &str,
    ) -> Result<Value, CoercionWarning> {
        // Determine target type from OCSF field path convention.
        // For numeric OCSF fields (those ending in standard numeric suffixes),
        // attempt string-to-number coercion.
        let target_is_numeric = is_numeric_ocsf_field(ocsf_field_path);

        if target_is_numeric {
            // Try to coerce string to integer
            if let Value::String(s) = value {
                if let Ok(n) = s.parse::<i64>() {
                    return Ok(Value::Number(serde_json::Number::from(n)));
                }
                // Failed coercion — produce warning
                return Err(CoercionWarning {
                    column_name: column.name.clone(),
                    expected_ocsf_type: "integer".to_string(),
                    actual_value: s.clone(),
                });
            }
        }

        // Default: return value as-is (no coercion needed)
        Ok(value.clone())
    }
}

/// Heuristic: OCSF fields whose last path segment is a known numeric field name
/// are treated as integer targets for coercion purposes.
/// This is a simplified model — the full implementation uses the embedded OCSF schema.
fn is_numeric_ocsf_field(path: &str) -> bool {
    let numeric_suffixes = [
        "event_code",
        "class_uid",
        "activity_id",
        "type_uid",
        "severity_id",
        "status_id",
        "action_id",
        "count",
        "duration",
        "port",
        "pid",
        "uid",
        "code",
    ];
    let last_segment = path.split('.').last().unwrap_or(path);
    numeric_suffixes.iter().any(|s| *s == last_segment)
}
