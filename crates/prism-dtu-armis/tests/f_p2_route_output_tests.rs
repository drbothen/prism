//! Route-output tests for F-P2-CRIT-001 and F-P2-CRIT-002 closures.
//!
//! # What these tests assert
//!
//! These tests drive the ACTUAL ROUTE via an in-process HTTP server started by
//! `ArmisClone::new_with_seed`. They assert on ROUTE RESPONSES at the adapter's
//! real path_template, not on raw `state.generated_records`.
//!
//! This is the root-cause fix for the recurring miss pattern identified in Pass-2:
//! raw-state assertions (inspecting `state.generated_records`) do NOT catch routing
//! bugs where the handler serves the wrong path (static fixture vs generated path).
//!
//! # Tests in this file
//!
//! | Test | Finding | What it proves |
//! |------|---------|----------------|
//! | test_f_p2_crit_001_search_devices_generated_route_output | F-P2-CRIT-001 | /api/v1/search?aql=in:devices serves generated device records |
//! | test_f_p2_crit_001_search_alerts_generated_route_output | F-P2-CRIT-001 | /api/v1/search?aql=in:alerts serves generated alert records |
//! | test_f_p2_crit_001_search_devices_disjoint_across_seeds | F-P2-CRIT-001 + INV-DISTINCT-DATA-001 | two seeds → disjoint device results at /api/v1/search |
//! | test_f_p2_crit_001_search_alerts_disjoint_across_seeds | F-P2-CRIT-001 + INV-DISTINCT-DATA-001 | two seeds → disjoint alert results at /api/v1/search |
//! | test_f_p2_crit_002_devices_endpoint_generated_route_output | F-P2-CRIT-002 | /api/v1/devices serves generated device records (non-empty) |
//! | test_f_p2_crit_002_devices_disjoint_across_seeds | F-P2-CRIT-002 + INV-DISTINCT-DATA-001 | two seeds → disjoint device results at /api/v1/devices |
//!
//! # Red-proof
//!
//! Tests `test_f_p2_crit_001_search_devices_generated_route_output` and
//! `test_f_p2_crit_002_devices_endpoint_generated_route_output` were RED-PROOFED
//! by temporarily reverting the fix (serving static fixture when generated non-empty)
//! and confirming both tests FAILED before restoring the correct implementation.
//! Results documented in the fix-burst commit message.
//!
//! # Feature gate
//!
//! All tests require `fixture-gen` + `dtu` features (in-process HTTP server requires
//! `dtu`; `new_with_seed` requires `fixture-gen`).

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Canonical test org: bytes [0xde, 0xad, 0xbe, 0xef, ...].
fn deadbeef_org() -> prism_dtu_common::OrgId {
    prism_dtu_common::OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Second test org: bytes [0xca, 0xfe, 0xba, 0xbe, ...] for disjoint-ID tests.
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
// F-P2-CRIT-001: /api/v1/search device path serves generated records
// ---------------------------------------------------------------------------

/// F-P2-CRIT-001: GET /api/v1/search?aql=in:devices on a seeded clone returns
/// generated device records in `$.data.results`, NOT the static fixture.
///
/// RED-PROOFED: verified this test FAILS when the search handler serves the
/// static fixture instead of generated records when generated_records is non-empty.
///
/// Asserts:
/// - HTTP 200
/// - `data.results` is non-empty
/// - First record has `asset_id` containing the seed-specific prefix (generated shape)
/// - `data.total` > 0
#[tokio::test]
async fn test_f_p2_crit_001_search_devices_generated_route_output() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:devices")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P2-CRIT-001: GET /api/v1/search?aql=in:devices on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let results = body["data"]["results"]
        .as_array()
        .expect("F-P2-CRIT-001: $.data.results must be an array");

    assert!(
        !results.is_empty(),
        "F-P2-CRIT-001: $.data.results must be non-empty for generated clone with in:devices AQL; \
         got empty array — handler is likely serving static fixture (F-P2-CRIT-001 not closed)"
    );

    // Generated device records have "asset_id" with the seed-specific prefix.
    // Static fixture records have "device_id" (snake_case). This assertion is the
    // load-bearing probe that distinguishes generated-path from static-fixture-path.
    let expected_prefix = format!("dev-{slug}-{seed}-");
    let first = &results[0];
    let asset_id = first["asset_id"].as_str().expect(
        "F-P2-CRIT-001: first result must have 'asset_id' field (generated record shape); \
             static fixture records have 'device_id' — dual-path not wired to search handler",
    );
    assert!(
        asset_id.starts_with(&expected_prefix),
        "F-P2-CRIT-001: asset_id '{asset_id}' must start with '{expected_prefix}' \
         (generated record ID format dev-{{slug}}-{{seed}}-N)"
    );

    let total = body["data"]["total"]
        .as_u64()
        .expect("F-P2-CRIT-001: data.total must be a number");
    assert!(
        total > 0,
        "F-P2-CRIT-001: data.total must be > 0 for generated device records"
    );
}

