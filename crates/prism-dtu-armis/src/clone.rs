//! `ArmisClone` — `BehavioralClone` implementation for the Armis Centrix DTU.
//!
//! Lifecycle:
//! 1. `ArmisClone::new()` — allocates state; loads fixtures into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears tag store + AQL log; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).
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
use prism_dtu_common::{BehavioralClone, FailureLayer};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::{
    alerts::get_alerts,
    devices::{get_device_activity, get_device_risk, get_or_post_devices, post_devices},
    dtu::{get_aql_log, get_health, post_configure, post_reset},
    tags::{delete_device_tag, post_device_tag},
};
use crate::state::ArmisState;
use crate::types::{ActivityRecord, AlertRecord, DeviceRecord};

/// L2-fidelity behavioral clone of the Armis Centrix API.
pub struct ArmisClone {
    state: Arc<ArmisState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
}

impl ArmisClone {
    /// Create a new `ArmisClone`. Loads all fixtures from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture_as` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let devices: Vec<DeviceRecord> = prism_dtu_common::load_fixture_as(crate_dir, "devices")?;
        let activity: Vec<ActivityRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "device-activity")?;
        let alerts: Vec<AlertRecord> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;

        let state = Arc::new(ArmisState::new(devices, activity, alerts));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
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

    fn build_router(&self) -> Router {
        let failure_layer = FailureLayer::shared(Arc::clone(&self.state.failure_mode));

        // Vendor API routes — wrapped with FailureLayerShared so failure injection
        // applies only to the real API surface. DTU-internal routes MUST remain
        // reachable even when a failure mode is active (configure/reset must always work).
        let vendor_router = Router::new()
            .route("/api/v1/devices", get(get_or_post_devices))
            .route("/api/v1/devices", post(post_devices))
            .route(
                "/api/v1/devices/:device_id/activity",
                get(get_device_activity),
            )
            .route("/api/v1/devices/:device_id/risk", get(get_device_risk))
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/devices/:device_id/tags/", post(post_device_tag))
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(delete_device_tag),
            )
            .layer(failure_layer);

        // DTU-internal routes — NOT wrapped by FailureLayer; always reachable.
        Router::new()
            .merge(vendor_router)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            // Armis-specific introspection: AQL capture log
            .route("/dtu/aql-log", get(get_aql_log))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for ArmisClone {
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
                    .expect("ArmisClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("ArmisClone TLS server failed to start"))?;
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
                    .expect("Armis DTU server error");
            } else {
                server.await.expect("Armis DTU server error");
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
    ///
    /// Per ADR-002 §4: delegates to `self.state.reset()` with no additional logic.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime.
    ///
    /// Per ADR-002 §4: delegates to `self.state.apply_config(&config)` — no inline
    /// JSON field inspection in `clone.rs`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("ArmisClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }
}
