// ADR-023 §DTU-EXEMPT: This file IS the DTU behavioral clone for Cyberint.
// Sensor-named references here are intentional — this IS the clone, not a consumer.
// Exempt from tests/external/no-hardcoded-sensors/ compile-fail gate.
// Imports from deleted prism-sensors::auth::cyberint modules (001-A) verified absent.
// PLUGIN-MIGRATION-001-F AC-008 audit: no stale imports from deleted prism-sensors::auth modules.
//
//! Cyberint-specific harness clone router.
//!
//! Self-contained axum Router factory for `DtuType::Cyberint`.  Provides all
//! Cyberint-specific routes alongside the shared DTU control endpoints:
//!
//! # Cyberint routes
//!
//! - `GET  /api/v1/alerts`                       — paginated alert list (requires access_token cookie)
//! - `POST /api/v1/alerts`                       — alias for GET alerts
//! - `GET  /api/v1/alerts/:alert_id`             — alert detail (requires access_token cookie)
//! - `PATCH /api/v1/alerts/:alert_id/status`     — status transition (requires access_token cookie)
//! - `POST  /api/v1/alerts/:alert_id/close`      — irreversible close (requires access_token cookie)
//! - `GET  /api/v1/threat-intel`                 — threat-intel feed (requires access_token cookie)
//!
//! # Shared DTU control routes
//!
//! - `POST /dtu/configure`  — failure injection (X-Admin-Token guarded)
//! - `POST /dtu/reset`      — state reset
//! - `GET  /dtu/health`     — liveness check
//!
//! # Alert ID generation
//!
//! When `seed == DEFAULT_SEED` (42), alert IDs use the canonical fixture format
//! `CYB-2024-NNN` (backward-compatible with single-org AC tests).
//! For any other seed, alert IDs use the org-specific format
//! `alert-{org_slug}-{seed}-{index}` (guarantees disjoint sets for multi-org
//! isolation tests; BC-3.5.001 postcondition 2; TV-2).
//!
//! # Static cookie auth
//!
//! Cyberint uses static cookie-based auth (`access_token=<api_key>`) per ADR-031 §D3-a.
//! There is no legacy session-acquisition route; the real Cyberint API requires no
//! such endpoint (ADR-031 §D1-b).
//! The per-clone `CyberintCloneState` holds an `access_token_store: HashSet<String>`
//! (static allowlist; no org-keying needed in single-instance harness clones).
//! A demo token is registered at startup via `register_access_token()`.
//!
//! # Architecture Anchors
//!
//! - S-3.4.04 — Cyberint harness migration story
//! - ADR-031 §D1-b / §D3-a — no login step; static access_token cookie auth
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants
//! - BC-3.6.001 — Per-Org Failure Injection

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::clone_server::{CloneState, StartedClone};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default seed value used by `CustomerSpec::new()`.
///
/// When the harness clone is started with this seed, alert IDs use the canonical
/// fixture format `CYB-2024-NNN` for backward-compatibility with the single-org
/// AC tests migrated from the original `prism-dtu-cyberint` test suite.
const DEFAULT_SEED: u64 = 42;

/// Number of alerts on page 1 of the fixture set (when seed == DEFAULT_SEED).
const FIXTURE_PAGE1_COUNT: usize = 20;
/// Number of alerts on page 2 of the fixture set (when seed == DEFAULT_SEED).
const FIXTURE_PAGE2_COUNT: usize = 5;

/// Fixed demo `access_token` value pre-registered at clone startup.
///
/// Tests that need a valid token without going through `/dtu/configure` can use
/// this constant directly. ADR-031 §D3-a: the token is static (no login step).
pub const DEMO_ACCESS_TOKEN: &str = "harness-cyberint-demo-access-token";

/// Maximum number of entries permitted in `CyberintCloneState::access_token_store`.
///
/// Bounds unbounded `HashSet` growth from repeated `POST /dtu/configure` calls
/// (SEC-002 / CWE-400; F-P7-HIGH-001). Tokens beyond this cap are silently ignored
/// by `register_access_token`, mirroring the `MAX_ALLOWLIST_SIZE` cap in
/// `prism-dtu-cyberint::state::CyberintState` to maintain DTU↔harness fidelity.
///
/// 100 is sufficient for all harness test scenarios (each test registers at most a
/// handful of demo tokens) while preventing runaway memory growth from adversarial
/// or accidental bulk registration via the admin-guarded configure endpoint.
pub const MAX_HARNESS_ALLOWLIST_SIZE: usize = 100;

// ---------------------------------------------------------------------------
// In-memory alert record
// ---------------------------------------------------------------------------

/// Lightweight alert record held in the Cyberint harness clone state.
#[derive(Clone, Debug)]
pub struct HarnessAlert {
    pub alert_id: String,
    pub title: String,
    pub severity: String,
    pub created_at: Value, // Either a string (ISO 8601) or number (Unix epoch)
    pub source: String,
    pub alert_type: String,
}

impl HarnessAlert {
    fn to_json(&self, status: &str) -> Value {
        json!({
            "alert_id": self.alert_id,
            "title": self.title,
            "severity": self.severity,
            "status": status,
            "created_at": self.created_at,
            "source": self.source,
            "type": self.alert_type,
            "affected_assets": [],
        })
    }
}

// ---------------------------------------------------------------------------
// Alert status record
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AlertStatusRecord {
    pub status: String,
    pub closed: bool,
}

impl AlertStatusRecord {
    fn open() -> Self {
        Self {
            status: "open".to_owned(),
            closed: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Auth mode
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Default)]
enum AuthMode {
    #[default]
    Accept,
    Reject,
}

// ---------------------------------------------------------------------------
// Cyberint-specific clone state
// ---------------------------------------------------------------------------

/// Mutable state for the Cyberint harness clone.
///
/// Separate from the generic `CloneState` (which handles generic failure injection
/// and admin token validation) — this holds Cyberint-specific state: access tokens,
/// alert statuses, auth mode, and rate-limit config.
pub struct CyberintCloneState {
    /// Static access-token allowlist (ADR-031 §D3-a rule 3).
    ///
    /// Replaces the legacy `session_store` model: tokens are registered at startup
    /// (or via `POST /dtu/configure`) rather than issued per-login. A demo token
    /// (`DEMO_ACCESS_TOKEN`) is pre-registered in `start_cyberint_clone`.
    pub access_token_store: Mutex<HashSet<String>>,

    /// Per-alert status: `alert_id → AlertStatusRecord`.
    pub alert_store: Mutex<HashMap<String, AlertStatusRecord>>,

