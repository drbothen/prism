//! JSON static reference data infusion source.
//!
//! Loads a JSON file as `serde_json::Map<String, Value>`.
//! Key is the lookup key; value object provides declared output fields.
//!
//! Supports `refresh_interval_secs`.
//!
//! # Stub (S-1.14-REDO)
//! All non-trivial bodies are `todo!()`.

use super::super::InfusionSource;

/// JSON static lookup infusion source.
///
/// `_data` holds the loaded JSON map (key → object). Prefixed `_` in stub.
/// Implementer: rename to `data` and use `arc_swap::ArcSwap<serde_json::Map<String, serde_json::Value>>`
/// for hot-reload support.
#[derive(Debug)]
pub struct JsonLookupSource {
    pub json_path: String,
}

impl JsonLookupSource {
    /// Load a JSON lookup file and return a `JsonLookupSource`.
    ///
    /// Implementer: read the JSON file, deserialize as
    /// `serde_json::Map<String, serde_json::Value>`, store in arc_swap field.
    pub fn load(_json_path: &str) -> Result<Self, prism_core::InfusionError> {
        todo!(
            "S-1.14-REDO: implement JsonLookupSource::load (read JSON file, deserialize as \
             Map<String, Value>) — BC-2.19.001"
        )
    }
}

impl InfusionSource for JsonLookupSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        todo!(
            "S-1.14-REDO: implement JsonLookupSource::enrich_single (JSON map lookup by key) \
             — BC-2.19.001"
        )
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        todo!(
            "S-1.14-REDO: implement JsonLookupSource::enrich_batch (delegate to enrich_single) \
             — BC-2.19.001"
        )
    }
}
