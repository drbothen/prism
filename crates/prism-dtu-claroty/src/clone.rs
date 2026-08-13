//! `ClarotyClone` — implements `BehavioralClone` for the Claroty xDome DTU.
//!
//! Binds to `127.0.0.1:0` (ephemeral port) on `start()`, spawns an axum
//! server with `LatencyLayer` + `FailureLayer`, and serves 9 application
//! endpoints (7 read via POST-body filtering, 2 write via stateful tag store)
//! plus the DTU control endpoints.  The 7 read endpoints are: `list_devices`,
//! `list_alerts`, `list_alerted_devices`, `list_vulnerabilities`,
//! `list_vulnerability_devices`, `list_audit_logs` (added by
//! S-DEMO-CLAROTY-AUDIT-DTU-001), and `list_device_alert_relations` (Tier 3).
//! See `dtu-assessment.md §3.2` for the original 7-endpoint scope matrix;
//! `audit_log` and `device_alert_relations` are Wave-5 fidelity additions beyond
//! that baseline.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum::{
    routing::{delete, get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::{net::TcpListener, sync::broadcast, task::JoinHandle};
use tower::Layer as _;
use tower_http::normalize_path::NormalizePathLayer;

use crate::{
    routes::{alerts, audit_log, device_alert_relations, devices, tags, vulnerabilities},
    state::ClarotyState,
};

/// L4 (adversarial) behavioral clone of the Claroty xDome API.
///
/// Maintains a stateful device tag store and supports full failure injection
/// via `LatencyLayer` + `FailureLayer` from `prism-dtu-common`.
///
/// Binds to an ephemeral port on `127.0.0.1`; use `base_url()` to construct
/// HTTP client URLs in tests.
pub struct ClarotyClone {
    pub config: StubConfig,
    pub state: Arc<ClarotyState>,
    pub bound_addr: Option<SocketAddr>,
    pub server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
    /// Internal broadcast sender for graceful HTTP shutdown (S-PERF-GATE-005).
    ///
    /// Set by `start_on` when `shutdown=None` (the direct `start()` path). Fired by
    /// `stop()` so the server task drains immediately instead of timing out on the
    /// 5s select! fallback. `None` when the harness-provided or TLS path is used.
    internal_shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl ClarotyClone {
    /// Create a new clone with default `StubConfig` and empty tag store.
    pub fn new() -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config: StubConfig::default(),
            state: Arc::new(ClarotyState::with_admin_token(admin_token.clone())),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
        }
    }

    // -----------------------------------------------------------------------
    // Story A: new_with_seed constructor stub (BC-2.06.018 / ADR-036 §2.3)
    // -----------------------------------------------------------------------

    /// Construct a `ClarotyClone` with deterministic fixture data generated at
    /// construction time from `(seed, archetype, org_id)`.
    ///
    /// Gated `#[cfg(feature = "fixture-gen")]`.
    ///
    /// Sets `state.fixture_gen_seeded = true`. Route handlers check this flag (not
    /// `generated_records.is_empty()`) as the dual-path sentinel so that
    /// `Archetype::DormantTenant` (seeded=true, 0 records) serves EMPTY — it does
    /// NOT fall back to the static-JSON path. F-P6-HIGH-001 / ADR-036 v2.2.
    ///
    /// `ClarotyClone::new()` is unchanged (backward-compatible, ADR-036 §2.5);
    /// it leaves `fixture_gen_seeded = false` and route handlers use the static fixture.
    ///
    /// `ClarotyClone::new_with_seed` is INFALLIBLE (`-> Self`) per ADR-036 §2.3
    /// (mirrors the existing infallible `ClarotyClone::new()`).
    ///
    /// ADR-036 v2.2: canonical 3-arg form — `archetype` is forwarded to `generate()`;
    /// NO hardcoded archetype inside this constructor.
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_seed(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
    ) -> Self {
        // Default anchor: fixed demo-era constant (review-2026-06-10 P1-01).
        Self::new_with_seed_anchored(
            seed,
            archetype,
            org_id,
            prism_dtu_common::demo_time_anchor(),
        )
    }

    /// `new_with_seed` with an explicit `time_anchor` for generated timestamps.
    ///
    /// Review-2026-06-10 P1-01: exposes the `GenOpts::time_anchor` input on the
    /// construction chain so callers can anchor generated data to a chosen era.
    /// Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) wires `scenario_start_secs` →
    /// `time_anchor`; this fix-burst deliberately does NOT wire
    /// `ScenarioConfig.scenario_start_secs` into generation (BC-2.06.019 scope).
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_seed_anchored(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        use crate::generator::generate;
        use prism_dtu_common::GenOpts;

        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture = generate(&org_id, archetype, &opts);

        let admin_token = uuid::Uuid::new_v4().to_string();
        let mut state = ClarotyState::with_admin_token(admin_token.clone());
        state.generated_records = fixture.records;
        // Mark as seeded so route handlers use the generated path (even for DormantTenant
        // which produces 0 records). Without this flag, DormantTenant would fall back to
        // the static fixture — violating BC EC-018-003 / F-P6-HIGH-001.
        state.fixture_gen_seeded = true;

        Self {
            config: prism_dtu_common::StubConfig::default(),
            state: Arc::new(state),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
        }
    }

    // -----------------------------------------------------------------------
    // Story B: new_with_scenario constructor (BC-2.06.019 / ADR-036 v2.3 §2.4)
    // -----------------------------------------------------------------------

    /// Construct a `ClarotyClone` with the scenario timeline layer.
    ///
    /// 5-arg form per ADR-036 v2.3 §2.4. Gated `#[cfg(feature = "fixture-gen")]`
    /// because `chrono::DateTime<Utc>` is only available under `fixture-gen`
    /// in this crate (dep:chrono gating in Cargo.toml).
    ///
    /// Internally calls `new_with_seed_anchored(seed, archetype, org_id, time_anchor)`
    /// (NOT the forbidden 3-arg `new_with_seed` which would produce stale timestamps).
    ///
    /// Sets `state.timeline = Some(Arc::clone(&timeline))` so route handlers can
    /// compute the current stage index and apply StageMask filtering.
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_scenario(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        timeline: std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        // Call new_with_seed_anchored (NOT the 3-arg new_with_seed) to use the
        // caller-supplied time_anchor for era-coherent generated timestamps.
        // ADR-036 v2.3 §2.3 mandates this; the 3-arg path is FORBIDDEN here.
        let mut clone = Self::new_with_seed_anchored(seed, archetype, org_id, time_anchor);
        // Attach the timeline BEFORE any other reference can be taken.
        //
        // Structural threading (B-P5-OBS-1): use Arc::try_unwrap to reclaim the state
        // struct (refcount=1 immediately post-construction), set timeline, then re-wrap.
        // This is safe because new_with_seed_anchored just returned the only Arc clone;
        // no other thread can hold a reference at this point.
        //
        // Prefer try_unwrap over get_mut to avoid silent-drop risk: if a future refactor
        // creates a second Arc clone before this point, try_unwrap returns Err with the
        // original Arc, which we catch with expect() so the bug is loud.
        //
        // ADR-036 v2.3 §2.3: Arc<IncidentTimeline> is read-only after construction.
        let mut state = Arc::try_unwrap(clone.state).unwrap_or_else(|_| {
            panic!(
                "ClarotyClone::new_with_scenario: Arc refcount must be 1 immediately after \
                 new_with_seed_anchored; a second Arc clone would indicate a refactor \
                 invariant violation (B-P5-OBS-1)"
            )
        });
        state.timeline = Some(Arc::clone(&timeline));
        clone.state = Arc::new(state);
        clone
    }

    /// Create with explicit configuration.
    pub fn with_config(config: StubConfig) -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config,
            state: Arc::new(ClarotyState::with_admin_token(admin_token.clone())),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
        }
    }

    /// Create a new clone bound to a specific `instance_org_id`.
    ///
    /// Used by tests that need strict per-org X-Org-Id header validation
    /// (W3-FIX-SEC-001 / SEC-001). The clone's state enforces the
    /// org guard on all routes that support it.
    pub fn with_org(instance_org_id: prism_core::OrgId) -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config: StubConfig::default(),
            state: Arc::new(ClarotyState::with_admin_token_and_org(
                admin_token.clone(),
                instance_org_id,
            )),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
        }
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Read endpoints (POST-body filtering)
            .route("/api/v1/devices", post(devices::list_devices))
            .route("/api/v1/alerts", post(alerts::list_alerts))
            .route("/api/v1/audit_log/get", post(audit_log::list_audit_logs))
            .route(
                "/api/v1/device_alert_relations",
                post(device_alert_relations::list_device_alert_relations),
            )
            .route(
                "/api/v1/alerts/:alert_id/devices",
                post(alerts::list_alerted_devices),
            )
            .route(
                "/api/v1/vulnerabilities",
                post(vulnerabilities::list_vulnerabilities),
            )
            .route(
                "/api/v1/vulnerabilities/:vuln_id/devices",
                post(vulnerabilities::list_vulnerability_devices),
            )
            // Write endpoints (stateful tag store)
            // Route registered WITHOUT trailing slash so NormalizePathLayer::trim_trailing_slash()
            // can strip inbound `/tags/` → `/tags` and match. the AC-005 tags regression guard (`test_BC_2_16_013_tags_route_with_slash_still_works`) verifies this.
            .route("/api/v1/devices/:device_id/tags", post(tags::add_tag))
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(tags::remove_tag),
            )
            // DTU control endpoints
            .route("/dtu/configure", post(devices::dtu_configure))
            .route("/dtu/reset", post(devices::dtu_reset))
            .route("/dtu/reset_for/:org_id", post(devices::dtu_reset_for))
            .route("/dtu/health", get(devices::dtu_health))
            .with_state(Arc::clone(&self.state))
    }
}

