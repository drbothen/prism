//! Route-output surface-purity tests for F-P4-CRIT-001 closure: Claroty devices dual-path.
//!
//! # What these tests assert
//!
//! These tests drive the ACTUAL ROUTE via an in-process HTTP server started by
//! `ClarotyClone::new_with_seed`. They assert SURFACE PURITY at the route level —
//! not merely that the first record looks right, but that:
//!   (a) EVERY record belongs to the device surface (no foreign-surface alert records)
//!   (b) The exact count of device-surface records matches the generator baseline
//!   (c) Seeded IDs have the expected prefix
//!   (d) Two seeds → disjoint device IDs
//!
//! # Finding closed
//!
//! F-P4-CRIT-001: `list_devices` served `state.generated_records.clone()` (full slice)
//! including 20 alert records from the CompromisedEndpoint archetype. The alert records
//! have `alert_id` but not `asset_id`, causing null-filled garbage rows under
//! claroty.sensor.toml device table columns.
//!
//! Fix: filter to records where `asset_id` is present AND `alert_id` is absent,
//! mirroring the Armis `paginate_devices` discriminator.
//!
//! # Generator baselines (CompromisedEndpoint, scale=1.0)
//!
//! | Surface | Count | Discriminator |
//! |---------|-------|---------------|
//! | device  | 50    | asset_id present, alert_id absent |
//! | alert   | 20    | alert_id present, asset_id absent |
//! | total   | 70    | all records in generated_records  |
//!
//! # Red-proof record
//!
//! `test_f_p4_crit_001_devices_surface_purity`: RED-PROOFED by temporarily reverting
//! the filter (`state.generated_records.clone()` with NO surface filter) and confirming
//! this test FAILED — the purity assertion panicked when 20 alert records (which lack
//! `asset_id` but have `alert_id`) appeared in `$.devices`. Fix restored, test PASSES.
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

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("test client build must succeed")
}

// ---------------------------------------------------------------------------
// F-P4-CRIT-001: POST /api/v1/devices surface purity — no alert records
// ---------------------------------------------------------------------------

/// F-P4-CRIT-001: POST /api/v1/devices on a seeded Claroty clone returns ONLY
/// device-surface records — zero alert records in the response.
///
/// # Root cause
///
/// `list_devices` served `state.generated_records.clone()` with no surface filter.
/// CompromisedEndpoint archetype: 50 device records + 20 alert records (total 70).
/// All 70 were served at `/api/v1/devices`, leaking 20 alert records as garbage rows
/// (null asset_id, null uid, etc.) under the claroty.sensor.toml devices table columns.
///
/// # Fix
///
/// Filter applied: `rec.get("asset_id").is_some() && rec.get("alert_id").is_none()`
/// This matches the authoritative discriminator used by Armis paginate_devices.
///
/// # RED-PROOFED
///
/// Reverting to the broken `state.generated_records.clone()` (no filter) caused this
/// test to FAIL: the purity assertion
/// "EVERY device record must have NO alert_id — 20 alert records leaked" panicked
/// for records 50-69 which had `alert_id` but no `asset_id`.
/// Fix restored. Test PASSES.
///
/// # Generator baseline (CompromisedEndpoint, scale=1.0)
///
/// device records: 50 (asset_id present, alert_id absent)
/// alert records:  20 (alert_id present, asset_id absent)
///
/// Asserts:
/// - HTTP 200
/// - `$.total` == 50 (device-surface count only)
/// - Every record in `$.devices` has NO `alert_id` field
/// - Every record in `$.devices` HAS an `asset_id` field
/// - First record `asset_id` starts with `ASSET-{slug}-{seed}-`
#[tokio::test]
async fn test_f_p4_crit_001_devices_surface_purity() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ClarotyClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    );
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

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
        "F-P4-CRIT-001: POST /api/v1/devices on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let devices = body["devices"]
        .as_array()
        .expect("F-P4-CRIT-001: $.devices must be an array");

    // (b) Exact count: CompromisedEndpoint at scale=1.0 has 50 device records.
    // The total includes only device-surface records (alert records must not be counted).
    let total = body["total"]
        .as_u64()
        .expect("F-P4-CRIT-001: $.total must be a number");

    assert_eq!(
        total, 50,
        "F-P4-CRIT-001: $.total must be exactly 50 (device-surface count for \
         CompromisedEndpoint at scale=1.0); got {total} — if total=70, the filter is \
         not applied (50 devices + 20 alerts = 70 total generated_records)"
    );

    assert_eq!(
        devices.len(),
        50,
        "F-P4-CRIT-001: $.devices array must have exactly 50 records (device surface); \
         got {} — expected 50 device records, not 70 (devices + alerts)",
        devices.len()
    );

    // (a) Surface purity: EVERY record must NOT have alert_id.
    for (i, record) in devices.iter().enumerate() {
        assert!(
            record.get("alert_id").is_none(),
            "F-P4-CRIT-001: $.devices[{i}] has an 'alert_id' field — alert record leaked \
             into the devices response. Before the fix, list_devices served full \
             generated_records (70 records including 20 alerts). Fix: filter to \
             asset_id present AND alert_id absent. \
             record keys: {:?}",
            record.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
    }

    // (a) Surface purity: EVERY record must have asset_id.
    for (i, record) in devices.iter().enumerate() {
        assert!(
            record.get("asset_id").is_some(),
            "F-P4-CRIT-001: $.devices[{i}] is missing 'asset_id' — not a device record. \
             device surface discriminator: asset_id present AND alert_id absent."
        );
    }

    // (c) Seeded prefix check on first record.
    let expected_asset_prefix = format!("ASSET-{slug}-{seed}-");
    let first = &devices[0];
    let asset_id = first["asset_id"]
        .as_str()
        .expect("F-P4-CRIT-001: first device must have asset_id string");
    assert!(
        asset_id.starts_with(&expected_asset_prefix),
        "F-P4-CRIT-001: first device asset_id '{asset_id}' must start with \
         '{expected_asset_prefix}' (Claroty generator format ASSET-{{slug}}-{{seed}}-N)"
    );
}

