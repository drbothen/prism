//! `NvdClone` — `BehavioralClone` implementation for the NVD API 2.0 DTU.
//!
//! Lifecycle:
//! 1. `NvdClone::new()` — allocates state; loads `fixtures/cves.json` into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears counters + rate-limit buckets; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::BehavioralClone;

use crate::routes::{
    cves::get_cves,
    dtu::{get_request_count, post_configure, post_reset},
};
use crate::state::NvdState;
use crate::types::CveRecord;

/// L2-fidelity behavioral clone of the NVD/NIST CVE API 2.0.
pub struct NvdClone {
    state: Arc<NvdState>,
    bound_addr: Option<SocketAddr>,
}

impl NvdClone {
    /// Create a new `NvdClone`. Loads `fixtures/cves.json` from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture_as` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let records: Vec<CveRecord> = prism_dtu_common::load_fixture_as(crate_dir, "cves");

        let registry: HashMap<String, CveRecord> = records
            .into_iter()
            .map(|r| (r.id.to_uppercase(), r))
            .collect();

        let state = Arc::new(NvdState::new(registry));
        Ok(Self {
            state,
            bound_addr: None,
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
            .route("/dtu/request-count/:cve_id", get(get_request_count))
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for NvdClone {
    /// Bind to a local ephemeral port and start the axum HTTP server.
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);

        let router = self.build_router();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("NVD DTU server error");
        });

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
}
