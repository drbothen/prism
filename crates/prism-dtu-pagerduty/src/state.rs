//! `PagerDutyState` — in-memory state for the PagerDuty Events API v2 DTU behavioral clone.
//!
//! Maintains:
//! - Incident registry: `dedup_key → IncidentRecord` — stateful incident lifecycle
//! - Auth mode: controls whether routing key validation rejects all requests
//! - Request counter: used by `FailureMode::RateLimit` for threshold tracking
//! - Failure mode: shared with `FailureLayer` for configurable error injection
//!
//! No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear here.
//! `PagerDutyState` is pure Rust — no Axum dependency for its public methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use prism_dtu_common::FailureMode;

/// Incident lifecycle status per PagerDuty Events API v2 spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    /// Incident is open and active.
    Triggered,
    /// Incident has been acknowledged.
    Acknowledged,
    /// Incident has been resolved (terminal state for this lifecycle instance).
    Resolved,
}

use serde::Serialize;

impl IncidentStatus {
    /// String representation for JSON serialization / introspection.
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentStatus::Triggered => "triggered",
            IncidentStatus::Acknowledged => "acknowledged",
            IncidentStatus::Resolved => "resolved",
        }
    }
}

/// A single incident record in the PagerDuty DTU incident registry.
#[derive(Debug, Clone)]
pub struct IncidentRecord {
    /// Deduplication key — identifies the incident across lifecycle transitions.
    pub dedup_key: String,
    /// Current lifecycle status.
    pub status: IncidentStatus,
    /// Severity at the time of the triggering event (e.g. `"critical"`).
    pub severity: String,
    /// Human-readable summary from the trigger payload.
    pub summary: String,
}

/// Validated configuration payload for `POST /dtu/configure`.
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Auth mode: `"reject"` causes all subsequent requests to return HTTP 403.
    #[serde(default)]
    auth_mode: Option<String>,
    /// Failure mode: `"none"`, `"rate_limit"`, `"malformed_response"`,
    /// `"auth_reject"`, `"internal_error"`, `"network_timeout"`.
    #[serde(default)]
    failure_mode: Option<String>,
    /// Companion for `"rate_limit"`: number of requests before triggering 429.
    #[serde(default)]
    after_n_requests: Option<u32>,
    /// Companion for `"rate_limit"`: seconds in `Retry-After` header.
    #[serde(default)]
    retry_after_secs: Option<u32>,
    /// Companion for `"internal_error"`: inject 500 at this 1-indexed request number.
    #[serde(default)]
    at_request_n: Option<u32>,
    /// Companion for `"network_timeout"`: milliseconds to delay.
    #[serde(default)]
    after_ms: Option<u64>,
}

/// Shared mutable state for the PagerDuty Events API v2 DTU clone.
///
/// `Arc<PagerDutyState>` is passed to every axum route handler via `axum::extract::State`.
pub struct PagerDutyState {
    // --- Mutable incident registry ---
    /// Incident registry: `dedup_key → IncidentRecord`.
    ///
    /// Populated by `trigger` actions; transitioned by `acknowledge` and `resolve`.
    /// Cleared by `reset()`.
    pub incident_registry: Mutex<HashMap<String, IncidentRecord>>,

    // --- Auth mode ---
    /// When `true`, all requests to vendor endpoints return HTTP 403.
    ///
    /// Set to `true` via `POST /dtu/configure {"auth_mode": "reject"}`.
    /// Reset to `false` by `POST /dtu/reset`.
    pub auth_reject: Mutex<bool>,

    // --- Failure mode ---
    /// Shared failure mode, read by `FailureLayerShared` on every request.
    ///
    /// Wrapped in `Arc` so `build_router()` can clone it into the tower layer
    /// while `apply_config()` can mutate it after the server starts.
    pub failure_mode: Arc<Mutex<FailureMode>>,

    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,
}

impl Default for PagerDutyState {
    fn default() -> Self {
        Self::new()
    }
}

impl PagerDutyState {
    /// Construct a fresh `PagerDutyState` with an auto-generated admin token.
    pub fn new() -> Self {
        Self::with_admin_token(uuid::Uuid::new_v4().to_string())
    }

    /// Construct with a specific admin token (used by clone to share between
    /// the route handler and BehavioralClone::admin_token()).
    pub fn with_admin_token(admin_token: String) -> Self {
        Self {
            incident_registry: Mutex::new(HashMap::new()),
            auth_reject: Mutex::new(false),
            failure_mode: Arc::new(Mutex::new(FailureMode::None)),
            admin_token,
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Clears the incident registry (all incidents removed).
    /// - Resets auth_reject to `false`.
    /// - Resets the failure mode to `FailureMode::None`.
    pub fn reset(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = self
            .incident_registry
            .lock()
            .expect("incident_registry poisoned");
        registry.clear();

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut auth = self.auth_reject.lock().expect("auth_reject poisoned");
        *auth = false;

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut mode = self.failure_mode.lock().expect("failure_mode poisoned");
        *mode = FailureMode::None;
    }

    /// Returns `true` if auth-reject mode is currently active.
    pub fn is_auth_reject(&self) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        *self.auth_reject.lock().expect("auth_reject poisoned")
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Recognised keys:
    /// - `"auth_mode"` — `"reject"` activates auth rejection; any other value deactivates it.
    /// - `"failure_mode"` — one of `"none"`, `"rate_limit"`, `"malformed_response"`,
    ///   `"auth_reject"`, `"internal_error"`, `"network_timeout"`.
    ///
    /// Unknown fields are rejected with an error (deny_unknown_fields).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        if let Some(mode_str) = payload.auth_mode.as_deref() {
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut auth = self.auth_reject.lock().expect("auth_reject poisoned");
            *auth = mode_str == "reject";
        }

        if let Some(mode_str) = payload.failure_mode.as_deref() {
            let new_mode = match mode_str {
                "none" => FailureMode::None,
                "rate_limit" => {
                    let after_n = payload.after_n_requests.unwrap_or(0);
                    let retry_after = payload.retry_after_secs.unwrap_or(60);
                    FailureMode::RateLimit {
                        after_n_requests: after_n,
                        retry_after_secs: retry_after,
                    }
                }
                "malformed_response" => FailureMode::MalformedResponse,
                "auth_reject" => FailureMode::AuthReject,
                "internal_error" => {
                    let at_n = payload.at_request_n.unwrap_or(1);
                    FailureMode::InternalError { at_request_n: at_n }
                }
                "network_timeout" => {
                    let after_ms = payload.after_ms.unwrap_or(5000);
                    FailureMode::NetworkTimeout { after_ms }
                }
                other => {
                    anyhow::bail!("unknown failure_mode: {other}");
                }
            };
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut guard = self
                .failure_mode
                .lock()
                .expect("PagerDutyState: failure_mode lock poisoned");
            *guard = new_mode;
        }
        Ok(())
    }

    /// Return all current incidents as a snapshot (for `GET /dtu/incidents`).
    pub fn incidents_snapshot(&self) -> Vec<IncidentRecord> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = self
            .incident_registry
            .lock()
            .expect("incident_registry poisoned");
        registry.values().cloned().collect()
    }
}