    /// Auth mode (toggled via `POST /dtu/configure`).
    // Field is private to avoid leaking the private AuthMode type.
    auth_mode: Mutex<AuthMode>,

    /// Rate-limit threshold — after N authenticated requests, respond 429.
    pub rate_limit_after: Mutex<Option<u32>>,

    /// Counter of authenticated requests since last reset.
    pub auth_request_count: Mutex<u32>,

    /// Page 1 alert fixtures (immutable after construction).
    pub alerts_page1: Vec<HarnessAlert>,

    /// Page 2 alert fixtures (immutable after construction).
    pub alerts_page2: Vec<HarnessAlert>,

    /// Threat intel records (immutable after construction).
    pub threat_intel: Vec<Value>,

    /// Demo token pre-registered at startup. Re-registered on `reset()`.
    ///
    /// Stored so `reset()` can restore the token without needing an HTTP call.
    /// ADR-031 §D3-a: the clone is usable immediately after `reset()` without
    /// a new login call.
    demo_token: String,
}

impl CyberintCloneState {
    /// Construct state for a clone identified by `org_slug` and `seed`.
    ///
    /// When `seed == DEFAULT_SEED`, alert IDs use the canonical fixture format
    /// `CYB-2024-NNN` (backward compat with single-org AC tests).
    /// Otherwise, alert IDs use `alert-{org_slug}-{seed}-{index}`.
    pub fn new(org_slug: &str, seed: u64) -> Self {
        Self::with_demo_token(org_slug, seed, DEMO_ACCESS_TOKEN.to_owned())
    }

    /// Construct state with a specific demo access token (for tests that need a known value).
    ///
    /// The demo token is routed through `register_access_token` so SEC-002 validation
    /// (length, CTL chars, size cap) is applied even at construction time — matching the
    /// pattern used in `prism-dtu-cyberint::state::CyberintState::with_demo_token`
    /// (F-P7-HIGH-001 sibling-sweep).
    pub fn with_demo_token(org_slug: &str, seed: u64, demo_token: String) -> Self {
        let (page1, page2) = generate_alerts(org_slug, seed);
        let threat_intel = generate_threat_intel(org_slug, seed);

        // Pre-populate alert_store from fixture (all "open", not closed).
        let mut alert_store: HashMap<String, AlertStatusRecord> = HashMap::new();
        for a in page1.iter().chain(page2.iter()) {
            alert_store.insert(a.alert_id.clone(), AlertStatusRecord::open());
        }

        // Build with an empty store first, then route the demo token through
        // register_access_token so SEC-002 validation applies (F-P7-HIGH-001).
        let state = Self {
            access_token_store: Mutex::new(HashSet::new()),
            alert_store: Mutex::new(alert_store),
            auth_mode: Mutex::new(AuthMode::Accept),
            rate_limit_after: Mutex::new(None),
            auth_request_count: Mutex::new(0),
            alerts_page1: page1,
            alerts_page2: page2,
            threat_intel,
            demo_token: demo_token.clone(),
        };
        // Register through the validated path (length cap, CTL rejection, size cap).
        state.register_access_token(demo_token);
        state
    }

