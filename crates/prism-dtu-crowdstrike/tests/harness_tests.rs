//! `harness_tests.rs` — prism-dtu-crowdstrike harness-hosted test suite.
//!
//! Migrates all `prism-dtu-crowdstrike` acceptance criteria, integration tests,
//! edge case tests, TD tests, and fidelity validator to use `prism-dtu-harness`.
//! Adds isolation ACs required by BC-3.5.001 and BC-3.5.002.
//!
//! # Story
//!
//! S-3.4.03 — Migrate prism-dtu-crowdstrike tests to prism-dtu-harness
//!
//! # BC Anchors
//!
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.5.002 — Harness Network Isolation Invariants
//! - BC-3.2.003 — Per-Org Session Token Isolation (D-048; session_registry NOT re-keyed)
//!
//! # Acceptance Criteria Coverage
//!
//! | AC | Function(s) |
//! |----|-------------|
//! | AC-001 (13 original ACs via harness) | ac_1_*, ac_2_*, ac_3_*, ac_4_*, ac_5_*, ac_6_*, ac_7_*, ac_8_* |
//! | AC-002 (integration tests via harness) | integration_vp033_*, integration_vp036_* |
//! | AC-003 (edge cases via harness) | ec_001_*, ec_002_*, ec_003_*, ec_004_*, ec_005_*, ec_006_* |
//! | AC-004 (fidelity validator via harness) | test_BC_3_5_001_fidelity_validator_checks_failed_zero |
//! | AC-005 (2-org logical disjoint) | test_BC_3_5_001_ac_multi_org_logical_isolation |
//! | AC-006 (network 401 cross-creds) | test_BC_3_5_002_ac_network_cross_creds_401 |
//! | AC-007 (no direct CrowdstrikeClone::start) | enforced by file structure; no direct clone instantiation here |
//!
//! # Feature gate
//!
//! This test binary is only compiled with `--features dtu`.

#![cfg(feature = "dtu")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(non_snake_case)]

use prism_dtu_common::{FailureMode, FidelityCheck, FidelityValidator};
use prism_dtu_harness::types::DtuType;
use prism_dtu_harness::{HarnessBuilder, IsolationMode};

// ============================================================================
// Helpers
// ============================================================================

/// Build a single-org CrowdStrike-only harness with `IsolationMode::Logical`.
///
/// Uses seed=42 for deterministic fixture generation.
async fn build_cs_harness(slug: &str) -> prism_dtu_harness::Harness {
    HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(slug, |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("single-org CrowdStrike harness must build")
}

/// Build a single-org CrowdStrike-only harness with a specific seed.
async fn build_cs_harness_with_seed(slug: &str, seed: u64) -> prism_dtu_harness::Harness {
    HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(slug, |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(seed);
        })
        .build()
        .await
        .expect("single-org CrowdStrike harness must build")
}

/// Build a single-org CrowdStrike harness with a pre-injected failure mode.
async fn build_cs_harness_with_failure(
    slug: &str,
    mode: FailureMode,
) -> prism_dtu_harness::Harness {
    HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(slug, |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed = 42;
            spec.initial_failure.insert(DtuType::CrowdStrike, mode);
        })
        .build()
        .await
        .expect("single-org CrowdStrike harness with failure must build")
}

/// Return the bound SocketAddr for a given slug + DtuType::CrowdStrike.
fn cs_addr(harness: &prism_dtu_harness::Harness, slug: &str) -> std::net::SocketAddr {
    harness
        .endpoint_for(slug, DtuType::CrowdStrike)
        .unwrap_or_else(|| panic!("no CrowdStrike endpoint for slug={slug:?}"))
}

/// Build the base URL for CrowdStrike endpoint in the harness.
fn cs_base_url(harness: &prism_dtu_harness::Harness, slug: &str) -> String {
    let addr = cs_addr(harness, slug);
    format!("http://{addr}")
}

/// Build a reqwest client with a 10-second timeout.
fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("reqwest client must build")
}

// ============================================================================
// AC-1: start + bound port + detection/host endpoints return 200
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-1)
// ============================================================================

/// AC-1 (harness): CrowdStrike clone hosted by harness binds a port and
/// `GET /detects/queries/detects/v1` returns HTTP 200 with a `resources` array.
///
/// Migrated from: ac_1_happy_path.rs::ac_1_start_binds_port_and_detections_returns_200
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_1_start_binds_port_and_detections_returns_200() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1: detection ID list request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1 (harness): GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-1: body must be valid JSON");

    assert!(
        body["resources"].is_array(),
        "AC-1 (harness): response must contain a 'resources' array, got: {body}"
    );
}

/// AC-1 (harness): Detection response includes pagination metadata.
///
/// Migrated from: ac_1_happy_path.rs::ac_1_detections_response_includes_pagination_meta
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_1_detections_response_includes_pagination_meta() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1 meta: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1 meta (harness): must return 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-1 meta: body must be JSON");

    assert!(
        body.get("meta").is_some(),
        "AC-1 meta (harness): response must contain 'meta' field"
    );
    let meta = &body["meta"];
    assert!(
        meta["pagination"].is_object(),
        "AC-1 meta (harness): meta.pagination must be an object"
    );
    assert!(
        meta["pagination"]["total"].is_number(),
        "AC-1 meta (harness): meta.pagination.total must be a number"
    );
}

/// AC-1 (harness): Hosts query returns HTTP 200.
///
/// Migrated from: ac_1_happy_path.rs::ac_1_hosts_query_returns_200
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_1_hosts_query_returns_200() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-1 hosts: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1 hosts (harness): GET /devices/queries/devices/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-1 hosts: body must be JSON");

    assert!(
        body["resources"].is_array(),
        "AC-1 hosts (harness): response must contain a 'resources' array"
    );
}

// ============================================================================
// AC-2: two-step pagination
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-2)
// ============================================================================

/// AC-2 (harness): Step 1 registers IDs, step 2 returns detail records.
///
/// Migrated from: ac_2_two_step_pagination.rs::ac_2_step1_registers_ids_step2_returns_detail
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_2_step1_registers_ids_step2_returns_detail() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-test-session-ac2-001";
    let client = make_client();

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
        "AC-2 (harness): Step 1 must return HTTP 200"
    );

    let step1_body: serde_json::Value = step1.json().await.expect("AC-2: Step 1 body must be JSON");

    let host_ids = step1_body["resources"]
        .as_array()
        .expect("AC-2: Step 1 resources must be an array");
    assert!(
        !host_ids.is_empty(),
        "AC-2 (harness): Step 1 resources must not be empty"
    );

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
        "AC-2 (harness): Step 2 must return HTTP 200"
    );

    let step2_body: serde_json::Value = step2.json().await.expect("AC-2: Step 2 body must be JSON");

    let detail_records = step2_body["resources"]
        .as_array()
        .expect("AC-2: Step 2 resources must be an array");
    assert!(
        !detail_records.is_empty(),
        "AC-2 (harness): Step 2 resources must not be empty for IDs registered in Step 1"
    );

    assert!(
        detail_records[0].get("device_id").is_some(),
        "AC-2 (harness): host detail record must contain 'device_id' field"
    );
}

