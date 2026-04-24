//! `ThreatIntelClone` — implements `BehavioralClone` for the Threat Intel Aggregator DTU.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

#![allow(clippy::expect_used)]
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::dtu::{dtu_health, dtu_reset};
use crate::routes::lookup::{configure, domain_lookup, hash_lookup, ip_lookup};
use crate::state::ThreatIntelState;

/// L2-stateful behavioral clone of the Threat Intel Aggregator infusion plugin API.
///
/// Maintains a fixture registry and per-request rate-limit counter.
/// Binds to `127.0.0.1:0` (ephemeral port) on `start()`.
pub struct ThreatIntelClone {
    pub config: StubConfig,
    pub state: Arc<ThreatIntelState>,
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

impl ThreatIntelClone {
    /// Create a new clone with default `StubConfig` and default fixture registry.
    pub fn new() -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config: StubConfig::default(),
            state: Arc::new(ThreatIntelState::with_admin_token(admin_token.clone())),
            bound_addr: None,
            server_handle: None,
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
            state: Arc::new(ThreatIntelState::with_admin_token(admin_token.clone())),
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
            .route("/v3/ip/:ip", get(ip_lookup))
            .route("/v3/domain/:domain", get(domain_lookup))
            .route("/v3/hash/:hash", get(hash_lookup))
            .route("/dtu/health", get(dtu_health))
            .route("/dtu/configure", post(configure))
            .route("/dtu/reset", post(dtu_reset))
            .with_state(Arc::clone(&self.state))
    }
}

impl Default for ThreatIntelClone {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralClone for ThreatIntelClone {
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
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await
                    .expect("ThreatIntelClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("ThreatIntelClone TLS server failed to start"))?;
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
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await
                    .expect("ThreatIntelClone server error");
            } else {
                server.await.expect("ThreatIntelClone server error");
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
        // Delegate to the same logic as the HTTP configure endpoint.
        if let Some(n) = config.get("rate_limit_after").and_then(|v| v.as_u64()) {
            let mut threshold = self
                .state
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned");
            *threshold = Some(n as u32);
        }

        let lookup_value = config
            .get("ip")
            .or_else(|| config.get("hash"))
            .or_else(|| config.get("domain"))
            .and_then(|v| v.as_str());

        if let (Some(value), Some(fixture_str)) =
            (lookup_value, config.get("fixture").and_then(|v| v.as_str()))
        {
            use crate::types::FixtureKey;
            let fixture_key = match fixture_str {
                "malicious" => FixtureKey::Malicious,
                "benign" => FixtureKey::Benign,
                "unknown" => FixtureKey::Unknown,
                other => anyhow::bail!("unknown fixture key: {}", other),
            };
            let mut registry = self
                .state
                .fixture_registry
                .lock()
                .expect("fixture_registry poisoned");
            registry.insert(value.to_string(), fixture_key);
        }

        Ok(())
    }

    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("ThreatIntelClone::start() must be called before bound_addr()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
