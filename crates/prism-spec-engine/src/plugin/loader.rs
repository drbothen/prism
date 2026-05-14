//! Plugin loader — wasmtime Engine/Linker setup and `.prx` loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use prism_core::PluginError;
use reqwest::Client;

/// Per-plugin configuration map — string key/value pairs from `[plugin_config]` TOML.
pub type PluginConfigMap = HashMap<String, String>;

/// KV_SIZE_LIMIT: 1MB per plugin total in the KV store (E-PLUGIN-003).
const KV_SIZE_LIMIT_BYTES: usize = 1024 * 1024;

/// Simple per-plugin key-value store.
pub struct PluginKvStore {
    inner: std::sync::Mutex<HashMap<String, String>>,
}

impl Default for PluginKvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginKvStore {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Get a value scoped to `plugin_id`.
    pub fn get(&self, plugin_id: &str, key: &str) -> Option<String> {
        let scoped_key = format!("{}:{}", plugin_id, key);
        self.inner
            .lock()
            .expect("PluginKvStore lock poisoned")
            .get(&scoped_key)
            .cloned()
    }

    /// Set a value scoped to `plugin_id`.
    pub fn set(&self, plugin_id: &str, key: &str, value: &str) -> Result<(), PluginError> {
        let scoped_key = format!("{}:{}", plugin_id, key);
        let mut store = self.inner.lock().expect("PluginKvStore lock poisoned");

        let plugin_prefix = format!("{}:", plugin_id);
        let current_size: usize = store
            .iter()
            .filter(|(k, _)| k.starts_with(&plugin_prefix))
            .map(|(k, v)| k.len() + v.len())
            .sum();

        let new_entry_size = scoped_key.len() + value.len();
        let existing_size = store
            .get(&scoped_key)
            .map(|v| scoped_key.len() + v.len())
            .unwrap_or(0);
        let net_addition = new_entry_size.saturating_sub(existing_size);

        if current_size + net_addition > KV_SIZE_LIMIT_BYTES {
            return Err(PluginError::SandboxViolation {
                plugin_id: plugin_id.to_string(),
                url: format!("kv_store size limit (1MB) exceeded for plugin '{plugin_id}'"),
            });
        }

        store.insert(scoped_key, value.to_string());
        Ok(())
    }
}

/// Metadata for a registered plugin.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

/// A compiled and pre-instantiated plugin binary.
pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub component: wasmtime::component::Component,
    pub pre_instance: wasmtime::component::InstancePre<HostState>,
    /// Core WASM module, present when the `.prx` was a core module wrapped as a component.
    /// Used to call exports via the core module API (not the Component Model API).
    pub core_module: Option<wasmtime::Module>,
    /// Raw bytes of the original `.prx` file (used for core module re-instantiation).
    pub raw_bytes: Vec<u8>,
    /// Per-plugin HTTP allowlist parsed from the manifest (AC-7 / VP-PLUGIN-007).
    ///
    /// Empty Vec = default-deny (no outbound HTTP). Stored here so `enrich_single`,
    /// `enrich_batch`, and other callers can pass it to `make_host_state()`.
    pub allowed_urls: Vec<String>,
}

/// Thread-safe host state passed to every plugin invocation via `wasmtime::Store`.
///
/// The `limits` field is wired as the `ResourceLimiter` on the Store via
/// `Store::limiter()` in `create_store()`, enforcing the 64 MiB memory cap
/// (BC-2.17.003 / INV-PLUGIN-003) on every WASM linear memory grow operation.
///
/// Marked `#[non_exhaustive]` per project convention (CLAUDE.md) — external callers
/// must use `HostState::test_default()` (test-gated) and functional update syntax.
/// AC-17: `allowed_urls: Vec<String>` replaces `Option<Vec<String>>`; default-deny
/// semantics: `vec![]` = deny all outbound HTTP (VP-PLUGIN-007).
#[non_exhaustive]
pub struct HostState {
    pub http_client: Arc<Client>,
    pub config: Arc<PluginConfigMap>,
    pub kv_store: Arc<PluginKvStore>,
    pub plugin_id: String,
    /// Per-plugin HTTP allowlist (AC-7 / AC-17 / VP-PLUGIN-007).
    ///
    /// REQUIRED field (Vec<String>, not Option). Default-deny semantics:
    /// - `vec![]` (empty) → deny ALL outbound HTTP from this plugin (default).
    /// - `vec!["api.example.com"]` → allow only `api.example.com`.
    ///
    /// Populated from the plugin manifest `allowed_urls` field by `make_host_state()`.
    pub allowed_urls: Vec<String>,
    /// ResourceLimiter state — wired via `Store::limiter()` in `create_store()`.
    /// Default (no limit) until `create_store` configures it with `StoreLimitsBuilder`.
    pub limits: wasmtime::StoreLimits,
}

