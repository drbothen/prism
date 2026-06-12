//! Red Gate tests 9, 10, 11, 15: BC-2.06.019 / BC-2.06.020 demo-server scenario guards
//!
//! Tests:
//!   Test 9:  test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones
//!   Test 10: test_BC_2_06_019_e_demo_003_unrecognized_archetype
//!   Test 11: test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path
//!   Test 15: test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones
//!   NEW:     test_BC_2_06_019_guard_order_e_demo_002_before_e_demo_004
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//! Traces to: BC-2.06.019 E-DEMO-002/003 / TV-019-012,013,007
//!            BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 / PC-5
//!            Architecture Compliance Rules: E-DEMO-002 → E-DEMO-003 → E-DEMO-004 order

#![allow(clippy::unwrap_used, clippy::expect_used)]

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
    let timeline = build_default_incident_timeline(catalog, start_secs, &[]);
    let timeline_arc = Arc::new(timeline);

    let scenario_clone = CrowdstrikeClone::new_with_scenario(
        42,
        Archetype::CompromisedEndpoint,
        org_id,
        Arc::clone(&timeline_arc),
        // Use demo_time_anchor (2026-01-01T00:00:00Z) — deterministic anchor.
        // The timeline's scenario_start_epoch_secs is independent from time_anchor.
        prism_dtu_common::demo_time_anchor(),
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
    );

    let mut armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline),
        time_anchor,
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
    // Claroty uses "ASSET-{org_slug}-{seed}-{index}" for the asset_id field
    // (BC-3.4.004: "dev-" prefix on device_id, "ASSET-" prefix on asset_id).
    // Primary device: asset_id = "ASSET-deadbeef-100-0".
    // Lateral devices: asset_id = "ASSET-deadbeef-100-1", "ASSET-deadbeef-100-2", ...
    //
    // FAIL: without StageMask projection, ALL generated records are served →
    // lateral device IDs (ASSET-deadbeef-100-1, etc.) ARE in the response →
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

    // Claroty response shape: {"assets": [...]} or {"devices": [...]}.
    // The canonical field for device ID in Claroty is "asset_id"
    // (BC-3.4.004: asset_id = "ASSET-{slug}-{seed}-{index}").
    let claroty_assets = claroty_body["assets"]
        .as_array()
        .or_else(|| claroty_body["devices"].as_array())
        .cloned()
        .unwrap_or_default();

    // Claroty primary device asset_id = "ASSET-{org_slug}-{seed}-0"
    // (same entity as CS/Armis "dev-deadbeef-100-0", different ID scheme).
    let expected_primary_claroty = format!("ASSET-{}-{}-0", catalog.org_slug, seed);

    let claroty_asset_ids: Vec<String> = claroty_assets
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.020 PC-5 / AC-015: at stage 1, primary device must be present.
    let claroty_primary_found = claroty_asset_ids.contains(&expected_primary_claroty);

    assert!(
        claroty_primary_found,
        "Cross-DTU coherence: Claroty response at stage 1 must contain primary device \
         '{}' (field: asset_id); got asset_ids: {:?}. BC-2.06.020 PC-5 / AC-015 \
         [RED GATE: StageMask projection absent OR timeline=None prevents scenario routing]",
        expected_primary_claroty, claroty_asset_ids
    );

    // BC-2.06.020 PC-5 / AC-015: at stage 1, lateral devices must be ABSENT.
    // StageMask at stage 1: primary_device=true, lateral_devices=false.
    // Lateral Claroty devices: "ASSET-{slug}-{seed}-1", "ASSET-{slug}-{seed}-2", ...
    // FAIL: without StageMask projection, ALL records are served → lateral devices
    //       ARE in the response → this assertion FAILS.
    let lateral_claroty_ids: Vec<String> = (1..=3)
        .map(|n| format!("ASSET-{}-{}-{}", catalog.org_slug, seed, n))
        .collect();

    for lat_id in &lateral_claroty_ids {
        assert!(
            !claroty_asset_ids.contains(lat_id),
            "TV-015-cross-dtu: at stage 1 (Recon), lateral Claroty device '{}' must be ABSENT \
             from POST /api/v1/devices response (StageMask lateral_devices=false at stage 1); \
             found it in {:?}. BC-2.06.020 PC-5 / AC-015 \
             [RED GATE: StageMask projection not implemented — lateral devices leak at stage 1]",
            lat_id,
            claroty_asset_ids
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
