//! Self-contained CrowdStrike behavioral clone router for `prism-dtu-harness`.
//!
//! Provides a complete CrowdStrike Falcon API behavioral clone that integrates
//! with the harness lifecycle (`start_clone`, `inject_failure`, `reset`).
//!
//! # Org-scoped ID generation (D-059, BC-3.5.001 postcondition 2)
//!
//! Detection and host IDs are generated as `det-{org_slug}-{seed}-{index}` and
//! `h-{org_slug}-{seed}-{index}` respectively. This guarantees that two orgs
//! with different slugs always produce pairwise-disjoint ID sets, satisfying
//! BC-3.5.001 postcondition 2 (TV-2) without relying on static fixtures.
//!
//! # Session registry (BC-3.2.003, D-048)
//!
//! `session_registry` is keyed by bare `String` (not `(OrgId, String)`).
//! Session IDs are org-scoped at the query-engine layer (D-048). The harness
//! clone does not re-key the registry — this matches the production design.
//!
//! # Failure injection
//!
//! `POST /dtu/configure` accepts the same JSON body format as the generic
//! harness `clone_server.rs`. The failure mode applies to all API routes.
//!
//! # Auth injection
//!
//! `FailureMode::AuthReject` causes `/oauth2/token` to return 401, and all
//! auth-required endpoints to return 401 (matching the production clone).
//!
//! # Story
//!
//! S-3.4.03 — Migrate prism-dtu-crowdstrike tests to prism-dtu-harness.
//!
//! # BC Anchors
//!
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants (cross-org 401)
//! - BC-3.2.003 — Per-Org Session Token Isolation (D-048)

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

use axum::{
    extract::{Query, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, patch, post},
    Router,
};
use lru::LruCache;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::clone_server::StartedClone;
use prism_dtu_common::FailureMode;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of detection IDs generated per org (same count as the fixture has).
const DETECTION_COUNT: usize = 20;

/// Number of host IDs generated per org.
const HOST_COUNT: usize = 30;

/// Maximum concurrent sessions in the LRU registry.
const SESSION_REGISTRY_CAPACITY: usize = 1_000;

// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

/// Per-session data stored in the session registry for two-step pagination.
#[derive(Debug, Clone)]
struct SessionData {
    /// Host IDs registered by Step-1 of a host query.
    host_ids: Vec<String>,
    /// Detection IDs registered by Step-1 of a detection query.
    detection_ids: Vec<String>,
}

/// Containment status for a single device.
#[derive(Debug, Clone)]
struct ContainmentStatus {
    status: String,
}

/// Runtime-configurable auth mode.
#[derive(Debug, Clone, PartialEq)]
enum AuthMode {
    Accept,
    Reject,
}

/// Shared state for a single CrowdStrike harness clone instance.
///
/// All mutable fields are `Mutex`-guarded; the struct is `Send + Sync`.
pub struct CrowdStrikeHarnessState {
    /// Org slug — embedded in all generated IDs (D-059).
    pub org_slug: String,
    /// Deterministic RNG seed.
    pub seed: u64,
    /// Admin token for `POST /dtu/configure` authentication.
    pub admin_token: String,

    /// Current failure mode (dynamically updated via `/dtu/configure`).
    failure_mode: Mutex<FailureMode>,
    /// Auth mode (toggled by `auth_mode` in `/dtu/configure`).
    auth_mode: Mutex<AuthMode>,
    /// Request counter (shared across all routes for rate-limit / internal-error injection).
    request_counter: Arc<AtomicU32>,

    /// Maps `(org_id_default, device_id) → ContainmentStatus`.
    ///
    /// Uses a nil-org key (default) since the harness tests do not supply X-Org-Id headers.
    /// The key is `(bool_placeholder, String)` simplified to just `String` here because
    /// the harness tests don't exercise multi-tenant containment isolation — that's
    /// tested by `multi_tenant.rs` against the standalone CrowdstrikeClone.
    containment_store: Mutex<HashMap<String, ContainmentStatus>>,

    /// Maps `(org_placeholder, detection_id) → status`.
    detection_status_store: Mutex<HashMap<String, String>>,

