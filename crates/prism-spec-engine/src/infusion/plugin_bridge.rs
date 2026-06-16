//! Plugin infusion source — delegates to the WASM runtime (S-1.15).
//!
//! `PluginInfusionSource` implements `InfusionSource` by calling
//! `PluginRuntime::enrich_single` on the named `.prx` plugin.
//!
//! `plugin_id` and `config` are captured at construction time (in
//! `InfusionRegistry::load_spec_with_runtime`) because the `InfusionSource` trait signature
//! only receives `input` and `input_type`.
//!
//! # Architecture Compliance
//! - PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003 / INV-INFUSE-003).
//! - Uses the UNTYPED `component::Val` path via `PluginRuntime::enrich_single` — NOT `TypedFunc`.
//! - `post_return` is NOT called — removed in wasmtime >=44 (plugin/mod.rs ~L970).
//! - `PluginError` is mapped to `InfusionError` at this boundary.
//!
//! # Implementation (S-DEMO-ENRICHMENT-PIVOT-001)
//! `enrich_single` delegates to `PluginRuntime::enrich_single` using the UNTYPED
//! `component::Val` path. Maps `PluginError → InfusionError`. Since the `InfusionSource`
//! trait returns `Option<Value>` (not `Result`), failures (including `NotLoaded`) are logged
//! at WARN level and returned as `None` — never panics.

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

    /// The plugin configuration map for this infusion.
    ///
    /// Constructed as an empty `PluginConfigMap` in `InfusionRegistry::load_spec_with_runtime`;
    /// credential values are resolved at call time from env vars per AD-017, not pre-populated
    /// here. Passed as `config` to `PluginRuntime::enrich_single`.
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
    /// Maps `PluginError → InfusionError` at this boundary. Since `InfusionSource::enrich_single`
    /// returns `Option<Value>` (not `Result`), plugin failures are logged at WARN level
    /// and returned as `None` (no enrichment available for this input).
    ///
    /// # NotLoaded handling
    /// If `PluginRuntime::enrich_single` returns `PluginError::NotLoaded` for the plugin_id,
    /// it means the plugin has not yet been loaded into the runtime (e.g., plugin binary missing at
    /// boot time, S-1.15 boot-wiring not yet reached, or plugin misnamed in the spec). This is a
    /// routine runtime error — the method logs at WARN level and returns `Ok(None)` (no enrichment
    /// available), exactly as all other `PluginError` variants do. A live query MUST NOT panic the
    /// query engine due to a plugin not being loaded at runtime.
    fn enrich_single(&self, input: &str, input_type: &str) -> Option<serde_json::Value> {
        match self
            .runtime
            .enrich_single(&self.plugin_id, input, input_type, &self.config)
        {
            Ok(result) => result,
            Err(prism_core::PluginError::NotLoaded { ref plugin_id }) => {
                // NotLoaded is a routine runtime error (plugin not in the loaded map —
                // misnamed/unbooted/failed-load). Log at WARN and return None so the query
                // engine continues without enrichment for this row.
                let infusion_err = map_plugin_error_to_infusion_error(
                    &self.plugin_id,
                    prism_core::PluginError::NotLoaded {
                        plugin_id: plugin_id.clone(),
                    },
                );
                tracing::warn!(
                    plugin_id = %self.plugin_id,
                    input_type = %input_type,
                    error = %infusion_err,
                    "plugin not loaded in PluginRuntime — returning None for input (boot-wiring required)"
                );
                None
            }
            Err(plugin_err) => {
                let infusion_err = map_plugin_error_to_infusion_error(&self.plugin_id, plugin_err);
                tracing::warn!(
                    plugin_id = %self.plugin_id,
                    input_type = %input_type,
                    error = %infusion_err,
                    "plugin enrichment call failed — returning None for input"
                );
                None
            }
        }
    }

    /// Enrich a batch of input values.
    ///
    /// Delegates to `enrich_single` for each input.
    /// May be overridden for true batching when the plugin ABI supports it.
    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|input| self.enrich_single(input, input_type))
            .collect()
    }
}

/// Map a `PluginError` to the nearest `InfusionError` variant.
///
/// Called at the `PluginInfusionSource::enrich_single` boundary so plugin failures
/// propagate through the infusion error surface without leaking WASM internals.
///
/// Current mapping: `PluginError` → `InfusionError::MissingRequiredField` with a
/// descriptive message capturing the plugin failure reason.
///
/// TODO(S-1.14-REDO): add `InfusionError::PluginCallFailed` variant to the error
/// taxonomy + InfusionError enum for a proper first-class error code (E-INFUSE-008).
/// Note: E-INFUSE-006 is already assigned ("Infusion not found"); E-INFUSE-008 is the
/// next-free code for PluginCallFailed. The taxonomy row will be allocated when that
/// variant is actually built in S-1.14-REDO.
pub(crate) fn map_plugin_error_to_infusion_error(
    plugin_id: &str,
    err: prism_core::PluginError,
) -> InfusionError {
    // Using MissingRequiredField as a stand-in until E-INFUSE-008 PluginCallFailed
    // is added to the error taxonomy in S-1.14-REDO.
    InfusionError::MissingRequiredField {
        field: format!("plugin_call_failed({}): {}", plugin_id, err),
        spec_path: plugin_id.to_string(),
    }
}
