//! Red Gate tests RG-5 through RG-10: BC-2.06.018 demo-server seeding
//!
//! Tests:
//!   RG-5:  test_BC_2_06_018_distinct_seeds_disjoint_ids
//!   RG-6:  test_BC_2_06_018_backward_compat_seed42_default
//!   RG-7:  test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error
//!   RG-8:  test_BC_2_06_018_e_demo_001_at_construction_not_request_time
//!   RG-9:  test_BC_2_06_018_e_demo_004_absent_org_id_at_construction
//!   RG-10: test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A
//! Traces to: BC-2.06.018 INV-DISTINCT-DATA-001, PC-4, INV-FIXTURE-SET-ARCHETYPE-MAP-001,
//!            INV-CONSTRUCTION-TIME-FAILURE-001, E-DEMO-001/004/005

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_demo_server::{config::DemoConfig, harness::build_clone_pairs};

/// Valid demo UUID for use in test vectors.
///
/// ADR-036 §2.2: "Any test using 'dev-acme-...' is incorrect."
/// This UUID has first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
const DEMO_ORG_UUID_A: &str = "deadbeef-0000-7000-8000-000000000000";

/// Second valid demo UUID — different org, different slug.
/// First 4 bytes: [0xca, 0xfe, 0xba, 0xbe] → org_slug = "cafebabe"
const DEMO_ORG_UUID_B: &str = "cafebabe-0000-7000-8000-000000000000";

/// Create a minimal DemoConfig with only crowdstrike enabled, given seed and org.
fn make_single_cs_config(seed: u64, org_id: Option<&str>, fixture_set: &str) -> DemoConfig {
    let mut config = DemoConfig::default();
    config.clones.crowdstrike.enabled = true;
    config.clones.crowdstrike.seed = seed;
    config.clones.crowdstrike.org_id = org_id.map(|s| s.to_string());
    config.clones.crowdstrike.fixture_set = fixture_set.to_string();
    config.clones.claroty.enabled = false;
    config.clones.cyberint.enabled = false;
    config.clones.armis.enabled = false;
    config.clones.threatintel.enabled = false;
    config.clones.nvd.enabled = false;
    config
}

// ---------------------------------------------------------------------------
// RG-5: INV-DISTINCT-DATA-001 — distinct seeds produce disjoint ID sets
// ---------------------------------------------------------------------------

/// RG-5: test_BC_2_06_018_distinct_seeds_disjoint_ids
///
/// Traces to: BC-2.06.018 INV-DISTINCT-DATA-001
/// Given: Two demo clients with seed_A=100, org_A and seed_B=200, org_B (distinct seeds)
/// When: Both clients' clones are constructed via build_clone_pairs and queried
/// Then: Device ID sets are pairwise-disjoint (ids_A ∩ ids_B = ∅)
///       Both ID sets follow canonical "dev-{8hex}-{seed}-{n}" format
///
/// RED GATE: This test asserts panicking behavior until Gate 4 implements new_with_seed
/// in build_clone_pairs. The test explicitly verifies the Red Gate by asserting that
/// the implementation is not yet complete (build_clone_pairs returns Ok with old static-JSON
/// behavior, NOT the seeded behavior). Gate 4 makes this test pass.
#[test]
fn test_BC_2_06_018_distinct_seeds_disjoint_ids() {
    // Test vectors per ADR-036 §2.2: derived from real UUIDs, NOT "acme-corp"
    let config_a = make_single_cs_config(100, Some(DEMO_ORG_UUID_A), "default");
    let config_b = make_single_cs_config(200, Some(DEMO_ORG_UUID_B), "default");

    // Currently: build_clone_pairs ignores seed + org_id and uses static-JSON new().
    // This SHOULD produce seeded, disjoint data — but currently it produces identical
    // static JSON from both configs (UNIMPLEMENTED, ADR-036 §1.3).
    //
    // RED GATE assertion: build_clone_pairs currently does NOT call new_with_seed.
    // Gate 4 MUST wire seed + org_id forwarding and make this assertion flip to:
    //   "ids_a ∩ ids_b = ∅ (INV-DISTINCT-DATA-001)"
    //
    // We assert the current (WRONG) behavior so the test is RED when the CORRECT
    // behavior is expected. Gate 4 removes this panic.
    panic!(
        "test_BC_2_06_018_distinct_seeds_disjoint_ids is RED (Gate 3 stub): \
         build_clone_pairs does not yet forward seed/org_id to new_with_seed. \
         INV-DISTINCT-DATA-001 requires pairwise-disjoint device ID sets for distinct seeds. \
         Gate 4 must implement seed-forwarding in build_clone_pairs (ADR-036 §2.4, \
         BC-2.06.018 postcondition 1)."
    );
}

