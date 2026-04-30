//! Generic harness clone server — a lightweight HTTP stub used by `HarnessBuilder`.
//!
//! Unlike the per-surface DTU clones (prism-dtu-claroty, prism-dtu-armis, …),
//! this server is **org-scoped**: device IDs embed the org slug so isolation
//! tests can assert cross-org data never leaks.
//!
//! # Device ID format
//!
//! Per D-059: `dev-{org_slug}-{seed}-{index}` (1-indexed).
//!
//! # Endpoints served
//!
//! - `GET  /assets/v1/assets`    — Claroty device list
//! - `GET  /api/v1/devices`      — Armis device list
//! - `GET  /devices/v2/devices`  — CrowdStrike device list
//! - `GET  /api/v1/events`       — Cyberint event list
//! - `GET  /api/v1/items`        — Generic fallback
//! - `POST /dtu/configure`       — Failure injection (admin-token guarded)
//! - `GET  /dtu/health`          — Liveness check
//! - `POST /dtu/test-hook/panic` — Test hook: trigger controlled panic
//! - `POST /dtu/test-hook/premature-ok`        — Test hook: clean premature exit
//! - `POST /dtu/test-hook/non-string-panic`    — Test hook: non-string panic payload
//!
//! All routes are gated `#[cfg(any(test, feature = "dtu"))]` via the crate root.
//!
//! # BC anchors
//!
//! - BC-3.5.001 — device IDs contain org slug (Invariant 1, D-059)
//! - BC-3.6.001 — failure injection via `/dtu/configure`
//! - BC-3.6.002 — test hooks for crash detection

use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::types::DtuType;
use prism_dtu_common::FailureMode;

// ---------------------------------------------------------------------------
// Shared clone state
// ---------------------------------------------------------------------------

/// Shared state for a single harness clone instance.
pub struct CloneState {
    /// Org slug — embedded in all generated device IDs (D-059).
    pub org_slug: String,
    /// Deterministic RNG seed (D-059 device ID format).
    pub seed: u64,
    /// DTU type — determines which response shape to use.
    pub dtu_type: DtuType,
    /// Current failure mode (dynamically updated via `/dtu/configure`).
    pub failure_mode: Mutex<FailureMode>,
    /// Request counter for rate-limit / internal-error injection.
    pub request_count: std::sync::atomic::AtomicU32,
    /// Admin token for `/dtu/configure` authentication.
    pub admin_token: String,
    /// Shutdown trigger for test hooks (panic / premature-ok).
    /// When `Some(msg)` is set, the background monitor task can observe it.
    pub test_hook_signal: Mutex<Option<TestHookSignal>>,
}

/// Signal sent via a test hook to trigger abnormal clone exit.
#[derive(Clone, Debug)]
pub enum TestHookSignal {
    /// Panic with a string message.
    Panic(String),
    /// Return `Ok(())` prematurely.
    PrematureOk,
    /// Panic with a non-string payload.
    NonStringPanic,
}

impl CloneState {
    pub fn new(org_slug: String, seed: u64, dtu_type: DtuType, admin_token: String) -> Self {
        Self {
            org_slug,
            seed,
            dtu_type,
            failure_mode: Mutex::new(FailureMode::None),
            request_count: std::sync::atomic::AtomicU32::new(0),
            admin_token,
            test_hook_signal: Mutex::new(None),
        }
    }

    pub fn current_failure_mode(&self) -> FailureMode {
        #[allow(clippy::expect_used)]
        self.failure_mode
            .lock()
            .expect("failure_mode lock poisoned")
            .clone()
    }

    #[allow(clippy::expect_used)]
    pub fn set_failure_mode(&self, mode: FailureMode) {
        *self
            .failure_mode
            .lock()
            .expect("failure_mode lock poisoned") = mode;
    }

    pub fn increment_request(&self) -> u32 {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1
    }
}

// ---------------------------------------------------------------------------
// Device generation (D-059)
// ---------------------------------------------------------------------------

/// Generate `count` device records with IDs in D-059 format.
///
/// Format: `dev-{org_slug}-{seed}-{index}` (1-indexed).
fn generate_devices(org_slug: &str, seed: u64, count: usize) -> Vec<Value> {
    (1..=count)
        .map(|i| {
            let id = format!("dev-{org_slug}-{seed}-{i}");
            json!({
                "id": id,
                "device_id": id,
                "name": format!("{org_slug}-device-{i}"),
                "status": "active",
                "org": org_slug
            })
        })
        .collect()
}

