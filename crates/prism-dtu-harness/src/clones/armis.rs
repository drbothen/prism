//! Self-contained Armis Centrix DTU behavioral clone for the harness.
//!
//! This module provides a complete Armis API implementation suitable for
//! multi-tenant harness testing. It is intentionally self-contained — it does
//! NOT import from `prism-dtu-armis` to avoid circular dev-dependency chains.
//!
//! All fixture data is embedded as `include_str!` constants so the harness
//! compiles independently without referencing the Armis crate's `fixtures/`
//! directory at runtime.
//!
//! # Routes served
//!
//! | Method | Path | Notes |
//! |--------|------|-------|
//! | GET    | /api/v1/devices | Device list; Bearer auth required (403 if missing) |
//! | POST   | /api/v1/devices | Device list (body AQL); Bearer auth required |
//! | GET    | /api/v1/devices/:id/activity | Activity log; Bearer auth required |
//! | GET    | /api/v1/devices/:id/risk | Risk score; Bearer auth required |
//! | GET    | /api/v1/alerts | Alert list; Bearer auth required |
//! | POST   | /api/v1/devices/:id/tags/ | Add tag; Bearer auth required |
//! | DELETE | /api/v1/devices/:id/tags/:key | Remove tag; Bearer auth required |
//! | GET    | /dtu/aql-log | AQL capture log; no auth required |
//! | POST   | /dtu/configure | Failure injection; X-Admin-Token required |
//! | POST   | /dtu/reset | Clear mutable state; no auth required |
//! | GET    | /dtu/health | Liveness; no auth required |
//!
//! # Auth behaviour
//!
//! Armis returns **HTTP 403** (not 401) for missing/malformed Bearer tokens.
//! This is intentional per AC-5 and the Armis API spec.
//!
//! # Isolation (BC-3.5.001 postcondition 2)
//!
//! Device IDs are prefixed with `{org_slug}-` when the org slug is not the
//! canonical "test-tenant" slug. This ensures that when two harness orgs are
//! started simultaneously, their device ID sets are pairwise-disjoint, satisfying
//! BC-3.5.001 postcondition 2 (TV-2; VP-122, VP-123; AC-003).
//!
//! The "test-tenant" slug is treated as the canonical test org and serves fixture
//! device IDs verbatim (`d-001`..`d-025`), allowing AC-002 timestamp-fallback tests
//! to locate `d-001` by its exact ID. All other org slugs receive IDs prefixed with
//! the org slug (e.g. `other-tenant-d-001`), which are disjoint from `d-001..d-025`.
//!
//! # Architecture Anchors
//!
//! - S-3.4.02 — Armis harness migration
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants
//! - ADR-011 §2.2 — in-process org-keyed routing

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::Deserialize;
use serde_json::{json, Value};

// Re-export TestHookSignal for use in start_armis_clone crash monitoring.
pub use crate::clone_server::TestHookSignal;

// ---------------------------------------------------------------------------
// Embedded fixture data
// ---------------------------------------------------------------------------

/// 25 Armis device records (includes d-001 with last_seen:null for timestamp-fallback test).
const DEVICES_FIXTURE: &str = include_str!("../../../prism-dtu-armis/fixtures/devices.json");

/// 12+ Armis alert records.
const ALERTS_FIXTURE: &str = include_str!("../../../prism-dtu-armis/fixtures/alerts.json");

/// Device activity records.
const ACTIVITY_FIXTURE: &str =
    include_str!("../../../prism-dtu-armis/fixtures/device-activity.json");

// ---------------------------------------------------------------------------
// Clone state
// ---------------------------------------------------------------------------

/// Shared mutable state for one Armis harness clone instance.
///
/// One instance exists per `(OrgId, DtuType::Armis)` pair in the harness.
/// State is never shared across org boundaries (BC-3.5.001 postcondition 2).
pub struct ArmisHarnessState {
    /// Org slug — used for device ID scoping (BC-3.5.001 postcondition 2).
    pub org_slug: String,
    /// Per-device tag store keyed by device_id. Cleared on reset.
    ///
    /// In the harness context, each clone is already scoped to a single org,
    /// so no OrgId key is needed here.
    pub tag_store: Mutex<HashMap<String, HashSet<String>>>,
    /// AQL capture log: ordered list of AQL strings received since last reset.
    pub aql_log: Mutex<Vec<String>>,
    /// Monotonically increasing request counter for failure injection.
    pub request_counter: std::sync::atomic::AtomicU32,
    /// Current failure injection mode (updated via `/dtu/configure`).
    pub failure_mode: Mutex<FailureMode>,
    /// Admin shared-secret for `POST /dtu/configure` (TD-WV0-07).
    pub admin_token: String,
    /// Test hook signal for crash detection (BC-3.6.002).
    ///
    /// Polled by the background crash monitor task. When `Some(sig)` is set
    /// via a test-hook POST, the monitor propagates the crash cause.
    pub test_hook_signal: Mutex<Option<TestHookSignal>>,
}

