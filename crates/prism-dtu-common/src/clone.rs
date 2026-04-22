//! `BehavioralClone` — the core trait all DTU behavioral clones must implement.

use async_trait::async_trait;
use std::net::SocketAddr;

/// Trait implemented by every per-surface DTU behavioral clone.
///
/// Each clone starts a local stub server, accepts reconfiguration at runtime,
/// and exposes its bound address for client construction.
#[async_trait]
pub trait BehavioralClone: Send + Sync + 'static {
    /// Start the stub server and bind to a local port.
    async fn start(&mut self) -> anyhow::Result<()>;

    /// Reset all captured state (requests, counters, injected errors) to initial values.
    async fn reset(&self) -> anyhow::Result<()>;

    /// Reconfigure the stub at runtime (e.g. change failure mode, latency).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>;

    /// Return the `SocketAddr` the stub is actually bound to.
    fn bound_addr(&self) -> SocketAddr;

    /// Convenience: HTTP base URL derived from `bound_addr`.
    fn base_url(&self) -> String {
        format!("http://{}", self.bound_addr())
    }
}
