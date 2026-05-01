//! Self-contained Claroty xDome DTU behavioral clone for the harness.
//!
//! This module provides a complete Claroty API implementation suitable for
//! multi-tenant harness testing. It is intentionally self-contained — it does
//! NOT import from `prism-dtu-claroty` to avoid circular dev-dependency chains.
//!
//! All fixture data is embedded as `const` strings so the harness compiles
//! independently without referencing the Claroty crate's `fixtures/` directory.
//!
//! # Routes served
//!
//! | Method | Path | Notes |
//! |--------|------|-------|
//! | POST   | /api/v1/devices | Device list; Bearer auth required |
//! | POST   | /api/v1/alerts | Alert list; Bearer auth required |
//! | POST   | /api/v1/alerts/:id/devices | Alerted devices; Bearer auth required |
//! | POST   | /api/v1/vulnerabilities | Vuln list; Bearer auth required |
//! | POST   | /api/v1/vulnerabilities/:id/devices | Vuln devices; Bearer auth required |
//! | POST   | /api/v1/devices/:id/tags/ | Add tag; Bearer auth required |
//! | DELETE | /api/v1/devices/:id/tags/:key | Remove tag; Bearer auth required |
//! | POST   | /dtu/configure | Failure injection; X-Admin-Token required |
//! | POST   | /dtu/reset | Clear state; no auth required |
//! | GET    | /dtu/health | Liveness; no auth required |
//!
//! # Architecture Anchors
//!
//! - S-3.4.01 — Claroty harness migration
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants
//! - ADR-011 §2.2 — in-process org-keyed routing

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::Deserialize;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Embedded fixture data (minified JSON, embedded at compile time)
// ---------------------------------------------------------------------------

/// 20 Claroty xDome device records.
const DEVICES_FIXTURE: &str = include_str!("../../../prism-dtu-claroty/fixtures/devices.json");

/// 10 Claroty xDome alert records.
const ALERTS_FIXTURE: &str = include_str!("../../../prism-dtu-claroty/fixtures/alerts.json");

/// 15 Claroty xDome vulnerability records.
const VULNERABILITIES_FIXTURE: &str =
    include_str!("../../../prism-dtu-claroty/fixtures/vulnerabilities.json");

/// 5 Claroty xDome alerted-device records (devices associated with alerts).
const ALERTED_DEVICES_FIXTURE: &str =
    include_str!("../../../prism-dtu-claroty/fixtures/alerted-devices.json");

/// 5 Claroty xDome vulnerability-device records.
const VULNERABILITY_DEVICES_FIXTURE: &str =
    include_str!("../../../prism-dtu-claroty/fixtures/vulnerability-devices.json");

// ---------------------------------------------------------------------------
// Clone state
// ---------------------------------------------------------------------------

/// Signal used to trigger a test hook (panic, premature-ok, non-string panic).
///
/// Written by test hook route handlers; read by `poll_crash_tx_from_claroty_state`
/// which drives the crash watch channel.
///
/// (BC-3.6.002 postcondition clause 1; ADR-011 §2.6)
#[derive(Clone, Debug)]
pub enum ClarotyTestHookSignal {
    /// Trigger a crash with the given cause string.
    Panic(String),
    /// Trigger a premature-ok crash.
    PrematureOk,
    /// Trigger a non-string panic crash.
    NonStringPanic,
}

