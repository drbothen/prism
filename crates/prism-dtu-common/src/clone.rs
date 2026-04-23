//! `BehavioralClone` — the core trait all DTU behavioral clones must implement.

use async_trait::async_trait;
use std::net::SocketAddr;

/// Trait implemented by every per-surface DTU behavioral clone.
///
/// Each clone starts a local stub server, accepts reconfiguration at runtime,
/// and exposes its bound address for client construction.
///
/// # ADR-002 Amendment (S-6.20 Pass 3)
///
/// Two new methods were added to support the demo harness (`prism-dtu-demo-server`):
/// - `start_on`: start with an explicit bind address and optional graceful-shutdown signal.
/// - `stop`: forcibly abort the server task (hard-abort fallback after 5s graceful drain).
///
/// The existing `start()` method now delegates to `start_on()` via a default impl —
/// all existing call sites continue to compile and run without modification.
#[async_trait]
pub trait BehavioralClone: Send + Sync + 'static {
    /// Start the clone with the default bind address (`127.0.0.1:0`) and no shutdown signal.
    ///
    /// Existing clone implementations and integration tests call this method — no changes
    /// required in any of the 6 merged clone crates for backward compatibility.
    ///
    /// # Default implementation
    ///
    /// Delegates to `start_on("127.0.0.1:0".parse().unwrap(), None)`.
    /// NOTE: bind addr comes from start_on param; StubConfig.bind only used by this shim.
    async fn start(&mut self) -> anyhow::Result<()> {
        let addr = "127.0.0.1:0"
            .parse()
            .expect("127.0.0.1:0 is a valid SocketAddr; this is a static compile-time string");
        self.start_on(addr, None).await.map(|_| ())
    }

    /// Start with an explicit bind address and optional graceful-shutdown receiver.
    ///
    /// Returns the bound `SocketAddr`. The demo harness calls this method.
    ///
    /// Implementations MUST wire the shutdown receiver into
    /// `axum::serve(...).with_graceful_shutdown(...)` for graceful drain.
    async fn start_on(
        &mut self,
        _bind: SocketAddr,
        _shutdown: Option<tokio::sync::broadcast::Receiver<()>>,
    ) -> anyhow::Result<SocketAddr> {
        unimplemented!(
            "start_on() not yet implemented for this clone — \
             implement BehavioralClone::start_on in the clone crate"
        )
    }

    /// Forcibly abort the server task via `JoinHandle::abort()`.
    ///
    /// Called by the harness when the 5-second graceful drain timeout elapses,
    /// and during partial-startup cleanup (N-1 clones already started).
    ///
    /// Each clone crate must implement this; the default panics so that
    /// partial-startup cleanup is not silently skipped.
    async fn stop(&mut self) -> anyhow::Result<()> {
        unimplemented!(
            "stop() not yet implemented for this clone — \
             implement BehavioralClone::stop in the clone crate"
        )
    }

    /// Reset all captured state (requests, counters, injected errors) to initial values.
    /// Does NOT stop the HTTP server.
    async fn reset(&self) -> anyhow::Result<()>;

    /// Reconfigure the stub at runtime (e.g. change failure mode, latency).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()>;

    /// Return the `SocketAddr` the stub is actually bound to.
    ///
    /// Panics if called before `start_on()`. The harness MUST only call this
    /// after `start_on()` returns `Ok` — it reads `pair.bound_addr` (the returned
    /// `SocketAddr`) rather than calling `bound_addr()` directly.
    fn bound_addr(&self) -> SocketAddr;

    /// Convenience: HTTP base URL derived from `bound_addr`.
    fn base_url(&self) -> String {
        format!("http://{}", self.bound_addr())
    }
}