impl ArmisHarnessState {
    /// Create state with a specific admin token and org slug.
    pub fn new(org_slug: String, admin_token: String) -> Self {
        Self {
            org_slug,
            tag_store: Mutex::new(HashMap::new()),
            aql_log: Mutex::new(Vec::new()),
            request_counter: std::sync::atomic::AtomicU32::new(0),
            failure_mode: Mutex::new(FailureMode::None),
            admin_token,
            test_hook_signal: Mutex::new(None),
        }
    }

    /// Increment the request counter and return the new 1-indexed value.
    pub fn increment_counter(&self) -> u32 {
        self.request_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1
    }

    #[allow(clippy::expect_used)]
    pub fn current_failure_mode(&self) -> FailureMode {
        self.failure_mode
            .lock()
            .expect("failure_mode poisoned")
            .clone()
    }

    #[allow(clippy::expect_used)]
    pub fn set_failure_mode(&self, mode: FailureMode) {
        *self.failure_mode.lock().expect("failure_mode poisoned") = mode;
    }

    #[allow(clippy::expect_used)]
    pub fn reset(&self) {
        self.tag_store.lock().expect("tag_store poisoned").clear();
        self.aql_log.lock().expect("aql_log poisoned").clear();
        self.request_counter
            .store(0, std::sync::atomic::Ordering::SeqCst);
        self.set_failure_mode(FailureMode::None);
    }

    #[allow(clippy::expect_used)]
    pub fn capture_aql(&self, aql: &str) {
        self.aql_log
            .lock()
            .expect("aql_log poisoned")
            .push(aql.to_owned());
    }

    #[allow(clippy::expect_used)]
    pub fn aql_log_snapshot(&self) -> Vec<String> {
        self.aql_log.lock().expect("aql_log poisoned").clone()
    }

    #[allow(clippy::expect_used)]
    pub fn add_tag(&self, device_id: &str, tag_key: &str) -> bool {
        self.tag_store
            .lock()
            .expect("tag_store poisoned")
            .entry(device_id.to_string())
            .or_default()
            .insert(tag_key.to_string())
    }

    /// Returns `true` if the tag existed and was removed, `false` otherwise.
    #[allow(clippy::expect_used)]
    pub fn remove_tag(&self, device_id: &str, tag_key: &str) -> bool {
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        if let Some(tags) = store.get_mut(device_id) {
            tags.remove(tag_key)
        } else {
            false
        }
    }

    #[allow(clippy::expect_used)]
    pub fn get_tags(&self, device_id: &str) -> Vec<String> {
        self.tag_store
            .lock()
            .expect("tag_store poisoned")
            .get(device_id)
            .map(|s| {
                let mut v: Vec<String> = s.iter().cloned().collect();
                v.sort();
                v
            })
            .unwrap_or_default()
    }