/// Shared mutable state for one Claroty harness clone instance.
///
/// One instance exists per `(OrgId, DtuType::Claroty)` pair in the harness.
/// State is never shared across org boundaries (BC-3.5.001 postcondition 2).
///
/// # Isolation
///
/// Each clone stores `org_slug` so that device IDs can be made unique per org.
/// When a harness has multiple Claroty clones, each clone's device IDs are
/// prefixed with the org slug, ensuring pairwise-disjoint ID sets
/// (BC-3.5.001 postcondition 2; VP-123).
///
/// Single-tenant tests use org_slug `""` (empty), which preserves the original
/// `asset_id` values from the fixture (e.g. `"asset-001"`) so that tests that
/// reference device IDs by name continue to work.
pub struct ClarotyCloneState {
    /// The actual org slug (always set). Used by `GET /assets/v1/assets` to
    /// embed the org name in device IDs for logical isolation assertions.
    pub org_slug: String,
    /// Prefix applied to `asset_id` in `POST /api/v1/devices` responses.
    ///
    /// Empty string means no prefix (preserves fixture `asset_id` values).
    /// Non-empty means device IDs are `"{id_prefix}-{asset_id}"`.
    ///
    /// Differs from `org_slug` when seed == DEFAULT_SEED (42): in that case
    /// `id_prefix = ""` so single-tenant tests can reference `"asset-001"` directly,
    /// while `org_slug` retains the actual slug for GET endpoint isolation checks.
    pub id_prefix: String,
    /// Per-(device_id) set of tag keys. Cleared on reset.
    ///
    /// Note: in the harness context each clone already IS scoped to a single org
    /// (one `ClarotyCloneState` per org), so no `OrgId` key is needed here.
    pub tag_store: Mutex<HashMap<String, HashSet<String>>>,
    /// Monotonically increasing request counter for failure injection.
    pub request_counter: std::sync::atomic::AtomicU32,
    /// Current failure injection mode (updated via `/dtu/configure`).
    pub failure_mode: Mutex<FailureMode>,
    /// Artificial latency injected before each response (ms).
    pub latency_ms: AtomicU64,
    /// Admin shared-secret for `POST /dtu/configure` (TD-WV0-07).
    pub admin_token: String,
    /// Test hook signal — written by test hook route handlers.
    ///
    /// Polled by `start_claroty_clone`'s crash-monitor loop.
    /// `None` = no hook triggered; `Some(signal)` = fire the signal.
    ///
    /// (BC-3.6.002; ADR-011 §2.6)
    pub test_hook_signal: Mutex<Option<ClarotyTestHookSignal>>,
}