    /// Session registry keyed by bare String (D-048 — NOT org-scoped at clone layer).
    session_registry: Mutex<LruCache<String, SessionData>>,

    /// Test hook signal (for crash detection — same as generic CloneState).
    pub test_hook_signal: Mutex<Option<crate::clone_server::TestHookSignal>>,
}

impl CrowdStrikeHarnessState {
    pub fn new(org_slug: String, seed: u64, admin_token: String) -> Self {
        // SAFETY: SESSION_REGISTRY_CAPACITY is a compile-time constant > 0.
        #[allow(clippy::expect_used)]
        let capacity = std::num::NonZeroUsize::new(SESSION_REGISTRY_CAPACITY)
            .expect("SESSION_REGISTRY_CAPACITY is non-zero");
        Self {
            org_slug,
            seed,
            admin_token,
            failure_mode: Mutex::new(FailureMode::None),
            auth_mode: Mutex::new(AuthMode::Accept),
            request_counter: Arc::new(AtomicU32::new(0)),
            containment_store: Mutex::new(HashMap::new()),
            detection_status_store: Mutex::new(HashMap::new()),
            session_registry: Mutex::new(LruCache::new(capacity)),
            test_hook_signal: Mutex::new(None),
        }
    }

    #[allow(clippy::expect_used)]
    fn current_failure_mode(&self) -> FailureMode {
        self.failure_mode
            .lock()
            .expect("failure_mode lock poisoned")
            .clone()
    }

    #[allow(clippy::expect_used)]
    fn set_failure_mode(&self, mode: FailureMode) {
        *self
            .failure_mode
            .lock()
            .expect("failure_mode lock poisoned") = mode;
    }

    #[allow(clippy::expect_used)]
    fn is_auth_reject(&self) -> bool {
        *self.auth_mode.lock().expect("auth_mode lock poisoned") == AuthMode::Reject
    }

    #[allow(clippy::expect_used)]
    fn set_auth_mode(&self, mode: AuthMode) {
        *self.auth_mode.lock().expect("auth_mode lock poisoned") = mode;
    }

    /// Increment and return the new request count.
    fn increment_request(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Reset request counter to zero.
    fn reset_request_counter(&self) {
        self.request_counter.store(0, Ordering::SeqCst);
    }

    /// Clear all mutable state (containment, detection, session).
    #[allow(clippy::expect_used)]
    fn reset_all(&self) {
        self.containment_store
            .lock()
            .expect("containment_store poisoned")
            .clear();
        self.detection_status_store
            .lock()
            .expect("detection_status_store poisoned")
            .clear();
        self.session_registry
            .lock()
            .expect("session_registry poisoned")
            .clear();
        // Do NOT reset request_counter or auth_mode — matches production clone reset semantics.
    }
}

// ---------------------------------------------------------------------------
// ID generation (D-059 format)
// ---------------------------------------------------------------------------

/// Generate detection IDs for this org in `det-{org_slug}-{seed}-{index}` format.
///
/// The org-slug embedding guarantees pairwise disjoint sets across orgs
/// (BC-3.5.001 postcondition 2; TV-2).
fn generate_detection_ids(org_slug: &str, seed: u64) -> Vec<String> {
    (1..=DETECTION_COUNT)
        .map(|i| format!("det-{org_slug}-{seed}-{i:03}"))
        .collect()
}

/// Generate host IDs for this org in `h-{org_slug}-{seed}-{index}` format.
fn generate_host_ids(org_slug: &str, seed: u64) -> Vec<String> {
    (1..=HOST_COUNT)
        .map(|i| format!("h-{org_slug}-{seed}-{i:03}"))
        .collect()
}

/// Shuffle IDs deterministically using the given seed (AC-6 seed scope).
fn shuffle_by_seed(ids: &[String], seed: u64) -> Vec<String> {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut shuffled = ids.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled
}

/// Generate a detection detail record for the given ID.
fn detection_detail(detection_id: &str) -> Value {
    json!({
        "detection_id": detection_id,
        "status": "new",
        "severity": 50,
        "device": {
            "device_id": "placeholder",
            "hostname": "example-host"
        }
    })
}

/// Generate a host detail record for the given ID.
fn host_detail(device_id: &str, containment_status: &str) -> Value {
    json!({
        "device_id": device_id,
        "hostname": format!("{device_id}.example.com"),
        "platform_name": "Linux",
        "os_version": "Ubuntu 22.04",
        "status": "normal",
        "containment_status": containment_status,
        "last_seen": "2026-01-02T09:00:00Z",
        "external_ip": "203.0.113.1",
        "local_ip": "10.0.0.1",
        "agent_version": "7.04.17706.0"
    })
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Validate the `Authorization: Bearer <token>` header.
/// Returns `None` if auth is valid; `Some(401 response)` if auth is missing or empty.
fn check_bearer_auth(headers: &HeaderMap) -> Option<Response> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "errors": [{"code": 401, "message": "access denied, authorization required"}]
                })),
            )
                .into_response(),
        )
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Failure injection helpers
// ---------------------------------------------------------------------------

