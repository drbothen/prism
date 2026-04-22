// AC-2: FailureLayer with RateLimit returns HTTP 429 + Retry-After after threshold.
//
// Test starts an axum server with FailureLayer applied via Router::layer().
// Sends 6 requests. The 6th must get HTTP 429 with Retry-After: 60.
//
// Expected failure mode: FailureLayer::call is todo!() — panics on the first
// request (server task crashes or returns an error body). The reqwest client will
// see a connection error or unexpected status. Either outcome is a valid Red Gate
// failure because the stub is not yet implemented.

use axum::routing::get;
use prism_dtu_common::{FailureLayer, FailureMode};

async fn start_server_with_rate_limit() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        // Apply real FailureLayer at the router level — call() is todo!().
        let app = axum::Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(FailureLayer {
                mode: FailureMode::RateLimit {
                    after_n_requests: 5,
                    retry_after_secs: 60,
                },
            });
        axum::serve(listener, app).await.ok();
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn ac_2_failure_layer_rate_limit_returns_429_after_threshold() {
    let base_url = start_server_with_rate_limit().await;
    let client = reqwest::Client::new();

    // Send 5 "allowed" requests.
    for i in 1..=5 {
        let resp = client
            .get(format!("{base_url}/ping"))
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-2: request {i} failed unexpectedly"));
        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-2: request {i} must succeed (within rate limit)"
        );
    }

    // 6th request must receive 429 with Retry-After: 60.
    let resp = client
        .get(format!("{base_url}/ping"))
        .send()
        .await
        .expect("AC-2: 6th request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-2: 6th request must return HTTP 429"
    );

    let retry_after = resp
        .headers()
        .get("Retry-After")
        .expect("AC-2: HTTP 429 response must include Retry-After header");
    assert_eq!(
        retry_after
            .to_str()
            .expect("Retry-After header is valid ASCII"),
        "60",
        "AC-2: Retry-After must equal configured retry_after_secs (60)"
    );
}
