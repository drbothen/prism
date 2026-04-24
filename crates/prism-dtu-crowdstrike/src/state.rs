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

/// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Auth mode: `"accept"` (default) or `"reject"` — toggles auth rejection.
    #[serde(default)]
    auth_mode: Option<String>,
    /// Seed for deterministic response ordering.
    #[serde(default)]
    seed: Option<u64>,
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
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    ///
    /// Route handlers check the `X-Admin-Token` request header against this value.
    pub admin_token: String,
}

impl CrowdstrikeState {
    /// Create a fresh state with empty stores and a 1,000-entry LRU registry.
    ///
    /// `admin_token` must be provided by the clone (generated via UUID v4 in `new()`).
    pub fn new() -> Self {
        // admin_token is seeded in clone.rs; state uses a placeholder here.
        // The clone overwrites it before starting via `with_admin_token`.
        Self::with_admin_token(uuid::Uuid::new_v4().to_string())
    }

    /// Create state with a specific admin token (used by the clone to share
    /// the token between the route handler and the BehavioralClone trait method).
    pub fn with_admin_token(admin_token: String) -> Self {
        // SAFETY: SESSION_REGISTRY_CAPACITY is a compile-time constant > 0; can never be zero.
        #[allow(clippy::expect_used)]
        let capacity = std::num::NonZeroUsize::new(SESSION_REGISTRY_CAPACITY)
            .expect("SESSION_REGISTRY_CAPACITY is non-zero");
        Self {
            containment_store: Mutex::new(HashMap::new()),
            detection_status_store: Mutex::new(HashMap::new()),
            session_registry: Mutex::new(LruCache::new(capacity)),
            runtime_config: Mutex::new(RuntimeConfig::default()),
            request_counter: Arc::new(AtomicU32::new(0)),
            admin_token,
        }
    }

    /// Increment the shared request counter and return the new count.
    pub fn next_request_count(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Clear all three stores — called by `CrowdstrikeClone::reset()`.
    pub fn reset(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.containment_store
            .lock()
            .expect("containment_store poisoned")
            .clear();
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        self.detection_status_store
            .lock()
            .expect("detection_status_store poisoned")
            .clear();
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        self.session_registry
            .lock()
            .expect("session_registry poisoned")
            .clear();
        // Runtime config is preserved across reset (reset clears data, not config).
    }

    /// Apply runtime configuration.
    ///
    /// Accepts JSON config such as `{"auth_mode": "reject"}`. Unknown fields
    /// are rejected with an error (TD-WV0-04: `deny_unknown_fields`).
    ///
    /// Recognised fields:
    /// - `"auth_mode"`: `"accept"` | `"reject"` — toggles auth rejection for all auth-required endpoints.
    /// - `"seed"`: u64 — seed for deterministic response ordering.
    ///
    /// # Spec decision: fidelity auth bypass
    ///
    /// The `FidelityValidator` sends probes without an `Authorization` header.
    /// To allow fidelity probes through: the DTU treats any non-empty bearer token
    /// as valid. For `auth_mode="reject"`, all auth-required endpoints return 401
    /// (including token endpoint). The auth check in route handlers reads
    /// `runtime_config.auth_reject` from state.
    pub fn apply_config(&self, config: &Value) -> Result<()> {
        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut rc = self.runtime_config.lock().expect("runtime_config poisoned");
        if let Some(auth_mode) = payload.auth_mode.as_deref() {
            rc.auth_reject = auth_mode == "reject";
        }
        if let Some(seed) = payload.seed {
            rc.seed = seed;
        }
        Ok(())
    }

    /// Read current `auth_reject` flag without holding the lock.
    pub fn is_auth_reject(&self) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
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