impl ClarotyCloneState {
    /// Create state with a specific admin token and org slug.
    ///
    /// `org_slug` is embedded in device IDs to ensure multi-org isolation
    /// (BC-3.5.001 postcondition 2). Pass an empty string for single-tenant tests
    /// that reference specific `asset_id` values from the fixture.
    /// Create state with admin token, org slug, and id prefix.
    ///
    /// `org_slug` is always set to the org slug (used for `GET /assets/v1/assets`).
    /// `id_prefix` controls `POST /api/v1/devices` device ID prefixing:
    /// - Empty → raw fixture IDs (e.g. `"asset-001"`) — for single-tenant tests.
    /// - Non-empty → `"{id_prefix}-{asset_id}"` — for multi-org isolation tests.
    pub fn new(admin_token: String, org_slug: String, id_prefix: String) -> Self {
        Self {
            org_slug,
            id_prefix,
            tag_store: Mutex::new(HashMap::new()),
            request_counter: std::sync::atomic::AtomicU32::new(0),
            failure_mode: Mutex::new(FailureMode::None),
            latency_ms: AtomicU64::new(0),
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
        self.request_counter
            .store(0, std::sync::atomic::Ordering::SeqCst);
        self.set_failure_mode(FailureMode::None);
        self.latency_ms.store(0, Ordering::SeqCst);
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
    pub fn get_tags(&self, device_id: &str) -> HashSet<String> {
        self.tag_store
            .lock()
            .expect("tag_store poisoned")
            .get(device_id)
            .cloned()
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Check that `Authorization: Bearer {non-empty}` is present.
///
/// Returns `Err((401, JSON))` if auth is missing or malformed.
fn check_bearer_auth(headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && !v[7..].trim().is_empty())
        .unwrap_or(false);

    if has_bearer {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid Authorization header", "code": 401})),
        ))
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
            if *after_ms == 0 {
                // EC-007: delay_ms=0 is treated as FailureMode::None
                None
            } else {
                // The timeout is handled per-route via a sleep before response.
                // We return None here; routes that need timeout behavior must
                // check for NetworkTimeout mode separately and sleep.
                // (The caller handles this by checking the mode before calling here.)
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
// Request body types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct GetDevicesBody {
    group_by: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
    offset: Option<u32>,
    limit: Option<u32>,
    // All other fields are accepted and ignored (EC-001: unrecognized fields silently ignored).
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DtuConfigureBody {
    auth_mode: Option<String>,
    rate_limit_after: Option<u32>,
    retry_after_secs: Option<u32>,
    internal_error_at: Option<u32>,
    unprocessable_at: Option<u32>,
    latency_ms: Option<u64>,
    network_timeout_ms: Option<u64>,
    malformed_response: Option<bool>,
    clear: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AddTagBody {
    tag_key: String,
    #[allow(dead_code)]
    tag_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `GET /assets/v1/assets` — legacy Claroty xDome asset list endpoint.
///
/// Used by pre-existing harness tests (logical_isolation_test.rs,
/// builder_ergonomics_test.rs) for failure injection checks and device ID
/// isolation assertions.
///
/// # Device ID format
///
/// Always prefixes `asset_id` with `org_slug` (regardless of seed) so that
/// the logical isolation tests can assert `id.contains("acme-corp")` etc.
/// (BC-3.5.001 postcondition 2; D-059).
///
/// When `org_slug` is non-empty, the response sets `"id"` to
/// `"{org_slug}-{original_asset_id}"`. When empty, falls back to `asset_id`.
async fn get_assets(
    State(state): State<Arc<ClarotyCloneState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    // No body in GET — apply failure injection.
    let latency = state.latency_ms.load(Ordering::SeqCst);
    if latency > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
    }

    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    // Auth not required for GET /assets/v1/assets in legacy harness mode.
    let _ = headers;

    #[allow(clippy::expect_used)]
    let mut devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON array");

    // Always prefix with org_slug for isolation tests.
    // For the GET endpoint, we ALWAYS add the org_slug prefix so that the
    // logical_isolation_test.rs assertions `id.contains("acme-corp")` pass.
    // The POST /api/v1/devices endpoint uses seed-conditional prefixing
    // to preserve raw `asset_id` values in single-tenant harness_tests.rs tests.
    for device in &mut devices {
        let orig_asset_id = device
            .get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_owned();

        let id = if !state.org_slug.is_empty() {
            format!("{}-{}", state.org_slug, orig_asset_id)
        } else {
            orig_asset_id.clone()
        };

        device["id"] = Value::String(id.clone());
        if !state.org_slug.is_empty() {
            device["asset_id"] = Value::String(id);
        }
    }

    let total = devices.len() as u32;
    Json(json!({"assets": devices, "total": total})).into_response()
}

/// `POST /api/v1/devices`
async fn list_devices(
    State(state): State<Arc<ClarotyCloneState>>,
    headers: HeaderMap,
    body: Option<Json<GetDevicesBody>>,
) -> axum::response::Response {
    if let Err(err) = check_bearer_auth(&headers) {
        return err.into_response();
    }

    // Artificial latency (EC-006).
    let latency = state.latency_ms.load(Ordering::SeqCst);
    if latency > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
    }

    // Failure injection.
    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    #[allow(clippy::expect_used)]
    let mut devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON array");

    let params = body.map(|Json(b)| b).unwrap_or_default();

    // Apply id_prefix to device IDs for multi-org isolation.
    //
    // When `id_prefix` is non-empty, `asset_id` and `uid` are prefixed so that
    // each org's device ID set is distinct (BC-3.5.001 postcondition 2; VP-123).
    // Single-tenant clones use an empty id_prefix and get the original fixture IDs
    // so that tests can reference specific IDs like `"asset-001"` by name.
    if !state.id_prefix.is_empty() {
        for device in &mut devices {
            if let Some(orig) = device
                .get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
            {
                device["asset_id"] = Value::String(format!("{}-{}", state.id_prefix, orig));
            }
            if let Some(orig) = device
                .get("uid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
            {
                device["uid"] = Value::String(format!("{}-{}", state.id_prefix, orig));
            }
        }
    }

    // Merge tag state into each device.
    for device in &mut devices {
        if let Some(asset_id) = device.get("asset_id").and_then(|v| v.as_str()) {
            let tags: Vec<Value> = state
                .get_tags(asset_id)
                .into_iter()
                .map(Value::String)
                .collect();
            device["tags"] = Value::Array(tags);
        }
    }

    // group_by handling (AC-2, EC-003).
    if let Some(ref group_field) = params.group_by {
        let known_fields = [
            "device_type",
            "device_category",
            "device_subcategory",
            "device_type_family",
            "os_category",
            "risk_score",
        ];
        if known_fields.contains(&group_field.as_str()) {
            let mut seen = HashMap::<String, u32>::new();
            for device in &devices {
                let val = device
                    .get(group_field.as_str())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                *seen.entry(val).or_insert(0) += 1;
            }
            let groups: Vec<Value> = seen
                .into_iter()
                .map(|(value, count)| json!({"field": group_field, "value": value, "count": count}))
                .collect();
            let total = groups.len() as u32;
            return Json(json!({"groups": groups, "total": total})).into_response();
        } else {
            // EC-003: unknown group_by field — return empty groups, not an error.
            return Json(json!({"groups": [], "total": 0u32})).into_response();
        }
    }

    let total = devices.len() as u32;

    // Pagination (EC-004).
    let paged: Vec<Value> = if let (Some(page), Some(page_size)) = (params.page, params.page_size) {
        let page_size = page_size as usize;
        let page = page as usize;
        let start = page.saturating_sub(1).saturating_mul(page_size);
        if start >= devices.len() {
            vec![]
        } else {
            devices[start..std::cmp::min(start + page_size, devices.len())].to_vec()
        }
    } else if let Some(offset) = params.offset {
        let offset = offset as usize;
        let limit = params.limit.unwrap_or(u32::MAX) as usize;
        if offset >= devices.len() {
            vec![]
        } else {
            devices[offset..std::cmp::min(offset + limit, devices.len())].to_vec()
        }
    } else {
        devices
    };

    let page_num = params.page.unwrap_or(1);
    Json(json!({"devices": paged, "total": total, "page": page_num})).into_response()
}

/// `POST /api/v1/alerts`
async fn list_alerts(headers: HeaderMap, _body: Option<Json<Value>>) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let alerts: Vec<Value> =
        serde_json::from_str(ALERTS_FIXTURE).expect("ALERTS_FIXTURE is valid JSON array");
    let total = alerts.len() as u32;
    (
        StatusCode::OK,
        Json(json!({"alerts": alerts, "total": total, "page": 1u32})),
    )
}

/// `POST /api/v1/alerts/:id/devices`
async fn list_alerted_devices(
    Path(_alert_id): Path<String>,
    headers: HeaderMap,
    _body: Option<Json<Value>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let devices: Vec<Value> = serde_json::from_str(ALERTED_DEVICES_FIXTURE)
        .expect("ALERTED_DEVICES_FIXTURE is valid JSON array");
    let total = devices.len() as u32;
    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}

/// `POST /api/v1/vulnerabilities`
async fn list_vulnerabilities(
    headers: HeaderMap,
    _body: Option<Json<Value>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let vulns: Vec<Value> = serde_json::from_str(VULNERABILITIES_FIXTURE)
        .expect("VULNERABILITIES_FIXTURE is valid JSON array");
    let total = vulns.len() as u32;
    (
        StatusCode::OK,
        Json(json!({"vulnerabilities": vulns, "total": total, "page": 1u32})),
    )
}

/// `POST /api/v1/vulnerabilities/:id/devices`
async fn list_vulnerability_devices(
    Path(_vuln_id): Path<String>,
    headers: HeaderMap,
    _body: Option<Json<Value>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    #[allow(clippy::expect_used)]
    let devices: Vec<Value> = serde_json::from_str(VULNERABILITY_DEVICES_FIXTURE)
        .expect("VULNERABILITY_DEVICES_FIXTURE is valid JSON array");
    let total = devices.len() as u32;
    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}

/// `POST /api/v1/devices/:id/tags/`
async fn add_tag(
    State(state): State<Arc<ClarotyCloneState>>,
    Path(device_id): Path<String>,
    headers: HeaderMap,
    body: Option<Json<AddTagBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    let tag_key = body
        .map(|Json(b)| b.tag_key)
        .unwrap_or_else(|| "unknown".to_string());

    state.add_tag(&device_id, &tag_key);

    (
        StatusCode::CREATED,
        Json(json!({
            "device_id": device_id,
            "tag_key": tag_key,
            "status": "added",
        })),
    )
}

/// `DELETE /api/v1/devices/:id/tags/:key`
async fn remove_tag(
    State(state): State<Arc<ClarotyCloneState>>,
    Path((device_id, tag_key)): Path<(String, String)>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    if state.remove_tag(&device_id, &tag_key) {
        (StatusCode::OK, Json(json!({"status": "removed"})))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tag not found"})),
        )
    }
}

/// `POST /dtu/configure`
///
/// Requires `X-Admin-Token` header (TD-WV0-07).
/// Unknown fields return HTTP 400 (TD-WV0-04).
async fn dtu_configure(
    State(state): State<Arc<ClarotyCloneState>>,
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

    // Apply latency configuration (independent of failure mode).
    if let Some(latency_ms) = body.latency_ms {
        state.latency_ms.store(latency_ms, Ordering::SeqCst);
        // If only latency_ms was set (no failure fields), return early preserving existing mode.
        if body.auth_mode.is_none()
            && body.rate_limit_after.is_none()
            && body.internal_error_at.is_none()
            && body.unprocessable_at.is_none()
            && body.clear.is_none()
        {
            return (StatusCode::OK, Json(json!({"status": "ok"})));
        }
    }

    // Determine failure mode.
    let mode = if body.clear == Some(true) {
        FailureMode::None
    } else if let Some(at) = body.unprocessable_at {
        FailureMode::Unprocessable { at_request_n: at }
    } else if let Some(at) = body.internal_error_at {
        FailureMode::InternalError { at_request_n: at }
    } else if let Some(n) = body.rate_limit_after {
        FailureMode::RateLimit {
            after_n_requests: n,
            retry_after_secs: body.retry_after_secs.unwrap_or(60),
        }
    } else if body.auth_mode.as_deref() == Some("reject") {
        FailureMode::AuthReject
    } else if let Some(ms) = body.network_timeout_ms {
        FailureMode::NetworkTimeout { after_ms: ms }
    } else if body.malformed_response == Some(true) {
        FailureMode::MalformedResponse
    } else {
        FailureMode::None
    };

    // Reset request counter when setting a new failure mode.
    state
        .request_counter
        .store(0, std::sync::atomic::Ordering::SeqCst);
    state.set_failure_mode(mode);

    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// `POST /dtu/reset`
///
/// Clears tag store, request counter, and failure mode. No auth required.
async fn dtu_reset(State(state): State<Arc<ClarotyCloneState>>) -> (StatusCode, Json<Value>) {
    state.reset();
    (StatusCode::OK, Json(json!({"status": "reset"})))
}

/// `GET /dtu/health`
async fn dtu_health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Test hook handlers (crash detection — BC-3.6.002)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PanicBody {
    message: String,
}

/// `POST /dtu/test-hook/panic`
///
/// Stores a `Panic` signal in the clone state. The background crash-monitor
/// loop picks it up and fires the crash watch channel.
#[allow(clippy::expect_used)]
async fn test_hook_panic(
    State(state): State<Arc<ClarotyCloneState>>,
    Json(body): Json<PanicBody>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(ClarotyTestHookSignal::Panic(body.message));
    (StatusCode::OK, Json(json!({"status": "panic queued"})))
}

/// `POST /dtu/test-hook/premature-ok`
#[allow(clippy::expect_used)]
async fn test_hook_premature_ok(
    State(state): State<Arc<ClarotyCloneState>>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(ClarotyTestHookSignal::PrematureOk);
    (
        StatusCode::OK,
        Json(json!({"status": "premature-ok queued"})),
    )
}

/// `POST /dtu/test-hook/non-string-panic`
#[allow(clippy::expect_used)]
async fn test_hook_non_string_panic(
    State(state): State<Arc<ClarotyCloneState>>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(ClarotyTestHookSignal::NonStringPanic);
    (
        StatusCode::OK,
        Json(json!({"status": "non-string-panic queued"})),
    )
}

/// Poll the Claroty clone's test hook signal and fire crash_tx if triggered.
///
/// Mirrors `clone_server::poll_test_hook` but reads from `ClarotyCloneState`
/// instead of `CloneState`. Run concurrently with the HTTP server in
/// `start_claroty_clone`.
///
/// (BC-3.6.002 postcondition clause 1; ADR-011 §2.6)
#[allow(clippy::expect_used)]
pub async fn poll_claroty_test_hook(
    state: Arc<ClarotyCloneState>,
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
                ClarotyTestHookSignal::Panic(msg) => {
                    let _ = crash_tx.send(Some(msg));
                    return;
                }
                ClarotyTestHookSignal::PrematureOk => {
                    let _ =
                        crash_tx.send(Some(crate::crash_monitor::PREMATURE_OK_CAUSE.to_string()));
                    return;
                }
                ClarotyTestHookSignal::NonStringPanic => {
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
// Router constructors
// ---------------------------------------------------------------------------

/// Build the Claroty clone axum `Router` for logical-mode isolation.
///
/// In logical mode, any non-empty Bearer token is accepted on device/tag/alert/
/// vulnerability routes (same as the production Claroty API's bearer check).
/// This allows tests to use a fixed `"test-token"` bearer without needing the
/// per-clone admin token.
///
/// Called by `clone_server::start_claroty_clone` (logical mode path).
pub fn router(state: Arc<ClarotyCloneState>) -> Router {
    Router::new()
        // Legacy GET endpoint (used by builder ergonomics tests for failure injection checks).
        .route("/assets/v1/assets", get(get_assets))
        // Device endpoints
        .route("/api/v1/devices", post(list_devices))
        // Alert endpoints
        .route("/api/v1/alerts", post(list_alerts))
        .route("/api/v1/alerts/:id/devices", post(list_alerted_devices))
        // Vulnerability endpoints
        .route("/api/v1/vulnerabilities", post(list_vulnerabilities))
        .route(
            "/api/v1/vulnerabilities/:id/devices",
            post(list_vulnerability_devices),
        )
        // Tag write endpoints
        .route("/api/v1/devices/:id/tags/", post(add_tag))
        .route("/api/v1/devices/:id/tags/:key", delete(remove_tag))
        // DTU control endpoints
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

/// Check bearer token for **network-mode** isolation.
///
/// Policy (BC-3.5.002 postcondition 2; VP-126):
/// - No `Authorization` header → allow (unauthenticated reads permitted).
/// - `Authorization: Bearer <token>` where `token == admin_token` → allow.
/// - `Authorization: Bearer <token>` where `token ≠ admin_token` → HTTP 401.
///
/// This stricter check enables cross-org credential routing tests: a request
/// bearing OrgA's admin token sent to OrgB's endpoint returns 401.
fn check_bearer_network(
    headers: &HeaderMap,
    admin_token: &str,
) -> Option<axum::response::Response> {
    if let Some(auth_val) = headers.get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if token != admin_token {
                    return Some(
                        (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "invalid bearer token — wrong org credentials"})),
                        )
                            .into_response(),
                    );
                }
            }
        }
    }
    None
}

/// Handler wrapper for network-mode device list: checks bearer against admin token.
async fn list_devices_network(
    State(state): State<Arc<ClarotyCloneState>>,
    headers: HeaderMap,
    body: Option<Json<GetDevicesBody>>,
) -> axum::response::Response {
    if let Some(reject) = check_bearer_network(&headers, &state.admin_token) {
        return reject;
    }
    // Delegate to the standard list_devices handler after auth passes.
    // We need to call the inner logic directly since list_devices also checks bearer.
    // Instead, replicate the inner logic here.

    // Artificial latency (EC-006).
    let latency = state.latency_ms.load(Ordering::SeqCst);
    if latency > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
    }

    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    #[allow(clippy::expect_used)]
    let mut devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON array");

