//! Plugin hot reload — `notify` integration and atomic `arc-swap` registry updates.
//!
//! Watches `{config_dir}/plugins/*.prx` via the `notify` crate (same watcher pattern
//! as sensor/infusion specs from S-1.12). On file create/modify/delete events:
//!
//! - **Create/Modify:** Compile new binary in `tokio::task::spawn_blocking`, validate
//!   WIT, swap registry entry via `ArcSwap`. In-flight calls using the old `Arc<LoadedPlugin>`
//!   complete normally (Arc ref-count keeps the old module alive).
//! - **Delete:** Remove plugin from registry. New calls return `E-PLUGIN-011`.
//! - **Failed compile:** Retain old plugin, log error. A working plugin is NEVER unloaded
//!   for a bad new version (CI-002 / BC-2.17.005 / VP-042).
//!
//! # Red Gate stubs (S-1.15)
//! All functions are `unimplemented!()`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use prism_core::PluginError;

use super::loader::{HostState, LoadedPlugin};

/// Attempt to hot-reload a plugin by replacing its registry entry.
///
/// This is the target function for VP-042 (proptest): given a valid plugin in the
/// registry, calling `hot_reload(id, invalid_bytes)` must leave the registry entry
/// unchanged when compilation or WIT validation fails.
///
/// # Success path
/// 1. Compile `new_bytes` in `spawn_blocking` (must be called from async context).
/// 2. Validate WIT interface (BC-2.17.006).
/// 3. Swap registry entry via `ArcSwap` atomically.
/// 4. Log `INFO "Plugin '{plugin_id}' hot-reloaded from '{path}'"`.
///
/// # Failure path (failed compile or failed WIT validation)
/// - Registry entry is NOT updated.
/// - Log `ERROR "Plugin '{plugin_id}' hot-reload failed: {error}. Previous version retained."`.
/// - Returns `Err(PluginError::CompilationFailed)` or `Err(PluginError::InvalidInterface)`.
///
/// # In-flight safety
/// Callers that hold `Arc<LoadedPlugin>` from before the swap will complete normally —
/// the old Arc is not dropped until all holders release it.
pub fn hot_reload(
    registry: &ArcSwap<HashMap<String, Arc<LoadedPlugin>>>,
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
    plugin_id: &str,
    path: &Path,
    new_bytes: &[u8],
) -> Result<(), PluginError> {
    unimplemented!("S-1.15 Red Gate: hot_reload not yet implemented")
}

/// Remove a plugin from the registry when its `.prx` file is deleted.
///
/// In-flight callers holding `Arc<LoadedPlugin>` complete normally. New calls
/// after removal return `Err(PluginError::NotLoaded { plugin_id })`.
pub fn hot_unload(
    registry: &ArcSwap<HashMap<String, Arc<LoadedPlugin>>>,
    plugin_id: &str,
) {
    unimplemented!("S-1.15 Red Gate: hot_unload not yet implemented")
}

/// Start the file watcher for the plugins directory.
///
/// Uses `notify::RecommendedWatcher` with a debounce to suppress rapid duplicate
/// events (EC-17-020: 3 rapid replacements → only final version triggers reload).
///
/// On `Create` or `Modify` events: calls `hot_reload`.
/// On `Remove` events: calls `hot_unload`.
///
/// Returns a `notify::RecommendedWatcher` handle — the caller must keep it alive.
pub fn start_plugin_watcher(
    plugins_dir: &Path,
    registry: Arc<ArcSwap<HashMap<String, Arc<LoadedPlugin>>>>,
    engine: wasmtime::Engine,
    linker: wasmtime::component::Linker<HostState>,
) -> Result<notify::RecommendedWatcher, prism_core::PrismError> {
    unimplemented!("S-1.15 Red Gate: start_plugin_watcher not yet implemented")
}
