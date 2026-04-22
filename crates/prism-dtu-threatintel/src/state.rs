//! Shared server state: fixture registry and rate-limit counter.

use crate::types::FixtureKey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

/// Default fixture registry entries required by story spec.
fn default_registry() -> HashMap<String, FixtureKey> {
    let mut m = HashMap::new();
    m.insert("45.55.100.1".to_string(), FixtureKey::Malicious);
    m.insert("8.8.8.8".to_string(), FixtureKey::Benign);
    m.insert("0.0.0.0".to_string(), FixtureKey::Unknown);
    m.insert("evil.example.com".to_string(), FixtureKey::Malicious);
    m.insert("safe.example.com".to_string(), FixtureKey::Benign);
    m
}

/// Shared mutable state for the ThreatIntel DTU server.
pub struct ThreatIntelState {
    /// Maps lookup values (IP/domain/hash) to fixture keys.
    pub fixture_registry: Mutex<HashMap<String, FixtureKey>>,
    /// Incremented on each lookup request; used to enforce rate-limit threshold.
    pub request_counter: AtomicU32,
    /// Rate-limit threshold: when counter exceeds this value, return 429.
    pub rate_limit_after: Mutex<Option<u32>>,
}

impl ThreatIntelState {
    /// Create state with default fixture registry.
    pub fn new() -> Self {
        Self {
            fixture_registry: Mutex::new(default_registry()),
            request_counter: AtomicU32::new(0),
            rate_limit_after: Mutex::new(None),
        }
    }

    /// Reset counter to zero and restore default registry (removes custom entries).
    pub fn reset(&self) {
        self.request_counter.store(0, Ordering::SeqCst);
        let mut registry = self
            .fixture_registry
            .lock()
            .expect("fixture_registry poisoned");
        *registry = default_registry();
        let mut threshold = self
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        *threshold = None;
    }

    /// Increment the request counter and return its new value.
    pub fn increment_counter(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Check whether the current request count exceeds the rate-limit threshold.
    pub fn is_rate_limited(&self, current_count: u32) -> bool {
        let threshold = self
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        match *threshold {
            Some(n) => current_count > n,
            None => false,
        }
    }

    /// Look up the fixture key for a given value.
    pub fn lookup_fixture(&self, key: &str) -> Option<FixtureKey> {
        let registry = self
            .fixture_registry
            .lock()
            .expect("fixture_registry poisoned");
        registry.get(key).cloned()
    }
}

impl Default for ThreatIntelState {
    fn default() -> Self {
        Self::new()
    }
}
