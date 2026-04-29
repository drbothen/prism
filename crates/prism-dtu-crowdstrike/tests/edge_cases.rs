//! Edge case coverage for the CrowdStrike DTU (S-6.07).
//!
//! Covers EC-001 through EC-006 from the story spec Edge Cases table.
//!
//! Was Red Gate at implementation start; CrowdstrikeClone::start() now implemented.
//! "not yet implemented".

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{BehavioralClone, FailureMode, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

// ──────────────────────────────────────────────────────────────────────────────
// EC-001: `contain` called with empty `ids` array → HTTP 400
// ──────────────────────────────────────────────────────────────────────────────

/// EC-001: POST contain with empty ids returns 400 with structured error.
#[tokio::test]
async fn ec_001_contain_empty_ids_returns_400() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("EC-001: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": []}))
        .send()
        .await
        .expect("EC-001: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-001: contain with empty ids must return HTTP 400"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-001: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("EC-001: errors must be array");
    assert!(!errors.is_empty(), "EC-001: errors array must not be empty");

    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        400,
        "EC-001: error code must be 400"
    );

    let msg = errors[0]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("empty") || msg.contains("ids"),
        "EC-001: error message must mention 'ids' or 'empty', got: {msg:?}"
    );
}

/// EC-001: lift_containment with empty ids also returns 400.
#[tokio::test]
async fn ec_001_lift_containment_empty_ids_returns_400() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("EC-001 lift: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "lift_containment")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": []}))
        .send()
        .await
        .expect("EC-001 lift: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-001 lift: lift_containment with empty ids must return HTTP 400"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-002: `contain` on already-contained device → HTTP 400
// ──────────────────────────────────────────────────────────────────────────────

/// EC-002: Second contain on same device returns 400 "device already contained".
#[tokio::test]
async fn ec_002_contain_already_contained_returns_400() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("EC-002: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // First contain — must succeed.
    let first = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-ec002"]}))
        .send()
        .await
        .expect("EC-002: first contain must reach server");

    assert_eq!(
        first.status().as_u16(),
        202,
        "EC-002: first contain must return 202"
    );

    // Second contain on already-contained device — must return 400.
    let second = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-ec002"]}))
        .send()
        .await
        .expect("EC-002: second contain must reach server");

    assert_eq!(
        second.status().as_u16(),
        400,
        "EC-002: contain on already-contained device must return HTTP 400"
    );

    let body: serde_json::Value = second.json().await.expect("EC-002: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("EC-002: errors must be array");
    assert!(!errors.is_empty(), "EC-002: errors array must not be empty");

    let msg = errors[0]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("already contained"),
        "EC-002: error must say 'device already contained', got: {msg:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-003: Step 2 with IDs not in session registry → 200 empty resources
// ──────────────────────────────────────────────────────────────────────────────

/// EC-003: Step 2 with unknown IDs (no Step 1 registered) returns 200 empty resources.
#[tokio::test]
async fn ec_003_step2_unknown_ids_returns_200_empty() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("EC-003: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Step 2 without a prior Step 1 — IDs are not in the session registry.
    let resp = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "does-not-exist-001"), ("ids", "does-not-exist-002")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "ec003-orphan-session")
        .send()
        .await
        .expect("EC-003: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003: Step 2 with unknown IDs must return HTTP 200 (not an error)"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-003: body must be JSON");

    let resources = body["resources"]
        .as_array()
        .expect("EC-003: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-003: Step 2 with unknown IDs must return empty resources array"
    );
}

/// EC-003: Same for detection summaries — unknown IDs return 200 empty.
#[tokio::test]
async fn ec_003_detection_step2_unknown_ids_returns_200_empty() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("EC-003 det: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "ec003-orphan-det-session")
        .json(&serde_json::json!({"ids": ["not-registered-001"]}))
        .send()
        .await
        .expect("EC-003 det: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003 det: detection Step 2 with unknown IDs must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-003 det: body must be JSON");
    let resources = body["resources"]
        .as_array()
        .expect("EC-003 det: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-003 det: unknown detection IDs must return empty resources"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-004: LRU eviction at 1,000 entries
// ──────────────────────────────────────────────────────────────────────────────

/// EC-004: When 1,001 sessions are registered, the oldest is evicted; no panic.
///
/// This test verifies the LRU capacity boundary (max 1,000 entries). It fills
/// the registry to 1,000 + 1 entries and asserts:
/// - The 1,001st session is registered successfully (no error).
/// - The 1st session's IDs are evicted (Step 2 returns empty for session-0).
/// - No panic occurs during or after eviction.
#[tokio::test]
async fn ec_004_lru_eviction_at_1000_sessions_no_panic() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("EC-004: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    // Fill the registry to capacity (1,000 unique sessions).
    for i in 0..1000usize {
        let session_id = format!("ec004-session-{i:04}");
        let resp = client
            .get(format!("{base_url}/devices/queries/devices/v1"))
            .header("Authorization", "Bearer dtu-fake-cs-token")
            .header("X-DTU-Session-Id", &session_id)
            .send()
            .await
            .unwrap_or_else(|_| panic!("EC-004: session {i} registration must reach server"));

        assert_eq!(
            resp.status().as_u16(),
            200,
            "EC-004: session {i} registration must return 200"
        );
    }

    // Register session 1,001 — this must evict session-0 (LRU policy).
    let overflow_session = "ec004-overflow-1001";
    let overflow_resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", overflow_session)
        .send()
        .await
        .expect("EC-004: overflow session must reach server");

    assert_eq!(
        overflow_resp.status().as_u16(),
        200,
        "EC-004: overflow session registration must return 200 (no error on eviction)"
    );

    // Verify: session-0 was evicted — Step 2 under that session returns empty resources.
    let evicted_step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "ec004-session-0000")
        .send()
        .await
        .expect("EC-004: evicted session Step 2 must reach server");

    assert_eq!(
        evicted_step2.status().as_u16(),
        200,
        "EC-004: Step 2 for evicted session must return 200"
    );

    let evicted_body: serde_json::Value = evicted_step2
        .json()
        .await
        .expect("EC-004: evicted body must be JSON");
    let resources = evicted_body["resources"]
        .as_array()
        .expect("EC-004: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-004: evicted session's Step 2 must return empty resources (LRU evicted)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-005: Mid-pagination 503 on Step 2 batch 2 of 3
// ──────────────────────────────────────────────────────────────────────────────

/// EC-005: FailureMode::InternalError { at_request_n: 3 } returns 500 on 3rd request.
///
/// Simulates a mid-pagination failure: Step 1 (request 1 + 2) succeeds, but
/// Step 2 batch 2 (request 3) returns HTTP 500. This maps to E-SENSOR-005
/// (partial results). The test verifies the 500 is returned, not a panic.
#[tokio::test]
async fn ec_005_mid_pagination_500_on_step2_batch2() {
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::InternalError { at_request_n: 3 },
        bind: None,
        ..Default::default()
    });
    clone.start().await.expect("EC-005: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "ec005-session";
    let client = reqwest::Client::new();

    // Request 1: Step 1 (detection list) — succeeds.
    let step1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("EC-005: request 1 (Step 1) must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "EC-005: request 1 must succeed (Step 1 detection list)"
    );

    // Request 2: Step 2 batch 1 — succeeds.
    let batch1 = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("EC-005: request 2 (Step 2 batch 1) must reach server");

    assert_eq!(
        batch1.status().as_u16(),
        200,
        "EC-005: request 2 must succeed (Step 2 batch 1)"
    );

    // Request 3: Step 2 batch 2 — must return 500 (FailureMode injection).
    let batch2 = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .json(&serde_json::json!({"ids": ["det-002"]}))
        .send()
        .await
        .expect("EC-005: request 3 (Step 2 batch 2) must reach server");

    assert_eq!(
        batch2.status().as_u16(),
        500,
        "EC-005: request 3 must return HTTP 500 (FailureMode::InternalError at_request_n=3)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EC-006: reset() called during active query — no panic, in-flight returns empty
// ──────────────────────────────────────────────────────────────────────────────

/// EC-006: reset() during an active query does not panic; subsequent Step 2 returns empty.
///
/// This test simulates reset() being called between Step 1 and Step 2 of a query.
/// The expected behavior is:
/// - reset() clears the session registry (and other stores).
/// - The in-flight Step 2 call (after reset) returns empty resources.
/// - No panic occurs in the server.
#[tokio::test]
async fn ec_006_reset_during_active_query_returns_empty_no_panic() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("EC-006: start() must succeed");

    let base_url = clone.base_url();
    let session_id = "ec006-active-session";
    let client = reqwest::Client::new();

    // Step 1: register IDs in the session registry.
    let step1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("EC-006: Step 1 must reach server");

    assert_eq!(step1.status().as_u16(), 200, "EC-006: Step 1 must succeed");

    // Simulate reset() being called between Step 1 and Step 2 (mid-query).
    // In production this would happen from another thread, but here we call it
    // synchronously to test the state invariant.
    clone
        .reset()
        .await
        .expect("EC-006: reset() must succeed during active query");

    // Step 2: the session was cleared by reset — must return empty resources, not panic.
    let step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("EC-006: Step 2 after reset must reach server (no panic)");

    assert_eq!(
        step2.status().as_u16(),
        200,
        "EC-006: Step 2 after reset must return 200 (cleared state is not an error)"
    );

    let body: serde_json::Value = step2
        .json()
        .await
        .expect("EC-006: Step 2 body must be JSON");
    let resources = body["resources"]
        .as_array()
        .expect("EC-006: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-006: Step 2 after reset must return empty resources (session cleared)"
    );

    // Verify the server is still healthy after the reset-during-query scenario.
    let health_check = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("EC-006: health check after reset must reach server");

    assert_eq!(
        health_check.status().as_u16(),
        200,
        "EC-006: server must still be healthy after reset() during active query"
    );
}
