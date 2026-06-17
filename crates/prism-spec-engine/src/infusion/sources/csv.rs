//! CSV infusion source.
//!
//! Loads a CSV file with a designated `key_column` into a
//! `HashMap<String, HashMap<String, String>>`. Implements `InfusionSource`:
//! lookup by key, return all columns as `serde_json::Value`.
//!
//! Uses `csv = "1.4"` with `csv::ReaderBuilder::new().has_headers(true).from_path(path)`
//! and `csv::StringRecord` for row iteration.

use std::collections::HashMap;

use super::super::InfusionSource;

/// CSV file-backed infusion source.
///
/// Data is loaded at construction time into a `HashMap<key, row-columns-map>`.
/// `arc_swap::ArcSwap` holds the data for lock-free hot-reload support.
///
/// `#[non_exhaustive]`: forward-compat for infusion engine evolution — fields may expand
/// (e.g., reload policy, column subset filter) without a breaking semver change.
/// External callers must use `CsvSource::load()` for construction.
#[non_exhaustive]
#[derive(Debug)]
pub struct CsvSource {
    pub csv_path: String,
    pub key_column: String,
    /// Arc-swapped data for hot reload: key → row values map.
    data: arc_swap::ArcSwap<HashMap<String, HashMap<String, String>>>,
}

impl CsvSource {
    /// Load a CSV file and return a `CsvSource`.
    ///
    /// Uses `csv::ReaderBuilder::new().has_headers(true).from_path(path)`.
    /// Keys each row by the value in `key_column`. Stores all columns in the inner map.
    pub fn load(csv_path: &str, key_column: &str) -> Result<Self, prism_core::InfusionError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(csv_path)
            .map_err(|e| prism_core::InfusionError::MissingRequiredField {
                field: format!("csv_open_failed: {}", e),
                spec_path: csv_path.to_string(),
            })?;

        let headers = reader
            .headers()
            .map_err(|e| prism_core::InfusionError::MissingRequiredField {
                field: format!("csv_headers_failed: {}", e),
                spec_path: csv_path.to_string(),
            })?
            .clone();

        // Find key_column index.
        let key_col_idx = headers
            .iter()
            .position(|h| h == key_column)
            .ok_or_else(|| prism_core::InfusionError::MissingRequiredField {
                field: format!(
                    "csv_key_column_not_found: column '{}' not in headers",
                    key_column
                ),
                spec_path: csv_path.to_string(),
            })?;

        let mut data: HashMap<String, HashMap<String, String>> = HashMap::new();

        for result in reader.records() {
            let record = result.map_err(|e| prism_core::InfusionError::MissingRequiredField {
                field: format!("csv_record_error: {}", e),
                spec_path: csv_path.to_string(),
            })?;

            if let Some(key_val) = record.get(key_col_idx) {
                let mut row: HashMap<String, String> = HashMap::new();
                for (i, header) in headers.iter().enumerate() {
                    if let Some(cell) = record.get(i) {
                        row.insert(header.to_string(), cell.to_string());
                    }
                }
                data.insert(key_val.to_string(), row);
            }
        }

        Ok(CsvSource {
            csv_path: csv_path.to_string(),
            key_column: key_column.to_string(),
            data: arc_swap::ArcSwap::new(std::sync::Arc::new(data)),
        })
    }
}

impl InfusionSource for CsvSource {
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        let data = self.data.load();
        let row = data.get(input)?;
        // Return the entire row as a JSON object.
        let obj: serde_json::Map<String, serde_json::Value> = row
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        Some(serde_json::Value::Object(obj))
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}
