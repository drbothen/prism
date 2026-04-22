//! `ClarotyClone` — implements `BehavioralClone` for the Claroty xDome DTU.
//!
//! Binds to `127.0.0.1:0` (ephemeral port) on `start()`, spawns an axum
//! server with `LatencyLayer` + `FailureLayer`, and serves all 7 in-scope
//! Claroty xDome endpoints plus the DTU control endpoints.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{delete, get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::routes::{alerts, devices, tags, vulnerabilities};
use crate::state::ClarotyState;

/// L4 (adversarial) behavioral clone of the Claroty xDome API.
///
/// Maintains a stateful device tag store and supports full failure injection
/// via `LatencyLayer` + `FailureLayer` from `prism-dtu-common`.
///
/// Binds to an ephemeral port on `127.0.0.1`; use `base_url()` to construct
/// HTTP client URLs in tests.
pub struct ClarotyClone {
    pub config: StubConfig,
    pub state: Arc<ClarotyState>,
    pub bound_addr: Option<SocketAddr>,
    pub server_handle: Option<JoinHandle<()>>,
}

impl ClarotyClone {
    /// Create a new clone with default `StubConfig` and empty tag store.
    pub fn new() -> Self {
        Self {
            config: StubConfig::default(),
            state: Arc::new(ClarotyState::new()),
            bound_addr: None,
            server_handle: None,
        }
    }

    /// Create with explicit configuration.
    pub fn with_config(config: StubConfig) -> Self {
        Self {
            config,
            state: Arc::new(ClarotyState::new()),
            bound_addr: None,
            server_handle: None,
        }
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Read endpoints (POST-body filtering)
            .route("/api/v1/devices", post(devices::list_devices))
            .route("/api/v1/alerts", post(alerts::list_alerts))
            .route(
                "/api/v1/alerts/:alert_id/devices",
                post(alerts::list_alerted_devices),
            )
            .route(
                "/api/v1/vulnerabilities",
                post(vulnerabilities::list_vulnerabilities),
            )
            .route(
                "/api/v1/vulnerabilities/:vuln_id/devices",
                post(vulnerabilities::list_vulnerability_devices),
            )
            // Write endpoints (stateful tag store)
            .route("/api/v1/devices/:device_id/tags/", post(tags::add_tag))
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(tags::remove_tag),
            )
            // DTU control endpoints
            .route("/dtu/configure", post(devices::dtu_configure))
            .route("/dtu/reset", post(devices::dtu_reset))
            .route("/dtu/health", get(devices::dtu_health))
            .with_state(Arc::clone(&self.state))
    }
}

impl Default for ClarotyClone {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralClone for ClarotyClone {
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);

        let router = self.build_router();

        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("ClarotyClone server error");
        });
        self.server_handle = Some(handle);

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        // Apply latency if specified.
        if let Some(ms) = config.get("latency_ms").and_then(|v| v.as_u64()) {
            self.state.apply_latency(ms);
        }
        // Apply failure mode if any recognized key is present.
        use prism_dtu_common::FailureMode;
        let mode = if let Some(n) = config.get("unprocessable_at").and_then(|v| v.as_u64()) {
            Some(FailureMode::Unprocessable { at_request_n: n as u32 })
        } else if let Some(n) = config.get("internal_error_at").and_then(|v| v.as_u64()) {
            Some(FailureMode::InternalError { at_request_n: n as u32 })
        } else if let Some(n) = config.get("rate_limit_after").and_then(|v| v.as_u64()) {
            let retry = config.get("retry_after_secs").and_then(|v| v.as_u64()).unwrap_or(60);
            Some(FailureMode::RateLimit {
                after_n_requests: n as u32,
                retry_after_secs: retry as u32,
            })
        } else if config.get("auth_mode").and_then(|v| v.as_str()) == Some("reject") {
            Some(FailureMode::AuthReject)
        } else {
            None
        };
        if let Some(m) = mode {
            self.state.apply_config(m);
        }
        Ok(())
    }

    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("ClarotyClone::start() must be called before bound_addr()")
    }
}