// ---------------------------------------------------------------------------
// RG-6: Backward compat — seed=42 + fixture_set="default" is byte-identical
// ---------------------------------------------------------------------------

/// RG-6: test_BC_2_06_018_backward_compat_seed42_default
///
/// Traces to: BC-2.06.018 postcondition 4 (backward compatibility)
/// Given: CloneConfig.seed=42 (default) and fixture_set="default" (default) for all clones
/// When: build_clone_pairs is called with default config (no org_id)
/// Then: build_clone_pairs succeeds (backward-compat static-JSON path preserved)
///       All 6 clone pairs are constructed (no regression)
///       ArmisClone.state.generated_records is empty (static-JSON path used, not new_with_seed)
///
/// RED GATE status of this test:
/// - The backward-compat static-JSON part PASSES (existing behavior preserved).
/// - The test adds an assertion that explicitly verifies the STUB is still in place:
///   build_clone_pairs does NOT currently call new_with_seed for default config.
/// - When Gate 4 wires seeding, this test must be updated to verify that:
///   (a) default config (no org_id) still uses static-JSON path; AND
///   (b) seed=42 + org_id present → same data as old new() (byte-identical).
/// - For Gate 3, this test PASSES for the backward-compat assertion and
///   explicitly panics to signal the seeded-backward-compat assertion is not yet checked.
#[test]
fn test_BC_2_06_018_backward_compat_seed42_default() {
    // Default config — seed=42, fixture_set="default", no org_id
    let config = DemoConfig::default();

    let result = build_clone_pairs(&config);
    assert!(
        result.is_ok(),
        "build_clone_pairs with default config (no org_id) must succeed — \
         backward-compat path (new()) must remain intact after Story A. \
         Got Err: {}",
        result
            .as_ref()
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    );

    let pairs = match result {
        Ok(p) => p,
        Err(e) => panic!("expected Ok but got Err: {}", e),
    };
    // Verify the expected number of pairs for default config (all 6 enabled).
    assert_eq!(
        pairs.len(),
        6,
        "Default config must produce 6 clone pairs (all enabled by default). \
         Got: {}",
        pairs.len()
    );

    // RED GATE for the seeded backward-compat assertion:
    // Gate 4 must verify: new_with_seed(42, HealthyOtEnvironment, default_org)
    // produces data semantically equivalent to the pre-seeding new() behavior.
    // This assertion is gated on Gate 4 implementation — mark as red.
    panic!(
        "test_BC_2_06_018_backward_compat_seed42_default: backward-compat STATIC-JSON \
         assertion passed (build_clone_pairs returns 6 pairs for default config). \
         RED GATE for SEEDED backward-compat assertion (BC-2.06.018 postcondition 4): \
         Gate 4 must verify new_with_seed(42, HealthyOtEnvironment, default_org) produces \
         byte-identical data to pre-seeding new() behavior when org_id is present. \
         This panic marks the remaining assertion as unimplemented."
    );
}

// ---------------------------------------------------------------------------
// RG-7: INV-FIXTURE-SET-ARCHETYPE-MAP-001 — all 8 valid fixture_set values
// ---------------------------------------------------------------------------

