//! Red Gate tests 9, 10, 11, 15, 18: BC-2.06.019 / BC-2.06.020 demo-server scenario guards
//!
//! Tests:
//!   Test 9:  test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones
//!   Test 10: test_BC_2_06_019_e_demo_003_unrecognized_archetype
//!   Test 11: test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path
//!   Test 15: test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones
//!   NEW:     test_BC_2_06_019_guard_order_e_demo_002_before_e_demo_004
//!   Test 18: test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction (B-P3-01)
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//! Traces to: BC-2.06.019 E-DEMO-002/003 / TV-019-012,013,007
//!            BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 / PC-5
//!            BC-2.06.019 EC-019-012: archetype/fixture_set contradiction
//!            Architecture Compliance Rules: E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004 order

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use prism_dtu_demo_server::{config::DemoConfig, harness::build_clone_pairs};

// ---------------------------------------------------------------------------
// Constants for test vectors
// ---------------------------------------------------------------------------

/// Org UUID with first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
/// ADR-036 §2.2: "Any test using 'dev-acme-...' is incorrect."
const DEMO_ORG_UUID_DEADBEEF: &str = "deadbeef-0000-7000-8000-000000000000";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal DemoConfig with only CrowdStrike enabled, with scenario settings.
fn make_cs_only_with_scenario(
    seed: u64,
    org_id: Option<&str>,
    scenario_enabled: bool,
    scenario_archetype: &str,
    stage_duration_secs: Vec<u64>,
) -> DemoConfig {
    let mut config = DemoConfig::default();
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = seed;
    config.clones.crowdstrike.org_id = org_id.map(|s| s.to_string());
    config.clones.crowdstrike.fixture_set = "compromised".to_string();
    config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: scenario_enabled,
        archetype: scenario_archetype.to_string(),
        scenario_start_secs: None,
        stage_duration_secs,
    });
    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.armis.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

/// Create a DemoConfig with both CrowdStrike and Armis enabled with potentially
/// mismatched seeds (for E-DEMO-002 testing).
fn make_cs_armis_scenario(cs_seed: u64, armis_seed: u64, org_id: &str) -> DemoConfig {
    let mut config = DemoConfig::default();

    // CrowdStrike
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = cs_seed;
    config.clones.crowdstrike.org_id = Some(org_id.to_string());
    config.clones.crowdstrike.fixture_set = "compromised".to_string();
    config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    // Armis (different seed = E-DEMO-002 condition)
    config.clones.armis.enabled = true;
    config.clones.armis.seed = armis_seed;
    config.clones.armis.org_id = Some(org_id.to_string());
    config.clones.armis.fixture_set = "compromised".to_string();
    config.clones.armis.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

/// Create a DemoConfig with mismatched seeds AND missing org_id.
/// Used by the guard-order test (B-P1-03): E-DEMO-002 must fire before E-DEMO-004.
fn make_cs_armis_scenario_missing_org(cs_seed: u64, armis_seed: u64) -> DemoConfig {
    let mut config = DemoConfig::default();

    // CrowdStrike — no org_id (trips E-DEMO-004 if reached)
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = cs_seed;
    config.clones.crowdstrike.org_id = None; // missing → E-DEMO-004 if order is wrong
    config.clones.crowdstrike.fixture_set = "compromised".to_string();
    config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    // Armis — different seed (trips E-DEMO-002)
    config.clones.armis.enabled = true;
    config.clones.armis.seed = armis_seed;
    config.clones.armis.org_id = None; // also missing
    config.clones.armis.fixture_set = "compromised".to_string();
    config.clones.armis.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

// ---------------------------------------------------------------------------
// RED GATE TEST 9 — test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones
//
// BC-2.06.019 E-DEMO-002 / TV-019-012
// Verifies: build_clone_pairs returns Err containing "E-DEMO-002" when two
// scenario-enabled clones have different seeds.
//
// FAIL mode: E-DEMO-002 guard not yet implemented in build_clone_pairs.
// The function currently succeeds even with mismatched seeds.
// Assertion that result.is_err() will FAIL (function returns Ok).
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones() {
    // TV-019-012: clones.crowdstrike.seed = 100, clones.armis.seed = 200
    // Both have scenario.enabled = true. Seeds differ → E-DEMO-002.
    let config = make_cs_armis_scenario(
        100, // crowdstrike seed
        200, // armis seed — DIFFERENT → E-DEMO-002
        DEMO_ORG_UUID_DEADBEEF,
    );

    // build_clone_pairs must detect the seed mismatch BEFORE constructing any clone.
    // FAIL: E-DEMO-002 guard not implemented → function returns Ok instead of Err.
    let result = build_clone_pairs(&config);

    assert!(
        result.is_err(),
        "TV-019-012: build_clone_pairs must return Err when scenario-enabled clones \
         have different seeds (crowdstrike=100 vs armis=200); got Ok \
         — E-DEMO-002 guard not yet implemented in build_clone_pairs \
         [RED GATE: expected Err, got Ok]"
    );

    // Use .err() instead of .unwrap_err() — ClonePair doesn't impl Debug.
    let err = result.err().expect("verified is_err above");
    let err_str = err.to_string();

    // Error message must contain "E-DEMO-002".
    assert!(
        err_str.contains("E-DEMO-002"),
        "E-DEMO-002 error message must contain 'E-DEMO-002'; got: '{err_str}' \
         — BC-2.06.019 / TV-019-012"
    );

    // Error message must mention the clone names involved.
    assert!(
        err_str.contains("crowdstrike") || err_str.contains("armis"),
        "E-DEMO-002 error message must name the mismatched clones; got: '{err_str}'"
    );

    // Error message must include the seed values.
    assert!(
        err_str.contains("100") && err_str.contains("200"),
        "E-DEMO-002 error message must include both seed values (100 and 200); got: '{err_str}'"
    );
}

// ---------------------------------------------------------------------------
// RED GATE TEST 10 — test_BC_2_06_019_e_demo_003_unrecognized_archetype
//
// BC-2.06.019 E-DEMO-003 / TV-019-013
// Verifies: build_clone_pairs returns Err containing "E-DEMO-003" when
// scenario.archetype is an unrecognized value.
//
// Also tests the wrong-length stage_duration_secs variant of E-DEMO-003:
// CompromisedEndpoint requires exactly 4 entries; [60, 180, 360] has only 3.
//
// FAIL mode: E-DEMO-003 guard not yet implemented in build_clone_pairs.
// The function currently succeeds even with invalid archetype or wrong-length
// stage_duration_secs. Assertion that result.is_err() will FAIL.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_e_demo_003_unrecognized_archetype() {
    // Variant A: completely unrecognized archetype string.
    let config_bad_archetype = make_cs_only_with_scenario(
        42,
        Some(DEMO_ORG_UUID_DEADBEEF),
        true,
        "unknown_archetype_value", // TV-019-013: unrecognized
        vec![],
    );

    // FAIL: E-DEMO-003 guard not implemented → function returns Ok instead of Err.
    let result_a = build_clone_pairs(&config_bad_archetype);

    assert!(
        result_a.is_err(),
        "TV-019-013 Variant A: build_clone_pairs must return Err when \
         scenario.archetype='unknown_archetype_value' is unrecognized; got Ok \
         — E-DEMO-003 guard not yet implemented in build_clone_pairs \
         [RED GATE: expected Err, got Ok]"
    );

    // Use .err() instead of .unwrap_err() — ClonePair doesn't impl Debug.
    let err_a = result_a.err().expect("verified is_err above");
    let err_str_a = err_a.to_string();

    assert!(
        err_str_a.contains("E-DEMO-003"),
        "E-DEMO-003 Variant A error must contain 'E-DEMO-003'; got: '{err_str_a}'"
    );
    assert!(
        err_str_a.contains("unknown_archetype_value"),
        "E-DEMO-003 Variant A error must contain the invalid archetype string; got: '{err_str_a}'"
    );

    // Variant B: valid archetype but wrong stage_duration_secs length.
    // CompromisedEndpoint requires exactly 4 entries; we provide 3.
    let config_bad_stages = make_cs_only_with_scenario(
        42,
        Some(DEMO_ORG_UUID_DEADBEEF),
        true,
        "compromised_endpoint",
        vec![60, 180, 360], // 3 entries, not 4 → E-DEMO-003 Variant B
    );

    // FAIL: E-DEMO-003 Variant B guard not implemented → function returns Ok instead of Err.
    let result_b = build_clone_pairs(&config_bad_stages);

    assert!(
        result_b.is_err(),
        "TV-019-013 Variant B: build_clone_pairs must return Err when \
         stage_duration_secs has 3 entries but CompromisedEndpoint requires 4; got Ok \
         — E-DEMO-003 Variant B guard not yet implemented \
         [RED GATE: expected Err, got Ok]"
    );

    // Use .err() instead of .unwrap_err() — ClonePair doesn't impl Debug.
    let err_b = result_b.err().expect("verified is_err above");
    let err_str_b = err_b.to_string();

    assert!(
        err_str_b.contains("E-DEMO-003"),
        "E-DEMO-003 Variant B error must contain 'E-DEMO-003'; got: '{err_str_b}'"
    );
}

// ---------------------------------------------------------------------------
// RED GATE TEST 11 — test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path
//
// BC-2.06.019 INV-SCENARIO-DISABLED-COMPAT-001 / TV-019-007
// Verifies: a clone built with scenario.enabled=false and seed=42 produces
// the same generated_records as a clone built with new_with_seed(42, ...) directly.
// state.timeline must be None when scenario.enabled=false.
//
// This is a regression guard — the scenario path must not break the non-scenario path.
//
// FAIL mode: This test should mostly PASS on the current stub (scenario.enabled=false
// → falls back to new_with_seed which works correctly). However, the test explicitly
// verifies that `state.timeline.is_none()` (which the stub correctly returns).
// The test DOES fail because `build_clone_pairs` doesn't yet wire the scenario.enabled
// check to distinguish scenario vs non-scenario paths — specifically it doesn't call
// `new_with_scenario` even when scenario.enabled=true (because the guard isn't there).
//
// The failing assertion depends on what build_clone_pairs does for the enabled=false
// case. Currently it works correctly — this test should PASS.
//
// To ensure this is a Red Gate test (must FAIL), we add an assertion that verifies
// the scenario.enabled=true path would wire the timeline, which would be absent
// without the guard. We test this by building with scenario.enabled=true and
// checking that timeline IS set (which will FAIL because new_with_scenario is a stub).
//
// NOTE: The story spec defines this as a "regression" test — it verifies backward
// compat, not a new failing behavior. We write it to pass once the scenario path
// is correctly gated. The Red Gate failure comes from the companion assertion that
// scenario.enabled=true WOULD set timeline.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path() {
    use prism_dtu_common::{Archetype, OrgId};
    use prism_dtu_crowdstrike::CrowdstrikeClone;

    // Parse the deadbeef org UUID.
    let org_uuid = uuid::Uuid::parse_str(DEMO_ORG_UUID_DEADBEEF)
        .expect("DEMO_ORG_UUID_DEADBEEF must be valid UUID");
    let org_id = OrgId(*org_uuid.as_bytes());

    // Build a direct new_with_seed clone (scenario.enabled=false baseline).
    let baseline =
        CrowdstrikeClone::new_with_seed(42, Archetype::CompromisedEndpoint, org_id.clone());

    // The baseline must produce records.
    assert!(
        !baseline.state.generated_devices.is_empty(),
        "Baseline CrowdstrikeClone::new_with_seed must produce non-empty generated_devices"
    );

    // Baseline timeline must be None (scenario.enabled=false path).
    assert!(
        baseline.state.timeline.is_none(),
        "INV-SCENARIO-DISABLED-COMPAT-001: scenario.enabled=false path must have \
         state.timeline = None; got Some. BC-2.06.019 / TV-019-007"
    );

    // Now verify: if we build with scenario.enabled=true, the harness SHOULD call
    // new_with_scenario (once implemented), which WOULD set timeline to Some.
    // This assertion is the load-bearing Red Gate check:
    // build_clone_pairs with scenario.enabled=true should produce a clone with
    // state.timeline = Some, which currently fails because:
    // 1. build_clone_pairs doesn't call new_with_scenario (E-DEMO-002/003 guards absent)
    // 2. Even if it did, new_with_scenario stub leaves timeline = None
    //
    // We test this by directly calling new_with_scenario and checking timeline.
    use prism_dtu_common::{build_default_incident_timeline, build_scenario_entity_catalog};
    use std::sync::Arc;

    let catalog = build_scenario_entity_catalog(42, &org_id);
    let start_secs: i64 = 2_000_000;
    let timeline = build_default_incident_timeline(catalog.clone(), start_secs, &[]);
    let timeline_arc = Arc::new(timeline);

    let scenario_clone = CrowdstrikeClone::new_with_scenario(
        42,
        Archetype::CompromisedEndpoint,
        org_id,
        Arc::clone(&timeline_arc),
        // Use demo_time_anchor (2026-01-01T00:00:00Z) — deterministic anchor.
        // The timeline's scenario_start_epoch_secs is independent from time_anchor.
        prism_dtu_common::demo_time_anchor(),
        &catalog,
    );

    // FAIL: new_with_scenario stub leaves timeline = None.
    // Once implemented, this must be Some (proving scenario path wires the timeline).
    assert!(
        scenario_clone.state.timeline.is_some(),
        "INV-SCENARIO-DISABLED-COMPAT-001: new_with_scenario must set state.timeline = Some; \
         got None — stub not yet implemented. BC-2.06.019 / TV-019-007 \
         [RED GATE: expected Some, got None]"
    );

    // Byte-identity check: generated_devices from scenario path must contain the same
    // device IDs as the baseline path (catalog derivation does not shift primary stream).
    // This is informational — the primary failure is timeline.is_some() above.
    let baseline_ids: std::collections::HashSet<String> = baseline
        .state
        .generated_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    let scenario_ids: std::collections::HashSet<String> = scenario_clone
        .state
        .generated_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert_eq!(
        baseline_ids, scenario_ids,
        "INV-SCENARIO-DISABLED-COMPAT-001: generated_devices must be byte-identical between \
         scenario-disabled and scenario-enabled paths (same seed=42, same org_id). \
         Catalog derivation MUST NOT shift the primary generator stream. \
         BC-2.06.019 / TV-019-007"
    );
}

