//! `ArmisClone` — `BehavioralClone` implementation for the Armis Centrix DTU.
//!
//! Lifecycle:
//! 1. `ArmisClone::new()` — allocates state; loads fixtures into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears tag store + AQL log; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{delete, get, post},
    Router,
};
use prism_dtu_common::BehavioralClone;

use crate::routes::{
    alerts::get_alerts,
    devices::{get_device_activity, get_device_risk, get_or_post_devices},
    dtu::{get_aql_log, get_health, post_configure, post_reset},
    tags::{delete_device_tag, post_device_tag},
};
use crate::state::ArmisState;
use crate::types::{ActivityRecord, AlertRecord, DeviceRecord};

/// L2-fidelity behavioral clone of the Armis Centrix API.
pub struct ArmisClone {
    state: Arc<ArmisState>,
    bound_addr: Option<SocketAddr>,
}

impl ArmisClone {
    /// Create a new `ArmisClone`. Loads all fixtures from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture_as` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let devices: Vec<DeviceRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "devices")?;
        let activity: Vec<ActivityRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "device-activity")?;
        let alerts: Vec<AlertRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;

        let state = Arc::new(ArmisState::new(devices, activity, alerts));
        Ok(Self {
            state,
            bound_addr: None,
        })
    }

    /// Return the base URL for the bound server (e.g. `"http://127.0.0.1:12345"`).
    ///
    /// Panics if `start()` has not been called.
    pub fn base_url(&self) -> String {
        let addr = self.bound_addr();
        format!("http://{addr}")
    }

    fn build_router(&self) -> Router {
        Router::new()
            // Vendor API routes — Armis Centrix
            .route("/api/v1/devices", get(get_or_post_devices))
            .route("/api/v1/devices", post(get_or_post_devices))
            .route(
                "/api/v1/devices/:device_id/activity",
                get(get_device_activity),
            )
            .route("/api/v1/devices/:device_id/risk", get(get_device_risk))
            .route("/api/v1/alerts", get(get_alerts))
            .route(
                "/api/v1/devices/:device_id/tags/",
                post(post_device_tag),
            )
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(delete_device_tag),
            )
            // DTU internal test API routes (ADR-002 §6 required set)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            // Armis-specific introspection: AQL capture log
            .route("/dtu/aql-log", get(get_aql_log))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for ArmisClone {
    /// Bind to a local ephemeral port and start the axum HTTP server.
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);

        let router = self.build_router();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("Armis DTU server error");
        });

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
    ///
    /// Per ADR-002 §4: delegates to `self.state.apply_config(&config)` — no inline
    /// JSON field inspection in `clone.rs`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("ArmisClone::bound_addr() called before start()")
    }
}