    /// Scope a device ID for org isolation (BC-3.5.001 postcondition 2).
    ///
    /// The canonical "test-tenant" org serves fixture device IDs verbatim (e.g. `d-001`)
    /// so that AC-002 timestamp-fallback tests can find the fixture device by exact ID.
    ///
    /// All other org slugs have their device IDs prefixed with `{org_slug}-`, making
    /// device ID sets pairwise-disjoint across orgs (AC-003; VP-123).
    pub fn scope_device_id(&self, raw_id: &str) -> String {
        if self.org_slug == "test-tenant" {
            raw_id.to_owned()
        } else {
            format!("{}-{}", self.org_slug, raw_id)
        }
    }
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Check the `Authorization: Bearer` header against the expected admin token.
///
/// - Missing or malformed Bearer → **HTTP 403** (Armis AC-5 spec).
/// - Bearer present but token value does not match `expected_token` → **HTTP 401**
///   (cross-org credential mismatch; BC-3.5.002 postcondition 2 / VP-126 / AC-004).
/// - Bearer present and matches → `None` (request proceeds).
fn check_bearer_auth(
    headers: &HeaderMap,
    expected_token: &str,
) -> Option<axum::response::Response> {
    match headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        None => Some(
            (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "invalid or missing bearer token",
                    "code": 403
                })),
            )
                .into_response(),
        ),
        Some(token) if token == expected_token => None,
        Some(_) => Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid bearer token (cross-org credential rejected)",
                    "code": 401
                })),
            )
                .into_response(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Failure mode application
// ---------------------------------------------------------------------------

/// Apply the current failure mode. Returns `Some(response)` if a failure should be served.
fn apply_failure_mode(mode: &FailureMode, n: u32) -> Option<axum::response::Response> {
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "auth rejected by failure mode", "code": 401})),
            )
                .into_response(),
        ),
        FailureMode::InternalError { at_request_n } => {
            if n == *at_request_n {
                Some(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "internal server error (injected)", "code": 500})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => {
            if n > *after_n_requests {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "rate limit exceeded", "code": 429})),
                )
                    .into_response();
                #[allow(clippy::expect_used)]
                resp.headers_mut().insert(
                    "retry-after",
                    retry_after_secs
                        .to_string()
                        .parse()
                        .expect("retry_after_secs is a valid header value"),
                );
                Some(resp)
            } else {
                None
            }
        }
        FailureMode::NetworkTimeout { after_ms } => {
            if *after_ms > 0 {
                // For harness tests, treat network timeout as a no-op (can't sleep easily here).
                None
            } else {
                None
            }
        }
        FailureMode::Unprocessable { at_request_n } => {
            if n == *at_request_n {
                Some(
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"error": "unprocessable entity (injected)", "code": 422})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
        FailureMode::MalformedResponse =>
        {
            #[allow(clippy::expect_used)]
            Some(
                axum::response::Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        b"\xff\xfe{not valid json!@#$%^&*(" as &[u8],
                    ))
                    .expect("build malformed response"),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Request body / query types
// ---------------------------------------------------------------------------

/// Query parameters for `GET /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
struct DeviceQueryParams {
    aql: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

/// JSON body for `POST /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
struct DeviceQueryBody {
    aql: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

/// Query parameters for `GET /api/v1/alerts`.
#[derive(Debug, Deserialize, Default)]
struct AlertQueryParams {
    page: Option<u32>,
    size: Option<u32>,
}

/// JSON body for `POST /dtu/configure`.
///
/// Accepts BOTH:
/// - The Armis-native format (`failure_mode: "rate_limit"`, etc.) used by
///   direct test calls in harness_tests.rs (AC-006).
/// - The harness format (`auth_mode: "reject"`, `rate_limit_after: N`, etc.)
///   used by `Harness::inject_failure` via `failure_mode_to_json` in harness.rs.
///
/// `deny_unknown_fields` rejects unknown keys (TD-WV0-04).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DtuConfigureBody {
    // --- Armis-native format fields ---
    /// `"none"`, `"rate_limit"`, `"malformed_response"`, etc.
    failure_mode: Option<String>,
    /// Companion for Armis-native `"rate_limit"` mode.
    after_n_requests: Option<u32>,
    /// Companion for Armis-native `"rate_limit"` mode.
    retry_after_secs: Option<u32>,
    /// Companion for Armis-native `"internal_error"` mode.
    at_request_n: Option<u32>,
    /// Companion for Armis-native `"network_timeout"` mode (ms).
    after_ms: Option<u64>,

    // --- Harness format fields (used by Harness::inject_failure → failure_mode_to_json) ---
    /// `"reject"` → AuthReject; harness format.
    auth_mode: Option<String>,
    /// Rate limit after N requests; harness format.
    rate_limit_after: Option<u32>,
    /// Inject 500 at this request number; harness format.
    internal_error_at: Option<u32>,
    /// Delay in ms for NetworkTimeout; harness format.
    network_timeout_ms: Option<u64>,
    /// Serve malformed (non-JSON) response body; harness format.
    malformed_response: Option<bool>,
    /// Inject 422 at this request number; harness format.
    unprocessable_at: Option<u32>,
    /// Clear all failure modes; harness format.
    clear: Option<bool>,
}

// ---------------------------------------------------------------------------
// Device helper: load + scope
// ---------------------------------------------------------------------------

/// Load devices from fixture, merge tag state, scope device IDs, paginate.
fn device_page(state: &ArmisHarnessState, page: u32, size: u32) -> axum::response::Response {
    #[allow(clippy::expect_used)]
    let mut devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON");

    // Scope device IDs and merge tag state.
    for device in &mut devices {
        let raw_id = device["device_id"].as_str().unwrap_or_default().to_owned();
        let scoped_id = state.scope_device_id(&raw_id);

        // Update device_id to the scoped version.
        device["device_id"] = Value::String(scoped_id.clone());

        // Merge tag store into tags array.
        let mut fixture_tags: Vec<String> = device["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                    .collect()
            })
            .unwrap_or_default();
        let store_tags = state.get_tags(&scoped_id);
        for t in store_tags {
            if !fixture_tags.contains(&t) {
                fixture_tags.push(t);
            }
        }
        device["tags"] = Value::Array(fixture_tags.into_iter().map(Value::String).collect());
    }

    let total = devices.len() as u32;
    let page = page.max(1);
    let size = (size as usize).max(1);
    let offset = ((page - 1) as usize) * size;

    let page_devices: Vec<Value> = if offset >= devices.len() {
        vec![]
    } else {
        devices[offset..std::cmp::min(offset + size, devices.len())].to_vec()
    };

    Json(json!({
        "data": {
            "devices": page_devices,
            "total": total,
            "page": page
        }
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/devices` — device list with optional AQL query param.
async fn get_devices(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
) -> axum::response::Response {
    // Failure injection fires BEFORE bearer auth so that FailureMode::AuthReject
    // (which returns 401) takes precedence over the bearer-absent 403.
    // This matches the generic clone_server's behaviour and the test contract
    // (BC-3.6.001 invariant 1; CR-011).
    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    // Capture AQL verbatim (R-DTU-002).
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    let page = params.page.unwrap_or(1);
    let size = params.size.unwrap_or(25);
    device_page(&state, page, size)
}

/// `POST /api/v1/devices` — device list with AQL in JSON body (EC-005).
async fn post_devices(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
    body: Option<Json<DeviceQueryBody>>,
) -> axum::response::Response {
    // Failure injection fires BEFORE bearer auth (CR-011; BC-3.6.001 invariant 1).
    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    // AQL: body takes priority over query param.
    let aql = body
        .as_ref()
        .and_then(|Json(b)| b.aql.clone())
        .or_else(|| params.aql.clone());

    if let Some(ref aql_str) = aql {
        state.capture_aql(aql_str);
    }

    let page = body
        .as_ref()
        .and_then(|Json(b)| b.page)
        .or(params.page)
        .unwrap_or(1);
    let size = body
        .as_ref()
        .and_then(|Json(b)| b.size)
        .or(params.size)
        .unwrap_or(25);

    device_page(&state, page, size)
}

/// `GET /api/v1/devices/:device_id/activity`
async fn get_device_activity(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Path(_device_id): Path<String>,
) -> axum::response::Response {
    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let activities: Vec<Value> =
        serde_json::from_str(ACTIVITY_FIXTURE).expect("ACTIVITY_FIXTURE is valid JSON");
    let total = activities.len() as u32;

    Json(json!({
        "data": {
            "activities": activities,
            "total": total
        }
    }))
    .into_response()
}

/// `GET /api/v1/devices/:device_id/risk`
///
/// EC-002: unknown device → HTTP 404.
async fn get_device_risk(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> axum::response::Response {
    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    // Load devices and look up the requested device_id (scoped).
    #[allow(clippy::expect_used)]
    let devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON");

    // The incoming device_id is the scoped form. Reverse-scope to find the fixture entry.
    // For "test-tenant": scoped_id == raw_id. For others: scoped_id == "{org_slug}-{raw_id}".
    let raw_id = if state.org_slug == "test-tenant" {
        device_id.clone()
    } else {
        let prefix = format!("{}-", state.org_slug);
        device_id
            .strip_prefix(&prefix)
            .unwrap_or(&device_id)
            .to_owned()
    };

    match devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some(&raw_id))
    {
        Some(device) => {
            let risk_score = device["risk_score"].as_u64().unwrap_or(0) as u32;
            let risk_factors: Vec<Value> = device["risk_factors"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            Json(json!({
                "data": {
                    "device_id": device_id,
                    "risk_score": risk_score,
                    "risk_factors": risk_factors
                }
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "device not found", "code": 404})),
        )
            .into_response(),
    }
}

/// `GET /api/v1/alerts`
async fn get_alerts(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Query(params): Query<AlertQueryParams>,
) -> axum::response::Response {
    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let all_alerts: Vec<Value> =
        serde_json::from_str(ALERTS_FIXTURE).expect("ALERTS_FIXTURE is valid JSON");

    let total = all_alerts.len() as u32;
    let page = params.page.unwrap_or(1).max(1);
    let size = (params.size.unwrap_or(25) as usize).max(1);
    let offset = ((page - 1) as usize) * size;

    let page_alerts: Vec<Value> = if offset >= all_alerts.len() {
        vec![]
    } else {
        all_alerts[offset..std::cmp::min(offset + size, all_alerts.len())].to_vec()
    };

    Json(json!({
        "data": {
            "alerts": page_alerts,
            "total": total
        }
    }))
    .into_response()
}

/// `POST /api/v1/devices/:device_id/tags/`
///
/// Requires Bearer auth; returns 201 with `{device_id, tag_key, status: "added"}`.
async fn post_device_tag(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    let tag_key = body
        .get("tag_key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    state.add_tag(&device_id, &tag_key);

    (
        StatusCode::CREATED,
        Json(json!({
            "device_id": device_id,
            "tag_key": tag_key,
            "status": "added"
        })),
    )
        .into_response()
}

/// `DELETE /api/v1/devices/:device_id/tags/:tag_key`
///
/// Requires Bearer auth. Returns 200 `{status: "removed"}` or 404 if tag not found (EC-003).
async fn delete_device_tag(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Path((device_id, tag_key)): Path<(String, String)>,
) -> axum::response::Response {
    if let Some(err) = check_bearer_auth(&headers, &state.admin_token) {
        return err;
    }

    if state.remove_tag(&device_id, &tag_key) {
        Json(json!({"status": "removed"})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tag not found", "code": 404})),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// DTU-internal routes
// ---------------------------------------------------------------------------

/// `GET /dtu/aql-log` — returns all AQL strings captured since last reset.
///
/// No auth required (DTU-internal route per ADR-002 §6).
async fn get_aql_log(State(state): State<Arc<ArmisHarnessState>>) -> (StatusCode, Json<Value>) {
    let aql_strings = state.aql_log_snapshot();
    (StatusCode::OK, Json(json!({"aql_strings": aql_strings})))
}

/// `POST /dtu/configure` — failure injection endpoint.
///
/// Requires `X-Admin-Token` header (TD-WV0-07).
/// Unknown fields return HTTP 400 (TD-WV0-04, `deny_unknown_fields`).
async fn dtu_configure(
    State(state): State<Arc<ArmisHarnessState>>,
    headers: HeaderMap,
    Json(raw): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Admin token check (TD-WV0-07).
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        );
    }

    // Deny unknown fields (TD-WV0-04).
    let body: DtuConfigureBody = match serde_json::from_value(raw) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid /dtu/configure payload: {e}")})),
            );
        }
    };

    // Determine failure mode. Check harness-format fields first (used by inject_failure),
    // then fall back to Armis-native `failure_mode` string format (used by direct test calls).
    let mode = if body.clear == Some(true) {
        FailureMode::None
    } else if body.auth_mode.as_deref() == Some("reject") {
        // Harness format: { "auth_mode": "reject" }
        FailureMode::AuthReject
    } else if let Some(n) = body.rate_limit_after {
        // Harness format: { "rate_limit_after": N, "retry_after_secs": M }
        FailureMode::RateLimit {
            after_n_requests: n,
            retry_after_secs: body.retry_after_secs.unwrap_or(60),
        }
    } else if let Some(n) = body.internal_error_at {
        // Harness format: { "internal_error_at": N }
        FailureMode::InternalError { at_request_n: n }
    } else if let Some(ms) = body.network_timeout_ms {
        // Harness format: { "network_timeout_ms": N }
        FailureMode::NetworkTimeout { after_ms: ms }
    } else if body.malformed_response == Some(true) {
        // Harness format: { "malformed_response": true }
        FailureMode::MalformedResponse
    } else if let Some(n) = body.unprocessable_at {
        // Harness format: { "unprocessable_at": N }
        FailureMode::Unprocessable { at_request_n: n }
    } else if body.auth_mode.as_deref() == Some("none") {
        // Harness format: { "auth_mode": "none" } → clear
        FailureMode::None
    } else {
        // Armis-native format: { "failure_mode": "..." }
        match body.failure_mode.as_deref() {
            Some("none") | None => FailureMode::None,
            Some("rate_limit") => FailureMode::RateLimit {
                after_n_requests: body.after_n_requests.unwrap_or(0),
                retry_after_secs: body.retry_after_secs.unwrap_or(30),
            },
            Some("malformed_response") => FailureMode::MalformedResponse,
            Some("auth_reject") => FailureMode::AuthReject,
            Some("internal_error") => FailureMode::InternalError {
                at_request_n: body.at_request_n.unwrap_or(1),
            },
            Some("network_timeout") => FailureMode::NetworkTimeout {
                after_ms: body.after_ms.unwrap_or(5000),
            },
            Some(other) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("unknown failure_mode: {other}")})),
                );
            }
        }
    };

    // Reset request counter when setting a new mode (fresh count).
    state
        .request_counter
        .store(0, std::sync::atomic::Ordering::SeqCst);
    state.set_failure_mode(mode);

    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// `POST /dtu/reset` — clear all mutable state.
///
/// Clears: tag store, AQL log, request counter, failure mode.
/// Fixture data (devices, alerts, activity) is NOT affected.
/// No auth required.
async fn dtu_reset(State(state): State<Arc<ArmisHarnessState>>) -> (StatusCode, Json<Value>) {
    state.reset();
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// `GET /dtu/health` — liveness check.
async fn dtu_health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Test hook endpoints (BC-3.6.002 crash detection)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PanicBody {
    message: String,
}

/// `POST /dtu/test-hook/panic` — trigger controlled panic for crash detection tests.
#[allow(clippy::expect_used)]
async fn test_hook_panic(
    State(state): State<Arc<ArmisHarnessState>>,
    Json(body): Json<PanicBody>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::Panic(body.message));
    (StatusCode::OK, Json(json!({"status": "panic queued"})))
}

/// `POST /dtu/test-hook/premature-ok` — trigger premature clean exit.
#[allow(clippy::expect_used)]
async fn test_hook_premature_ok(
    State(state): State<Arc<ArmisHarnessState>>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::PrematureOk);
    (
        StatusCode::OK,
        Json(json!({"status": "premature-ok queued"})),
    )
}

/// `POST /dtu/test-hook/non-string-panic` — trigger non-string panic payload.
#[allow(clippy::expect_used)]
async fn test_hook_non_string_panic(
    State(state): State<Arc<ArmisHarnessState>>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::NonStringPanic);
    (
        StatusCode::OK,
        Json(json!({"status": "non-string-panic queued"})),
    )
}

/// Poll the test hook signal on `ArmisHarnessState` and propagate crash causes.
///
/// Mirrors `clone_server::poll_test_hook` but uses `ArmisHarnessState`.
/// Called concurrently with the axum server in `start_armis_clone`.
#[allow(clippy::expect_used)]
pub async fn poll_armis_test_hook(
    state: Arc<ArmisHarnessState>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
) {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let signal = state
            .test_hook_signal
            .lock()
            .expect("test_hook_signal poisoned")
            .clone();

        if let Some(sig) = signal {
            match sig {
                TestHookSignal::Panic(msg) => {
                    let _ = crash_tx.send(Some(msg));
                    return;
                }
                TestHookSignal::PrematureOk => {
                    let _ =
                        crash_tx.send(Some(crate::crash_monitor::PREMATURE_OK_CAUSE.to_string()));
                    return;
                }
                TestHookSignal::NonStringPanic => {
                    let _ = crash_tx.send(Some(
                        crate::crash_monitor::NON_STRING_PANIC_CAUSE.to_string(),
                    ));
                    return;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Router constructor
// ---------------------------------------------------------------------------

/// Build the Armis clone axum `Router` for the given state.
///
/// Called by `clone_server::start_clone` when `DtuType::Armis` is dispatched.
/// The state is constructed with the org slug and admin token so that device IDs
/// can be scoped for multi-tenant isolation (BC-3.5.001 postcondition 2).
pub fn router(state: Arc<ArmisHarnessState>) -> Router {
    Router::new()
        // Vendor API endpoints (all require Bearer auth → 403 if missing)
        .route("/api/v1/devices", get(get_devices))
        .route("/api/v1/devices", post(post_devices))
        .route(
            "/api/v1/devices/:device_id/activity",
            get(get_device_activity),
        )
        .route("/api/v1/devices/:device_id/risk", get(get_device_risk))
        .route("/api/v1/alerts", get(get_alerts))
        .route("/api/v1/devices/:device_id/tags/", post(post_device_tag))
        .route(
            "/api/v1/devices/:device_id/tags/:tag_key",
            delete(delete_device_tag),
        )
        // DTU-internal endpoints (no auth required)
        .route("/dtu/aql-log", get(get_aql_log))
        .route("/dtu/configure", post(dtu_configure))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/health", get(dtu_health))
        // Test hook endpoints (BC-3.6.002 crash detection)
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(state)
}