    /// Reset all mutable state to initial values.
    ///
    /// - Clears `access_token_store` and re-registers the configured demo token
    ///   (so the clone is immediately usable again without a new configure call).
    /// - Resets all alert statuses to "open" / not closed.
    /// - Resets auth_mode to Accept.
    /// - Resets rate_limit_after to None.
    /// - Resets auth_request_count to 0.
    ///
    /// ADR-031 §D3-a: re-registering the demo token ensures no login step is needed
    /// after reset.
    #[allow(clippy::expect_used)]
    pub fn reset(&self) {
        // Clear access_token_store, then re-register the demo token via the validated
        // path (SEC-002 guards) so reset() does not bypass length/CTL/cap checks
        // (F-P7-HIGH-001 sibling-sweep; mirrors prism-dtu-cyberint::state::reset_all).
        {
            let mut store = self
                .access_token_store
                .lock()
                .expect("access_token_store poisoned");
            store.clear();
        } // lock released before calling register_access_token (which re-acquires it)
        self.register_access_token(self.demo_token.clone());

        let mut alert_store = self.alert_store.lock().expect("alert_store poisoned");
        for val in alert_store.values_mut() {
            *val = AlertStatusRecord::open();
        }
        drop(alert_store);

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

    /// Register an access token in the static allowlist.
    ///
    /// ADR-031 §D3-a rule 3: tokens are registered statically, not issued per-login.
    ///
    /// # Validation (SEC-002 / CWE-20 / CWE-400; F-P7-HIGH-001)
    ///
    /// Mirrors the production E-AUTH-006 contract (`StaticCookieAuthProvider`) and the
    /// identical guards in `prism-dtu-cyberint::state::CyberintState::register_access_token`
    /// to maintain DTU↔harness fidelity:
    ///
    /// - Tokens exceeding 4096 bytes are silently ignored.
    /// - Tokens containing ASCII control characters (0x00–0x1F, DEL 0x7F) are silently
    ///   ignored (defense against CWE-113 CRLF injection and CWE-93).
    /// - Once the store reaches `MAX_HARNESS_ALLOWLIST_SIZE`, further inserts are
    ///   silently ignored (defense against CWE-400 unbounded growth).
    ///
    /// Silent-ignore is chosen (over returning an error) because callers do not propagate
    /// errors to HTTP responses for this path, and the harness is not a production auth
    /// service.
    #[allow(clippy::expect_used)]
    pub fn register_access_token(&self, token: String) {
        // SEC-002 / CWE-20: reject oversized tokens (mirror E-AUTH-006 4096-byte limit).
        if token.len() > 4096 {
            return;
        }
        // SEC-002 / CWE-93 / CWE-113: reject tokens containing ASCII control chars
        // (0x00–0x1F incl. CR/LF, and DEL 0x7F) — mirrors E-AUTH-006 CTL rejection.
        if token.bytes().any(|b| b < 0x21 || b == 0x7F) {
            return;
        }
        let mut store = self
            .access_token_store
            .lock()
            .expect("access_token_store poisoned");
        // SEC-002 / CWE-400: cap allowlist size to prevent unbounded memory growth.
        if store.len() >= MAX_HARNESS_ALLOWLIST_SIZE {
            return;
        }
        store.insert(token);
    }

    #[allow(clippy::expect_used)]
    fn is_valid_access_token(&self, token: &str) -> bool {
        self.access_token_store
            .lock()
            .expect("access_token_store poisoned")
            .contains(token)
    }

    #[allow(clippy::expect_used)]
    fn auth_mode(&self) -> AuthMode {
        self.auth_mode.lock().expect("auth_mode poisoned").clone()
    }

    /// Apply the configure payload from `POST /dtu/configure`.
    ///
    /// Supported fields (deny-unknown-fields):
    /// - `auth_mode`: `"reject"` | `"accept"`
    /// - `rate_limit_after`: u32
    /// - `access_token`: String — register an additional allowed token
    /// - `clear`: bool — clears all failure modes
    ///
    /// Returns `Err(msg)` if the payload contains unknown fields (TD-WV0-04).
    #[allow(clippy::expect_used)]
    pub fn apply_config(&self, body: &Value) -> Result<(), String> {
        let cfg: ConfigureBody = serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid /dtu/configure payload: {e}"))?;

        if cfg.clear == Some(true) {
            *self.auth_mode.lock().expect("auth_mode poisoned") = AuthMode::Accept;
            *self
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned") = None;
            *self
                .auth_request_count
                .lock()
                .expect("auth_request_count poisoned") = 0;
            return Ok(());
        }

        if let Some(mode) = cfg.auth_mode.as_deref() {
            *self.auth_mode.lock().expect("auth_mode poisoned") = match mode {
                "reject" => AuthMode::Reject,
                _ => AuthMode::Accept,
            };
        }

        if let Some(n) = cfg.rate_limit_after {
            *self
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned") = Some(n);
            // Reset counter when a new limit is set.
            *self
                .auth_request_count
                .lock()
                .expect("auth_request_count poisoned") = 0;
        }

        if let Some(token) = cfg.access_token {
            self.register_access_token(token);
        }

        Ok(())
    }

    /// Check and increment the request counter for rate-limit enforcement.
    ///
    /// Returns `true` if the request should be rate-limited (429).
    #[allow(clippy::expect_used)]
    fn check_and_increment_rate_limit(&self) -> bool {
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
}

// ---------------------------------------------------------------------------
// Configure body (deny_unknown_fields — TD-WV0-04)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigureBody {
    auth_mode: Option<String>,
    rate_limit_after: Option<u32>,
    /// Register an additional access_token in the static allowlist.
    access_token: Option<String>,
    // Fields below are present to support deny_unknown_fields validation (TD-WV0-04).
    // The values are consumed via the GenericConfigBody path in dtu_configure.
    #[allow(dead_code)]
    retry_after_secs: Option<u32>,
    #[allow(dead_code)]
    internal_error_at: Option<u32>,
    #[allow(dead_code)]
    network_timeout_ms: Option<u64>,
    #[allow(dead_code)]
    malformed_response: Option<bool>,
    #[allow(dead_code)]
    unprocessable_at: Option<u32>,
    clear: Option<bool>,
}

// ---------------------------------------------------------------------------
// Alert fixture generation
// ---------------------------------------------------------------------------

/// Build page1 and page2 alert lists for a clone.
///
/// When `seed == DEFAULT_SEED`, uses canonical `CYB-2024-NNN` IDs and mixed
/// ISO 8601 / Unix epoch timestamps matching the original fixture file.
/// Otherwise, generates org-specific IDs for multi-org isolation.
fn generate_alerts(org_slug: &str, seed: u64) -> (Vec<HarnessAlert>, Vec<HarnessAlert>) {
    if seed == DEFAULT_SEED {
        generate_fixture_alerts()
    } else {
        generate_seeded_alerts(org_slug, seed)
    }
}

/// Return page1 and page2 alerts matching the canonical `fixtures/alerts.json` format.
///
/// IDs "CYB-2024-001" through "CYB-2024-020" on page 1, "CYB-2024-021" through
/// "CYB-2024-025" on page 2.  Mixed ISO 8601 / Unix epoch timestamps (AC-5).
fn generate_fixture_alerts() -> (Vec<HarnessAlert>, Vec<HarnessAlert>) {
    // Alternating ISO 8601 / Unix epoch timestamps (AC-5: must have both kinds).
    let page1_data: &[(&str, &str, &str, Value, &str, &str)] = &[
        (
            "CYB-2024-001",
            "Phishing Campaign Targeting Finance Team",
            "critical",
            json!("2024-01-15T08:23:41Z"),
            "cyberint",
            "phishing",
        ),
        (
            "CYB-2024-002",
            "Ransomware Activity Detected",
            "high",
            json!(1705312800u64),
            "cyberint",
            "ransomware",
        ),
        (
            "CYB-2024-003",
            "Data Exfiltration Attempt",
            "critical",
            json!("2024-01-16T14:05:00Z"),
            "cyberint",
            "data_leak",
        ),
        (
            "CYB-2024-004",
            "Credential Stuffing Campaign",
            "medium",
            json!(1705399200u64),
            "cyberint",
            "credential_stuffing",
        ),
        (
            "CYB-2024-005",
            "Brand Abuse on Social Media",
            "low",
            json!("2024-01-17T09:12:33Z"),
            "cyberint",
            "brand_abuse",
        ),
        (
            "CYB-2024-006",
            "Dark Web Data Exposure",
            "high",
            json!(1705485600u64),
            "cyberint",
            "data_exposure",
        ),
        (
            "CYB-2024-007",
            "Malicious Domain Registration",
            "medium",
            json!("2024-01-18T11:30:00Z"),
            "cyberint",
            "typosquatting",
        ),
        (
            "CYB-2024-008",
            "Supply Chain Compromise",
            "critical",
            json!(1705572000u64),
            "cyberint",
            "supply_chain",
        ),
        (
            "CYB-2024-009",
            "Executive Impersonation",
            "medium",
            json!("2024-01-19T16:45:00Z"),
            "cyberint",
            "impersonation",
        ),
        (
            "CYB-2024-010",
            "VPN Credential Leak",
            "high",
            json!(1705658400u64),
            "cyberint",
            "credential_leak",
        ),
        (
            "CYB-2024-011",
            "Malware Distribution Campaign",
            "high",
            json!("2024-01-20T08:00:00Z"),
            "cyberint",
            "malware",
        ),
        (
            "CYB-2024-012",
            "Exposed API Endpoint",
            "medium",
            json!(1705744800u64),
            "cyberint",
            "api_exposure",
        ),
        (
            "CYB-2024-013",
            "Botnet Recruitment Attempt",
            "low",
            json!("2024-01-21T13:15:00Z"),
            "cyberint",
            "botnet",
        ),
        (
            "CYB-2024-014",
            "Insider Threat Indicators",
            "high",
            json!(1705831200u64),
            "cyberint",
            "insider_threat",
        ),
        (
            "CYB-2024-015",
            "Zero-Day Vulnerability Exploitation",
            "critical",
            json!("2024-01-22T10:00:00Z"),
            "cyberint",
            "zero_day",
        ),
        (
            "CYB-2024-016",
            "Spear Phishing Email Campaign",
            "high",
            json!(1705917600u64),
            "cyberint",
            "spear_phishing",
        ),
        (
            "CYB-2024-017",
            "Fraudulent Mobile Application",
            "medium",
            json!("2024-01-23T14:30:00Z"),
            "cyberint",
            "mobile_fraud",
        ),
        (
            "CYB-2024-018",
            "Database Credentials on Paste Site",
            "critical",
            json!(1706004000u64),
            "cyberint",
            "data_exposure",
        ),
        (
            "CYB-2024-019",
            "DDoS Attack Planning Forum",
            "medium",
            json!("2024-01-24T09:45:00Z"),
            "cyberint",
            "ddos",
        ),
        (
            "CYB-2024-020",
            "Industrial Control System Probing",
            "high",
            json!(1706090400u64),
            "cyberint",
            "ics_attack",
        ),
    ];

    let page2_data: &[(&str, &str, &str, Value, &str, &str)] = &[
        (
            "CYB-2024-021",
            "Shadow IT Cloud Storage Exposure",
            "medium",
            json!("2024-01-25T11:00:00Z"),
            "cyberint",
            "shadow_it",
        ),
        (
            "CYB-2024-022",
            "Compromised Partner Credentials",
            "high",
            json!(1706176800u64),
            "cyberint",
            "credential_leak",
        ),
        (
            "CYB-2024-023",
            "Threat Actor Targeting Discussion",
            "low",
            json!("2024-01-26T15:20:00Z"),
            "cyberint",
            "threat_intel",
        ),
        (
            "CYB-2024-024",
            "Leaked Source Code Repository",
            "high",
            json!(1706263200u64),
            "cyberint",
            "data_exposure",
        ),
        (
            "CYB-2024-025",
            "Certificate Transparency Log Anomaly",
            "low",
            json!("2024-01-27T08:30:00Z"),
            "cyberint",
            "cert_anomaly",
        ),
    ];

    let page1 = page1_data
        .iter()
        .map(
            |(id, title, sev, created_at, src, alert_type)| HarnessAlert {
                alert_id: id.to_string(),
                title: title.to_string(),
                severity: sev.to_string(),
                created_at: created_at.clone(),
                source: src.to_string(),
                alert_type: alert_type.to_string(),
            },
        )
        .collect();

    let page2 = page2_data
        .iter()
        .map(
            |(id, title, sev, created_at, src, alert_type)| HarnessAlert {
                alert_id: id.to_string(),
                title: title.to_string(),
                severity: sev.to_string(),
                created_at: created_at.clone(),
                source: src.to_string(),
                alert_type: alert_type.to_string(),
            },
        )
        .collect();

    (page1, page2)
}

/// Generate org-specific alerts for `seed != DEFAULT_SEED`.
///
/// Alert IDs: `alert-{org_slug}-{seed}-{index}`.
/// Mixed timestamps: even indices get ISO 8601, odd indices get Unix epoch.
fn generate_seeded_alerts(org_slug: &str, seed: u64) -> (Vec<HarnessAlert>, Vec<HarnessAlert>) {
    let severities = ["low", "medium", "high", "critical"];
    let types = [
        "phishing",
        "malware",
        "data_exposure",
        "ransomware",
        "botnet",
    ];

    let make_alert = |i: usize| {
        let sev = severities[i % severities.len()];
        let atype = types[i % types.len()];
        let created_at: Value = if i.is_multiple_of(2) {
            json!(format!("2024-01-{:02}T10:00:00Z", (i % 28) + 1))
        } else {
            json!(1705312800u64 + (i as u64 * 86400))
        };
        HarnessAlert {
            alert_id: format!("alert-{}-{}-{}", org_slug, seed, i),
            title: format!("Alert {} for {}", i, org_slug),
            severity: sev.to_owned(),
            created_at,
            source: "cyberint".to_owned(),
            alert_type: atype.to_owned(),
        }
    };

    let page1: Vec<HarnessAlert> = (0..FIXTURE_PAGE1_COUNT).map(make_alert).collect();
    let page2: Vec<HarnessAlert> = (FIXTURE_PAGE1_COUNT..FIXTURE_PAGE1_COUNT + FIXTURE_PAGE2_COUNT)
        .map(make_alert)
        .collect();

    (page1, page2)
}

/// Generate threat intel records for this clone.
///
/// For seed == DEFAULT_SEED: uses canonical threat format matching `fixtures/threats.json`.
/// For other seeds: generates org-specific threat indicators.
fn generate_threat_intel(org_slug: &str, seed: u64) -> Vec<Value> {
    if seed == DEFAULT_SEED {
        // Minimal threat-intel fixture compatible with fidelity checks.
        vec![
            json!({
                "indicator_id": "TI-2024-001",
                "type": "domain",
                "value": "malicious.example.com",
                "confidence": 85,
                "source": "cyberint",
            }),
            json!({
                "indicator_id": "TI-2024-002",
                "type": "ip",
                "value": "192.168.100.1",
                "confidence": 70,
                "source": "cyberint",
            }),
        ]
    } else {
        // Org-specific threat indicators with seed-derived IDs.
        (0..3)
            .map(|i| {
                json!({
                    "indicator_id": format!("ti-{}-{}-{}", org_slug, seed, i),
                    "type": "domain",
                    "value": format!("malicious-{}-{}.example.com", org_slug, i),
                    "confidence": 70 + i as u64,
                    "source": "cyberint",
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Shared axum state wrapper
// ---------------------------------------------------------------------------

/// Combined state passed to all Cyberint route handlers.
pub struct CyberintRouteState {
    /// Cyberint-specific mutable state (access tokens, alert statuses, auth mode).
    pub cyberint: Arc<CyberintCloneState>,
    /// Generic clone state (failure injection, admin token, request counter).
    pub clone_state: Arc<CloneState>,
}

// ---------------------------------------------------------------------------
// Cookie auth helpers
// ---------------------------------------------------------------------------

/// Extract the `access_token` cookie value from the `Cookie` header.
///
/// ADR-031 §D3-a: validates the `access_token` cookie (NOT the legacy session-identifier cookie).
fn extract_access_token(headers: &HeaderMap) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(token) = part.strip_prefix("access_token=") {
            return Some(token.to_owned());
        }
    }
    None
}

/// Build an HTTP 401 unauthorized response.
fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "unauthorized", "code": 401})),
    )
        .into_response()
}

/// Check cookie auth and rate limit. Returns `Ok(())` to proceed or `Err(response)` to short-circuit.
///
/// Validates the `access_token` cookie against the static allowlist (ADR-031 §D3-a).
/// The legacy session-identifier cookie name is NOT accepted.
#[allow(clippy::result_large_err)]
fn check_auth(
    state: &CyberintCloneState,
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    // auth_mode=reject: always 401 regardless of cookie (EC-006).
    if state.auth_mode() == AuthMode::Reject {
        return Err(unauthorized());
    }

    let token = extract_access_token(headers).ok_or_else(unauthorized)?;
    if !state.is_valid_access_token(&token) {
        return Err(unauthorized());
    }

    if state.check_and_increment_rate_limit() {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "rate limit exceeded", "code": 429})),
        )
            .into_response());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/alerts` (and `POST /api/v1/alerts`)
///
/// Returns paginated alerts from the in-memory fixture. Requires access_token cookie auth.
#[derive(Debug, Deserialize, Default)]
struct AlertListParams {
    cursor: Option<String>,
}

#[allow(clippy::expect_used)]
async fn get_alerts(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Query(params): Query<AlertListParams>,
) -> impl IntoResponse {
    // Check for clone-level failure injection first (NetworkTimeout, etc.)
    let count = state.clone_state.increment_request();
    let failure_mode = state.clone_state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &failure_mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    // Auth check.
    if let Err(resp) = check_auth(&state.cyberint, &headers) {
        return resp;
    }

    // Apply general failure modes (AuthReject counts towards failure injection,
    // RateLimit applies separately from the cyberint auth rate limit).
    match &failure_mode {
        FailureMode::AuthReject => return unauthorized(),
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } if count > *after_n_requests => {
            let mut resp = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "rate limited"})),
            )
                .into_response();
            resp.headers_mut().insert(
                "retry-after",
                #[allow(clippy::expect_used)]
                retry_after_secs
                    .to_string()
                    .parse()
                    .expect("retry_after_secs is valid header value"),
            );
            return resp;
        }
        FailureMode::RateLimit { .. } => {}
        FailureMode::MalformedResponse => {
            return axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    b"\xff\xfe{not valid json!@#$%^&*(" as &[u8],
                ))
                .expect("build malformed response");
        }
        _ => {}
    }

    #[allow(clippy::expect_used)]
    let alert_store = state
        .cyberint
        .alert_store
        .lock()
        .expect("alert_store poisoned");

    let (alerts, next_cursor) = if params.cursor.as_deref() == Some("page2") {
        (&state.cyberint.alerts_page2, Value::Null)
    } else {
        (&state.cyberint.alerts_page1, json!("page2"))
    };

    let data: Vec<Value> = alerts
        .iter()
        .map(|a| {
            let status = alert_store
                .get(&a.alert_id)
                .map(|s| s.status.as_str())
                .unwrap_or("open");
            a.to_json(status)
        })
        .collect();

    drop(alert_store);

    (
        StatusCode::OK,
        Json(json!({"data": data, "next_cursor": next_cursor})),
    )
        .into_response()
}

/// `GET /api/v1/alerts/:alert_id`
///
/// Returns alert detail with current status. Requires access_token cookie auth.
#[allow(clippy::expect_used)]
async fn get_alert_by_id(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.cyberint, &headers) {
        return resp;
    }

    let alert_store = state
        .cyberint
        .alert_store
        .lock()
        .expect("alert_store poisoned");

    let status_record = match alert_store.get(&alert_id) {
        Some(r) => r.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "alert not found"})),
            )
                .into_response()
        }
    };
    drop(alert_store);

    let alert = state
        .cyberint
        .alerts_page1
        .iter()
        .chain(state.cyberint.alerts_page2.iter())
        .find(|a| a.alert_id == alert_id);

    match alert {
        Some(a) => (StatusCode::OK, Json(a.to_json(&status_record.status))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "alert not found"})),
        )
            .into_response(),
    }
}

/// Body for `PATCH /api/v1/alerts/:alert_id/status`.
#[derive(Debug, Deserialize)]
struct PatchStatusBody {
    status: String,
}

/// `PATCH /api/v1/alerts/:alert_id/status`
///
/// Acknowledges an alert. Returns 400 if the alert is already closed.
#[allow(clippy::expect_used)]
async fn patch_alert_status(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
    Json(body): Json<PatchStatusBody>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.cyberint, &headers) {
        return resp;
    }

    let mut alert_store = state
        .cyberint
        .alert_store
        .lock()
        .expect("alert_store poisoned");

    match alert_store.get_mut(&alert_id) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = body.status.clone();
            (
                StatusCode::OK,
                Json(json!({"alert_id": alert_id, "status": body.status})),
            )
                .into_response()
        }
    }
}

/// `POST /api/v1/alerts/:alert_id/close`
///
/// Irreversibly closes an alert within this clone session.
#[allow(clippy::expect_used)]
async fn post_close_alert(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.cyberint, &headers) {
        return resp;
    }

    let mut alert_store = state
        .cyberint
        .alert_store
        .lock()
        .expect("alert_store poisoned");

    match alert_store.get_mut(&alert_id) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = "closed".to_owned();
            record.closed = true;
            (
                StatusCode::OK,
                Json(json!({"alert_id": alert_id, "status": "closed"})),
            )
                .into_response()
        }
    }
}

/// `GET /api/v1/threat-intel`
///
/// Threat intelligence feed. Requires access_token cookie auth.
#[derive(Debug, Deserialize, Default)]
struct ThreatListParams {
    cursor: Option<String>,
}

async fn get_threat_intel(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Query(params): Query<ThreatListParams>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.cyberint, &headers) {
        return resp;
    }

    let (data, next_cursor) = if params.cursor.is_some() {
        (vec![], Value::Null)
    } else {
        (state.cyberint.threat_intel.clone(), Value::Null)
    };

    (
        StatusCode::OK,
        Json(json!({"data": data, "next_cursor": next_cursor})),
    )
        .into_response()
}

/// `POST /dtu/configure`
///
/// Failure injection endpoint. Guarded by `X-Admin-Token` (generic clone admin token).
/// Also accepts Cyberint-specific config fields like `auth_mode`, `rate_limit_after`, and `access_token`.
#[allow(clippy::expect_used)]
async fn dtu_configure(
    State(state): State<Arc<CyberintRouteState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Admin token check using generic clone state token.
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.clone_state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }

    // Apply Cyberint-specific config.
    if let Err(e) = state.cyberint.apply_config(&body) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e}))).into_response();
    }

    // Also apply generic failure mode (rate_limit_after, network_timeout_ms, etc.)
    // by delegating to the generic `CloneState`.
    let cfg_result: Result<GenericConfigBody, _> = serde_json::from_value(body.clone());
    if let Ok(cfg) = cfg_result {
        let mode = if cfg.clear == Some(true) {
            FailureMode::None
        } else if cfg.auth_mode.as_deref() == Some("reject") {
            FailureMode::AuthReject
        } else if let Some(n) = cfg.rate_limit_after {
            FailureMode::RateLimit {
                after_n_requests: n,
                retry_after_secs: cfg.retry_after_secs.unwrap_or(60),
            }
        } else if let Some(n) = cfg.internal_error_at {
            FailureMode::InternalError { at_request_n: n }
        } else if let Some(ms) = cfg.network_timeout_ms {
            FailureMode::NetworkTimeout { after_ms: ms }
        } else if cfg.malformed_response == Some(true) {
            FailureMode::MalformedResponse
        } else if let Some(n) = cfg.unprocessable_at {
            FailureMode::Unprocessable { at_request_n: n }
        } else {
            FailureMode::None
        };

        state
            .clone_state
            .request_count
            .store(0, std::sync::atomic::Ordering::SeqCst);
        state.clone_state.set_failure_mode(mode);
    }

    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// Permissive body struct for extracting generic failure mode fields.
/// Does NOT use deny_unknown_fields so it can be parsed alongside Cyberint fields.
#[derive(Debug, Deserialize, Default)]
struct GenericConfigBody {
    auth_mode: Option<String>,
    rate_limit_after: Option<u32>,
    retry_after_secs: Option<u32>,
    internal_error_at: Option<u32>,
    network_timeout_ms: Option<u64>,
    malformed_response: Option<bool>,
    unprocessable_at: Option<u32>,
    clear: Option<bool>,
}

/// `POST /dtu/reset`
///
/// Resets all Cyberint clone state (access token store re-registers demo token,
/// alert statuses restored, auth mode reset, counters zeroed).
/// Also resets the generic CloneState request counter and failure mode.
///
/// ADR-031 §D3-a: `reset()` re-registers the demo token so the clone is
/// immediately usable without a configure call after reset.
async fn dtu_reset(State(state): State<Arc<CyberintRouteState>>) -> impl IntoResponse {
    state.cyberint.reset();
    state.clone_state.set_failure_mode(FailureMode::None);
    state
        .clone_state
        .request_count
        .store(0, std::sync::atomic::Ordering::SeqCst);
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health`
///
/// Liveness check. No auth required, no state access.
async fn dtu_health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `GET /api/v1/events`
///
/// Legacy Cyberint event-list endpoint alias used by the harness isolation tests
/// that were written before the Cyberint-specific router was introduced.
/// Does NOT require cookie auth — mirrors the generic clone's unauthenticated
/// device-list behaviour so isolation tests can use it to verify org-disjointness.
async fn get_events(State(state): State<Arc<CyberintRouteState>>) -> impl IntoResponse {
    // Serve a simple list of event IDs derived from the alert fixture (no auth required).
    // IDs are org-slug-qualified so the harness isolation tests detect cross-org leakage.
    let items: Vec<Value> = state
        .cyberint
        .alerts_page1
        .iter()
        .map(|a| {
            json!({
                "id": format!("evt-{}", a.alert_id),
                "device_id": format!("evt-{}", a.alert_id),
                "org": state.clone_state.org_slug,
            })
        })
        .collect();
    (StatusCode::OK, Json(json!({"items": items}))).into_response()
}