/// AC-2 (harness): Detection two-step pipeline returns summaries.
///
/// Migrated from: ac_2_two_step_pagination.rs::ac_2_detection_two_step_pipeline_returns_summaries
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_2_detection_two_step_pipeline_returns_summaries() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-test-session-ac2-det";
    let client = make_client();

    // Step 1: GET /detects/queries/detects/v1.
    let step1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-2 det: Step 1 request must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "AC-2 det (harness): Step 1 must be 200"
    );

    let step1_body: serde_json::Value = step1
        .json()
        .await
        .expect("AC-2 det: Step 1 body must be JSON");

    let det_ids = step1_body["resources"]
        .as_array()
        .expect("AC-2 det: resources must be array");
    assert!(
        !det_ids.is_empty(),
        "AC-2 det (harness): detection IDs must not be empty"
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

    assert_eq!(
        step2.status().as_u16(),
        200,
        "AC-2 det (harness): Step 2 must be 200"
    );

    let step2_body: serde_json::Value = step2
        .json()
        .await
        .expect("AC-2 det: Step 2 body must be JSON");

    let summaries = step2_body["resources"]
        .as_array()
        .expect("AC-2 det: Step 2 resources must be array");
    assert!(
        !summaries.is_empty(),
        "AC-2 det (harness): summaries must not be empty"
    );

    assert!(
        summaries[0].get("detection_id").is_some(),
        "AC-2 det (harness): detection summary must contain 'detection_id' field"
    );
}

/// AC-2 (harness): Different sessions are isolated — session_registry scopes pagination
/// state by session_id, not by OrgId (D-048; BC-3.2.003 invariant).
///
/// Migrated from: ac_2_two_step_pagination.rs::ac_2_different_sessions_are_isolated
/// (traces to BC-3.5.001 postcondition 1; BC-3.2.003)
#[tokio::test]
async fn test_BC_3_5_001_ac_2_different_sessions_are_isolated() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // Session A: Step 1 — registers host IDs under session-A.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "harness-session-A")
        .send()
        .await
        .expect("AC-2 iso: session A Step 1 must reach server");

    // Session B: Step 2 with IDs registered under session-A —
    // using a different X-DTU-Session-Id. The registry is keyed by session ID,
    // so Step 2 under session-B must return an empty resources array.
    let step2_b = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "harness-session-B")
        .send()
        .await
        .expect("AC-2 iso: session B Step 2 must reach server");

    assert_eq!(
        step2_b.status().as_u16(),
        200,
        "AC-2 iso (harness): cross-session Step 2 must return 200"
    );

    let body: serde_json::Value = step2_b.json().await.expect("AC-2 iso: body must be JSON");

    let resources = body["resources"]
        .as_array()
        .expect("AC-2 iso: resources must be array");
    assert!(
        resources.is_empty(),
        "AC-2 iso (harness): Step 2 under different session must return empty resources"
    );
}

// ============================================================================
// AC-3: contain write
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-3)
// ============================================================================

/// AC-3 (harness): Contain returns 202 with `contained` status.
///
/// Migrated from: ac_3_contain_write.rs::ac_3_contain_returns_202_with_contained_status
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_3_contain_returns_202_with_contained_status() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-3: contain request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "AC-3 (harness): contain must return HTTP 202"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-3: body must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("AC-3: resources must be an array");
    assert!(
        !resources.is_empty(),
        "AC-3 (harness): resources must not be empty"
    );

    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "AC-3 (harness): contain response must set containment_status to 'contained'"
    );
    assert_eq!(
        resources[0]["device_id"].as_str().unwrap_or(""),
        "h-001",
        "AC-3 (harness): contain response resources[0].device_id must be 'h-001'"
    );
}

/// AC-3 (harness): Contain persists to store; subsequent GET reflects updated status.
///
/// Migrated from: ac_3_contain_write.rs::ac_3_contain_persists_to_store_subsequent_get_reflects_status
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_3_contain_persists_to_store_subsequent_get_reflects_status() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-test-session-ac3-persist";
    let client = make_client();

    // Step 1: Register session.
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
        "AC-3 persist (harness): contain must return 202"
    );

    // GET /devices/entities/devices/v2?ids=h-001 — must reflect contained status.
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
        "AC-3 persist (harness): GET host detail must return 200"
    );

    let get_body: serde_json::Value = get_resp
        .json()
        .await
        .expect("AC-3 persist: GET body must be JSON");

    let records = get_body["resources"]
        .as_array()
        .expect("AC-3 persist: resources must be array");
    assert!(
        !records.is_empty(),
        "AC-3 persist (harness): resources must not be empty"
    );

    assert_eq!(
        records[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "AC-3 persist (harness): GET host detail must reflect containment_status 'contained'"
    );
}

/// AC-3 (harness): Lift containment returns 202 with `normal` status.
///
/// Migrated from: ac_3_contain_write.rs::ac_3_lift_containment_returns_202_with_normal_status
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_3_lift_containment_returns_202_with_normal_status() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

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
        "AC-3 lift (harness): lift_containment must return HTTP 202"
    );

    let lift_body: serde_json::Value = lift_resp
        .json()
        .await
        .expect("AC-3 lift: body must be JSON");

    let resources = lift_body["resources"]
        .as_array()
        .expect("AC-3 lift: resources must be array");
    assert!(
        !resources.is_empty(),
        "AC-3 lift (harness): resources must not be empty"
    );

    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "normal",
        "AC-3 lift (harness): lift_containment must set containment_status to 'normal'"
    );
}

// ============================================================================
// AC-4: rate limiting
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-4)
// ============================================================================

/// AC-4 (harness): Rate limit 429 on 4th request with Retry-After: 60.
///
/// Migrated from: ac_4_rate_limit.rs::ac_4_rate_limit_429_on_4th_request_with_retry_after_60
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_4_rate_limit_429_on_4th_request_with_retry_after_60() {
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::RateLimit {
            after_n_requests: 3,
            retry_after_secs: 60,
        },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // Requests 1–3 must succeed (within the rate limit window).
    for i in 1..=3u32 {
        let resp = client
            .get(format!("{base_url}/detects/queries/detects/v1"))
            .header("Authorization", "Bearer dtu-fake-cs-token")
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-4: request {i} must reach server"));

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-4 (harness): request {i} must return 200"
        );
    }

    // 4th request must receive HTTP 429.
    let resp4 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4: 4th request must reach server");

    assert_eq!(
        resp4.status().as_u16(),
        429,
        "AC-4 (harness): 4th request must return HTTP 429"
    );

    let retry_after = resp4
        .headers()
        .get("retry-after")
        .expect("AC-4: HTTP 429 must include Retry-After header");

    assert_eq!(
        retry_after.to_str().expect("Retry-After must be ASCII"),
        "60",
        "AC-4 (harness): Retry-After header value must be '60'"
    );
}