// ---------------------------------------------------------------------------
// RED GATE TEST 15 — test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones
//
// BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 / PC-5
// HTTP-level load-bearing test (B-P1-02).
//
// Verifies: given three clones (Armis, CrowdStrike, Claroty) all built with
// the same (seed=100, org_id=deadbeef-...) and scenario.enabled=true,
// queried via real HTTP servers at stage 1 (scenario_start = now - 90s),
// all three serve the canonical primary device ID "dev-deadbeef-100-0".
//
// FAIL mode:
// 1. new_with_scenario stubs leave timeline=None → StageMask projection absent.
// 2. Without projection, stage-1 might not filter correctly (lateral devices present).
// 3. Primary assertion on cross-DTU device ID coherence fails if IDs differ.
// 4. current_stage_index stub returns 0, so stage-1 mask is not applied correctly.
//
// The primary load-bearing RED GATE failure: CrowdStrike, Armis, and Claroty handlers
// do not yet implement StageMask projection, so the HTTP responses do not reflect
// stage-filtered content. Additionally, timeline=None means the scenario path is not
// activated, so the stage-clock is ignored.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[tokio::test]
async fn test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones() {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_common::{
        build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
        OrgId,
    };
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use std::sync::Arc;

    let seed: u64 = 100;
    let org_uuid = uuid::Uuid::parse_str(DEMO_ORG_UUID_DEADBEEF)
        .expect("DEMO_ORG_UUID_DEADBEEF must be valid UUID");
    let org_id = OrgId(*org_uuid.as_bytes());

    // Stage 1 clock control: scenario_start = now - 90s → elapsed ≈ 90s ≥ 60s, < 180s.
    // At request time: current_stage_index returns 1 (Recon).
    let now = chrono::Utc::now().timestamp();
    let start_stage1: i64 = now - 90;

    let catalog = build_scenario_entity_catalog(seed, &org_id);

    // Verify canonical IDs (BC-2.06.020 PC-5 / AC-015).
    let expected_primary_cs = "dev-deadbeef-100-0";
    let expected_primary_armis = "dev-deadbeef-100-0";

    assert_eq!(
        catalog.primary_device_id_cs, expected_primary_cs,
        "Cross-DTU coherence: catalog.primary_device_id_cs must be '{expected_primary_cs}'; \
         got '{}'. ADR-036 §2.2 canonical format: dev-{{org_slug}}-{{seed}}-0",
        catalog.primary_device_id_cs
    );
    assert_eq!(
        catalog.primary_device_id_armis, expected_primary_armis,
        "Cross-DTU coherence: catalog.primary_device_id_armis must be '{expected_primary_armis}'; \
         got '{}'. ADR-036 §2.2 canonical format: dev-{{org_slug}}-{{seed}}-0",
        catalog.primary_device_id_armis
    );

    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    // Construct all three scenario clones with the SAME seed, org_id, and timeline Arc.
    let mut cs_clone = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    );

    let mut armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("ArmisClone::new_with_scenario must succeed");

    let mut claroty_clone = ClarotyClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id,
        Arc::clone(&timeline),
        time_anchor,
    );

    // All three must have timeline attached before starting.
    // FAIL: all three stubs leave timeline = None.
    assert!(
        cs_clone.state.timeline.is_some(),
        "Cross-DTU coherence: CrowdstrikeClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE: stub leaves timeline=None]"
    );
    assert!(
        armis_clone.state.timeline.is_some(),
        "Cross-DTU coherence: ArmisClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE: stub leaves timeline=None]"
    );
    assert!(
        claroty_clone.state.timeline.is_some(),
        "Cross-DTU coherence: ClarotyClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE: stub leaves timeline=None]"
    );

    // Start all three servers.
    cs_clone
        .start()
        .await
        .expect("CrowdstrikeClone start must succeed");
    armis_clone
        .start()
        .await
        .expect("ArmisClone start must succeed");
    claroty_clone
        .start()
        .await
        .expect("ClarotyClone start must succeed");

    let cs_url = cs_clone.base_url();
    let cs_token = cs_clone.admin_token().to_owned();
    let armis_url = armis_clone.base_url();
    let armis_token = armis_clone.admin_token().to_owned();
    let claroty_url = claroty_clone.base_url();
    let claroty_token = claroty_clone.admin_token().to_owned();

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // CrowdStrike: GET /devices/entities/devices/v2?ids=dev-deadbeef-100-0
    // At stage 1 (Recon), primary device must be present in response.
    // -------------------------------------------------------------------------
    let cs_resp = client
        .get(format!(
            "{cs_url}/devices/entities/devices/v2?ids={expected_primary_cs}"
        ))
        .header("Authorization", format!("Bearer {cs_token}"))
        .send()
        .await
        .expect("CS GET /devices/entities/devices/v2 must reach the server");

    assert_eq!(
        cs_resp.status().as_u16(),
        200,
        "Cross-DTU coherence: CrowdStrike GET /devices/entities/devices/v2 must return 200; \
         got {}",
        cs_resp.status().as_u16()
    );

    let cs_body: serde_json::Value = cs_resp.json().await.expect("CS response must be JSON");
    let cs_resources = cs_body["resources"].as_array().cloned().unwrap_or_default();

    let cs_resource_device_ids: Vec<String> = cs_resources
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    let cs_primary_found = cs_resource_device_ids.contains(&expected_primary_cs.to_string());

    // BC-2.06.020 PC-5: primary device must be present at stage 1.
    assert!(
        cs_primary_found,
        "Cross-DTU coherence: CrowdStrike response at stage 1 must contain primary device \
         '{}'; got device_ids: {:?}. BC-2.06.020 PC-5 / AC-015 \
         [RED GATE: StageMask projection absent OR timeline=None prevents scenario routing]",
        expected_primary_cs, cs_resource_device_ids
    );

    // BC-2.06.020 PC-5: at stage 1, lateral devices must be ABSENT.
    // StageMask at stage 1: primary_device=true, lateral_devices=false.
    // FAIL: without StageMask projection, ALL records served → lateral devices present.
    for lat_id in &catalog.lateral_device_ids_cs {
        assert!(
            !cs_resource_device_ids.contains(lat_id),
            "TV-015-cross-dtu: at stage 1 (Recon), lateral CS device '{}' must be ABSENT \
             from /devices/entities/devices/v2 response (StageMask lateral_devices=false); \
             found in {:?}. BC-2.06.020 PC-5 / AC-015 \
             [RED GATE: StageMask projection not implemented — lateral devices leak at stage 1]",
            lat_id,
            cs_resource_device_ids
        );
    }

    // -------------------------------------------------------------------------
    // Armis: GET /api/v1/devices
    // At stage 1 (Recon), primary device must be present in response.
    // -------------------------------------------------------------------------
    // Armis DTU uses DTU_DEFAULT_INSTANCE_ORG_ID as its instance_org_id (not the deadbeef org).
    // When X-Org-Id header is absent and instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID,
    // the validate-on-presence policy skips validation → request proceeds.
    // (See devices.rs dual-mode comment: "Header absent → guard skipped for default-instance.")
    let armis_resp = client
        .get(format!("{armis_url}/api/v1/devices"))
        .header("Authorization", format!("Bearer {armis_token}"))
        .send()
        .await
        .expect("Armis GET /api/v1/devices must reach the server");

    assert_eq!(
        armis_resp.status().as_u16(),
        200,
        "Cross-DTU coherence: Armis GET /api/v1/devices must return 200; got {}",
        armis_resp.status().as_u16()
    );

    let armis_body: serde_json::Value = armis_resp
        .json()
        .await
        .expect("Armis response must be JSON");
    let armis_devices = armis_body["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let armis_device_ids: Vec<String> = armis_devices
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    let armis_primary_found = armis_device_ids.contains(&expected_primary_armis.to_string());

    // BC-2.06.020 PC-5: primary device must be present at stage 1.
    assert!(
        armis_primary_found,
        "Cross-DTU coherence: Armis response at stage 1 must contain primary device \
         '{}'; got asset_ids: {:?}. BC-2.06.020 PC-5 / AC-015 \
         [RED GATE: StageMask projection absent OR timeline=None prevents scenario routing]",
        expected_primary_armis, armis_device_ids
    );

    // BC-2.06.020 PC-5: at stage 1, lateral devices must be ABSENT.
    // StageMask at stage 1: primary_device=true, lateral_devices=false.
    // FAIL: without StageMask projection, ALL records served → lateral devices present.
    for lat_id in &catalog.lateral_device_ids_armis {
        assert!(
            !armis_device_ids.contains(lat_id),
            "TV-015-cross-dtu: at stage 1 (Recon), lateral Armis device '{}' must be ABSENT \
             from /api/v1/devices response (StageMask lateral_devices=false at stage 1); \
             found in {:?}. BC-2.06.020 PC-5 / AC-015 \
             [RED GATE: StageMask projection not implemented — lateral devices leak at stage 1]",
            lat_id,
            armis_device_ids
        );
    }

    // -------------------------------------------------------------------------
    // Claroty: POST /api/v1/devices (empty body → returns all devices)
    // At stage 1 (Recon), primary device must be present; lateral devices must
    // be ABSENT (StageMask lateral_devices=false at stage 1).
    //
    // Cross-DTU coherence (BC-2.06.020 PC-5 / INV-CROSS-DTU-ENTITY-COHERENCE-001):
    // Claroty's canonical stage-gating key is `device_id` with the `dev-` prefix
    // (BC-3.4.004 TV-3.4.004-01/02/04). The `device_id` value EQUALS `primary_device_id_cs`
    // from the catalog — both are "dev-{slug}-{seed}-0" per ADR-036 §2.2. This is the
    // JOIN key that makes cross-DTU correlation possible.
    //
    // `asset_id` (format "ASSET-{slug}-{seed}-{index}") is an additive Claroty-specific
    // field present in every record, but it is NOT the coherence key.
    //
    // FAIL: without StageMask projection, ALL generated records are served →
    // lateral device IDs (`dev-deadbeef-100-1`, etc.) ARE in the response →
    // the lateral-absent assertion FAILS.
    // -------------------------------------------------------------------------
    let claroty_resp = client
        .post(format!("{claroty_url}/api/v1/devices"))
        .header("Authorization", format!("Bearer {claroty_token}"))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("Claroty POST /api/v1/devices must reach the server");

    assert_eq!(
        claroty_resp.status().as_u16(),
        200,
        "Cross-DTU coherence: Claroty POST /api/v1/devices must return 200; got {}",
        claroty_resp.status().as_u16()
    );

    let claroty_body: serde_json::Value = claroty_resp
        .json()
        .await
        .expect("Claroty response must be JSON");

    // Claroty response shape: {"devices": [...]} from the route handler.
    let claroty_devices = claroty_body["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // The coherence key is `device_id` = "dev-{org_slug}-{seed}-0" — same value as
    // `catalog.primary_device_id_cs`. This is what INV-CROSS-DTU-ENTITY-COHERENCE-001
    // requires: `id_cs ∈ Claroty.devices response (device_id field)`.
    // BC-2.06.020 PC-5 / BC-3.4.004 TV-3.4.004-01.
    let expected_primary_claroty_device_id = &catalog.primary_device_id_cs; // "dev-deadbeef-100-0"

    let claroty_device_ids: Vec<String> = claroty_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.020 PC-5 / AC-015: at stage 1, primary device must be present via device_id.
    // This is the load-bearing cross-DTU JOIN assertion: device_id must equal primary_device_id_cs.
    let claroty_primary_found =
        claroty_device_ids.contains(&expected_primary_claroty_device_id.to_string());

    assert!(
        claroty_primary_found,
        "Cross-DTU coherence: Claroty response at stage 1 must contain primary device \
         device_id='{}'; got device_ids: {:?}. BC-2.06.020 PC-5 / INV-CROSS-DTU-ENTITY-COHERENCE-001 \
         [RED GATE: StageMask projection absent OR route filters on ASSET- key instead of dev- key]",
        expected_primary_claroty_device_id, claroty_device_ids
    );

    // Verify asset_id is also present (additive Claroty-specific field, not the JOIN key).
    // BC-3.4.004: every record carries both device_id (dev-) and asset_id (ASSET-).
    let expected_primary_asset_id = format!("ASSET-{}-{}-0", catalog.org_slug, seed);
    let claroty_asset_ids: Vec<String> = claroty_devices
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();
    assert!(
        claroty_asset_ids.contains(&expected_primary_asset_id),
        "Claroty primary device record must also carry asset_id='{}' (additive field, BC-3.4.004); \
         got asset_ids: {:?}",
        expected_primary_asset_id, claroty_asset_ids
    );

    // BC-2.06.020 PC-5 / AC-015: at stage 1, lateral devices must be ABSENT.
    // StageMask at stage 1: primary_device=true, lateral_devices=false.
    // Lateral device_ids: "dev-deadbeef-100-1", "dev-deadbeef-100-2", "dev-deadbeef-100-3".
    // FAIL: without StageMask projection, ALL records are served → lateral devices
    //       ARE in the response → this assertion FAILS.
    for lat_id in &catalog.lateral_device_ids_cs {
        assert!(
            !claroty_device_ids.contains(lat_id),
            "TV-015-cross-dtu: at stage 1 (Recon), lateral Claroty device device_id='{}' must be ABSENT \
             from POST /api/v1/devices response (StageMask lateral_devices=false at stage 1); \
             found it in {:?}. BC-2.06.020 PC-5 / AC-015 \
             [RED GATE: StageMask projection not implemented — lateral devices leak at stage 1]",
            lat_id,
            claroty_device_ids
        );
    }

    // Stop all servers.
    cs_clone
        .stop()
        .await
        .expect("CrowdStrike stop must succeed");
    armis_clone.stop().await.expect("Armis stop must succeed");
    claroty_clone
        .stop()
        .await
        .expect("Claroty stop must succeed");
}

// ---------------------------------------------------------------------------
// NEW TEST (B-P1-03 test half) — test_BC_2_06_019_guard_order_e_demo_002_before_e_demo_004
//
// Architecture Compliance Rules (story spec line ~548):
//   "E-DEMO-004 guard order: seed-mismatch (E-DEMO-002) → bad-archetype (E-DEMO-003) →
//    missing-org_id (E-DEMO-004) — all before any constructor"
//
// This test constructs a config that trips BOTH E-DEMO-002 (mismatched seeds) AND
// E-DEMO-004 (missing org_id). The error returned must contain "E-DEMO-002" — not
// "E-DEMO-004" — proving the guard order is 002 → 004.
//
// FAIL mode: Current build_clone_pairs does not implement E-DEMO-002 guard. The
// E-DEMO-004 guard from Story A fires first (missing org_id check executes before
// the seed-mismatch check). The error returned contains "E-DEMO-004" instead of
// "E-DEMO-002". The assertion `err_str.contains("E-DEMO-002")` FAILS.
//
// NOTE: Even if build_clone_pairs currently returns Ok (because neither guard is
// implemented), the `result.is_err()` assertion FAILS first — which is a correct
// Red Gate failure.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_guard_order_e_demo_002_before_e_demo_004() {
    // Config trips BOTH E-DEMO-002 (seeds differ: 100 vs 200) AND E-DEMO-004
    // (both clones have org_id = None). Guard order: 002 must fire before 004.
    let config = make_cs_armis_scenario_missing_org(
        100, // crowdstrike seed
        200, // armis seed — different → E-DEMO-002
             // both have org_id = None → E-DEMO-004 if reached
    );

    // build_clone_pairs must return Err with E-DEMO-002 (not E-DEMO-004).
    // FAIL: E-DEMO-002 guard not yet implemented → either:
    //   (a) returns Ok (no guard at all) → result.is_err() assertion FAILS, OR
    //   (b) E-DEMO-004 fires first (wrong order) → err contains "E-DEMO-004" not "E-DEMO-002"
    //       → assertion err_str.contains("E-DEMO-002") FAILS.
    let result = build_clone_pairs(&config);

    assert!(
        result.is_err(),
        "Guard order test: build_clone_pairs with mismatched seeds + missing org_id must \
         return Err; got Ok. E-DEMO-002 guard (seed mismatch) must fire before E-DEMO-004 \
         (missing org_id). Architecture Compliance Rules line ~548 \
         [RED GATE: E-DEMO-002 guard not implemented — function returns Ok]"
    );

    let err = result.err().expect("verified is_err above");
    let err_str = err.to_string();

    // The error MUST be E-DEMO-002 (seed mismatch fires before E-DEMO-004 missing org_id).
    // FAIL: Current code emits E-DEMO-004 first (Story A's guard fires before E-DEMO-002).
    assert!(
        err_str.contains("E-DEMO-002"),
        "Guard order: when config trips both E-DEMO-002 (seeds 100 vs 200) and E-DEMO-004 \
         (org_id=None), the error must contain 'E-DEMO-002' (not 'E-DEMO-004'). \
         Spec order: seed-mismatch → bad-archetype → missing-org_id. \
         Got error: '{err_str}' \
         [RED GATE: E-DEMO-004 fires before E-DEMO-002 — guard order is wrong]"
    );

    // The error must NOT be E-DEMO-004 (that would indicate wrong guard order).
    assert!(
        !err_str.contains("E-DEMO-004"),
        "Guard order: error must be E-DEMO-002 (seed mismatch), not E-DEMO-004 \
         (missing org_id). Got error: '{err_str}'. \
         Implementer must reorder guards: E-DEMO-002 check BEFORE E-DEMO-004 check."
    );
}

