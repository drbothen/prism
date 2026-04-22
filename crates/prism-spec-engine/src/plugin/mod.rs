//! WASM Plugin Runtime — `prism-spec-engine` SS-17.
//!
//! Implements the WASM Component Model plugin runtime per AD-019. Loads `.prx` files
//! using `wasmtime` with component model support, enforces sandbox constraints
//! (memory limits, CPU epoch interruption, no WASI), implements hot reload via `notify`,
//! and isolates plugin panics from the host process.
//!
//! # Invariants (Red Gate stubs — all `unimplemented!()`)
//! - INV-PLUGIN-001: Plugin panic/trap MUST NOT terminate the host process (BC-2.17.001)
//! - INV-PLUGIN-002: No direct filesystem/network access from plugins (BC-2.17.002)
//! - INV-PLUGIN-003: 64MB memory limit per plugin instance (BC-2.17.003)
//! - INV-PLUGIN-004: 5s CPU time limit via epoch interruption (BC-2.17.004)
//! - INV-PLUGIN-005: Atomic module swap on hot reload; failed reload retains old (BC-2.17.005)
//! - INV-PLUGIN-006: WIT interface validation before registration (BC-2.17.006)

pub mod discovery;
pub mod host_functions;
pub mod hot_reload;
pub mod loader;
pub mod sandbox;

use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use prism_core::PluginError;
use serde_json::Value;

// Re-export public types used by callers (S-1.14, S-4.08).
pub use loader::{HostState, LoadedPlugin, PluginConfigMap, PluginKvStore};

/// The three Prism plugin types recognised by WIT validation.
///
/// Each type corresponds to a `.wit` interface file and a distinct set of required exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginType {
    /// `prism:sensor-plugin` — implements `fetch-page`, `name`, `version`.
    Sensor,
    /// `prism:infusion-plugin` — implements `enrich-single`, `enrich-batch`, `name`, `version`.
    Infusion,
    /// `prism:action-plugin` — implements `fire-alert`, `fire-case`, `fire-report`, `name`, `version`.
    Action,
}

/// Context for action plugin `fire-alert` calls.
#[derive(Debug, Clone)]
pub struct AlertContext {
    pub alert_id: String,
    pub severity: String,
    pub title: String,
    pub raw_json: Value,
}

/// Context for action plugin `fire-case` calls.
#[derive(Debug, Clone)]
pub struct CaseContext {
    pub case_id: String,
    pub title: String,
    pub raw_json: Value,
}

/// Context for action plugin `fire-report` calls.
#[derive(Debug, Clone)]
pub struct ReportContext {
    pub report_id: String,
    pub title: String,
    pub raw_json: Value,
}

/// Result returned by action plugin dispatch methods.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
    pub raw_response: Option<Value>,
}

/// The WASM plugin runtime.
///
/// Holds the wasmtime `Engine`, the component `Linker`, and the plugin registry.
/// The registry maps `plugin_id -> Arc<LoadedPlugin>` behind an `ArcSwap` for
/// lock-free hot reload.
///
/// Construction (`PluginRuntime::new`) starts the epoch ticker background task.
/// All plugin calls create a fresh `Store` per invocation and wrap `instance.call_*`
/// in a trap-catching boundary.
pub struct PluginRuntime {
    pub engine: wasmtime::Engine,
    pub linker: wasmtime::component::Linker<HostState>,
    /// `plugin_id -> Arc<LoadedPlugin>`, swapped atomically on hot reload.
    pub registry: ArcSwap<HashMap<String, Arc<LoadedPlugin>>>,
}

impl PluginRuntime {
    /// Create a new `PluginRuntime`.
    ///
    /// - Initialises `wasmtime::Engine` with component model + epoch interruption.
    /// - Builds the `Linker` with only Prism host interface bindings (no WASI).
    /// - Starts the background epoch ticker task.
    ///
    /// # Errors
    /// Returns `PrismError` if engine configuration fails.
    pub fn new() -> Result<Self, prism_core::PrismError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::new not yet implemented")
    }

    /// Build the `wasmtime::component::Linker<HostState>` for this runtime.
    ///
    /// Only Prism host interface functions are linked — NO WASI imports.
    /// This is the target of VP-040 (no `wasi:` namespaces in the built linker).
    pub fn build_linker(
        engine: &wasmtime::Engine,
    ) -> Result<wasmtime::component::Linker<HostState>, prism_core::PrismError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::build_linker not yet implemented")
    }

    /// Load and validate a `.prx` plugin binary from `path`.
    ///
    /// 1. Reads the bytes from disk.
    /// 2. Compiles via `wasmtime::component::Component::from_binary`.
    /// 3. Pre-instantiates via `Linker::instantiate_pre`.
    /// 4. Validates WIT interface (calls `discovery::validate_wit_interface`).
    /// 5. Adds to registry on success.
    ///
    /// Returns `Err(PluginError::InvalidInterface)` if WIT validation fails (`E-PLUGIN-001`).
    /// Returns `Err(PluginError::CompilationFailed)` if compilation fails (`E-PLUGIN-008`).
    pub fn load_plugin(
        &self,
        path: &std::path::Path,
    ) -> Result<Arc<LoadedPlugin>, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::load_plugin not yet implemented")
    }

    /// Return an `Arc<LoadedPlugin>` for `plugin_id`, or `Err(NotLoaded)`.
    pub fn get_plugin(&self, plugin_id: &str) -> Result<Arc<LoadedPlugin>, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::get_plugin not yet implemented")
    }

    /// List all registered plugin_ids.
    pub fn list_plugins(&self) -> Vec<String> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::list_plugins not yet implemented")
    }

    // ---- Public dispatch API (used by S-1.14 infusion bridge and S-4.08 action engine) ----

    /// Call `enrich_single` on the named infusion plugin.
    ///
    /// Creates a fresh `Store`, sets epoch deadline, instantiates from `InstancePre`,
    /// calls the `enrich-single` WIT export, and returns the result or a `PluginError`.
    pub fn enrich_single(
        &self,
        plugin_id: &str,
        input_value: &str,
        input_type: &str,
        config: &PluginConfigMap,
    ) -> Result<Option<Value>, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::enrich_single not yet implemented")
    }

    /// Call `enrich_batch` on the named infusion plugin.
    pub fn enrich_batch(
        &self,
        plugin_id: &str,
        inputs: &[String],
        input_type: &str,
        config: &PluginConfigMap,
    ) -> Result<Vec<Option<Value>>, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::enrich_batch not yet implemented")
    }

    /// Call `fire_alert` on the named action plugin.
    pub fn fire_alert(
        &self,
        plugin_id: &str,
        ctx: AlertContext,
        config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::fire_alert not yet implemented")
    }

    /// Call `fire_case` on the named action plugin.
    pub fn fire_case(
        &self,
        plugin_id: &str,
        ctx: CaseContext,
        config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::fire_case not yet implemented")
    }

    /// Call `fire_report` on the named action plugin.
    pub fn fire_report(
        &self,
        plugin_id: &str,
        ctx: ReportContext,
        config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginRuntime::fire_report not yet implemented")
    }
}
