//! `PagerDutyClone` — `BehavioralClone` implementation for the PagerDuty Events API v2 DTU.
//!
//! Lifecycle:
//! 1. `PagerDutyClone::new()` — allocates state with fresh incident registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears incident registry, auth mode, and failure mode.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS. When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, DtuMode, FailureLayer};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    routes::{
        dtu::{get_health, get_incidents, post_configure, post_reset},
        enqueue::post_enqueue,
    },
    state::PagerDutyState,
};

/// Deployment-time DTU operating mode for the PagerDuty clone (BC-3.2.005 / ADR-007).
///
/// The PagerDuty DTU is a shared-infra service: one instance serves all client orgs.
/// `OrgId` is embedded in each captured `IncidentRecord.org_id` field at ingress
/// (ADR-007 §2.6 Step 3 / BC-3.2.004 postcondition 1).
/// The `incident_registry` is NOT re-keyed by OrgId (ADR-008 §1.2).
///
/// The authoritative mode is registered in the prism-core mode registry slice
/// under the `"pagerduty"` type name (ADR-007 §2.3). This crate-local constant mirrors
/// it for compile-time assertion in tests only — see `org_tagging.rs`.
///
/// Per ADR-007 §2.3: mode classification MUST live exclusively in `prism-core`.
pub const PAGERDUTY_DTU_MODE: DtuMode = DtuMode::Shared;

/// L3-fidelity behavioral clone of the PagerDuty Events API v2.
pub struct PagerDutyClone {
    state: Arc<PagerDutyState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
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

impl PagerDutyClone {
    /// Create a new `PagerDutyClone` with fresh in-memory state.
    pub fn new() -> anyhow::Result<Self> {
        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(PagerDutyState::with_admin_token(admin_token.clone()));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
        })
    }

    /// Return the base URL for the bound server (e.g. `"http://127.0.0.1:12345"`).
    ///
    /// Delegates to the trait's `base_url()` which checks `is_tls_active()`.
    ///
    /// Panics if `start()` has not been called.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    /// Return a snapshot of all incidents in the registry (test API convenience method).
    pub fn incidents(&self) -> Vec<crate::state::IncidentRecord> {
        self.state.incidents_snapshot()
    }

    fn build_router(&self) -> Router {
        let failure_layer = FailureLayer::shared(Arc::clone(&self.state.failure_mode));

        // Vendor API routes — wrapped with FailureLayerShared so failure injection
        // applies only to the real API surface. DTU-internal routes MUST remain
        // reachable even when a failure mode is active (configure/reset must always work).
        let vendor_router = Router::new()
            .route("/v2/enqueue", post(post_enqueue))
            .layer(failure_layer);

        // DTU-internal routes — NOT wrapped by FailureLayer; always reachable.
        Router::new()
            .merge(vendor_router)
            .route("/dtu/incidents", get(get_incidents))
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for PagerDutyClone {
    /// Start with an explicit bind address, optional graceful-shutdown receiver, and
    /// optional TLS configuration.
    async fn start_on(
        &mut self,
        bind: SocketAddr,
        shutdown: Option<broadcast::Receiver<()>>,
        #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<SocketAddr> {
        let router = self.build_router();

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let server_task = tokio::spawn(async move {
                let result = axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await;
                // SAFETY: server crash inside the task should propagate as a fatal error; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                result.expect("PagerDutyClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("PagerDutyClone TLS server failed to start"))?;
            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path.
        let _ = tls;
        let listener = tokio::net::TcpListener::bind(bind).await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);
        self.tls_active = false;

        if let Some(mut rx) = shutdown {
            // Harness-provided shutdown: keep the external broadcast receiver path unchanged.
            let handle = tokio::spawn(async move {
                let server = axum::serve(listener, router);
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task panic is fatal; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("PagerDuty DTU server error");
            });
            self.server_handle = Some(handle);
        } else {
            // Internal shutdown path (S-PERF-GATE-005): wire a broadcast channel so
            // stop() can signal the server task directly, completing in < 10ms for idle
            // clones instead of timing out on the 5s hard-abort fallback.
            let (handle, tx) = prism_dtu_common::server::spawn_with_internal_shutdown(
                listener,
                router,
                "PagerDuty DTU server error",
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

    /// Reset all captured state to initial values.
    ///
    /// Per ADR-002 §4: delegates to `self.state.reset()` with no additional logic.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime.
    ///
    /// Per ADR-002 §4: delegates to `self.state.apply_config(&config)`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers are required to call start() before bound_addr(); the expect message documents the contract.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("PagerDutyClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