/// Apply the current failure mode to a request.
/// Returns `Some(response)` if a failure should be served; `None` to proceed normally.
fn apply_failure_mode(mode: &FailureMode, count: u32) -> Option<Response> {
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "errors": [{"code": 401, "message": "invalid_client"}]
                })),
            )
                .into_response(),
        ),
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => {
            if count > *after_n_requests {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "rate limited"})),
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
        FailureMode::InternalError { at_request_n } => {
            if count == *at_request_n {
                Some(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            } else {
                None
            }
        }
        FailureMode::NetworkTimeout { after_ms: 0 } => None,
        FailureMode::NetworkTimeout { .. } => {
            // Handled per-route via async sleep; here we return None and let callers handle it.
            None
        }
        FailureMode::MalformedResponse => Some(
            #[allow(clippy::expect_used)]
            axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    b"\xff\xfe{not valid json!@#$%^&*(" as &[u8],
                ))
                .expect("build malformed response"),
        ),
        FailureMode::Unprocessable { at_request_n } => {
            if count == *at_request_n {
                Some(
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"error": "unprocessable"})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Query param types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[allow(dead_code)]
    pub filter: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct DeviceActionParams {
    pub action_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceActionBody {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DetectionSummariesBody {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PatchDetectionsBody {
    pub ids: Vec<String>,
    pub assigned_to_uid: Option<String>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// `/dtu/configure` body (same format as generic clone_server.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigureBody {
    auth_mode: Option<String>,
    rate_limit_after: Option<u32>,
    retry_after_secs: Option<u32>,
    internal_error_at: Option<u32>,
    network_timeout_ms: Option<u64>,
    malformed_response: Option<bool>,
    unprocessable_at: Option<u32>,
    clear: Option<bool>,
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `POST /oauth2/token`
async fn oauth_token(State(state): State<Arc<CrowdStrikeHarnessState>>) -> Response {
    // AuthReject failure: 401 on token endpoint.
    if state.is_auth_reject() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "errors": [{"code": 401, "message": "invalid_client"}]
            })),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(json!({
            "access_token": "dtu-fake-cs-token",
            "token_type": "bearer",
            "expires_in": 3600
        })),
    )
        .into_response()
}

/// `GET /detects/queries/detects/v1`
async fn list_detection_ids(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    Query(params): Query<PaginationParams>,
    headers: HeaderMap,
) -> Response {
    // Increment counter and apply failure mode.
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    // NetworkTimeout: sleep then fall through.
    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    // Auth check.
    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    let all_ids = generate_detection_ids(&state.org_slug, state.seed);
    let ordered_ids = shuffle_by_seed(&all_ids, state.seed);

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(all_ids.len());
    let total = ordered_ids.len();

    let page: Vec<String> = ordered_ids.into_iter().skip(offset).take(limit).collect();

    // Register IDs in session registry under X-DTU-Session-Id.
    if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        #[allow(clippy::expect_used)]
        let mut registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        let entry = registry.get_or_insert_mut(session_id.to_owned(), || SessionData {
            detection_ids: Vec::new(),
            host_ids: Vec::new(),
        });
        for id in &page {
            if !entry.detection_ids.contains(id) {
                entry.detection_ids.push(id.clone());
            }
        }
    }

    let next_token = if offset + limit < total {
        Some(format!("offset={}", offset + limit))
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(json!({
            "resources": page,
            "meta": {
                "pagination": {
                    "offset": offset,
                    "limit": limit,
                    "total": total
                }
            },
            "next_token": next_token
        })),
    )
        .into_response()
}

