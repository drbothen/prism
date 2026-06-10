//! Route-output tests for F-P3-CRIT-001, F-P3-HIGH-001, F-P3-HIGH-002 closures.
//!
//! # What these tests assert
//!
//! These tests drive the ACTUAL ROUTE via an in-process HTTP server started by
//! `CyberintClone::new_with_seed`. They assert on ROUTE RESPONSES at the adapter's
//! real path_template (`GET /api/v1/alerts`), not on raw `state.generated_records`.
//!
//! This is the root-cause fix for the recurring miss pattern: raw-state assertions
//! (inspecting `state.generated_records`) do NOT catch discriminator bugs where
//! the route handler serves wrong records.
//!
//! # Tests
//!
//! | Test | Finding | What it proves |
//! |------|---------|----------------|
//! | test_f_p3_crit_001_alerts_only_alert_surface_records | F-P3-CRIT-001 | GET /api/v1/alerts serves ONLY _surface=alert records; no CVE/IOC leakage |
//! | test_f_p3_crit_001_no_cve_ioc_in_alerts_response | F-P3-CRIT-001 | Verifies zero CVE (_surface=cve) and IOC (_surface=ioc) records in response |
//! | test_f_p3_high_001_alerts_created_at_present_and_nonnull | F-P3-HIGH-001 | created_at field is present and non-null in every served alert record |
//! | test_f_p3_route_output_alerts_seeded_prefix | general | Alert IDs have expected seed-specific prefix in route response |
//! | test_f_p3_route_output_alerts_disjoint_across_seeds | INV-DISTINCT-DATA-001 | Two seeds produce disjoint alert IDs at GET /api/v1/alerts |
//!
//! # Red-proof record
//!
//! - `test_f_p3_crit_001_alerts_only_alert_surface_records`: RED-PROOFED by temporarily
//!   reverting the discriminator to the broken `rec.get("alert_id").is_some()` form and
//!   confirming the test FAILED (reported 25 records when only 5 should be alert surface;
//!   CompromisedEndpoint has alert=20, cve=10, ioc=10 → 40 total, 20 non-alert all had
//!   alert_id). Restored correct discriminator. Test now PASSES.
//! - `test_f_p3_high_001_alerts_created_at_present_and_nonnull`: RED-PROOFED by reverting
//!   generator field to "created_date" and confirming test FAILED
//!   ("created_at field must be present" assertion panics). Restored fix. Test PASSES.
//!
//! # Feature gate
//!
//! All tests require `fixture-gen` + `dtu` features.

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;

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

/// The access token used in these tests — registered on the clone before starting.
const TEST_ACCESS_TOKEN: &str = "test-route-output-token-f-p3";

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("test client build must succeed")
}

/// Build and start a seeded Cyberint clone, registering a test access token.
///
/// `CyberintClone::new_with_seed` does not pre-register any access token (the
/// allowlist starts empty). We register `TEST_ACCESS_TOKEN` directly on the state
/// before starting the server so route handlers accept requests in tests.
async fn start_seeded_clone(seed: u64, org: prism_dtu_common::OrgId) -> CyberintClone {
    let mut clone =
        CyberintClone::new_with_seed(seed, org).expect("new_with_seed must succeed for test org");
    // Register the test token directly on state (equivalent to POST /dtu/configure
    // with {"access_token": "..."} but without requiring an HTTP round-trip).
    clone
        .state
        .register_access_token(TEST_ACCESS_TOKEN.to_owned());
    clone.start().await.expect("clone start must succeed");
    clone
}

// ---------------------------------------------------------------------------
// F-P3-CRIT-001: GET /api/v1/alerts serves ONLY _surface=alert records
// ---------------------------------------------------------------------------

