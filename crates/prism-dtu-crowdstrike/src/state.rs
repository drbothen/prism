//! `CrowdstrikeState` — shared mutable state for the CrowdStrike DTU server.
//!
//! Maintains:
//! - `containment_store`: device containment status (stateful write target)
//! - `detection_status_store`: detection workflow status (stateful write target)
//! - `session_registry`: LRU cache of session ID → registered IDs (two-step pagination)

use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use lru::LruCache;
use serde_json::Value;

/// Maximum number of concurrent sessions held in the LRU registry.
const SESSION_REGISTRY_CAPACITY: usize = 1_000;

/// Containment status for a single device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainmentStatus {
    /// `"normal"` or `"contained"`
    pub status: String,
    /// ISO-8601 timestamp of the last status change.
    pub updated_at: String,
}

/// Per-session data stored in the LRU registry for two-step pagination.
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Detection IDs registered by Step-1 of a detection query.
    pub detection_ids: Vec<String>,
    /// Host IDs registered by Step-1 of a host query.
    pub host_ids: Vec<String>,
}

/// Shared mutable state for the CrowdStrike DTU.
///
/// All fields are `Mutex`-guarded; the struct is `Send + Sync`.
pub struct CrowdstrikeState {
    /// Maps `device_id → ContainmentStatus`.
    pub containment_store: Mutex<HashMap<String, ContainmentStatus>>,
    /// Maps `detection_id → status string`.
    pub detection_status_store: Mutex<HashMap<String, String>>,
    /// LRU cache keyed by `X-DTU-Session-Id` header value; max 1,000 entries.
    pub session_registry: Mutex<LruCache<String, SessionData>>,
}

impl CrowdstrikeState {
    /// Create a fresh state with empty stores and a 1,000-entry LRU registry.
    pub fn new() -> Self {
        let capacity = std::num::NonZeroUsize::new(SESSION_REGISTRY_CAPACITY)
            .expect("SESSION_REGISTRY_CAPACITY is non-zero");
        Self {
            containment_store: Mutex::new(HashMap::new()),
            detection_status_store: Mutex::new(HashMap::new()),
            session_registry: Mutex::new(LruCache::new(capacity)),
        }
    }

    /// Clear all three stores — called by `CrowdstrikeClone::reset()`.
    pub fn reset(&self) {
        self.containment_store
            .lock()
            .expect("containment_store poisoned")
            .clear();
        self.detection_status_store
            .lock()
            .expect("detection_status_store poisoned")
            .clear();
        self.session_registry
            .lock()
            .expect("session_registry poisoned")
            .clear();
    }

    /// Apply runtime configuration — placeholder for FailureLayer wiring.
    ///
    /// Accepts JSON config such as `{"auth_mode": "reject"}` or failure-injection params.
    pub fn apply_config(&self, _config: &Value) -> Result<()> {
        unimplemented!("CrowdstrikeState::apply_config — not yet implemented")
    }
}

impl Default for CrowdstrikeState {
    fn default() -> Self {
        Self::new()
    }
}