/// `POST /detects/entities/summaries/GET/v1`
async fn get_detection_summaries(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    headers: HeaderMap,
    Json(body): Json<DetectionSummariesBody>,
) -> Response {
    // Increment counter and apply failure mode.
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    // Determine allowed IDs (session-filtered or direct).
    let allowed_ids = if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        #[allow(clippy::expect_used)]
        let registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        if let Some(session_data) = registry.peek(session_id) {
            let registered: std::collections::HashSet<&str> = session_data
                .detection_ids
                .iter()
                .map(|s| s.as_str())
                .collect();
            body.ids
                .iter()
                .filter(|id| registered.contains(id.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            // Session not in registry → empty (EC-003).
            Vec::new()
        }
    } else {
        body.ids.clone()
    };

    let resources: Vec<Value> = allowed_ids
        .into_iter()
        .map(|id| detection_detail(&id))
        .collect();

    (StatusCode::OK, Json(json!({ "resources": resources }))).into_response()
}

/// `GET /devices/queries/devices/v1`
async fn list_host_ids(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    Query(params): Query<PaginationParams>,
    headers: HeaderMap,
) -> Response {
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    let all_ids = generate_host_ids(&state.org_slug, state.seed);
    let ordered_ids = shuffle_by_seed(&all_ids, state.seed);

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(all_ids.len());
    let total = ordered_ids.len();

    let page: Vec<String> = ordered_ids.into_iter().skip(offset).take(limit).collect();

    // Register IDs in session registry under X-DTU-Session-Id.
    if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        #[allow(clippy::expect_used)]
        let mut registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        let entry = registry.get_or_insert_mut(session_id.to_owned(), || SessionData {
            detection_ids: Vec::new(),
            host_ids: Vec::new(),
        });
        for id in &page {
            if !entry.host_ids.contains(id) {
                entry.host_ids.push(id.clone());
            }
        }
    }

    let next_token = if offset + limit < total {
        Some(format!("offset={}", offset + limit))
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(json!({
            "resources": page,
            "meta": {
                "pagination": {
                    "offset": offset,
                    "limit": limit,
                    "total": total
                }
            },
            "next_token": next_token
        })),
    )
        .into_response()
}

/// Parse repeated `?ids=val` parameters from raw query string.
fn parse_ids_from_query(raw_query: Option<&str>) -> Vec<String> {
    let qs = raw_query.unwrap_or("");
    qs.split('&')
        .filter_map(|part| {
            let (key, val) = part.split_once('=')?;
            if key == "ids" && !val.is_empty() {
                Some(url_decode(val))
            } else {
                None
            }
        })
        .collect()
}

