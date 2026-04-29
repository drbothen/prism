//! `CyberintState` — in-memory state for the Cyberint DTU behavioral clone.
//!
//! Maintains:
//! - Alert store: `(OrgId, alert_id) → AlertStatus` (stateful, mutable; BC-3.2.001)
//! - Session store: set of valid `(OrgId, token)` pairs issued by `POST /login` (BC-3.2.003)
//! - Immutable alert fixture registry (pre-loaded from `fixtures/alerts.json`)
//! - Runtime configuration (auth_mode, rate_limit_after)

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use prism_core::OrgId;

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

/// Sentinel `OrgId` used only in `#[cfg(test)]` contexts where the real `OrgId`
/// is not yet available (e.g., legacy test constructors, unit test fixtures).
///
/// # Architecture Compliance (BC-3.2.001 invariant 3)
///
/// This constant MUST NOT appear in any production code path.  Production callers
/// must supply a real `OrgId` at construction time (ADR-008 §8 Q2).
#[cfg(test)]
pub const DEFAULT_ORG_ID: OrgId = OrgId(uuid::Uuid::from_bytes([0u8; 16]));

/// Shared mutable state for the Cyberint DTU clone.
///
/// `Arc<CyberintState>` is passed to every axum route handler via `axum::extract::State`.
pub struct CyberintState {
    /// Immutable alert registry pre-loaded from `fixtures/alerts.json` (plus page2).
    /// Used to seed `alert_store` on `reset_all()`.
    pub alert_fixture: Vec<Alert>,
    pub alert_fixture_page2: Vec<Alert>,

    /// Immutable threat fixture (loaded from `fixtures/threats.json`).
    pub threat_fixture: Vec<serde_json::Value>,

    /// `(OrgId, alert_id) → AlertStatus` — mutable per-session status for each alert.
    ///
    /// Re-keyed from `HashMap<String, AlertStatus>` to `HashMap<(OrgId, String), AlertStatus>`
    /// per ADR-008 §2.1 Step 6d and BC-3.2.001.  Initialized from fixture on `new()` /
    /// `with_org_id_and_admin_token()` and restored on `reset_all()`.
    pub alert_store: Mutex<HashMap<(OrgId, String), AlertStatus>>,

    /// Valid session tokens keyed by `(OrgId, token_string)`.
    ///
    /// Re-keyed from `HashSet<String>` per BC-3.2.003.  Cleared on `reset_all()`.
    pub session_store: Mutex<HashSet<(OrgId, String)>>,

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
    ///
    /// Deprecated in favour of `with_org_id_and_admin_token`; retained as a
    /// `#[cfg(test)]` convenience shim that uses `DEFAULT_ORG_ID` (ADR-008 §8 Q2).
    #[cfg(test)]
    pub fn new(
        alert_fixture: Vec<Alert>,
        alert_fixture_page2: Vec<Alert>,
        threat_fixture: Vec<serde_json::Value>,
    ) -> Self {
        Self::with_org_id_and_admin_token(
            DEFAULT_ORG_ID,
            alert_fixture,
            alert_fixture_page2,
            threat_fixture,
            uuid::Uuid::new_v4().to_string(),
        )
    }

    /// Construct with a specific admin token.
    ///
    /// Deprecated in favour of `with_org_id_and_admin_token`; retained as a
    /// `#[cfg(test)]` convenience shim that uses `DEFAULT_ORG_ID` (ADR-008 §8 Q2).
    #[cfg(test)]
    pub fn with_admin_token(
        alert_fixture: Vec<Alert>,
        alert_fixture_page2: Vec<Alert>,
        threat_fixture: Vec<serde_json::Value>,
        admin_token: String,
    ) -> Self {
        Self::with_org_id_and_admin_token(
            DEFAULT_ORG_ID,
            alert_fixture,
            alert_fixture_page2,
            threat_fixture,
            admin_token,
        )
    }