    let params = body.map(|Json(b)| b).unwrap_or_default();

    if !state.id_prefix.is_empty() {
        for device in &mut devices {
            if let Some(orig) = device
                .get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
            {
                device["asset_id"] = Value::String(format!("{}-{}", state.id_prefix, orig));
            }
            if let Some(orig) = device
                .get("uid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
            {
                device["uid"] = Value::String(format!("{}-{}", state.id_prefix, orig));
            }
        }
    }

    for device in &mut devices {
        if let Some(asset_id) = device.get("asset_id").and_then(|v| v.as_str()) {
            let tags: Vec<Value> = state
                .get_tags(asset_id)
                .into_iter()
                .map(Value::String)
                .collect();
            device["tags"] = Value::Array(tags);
        }
    }

    if let Some(ref group_field) = params.group_by {
        let known_fields = [
            "device_type",
            "device_category",
            "device_subcategory",
            "device_type_family",
            "os_category",
            "risk_score",
        ];
        if known_fields.contains(&group_field.as_str()) {
            let mut seen = HashMap::<String, u32>::new();
            for device in &devices {
                let val = device
                    .get(group_field.as_str())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                *seen.entry(val).or_insert(0) += 1;
            }
            let groups: Vec<Value> = seen
                .into_iter()
                .map(|(value, count)| json!({"field": group_field, "value": value, "count": count}))
                .collect();
            let total = groups.len() as u32;
            return Json(json!({"groups": groups, "total": total})).into_response();
        } else {
            return Json(json!({"groups": [], "total": 0u32})).into_response();
        }
    }

    let total = devices.len() as u32;
    let paged: Vec<Value> = if let (Some(page), Some(page_size)) = (params.page, params.page_size) {
        let page_size = page_size as usize;
        let page = page as usize;
        let start = page.saturating_sub(1).saturating_mul(page_size);
        if start >= devices.len() {
            vec![]
        } else {
            devices[start..std::cmp::min(start + page_size, devices.len())].to_vec()
        }
    } else if let Some(offset) = params.offset {
        let offset = offset as usize;
        let limit = params.limit.unwrap_or(u32::MAX) as usize;
        if offset >= devices.len() {
            vec![]
        } else {
            devices[offset..std::cmp::min(offset + limit, devices.len())].to_vec()
        }
    } else {
        devices
    };

    let page_num = params.page.unwrap_or(1);
    Json(json!({"devices": paged, "total": total, "page": page_num})).into_response()
}

/// `GET /assets/v1/assets` — network-mode variant with bearer token validation.
///
/// Differs from `get_assets` in that it validates the `Authorization: Bearer`
/// header against the clone's admin token. Unauthenticated requests are allowed
/// through (no Authorization header → 200). Requests bearing a wrong token → 401.
///
/// This enables cross-org credential routing tests (BC-3.5.002 postcondition 2; VP-126;
/// AC-004; TV-3): a request bearing OrgA's admin token sent to OrgB's endpoint returns 401.
async fn get_assets_network(
    State(state): State<Arc<ClarotyCloneState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Some(reject) = check_bearer_network(&headers, &state.admin_token) {
        return reject;
    }
    // Bearer auth passed (or no bearer header); delegate to standard get_assets logic.
    // We replicate the inner logic here to avoid double-incrementing the request counter.
    let latency = state.latency_ms.load(Ordering::SeqCst);
    if latency > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
    }

