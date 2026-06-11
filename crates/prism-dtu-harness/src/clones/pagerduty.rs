//! PagerDuty Events API v2 clone router for the DTU harness (S-3.4.05).
//!
//! Provides PagerDuty-specific route handlers running inside the harness
//! for `DtuType::PagerDuty` clones. Uses `CloneState` for failure mode injection.
//!
//! # Routes served
//!
//! - `POST /v2/enqueue`          — PagerDuty Events API v2 (trigger/ack/resolve)
//! - `GET  /dtu/incidents`       — test API: incident registry snapshot
//! - `POST /dtu/reset`           — clear incident registry + reset failure mode
//! - `GET  /dtu/health`          — liveness check
//! - `POST /dtu/configure`       — harness failure injection (AuthReject, RateLimit, …)
//!
//! # OrgId tagging (BC-3.2.004)
//!
//! Every triggered incident record stores the originating OrgId UUID in
//! `IncidentRecord.org_id`, resolved from the `X-Prism-Org-Id` header.
//!
//! # BC anchors
//!
//! - BC-3.2.004 — shared-mode org-id tagging
//! - BC-3.5.001 — harness logical isolation

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc, Mutex},
};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::clone_server::CloneState;

// ---------------------------------------------------------------------------
// Incident state types (mirrors prism-dtu-pagerduty, self-contained)
// ---------------------------------------------------------------------------

/// Incident lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    Triggered,
    Acknowledged,
    Resolved,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Triggered => "triggered",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
        }
    }
}

/// A single incident record in the PagerDuty harness incident registry.
#[derive(Debug, Clone)]
pub struct IncidentRecord {
    pub dedup_key: String,
    pub status: IncidentStatus,
    pub severity: String,
    pub summary: String,
    /// OrgId UUID of the originating organisation (BC-3.2.004).
    pub org_id: Option<String>,
}

// ---------------------------------------------------------------------------
// PagerDuty-specific state
// ---------------------------------------------------------------------------

/// In-memory state for harness-hosted PagerDuty clone.
pub struct PdHarnessState {
    /// Incident registry keyed by `dedup_key`.
    pub incident_registry: Mutex<HashMap<String, IncidentRecord>>,
    /// When `true`, all enqueue requests return HTTP 403.
    pub auth_reject: Mutex<bool>,
}

impl PdHarnessState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            incident_registry: Mutex::new(HashMap::new()),
            auth_reject: Mutex::new(false),
        })
    }

    #[allow(clippy::expect_used)]
    pub fn is_auth_reject(&self) -> bool {
        *self.auth_reject.lock().expect("auth_reject poisoned")
    }

    #[allow(clippy::expect_used)]
    pub fn set_auth_reject(&self, val: bool) {
        *self.auth_reject.lock().expect("auth_reject poisoned") = val;
    }

    #[allow(clippy::expect_used)]
    pub fn reset(&self) {
        self.incident_registry
            .lock()
            .expect("incident_registry poisoned")
            .clear();
        self.set_auth_reject(false);
    }

    #[allow(clippy::expect_used)]
    pub fn incidents_snapshot(&self) -> Vec<IncidentRecord> {
        self.incident_registry
            .lock()
            .expect("incident_registry poisoned")
            .values()
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Combined state
// ---------------------------------------------------------------------------

/// Combined state for harness-hosted PagerDuty clone.
pub struct PdCloneCtx {
    pub clone_state: Arc<CloneState>,
    pub pd_state: Arc<PdHarnessState>,
}

// ---------------------------------------------------------------------------
// Request/response types (self-contained, mirrors prism-dtu-pagerduty/types.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EnqueueRequest {
    routing_key: Option<String>,
    event_action: Option<String>,
    dedup_key: Option<String>,
    payload: Option<EventPayload>,
}

#[derive(Debug, Deserialize)]
struct EventPayload {
    summary: Option<String>,
    severity: Option<String>,
    #[allow(dead_code)]
    source: Option<String>,
}

const VALID_SEVERITIES: &[&str] = &["critical", "error", "warning", "info"];

// ---------------------------------------------------------------------------
// Helper: resolve OrgId from headers
// ---------------------------------------------------------------------------

