//! Plugin hot reload — `notify` integration and atomic `arc-swap` registry updates.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use prism_core::PluginError;
use tracing::{error, info};

use super::discovery::load_plugin_from_bytes;
use super::loader::{HostState, LoadedPlugin};

/// Attempt to hot-reload a plugin by replacing its registry entry.
///
/// On success, swaps the registry entry atomically.
/// On failure, retains old entry and returns Err.
pub fn hot_reload(
    registry: &ArcSwap<HashMap<String, Arc<LoadedPlugin>>>,
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
    plugin_id: &str,
    path: &Path,
    new_bytes: &[u8],
) -> Result<(), PluginError> {
    // Attempt to compile and validate the new plugin binary.
    let new_plugin = match load_plugin_from_bytes(engine, linker, path, new_bytes) {
        Ok(p) => p,
        Err(err) => {
            error!(
                "Plugin '{}' hot-reload failed: {}. Previous version retained.",
                plugin_id, err
            );
            return Err(err);
        }
    };

    // Atomic swap using ArcSwap::rcu.
    let new_arc = Arc::new(new_plugin);
    let plugin_id_owned = plugin_id.to_string();

    registry.rcu(|current| {
        let mut updated = (**current).clone();
        updated.insert(plugin_id_owned.clone(), new_arc.clone());
        updated
    });

    info!(
        "Plugin '{}' hot-reloaded from '{}'",
        plugin_id,
        path.display()
    );
    Ok(())
}

/// Remove a plugin from the registry when its `.prx` file is deleted.
pub fn hot_unload(registry: &ArcSwap<HashMap<String, Arc<LoadedPlugin>>>, plugin_id: &str) {
    let plugin_id_owned = plugin_id.to_string();
    registry.rcu(|current| {
        let mut updated = (**current).clone();
        updated.remove(&plugin_id_owned);
        updated
    });

    info!("Plugin '{}' unloaded (file deleted)", plugin_id);
}

/// Start the file watcher for the plugins directory.
pub fn start_plugin_watcher(
    plugins_dir: &Path,
    registry: Arc<ArcSwap<HashMap<String, Arc<LoadedPlugin>>>>,
    engine: wasmtime::Engine,
    linker: wasmtime::component::Linker<HostState>,
) -> Result<notify::RecommendedWatcher, prism_core::PrismError> {
    use notify::{Event, EventKind, RecursiveMode, Watcher};

    let plugins_dir = plugins_dir.to_path_buf();

    let mut watcher =
        notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                for path in &event.paths {
                    if path.extension().and_then(|e| e.to_str()) != Some("prx") {
                        continue;
                    }

                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            let plugin_id = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            match std::fs::read(path) {
                                Ok(bytes) => {
                                    let _ = hot_reload(
                                        &registry, &engine, &linker, &plugin_id, path, &bytes,
                                    );
                                }
                                Err(e) => {
                                    error!("Plugin watcher: failed to read {:?}: {}", path, e);
                                }
                            }
                        }
                        EventKind::Remove(_) => {
                            let plugin_id = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            hot_unload(&registry, &plugin_id);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("Plugin watcher error: {}", e);
            }
        })
        .map_err(|e| prism_core::PrismError::Internal {
            detail: format!("notify watcher creation failed: {}", e),
        })?;

    watcher
        .watch(&plugins_dir, RecursiveMode::NonRecursive)
        .map_err(|e| prism_core::PrismError::Internal {
            detail: format!("notify watcher watch failed: {}", e),
        })?;

    Ok(watcher)
}
