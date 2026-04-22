//! AC-3: Contain write persists to containment_store (S-6.07).
//!
//! Given `POST /devices/entities/devices-actions/v2?action_name=contain` is
//! called with `{"ids": ["h-001"]}`, Then the response is HTTP 202 with
//! `containment_status: "contained"`, AND subsequent
//! `GET /devices/entities/devices/v2?ids=h-001` returns
//! `containment_status: "contained"` in the host record.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-3: Contain write returns 202 and containment_status becomes "contained".
#[tokio::test]
async fn ac_3_contain_returns_202_with_contained_status() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-3: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .post(format!(
            "{base_url}/devices/entities/devices-actions/v2"
        ))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-3: contain request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "AC-3: contain must return HTTP 202"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-3: body must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("AC-3: resources must be an array");
    assert!(!resources.is_empty(), "AC-3: resources must not be empty");

    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "AC-3: contain response must set containment_status to 'contained'"
    );
    assert_eq!(
        resources[0]["device_id"].as_str().unwrap_or(""),
        "h-001",
        "AC-3: contain response resources[0].device_id must be 'h-001'"
    );
}

/// AC-3: Write persists — subsequent GET reflects the contained status.
#[tokio::test]
async fn ac_3_contain_persists_to_store_subsequent_get_reflects_status() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-3 persist: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "test-session-ac3-persist";
    let client = reqwest::Client::new();

    // First: Step 1 to register IDs in session registry.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-3 persist: Step 1 must reach server");

    // Issue the contain write.
    let contain_resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-3 persist: contain request must reach server");

    assert_eq!(
        contain_resp.status().as_u16(),
        202,
        "AC-3 persist: contain must return 202"
    );

    // Now GET /devices/entities/devices/v2?ids=h-001 — must reflect contained status.
    let get_resp = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-3 persist: GET detail must reach server");

    assert_eq!(
        get_resp.status().as_u16(),
        200,
        "AC-3 persist: GET host detail must return 200"
    );

    let get_body: serde_json::Value =
        get_resp.json().await.expect("AC-3 persist: GET body must be JSON");

    let records = get_body["resources"]
        .as_array()
        .expect("AC-3 persist: resources must be array");
    assert!(!records.is_empty(), "AC-3 persist: resources must not be empty");

    assert_eq!(
        records[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "AC-3 persist: GET host detail must reflect containment_status 'contained' after contain write"
    );
}

/// AC-3: Lift containment returns 202 and transitions back to "normal".
#[tokio::test]
async fn ac_3_lift_containment_returns_202_with_normal_status() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-3 lift: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // First contain the device.
    client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-lift-001"]}))
        .send()
        .await
        .expect("AC-3 lift: initial contain must reach server");

    // Now lift containment.
    let lift_resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "lift_containment")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-lift-001"]}))
        .send()
        .await
        .expect("AC-3 lift: lift_containment request must reach server");

    assert_eq!(
        lift_resp.status().as_u16(),
        202,
        "AC-3 lift: lift_containment must return HTTP 202"
    );

    let lift_body: serde_json::Value =
        lift_resp.json().await.expect("AC-3 lift: body must be JSON");

    let resources = lift_body["resources"]
        .as_array()
        .expect("AC-3 lift: resources must be array");
    assert!(!resources.is_empty(), "AC-3 lift: resources must not be empty");

    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "normal",
        "AC-3 lift: lift_containment must set containment_status to 'normal'"
    );
}
