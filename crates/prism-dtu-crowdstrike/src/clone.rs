//! `CrowdstrikeClone` — implements [`BehavioralClone`] for the CrowdStrike Falcon API DTU.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::task::JoinHandle;

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
    async fn start(&mut self) -> anyhow::Result<()> {
        unimplemented!("CrowdstrikeClone::start — not yet implemented")
    }

    async fn reset(&self) -> anyhow::Result<()> {
        unimplemented!("CrowdstrikeClone::reset — not yet implemented")
    }

    async fn configure(&self, _config: serde_json::Value) -> anyhow::Result<()> {
        unimplemented!("CrowdstrikeClone::configure — not yet implemented")
    }

    fn bound_addr(&self) -> SocketAddr {
        unimplemented!("CrowdstrikeClone::bound_addr — not yet implemented")
    }
}
