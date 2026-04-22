//! `NvdState` — in-memory state for the NVD DTU behavioral clone.
//!
//! Maintains:
//! - Immutable CVE registry pre-loaded from `fixtures/cves.json`
//! - Per-CVE request counters (for cache-hit assertion via test API)
//! - Dual rate-limit buckets keyed by `apiKey` value (None = unauthenticated)
//! - Runtime configuration (auth_mode, failure injection)

use std::collections::HashMap;
use std::sync::Mutex;

use crate::types::{CveRecord, RateLimitBucket};

/// Auth mode governing how `apiKey` query parameters are handled.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Default: `apiKey` presence upgrades rate-limit bucket but is not validated.
    #[default]
    Accept,
    /// `auth_mode=reject`: any request bearing an `apiKey` receives HTTP 403.
    Reject,
}

/// Shared mutable state for the NVD DTU clone.
///
/// `Arc<NvdState>` is passed to every axum route handler via `axum::extract::State`.
pub struct NvdState {
    /// Immutable CVE registry keyed by normalized CVE ID (uppercase).
    /// Loaded once at construction from `fixtures/cves.json`.
    pub cve_registry: HashMap<String, CveRecord>,

    /// `cve_id → request count` — incremented on every single-CVE lookup.
    /// Used by the `GET /dtu/request-count/{cve_id}` test API.
    pub request_counters: Mutex<HashMap<String, u32>>,

    /// Rate-limit buckets keyed by `apiKey` value.
    /// `None` key = unauthenticated bucket (limit: 5/30s).
    /// Any `Some(key)` value shares the authenticated bucket (limit: 50/30s).
    pub rate_limit_buckets: Mutex<HashMap<Option<String>, RateLimitBucket>>,

    /// Runtime auth mode — toggled via `POST /dtu/configure`.
    pub auth_mode: Mutex<AuthMode>,
}

impl NvdState {
    /// Construct a fresh `NvdState` with the given CVE registry.
    pub fn new(cve_registry: HashMap<String, CveRecord>) -> Self {
        let mut buckets: HashMap<Option<String>, RateLimitBucket> = HashMap::new();
        buckets.insert(None, RateLimitBucket::unauthenticated());

        Self {
            cve_registry,
            request_counters: Mutex::new(HashMap::new()),
            rate_limit_buckets: Mutex::new(buckets),
            auth_mode: Mutex::new(AuthMode::default()),
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Clears all request counters.
    /// - Resets all rate-limit buckets (unauthenticated bucket re-seeded; authenticated
    ///   buckets removed — they are created lazily on first authenticated request).
    /// - Resets auth_mode to `Accept`.
    pub fn reset(&self) {
        todo!()
    }

    /// Look up a CVE by ID (case-insensitive; normalizes to uppercase).
    /// Increments the request counter for the resolved CVE ID.
    pub fn lookup_and_count(&self, cve_id: &str) -> Option<CveRecord> {
        todo!()
    }

    /// Return the request count for a given CVE ID (for test API).
    pub fn request_count_for(&self, cve_id: &str) -> u32 {
        todo!()
    }

    /// Check and consume one slot from the appropriate rate-limit bucket.
    ///
    /// Returns `Ok(())` if the request is within limits, or `Err(RateLimitError)`
    /// indicating which error response should be returned.
    pub fn check_rate_limit(&self, api_key: Option<&str>) -> Result<(), RateLimitError> {
        todo!()
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        todo!()
    }
}

/// Error variants returned by `check_rate_limit`.
#[derive(Debug)]
pub enum RateLimitError {
    /// Unauthenticated bucket exhausted — respond HTTP 403.
    UnauthenticatedExceeded,
    /// Authenticated bucket exhausted — respond HTTP 429.
    AuthenticatedExceeded,
    /// `auth_mode=reject` and a key was presented — respond HTTP 403.
    ApiKeyRejected,
}