/// AC-4 (harness): Rate limit applies to all endpoints.
///
/// Migrated from: ac_4_rate_limit.rs::ac_4_rate_limit_applies_to_all_endpoints
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_4_rate_limit_applies_to_all_endpoints() {
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::RateLimit {
            after_n_requests: 2,
            retry_after_secs: 60,
        },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // Request 1 to detection endpoint.
    let r1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 1 must reach server");
    assert_eq!(
        r1.status().as_u16(),
        200,
        "AC-4 all (harness): request 1 must be 200"
    );

    // Request 2 to host endpoint (counter is shared across endpoints).
    let r2 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 2 must reach server");
    assert_eq!(
        r2.status().as_u16(),
        200,
        "AC-4 all (harness): request 2 must be 200"
    );

    // Request 3 to any endpoint — must be rate limited.
    let r3 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 3 must reach server");

    assert_eq!(
        r3.status().as_u16(),
        429,
        "AC-4 all (harness): 3rd request must return 429 (rate limit after_n=2 is shared)"
    );
}

/// AC-4 (harness): Retry-After header matches configured secs.
///
/// Migrated from: ac_4_rate_limit.rs::ac_4_retry_after_header_matches_configured_secs
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_4_retry_after_header_matches_configured_secs() {
    let retry_after_secs = 120u32;
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::RateLimit {
            after_n_requests: 1,
            retry_after_secs,
        },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // First request succeeds (after_n_requests = 1).
    client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 header: request 1 must reach server");

    // Second request must be rate limited with exact Retry-After value.
    let r2 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 header: request 2 must reach server");

    assert_eq!(
        r2.status().as_u16(),
        429,
        "AC-4 header (harness): must be 429"
    );

    let header_val = r2
        .headers()
        .get("retry-after")
        .expect("AC-4 header: Retry-After must be present")
        .to_str()
        .expect("Retry-After must be ASCII");

    assert_eq!(
        header_val,
        retry_after_secs.to_string().as_str(),
        "AC-4 header (harness): Retry-After must exactly match configured retry_after_secs={retry_after_secs}"
    );
}

// ============================================================================
// AC-5: OAuth token
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-5)
// ============================================================================

/// AC-5 (harness): OAuth token endpoint returns 200 with fake CrowdStrike token.
///
/// Migrated from: ac_5_oauth.rs::ac_5_oauth_token_returns_200_with_fake_cs_token
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_5_oauth_token_returns_200_with_fake_cs_token() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-client-id",
            "client_secret": "test-client-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5: oauth token request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-5 (harness): POST /oauth2/token must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: body must be valid JSON");

    assert_eq!(
        body["access_token"].as_str().unwrap_or(""),
        "dtu-fake-cs-token",
        "AC-5 (harness): access_token must be 'dtu-fake-cs-token'"
    );
    assert_eq!(
        body["token_type"].as_str().unwrap_or("").to_lowercase(),
        "bearer",
        "AC-5 (harness): token_type must be 'bearer'"
    );
    assert_eq!(
        body["expires_in"].as_u64().unwrap_or(0),
        3600,
        "AC-5 (harness): expires_in must be 3600"
    );
}

/// AC-5 (harness): Token obtained from OAuth works on authenticated endpoint.
///
/// Migrated from: ac_5_oauth.rs::ac_5_token_from_oauth_works_on_authenticated_endpoint
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_5_token_from_oauth_works_on_authenticated_endpoint() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // Get the token.
    let token_resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-id",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5 use: token request must reach server");

    assert_eq!(
        token_resp.status().as_u16(),
        200,
        "AC-5 use (harness): token must be 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("AC-5 use: token body must be JSON");
    let token = token_body["access_token"]
        .as_str()
        .expect("AC-5 use: access_token must be string");

    // Use the token on an authenticated endpoint.
    let auth_resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("AC-5 use: authenticated request must reach server");

    assert_eq!(
        auth_resp.status().as_u16(),
        200,
        "AC-5 use (harness): token obtained from /oauth2/token must authenticate on detection endpoint"
    );
}

/// AC-5 (harness): OAuth reject mode returns 401.
///
/// Migrated from: ac_5_oauth.rs::ac_5_oauth_reject_mode_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_5_oauth_reject_mode_returns_401() {
    let harness = build_cs_harness_with_failure("test-tenant", FailureMode::AuthReject).await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-id",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5 reject: token request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-5 reject (harness): AuthReject mode must return HTTP 401 from token endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5 reject: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("AC-5 reject: errors must be array");
    assert!(
        !errors.is_empty(),
        "AC-5 reject (harness): errors array must not be empty"
    );
    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        401,
        "AC-5 reject (harness): error code must be 401"
    );
}

// ============================================================================
// AC-6: determinism
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-6)
// ============================================================================

