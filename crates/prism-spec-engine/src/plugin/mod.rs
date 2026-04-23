//! WASM Plugin Runtime — `prism-spec-engine` SS-17.
//!
//! Implements the WASM Component Model plugin runtime per AD-019.

pub mod discovery;
pub mod host_functions;
pub mod hot_reload;
pub mod loader;
pub mod sandbox;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use prism_core::PluginError;
use serde_json::Value;
use tracing::info;

// Re-export public types used by callers (S-1.14, S-4.08).
pub use loader::{HostState, LoadedPlugin, PluginConfigMap, PluginKvStore};
use sandbox::{
    classify_wasm_error, create_store, EpochTickerHandle, DEFAULT_MEMORY_LIMIT_MB,
    DEFAULT_TIMEOUT_SECONDS,
};

/// The three Prism plugin types recognised by WIT validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginType {
    Sensor,
    Infusion,
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
/// `engine`, `linker`, and `registry` are public for use by the hot_reload and
/// VP-042 integration test harness. Callers should prefer the typed methods
/// (`load_plugin`, `enrich_single`, etc.) over direct field access.
pub struct PluginRuntime {
    pub engine: wasmtime::Engine,
    pub linker: wasmtime::component::Linker<HostState>,
    pub registry: ArcSwap<HashMap<String, Arc<LoadedPlugin>>>,
    http_client: Arc<reqwest::Client>,
    /// Epoch ticker handle — kept alive to keep background thread running.
    _epoch_ticker: EpochTickerHandle,
}

