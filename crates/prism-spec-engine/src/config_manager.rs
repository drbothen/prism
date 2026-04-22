// S-1.12: ConfigManager wrapping ArcSwap<ConfigSnapshot>.
// BC-2.16.006: Lock-free config reads on query hot path via ArcSwap.
// AD-018: ArcSwap<ConfigSnapshot> is the mandated config access pattern.
//
// STUB — implementation not yet written. Tests in hot_reload_tests.rs will fail
// until implementation exists (Red Gate).

use arc_swap::ArcSwap;
use std::sync::Arc;

use crate::types::ConfigSnapshot;

/// ConfigManager wraps ArcSwap<ConfigSnapshot> providing lock-free reads on the
/// query hot path and atomic swap for hot reload.
///
/// # Contract (BC-2.16.006)
/// - `load()` returns a Guard holding the current snapshot — lock-free, wait-free on x86_64
/// - `store()` atomically replaces the current snapshot — called only by reload_config
/// - In-flight readers hold a Guard for the query's full lifecycle; reloads do not affect them
/// - At most 2 ConfigSnapshot instances exist simultaneously
/// - ConfigSnapshot is immutable after construction (no interior mutability)
pub struct ConfigManager {
    inner: ArcSwap<ConfigSnapshot>,
}

impl ConfigManager {
    /// Create a new ConfigManager with the given initial snapshot.
    pub fn new(initial: ConfigSnapshot) -> Self {
        Self {
            inner: ArcSwap::from_pointee(initial),
        }
    }

    /// Create a new ConfigManager with an empty snapshot (for testing).
    pub fn empty() -> Self {
        Self::new(ConfigSnapshot::empty())
    }

    /// Lock-free load of the current config snapshot.
    /// Returns a Guard that holds a reference to the Arc<ConfigSnapshot>.
    /// The guard keeps the snapshot alive for the duration of the query.
    ///
    /// # Contract (BC-2.16.006)
    /// - No mutex or RwLock on this path
    /// - The guard's snapshot is stable for the caller's lifetime (DEC-037)
    pub fn load(&self) -> arc_swap::Guard<Arc<ConfigSnapshot>> {
        self.inner.load()
    }

    /// Atomically replace the current snapshot.
    /// Called only by reload_config (BC-2.16.005). O(1) operation.
    ///
    /// # Contract (BC-2.16.006)
    /// - `store()` is the sole write path
    /// - In-flight Guards are unaffected; they continue using the previous snapshot
    pub fn store(&self, new_snapshot: ConfigSnapshot) {
        self.inner.store(Arc::new(new_snapshot));
    }

    /// Return the current snapshot hash (for change detection).
    pub fn current_hash(&self) -> String {
        self.inner.load().snapshot_hash.clone()
    }
}

/// STUB: Parse a directory of .sensor.toml files into a ConfigSnapshot.
/// Implementation not yet written — will be provided by the implementer.
///
/// # Expected behavior (BC-2.16.005, BC-2.16.007)
/// - Reads all *.sensor.toml files from spec_dir
/// - Validates each file using the same pipeline as startup loading
/// - Computes per-file SHA-256 hashes and a combined snapshot hash
/// - Returns Ok(ConfigSnapshot) if at least Tier 1/2 config is valid
/// - Tier 3 (sensor spec) failures produce partial errors recorded in failed_specs
pub fn parse_spec_directory(
    _spec_dir: &std::path::Path,
) -> Result<ConfigSnapshot, crate::error::SpecEngineError> {
    unimplemented!("S-1.12: parse_spec_directory not yet implemented — Red Gate stub")
}

/// STUB: Compute SHA-256 hash of file contents for change detection.
pub fn compute_file_hash(_content: &str) -> String {
    unimplemented!("S-1.12: compute_file_hash not yet implemented — Red Gate stub")
}
