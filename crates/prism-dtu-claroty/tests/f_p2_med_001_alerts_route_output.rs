//! Route-output tests for F-P2-MED-001 closure: Claroty alerts dual-path.
//!
//! # What these tests assert
//!
//! These tests drive the ACTUAL ROUTE via an in-process HTTP server started by
//! `ClarotyClone::new_with_seed`. They assert on ROUTE RESPONSES at the adapter's
//! real path_template (`POST /api/v1/alerts/`), not on raw `state.generated_records`.
//!
//! # Finding
//!
//! F-P2-MED-001: Claroty `list_alerts` had no generated_records branch.
//! A seeded clone would serve the static fixture alerts regardless of the seeded data.
//! Fix: added dual-path to `list_alerts` in `routes/alerts.rs`.
//!
//! # Tests
//!
//! | Test | What it proves |
//! |------|----------------|
//! | test_f_p2_med_001_claroty_alerts_generated_route_output | POST /api/v1/alerts/ serves generated alert records |
//! | test_f_p2_med_001_claroty_alerts_disjoint_across_seeds | two seeds → disjoint alert results at /api/v1/alerts/ |
//! | test_f_p2_med_001_claroty_devices_already_wired_still_works | regression: devices dual-path still works |
//!
//! # Feature gate
//!
//! All tests require `fixture-gen` + `dtu` features.

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn deadbeef_org() -> prism_dtu_common::OrgId {
    prism_dtu_common::OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

fn cafebabe_org() -> prism_dtu_common::OrgId {
    prism_dtu_common::OrgId([
        0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

fn org_slug(org: &prism_dtu_common::OrgId) -> String {
    let b = org.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("test client build must succeed")
}

// ---------------------------------------------------------------------------
// F-P2-MED-001: POST /api/v1/alerts/ serves generated records
// ---------------------------------------------------------------------------

/// F-P2-MED-001: POST /api/v1/alerts/ on a seeded Claroty clone returns
/// generated alert records in `$.alerts`, NOT the static fixture.
///
/// The Claroty generator emits alert records with `alert_id` field
/// (generator.rs: `"alert_id": format!("alert-{slug}-{seed}-{index}")`).
///
/// Asserts:
/// - HTTP 200
/// - `alerts` is non-empty
/// - First record has `alert_id` containing the seed-specific prefix
/// - `total` > 0
#[tokio::test]
async fn test_f_p2_med_001_claroty_alerts_generated_route_output() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ClarotyClone::new_with_seed(seed, org.clone());
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/alerts/ must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P2-MED-001: POST /api/v1/alerts/ on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let alerts = body["alerts"]
        .as_array()
        .expect("F-P2-MED-001: $.alerts must be an array");

    assert!(
        !alerts.is_empty(),
        "F-P2-MED-001: $.alerts must be non-empty for generated clone; \
         got empty array — handler is serving static fixture (dual-path not wired)"
    );

    // Generated Claroty alert records have "alert_id" with the seed-specific prefix.
    // Static fixture records have different IDs (from fixtures/alerts.json).
    let expected_prefix = format!("alert-{slug}-{seed}-");
    let first = &alerts[0];
    let alert_id = first["alert_id"].as_str().expect(
        "F-P2-MED-001: first alert must have 'alert_id' field (generated Claroty alert shape)",
    );
    assert!(
        alert_id.starts_with(&expected_prefix),
        "F-P2-MED-001: alert_id '{alert_id}' must start with '{expected_prefix}' \
         (generated Claroty alert ID format alert-{{slug}}-{{seed}}-N)"
    );

    let total = body["total"]
        .as_u64()
        .expect("F-P2-MED-001: $.total must be a number");
    assert!(
        total > 0,
        "F-P2-MED-001: $.total must be > 0 for generated alert records"
    );
}

/// F-P2-MED-001 + INV-DISTINCT-DATA-001:
/// Two seeded Claroty clones serve DISJOINT alert IDs at POST /api/v1/alerts/.
#[tokio::test]
async fn test_f_p2_med_001_claroty_alerts_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    let mut clone_a = ClarotyClone::new_with_seed(100, org_a.clone());
    let mut clone_b = ClarotyClone::new_with_seed(200, org_b.clone());

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_alerts = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .post(format!("{base}/api/v1/alerts/"))
                .header("Authorization", "Bearer test-token")
                .json(&serde_json::json!({}))
                .send()
                .await
                .expect("alerts request must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["alerts"]
                .as_array()
                .expect("alerts must be array")
                .iter()
                .filter_map(|r| r["alert_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_alerts(base_a).await;
    let ids_b = fetch_alerts(base_b).await;

    assert!(!ids_a.is_empty(), "clone_a must serve non-empty alerts");
    assert!(!ids_b.is_empty(), "clone_b must serve non-empty alerts");

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output Claroty alerts): alert_ids from \
         clone_a (seed=100, slug={slug_a}) and clone_b (seed=200, slug={slug_b}) \
         must be DISJOINT at POST /api/v1/alerts/; intersection={intersection:?}"
    );
}

/// Regression: Claroty devices dual-path still works after alerts dual-path was added.
///
/// The devices dual-path was already wired (CRIT-001 from a prior pass). This test
/// ensures the alerts-dual-path addition didn't accidentally break the devices path.
#[tokio::test]
async fn test_f_p2_med_001_claroty_devices_already_wired_still_works() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ClarotyClone::new_with_seed(seed, org.clone());
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/devices/ must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "regression: POST /api/v1/devices/ on seeded clone must still return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    // Claroty devices dual-path serves records with "asset_id" (generator emits asset_id).
    // The Claroty generator uses "ASSET-{slug}-{seed}-{n}" format (different from Armis).
    let devices = body["devices"]
        .as_array()
        .expect("$.devices must be an array");

    assert!(
        !devices.is_empty(),
        "regression: $.devices must be non-empty for generated clone (devices dual-path still wired)"
    );

    // Claroty generator (generator.rs) uses "ASSET-{slug}-{seed}-{n}" format for asset_id.
    let expected_prefix = format!("ASSET-{slug}-{seed}-");
    let first = &devices[0];
    let asset_id = first["asset_id"]
        .as_str()
        .expect("first device must have asset_id (generated shape)");
    assert!(
        asset_id.starts_with(&expected_prefix),
        "regression: device asset_id '{asset_id}' must start with '{expected_prefix}' \
         (Claroty generator uses ASSET-{{slug}}-{{seed}}-N format)"
    );
}
