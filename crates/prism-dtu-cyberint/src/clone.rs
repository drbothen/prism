//! `CyberintClone` — `BehavioralClone` implementation for the Cyberint API DTU.
//!
//! Lifecycle:
//! 1. `CyberintClone::new()` — allocates state; loads fixtures from crate root.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — delegates to `state.reset()`.
//! 5. `configure()` — delegates to `state.apply_config()`.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, patch, post},
    Router,
};
use prism_dtu_common::BehavioralClone;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::{
    alerts::{get_alert_by_id, get_alerts, patch_alert_status, post_close_alert},
    auth::post_login,
    dtu::{get_health, post_configure, post_reset},
    threats::get_threat_intel,
};
use crate::state::CyberintState;
use crate::types::Alert;

/// L2-fidelity behavioral clone of the Cyberint API.
pub struct CyberintClone {
    state: Arc<CyberintState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
}

impl CyberintClone {
    /// Create a new `CyberintClone`. Loads fixtures from the crate root.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<Alert> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(CyberintState::with_admin_token(
            alerts,
            alerts_page2,
            threats,
            admin_token.clone(),
        ));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        })
    }

    /// Return the base URL for this clone (e.g. `http://127.0.0.1:PORT`).
    ///
    /// Delegates to the trait's `base_url()` which checks `is_tls_active()`.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Auth
            .route("/login", post(post_login))
            // Alerts
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/alerts", post(get_alerts))
            .route("/api/v1/alerts/:alert_id", get(get_alert_by_id))
            .route("/api/v1/alerts/:alert_id/status", patch(patch_alert_status))
            .route("/api/v1/alerts/:alert_id/close", post(post_close_alert))
            // Threat intel
            .route("/api/v1/threat-intel", get(get_threat_intel))
            // DTU internal
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for CyberintClone {
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
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await
                    .expect("CyberintClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("CyberintClone TLS server failed to start"))?;
            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            // Retain handle so stop() can call graceful_shutdown() (MEDIUM-001 fix).
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path.
        let _ = tls;
        let listener = tokio::net::TcpListener::bind(bind).await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);
        self.tls_active = false;

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("Cyberint DTU server error");
            } else {
                // SAFETY: same as above.
                #[allow(clippy::expect_used)]
                server.await.expect("Cyberint DTU server error");
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

    /// Reset all captured state to initial values.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime (auth_mode, rate_limit_after, etc.).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers must call start() before bound_addr(); panic documents the programming error.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("CyberintClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
