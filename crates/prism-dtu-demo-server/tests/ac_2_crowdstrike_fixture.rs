//! AC-2: CrowdStrike clone fixture contract — devices query returns cursor pagination.
//!
//! Given the harness is running with the crowdstrike clone, when
//! `GET /devices/queries/devices/v1` is called with a valid Authorization header,
//! then the response has HTTP 200 and a `resources` array (S-6.07 fixture contract).
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// AC-2: crowdstrike device query returns 200 with `resources` array after harness start.
#[tokio::test]
async fn ac_2_crowdstrike_devices_query_returns_200_with_resources() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-2: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-2: start_all must succeed");

    // Find the crowdstrike clone's bound address.
    let cs_addr = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .and_then(|p| p.bound_addr)
        .expect("AC-2: crowdstrike clone must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/devices/queries/devices/v1");

    let resp = client
        .get(&url)
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-2: request to crowdstrike must succeed");

    assert_eq!(
        resp.status(),
        200,
        "AC-2: /devices/queries/devices/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-2: response must be valid JSON");

    assert!(
        body.get("resources").is_some(),
        "AC-2: response body must contain a `resources` key (S-6.07 fixture contract); got: {body}"
    );

    harness.stop_all().await;
}

/// AC-2: crowdstrike detections query returns 200 with cursor pagination fields.
#[tokio::test]
async fn ac_2_crowdstrike_detections_returns_200_with_pagination() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-2: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-2: start_all must succeed");

    let cs_addr = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .and_then(|p| p.bound_addr)
        .expect("AC-2: crowdstrike clone must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/detects/queries/detects/v1");

    let resp = client
        .get(&url)
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-2: request must succeed");

    assert_eq!(
        resp.status(),
        200,
        "AC-2: detections endpoint must return 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-2: must be valid JSON");

    // S-6.07 fixture contract: cursor pagination uses `resources` array.
    assert!(
        body.get("resources").is_some(),
        "AC-2: detections body must contain `resources` for cursor pagination; got: {body}"
    );

    harness.stop_all().await;
}