/// F-P4-CRIT-001 — verify alerts route is UNCHANGED and still serves 20 alert records.
///
/// Regression guard: the device-surface filter must not affect the alerts route.
/// CompromisedEndpoint at scale=1.0: 20 alert records at /api/v1/alerts.
///
/// Asserts:
/// - $.total == 20
/// - Every record in $.alerts has an `alert_id` field (alert surface)
/// - NO record in $.alerts has no `alert_id` (device records must not appear)
#[tokio::test]
async fn test_f_p4_crit_001_alerts_still_serve_20_alert_records() {
    let org = deadbeef_org();
    let seed = 42u64;

    let mut clone = ClarotyClone::new_with_seed(
        seed,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org.clone(),
    );
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/alerts must not fail");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");

    let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
    let alerts = body["alerts"]
        .as_array()
        .expect("F-P4-CRIT-001 regression: $.alerts must be an array");

    let total = body["total"].as_u64().expect("$.total must be a number");

    assert_eq!(
        total, 20,
        "F-P4-CRIT-001 regression: $.total for alerts must be exactly 20 \
         (CompromisedEndpoint at scale=1.0 has 20 alert records); got {total}"
    );

    assert_eq!(
        alerts.len(),
        20,
        "F-P4-CRIT-001 regression: $.alerts must have exactly 20 records; got {}",
        alerts.len()
    );

    // Every alert record must have alert_id.
    for (i, record) in alerts.iter().enumerate() {
        assert!(
            record.get("alert_id").is_some(),
            "F-P4-CRIT-001 regression: $.alerts[{i}] is missing 'alert_id' — not an alert record. \
             alerts route filter: alert_id present."
        );
    }
}

/// F-P4-CRIT-001 + INV-DISTINCT-DATA-001:
/// Two seeded Claroty clones serve DISJOINT device IDs at POST /api/v1/devices.
///
/// Confirms surface purity + data disjointness together.
#[tokio::test]
async fn test_f_p4_crit_001_devices_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    let mut clone_a = ClarotyClone::new_with_seed(
        100,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_a.clone(),
    );
    let mut clone_b = ClarotyClone::new_with_seed(
        200,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_b.clone(),
    );

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_device_ids = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .post(format!("{base}/api/v1/devices"))
                .header("Authorization", "Bearer test-token")
                .json(&serde_json::json!({}))
                .send()
                .await
                .expect("POST /api/v1/devices must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            let devices = body["devices"].as_array().expect("$.devices must be array");

            // Also assert surface purity inline (every device has no alert_id).
            for (i, rec) in devices.iter().enumerate() {
                assert!(
                    rec.get("alert_id").is_none(),
                    "INV-DISTINCT-DATA-001 disjoint test: $.devices[{i}] has alert_id — \
                     surface purity violated in clone at {base}"
                );
            }

            devices
                .iter()
                .filter_map(|r| r["asset_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_device_ids(base_a).await;
    let ids_b = fetch_device_ids(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a (seed=100, slug={slug_a}) must serve non-empty device results"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b (seed=200, slug={slug_b}) must serve non-empty device results"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output Claroty devices): asset_ids from \
         clone_a (seed=100, slug={slug_a}) and clone_b (seed=200, slug={slug_b}) \
         must be DISJOINT at POST /api/v1/devices; intersection={intersection:?}"
    );
}
