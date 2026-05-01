//! Cyberint-specific harness clone router.
//!
//! Self-contained axum Router factory for `DtuType::Cyberint`.  Provides all
//! Cyberint-specific routes alongside the shared DTU control endpoints:
//!
//! # Cyberint routes
//!
//! - `POST /login`                               — cookie-based auth, issues `cyberint_session=<token>`
//! - `GET  /api/v1/alerts`                       — paginated alert list (requires cookie)
//! - `POST /api/v1/alerts`                       — alias for GET alerts
//! - `GET  /api/v1/alerts/:alert_id`             — alert detail (requires cookie)
//! - `PATCH /api/v1/alerts/:alert_id/status`     — status transition (requires cookie)
//! - `POST  /api/v1/alerts/:alert_id/close`      — irreversible close (requires cookie)
//! - `GET  /api/v1/threat-intel`                 — threat-intel feed (requires cookie)
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
//! # Cookie auth
//!
//! Cyberint uses cookie-based auth (`cyberint_session=<token>`) rather than the
//! bearer-token pattern used by other harness DTUs.  The per-clone `CyberintCloneState`
//! holds a `session_store: HashSet<String>` (tokens only; no org-keying needed in
//! single-instance harness clones).
//!
//! # Architecture Anchors
//!
//! - S-3.4.04 — Cyberint harness migration story
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants
//! - BC-3.6.001 — Per-Org Failure Injection

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::clone_server::{CloneState, StartedClone};
use prism_dtu_common::FailureMode;

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
/// and admin token validation) — this holds Cyberint-specific state: session tokens,
/// alert statuses, auth mode, and rate-limit config.
pub struct CyberintCloneState {
    /// Valid session tokens (opaque strings issued by `POST /login`).
    pub session_store: Mutex<HashSet<String>>,

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
}

impl CyberintCloneState {
    /// Construct state for a clone identified by `org_slug` and `seed`.
    ///
    /// When `seed == DEFAULT_SEED`, alert IDs use the canonical fixture format
    /// `CYB-2024-NNN` (backward compat with single-org AC tests).
    /// Otherwise, alert IDs use `alert-{org_slug}-{seed}-{index}`.
    pub fn new(org_slug: &str, seed: u64) -> Self {
        let (page1, page2) = generate_alerts(org_slug, seed);
        let threat_intel = generate_threat_intel(org_slug, seed);

        // Pre-populate alert_store from fixture (all "open", not closed).
        let mut alert_store: HashMap<String, AlertStatusRecord> = HashMap::new();
        for a in page1.iter().chain(page2.iter()) {
            alert_store.insert(a.alert_id.clone(), AlertStatusRecord::open());
        }

        Self {
            session_store: Mutex::new(HashSet::new()),
            alert_store: Mutex::new(alert_store),
            auth_mode: Mutex::new(AuthMode::Accept),
            rate_limit_after: Mutex::new(None),
            auth_request_count: Mutex::new(0),
            alerts_page1: page1,
            alerts_page2: page2,
            threat_intel,
        }
    }

    /// Reset all mutable state to initial values.
    ///
    /// - Clears session_store.
    /// - Resets all alert statuses to "open" / not closed.
    /// - Resets auth_mode to Accept.
    /// - Resets rate_limit_after to None.
    /// - Resets auth_request_count to 0.
    #[allow(clippy::expect_used)]
    pub fn reset(&self) {
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .clear();

        let mut store = self.alert_store.lock().expect("alert_store poisoned");
        for val in store.values_mut() {
            *val = AlertStatusRecord::open();
        }
        drop(store);

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

    #[allow(clippy::expect_used)]
    fn register_session(&self, token: String) {
        self.session_store
            .lock()
            .expect("session_store poisoned")
            .insert(token);
    }

    #[allow(clippy::expect_used)]
    fn is_valid_session(&self, token: &str) -> bool {
        self.session_store
            .lock()
            .expect("session_store poisoned")
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
    /// Cyberint-specific mutable state (sessions, alert statuses, auth mode).
    pub cyberint: Arc<CyberintCloneState>,
    /// Generic clone state (failure injection, admin token, request counter).
    pub clone_state: Arc<CloneState>,
}

// ---------------------------------------------------------------------------
// Cookie auth helpers
// ---------------------------------------------------------------------------

/// Extract the `cyberint_session` token from the `Cookie` header.
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(token) = part.strip_prefix("cyberint_session=") {
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
#[allow(clippy::result_large_err)]
fn check_auth(
    state: &CyberintCloneState,
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    // auth_mode=reject: always 401 regardless of cookie (EC-006).
    if state.auth_mode() == AuthMode::Reject {
        return Err(unauthorized());
    }

    let token = extract_session_token(headers).ok_or_else(unauthorized)?;
    if !state.is_valid_session(&token) {
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

/// `POST /login`
///
/// Issues a UUID session token as `Set-Cookie: cyberint_session=<token>`.
/// No auth required.
#[allow(clippy::expect_used)]
async fn post_login(State(state): State<Arc<CyberintRouteState>>) -> impl IntoResponse {
    let token = uuid::Uuid::new_v4().to_string();
    state.cyberint.register_session(token.clone());

    let cookie_value = format!("cyberint_session={token}; Path=/; HttpOnly");

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_value)],
        Json(json!({"message": "Login successful"})),
    )
        .into_response()
}

/// `GET /api/v1/alerts` (and `POST /api/v1/alerts`)
///
/// Returns paginated alerts from the in-memory fixture. Requires cookie auth.
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
/// Returns alert detail with current status. Requires cookie auth.
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
/// Threat intelligence feed. Requires cookie auth.
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
/// Also accepts Cyberint-specific config fields like `auth_mode` and `rate_limit_after`.
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
/// Resets all Cyberint clone state (sessions, alert statuses, auth mode, counters).
/// Also resets the generic CloneState request counter and failure mode.
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
// Network-mode: cookie-aware auth check
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
/// Routes handle cookie-based auth, alert lifecycle, threat-intel, and DTU
/// control endpoints.  The `clone_state` provides the generic failure-injection
/// machinery and admin token; the `cyberint_state` provides Cyberint-specific
/// session/alert state.
fn build_cyberint_router(
    clone_state: Arc<CloneState>,
    cyberint_state: Arc<CyberintCloneState>,
) -> Router {
    let route_state = Arc::new(CyberintRouteState {
        cyberint: cyberint_state,
        clone_state,
    });

    Router::new()
        // Cookie auth
        .route("/login", post(post_login))
        // Alert routes
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
        .route("/login", post(post_login))
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
/// Creates a `CyberintCloneState` from `org_slug` and `seed`, wires it alongside
/// the generic `CloneState`, builds the Cyberint router, and spawns the server.
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