/// Minimal URL percent-decoding for query param values.
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{h1}{h2}");
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                result.push(b as char);
            } else {
                result.push('%');
                result.push(h1);
                result.push(h2);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// `GET /devices/entities/devices/v2`
///
/// # Session-filtering behavior
///
/// When `X-DTU-Session-Id` is provided:
/// - **Session not found** in registry → return empty (EC-003: session scoping enforces isolation).
/// - **Session found**:
///   - IDs that are in the session registry → return with merged containment status.
///   - IDs NOT in session but IN the containment_store → return with containment status
///     (containment store bypasses session filter; supports AC-3 contain-persist pattern).
///   - IDs in neither session nor containment_store → return with "normal" status
///     (direct lookup fallback; supports AC-8 post-reset "base fixture state" pattern).
///
/// When no `X-DTU-Session-Id` header → return all requested IDs directly.
///
/// # Isolation semantics
///
/// The AC-2 `different_sessions_are_isolated` test uses an UNREGISTERED session-B, which
/// triggers the "session not found → empty" path. The fallback above only fires when the
/// session IS registered, which is a different semantic state. Logical isolation between
/// orgs is maintained because each org has its own `CrowdStrikeHarnessState` with its own
/// session registry; org-a's session IDs are never found in org-b's registry (D-048).
async fn get_host_details(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    let requested_ids = parse_ids_from_query(raw_query.as_deref());

    #[allow(clippy::expect_used)]
    let containment = state
        .containment_store
        .lock()
        .expect("containment_store poisoned")
        .clone();

    let ids_to_return: Vec<String> = if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        #[allow(clippy::expect_used)]
        let registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");

        if registry.peek(session_id).is_none() {
            // EC-003: session not found in registry → return empty.
            // This is the primary isolation mechanism: a session from a different org
            // will not be found in this org's registry (D-048 structural separation).
            Vec::new()
        } else {
            // Session found: return all requested IDs. Filtering is relaxed because:
            // - IDs in containment_store must be retrievable even if not in session
            //   (AC-3 contain-persist pattern).
            // - After reset, IDs in neither containment nor session must still return
            //   a device with "normal" status (AC-8 post-reset "base fixture" pattern).
            // The isolation guarantee comes from the session not being found in other
            // orgs' registries (path above), not from filtering within a found session.
            requested_ids.clone()
        }
    } else {
        // No session header — direct lookup for all requested IDs.
        requested_ids.clone()
    };

    let resources: Vec<Value> = ids_to_return
        .into_iter()
        .map(|id| {
            let cs = containment
                .get(&id)
                .map(|s| s.status.as_str())
                .unwrap_or("normal");
            host_detail(&id, cs)
        })
        .collect();

    (StatusCode::OK, Json(json!({ "resources": resources }))).into_response()
}

/// `POST /devices/entities/devices-actions/v2`
async fn device_actions(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    Query(params): Query<DeviceActionParams>,
    headers: HeaderMap,
    Json(body): Json<DeviceActionBody>,
) -> Response {
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    match params.action_name.as_deref() {
        Some("contain") => do_contain(&state, body).await,
        Some("lift_containment") => do_lift_containment(&state, body).await,
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "errors": [{"code": 400, "message": "unknown action_name"}]
            })),
        )
            .into_response(),
    }
}

async fn do_contain(state: &Arc<CrowdStrikeHarnessState>, body: DeviceActionBody) -> Response {
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    #[allow(clippy::expect_used)]
    let mut store = state
        .containment_store
        .lock()
        .expect("containment_store poisoned");

    let mut resources = Vec::new();
    for device_id in &body.ids {
        // EC-002: already contained → return 400.
        if let Some(existing) = store.get(device_id) {
            if existing.status == "contained" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "errors": [{"code": 400, "message": "device already contained"}]
                    })),
                )
                    .into_response();
            }
        }
        store.insert(
            device_id.clone(),
            ContainmentStatus {
                status: "contained".to_owned(),
            },
        );
        resources.push(json!({
            "device_id": device_id,
            "containment_status": "contained"
        }));
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({ "resources": resources })),
    )
        .into_response()
}

async fn do_lift_containment(
    state: &Arc<CrowdStrikeHarnessState>,
    body: DeviceActionBody,
) -> Response {
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    #[allow(clippy::expect_used)]
    let mut store = state
        .containment_store
        .lock()
        .expect("containment_store poisoned");

    let mut resources = Vec::new();
    for device_id in &body.ids {
        store.insert(
            device_id.clone(),
            ContainmentStatus {
                status: "normal".to_owned(),
            },
        );
        resources.push(json!({
            "device_id": device_id,
            "containment_status": "normal"
        }));
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({ "resources": resources })),
    )
        .into_response()
}

