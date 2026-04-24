//! `NvdClone` — `BehavioralClone` implementation for the NVD API 2.0 DTU.
//!
//! Lifecycle:
//! 1. `NvdClone::new()` — allocates state; loads `fixtures/cves.json` into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears counters + rate-limit buckets; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::BehavioralClone;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::{
    cves::get_cves,
    dtu::{get_health, get_request_count, post_configure, post_reset},
};
use crate::state::NvdState;
use crate::types::CveRecord;

/// L2-fidelity behavioral clone of the NVD/NIST CVE API 2.0.
pub struct NvdClone {
    state: Arc<NvdState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
}

impl NvdClone {
    /// Create a new `NvdClone`. Loads `fixtures/cves.json` from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture_as` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let records: Vec<CveRecord> = prism_dtu_common::load_fixture_as(crate_dir, "cves")?;

        let registry: HashMap<String, CveRecord> = records
            .into_iter()
            .map(|r| (r.id.to_uppercase(), r))
            .collect();

        let state = Arc::new(NvdState::new(registry));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
        })
    }

    /// Return the request count for a specific CVE ID.
    ///
    /// Used by integration tests to assert Prism's cache-hit behavior:
    /// after two identical CVE requests, `request_count_for("CVE-2024-0001")` should
    /// return `1` (second request hit Prism's cache, never reached the DTU).
    pub fn request_count_for(&self, cve_id: &str) -> u32 {
        self.state.request_count_for(cve_id)
    }

    fn build_router(&self) -> Router {
        Router::new()
            .route("/rest/json/cves/2.0", get(get_cves))
            .route("/dtu/health", get(get_health))
            .route("/dtu/request-count/:cve_id", get(get_request_count))
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for NvdClone {
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
                    .expect("NvdClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("NvdClone TLS server failed to start"))?;
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
                server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await
                    .expect("NVD DTU server error");
            } else {
                server.await.expect("NVD DTU server error");
            }
        });
        self.server_handle = Some(handle);

        Ok(addr)
    }

    /// Stop the server: graceful drain for TLS (via axum_server::Handle), abort for HTTP.
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

    /// Reset all captured state to initial values.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime (auth_mode, failure injection, etc.).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("NvdClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }
}