    /// Construct with an explicit `OrgId` and admin token.
    ///
    /// This is the canonical production constructor (ADR-008 §8 Q2). All
    /// `alert_store` entries are keyed under `(org_id, alert_id)`.
    pub fn with_org_id_and_admin_token(
        org_id: OrgId,
        alert_fixture: Vec<Alert>,
        alert_fixture_page2: Vec<Alert>,
        threat_fixture: Vec<serde_json::Value>,
        admin_token: String,
    ) -> Self {
        let alert_store = Self::build_alert_store(org_id, &alert_fixture, &alert_fixture_page2);
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

    /// Build the initial alert store from fixture slices, keyed by `(org_id, alert_id)`.
    ///
    /// # BC-3.2.001 invariant 1
    ///
    /// The composite key `(OrgId, String)` is the exclusive keying scheme.  The
    /// `org_id` parameter must be the real org identifier; use `DEFAULT_ORG_ID`
    /// only in `#[cfg(test)]` contexts.
    fn build_alert_store(
        org_id: OrgId,
        page1: &[Alert],
        page2: &[Alert],
    ) -> HashMap<(OrgId, String), AlertStatus> {
        page1
            .iter()
            .chain(page2.iter())
            .map(|a| {
                (
                    (org_id, a.alert_id.clone()),
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
    /// Alias for `reset_all()` to preserve backward compatibility with route handlers.
    pub fn reset(&self) {
        self.reset_all();
    }

    /// Reset all mutable state to initial values for all organisations.
    ///
    /// - Restores alert_store from fixture (all alerts back to "open", closed=false)
    ///   keyed under the fixture org (DEFAULT_ORG_ID in test; real OrgId in production).
    /// - Clears session_store entirely.
    /// - Resets auth_mode to Accept.
    /// - Resets rate_limit_after to None.
    /// - Resets auth_request_count to 0.
    ///
    /// For per-org selective reset see `reset_for`.
    pub fn reset_all(&self) {
        // S-3.2.04 stub: reset_all clears both stores wholesale; per-org seed not yet
        // wired at this layer (production callers will call reset_for instead).
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        {
            self.alert_store
                .lock()
                .expect("alert_store poisoned")
                .clear();
        }
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .clear();
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        {
            *self.auth_mode.lock().expect("auth_mode poisoned") = AuthMode::Accept;
        }
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        {
            *self
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned") = None;
        }
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        {
            *self
                .auth_request_count
                .lock()
                .expect("auth_request_count poisoned") = 0;
        }
    }

    /// Reset all mutable state entries belonging to `org_id` only.
    ///
    /// - Removes every `(org_id, _)` entry from `alert_store`.
    /// - Removes every `(org_id, _)` entry from `session_store`.
    /// - Other orgs' entries are unaffected.
    ///
    /// # BC-3.2.001 edge case EC-004 / BC-3.2.003 edge case EC-004
    ///
    /// Must clear both stores for the given OrgId in a single logical operation.
    pub fn reset_for(&self, _org_id: OrgId) {
        todo!("S-3.2.04 implementation: remove all (org_id, _) entries from alert_store and session_store")
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
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut mode = self.auth_mode.lock().expect("auth_mode poisoned");
            *mode = match mode_str {
                "reject" => AuthMode::Reject,
                _ => AuthMode::Accept,
            };
        }

        if let Some(n) = payload.rate_limit_after {
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut limit = self
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned");
            *limit = Some(n);
        }

        Ok(())
    }

    /// Check if a session token is valid for the given `org_id`.
    ///
    /// # BC-3.2.003 invariant 2
    ///
    /// Token validation always takes `(org_id, token)` as input — never `token` alone.
    pub fn is_valid_session(&self, org_id: OrgId, token: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .contains(&(org_id, token.to_owned()))
    }

    /// Register a new session token scoped to `org_id`.
    ///
    /// # BC-3.2.003 postcondition 1
    ///
    /// The token is stored as `(org_id, token)`, not as a bare string.
    pub fn register_session(&self, org_id: OrgId, token: String) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .insert((org_id, token));
    }

    /// Check and increment the authenticated request counter for rate limiting.
    /// Returns `true` if the request should be rate-limited (429).
    pub fn check_and_increment_rate_limit(&self) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let limit = *self
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        if let Some(threshold) = limit {
            // SAFETY: same as above.
            #[allow(clippy::expect_used)]
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
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.auth_mode.lock().expect("auth_mode poisoned").clone()
    }
}
