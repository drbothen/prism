//! Plugin loader — wasmtime Engine/Linker setup and `.prx` loading.
//!
//! # Red Gate stubs (S-1.15)
//! All functions are `unimplemented!()`. Tests in `plugin_tests.rs` cover:
//! - Loading a valid infusion `.prx` fixture → registered in runtime (AC-1)
//! - Loading a `.prx` missing required exports → `Err(InvalidInterface)` (AC-7)
//! - Compilation of garbage bytes → `Err(CompilationFailed)` (EC-17-008)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use prism_core::PluginError;
use reqwest::Client;

/// Per-plugin configuration map — string key/value pairs from `[plugin_config]` TOML.
pub type PluginConfigMap = HashMap<String, String>;

/// Simple per-plugin key-value store (backed by the `plugin_state` CF in production).
///
/// Scoped per plugin: key format `"{plugin_id}:{key}"`.
/// In tests, this is a simple in-memory `HashMap`.
pub struct PluginKvStore {
    // TODO(S-1.15 impl): replace with RocksDB-backed `CacheBackend` injection.
    inner: std::sync::Mutex<HashMap<String, Vec<u8>>>,
}

impl PluginKvStore {
    pub fn new() -> Self {
        unimplemented!("S-1.15 Red Gate: PluginKvStore::new not yet implemented")
    }

    /// Get a value scoped to `plugin_id`.
    pub fn get(&self, plugin_id: &str, key: &str) -> Option<String> {
        unimplemented!("S-1.15 Red Gate: PluginKvStore::get not yet implemented")
    }

    /// Set a value scoped to `plugin_id`. Returns `Err` if the 1MB per-plugin limit
    /// is exceeded (`E-PLUGIN-003`).
    pub fn set(
        &self,
        plugin_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), PluginError> {
        unimplemented!("S-1.15 Red Gate: PluginKvStore::set not yet implemented")
    }
}

/// Metadata for a registered plugin (duplicated here from mod.rs to avoid circular imports).
///
/// See `plugin::PluginMetadata` for the authoritative definition.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

/// A compiled and pre-instantiated plugin binary, ready for per-call instantiation.
///
/// `LoadedPlugin` is stored in the registry behind `ArcSwap`. Per-call instantiation
/// from `InstancePre` is fast (~1-10 microseconds).
pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub component: wasmtime::component::Component,
    pub pre_instance: wasmtime::component::InstancePre<HostState>,
}

/// Thread-safe host state passed to every plugin invocation via `wasmtime::Store`.
///
/// `HostState` is constructed once per plugin call (alongside the fresh `Store`)
/// and holds:
/// - An `Arc<reqwest::Client>` for `host::http_request` proxying
/// - The config map for this plugin call
/// - A reference to the shared `PluginKvStore`
///
/// Per architecture compliance: `wasmtime::Store` is created fresh per call — stores
/// are not thread-safe and MUST NOT be reused across async tasks.
pub struct HostState {
    /// Shared HTTP client for `host::http_request` proxying (BC-2.17.002).
    pub http_client: Arc<Client>,
    /// Per-plugin configuration from TOML `[plugin_config]` section.
    pub config: Arc<PluginConfigMap>,
    /// Shared per-plugin KV store (scoped by plugin_id at the key level).
    pub kv_store: Arc<PluginKvStore>,
    /// The plugin_id of the currently-executing plugin (for KV scoping and audit logging).
    pub plugin_id: String,
    /// Optional URL allowlist. `None` = open (all URLs allowed). `Some(set)` = only
    /// listed domains are allowed through `host::http_request`.
    pub allowed_urls: Option<Vec<String>>,
}

/// Load a compiled `wasmtime::component::Component` from `.prx` bytes.
///
/// This is CPU-intensive and MUST be called from `tokio::task::spawn_blocking`
/// during hot reload (see `hot_reload.rs`). At startup discovery, blocking is
/// acceptable since the tokio runtime is not yet serving requests.
///
/// # Errors
/// - `PluginError::CompilationFailed` if the bytes are not a valid WASM Component.
/// - `PluginError::InvalidInterface` if WIT validation fails after compilation.
pub fn compile_component(
    engine: &wasmtime::Engine,
    path: &Path,
    bytes: &[u8],
) -> Result<wasmtime::component::Component, PluginError> {
    unimplemented!("S-1.15 Red Gate: compile_component not yet implemented")
}

/// Pre-instantiate a compiled `Component` against the `Linker`.
///
/// `InstancePre<HostState>` is thread-safe and cheap to clone. It is stored in
/// `LoadedPlugin` and used to create fresh `Store`/instance pairs per plugin call.
///
/// # Errors
/// Returns `PluginError::CompilationFailed` (wrapping the wasmtime error) if
/// pre-instantiation fails (e.g., WASI imports not satisfied — the linker has no
/// WASI bindings, so WASI-importing components are rejected here).
pub fn pre_instantiate(
    linker: &wasmtime::component::Linker<HostState>,
    component: &wasmtime::component::Component,
    path: &Path,
) -> Result<wasmtime::component::InstancePre<HostState>, PluginError> {
    unimplemented!("S-1.15 Red Gate: pre_instantiate not yet implemented")
}