/// F-P2-CRIT-001: GET /api/v1/search?aql=in:alerts on a seeded clone returns
/// generated alert records in `$.data.results`, NOT the static fixture.
///
/// Asserts:
/// - HTTP 200
/// - `data.results` is non-empty
/// - First record has `alert_id` containing the seed-specific prefix (generated shape)
#[tokio::test]
async fn test_f_p2_crit_001_search_alerts_generated_route_output() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:alerts")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:alerts must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P2-CRIT-001: GET /api/v1/search?aql=in:alerts on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let results = body["data"]["results"]
        .as_array()
        .expect("F-P2-CRIT-001: $.data.results must be an array for in:alerts");

    assert!(
        !results.is_empty(),
        "F-P2-CRIT-001: $.data.results must be non-empty for generated clone with in:alerts AQL; \
         got empty array — handler is serving static fixture (CompromisedEndpoint archetype has alerts)"
    );

    // Generated alert records have "alert_id" with the seed-specific prefix.
    // This is the discriminator used by the dual-path in get_search.
    let expected_alert_prefix = format!("alert-{slug}-{seed}-");
    let first = &results[0];
    let alert_id = first["alert_id"].as_str().expect(
        "F-P2-CRIT-001: first result must have 'alert_id' field (generated alert record shape)",
    );
    assert!(
        alert_id.starts_with(&expected_alert_prefix),
        "F-P2-CRIT-001: alert_id '{alert_id}' must start with '{expected_alert_prefix}' \
         (generated alert ID format alert-{{slug}}-{{seed}}-N)"
    );
}

