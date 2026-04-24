#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-3: FailureLayer with AuthReject returns HTTP 401 unconditionally.
//
// Test starts an axum server with FailureLayer(AuthReject) applied via Router::layer().
// Any request — even with a valid Authorization header — must receive 401.
//
// Expected failure mode: FailureLayer::call is todo!() — panics on the first
// request (server task crashes). The reqwest client receives a connection error
// or unexpected status. Either outcome is a valid Red Gate failure.

use axum::routing::get;
use prism_dtu_common::{FailureLayer, FailureMode};

async fn start_server_with_auth_reject() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        // Apply real FailureLayer at the router level — call() is todo!().
        let app = axum::Router::new()
            .route("/api/resource", get(|| async { "secret data" }))
            .layer(FailureLayer {
                mode: FailureMode::AuthReject,
            });
        axum::serve(listener, app).await.ok();
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn ac_3_failure_layer_auth_reject_returns_401_unconditionally() {
    let base_url = start_server_with_auth_reject().await;
    let client = reqwest::Client::new();

    // Request without auth header.
    let resp_no_auth = client
        .get(format!("{base_url}/api/resource"))
        .send()
        .await
        .expect("AC-3: unauthenticated request must be sent");

    assert_eq!(
        resp_no_auth.status().as_u16(),
        401,
        "AC-3: AuthReject must return 401 for request without Authorization header"
    );

    // Request WITH a valid-looking auth header — must still get 401.
    let resp_with_auth = client
        .get(format!("{base_url}/api/resource"))
        .header("Authorization", "Bearer valid-token-123")
        .send()
        .await
        .expect("AC-3: authenticated request must be sent");

    assert_eq!(
        resp_with_auth.status().as_u16(),
        401,
        "AC-3: AuthReject must return 401 regardless of Authorization header value"
    );
}