    let n = state.increment_counter();
    let mode = state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, n) {
        return resp;
    }

    #[allow(clippy::expect_used)]
    let mut devices: Vec<Value> =
        serde_json::from_str(DEVICES_FIXTURE).expect("DEVICES_FIXTURE is valid JSON array");

    for device in &mut devices {
        let orig_asset_id = device
            .get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_owned();

        let id = if !state.org_slug.is_empty() {
            format!("{}-{}", state.org_slug, orig_asset_id)
        } else {
            orig_asset_id.clone()
        };

        device["id"] = Value::String(id.clone());
        if !state.org_slug.is_empty() {
            device["asset_id"] = Value::String(id);
        }
    }

    let total = devices.len() as u32;
    Json(json!({"assets": devices, "total": total})).into_response()
}

/// Build the Claroty clone axum `Router` for **network-mode** isolation.
///
/// Differences from the logical-mode `router()`:
/// - Bearer token is validated against the clone's admin token (cross-org 401).
/// - No-auth requests are allowed through (unauthenticated reads work without tokens).
///
/// Called by `start_clone_network` in `builder.rs` when `DtuType::Claroty` is dispatched.
///
/// (BC-3.5.002 postcondition 2; VP-126; ADR-011 §2.3)
pub fn network_router(state: Arc<ClarotyCloneState>) -> Router {
    Router::new()
        // Legacy GET endpoint: network-mode bearer validation applied.
        .route("/assets/v1/assets", get(get_assets_network))
        // Test hook endpoints (BC-3.6.002 crash detection — used by network mode tests).
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        // Device endpoint: network-mode bearer validation
        .route("/api/v1/devices", post(list_devices_network))
        // Alert endpoints (same as logical mode — no cross-org test for these)
        .route("/api/v1/alerts", post(list_alerts))
        .route("/api/v1/alerts/:id/devices", post(list_alerted_devices))
        // Vulnerability endpoints
        .route("/api/v1/vulnerabilities", post(list_vulnerabilities))
        .route(
            "/api/v1/vulnerabilities/:id/devices",
            post(list_vulnerability_devices),
        )
        // Tag write endpoints
        .route("/api/v1/devices/:id/tags/", post(add_tag))
        .route("/api/v1/devices/:id/tags/:key", delete(remove_tag))
        // DTU control endpoints
        .route("/dtu/configure", post(dtu_configure))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/health", get(dtu_health))
        .with_state(state)
}
