// AC-1: BehavioralClone::start() binds a port; bound_addr() returns a reachable SocketAddr.
//
// Test defines a minimal TestClone implementing BehavioralClone. The start() method
// binds an axum server (with LatencyLayer applied at the router level, exercising the
// real type) on 127.0.0.1:0. LatencyLayer::call is todo!() and will panic when the
// first request is processed — causing the server task to crash and the health check
// to fail. That is the expected Red Gate failure.
//
// Expected failure mode: LatencyLayer::call panic causes the server to crash;
// the /health GET returns a connection error instead of 200.

use async_trait::async_trait;
use axum::routing::get;
use prism_dtu_common::{BehavioralClone, LatencyLayer};
use std::net::SocketAddr;

struct TestClone {
    addr: Option<SocketAddr>,
}

#[async_trait]
impl BehavioralClone for TestClone {
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        self.addr = Some(listener.local_addr()?);

        tokio::spawn(async move {
            // Real LatencyLayer applied at router level — call() is todo!().
            let app = axum::Router::new()
                .route("/health", get(|| async { "ok" }))
                .layer(LatencyLayer { latency_ms: 0 });
            axum::serve(listener, app).await.ok();
        });

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn configure(&self, _config: serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }

    fn bound_addr(&self) -> SocketAddr {
        self.addr.expect("AC-1: bound_addr called before start()")
    }
}

#[tokio::test]
async fn ac_1_behavioral_clone_start_binds_and_bound_addr_is_reachable() {
    let mut clone = TestClone { addr: None };

    clone.start().await.expect("AC-1: start() must succeed");

    let addr = clone.bound_addr();
    assert!(addr.port() > 0, "AC-1: bound_addr must have non-zero port");
    assert_eq!(
        addr.ip().to_string(),
        "127.0.0.1",
        "AC-1: must bind to loopback"
    );

    // Verify the server is actually reachable at base_url/health.
    // This request reaches LatencyLayer::call which is todo!() — panics.
    // The test then fails on the assert_eq below (connection error or panic).
    let url = clone.base_url();
    let response = reqwest::get(format!("{url}/health"))
        .await
        .expect("AC-1: server must be reachable at base_url");
    assert_eq!(
        response.status().as_u16(),
        200,
        "AC-1: /health must return 200"
    );
}