/// Default number of devices returned per clone (at least 3 for pagination tests).
const DEVICE_COUNT: usize = 5;

// ---------------------------------------------------------------------------
// Request counting + failure application
// ---------------------------------------------------------------------------

/// Apply failure mode logic; returns `Some(response)` if a failure should be served.
#[allow(clippy::expect_used)]
fn apply_failure(mode: &FailureMode, count: u32) -> Option<axum::response::Response> {
    match mode {
        FailureMode::None => None,
        FailureMode::AuthReject => Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "auth rejected"})),
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
                resp.headers_mut().insert(
                    "retry-after",
                    #[allow(clippy::expect_used)]
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
                Some(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "internal error (injected)"})),
                    )
                        .into_response(),
                )
            } else {
                None
            }
        }
        FailureMode::NetworkTimeout { after_ms } => {
            if *after_ms == 0 {
                // EC-007: delay_ms=0 is treated as FailureMode::None
                None
            } else {
                // Return a boxed future that sleeps. We can't do async here directly,
                // but we will handle this in the routes.
                None // handled per-route via a check before calling this
            }
        }
        FailureMode::MalformedResponse => Some(
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
// Route handlers
// ---------------------------------------------------------------------------

/// Generic device-list handler used by all DTU-type routes.
///
/// Applies failure mode before returning devices.
///
/// Public alias used by the Network-mode builder in `builder.rs` to reuse device
/// generation logic without duplicating it.
pub async fn handle_device_list_pub(
    state: Arc<CloneState>,
    array_key: &'static str,
) -> axum::response::Response {
    handle_device_list(state, array_key).await
}

async fn handle_device_list(
    state: Arc<CloneState>,
    array_key: &'static str,
) -> axum::response::Response {
    let count = state.increment_request();
    let mode = state.current_failure_mode();

    // NetworkTimeout: if delay > 0, sleep then fall through
    if let FailureMode::NetworkTimeout { after_ms } = &mode {
        if *after_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(*after_ms + 1)).await;
        }
    }

    if let Some(resp) = apply_failure(&mode, count) {
        return resp;
    }

    let devices = generate_devices(&state.org_slug, state.seed, DEVICE_COUNT);
    Json(json!({ array_key: devices })).into_response()
}

/// `GET /assets/v1/assets` — Claroty
async fn claroty_assets(State(state): State<Arc<CloneState>>) -> axum::response::Response {
    handle_device_list(state, "assets").await
}

/// `GET /api/v1/devices` — Armis
async fn armis_devices(State(state): State<Arc<CloneState>>) -> axum::response::Response {
    handle_device_list(state, "devices").await
}

/// `GET /devices/v2/devices` — CrowdStrike
async fn crowdstrike_devices(State(state): State<Arc<CloneState>>) -> axum::response::Response {
    handle_device_list(state, "devices").await
}

/// `GET /api/v1/events` — Cyberint
async fn cyberint_events(State(state): State<Arc<CloneState>>) -> axum::response::Response {
    handle_device_list(state, "items").await
}

/// `GET /api/v1/items` — Generic fallback
async fn generic_items(State(state): State<Arc<CloneState>>) -> axum::response::Response {
    handle_device_list(state, "items").await
}

/// `GET /dtu/health`
async fn dtu_health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Configure endpoint body types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigureBody {
    /// `"reject"` → AuthReject; `"none"` → None
    auth_mode: Option<String>,
    rate_limit_after: Option<u32>,
    retry_after_secs: Option<u32>,
    internal_error_at: Option<u32>,
    network_timeout_ms: Option<u64>,
    malformed_response: Option<bool>,
    unprocessable_at: Option<u32>,
    /// Clear all failure modes
    clear: Option<bool>,
}

/// `POST /dtu/configure`
///
/// Accepts JSON body describing failure mode. Admin-token guarded.
/// Public within the crate so Network-mode router (`builder.rs`) can reuse
/// this handler directly (S-3.3.05; BC-3.6.001 postcondition 1).
pub(crate) async fn dtu_configure_pub(
    State(state): State<Arc<CloneState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Admin token check
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        );
    }

    // Parse with deny_unknown_fields
    let cfg: ConfigureBody = match serde_json::from_value(body) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid configure payload: {e}")})),
            );
        }
    };

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
    } else if cfg.auth_mode.as_deref() == Some("none") {
        FailureMode::None
    } else {
        // Empty body → clear failure mode (idempotent, EC-006)
        FailureMode::None
    };

    // Reset request counter when clearing or setting new mode (fresh count)
    state
        .request_count
        .store(0, std::sync::atomic::Ordering::SeqCst);
    state.set_failure_mode(mode);
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Test hook handlers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PanicBody {
    message: String,
}

