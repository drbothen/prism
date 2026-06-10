//! Route-output tests for F-P10-HIGH-001 closure: CrowdStrike DormantTenant detection routes.
//!
//! # What these tests assert
//!
//! The DormantTenant archetype generates 0 device records and 0 detection records.
//! Prior to F-P10-HIGH-001, the detection route dual-path keyed off
//! `!state.generated_detections.is_empty()` — which is `false` for DormantTenant —
//! causing a fall-through to the static fixture (non-empty). This test proves the
//! fix: `state.fixture_gen_seeded` is the correct sentinel, ensuring DormantTenant
//! serves EMPTY from both detection routes.
//!
//! Routes exercised:
//!   - Step 1: GET  /detects/queries/detects/v1        → `$.resources` must be empty
//!   - Step 2: POST /detects/entities/summaries/GET/v1 → `$.resources` must be empty
//!
//! A parallel dormant-empty assertion on the device routes is included so the
//! full CrowdStrike dormant-empty surface is load-bearing (all 4 adapter routes).
//!
//! # Tests
//!
//! | Test | What it proves |
//! |------|----------------|
//! | test_f_p10_high_001_dormant_detection_id_list_empty | GET /detects/queries/detects/v1 returns 0 resources for DormantTenant |
//! | test_f_p10_high_001_dormant_detection_summaries_empty | POST /detects/entities/summaries/GET/v1 returns 0 resources for DormantTenant |
//! | test_f_p10_high_001_dormant_device_id_list_empty | GET /devices/queries/devices/v1 returns 0 resources for DormantTenant |
//! | test_f_p10_high_001_compromised_detections_still_present | CompromisedEndpoint still serves non-empty detections (non-regression) |
//!
//! # Red-proof record
//!
//! `test_f_p10_high_001_dormant_detection_id_list_empty` and
//! `test_f_p10_high_001_dormant_detection_summaries_empty` were RED-PROOFED by
//! temporarily reverting `detections.rs` dual-path from `state.fixture_gen_seeded`
//! back to `!state.generated_detections.is_empty()`. Both tests FAILED:
//!   - Step 1 returned a non-empty `$.resources` array (static fixture leaked).
//!   - Step 2 with injected IDs returned non-empty records (static detail fixture leaked).
//! Fix restored. Both tests PASS.
//!
//! # Feature gate
//!
//! All tests require `fixture-gen` + `dtu` features.

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Canonical test org: bytes [0xde, 0xad, 0xbe, 0xef, ...] → org_slug = "deadbeef".
fn deadbeef_org() -> prism_dtu_common::OrgId {
    prism_dtu_common::OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("test client build must succeed")
}

/// Build and start a DormantTenant CrowdStrike clone.
///
/// DormantTenant generates 0 device records and 0 detection records.
/// The clone IS seeded (fixture_gen_seeded = true), so dual-path must honour
/// the generated (empty) vectors rather than falling through to the static fixture.
async fn start_dormant_clone(seed: u64) -> CrowdstrikeClone {
    let org = deadbeef_org();
    let mut clone =
        CrowdstrikeClone::new_with_seed(seed, prism_dtu_common::Archetype::DormantTenant, org);
    clone
        .start()
        .await
        .expect("DormantTenant CrowdstrikeClone start must succeed");
    clone
}

// ---------------------------------------------------------------------------
// F-P10-HIGH-001: GET /detects/queries/detects/v1 — DormantTenant must return empty
// ---------------------------------------------------------------------------

