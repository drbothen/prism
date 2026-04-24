//! `ClarotyClone` — implements `BehavioralClone` for the Claroty xDome DTU.
//!
//! Binds to `127.0.0.1:0` (ephemeral port) on `start()`, spawns an axum
//! server with `LatencyLayer` + `FailureLayer`, and serves all 7 in-scope
//! Claroty xDome endpoints plus the DTU control endpoints.
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
    routing::{delete, get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::{alerts, devices, tags, vulnerabilities};
use crate::state::ClarotyState;

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
}

impl ClarotyClone {
    /// Create a new clone with default `StubConfig` and empty tag store.
    pub fn new() -> Self {
        Self {
            config: StubConfig::default(),
            state: Arc::new(ClarotyState::new()),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
        }
    }

    /// Create with explicit configuration.
    pub fn with_config(config: StubConfig) -> Self {
        Self {
            config,
            state: Arc::new(ClarotyState::new()),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
        }
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Read endpoints (POST-body filtering)
            .route("/api/v1/devices", post(devices::list_devices))
            .route("/api/v1/alerts", post(alerts::list_alerts))
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
            .route("/api/v1/devices/:device_id/tags/", post(tags::add_tag))
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(tags::remove_tag),
            )
            // DTU control endpoints
            .route("/dtu/configure", post(devices::dtu_configure))
            .route("/dtu/reset", post(devices::dtu_reset))
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

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let server_task = tokio::spawn(async move {
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
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
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await
                    .expect("ClarotyClone server error");
            } else {
                server.await.expect("ClarotyClone server error");
            }
        });
        self.server_handle = Some(handle);

        Ok(addr)
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        }
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
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
        self.bound_addr
            .expect("ClarotyClone::start() must be called before bound_addr()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }
}
