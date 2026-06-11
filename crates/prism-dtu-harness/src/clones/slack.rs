//! Slack Incoming Webhook clone router for the DTU harness (S-3.4.05).
//!
//! Provides the Slack-specific route handlers that run inside the harness
//! for `DtuType::Slack` clones. Uses `CloneState` for failure mode injection
//! (via the harness standard `/dtu/configure` endpoint) so that
//! `Harness::inject_failure` works identically for shared-mode DTUs.
//!
//! # Routes served
//!
//! - `POST /services/*token`           — Slack Incoming Webhook endpoint
//! - `GET  /dtu/received-payloads`     — test API: captured payloads
//! - `POST /dtu/reset`                 — clear captured payloads + reset counter
//! - `GET  /dtu/health`                — liveness check
//! - `POST /dtu/configure`             — harness failure injection (from CloneState)
//!
//! # OrgId tagging (BC-3.2.004)
//!
//! Every captured payload is wrapped in `{"org_id": "<uuid>", "payload": {...}}`.
//! The OrgId is resolved from the `X-Prism-Org-Id` request header; if absent,
//! a fresh UUID is generated (anonymous-ingress fallback).
//!
//! # BC anchors
//!
//! - BC-3.2.004 — shared-mode org-id tagging
//! - BC-3.5.001 — harness logical isolation

use std::sync::{atomic::Ordering, Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::clone_server::CloneState;

// ---------------------------------------------------------------------------
// Slack-specific state stored alongside the generic CloneState
// ---------------------------------------------------------------------------

/// In-memory store for Slack captured payloads (tagged with org_id).
pub struct SlackHarnessState {
    /// Ordered list of tagged payloads: `{"org_id": "...", "payload": {...}}`.
    pub received_payloads: Mutex<Vec<Value>>,
}

impl SlackHarnessState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            received_payloads: Mutex::new(Vec::new()),
        })
    }

    /// Append a tagged payload entry.
    #[allow(clippy::expect_used)]
    pub fn capture(&self, org_id: &str, payload: Value) {
        let tagged = json!({ "org_id": org_id, "payload": payload });
        self.received_payloads
            .lock()
            .expect("received_payloads poisoned")
            .push(tagged);
    }

    /// Return a snapshot of all captured payloads.
    #[allow(clippy::expect_used)]
    pub fn all(&self) -> Vec<Value> {
        self.received_payloads
            .lock()
            .expect("received_payloads poisoned")
            .clone()
    }

    /// Clear all captured payloads.
    #[allow(clippy::expect_used)]
    pub fn clear(&self) {
        self.received_payloads
            .lock()
            .expect("received_payloads poisoned")
            .clear();
    }
}

// ---------------------------------------------------------------------------
// Combined state passed to route handlers
// ---------------------------------------------------------------------------

/// Combined state for harness-hosted Slack clone.
///
/// Holds both the generic `CloneState` (failure mode, request counter, admin token)
/// and the Slack-specific payload capture store.
pub struct SlackCloneCtx {
    pub clone_state: Arc<CloneState>,
    pub slack_state: Arc<SlackHarnessState>,
}

// ---------------------------------------------------------------------------
// Allowed top-level Block Kit keys (mirrors prism-dtu-slack)
// ---------------------------------------------------------------------------

const ALLOWED_BLOCK_KIT_KEYS: &[&str] = &[
    "blocks",
    "text",
    "username",
    "icon_emoji",
    "icon_url",
    "attachments",
];

