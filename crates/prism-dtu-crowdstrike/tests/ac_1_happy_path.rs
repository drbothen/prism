//! AC-1: start + bound_addr + GET /detects/queries/detects/v1 returns 200 (S-6.07).
//!
//! Given `CrowdstrikeClone::start()` is called, Then `bound_addr()` returns a
//! valid socket address and `GET /detects/queries/detects/v1` with a valid Bearer
//! token returns HTTP 200 with a `resources` array.
//!
//! Was Red Gate at implementation start; CrowdstrikeClone::start() now implemented.
//! "not yet implemented".

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{BehavioralClone, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-1: CrowdstrikeClone::start() binds a loopback port and the detection ID
/// endpoint returns HTTP 200 with a resources array when a valid bearer is sent.
#[tokio::test]
async fn ac_1_start_binds_port_and_detections_returns_200() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::with_config(StubConfig::default());
    clone.start().await.expect("AC-1: start() must succeed");

    // bound_addr() must return a valid loopback address after start().
    let addr = clone.bound_addr();
    assert!(
        addr.ip().is_loopback(),
        "AC-1: bound_addr must be a loopback address, got: {addr}"
    );
    assert_ne!(addr.port(), 0, "AC-1: bound port must be non-zero");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // GET /detects/queries/detects/v1 with a valid bearer token must return 200.
    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1: detection ID list request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-1: response body must be valid JSON");

    assert!(
        body["resources"].is_array(),
        "AC-1: response must contain a 'resources' array, got: {body}"
    );
}

/// AC-1 meta field: pagination metadata is present in the response.
#[tokio::test]
async fn ac_1_detections_response_includes_pagination_meta() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("AC-1 meta: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1 meta: request must reach server");

    assert_eq!(resp.status().as_u16(), 200, "AC-1 meta: must return 200");

    let body: serde_json::Value = resp.json().await.expect("AC-1 meta: body must be JSON");

    assert!(
        body.get("meta").is_some(),
        "AC-1 meta: response must contain 'meta' field"
    );
    let meta = &body["meta"];
    assert!(
        meta["pagination"].is_object(),
        "AC-1 meta: meta.pagination must be an object"
    );
    assert!(
        meta["pagination"]["total"].is_number(),
        "AC-1 meta: meta.pagination.total must be a number"
    );
}

/// AC-1 hosts read: GET /devices/queries/devices/v1 also returns 200 after start().
#[tokio::test]
async fn ac_1_hosts_query_returns_200() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("AC-1 hosts: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1 hosts: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1 hosts: GET /devices/queries/devices/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-1 hosts: body must be JSON");

    assert!(
        body["resources"].is_array(),
        "AC-1 hosts: response must contain a 'resources' array"
    );
}
