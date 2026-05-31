//! `CyberintState` — in-memory state for the Cyberint DTU behavioral clone.
//!
//! Maintains:
//! - Alert store: `(OrgId, alert_id) → AlertStatus` (stateful, mutable; BC-3.2.001)
//! - Access-token allowlist: set of valid `access_token` strings (ADR-031 §D3-a; AC-004)
//! - Immutable alert fixture registry (pre-loaded from `fixtures/alerts.json`)
//! - Runtime configuration (auth_mode, rate_limit_after)

use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use prism_core::OrgId;

use crate::types::{Alert, AlertStatus};

/// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
///
/// AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001): `access_token` field added to allow
/// test harnesses to register the demo token via `POST /dtu/configure` without
/// restarting the DTU. The `apply_config` method calls `register_access_token` when
/// this field is present.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Auth mode: `"accept"` (default) or `"reject"`.
    #[serde(default)]
    auth_mode: Option<String>,
    /// Number of authenticated requests before 429 rate-limit is triggered.
    #[serde(default)]
    rate_limit_after: Option<u32>,
    /// Register an access_token value in the static allowlist.
    ///
    /// When present, the token is added to `access_token_allowlist` via
    /// `register_access_token()`. Multiple calls are additive (each registered token
    /// remains valid until `reset()` clears the allowlist).
    ///
    /// AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001); ADR-031 §D3-a rule 3.
    #[serde(default)]
    access_token: Option<String>,
}

/// Auth mode governing how cookie-based auth is handled.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Default: validate access_token cookie against allowlist.
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

