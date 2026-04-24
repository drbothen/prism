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
}

impl CrowdstrikeClone {
    /// Create a new clone with default `StubConfig` and empty state stores.
    pub fn new() -> Self {
        Self {
            config: StubConfig::default(),
            state: Arc::new(CrowdstrikeState::new()),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
        }
    }

    /// Create with explicit config.
    pub fn with_config(config: StubConfig) -> Self {
        Self {
            config,
            state: Arc::new(CrowdstrikeState::new()),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
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

    /// Forcibly abort the server task.
    async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
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
}