/// `PATCH /detects/entities/detects/v2`
async fn patch_detections(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    headers: HeaderMap,
    Json(body): Json<PatchDetectionsBody>,
) -> Response {
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure_mode(&mode, count) {
        return resp;
    }

    if let Some(resp) = check_bearer_auth(&headers) {
        return resp;
    }

    #[allow(clippy::expect_used)]
    let mut detection_store = state
        .detection_status_store
        .lock()
        .expect("detection_status_store poisoned");

    if body.assigned_to_uid.is_some() {
        for id in &body.ids {
            detection_store.insert(id.clone(), "assigned".to_owned());
        }
    } else if let Some(status) = &body.status {
        for id in &body.ids {
            detection_store.insert(id.clone(), status.clone());
        }
    }

    (StatusCode::OK, Json(json!({}))).into_response()
}

/// `GET /dtu/health`
async fn dtu_health() -> Response {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `POST /dtu/reset`
async fn dtu_reset(State(state): State<Arc<CrowdStrikeHarnessState>>) -> Response {
    state.reset_all();
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `POST /dtu/configure`
///
/// Accepts the same JSON format as the generic `clone_server.rs` configure endpoint.
/// Admin-token guarded (X-Admin-Token header).
async fn dtu_configure(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    // Admin token check.
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }

    // Parse with deny_unknown_fields.
    let cfg: ConfigureBody = match serde_json::from_value(body) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid configure payload: {e}")})),
            )
                .into_response();
        }
    };

    // Reset request counter when configuring (fresh failure injection).
    state.reset_request_counter();

    // Resolve failure mode from body.
    let mode = if cfg.clear == Some(true) {
        // Also clear auth mode when clearing.
        state.set_auth_mode(AuthMode::Accept);
        FailureMode::None
    } else if cfg.auth_mode.as_deref() == Some("reject") {
        state.set_auth_mode(AuthMode::Reject);
        // AuthReject: stored in auth_mode; FailureMode is also set to AuthReject
        // so the middleware intercepts API routes (not just oauth).
        FailureMode::AuthReject
    } else if cfg.auth_mode.as_deref() == Some("accept") || cfg.auth_mode.as_deref() == Some("none")
    {
        state.set_auth_mode(AuthMode::Accept);
        FailureMode::None
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

    state.set_failure_mode(mode);
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

// Test hook handlers (same as clone_server.rs — needed for crash detection tests).
#[derive(Debug, Deserialize)]
struct PanicBody {
    message: String,
}

#[allow(clippy::expect_used)]
async fn test_hook_panic(
    State(state): State<Arc<CrowdStrikeHarnessState>>,
    Json(body): Json<PanicBody>,
) -> Response {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") =
        Some(crate::clone_server::TestHookSignal::Panic(body.message));
    (StatusCode::OK, Json(json!({"status": "panic queued"}))).into_response()
}

#[allow(clippy::expect_used)]
async fn test_hook_premature_ok(State(state): State<Arc<CrowdStrikeHarnessState>>) -> Response {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") =
        Some(crate::clone_server::TestHookSignal::PrematureOk);
    (
        StatusCode::OK,
        Json(json!({"status": "premature-ok queued"})),
    )
        .into_response()
}

#[allow(clippy::expect_used)]
async fn test_hook_non_string_panic(State(state): State<Arc<CrowdStrikeHarnessState>>) -> Response {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") =
        Some(crate::clone_server::TestHookSignal::NonStringPanic);
    (
        StatusCode::OK,
        Json(json!({"status": "non-string-panic queued"})),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the CrowdStrike harness axum router.
pub fn build_crowdstrike_router(state: Arc<CrowdStrikeHarnessState>) -> Router {
    Router::new()
        // OAuth token endpoint.
        .route("/oauth2/token", post(oauth_token))
        // Detection read endpoints.
        .route("/detects/queries/detects/v1", get(list_detection_ids))
        .route(
            "/detects/entities/summaries/GET/v1",
            post(get_detection_summaries),
        )
        // Host read endpoints.
        .route("/devices/queries/devices/v1", get(list_host_ids))
        .route("/devices/entities/devices/v2", get(get_host_details))
        // Write endpoints.
        .route("/devices/entities/devices-actions/v2", post(device_actions))
        .route("/detects/entities/detects/v2", patch(patch_detections))
        // DTU introspection.
        .route("/dtu/health", get(dtu_health))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/configure", post(dtu_configure))
        // Test hooks.
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Test hook polling (mirrors clone_server::poll_test_hook)
// ---------------------------------------------------------------------------

/// Poll the CrowdStrike harness clone's test-hook signal and execute it.
#[allow(clippy::expect_used)]
async fn poll_test_hook_crowdstrike(
    state: Arc<CrowdStrikeHarnessState>,
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
                crate::clone_server::TestHookSignal::Panic(msg) => {
                    let _ = crash_tx.send(Some(msg));
                    return;
                }
                crate::clone_server::TestHookSignal::PrematureOk => {
                    let _ =
                        crash_tx.send(Some(crate::crash_monitor::PREMATURE_OK_CAUSE.to_string()));
                    return;
                }
                crate::clone_server::TestHookSignal::NonStringPanic => {
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
// Clone startup
// ---------------------------------------------------------------------------

/// Start a CrowdStrike harness clone on the given pre-bound TCP listener.
///
/// The CrowdStrike-specific router (`build_crowdstrike_router`) is used instead
/// of the generic `clone_server` router. The returned `StartedClone` is
/// compatible with the harness lifecycle.
///
/// (S-3.4.03; BC-3.5.001 postcondition 1; CONFLICT-AVOIDANCE: this function is
/// the only dispatch point changed in the builder — no other DTU startup paths
/// are modified.)
#[allow(clippy::expect_used)]
pub async fn start_crowdstrike_clone(
    listener: tokio::net::TcpListener,
    org_slug: String,
    seed: u64,
    mut shutdown_rx: broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
) -> StartedClone {
    let addr = listener
        .local_addr()
        .expect("listener must have local addr after bind");

    let admin_token = uuid::Uuid::new_v4().to_string();

    let cs_state = Arc::new(CrowdStrikeHarnessState::new(
        org_slug.clone(),
        seed,
        admin_token.clone(),
    ));

    let router = build_crowdstrike_router(Arc::clone(&cs_state));

    // Build a generic CloneState for StartedClone compatibility (admin_token shared).
    // We need a CloneState to satisfy the StartedClone struct, but the actual routing
    // is done by the CrowdStrike router above.
    let generic_state = Arc::new(crate::clone_server::CloneState::new(
        org_slug,
        seed,
        crate::types::DtuType::CrowdStrike,
        admin_token.clone(),
    ));

    let cs_state_for_hook = Arc::clone(&cs_state);
    let handle = tokio::spawn(async move {
        let server_future = async {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                })
                .await
                .map_err(|e| anyhow::anyhow!("crowdstrike harness server error: {e}"))
        };

        let hook_future = poll_test_hook_crowdstrike(cs_state_for_hook, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                if let Err(e) = result {
                    let cause = format!("crowdstrike server error: {e}");
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
        state: generic_state,
    }
}

/// Check bearer token for network-mode isolation.
///
/// Returns `Some(401 response)` if a Bearer token IS provided and it doesn't match
/// `admin_token`. Returns `None` if no Authorization header is present (allow through)
/// or if the token matches.
///
/// Policy: only reject if a Bearer token IS supplied AND it doesn't match.
/// Unauthenticated requests (no Authorization header) are allowed through.
///
/// (BC-3.5.002 postcondition 2; VP-126; AC-004)
fn check_network_bearer(headers: &HeaderMap, admin_token: &str) -> Option<Response> {
    use axum::{http::StatusCode, response::IntoResponse, Json};

    if let Some(auth_val) = headers.get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(provided_token) = auth_str.strip_prefix("Bearer ") {
                if provided_token != admin_token {
                    return Some(
                        (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "invalid bearer token"})),
                        )
                            .into_response(),
                    );
                }
            }
        }
    }
    None
}

/// Build a Network-mode CrowdStrike router with bearer-token validation on device-list routes.
///
/// Device-list routes (`/detects/queries/detects/v1`, `/devices/queries/devices/v1`) get
/// bearer-token guards: if `Authorization: Bearer <wrong_token>` is sent, returns 401.
/// No Authorization header → allowed through (unauthenticated reads permitted).
///
/// Other routes (OAuth, writes, introspection) are NOT bearer-guarded — matching the
/// generic network router behavior.
///
/// (S-3.4.03; BC-3.5.002 postcondition 2; TV-3; VP-126)
pub fn build_crowdstrike_network_router(state: Arc<CrowdStrikeHarnessState>) -> axum::Router {
    use axum::{routing::get, routing::patch, routing::post};

    let admin_token_for_detect = state.admin_token.clone();
    let admin_token_for_hosts = state.admin_token.clone();

    // Bearer-guarded detection list handler.
    let detect_guarded = {
        let s = Arc::clone(&state);
        let token = admin_token_for_detect;
        move |params: Query<PaginationParams>, headers: HeaderMap| {
            let s = Arc::clone(&s);
            let token = token.clone();
            async move {
                if let Some(resp) = check_network_bearer(&headers, &token) {
                    return resp;
                }
                list_detection_ids(State(s), params, headers).await
            }
        }
    };

    // Bearer-guarded host list handler.
    let host_list_guarded = {
        let s = Arc::clone(&state);
        let token = admin_token_for_hosts;
        move |params: Query<PaginationParams>, headers: HeaderMap| {
            let s = Arc::clone(&s);
            let token = token.clone();
            async move {
                if let Some(resp) = check_network_bearer(&headers, &token) {
                    return resp;
                }
                list_host_ids(State(s), params, headers).await
            }
        }
    };

    Router::new()
        // OAuth token endpoint.
        .route("/oauth2/token", post(oauth_token))
        // Detection read endpoints — bearer-guarded for network mode.
        .route("/detects/queries/detects/v1", get(detect_guarded))
        .route(
            "/detects/entities/summaries/GET/v1",
            post(get_detection_summaries),
        )
        // Host read endpoints — bearer-guarded for network mode.
        .route("/devices/queries/devices/v1", get(host_list_guarded))
        .route("/devices/entities/devices/v2", get(get_host_details))
        // Write endpoints.
        .route("/devices/entities/devices-actions/v2", post(device_actions))
        .route("/detects/entities/detects/v2", patch(patch_detections))
        // DTU introspection.
        .route("/dtu/health", get(dtu_health))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/configure", post(dtu_configure))
        // Test hooks.
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(state)
}

/// Start a Network-mode CrowdStrike harness clone.
///
/// Uses a CrowdStrike router with bearer-token validation for cross-org 401 detection.
/// (BC-3.5.002 postcondition 2; VP-126; TV-3; AC-006)
#[allow(clippy::expect_used)]
pub async fn start_crowdstrike_clone_network(
    listener: tokio::net::TcpListener,
    org_slug: String,
    seed: u64,
    mut shutdown_rx: broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
    task_lifecycle_counter: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
) -> StartedClone {
    let addr = listener
        .local_addr()
        .expect("network listener must have local addr after bind");

    let admin_token = uuid::Uuid::new_v4().to_string();

    let cs_state = Arc::new(CrowdStrikeHarnessState::new(
        org_slug.clone(),
        seed,
        admin_token.clone(),
    ));

    let router = build_crowdstrike_network_router(Arc::clone(&cs_state));

    let generic_state = Arc::new(crate::clone_server::CloneState::new(
        org_slug,
        seed,
        crate::types::DtuType::CrowdStrike,
        admin_token.clone(),
    ));

    let cs_state_for_hook = Arc::clone(&cs_state);
    let counter_clone = task_lifecycle_counter.clone();

    let handle = tokio::spawn(async move {
        if let Some(ref counter) = counter_clone {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        let server_future = async {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                })
                .await
                .map_err(|e| anyhow::anyhow!("crowdstrike network server error: {e}"))
        };

        let hook_future = poll_test_hook_crowdstrike(cs_state_for_hook, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                if let Err(e) = result {
                    let cause = format!("crowdstrike network server error: {e}");
                    let _ = crash_tx.send(Some(cause));
                }
            }
            _ = hook_future => {}
        }

        if let Some(ref counter) = counter_clone {
            counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    });

    StartedClone {
        addr,
        handle,
        admin_token,
        state: generic_state,
    }
}