/// AC-6 (harness): Seed 42 detection query is deterministic across two builds.
///
/// Migrated from: ac_6_determinism.rs::ac_6_seed_42_detection_query_is_deterministic
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_6_seed_42_detection_query_is_deterministic() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    // First call.
    let resp1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("limit", "10"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6: first request must reach server");

    assert_eq!(
        resp1.status().as_u16(),
        200,
        "AC-6 (harness): first call must return 200"
    );

    let body1: serde_json::Value = resp1.json().await.expect("AC-6: first body must be JSON");

    // Second call with same params.
    let resp2 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("limit", "10"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6: second request must reach server");

    assert_eq!(
        resp2.status().as_u16(),
        200,
        "AC-6 (harness): second call must return 200"
    );

    let body2: serde_json::Value = resp2.json().await.expect("AC-6: second body must be JSON");

    assert_eq!(
        body1, body2,
        "AC-6 (harness): two calls with seed=42 and identical params must return identical responses"
    );
}

/// AC-6 (harness): Seed 42 host query is deterministic.
///
/// Migrated from: ac_6_determinism.rs::ac_6_seed_42_host_query_is_deterministic
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_6_seed_42_host_query_is_deterministic() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .query(&[("limit", "5"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 hosts: first request must reach server");

    let body1: serde_json::Value = resp1
        .json()
        .await
        .expect("AC-6 hosts: first body must be JSON");

    let resp2 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .query(&[("limit", "5"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 hosts: second request must reach server");

    let body2: serde_json::Value = resp2
        .json()
        .await
        .expect("AC-6 hosts: second body must be JSON");

    assert_eq!(
        body1, body2,
        "AC-6 hosts (harness): two calls with seed=42 and identical params must return identical responses"
    );
}

/// AC-6 (harness): Different seeds produce different responses.
///
/// Migrated from: ac_6_determinism.rs::ac_6_different_seeds_produce_different_responses
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_6_different_seeds_produce_different_responses() {
    // Build two harnesses with different seeds.
    let harness_42 = build_cs_harness_with_seed("org-seed-42", 42).await;
    let harness_99 = build_cs_harness_with_seed("org-seed-99", 99).await;

    let base_url_42 = cs_base_url(&harness_42, "org-seed-42");
    let base_url_99 = cs_base_url(&harness_99, "org-seed-99");

    let client = make_client();

    let resp_42 = client
        .get(format!("{base_url_42}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 seeds: harness_42 request must reach server");

    let resp_99 = client
        .get(format!("{base_url_99}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 seeds: harness_99 request must reach server");

    let body_42: serde_json::Value = resp_42
        .json()
        .await
        .expect("AC-6 seeds: body_42 must be JSON");
    let body_99: serde_json::Value = resp_99
        .json()
        .await
        .expect("AC-6 seeds: body_99 must be JSON");

    assert_ne!(
        body_42, body_99,
        "AC-6 seeds (harness): seed=42 and seed=99 must produce different responses"
    );
}

// ============================================================================
// AC-7: auth rejection
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-7)
// ============================================================================

/// Helper: assert 401 error body shape.
fn assert_401_error_body(body: &serde_json::Value, context: &str) {
    let errors = body["errors"].as_array().unwrap_or_else(|| {
        panic!("{context}: response body must contain 'errors' array, got: {body}")
    });
    assert!(
        !errors.is_empty(),
        "{context}: errors array must not be empty"
    );
    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        401,
        "{context}: error code must be 401"
    );
    assert!(
        errors[0]["message"].as_str().is_some(),
        "{context}: error message must be a string"
    );
}

/// AC-7 (harness): Detection list without auth returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_detection_list_without_auth_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_detection_list_without_auth_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        // No Authorization header.
        .send()
        .await
        .expect("AC-7 det: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 det (harness): missing Authorization must return HTTP 401 on detection list endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 det: body must be JSON");
    assert_401_error_body(&body, "AC-7 det (harness)");
}

/// AC-7 (harness): Detection summaries without auth returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_detection_summaries_without_auth_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_detection_summaries_without_auth_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("AC-7 summ: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 summ (harness): missing Authorization must return HTTP 401 on detection summaries endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 summ: body must be JSON");
    assert_401_error_body(&body, "AC-7 summ (harness)");
}

/// AC-7 (harness): Host list without auth returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_host_list_without_auth_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_host_list_without_auth_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .send()
        .await
        .expect("AC-7 host: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 host (harness): missing Authorization must return HTTP 401 on host list endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 host: body must be JSON");
    assert_401_error_body(&body, "AC-7 host (harness)");
}

/// AC-7 (harness): Host detail without auth returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_host_detail_without_auth_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_host_detail_without_auth_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .send()
        .await
        .expect("AC-7 hostd: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 hostd (harness): missing Authorization must return HTTP 401 on host detail endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 hostd: body must be JSON");
    assert_401_error_body(&body, "AC-7 hostd (harness)");
}

/// AC-7 (harness): Contain without auth returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_contain_without_auth_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_contain_without_auth_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-7 contain: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 contain (harness): missing Authorization must return HTTP 401 on contain endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 contain: body must be JSON");
    assert_401_error_body(&body, "AC-7 contain (harness)");
}

/// AC-7 (harness): Empty Authorization header returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_empty_authorization_header_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_empty_authorization_header_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "")
        .send()
        .await
        .expect("AC-7 empty: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 empty (harness): empty Authorization value must return HTTP 401"
    );
}

/// AC-7 (harness): Bearer with no token returns 401.
///
/// Migrated from: ac_7_auth.rs::ac_7_bearer_with_no_token_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_7_bearer_with_no_token_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .expect("AC-7 bare: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 bare (harness): 'Bearer ' with no token must return HTTP 401"
    );
}

// ============================================================================
// AC-8: reset
// (traces to BC-3.5.001 postcondition 1; S-6.07 AC-8)
// ============================================================================

/// AC-8 (harness): Reset clears containment store.
///
/// Migrated from: ac_8_reset.rs::ac_8_reset_clears_containment_store
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_8_reset_clears_containment_store() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-test-session-ac8";
    let client = make_client();

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
        "AC-8 (harness): contain must return 202 before reset"
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
        "AC-8 (harness): pre-reset containment_status must be 'contained'"
    );

    // POST /dtu/reset to clear state.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-8: POST /dtu/reset must reach server");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "AC-8 (harness): POST /dtu/reset must return 200"
    );

    // After reset, issue a fresh Step 1 with a new session ID, then Step 2 must
    // return base fixture state ("normal").
    let new_session_id = "harness-test-session-ac8-post-reset";
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
        "AC-8 (harness): post-reset GET must return 200"
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
        "AC-8 (harness): after reset, containment_status must return to 'normal' (base fixture state)"
    );
}

/// AC-8 (harness): Reset clears session registry.
///
/// Migrated from: ac_8_reset.rs::ac_8_reset_clears_session_registry
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_8_reset_clears_session_registry() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-test-session-ac8-registry";
    let client = make_client();

    // Step 1: register host IDs under the session.
    client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("AC-8 session: Step 1 must reach server");

    // POST /dtu/reset — clears the session registry.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-8 session: POST /dtu/reset must reach server");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "AC-8 session (harness): POST /dtu/reset must return 200"
    );

    // Step 2: request detail for IDs that were registered — registry is now empty.
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
        "AC-8 session (harness): Step 2 after reset must return 200"
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
        "AC-8 session (harness): after reset, Step 2 with previously-registered IDs must return empty resources"
    );
}

/// AC-8 (harness): Reset clears detection status store.
///
/// Migrated from: ac_8_reset.rs::ac_8_reset_clears_detection_status_store
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ac_8_reset_clears_detection_status_store() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

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
        "AC-8 det (harness): PATCH detection status must return 200"
    );

    // POST /dtu/reset.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-8 det: POST /dtu/reset must reach server");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "AC-8 det (harness): POST /dtu/reset must return 200"
    );

    // After reset, detection list must still return 200 (server is healthy).
    let post_reset_list = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-8 det: post-reset list must reach server");

    assert_eq!(
        post_reset_list.status().as_u16(),
        200,
        "AC-8 det (harness): after reset(), detection list must still return 200 (server is healthy)"
    );
}

