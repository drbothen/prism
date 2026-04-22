//! Plugin infusion source — delegates to the WASM runtime (S-1.15).
//!
//! `PluginInfusionSource` implements `InfusionSource` by calling
//! `PluginRuntime::enrich_single` / `enrich_batch` on the named `.prx` file.
//!
//! If S-1.15 is not yet built, this stub panics with a clear error message.
//! Local lookup types (MMDB, CSV, JSON) work independently of the plugin runtime.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14/S-1.15.

use super::InfusionSource;

/// Plugin-backed infusion source.
///
/// Delegates enrichment calls to the WASM plugin runtime (S-1.15).
/// This source type is PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003).
#[derive(Debug)]
pub struct PluginInfusionSource {
    pub plugin_path: String,
    pub infusion_id: String,
}

impl InfusionSource for PluginInfusionSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        unimplemented!(
            "PluginInfusionSource::enrich_single — requires S-1.15 WASM plugin runtime. \
             Local lookup infusions (maxmind_mmdb, csv, json_lookup) work without S-1.15."
        )
    }

    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        unimplemented!(
            "PluginInfusionSource::enrich_batch — requires S-1.15 WASM plugin runtime. \
             Local lookup infusions (maxmind_mmdb, csv, json_lookup) work without S-1.15."
        )
    }
}
