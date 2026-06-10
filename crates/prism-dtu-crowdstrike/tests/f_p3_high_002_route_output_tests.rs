//! Route-output tests for F-P3-HIGH-002 closure: CrowdStrike device routes.
//!
//! # What these tests assert
//!
//! These tests drive the ACTUAL ROUTES via an in-process HTTP server started by
//! `CrowdstrikeClone::new_with_seed`. They assert on ROUTE RESPONSES at the
//! adapter's real path_templates:
//!   - Step 1: GET /devices/queries/devices/v1 → `$.resources` (ID list)
//!   - Step 2: GET /devices/entities/devices/v2?ids=... → `$.resources` (details)
//!
//! This is the root-cause fix for the recurring miss pattern identified in Pass-3:
//! raw-state assertions (inspecting `state.generated_devices`) do NOT catch routing
//! bugs. Route-output tests catch dual-path wiring failures.
//!
//! # Tests
//!
//! | Test | What it proves |
//! |------|----------------|
//! | test_f_p3_high_002_device_id_list_serves_generated_ids | GET /devices/queries/devices/v1 returns seeded `dev-{slug}-{seed}-N` IDs |
//! | test_f_p3_high_002_device_detail_fetch_serves_generated_records | GET /devices/entities/devices/v2?ids=... returns generated device records |
//! | test_f_p3_high_002_device_ids_disjoint_across_seeds | Two seeds → disjoint device IDs at /devices/queries/devices/v1 |
//!
//! # Red-proof record
//!
//! - `test_f_p3_high_002_device_id_list_serves_generated_ids`: RED-PROOFED by
//!   temporarily reverting the `list_host_ids` dual-path to always use `load_host_ids()`
//!   (static fixture) regardless of `state.generated_devices`. The test FAILED because
//!   the static fixture device IDs do not contain the `dev-deadbeef-42-` prefix.
//!   Fix restored. Test PASSES.
//! - `test_f_p3_high_002_device_detail_fetch_serves_generated_records`: RED-PROOFED
//!   by reverting `get_host_details` dual-path to always use `load_host_details()`.
//!   Test FAILED because the static fixture records did not match seeded ID lookups.
//!   Fix restored. Test PASSES.
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

/// Build and start a seeded CrowdStrike clone.
///
/// CrowdStrike uses Bearer token auth. Any non-empty token is accepted by
/// the DTU `check_auth` function in routes/hosts.rs (validates non-empty Bearer,
/// does not validate against a specific allowlist).
async fn start_seeded_clone(seed: u64, org: prism_dtu_common::OrgId) -> CrowdstrikeClone {
    let mut clone = CrowdstrikeClone::new_with_seed(seed, org);
    clone
        .start()
        .await
        .expect("CrowdstrikeClone start must succeed");
    clone
}

// ---------------------------------------------------------------------------
// F-P3-HIGH-002: GET /devices/queries/devices/v1 (Step 1 — ID list)
// ---------------------------------------------------------------------------

