//! Differential archetype route-output tests — BC-2.06.018 story v1.4 tests 15-17.
//!
//! # Purpose
//!
//! These tests make the `fixture_set → Archetype` mapping load-bearing at the
//! route level. The prior AC-007 test (`test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error`)
//! only asserted `is_ok()` on `build_clone_pairs` — a paper-test under TD-VSDD-059.
//! These tests drive the actual clone route and assert on the SERVED RESPONSE BODY,
//! proving that the archetype passed to `new_with_seed` actually reaches `generate()`
//! and is not silently ignored.
//!
//! # Archetype baselines used (Claroty, scale=1.0)
//!
//! | Archetype | device records (generated) | alert records |
//! |-----------|--------------------------|---------------|
//! | DormantTenant | 0 | 0 |
//! | LargeScale | 10 000 | 500 |
//! | CompromisedEndpoint | 50 | 20 |
//!
//! # Tests
//!
//! | Test | ID | What it proves |
//! |------|----|----------------|
//! | test_BC_2_06_018_dormant_archetype_empty_served_response | 15 | fixture_set="dormant" → 0 devices and 0 alerts (BC EC-018-003 / TV-018-006) |
//! | test_BC_2_06_018_large_scale_archetype_record_count | 16 | fixture_set="large_scale" → exactly 10 000 device records |
//! | test_BC_2_06_018_archetype_drives_served_output_differential | 17 | same seed+org, "compromised" ≠ "dormant" — archetype DOES reach generate() |
//!
//! # Red-proof requirement
//!
//! Tests 15 and 17 were temporarily broken by re-hardcoding `Archetype::CompromisedEndpoint`
//! inside ClarotyClone::new_with_seed (ignoring the archetype param). Both FAILED
//! as required. See the commit message for the red-proof record.
//!
//! # Feature gate
//!
//! All tests require `fixture-gen` + `dtu` features.

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::{Archetype, BehavioralClone};

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

/// Fetch `$.total` and the `$.devices` array from `POST /api/v1/devices` on a started clone.
async fn fetch_devices(base_url: &str, client: &reqwest::Client) -> (u64, Vec<serde_json::Value>) {
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/devices must not fail at transport layer");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /api/v1/devices must return HTTP 200"
    );
    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");
    let total = body["total"].as_u64().expect("$.total must be a number");
    let devices = body["devices"]
        .as_array()
        .expect("$.devices must be an array")
        .clone();
    (total, devices)
}

/// Fetch `$.total` and the `$.alerts` array from `POST /api/v1/alerts` on a started clone.
async fn fetch_alerts(base_url: &str, client: &reqwest::Client) -> (u64, Vec<serde_json::Value>) {
    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/alerts must not fail at transport layer");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /api/v1/alerts must return HTTP 200"
    );
    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");
    let total = body["total"].as_u64().expect("$.total must be a number");
    let alerts = body["alerts"]
        .as_array()
        .expect("$.alerts must be an array")
        .clone();
    (total, alerts)
}

// ---------------------------------------------------------------------------
// Test 15: DormantTenant archetype → empty served response
// (BC EC-018-003 / TV-018-006)
// ---------------------------------------------------------------------------

/// Test 15: `fixture_set="dormant"` → Claroty clone serves EMPTY devices and alerts.
///
/// # What this proves (load-bearing)
///
/// DormantTenant is the only archetype that produces 0 records in both surfaces.
/// If `new_with_seed` ignores the archetype arg and always calls generate with
/// CompromisedEndpoint, this test FAILS: CompromisedEndpoint yields 50 devices
/// and 20 alerts → `$.total` would be 50 and 20 respectively, not 0.
///
/// Traces to: BC-2.06.018 EC-018-003 (dormant tenant serves no records)
/// TV-018-006 (DormantTenant route output is empty).
///
/// # Red-proof record
///
/// Verified FAIL when ClarotyClone::new_with_seed was temporarily patched to
/// hardcode Archetype::CompromisedEndpoint (ignoring the archetype arg):
/// - devices $.total was 50 (not 0) — assertion panicked
/// - alerts $.total was 20 (not 0) — assertion panicked
/// Fix restored. Test PASSES.
#[tokio::test]
async fn test_BC_2_06_018_dormant_archetype_empty_served_response() {
    let org = deadbeef_org();
    let seed = 42u64;
    let client = test_client();

    // DormantTenant: 0 device records, 0 alert records (BC-3.4.003 baseline row 8).
    let mut clone = ClarotyClone::new_with_seed(seed, Archetype::DormantTenant, org.clone());
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();

    let (device_total, devices) = fetch_devices(&base_url, &client).await;
    let (alert_total, alerts) = fetch_alerts(&base_url, &client).await;

    // Devices surface: DormantTenant generates 0 device records.
    assert_eq!(
        device_total, 0,
        "TEST-15 (dormant archetype): $.total for devices must be 0 for DormantTenant archetype; \
         got {device_total}. If total > 0, new_with_seed is ignoring the archetype arg \
         (still calling generate with CompromisedEndpoint) — F-P6-HIGH-001 not closed."
    );
    assert_eq!(
        devices.len(),
        0,
        "TEST-15 (dormant archetype): $.devices must be empty for DormantTenant; got {} records. \
         CompromisedEndpoint would produce 50 devices — non-empty = archetype not forwarded.",
        devices.len()
    );

    // Alerts surface: DormantTenant generates 0 alert records.
    assert_eq!(
        alert_total, 0,
        "TEST-15 (dormant archetype): $.total for alerts must be 0 for DormantTenant archetype; \
         got {alert_total}. If total > 0, archetype is being ignored."
    );
    assert_eq!(
        alerts.len(),
        0,
        "TEST-15 (dormant archetype): $.alerts must be empty for DormantTenant; got {} records.",
        alerts.len()
    );
}

