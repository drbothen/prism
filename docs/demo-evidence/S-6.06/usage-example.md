# Usage Example — Downstream DTU crate (e.g. prism-dtu-crowdstrike)

This snippet shows how S-6.07 (`prism-dtu-crowdstrike`) or any other per-surface
DTU crate would wire together `BehavioralClone`, `LatencyLayer`, `FailureLayer`,
and `load_fixture`. The pattern is derived directly from the AC integration tests.

```rust
// In crates/prism-dtu-crowdstrike/src/lib.rs (dev/test builds only)
#![cfg(any(test, feature = "dtu"))]

use async_trait::async_trait;
use axum::routing::get;
use prism_dtu_common::{
    BehavioralClone, FailureLayer, FailureMode, LatencyLayer, StubConfig, load_fixture,
};
use std::net::SocketAddr;

pub struct CrowdStrikeDtu {
    config: StubConfig,
    addr: Option<SocketAddr>,
}

impl CrowdStrikeDtu {
    pub fn new(config: StubConfig) -> Self {
        Self { config, addr: None }
    }
}

#[async_trait]
impl BehavioralClone for CrowdStrikeDtu {
    async fn start(&mut self) -> anyhow::Result<()> {
        // Load a fixture to seed the stub's response data.
        let devices = load_fixture(env!("CARGO_MANIFEST_DIR"), "devices-page1");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        self.addr = Some(listener.local_addr()?);

        let latency_ms = self.config.latency_ms;
        let failure_mode = self.config.failure_mode.clone();

        tokio::spawn(async move {
            let app = axum::Router::new()
                .route("/devices/v1/devices/query", get(move || {
                    let d = devices.clone();
                    async move { axum::Json(d) }
                }))
                .layer(FailureLayer { mode: failure_mode })
                .layer(LatencyLayer { latency_ms });
            axum::serve(listener, app).await.ok();
        });

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> { Ok(()) }

    async fn configure(&self, _cfg: serde_json::Value) -> anyhow::Result<()> { Ok(()) }

    fn bound_addr(&self) -> SocketAddr {
        self.addr.expect("start() must be called before bound_addr()")
    }
}
```

### Usage in an integration test

```rust
#[tokio::test]
async fn crowdstrike_dtu_rate_limits_after_5_requests() {
    let config = StubConfig {
        latency_ms: 0,
        failure_mode: FailureMode::RateLimit { after_n_requests: 5, retry_after_secs: 60 },
        ..Default::default()
    };
    let mut dtu = CrowdStrikeDtu::new(config);
    dtu.start().await.unwrap();

    let client = reqwest::Client::new();
    for _ in 0..5 {
        let resp = client.get(format!("{}/devices/v1/devices/query", dtu.base_url()))
            .send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);
    }
    // 6th request hits the rate limit
    let resp = client.get(format!("{}/devices/v1/devices/query", dtu.base_url()))
        .send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 429);
    assert_eq!(resp.headers().get("Retry-After").unwrap(), "60");
}
```