// ---------------------------------------------------------------------------
// Test-hook handlers (BC-3.6.002 crash detection tests)
// ---------------------------------------------------------------------------

/// Body for `POST /dtu/test-hook/panic`.
#[derive(Debug, Deserialize)]
struct PanicBody {
    message: String,
}

/// `POST /dtu/test-hook/panic`
///
/// Stores a `TestHookSignal::Panic` in the clone state.
/// The background task loop observes this and propagates the panic.
#[allow(clippy::expect_used)]
async fn test_hook_panic(
    State(state): State<Arc<CyberintRouteState>>,
    Json(body): Json<PanicBody>,
) -> impl IntoResponse {
    use crate::clone_server::TestHookSignal;
    *state
        .clone_state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::Panic(body.message));
    (StatusCode::OK, Json(json!({"status": "panic queued"}))).into_response()
}

/// `POST /dtu/test-hook/premature-ok`
#[allow(clippy::expect_used)]
async fn test_hook_premature_ok(State(state): State<Arc<CyberintRouteState>>) -> impl IntoResponse {
    use crate::clone_server::TestHookSignal;
    *state
        .clone_state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::PrematureOk);
    (
        StatusCode::OK,
        Json(json!({"status": "premature-ok queued"})),
    )
        .into_response()
}

/// `POST /dtu/test-hook/non-string-panic`
#[allow(clippy::expect_used)]
async fn test_hook_non_string_panic(
    State(state): State<Arc<CyberintRouteState>>,
) -> impl IntoResponse {
    use crate::clone_server::TestHookSignal;
    *state
        .clone_state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::NonStringPanic);
    (
        StatusCode::OK,
        Json(json!({"status": "non-string-panic queued"})),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Network-mode: bearer-aware auth check
// ---------------------------------------------------------------------------

/// Result of bearer-token validation for Network-mode routes.
#[derive(Debug, PartialEq, Eq)]
enum BearerCheck {
    /// No `Authorization` header present — fall through to normal auth.
    Absent,
    /// Correct bearer token — bypass downstream auth and serve directly.
    Valid,
    /// Wrong bearer token — return HTTP 401 immediately.
    Invalid,
}

/// Classify the `Authorization: Bearer` header for Network-mode cross-org tests.
///
/// Policy (BC-3.5.002 postcondition 2; VP-126; TV-3):
/// - No Authorization header → `Absent` (caller may fall through to cookie auth)
/// - Correct bearer token    → `Valid` (caller may bypass cookie auth)
/// - Wrong bearer token      → `Invalid` (caller must return 401)
fn classify_bearer(headers: &HeaderMap, admin_token: &str) -> BearerCheck {
    if let Some(auth_val) = headers.get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return if token == admin_token {
                    BearerCheck::Valid
                } else {
                    BearerCheck::Invalid
                };
            }
        }
    }
    BearerCheck::Absent
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the Cyberint-specific axum Router for use in the harness.
///
/// Routes handle static access_token cookie auth, alert lifecycle, threat-intel, and DTU
/// control endpoints.  No legacy session-acquisition route is registered (ADR-031 §D1-b).
/// The `clone_state` provides the generic failure-injection machinery and admin token;
/// the `cyberint_state` provides Cyberint-specific access_token/alert state.
fn build_cyberint_router(
    clone_state: Arc<CloneState>,
    cyberint_state: Arc<CyberintCloneState>,
) -> Router {
    let route_state = Arc::new(CyberintRouteState {
        cyberint: cyberint_state,
        clone_state,
    });

    Router::new()
        // Alert routes (no legacy session-acquisition route — ADR-031 §D1-b)
        .route("/api/v1/alerts", get(get_alerts))
        .route("/api/v1/alerts", post(get_alerts))
        .route("/api/v1/alerts/:alert_id", get(get_alert_by_id))
        .route("/api/v1/alerts/:alert_id/status", patch(patch_alert_status))
        .route("/api/v1/alerts/:alert_id/close", post(post_close_alert))
        // Threat intel
        .route("/api/v1/threat-intel", get(get_threat_intel))
        // Legacy event list alias (used by harness isolation tests)
        .route("/api/v1/events", get(get_events))
        // DTU control
        .route("/dtu/configure", post(dtu_configure))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/health", get(dtu_health))
        // Test hooks (BC-3.6.002 crash detection)
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(route_state)
}