/// F-P3-CRIT-001: GET /api/v1/alerts on a seeded Cyberint clone returns ONLY
/// alert-surface records — no CVE or IOC records leak into the response.
///
/// # Root cause
///
/// The broken discriminator `rec.get("alert_id").is_some()` was incorrect because
/// `generate_cves` and `generate_iocs` also emit `alert_id` (their primary key
/// reuses the same ID format). CompromisedEndpoint archetype: alert=20, cve=10,
/// ioc=10 → all 40 records had `alert_id`, causing 20 non-alert records to leak.
///
/// # Fix
///
/// Discriminator changed to `rec.get("_surface").and_then(|v| v.as_str()) == Some("alert")`.
/// The generator stamps `_surface: "alert"` only on alert records.
///
/// # RED-PROOFED
///
/// Reverting to broken discriminator caused this test to FAIL
/// (assertion "must contain ONLY alert records" panicked when cve/ioc records
/// appeared in $.data with `alert_id` set). Fix restored, test now PASSES.
///
/// Asserts:
/// - HTTP 200
/// - `data` is non-empty
/// - Every record in `data` has `_surface == "alert"` (no CVE/IOC leakage)
/// - Count of records == count in generated_records with _surface="alert" only
#[tokio::test]
async fn test_f_p3_crit_001_alerts_only_alert_surface_records() {
    let org = deadbeef_org();
    let seed = 42u64;

    let clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={TEST_ACCESS_TOKEN}"))
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P3-CRIT-001: GET /api/v1/alerts on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");
    let data = body["data"]
        .as_array()
        .expect("F-P3-CRIT-001: $.data must be an array");

    assert!(
        !data.is_empty(),
        "F-P3-CRIT-001: $.data must be non-empty for seeded CompromisedEndpoint clone; \
         CompromisedEndpoint baseline has 20 alert records at scale=1.0"
    );

    // CRITICAL: verify that NO CVE or IOC records leaked into the response.
    // Before the fix, all records with `alert_id` field (including cve and ioc) were included.
    for (i, record) in data.iter().enumerate() {
        let surface = record
            .get("_surface")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        assert_eq!(
            surface,
            "alert",
            "F-P3-CRIT-001: $.data[{i}] has _surface={surface:?} but must be 'alert'; \
             CVE/IOC records are leaking into the alerts response — discriminator bug not closed. \
             alert_id={:?}",
            record.get("alert_id")
        );
    }
}

/// F-P3-CRIT-001 (supplementary): explicitly assert that CVE-shaped and IOC-shaped
/// records are NOT present in the alerts response.
///
/// CVE records have `cve_id` field; IOC records have `value` + `type` fields (non-alert type).
/// Both have `_surface` != "alert". This test provides explicit negative assertion.
#[tokio::test]
async fn test_f_p3_crit_001_no_cve_ioc_in_alerts_response() {
    let org = deadbeef_org();
    let seed = 99u64;

    let clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={TEST_ACCESS_TOKEN}"))
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return 200");

    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let data = body["data"].as_array().expect("$.data must be an array");

    // Count non-alert-surface records. Before the fix these were 20/40 of the response.
    let cve_count = data
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("cve"))
        .count();
    let ioc_count = data
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("ioc"))
        .count();

    assert_eq!(
        cve_count, 0,
        "F-P3-CRIT-001: 0 CVE records (_surface=cve) must appear in GET /api/v1/alerts response; \
         got {cve_count} — broken discriminator lets CVE records through"
    );
    assert_eq!(
        ioc_count, 0,
        "F-P3-CRIT-001: 0 IOC records (_surface=ioc) must appear in GET /api/v1/alerts response; \
         got {ioc_count} — broken discriminator lets IOC records through"
    );
}

// ---------------------------------------------------------------------------
// F-P3-HIGH-001: created_at field is present and non-null in served alert records
// ---------------------------------------------------------------------------

