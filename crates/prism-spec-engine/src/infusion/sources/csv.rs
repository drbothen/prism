//! CSV infusion source.
//!
//! Loads a CSV file with a designated `key_column` into a
//! `HashMap<String, HashMap<String, String>>`. Implements `InfusionSource`:
//! lookup by key, return declared `csv_column` fields.
//!
//! Supports `refresh_interval_secs` — reloads into a new HashMap, then arc-swaps.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use std::collections::HashMap;

use super::super::InfusionSource;

/// CSV file-backed infusion source.
#[derive(Debug)]
pub struct CsvSource {
    pub csv_path: String,
    pub key_column: String,
    /// Arc-swapped data for hot reload: key → row values map.
    _data: arc_swap::ArcSwap<HashMap<String, HashMap<String, String>>>,
}

impl CsvSource {
    /// Load a CSV file and return a `CsvSource`.
    pub fn load(csv_path: &str, key_column: &str) -> Result<Self, prism_core::InfusionError> {
        unimplemented!(
            "CsvSource::load — implement in S-1.14 (BC-2.19.001 / AC-7)"
        )
    }
}

impl InfusionSource for CsvSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        unimplemented!(
            "CsvSource::enrich_single — implement in S-1.14 (BC-2.19.001 / AC-7)"
        )
    }

    fn enrich_batch(&self, _inputs: &[String], _input_type: &str) -> Vec<Option<serde_json::Value>> {
        unimplemented!(
            "CsvSource::enrich_batch — implement in S-1.14 (BC-2.19.001 / AC-7)"
        )
    }
}