fn resolve_org_id(headers: &HeaderMap) -> String {
    headers
        .get("X-Prism-Org-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| uuid::Uuid::parse_str(s).is_ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

// ---------------------------------------------------------------------------
// Failure mode application (mirrors claroty::apply_failure_mode)
//
// Auth ordering (per BC-3.6.001 work-order): no explicit auth check on the
// enqueue route (PagerDuty uses routing_key validation, not a header auth
// check). Failure injection runs BEFORE routing_key validation — same
// pattern as claroty.rs get_assets (which has no auth check either):
// counter → apply_failure_mode → normal processing.
//
// All 6 modes are represented:
//   AuthReject → 403 "invalid key" (PagerDuty-specific; real Events API returns 403 for bad routing_key)
//   InternalError → 500 at at_request_n
//   RateLimit → 429 after N requests
//   NetworkTimeout → sleep(after_ms) (EC-007: 0ms = no-op)
//   MalformedResponse → non-JSON body
//   Unprocessable → 422 at at_request_n
// ---------------------------------------------------------------------------

/// Apply the current failure mode. Returns `Some(response)` if a failure should be served.
///
/// For NetworkTimeout: returns `None` here; callers must check for NetworkTimeout
/// BEFORE calling this function and sleep appropriately (same pattern as claroty.rs).
fn apply_failure_mode(mode: &FailureMode, n: u32) -> Option<axum::response::Response> {
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => Some(
            (
                StatusCode::FORBIDDEN,
                Json(json!({"status": "invalid key", "message": "Forbidden"})),
            )
                .into_response(),
        ),
        FailureMode::InternalError { at_request_n } => {
            if n == *at_request_n {
                Some(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"status": "internal error (injected)"})),
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
                let retry_str = retry_after_secs.to_string();
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"status": "rate limited"})),
                )
                    .into_response();
                #[allow(clippy::expect_used)]
                resp.headers_mut().insert(
                    "retry-after",
                    retry_str
                        .parse()
                        .expect("retry_after_secs is a valid header value"),
                );
                Some(resp)
            } else {
                None
            }
        }
        FailureMode::NetworkTimeout { after_ms } => {
            // EC-007: delay_ms=0 is a no-op. Non-zero delays are handled
            // per-route via tokio::time::sleep before this function is called.
            let _ = after_ms;
            None
        }
        FailureMode::Unprocessable { at_request_n } => {
            if n == *at_request_n {
                Some(
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"status": "unprocessable entity (injected)"})),
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
// Route handlers
// ---------------------------------------------------------------------------

/// `POST /v2/enqueue` — PagerDuty Events API v2.
///
/// Implements the full incident lifecycle: trigger / acknowledge / resolve.
/// Applies full failure mode from CloneState before processing.
/// Auth ordering: failure injection runs before routing_key validation
/// (PagerDuty has no header auth check; mirrors claroty.rs get_assets pattern).
async fn post_enqueue(
    State(ctx): State<Arc<PdCloneCtx>>,
    headers: HeaderMap,
    Json(body): Json<EnqueueRequest>,
) -> impl IntoResponse {
    // NetworkTimeout: sleep before counting (EC-007: 0ms = no-op).
    let mode_pre = ctx.clone_state.current_failure_mode();
    if let FailureMode::NetworkTimeout { after_ms } = &mode_pre {
        if *after_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(*after_ms)).await;
        }
    }

    // Apply full failure mode (all 6 variants).
    let count = ctx.clone_state.increment_request();
    let mode = ctx.clone_state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    // Validate `routing_key`.
    if body
        .routing_key
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "missing routing_key"})),
        )
            .into_response();
    }

    // Validate `event_action`.
    let event_action = match body.event_action.as_deref() {
        Some("trigger") => "trigger",
        Some("acknowledge") => "acknowledge",
        Some("resolve") => "resolve",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "invalid event_action"})),
            )
                .into_response();
        }
    };

    // Validate `payload.severity` if payload present.
    if let Some(ref payload) = body.payload {
        if let Some(severity) = payload.severity.as_deref() {
            if !VALID_SEVERITIES.contains(&severity) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"status": "invalid severity"})),
                )
                    .into_response();
            }
        }
    }

    // Dispatch to action.
    match event_action {
        "trigger" => handle_trigger(&ctx, &body, &headers).await,
        "acknowledge" => handle_acknowledge(&ctx, &body).await,
        "resolve" => handle_resolve(&ctx, &body).await,
        _ => unreachable!("event_action already validated"),
    }
}