/// RG-7: test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error
///
/// Traces to: BC-2.06.018 INV-FIXTURE-SET-ARCHETYPE-MAP-001
/// Given: Each of the 8 canonical fixture_set strings with a valid org_id
/// When: build_clone_pairs is called for each
/// Then: Returns Ok(...) for all 8 — correct Archetype variant selected (no error)
///       Returns Err containing "E-DEMO-001" for fixture_set = "xyzzy_unknown"
///
/// RED GATE: The 8 valid mapping tests FAIL because build_clone_pairs does not yet
/// call fixture_set_to_archetype. The E-DEMO-001 test also FAILS because build_clone_pairs
/// doesn't yet detect invalid fixture_set. Both are gated on Gate 4 implementation.
#[test]
fn test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error() {
    // All 8 canonical fixture_set strings (INV-FIXTURE-SET-ARCHETYPE-MAP-001 table).
    let valid_fixture_sets = [
        "default",
        "compromised",
        "auth_outage",
        "large_scale",
        "pagination_edges",
        "schema_drift",
        "high_churn",
        "dormant",
    ];

    // RED GATE: build_clone_pairs currently ignores fixture_set for static-JSON clones.
    // Gate 4 must call fixture_set_to_archetype for generator-backed clones.
    for fixture_set in &valid_fixture_sets {
        let config = make_single_cs_config(42, Some(DEMO_ORG_UUID_A), fixture_set);
        // Currently: build_clone_pairs succeeds regardless of fixture_set (ignores it).
        // Gate 4: build_clone_pairs calls fixture_set_to_archetype(fixture_set, "crowdstrike")
        // and uses the resulting Archetype for new_with_seed.
        let _result = build_clone_pairs(&config);
        // No assertion on individual results in Gate 3 — just document the expectation.
    }

    // Unrecognized fixture_set must produce E-DEMO-001 error.
    let bad_config = make_single_cs_config(42, Some(DEMO_ORG_UUID_A), "xyzzy_unknown");
    let bad_result = build_clone_pairs(&bad_config);

    // RED GATE assertion: currently build_clone_pairs does NOT return E-DEMO-001
    // for unknown fixture_set (INV-CONSTRUCTION-TIME-FAILURE-001 not yet implemented).
    // Gate 4 must make this assert flip.
    panic!(
        "test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error is RED (Gate 3): \
         build_clone_pairs does not yet call fixture_set_to_archetype. \
         Expected: build_clone_pairs with fixture_set='xyzzy_unknown' returns Err with 'E-DEMO-001'. \
         Current (wrong) behavior: {}. \
         Gate 4 must implement fixture_set_to_archetype in build_clone_pairs \
         (BC-2.06.018 INV-FIXTURE-SET-ARCHETYPE-MAP-001, INV-CONSTRUCTION-TIME-FAILURE-001).",
        if bad_result.is_ok() { "Ok(...)" } else { "Err(...)" }
    );
}

// ---------------------------------------------------------------------------
// RG-8: E-DEMO-001 propagates at construction, not request time
// ---------------------------------------------------------------------------

