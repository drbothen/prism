//! `SlackClone` — `BehavioralClone` implementation for the Slack Incoming Webhook DTU.
//!
//! Lifecycle:
//! 1. `SlackClone::new()` — allocates state with random admin token.
//! 2. `start()` — binds an ephemeral TCP port (`127.0.0.1:0`), builds the axum router, spawns server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears payload capture store and request counter; failure mode reset to None.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS. When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, DtuMode, FailureLayer};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::routes::{
    dtu::{get_health, get_received_payloads, post_configure, post_reset},
    webhook::post_webhook,
};
use crate::state::SlackState;

/// Deployment-time DTU operating mode for the Slack clone (BC-3.2.005 / ADR-007).
///
/// The Slack DTU is a shared-infra service: one instance serves all client orgs.
/// `OrgId` is embedded in each captured payload body at ingress (ADR-007 §2.6 Step 3).
/// The state store (`received_payloads`) is NOT re-keyed by OrgId (ADR-008 §1.2).
///
/// This constant is `DtuMode::Shared` for the lifetime of the process — it MUST NOT
/// be changed at runtime (BC-3.2.005 invariant 4 / AC-007).
pub const DTU_DEFAULT_MODE: DtuMode = DtuMode::Shared;

/// L2-fidelity behavioral clone of the Slack Incoming Webhook API.
pub struct SlackClone {
    state: Arc<SlackState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS.
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
}

impl SlackClone {
    /// Create a new `SlackClone` with a fresh in-memory state.
    pub fn new() -> anyhow::Result<Self> {
        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(SlackState::with_admin_token(admin_token.clone()));
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

    /// Return the base URL for the bound server (e.g. `"http://127.0.0.1:12345"`).
    ///
    /// Delegates to the trait's `base_url()` which checks `is_tls_active()`.
    ///
    /// Panics if `start()` has not been called.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    /// Return all Block Kit payloads received since the last reset.
    ///
    /// Direct in-process test API for assertions without HTTP round-trip.
    /// Per story Task 2 (`received_payloads(&self) -> Vec<serde_json::Value>`).
    pub fn received_payloads(&self) -> Vec<serde_json::Value> {
        self.state.all_payloads()
    }

    fn build_router(&self) -> Router {
        let failure_layer = FailureLayer::shared(Arc::clone(&self.state.failure_mode));

        // Vendor API routes — wrapped with FailureLayerShared so failure injection
        // applies only to the real API surface. DTU-internal routes MUST remain
        // reachable even when a failure mode is active.
        // Real Slack webhook URLs are multi-segment: /services/T.../B.../xxx
        // Use a wildcard route to accept any path under /services/.
        let vendor_router = Router::new()
            .route("/services/*token", post(post_webhook))
            .layer(failure_layer);

        // DTU-internal routes — NOT wrapped by FailureLayer; always reachable.
        Router::new()
            .merge(vendor_router)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .route("/dtu/received-payloads", get(get_received_payloads))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for SlackClone {
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
                // SAFETY: server crash inside the task should propagate as a fatal error.
                #[allow(clippy::expect_used)]
                result.expect("SlackClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("SlackClone TLS server failed to start"))?;
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

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                let result = server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await;
                // SAFETY: server task panic is fatal; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                result.expect("Slack DTU server error");
            } else {
                let result = server.await;
                // SAFETY: same as above — server task panic must surface immediately.
                #[allow(clippy::expect_used)]
                result.expect("Slack DTU server error");
            }
        });
        self.server_handle = Some(handle);

        Ok(addr)
    }

    /// Stop the server: graceful drain then hard-abort fallback (matching sibling clones).
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
    ///
    /// Per AC-6: clears `received_payloads`, resets `request_count` to 0, failure mode → None.
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
            .expect("SlackClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