// ---------------------------------------------------------------------------
// Helpers for test 18
// ---------------------------------------------------------------------------

/// Create a DemoConfig with only CrowdStrike enabled, scenario.enabled=true,
/// and the given scenario.archetype + fixture_set combination.
///
/// Used by test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction.
fn make_cs_with_archetype_and_fixture(
    org_id: &str,
    scenario_archetype: &str,
    fixture_set: &str,
) -> DemoConfig {
    let mut config = DemoConfig::default();
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = 42;
    config.clones.crowdstrike.org_id = Some(org_id.to_string());
    config.clones.crowdstrike.fixture_set = fixture_set.to_string();
    config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: scenario_archetype.to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });
    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.armis.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

// ---------------------------------------------------------------------------
// RED GATE TEST 18 — test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction
//
// BC-2.06.019 EC-019-012 / AC-017 (B-P3-01 ground-truth-verified MEDIUM)
//
// Two contradiction directions as named by EC-019-012:
//
// Direction 1 — archetype does not support scenario progression:
//   scenario.archetype = "healthy" (HealthyOtEnvironment) with scenario.enabled=true.
//   HealthyOtEnvironment does not support 5-stage IncidentTimeline.
//   build_clone_pairs must return Err(e) where e contains "E-DEMO-003".
//
// Direction 2 — archetype/fixture_set incoherence:
//   scenario.archetype = "compromised_endpoint" but fixture_set = "dormant"
//   (which maps to DormantTenant internally).
//   The fixture_set-derived archetype (DormantTenant) contradicts the declared
//   scenario archetype (CompromisedEndpoint). DormantTenant produces empty generated
//   records — driving a CompromisedEndpoint 5-stage timeline over empty data is incoherent.
//   build_clone_pairs must return Err(e) where e contains "E-DEMO-003".
//
// Guard position verification:
//   Both directions must fire AFTER E-DEMO-002 (seed mismatch) and BEFORE E-DEMO-004
//   (missing org_id). This test provides org_id=DEMO_ORG_UUID_DEADBEEF (no E-DEMO-004
//   trigger) and matching seeds (no E-DEMO-002 trigger), so the E-DEMO-003 guard is
//   isolated.
//
// FAIL mode (RED):
//   Current build_clone_pairs has no archetype/fixture_set contradiction check.
//   The function currently succeeds for both Direction 1 and Direction 2 —
//   Direction 1: "healthy" scenario.archetype is not validated against scenario.enabled;
//                build_clone_pairs uses fixture_set-derived archetype for the constructor,
//                ignoring scenario.archetype entirely → returns Ok instead of Err.
//   Direction 2: fixture_set="dormant" → fixture_set_to_archetype → DormantTenant;
//                scenario.archetype="compromised_endpoint" is never compared to DormantTenant;
//                build_clone_pairs uses DormantTenant for the constructor → returns Ok.
//   Assertion result.is_err() FAILS in both directions.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction() {
    // Direction 1: scenario.archetype = "healthy" with scenario.enabled = true.
    // HealthyOtEnvironment does not support 5-stage scenario progression.
    // fixture_set = "default" maps to HealthyOtEnvironment — both sides agree on the
    // archetype, but "healthy" does not support IncidentTimeline progression.
    // build_clone_pairs must reject this with E-DEMO-003.
    let config_healthy = make_cs_with_archetype_and_fixture(
        DEMO_ORG_UUID_DEADBEEF,
        "healthy", // scenario archetype: HealthyOtEnvironment (does not support progression)
        "default", // fixture_set: also maps to HealthyOtEnvironment
    );

    // RED: E-DEMO-003 archetype-not-supporting-progression guard not implemented.
    // build_clone_pairs currently returns Ok — this assertion will FAIL.
    let result_d1 = build_clone_pairs(&config_healthy);

    assert!(
        result_d1.is_err(),
        "EC-019-012 Direction 1: build_clone_pairs must return Err when \
         scenario.archetype='healthy' (HealthyOtEnvironment) with scenario.enabled=true; \
         HealthyOtEnvironment does not support 5-stage IncidentTimeline. Got Ok. \
         E-DEMO-003 guard not yet implemented in build_clone_pairs. \
         BC-2.06.019 EC-019-012 / AC-017 [RED GATE: expected Err, got Ok]"
    );

    let err_d1 = result_d1.err().expect("verified is_err above");
    let err_str_d1 = err_d1.to_string();

    assert!(
        err_str_d1.contains("E-DEMO-003"),
        "EC-019-012 Direction 1: error must contain 'E-DEMO-003'; got: '{err_str_d1}' \
         — BC-2.06.019 EC-019-012"
    );

    assert!(
        err_str_d1.contains("crowdstrike"),
        "EC-019-012 Direction 1: error must name the failing clone; got: '{err_str_d1}'"
    );

    // Direction 2: scenario.archetype = "compromised_endpoint" but fixture_set = "dormant"
    // (maps to DormantTenant internally). The two sides disagree:
    //   scenario.archetype-derived archetype = CompromisedEndpoint
    //   fixture_set-derived archetype        = DormantTenant
    // This is an incoherent combination — build_clone_pairs must reject with E-DEMO-003.
    let config_contradiction = make_cs_with_archetype_and_fixture(
        DEMO_ORG_UUID_DEADBEEF,
        "compromised_endpoint", // scenario archetype: CompromisedEndpoint
        "dormant",              // fixture_set: maps to DormantTenant — MISMATCH
    );

    // RED: E-DEMO-003 archetype/fixture_set contradiction guard not implemented.
    // build_clone_pairs currently ignores scenario.archetype and uses fixture_set-derived
    // DormantTenant for the constructor → returns Ok. This assertion will FAIL.
    let result_d2 = build_clone_pairs(&config_contradiction);

    assert!(
        result_d2.is_err(),
        "EC-019-012 Direction 2: build_clone_pairs must return Err when \
         scenario.archetype='compromised_endpoint' but fixture_set='dormant' (→ DormantTenant); \
         the fixture_set-derived archetype (DormantTenant) contradicts the declared scenario \
         archetype (CompromisedEndpoint). Got Ok. E-DEMO-003 guard not yet implemented. \
         BC-2.06.019 EC-019-012 / AC-017 [RED GATE: expected Err, got Ok]"
    );

    let err_d2 = result_d2.err().expect("verified is_err above");
    let err_str_d2 = err_d2.to_string();

    assert!(
        err_str_d2.contains("E-DEMO-003"),
        "EC-019-012 Direction 2: error must contain 'E-DEMO-003'; got: '{err_str_d2}' \
         — BC-2.06.019 EC-019-012"
    );

    assert!(
        err_str_d2.contains("crowdstrike"),
        "EC-019-012 Direction 2: error must name the failing clone; got: '{err_str_d2}'"
    );

    // Guard position check: verify E-DEMO-003 fires BEFORE E-DEMO-004.
    // Config has org_id set (no E-DEMO-004) and single clone with matching seeds
    // (no E-DEMO-002), so the errors above must purely come from the E-DEMO-003 guard.
    // This implicitly verifies guard ordering: neither E-DEMO-002 nor E-DEMO-004
    // masked the E-DEMO-003 error.
    assert!(
        !err_str_d1.contains("E-DEMO-004"),
        "EC-019-012 Direction 1: E-DEMO-003 must fire without E-DEMO-004 masking it; \
         org_id is provided so E-DEMO-004 must NOT appear. Got: '{err_str_d1}'"
    );
    assert!(
        !err_str_d2.contains("E-DEMO-004"),
        "EC-019-012 Direction 2: E-DEMO-003 must fire without E-DEMO-004 masking it; \
         org_id is provided so E-DEMO-004 must NOT appear. Got: '{err_str_d2}'"
    );
}