impl PluginRuntime {
    /// Create a new `PluginRuntime`.
    pub fn new() -> Result<Self, prism_core::PrismError> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);

        let engine =
            wasmtime::Engine::new(&config).map_err(|e| prism_core::PrismError::Internal {
                detail: format!("wasmtime Engine construction failed: {}", e),
            })?;

        let mut linker = wasmtime::component::Linker::<HostState>::new(&engine);
        host_functions::register_host_functions(&mut linker)?;

        let epoch_engine = engine.clone();
        let epoch_ticker = sandbox::start_epoch_ticker(epoch_engine);

        let http_client = Arc::new(reqwest::Client::new());

        Ok(Self {
            engine,
            linker,
            registry: ArcSwap::new(Arc::new(HashMap::new())),
            http_client,
            _epoch_ticker: epoch_ticker,
        })
    }

    /// Build a `Linker<HostState>` (no WASI — only Prism host functions).
    pub fn build_linker(
        engine: &wasmtime::Engine,
    ) -> Result<wasmtime::component::Linker<HostState>, prism_core::PrismError> {
        let mut linker = wasmtime::component::Linker::<HostState>::new(engine);
        host_functions::register_host_functions(&mut linker)?;
        Ok(linker)
    }

    /// Load and validate a `.prx` plugin binary from `path`.
    pub fn load_plugin(&self, path: &std::path::Path) -> Result<Arc<LoadedPlugin>, PluginError> {
        let bytes = std::fs::read(path).map_err(|e| PluginError::CompilationFailed {
            path: path.display().to_string(),
            message: format!("failed to read file: {}", e),
        })?;

        let plugin = discovery::load_plugin_from_bytes(&self.engine, &self.linker, path, &bytes)?;

        let plugin_arc = Arc::new(plugin);
        let plugin_id = plugin_arc.metadata.plugin_id.clone();

        self.registry.rcu(|current| {
            let mut updated = (**current).clone();
            updated.insert(plugin_id.clone(), plugin_arc.clone());
            updated
        });

        info!("Loaded plugin '{}' from '{}'", plugin_id, path.display());
        Ok(plugin_arc)
    }

    /// Return an `Arc<LoadedPlugin>` for `plugin_id`, or `Err(NotLoaded)`.
    pub fn get_plugin(&self, plugin_id: &str) -> Result<Arc<LoadedPlugin>, PluginError> {
        let registry = self.registry.load();
        registry
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| PluginError::NotLoaded {
                plugin_id: plugin_id.to_string(),
            })
    }

    /// List all registered plugin_ids.
    pub fn list_plugins(&self) -> Vec<String> {
        self.registry.load().keys().cloned().collect()
    }

    /// Build a `HostState` for a new plugin call store.
    ///
    /// `allowed_urls` is `None` (all URLs allowed) until per-plugin URL allowlist
    /// configuration is loaded from TOML plugin specs — deferred to S-4.08 when
    /// plugin config integration with the sensor TOML spec format is implemented.
    /// The `limits` field is a sentinel; `create_store()` overwrites it with the
    /// configured `StoreLimitsBuilder` value before registering the ResourceLimiter.
    fn make_host_state(&self, plugin_id: &str, config: &PluginConfigMap) -> HostState {
        HostState {
            http_client: self.http_client.clone(),
            config: Arc::new(config.clone()),
            kv_store: Arc::new(PluginKvStore::new()),
            plugin_id: plugin_id.to_string(),
            // TODO(S-4.08): load per-plugin URL allowlist from plugin TOML config.
            allowed_urls: None,
            // Sentinel — overwritten by create_store() before ResourceLimiter registration.
            limits: wasmtime::StoreLimits::default(),
        }
    }

    /// Call `enrich_single` on the named infusion plugin.
    pub fn enrich_single(
        &self,
        plugin_id: &str,
        input_value: &str,
        input_type: &str,
        config: &PluginConfigMap,
    ) -> Result<Option<Value>, PluginError> {
        let plugin = self.get_plugin(plugin_id)?;

        // If this is a core module (WAT fixture), use the core module call path.
        if let Some(ref core_mod) = plugin.core_module {
            return self
                .call_core_export(
                    plugin_id,
                    core_mod,
                    "enrich-single",
                    DEFAULT_MEMORY_LIMIT_MB,
                    DEFAULT_TIMEOUT_SECONDS,
                )
                .map(|_| None);
        }

        // Component Model path (true .prx with lifted exports).
        let host_state = self.make_host_state(plugin_id, config);
        let mut store = create_store(
            &self.engine,
            host_state,
            DEFAULT_MEMORY_LIMIT_MB,
            DEFAULT_TIMEOUT_SECONDS,
        );

        let start = Instant::now();

        let instance = plugin.pre_instance.instantiate(&mut store).map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            classify_wasm_error(
                plugin_id,
                e,
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )
        })?;

        let func = instance
            .get_func(&mut store, "enrich-single")
            .ok_or_else(|| PluginError::InvalidInterface {
                path: plugin_id.to_string(),
                missing_export: "enrich-single".to_string(),
            })?;

        let params = [
            wasmtime::component::Val::S32(0),
            wasmtime::component::Val::S32(input_value.len() as i32),
            wasmtime::component::Val::S32(0),
            wasmtime::component::Val::S32(input_type.len() as i32),
        ];
        let mut results = vec![wasmtime::component::Val::S32(0)];

        let call_result = func.call(&mut store, &params, &mut results);
        let _ = func.post_return(&mut store);

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => Ok(None),
            Err(e) => Err(classify_wasm_error(
                plugin_id,
                e,
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )),
        }
    }

    /// Call `enrich_batch` on the named infusion plugin.
    pub fn enrich_batch(
        &self,
        plugin_id: &str,
        inputs: &[String],
        input_type: &str,
        config: &PluginConfigMap,
    ) -> Result<Vec<Option<Value>>, PluginError> {
        let plugin = self.get_plugin(plugin_id)?;

        // Core module path.
        if let Some(ref core_mod) = plugin.core_module {
            return self
                .call_core_export(
                    plugin_id,
                    core_mod,
                    "enrich-batch",
                    DEFAULT_MEMORY_LIMIT_MB,
                    DEFAULT_TIMEOUT_SECONDS,
                )
                .map(|_| inputs.iter().map(|_| None).collect());
        }

        // Component Model path.
        let host_state = self.make_host_state(plugin_id, config);
        let mut store = create_store(
            &self.engine,
            host_state,
            DEFAULT_MEMORY_LIMIT_MB,
            DEFAULT_TIMEOUT_SECONDS,
        );

        let start = Instant::now();

        let instance = plugin.pre_instance.instantiate(&mut store).map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            classify_wasm_error(
                plugin_id,
                e,
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )
        })?;

        let func = instance
            .get_func(&mut store, "enrich-batch")
            .ok_or_else(|| PluginError::InvalidInterface {
                path: plugin_id.to_string(),
                missing_export: "enrich-batch".to_string(),
            })?;

        let params = [
            wasmtime::component::Val::S32(0),
            wasmtime::component::Val::S32(inputs.len() as i32),
            wasmtime::component::Val::S32(0),
            wasmtime::component::Val::S32(input_type.len() as i32),
        ];
        let mut results = vec![
            wasmtime::component::Val::S32(0),
            wasmtime::component::Val::S32(0),
        ];

        let call_result = func.call(&mut store, &params, &mut results);
        let _ = func.post_return(&mut store);

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => Ok(inputs.iter().map(|_| None).collect()),
            Err(e) => Err(classify_wasm_error(
                plugin_id,
                e,
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )),
        }
    }

    /// Call a named export on a core WASM module with epoch interruption for CPU time limiting.
    fn call_core_export(
        &self,
        plugin_id: &str,
        module: &wasmtime::Module,
        func_name: &str,
        memory_limit_mb: u64,
        timeout_seconds: u64,
    ) -> Result<(), PluginError> {
        use wasmtime::{Linker, Store};

        let mut store: Store<()> = Store::new(&self.engine, ());
        store.set_epoch_deadline(timeout_seconds * sandbox::EPOCH_TICKS_PER_SECOND);

        // Simple linker with no imports — WAT test fixtures have no imports.
        let linker: Linker<()> = Linker::new(&self.engine);

        let start = Instant::now();

        let instance = linker.instantiate(&mut store, module).map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            classify_wasm_error(
                plugin_id,
                e,
                memory_limit_mb,
                elapsed_ms,
                timeout_seconds * 1000,
            )
        })?;

        let func = instance.get_func(&mut store, func_name).ok_or_else(|| {
            PluginError::InvalidInterface {
                path: plugin_id.to_string(),
                missing_export: func_name.to_string(),
            }
        })?;

        // Call with dummy i32 params (4 i32 params, 1 or 2 i32 results depending on func).
        // We don't care about results — just whether it traps/times out.
        let param_vals = vec![wasmtime::Val::I32(0); func.ty(&store).params().len()];
        let result_count = func.ty(&store).results().len();
        let mut results = vec![wasmtime::Val::I32(0); result_count];

        let call_result = func.call(&mut store, &param_vals, &mut results);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => Ok(()),
            Err(e) => Err(classify_wasm_error(
                plugin_id,
                e,
                memory_limit_mb,
                elapsed_ms,
                timeout_seconds * 1000,
            )),
        }
    }

    /// Call `fire_alert` on the named action plugin.
    ///
    /// # Stub — TODO(S-4.08)
    /// The actual WASM call to the plugin's `fire-alert` export is not yet wired.
    /// This stub validates that the plugin is registered and returns a synthetic
    /// success result. Full WASM dispatch will be implemented in S-4.08.
    pub fn fire_alert(
        &self,
        plugin_id: &str,
        ctx: AlertContext,
        _config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        let _plugin = self.get_plugin(plugin_id)?;
        // TODO(S-4.08): invoke plugin.pre_instance → get_func("fire-alert") → call with ctx.
        Ok(ActionResult {
            success: true,
            message: Some(format!(
                "alert {} acknowledged by plugin (WASM dispatch deferred to S-4.08)",
                ctx.alert_id
            )),
            raw_response: None,
        })
    }

    /// Call `fire_case` on the named action plugin.
    ///
    /// # Stub — TODO(S-4.08)
    /// The actual WASM call to the plugin's `fire-case` export is not yet wired.
    /// Full WASM dispatch will be implemented in S-4.08.
    pub fn fire_case(
        &self,
        plugin_id: &str,
        ctx: CaseContext,
        _config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        let _plugin = self.get_plugin(plugin_id)?;
        // TODO(S-4.08): invoke plugin.pre_instance → get_func("fire-case") → call with ctx.
        Ok(ActionResult {
            success: true,
            message: Some(format!(
                "case {} acknowledged by plugin (WASM dispatch deferred to S-4.08)",
                ctx.case_id
            )),
            raw_response: None,
        })
    }

    /// Call `fire_report` on the named action plugin.
    ///
    /// # Stub — TODO(S-4.08)
    /// The actual WASM call to the plugin's `fire-report` export is not yet wired.
    /// Full WASM dispatch will be implemented in S-4.08.
    pub fn fire_report(
        &self,
        plugin_id: &str,
        ctx: ReportContext,
        _config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        let _plugin = self.get_plugin(plugin_id)?;
        // TODO(S-4.08): invoke plugin.pre_instance → get_func("fire-report") → call with ctx.
        Ok(ActionResult {
            success: true,
            message: Some(format!(
                "report {} acknowledged by plugin (WASM dispatch deferred to S-4.08)",
                ctx.report_id
            )),
            raw_response: None,
        })
    }
}