#[allow(clippy::expect_used)]
async fn handle_trigger(
    ctx: &Arc<PdCloneCtx>,
    body: &EnqueueRequest,
    headers: &HeaderMap,
) -> axum::response::Response {
    let dedup_key = body
        .dedup_key
        .clone()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let severity = body
        .payload
        .as_ref()
        .and_then(|p| p.severity.clone())
        .unwrap_or_else(|| "info".to_string());

    let summary = body
        .payload
        .as_ref()
        .and_then(|p| p.summary.clone())
        .unwrap_or_default();

    let org_id = resolve_org_id(headers);

    let mut registry = ctx
        .pd_state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    // Idempotency: if active incident exists, return 202 without creating.
    if let Some(existing) = registry.get(&dedup_key) {
        if existing.status != IncidentStatus::Resolved {
            let dk = dedup_key.clone();
            drop(registry);
            return (
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "success",
                    "message": "Event processed",
                    "dedup_key": dk
                })),
            )
                .into_response();
        }
    }

    // New or re-triggered incident.
    registry.insert(
        dedup_key.clone(),
        IncidentRecord {
            dedup_key: dedup_key.clone(),
            status: IncidentStatus::Triggered,
            severity,
            summary,
            org_id: Some(org_id),
        },
    );

    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "success",
            "message": "Event processed",
            "dedup_key": dedup_key
        })),
    )
        .into_response()
}

#[allow(clippy::expect_used)]
async fn handle_acknowledge(
    ctx: &Arc<PdCloneCtx>,
    body: &EnqueueRequest,
) -> axum::response::Response {
    let dedup_key = match body.dedup_key.as_deref().filter(|k| !k.is_empty()) {
        Some(k) => k.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "invalid dedup_key"})),
            )
                .into_response();
        }
    };

    let mut registry = ctx
        .pd_state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    match registry.get_mut(&dedup_key) {
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "invalid dedup_key"})),
        )
            .into_response(),
        Some(incident) if incident.status == IncidentStatus::Resolved => (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "cannot acknowledge a resolved incident"})),
        )
            .into_response(),
        Some(incident) => {
            incident.status = IncidentStatus::Acknowledged;
            (StatusCode::OK, Json(json!({"status": "success"}))).into_response()
        }
    }
}

#[allow(clippy::expect_used)]
async fn handle_resolve(ctx: &Arc<PdCloneCtx>, body: &EnqueueRequest) -> axum::response::Response {
    let dedup_key = match body.dedup_key.as_deref().filter(|k| !k.is_empty()) {
        Some(k) => k.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "invalid dedup_key"})),
            )
                .into_response();
        }
    };

    let mut registry = ctx
        .pd_state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    match registry.get_mut(&dedup_key) {
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "invalid dedup_key"})),
        )
            .into_response(),
        Some(incident) => {
            incident.status = IncidentStatus::Resolved;
            (StatusCode::OK, Json(json!({"status": "success"}))).into_response()
        }
    }
}

/// `GET /dtu/incidents` — return snapshot of all incidents.
#[allow(clippy::expect_used)]
async fn get_incidents(State(ctx): State<Arc<PdCloneCtx>>) -> impl IntoResponse {
    let incidents = ctx.pd_state.incidents_snapshot();
    let summaries: Vec<Value> = incidents
        .into_iter()
        .map(|r| {
            json!({
                "dedup_key": r.dedup_key,
                "status": r.status.as_str(),
                "severity": r.severity,
                "summary": r.summary,
                "org_id": r.org_id
            })
        })
        .collect();
    (StatusCode::OK, Json(json!({"incidents": summaries}))).into_response()
}