/// Build the Cyberint-specific router for Network isolation mode.
///
/// In Network mode, the server additionally validates `Authorization: Bearer`
/// tokens on device-list routes so that cross-org credential-mismatch tests
/// (BC-3.5.002 postcondition 2; VP-126; TV-3) produce HTTP 401.
///
/// The Cyberint alert endpoint `/api/v1/alerts` is wrapped with bearer-check
/// middleware: if a Bearer token is present and mismatched, return 401 before
/// even attempting cookie auth.
fn build_cyberint_network_router(
    clone_state: Arc<CloneState>,
    cyberint_state: Arc<CyberintCloneState>,
) -> Router {
    let route_state = Arc::new(CyberintRouteState {
        cyberint: cyberint_state,
        clone_state,
    });

    // For network mode: alerts routes get bearer-aware wrapper.
    //
    // Policy (BC-3.5.002 postcondition 2; VP-126; TV-3):
    // - Valid bearer (matching admin_token) → bypass cookie auth, serve alerts (HTTP 200).
    // - Invalid bearer (present but wrong)  → reject with HTTP 401.
    // - Absent bearer                        → fall through to normal cookie auth.
    let rs_for_bearer = Arc::clone(&route_state);
    let alerts_with_bearer = move |headers: HeaderMap,
                                   state: State<Arc<CyberintRouteState>>,
                                   query: Query<AlertListParams>| {
        let rs = Arc::clone(&rs_for_bearer);
        async move {
            match classify_bearer(&headers, &rs.clone_state.admin_token) {
                BearerCheck::Invalid => (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "invalid bearer token"})),
                )
                    .into_response(),
                BearerCheck::Valid => {
                    // Correct bearer token: bypass cookie auth, serve alerts directly.
                    let count = rs.clone_state.increment_request();
                    let failure_mode = rs.clone_state.current_failure_mode();

                    if let FailureMode::NetworkTimeout { after_ms } = &failure_mode {
                        if *after_ms > 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1))
                                .await;
                        }
                    }

                    #[allow(clippy::expect_used)]
                    let alert_store = rs
                        .cyberint
                        .alert_store
                        .lock()
                        .expect("alert_store poisoned");
                    let (alerts, next_cursor) = if query.cursor.as_deref() == Some("page2") {
                        (&rs.cyberint.alerts_page2, serde_json::Value::Null)
                    } else {
                        (&rs.cyberint.alerts_page1, json!("page2"))
                    };
                    let _ = count; // suppress unused warning
                    let data: Vec<serde_json::Value> = alerts
                        .iter()
                        .map(|a| {
                            let status = alert_store
                                .get(&a.alert_id)
                                .map(|s| s.status.as_str())
                                .unwrap_or("open");
                            a.to_json(status)
                        })
                        .collect();
                    drop(alert_store);
                    (
                        StatusCode::OK,
                        Json(json!({"data": data, "next_cursor": next_cursor})),
                    )
                        .into_response()
                }
                BearerCheck::Absent => {
                    // No bearer header: use normal cookie auth.
                    get_alerts(state, headers, query).await.into_response()
                }
            }
        }
    };

    Router::new()
        // No legacy session-acquisition route — ADR-031 §D1-b
        .route("/api/v1/alerts", get(alerts_with_bearer))
        .route("/api/v1/alerts/:alert_id", get(get_alert_by_id))
        .route("/api/v1/alerts/:alert_id/status", patch(patch_alert_status))
        .route("/api/v1/alerts/:alert_id/close", post(post_close_alert))
        .route("/api/v1/threat-intel", get(get_threat_intel))
        // Legacy event list alias (used by harness isolation tests)
        .route("/api/v1/events", get(get_events))
        // DTU control
        .route("/dtu/configure", post(dtu_configure))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/health", get(dtu_health))
        // Test hooks (BC-3.6.002 crash detection)
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(route_state)
}

