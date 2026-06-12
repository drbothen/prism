//! `ClarotyClone` — implements `BehavioralClone` for the Claroty xDome DTU.
//!
//! Binds to `127.0.0.1:0` (ephemeral port) on `start()`, spawns an axum
//! server with `LatencyLayer` + `FailureLayer`, and serves 8 application
//! endpoints (6 read via POST-body filtering, 2 write via stateful tag store)
//! plus the DTU control endpoints.  The 6 read endpoints are: `list_devices`,
//! `list_alerts`, `list_alerted_devices`, `list_vulnerabilities`,
//! `list_vulnerability_devices`, and `list_audit_logs` (added by
//! S-DEMO-CLAROTY-AUDIT-DTU-001).  See `dtu-assessment.md §3.2` for the
//! original 7-endpoint scope matrix; `audit_log` is a Wave-5 fidelity
//! addition beyond that baseline.
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
    routes::{alerts, audit_log, devices, tags, vulnerabilities},
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
        }
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
        }
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Read endpoints (POST-body filtering)
            .route("/api/v1/devices", post(devices::list_devices))
            .route("/api/v1/alerts", post(alerts::list_alerts))
            .route("/api/v1/audit_log/get", post(audit_log::list_audit_logs))
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

        let handle = tokio::spawn(async move {
            // Fully-qualified axum::ServiceExt to satisfy type inference (axum#2377).
            let make_svc = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);
            let server = axum::serve(listener, make_svc);
            if let Some(mut rx) = shutdown {
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("ClarotyClone server error");
            } else {
                // SAFETY: same as above.
                #[allow(clippy::expect_used)]
                server.await.expect("ClarotyClone server error");
            }
        });
        self.server_handle = Some(handle);

        Ok(addr)
    }

    /// Stop the server: graceful drain then hard-abort fallback for both TLS and HTTP.
    ///
    /// # TD-WV1-04-FU-001 — shutdown symmetry
    ///
    /// Both TLS and HTTP paths now use the same graceful-drain-then-abort pattern:
    ///
    /// - **TLS path**: signals `axum_server::Handle::graceful_shutdown(5s)` to begin
    ///   draining, then awaits the `JoinHandle` up to 5 s before hard-aborting.
    /// - **HTTP path**: the harness broadcast signal has already been sent before
    ///   `stop()` is called, so axum's `with_graceful_shutdown` future is already
    ///   resolving. We await the `JoinHandle` up to 5 s before hard-aborting —
    ///   matching the TLS drain window instead of the previous immediate abort.
    async fn stop(&mut self) -> anyhow::Result<()> {
        // TLS path: signal graceful shutdown via the retained axum_server::Handle.
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        }

        // Both paths: attempt graceful drain; hard-abort after 5s.
        if let Some(mut handle) = self.server_handle.take() {
            tokio::select! {
                _ = &mut handle => {
                    // Server task completed within the drain window — clean shutdown.
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    // Drain window expired — hard-abort the server task.
                    handle.abort();
                }
            }
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