// ============================================================================
// Integration test: VP-033
// (traces to BC-3.5.001 precondition 3; S-6.07 integration_vp033)
// ============================================================================

/// Integration (harness): Write intent before DTU arrival — harness-hosted clone.
///
/// Migrated from: integration_vp033.rs::crowdstrike_vp033_write_intent_before_dtu_arrival
/// (traces to BC-3.5.001 precondition 3; VP-033)
#[tokio::test]
#[ignore = "needs-prism-audit"]
async fn test_BC_3_5_001_integration_vp033_write_intent_before_dtu_arrival() {
    // This test requires prism-audit InMemoryBackend — blocked on S-3.07.
    // The harness provides the CrowdStrike clone; the audit + ordering assertions
    // are stubs until prism-audit lands.
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");

    // TODO(S-3.07): configure prism_audit::InMemoryBackend here.
    // TODO(S-3.07): configure SensorAdapter with base_url.
    // TODO(S-3.07): issue contain write through SensorAdapter.
    // TODO(S-3.07): assert WRITE_INTENT.committed_at < dtu_arrival_at < WRITE_OUTCOME.committed_at.

    let _ = base_url;

    panic!(
        "VP-033 (harness): test body is a stub — un-ignore after S-3.07 lands (prism-audit InMemoryBackend)"
    );
}

/// Integration (harness): Contain endpoint returns 202 smoke.
///
/// Migrated from: integration_vp033.rs::crowdstrike_vp033_contain_endpoint_returns_202_smoke
/// (traces to BC-3.5.001 precondition 3; VP-033)
#[tokio::test]
async fn test_BC_3_5_001_integration_vp033_contain_endpoint_returns_202_smoke() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("VP-033 smoke (harness): contain request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "VP-033 smoke (harness): contain must return HTTP 202"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("VP-033 smoke: body must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("VP-033 smoke: resources must be array");
    assert!(
        !resources.is_empty(),
        "VP-033 smoke (harness): resources must not be empty"
    );
    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "VP-033 smoke (harness): containment_status must be 'contained'"
    );
}

// ============================================================================
// Integration test: VP-036
// (traces to BC-3.5.001 precondition 3; S-6.07 integration_vp036)
// ============================================================================

/// Integration (harness): Session context drops before error.
///
/// Migrated from: integration_vp036.rs::crowdstrike_vp036_session_context_drops_before_error
/// (traces to BC-3.5.001 precondition 3; VP-036)
#[tokio::test]
#[ignore = "needs-prism-audit"]
async fn test_BC_3_5_001_integration_vp036_session_context_drops_before_error() {
    // This test requires prism-sensors::SessionContext — blocked on S-3.06.
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::InternalError { at_request_n: 2 },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");

    // TODO(S-3.06): configure SensorAdapter with base_url, capture session_weak_ref.
    // TODO(S-3.06): execute crowdstrike_hosts query, assert Arc::weak_count drops to 0
    //               before Err(E-SENSOR-002) is returned.

    let _ = base_url;

    panic!("VP-036 (harness): test body is a stub — un-ignore after S-3.06 lands (SensorAdapter + SessionContext)");
}

/// Integration (harness): Step2 returns 500 on internal error injection.
///
/// Migrated from: integration_vp036.rs::crowdstrike_vp036_step2_returns_500_on_internal_error_injection
/// (traces to BC-3.5.001 precondition 3; VP-036)
#[tokio::test]
async fn test_BC_3_5_001_integration_vp036_step2_returns_500_on_internal_error_injection() {
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::InternalError { at_request_n: 2 },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-vp036-smoke-session";
    let client = make_client();

    // Request 1 (Step 1): list host IDs — must succeed.
    let step1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("VP-036 smoke (harness): Step 1 request must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "VP-036 smoke (harness): Step 1 must return HTTP 200"
    );

    // Request 2 (Step 2): batch detail fetch — must return 500 per FailureMode.
    let step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("VP-036 smoke (harness): Step 2 request must reach server");

    assert_eq!(
        step2.status().as_u16(),
        500,
        "VP-036 smoke (harness): Step 2 must return HTTP 500 (FailureMode::InternalError at_request_n=2)"
    );
}

// ============================================================================
// Edge cases
// (traces to BC-3.5.001 postcondition 1; S-6.07 edge_cases)
// ============================================================================

/// EC-001 (harness): Contain with empty IDs returns 400.
///
/// Migrated from: edge_cases.rs::ec_001_contain_empty_ids_returns_400
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_001_contain_empty_ids_returns_400() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

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
        "EC-001 (harness): contain with empty ids must return HTTP 400"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-001: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("EC-001: errors must be array");
    assert!(
        !errors.is_empty(),
        "EC-001 (harness): errors array must not be empty"
    );

    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        400,
        "EC-001 (harness): error code must be 400"
    );

    let msg = errors[0]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("empty") || msg.contains("ids"),
        "EC-001 (harness): error message must mention 'ids' or 'empty', got: {msg:?}"
    );
}

/// EC-001 (harness): Lift containment with empty IDs returns 400.
///
/// Migrated from: edge_cases.rs::ec_001_lift_containment_empty_ids_returns_400
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_001_lift_containment_empty_ids_returns_400() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

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
        "EC-001 lift (harness): lift_containment with empty ids must return HTTP 400"
    );
}

/// EC-002 (harness): Contain already-contained host returns 400.
///
/// Migrated from: edge_cases.rs::ec_002_contain_already_contained_returns_400
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_002_contain_already_contained_returns_400() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

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
        "EC-002 (harness): first contain must return 202"
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
        "EC-002 (harness): contain on already-contained device must return HTTP 400"
    );

    let body: serde_json::Value = second.json().await.expect("EC-002: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("EC-002: errors must be array");
    assert!(
        !errors.is_empty(),
        "EC-002 (harness): errors array must not be empty"
    );

    let msg = errors[0]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("already contained"),
        "EC-002 (harness): error must say 'device already contained', got: {msg:?}"
    );
}

/// EC-003 (harness): Step2 with unknown IDs returns 200 empty.
///
/// Migrated from: edge_cases.rs::ec_003_step2_unknown_ids_returns_200_empty
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_003_step2_unknown_ids_returns_200_empty() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "does-not-exist-001"), ("ids", "does-not-exist-002")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "harness-ec003-orphan-session")
        .send()
        .await
        .expect("EC-003: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003 (harness): Step 2 with unknown IDs must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-003: body must be JSON");

    let resources = body["resources"]
        .as_array()
        .expect("EC-003: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-003 (harness): Step 2 with unknown IDs must return empty resources array"
    );
}

