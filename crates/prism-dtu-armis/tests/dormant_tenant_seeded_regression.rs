//! Red Gate test 17: DormantTenant regression guard
//!
//! test_dormant_tenant_seeded_empty_records_not_static_fallback
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//! Traces to: DormantTenant regression invariant — `fixture_gen_seeded=true +
//!            generated_records=[]` must NOT fall back to static JSON fixture data.
//!
//! This test guards against route handlers branching on `generated_records.is_empty()`
//! instead of `fixture_gen_seeded`. The DormantTenant archetype legitimately produces
//! zero generated records, but the route must still serve the generated (empty) path,
//! NOT the static fixture path.
//!
//! FAIL mode: This test should PASS with the current implementation (the Armis route
//! already uses `fixture_gen_seeded` as the sentinel, not `generated_records.is_empty()`).
//! The test is included in the Red Gate suite because Story B's new scenario code paths
//! must also honor this invariant — if a handler branches on `generated_records.is_empty()`
//! for the scenario path, this test catches it.
//!
//! To make this a load-bearing Red Gate test, we add an assertion that verifies the
//! `fixture_gen_seeded` flag is set on a scenario-path clone (new_with_scenario), even
//! when the archetype might produce empty records. The scenario-path stub sets
//! `fixture_gen_seeded=true` (via new_with_seed_anchored) but `timeline=None`.
//!
//! We also verify via HTTP (starting the server) that a DormantTenant seeded clone
//! returns empty device list, not the static fixture's device list.
//!
//! Primary Red Gate FAIL: new_with_scenario leaves `timeline=None`, so any assertion
//! checking `state.timeline.is_some()` FAILS. This test adds that assertion to ensure
//! the scenario path is correctly wired even for DormantTenant.
//!
//! Secondary guard: route handler MUST NOT branch on `generated_records.is_empty()`.
//! We verify this by checking that fixture_gen_seeded=true prevents static fallback.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};