/// F-P3-HIGH-002: GET /devices/queries/devices/v1 on a seeded CrowdStrike clone
/// returns generated device IDs in `$.resources`.
///
/// # What this proves
///
/// The `list_host_ids` dual-path serves generated device IDs when
/// `state.generated_devices` is non-empty. The IDs follow the canonical format
/// `dev-{org_slug}-{seed}-{n}` (ADR-036 §2.2 / BC-3.4.004).
///
/// # RED-PROOFED
///
/// Reverting `list_host_ids` dual-path to always use `load_host_ids()` (static fixture)
/// caused this test to FAIL: the static fixture IDs do not start with the
/// `dev-deadbeef-42-` prefix — the seed-specific assertion panicked.
/// Fix restored. Test PASSES.
///
/// Asserts:
/// - HTTP 200
/// - `$.resources` is non-empty (generated devices present)
/// - First ID starts with `dev-{slug}-{seed}-` (generated format)
/// - `$.meta.pagination.total` > 0
#[tokio::test]
async fn test_f_p3_high_002_device_id_list_serves_generated_ids() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token-f-p3")
        .send()
        .await
        .expect("GET /devices/queries/devices/v1 must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P3-HIGH-002: GET /devices/queries/devices/v1 on seeded clone must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("F-P3-HIGH-002: $.resources must be an array");

    assert!(
        !resources.is_empty(),
        "F-P3-HIGH-002: $.resources must be non-empty for seeded clone; \
         CompromisedEndpoint baseline has 50 device records at scale=1.0 — \
         dual-path not wired to list_host_ids"
    );

    // Validate that IDs follow the generated format `dev-{slug}-{seed}-{n}`.
    // Static fixture IDs use a different format (no seed embedded).
    let expected_prefix = format!("dev-{slug}-{seed}-");
    let first_id = resources[0]
        .as_str()
        .expect("F-P3-HIGH-002: $.resources[0] must be a string (device ID)");

    assert!(
        first_id.starts_with(&expected_prefix),
        "F-P3-HIGH-002: first device ID '{first_id}' must start with '{expected_prefix}' \
         (generated device ID format dev-{{slug}}-{{seed}}-N); \
         static fixture IDs do not embed the seed — dual-path is serving static fixture"
    );

    let total = body["meta"]["pagination"]["total"]
        .as_u64()
        .expect("F-P3-HIGH-002: $.meta.pagination.total must be a number");
    assert!(
        total > 0,
        "F-P3-HIGH-002: $.meta.pagination.total must be > 0 for seeded clone"
    );

    clone.stop().await.expect("clone stop must succeed");
}

