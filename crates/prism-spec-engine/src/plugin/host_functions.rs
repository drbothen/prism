//! Plugin host functions — the only interfaces available to WASM plugins.
//!
//! Links the following into the `Linker<HostState>`:
//! - `host::http_request(method, url, headers, body) -> HttpResponse`
//! - `host::log(level, message)`
//! - `host::get_config(key) -> Option<String>`
//! - `host::kv_get(key) -> Option<String>`
//! - `host::kv_set(key, value) -> Result<(), E-PLUGIN-003>`
//!
//! # Architecture Compliance
//! - WASI filesystem, network, process, and environment MUST NOT be linked.
//! - All outbound HTTP from plugins goes through `host::http_request` via `HostState.http_client`.
//! - URL allowlist is enforced per-request if `HostState.allowed_urls` is `Some`.
//! - All outbound HTTP calls are audit-logged: `(plugin_id, method, url, status, latency_ms)`.
//!
//! # Red Gate stubs (S-1.15)
//! All functions are `unimplemented!()`.

use super::loader::HostState;

/// HTTP response type returned to the WASM plugin from `host::http_request`.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Log level mirroring the `log-level` enum in the WIT `host` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Execute an HTTP request on behalf of a plugin via the host's `reqwest::Client`.
///
/// - Validates the URL against `HostState.allowed_urls` (if configured).
/// - Enforces a 10-second per-request timeout (separate from the per-call epoch limit).
/// - Audit-logs `(plugin_id, method, url, status, latency_ms)` at `INFO` level.
///
/// Returns HTTP 403 equivalent (status 403, empty body) if URL is not allowlisted.
/// Returns HTTP 408 equivalent (status 408) if the request times out.
pub fn host_http_request(
    state: &HostState,
    method: &str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
) -> HttpResponse {
    unimplemented!("S-1.15 Red Gate: host_http_request not yet implemented")
}

/// Forward a plugin log message to `tracing` at the appropriate level.
///
/// Prefix format: `"[plugin:{plugin_id}] {message}"`.
pub fn host_log(state: &HostState, level: LogLevel, message: &str) {
    unimplemented!("S-1.15 Red Gate: host_log not yet implemented")
}

/// Look up a key in the plugin's config map (`HostState.config`).
///
/// Returns `None` for unknown keys — never errors.
pub fn host_get_config(state: &HostState, key: &str) -> Option<String> {
    unimplemented!("S-1.15 Red Gate: host_get_config not yet implemented")
}

/// Get a value from the plugin's KV store.
///
/// KV stores are scoped per plugin: the underlying key is `"{plugin_id}:{key}"`.
/// Returns `None` if the key has not been set.
pub fn host_kv_get(state: &HostState, key: &str) -> Option<String> {
    unimplemented!("S-1.15 Red Gate: host_kv_get not yet implemented")
}

/// Set a value in the plugin's KV store.
///
/// Returns `E-PLUGIN-003` if the 1MB per-plugin KV limit is exceeded.
pub fn host_kv_set(
    state: &HostState,
    key: &str,
    value: &str,
) -> Result<(), prism_core::PluginError> {
    unimplemented!("S-1.15 Red Gate: host_kv_set not yet implemented")
}

/// Register all host functions into the `Linker<HostState>`.
///
/// This is called once during `PluginRuntime::build_linker()`. After this call,
/// the linker is ready to pre-instantiate any plugin component that uses only
/// the Prism host interface.
///
/// # Architecture Compliance
/// MUST NOT call any `wasmtime_wasi::add_to_linker_*` function — WASI MUST NOT
/// be added to plugin instances (BC-2.17.002 / VP-040).
pub fn register_host_functions(
    linker: &mut wasmtime::component::Linker<HostState>,
) -> Result<(), prism_core::PrismError> {
    unimplemented!("S-1.15 Red Gate: register_host_functions not yet implemented")
}