// ---------------------------------------------------------------------------
// Clone startup — called from builder.rs
// ---------------------------------------------------------------------------

/// Start a Cyberint-specific harness clone on the given pre-bound TCP listener.
///
/// Creates a `CyberintCloneState` from `org_slug` and `seed`, pre-registers the
/// `DEMO_ACCESS_TOKEN` (ADR-031 §D3-a: no login step required), wires the generic
/// `CloneState`, builds the Cyberint router, and spawns the server.
///
/// Returns a `StartedClone` compatible with the generic harness machinery.
///
/// This is called from `builder.rs` when `DtuType::Cyberint` is dispatched.
#[allow(clippy::expect_used)]
pub async fn start_cyberint_clone(
    listener: tokio::net::TcpListener,
    org_slug: String,
    seed: u64,
    shutdown_rx: broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
    network_mode: bool,
) -> StartedClone {
    use crate::types::DtuType;

    let addr = listener
        .local_addr()
        .expect("listener must have local addr after bind");
    let admin_token = uuid::Uuid::new_v4().to_string();

    let clone_state = Arc::new(CloneState::new(
        org_slug.clone(),
        seed,
        DtuType::Cyberint,
        admin_token.clone(),
    ));

    // Pre-register DEMO_ACCESS_TOKEN so tests can auth immediately without a configure call.
    // ADR-031 §D3-a rule 3: tokens are static; no login step required.
    let cyberint_state = Arc::new(CyberintCloneState::new(&org_slug, seed));

    let router = if network_mode {
        build_cyberint_network_router(Arc::clone(&clone_state), cyberint_state)
    } else {
        build_cyberint_router(Arc::clone(&clone_state), cyberint_state)
    };

    let state_for_hook = Arc::clone(&clone_state);

    let handle: JoinHandle<()> = tokio::spawn(async move {
        let server_future = run_cyberint_server(listener, router, shutdown_rx);
        let hook_future = crate::clone_server::poll_test_hook_pub(state_for_hook, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                if let Err(e) = result {
                    let cause = format!("cyberint server error: {e}");
                    let _ = crash_tx.send(Some(cause));
                }
            }
            _ = hook_future => {}
        }
    });

    StartedClone {
        addr,
        handle,
        admin_token,
        state: clone_state,
    }
}

