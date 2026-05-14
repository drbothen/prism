//! WASM Plugin Runtime — `prism-spec-engine` SS-17.
//!
//! Implements the WASM Component Model plugin runtime per AD-019.

pub mod discovery;
pub mod host_functions;
pub mod hot_reload;
pub mod loader;
pub mod sandbox;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use prism_core::PluginError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};

// Re-export public types used by callers (S-1.14, S-4.08).
pub use loader::{HostState, LoadedPlugin, PluginConfigMap, PluginKvStore};
use sandbox::{
    DEFAULT_MEMORY_LIMIT_MB, DEFAULT_TIMEOUT_SECONDS, EpochTickerHandle, classify_wasm_error,
    create_store,
};

// ---------------------------------------------------------------------------
// Constants (AC-9 / AC-5 / S-PLUGIN-PREREQ-D)
// ---------------------------------------------------------------------------

/// Per-request HTTP timeout for plugin outbound HTTP calls.
///
/// This constant defines the timeout configured at `reqwest::Client::builder()` level
/// in `boot.rs`. It is 30 seconds — NOT 10 seconds (TD-S-PLUGIN-PREREQ-B-005 closure).
/// The constant lives here so boot.rs can import it without a circular dep.
pub const PLUGIN_HTTP_CLIENT_TIMEOUT_SECS: u64 = 30;