impl Default for ClarotyClone {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralClone for ClarotyClone {
    async fn start_on(
        &mut self,
        bind: std::net::SocketAddr,
        shutdown: Option<broadcast::Receiver<()>>,
        #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<std::net::SocketAddr> {
        let router = self.build_router();
        // BC-2.16.013: wrap the OUTER service with NormalizePathLayer::trim_trailing_slash()
        // so inbound trailing-slash requests (e.g. /api/v1/alerts/) are stripped to the
        // registered route path (/api/v1/alerts) before routing. Applied at the outer service
        // level — NOT via Router::layer() which no-ops in axum 0.7 because the Router matches
        // the path before inner layers run (axum#2377). Applied at BOTH serve sites (TLS and
        // plain HTTP) to ensure consistent behavior regardless of the serve path.
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            // Fully-qualified axum::ServiceExt to satisfy type inference (axum#2377).
            let make_svc = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);
            let server_task = tokio::spawn(async move {
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(make_svc)
                    .await
                    .expect("ClarotyClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("ClarotyClone TLS server failed to start"))?;
            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            // Retain handle so stop() can call graceful_shutdown() (MEDIUM-001 fix).
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path.
        let _ = tls;
        let listener = TcpListener::bind(bind).await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);
        self.tls_active = false;

        // Fully-qualified axum::ServiceExt to satisfy type inference (axum#2377).
        // Created here (outside the spawn) so both branches of the if/else can own it.
        let make_svc = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);

        if let Some(mut rx) = shutdown {
            // Harness-provided shutdown: keep the external broadcast receiver path unchanged.
            let handle = tokio::spawn(async move {
                let server = axum::serve(listener, make_svc);
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("ClarotyClone server error");
            });
            self.server_handle = Some(handle);
        } else {
            // Internal shutdown path (S-PERF-GATE-005): wire a broadcast channel so
            // stop() can signal the server task directly, completing in < 10ms for idle
            // clones instead of timing out on the 5s hard-abort fallback.
            let (handle, tx) = prism_dtu_common::server::spawn_with_internal_shutdown(
                listener,
                make_svc,
                "ClarotyClone server error",
            );
            self.server_handle = Some(handle);
            self.internal_shutdown_tx = Some(tx);
        }

        Ok(addr)
    }

