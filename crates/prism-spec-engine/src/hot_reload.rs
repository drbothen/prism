// S-1.12: Filesystem watcher with debounce; triggers re-validation and arc-swap.
// BC-2.16.007: Sensor spec hot reload — add/remove/update sensor tables without restart.
// AD-018: notify crate (cross-platform filesystem events) or poll on configurable interval.

use std::path::PathBuf;
use std::sync::Arc;

use crate::config_manager::{compute_file_hash, extract_sensor_id_from_path, ConfigManager};
use crate::error::SpecEngineError;
use crate::types::{ModifiedSpec, ReloadResult, ReloadStatus, ValidationError};

/// Mechanism for filesystem change detection.
#[derive(Debug, Clone, Default)]
pub enum WatchMechanism {
    /// Use the `notify` crate for filesystem events (default)
    #[default]
    FsEvents,
    /// Poll the directory on the given interval (fallback)
    Poll { interval_ms: u64 },
}

/// Configuration for the hot-reload watcher (BC-2.16.007).
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Directory to watch for .sensor.toml changes
    pub spec_dir: PathBuf,
    /// Debounce duration in milliseconds (prevents burst re-reads)
    pub debounce_ms: u64,
    /// Watch mechanism — default is filesystem events
    pub mechanism: WatchMechanism,
}

/// Event emitted by the watcher when spec files change.
#[derive(Debug, Clone, PartialEq)]
pub enum SpecChangeEvent {
    /// A new .sensor.toml file appeared
    Added(PathBuf),
    /// An existing .sensor.toml file was removed
    Removed(PathBuf),
    /// An existing .sensor.toml file was modified
    Modified(PathBuf),
}

/// Hot reload watcher that monitors the spec directory and triggers
/// arc-swap publication on changes.
///
/// # Contract (BC-2.16.007)
/// - Detects add/remove/modify of *.sensor.toml files
/// - Debounces bursts of filesystem events
/// - On change: re-validates affected files, updates ConfigManager atomically
/// - In-flight queries using old snapshot are unaffected (DEC-037 via arc-swap guard)
pub struct HotReloadWatcher {
    #[allow(dead_code)]
    config: HotReloadConfig,
    #[allow(dead_code)]
    manager: Arc<ConfigManager>,
}

impl HotReloadWatcher {
    /// Create and start the watcher.
    /// Currently a stub — the test suite expects this to panic with "not yet implemented".
    pub fn start(
        _config: HotReloadConfig,
        _manager: Arc<ConfigManager>,
    ) -> Result<Self, SpecEngineError> {
        unimplemented!("S-1.12: HotReloadWatcher::start not yet implemented — Red Gate stub")
    }

    /// Stop the watcher and release filesystem handles.
    #[allow(dead_code)]
    pub fn stop(self) -> Result<(), SpecEngineError> {
        unimplemented!("S-1.12: HotReloadWatcher::stop not yet implemented — Red Gate stub")
    }
}

