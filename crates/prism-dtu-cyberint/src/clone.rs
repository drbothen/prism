//! `CyberintClone` — `BehavioralClone` implementation for the Cyberint API DTU.
//!
//! Lifecycle:
//! 1. `CyberintClone::new()` — allocates state; loads fixtures from crate root.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — delegates to `state.reset()`.
//! 5. `configure()` — delegates to `state.apply_config()`.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, patch, post},
    Router,
};
use prism_dtu_common::BehavioralClone;

use crate::routes::{
    alerts::{get_alert_by_id, get_alerts, patch_alert_status, post_close_alert},
    auth::post_login,
    dtu::{get_health, post_configure, post_reset},
    threats::get_threat_intel,
};
use crate::state::CyberintState;
use crate::types::Alert;

/// L2-fidelity behavioral clone of the Cyberint API.
pub struct CyberintClone {
    state: Arc<CyberintState>,
    bound_addr: Option<SocketAddr>,
}

impl CyberintClone {
    /// Create a new `CyberintClone`. Loads fixtures from the crate root.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<Alert> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let state = Arc::new(CyberintState::new(alerts, alerts_page2, threats));
        Ok(Self {
            state,
            bound_addr: None,
        })
    }

    /// Return the base URL for this clone (e.g. `http://127.0.0.1:PORT`).
    pub fn base_url(&self) -> String {
        let addr = self.bound_addr();
        format!("http://{addr}")
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Auth
            .route("/login", post(post_login))
            // Alerts
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/alerts", post(get_alerts))
            .route("/api/v1/alerts/:alert_id", get(get_alert_by_id))
            .route("/api/v1/alerts/:alert_id/status", patch(patch_alert_status))
            .route("/api/v1/alerts/:alert_id/close", post(post_close_alert))
            // Threat intel
            .route("/api/v1/threat-intel", get(get_threat_intel))
            // DTU internal
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for CyberintClone {
    /// Bind to a local ephemeral port and start the axum HTTP server.
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);

        let router = self.build_router();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("Cyberint DTU server error");
        });

        Ok(())
    }

    /// Reset all captured state to initial values.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime (auth_mode, rate_limit_after, etc.).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("CyberintClone::bound_addr() called before start()")
    }
}