#[cfg(any(test, feature = "test-helpers"))]
impl HostState {
    /// Test-only constructor returning a `HostState` with safe defaults.
    ///
    /// Production callers use `make_host_state()` with explicit field values populated
    /// from the plugin manifest. This constructor is ONLY for tests and test-helpers.
    ///
    /// Defaults:
    /// - `http_client`: bare `reqwest::Client::new()` (no 30s timeout — tests own their client)
    /// - `config`: empty `PluginConfigMap`
    /// - `kv_store`: fresh `PluginKvStore`
    /// - `plugin_id`: `"test-plugin"`
    /// - `allowed_urls`: `vec![]` (empty list = default-deny under AC-7 Vec<String> contract;
    ///   tests that need allowlist enforcement must use `test_with_allowed_urls`)
    /// - `limits`: `StoreLimits::default()` (no limit, overwritten by `create_store`)
    ///
    /// Feature-gated: `#[cfg(any(test, feature = "test-helpers"))]` — same gate as
    /// `auth_provider.rs` test helpers per project convention.
    pub fn test_default() -> Self {
        HostState {
            http_client: Arc::new(Client::new()),
            config: Arc::new(PluginConfigMap::new()),
            kv_store: Arc::new(PluginKvStore::new()),
            plugin_id: "test-plugin".to_string(),
            allowed_urls: vec![], // empty list = default-deny under AC-7 Vec<String> contract
            limits: wasmtime::StoreLimits::default(),
        }
    }

    /// Test-only constructor with a specific `plugin_id` and default-deny `allowed_urls`.
    ///
    /// Use when the test needs to identify the plugin (e.g., for log assertions) but doesn't
    /// need a specific allowlist (default-deny behavior).
    pub fn test_with_plugin_id(plugin_id: &str) -> Self {
        HostState {
            plugin_id: plugin_id.to_string(),
            ..HostState::test_default()
        }
    }

    /// Test-only constructor with specific `plugin_id` and `allowed_urls`.
    ///
    /// Use when the test needs both a specific plugin identity and a non-empty allowlist.
    pub fn test_with_allowed_urls(plugin_id: &str, allowed_urls: Vec<String>) -> Self {
        HostState {
            plugin_id: plugin_id.to_string(),
            allowed_urls,
            ..HostState::test_default()
        }
    }

    /// Test-only constructor with a custom `http_client`, `plugin_id`, and `allowed_urls`.
    ///
    /// Use when the test needs to inject a mock HTTP client (e.g., wiremock).
    pub fn test_with_client(
        http_client: Arc<Client>,
        plugin_id: &str,
        allowed_urls: Vec<String>,
    ) -> Self {
        HostState {
            http_client,
            plugin_id: plugin_id.to_string(),
            allowed_urls,
            ..HostState::test_default()
        }
    }
}

/// Load a compiled `wasmtime::component::Component` from `.prx` bytes.
///
/// Tries Component::from_binary first (for Component Model binaries).
/// Falls back to wrapping a core module.
pub fn compile_component(
    engine: &wasmtime::Engine,
    path: &Path,
    bytes: &[u8],
) -> Result<wasmtime::component::Component, PluginError> {
    // Try as a Component Model binary first.
    if let Ok(component) = wasmtime::component::Component::from_binary(engine, bytes) {
        return Ok(component);
    }

    // Not a component — try wrapping core module.
    wrap_core_module_as_component(engine, path, bytes)
}

/// Wrap a core WASM module as a minimal component binary.
fn wrap_core_module_as_component(
    engine: &wasmtime::Engine,
    path: &Path,
    bytes: &[u8],
) -> Result<wasmtime::component::Component, PluginError> {
    let path_str = path.display().to_string();

    // Validate WASM magic.
    if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
        return Err(PluginError::CompilationFailed {
            path: path_str,
            message: "not a valid WASM binary (bad magic number)".to_string(),
        });
    }

    // Build a minimal Component Model binary that wraps our core module.
    let component_bytes = build_component_wrapper(bytes);

    wasmtime::component::Component::from_binary(engine, &component_bytes).map_err(|e| {
        PluginError::CompilationFailed {
            path: path_str,
            message: format!("component wrapping failed: {}", e),
        }
    })
}

/// Build a Component Model binary that wraps a core WASM module.
///
/// Component Model binary format:
///   Header: \0asm + version 0x0d 0x00 0x01 0x00
///   Section 1 (core:module): the raw core module bytes
///   Section 2 (core:instance): instantiate module 0 with no imports
fn build_component_wrapper(module_bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();

    // Component Model magic + version (layer 1 = component)
    out.extend_from_slice(b"\0asm");
    out.extend_from_slice(&[0x0d, 0x00, 0x01, 0x00]);

    // Section 1: core:module (embed the raw module bytes)
    out.push(1u8);
    write_leb128_u32(&mut out, module_bytes.len() as u32);
    out.extend_from_slice(module_bytes);

    // Section 2: core:instance (instantiate module 0, no imports)
    let mut inst = Vec::new();
    inst.push(1u8); // count = 1
    inst.push(0u8); // instantiate
    write_leb128_u32(&mut inst, 0); // module_idx = 0
    inst.push(0u8); // import count = 0

    out.push(2u8);
    write_leb128_u32(&mut out, inst.len() as u32);
    out.extend_from_slice(&inst);

    out
}

pub(crate) fn write_leb128_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            break;
        } else {
            out.push(byte | 0x80);
        }
    }
}

/// Pre-instantiate a compiled `Component` against the `Linker`.
pub fn pre_instantiate(
    linker: &wasmtime::component::Linker<HostState>,
    component: &wasmtime::component::Component,
    path: &Path,
) -> Result<wasmtime::component::InstancePre<HostState>, PluginError> {
    let path_str = path.display().to_string();
    linker.instantiate_pre(component).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("wasi") || msg.contains("import") {
            PluginError::SandboxViolation {
                plugin_id: path_str.clone(),
                url: format!("unsatisfied import (possible WASI): {}", msg),
            }
        } else {
            PluginError::CompilationFailed {
                path: path_str,
                message: msg,
            }
        }
    })
}
