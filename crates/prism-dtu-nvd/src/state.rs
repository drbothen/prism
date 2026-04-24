//! `NvdState` — in-memory state for the NVD DTU behavioral clone.
//!
//! Maintains:
//! - Immutable CVE registry pre-loaded from `fixtures/cves.json`
//! - Per-CVE request counters (for cache-hit assertion via test API)
//! - Dual rate-limit buckets keyed by `apiKey` value (None = unauthenticated)
//! - Runtime configuration (auth_mode, failure injection)

#![allow(clippy::expect_used)]
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

    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,
}

impl NvdState {
    /// Construct a fresh `NvdState` with the given CVE registry.
    pub fn new(cve_registry: HashMap<String, CveRecord>) -> Self {
        Self::with_admin_token(cve_registry, uuid::Uuid::new_v4().to_string())
    }

    /// Construct with a specific admin token.
    pub fn with_admin_token(cve_registry: HashMap<String, CveRecord>, admin_token: String) -> Self {
        let mut buckets: HashMap<Option<String>, RateLimitBucket> = HashMap::new();
        buckets.insert(None, RateLimitBucket::unauthenticated());

        Self {
            cve_registry,
            request_counters: Mutex::new(HashMap::new()),
            rate_limit_buckets: Mutex::new(buckets),
            auth_mode: Mutex::new(AuthMode::default()),
            admin_token,
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Clears all request counters.
    /// - Resets all rate-limit buckets (unauthenticated bucket re-seeded; authenticated
    ///   buckets removed — they are created lazily on first authenticated request).
    /// - Resets auth_mode to `Accept`.
    pub fn reset(&self) {
        let mut counters = self
            .request_counters
            .lock()
            .expect("request_counters poisoned");
        counters.clear();

        let mut buckets = self
            .rate_limit_buckets
            .lock()
            .expect("rate_limit_buckets poisoned");
        buckets.clear();
        buckets.insert(None, RateLimitBucket::unauthenticated());

        let mut mode = self.auth_mode.lock().expect("auth_mode poisoned");
        *mode = AuthMode::Accept;
    }

    /// Look up a CVE by ID (case-insensitive; normalizes to uppercase).
    /// Increments the request counter for the resolved CVE ID.
    pub fn lookup_and_count(&self, cve_id: &str) -> Option<CveRecord> {
        let normalized = cve_id.to_uppercase();
        let record = self.cve_registry.get(&normalized).cloned();

        // Always increment the counter, whether found or not is caller's concern;
        // only increment for actual lookups (caller decides to call this).
        let mut counters = self
            .request_counters
            .lock()
            .expect("request_counters poisoned");
        *counters.entry(normalized).or_insert(0) += 1;

        record
    }

    /// Return the request count for a given CVE ID (for test API).
    pub fn request_count_for(&self, cve_id: &str) -> u32 {
        let normalized = cve_id.to_uppercase();
        let counters = self
            .request_counters
            .lock()
            .expect("request_counters poisoned");
        *counters.get(&normalized).unwrap_or(&0)
    }

    /// Check and consume one slot from the appropriate rate-limit bucket.
    ///
    /// Returns `Ok(())` if the request is within limits, or `Err(RateLimitError)`
    /// indicating which error response should be returned.
    pub fn check_rate_limit(&self, api_key: Option<&str>) -> Result<(), RateLimitError> {
        // Check auth_mode first — if reject and api_key is Some, reject immediately.
        {
            let mode = self.auth_mode.lock().expect("auth_mode poisoned");
            if *mode == AuthMode::Reject && api_key.is_some() {
                return Err(RateLimitError::ApiKeyRejected);
            }
        }

        let mut buckets = self
            .rate_limit_buckets
            .lock()
            .expect("rate_limit_buckets poisoned");

        if let Some(key) = api_key {
            // Authenticated bucket — keyed by the specific key value.
            let bucket_key = Some(key.to_owned());
            let bucket = buckets
                .entry(bucket_key)
                .or_insert_with(RateLimitBucket::authenticated);

            // Reset window if 30 seconds have elapsed.
            if bucket.window_start.elapsed().as_secs() >= 30 {
                bucket.count = 0;
                bucket.window_start = std::time::Instant::now();
            }

            if bucket.count >= bucket.limit {
                return Err(RateLimitError::AuthenticatedExceeded);
            }
            bucket.count += 1;
        } else {
            // Unauthenticated bucket.
            let bucket = buckets
                .entry(None)
                .or_insert_with(RateLimitBucket::unauthenticated);

            // Reset window if 30 seconds have elapsed.
            if bucket.window_start.elapsed().as_secs() >= 30 {
                bucket.count = 0;
                bucket.window_start = std::time::Instant::now();
            }

            if bucket.count >= bucket.limit {
                return Err(RateLimitError::UnauthenticatedExceeded);
            }
            bucket.count += 1;
        }

        Ok(())
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Unknown fields are rejected with an error (TD-WV0-04: `deny_unknown_fields`).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        /// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
        ///
        /// Unknown fields are rejected by serde to prevent silent misconfiguration.
        #[derive(Debug, serde::Deserialize, Default)]
        #[serde(deny_unknown_fields)]
        struct ConfigPayload {
            /// Auth mode: `"accept"` (default) or `"reject"`.
            #[serde(default)]
            auth_mode: Option<String>,
            /// When `true`, pre-exhausts the authenticated rate-limit bucket (test helper).
            #[serde(default)]
            exhaust_authenticated_bucket: Option<bool>,
        }

        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        // Handle auth_mode field.
        if let Some(mode_str) = payload.auth_mode.as_deref() {
            let mut mode = self.auth_mode.lock().expect("auth_mode poisoned");
            *mode = match mode_str {
                "reject" => AuthMode::Reject,
                _ => AuthMode::Accept,
            };
        }

        // Handle exhaust_authenticated_bucket — pre-fills authenticated bucket to limit.
        if payload.exhaust_authenticated_bucket.unwrap_or(false) {
            let mut buckets = self
                .rate_limit_buckets
                .lock()
                .expect("rate_limit_buckets poisoned");
            // Use a fixed key "valid-key" as the exhausted authenticated bucket.
            let bucket = buckets
                .entry(Some("valid-key".to_owned()))
                .or_insert_with(RateLimitBucket::authenticated);
            bucket.count = bucket.limit;
        }

        Ok(())
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
