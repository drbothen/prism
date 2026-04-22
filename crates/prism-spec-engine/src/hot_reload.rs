// S-1.12: Filesystem watcher with debounce; triggers re-validation and arc-swap.
// BC-2.16.007: Sensor spec hot reload — add/remove/update sensor tables without restart.
// AD-018: notify crate (cross-platform filesystem events) or poll on configurable interval.
//
// STUB — implementation not yet written. Tests in hot_reload_tests.rs will fail
// until implementation exists (Red Gate).

use std::path::PathBuf;
use std::sync::Arc;

use crate::config_manager::ConfigManager;
use crate::error::SpecEngineError;
use crate::types::ReloadResult;

/// Mechanism for filesystem change detection.
#[derive(Debug, Clone)]
pub enum WatchMechanism {
    /// Use the `notify` crate for filesystem events (default)
    FsEvents,
    /// Poll the directory on the given interval (fallback)
    Poll { interval_ms: u64 },
}

impl Default for WatchMechanism {
    fn default() -> Self {
        WatchMechanism::FsEvents
    }
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

/// STUB: Hot reload watcher that monitors the spec directory and triggers
/// arc-swap publication on changes.
///
/// # Contract (BC-2.16.007)
/// - Detects add/remove/modify of *.sensor.toml files
/// - Debounces bursts of filesystem events
/// - On change: re-validates affected files, updates ConfigManager atomically
/// - New specs register tables; removed specs unregister tables; modified specs re-register
/// - In-flight queries using old snapshot are unaffected (DEC-037 via arc-swap guard)
/// - If a modified spec fails validation, previous version remains active (DI-030)
pub struct HotReloadWatcher {
    config: HotReloadConfig,
    manager: Arc<ConfigManager>,
}

impl HotReloadWatcher {
    /// STUB: Create and start the watcher.
    pub fn start(
        _config: HotReloadConfig,
        _manager: Arc<ConfigManager>,
    ) -> Result<Self, SpecEngineError> {
        unimplemented!("S-1.12: HotReloadWatcher::start not yet implemented — Red Gate stub")
    }

    /// STUB: Stop the watcher and release filesystem handles.
    pub fn stop(self) -> Result<(), SpecEngineError> {
        unimplemented!("S-1.12: HotReloadWatcher::stop not yet implemented — Red Gate stub")
    }
}

/// STUB: Process a batch of spec change events and update the ConfigManager.
/// Called by the watcher after debouncing.
///
/// # Contract (BC-2.16.007)
/// - Only actually-changed files (hash differs) trigger re-registration
/// - New tables registered; removed tables unregistered; modified tables re-registered
/// - Returns ReloadResult with added/removed/modified/unchanged entries
pub fn process_spec_changes(
    _events: Vec<SpecChangeEvent>,
    _manager: &ConfigManager,
    _spec_dir: &std::path::Path,
) -> Result<ReloadResult, SpecEngineError> {
    unimplemented!("S-1.12: process_spec_changes not yet implemented — Red Gate stub")
}
