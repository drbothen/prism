//! Route-output tests for F-P15-MED-001 closure.
//!
//! # Finding
//!
//! `GET /api/v1/alerts` (registered in `build_router()`) served `state.alert_fixture`
//! UNCONDITIONALLY — no `fixture_gen_seeded` dual-path.  Seeded clients therefore received
//! the static alert fixture instead of generated per-org alerts, violating:
//! - BC-2.06.018 INV-DISTINCT-DATA-001 (seeded clones must serve org-distinct data)
//! - EC-018-003 (DormantTenant must serve empty, not static fixture)
//!
//! # Fix
//!
//! Added a `fixture_gen_seeded` dual-path to `get_alerts` in `routes/alerts.rs`,
//! mirroring the pattern from `search.rs` (lines ~198-222) for the `in:alerts` branch.
//!
//! # Tests
//!
//! | Test | What it proves |
//! |------|----------------|
//! | `test_f_p15_med_001_alerts_generated_route_output` | seeded clone serves generated alert records at GET /api/v1/alerts |
//! | `test_f_p15_med_001_alerts_disjoint_across_seeds` | two seeds → DISJOINT alert_id sets (INV-DISTINCT-DATA-001) |
//! | `test_f_p15_med_001_dormant_tenant_alerts_empty` | DormantTenant seeded clone → empty alerts response (EC-018-003) |
//!
//! # Red-proof
//!
//! `test_f_p15_med_001_alerts_disjoint_across_seeds` and
//! `test_f_p15_med_001_dormant_tenant_alerts_empty` were RED-PROOFED:
//! by temporarily reverting the dual-path (serving `state.alert_fixture` unconditionally),
//! both tests FAILED — disjointness failed (same static fixture IDs returned for both seeds),
//! and dormant-empty failed (static fixture is non-empty). Results documented in the commit
//! message for this fix burst (F-P15-MED-001).
//!
//! # Feature gate
//!
//! Requires `fixture-gen` + `dtu` features (in-process HTTP server + `new_with_seed`).

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{Archetype, BehavioralClone, OrgId};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Canonical test org: bytes [0xde, 0xad, 0xbe, 0xef, ...].
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Second test org: bytes [0xca, 0xfe, 0xba, 0xbe, ...].
fn cafebabe_org() -> OrgId {
    OrgId([
        0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Derive the org_slug hex prefix expected in generated IDs.
fn org_slug(org: &OrgId) -> String {
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
// F-P15-MED-001: GET /api/v1/alerts serves generated records on seeded clone
// ---------------------------------------------------------------------------

/// F-P15-MED-001: GET /api/v1/alerts on a seeded clone returns generated alert records,
/// NOT the static `alert_fixture`.
///
/// The generated alert record shape has `alert_id` with the prefix
/// `alert-{org_slug}-{seed}-` (from `generator.rs::build_alert`).
/// The static fixture alert records do NOT have this prefix — they use hard-coded IDs
/// like `"alert-001"`.  The load-bearing assertion is the seed-prefixed `alert_id`.
///
/// RED-PROOFED: verified that when `get_alerts` serves `state.alert_fixture`
/// unconditionally (the pre-fix state), this test FAILS because the static fixture
/// alert_ids do not start with the expected seed-prefixed pattern.
#[tokio::test]
async fn test_f_p15_med_001_alerts_generated_route_output() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = ArmisClone::new_with_seed(seed, Archetype::CompromisedEndpoint, org.clone())
        .expect("new_with_seed must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P15-MED-001: GET /api/v1/alerts on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let alerts = body["data"]["alerts"]
        .as_array()
        .expect("F-P15-MED-001: $.data.alerts must be an array");

    assert!(
        !alerts.is_empty(),
        "F-P15-MED-001: $.data.alerts must be non-empty for CompromisedEndpoint seeded clone; \
         got empty — handler is likely still serving static fixture (no dual-path wired to /api/v1/alerts)"
    );

    // Generated alert records have "alert_id" with the seed-specific prefix.
    // Static fixture records have hard-coded IDs that do NOT start with this prefix.
    // This is the load-bearing assertion that distinguishes generated-path from static-path.
    let expected_prefix = format!("alert-{slug}-{seed}-");
    let first = &alerts[0];
    let alert_id = first["alert_id"]
        .as_str()
        .expect("F-P15-MED-001: first alert must have 'alert_id' field (generated record shape)");
    assert!(
        alert_id.starts_with(&expected_prefix),
        "F-P15-MED-001: alert_id '{alert_id}' must start with '{expected_prefix}' \
         (generated alert ID format alert-{{slug}}-{{seed}}-N); \
         static fixture IDs look like 'alert-001' — if you see that, the dual-path is NOT wired"
    );

    let total = body["data"]["total"]
        .as_u64()
        .expect("F-P15-MED-001: $.data.total must be a number");
    assert!(
        total > 0,
        "F-P15-MED-001: $.data.total must be > 0 for generated alert records"
    );
}

// ---------------------------------------------------------------------------
// F-P15-MED-001 + INV-DISTINCT-DATA-001: cross-seed disjointness
// ---------------------------------------------------------------------------

/// F-P15-MED-001 + INV-DISTINCT-DATA-001:
/// Two seeded clones serve DISJOINT alert IDs at `GET /api/v1/alerts`.
///
/// This is the definitive invariant test: the REAL ROUTE output must be disjoint,
/// not just the raw `state.generated_records`.  Before the fix, both seeds returned
/// the same static `alert_fixture`, making intersection non-empty.
///
/// RED-PROOFED: verified this test FAILS when `get_alerts` serves `state.alert_fixture`
/// unconditionally — the intersection of (static fixture) ∩ (static fixture) is the
/// entire fixture set (non-empty), triggering the assertion.
#[tokio::test]
async fn test_f_p15_med_001_alerts_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let slug_a = org_slug(&org_a);
    let org_b = cafebabe_org();
    let slug_b = org_slug(&org_b);

    let mut clone_a = ArmisClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_a.clone())
        .expect("new_with_seed(100) must succeed");
    let mut clone_b = ArmisClone::new_with_seed(200, Archetype::CompromisedEndpoint, org_b.clone())
        .expect("new_with_seed(200) must succeed");

    clone_a.start().await.expect("clone_a start must succeed");
    clone_b.start().await.expect("clone_b start must succeed");
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_alert_ids = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/api/v1/alerts"))
                .header("Authorization", "Bearer test-token")
                .send()
                .await
                .expect("GET /api/v1/alerts must not fail");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["data"]["alerts"]
                .as_array()
                .expect("$.data.alerts must be array")
                .iter()
                .filter_map(|r| r["alert_id"].as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_alert_ids(base_a).await;
    let ids_b = fetch_alert_ids(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a (seed=100, slug={slug_a}) must serve non-empty alert results at /api/v1/alerts"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b (seed=200, slug={slug_b}) must serve non-empty alert results at /api/v1/alerts"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (F-P15-MED-001 route-output): alert_ids from clone_a \
         (seed=100, slug={slug_a}) and clone_b (seed=200, slug={slug_b}) must be DISJOINT \
         at GET /api/v1/alerts; intersection={intersection:?}. \
         Before the fix, both seeds serve the same static alert_fixture → non-empty intersection."
    );
}

// ---------------------------------------------------------------------------
// EC-018-003: DormantTenant → empty alerts at GET /api/v1/alerts
// ---------------------------------------------------------------------------

/// EC-018-003 / F-P6-HIGH-001:
/// A `DormantTenant` seeded clone serves an EMPTY alert list at `GET /api/v1/alerts`.
///
/// DormantTenant generates 0 records.  The `fixture_gen_seeded` sentinel ensures the
/// route takes the generated path (returning 0 records) rather than falling back to
/// the non-empty static `alert_fixture`.
///
/// RED-PROOFED: verified this test FAILS when `get_alerts` serves `state.alert_fixture`
/// unconditionally — the static fixture is non-empty (3 records), so `total > 0` and
/// `$.data.alerts` is non-empty, triggering both assertions.
#[tokio::test]
async fn test_f_p15_med_001_dormant_tenant_alerts_empty() {
    let org = deadbeef_org();

    let mut clone = ArmisClone::new_with_seed(999, Archetype::DormantTenant, org.clone())
        .expect("new_with_seed(DormantTenant) must succeed");
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("GET /api/v1/alerts must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-018-003: GET /api/v1/alerts on DormantTenant seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let total = body["data"]["total"]
        .as_u64()
        .expect("EC-018-003: $.data.total must be a number");
    assert_eq!(
        total, 0,
        "EC-018-003: DormantTenant seeded clone must serve total=0 at GET /api/v1/alerts; \
         got total={total}. Before the fix, the static alert_fixture (non-empty) is served \
         because fixture_gen_seeded=true but generated_records is empty — the sentinel must be \
         fixture_gen_seeded, NOT generated_records.is_empty()."
    );

    let alerts = body["data"]["alerts"]
        .as_array()
        .expect("EC-018-003: $.data.alerts must be an array");
    assert!(
        alerts.is_empty(),
        "EC-018-003: DormantTenant seeded clone must serve empty alerts array at \
         GET /api/v1/alerts; got {} records. \
         The static fixture must NOT be used for seeded clones.",
        alerts.len()
    );
}