/// F-P2-CRIT-001 + INV-DISTINCT-DATA-001:
/// Two seeded clones serve DISJOINT device IDs at /api/v1/search?aql=in:devices.
///
/// This test exercises the real route path (not raw state) and asserts that
/// INV-DISTINCT-DATA-001 holds where it matters — the federated adapter path.
#[tokio::test]
async fn test_f_p2_crit_001_search_devices_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    // Canonical 3-arg form: org_slug derived internally (ADR-036 v2.2).
    let mut clone_a = ArmisClone::new_with_seed(
        100,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_a.clone(),
    )
    .expect("new_with_seed(100) must succeed");
    let mut clone_b = ArmisClone::new_with_seed(
        200,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_b.clone(),
    )
    .expect("new_with_seed(200) must succeed");

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/api/v1/search"))
                .query(&[("aql", "in:devices")])
                .header("Authorization", "Bearer test-token")
                .send()
                .await
                .expect("search request must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["data"]["results"]
                .as_array()
                .expect("results must be array")
                .iter()
                .filter_map(|r| r["asset_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch(base_a).await;
    let ids_b = fetch(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a must serve non-empty device results"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b must serve non-empty device results"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output): asset_ids from clone_a (seed=100, slug={slug_a}) \
         and clone_b (seed=200, slug={slug_b}) must be DISJOINT at /api/v1/search; \
         intersection={intersection:?}"
    );
}

/// F-P2-CRIT-001 + INV-DISTINCT-DATA-001:
/// Two seeded clones serve DISJOINT alert IDs at /api/v1/search?aql=in:alerts.
#[tokio::test]
async fn test_f_p2_crit_001_search_alerts_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    // Canonical 3-arg form: org_slug derived internally (ADR-036 v2.2).
    let mut clone_a = ArmisClone::new_with_seed(
        100,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_a.clone(),
    )
    .expect("new_with_seed(100) must succeed");
    let mut clone_b = ArmisClone::new_with_seed(
        200,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_b.clone(),
    )
    .expect("new_with_seed(200) must succeed");

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_alerts = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/api/v1/search"))
                .query(&[("aql", "in:alerts")])
                .header("Authorization", "Bearer test-token")
                .send()
                .await
                .expect("search request must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["data"]["results"]
                .as_array()
                .expect("results must be array")
                .iter()
                .filter_map(|r| r["alert_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_alerts(base_a).await;
    let ids_b = fetch_alerts(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a must serve non-empty alert results"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b must serve non-empty alert results"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output): alert_ids from clone_a (seed=100, slug={slug_a}) \
         and clone_b (seed=200, slug={slug_b}) must be DISJOINT at /api/v1/search?aql=in:alerts; \
         intersection={intersection:?}"
    );
}

// ---------------------------------------------------------------------------
// F-P2-CRIT-002: /api/v1/devices serves generated records (non-empty, correct shape)
// ---------------------------------------------------------------------------

/// F-P2-CRIT-002: GET /api/v1/devices on a seeded clone returns generated device
/// records in `$.data.devices`, NOT an empty array from silent deserialization failure.
///
/// RED-PROOFED: verified this test FAILS when the handler uses the old
/// `serde_json::from_value::<DeviceRecord>(v).ok()` pattern (which silently drops
/// all generated records because they use camelCase field names).
///
/// Asserts:
/// - HTTP 200
/// - `data.devices` is non-empty
/// - First record has `asset_id` containing the seed-specific prefix (generated shape)
/// - `data.total` > 0 (was 0 before the fix — the P2 bug report symptom)
#[tokio::test]
async fn test_f_p2_crit_002_devices_endpoint_generated_route_output() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/devices must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P2-CRIT-002: GET /api/v1/devices on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let total = body["data"]["total"].as_u64().expect(
        "F-P2-CRIT-002: $.data.total must be a number; \
             before the fix, silent serde_json deserialization drops all records and total=0",
    );
    assert!(
        total > 0,
        "F-P2-CRIT-002: $.data.total must be > 0 for generated clone; \
         got total=0 — this is the exact symptom of the silent .ok()-drop bug (F-P2-CRIT-002). \
         The fix: serve generated records as raw serde_json::Value, no DeviceRecord deserialization."
    );

    let devices = body["data"]["devices"]
        .as_array()
        .expect("F-P2-CRIT-002: $.data.devices must be an array");

    assert!(
        !devices.is_empty(),
        "F-P2-CRIT-002: $.data.devices must be non-empty for generated clone"
    );

    // Generated device records have "asset_id" with the seed-specific prefix.
    // The old broken path would return DeviceRecord structs (with "device_id") if any
    // deserialization succeeded — or empty if all failed (the actual bug).
    let expected_prefix = format!("dev-{slug}-{seed}-");
    let first = &devices[0];
    let asset_id = first["asset_id"].as_str().expect(
        "F-P2-CRIT-002: first device must have 'asset_id' field (generated record shape); \
             the fix serves raw Values — camelCase fields are preserved",
    );
    assert!(
        asset_id.starts_with(&expected_prefix),
        "F-P2-CRIT-002: asset_id '{asset_id}' must start with '{expected_prefix}' \
         (generated device ID format dev-{{slug}}-{{seed}}-N)"
    );
}

/// F-P2-CRIT-002 + INV-DISTINCT-DATA-001:
/// Two seeded clones serve DISJOINT device IDs at /api/v1/devices.
///
/// This exercises the real route path (not raw state) and proves that
/// INV-DISTINCT-DATA-001 holds for the back-compat /api/v1/devices endpoint.
#[tokio::test]
async fn test_f_p2_crit_002_devices_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    // Canonical 3-arg form: org_slug derived internally (ADR-036 v2.2).
    let mut clone_a = ArmisClone::new_with_seed(
        100,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_a.clone(),
    )
    .expect("new_with_seed(100) must succeed");
    let mut clone_b = ArmisClone::new_with_seed(
        200,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_b.clone(),
    )
    .expect("new_with_seed(200) must succeed");

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_devices = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/api/v1/devices"))
                .header("Authorization", "Bearer test-token")
                .send()
                .await
                .expect("devices request must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["data"]["devices"]
                .as_array()
                .expect("devices must be array")
                .iter()
                .filter_map(|r| r["asset_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_devices(base_a).await;
    let ids_b = fetch_devices(base_b).await;

    assert!(!ids_a.is_empty(), "clone_a must serve non-empty devices");
    assert!(!ids_b.is_empty(), "clone_b must serve non-empty devices");

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output): asset_ids from clone_a (seed=100, slug={slug_a}) \
         and clone_b (seed=200, slug={slug_b}) must be DISJOINT at /api/v1/devices; \
         intersection={intersection:?}"
    );
}

// ---------------------------------------------------------------------------
// F-P4-MED-001: Surface-purity assertions for Armis routes
//
// CompromisedEndpoint baseline (scale=1.0):
//   device records: 50 — discriminator: asset_id present, alert_id absent
//   alert records:  20 — discriminator: alert_id present
//   total generated_records: 70
//
// These tests assert:
//   (a) ONLY device-surface records appear in /api/v1/search?aql=in:devices
//   (b) ONLY device-surface records appear in /api/v1/devices
//   (c) ONLY alert-surface records appear in /api/v1/search?aql=in:alerts
//   (d) Exact expected counts (no foreign-surface leakage)
// ---------------------------------------------------------------------------

/// F-P4-MED-001 (Armis): GET /api/v1/search?aql=in:devices serves ONLY device-surface
/// records — zero alert records in the response.
///
/// Armis CompromisedEndpoint (scale=1.0): 50 device records + 20 alert records = 70 total.
/// The `in:devices` route must serve exactly 50 device records (no alert records).
///
/// Asserts:
/// - `data.total` == 50
/// - Every record in `data.results` has NO `alert_id` field
/// - Every record in `data.results` HAS an `asset_id` field
#[tokio::test]
async fn test_f_p4_med_001_armis_search_devices_surface_purity() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:devices")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");
    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let results = body["data"]["results"]
        .as_array()
        .expect("F-P4-MED-001: $.data.results must be an array");

    let total = body["data"]["total"]
        .as_u64()
        .expect("F-P4-MED-001: $.data.total must be a number");

    assert_eq!(
        total, 50,
        "F-P4-MED-001 (Armis search devices): $.data.total must be exactly 50 \
         (device-surface count for CompromisedEndpoint at scale=1.0); got {total}"
    );

    // (a) Surface purity: EVERY record must NOT have alert_id.
    for (i, record) in results.iter().enumerate() {
        assert!(
            record.get("alert_id").is_none(),
            "F-P4-MED-001 (Armis search devices): $.data.results[{i}] has 'alert_id' — \
             alert record leaked into the in:devices response. \
             discriminator: asset_id present AND alert_id absent."
        );
    }

    // (a) Surface purity: EVERY record must have asset_id.
    for (i, record) in results.iter().enumerate() {
        assert!(
            record.get("asset_id").is_some(),
            "F-P4-MED-001 (Armis search devices): $.data.results[{i}] missing 'asset_id'"
        );
    }
}