/// `POST /dtu/reset` — clear all state.
async fn post_reset(State(ctx): State<Arc<PdCloneCtx>>) -> impl IntoResponse {
    ctx.pd_state.reset();
    ctx.clone_state.request_count.store(0, Ordering::SeqCst);
    ctx.clone_state.set_failure_mode(FailureMode::None);
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health` — liveness check.
async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

// ---------------------------------------------------------------------------
// Configure handler (harness format)
// ---------------------------------------------------------------------------

/// F10 / finding ⑫ (2026-06-10 review): strict payload schema — an unknown
/// field in a configure payload is a harness-side bug (typo'd failure-mode key
/// silently ignored), so it must be rejected with 400, not dropped.
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct HarnessConfigure {
    #[serde(default)]
    auth_mode: Option<String>,
    #[serde(default)]
    rate_limit_after: Option<u32>,
    #[serde(default)]
    retry_after_secs: Option<u32>,
    #[serde(default)]
    internal_error_at: Option<u32>,
    #[serde(default)]
    network_timeout_ms: Option<u64>,
    #[serde(default)]
    malformed_response: Option<bool>,
    #[serde(default)]
    unprocessable_at: Option<u32>,
    #[serde(default)]
    clear: Option<bool>,
}

/// `POST /dtu/configure` — harness configure endpoint.
async fn post_configure(
    State(ctx): State<Arc<PdCloneCtx>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(ctx.clone_state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        );
    }

    let cfg: HarnessConfigure = match serde_json::from_value(body) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid configure payload: {e}")})),
            );
        }
    };

    // BC-3.6.001 v0.6 Postcondition 5: a structurally-valid payload that requests
    // an unrecognized mode (e.g. a future FailureMode variant) must return HTTP 400
    // with {"error":"unsupported_failure_mode","mode":"<variant-name>"}.  This is
    // distinct from the serde deny_unknown_fields path above (schema-invalid payload).
    // EC-009: the 400 path must be stateless — no state change occurs on rejection.
    match harness_configure_to_failure_mode(&cfg) {
        Ok(mode) => {
            // Clear the legacy pd_state.auth_reject flag on successful configure so it
            // does not interfere with the apply_failure_mode path in CloneState.
            ctx.pd_state.set_auth_reject(false);
            ctx.clone_state.request_count.store(0, Ordering::SeqCst);
            ctx.clone_state.set_failure_mode(mode);
            (StatusCode::OK, Json(json!({"status": "ok"})))
        }
        Err(mode_name) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "unsupported_failure_mode", "mode": mode_name})),
        ),
    }
}

/// Convert `HarnessConfigure` to a `FailureMode`.
///
/// Returns `Ok(FailureMode)` for recognized modes (including `None` for
/// empty / clear payloads).  Returns `Err(mode_name)` when a known field
/// carries an unrecognized value — the caller maps this to HTTP 400 with the
/// BC-3.6.001 v0.6 Postcondition-5 body
/// `{"error":"unsupported_failure_mode","mode":"<mode_name>"}`.
fn harness_configure_to_failure_mode(cfg: &HarnessConfigure) -> Result<FailureMode, String> {
    if cfg.clear == Some(true) {
        return Ok(FailureMode::None);
    }
    if let Some(mode) = &cfg.auth_mode {
        return match mode.as_str() {
            "reject" => Ok(FailureMode::AuthReject),
            // "none" is an explicit no-op clear via auth_mode field.
            "none" => Ok(FailureMode::None),
            // Any other value is an unsupported mode (future variant contract).
            other => Err(other.to_owned()),
        };
    }
    if let Some(n) = cfg.rate_limit_after {
        return Ok(FailureMode::RateLimit {
            after_n_requests: n,
            retry_after_secs: cfg.retry_after_secs.unwrap_or(60),
        });
    }
    if let Some(n) = cfg.internal_error_at {
        return Ok(FailureMode::InternalError { at_request_n: n });
    }
    if let Some(ms) = cfg.network_timeout_ms {
        return Ok(FailureMode::NetworkTimeout { after_ms: ms });
    }
    if cfg.malformed_response == Some(true) {
        return Ok(FailureMode::MalformedResponse);
    }
    if let Some(n) = cfg.unprocessable_at {
        return Ok(FailureMode::Unprocessable { at_request_n: n });
    }
    Ok(FailureMode::None)
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the axum router for a harness-hosted PagerDuty clone.
pub fn build_pagerduty_router(
    clone_state: Arc<CloneState>,
    pd_state: Arc<PdHarnessState>,
) -> Router {
    let ctx = Arc::new(PdCloneCtx {
        clone_state,
        pd_state,
    });

    Router::new()
        // PagerDuty Events API v2
        .route("/v2/enqueue", post(post_enqueue))
        // DTU test API
        .route("/dtu/incidents", get(get_incidents))
        .route("/dtu/reset", post(post_reset))
        .route("/dtu/health", get(get_health))
        // Harness configure
        .route("/dtu/configure", post(post_configure))
        .with_state(ctx)
}