// ---------------------------------------------------------------------------
// NEW TEST (B-P4-01) — test_BC_2_06_019_guard_order_e_demo_003_before_e_demo_004
//
// Architecture Compliance Rules (story spec AC-017 + guard order):
//   E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004, all before any constructor.
//
// This is the SIBLING of the B-P1-03 fix (which covered 002→004 order).
// B-P4-01 covers the 003→004 slot: E-DEMO-003 must fire BEFORE E-DEMO-004
// even when org_id is missing.
//
// Two variants to exercise both E-DEMO-003 directions (unrecognized archetype,
// archetype/fixture_set contradiction) each paired with org_id=None.
//
// Reachable violation path (pre-fix):
//   scenario.enabled=true + scenario.archetype="bogus" + fixture_set="compromised" +
//   org_id=None → validated_gen loop calls require_org_id → E-DEMO-004 fires;
//   scenario_ctx E-DEMO-003 check never reached.
//
// FAIL mode (RED):
//   Variant A: archetype="bogus" + org_id=None → E-DEMO-004 fires first (wrong order).
//              err contains "E-DEMO-004" and assertion err_str.contains("E-DEMO-003") FAILS.
//   Variant B: archetype="compromised_endpoint" + fixture_set="dormant" + org_id=None →
//              validated_gen: fixture_set_to_archetype("dormant") → DormantTenant;
//              DormantTenant != HealthyOtEnvironment → require_org_id fires E-DEMO-004.
//              E-DEMO-003 contradiction check never reached.
//
// BC-2.06.019 EC-019-012 / AC-017 / Architecture Compliance Rules guard order.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_guard_order_e_demo_003_before_e_demo_004() {
    // Variant A: scenario.archetype = "bogus" (unrecognized) + org_id = None.
    // E-DEMO-003 (unrecognized archetype) must fire before E-DEMO-004 (missing org_id).
    // Pre-fix: validated_gen hits require_org_id for CompromisedEndpoint (from fixture_set
    // "compromised") with org_id=None → E-DEMO-004 fires before the scenario_ctx block.
    let config_a = {
        let mut config = DemoConfig::default();
        config.clones.crowdstrike.enabled = true;
        config.clones.crowdstrike.seed = 42;
        config.clones.crowdstrike.org_id = None; // missing → E-DEMO-004 if order is wrong
        config.clones.crowdstrike.fixture_set = "compromised".to_string(); // → CompromisedEndpoint
        config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
            enabled: true,
            archetype: "bogus_archetype_string".to_string(), // unrecognized → E-DEMO-003
            scenario_start_secs: None,
            stage_duration_secs: vec![],
        });
        config.clones.claroty.enabled = false;
        config.clones.cyberint.enabled = false;
        config.clones.armis.enabled = false;
        config.clones.threatintel.enabled = false;
        config.clones.nvd.enabled = false;
        config
    };

    // build_clone_pairs must return Err. The error must be E-DEMO-003, not E-DEMO-004.
    // FAIL: pre-fix code emits E-DEMO-004 (validated_gen/require_org_id fires first).
    let result_a = build_clone_pairs(&config_a);

    assert!(
        result_a.is_err(),
        "B-P4-01 Variant A: build_clone_pairs must return Err when scenario.archetype='bogus' \
         (E-DEMO-003) + org_id=None (E-DEMO-004); got Ok. \
         Architecture Compliance Rules: E-DEMO-003 must fire before E-DEMO-004. \
         [RED GATE: function returns Ok — neither guard reached]"
    );

    let err_a = result_a.err().expect("verified is_err above");
    let err_str_a = err_a.to_string();

    // FAIL: pre-fix err_str_a contains "E-DEMO-004" (fired by validated_gen), not "E-DEMO-003".
    assert!(
        err_str_a.contains("E-DEMO-003"),
        "B-P4-01 Variant A: guard order E-DEMO-003 before E-DEMO-004 — error must contain \
         'E-DEMO-003' (unrecognized archetype 'bogus_archetype_string'); got: '{err_str_a}'. \
         Pre-fix: validated_gen/require_org_id fires E-DEMO-004 first. \
         Architecture Compliance Rules: 002→003→004 order required. \
         [RED GATE: E-DEMO-004 fires before E-DEMO-003 — guard order wrong]"
    );

    assert!(
        !err_str_a.contains("E-DEMO-004"),
        "B-P4-01 Variant A: error must NOT contain 'E-DEMO-004' — E-DEMO-003 must fire first; \
         got: '{err_str_a}'"
    );

    // Variant B: scenario.archetype="compromised_endpoint" + fixture_set="dormant" +
    // org_id=None. Direction 2 contradiction (DormantTenant vs CompromisedEndpoint) +
    // missing org_id. E-DEMO-003 must fire before E-DEMO-004.
    // Pre-fix: validated_gen: fixture_set_to_archetype("dormant") → DormantTenant;
    // DormantTenant != HealthyOtEnvironment → require_org_id fires E-DEMO-004.
    // The scenario_ctx contradiction check never runs.
    let config_b = {
        let mut config = DemoConfig::default();
        config.clones.crowdstrike.enabled = true;
        config.clones.crowdstrike.seed = 42;
        config.clones.crowdstrike.org_id = None; // missing → E-DEMO-004 if order is wrong
        config.clones.crowdstrike.fixture_set = "dormant".to_string(); // → DormantTenant
        config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
            enabled: true,
            archetype: "compromised_endpoint".to_string(), // contradicts DormantTenant → E-DEMO-003
            scenario_start_secs: None,
            stage_duration_secs: vec![],
        });
        config.clones.claroty.enabled = false;
        config.clones.cyberint.enabled = false;
        config.clones.armis.enabled = false;
        config.clones.threatintel.enabled = false;
        config.clones.nvd.enabled = false;
        config
    };

    // build_clone_pairs must return Err. The error must be E-DEMO-003, not E-DEMO-004.
    // FAIL: pre-fix code emits E-DEMO-004 from validated_gen (DormantTenant + org_id=None
    // triggers require_org_id) before the scenario_ctx contradiction check runs.
    let result_b = build_clone_pairs(&config_b);

    assert!(
        result_b.is_err(),
        "B-P4-01 Variant B: build_clone_pairs must return Err when \
         scenario.archetype='compromised_endpoint' + fixture_set='dormant' (→ DormantTenant, \
         Direction 2 contradiction) + org_id=None; got Ok. \
         Architecture Compliance Rules: E-DEMO-003 must fire before E-DEMO-004. \
         [RED GATE: function returns Ok — neither guard reached]"
    );

    let err_b = result_b.err().expect("verified is_err above");
    let err_str_b = err_b.to_string();

    // FAIL: pre-fix err_str_b contains "E-DEMO-004" (validated_gen fires it), not "E-DEMO-003".
    assert!(
        err_str_b.contains("E-DEMO-003"),
        "B-P4-01 Variant B: guard order E-DEMO-003 before E-DEMO-004 — error must contain \
         'E-DEMO-003' (Direction 2: DormantTenant contradicts CompromisedEndpoint scenario); \
         got: '{err_str_b}'. Pre-fix: validated_gen fires E-DEMO-004 first (DormantTenant + \
         org_id=None). Architecture Compliance Rules: 002→003→004 order required. \
         [RED GATE: E-DEMO-004 fires before E-DEMO-003 — guard order wrong]"
    );

    assert!(
        !err_str_b.contains("E-DEMO-004"),
        "B-P4-01 Variant B: error must NOT contain 'E-DEMO-004' — E-DEMO-003 must fire first; \
         got: '{err_str_b}'"
    );
}