/// F-P3-HIGH-002: GET /devices/entities/devices/v2?ids=... on a seeded CrowdStrike
/// clone returns the generated device detail records.
///
/// Two-step pattern: Step 1 gets IDs from `/devices/queries/devices/v1`,
/// then Step 2 fetches details from `/devices/entities/devices/v2?ids=<id1>&ids=<id2>`.
///
/// # RED-PROOFED
///
/// Reverting `get_host_details` dual-path to always use `load_host_details()` (static fixture)
/// caused this test to FAIL: looking up seeded IDs (e.g., `dev-deadbeef-42-0`) against the
/// static fixture returns no results (the static fixture doesn't have those IDs) → empty
/// `$.resources` array → assertion panics. Fix restored. Test PASSES.
///
/// Asserts:
/// - HTTP 200 from Step 1
/// - HTTP 200 from Step 2
/// - Step 2 `$.resources` is non-empty
/// - First resource has `device_id` starting with `dev-{slug}-{seed}-`
#[tokio::test]
async fn test_f_p3_high_002_device_detail_fetch_serves_generated_records() {
    let org = deadbeef_org();
    let slug = org_slug(&org);
    let seed = 42u64;

    let mut clone = start_seeded_clone(seed, org.clone()).await;
    let base_url = clone.base_url();
    let client = test_client();

    // Step 1: Fetch device IDs.
    let step1_resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token-f-p3")
        .query(&[("limit", "5")])
        .send()
        .await
        .expect("Step 1 GET /devices/queries/devices/v1 must succeed");

    assert_eq!(
        step1_resp.status().as_u16(),
        200,
        "F-P3-HIGH-002 Step 1: must return HTTP 200"
    );

    let step1_body: serde_json::Value = step1_resp
        .json()
        .await
        .expect("Step 1 response must be valid JSON");
    let ids: Vec<String> = step1_body["resources"]
        .as_array()
        .expect("Step 1 $.resources must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .take(3)
        .collect();

    assert!(
        !ids.is_empty(),
        "F-P3-HIGH-002 Step 1: must return at least 1 device ID"
    );

    // Step 2: Fetch device details by ID (no session header — direct fixture lookup path).
    // Device IDs have format "dev-{slug}-{seed}-{n}" — no special chars needing encoding.
    let mut step2_url = format!("{base_url}/devices/entities/devices/v2");
    let query: String = ids
        .iter()
        .map(|id| format!("ids={id}"))
        .collect::<Vec<_>>()
        .join("&");
    step2_url.push('?');
    step2_url.push_str(&query);

    let step2_resp = client
        .get(&step2_url)
        .header("Authorization", "Bearer test-token-f-p3")
        .send()
        .await
        .expect("Step 2 GET /devices/entities/devices/v2 must succeed");

    assert_eq!(
        step2_resp.status().as_u16(),
        200,
        "F-P3-HIGH-002 Step 2: must return HTTP 200"
    );

    let step2_body: serde_json::Value = step2_resp
        .json()
        .await
        .expect("Step 2 response must be valid JSON");
    let resources = step2_body["resources"]
        .as_array()
        .expect("F-P3-HIGH-002 Step 2: $.resources must be an array");

    assert!(
        !resources.is_empty(),
        "F-P3-HIGH-002 Step 2: $.resources must be non-empty; \
         requested IDs {ids:?} must be found in generated device store — \
         static fixture does not contain seeded IDs"
    );

    // Generated device records have `device_id` with the seed-specific prefix.
    let expected_prefix = format!("dev-{slug}-{seed}-");
    let first = &resources[0];
    let device_id = first["device_id"]
        .as_str()
        .expect("F-P3-HIGH-002 Step 2: first resource must have 'device_id' field");

    assert!(
        device_id.starts_with(&expected_prefix),
        "F-P3-HIGH-002 Step 2: device_id '{device_id}' must start with '{expected_prefix}' \
         (generated device ID format dev-{{slug}}-{{seed}}-N)"
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// INV-DISTINCT-DATA-001: Two seeds → disjoint device IDs at Step 1
// ---------------------------------------------------------------------------

/// INV-DISTINCT-DATA-001 (route-output): Two seeded CrowdStrike clones serve
/// DISJOINT device IDs at GET /devices/queries/devices/v1.
///
/// This test exercises the real route path and proves that INV-DISTINCT-DATA-001
/// holds where it matters — the adapter-visible Step 1 response.
#[tokio::test]
async fn test_f_p3_high_002_device_ids_disjoint_across_seeds() {
    let org_a = deadbeef_org();
    let org_b = cafebabe_org();
    let slug_a = org_slug(&org_a);
    let slug_b = org_slug(&org_b);

    let mut clone_a = start_seeded_clone(100, org_a.clone()).await;
    let mut clone_b = start_seeded_clone(200, org_b.clone()).await;
    let base_a = clone_a.base_url();
    let base_b = clone_b.base_url();
    let client = test_client();

    let fetch_device_ids = |base: String| {
        let client = client.clone();
        async move {
            let resp = client
                .get(format!("{base}/devices/queries/devices/v1"))
                .header("Authorization", "Bearer test-token-f-p3")
                .send()
                .await
                .expect("GET /devices/queries/devices/v1 must succeed");
            let body: serde_json::Value = resp.json().await.expect("must be valid JSON");
            body["resources"]
                .as_array()
                .expect("$.resources must be array")
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                .collect::<std::collections::HashSet<String>>()
        }
    };

    let ids_a = fetch_device_ids(base_a).await;
    let ids_b = fetch_device_ids(base_b).await;

    assert!(
        !ids_a.is_empty(),
        "clone_a (seed=100, slug={slug_a}) must serve non-empty device IDs"
    );
    assert!(
        !ids_b.is_empty(),
        "clone_b (seed=200, slug={slug_b}) must serve non-empty device IDs"
    );

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 (route-output CrowdStrike devices): device IDs from \
         clone_a (seed=100, slug={slug_a}) and clone_b (seed=200, slug={slug_b}) \
         must be DISJOINT at GET /devices/queries/devices/v1; \
         intersection={intersection:?}"
    );

    clone_a.stop().await.expect("clone_a stop must succeed");
    clone_b.stop().await.expect("clone_b stop must succeed");
}
