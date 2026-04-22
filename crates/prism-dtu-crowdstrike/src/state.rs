//! `CrowdstrikeState` — shared mutable state for the CrowdStrike DTU server.
//!
//! Maintains:
//! - `containment_store`: device containment status (stateful write target)
//! - `detection_status_store`: detection workflow status (stateful write target)
//! - `session_registry`: LRU cache of session ID → registered IDs (two-step pagination)
//!
//! # Spec decision: AC-6 seed scope
//!
//! `ChaCha20Rng` (from `StubConfig::seed`) affects response ordering (the IDs in the
//! `resources` array are shuffled deterministically by seed) — NOT fixture content.
//! Static fixture data remains stable across seeds; only ordering is seed-influenced.
//! This satisfies both AC-6 (same seed → same response) and the `different_seeds`
//! sub-test (different seeds → different orderings → different responses).

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

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

/// Runtime-mutable server configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// When `true`, all auth-required endpoints return HTTP 401.
    /// Spec decision: `auth_mode="reject"` is handled here so the configure()
    /// call can toggle it at runtime without restarting the server.
    pub auth_reject: bool,
    /// Seed for deterministic response ordering (from `StubConfig`).
    pub seed: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            auth_reject: false,
            seed: 42,
        }
    }
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
    /// Runtime configuration (auth_mode, etc.) — updated by `configure()`.
    pub runtime_config: Mutex<RuntimeConfig>,
    /// Shared request counter for FailureLayer — counts ALL requests across ALL
    /// routes. Stored here (not in tower layer) so axum's per-route-group layer
    /// cloning does not reset it. See S-6.07 AC-4 note.
    pub request_counter: Arc<AtomicU32>,
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
            runtime_config: Mutex::new(RuntimeConfig::default()),
            request_counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Increment the shared request counter and return the new count.
    pub fn next_request_count(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
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
        // Runtime config is preserved across reset (reset clears data, not config).
    }

    /// Apply runtime configuration.
    ///
    /// Accepts JSON config such as `{"auth_mode": "reject"}`.
    ///
    /// # Spec decision: fidelity auth bypass
    ///
    /// The `FidelityValidator` sends probes without an `Authorization` header.
    /// To allow fidelity probes through: the DTU treats any non-empty bearer token
    /// as valid. For `auth_mode="reject"`, all auth-required endpoints return 401
    /// (including token endpoint). The auth check in route handlers reads
    /// `runtime_config.auth_reject` from state.
    pub fn apply_config(&self, config: &Value) -> Result<()> {
        let mut rc = self
            .runtime_config
            .lock()
            .expect("runtime_config poisoned");
        if let Some(auth_mode) = config.get("auth_mode").and_then(|v| v.as_str()) {
            rc.auth_reject = auth_mode == "reject";
        }
        if let Some(seed) = config.get("seed").and_then(|v| v.as_u64()) {
            rc.seed = seed;
        }
        Ok(())
    }

    /// Read current `auth_reject` flag without holding the lock.
    pub fn is_auth_reject(&self) -> bool {
        self.runtime_config
            .lock()
            .expect("runtime_config poisoned")
            .auth_reject
    }
}

impl Default for CrowdstrikeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared `Arc<CrowdstrikeState>` passed through axum extension.
pub type SharedState = Arc<CrowdstrikeState>;
