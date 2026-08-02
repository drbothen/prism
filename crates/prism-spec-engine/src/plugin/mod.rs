//! WASM Plugin Runtime — `prism-spec-engine` SS-17.
//!
//! Implements the WASM Component Model plugin runtime per AD-019.

pub mod discovery;
pub mod host_functions;
pub mod hot_reload;
pub mod loader;
pub mod sandbox;

use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};

use arc_swap::ArcSwap;
// Re-export public types used by callers (S-1.14, S-4.08).
pub use loader::{HostState, LoadedPlugin, PluginConfigMap, PluginKvStore};
use prism_core::PluginError;
use sandbox::{
    DEFAULT_MEMORY_LIMIT_MB, DEFAULT_TIMEOUT_SECONDS, EpochTickerHandle, classify_wasm_error,
    create_store,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};

use crate::plugin_audit_sink::{NoOpPluginAuditSink, PluginLoadAuditSink};

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

/// The Prism plugin types recognised by WIT validation.
///
/// `SensorAuth` is added for PLUGIN-MIGRATION-001-E: OAuth2 client-credentials
/// authentication plugins that export `auth-type-name`, `acquire-token`, and
/// `get-token` per BC-2.17.006 WIT validation gate.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginType {
    Sensor,
    Infusion,
    Action,
    /// Sensor authentication plugin (e.g., crowdstrike-oauth2).
    ///
    /// Required exports: `auth-type-name`, `acquire-token`, `get-token`.
    /// Registered with PluginRuntime and dispatched from the spec-engine
    /// auth path when SensorSpec.auth_plugin is Some.
    SensorAuth,
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
    /// Durable audit sink for plugin load events (HIGH-002 / AC-4 / BC-2.05.012).
    ///
    /// Production: `Arc<RocksDbPluginAuditSink>` wired from boot.rs step 6 result.
    /// Tests: `Arc<NoOpPluginAuditSink>` (no I/O).
    /// Default (PluginRuntime::new): `Arc<NoOpPluginAuditSink>` — callers that need
    /// durable audit MUST use `PluginRuntime::new_with_audit_sink`.
    audit_sink: Arc<dyn PluginLoadAuditSink>,
    /// Epoch ticker handle — kept alive to keep background thread running.
    _epoch_ticker: EpochTickerHandle,
}

// ---------------------------------------------------------------------------
// wasmtime compilation cache helper (S-PERF-GATE-008 / ADR-049 D3)
// ---------------------------------------------------------------------------

/// Attempts to enable the wasmtime compilation cache on `config`.
///
/// Cache-init failure is DEGRADABLE (ADR-049 D3 — LOCKED decision): on `Err`, emits a
/// `WARN` structured event and returns without error, leaving the engine uncached.
/// On `Ok`, attaches the cache to `config` via `config.cache(Some(cache))`.
///
/// Extracted as a standalone function (not inline in `new_with_audit_sink`) so that
/// the `Err` branch can be exercised by unit tests without needing a live wasmtime
/// Engine or a real `.prx` artifact (SID-1).
///
/// S-PERF-GATE-008 / ADR-049 D3.
fn apply_wasmtime_cache(
    config: &mut wasmtime::Config,
    cache_result: Result<wasmtime::Cache, wasmtime::Error>,
) {
    match cache_result {
        Ok(cache) => {
            config.cache(Some(cache));
        }
        Err(e) => {
            tracing::warn!(
                event_type = "plugin.compilation_cache_init_skipped",
                error = %e,
                "wasmtime compilation cache init failed; proceeding without cache (degraded performance)"
            );
        }
    }
}

impl PluginRuntime {
    /// Create a new `PluginRuntime` with the given `http_client` and a `NoOpPluginAuditSink`.
    ///
    /// The `http_client` MUST be constructed at boot with `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))`
    /// (TD-S-PLUGIN-PREREQ-B-005 closure; AC-9). `boot.rs` constructs the single shared client
    /// and passes it here via owned value; `PluginRuntime` wraps it in `Arc<reqwest::Client>`.
    ///
    /// **Tests** use this constructor — the `NoOpPluginAuditSink` produces no I/O.
    /// **Production boot** MUST use `new_with_audit_sink` to wire the RocksDB audit channel
    /// (HIGH-002 / AC-4 / BC-2.05.012).
    ///
    /// # Errors
    ///
    /// Returns `Err(PrismError::Internal)` if the wasmtime `Engine` cannot be constructed.
    pub fn new(http_client: reqwest::Client) -> Result<Self, prism_core::PrismError> {
        Self::new_with_audit_sink(http_client, Arc::new(NoOpPluginAuditSink))
    }