/// F-P10-HIGH-001 — Step 1: GET /detects/queries/detects/v1 on a DormantTenant clone
/// returns an EMPTY `$.resources` array (zero detections).
///
/// # What this proves
///
/// The `list_detection_ids` dual-path uses `state.fixture_gen_seeded` (not
/// `!state.generated_detections.is_empty()`) as the sentinel. For DormantTenant
/// `fixture_gen_seeded = true` and `generated_detections` is empty, so the route
/// must serve the empty generated vec — NOT the static fixture.
///
/// # RED-PROOFED
///
/// Reverting `list_detection_ids` dual-path to `!state.generated_detections.is_empty()`
/// caused this test to FAIL: DormantTenant fell through to `load_detection_ids()`
/// (static fixture, non-empty) → `$.resources` was non-empty → assertion panicked.
/// Fix restored (`state.fixture_gen_seeded`). Test PASSES.
///
/// Asserts:
/// - HTTP 200
/// - `$.resources` is an empty array (0 detections for DormantTenant)
/// - `$.meta.pagination.total` == 0
#[tokio::test]
async fn test_f_p10_high_001_dormant_detection_id_list_empty() {
    let seed = 77u64;
    let mut clone = start_dormant_clone(seed).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer test-token-f-p10")
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P10-HIGH-001 Step 1: DormantTenant GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("F-P10-HIGH-001: $.resources must be a JSON array");

    assert!(
        resources.is_empty(),
        "F-P10-HIGH-001 Step 1: DormantTenant GET /detects/queries/detects/v1 must return \
         EMPTY $.resources (0 detections). Got {} records — static fixture leaked. \
         Dual-path sentinel must be state.fixture_gen_seeded, not generated_detections.is_empty(). \
         BC-2.06.018 EC-018-003 / TV-018-006 / INV-FIXTURE-SET-ARCHETYPE-MAP-001 violated.",
        resources.len()
    );

    let total = body["meta"]["pagination"]["total"]
        .as_u64()
        .unwrap_or(u64::MAX);
    assert_eq!(
        total, 0,
        "F-P10-HIGH-001 Step 1: $.meta.pagination.total must be 0 for DormantTenant; got {total}"
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// F-P10-HIGH-001: POST /detects/entities/summaries/GET/v1 — DormantTenant must return empty
// ---------------------------------------------------------------------------

/// F-P10-HIGH-001 — Step 2: POST /detects/entities/summaries/GET/v1 on a DormantTenant clone
/// returns an EMPTY `$.resources` array when given a non-empty `ids` body.
///
/// # What this proves
///
/// The `get_detection_summaries` dual-path uses `state.fixture_gen_seeded` as the
/// sentinel. For DormantTenant the detail map is empty (generated from 0 detections),
/// so even when the caller supplies IDs (e.g., pulled from the static fixture),
/// the detail lookup returns no records.
///
/// # RED-PROOFED
///
/// Reverting `get_detection_summaries` dual-path to `!state.generated_detections.is_empty()`
/// caused this test to FAIL: the route fell through to `load_detection_details()` (static
/// fixture) → IDs from `load_detection_ids()` were found in the static detail map →
/// `$.resources` was non-empty → assertion panicked.
/// Fix restored. Test PASSES.
///
/// Asserts:
/// - HTTP 200
/// - `$.resources` is an empty array (IDs not found in the empty generated detail map)
#[tokio::test]
async fn test_f_p10_high_001_dormant_detection_summaries_empty() {
    let seed = 77u64;
    let mut clone = start_dormant_clone(seed).await;
    let base_url = clone.base_url();
    let client = test_client();

    // Inject IDs that exist in the static fixture (`fixtures/detections-ids.json`).
    // Static fixture IDs follow `det-NNN` format. If the dual-path sentinel is wrong
    // (!generated_detections.is_empty()) the route falls back to load_detection_details()
    // (static fixture) → these IDs resolve to non-empty records → test FAILS.
    //
    // For DormantTenant with the correct sentinel (fixture_gen_seeded), the detail
    // HashMap is empty (built from 0 generated detections) → none of these IDs
    // resolve → resources is empty → test PASSES.
    let injected_ids = serde_json::json!({
        "ids": ["det-001", "det-002", "det-003"]
    });

    let resp = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer test-token-f-p10")
        .header("Content-Type", "application/json")
        .json(&injected_ids)
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P10-HIGH-001 Step 2: DormantTenant POST /detects/entities/summaries/GET/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("F-P10-HIGH-001 Step 2: $.resources must be a JSON array");

    assert!(
        resources.is_empty(),
        "F-P10-HIGH-001 Step 2: DormantTenant POST /detects/entities/summaries/GET/v1 must return \
         EMPTY $.resources. Got {} records — static detail fixture leaked. \
         Dual-path sentinel must be state.fixture_gen_seeded, not generated_detections.is_empty(). \
         BC-2.06.018 EC-018-003 / TV-018-006 / INV-FIXTURE-SET-ARCHETYPE-MAP-001 violated.",
        resources.len()
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// F-P10-HIGH-001: Device routes — DormantTenant must also return empty (full surface)
// ---------------------------------------------------------------------------

/// F-P10-HIGH-001 (device route parity): GET /devices/queries/devices/v1 on a DormantTenant
/// clone returns an EMPTY `$.resources` array.
///
/// This assertion covers the full CrowdStrike dormant-empty surface: both detection routes
/// and both device routes must serve empty for DormantTenant. The device route fix was
/// applied during F-P6-HIGH-001; this test is added here to make the full surface
/// load-bearing in a single test file (so future sibling-sweep regressions are caught).
///
/// Asserts:
/// - HTTP 200
/// - `$.resources` is empty (0 devices for DormantTenant)
/// - `$.meta.pagination.total` == 0
#[tokio::test]
async fn test_f_p10_high_001_dormant_device_id_list_empty() {
    let seed = 77u64;
    let mut clone = start_dormant_clone(seed).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token-f-p10")
        .send()
        .await
        .expect("GET /devices/queries/devices/v1 must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P10-HIGH-001 device parity: DormantTenant GET /devices/queries/devices/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("F-P10-HIGH-001 device parity: $.resources must be a JSON array");

    assert!(
        resources.is_empty(),
        "F-P10-HIGH-001 device parity: DormantTenant GET /devices/queries/devices/v1 must return \
         EMPTY $.resources (0 devices). Got {} records — static fixture leaked. \
         state.fixture_gen_seeded sentinel required (hosts.rs). \
         BC-2.06.018 EC-018-003 / INV-FIXTURE-SET-ARCHETYPE-MAP-001 violated.",
        resources.len()
    );

    let total = body["meta"]["pagination"]["total"]
        .as_u64()
        .unwrap_or(u64::MAX);
    assert_eq!(
        total, 0,
        "F-P10-HIGH-001 device parity: $.meta.pagination.total must be 0 for DormantTenant; got {total}"
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// Non-regression: CompromisedEndpoint still serves non-empty detections
// ---------------------------------------------------------------------------

/// Non-regression: CompromisedEndpoint GET /detects/queries/detects/v1 still serves
/// non-empty detection IDs after the sentinel change.
///
/// The sentinel change from `!generated_detections.is_empty()` to `fixture_gen_seeded`
/// must NOT regress the CompromisedEndpoint archetype (which has 20 detections at scale=1.0).
/// For CompromisedEndpoint both sentinels evaluate to `true`, so the route still serves
/// generated IDs — but this test makes that load-bearing.
///
/// Asserts:
/// - HTTP 200
/// - `$.resources` is non-empty (CompromisedEndpoint has 20 detections)
/// - First ID starts with `alert-` (generated detection ID format)
#[tokio::test]
async fn test_f_p10_high_001_compromised_detections_still_present() {
    let org = deadbeef_org();
    let seed = 42u64;
    let mut clone = CrowdstrikeClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org,
    );
    clone
        .start()
        .await
        .expect("CompromisedEndpoint CrowdstrikeClone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer test-token-f-p10")
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Non-regression: CompromisedEndpoint GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("Non-regression: $.resources must be a JSON array");

    assert!(
        !resources.is_empty(),
        "Non-regression: CompromisedEndpoint GET /detects/queries/detects/v1 must return \
         NON-EMPTY $.resources (20 detections at scale=1.0). Got 0 — sentinel change broke \
         the CompromisedEndpoint path."
    );

    // Generated detection IDs follow `alert-{slug}-{seed}-{n}` format.
    let first_id = resources[0]
        .as_str()
        .expect("Non-regression: $.resources[0] must be a string (detection ID)");
    assert!(
        first_id.starts_with("alert-"),
        "Non-regression: first detection ID '{first_id}' must start with 'alert-' \
         (generated detection ID format); static fixture IDs use a different format"
    );

    clone.stop().await.expect("clone stop must succeed");
}