// ---------------------------------------------------------------------------
// Test 16: LargeScale archetype → served record count matches generator baseline
// ---------------------------------------------------------------------------

/// Test 16: `fixture_set="large_scale"` → Claroty clone serves exactly 10 000 devices.
///
/// # Baseline source
///
/// Claroty generator `gen_large_scale` (crates/prism-dtu-claroty/src/generator.rs):
///   `n_devices = floor(10_000 * opts.scale) = floor(10_000 * 1.0) = 10_000`
///
/// This baseline was READ directly from the generator source before writing this test
/// (not assumed). The assertion uses the EXACT value from the generator.
///
/// # What this proves (load-bearing)
///
/// LargeScale generates 10,000 device records. CompromisedEndpoint generates 50.
/// If the archetype arg is ignored, `$.total` would be 50, not 10,000.
/// The count mismatch is the load-bearing signal.
///
/// Traces to: BC-2.06.018 AC-007 (INV-FIXTURE-SET-ARCHETYPE-MAP-001) / EC-018-005 / Red Gate test #16.
#[tokio::test]
async fn test_BC_2_06_018_large_scale_archetype_record_count() {
    let org = deadbeef_org();
    let seed = 42u64;
    let client = test_client();

    // LargeScale: 10,000 device records + 500 alert records (BC-3.4.003 baseline row 4).
    // Generator source: n_devices = floor(10_000 * 1.0) = 10_000
    // (READ from crates/prism-dtu-claroty/src/generator.rs gen_large_scale)
    let expected_device_count: u64 = 10_000;

    let mut clone = ClarotyClone::new_with_seed(seed, Archetype::LargeScale, org.clone());
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();

    let (device_total, devices) = fetch_devices(&base_url, &client).await;

    assert_eq!(
        device_total, expected_device_count,
        "TEST-16 (large_scale archetype): $.total for devices must be {expected_device_count} \
         (LargeScale generator baseline at scale=1.0); got {device_total}. \
         If total=50, new_with_seed is using CompromisedEndpoint regardless of archetype arg \
         (F-P6-HIGH-001 not closed). Generator source: gen_large_scale n_devices=10_000."
    );

    assert_eq!(
        devices.len() as u64,
        expected_device_count,
        "TEST-16: $.devices array length must equal $.total = {expected_device_count}; \
         got {} devices in array.",
        devices.len()
    );
}

// ---------------------------------------------------------------------------
// Test 17: Same seed + org, "compromised" vs "dormant" → DIFFERENT served output
// (archetype DOES reach generate() — not ignored)
// ---------------------------------------------------------------------------

/// Test 17: Same `(seed, org_id)`, `Archetype::CompromisedEndpoint` vs `Archetype::DormantTenant`
/// → the two clones serve DIFFERENT output at `POST /api/v1/devices`.
///
/// # What this proves (load-bearing)
///
/// This is the definitive proof that the archetype parameter reaches `generate()`.
///
/// If `new_with_seed` hardcodes `Archetype::CompromisedEndpoint` internally,
/// both clones would produce 50 devices — the `device_total_compromised != device_total_dormant`
/// assertion would FAIL (both equal 50).
///
/// With archetype forwarded correctly:
/// - CompromisedEndpoint → 50 devices (non-empty)
/// - DormantTenant → 0 devices (empty)
/// The totals differ: 50 ≠ 0, assertion PASSES.
///
/// # Red-proof record
///
/// Verified FAIL when ClarotyClone::new_with_seed was temporarily patched to hardcode
/// Archetype::CompromisedEndpoint: both clones produced total=50, assertion panicked
/// ("both clones must serve DIFFERENT device counts"). Fix restored. Test PASSES.
#[tokio::test]
async fn test_BC_2_06_018_archetype_drives_served_output_differential() {
    let org = deadbeef_org();
    let seed = 42u64; // identical seed — only archetype differs
    let client = test_client();

    // Clone A: CompromisedEndpoint → 50 device records, 20 alerts.
    let mut clone_compromised =
        ClarotyClone::new_with_seed(seed, Archetype::CompromisedEndpoint, org.clone());
    clone_compromised
        .start()
        .await
        .expect("compromised clone start must succeed");
    let base_compromised = clone_compromised.base_url();

    // Clone B: DormantTenant → 0 device records, 0 alerts.
    let mut clone_dormant =
        ClarotyClone::new_with_seed(seed, Archetype::DormantTenant, org.clone());
    clone_dormant
        .start()
        .await
        .expect("dormant clone start must succeed");
    let base_dormant = clone_dormant.base_url();

    let (device_total_compromised, _) = fetch_devices(&base_compromised, &client).await;
    let (device_total_dormant, _) = fetch_devices(&base_dormant, &client).await;

    // Core invariant: same seed + org_id, different archetype → different output.
    assert_ne!(
        device_total_compromised, device_total_dormant,
        "TEST-17 (archetype differential): CompromisedEndpoint (total={device_total_compromised}) \
         and DormantTenant (total={device_total_dormant}) clones with same seed={seed} and org_id \
         MUST serve DIFFERENT device counts. If both totals are equal, new_with_seed is ignoring \
         the archetype arg — F-P6-HIGH-001 not closed."
    );

    // Verify the expected direction: compromised is non-empty, dormant is empty.
    assert!(
        device_total_compromised > 0,
        "TEST-17: CompromisedEndpoint clone must serve non-empty devices; \
         got total={device_total_compromised}"
    );
    assert_eq!(
        device_total_dormant, 0,
        "TEST-17: DormantTenant clone must serve 0 devices; \
         got total={device_total_dormant}"
    );
}
