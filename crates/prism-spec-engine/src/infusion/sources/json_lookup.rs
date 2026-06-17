//! JSON static reference data infusion source.
//!
//! Loads a JSON file as `serde_json::Map<String, Value>`.
//! Key is the lookup key; value object provides declared output fields.

use super::super::InfusionSource;

/// JSON static lookup infusion source.
///
/// `data` holds the loaded JSON map (key → object) under `arc_swap` for hot-reload support.
#[derive(Debug)]
pub struct JsonLookupSource {
    pub json_path: String,
    data: arc_swap::ArcSwap<serde_json::Map<String, serde_json::Value>>,
}

impl JsonLookupSource {
    /// Load a JSON lookup file and return a `JsonLookupSource`.
    ///
    /// Deserializes the file as `serde_json::Map<String, serde_json::Value>`.
    pub fn load(json_path: &str) -> Result<Self, prism_core::InfusionError> {
        let content = std::fs::read_to_string(json_path).map_err(|e| {
            prism_core::InfusionError::MissingRequiredField {
                field: format!("json_open_failed: {}", e),
                spec_path: json_path.to_string(),
            }
        })?;

        let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| prism_core::InfusionError::MissingRequiredField {
                field: format!("json_parse_failed: {}", e),
                spec_path: json_path.to_string(),
            })?;

        Ok(JsonLookupSource {
            json_path: json_path.to_string(),
            data: arc_swap::ArcSwap::new(std::sync::Arc::new(map)),
        })
    }
}

impl InfusionSource for JsonLookupSource {
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        let data = self.data.load();
        data.get(input).cloned()
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}