    /// Create a new `PluginRuntime` with the given `http_client` and a custom `audit_sink`.
    ///
    /// Production boot path MUST use this constructor and pass a `RocksDbPluginAuditSink`
    /// (defined in `prism-bin`) wired from the step 6 `Arc<RocksDbBackend>` result.
    /// The audit sink records durable, fsync-confirmed entries in the `audit_buffer` CF
    /// for each plugin load event (AC-4 / BC-2.05.012 / HIGH-002).
    ///
    /// # Errors
    ///
    /// Returns `Err(PrismError::Internal)` if the wasmtime `Engine` cannot be constructed.
    pub fn new_with_audit_sink(
        http_client: reqwest::Client,
        audit_sink: Arc<dyn PluginLoadAuditSink>,
    ) -> Result<Self, prism_core::PrismError> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);

        // Enable the wasmtime compilation cache (S-PERF-GATE-008).
        //
        // wasmtime::Component::new() (WASM-to-native Cranelift compilation) caches compiled
        // native code to disk, addressed by (wasm_binary_hash, compiler_version, cpu_isa_flags).
        // Warm cache hits skip Cranelift entirely; see ADR-049 / S-PERF-GATE-008 for
        // measured figures. Cache directory: OS default (~/.cache/wasmtime/ or
        // ~/Library/Caches/wasmtime/). Created automatically on first use.
        //
        // Cache-init failure is DEGRADABLE (ADR-049 D3): a disk-full, permissions, or
        // read-only-filesystem condition must not abort the analyst's session. PluginRuntime
        // construction continues without the cache; plugins recompile on each cold start
        // (slower but functionally correct).
        apply_wasmtime_cache(
            &mut config,
            wasmtime::Cache::new(wasmtime::CacheConfig::new()),
        );

        // SECURITY INVARIANT — SINGLE ENGINE PER PROCESS (RUSTSEC-2026-0222 / AC-006)
        //
        // This is the ONE AND ONLY production `wasmtime::Engine` construction in the entire
        // prism workspace. This invariant is load-bearing for RUSTSEC-2026-0222 mitigation:
        // the advisory describes a use-after-free in the epoch-ticker interrupt path that
        // is only reachable when two *independent* `Engine` instances exist in the same
        // process. Prism is not reachable because exactly one `Engine` is ever constructed.
        //
        // DO NOT introduce a second `Engine::new(...)` call anywhere in prism-spec-engine
        // or any crate that depends on it without first obtaining security clearance and
        // verifying the resolved wasmtime version carries a fix for RUSTSEC-2026-0222.
        //
        // `Engine::clone()` (used below for `epoch_engine`) is reference-counted to this
        // same instance — it does NOT create a second independent engine and does NOT
        // violate this invariant.
        //
        // Enforcement: workspace-wide grep for `Engine::new` must return only this site
        // (excluding test-only and proofs files) before any PR that touches
        // `PluginRuntime::new_with_audit_sink` or adds new wasmtime Engine usage can merge.
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
            audit_sink,
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
    /// Returns `(n_loaded, pending_write_tool_registrations)` on success.
    ///
    /// `pending_write_tool_registrations` is `Vec<(plugin_name, ManifestWriteTool)>` — each
    /// entry must be registered with `prism_query::invalidation::register_write_tool` by the
    /// caller (boot.rs step 7.5). Cannot be done here because prism-spec-engine cannot depend
    /// on prism-query (circular dependency; F-LP-IMPL-P1-002 / ADR-026 §D7).
    pub async fn load_all_plugins(
        &self,
        plugin_dir: &Path,
    ) -> Result<(usize, Vec<(String, ManifestWriteTool)>), prism_core::PrismError> {
        // EC-D-001: plugin directory does not exist → Ok(0), INFO log.
        if !plugin_dir.exists() {
            info!(
                plugin_dir = %plugin_dir.display(),
                event_type = "plugin_directory_not_found",
                "plugin directory not found, skipping plugin load"
            );
            return Ok((0, Vec::new()));
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
            return Ok((0, Vec::new()));
        }

        // One-time unsigned-plugin boot warning (emitted once per boot, not per plugin).
        warn!(
            "WARNING: Plugin signing not yet implemented (TD-PLUGIN-SIGNING-001). \
             Loaded plugins are NOT cryptographically verified. Do not run untrusted plugins."
        );

        let mut n_loaded = 0usize;
        // Accumulates write tool declarations from each loaded plugin's manifest.
        // Returned to the caller (boot.rs) for registration with prism-query's invalidation map.
        // Cannot be registered here due to prism-spec-engine ↛ prism-query (circular dep avoidance).
        let mut pending_write_tool_registrations: Vec<(String, ManifestWriteTool)> = Vec::new();
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

            // Parse manifest — companion TOML file approach.
            //
            // Current manifest discovery strategy:
            //   1. Read `{path}.manifest.toml` companion file — the canonical path for
            //      production .prx files and WAT-compiled test fixtures alike.
            //   2. Future: WASM custom section embedding (tracked in STORY-INDEX as a
            //      Wave 4/5 enhancement; no story ID assigned yet — requires product-owner
            //      authorship before implementation).
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
            // Returns write_tools as the 5th element (F-LP-IMPL-P1-002 / S-PLUGIN-PREREQ-E AC-9 / ADR-026 §D7).
            let (plugin_name, plugin_version, _format_version, allowed_urls, write_tools) =
                match parse_manifest(manifest_toml.as_deref(), &path_str) {
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
            // HIGH-002 (F-IMPL-LP1-HIGH-002): emit DURABLE audit entry via audit_sink
            // (not just tracing::warn!) per BC-2.05.012 "synchronous and confirmed durable".
            // The audit_sink persists to audit_buffer CF via append_audit_entry_sync (fsync).
            // In tests, NoOpPluginAuditSink is a no-op; production uses RocksDbPluginAuditSink.
            warn!(
                event_type = "plugin_load_unsigned",
                plugin_path = %path_str,
                plugin_hash = %plugin_hash,
                "Plugin loaded (unsigned — TD-PLUGIN-SIGNING-001)"
            );
            if let Err(audit_err) = self.audit_sink.record_plugin_load_event(
                "plugin_load_unsigned",
                &path_str,
                &plugin_hash,
                None,
            ) {
                // Audit sink failure is non-fatal per n-1 survivor rule.
                // Log at ERROR so operators are alerted — this indicates an audit gap.
                error!(
                    plugin_path = %path_str,
                    audit_error = %audit_err,
                    "AUDIT SINK FAILURE: plugin_load_unsigned entry could not be persisted \
                     (RocksDB write error). Audit gap for this plugin load. \
                     (BC-2.05.012 durable audit channel degraded)"
                );
            }

            info!(
                plugin_id = %plugin_id,
                plugin_path = %path_str,
                "Plugin '{}' registered in runtime",
                plugin_id
            );

            n_loaded += 1;

            // Collect write tool declarations for the caller to register with the invalidation map.
            // prism-spec-engine CANNOT call prism_query::invalidation::register_write_tool
            // directly (circular dependency: prism-query -> prism-spec-engine).
            // The registration is performed by prism-bin/src/boot.rs::plugin_load_step_with_audit
            // which has access to both crates (F-LP-IMPL-P1-002 / ADR-026 §D7 step 7.5).
            for manifest_tool in write_tools {
                pending_write_tool_registrations.push((plugin_name.clone(), manifest_tool));
            }
        }

        info!(
            n_loaded = n_loaded,
            pending_write_tool_registrations = pending_write_tool_registrations.len(),
            plugin_dir = %plugin_dir.display(),
            "boot: plugin-load step complete ({} plugins loaded, {} write tools pending registration)",
            n_loaded,
            pending_write_tool_registrations.len()
        );

        Ok((n_loaded, pending_write_tool_registrations))
    }

    /// Remove a plugin from the registry by plugin_id.
    ///
    /// Used by boot step 7.6 to roll back a plugin whose write-tool registration failed.
    /// A plugin with partial write-tool registration is in an inconsistent state — its
    /// read queries succeed but its write paths are silently broken. Unregistering it
    /// prevents stale reads after writes (BC-2.07.004 §write-then-read consistency).
    ///
    /// Returns `true` if the plugin was present and removed; `false` if it was not found.
    ///
    /// Implementation: loads the current `Arc<HashMap>`, clones the inner `HashMap`,
    /// removes the key, then stores the new `Arc`. This is a single-threaded pattern
    /// intended for use during boot (step 7.6 fail-closed rollback, per ADR-022 §B
    /// and BC-2.16.012 EC-016-012-004) before the query phase starts. At that point
    /// no query has yet been served, so the single-threaded assumption holds.
    ///
    /// NOT safe for concurrent callers in the query-phase steady state — that would
    /// require a `compare_and_swap` / `rcu` loop or a `Mutex` wrapper.
    ///
    /// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P4-002 | BC-2.07.004 | BC-2.16.012 EC-016-012-004
    pub fn unregister_plugin(&self, plugin_id: &str) -> bool {
        let current = self.registry.load();
        if !current.contains_key(plugin_id) {
            return false;
        }
        let mut new_registry = (**current).clone();
        new_registry.remove(plugin_id);
        self.registry.store(Arc::new(new_registry));
        true
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

    /// Dispatch the `acquire-token` WIT export on a sensor-auth plugin.
    ///
    /// Calls the plugin's exported `acquire-token` function, which reads `client_id`,
    /// `client_secret`, and `token_endpoint` from the host config map (PluginConfigMap),
    /// issues a POST to the OAuth2 token endpoint via `host::http-request`, caches the
    /// token in the plugin's KV store, and returns the bearer token string.
    ///
    /// For WAT-fixture plugins (core modules), this invokes the core-module export
    /// directly (the WAT fixture returns a hardcoded token for testing). For real
    /// Component Model plugins, this goes through the Component Model dispatch path.
    ///
    /// # Arguments
    ///
    /// - `plugin_id`: registry key for the sensor-auth plugin (e.g., `"crowdstrike-oauth2"`).
    /// - `config`: plugin config map (ADR-028 §D11 Option C) — MUST contain:
    ///   - `"client_id"` — resolved OAuth2 client ID (never an opaque handle)
    ///   - `"client_secret"` — resolved OAuth2 client secret (never an opaque handle)
    ///   - `"token_endpoint"` — full URL for POST /oauth2/token
    ///
    /// The caller (PluginAuthProvider::acquire_token) resolves credentials from
    /// prism_credentials before calling this function. Credentials are injected via
    /// PluginConfigMap and are NOT passed as WIT parameters (AD-017 compliance).
    ///
    /// # Errors
    ///
    /// Returns `PluginError::NotLoaded` if `plugin_id` is not in the registry.
    /// Returns `PluginError::AuthTokenNotCached` (E-PLUGIN-022) if the WASM call fails.
    ///
    /// Story: PLUGIN-MIGRATION-001-E / CRIT-1 + CRIT-2 (F-PR154-CRIT-1 + F-PR154-CRIT-2)
    /// Traces to: BC-2.01.016 §Postcondition; VP-150 end-to-end auth dispatch; ADR-028 §D11
    pub fn dispatch_plugin_acquire_token(
        &self,
        plugin_id: &str,
        config: &PluginConfigMap,
    ) -> Result<String, PluginError> {
        let plugin = self.get_plugin(plugin_id)?;
        // F-LP2-CRIT-001: clone the plugin's persistent kv_store Arc — do NOT construct a fresh
        // PluginKvStore::new() here. All dispatches for the same plugin share the same instance
        // so the token cache survives across calls (AC-004 "token cached within TTL").
        //
        // ADR-028 §D11 Option C: use the caller-provided config directly (contains client_id,
        // client_secret, token_endpoint). No internal PluginConfigMap construction here —
        // the caller (PluginAuthProvider::acquire_token) already resolved credentials.
        let host_state = self.make_host_state(
            plugin_id,
            config,
            plugin.kv_store.clone(),
            plugin.allowed_urls.clone(),
        );

        // Core module path (WAT fixtures — TEST-ONLY path; NEVER reachable from production load).
        //
        // F-LP8-LOW-001 closure: LOUDLY document test-only semantics. This branch is
        // structurally gated by `discovery.rs`: `core_module` is only `Some(...)` when
        // the binary's 4-byte magic header is a WAT core-module header (NOT Component Model
        // magic `[0x0d, 0x00, 0x01, 0x00]`). Production `.prx` files MUST be Component Model
        // binaries; WAT core-module bytes are only valid as test inputs.
        //
        // Emission reachability note: `plugin_auth_token_parse_error` (BC-2.16.002 row 37)
        // is emitted ONLY from the Component Model path below, NOT from this WAT path.
        // This is intentional — WAT fixture plugins short-circuit to a sentinel token return.
        // The unit test `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`
        // exercises the emission function directly without WAT infrastructure.
        //
        // Production invariant: `discovery::load_plugin_from_bytes` sets `core_module = None`
        // for any binary with Component Model magic — the only path into this arm requires
        // a WAT core-module binary, which is only provided by test helpers.
        if let Some(ref core_mod) = plugin.core_module {
            // F-LP9-HIGH-001 + F-LP8-LOW-001 re-closure (F-LP9-OBS-001: assert! not debug_assert!).
            // Production invariant: this branch is only reachable in test or test-helpers builds.
            // If reached in a production binary (without test-helpers feature), a .prx artifact
            // has a WAT core-module header instead of Component Model magic [0x0d,0x00,0x01,0x00],
            // indicating either a corrupted/tampered artifact or a programming error in plugin
            // construction.
            //
            // Implementation: use #[cfg] conditional compilation (not assert!(cfg!())) because
            // clippy::assertions_on_constants rejects assert!(cfg!(...)) — cfg!() is a compile-time
            // constant so clippy correctly identifies the assertion as trivially true or false.
            // Instead: the panic is compiled-in ONLY in non-test, non-test-helpers builds; in
            // test/test-helpers builds the block is not present (the WAT path is exercised by tests).
            // Integration test binaries compile library code with the test-helpers feature enabled
            // (self-referential dev-dependency in Cargo.toml), so the panic is absent there too.
            #[cfg(not(any(test, feature = "test-helpers")))]
            {
                // Suppress clippy::unused_variables: core_mod is used in the test-helpers path
                // below. In non-test builds this block panics, making the subsequent call_core_export
                // unreachable — but the binding is structurally necessary for symmetry.
                let _ = core_mod;
                panic!(
                    "core_module path is test-only; production plugins MUST be Component Model. \
                     Reaching this branch in a production binary (without test-helpers feature) \
                     indicates either: \
                     (a) a corrupted or tampered .prx artifact with WAT core-module magic bytes, or \
                     (b) a programming error constructing a core_module = Some(_) plugin outside \
                     test infrastructure. F-LP8-LOW-001 + F-LP9-HIGH-001 closure."
                );
            }
            // Core module WAT fixture: call "acquire-token" export.
            // The WAT fixture returns a hardcoded string (e.g., "crowdstrike-oauth2").
            // For real WASM Components, the Component Model dispatch path below runs.
            // Only reached in test/test-helpers builds — the non-test block above panics.
            // SAFETY: The panic!() above is only compiled in non-test builds. In test/test-helpers
            // builds the panic!() is absent, making this code reachable. In non-test builds
            // this code is unreachable by design — the panic fires before here.
            #[allow(unreachable_code)]
            self.call_core_export(
                plugin_id,
                core_mod,
                "acquire-token",
                DEFAULT_MEMORY_LIMIT_MB,
                DEFAULT_TIMEOUT_SECONDS,
            )?;

            // After core-module dispatch, read the cached token from the host KV store.
            // The WAT fixture's "acquire-token" doesn't actually call host::kv-set,
            // so we return a sentinel token for the WAT test path.
            // Production Component plugins write to KV via host::kv-set → HostState.kv_store.
            return Ok("wat-fixture-token".to_string());
        }

        // Component Model path: full Component with lifted WIT exports.
        let mut store = sandbox::create_store(
            &self.engine,
            host_state,
            DEFAULT_MEMORY_LIMIT_MB,
            DEFAULT_TIMEOUT_SECONDS,
        );

        let start = Instant::now();

        let instance = plugin.pre_instance.instantiate(&mut store).map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            sandbox::classify_wasm_error(
                plugin_id,
                e.into(),
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )
        })?;

        // Resolve the `acquire-token` export function.
        //
        // Core module WAT fixtures (test builds): exported as bare "acquire-token" at
        // the component's top level — `get_func(store, "acquire-token")` works.
        //
        // Real wasm-tools Component Model binaries (production .prx built with wit_bindgen):
        //   The component exports `sensor-auth` as an INSTANCE (WIT interface), not directly
        //   as individual functions. The interface is named "prism:{plugin_id}/sensor-auth@0.1.0"
        //   and contains the functions "acquire-token", "auth-type-name", "get-token".
        //
        //   Correct navigation per wasmtime Component Model API:
        //     1. Get the interface's export index (top-level instance export).
        //     2. Get the function's export index within that interface.
        //     3. Use that function index with get_func.
        //
        //   This pattern comes from the wasmtime docs example for nested instance exports:
        //     let instance_index = component.get_export_index(None, "interface-name")?;
        //     let func_index = component.get_export_index(Some(&instance_index), "fn-name")?;
        //     let func = instance.get_func(&mut store, &func_index)?;
        let func = {
            // Try 1: bare name (WAT core module test fixtures).
            let mut f = instance.get_func(&mut store, "acquire-token");

            // Try 2: Component Model nested export lookup.
            // Sensor-auth plugins export functions within a named interface instance.
            // Interface name format: "prism:{plugin_id}/sensor-auth@{version}".
            // Try the known production version first, then scan for other versions.
            if f.is_none() {
                let component = plugin.pre_instance.component();

                // Candidate interface names (most common first).
                // Version-agnostic: scan all component top-level exports for an interface
                // whose functions include "acquire-token".
                let interface_candidates: Vec<String> = {
                    let known = format!("prism:{plugin_id}/sensor-auth@0.1.0");
                    let mut candidates = vec![known];
                    // Also scan component exports for other sensor-auth interface names.
                    for (name, _) in component.component_type().exports(&self.engine) {
                        if name.contains("/sensor-auth@") && !candidates.contains(&name.to_string())
                        {
                            candidates.push(name.to_string());
                        }
                    }
                    candidates
                };

                'outer: for interface_name in &interface_candidates {
                    if let Some(iface_idx) =
                        component.get_export_index(None, interface_name.as_str())
                        && let Some(fn_idx) =
                            component.get_export_index(Some(&iface_idx), "acquire-token")
                        && let Some(found) = instance.get_func(&mut store, fn_idx)
                    {
                        tracing::debug!(
                            plugin_id = %plugin_id,
                            interface_name = %interface_name,
                            "dispatch_plugin_acquire_token: resolved via nested interface export"
                        );
                        f = Some(found);
                        break 'outer;
                    }
                }
            }

            f
        };

        let func = func.ok_or_else(|| PluginError::InvalidInterface {
            path: plugin_id.to_string(),
            missing_export: "acquire-token".to_string(),
        })?;

        // Component Model ABI (ADR-028 §D11 Option C, Path 4a — F-PR154-CRIT-1 closure):
        // acquire-token takes ZERO WIT params. Credentials are passed via PluginConfigMap
        // (HostState.config), which the guest reads via host::get-config("client_id") /
        // host::get-config("client_secret") / host::get-config("token_endpoint").
        //
        // Result type: result<string, auth-error> — lifted WIT result enum.
        // On success: guest calls host::kv-set("token", bearer_token); host reads from KV after return.
        // On error: guest returns err variant; KV has no "token" key → emit_acquire_token_parse_error_and_fail.
        let params: [wasmtime::component::Val; 0] = [];
        // Val::Result(Result<Option<Box<Val>>, Option<Box<Val>>>):
        // The variant payload is an unboxed Rust Result; the Ok/Err values are Box<Val>.
        let mut results = vec![wasmtime::component::Val::Result(Ok(Some(Box::new(
            wasmtime::component::Val::String(String::new()),
        ))))];

        let call_result = func.call(&mut store, &params, &mut results);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        match call_result {
            Ok(_) => {
                // Read the cached token from the host KV store after dispatch.
                let kv_store = store.data().kv_store.clone();
                match kv_store.get(plugin_id, "token") {
                    Some(token) => Ok(token),
                    None => emit_acquire_token_parse_error_and_fail(plugin_id),
                }
            }
            Err(e) => Err(sandbox::classify_wasm_error(
                plugin_id,
                e.into(),
                DEFAULT_MEMORY_LIMIT_MB,
                elapsed_ms,
                DEFAULT_TIMEOUT_SECONDS * 1000,
            )),
        }
    }

    /// Build a `HostState` for a new plugin call store.
    ///
    /// `kv_store` is the per-plugin persistent KV store from `LoadedPlugin.kv_store`
    /// (F-LP2-CRIT-001): callers MUST pass `plugin.kv_store.clone()` so that all
    /// dispatches for the same plugin share the SAME Arc<PluginKvStore> instance.
    /// This enables the token cache (host::kv-set / host::kv-get) to survive across
    /// separate dispatch calls, satisfying AC-004 "token cached within TTL".
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
        kv_store: Arc<PluginKvStore>,
        allowed_urls: Vec<String>,
    ) -> HostState {
        HostState {
            http_client: self.http_client.clone(),
            config: Arc::new(config.clone()),
            kv_store,
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
        // F-LP2-CRIT-001: clone the plugin's persistent kv_store Arc.
        let host_state = self.make_host_state(
            plugin_id,
            config,
            plugin.kv_store.clone(),
            plugin.allowed_urls.clone(),
        );
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

        // ADR-040 D2: two-phase function resolution.
        // Phase 1: try bare name (works for WAT test fixtures exported at component level).
        // Phase 2: scan for the interface instance export (real .prx Component Model binaries).
        let func = {
            let bare = instance.get_func(&mut store, "enrich-single");
            if bare.is_some() {
                bare
            } else {
                let component = plugin.pre_instance.component();
                let interface_candidates: Vec<String> = {
                    let known = "prism:infusion-plugin/infusion-plugin@0.1.0".to_string();
                    let mut candidates = vec![known];
                    for (name, _) in component.component_type().exports(&self.engine) {
                        if name.contains("/infusion-plugin@")
                            && !candidates.contains(&name.to_string())
                        {
                            candidates.push(name.to_string());
                        }
                    }
                    candidates
                };
                let mut found = None;
                'outer: for iface_name in &interface_candidates {
                    if let Some(iface_idx) = component.get_export_index(None, iface_name.as_str())
                        && let Some(fn_idx) =
                            component.get_export_index(Some(&iface_idx), "enrich-single")
                        && let Some(f) = instance.get_func(&mut store, fn_idx)
                    {
                        found = Some(f);
                        break 'outer;
                    }
                }
                found
            }
        };

        let func = func.ok_or_else(|| PluginError::InvalidInterface {
            path: plugin_id.to_string(),
            missing_export: "enrich-single".to_string(),
        })?;

        // ADR-040 D2: pass string arguments as Val::String (Component Model canonical ABI).
        // The wasmtime runtime lowers Val::String into the guest's linear memory automatically.
        // Do NOT use Val::S32 ptr/len pairs — the Component Model ABI does not use raw pointers.
        let params = [
            wasmtime::component::Val::String(input_value.to_string()),
            wasmtime::component::Val::String(input_type.to_string()),
        ];
        // Pre-populate results with Val::Option(None) to match the WIT return type `option<string>`.
        // wasmtime overwrites this with the actual return value from the guest.
        let mut results = vec![wasmtime::component::Val::Option(None)];

        let call_result = func.call(&mut store, &params, &mut results);
        // post_return removed — no longer needed in wasmtime >=44 (no-op, deprecated).

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // ADR-040 D2: lift the result from Val::Option per the WIT return type `option<string>`.
        match call_result {
            Ok(_) => match results.into_iter().next() {
                Some(wasmtime::component::Val::Option(Some(boxed_val))) => match *boxed_val {
                    wasmtime::component::Val::String(json_str) => {
                        match serde_json::from_str::<Value>(&json_str) {
                            Ok(v) => Ok(Some(v)),
                            Err(e) => {
                                error!(
                                    event_type = "plugin_enrich_json_parse_error",
                                    plugin_id = %plugin_id,
                                    error = %e,
                                    "plugin enrich-single returned non-JSON string"
                                );
                                Err(classify_enrich_call_failed(plugin_id, &e.to_string()))
                            }
                        }
                    }
                    other => {
                        error!(
                            event_type = "plugin_enrich_unexpected_val",
                            plugin_id = %plugin_id,
                            "plugin enrich-single returned unexpected Val type inside Option<Some>"
                        );
                        Err(classify_enrich_call_failed(
                            plugin_id,
                            &format!("unexpected Val inside Option::Some: {:?}", other),
                        ))
                    }
                },
                Some(wasmtime::component::Val::Option(None)) | None => {
                    // Plugin returned option::none — no enrichment data available.
                    Ok(None)
                }
                Some(other) => {
                    // Plugin returned wrong type entirely — protocol error (E-PLUGIN-023).
                    error!(
                        event_type = "plugin_enrich_unexpected_val",
                        plugin_id = %plugin_id,
                        "plugin enrich-single returned unexpected Val variant (expected option<string>)"
                    );
                    Err(classify_enrich_call_failed(
                        plugin_id,
                        &format!(
                            "unexpected result Val (expected option<string>): {:?}",
                            other
                        ),
                    ))
                }
            },
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
        // F-LP2-CRIT-001: clone the plugin's persistent kv_store Arc.
        let host_state = self.make_host_state(
            plugin_id,
            config,
            plugin.kv_store.clone(),
            plugin.allowed_urls.clone(),
        );
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
// Host-side acquire-token error emission helper (BC-2.16.002 row 37)
// ---------------------------------------------------------------------------

/// Emit the `plugin_auth_token_parse_error` audit event and return the appropriate error.
///
/// Called by `dispatch_plugin_acquire_token` when the Component Model guest's
/// `acquire-token` call completes (`func.call` returns `Ok`) but no token was
/// written to the KV store — the host-observable symptom of an `AuthError::ResponseParse`
/// in the guest (the guest failed to parse the token response and did not call `kv_set`).
///
/// ## Why a separate function?
///
/// Extracted for testability: unit tests can call this function directly to assert
/// the `plugin_auth_token_parse_error` emission fires. Tests that go through the full
/// `dispatch_plugin_acquire_token` Component Model path require a real `.prx` artifact
/// (not available in unit tests). This function is the load-bearing test target.
///
/// ## Architectural correctness
///
/// The emission is UNCONDITIONAL — no `#[cfg(test)]` gate — because the wasm32 guest
/// runs in a sandboxed wasmtime instance with NO tracing subscriber. Only the HOST
/// owns the tracing subscriber in production. This is the fix for the paper-fix detected
/// in FB-IMPL-6 (PLUGIN-MIGRATION-001-E pass-7 CORRECTION burst, 2026-05-23).
///
/// BC-2.16.002 Canonical Structured Event Catalog row 37.
/// F-LP7-MED-001 closure (CORRECTION).
pub(crate) fn emit_acquire_token_parse_error_and_fail(
    plugin_id: &str,
) -> Result<String, PluginError> {
    error!(
        event_type = "plugin_auth_token_parse_error",
        plugin_id = %plugin_id,
        error = "acquire-token dispatch completed but no token was cached in KV store \
                 (guest AuthError::ResponseParse or missing kv_set call)",
        "plugin auth token JSON parse failed"
    );
    // F-LP8-MED-002: return AuthTokenNotCached (E-PLUGIN-022) — NOT CompilationFailed
    // (E-PLUGIN-008). The WASM binary compiled and was dispatched successfully; this is a
    // runtime-behavioral failure (guest did not cache a token), not a compilation failure.
    // Using CompilationFailed caused operator triage confusion: `journalctl | grep E-PLUGIN-008`
    // would mix token-parse errors with real compilation failures.
    Err(PluginError::AuthTokenNotCached {
        plugin_id: plugin_id.to_string(),
        message: "acquire-token dispatch completed but no token was cached in KV store \
                  (guest AuthError::ResponseParse or missing kv_set call)"
            .to_string(),
    })
}

/// Build a `PluginError::EnrichCallFailed` (E-PLUGIN-023) for unexpected or unparseable
/// return values from `enrich-single`.
///
/// Called when the guest returns:
/// - A non-JSON string inside `Val::Option(Some(Val::String(...)))`
/// - An unexpected `Val` variant inside `Val::Option(Some(...))`
/// - An unexpected top-level `Val` variant (not `Val::Option`)
///
/// Mapped to `InfusionError::PluginCallFailed` (E-INFUSE-008) at the `plugin_bridge.rs`
/// boundary per ADR-040 D5.
fn classify_enrich_call_failed(plugin_id: &str, reason: &str) -> PluginError {
    PluginError::EnrichCallFailed {
        plugin_id: plugin_id.to_string(),
        reason: reason.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Manifest parsing helpers (BC-2.17.007)
// ---------------------------------------------------------------------------

/// A single write tool entry declared in a plugin manifest.
///
/// Plugin authors declare write tools in their `.manifest.toml` under the
/// `[[write_tools]]` array; each entry maps a tool name to the sensor source_ids
/// it invalidates and the sensor_id it belongs to.
///
/// Example TOML:
/// ```toml
/// [[write_tools]]
/// tool_name = "my_plugin_close_alert"
/// sensor_id = "my_sensor"
/// source_ids = ["my_sensor_alerts"]
/// ```
///
/// `sensor_id` is optional — if omitted, defaults to an empty string, meaning
/// "sensor-unscoped"; the `invalidate_for_sensor` lookup will not fire for that
/// sensor but `invalidate_for_write_tool` by tool_name still works.
///
/// Story: S-PLUGIN-PREREQ-E AC-9 / F-LP-IMPL-P1-002 | ADR-026 §D7 | BC-2.16.012
#[derive(Debug, serde::Deserialize, Clone)]
pub struct ManifestWriteTool {
    /// Write tool name (e.g., `"my_plugin_close_alert"`).
    pub tool_name: String,
    /// Sensor ID that owns this write tool (e.g., `"my_sensor"`).
    /// Defaults to empty string if omitted.
    #[serde(default)]
    pub sensor_id: String,
    /// source_id values to invalidate when this write tool fires.
    #[serde(default)]
    pub source_ids: Vec<String>,
}

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
    /// Optional write tool declarations for cache invalidation wiring.
    ///
    /// Absent or empty = no write tool registrations for this plugin.
    /// Each entry is registered via `prism_query::invalidation::register_write_tool`
    /// during boot step 7.5 (F-LP-IMPL-P1-002; S-PLUGIN-PREREQ-E AC-9; ADR-026 §D7).
    #[serde(default)]
    write_tools: Vec<ManifestWriteTool>,
}

/// Parse and validate a plugin manifest TOML string.
///
/// Validation order (BC-2.17.007 EC-17-032 first-failure-returns):
/// 1. `name` — non-empty string (E-PLUGIN-015)
/// 2. `version` — semver-parseable string (E-PLUGIN-016)
/// 3. `format_version` — `<= CURRENT_SUPPORTED_VERSION` (E-PLUGIN-014)
/// 4. `allowed_urls` — explicitly present (E-PLUGIN-013)
///
/// Parsed manifest fields: (name, version, format_version, allowed_urls, write_tools).
type ParsedManifestFields = (String, String, u32, Vec<String>, Vec<ManifestWriteTool>);

/// Returns `(name, version, format_version, allowed_urls, write_tools)` on success.
///
/// Returns appropriate `PluginError` variant on the first failing field.
fn parse_manifest(
    manifest_toml: Option<&str>,
    path: &str,
) -> Result<ParsedManifestFields, PluginError> {
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

    Ok((
        name,
        version_str,
        format_version,
        allowed_urls,
        manifest.write_tools,
    ))
}

// ---------------------------------------------------------------------------
// Unit tests for host-side plugin dispatch behavior (F-LP7-MED-001)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    /// F-LP7-MED-001 CORRECTION (unit test): `emit_acquire_token_parse_error_and_fail`
    /// emits `plugin_auth_token_parse_error` from the HOST and returns `Err`.
    ///
    /// This is the LOAD-BEARING test for BC-2.16.002 row 37 host-side emission.
    ///
    /// ## Strategy
    ///
    /// Rather than going through the full `dispatch_plugin_acquire_token` Component Model
    /// path (which requires a real `.prx` artifact — not available in unit tests), this
    /// test directly calls `emit_acquire_token_parse_error_and_fail`, the extracted helper
    /// that contains the emission. This is the canonical Rust pattern for testing private
    /// helper functions whose call site requires complex infrastructure.
    ///
    /// ## Load-bearing semantics
    ///
    /// - Removing the `error!` call from `emit_acquire_token_parse_error_and_fail` causes
    ///   assertion (a) to fail.
    /// - Removing `emit_acquire_token_parse_error_and_fail` from the `None` arm of the
    ///   `kv_store.get` match in `dispatch_plugin_acquire_token` causes the full
    ///   `dispatch_plugin_acquire_token` Component Model integration test to fail (when
    ///   un-ignored in CI with a real .prx artifact — story S-PLUGIN-CI-001).
    /// - Changing `emit_acquire_token_parse_error_and_fail` to use `#[cfg(test)]` (paper-fix
    ///   pattern) causes this test to pass but production emission would still be absent,
    ///   detectable by code inspection.
    ///
    /// BC-2.16.002 row 37 — host-side emission site verification.
    /// F-LP7-MED-001 CORRECTION burst (2026-05-23).
    #[test]
    fn test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally() {
        // Tracing capture: Arc<Mutex<Vec<u8>>> buffer.
        let captured: Arc<std::sync::Mutex<Vec<u8>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || {
                struct BufWriter(Arc<std::sync::Mutex<Vec<u8>>>);
                impl std::io::Write for BufWriter {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                        if let Ok(mut guard) = self.0.lock() {
                            guard.extend_from_slice(buf);
                        }
                        Ok(buf.len())
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                BufWriter(captured_clone.clone())
            })
            .with_ansi(false)
            .with_max_level(tracing::Level::ERROR)
            .finish();

        use tracing_subscriber::util::SubscriberInitExt;
        let _guard = subscriber.set_default();

        // Call the extracted helper DIRECTLY.
        // This is the production code path that fires when acquire-token dispatch
        // completes (func.call Ok) but kv_store.get returns None (no token cached).
        //
        // The helper is `pub(crate)` — accessible from tests within the same crate.
        // In production, this is called from the `None =>` arm of the kv_store.get match
        // inside `dispatch_plugin_acquire_token` (Component Model path).
        let result = emit_acquire_token_parse_error_and_fail("crowdstrike-oauth2");

        // Read captured output BEFORE dropping the guard.
        let output = captured.lock().expect("capture mutex not poisoned").clone();
        let output_str = String::from_utf8_lossy(&output);
        drop(_guard);

        // Assertion (c): function returns Err (never Ok).
        assert!(
            result.is_err(),
            "F-LP7-MED-001: emit_acquire_token_parse_error_and_fail MUST return Err; got Ok"
        );

        // Assertion (a): event_type = "plugin_auth_token_parse_error" present.
        // LOAD-BEARING: this assertion FAILS if `error!` is removed from the helper,
        // or if the helper is wrapped in #[cfg(test)] (paper-fix pattern).
        assert!(
            output_str.contains("plugin_auth_token_parse_error"),
            "F-LP7-MED-001 CORRECTION: emit_acquire_token_parse_error_and_fail MUST emit \
             event_type='plugin_auth_token_parse_error' UNCONDITIONALLY (no #[cfg(test)] gate). \
             This is the production audit emission for AuthError::ResponseParse on the guest. \
             Captured output: {output_str}"
        );

        // Assertion (b): plugin_id field present.
        assert!(
            output_str.contains("crowdstrike-oauth2"),
            "F-LP7-MED-001: emission MUST include plugin_id 'crowdstrike-oauth2'; \
             got: {output_str}"
        );

        // Assertion (d): F-LP8-MED-002 — function returns AuthTokenNotCached (not CompilationFailed).
        // LOAD-BEARING: this assertion FAILS until PluginError::AuthTokenNotCached variant is added
        // and emit_acquire_token_parse_error_and_fail is updated to return it.
        // The runtime-behavioral failure (token not cached after successful dispatch) MUST NOT
        // reuse CompilationFailed (E-PLUGIN-008: binary compilation failure — different semantics).
        match result {
            Err(PluginError::AuthTokenNotCached { ref plugin_id, .. }) => {
                assert_eq!(
                    plugin_id, "crowdstrike-oauth2",
                    "F-LP8-MED-002: AuthTokenNotCached plugin_id MUST match the input plugin_id"
                );
            }
            Err(other) => {
                panic!(
                    "F-LP8-MED-002: emit_acquire_token_parse_error_and_fail MUST return \
                     PluginError::AuthTokenNotCached, got: {:?}",
                    other
                );
            }
            Ok(_) => {
                panic!("F-LP8-MED-002: emit_acquire_token_parse_error_and_fail MUST return Err");
            }
        }
    }

    /// F-LP7-MED-001 CORRECTION (integration test #[ignore]):
    /// `dispatch_plugin_acquire_token` Component Model path emits `plugin_auth_token_parse_error`
    /// when the guest acquire-token succeeds but doesn't cache a token.
    ///
    /// ## Why #[ignore]
    ///
    /// This test requires a real `.prx` Component Model binary with proper WIT-lifted exports.
    /// The unit test (`test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`)
    /// covers the emission function directly. This integration test exercises the full
    /// `dispatch_plugin_acquire_token` Component Model path.
    ///
    /// Un-ignored when the `todo!()` body is replaced with a real implementation that loads
    /// the pre-built `.prx` and exercises the full Component Model dispatch path.
    ///
    /// BC-2.16.002 row 37 — end-to-end host-side emission integration test.
    /// F-LP7-MED-001 CORRECTION burst (2026-05-23).
    #[test]
    #[ignore = "todo!() body — integration test requires pre-built crowdstrike-oauth2.prx \
                and full dispatch_plugin_acquire_token Component Model path wiring; \
                unit test test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally \
                covers the load-bearing emission assertion in the interim"]
    fn test_F_LP7_MED_001_host_dispatch_acquire_token_component_model_path_emits_audit_event() {
        // This test will be implemented when the pre-built .prx is available.
        // The unit test above (emit_acquire_token_parse_error_fires_unconditionally)
        // covers the load-bearing assertion in the interim.
        todo!(
            "F-LP7-MED-001 integration test: load pre-built crowdstrike-oauth2.prx, \
             call dispatch_plugin_acquire_token with modified plugin (core_module=None), \
             assert plugin_auth_token_parse_error fires from host path"
        )
    }

    // ---------------------------------------------------------------------------
    // S-PERF-GATE-008 Red Gate tests (RG-001, RG-002 / AC-004, AC-005 / ADR-049 D3+D8)
    // ---------------------------------------------------------------------------

    /// RG-001: `apply_wasmtime_cache` degradable path does not panic on Err.
    ///
    /// Verifies that calling `apply_wasmtime_cache` with a forced-failure
    /// `Result::Err` (from `CacheConfig::with_directory("relative/not/absolute")`)
    /// returns normally — no panic, no `Err` propagation.
    ///
    /// Pre-implementation state: `todo!()` body panics → RED.
    /// Post-implementation state: degradable match arm returns normally → GREEN.
    ///
    /// SID-1: not `#[ignore]`'d; forced-failure uses `is_absolute()` check (pre-FS, deterministic).
    /// S-PERF-GATE-008 / AC-004 / ADR-049 D3.
    #[test]
    fn test_S_PERF_GATE_008_apply_wasmtime_cache_degradable_path_does_not_panic() {
        // S-PERF-GATE-008 / AC-004 / ADR-049 D3 / SID-1
        //
        // Forced-failure driver: CacheConfig::with_directory("relative/not/absolute")
        // triggers the is_absolute() check in CacheConfig::validate() BEFORE any
        // filesystem I/O, returning Err deterministically on all platforms.
        // Zero side effects — no temp dirs, no permission juggling, no external services.
        // Source: .factory/research/wasmtime-44-cache-api-S-PERF-GATE-008.md §2
        let mut cfg = wasmtime::CacheConfig::new();
        cfg.with_directory("relative/not/absolute");
        let err_result = wasmtime::Cache::new(cfg);

        // Pre-assert: the forced-failure driver produces a real wasmtime::Error (not a
        // synthetic injection). If this fails, the SID-1 test design is invalid.
        assert!(
            err_result.is_err(),
            "S-PERF-GATE-008 RG-001: forced-failure driver MUST return Err; \
             relative path 'relative/not/absolute' must fail is_absolute() check \
             in wasmtime 44 CacheConfig::validate_directory_or_default()"
        );

        // Exercise the degradable path.
        //
        // Red Gate: apply_wasmtime_cache has todo!() body — this call panics, test FAILS.
        // Post-implementation: the Err branch emits WARN and returns () — test PASSES.
        //
        // apply_wasmtime_cache returns () so reaching this point confirms the Err branch
        // did not panic, did not return Err, and did not abort construction (ADR-049 D3).
        let mut config = wasmtime::Config::new();
        apply_wasmtime_cache(&mut config, err_result);
        // Reaching here: degradable-path semantics confirmed — Err does not abort.
    }

    /// RG-002: `apply_wasmtime_cache` emits `plugin.compilation_cache_init_skipped` WARN on Err.
    ///
    /// Verifies that the `Err` branch of `apply_wasmtime_cache` fires
    /// `tracing::warn!(event_type = "plugin.compilation_cache_init_skipped", ...)`.
    /// Uses `Arc<Mutex<Vec<u8>>>` tracing capture pattern (established in
    /// `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`).
    ///
    /// Pre-implementation state: `todo!()` body panics → RED.
    /// Post-implementation state: WARN fires with correct event_type → GREEN.
    ///
    /// SID-1: not `#[ignore]`'d; SAP-1 load-bearing runtime assertion for BC-2.16.002 catalog row.
    /// S-PERF-GATE-008 / AC-005 / ADR-049 D8 / SAP-1.
    #[test]
    fn test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err() {
        // S-PERF-GATE-008 / AC-005 / ADR-049 D8 / SAP-1
        //
        // SAP-1 load-bearing: independently asserts at RUNTIME that the WARN fires
        // with the correct event_type value. Removes reliance on grep-only SAP-1 probes.
        //
        // Pattern: Arc<Mutex<Vec<u8>>> + BufWriter tracing capture, established in
        // test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally.

        // Set up tracing capture buffer.
        let captured: Arc<std::sync::Mutex<Vec<u8>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || {
                struct BufWriter(Arc<std::sync::Mutex<Vec<u8>>>);
                impl std::io::Write for BufWriter {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                        if let Ok(mut guard) = self.0.lock() {
                            guard.extend_from_slice(buf);
                        }
                        Ok(buf.len())
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                BufWriter(captured_clone.clone())
            })
            .with_ansi(false)
            // WARN level: capture WARN + ERROR (WARN alone would miss co-emitted ERROR).
            // Must NOT be ERROR-only — that would filter out the WARN emission under test.
            .with_max_level(tracing::Level::WARN)
            .finish();

        use tracing_subscriber::util::SubscriberInitExt;
        let _guard = subscriber.set_default();

        // Forced-failure driver (same as RG-001): relative path triggers is_absolute() pre-FS.
        let mut cfg = wasmtime::CacheConfig::new();
        cfg.with_directory("relative/not/absolute");
        let err_result = wasmtime::Cache::new(cfg);
        assert!(
            err_result.is_err(),
            "S-PERF-GATE-008 RG-002: forced-failure driver MUST return Err"
        );

        // Red Gate: apply_wasmtime_cache has todo!() body — panics, test FAILS.
        // Post-implementation: WARN fires with event_type field → captured, test PASSES.
        let mut config = wasmtime::Config::new();
        apply_wasmtime_cache(&mut config, err_result);

        // Read captured output BEFORE dropping the subscriber guard.
        let output = captured.lock().expect("capture mutex not poisoned").clone();
        let output_str = String::from_utf8_lossy(&output);
        drop(_guard);

        // LOAD-BEARING assertion (SAP-1 / BC-2.16.002 catalog row):
        // The WARN emission MUST carry event_type = "plugin.compilation_cache_init_skipped".
        // This assertion fails if:
        //   - apply_wasmtime_cache uses a different event_type value
        //   - the tracing::warn! is gated with #[cfg(test)] (paper-fix pattern — TD-VSDD-059)
        //   - the Err branch is missing the tracing::warn! call entirely
        assert!(
            output_str.contains("plugin.compilation_cache_init_skipped"),
            "S-PERF-GATE-008 RG-002 / SAP-1: apply_wasmtime_cache Err branch MUST emit \
             event_type='plugin.compilation_cache_init_skipped' UNCONDITIONALLY \
             (ADR-049 D8, BC-2.16.002 catalog row). \
             Captured output: {output_str}"
        );
    }
}