// ---------------------------------------------------------------------------
// RED GATE TEST 19 — test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones
//
// BC-2.06.019 PRE-6 / E-DEMO-006 / EC-019-013 / TV-019-015 / VP-019-I
//
// Verifies: build_clone_pairs returns Err containing "E-DEMO-006" when two
// scenario-enabled clones have the same seed but different org_id values.
//
// Guard order assertion: E-DEMO-006 fires BEFORE E-DEMO-003 and E-DEMO-004,
// and AFTER E-DEMO-002. The main variant has valid seed parity (no E-DEMO-002
// trigger) and valid archetypes (no E-DEMO-003 trigger) so E-DEMO-006 is
// isolated. A second variant additionally trips E-DEMO-004 (missing org_id on
// a third clone or org_id=None on one of the two mismatched clones) — omitted
// here because the two clones already have present org_ids; order relative to
// E-DEMO-004 is validated by the None-semantics encoding described in the body.
//
// None-org_id semantics (BC-2.06.019 PRE-6 + BC guard order):
//   The E-DEMO-006 guard compares org_ids ONLY for scenario-enabled clones
//   where org_id is Some(_). If a clone has org_id=None, that clone does NOT
//   participate in the E-DEMO-006 equality check (its absent org_id falls
//   through to E-DEMO-004 later, per the canonical guard order
//   E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004).
//   Rationale: the mismatch check is about catalog derivation coherence — you
//   can only compare two org_ids when both are present. An absent org_id is a
//   distinct class of error (E-DEMO-004) and must not be conflated with a
//   present-but-mismatched org_id (E-DEMO-006).
//
// FAIL mode (RED): E-DEMO-006 guard not yet implemented in build_clone_pairs.
//   build_clone_pairs currently returns Ok for same-seed / different-org_id
//   configs (the org_id mismatch is silently accepted).
//   Assertion result.is_err() FAILS.
// ---------------------------------------------------------------------------

