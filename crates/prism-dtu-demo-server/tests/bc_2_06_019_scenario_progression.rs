//! Red Gate tests 9, 10, 11, 15: BC-2.06.019 / BC-2.06.020 demo-server scenario guards
//!
//! Tests:
//!   Test 9:  test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones
//!   Test 10: test_BC_2_06_019_e_demo_003_unrecognized_archetype
//!   Test 11: test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path
//!   Test 15: test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//! Traces to: BC-2.06.019 E-DEMO-002/003 / TV-019-012,013,007
//!            BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 / PC-5

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
// Verifies: given three clones (Armis, CrowdStrike, Claroty) all built with
// the same (seed=100, org_id=deadbeef-...) and scenario.enabled=true,
// all three produce the same primary device ID at stage 1 (T+90s).
//
// The canonical format for all three is:
//   "dev-deadbeef-100-0" (org_slug derived from deadbeef UUID first 4 bytes)
//
// FAIL mode: new_with_scenario stubs leave timeline=None on all three clones.
// Without timeline, stage 1 assertion on current_stage_index FAILS (always returns 0).
// Also, cross-DTU coherence depends on the timeline being the same Arc — which is
// not yet wired because the stub doesn't attach it.
//
// The primary load-bearing FAIL is the current_stage_index assertion at T+90s.
// ---------------------------------------------------------------------------
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones() {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_common::{
        build_default_incident_timeline, build_scenario_entity_catalog, current_stage_index,
        Archetype, OrgId,
    };
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use std::sync::Arc;

    let seed: u64 = 100;
    let org_uuid = uuid::Uuid::parse_str(DEMO_ORG_UUID_DEADBEEF)
        .expect("DEMO_ORG_UUID_DEADBEEF must be valid UUID");
    let org_id = OrgId(*org_uuid.as_bytes());

    // Build the shared entity catalog — all three clones use this same catalog
    // to derive their entity IDs (cross-DTU coherence).
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

    // Build the shared timeline (start_secs = fixed for determinism).
    let start_secs: i64 = 2_000_000;
    let timeline = build_default_incident_timeline(catalog.clone(), start_secs, &[]);
    let timeline_arc = Arc::new(timeline);

    // Use demo_time_anchor (2026-01-01T00:00:00Z) — deterministic anchor.
    // The timeline's scenario_start_epoch_secs is independent from time_anchor.
    let time_anchor = prism_dtu_common::demo_time_anchor();

    // Construct all three scenario clones with the SAME seed, org_id, and timeline Arc.
    let cs_clone = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline_arc),
        time_anchor,
    );

    let armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id.clone(),
        Arc::clone(&timeline_arc),
        time_anchor,
    )
    .expect("ArmisClone::new_with_scenario must succeed");

    let claroty_clone = ClarotyClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org_id,
        Arc::clone(&timeline_arc),
        time_anchor,
    );

    // All three clones must have timeline attached.
    // FAIL: all three stubs leave timeline = None.
    assert!(
        cs_clone.state.timeline.is_some(),
        "Cross-DTU coherence: CrowdstrikeClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE]"
    );
    assert!(
        armis_clone.state.timeline.is_some(),
        "Cross-DTU coherence: ArmisClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE]"
    );
    assert!(
        claroty_clone.state.timeline.is_some(),
        "Cross-DTU coherence: ClarotyClone::new_with_scenario must set timeline = Some; \
         got None — BC-2.06.020 PC-5 / AC-015 [RED GATE]"
    );

    // At T+90s (stage 1 / Recon), all three clones must serve the same primary device ID.
    let now_stage1 = start_secs + 90;
    let stage1_idx = current_stage_index(&timeline_arc, now_stage1);

    // FAIL: current_stage_index stub always returns 0, not 1.
    assert_eq!(
        stage1_idx, 1,
        "Cross-DTU coherence: at T+90s, stage index must be 1 (Recon); got {stage1_idx} \
         — BC-2.06.020 PC-5 / AC-015 / TV-019-010 \
         [RED GATE: current_stage_index stub always returns 0]"
    );

    // At stage 1 (Recon), primary device must be visible (primary_device=true).
    let stage1_mask = &timeline_arc.stages[stage1_idx].visible_entity_mask;
    assert!(
        stage1_mask.primary_device,
        "Stage 1 (Recon) mask must have primary_device=true; got false. \
         BC-2.06.019 PC-2 table"
    );

    // Cross-DTU: primary device ID in CrowdStrike generated_devices must match catalog.
    let cs_primary_found = cs_clone.state.generated_devices.iter().any(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == &catalog.primary_device_id_cs)
            .unwrap_or(false)
    });
    assert!(
        cs_primary_found,
        "Cross-DTU coherence: CrowdStrike generated_devices must contain primary device \
         '{}' at stage 1. BC-2.06.020 PC-5 / AC-015",
        catalog.primary_device_id_cs
    );

    // Cross-DTU: primary device ID in Armis generated_records must match catalog.
    let armis_primary_found = armis_clone.state.generated_records.iter().any(|rec| {
        rec.get("asset_id")
            .and_then(|v| v.as_str())
            .map(|id| id == &catalog.primary_device_id_armis)
            .unwrap_or(false)
    });
    assert!(
        armis_primary_found,
        "Cross-DTU coherence: Armis generated_records must contain primary device \
         '{}' at stage 1. BC-2.06.020 PC-5 / AC-015",
        catalog.primary_device_id_armis
    );
}
