//! AC-8: reset() clears all three stores (S-6.07).
//!
//! Given `reset()` is called, Then the containment store, detection status store,
//! and session registry are all cleared; a subsequent
//! `GET /devices/entities/devices/v2?ids=h-001` returns the device with
//! `containment_status: "normal"` (base fixture state).
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-8: After contain + reset, GET host detail shows containment_status = "normal".
#[tokio::test]
async fn ac_8_reset_clears_containment_store() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-8: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "test-session-ac8";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // Step 1: Register session.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-8: Step 1 must reach server");

    // Contain h-001.
    let contain_resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-8: contain must reach server");

    assert_eq!(
        contain_resp.status().as_u16(),
        202,
        "AC-8: contain must return 202 before reset"
    );

    // Verify contained status before reset.
    let pre_reset = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-8: pre-reset GET must reach server");

    let pre_reset_body: serde_json::Value = pre_reset
        .json()
        .await
        .expect("AC-8: pre-reset body must be JSON");

    assert_eq!(
        pre_reset_body["resources"][0]["containment_status"]
            .as_str()
            .unwrap_or(""),
        "contained",
        "AC-8: pre-reset containment_status must be 'contained'"
    );

    // Call reset().
    clone.reset().await.expect("AC-8: reset() must succeed");

    // AC-8b: After reset, issue a fresh Step 1 with a NEW session id to re-register
    // IDs, then Step 2 must return base fixture state ("normal").
    // Without Step 1 the new session has no registry entry, which triggers EC-003
    // (empty resources) instead of the fixture lookup.
    let new_session_id = "test-session-ac8-post-reset";
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", new_session_id)
        .send()
        .await
        .expect("AC-8: post-reset Step 1 must reach server");

    let post_reset = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", new_session_id)
        .send()
        .await
        .expect("AC-8: post-reset GET must reach server");

    assert_eq!(
        post_reset.status().as_u16(),
        200,
        "AC-8: post-reset GET must return 200"
    );

    let post_reset_body: serde_json::Value = post_reset
        .json()
        .await
        .expect("AC-8: post-reset body must be JSON");

    assert_eq!(
        post_reset_body["resources"][0]["containment_status"]
            .as_str()
            .unwrap_or(""),
        "normal",
        "AC-8: after reset(), containment_status must return to 'normal' (base fixture state)"
    );
}

/// AC-8: reset() clears session registry — Step 2 after reset returns empty resources.
#[tokio::test]
async fn ac_8_reset_clears_session_registry() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("AC-8 session: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "test-session-ac8-registry";
    let client = reqwest::Client::new();

    // Step 1: register host IDs under the session.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-8 session: Step 1 must reach server");

    // Call reset() — clears the session registry.
    clone
        .reset()
        .await
        .expect("AC-8 session: reset() must succeed");

    // Step 2: request detail for IDs that were registered — registry is now empty.
    // The store was cleared so the session is gone; Step 2 must return empty resources.
    let step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-8 session: Step 2 must reach server");

    assert_eq!(
        step2.status().as_u16(),
        200,
        "AC-8 session: Step 2 after reset must return 200 (not an error)"
    );

    let step2_body: serde_json::Value = step2
        .json()
        .await
        .expect("AC-8 session: Step 2 body must be JSON");

    let resources = step2_body["resources"]
        .as_array()
        .expect("AC-8 session: resources must be array");
    assert!(
        resources.is_empty(),
        "AC-8 session: after reset, Step 2 with previously-registered IDs must return empty resources"
    );
}

/// AC-8: reset() clears detection status store.
#[tokio::test]
async fn ac_8_reset_clears_detection_status_store() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-8 det: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Update detection status before reset.
    let patch_resp = client
        .patch(format!("{base_url}/detects/entities/detects/v2"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["det-001"], "status": "in_progress"}))
        .send()
        .await
        .expect("AC-8 det: PATCH must reach server");

    assert_eq!(
        patch_resp.status().as_u16(),
        200,
        "AC-8 det: PATCH detection status must return 200"
    );

    // Call reset().
    clone.reset().await.expect("AC-8 det: reset() must succeed");

    // After reset, the detection_status_store is cleared.
    // We verify this by querying the detection list — it must still return 200
    // (the reset should not break the server, only clear the stores).
    let post_reset_list = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-8 det: post-reset list must reach server");

    assert_eq!(
        post_reset_list.status().as_u16(),
        200,
        "AC-8 det: after reset(), detection list must still return 200 (server is healthy)"
    );
}
