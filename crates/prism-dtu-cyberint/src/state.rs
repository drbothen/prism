//! `CyberintState` — in-memory state for the Cyberint DTU behavioral clone.
//!
//! Maintains:
//! - Alert store: `alert_id → AlertStatus` (stateful, mutable)
//! - Session store: set of valid session token UUIDs (from POST /login)
//! - Immutable alert fixture registry (pre-loaded from `fixtures/alerts.json`)
//! - Runtime configuration (auth_mode, rate_limit_after)

#![allow(clippy::expect_used)]
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::types::{Alert, AlertStatus};

/// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Auth mode: `"accept"` (default) or `"reject"`.
    #[serde(default)]
    auth_mode: Option<String>,
    /// Number of authenticated requests before 429 rate-limit is triggered.
    #[serde(default)]
    rate_limit_after: Option<u32>,
}

/// Auth mode governing how cookie-based auth is handled.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Default: validate cookie from session_store.
    #[default]
    Accept,
    /// `auth_mode=reject`: all authenticated requests receive 401 regardless of cookie.
    Reject,
}

/// Shared mutable state for the Cyberint DTU clone.
///
/// `Arc<CyberintState>` is passed to every axum route handler via `axum::extract::State`.
pub struct CyberintState {
    /// Immutable alert registry pre-loaded from `fixtures/alerts.json` (plus page2).
    /// Used to seed `alert_store` on `reset()`.
    pub alert_fixture: Vec<Alert>,
    pub alert_fixture_page2: Vec<Alert>,

    /// Immutable threat fixture (loaded from `fixtures/threats.json`).
    pub threat_fixture: Vec<serde_json::Value>,

    /// `alert_id → AlertStatus` — mutable per-session status for each alert.
    /// Initialized from fixture on `new()` and restored on `reset()`.
    pub alert_store: Mutex<HashMap<String, AlertStatus>>,

    /// Valid session tokens (UUID strings) issued by `POST /login`.
    /// Cleared on `reset()`.
    pub session_store: Mutex<HashSet<String>>,

    /// Runtime auth mode — toggled via `POST /dtu/configure`.
    pub auth_mode: Mutex<AuthMode>,

    /// Optional rate-limit threshold — after N requests to authenticated routes,
    /// respond HTTP 429. `None` means no rate limiting.
    pub rate_limit_after: Mutex<Option<u32>>,

    /// Counter of authenticated requests since last reset.
    pub auth_request_count: Mutex<u32>,

    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,
}

impl CyberintState {
    /// Construct a fresh `CyberintState` from loaded fixtures.
    pub fn new(
        alert_fixture: Vec<Alert>,
        alert_fixture_page2: Vec<Alert>,
        threat_fixture: Vec<serde_json::Value>,
    ) -> Self {
        Self::with_admin_token(
            alert_fixture,
            alert_fixture_page2,
            threat_fixture,
            uuid::Uuid::new_v4().to_string(),
        )
    }

    /// Construct with a specific admin token.
    pub fn with_admin_token(
        alert_fixture: Vec<Alert>,
        alert_fixture_page2: Vec<Alert>,
        threat_fixture: Vec<serde_json::Value>,
        admin_token: String,
    ) -> Self {
        let alert_store = Self::build_alert_store(&alert_fixture, &alert_fixture_page2);
        Self {
            alert_fixture,
            alert_fixture_page2,
            threat_fixture,
            alert_store: Mutex::new(alert_store),
            session_store: Mutex::new(HashSet::new()),
            auth_mode: Mutex::new(AuthMode::default()),
            rate_limit_after: Mutex::new(None),
            auth_request_count: Mutex::new(0),
            admin_token,
        }
    }

    /// Build the initial alert store from fixture slices.
    fn build_alert_store(page1: &[Alert], page2: &[Alert]) -> HashMap<String, AlertStatus> {
        page1
            .iter()
            .chain(page2.iter())
            .map(|a| {
                (
                    a.alert_id.clone(),
                    AlertStatus {
                        alert_id: a.alert_id.clone(),
                        status: "open".to_owned(),
                        closed: false,
                    },
                )
            })
            .collect()
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Restores alert_store from fixture (all alerts back to "open", closed=false).
    /// - Clears session_store.
    /// - Resets auth_mode to Accept.
    /// - Resets rate_limit_after to None.
    /// - Resets auth_request_count to 0.
    pub fn reset(&self) {
        let store = Self::build_alert_store(&self.alert_fixture, &self.alert_fixture_page2);
        *self.alert_store.lock().expect("alert_store poisoned") = store;
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .clear();
        *self.auth_mode.lock().expect("auth_mode poisoned") = AuthMode::Accept;
        *self
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned") = None;
        *self
            .auth_request_count
            .lock()
            .expect("auth_request_count poisoned") = 0;
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Supported fields:
    /// - `"auth_mode"`: `"accept"` | `"reject"`
    /// - `"rate_limit_after"`: u32 — number of authenticated requests before 429
    ///
    /// Unknown fields are rejected with an error (TD-WV0-04: `deny_unknown_fields`).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        if let Some(mode_str) = payload.auth_mode.as_deref() {
            let mut mode = self.auth_mode.lock().expect("auth_mode poisoned");
            *mode = match mode_str {
                "reject" => AuthMode::Reject,
                _ => AuthMode::Accept,
            };
        }

        if let Some(n) = payload.rate_limit_after {
            let mut limit = self
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned");
            *limit = Some(n);
        }

        Ok(())
    }

    /// Check if a session token is valid.
    pub fn is_valid_session(&self, token: &str) -> bool {
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .contains(token)
    }

    /// Register a new session token.
    pub fn register_session(&self, token: String) {
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .insert(token);
    }

    /// Check and increment the authenticated request counter for rate limiting.
    /// Returns `true` if the request should be rate-limited (429).
    pub fn check_and_increment_rate_limit(&self) -> bool {
        let limit = *self
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        if let Some(threshold) = limit {
            let mut count = self
                .auth_request_count
                .lock()
                .expect("auth_request_count poisoned");
            *count += 1;
            *count > threshold
        } else {
            false
        }
    }

    /// Returns the current auth mode.
    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode.lock().expect("auth_mode poisoned").clone()
    }
}
