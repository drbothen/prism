// S-1.12: ConfigManager wrapping ArcSwap<ConfigSnapshot>.
// BC-2.16.006: Lock-free config reads on query hot path via ArcSwap.
// AD-018: ArcSwap<ConfigSnapshot> is the mandated config access pattern.

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};

use crate::{
    error::SpecEngineError,
    spec_parser::SensorSpec,
    types::{ConfigSnapshot, ValidationError},
};

/// Callback invoked after every config snapshot swap (P1-03 interim mitigation).
type SwapListener = Box<dyn Fn() + Send + Sync>;

/// ConfigManager wraps ArcSwap<ConfigSnapshot> providing lock-free reads on the
/// query hot path and atomic swap for hot reload.
///
/// # Contract (BC-2.16.006)
/// - `load()` returns a Guard holding the current snapshot — lock-free, wait-free on x86_64
/// - `store()` atomically replaces the current snapshot — called only by reload_config
/// - In-flight readers hold a Guard for the query's full lifecycle; reloads do not affect them
/// - At most 2 ConfigSnapshot instances exist simultaneously
/// - ConfigSnapshot is immutable after construction (no interior mutability)
///
/// # Swap listeners (P1-03 interim mitigation)
/// `store()` is the sole snapshot write path — every production reload trigger
/// (`reload_config`, `add_sensor_spec`, the S-1.12-FOLLOWUP hot-reload watcher)
/// funnels through it. Listeners registered via [`register_swap_listener`]
/// (`ConfigManager::register_swap_listener`) are invoked synchronously AFTER
/// each swap, so cross-cutting reactions to a config change (e.g. the boot-wired
/// response-cache flush — review 2026-06-10 Item 1) are structurally on every
/// reload path. The listener lock is never touched by `load()` — the
/// BC-2.16.006 lock-free hot-path guarantee is unaffected.
pub struct ConfigManager {
    inner: ArcSwap<ConfigSnapshot>,
    /// Swap listeners invoked after every `store()` (P1-03).
    ///
    /// Registration happens at boot (single-threaded, before the MCP server
    /// starts); `store()` only takes the read lock. `RwLock` (not `Mutex`) so a
    /// panicking listener cannot poison the lock — read-guard panics do not
    /// poison an `RwLock`.
    swap_listeners: RwLock<Vec<SwapListener>>,
}

impl ConfigManager {
    /// Create a new ConfigManager with the given initial snapshot.
    pub fn new(initial: ConfigSnapshot) -> Self {
        Self {
            inner: ArcSwap::from_pointee(initial),
            swap_listeners: RwLock::new(Vec::new()),
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

    /// Atomically replace the current snapshot, then notify swap listeners.
    /// Called only by reload_config (BC-2.16.005). The swap itself is O(1).
    ///
    /// # Contract (BC-2.16.006)
    /// - `store()` is the sole write path
    /// - In-flight Guards are unaffected; they continue using the previous snapshot
    ///
    /// # Swap-before-notify ordering (P1-03)
    /// Listeners run AFTER the new snapshot is visible to readers. For the
    /// response-cache flush listener, the guarantee this provides is: the
    /// flush evicts every entry cached before the swap (pre-swap-epoch
    /// entries), and any query that STARTS after the swap loads the new
    /// snapshot. It does NOT guarantee that no old-snapshot-derived entry can
    /// be inserted after the flush — under the in-flight Guard semantics
    /// above (BC-2.16.006), a query that loaded its Guard before the swap
    /// completes under the OLD snapshot and may insert its result after the
    /// flush has run. Today that residual window is benign: fetch-path
    /// normalization is boot-frozen (`SpecDrivenSensorAdapter` holds an
    /// `Arc<ResolvedSensorSpec>` resolved at boot; `store()` swaps only the
    /// `ConfigSnapshot` and never rebuilds the `AdapterRegistry`), so a
    /// post-flush insert from an in-flight query is no staler than a fresh
    /// fetch. Full reload-effectiveness arrives with
    /// S-CACHE-SPEC-COMPLIANCE-001's normalize-on-read model.
    pub fn store(&self, new_snapshot: ConfigSnapshot) {
        self.inner.store(Arc::new(new_snapshot));
        self.notify_swap_listeners();
    }

    /// Register a callback invoked synchronously after every snapshot swap.
    ///
    /// P1-03 interim mitigation (review 2026-06-10 Item 1): the boot path
    /// registers the response-cache flush here so that every config hot-reload
    /// purges cached sensor responses whose normalization may have changed.
    /// Listeners must be cheap and must not call back into `store()`.
    ///
    /// Registration is expected at boot, before concurrent `store()` callers
    /// exist. A poisoned write lock (a registrant panicked mid-push) is
    /// recovered: `Vec::push` either completed or did not — the Vec is
    /// structurally valid either way, and dropping listeners would silently
    /// disable the cache-flush safety hook (fail-safe recovery: preserve the
    /// listener chain rather than abandon it on lock poison).
    pub fn register_swap_listener(&self, listener: SwapListener) {
        let mut guard = match self.swap_listeners.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // SAP-1 exemption: no event_type field — operational diagnostic for a
                // boot-time-only condition, not a catalog-tracked structured event.
                tracing::error!(
                    "swap-listener registry write lock poisoned — recovering; \
                     a previous registrant panicked mid-registration"
                );
                poisoned.into_inner()
            }
        };
        guard.push(listener);
    }

