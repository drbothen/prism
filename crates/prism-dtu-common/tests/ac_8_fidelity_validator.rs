// AC-8: FidelityValidator flags a missing required field in the DTU response body.
//
// Test starts a minimal axum server that returns {"result": "ok"} (no "status" field).
// Runs FidelityValidator with a check requiring field "status".
// Expects FidelityReport to contain a FidelityFailure for that endpoint.
//
// Expected failure mode: FidelityValidator::run is todo!() — panics at runtime.

use prism_dtu_common::{FidelityCheck, FidelityValidator};

async fn start_stub_without_status_field() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/api/v1/resource",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({"result": "ok"}))
            }),
        );
        axum::serve(listener, app).await.ok();
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn ac_8_fidelity_validator_flags_missing_required_field() {
    let base_url = start_stub_without_status_field().await;

    let checks = vec![FidelityCheck {
        endpoint: "/api/v1/resource".to_owned(),
        method: http::Method::GET,
        body: None,
        expected_status: 200,
        required_fields: vec!["status".to_owned()],
    }];

    let report = FidelityValidator::run(&base_url, checks).await;

    assert_eq!(
        report.checks_failed, 1,
        "AC-8: one check must fail (missing 'status' field)"
    );
    assert_eq!(
        report.checks_passed, 0,
        "AC-8: zero checks must pass"
    );
    assert!(
        !report.failures.is_empty(),
        "AC-8: FidelityReport.failures must be non-empty"
    );

    let failure = &report.failures[0];
    assert_eq!(
        failure.endpoint, "/api/v1/resource",
        "AC-8: failure endpoint must match the check endpoint"
    );
    assert!(
        failure.reason.contains("status"),
        "AC-8: failure reason must mention the missing field 'status'; got: {}",
        failure.reason
    );
}
