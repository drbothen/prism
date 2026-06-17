//! CSV infusion source.
//!
//! Loads a CSV file with a designated `key_column` into a
//! `HashMap<String, HashMap<String, String>>`. Implements `InfusionSource`:
//! lookup by key, return declared `source_column` fields as `serde_json::Value`.
//!
//! Uses `csv = "1.4"` with `csv::ReaderBuilder::new().has_headers(true).from_path(path)`
//! and `csv::StringRecord` for row iteration.
//!
//! Supports `refresh_interval_secs` — reloads into a new HashMap, then arc-swaps.
//!
//! # Stub (S-1.14-REDO)
//! All non-trivial bodies are `todo!()`. The `csv` crate is imported but unused here —
//! the import exists to validate the dep resolves at compile time.

use std::collections::HashMap;

use super::super::InfusionSource;

// Validate that `csv` dep resolves at compile time (S-1.14-REDO dep addition).
// The real usage is `csv::ReaderBuilder::new().has_headers(true).from_path(path)`.
#[allow(unused_imports)]
use csv as csv_crate;

/// CSV file-backed infusion source.
///
/// `_data` uses `arc_swap::ArcSwap` for lock-free hot reload (AD-007).
/// Key: the `key_column` value; inner value: map of column name → cell value.
/// The `_data` field is prefixed with `_` in the stub — implementer removes the prefix.
#[derive(Debug)]
pub struct CsvSource {
    pub csv_path: String,
    pub key_column: String,
    /// Arc-swapped data for hot reload: key → row values map.
    /// Prefixed `_data` in stub. Implementer: rename to `data` when implementing.
    _data: arc_swap::ArcSwap<HashMap<String, HashMap<String, String>>>,
}

impl CsvSource {
    /// Load a CSV file and return a `CsvSource`.
    ///
    /// Implementer: use `csv::ReaderBuilder::new().has_headers(true).from_path(path)`
    /// with `csv::StringRecord` for row iteration.
    /// Key each row by the value in `key_column`. Store all columns in the inner map.
    pub fn load(_csv_path: &str, _key_column: &str) -> Result<Self, prism_core::InfusionError> {
        todo!(
            "S-1.14-REDO: implement CsvSource::load using csv 1.4 ReaderBuilder API \
             — BC-2.19.001 / AC-7"
        )
    }
}

impl InfusionSource for CsvSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        todo!(
            "S-1.14-REDO: implement CsvSource::enrich_single (HashMap lookup by key, \
             return serde_json::Value of matched columns) — BC-2.19.001 / AC-7"
        )
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        todo!(
            "S-1.14-REDO: implement CsvSource::enrich_batch (delegate to enrich_single) \
             — BC-2.19.001 / AC-7"
        )
    }
}
