//! `ThreatIntelClone` — implements `BehavioralClone` for the Threat Intel Aggregator DTU.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Router,
};
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::net::TcpListener;

use crate::routes::lookup::{configure, domain_lookup, hash_lookup, ip_lookup};
use crate::state::ThreatIntelState;

/// L2-stateful behavioral clone of the Threat Intel Aggregator infusion plugin API.
///
/// Maintains a fixture registry and per-request rate-limit counter.
/// Binds to `127.0.0.1:0` (ephemeral port) on `start()`.
pub struct ThreatIntelClone {
    pub config: StubConfig,
    pub state: Arc<ThreatIntelState>,
    pub bound_addr: Option<SocketAddr>,
}

impl ThreatIntelClone {
    /// Create a new clone with default `StubConfig` and default fixture registry.
    pub fn new() -> Self {
        Self {
            config: StubConfig::default(),
            state: Arc::new(ThreatIntelState::new()),
            bound_addr: None,
        }
    }

    /// Create with explicit config.
    pub fn with_config(config: StubConfig) -> Self {
        Self {
            config,
            state: Arc::new(ThreatIntelState::new()),
            bound_addr: None,
        }
    }

    fn build_router(&self) -> Router {
        Router::new()
            .route("/v3/ip/:ip", get(ip_lookup))
            .route("/v3/domain/:domain", get(domain_lookup))
            .route("/v3/hash/:hash", get(hash_lookup))
            .route("/dtu/configure", post(configure))
            .with_state(Arc::clone(&self.state))
    }
}

impl Default for ThreatIntelClone {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralClone for ThreatIntelClone {
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);

        let router = self.build_router();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("ThreatIntelClone server error");
        });

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        // Delegate to the same logic as the HTTP configure endpoint.
        if let Some(n) = config.get("rate_limit_after").and_then(|v| v.as_u64()) {
            let mut threshold = self
                .state
                .rate_limit_after
                .lock()
                .expect("rate_limit_after poisoned");
            *threshold = Some(n as u32);
        }

        let lookup_value = config
            .get("ip")
            .or_else(|| config.get("hash"))
            .or_else(|| config.get("domain"))
            .and_then(|v| v.as_str());

        if let (Some(value), Some(fixture_str)) =
            (lookup_value, config.get("fixture").and_then(|v| v.as_str()))
        {
            use crate::types::FixtureKey;
            let fixture_key = match fixture_str {
                "malicious" => FixtureKey::Malicious,
                "benign" => FixtureKey::Benign,
                "unknown" => FixtureKey::Unknown,
                other => anyhow::bail!("unknown fixture key: {}", other),
            };
            let mut registry = self
                .state
                .fixture_registry
                .lock()
                .expect("fixture_registry poisoned");
            registry.insert(value.to_string(), fixture_key);
        }

        Ok(())
    }

    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
            .expect("ThreatIntelClone::start() must be called before bound_addr()")
    }
}