/// EC-003 (harness): Detection step2 with unknown IDs returns 200 empty.
///
/// Migrated from: edge_cases.rs::ec_003_detection_step2_unknown_ids_returns_200_empty
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_003_detection_step2_unknown_ids_returns_200_empty() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "harness-ec003-orphan-det-session")
        .json(&serde_json::json!({"ids": ["not-registered-001"]}))
        .send()
        .await
        .expect("EC-003 det: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003 det (harness): detection Step 2 with unknown IDs must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-003 det: body must be JSON");
    let resources = body["resources"]
        .as_array()
        .expect("EC-003 det: resources must be array");
    assert!(
        resources.is_empty(),
        "EC-003 det (harness): unknown detection IDs must return empty resources"
    );
}

/// EC-004 (harness): LRU eviction at 1000 sessions does not panic.
///
/// Migrated from: edge_cases.rs::ec_004_lru_eviction_at_1000_sessions_no_panic
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_004_lru_eviction_at_1000_sessions_no_panic() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("reqwest client must build");

    // Fill the registry to capacity (1,000 unique sessions).
    for i in 0..1000usize {
        let session_id = format!("harness-ec004-session-{i:04}");
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
            "EC-004 (harness): session {i} registration must return 200"
        );
    }

    // Register session 1,001 — this must evict session-0 (LRU policy).
    let overflow_session = "harness-ec004-overflow-1001";
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
        "EC-004 (harness): overflow session registration must return 200 (no error on eviction)"
    );

    // Verify: session-0 was evicted — Step 2 under that session returns empty resources.
    let evicted_step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", "harness-ec004-session-0000")
        .send()
        .await
        .expect("EC-004: evicted session Step 2 must reach server");

    assert_eq!(
        evicted_step2.status().as_u16(),
        200,
        "EC-004 (harness): Step 2 for evicted session must return 200"
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
        "EC-004 (harness): evicted session's Step 2 must return empty resources (LRU evicted)"
    );
}

/// EC-005 (harness): Mid-pagination 500 on step2 batch2.
///
/// Migrated from: edge_cases.rs::ec_005_mid_pagination_500_on_step2_batch2
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_005_mid_pagination_500_on_step2_batch2() {
    let harness = build_cs_harness_with_failure(
        "test-tenant",
        FailureMode::InternalError { at_request_n: 3 },
    )
    .await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-ec005-session";
    let client = make_client();

    // Request 1: Step 1 (detection list) — succeeds.
    let step1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("EC-005 (harness): request 1 (Step 1) must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "EC-005 (harness): request 1 must succeed (Step 1 detection list)"
    );

    // Request 2: Step 2 batch 1 — succeeds.
    let batch1 = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("EC-005 (harness): request 2 (Step 2 batch 1) must reach server");

    assert_eq!(
        batch1.status().as_u16(),
        200,
        "EC-005 (harness): request 2 must succeed (Step 2 batch 1)"
    );

    // Request 3: Step 2 batch 2 — must return 500 (FailureMode injection).
    let batch2 = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .json(&serde_json::json!({"ids": ["det-002"]}))
        .send()
        .await
        .expect("EC-005 (harness): request 3 (Step 2 batch 2) must reach server");

    assert_eq!(
        batch2.status().as_u16(),
        500,
        "EC-005 (harness): request 3 must return HTTP 500 (FailureMode::InternalError at_request_n=3)"
    );
}

/// EC-006 (harness): Reset during active query returns empty, no panic.
///
/// Migrated from: edge_cases.rs::ec_006_reset_during_active_query_returns_empty_no_panic
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_ec_006_reset_during_active_query_returns_empty_no_panic() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let session_id = "harness-ec006-active-session";
    let client = make_client();

    // Step 1: register IDs in the session registry.
    let step1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("EC-006: Step 1 must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "EC-006 (harness): Step 1 must succeed"
    );

    // POST /dtu/reset between Step 1 and Step 2 (simulates mid-query reset).
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("EC-006: POST /dtu/reset must reach server");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "EC-006 (harness): POST /dtu/reset must return 200"
    );

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
        "EC-006 (harness): Step 2 after reset must return 200 (cleared state is not an error)"
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
        "EC-006 (harness): Step 2 after reset must return empty resources (session cleared)"
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
        "EC-006 (harness): server must still be healthy after reset() during active query"
    );
}

// ============================================================================
// TD tests via harness
// (traces to BC-3.5.001 postcondition 1)
// ============================================================================

/// TD-WV0-04 (harness): Configure known field returns 200.
///
/// Migrated from: td_wv0_04_configure_deny_unknown.rs::configure_known_field_returns_200
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_td_wv0_04_configure_known_field_returns_200() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let admin_token = harness
        .admin_token_for("test-tenant", DtuType::CrowdStrike)
        .expect("admin token must be present")
        .to_string();
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"auth_mode": "reject"}))
        .send()
        .await
        .expect("TD-WV0-04 (harness): request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "TD-WV0-04 (harness): known field must return 200"
    );
}

/// TD-WV0-04 (harness): Configure unknown field returns 400.
///
/// Migrated from: td_wv0_04_configure_deny_unknown.rs::configure_unknown_field_returns_400
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_td_wv0_04_configure_unknown_field_returns_400() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let admin_token = harness
        .admin_token_for("test-tenant", DtuType::CrowdStrike)
        .expect("admin token must be present")
        .to_string();
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"bogus": "val"}))
        .send()
        .await
        .expect("TD-WV0-04 (harness): request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "TD-WV0-04 (harness): unknown field must return 400 Bad Request, not silently accept"
    );
}

/// TD-WV0-07 (harness): Configure without token returns 401.
///
/// Migrated from: td_wv0_07_configure_requires_admin_token.rs::configure_without_token_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_td_wv0_07_configure_without_token_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({"auth_mode": "accept"}))
        .send()
        .await
        .expect("TD-WV0-07 (harness): request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "TD-WV0-07 (harness): missing X-Admin-Token must return 401"
    );
}

/// TD-WV0-07 (harness): Configure with wrong token returns 401.
///
/// Migrated from: td_wv0_07_configure_requires_admin_token.rs::configure_with_wrong_token_returns_401
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_td_wv0_07_configure_with_wrong_token_returns_401() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", "wrong-token-that-will-never-match")
        .json(&serde_json::json!({"auth_mode": "accept"}))
        .send()
        .await
        .expect("TD-WV0-07 (harness): request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "TD-WV0-07 (harness): incorrect X-Admin-Token must return 401"
    );
}