/// Process a batch of spec change events and update the ConfigManager.
/// Called by the watcher after debouncing.
///
/// # Contract (BC-2.16.007)
/// - Only actually-changed files (hash differs) trigger re-registration
/// - New tables registered; removed tables unregistered; modified tables re-registered
/// - Returns ReloadResult with added/removed/modified/unchanged entries
/// - If a modified spec fails validation, previous version remains active (DI-030)
pub fn process_spec_changes(
    events: Vec<SpecChangeEvent>,
    manager: &ConfigManager,
    _spec_dir: &std::path::Path,
) -> Result<ReloadResult, SpecEngineError> {
    let mut added: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut modified: Vec<ModifiedSpec> = Vec::new();
    let mut unchanged: Vec<String> = Vec::new();
    let mut validation_errors: Vec<ValidationError> = Vec::new();

    // Clone current snapshot so we can mutate and atomically swap
    let mut new_snapshot = {
        let guard = manager.load();
        (**guard).clone()
    };

    for event in events {
        match event {
            SpecChangeEvent::Added(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if !file_name.ends_with(".sensor.toml") {
                    continue;
                }

                let content =
                    std::fs::read_to_string(&path).map_err(|e| SpecEngineError::FileReadError {
                        path: path.to_string_lossy().to_string(),
                        os_error: e.to_string(),
                    })?;

                let file_hash = compute_file_hash(&content);

                match crate::add_sensor_spec::parse_and_validate_spec_toml(
                    &content,
                    &path.to_string_lossy(),
                ) {
                    Ok(mut spec) => {
                        spec.file_hash = file_hash;
                        spec.source_path = path.to_string_lossy().to_string();
                        for table in &spec.tables {
                            added.push(table.table_name.clone());
                        }
                        new_snapshot
                            .sensor_specs
                            .insert(spec.sensor_id.clone(), spec);
                    }
                    Err(errors) => {
                        let sensor_id = extract_sensor_id_from_path(&file_name);
                        validation_errors.extend(errors);
                        new_snapshot.failed_specs.insert(
                            sensor_id.clone(),
                            ValidationError {
                                sensor_id: Some(sensor_id),
                                source_path: path.to_string_lossy().to_string(),
                                errors: Vec::new(),
                            },
                        );
                    }
                }
            }

            SpecChangeEvent::Removed(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if !file_name.ends_with(".sensor.toml") {
                    continue;
                }
                let sensor_id = extract_sensor_id_from_path(&file_name);

                if let Some(old_spec) = new_snapshot.sensor_specs.remove(&sensor_id) {
                    for table in &old_spec.tables {
                        removed.push(table.table_name.clone());
                    }
                }
                new_snapshot.failed_specs.remove(&sensor_id);
            }

            SpecChangeEvent::Modified(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if !file_name.ends_with(".sensor.toml") {
                    continue;
                }

                // If file no longer exists on disk (e.g., deleted during a Modified event), skip
                if !path.exists() {
                    continue;
                }

                let content =
                    std::fs::read_to_string(&path).map_err(|e| SpecEngineError::FileReadError {
                        path: path.to_string_lossy().to_string(),
                        os_error: e.to_string(),
                    })?;

                let new_file_hash = compute_file_hash(&content);
                let sensor_id = extract_sensor_id_from_path(&file_name);

                // Hash-based unchanged detection
                let old_hash = new_snapshot
                    .sensor_specs
                    .get(&sensor_id)
                    .map(|s| s.file_hash.clone())
                    .unwrap_or_default();

                if old_hash == new_file_hash {
                    unchanged.push(sensor_id);
                    continue;
                }

                // Parse new version
                match crate::add_sensor_spec::parse_and_validate_spec_toml(
                    &content,
                    &path.to_string_lossy(),
                ) {
                    Ok(mut new_spec) => {
                        new_spec.file_hash = new_file_hash;
                        new_spec.source_path = path.to_string_lossy().to_string();

                        // Compare schema with old version
                        let schema_changed = new_snapshot
                            .sensor_specs
                            .get(&sensor_id)
                            .map(|old| old.tables != new_spec.tables)
                            .unwrap_or(true);

                        let table_names: Vec<String> = new_spec
                            .tables
                            .iter()
                            .map(|t| t.table_name.clone())
                            .collect();

                        modified.push(ModifiedSpec {
                            sensor_id: sensor_id.clone(),
                            table_names,
                            schema_changed,
                        });

                        new_snapshot.sensor_specs.insert(sensor_id, new_spec);
                    }
                    Err(errors) => {
                        // DI-030: validation failure retains previous version
                        validation_errors.extend(errors);
                        // Do NOT remove existing spec from new_snapshot
                    }
                }
            }
        }
    }

    // Recompute snapshot hash
    let mut file_hashes: Vec<(String, String)> = new_snapshot
        .sensor_specs
        .values()
        .map(|s| (s.source_path.clone(), s.file_hash.clone()))
        .collect();
    file_hashes.sort_by(|a, b| a.0.cmp(&b.0));
    new_snapshot.snapshot_hash =
        crate::config_manager::compute_snapshot_hash_from_hashes(&file_hashes);

    // Atomic swap
    manager.store(new_snapshot);

    let status = if validation_errors.is_empty() {
        ReloadStatus::Ok
    } else {
        ReloadStatus::PartialReload
    };

    Ok(ReloadResult {
        status,
        added,
        removed,
        modified,
        unchanged,
        validation_errors,
    })
}
