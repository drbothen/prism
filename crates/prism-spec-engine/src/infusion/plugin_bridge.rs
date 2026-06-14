//! Plugin infusion source — delegates to the WASM runtime (S-1.15).
//!
//! `PluginInfusionSource` implements `InfusionSource` by calling
//! `PluginRuntime::enrich_single` on the named `.prx` plugin.
//!
//! `plugin_id` and `config` are captured at construction time (in `InfusionLoader::load_all`)
//! because the `InfusionSource` trait signature only receives `input` and `input_type`.
//!
//! # Architecture Compliance
//! - PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003 / INV-INFUSE-003).
//! - Uses the UNTYPED `component::Val` path via `PluginRuntime::enrich_single` — NOT `TypedFunc`.
//! - `post_return` is NOT called — removed in wasmtime >=44 (plugin/mod.rs ~L970).
//! - `PluginError` is mapped to `InfusionError` at this boundary.
//!
//! # Stub status (S-DEMO-ENRICHMENT-PIVOT-001)
//! `enrich_single` and `enrich_batch` are `todo!()` stubs — implementation in this story.
//! Struct fields `plugin_id` and `config` are NET-NEW vs the S-1.14 partial-merge version.

use std::sync::Arc;

use prism_core::InfusionError;

use super::InfusionSource;
use crate::plugin::{PluginConfigMap, PluginRuntime};

/// Plugin-backed infusion source.
///
/// Delegates enrichment calls to the WASM plugin runtime (S-1.15) via
/// `PluginRuntime::enrich_single`. `plugin_id` and `config` must be populated
/// at construction time from the parsed `.infusion.toml` spec.
///
/// This source type is PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003).
///
/// `#[non_exhaustive]`: forward-compat per CLAUDE.md §Conventions — new fields may be
/// added (e.g., timeout overrides, retry policy) without a breaking semver change.
/// External callers must use `PluginInfusionSource::new()`.
#[non_exhaustive]
pub struct PluginInfusionSource {
    /// The plugin_id as registered in `PluginRuntime` (e.g., `"threat_intel"`).
    ///
    /// Passed as the first argument to `PluginRuntime::enrich_single(plugin_id, ...)`.
    pub plugin_id: String,

    /// The plugin configuration map (credentials, endpoint URLs) for this infusion.
    ///
    /// Populated from `[[infusion.credentials]]` and source config in `load_all`.
    /// Passed as `config` to `PluginRuntime::enrich_single`.
    /// Values are `SecretString` — credentials never transit AI context (AD-017).
    pub config: Arc<PluginConfigMap>,

    /// Reference to the shared `PluginRuntime` for WASM dispatch.
    ///
    /// `Arc` so the source can be cloned into `InfusionUdfDescriptor::source` without
    /// copying the runtime engine.
    pub runtime: Arc<PluginRuntime>,
}

impl std::fmt::Debug for PluginInfusionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginInfusionSource")
            .field("plugin_id", &self.plugin_id)
            .field("config", &format!("<{} keys>", self.config.len()))
            .field("runtime", &"<PluginRuntime>")
            .finish()
    }
}

impl PluginInfusionSource {
    /// Construct a `PluginInfusionSource`.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction; use this constructor.
    pub fn new(
        plugin_id: impl Into<String>,
        config: Arc<PluginConfigMap>,
        runtime: Arc<PluginRuntime>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            config,
            runtime,
        }
    }
}

impl InfusionSource for PluginInfusionSource {
    /// Enrich a single input value via the WASM plugin runtime.
    ///
    /// Delegates to `PluginRuntime::enrich_single(plugin_id, input, input_type, config)`
    /// using the UNTYPED `component::Val` path — NOT `TypedFunc`, NOT `post_return`.
    ///
    /// Maps `PluginError → InfusionError` at this boundary.
    ///
    /// # S-DEMO-ENRICHMENT-PIVOT-001 Red Gate stub
    /// Body is `todo!()` — implementation in this story's TDD green phase.
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        todo!(
            "PluginInfusionSource::enrich_single — S-DEMO-ENRICHMENT-PIVOT-001 Red Gate: \
             implement by calling PluginRuntime::enrich_single and mapping PluginError → InfusionError"
        )
    }

    /// Enrich a batch of input values.
    ///
    /// Default implementation: calls `enrich_single` for each input.
    /// May be overridden for true batching when the plugin ABI supports it.
    ///
    /// # S-DEMO-ENRICHMENT-PIVOT-001 Red Gate stub
    /// Body is `todo!()` — implementation in this story's TDD green phase.
    fn enrich_batch(
        &self,
        _inputs: &[String],
        _input_type: &str,
    ) -> Vec<Option<serde_json::Value>> {
        todo!(
            "PluginInfusionSource::enrich_batch — S-DEMO-ENRICHMENT-PIVOT-001 Red Gate: \
             implement via enrich_single loop"
        )
    }
}

/// Map a `PluginError` to the nearest `InfusionError` variant.
///
/// Called at the `PluginInfusionSource::enrich_single` boundary so plugin failures
/// propagate through the infusion error surface without leaking WASM internals.
///
/// # Stub
/// Returns `InfusionError::MissingRequiredField` as a placeholder — will be replaced
/// with a proper `InfusionError::PluginCallFailed` variant when the error taxonomy
/// is extended for plugin-type infusion failures.
#[allow(dead_code)]
pub(crate) fn map_plugin_error_to_infusion_error(
    plugin_id: &str,
    err: prism_core::PluginError,
) -> InfusionError {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-001): add InfusionError::PluginCallFailed variant
    // to error-taxonomy.md + InfusionError enum when implementing the green phase.
    // Placeholder: use MissingRequiredField with a descriptive message for now.
    let _ = err; // Suppress unused warning until green phase wires the real mapping.
    InfusionError::MissingRequiredField {
        field: format!("plugin_call_failed({})", plugin_id),
        spec_path: plugin_id.to_string(),
    }
}