// ---------------------------------------------------------------------------
// Failure mode application (mirrors claroty::apply_failure_mode)
//
// Auth ordering (per BC-3.6.001 work-order): no explicit auth check on the
// webhook route (Slack uses a token-in-path model, not a header auth check).
// Failure injection runs BEFORE payload validation — same pattern as
// claroty.rs get_assets (which has no auth check either):
// counter → apply_failure_mode → normal processing.
//
// All 6 modes are represented:
//   AuthReject → 401
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
    use axum::response::IntoResponse as _;
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => {
            Some((StatusCode::UNAUTHORIZED, "\"unauthorized\"").into_response())
        }
        FailureMode::InternalError { at_request_n } => {
            if n == *at_request_n {
                Some((StatusCode::INTERNAL_SERVER_ERROR, "\"internal_error\"").into_response())
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
                Some(
                    (
                        StatusCode::TOO_MANY_REQUESTS,
                        [("Retry-After", retry_str.as_str())],
                        "\"ratelimited\"",
                    )
                        .into_response(),
                )
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
                Some((StatusCode::UNPROCESSABLE_ENTITY, "\"unprocessable_entity\"").into_response())
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

/// Resolve the OrgId UUID from `X-Prism-Org-Id` header, or generate a fresh one.
fn resolve_org_id(headers: &HeaderMap) -> String {
    headers
        .get("X-Prism-Org-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| uuid::Uuid::parse_str(s).is_ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

/// `POST /services/*token` — Slack Incoming Webhook endpoint.
///
/// Applies full failure mode from CloneState, validates Block Kit payload,
/// captures tagged payload, returns HTTP 200 `{"ok":true,"message_ts":"..."}`.
/// Auth ordering: failure injection runs before payload validation
/// (Slack has no header auth check; mirrors claroty.rs get_assets pattern).
async fn post_webhook(
    Path(_token): Path<String>,
    State(ctx): State<Arc<SlackCloneCtx>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Step 1: NetworkTimeout — sleep before counting (EC-007: 0ms = no-op).
    let mode_pre = ctx.clone_state.current_failure_mode();
    if let FailureMode::NetworkTimeout { after_ms } = &mode_pre {
        if *after_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(*after_ms)).await;
        }
    }

    // Step 2: increment request count and apply full failure mode (all 6 variants).
    let count = ctx.clone_state.increment_request();
    let mode = ctx.clone_state.current_failure_mode();
    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    // Step 3: parse and validate payload.
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
        }
    };

    let obj = match payload.as_object() {
        Some(o) => o,
        None => {
            return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
        }
    };

    // Check for unknown top-level fields.
    for key in obj.keys() {
        if !ALLOWED_BLOCK_KIT_KEYS.contains(&key.as_str()) {
            return (StatusCode::BAD_REQUEST, "\"unknown_field\"").into_response();
        }
    }

    // Check for presence of `blocks` or `text`.
    if !obj.contains_key("blocks") && !obj.contains_key("text") {
        return (StatusCode::BAD_REQUEST, "\"invalid_payload\"").into_response();
    }

    // Step 4: capture tagged payload (BC-3.2.004).
    let org_id = resolve_org_id(&headers);
    ctx.slack_state.capture(&org_id, payload);

    // Step 5: return success response with stable message_ts (AC-1, EC-004).
    (
        StatusCode::OK,
        Json(json!({"ok": true, "message_ts": "1234567890.123456"})),
    )
        .into_response()
}

/// `GET /dtu/received-payloads` — return all captured Block Kit payloads.
async fn get_received_payloads(State(ctx): State<Arc<SlackCloneCtx>>) -> impl IntoResponse {
    let payloads = ctx.slack_state.all();
    (StatusCode::OK, Json(json!({"payloads": payloads}))).into_response()
}

/// `POST /dtu/reset` — clear captured payloads and reset request counter + failure mode.
async fn post_reset(State(ctx): State<Arc<SlackCloneCtx>>) -> impl IntoResponse {
    ctx.slack_state.clear();
    ctx.clone_state.request_count.store(0, Ordering::SeqCst);
    ctx.clone_state.set_failure_mode(FailureMode::None);
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health` — liveness check.
async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

// ---------------------------------------------------------------------------
// Configure handler (harness format — mirrors clone_server::dtu_configure_pub)
// ---------------------------------------------------------------------------

/// The harness configure payload format (mirrors `clone_server::ConfigureBody`).
///
/// This is the same format that `Harness::inject_failure` sends via
/// `failure_mode_to_json`.
///
/// F10 / finding ⑫ (2026-06-10 review): strict payload schema — an unknown
/// field in a configure payload is a harness-side bug (typo'd failure-mode key
/// silently ignored), so it must be rejected with 400, not dropped. Human
/// overrode the prior "deliberately permissive" stance on 2026-06-10.
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
///
/// Accepts the harness JSON format (same as `clone_server::dtu_configure_pub`)
/// and updates `CloneState.failure_mode` directly.
async fn post_configure(
    State(ctx): State<Arc<SlackCloneCtx>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Admin token check.
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

    // BC-3.6.001 Postcondition 5: a structurally-valid payload that requests
    // an unrecognized mode (e.g. a future FailureMode variant) must return HTTP 400
    // with {"error":"unsupported_failure_mode","mode":"<variant-name>"}.  This is
    // distinct from the serde deny_unknown_fields path above (schema-invalid payload).
    match harness_configure_to_failure_mode(&cfg) {
        Ok(mode) => {
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
/// BC-3.6.001 Postcondition-5 body
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

/// Build the axum router for a harness-hosted Slack clone.
///
/// The `/dtu/configure` endpoint uses the harness's configure format
/// (backed by `CloneState.failure_mode`) so that `Harness::inject_failure`
/// works without any format translation.
pub fn build_slack_router(
    clone_state: Arc<CloneState>,
    slack_state: Arc<SlackHarnessState>,
) -> Router {
    let ctx = Arc::new(SlackCloneCtx {
        clone_state,
        slack_state,
    });

    Router::new()
        // Slack Incoming Webhook — wildcard token path (real URLs are multi-segment).
        .route("/services/*token", post(post_webhook))
        // DTU test API
        .route("/dtu/received-payloads", get(get_received_payloads))
        .route("/dtu/reset", post(post_reset))
        .route("/dtu/health", get(get_health))
        // Harness configure (uses harness format, updates CloneState.failure_mode)
        .route("/dtu/configure", post(post_configure))
        .with_state(ctx)
}