/// F-P4-MED-001 (Armis): GET /api/v1/search?aql=in:alerts serves ONLY alert-surface
/// records — zero device records in the response.
///
/// CompromisedEndpoint (scale=1.0): 20 alert records at /api/v1/search?aql=in:alerts.
///
/// Asserts:
/// - `data.total` == 20
/// - Every record in `data.results` HAS an `alert_id` field
/// - NO record has no `alert_id` (device records absent)
#[tokio::test]
async fn test_f_p4_med_001_armis_search_alerts_surface_purity() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:alerts")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:alerts must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");
    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let results = body["data"]["results"]
        .as_array()
        .expect("F-P4-MED-001: $.data.results must be an array");

    let total = body["data"]["total"]
        .as_u64()
        .expect("F-P4-MED-001: $.data.total must be a number");

    assert_eq!(
        total, 20,
        "F-P4-MED-001 (Armis search alerts): $.data.total must be exactly 20 \
         (alert-surface count for CompromisedEndpoint at scale=1.0); got {total}"
    );

    // (c) Surface purity: EVERY record must have alert_id.
    for (i, record) in results.iter().enumerate() {
        assert!(
            record.get("alert_id").is_some(),
            "F-P4-MED-001 (Armis search alerts): $.data.results[{i}] missing 'alert_id' — \
             not an alert record. \
             discriminator: alert_id present."
        );
    }
}

/// F-P4-MED-001 (Armis): GET /api/v1/devices serves ONLY device-surface records.
///
/// CompromisedEndpoint (scale=1.0): 50 device records at /api/v1/devices.
///
/// Asserts:
/// - `data.total` == 50
/// - Every record in `data.devices` has NO `alert_id` field
/// - Every record in `data.devices` HAS an `asset_id` field
#[tokio::test]
async fn test_f_p4_med_001_armis_devices_endpoint_surface_purity() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    )
    .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/devices must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");
    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let devices = body["data"]["devices"]
        .as_array()
        .expect("F-P4-MED-001: $.data.devices must be an array");

    let total = body["data"]["total"]
        .as_u64()
        .expect("F-P4-MED-001: $.data.total must be a number");

    assert_eq!(
        total, 50,
        "F-P4-MED-001 (Armis /api/v1/devices): $.data.total must be exactly 50 \
         (device-surface count for CompromisedEndpoint at scale=1.0); got {total}"
    );

    // (b) Surface purity: EVERY device record must NOT have alert_id.
    for (i, record) in devices.iter().enumerate() {
        assert!(
            record.get("alert_id").is_none(),
            "F-P4-MED-001 (Armis /api/v1/devices): $.data.devices[{i}] has 'alert_id' — \
             alert record leaked into the /api/v1/devices response. \
             discriminator: asset_id present AND alert_id absent."
        );
    }

    // (b) Surface purity: EVERY device record must have asset_id.
    for (i, record) in devices.iter().enumerate() {
        assert!(
            record.get("asset_id").is_some(),
            "F-P4-MED-001 (Armis /api/v1/devices): $.data.devices[{i}] missing 'asset_id'"
        );
    }
}