    /// Invoke all registered swap listeners (called by `store()` after the swap).
    ///
    /// Takes the read lock only — registration (the sole writer) happens at
    /// boot, so contention here is nil. A poisoned lock is recovered for the
    /// same fail-safe-recovery rationale as `register_swap_listener`: skipping
    /// listeners would skip the response-cache flush and serve stale
    /// normalization (the exact defect P1-03 mitigates).
    fn notify_swap_listeners(&self) {
        let guard = match self.swap_listeners.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // SAP-1 exemption: no event_type field — operational diagnostic, not a
                // catalog-tracked structured event.
                tracing::error!(
                    "swap-listener registry lock poisoned — recovering to run listeners; \
                     skipping would leave the response cache stale after a config swap"
                );
                poisoned.into_inner()
            }
        };
        for listener in guard.iter() {
            listener();
        }
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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    // -----------------------------------------------------------------------
    // P1-03 interim mitigation — config swap listeners
    // (BC-2.16.006 store() is the sole snapshot write path; every swap must
    //  notify registered listeners so the response cache can be flushed.)
    // -----------------------------------------------------------------------

    /// P1-03: a registered swap listener is invoked exactly once per `store()`.
    #[test]
    fn test_p1_03_swap_listener_invoked_on_store() {
        let manager = ConfigManager::empty();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_in_listener = Arc::clone(&calls);
        manager.register_swap_listener(Box::new(move || {
            calls_in_listener.fetch_add(1, Ordering::SeqCst);
        }));

        manager.store(ConfigSnapshot::empty());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "P1-03: swap listener must fire exactly once per store()"
        );

        manager.store(ConfigSnapshot::empty());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "P1-03: swap listener must fire on every store()"
        );
    }

    /// P1-03: construction and lock-free reads do NOT fire swap listeners —
    /// only the `store()` write path notifies (BC-2.16.006 hot-path purity).
    #[test]
    fn test_p1_03_swap_listener_not_invoked_on_construction_or_load() {
        let calls = Arc::new(AtomicUsize::new(0));

        let manager = ConfigManager::empty();
        let calls_in_listener = Arc::clone(&calls);
        manager.register_swap_listener(Box::new(move || {
            calls_in_listener.fetch_add(1, Ordering::SeqCst);
        }));

        // Reads must not notify.
        let _guard = manager.load();
        let _hash = manager.current_hash();

        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "P1-03: construction, load(), and current_hash() must not fire swap listeners"
        );
    }

    /// P1-03: all registered listeners are invoked on a swap (boot may register
    /// more than one — e.g., cache flush + future table re-registration).
    #[test]
    fn test_p1_03_multiple_swap_listeners_all_invoked() {
        let manager = ConfigManager::empty();
        let calls_a = Arc::new(AtomicUsize::new(0));
        let calls_b = Arc::new(AtomicUsize::new(0));

        let a = Arc::clone(&calls_a);
        manager.register_swap_listener(Box::new(move || {
            a.fetch_add(1, Ordering::SeqCst);
        }));
        let b = Arc::clone(&calls_b);
        manager.register_swap_listener(Box::new(move || {
            b.fetch_add(1, Ordering::SeqCst);
        }));

        manager.store(ConfigSnapshot::empty());

        assert_eq!(
            calls_a.load(Ordering::SeqCst),
            1,
            "P1-03: first listener must fire on store()"
        );
        assert_eq!(
            calls_b.load(Ordering::SeqCst),
            1,
            "P1-03: second listener must fire on store()"
        );
    }

    /// P1-03: the swapped snapshot is visible to readers BEFORE listeners run —
    /// a cache-flush listener must observe the new config, not the old one
    /// (flush-after-swap ordering prevents a re-populate race window where a
    /// query could re-cache under the retired spec after the flush).
    #[test]
    fn test_p1_03_listener_observes_new_snapshot() {
        let manager = Arc::new(ConfigManager::empty());
        let empty_hash = manager.current_hash();

        let observed_hash = Arc::new(std::sync::Mutex::new(String::new()));
        let manager_in_listener = Arc::clone(&manager);
        let observed_in_listener = Arc::clone(&observed_hash);
        manager.register_swap_listener(Box::new(move || {
            let hash = manager_in_listener.current_hash();
            *observed_in_listener.lock().expect("observer mutex") = hash;
        }));

        let mut snapshot = ConfigSnapshot::empty();
        snapshot.snapshot_hash = "deadbeef".to_string();
        manager.store(snapshot);

        let observed = observed_hash.lock().expect("observer mutex").clone();
        assert_eq!(
            observed, "deadbeef",
            "P1-03: listener must observe the NEW snapshot (swap-before-notify); \
             old hash was '{empty_hash}'"
        );
    }
}