/// Org ID with well-known first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// RED GATE TEST 17 — test_dormant_tenant_seeded_empty_records_not_static_fallback
///
/// DormantTenant regression: `fixture_gen_seeded=true + generated_records=[]` must NOT
/// fall back to static fixture data — it must return empty response.
///
/// Story B extension: scenario-path clones (via new_with_scenario) must ALSO honor this
/// invariant. The test verifies:
/// 1. DormantTenant seeded clone via new_with_seed: fixture_gen_seeded=true, generated_records=[].
///    HTTP /api/v1/devices must return empty array (not static fixture).
/// 2. CompromisedEndpoint via new_with_scenario: fixture_gen_seeded=true, timeline must be Some.
///    The route handler's three-way branch (scenario, seeded-no-scenario, static) must
///    correctly use fixture_gen_seeded flag, NOT generated_records.is_empty().
///
/// The primary Red Gate FAIL is assertion 2b: `timeline.is_some()` fails because the
/// stub leaves timeline=None.
#[tokio::test]
async fn test_dormant_tenant_seeded_empty_records_not_static_fallback() {
    let org = deadbeef_org();
    let seed: u64 = 999;

    // -------------------------------------------------------------------------
    // Part 1: DormantTenant via new_with_seed (existing behavior — must PASS).
    // Verifies the invariant is preserved by Story A's implementation.
    // -------------------------------------------------------------------------

    let dormant_clone = ArmisClone::new_with_seed(seed, Archetype::DormantTenant, org.clone())
        .expect("ArmisClone::new_with_seed(DormantTenant) must succeed");

    // DormantTenant: fixture_gen_seeded must be true.
    assert!(
        dormant_clone.state.fixture_gen_seeded,
        "DormantTenant clone must have fixture_gen_seeded=true; got false. \
         F-P6-HIGH-001 / ADR-036 v2.2: DormantTenant is a seeded path even with 0 records"
    );

    // DormantTenant: generated_records must be empty (by design).
    assert!(
        dormant_clone.state.generated_records.is_empty(),
        "DormantTenant clone must have empty generated_records; got {} records. \
         DormantTenant archetype produces zero records by design.",
        dormant_clone.state.generated_records.len()
    );

    // Start the server and verify HTTP route returns empty device list (not static fixture).
    let mut dormant_server = ArmisClone::new_with_seed(seed, Archetype::DormantTenant, org.clone())
        .expect("second new_with_seed(DormantTenant) must succeed");
    dormant_server
        .start()
        .await
        .expect("dormant_server start must succeed");

    let base_url = dormant_server.base_url();
    let client = prism_dtu_common::build_test_client();

    // GET /api/v1/devices — DormantTenant should return empty data.devices, not static fixture.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-key")
        .header("X-Org-Id", "00000000-0000-7000-8000-0000000000AA")
        .send()
        .await
        .expect("GET /api/v1/devices must reach the server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "DormantTenant GET /api/v1/devices must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");

    let devices_count = body["data"]["devices"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);

    // The DormantTenant path must return 0 devices (NOT fall back to static fixture).
    // Static fixture has non-zero devices (e.g., devices.json has d001, d002, etc.).
    // If the handler branched on generated_records.is_empty(), it would return static devices.
    assert_eq!(
        devices_count, 0,
        "DormantTenant: GET /api/v1/devices must return 0 devices (empty generated path); \
         got {devices_count}. Route handler MUST branch on fixture_gen_seeded, \
         NOT generated_records.is_empty(). F-P6-HIGH-001 / ADR-036 v2.2"
    );

    dormant_server
        .stop()
        .await
        .expect("dormant_server stop must succeed");

    // -------------------------------------------------------------------------
    // Part 2: CompromisedEndpoint via new_with_scenario — Story B Red Gate.
    // The scenario path must ALSO be seeded (fixture_gen_seeded=true) AND
    // have timeline attached (timeline=Some).
    //
    // FAIL: new_with_scenario stub leaves timeline=None.
    // -------------------------------------------------------------------------

    let catalog = build_scenario_entity_catalog(seed, &org);
    let start_secs: i64 = 5_000_000;
    let timeline = build_default_incident_timeline(catalog, start_secs, &[]);
    let timeline_arc = Arc::new(timeline);

    let scenario_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_arc),
        chrono::DateTime::from_timestamp(start_secs, 0)
            .expect("valid timestamp")
            .with_timezone(&chrono::Utc),
    )
    .expect("new_with_scenario must succeed");

    // 2a: fixture_gen_seeded must be true (scenario path is always seeded).
    assert!(
        scenario_clone.state.fixture_gen_seeded,
        "Scenario-path clone must have fixture_gen_seeded=true (route handler sentinel). \
         BC-2.06.019 / ADR-036 v2.3 §2.4"
    );

    // 2b: timeline must be Some (proves scenario path is correctly wired).
    // FAIL: stub leaves timeline = None.
    assert!(
        scenario_clone.state.timeline.is_some(),
        "Scenario-path clone must have state.timeline = Some after new_with_scenario; \
         got None — stub not yet implemented. \
         BC-2.06.019 / ADR-036 v2.3 §2.4 \
         [RED GATE: fixture_gen_seeded=true + timeline=None is invalid scenario state]"
    );

    // 2c: verify the route handler's three-way branch correctness.
    // Once timeline is Some, the handler path is: timeline.is_some() → scenario path.
    // This is checked at compile time by the handler logic; the fixture_gen_seeded=true
    // flag ensures the handler never falls through to static fixture for scenario clones.
    // (Informational assertion — timeline.is_some() above is the load-bearing Red Gate fail.)
    assert_ne!(
        scenario_clone.state.fixture_gen_seeded, false,
        "fixture_gen_seeded must never be false for scenario-path clones. \
         Handler must branch on fixture_gen_seeded, not generated_records.is_empty()."
    );
}
