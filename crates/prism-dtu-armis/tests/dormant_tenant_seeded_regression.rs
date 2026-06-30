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
//! FAIL mode: Part 2 of this test should FAIL (Red Gate). A scenario-mode server at
//! stage 0 must return primary device ABSENT per StageMask projection. Without StageMask
//! projection in the route handler, all generated records are served → the primary device
//! IS in the response → the "must be absent at stage 0" assertion FAILS.
//!
//! Part 1 tests the existing DormantTenant invariant (must PASS — Story A already
//! implemented fixture_gen_seeded sentinel). Part 2 is the new Story B Red Gate.
//!
//! Architecture Compliance: handler must use THREE-WAY branching on `fixture_gen_seeded`:
//!   - scenario path:  fixture_gen_seeded=true  && timeline.is_some() → apply StageMask
//!   - seeded path:    fixture_gen_seeded=true  && timeline.is_none() → serve all generated
//!   - static path:    fixture_gen_seeded=false                       → serve static JSON
//!   DO NOT branch on `generated_records.is_empty()` — DormantTenant guard.

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
/// Two-part test:
///
/// Part 1: DormantTenant invariant (must PASS — Story A already implemented).
/// Verifies `fixture_gen_seeded=true + generated_records=[]` returns empty device list,
/// NOT static fixture data. If the handler branched on `generated_records.is_empty()`,
/// it would fall through to the static path and return non-zero devices.
///
/// Part 2: Scenario-path Red Gate (must FAIL without StageMask projection).
/// Starts a CompromisedEndpoint scenario-mode server with scenario_start placed in the
/// past to land at stage 0 (elapsed < 60s). At stage 0, the primary device must be
/// ABSENT from GET /api/v1/devices per StageMask projection. Without StageMask
/// projection the handler serves all generated records → primary IS visible → FAILS.
///
/// This also guards the three-way composition invariant: the scenario branch
/// (`fixture_gen_seeded=true && timeline.is_some()`) must never fall through to the
/// seeded path or the static path. If any handler regressed to `generated_records.is_empty()`
/// branching, a DormantTenant scenario clone would wrongly serve static fixture data.
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
    // Part 2: Scenario-path HTTP Red Gate — Story B (must FAIL without StageMask).
    //
    // Start a CompromisedEndpoint scenario server at stage 0 (elapsed < 60s).
    // At stage 0, primary device must be ABSENT from /api/v1/devices.
    //
    // FAIL: without StageMask projection, the handler serves all generated records →
    // primary device IS in the response → assertion "primary ABSENT at stage 0" FAILS.
    //
    // Also guards three-way composition: if any handler regressed to branching on
    // `generated_records.is_empty()`, DormantTenant scenario mode would serve static
    // fixture data instead of empty + stage-masked records.
    // -------------------------------------------------------------------------

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();

    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now - 10; // elapsed ≈ 10s → stage 0 (Baseline)

    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut scenario_server = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-0 regression server");

    scenario_server
        .start()
        .await
        .expect("scenario stage-0 server start must succeed");

    let scenario_url = scenario_server.base_url();
    let scenario_token = scenario_server.admin_token().to_owned();

    // GET /api/v1/devices at stage 0: primary device must be ABSENT.
    let resp_s0 = client
        .get(format!("{scenario_url}/api/v1/devices"))
        .header("Authorization", format!("Bearer {scenario_token}"))
        .send()
        .await
        .expect("GET /api/v1/devices (scenario stage 0) must reach the server");

    assert_eq!(
        resp_s0.status().as_u16(),
        200,
        "Scenario stage 0: GET /api/v1/devices must return HTTP 200; got {}",
        resp_s0.status().as_u16()
    );

    let body_s0: serde_json::Value = resp_s0
        .json()
        .await
        .expect("stage-0 scenario response must be JSON");

    let device_ids_s0: Vec<String> = body_s0["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 PC-4 / AC-007: at stage 0 primary device must be ABSENT.
    // Three-way composition guard: scenario path (seeded + timeline.is_some()) must apply
    // StageMask, NOT fall through to serve all records (seeded path) or static fixture.
    assert!(
        !device_ids_s0.contains(&primary_id),
        "TV-017-regression: at stage 0 (elapsed clamped to 0s < 60s), primary device '{}' must be \
         ABSENT from GET /api/v1/devices on a scenario-mode server; found it in {:?}. \
         Route handler must apply StageMask projection for scenario path \
         (fixture_gen_seeded=true && timeline.is_some()); must NOT regress to serving \
         all generated records or branching on generated_records.is_empty(). \
         BC-2.06.019 PC-4 / ADR-036 v2.3 §2.4",
        primary_id,
        device_ids_s0
    );

    scenario_server
        .stop()
        .await
        .expect("scenario stage-0 server stop must succeed");
}