/// Second org UUID with first 4 bytes [0xca, 0xfe, 0xba, 0xbe] → org_slug = "cafebabe".
/// Used as the mismatched org_id in TV-019-015.
const DEMO_ORG_UUID_CAFEBABE: &str = "cafebabe-0000-7000-8000-000000000000";

/// Same UUID as DEMO_ORG_UUID_DEADBEEF but in uppercase hex — byte-identical after
/// Uuid::parse_str, distinct raw string.  Used by B-P5-05 case-variant test.
const DEMO_ORG_UUID_DEADBEEF_UPPER: &str = "DEADBEEF-0000-7000-8000-000000000000";

/// Create a DemoConfig with CrowdStrike + Armis enabled, same seed,
/// but different org_ids — the E-DEMO-006 test vector (TV-019-015).
fn make_cs_armis_same_seed_different_org(
    seed: u64,
    cs_org_id: &str,
    armis_org_id: &str,
) -> DemoConfig {
    let mut config = DemoConfig::default();

    // CrowdStrike
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = seed;
    config.clones.crowdstrike.org_id = Some(cs_org_id.to_string());
    config.clones.crowdstrike.fixture_set = "compromised".to_string();
    config.clones.crowdstrike.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    // Armis — same seed, DIFFERENT org_id → E-DEMO-006
    config.clones.armis.enabled = true;
    config.clones.armis.seed = seed;
    config.clones.armis.org_id = Some(armis_org_id.to_string());
    config.clones.armis.fixture_set = "compromised".to_string();
    config.clones.armis.scenario = Some(prism_dtu_demo_server::config::ScenarioConfig {
        enabled: true,
        archetype: "compromised_endpoint".to_string(),
        scenario_start_secs: None,
        stage_duration_secs: vec![],
    });

    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones() {
    // -------------------------------------------------------------------------
    // Variant A (TV-019-015): same seed=100, CrowdStrike org_id=deadbeef, Armis org_id=cafebabe.
    // Both org_ids are present. E-DEMO-006 must fire before any clone is constructed.
    // -------------------------------------------------------------------------
    let config = make_cs_armis_same_seed_different_org(
        100,                    // same seed — no E-DEMO-002
        DEMO_ORG_UUID_DEADBEEF, // CS org_id
        DEMO_ORG_UUID_CAFEBABE, // Armis org_id — DIFFERENT → E-DEMO-006
    );

    // FAIL: E-DEMO-006 guard not implemented → function returns Ok.
    let result = build_clone_pairs(&config);

    assert!(
        result.is_err(),
        "TV-019-015 Variant A: build_clone_pairs must return Err when two scenario-enabled \
         clones have the same seed but different org_ids \
         (crowdstrike={DEMO_ORG_UUID_DEADBEEF}, armis={DEMO_ORG_UUID_CAFEBABE}); \
         got Ok — E-DEMO-006 guard not yet implemented. \
         BC-2.06.019 PRE-6 / EC-019-013 / VP-019-I \
         [RED GATE: expected Err, got Ok]"
    );

    let err = result.err().expect("verified is_err above");
    let err_str = err.to_string();

    // Error must contain the E-DEMO-006 code.
    assert!(
        err_str.contains("E-DEMO-006"),
        "TV-019-015: error must contain 'E-DEMO-006'; got: '{err_str}' \
         — BC-2.06.019 PRE-6 / VP-019-I"
    );

    // Error must name both clone types involved.
    assert!(
        err_str.contains("crowdstrike") && err_str.contains("armis"),
        "TV-019-015: E-DEMO-006 error must name both clones ('crowdstrike' and 'armis'); \
         got: '{err_str}'"
    );

    // Error must include both org_id values (UUID strings are safe to echo per AD-017 §config).
    assert!(
        err_str.contains(DEMO_ORG_UUID_DEADBEEF) && err_str.contains(DEMO_ORG_UUID_CAFEBABE),
        "TV-019-015: E-DEMO-006 error must include both conflicting org_id values \
         ('{DEMO_ORG_UUID_DEADBEEF}' and '{DEMO_ORG_UUID_CAFEBABE}'); got: '{err_str}'"
    );

    // Guard order: E-DEMO-006 fires, not E-DEMO-002 (seeds match) or E-DEMO-003/004.
    assert!(
        !err_str.contains("E-DEMO-002"),
        "TV-019-015: seeds match (both=100) so E-DEMO-002 must NOT appear; got: '{err_str}'"
    );
    assert!(
        !err_str.contains("E-DEMO-003"),
        "TV-019-015: archetypes are valid so E-DEMO-003 must NOT appear; got: '{err_str}'"
    );
    assert!(
        !err_str.contains("E-DEMO-004"),
        "TV-019-015: both org_ids are present so E-DEMO-004 must NOT appear; got: '{err_str}'"
    );

    // -------------------------------------------------------------------------
    // Variant B (ordering guard): E-DEMO-006 fires BEFORE E-DEMO-003 and E-DEMO-004.
    // Config: same seed, different org_ids, and ALSO an invalid archetype for one clone.
    // Both E-DEMO-006 and E-DEMO-003 conditions are triggered. E-DEMO-006 must win
    // because its guard runs first (canonical order: 002 → 006 → 003 → 004).
    // -------------------------------------------------------------------------
    let mut config_b =
        make_cs_armis_same_seed_different_org(100, DEMO_ORG_UUID_DEADBEEF, DEMO_ORG_UUID_CAFEBABE);
    // Corrupt the armis archetype to also trigger E-DEMO-003 (if reached).
    if let Some(sc) = config_b.clones.armis.scenario.as_mut() {
        sc.archetype = "bogus_archetype_that_triggers_003".to_string();
    }

    // FAIL (same as Variant A): guard not implemented.
    let result_b = build_clone_pairs(&config_b);

    assert!(
        result_b.is_err(),
        "TV-019-015 Variant B (order guard): build_clone_pairs must return Err; got Ok. \
         E-DEMO-006 must fire before E-DEMO-003 even when both conditions are triggered. \
         BC-2.06.019 PRE-6 guard order [RED GATE: guard not implemented]"
    );

    let err_b = result_b.err().expect("verified is_err above");
    let err_str_b = err_b.to_string();

    // E-DEMO-006 must win over E-DEMO-003.
    assert!(
        err_str_b.contains("E-DEMO-006"),
        "TV-019-015 Variant B: guard order — E-DEMO-006 (org_id mismatch) must fire before \
         E-DEMO-003 (bad archetype); error must contain 'E-DEMO-006'; got: '{err_str_b}' \
         — BC-2.06.019 PRE-6 canonical guard order: 002→006→003→004"
    );
    assert!(
        !err_str_b.contains("E-DEMO-003"),
        "TV-019-015 Variant B: E-DEMO-006 fires before E-DEMO-003; error must NOT contain \
         'E-DEMO-003'; got: '{err_str_b}'"
    );
}

// ---------------------------------------------------------------------------
// B-P5-05 — test_BC_2_06_019_e_demo_006_case_variant_org_ids_succeed
//
// The E-DEMO-006 guard compares org_ids by PARSED UUID bytes, not raw strings.
// Two org_id strings that differ only in hex case ("deadbeef-..." vs "DEADBEEF-...")
// parse to identical Uuid bytes → same OrgId → NO E-DEMO-006 false-positive.
//
// FAIL mode (RED): current guard uses raw string comparison (`*org_id != first_org_id`),
//   so DEADBEEF-... and deadbeef-... (same bytes) are treated as different → E-DEMO-006
//   fires falsely → build_clone_pairs returns Err instead of Ok.
//   Assertion `result.is_ok()` FAILS.
//
// BC-2.06.019 PRE-6: the guard's invariant is slug/byte-based (rationale comment in
//   harness.rs ~398-403). Raw-string comparison violates that invariant.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_019_e_demo_006_case_variant_org_ids_succeed() {
    // Same UUID, different string case:
    //   crowdstrike org_id = "deadbeef-0000-7000-8000-000000000000" (lower)
    //   armis       org_id = "DEADBEEF-0000-7000-8000-000000000000" (upper)
    // Both parse to the same Uuid bytes → same OrgId → E-DEMO-006 must NOT fire.
    let config = make_cs_armis_same_seed_different_org(
        100,
        DEMO_ORG_UUID_DEADBEEF,       // lowercase
        DEMO_ORG_UUID_DEADBEEF_UPPER, // uppercase — byte-identical after parse
    );

    // build_clone_pairs must SUCCEED (no E-DEMO-006 false-positive).
    // FAIL (RED): raw-string comparison treats these as different → Err("E-DEMO-006").
    let result = build_clone_pairs(&config);

    assert!(
        result.is_ok(),
        "B-P5-05: build_clone_pairs must return Ok when two scenario-enabled clones share \
         the same UUID in different case forms \
         ('{DEMO_ORG_UUID_DEADBEEF}' vs '{DEMO_ORG_UUID_DEADBEEF_UPPER}'); \
         raw-string comparison produces a false-positive E-DEMO-006. \
         The guard invariant is byte-based (parsed Uuid), not string-based. \
         BC-2.06.019 PRE-6 \
         [RED GATE: E-DEMO-006 fires for case-variant of same UUID — false positive]"
    );
}

