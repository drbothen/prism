//! `ClarotyState` — shared mutable state for the Claroty DTU server.
//!
//! Maintains a stateful device tag store (`device_id → {tag_keys}`),
//! request counter for rate-limit enforcement, and runtime failure mode.
//! All mutation is Mutex-guarded; reset restores base fixture state.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use prism_dtu_common::FailureMode;

/// Shared mutable state for the Claroty xDome DTU behavioral clone.
///
/// # Stateful behavior
/// - `tag_store`: Device tag write paths (AC-3, AC-4). Maps `device_id` to the
///   set of tag keys assigned to that device. Cleared on `reset()`.
/// - `request_counter`: Incremented per API request; used by FailureLayer for
///   rate-limit and internal-error injection (AC-6, AC-7).
/// - `failure_mode`: Current failure injection mode; updated via `/dtu/configure`.
pub struct ClarotyState {
    /// Maps device_uid → set of tag keys. Stateful across requests until reset.
    pub tag_store: Mutex<HashMap<String, HashSet<String>>>,
    /// Monotonically increasing request counter (1-indexed).
    pub request_counter: AtomicU32,
    /// Current failure injection mode for this clone.
    pub failure_mode: Mutex<FailureMode>,
}

impl ClarotyState {
    /// Create state with an empty tag store and no failure injection.
    pub fn new() -> Self {
        Self {
            tag_store: Mutex::new(HashMap::new()),
            request_counter: AtomicU32::new(0),
            failure_mode: Mutex::new(FailureMode::None),
        }
    }

    /// Reset all tag state and counters to initial values (base fixture state).
    ///
    /// After `reset()`, device queries return devices with empty `tags` arrays
    /// (AC-8). Request counter is zeroed; failure mode cleared to `None`.
    pub fn reset(&self) {
        unimplemented!("ClarotyState::reset")
    }

    /// Apply a new failure mode at runtime (called by `/dtu/configure` handler).
    pub fn apply_config(&self, _mode: FailureMode) {
        unimplemented!("ClarotyState::apply_config")
    }

    /// Increment the request counter and return its new 1-indexed value.
    pub fn increment_counter(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Add a tag key to the device's tag set.
    /// Returns `true` if the tag was newly inserted, `false` if already present.
    pub fn add_tag(&self, _device_id: &str, _tag_key: &str) -> bool {
        unimplemented!("ClarotyState::add_tag")
    }

    /// Remove a tag key from the device's tag set.
    /// Returns `true` if the tag existed and was removed, `false` if not found (EC-002).
    pub fn remove_tag(&self, _device_id: &str, _tag_key: &str) -> bool {
        unimplemented!("ClarotyState::remove_tag")
    }

    /// Return the set of tag keys for a given device_id. Empty set if unknown.
    pub fn get_tags(&self, device_id: &str) -> HashSet<String> {
        let store = self.tag_store.lock().expect("tag_store poisoned");
        store.get(device_id).cloned().unwrap_or_default()
    }
}

impl Default for ClarotyState {
    fn default() -> Self {
        Self::new()
    }
}
