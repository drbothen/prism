//! JSON static reference data infusion source.
//!
//! Loads a JSON file as `serde_json::Map<String, Value>`.
//! Key is the lookup key; value object provides declared output fields.
//!
//! Supports `refresh_interval_secs`.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use super::super::InfusionSource;

/// JSON static lookup infusion source.
#[derive(Debug)]
pub struct JsonLookupSource {
    pub json_path: String,
}

impl JsonLookupSource {
    /// Load a JSON lookup file and return a `JsonLookupSource`.
    pub fn load(_json_path: &str) -> Result<Self, prism_core::InfusionError> {
        unimplemented!("JsonLookupSource::load — implement in S-1.14 (BC-2.19.001)")
    }
}

impl InfusionSource for JsonLookupSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        unimplemented!("JsonLookupSource::enrich_single — implement in S-1.14 (BC-2.19.001)")
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        unimplemented!("JsonLookupSource::enrich_batch — implement in S-1.14 (BC-2.19.001)")
    }
}