    /// Stop the server: signal internal shutdown then await with a short fallback.
    ///
    /// # S-PERF-GATE-005 — graceful shutdown wiring
    ///
    /// All three paths (HTTP-internal, HTTP-harness, TLS) now resolve promptly:
    ///
    /// - **HTTP internal** (`shutdown=None` on start): fires `internal_shutdown_tx`,
    ///   the server task's `with_graceful_shutdown` future resolves, the select!
    ///   handle-done arm fires in < 10ms.  Fallback hard-abort at 250ms (safety only).
    /// - **HTTP harness** (`shutdown=Some(rx)` on start): the harness already sent
    ///   the broadcast before calling `stop()`; we wait up to 250ms before abort.
    /// - **TLS path**: fires `axum_server::Handle::graceful_shutdown(250ms)`, then
    ///   waits up to 250ms before abort.
    async fn stop(&mut self) -> anyhow::Result<()> {
        // TLS path: signal graceful shutdown via the retained axum_server::Handle.
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_millis(250)));
        }

        // All paths: fire internal sender (if present) and await with short fallback.
        if let Some(handle) = self.server_handle.take() {
            prism_dtu_common::server::graceful_stop(
                self.internal_shutdown_tx.take(),
                handle,
                std::time::Duration::from_millis(250),
            )
            .await;
        }

        self.tls_active = false;
        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        // Apply latency if specified.
        if let Some(ms) = config.get("latency_ms").and_then(|v| v.as_u64()) {
            self.state.apply_latency(ms);
        }
        // Apply failure mode if any recognized key is present.
        use prism_dtu_common::FailureMode;
        let mode = if let Some(n) = config.get("unprocessable_at").and_then(|v| v.as_u64()) {
            Some(FailureMode::Unprocessable {
                at_request_n: n as u32,
            })
        } else if let Some(n) = config.get("internal_error_at").and_then(|v| v.as_u64()) {
            Some(FailureMode::InternalError {
                at_request_n: n as u32,
            })
        } else if let Some(n) = config.get("rate_limit_after").and_then(|v| v.as_u64()) {
            let retry = config
                .get("retry_after_secs")
                .and_then(|v| v.as_u64())
                .unwrap_or(60);
            Some(FailureMode::RateLimit {
                after_n_requests: n as u32,
                retry_after_secs: retry as u32,
            })
        } else if config.get("auth_mode").and_then(|v| v.as_str()) == Some("reject") {
            Some(FailureMode::AuthReject)
        } else {
            None
        };
        if let Some(m) = mode {
            self.state.apply_config(m);
        }
        Ok(())
    }

    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers must call start() before bound_addr(); panic documents the programming error.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("ClarotyClone::start() must be called before bound_addr()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
