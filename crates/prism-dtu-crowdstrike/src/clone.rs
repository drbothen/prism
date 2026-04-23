//! `CrowdstrikeClone` — implements [`BehavioralClone`] for the CrowdStrike Falcon API DTU.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::net::TcpListener;
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
}

impl CrowdstrikeClone {
    /// Create a new clone with default `StubConfig` and empty state stores.
    pub fn new() -> Self {
        Self {
            config: StubConfig::default(),
            state: Arc::new(CrowdstrikeState::new()),
            server_handle: None,
            bound_addr: None,
        }
    }

    /// Create with explicit config.
    pub fn with_config(config: StubConfig) -> Self {
        Self {
            config,
            state: Arc::new(CrowdstrikeState::new()),
            server_handle: None,
            bound_addr: None,
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
    /// Start the stub server on `127.0.0.1:0` (ephemeral port).
    ///
    /// Stores the bound address in `self.bound_addr` and spawns an axum server
    /// in the background. Initialises `RuntimeConfig::seed` from `StubConfig::seed`.
    async fn start(&mut self) -> anyhow::Result<()> {
        // Propagate seed from StubConfig into RuntimeConfig so route handlers see it.
        {
            let mut rc = self
                .state
                .runtime_config
                .lock()
                .expect("runtime_config poisoned");
            rc.seed = self.config.seed;
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| anyhow::anyhow!("failed to bind listener: {e}"))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("failed to get local addr: {e}"))?;

        self.bound_addr = Some(addr);

        let router = build_router(
            Arc::clone(&self.state),
            self.config.failure_mode.clone(),
            self.config.latency_ms,
        );

        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("DTU server crashed");
        });

        self.server_handle = Some(handle);
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
}