/// Current maximum supported plugin manifest `format_version` (BC-2.17.007).
///
/// Manifests with `format_version > CURRENT_SUPPORTED_VERSION` are rejected with
/// `E-PLUGIN-014 FormatVersionExceeded`.
pub const CURRENT_SUPPORTED_VERSION: u32 = 1;

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
    /// Create a new `PluginRuntime` with the given `http_client`.
    ///
    /// The `http_client` MUST be constructed at boot with `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))`
    /// (TD-S-PLUGIN-PREREQ-B-005 closure; AC-9). `boot.rs` constructs the single shared client
    /// and passes it here via owned value; `PluginRuntime` wraps it in `Arc<reqwest::Client>`.
    ///
    /// # Errors
    ///
    /// Returns `Err(PrismError::Internal)` if the wasmtime `Engine` cannot be constructed.
    pub fn new(http_client: reqwest::Client) -> Result<Self, prism_core::PrismError> {
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

        Ok(Self {
            engine,
            linker,
            registry: ArcSwap::new(Arc::new(HashMap::new())),
            http_client: Arc::new(http_client),
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

    /// Scan `plugin_dir` for `*.prx` files and load each one.
    ///
    /// For each `.prx` file found:
    /// 1. Read bytes and compute SHA-256 hash for the per-plugin audit entry.
    /// 2. Parse manifest fields: `name`, `version`, `format_version`, `allowed_urls`.
    /// 3. Validate manifest schema (BC-2.17.007; first-failure-returns per EC-17-032):
    ///    - `name` non-empty string → `E-PLUGIN-015` on failure
    ///    - `version` semver-parseable → `E-PLUGIN-016` on failure
    ///    - `format_version <= CURRENT_SUPPORTED_VERSION` → `E-PLUGIN-014` on failure
    ///    - `allowed_urls` explicitly present (empty list `[]` accepted) → `E-PLUGIN-013` on failure
    /// 4. Compile with `Component::from_binary` in `tokio::task::spawn_blocking`.
    /// 5. Validate WIT interface (`E-PLUGIN-001` on missing required export).
    /// 6. Register in arc-swap registry (first-registered wins on duplicate plugin_id).
    /// 7. Emit `plugin_load_unsigned` WARN audit entry with `plugin_path` + `plugin_hash`.
    ///
    /// On each success, emits a one-time boot WARN about unsigned plugins (VP-PLUGIN-004).
    ///
    /// Returns `Ok(n_loaded)` after all files are processed (n-1 survivor rule applies).
    /// Returns `Ok(0)` if `plugin_dir` does not exist (EC-D-001) or contains no `.prx` files.
    ///
    /// # Errors
    ///
    /// This method does NOT return `Err` for per-plugin failures — those are logged at ERROR
    /// and the n-1 survivor rule applies. Only `Err` cases: filesystem errors reading the directory.
    pub async fn load_all_plugins(
        &self,
        plugin_dir: &Path,
    ) -> Result<usize, prism_core::PrismError> {
        // EC-D-001: plugin directory does not exist → Ok(0), INFO log.
        if !plugin_dir.exists() {
            info!(
                plugin_dir = %plugin_dir.display(),
                event_type = "plugin_directory_not_found",
                "plugin directory not found, skipping plugin load"
            );
            return Ok(0);
        }

        let entries = match std::fs::read_dir(plugin_dir) {
            Ok(e) => e,
            Err(err) => {
                return Err(prism_core::PrismError::Internal {
                    detail: format!(
                        "failed to read plugin directory '{}': {}",
                        plugin_dir.display(),
                        err
                    ),
                });
            }
        };

        // Collect .prx paths.
        let mut prx_paths: Vec<std::path::PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("prx"))
            .collect();

        prx_paths.sort(); // deterministic load order

        if prx_paths.is_empty() {
            // EC-D-002: zero .prx files — Ok(0), INFO log (no unsigned-plugin WARN emitted).
            info!(
                plugin_dir = %plugin_dir.display(),
                "no .prx plugin files found in plugin directory"
            );
            return Ok(0);
        }

        // One-time unsigned-plugin boot warning (emitted once per boot, not per plugin).
        warn!(
            "WARNING: Plugin signing not yet implemented (TD-PLUGIN-SIGNING-001). \
             Loaded plugins are NOT cryptographically verified. Do not run untrusted plugins."
        );

        let mut n_loaded = 0usize;
        let engine = self.engine.clone();
        let linker = self.linker.clone();

        for path in prx_paths {
            let path_str = path.display().to_string();

            // Read bytes.
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(err) => {
                    error!(
                        plugin_path = %path_str,
                        error = %err,
                        event_type = "plugin_load_failed_read_error",
                        "failed to read plugin file; skipping"
                    );
                    continue;
                }
            };

            // Compute SHA-256 hash BEFORE compilation (for audit entry).
            let plugin_hash = {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                format!("{:x}", hasher.finalize())
            };

            // Parse manifest — embedded as a `[manifest]` section in the WASM custom section
            // or as a companion TOML file. For our WAT-compiled fixtures, we use a companion
            // `.manifest.toml` file at the same path (e.g., `minimal.manifest.toml`).
            //
            // Manifest parsing strategy:
            //   1. Try to read `{path}.manifest.toml` (companion file) — used by fixtures.
            //   2. TODO(S-4.08-manifest-embedding): parse WASM custom section for production .prx.
            let manifest_path = path.with_extension("manifest.toml");
            let manifest_toml = if manifest_path.exists() {
                match std::fs::read_to_string(&manifest_path) {
                    Ok(s) => Some(s),
                    Err(err) => {
                        error!(
                            plugin_path = %path_str,
                            error = %err,
                            "failed to read manifest file; skipping plugin"
                        );
                        continue;
                    }
                }
            } else {
                None
            };

            // Parse manifest fields (BC-2.17.007 validation order: name → version → format_version → allowed_urls).
            let (plugin_name, plugin_version, _format_version, allowed_urls) = match parse_manifest(
                manifest_toml.as_deref(),
                &path_str,
            ) {
                Ok(fields) => fields,
                Err(err) => {
                    // Emit appropriate structured event and log at ERROR.
                    match &err {
                        PluginError::ManifestNotFound {
                            expected_manifest_path,
                            ..
                        } => {
                            // HIGH-005 (F-IMPL-LP1-HIGH-005): E-PLUGIN-018 manifest not found.
                            error!(
                                plugin_path = %path_str,
                                expected_manifest_path = %expected_manifest_path,
                                error = "E-PLUGIN-018",
                                event_type = "plugin_load_failed_manifest_not_found",
                                "Plugin missing companion manifest file"
                            );
                        }
                        PluginError::ManifestParseError { detail, .. } => {
                            // HIGH-003 (F-IMPL-LP1-HIGH-003): E-PLUGIN-017 TOML parse error.
                            error!(
                                plugin_path = %path_str,
                                error = "E-PLUGIN-017",
                                detail = %detail,
                                event_type = "plugin_load_failed_manifest_parse_error",
                                "Plugin manifest TOML parse failed"
                            );
                        }
                        PluginError::ManifestNameMissing { .. } => {
                            error!(
                                plugin_path = %path_str,
                                error = "E-PLUGIN-015",
                                event_type = "plugin_load_failed_manifest_name_missing",
                                "Plugin manifest missing or empty required field 'name'"
                            );
                        }
                        PluginError::ManifestVersionMalformed { value, .. } => {
                            error!(
                                plugin_path = %path_str,
                                version_value = %value,
                                error = "E-PLUGIN-016",
                                event_type = "plugin_load_failed_manifest_version_malformed",
                                "Plugin manifest 'version' field is not valid semver"
                            );
                        }
                        PluginError::FormatVersionMissing { supported, .. } => {
                            // HIGH-006 (F-IMPL-LP1-HIGH-006): E-PLUGIN-019 absent format_version.
                            error!(
                                plugin_path = %path_str,
                                supported = supported,
                                error = "E-PLUGIN-019",
                                event_type = "plugin_load_failed_format_version_missing",
                                "Plugin manifest missing required field 'format_version'"
                            );
                        }
                        PluginError::FormatVersionExceeded {
                            actual, supported, ..
                        } => {
                            error!(
                                plugin_path = %path_str,
                                format_version = actual,
                                max_supported = supported,
                                error = "E-PLUGIN-014",
                                event_type = "plugin_load_failed_format_version_exceeded",
                                "Plugin manifest format_version exceeds maximum supported version"
                            );
                        }
                        PluginError::MissingAllowedUrls { .. } => {
                            error!(
                                plugin_path = %path_str,
                                error = "E-PLUGIN-013",
                                event_type = "plugin_load_failed_manifest_no_allowed_urls",
                                "Plugin manifest missing required field 'allowed_urls'"
                            );
                        }
                        _ => {
                            error!(
                                plugin_path = %path_str,
                                error = %err,
                                "Plugin manifest validation failed"
                            );
                        }
                    }
                    continue; // n-1 survivor rule
                }
            };

            // Spawn blocking WASM compilation (CPU-intensive).
            let bytes_clone = bytes.clone();
            let path_clone = path.clone();
            let engine_clone = engine.clone();
            let linker_clone = linker.clone();

            let compile_result = tokio::task::spawn_blocking(move || {
                discovery::load_plugin_from_bytes(
                    &engine_clone,
                    &linker_clone,
                    &path_clone,
                    &bytes_clone,
                )
            })
            .await
            .map_err(|e| prism_core::PrismError::Internal {
                detail: format!("spawn_blocking panicked for plugin '{}': {}", path_str, e),
            })?;

            let mut plugin = match compile_result {
                Ok(p) => p,
                Err(err) => {
                    match &err {
                        PluginError::InvalidInterface { missing_export, .. } => {
                            error!(
                                plugin_path = %path_str,
                                missing_export = %missing_export,
                                error = "E-PLUGIN-001",
                                event_type = "plugin_load_failed_wit_invalid",
                                "WIT interface validation failed — plugin missing required export"
                            );
                        }
                        PluginError::CompilationFailed { message, .. } => {
                            error!(
                                plugin_path = %path_str,
                                error = "E-PLUGIN-008",
                                message = %message,
                                event_type = "plugin_load_failed_compilation",
                                "Plugin WASM compilation failed"
                            );
                        }
                        _ => {
                            error!(
                                plugin_path = %path_str,
                                error = %err,
                                "Plugin load failed"
                            );
                        }
                    }
                    continue; // n-1 survivor rule
                }
            };

            // Override metadata from parsed manifest (supersedes name() export for production plugins).
            plugin.metadata.name = plugin_name.clone();
            plugin.metadata.plugin_id = plugin_name.clone();
            plugin.metadata.version = plugin_version;
            plugin.allowed_urls = allowed_urls;

            let plugin_id = plugin.metadata.plugin_id.clone();
            let plugin_arc = Arc::new(plugin);

            // Duplicate plugin_id check (EC-D-008: first-registered wins).
            {
                let current = self.registry.load();
                if current.contains_key(&plugin_id) {
                    warn!(
                        plugin_id = %plugin_id,
                        plugin_path = %path_str,
                        "Duplicate plugin_id '{}': first-registered plugin retained",
                        plugin_id
                    );
                    continue;
                }
            }

            // Register in arc-swap registry.
            self.registry.rcu(|current| {
                let mut updated = (**current).clone();
                updated.insert(plugin_id.clone(), plugin_arc.clone());
                updated
            });

            // Per-plugin audit entry: plugin_load_unsigned (AC-4 / VP-PLUGIN-004 / BC-2.16.002).
            // Single structured emission per BC-2.16.002 v1.12 catalog row.
            warn!(
                event_type = "plugin_load_unsigned",
                plugin_path = %path_str,
                plugin_hash = %plugin_hash,
                "Plugin loaded (unsigned — TD-PLUGIN-SIGNING-001)"
            );

            info!(
                plugin_id = %plugin_id,
                plugin_path = %path_str,
                "Plugin '{}' registered in runtime",
                plugin_id
            );

            n_loaded += 1;
        }

        info!(
            n_loaded = n_loaded,
            plugin_dir = %plugin_dir.display(),
            "boot: plugin-load step complete ({} plugins loaded)",
            n_loaded
        );

        Ok(n_loaded)
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
    /// `allowed_urls` is the per-plugin allowlist parsed from the manifest (AC-7 / AC-17).
    /// An empty Vec means default-deny (no outbound HTTP allowed). The function is pure:
    /// it receives `Arc<reqwest::Client>` via `Arc::clone` (no I/O, no construction).
    ///
    /// The `limits` field is a sentinel; `create_store()` overwrites it with the
    /// configured `StoreLimitsBuilder` value before registering the ResourceLimiter.
    fn make_host_state(
        &self,
        plugin_id: &str,
        config: &PluginConfigMap,
        allowed_urls: Vec<String>,
    ) -> HostState {
        HostState {
            http_client: self.http_client.clone(),
            config: Arc::new(config.clone()),
            kv_store: Arc::new(PluginKvStore::new()),
            plugin_id: plugin_id.to_string(),
            allowed_urls,
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
        let host_state = self.make_host_state(plugin_id, config, plugin.allowed_urls.clone());
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
                e.into(),
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
        // post_return removed — no longer needed in wasmtime >=44 (no-op, deprecated).

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => Ok(None),
            Err(e) => Err(classify_wasm_error(
                plugin_id,
                e.into(),
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
        let host_state = self.make_host_state(plugin_id, config, plugin.allowed_urls.clone());
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
                e.into(),
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
        // post_return removed — no longer needed in wasmtime >=44 (no-op, deprecated).

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => Ok(inputs.iter().map(|_| None).collect()),
            Err(e) => Err(classify_wasm_error(
                plugin_id,
                e.into(),
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
                e.into(),
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
                e.into(),
                memory_limit_mb,
                elapsed_ms,
                timeout_seconds * 1000,
            )),
        }
    }

    /// Call `fire_alert` on the named action plugin.
    ///
    /// # Stub — TODO(S-4.08-fire-alert-dispatch)
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
        // TODO(S-4.08-fire-alert-dispatch): invoke plugin.pre_instance → get_func("fire-alert") → call with ctx.
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
    /// # Stub — TODO(S-4.08-fire-case-dispatch)
    /// The actual WASM call to the plugin's `fire-case` export is not yet wired.
    /// Full WASM dispatch will be implemented in S-4.08.
    pub fn fire_case(
        &self,
        plugin_id: &str,
        ctx: CaseContext,
        _config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        let _plugin = self.get_plugin(plugin_id)?;
        // TODO(S-4.08-fire-case-dispatch): invoke plugin.pre_instance → get_func("fire-case") → call with ctx.
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
    /// # Stub — TODO(S-4.08-fire-report-dispatch)
    /// The actual WASM call to the plugin's `fire-report` export is not yet wired.
    /// Full WASM dispatch will be implemented in S-4.08.
    pub fn fire_report(
        &self,
        plugin_id: &str,
        ctx: ReportContext,
        _config: &PluginConfigMap,
    ) -> Result<ActionResult, PluginError> {
        let _plugin = self.get_plugin(plugin_id)?;
        // TODO(S-4.08-fire-report-dispatch): invoke plugin.pre_instance → get_func("fire-report") → call with ctx.
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

// ---------------------------------------------------------------------------
// Manifest parsing helpers (BC-2.17.007)
// ---------------------------------------------------------------------------

/// TOML manifest structure for a `.prx` plugin.
///
/// Validated by `parse_manifest()` before WIT compilation per BC-2.17.007.
/// Marked `#[non_exhaustive]` — future fields may be added without breaking external code.
#[non_exhaustive]
#[derive(Debug, serde::Deserialize)]
struct PluginManifest {
    /// Plugin display name (non-empty required, E-PLUGIN-015).
    name: Option<String>,
    /// Plugin semantic version string (semver-parseable required, E-PLUGIN-016).
    version: Option<String>,
    /// Manifest schema version — must be `<= CURRENT_SUPPORTED_VERSION` (E-PLUGIN-014).
    format_version: Option<u32>,
    /// Outbound HTTP allowlist (required field; empty list `[]` accepted; E-PLUGIN-013).
    allowed_urls: Option<Vec<String>>,
}

/// Parse and validate a plugin manifest TOML string.
///
/// Validation order (BC-2.17.007 EC-17-032 first-failure-returns):
/// 1. `name` — non-empty string (E-PLUGIN-015)
/// 2. `version` — semver-parseable string (E-PLUGIN-016)
/// 3. `format_version` — `<= CURRENT_SUPPORTED_VERSION` (E-PLUGIN-014)
/// 4. `allowed_urls` — explicitly present (E-PLUGIN-013)
///
/// Returns `(name, version, format_version, allowed_urls)` on success.
///
/// Returns appropriate `PluginError` variant on the first failing field.
fn parse_manifest(
    manifest_toml: Option<&str>,
    path: &str,
) -> Result<(String, String, u32, Vec<String>), PluginError> {
    let manifest: PluginManifest = if let Some(toml_str) = manifest_toml {
        // HIGH-003 (F-IMPL-LP1-HIGH-003): TOML parse failures map to E-PLUGIN-017
        // (ManifestParseError), NOT E-PLUGIN-015 (ManifestNameMissing).
        // This distinguishes "file exists but is invalid TOML" from "TOML parses
        // but the name field is absent or empty".
        toml::from_str(toml_str).map_err(|e| PluginError::ManifestParseError {
            path: path.to_string(),
            detail: e.to_string(),
        })?
    } else {
        // HIGH-005 (F-IMPL-LP1-HIGH-005): absent manifest → explicit E-PLUGIN-018
        // (ManifestNotFound), not synthesized all-None that silently fails on 'name'.
        // A manifest is REQUIRED for production plugins; no manifest = hard rejection.
        let expected = format!("{path}.manifest.toml");
        return Err(PluginError::ManifestNotFound {
            plugin_path: path.to_string(),
            expected_manifest_path: expected,
        });
    };

    // 1. Validate name (E-PLUGIN-015): must be non-empty string.
    let name = match manifest.name.as_deref() {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => {
            return Err(PluginError::ManifestNameMissing {
                path: path.to_string(),
            });
        }
    };

    // 2. Validate version (E-PLUGIN-016): must be parseable as semver.
    // HIGH-004 (F-IMPL-LP1-HIGH-004): use semver::Version::parse (strict semver 2.0.0),
    // replacing the permissive is_valid_semver() that accepted "a.b", "1.+", etc.
    // semver::Version::parse requires exactly "major.minor.patch[-prerelease][+build]".
    // "1.2" (missing patch) and "a.b" (non-integer) are correctly rejected.
    let version_str = match manifest.version.as_deref() {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            return Err(PluginError::ManifestVersionMalformed {
                path: path.to_string(),
                value: manifest.version.clone().unwrap_or_default(),
            });
        }
    };

    if semver::Version::parse(&version_str).is_err() {
        return Err(PluginError::ManifestVersionMalformed {
            path: path.to_string(),
            value: version_str,
        });
    }

    // 3. Validate format_version (E-PLUGIN-014 / E-PLUGIN-019).
    // HIGH-006 (F-IMPL-LP1-HIGH-006): absent format_version → E-PLUGIN-019
    // (FormatVersionMissing), NOT a silent default of 0 that passes the cap check.
    // AC-5 (story line 322): absent format_version MUST be rejected.
    let format_version = match manifest.format_version {
        Some(v) => v,
        None => {
            return Err(PluginError::FormatVersionMissing {
                path: path.to_string(),
                supported: CURRENT_SUPPORTED_VERSION,
            });
        }
    };
    if format_version > CURRENT_SUPPORTED_VERSION {
        return Err(PluginError::FormatVersionExceeded {
            path: path.to_string(),
            actual: format_version,
            supported: CURRENT_SUPPORTED_VERSION,
        });
    }

    // 4. Validate allowed_urls (E-PLUGIN-013): must be EXPLICITLY present (Some(_)).
    // An empty list `[]` is accepted (default-deny). Absent / null → rejection.
    // MED-007 (F-IMPL-LP1-MED-007): validate that no entry is an empty string.
    // An empty string in allowed_urls would match any URL with an empty host (unparseable
    // URLs return host_str() == ""), creating a de-facto allow-all bypass.
    let allowed_urls = match manifest.allowed_urls {
        Some(urls) => {
            if urls.iter().any(|u| u.is_empty()) {
                return Err(PluginError::MissingAllowedUrls {
                    path: format!(
                        "{path} — allowed_urls contains empty string entry (default-deny bypass)"
                    ),
                });
            }
            urls
        }
        None => {
            return Err(PluginError::MissingAllowedUrls {
                path: path.to_string(),
            });
        }
    };

    Ok((name, version_str, format_version, allowed_urls))
}
