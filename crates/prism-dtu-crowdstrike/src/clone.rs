//! `CrowdstrikeClone` — implements [`BehavioralClone`] for the CrowdStrike Falcon API DTU.
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
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::build_router;
use crate::state::CrowdstrikeState;

/// L4-adversarial behavioral clone of the CrowdStrike Falcon API.
///
/// Maintains stateful write stores (containment, detection status) and a session-scoped
/// ID registry for two-step pagination. Supports configurable failure injection via the
/// shared `FailureLayer` from `prism-dtu-common`.
///
/// Binds to `127.0.0.1:0` (ephemeral port) on `start()`.
pub struct CrowdstrikeClone {
    pub config: StubConfig,
    pub state: Arc<CrowdstrikeState>,
    pub server_handle: Option<JoinHandle<()>>,
    pub bound_addr: Option<SocketAddr>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers.
    /// Stored so `stop()` can call `handle.graceful_shutdown()` rather than
    /// relying on the broadcast signal (which is not wired to axum_server).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
}

impl CrowdstrikeClone {
    /// Create a new clone with default `StubConfig` and empty state stores.
    pub fn new() -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config: StubConfig::default(),
            state: Arc::new(CrowdstrikeState::with_admin_token(admin_token.clone())),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        }
    }

    /// Create with explicit config.
    pub fn with_config(config: StubConfig) -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config,
            state: Arc::new(CrowdstrikeState::with_admin_token(admin_token.clone())),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        }
    }
}

impl Default for CrowdstrikeClone {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralClone for CrowdstrikeClone {
    /// Start with an explicit bind address, optional graceful-shutdown receiver, and
    /// optional TLS configuration.
    ///
    /// Returns the bound `SocketAddr`. Wires the shutdown receiver into
    /// `axum::serve(...).with_graceful_shutdown(...)` for graceful drain.
    ///
    /// When `tls` is `Some`, binds via `axum_server::bind_rustls` (HTTPS).
    /// When `None`, uses plain `axum::serve` (HTTP).
    async fn start_on(
        &mut self,
        bind: SocketAddr,
        shutdown: Option<broadcast::Receiver<()>>,
        #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<SocketAddr> {
        // Propagate seed from StubConfig into RuntimeConfig so route handlers see it.
        {
            let mut rc = self
                .state
                .runtime_config
                .lock()
                .expect("runtime_config poisoned");
            rc.seed = self.config.seed;
        }

        let router = build_router(
            Arc::clone(&self.state),
            self.config.failure_mode.clone(),
            self.config.latency_ms,
        );

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            // TLS path: bind via axum_server::bind_rustls.
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let server_task = tokio::spawn(async move {
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await
                    .expect("CrowdstrikeClone TLS server crashed");
            });

            // Wait for the server to report its bound address.
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("CrowdstrikeClone TLS server failed to start"))?;

            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            // Retain handle so stop() can call graceful_shutdown() (MEDIUM-001 fix).
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path (also the no-tls feature path).
        let _ = tls; // consume no-tls Option<()> without warning
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .map_err(|e| anyhow::anyhow!("failed to bind listener on {bind}: {e}"))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("failed to get local addr: {e}"))?;

        self.bound_addr = Some(addr);
        self.tls_active = false;

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await
                    .expect("CrowdstrikeClone server crashed");
            } else {
                server.await.expect("CrowdstrikeClone server crashed");
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

    /// Reset all captured state: clears containment store, detection status store,
    /// and session registry. Does NOT change `RuntimeConfig` (auth_mode, seed, etc.).
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime.
    ///
    /// Accepts JSON such as `{"auth_mode": "reject"}`. Delegates to
    /// `CrowdstrikeState::apply_config`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("CrowdstrikeClone::bound_addr called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
