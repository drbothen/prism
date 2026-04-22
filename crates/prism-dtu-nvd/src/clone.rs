//! `NvdClone` — `BehavioralClone` implementation for the NVD API 2.0 DTU.
//!
//! Lifecycle:
//! 1. `NvdClone::new()` — allocates state; loads `fixtures/cves.json` into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears counters + rate-limit buckets; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use prism_dtu_common::BehavioralClone;

use crate::state::NvdState;

/// L2-fidelity behavioral clone of the NVD/NIST CVE API 2.0.
pub struct NvdClone {
    state: Arc<NvdState>,
    bound_addr: Option<SocketAddr>,
}

impl NvdClone {
    /// Create a new `NvdClone`. Loads `fixtures/cves.json` from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        todo!()
    }

    /// Return the request count for a specific CVE ID.
    ///
    /// Used by integration tests to assert Prism's cache-hit behavior:
    /// after two identical CVE requests, `request_count_for("CVE-2024-0001")` should
    /// return `1` (second request hit Prism's cache, never reached the DTU).
    pub fn request_count_for(&self, cve_id: &str) -> u32 {
        todo!()
    }
}

#[async_trait]
impl BehavioralClone for NvdClone {
    /// Bind to a local ephemeral port and start the axum HTTP server.
    async fn start(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    /// Reset all captured state to initial values.
    async fn reset(&self) -> anyhow::Result<()> {
        todo!()
    }

    /// Reconfigure the stub at runtime (auth_mode, failure injection, etc.).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        todo!()
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        todo!()
    }
}