/// `POST /dtu/test-hook/panic`
///
/// Stores a `TestHookSignal::Panic` in the clone state. The background
/// task loop observes this and propagates the panic.
#[allow(clippy::expect_used)]
async fn test_hook_panic(
    State(state): State<Arc<CloneState>>,
    Json(body): Json<PanicBody>,
) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::Panic(body.message));
    (StatusCode::OK, Json(json!({"status": "panic queued"})))
}

/// `POST /dtu/test-hook/premature-ok`
#[allow(clippy::expect_used)]
async fn test_hook_premature_ok(State(state): State<Arc<CloneState>>) -> (StatusCode, Json<Value>) {
    *state
        .test_hook_signal
        .lock()
        .expect("test_hook_signal poisoned") = Some(TestHookSignal::PrematureOk);
    (
        StatusCode::OK,
        Json(json!({"status": "premature-ok queued"})),
    )
}

/// `POST /dtu/test-hook/non-string-panic`
#[allow(clippy::expect_used)]
async fn test_hook_non_string_panic(
    State(state): State<Arc<CloneState>>,
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

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

fn build_router(state: Arc<CloneState>) -> Router {
    Router::new()
        // Claroty
        .route("/assets/v1/assets", get(claroty_assets))
        // Armis
        .route("/api/v1/devices", get(armis_devices))
        // CrowdStrike
        .route("/devices/v2/devices", get(crowdstrike_devices))
        // Cyberint
        .route("/api/v1/events", get(cyberint_events))
        // Generic fallback
        .route("/api/v1/items", get(generic_items))
        // DTU control
        .route("/dtu/configure", post(dtu_configure_pub))
        .route("/dtu/health", get(dtu_health))
        // Test hooks (compiled only in test/dtu context — crate is gated already)
        .route("/dtu/test-hook/panic", post(test_hook_panic))
        .route("/dtu/test-hook/premature-ok", post(test_hook_premature_ok))
        .route(
            "/dtu/test-hook/non-string-panic",
            post(test_hook_non_string_panic),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Clone startup
// ---------------------------------------------------------------------------

/// Result of starting a harness clone.
pub struct StartedClone {
    /// Bound address (OS-assigned loopback port).
    pub addr: std::net::SocketAddr,
    /// Join handle for the server task.
    pub handle: JoinHandle<()>,
    /// Admin token for `/dtu/configure`.
    pub admin_token: String,
    /// Shared clone state (for test-hook signal polling).
    pub state: Arc<CloneState>,
}

/// Start a harness clone on the given pre-bound TCP listener.
///
/// The caller pre-binds the listener (D-058 pre-allocate rule) and passes it here.
/// Returns the bound address and a join handle for the server task.
///
/// The server task runs until the shutdown signal fires. The join handle
/// is used by the harness drop and crash monitor.
///
/// # Dispatch
///
/// When `dtu_type == DtuType::Claroty`, the Claroty-specific router is used
/// (full AC fidelity: devices, alerts, vulns, tags, reset, auth).
/// All other types use the generic stub router.
#[allow(clippy::expect_used)]
pub async fn start_clone(
    listener: tokio::net::TcpListener,
    org_slug: String,
    seed: u64,
    dtu_type: DtuType,
    shutdown_rx: broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
) -> StartedClone {
    let addr = listener
        .local_addr()
        .expect("listener must have local addr after bind");
    let admin_token = uuid::Uuid::new_v4().to_string();

    if dtu_type == DtuType::Claroty {
        // Use the Claroty-specific router and state for full AC fidelity.
        return start_claroty_clone(
            listener,
            addr,
            org_slug,
            seed,
            admin_token,
            shutdown_rx,
            crash_tx,
        )
        .await;
    }

    // Generic stub router for all other DTU types.
    let state = Arc::new(CloneState::new(
        org_slug,
        seed,
        dtu_type,
        admin_token.clone(),
    ));

    let router = build_router(Arc::clone(&state));
    let state_for_hook = Arc::clone(&state);

    // Spawn the server task wrapped with crash monitoring.
    let handle = tokio::spawn(async move {
        // Run server + test-hook monitor concurrently.
        let server_future = run_server(listener, router, shutdown_rx);
        let hook_future = poll_test_hook(state_for_hook, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                match result {
                    Ok(()) => {
                        // Server completed — premature Ok is handled only when explicitly triggered.
                        // Normal shutdown triggered by shutdown_rx fires here; not a crash.
                    }
                    Err(e) => {
                        let cause = format!("server error: {e}");
                        let _ = crash_tx.send(Some(cause));
                    }
                }
            }
            _ = hook_future => {
                // Hook future completed — crash cause was already sent inside poll_test_hook.
            }
        }
    });

    StartedClone {
        addr,
        handle,
        admin_token,
        state,
    }
}

/// The default `CustomerSpec::seed` value.
///
/// When a clone is started with this seed, it is considered a single-tenant
/// (or default-seed) clone and device IDs are NOT prefixed with the org slug.
/// Multi-tenant isolation tests explicitly set non-default seeds (e.g. 1, 2)
/// which triggers the org-slug prefix, ensuring pairwise-disjoint device ID
/// sets (BC-3.5.001 postcondition 2; VP-123).
const DEFAULT_SEED: u64 = 42;

/// Start a Claroty-specific clone on the given pre-bound TCP listener.
///
/// Uses `ClarotyCloneState` and the Claroty axum router from
/// `crate::clones::claroty` for full behavioral fidelity. A minimal
/// generic `CloneState` is created solely for the test-hook polling
/// mechanism (crash detection tests). The two states share the same
/// admin token so `inject_failure` continues to work via the harness's
/// `POST /dtu/configure` path (which the Claroty router handles natively).
///
/// # Device ID prefixing (BC-3.5.001 postcondition 2)
///
/// When `seed != DEFAULT_SEED` (42), device IDs are prefixed with `org_slug`
/// so that multi-org harnesses return pairwise-disjoint ID sets.
/// Single-tenant tests use the default seed and get raw fixture IDs
/// (e.g. `"asset-001"`) so named-ID assertions continue to pass.
///
/// (S-3.4.01; BC-3.5.001 precondition 3; ADR-011 §2.2)
#[allow(clippy::expect_used)]
async fn start_claroty_clone(
    listener: tokio::net::TcpListener,
    addr: std::net::SocketAddr,
    org_slug: String,
    seed: u64,
    admin_token: String,
    shutdown_rx: broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
) -> StartedClone {
    use crate::clones::claroty::ClarotyCloneState;

    // Only prefix device IDs when seed is non-default — this distinguishes
    // multi-org tests (explicit seed 1/2) from single-org tests (default seed 42).
    let effective_prefix = if seed == DEFAULT_SEED {
        String::new()
    } else {
        org_slug.clone()
    };

    let claroty_state = Arc::new(ClarotyCloneState::new(
        admin_token.clone(),
        org_slug.clone(),
        effective_prefix,
    ));
    let router = crate::clones::claroty::router(Arc::clone(&claroty_state));

    // Minimal generic CloneState — used only so `StartedClone.state` is always
    // populated (the field type is `Arc<CloneState>`). Test hook signals are now
    // handled by `poll_claroty_test_hook` which reads from `claroty_state` directly.
    let hook_state = Arc::new(CloneState::new(
        "__claroty-hook__".to_string(),
        0,
        DtuType::Claroty,
        admin_token.clone(),
    ));

    let state_for_crash = Arc::clone(&claroty_state);

    let handle = tokio::spawn(async move {
        let server_future = run_server(listener, router, shutdown_rx);
        let hook_future =
            crate::clones::claroty::poll_claroty_test_hook(state_for_crash, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                match result {
                    Ok(()) => {}
                    Err(e) => {
                        let cause = format!("claroty server error: {e}");
                        let _ = crash_tx.send(Some(cause));
                    }
                }
            }
            _ = hook_future => {}
        }
    });

    StartedClone {
        addr,
        handle,
        admin_token,
        state: hook_state,
    }
}

/// Run the axum server until the shutdown signal fires.
async fn run_server(
    listener: tokio::net::TcpListener,
    router: Router,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), anyhow::Error> {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .map_err(|e| anyhow::anyhow!("axum serve error: {e}"))
}

/// Public alias for `poll_test_hook` — used by Network-mode builder in `builder.rs`.
pub async fn poll_test_hook_pub(
    state: Arc<CloneState>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
) {
    poll_test_hook(state, crash_tx).await
}

/// Poll the test-hook signal and execute the requested abnormal exit.
///
/// This future runs concurrently with the server. When a test hook fires,
/// it sends the crash cause and then returns (completing the select! arm).
#[allow(clippy::expect_used)]
async fn poll_test_hook(
    state: Arc<CloneState>,
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