/// Maximum number of entries permitted in the `access_token_allowlist`.
///
/// Bounds unbounded HashSet growth from repeated `/dtu/configure` calls
/// (SEC-002 / CWE-400). Tokens beyond this cap are silently ignored by
/// `register_access_token` — mirroring the production E-AUTH-006 contract
/// that also enforces a 4096-byte length bound per token.
///
/// The cap of 100 is sufficient for all test harness scenarios (each test
/// run registers at most a handful of demo tokens) while preventing runaway
/// memory growth from adversarial or accidental bulk registration.
pub const MAX_ALLOWLIST_SIZE: usize = 100;

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

    /// Static access-token allowlist for the Cyberint DTU.
    ///
    /// Replaces the session_store model: `access_token` values are registered at
    /// startup (or via `POST /dtu/configure`) rather than issued per-login.
    ///
    /// Auth validation checks the `access_token` cookie value against this set.
    ///
    /// ADR-031 §D3-a rule 3: session-store becomes static-auth registry.
    /// AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001).
    pub access_token_allowlist: Mutex<HashSet<String>>,

    /// Runtime auth mode — toggled via `POST /dtu/configure`.
    pub auth_mode: Mutex<AuthMode>,

    /// Optional rate-limit threshold — after N requests to authenticated routes,
    /// respond HTTP 429. `None` means no rate limiting.
    pub rate_limit_after: Mutex<Option<u32>>,

    /// Counter of authenticated requests since last reset.
    pub auth_request_count: Mutex<u32>,

    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,

    /// The `OrgId` this clone instance was created for (ADR-008 §2.1).
    ///
    /// Route handlers fall back to this when no `X-Prism-Org-Id` header is
    /// present — e.g., legacy tests that predate multi-tenancy headers.
    /// Production callers MUST supply the header; the fallback is a
    /// DTU-only convenience, not a production code path.
    pub instance_org_id: OrgId,
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
            access_token_allowlist: Mutex::new(HashSet::new()),
            auth_mode: Mutex::new(AuthMode::default()),
            rate_limit_after: Mutex::new(None),
            auth_request_count: Mutex::new(0),
            admin_token,
            instance_org_id: org_id,
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
    /// - Clears access_token_allowlist entirely (ADR-031 §D3-a; org-agnostic static token store).
    /// - Resets auth_mode to Accept.
    /// - Resets rate_limit_after to None.
    /// - Resets auth_request_count to 0.
    ///
    /// For per-org selective alert reset see `reset_for`. Note: `reset_for` does NOT
    /// clear the access_token_allowlist because tokens are org-agnostic (ADR-031 §D3-a rule 3).
    ///
    /// Callers of `reset_all` (i.e. `POST /dtu/reset` without `X-Prism-Org-Id`) are
    /// responsible for re-registering the demo token after this call if the clone must
    /// remain immediately usable (see `CyberintState::apply_config` and the DTU `post_reset` handler).
    pub fn reset_all(&self) {
        // Re-seed alert_store from fixtures under instance_org_id (BC-3.2.001 invariant 1).
        let fresh = Self::build_alert_store(
            self.instance_org_id,
            &self.alert_fixture,
            &self.alert_fixture_page2,
        );
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        {
            *self.alert_store.lock().expect("alert_store poisoned") = fresh;
        }
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        self.access_token_allowlist
            .lock()
            .expect("access_token_allowlist poisoned")
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
    /// - Does NOT clear `access_token_allowlist` — tokens are org-agnostic (ADR-031 §D3-a rule 3).
    ///   A per-org token selective clear would require extending the allowlist model with org keys.
    /// - Other orgs' alert entries are unaffected.
    ///
    /// # BC-3.2.001 edge case EC-004
    ///
    /// Clears alert state for the given OrgId in a single logical operation.
    pub fn reset_for(&self, org_id: OrgId) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.alert_store
            .lock()
            .expect("alert_store poisoned")
            .retain(|(key_org, _), _| *key_org != org_id);

        // NOTE: access_token_allowlist is org-agnostic (ADR-031 §D3-a rule 3).
        // Tokens are global to the clone instance; per-org selective clear is a no-op here.
        // If a per-org reset is needed in future, the allowlist model must be extended.
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Supported fields:
    /// - `"auth_mode"`: `"accept"` | `"reject"`
    /// - `"rate_limit_after"`: u32 — number of authenticated requests before 429
    /// - `"access_token"`: String — register an allowed access token (AC-004)
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

        if let Some(token) = payload.access_token {
            // AC-004: register the access_token in the static allowlist.
            // This allows test harnesses to provision the demo token at runtime
            // without restarting the DTU.
            self.register_access_token(token);
        }

        Ok(())
    }

    /// Check if an `access_token` cookie value is valid for this DTU clone.
    ///
    /// Validates the token against the static allowlist — no org-scoping needed because
    /// the real Cyberint API issues API keys at the account level, not per-org.
    ///
    /// # ADR-031 §D3-a rule 3 / AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
    pub fn is_valid_access_token(&self, token: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.access_token_allowlist
            .lock()
            .expect("access_token_allowlist poisoned")
            .contains(token)
    }

    /// Register an `access_token` value in the static allowlist.
    ///
    /// Called at startup (or via `/dtu/configure`) to provision the demo token.
    /// No UUID issuance — the token is provided externally (API key or demo value).
    ///
    /// # Validation (SEC-002 / CWE-20 / CWE-400)
    ///
    /// Mirrors the production E-AUTH-006 contract (StaticCookieAuthProvider) to
    /// maintain DTU↔production fidelity:
    ///
    /// - Tokens exceeding 4096 bytes are silently ignored.
    /// - Tokens containing ASCII control characters (0x00-0x1F, DEL 0x7F) are
    ///   silently ignored (defense against CWE-113 CRLF injection and CWE-93).
    /// - Once the allowlist reaches `MAX_ALLOWLIST_SIZE`, further inserts are
    ///   silently ignored (defense against CWE-400 unbounded growth).
    ///
    /// Silent-ignore is chosen over returning an error because the callers
    /// (`apply_config` / `with_demo_token`) do not propagate errors to HTTP
    /// responses for this path, and the DTU is not a production auth service.
    ///
    /// # ADR-031 §D3-a rule 3 / AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
    pub fn register_access_token(&self, token: String) {
        // SEC-002 / CWE-20: reject oversized tokens (mirror E-AUTH-006 4096-byte limit).
        if token.len() > 4096 {
            return;
        }
        // SEC-002 / CWE-93 / CWE-113: reject tokens containing ASCII control chars
        // (0x00-0x1F incl. CR/LF, and DEL 0x7F) — mirrors E-AUTH-006 CTL rejection.
        if token.bytes().any(|b| b < 0x21 || b == 0x7F) {
            return;
        }
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut allowlist = self
            .access_token_allowlist
            .lock()
            .expect("access_token_allowlist poisoned");
        // SEC-002 / CWE-400: cap allowlist size to prevent unbounded memory growth.
        if allowlist.len() >= MAX_ALLOWLIST_SIZE {
            return;
        }
        allowlist.insert(token);
    }

    /// Builder: register a demo access token and return `self`.
    ///
    /// Convenience for test harnesses that specify the allowed `access_token` at
    /// construction time without a separate `register_access_token()` / `configure()` call.
    ///
    /// Consuming builder pattern: `CyberintState::with_org_id_and_admin_token(...).with_demo_token(token)`
    ///
    /// # ADR-031 §D3-a / AC-004 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
    pub fn with_demo_token(self, demo_token: String) -> Self {
        // Route through register_access_token to apply SEC-002 validation
        // (length, CTL chars, size cap) without duplicating the logic here.
        self.register_access_token(demo_token);
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_state() -> CyberintState {
        CyberintState::new(vec![], vec![], vec![])
    }

    /// SEC-002 / CWE-20: register_access_token must reject tokens containing ASCII
    /// control characters (CTL: 0x00-0x1F, DEL 0x7F) or exceeding 4096 bytes.
    ///
    /// Mirrors the E-AUTH-006 validation contract in production StaticCookieAuthProvider
    /// (SEC-001). A DTU allowlist that accepts CTL-containing or oversized tokens would
    /// accept inputs that the production validator rejects, breaking DTU↔production fidelity.
    ///
    /// Verifies: register_access_token with invalid token does NOT grow the allowlist.
    /// ADR-031 §D3-a; SEC-002 (S-DTU-CYBERINT-AUTH-FIDELITY-001).
    #[test]
    fn test_register_access_token_rejects_ctl_and_oversized_tokens() {
        let state = empty_state();

        // ---- Case 1: CTL byte (0x01) ----
        let ctl_token = "abc\x01def".to_string();
        state.register_access_token(ctl_token.clone());
        assert!(
            !state.is_valid_access_token(&ctl_token),
            "SEC-002 / CWE-20: register_access_token must NOT accept tokens containing \
             ASCII control bytes (0x01 CTL). DTU allowlist accepted a CTL token."
        );

        // ---- Case 2: CRLF injection ----
        let crlf_token = "valid-prefix\r\nInjected: header".to_string();
        state.register_access_token(crlf_token.clone());
        assert!(
            !state.is_valid_access_token(&crlf_token),
            "SEC-002 / CWE-113: register_access_token must NOT accept tokens containing \
             CRLF (HTTP header injection). DTU allowlist accepted a CRLF token."
        );

        // ---- Case 3: oversized token (> 4096 bytes) ----
        let oversized_token = "a".repeat(4097);
        state.register_access_token(oversized_token.clone());
        assert!(
            !state.is_valid_access_token(&oversized_token),
            "SEC-002 / CWE-400: register_access_token must NOT accept tokens exceeding \
             4096 bytes. DTU allowlist accepted an oversized token."
        );

        // ---- Sanity: valid token still works ----
        let valid_token = "valid-demo-token-abc123".to_string();
        state.register_access_token(valid_token.clone());
        assert!(
            state.is_valid_access_token(&valid_token),
            "SEC-002 sanity: a valid token must still be accepted after implementing \
             the CTL/oversized rejection."
        );
    }

    /// SEC-002 / CWE-400: register_access_token must enforce MAX_ALLOWLIST_SIZE.
    ///
    /// Inserting more than MAX_ALLOWLIST_SIZE distinct valid tokens must not grow the
    /// allowlist past the cap. This bounds unbounded HashSet growth via /dtu/configure.
    ///
    /// ADR-031 §D3-a; SEC-002 (S-DTU-CYBERINT-AUTH-FIDELITY-001).
    #[test]
    fn test_register_access_token_caps_at_max_allowlist_size() {
        let state = empty_state();

        // Insert MAX_ALLOWLIST_SIZE + 10 distinct valid tokens.
        for i in 0..=(MAX_ALLOWLIST_SIZE + 10) {
            state.register_access_token(format!("valid-token-{i:04}"));
        }

        // SAFETY: mutex poison only occurs if a previous holder panicked.
        #[allow(clippy::expect_used)]
        let count = state
            .access_token_allowlist
            .lock()
            .expect("access_token_allowlist poisoned")
            .len();

        assert!(
            count <= MAX_ALLOWLIST_SIZE,
            "SEC-002 / CWE-400: allowlist must not exceed MAX_ALLOWLIST_SIZE ({MAX_ALLOWLIST_SIZE}). \
             Got {count} entries."
        );
    }
}
