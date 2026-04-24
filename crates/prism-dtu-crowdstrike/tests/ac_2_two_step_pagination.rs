//! AC-2: Two-step pagination — Step 1 registers IDs, Step 2 returns detail (S-6.07).
//!
//! Given `GET /devices/queries/devices/v1` is called (Step 1), Then the response
//! `resources` array contains host IDs AND those IDs are registered in the session
//! registry under the `X-DTU-Session-Id` header value; subsequent
//! `GET /devices/entities/devices/v2` with those IDs returns matching host detail
//! records.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-2: Step 1 returns host IDs and Step 2 returns matching detail records.
#[tokio::test]
async fn ac_2_step1_registers_ids_step2_returns_detail() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-2: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "test-session-ac2-001";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // Step 1: GET /devices/queries/devices/v1 — register session, return host IDs.
    let step1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-2: Step 1 request must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "AC-2: Step 1 must return HTTP 200"
    );

    let step1_body: serde_json::Value = step1
        .json()
        .await
        .expect("AC-2: Step 1 body must be valid JSON");

    let host_ids = step1_body["resources"]
        .as_array()
        .expect("AC-2: Step 1 resources must be an array");
    assert!(
        !host_ids.is_empty(),
        "AC-2: Step 1 resources must not be empty (fixture must contain at least one host ID)"
    );

    // Extract the first ID as a string.
    let first_id = host_ids[0]
        .as_str()
        .expect("AC-2: Step 1 resource IDs must be strings");

    // Step 2: GET /devices/entities/devices/v2?ids=<first_id> — retrieve detail.
    let step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", first_id)])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-2: Step 2 request must reach server");

    assert_eq!(
        step2.status().as_u16(),
        200,
        "AC-2: Step 2 must return HTTP 200"
    );

    let step2_body: serde_json::Value = step2
        .json()
        .await
        .expect("AC-2: Step 2 body must be valid JSON");

    let detail_records = step2_body["resources"]
        .as_array()
        .expect("AC-2: Step 2 resources must be an array");
    assert!(
        !detail_records.is_empty(),
        "AC-2: Step 2 resources must not be empty for IDs registered in Step 1"
    );

    // The returned record must include the device_id field.
    assert!(
        detail_records[0].get("device_id").is_some(),
        "AC-2: host detail record must contain 'device_id' field"
    );
}

/// AC-2 detection two-step: detection ID list → detection summaries.
#[tokio::test]
async fn ac_2_detection_two_step_pipeline_returns_summaries() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-2 det: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "test-session-ac2-det";
    let client = reqwest::Client::new();

    // Step 1: GET /detects/queries/detects/v1.
    let step1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-2 det: Step 1 request must reach server");

    assert_eq!(step1.status().as_u16(), 200, "AC-2 det: Step 1 must be 200");

    let step1_body: serde_json::Value = step1
        .json()
        .await
        .expect("AC-2 det: Step 1 body must be JSON");

    let det_ids = step1_body["resources"]
        .as_array()
        .expect("AC-2 det: resources must be array");
    assert!(
        !det_ids.is_empty(),
        "AC-2 det: detection IDs must not be empty"
    );

    let first_det_id = det_ids[0].as_str().expect("AC-2 det: IDs must be strings");

    // Step 2: POST /detects/entities/summaries/GET/v1.
    let step2 = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .json(&serde_json::json!({"ids": [first_det_id]}))
        .send()
        .await
        .expect("AC-2 det: Step 2 request must reach server");

    assert_eq!(step2.status().as_u16(), 200, "AC-2 det: Step 2 must be 200");

    let step2_body: serde_json::Value = step2
        .json()
        .await
        .expect("AC-2 det: Step 2 body must be JSON");

    let summaries = step2_body["resources"]
        .as_array()
        .expect("AC-2 det: Step 2 resources must be array");
    assert!(
        !summaries.is_empty(),
        "AC-2 det: summaries must not be empty"
    );

    assert!(
        summaries[0].get("detection_id").is_some(),
        "AC-2 det: detection summary must contain 'detection_id' field"
    );
}

/// AC-2 session isolation: two sessions do not share IDs.
#[tokio::test]
async fn ac_2_different_sessions_are_isolated() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-2 iso: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Session A: Step 1.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "session-A")
        .send()
        .await
        .expect("AC-2 iso: session A Step 1 must reach server");

    // Session B: Step 2 with IDs that were registered under session A —
    // using a different X-DTU-Session-Id. The registry is keyed by session ID,
    // so Step 2 under session B must return an empty resources array.
    let step2_b = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "session-B")
        .send()
        .await
        .expect("AC-2 iso: session B Step 2 must reach server");

    assert_eq!(
        step2_b.status().as_u16(),
        200,
        "AC-2 iso: cross-session Step 2 must return 200 (not an error)"
    );

    let body: serde_json::Value = step2_b.json().await.expect("AC-2 iso: body must be JSON");

    // IDs were not registered under session-B, so resources must be empty.
    // (This is EC-003 precondition — full EC-003 is in edge_cases.rs.)
    let resources = body["resources"]
        .as_array()
        .expect("AC-2 iso: resources must be array");
    assert!(
        resources.is_empty(),
        "AC-2 iso: Step 2 under different session must return empty resources"
    );
}