// ---------------------------------------------------------------------------
// F-CLARO-LIVE-P1-MED-001 — test_BC_2_06_019_stage_mask_device_alert_relations_scenario_path
//
// Coverage hardening: the StageMask arm of `list_device_alert_relations`
// (state.fixture_gen_seeded && timeline.is_some() branch) was ACTIVE in the
// BC-2.06.019 Route Coverage Table but reached by no test end-to-end from the
// HTTP surface.
//
// Enforces INV-CROSS-DTU-ENTITY-COHERENCE-001: a relation referencing a device
// that is hidden at the current stage MUST also be hidden from devices_alerts.
// Uses `_device_id` (stage-gating key stamped by make_device_alert_relation,
// present in the serialized wire response) to identify which device each
// relation belongs to.
//
// Stage plan (default thresholds [60, 180, 360, 600]s):
//   stage 0 (Baseline,  elapsed ≈ 0s):   primary relation WITHHELD
//                                         (`mask.primary_device && stage_idx > 0` = false)
//   stage 1 (Recon,     elapsed ≈ 90s):   primary PRESENT, lateral ABSENT
//                                         (mask.primary_device=true, mask.lateral_devices=false)
//   stage 2 (Lateral,   elapsed ≈ 270s):  lateral PRESENT
//                                         (mask.lateral_devices=true)
//
// Wire-shape discipline: every assertion reads from the parsed JSON response
// (`devices_alerts` array), NOT from pre-serialization Rust structures.
//
// This test is expected to PASS on arrival — the arm code is believed correct
// (it mirrors list_devices). If it FAILS that means the arm has a real
// entity-coherence bug (finding would escalate; production code MUST NOT be
// modified to make it pass — stop and report).
//
// BC-2.06.019 PC-4 / INV-CROSS-DTU-ENTITY-COHERENCE-001 / ADR-036 v2.3 §2.4
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[tokio::test]
async fn test_BC_2_06_019_stage_mask_device_alert_relations_scenario_path() {
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_common::{
        build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
        OrgId,
    };
    use std::sync::Arc;

    let seed: u64 = 100;
    let org_uuid = uuid::Uuid::parse_str(DEMO_ORG_UUID_DEADBEEF)
        .expect("DEMO_ORG_UUID_DEADBEEF must be valid UUID");
    let org_id = OrgId(*org_uuid.as_bytes());

    let now = chrono::Utc::now().timestamp();

    // Clock control: three distinct scenario_start values place the wall-clock
    // at stages 0, 1, and 2 respectively.
    //   stage 0: elapsed = 0  → stage_idx = 0  (Baseline)
    //   stage 1: elapsed ≈ 90 → stage_idx = 1  (Recon, threshold 60s)
    //   stage 2: elapsed ≈ 270 → stage_idx = 2 (LateralMovement, threshold 180s)
    let start_stage0: i64 = now;
    let start_stage1: i64 = now - 90;
    let start_stage2: i64 = now - 270;

    // Build the entity catalog once; share across all three timelines so the
    // primary/lateral IDs are identical across stages (same seed + org).
    let catalog = build_scenario_entity_catalog(seed, &org_id);
    let primary_id = catalog.primary_device_id_cs.clone(); // "dev-deadbeef-100-0"
    let lateral_ids = catalog.lateral_device_ids_cs.clone(); // ["dev-deadbeef-100-1/2/3"]

    let client = prism_dtu_common::build_test_client();

    // -----------------------------------------------------------------------
    // Stage 0 (Baseline) — primary relation WITHHELD
    //
    // `stage_idx = 0` → gate `mask.primary_device && stage_idx > 0` = false →
    // relation whose `_device_id == primary_id` must be absent from devices_alerts.
    // -----------------------------------------------------------------------
    let time_anchor_s0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("start_stage0 must be a valid Unix timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_s0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[], // → default thresholds [60, 180, 360, 600]
    ));

    let mut clone_s0 = ClarotyClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline_s0),
        time_anchor_s0,
    );

    // Confirm scenario path is active (timeline wired).
    assert!(
        clone_s0.state.timeline.is_some(),
        "F-CLARO-LIVE-P1-MED-001: stage-0 ClarotyClone must have timeline = Some \
         after new_with_scenario; got None — scenario path is inactive. \
         BC-2.06.019 PC-4"
    );

    clone_s0
        .start()
        .await
        .expect("stage-0 ClarotyClone start must succeed");
    let url_s0 = clone_s0.base_url();
    let token_s0 = clone_s0.admin_token().to_owned();

    let resp_s0 = client
        .post(format!("{url_s0}/api/v1/device_alert_relations"))
        .header("Authorization", format!("Bearer {token_s0}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/device_alert_relations at stage 0 must reach the server");

    assert_eq!(
        resp_s0.status().as_u16(),
        200,
        "Stage-0 POST /api/v1/device_alert_relations must return HTTP 200"
    );

    let body_s0: serde_json::Value = resp_s0
        .json()
        .await
        .expect("stage-0 response body must be valid JSON");

    // Wire-shape assertion: parse the serialized envelope.
    let entries_s0 = body_s0
        .get("devices_alerts")
        .and_then(|v| v.as_array())
        .expect(
            "stage-0 response MUST contain `devices_alerts` JSON array \
             (wire-shape, BC-2.06.019 AC-003 / response_path = '$.devices_alerts')",
        );

    // INV-CROSS-DTU-ENTITY-COHERENCE-001 / BC-2.06.019 PC-4:
    // At stage 0, `stage_idx > 0` = false → primary relation withheld.
    // Serialized wire check: no entry may carry `_device_id == primary_id`.
    let stage0_primary_entries: Vec<&serde_json::Value> = entries_s0
        .iter()
        .filter(|e| {
            e.get("_device_id")
                .and_then(|v| v.as_str())
                .map(|id| id == primary_id.as_str())
                .unwrap_or(false)
        })
        .collect();

    assert!(
        stage0_primary_entries.is_empty(),
        "F-CLARO-LIVE-P1-MED-001 / INV-CROSS-DTU-ENTITY-COHERENCE-001: \
         at stage 0 (Baseline), relation with _device_id='{}' must be WITHHELD \
         from devices_alerts — gate `mask.primary_device && stage_idx > 0` = false \
         (stage_idx=0). Found {} matching entries: {:?}. \
         A non-empty result means the StageMask arm is not filtering correctly; \
         this would be a silent entity-coherence leak at stage 0. \
         BC-2.06.019 PC-4",
        primary_id,
        stage0_primary_entries.len(),
        stage0_primary_entries
    );

    clone_s0
        .stop()
        .await
        .expect("stage-0 ClarotyClone stop must succeed");

    // -----------------------------------------------------------------------
    // Stage 1 (Recon) — primary PRESENT, laterals ABSENT
    //
    // `stage_idx = 1` → gate = `mask.primary_device=true && 1 > 0` = true → present.
    // `mask.lateral_devices=false` at stage 1 → lateral relations absent.
    // Mirrors the lateral-absent assertion in the Claroty devices section of
    // test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones.
    // -----------------------------------------------------------------------
    let time_anchor_s1 = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("start_stage1 must be a valid Unix timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_s1 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));

    let mut clone_s1 = ClarotyClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline_s1),
        time_anchor_s1,
    );

    assert!(
        clone_s1.state.timeline.is_some(),
        "F-CLARO-LIVE-P1-MED-001: stage-1 ClarotyClone must have timeline = Some; \
         got None — BC-2.06.019 PC-4"
    );

    clone_s1
        .start()
        .await
        .expect("stage-1 ClarotyClone start must succeed");
    let url_s1 = clone_s1.base_url();
    let token_s1 = clone_s1.admin_token().to_owned();

    let resp_s1 = client
        .post(format!("{url_s1}/api/v1/device_alert_relations"))
        .header("Authorization", format!("Bearer {token_s1}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/device_alert_relations at stage 1 must reach the server");

    assert_eq!(
        resp_s1.status().as_u16(),
        200,
        "Stage-1 POST /api/v1/device_alert_relations must return HTTP 200"
    );

    let body_s1: serde_json::Value = resp_s1
        .json()
        .await
        .expect("stage-1 response body must be valid JSON");

    let entries_s1 = body_s1
        .get("devices_alerts")
        .and_then(|v| v.as_array())
        .expect(
            "stage-1 response MUST contain `devices_alerts` JSON array \
             (wire-shape, response_path = '$.devices_alerts')",
        );

    // Primary PRESENT at stage 1.
    let stage1_primary_entries: Vec<&serde_json::Value> = entries_s1
        .iter()
        .filter(|e| {
            e.get("_device_id")
                .and_then(|v| v.as_str())
                .map(|id| id == primary_id.as_str())
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !stage1_primary_entries.is_empty(),
        "F-CLARO-LIVE-P1-MED-001 / INV-CROSS-DTU-ENTITY-COHERENCE-001: \
         at stage 1 (Recon), relation with _device_id='{}' must be PRESENT in \
         devices_alerts — gate `mask.primary_device=true && stage_idx=1 > 0` = true. \
         Got empty. If the arm is broken, a hidden primary relation would allow a \
         malicious actor to escape cross-DTU correlation silently. \
         BC-2.06.019 PC-4",
        primary_id
    );

    // Laterals ABSENT at stage 1 (mask.lateral_devices=false at Recon).
    for lat_id in &lateral_ids {
        let lat_present = entries_s1.iter().any(|e| {
            e.get("_device_id")
                .and_then(|v| v.as_str())
                .map(|id| id == lat_id.as_str())
                .unwrap_or(false)
        });
        assert!(
            !lat_present,
            "F-CLARO-LIVE-P1-MED-001: at stage 1 (Recon), lateral relation with \
             _device_id='{}' must be WITHHELD (mask.lateral_devices=false at stage 1); \
             found it in devices_alerts. INV-CROSS-DTU-ENTITY-COHERENCE-001 / BC-2.06.019 PC-4",
            lat_id
        );
    }

    clone_s1
        .stop()
        .await
        .expect("stage-1 ClarotyClone stop must succeed");

    // -----------------------------------------------------------------------
    // Stage 2 (LateralMovement) — laterals PRESENT
    //
    // `mask.lateral_devices=true` at stage 2 → lateral relations surfaced.
    // -----------------------------------------------------------------------
    let time_anchor_s2 = chrono::DateTime::from_timestamp(start_stage2, 0)
        .expect("start_stage2 must be a valid Unix timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_s2 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage2,
        &[],
    ));

    let mut clone_s2 = ClarotyClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline_s2),
        time_anchor_s2,
    );

    assert!(
        clone_s2.state.timeline.is_some(),
        "F-CLARO-LIVE-P1-MED-001: stage-2 ClarotyClone must have timeline = Some; \
         got None — BC-2.06.019 PC-4"
    );

    clone_s2
        .start()
        .await
        .expect("stage-2 ClarotyClone start must succeed");
    let url_s2 = clone_s2.base_url();
    let token_s2 = clone_s2.admin_token().to_owned();

    let resp_s2 = client
        .post(format!("{url_s2}/api/v1/device_alert_relations"))
        .header("Authorization", format!("Bearer {token_s2}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/device_alert_relations at stage 2 must reach the server");

    assert_eq!(
        resp_s2.status().as_u16(),
        200,
        "Stage-2 POST /api/v1/device_alert_relations must return HTTP 200"
    );

    let body_s2: serde_json::Value = resp_s2
        .json()
        .await
        .expect("stage-2 response body must be valid JSON");

    let entries_s2 = body_s2
        .get("devices_alerts")
        .and_then(|v| v.as_array())
        .expect(
            "stage-2 response MUST contain `devices_alerts` JSON array \
             (wire-shape, response_path = '$.devices_alerts')",
        );

    // Laterals PRESENT at stage 2 (mask.lateral_devices=true at LateralMovement).
    for lat_id in &lateral_ids {
        let lat_present = entries_s2.iter().any(|e| {
            e.get("_device_id")
                .and_then(|v| v.as_str())
                .map(|id| id == lat_id.as_str())
                .unwrap_or(false)
        });
        assert!(
            lat_present,
            "F-CLARO-LIVE-P1-MED-001: at stage 2 (LateralMovement), lateral relation with \
             _device_id='{}' must be PRESENT in devices_alerts \
             (mask.lateral_devices=true at stage 2); got absent. \
             INV-CROSS-DTU-ENTITY-COHERENCE-001 / BC-2.06.019 PC-4",
            lat_id
        );
    }

    clone_s2
        .stop()
        .await
        .expect("stage-2 ClarotyClone stop must succeed");
}
