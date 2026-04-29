//! `JiraClone` — `BehavioralClone` implementation for the Jira Cloud REST API v3 DTU.
//!
//! Lifecycle:
//! 1. `JiraClone::new()` — allocates state; no fixtures to load (state is built dynamically).
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears issue registry, resets next_issue_num to 1000.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).

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
    comments::add_comment,
    dtu::{get_dtu_issues, get_health, post_configure, post_reset},
    issues::{create_issue, get_issue},
    transitions::{execute_transition, list_transitions},
};
use crate::state::JiraState;

/// Deployment-time DTU operating mode for the Jira clone (BC-3.2.005 / ADR-007).
///
/// The Jira DTU is a shared-infra service: one instance serves all client orgs.
/// `OrgId` is embedded in each captured `IssueRecord.org_id` field at ingress
/// (ADR-007 §2.6 Step 3). The `issue_registry` is NOT re-keyed by OrgId (ADR-008 §1.2).
///
/// The authoritative mode is registered in the prism-core mode registry slice
/// under the `"jira"` type name (ADR-007 §2.3). This crate-local constant mirrors
/// it for compile-time assertion in tests only — see `org_tagging.rs`.
///
/// Per ADR-007 §2.3: mode classification MUST live exclusively in `prism-core`.
pub const JIRA_DTU_MODE: DtuMode = DtuMode::Shared;

/// L3-fidelity behavioral clone of the Jira Cloud REST API v3.
pub struct JiraClone {
    state: Arc<JiraState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS.
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers.
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure`.
    admin_token: String,
}

impl JiraClone {
    /// Create a new `JiraClone` with an empty issue registry.
    pub fn new() -> anyhow::Result<Self> {
        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(JiraState::with_admin_token(admin_token.clone()));
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
    /// Panics if `start()` has not been called.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    /// Return a reference to the shared `JiraState` for direct in-process test assertions.
    ///
    /// Enables test code to inspect `IssueRecord.org_id` without an HTTP round-trip
    /// (e.g. `clone.state().get_issue(key)` in S-3.2.07 org-tagging tests).
    pub fn state(&self) -> &JiraState {
        &self.state
    }

    fn build_router(&self) -> Router {
        let failure_layer = FailureLayer::shared(Arc::clone(&self.state.failure_mode));

        // Vendor API routes — wrapped with FailureLayer so failure injection
        // applies only to the real API surface. DTU-internal routes must remain
        // reachable even when a failure mode is active.
        let vendor_router = Router::new()
            .route("/rest/api/3/issue", post(create_issue))
            .route("/rest/api/3/issue/:issue_key", get(get_issue))
            .route("/rest/api/3/issue/:issue_key/comment", post(add_comment))
            .route(
                "/rest/api/3/issue/:issue_key/transitions",
                get(list_transitions).post(execute_transition),
            )
            .layer(failure_layer);

        // DTU-internal routes — NOT wrapped by FailureLayer; always reachable.
        Router::new()
            .merge(vendor_router)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .route("/dtu/issues", get(get_dtu_issues))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for JiraClone {
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
                result.expect("JiraClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("JiraClone TLS server failed to start"))?;
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
                result.expect("Jira DTU server error");
            } else {
                let result = server.await;
                // SAFETY: same as above — server task panic must surface immediately.
                #[allow(clippy::expect_used)]
                result.expect("Jira DTU server error");
            }
        });
        self.server_handle = Some(handle);

        Ok(addr)
    }

    /// Stop the server: graceful drain then hard-abort fallback.
    async fn stop(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        }

        if let Some(mut handle) = self.server_handle.take() {
            tokio::select! {
                _ = &mut handle => {
                    // Server task completed within the drain window — clean shutdown.
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    handle.abort();
                }
            }
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
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is actually bound to.
    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers are required to call start() before bound_addr().
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("JiraClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