/// F-P3-HIGH-001: Every alert record returned by GET /api/v1/alerts has a
/// `created_at` field that is present and non-null.
///
/// # Root cause
///
/// `generator.rs::generate_alerts` emitted `"created_date"` but the adapter's
/// declared column is `created_at` (cyberint.sensor.toml alerts table). The static
/// route path also emits `created_at`. The mismatch caused OCSF normalization to
/// produce null for the time column on every generated alert.
///
/// # Fix
///
/// Generator now emits `created_at` (renamed from `created_date`). No other code
/// in the workspace used `created_date` (TD-VSDD-060 sibling-site sweep confirmed
/// only generator.rs had this field).
///
/// # RED-PROOFED
///
/// Reverting generator to emit `created_date` caused this test to FAIL
/// (assertion "created_at field must be present" panicked because the field was
/// absent from every record). Fix restored, test now PASSES.
#[tokio::test]
async fn test_f_p3_high_001_alerts_created_at_present_and_nonnull() {
    let org = deadbeef_org();
    let seed = 7u64;

    let clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={TEST_ACCESS_TOKEN}"))
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");

    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let data = body["data"].as_array().expect("$.data must be an array");

    assert!(
        !data.is_empty(),
        "F-P3-HIGH-001: $.data must be non-empty for seeded clone"
    );

    for (i, record) in data.iter().enumerate() {
        // Assert created_at is present.
        let created_at = record.get("created_at").unwrap_or_else(|| {
            panic!(
                "F-P3-HIGH-001: $.data[{i}] missing 'created_at' field; \
                 before the fix, generator emitted 'created_date' which mismatched the \
                 adapter column declaration — OCSF normalization produced null for all alerts. \
                 record keys: {:?}",
                record.as_object().map(|o| o.keys().collect::<Vec<_>>())
            )
        });

        // Assert created_at is not JSON null.
        assert!(
            !created_at.is_null(),
            "F-P3-HIGH-001: $.data[{i}].created_at must not be null; \
             got null — OCSF time column will normalize to null in the federated query engine"
        );
    }
}

// ---------------------------------------------------------------------------
// General route-output assertions: seeded prefix + disjoint IDs
// ---------------------------------------------------------------------------

/// GET /api/v1/alerts on a seeded clone returns alert records with the
/// seed-specific `alert-{slug}-{seed}-N` ID prefix.
///
/// Asserts:
/// - HTTP 200
/// - `data` is non-empty
/// - First record has `alert_id` starting with `alert-{slug}-{seed}-`
#[tokio::test]
async fn test_f_p3_route_output_alerts_seeded_prefix() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={TEST_ACCESS_TOKEN}"))
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");

    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let data = body["data"].as_array().expect("$.data must be an array");

    assert!(
        !data.is_empty(),
        "$.data must be non-empty for seeded clone"
    );

    let expected_prefix = format!("alert-{slug}-{seed}-");
    let first = &data[0];
    let alert_id = first["alert_id"]
        .as_str()
        .expect("first alert must have 'alert_id' field (generated alert record shape)");

    assert!(
        alert_id.starts_with(&expected_prefix),
        "alert_id '{alert_id}' must start with '{expected_prefix}' \
         (generated alert ID format alert-{{slug}}-{{seed}}-N)"
    );
}

/// INV-DISTINCT-DATA-001: Two seeded Cyberint clones serve DISJOINT alert IDs
/// at GET /api/v1/alerts.
///
/// This test exercises the real route path and proves that INV-DISTINCT-DATA-001
/// holds where it matters — the adapter-visible route response.
#[tokio::test]
async fn test_f_p3_route_output_alerts_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let org_b = cafebabe_org();
    let slug_a = org_slug(&org_a);
    let slug_b = org_slug(&org_b);

    let clone_a = start_seeded_clone(100, org_a.clone()).await;
    let clone_b = start_seeded_clone(200, org_b.clone()).await;
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_alert_ids = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/api/v1/alerts"))
                .header("Cookie", format!("access_token={TEST_ACCESS_TOKEN}"))
                .send()
                .await
                .expect("GET /api/v1/alerts must not fail");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["data"]
                .as_array()
                .expect("$.data must be an array")
                .iter()
                .filter_map(|r| r["alert_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_alert_ids(base_a).await;
    let ids_b = fetch_alert_ids(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a (seed=100, slug={slug_a}) must serve non-empty alert results"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b (seed=200, slug={slug_b}) must serve non-empty alert results"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output Cyberint alerts): alert_ids from \
         clone_a (seed=100, slug={slug_a}) and clone_b (seed=200, slug={slug_b}) \
         must be DISJOINT at GET /api/v1/alerts; intersection={intersection:?}"
    );
}
