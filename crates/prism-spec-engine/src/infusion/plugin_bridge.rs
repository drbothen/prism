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
//! # Implementation (S-DEMO-ENRICHMENT-PIVOT-001)
//! `enrich_single` delegates to `PluginRuntime::enrich_single` using the UNTYPED
//! `component::Val` path. Maps `PluginError → InfusionError`. Since the `InfusionSource`
//! trait returns `Option<Value>` (not `Result`), failures are logged and returned as `None`.

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
    /// Maps `PluginError → InfusionError` at this boundary. Since `InfusionSource::enrich_single`
    /// returns `Option<Value>` (not `Result`), plugin failures are logged at WARN level
    /// and returned as `None` (no enrichment available for this input).
    ///
    /// # S-1.15 exemption
    /// If `PluginRuntime::enrich_single` returns `PluginError::NotLoaded` for the plugin_id,
    /// it means the plugin has not been loaded into the runtime (S-1.15 not yet operational
    /// for this plugin, or the plugin binary is missing at boot time). In this case the method
    /// panics with `todo!("S-1.15: ...")` per the S-DEMO-ENRICHMENT-PIVOT-001 risk mitigation —
    /// see story §risk_mitigations: "if S-1.15 PluginRuntime is not operational at dispatch time,
    /// implement as Err(InfusionError::PluginRuntimeNotAvailable) with annotated todo!(S-1.15)".
    /// This is a compile-time signal, not a production crash path: when S-1.15 is operational,
    /// the plugin will be loaded at boot and this branch will not be reached.
    fn enrich_single(&self, input: &str, input_type: &str) -> Option<serde_json::Value> {
        match self
            .runtime
            .enrich_single(&self.plugin_id, input, input_type, &self.config)
        {
            Ok(result) => result,
            Err(prism_core::PluginError::NotLoaded { ref plugin_id }) => {
                // S-1.15 exemption: plugin not loaded means S-1.15 WASM runtime is not
                // yet operational for this plugin. Signal as a todo!() per story risk mitigation.
                todo!(
                    "S-1.15: plugin '{}' not loaded in PluginRuntime — S-DEMO-ENRICHMENT-PIVOT-001 \
                     enrich_single requires S-1.15 plugin boot wiring. \
                     Load the plugin via PluginRuntime::load_plugin before calling enrich_single.",
                    plugin_id
                )
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
/// taxonomy + InfusionError enum for a proper first-class error code (E-INFUSE-006).
#[allow(dead_code)]
pub(crate) fn map_plugin_error_to_infusion_error(
    plugin_id: &str,
    err: prism_core::PluginError,
) -> InfusionError {
    // Using MissingRequiredField as a stand-in until E-INFUSE-006 PluginCallFailed
    // is added to the error taxonomy in S-1.14-REDO.
    InfusionError::MissingRequiredField {
        field: format!("plugin_call_failed({}): {}", plugin_id, err),
        spec_path: plugin_id.to_string(),
    }
}
