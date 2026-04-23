// S-1.12: ConfigManager wrapping ArcSwap<ConfigSnapshot>.
// BC-2.16.006: Lock-free config reads on query hot path via ArcSwap.
// AD-018: ArcSwap<ConfigSnapshot> is the mandated config access pattern.

use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::error::SpecEngineError;
use crate::types::{ConfigSnapshot, SensorSpec, ValidationError};

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

/// Parse a directory of .sensor.toml files into a ConfigSnapshot.
///
/// # Contract (BC-2.16.005, BC-2.16.007)
/// - Reads all *.sensor.toml files from spec_dir
/// - Validates each file using the same pipeline as startup loading
/// - Computes per-file SHA-256 hashes and a combined snapshot hash
/// - Returns Ok(ConfigSnapshot) always (partial failures recorded in failed_specs)
/// - Returns Err(FileReadError) only if the directory itself cannot be read
pub fn parse_spec_directory(spec_dir: &Path) -> Result<ConfigSnapshot, SpecEngineError> {
    let read_dir = std::fs::read_dir(spec_dir).map_err(|e| SpecEngineError::FileReadError {
        path: spec_dir.to_string_lossy().to_string(),
        os_error: e.to_string(),
    })?;

    let mut sensor_specs: HashMap<String, SensorSpec> = HashMap::new();
    let mut failed_specs: HashMap<String, ValidationError> = HashMap::new();
    let mut file_hashes: Vec<(String, String)> = Vec::new(); // (path, hash) for snapshot hash

    for entry in read_dir {
        let entry = entry.map_err(|e| SpecEngineError::FileReadError {
            path: spec_dir.to_string_lossy().to_string(),
            os_error: e.to_string(),
        })?;

        let path = entry.path();
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
        file_hashes.push((path.to_string_lossy().to_string(), file_hash.clone()));

        match crate::add_sensor_spec::parse_and_validate_spec_toml(
            &content,
            &path.to_string_lossy(),
        ) {
            Ok(mut spec) => {
                spec.file_hash = file_hash;
                spec.source_path = path.to_string_lossy().to_string();
                sensor_specs.insert(spec.sensor_id.clone(), spec);
            }
            Err(errors) => {
                let sensor_id = extract_sensor_id_from_path(&file_name);
                failed_specs.insert(
                    sensor_id.clone(),
                    ValidationError {
                        sensor_id: Some(sensor_id),
                        source_path: path.to_string_lossy().to_string(),
                        errors: errors.into_iter().flat_map(|e| e.errors).collect(),
                    },
                );
            }
        }
    }

    // Sort for deterministic hash
    file_hashes.sort_by(|a, b| a.0.cmp(&b.0));
    let snapshot_hash = compute_snapshot_hash_from_hashes(&file_hashes);

    Ok(ConfigSnapshot {
        sensor_specs,
        failed_specs,
        snapshot_hash,
    })
}

/// Extract sensor_id from file name like "vendor_a.sensor.toml" -> "vendor_a"
pub(crate) fn extract_sensor_id_from_path(file_name: &str) -> String {
    file_name
        .strip_suffix(".sensor.toml")
        .unwrap_or(file_name)
        .to_string()
}

/// Compute SHA-256 hash of file contents for change detection.
pub fn compute_file_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute combined snapshot hash from sorted (path, hash) pairs.
pub(crate) fn compute_snapshot_hash_from_hashes(file_hashes: &[(String, String)]) -> String {
    let mut hasher = Sha256::new();
    for (path, hash) in file_hashes {
        hasher.update(path.as_bytes());
        hasher.update(b":");
        hasher.update(hash.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}