/// TD-WV0-07 (harness): Configure with correct token returns 200.
///
/// Migrated from: td_wv0_07_configure_requires_admin_token.rs::configure_with_correct_token_returns_200
/// (traces to BC-3.5.001 postcondition 1)
#[tokio::test]
async fn test_BC_3_5_001_td_wv0_07_configure_with_correct_token_returns_200() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");
    let admin_token = harness
        .admin_token_for("test-tenant", DtuType::CrowdStrike)
        .expect("admin token must be present")
        .to_string();
    let client = make_client();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"auth_mode": "accept"}))
        .send()
        .await
        .expect("TD-WV0-07 (harness): request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "TD-WV0-07 (harness): correct X-Admin-Token must return 200"
    );
}

// ============================================================================
// Fidelity validator via harness
// (traces to BC-3.5.001 precondition 3; AC-004)
// ============================================================================

/// AC-004: Fidelity validator reports `checks_failed == 0` for all CrowdStrike
/// endpoints when the clone is hosted by the harness.
///
/// The base URL for the fidelity validator must come from `harness.endpoints()`
/// (Previous Story Intelligence from S-3.4.01).
///
/// (traces to BC-3.5.001 precondition 3; AC-004)
#[tokio::test]
async fn test_BC_3_5_001_fidelity_validator_checks_failed_zero() {
    let harness = build_cs_harness("test-tenant").await;
    let base_url = cs_base_url(&harness, "test-tenant");

    let checks = vec![
        // Endpoint 1: OAuth2 token (unauthenticated by design).
        FidelityCheck {
            endpoint: "/oauth2/token".to_owned(),
            method: http::Method::POST,
            body: Some(serde_json::json!({
                "client_id": "fidelity-test",
                "client_secret": "fidelity-secret",
                "grant_type": "client_credentials"
            })),
            expected_status: 200,
            required_fields: vec!["access_token".to_owned()],
            ..Default::default()
        },
        // Endpoint 2: DTU health (introspection — no auth required).
        FidelityCheck {
            endpoint: "/dtu/health".to_owned(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_owned()],
            ..Default::default()
        },
        // Endpoint 3: DTU reset (introspection — no auth required).
        FidelityCheck {
            endpoint: "/dtu/reset".to_owned(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_owned()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    assert_eq!(
        report.checks_failed, 0,
        "fidelity (harness): {} of 3 endpoint check(s) failed:\n{:#?}",
        report.checks_failed, report.failures
    );
    assert_eq!(
        report.checks_passed, 3,
        "fidelity (harness): expected 3 checks passed, got {}",
        report.checks_passed
    );
}

// ============================================================================
// AC-005: 2-org logical isolation — pairwise-disjoint device sets
// (traces to BC-3.5.001 postcondition 2; TV-2; story AC-005)
// ============================================================================

/// AC-005: A 2-org logical harness returns pairwise-disjoint device sets.
///
/// Given: Two customer orgs (org_a, org_b) registered with distinct seeds in
/// `IsolationMode::Logical`.
/// When: Device ID sets are queried for each org via `GET /devices/queries/devices/v1`.
/// Then: `devices(org_a) ∩ devices(org_b) = ∅` (BC-3.5.001 postcondition 2; TV-2).
///
/// (BC-3.5.001 postcondition 2; VP-122; VP-123; AC-005)
#[tokio::test]
async fn test_BC_3_5_001_ac_multi_org_logical_isolation() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("org-a", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(42);
        })
        .with_customer_overrides("org-b", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(99);
        })
        .build()
        .await
        .expect("2-org CrowdStrike logical harness must build");

    let client = make_client();

    // Fetch detection IDs for org_a.
    let addr_a = harness
        .endpoint_for("org-a", DtuType::CrowdStrike)
        .expect("org-a CrowdStrike endpoint must exist");
    let resp_a = client
        .get(format!("http://{addr_a}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-005: org_a detection list request must reach server");

    assert_eq!(
        resp_a.status().as_u16(),
        200,
        "AC-005: org_a detection list must return 200"
    );
    let body_a: serde_json::Value = resp_a
        .json()
        .await
        .expect("AC-005: org_a body must be JSON");
    let ids_a: std::collections::HashSet<String> = body_a["resources"]
        .as_array()
        .expect("AC-005: org_a resources must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    // Fetch detection IDs for org_b.
    let addr_b = harness
        .endpoint_for("org-b", DtuType::CrowdStrike)
        .expect("org-b CrowdStrike endpoint must exist");
    let resp_b = client
        .get(format!("http://{addr_b}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-005: org_b detection list request must reach server");

    assert_eq!(
        resp_b.status().as_u16(),
        200,
        "AC-005: org_b detection list must return 200"
    );
    let body_b: serde_json::Value = resp_b
        .json()
        .await
        .expect("AC-005: org_b body must be JSON");
    let ids_b: std::collections::HashSet<String> = body_b["resources"]
        .as_array()
        .expect("AC-005: org_b resources must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !ids_a.is_empty(),
        "AC-005: org_a detection IDs must not be empty"
    );
    assert!(
        !ids_b.is_empty(),
        "AC-005: org_b detection IDs must not be empty"
    );

    // Assert pairwise disjoint: devices(org_a) ∩ devices(org_b) = ∅
    // (BC-3.5.001 postcondition 2; TV-2)
    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "AC-005: org_a and org_b detection ID sets must be pairwise disjoint \
         (BC-3.5.001 postcondition 2; TV-2); intersection = {intersection:?}"
    );

    // Additionally verify host IDs from the same two orgs are disjoint.
    let host_resp_a = client
        .get(format!("http://{addr_a}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-005: org_a host list request must reach server");
    let host_body_a: serde_json::Value = host_resp_a
        .json()
        .await
        .expect("AC-005: org_a host body must be JSON");
    let host_ids_a: std::collections::HashSet<String> = host_body_a["resources"]
        .as_array()
        .expect("AC-005: org_a host resources must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    let host_resp_b = client
        .get(format!("http://{addr_b}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-005: org_b host list request must reach server");
    let host_body_b: serde_json::Value = host_resp_b
        .json()
        .await
        .expect("AC-005: org_b host body must be JSON");
    let host_ids_b: std::collections::HashSet<String> = host_body_b["resources"]
        .as_array()
        .expect("AC-005: org_b host resources must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    let host_intersection: std::collections::HashSet<&String> =
        host_ids_a.intersection(&host_ids_b).collect();
    assert!(
        host_intersection.is_empty(),
        "AC-005: org_a and org_b host ID sets must be pairwise disjoint \
         (BC-3.5.001 postcondition 2; detection_store + containment_store keyed by (OrgId, String)); \
         intersection = {host_intersection:?}"
    );
}

// ============================================================================
// AC-006: Network isolation — cross-org credential mismatch → HTTP 401
// (traces to BC-3.5.002 postcondition 2; TV-3; story AC-006)
// ============================================================================

/// AC-006: A 2-org network harness — cross-org credential mismatch — returns HTTP 401.
///
/// Given: Two customer orgs (org_a, org_b) registered in `IsolationMode::Network`.
/// When: A request bearing `org_a`'s OAuth token is routed to `org_b`'s endpoint.
/// Then: The response is HTTP 401 (BC-3.5.002 postcondition 2; TV-3).
///
/// This verifies that routing bugs are observable (not silently returning wrong data).
///
/// (BC-3.5.002 postcondition 2; VP-125; VP-126; AC-006)
#[tokio::test]
async fn test_BC_3_5_002_ac_network_cross_creds_401() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("org-a", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(42);
        })
        .with_customer_overrides("org-b", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(99);
        })
        .build()
        .await
        .expect("2-org CrowdStrike network harness must build");

    let client = make_client();

    // Obtain org_a's admin token (this is the "OAuth token" / credential for org_a's clone).
    let token_org_a = harness
        .admin_token_for("org-a", DtuType::CrowdStrike)
        .expect("org_a admin token must be present")
        .to_string();

    // Get org_b's endpoint address.
    let addr_org_b = harness
        .endpoint_for("org-b", DtuType::CrowdStrike)
        .expect("org_b CrowdStrike endpoint must exist");

    // Route a request bearing org_a's Bearer token to org_b's endpoint.
    // BC-3.5.002 postcondition 2: this must return HTTP 401 (wrong token for this clone).
    let resp = client
        .get(format!("http://{addr_org_b}/devices/queries/devices/v1"))
        .header("Authorization", format!("Bearer {token_org_a}"))
        .send()
        .await
        .expect("AC-006: cross-org credential request must reach org_b's endpoint");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-006: org_a's Bearer token sent to org_b's endpoint must return HTTP 401 \
         (BC-3.5.002 postcondition 2; TV-3; VP-126) — routing bugs must be observable"
    );
}

// ============================================================================
// Session registry per-org isolation (BC-3.2.003 / D-048)
// (traces to BC-3.5.001; BC-3.2.003 invariant; story AC-session-registry)
// ============================================================================

/// Session registry per-org isolation: session_id from query engine OrgId-scoped
/// bytes does NOT bleed across org boundaries.
///
/// This test validates D-048: the session_registry is keyed by bare String (session_id),
/// not by (OrgId, String). The session_id itself carries OrgId-scoped bytes (XOR UUID v7
/// embedding per S-3.2.08), so sessions from org_a are structurally distinct from
/// org_b's sessions. This test exercises that property end-to-end through the harness:
///
/// Given: Two orgs in a logical harness, each performing step-1 pagination (session
///        ID registration) with OrgId-scoped session_ids.
/// When:  Org_a's session_id is sent in a step-2 request routed to org_b's endpoint.
/// Then:  Org_b returns 200 with empty resources (session not found in its registry),
///        not org_a's data — the bytes-keyed session_id does not match any entry in
///        org_b's LRU cache.
///
/// (BC-3.2.003; BC-3.5.001 postcondition 2; D-048; VP-123; AC-session-registry)
#[tokio::test]
async fn test_BC_3_2_003_ac_session_registry_per_org_isolation() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("org-a", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(42);
        })
        .with_customer_overrides("org-b", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed_override = Some(99);
        })
        .build()
        .await
        .expect("2-org CrowdStrike logical harness must build for session registry isolation test");

    let client = make_client();

    // Construct an OrgId-scoped session_id for org_a.
    // In production (S-3.2.08), this would be a UUID v7 with XOR-embedded OrgId bytes.
    // For the harness test, we use a synthetic session_id that would only be generated
    // by org_a's query engine — structurally distinct from any org_b session_id.
    let org_a_session_id = "org-a-xor-uuid-v7-session-0000001a";

    let addr_org_a = harness
        .endpoint_for("org-a", DtuType::CrowdStrike)
        .expect("org_a CrowdStrike endpoint must exist");
    let addr_org_b = harness
        .endpoint_for("org-b", DtuType::CrowdStrike)
        .expect("org_b CrowdStrike endpoint must exist");

    // Step 1 on org_a: POST step-1 pagination, registering the org_a session_id.
    let step1_org_a = client
        .get(format!("http://{addr_org_a}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", org_a_session_id)
        .send()
        .await
        .expect("BC-3.2.003: org_a Step 1 must reach server");

    assert_eq!(
        step1_org_a.status().as_u16(),
        200,
        "BC-3.2.003: org_a Step 1 must return 200"
    );

    let step1_body: serde_json::Value = step1_org_a
        .json()
        .await
        .expect("BC-3.2.003: org_a Step 1 body must be JSON");
    let org_a_host_ids = step1_body["resources"]
        .as_array()
        .expect("BC-3.2.003: org_a Step 1 resources must be an array");
    assert!(
        !org_a_host_ids.is_empty(),
        "BC-3.2.003: org_a Step 1 must return at least one host ID (fixture must be non-empty)"
    );

    let org_a_first_host_id = org_a_host_ids[0]
        .as_str()
        .expect("BC-3.2.003: org_a host IDs must be strings");

    // Step 2 on org_b: send org_a's session_id to org_b's endpoint.
    // Per D-048, the session_registry is keyed by bare String. org_a's session_id
    // was registered in org_a's registry, not org_b's. The bytes-keyed string
    // "org-a-xor-uuid-v7-session-0000001a" does not appear in org_b's LRU cache,
    // so org_b must return 200 with empty resources (session not found).
    //
    // This is the structural separation property — NOT a 401 auth error.
    // The test exercises D-048 (session_registry NOT re-keyed by (OrgId, String)).
    let step2_on_org_b = client
        .get(format!("http://{addr_org_b}/devices/entities/devices/v2"))
        .query(&[("ids", org_a_first_host_id)])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", org_a_session_id) // org_a's session_id routed to org_b
        .send()
        .await
        .expect("BC-3.2.003: cross-org step-2 request must reach org_b's server");

    assert_eq!(
        step2_on_org_b.status().as_u16(),
        200,
        "BC-3.2.003: org_b Step 2 with org_a's session_id must return 200 \
         (session not found = empty resources, not an auth error; D-048)"
    );

    let step2_body: serde_json::Value = step2_on_org_b
        .json()
        .await
        .expect("BC-3.2.003: org_b step-2 response body must be JSON");

    let cross_org_resources = step2_body["resources"]
        .as_array()
        .expect("BC-3.2.003: org_b step-2 resources must be an array");

    assert!(
        cross_org_resources.is_empty(),
        "BC-3.2.003: org_b Step 2 with org_a's session_id must return empty resources \
         (session_id bytes from org_a do not match any entry in org_b's LRU registry; \
         D-048 structural separation — bytes-keyed session_id does not bleed across orgs; \
         VP-123)"
    );
}