/// Run the Cyberint clone axum server until the shutdown signal fires.
async fn run_cyberint_server(
    listener: tokio::net::TcpListener,
    router: Router,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), anyhow::Error> {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .map_err(|e| anyhow::anyhow!("cyberint axum serve error: {e}"))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> CyberintCloneState {
        CyberintCloneState::new("test-org", DEFAULT_SEED)
    }

    /// SEC-002 / CWE-20: register_access_token must reject tokens containing ASCII
    /// control characters (CTL: 0x00–0x1F, DEL 0x7F) or exceeding 4096 bytes.
    ///
    /// Mirrors the E-AUTH-006 validation contract and the SEC-002 guards applied to
    /// `prism-dtu-cyberint::state::CyberintState::register_access_token`.
    /// A harness allowlist that accepts CTL-containing or oversized tokens would break
    /// DTU↔production fidelity and introduce CWE-93/113/400 vectors via the admin-guarded
    /// `POST /dtu/configure` endpoint (F-P7-HIGH-001).
    ///
    /// Verifies: register_access_token with invalid token does NOT grow the allowlist.
    #[test]
    fn test_harness_register_access_token_rejects_ctl_and_oversized_tokens() {
        let state = make_state();

        // ---- Case 1: CTL byte (0x01) ----
        let ctl_token = "abc\x01def".to_string();
        state.register_access_token(ctl_token.clone());
        assert!(
            !state.is_valid_access_token(&ctl_token),
            "SEC-002 / CWE-20: register_access_token must NOT accept tokens containing \
             ASCII control bytes (0x01 CTL). Harness allowlist accepted a CTL token."
        );

        // ---- Case 2: CRLF injection (CR = 0x0D, LF = 0x0A) ----
        let crlf_token = "valid-prefix\r\nInjected: header".to_string();
        state.register_access_token(crlf_token.clone());
        assert!(
            !state.is_valid_access_token(&crlf_token),
            "SEC-002 / CWE-113: register_access_token must NOT accept tokens containing \
             CRLF (HTTP header injection). Harness allowlist accepted a CRLF token."
        );

        // ---- Case 3: DEL (0x7F) ----
        let del_token = "abc\x7fdef".to_string();
        state.register_access_token(del_token.clone());
        assert!(
            !state.is_valid_access_token(&del_token),
            "SEC-002 / CWE-20: register_access_token must NOT accept tokens containing \
             DEL (0x7F). Harness allowlist accepted a DEL token."
        );

        // ---- Case 4: oversized token (> 4096 bytes) ----
        let oversized_token = "a".repeat(4097);
        state.register_access_token(oversized_token.clone());
        assert!(
            !state.is_valid_access_token(&oversized_token),
            "SEC-002 / CWE-400: register_access_token must NOT accept tokens exceeding \
             4096 bytes. Harness allowlist accepted an oversized token."
        );

        // ---- Sanity: valid token still accepted ----
        let valid_token = "valid-api-key-abcdef123".to_string();
        state.register_access_token(valid_token.clone());
        assert!(
            state.is_valid_access_token(&valid_token),
            "SEC-002 sanity: a valid ASCII token must still be accepted after adding \
             CTL/oversized rejection guards."
        );
    }

    /// SEC-002 / CWE-400: register_access_token must enforce MAX_ALLOWLIST_SIZE.
    ///
    /// Inserting more than MAX_ALLOWLIST_SIZE distinct valid tokens must not grow the
    /// access_token_store past the cap. This bounds unbounded HashSet growth via the
    /// admin-guarded `POST /dtu/configure` endpoint (F-P7-HIGH-001; CWE-400).
    ///
    /// ADR-031 §D3-a; SEC-002 (S-DTU-CYBERINT-AUTH-FIDELITY-001; F-P7-HIGH-001).
    #[test]
    fn test_harness_register_access_token_caps_at_max_allowlist_size() {
        // Start from a clean state (no pre-registered demo token, so cap starts at 0).
        let state = CyberintCloneState::with_demo_token(
            "test-org",
            DEFAULT_SEED,
            // Use the constant demo token so we start with exactly 1 entry.
            DEMO_ACCESS_TOKEN.to_owned(),
        );

        // Insert MAX_ALLOWLIST_SIZE + 10 additional distinct valid tokens.
        for i in 0..=(MAX_HARNESS_ALLOWLIST_SIZE + 10) {
            state.register_access_token(format!("bulk-token-{i:04}"));
        }

        // SAFETY: mutex poison only occurs if a previous holder panicked.
        #[allow(clippy::expect_used)]
        let count = state
            .access_token_store
            .lock()
            .expect("access_token_store poisoned")
            .len();

        assert!(
            count <= MAX_HARNESS_ALLOWLIST_SIZE,
            "SEC-002 / CWE-400: access_token_store must not exceed MAX_HARNESS_ALLOWLIST_SIZE \
             ({MAX_HARNESS_ALLOWLIST_SIZE}). Got {count} entries (F-P7-HIGH-001)."
        );
    }

    /// Verify the pre-registered DEMO_ACCESS_TOKEN passes the new validation guards.
    ///
    /// Regression guard: if the demo token were to contain CTL chars or exceed 4096 bytes,
    /// all tests relying on it for cookie auth would silently break.
    #[test]
    fn test_harness_demo_token_is_valid_under_new_guards() {
        let state = make_state();
        assert!(
            state.is_valid_access_token(DEMO_ACCESS_TOKEN),
            "DEMO_ACCESS_TOKEN must pass the SEC-002 validation guards and be registered \
             in the allowlist at startup."
        );
    }
}
