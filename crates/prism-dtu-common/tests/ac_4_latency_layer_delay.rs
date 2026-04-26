#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-4: LatencyLayer delays response by at least 80ms when configured 100ms.
//
// Test starts an axum server with LatencyLayer applied via Router::layer().
// Measures elapsed time for a single request. Expects >= 80ms elapsed.
//
// Expected failure mode: LatencyLayer::call is todo!() — panics on the first
// request (server task crashes). The client sees a connection error before any
// Was Red Gate at implementation start; timing assertion now passes.

use axum::routing::get;
use prism_dtu_common::LatencyLayer;
use std::time::Instant;

async fn start_server_with_latency(latency_ms: u64) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        // Apply real LatencyLayer at the router level — call() is todo!().
        let app = axum::Router::new()
            .route("/slow", get(|| async { "eventually" }))
            .layer(LatencyLayer { latency_ms });
        axum::serve(listener, app).await.ok();
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn ac_4_latency_layer_delays_response_by_configured_ms() {
    let base_url = start_server_with_latency(100).await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("build reqwest client with 5s timeout");

    let start = Instant::now();
    let resp = client
        .get(format!("{base_url}/slow"))
        .send()
        .await
        .expect("AC-4: request must complete (timeout at 5s)");
    let elapsed = start.elapsed();

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-4: server must return 200 after the delay"
    );
    assert!(
        elapsed.as_millis() >= 80,
        "AC-4: response must be delayed at least 80ms; got {}ms",
        elapsed.as_millis()
    );
}