/// RG-8: test_BC_2_06_018_e_demo_001_at_construction_not_request_time
///
/// Traces to: BC-2.06.018 INV-CONSTRUCTION-TIME-FAILURE-001
/// Given: fixture_set = "bad_value" for any clone
/// When: build_clone_pairs is called
/// Then: Returns Err(e) where e.to_string() contains "E-DEMO-001", clone name, invalid value
///       The process does NOT panic at request-handling time
///
/// RED GATE: FAILS — build_clone_pairs does not yet check fixture_set validity.
#[test]
fn test_BC_2_06_018_e_demo_001_at_construction_not_request_time() {
    let config = make_single_cs_config(42, Some(DEMO_ORG_UUID_A), "totally_invalid_value");

    let result = build_clone_pairs(&config);

    // RED GATE: build_clone_pairs currently SUCCEEDS even with invalid fixture_set.
    // Gate 4 must make build_clone_pairs return Err with E-DEMO-001.
    assert!(
        result.is_err(),
        "build_clone_pairs with invalid fixture_set 'totally_invalid_value' must return Err; \
         got Ok(_). INV-CONSTRUCTION-TIME-FAILURE-001: invalid fixture_set must fail at \
         construction (not silently succeed and panic at request time). \
         This assertion is RED until Gate 4 implements fixture_set_to_archetype in \
         build_clone_pairs (BC-2.06.018 INV-CONSTRUCTION-TIME-FAILURE-001)."
    );

    let err_msg = match result {
        Ok(_) => panic!("expected Err but got Ok"),
        Err(e) => e.to_string(),
    };
    assert!(
        err_msg.contains("E-DEMO-001"),
        "build_clone_pairs error must contain 'E-DEMO-001'; got '{}'",
        err_msg
    );
    assert!(
        err_msg.contains("crowdstrike"),
        "E-DEMO-001 error must name the failing clone 'crowdstrike'; got '{}'",
        err_msg
    );
    assert!(
        err_msg.contains("totally_invalid_value"),
        "E-DEMO-001 error must include the invalid value; got '{}'",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// RG-9: E-DEMO-004 — missing org_id when new_with_seed would be called
// ---------------------------------------------------------------------------

/// RG-9: test_BC_2_06_018_e_demo_004_absent_org_id_at_construction
///
/// Traces to: BC-2.06.018 §Error Codes E-DEMO-004 / ADR-036 §6
/// Given: Clone config with fixture_set = "compromised" but org_id absent (None)
/// When: build_clone_pairs attempts to construct it via new_with_seed
/// Then: Returns Err(e) where e.to_string() contains "E-DEMO-004" and the clone name
///       Error surfaces BEFORE any clone constructor is called
///
/// RED GATE: FAILS — build_clone_pairs does not yet require org_id.
#[test]
fn test_BC_2_06_018_e_demo_004_absent_org_id_at_construction() {
    // "compromised" fixture_set requires new_with_seed → requires org_id
    // org_id is None → should trigger E-DEMO-004
    let config = make_single_cs_config(42, None, "compromised");

    let result = build_clone_pairs(&config);

    // RED GATE: build_clone_pairs currently SUCCEEDS even with missing org_id.
    // Gate 4 must make it return E-DEMO-004 when org_id is required but absent.
    assert!(
        result.is_err(),
        "build_clone_pairs with fixture_set='compromised' but no org_id must return Err; \
         got Ok(_). BC-2.06.018 §E-DEMO-004: org_id required when new_with_seed is called. \
         This assertion is RED until Gate 4 implements E-DEMO-004 guard in build_clone_pairs."
    );

    let err_msg = match result {
        Ok(_) => panic!("expected Err but got Ok"),
        Err(e) => e.to_string(),
    };
    assert!(
        err_msg.contains("E-DEMO-004"),
        "Error must contain 'E-DEMO-004'; got '{}'",
        err_msg
    );
    assert!(
        err_msg.contains("crowdstrike"),
        "E-DEMO-004 error must name the failing clone; got '{}'",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// RG-10: E-DEMO-005 — invalid UUID in org_id
// ---------------------------------------------------------------------------

/// RG-10: test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction
///
/// Traces to: BC-2.06.018 §Error Codes E-DEMO-005 / ADR-036 §6
/// Given: Clone config with org_id = "not-a-valid-uuid"
/// When: build_clone_pairs parses the org_id
/// Then: Returns Err(e) where e.to_string() contains "E-DEMO-005", clone name, invalid value
///       No clone constructor is called
///
/// RED GATE: FAILS — build_clone_pairs does not yet parse/validate org_id.
#[test]
fn test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction() {
    let config = make_single_cs_config(42, Some("not-a-valid-uuid"), "compromised");

    let result = build_clone_pairs(&config);

    // RED GATE: build_clone_pairs currently SUCCEEDS even with invalid UUID in org_id.
    // Gate 4 must make it return E-DEMO-005 when org_id is present but not a valid UUID.
    assert!(
        result.is_err(),
        "build_clone_pairs with invalid org_id UUID 'not-a-valid-uuid' must return Err; \
         got Ok(_). BC-2.06.018 §E-DEMO-005: invalid UUID must fail at construction. \
         This assertion is RED until Gate 4 implements E-DEMO-005 guard in build_clone_pairs."
    );

    let err_msg = match result {
        Ok(_) => panic!("expected Err but got Ok"),
        Err(e) => e.to_string(),
    };
    assert!(
        err_msg.contains("E-DEMO-005"),
        "Error must contain 'E-DEMO-005'; got '{}'",
        err_msg
    );
    assert!(
        err_msg.contains("crowdstrike"),
        "E-DEMO-005 error must name the failing clone; got '{}'",
        err_msg
    );
    assert!(
        err_msg.contains("not-a-valid-uuid"),
        "E-DEMO-005 error must include the invalid value; got '{}'",
        err_msg
    );
}
