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

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

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
/// Given: Two demo clients with seed_A=100, org_A="deadbeef..." and seed_B=200, org_B="cafebabe..."
/// When: Both CrowdstrikeClones are constructed via new_with_seed with distinct seeds
/// Then: Device ID sets are pairwise-disjoint (ids_A ∩ ids_B = ∅)
///       Both ID sets follow canonical "dev-{8hex}-{seed}-{n}" format
///
/// LOAD-BEARING: directly inspects state.generated_devices from both clones and asserts
/// true set-disjointness. A broken impl (both serving same static fixtures) FAILS here.
/// The adversary-identified paper-test (asserting on string literal prefixes instead of
/// actual data) has been replaced with real set-intersection logic.
#[cfg(feature = "fixture-gen")]
#[test]
fn test_BC_2_06_018_distinct_seeds_disjoint_ids() {
    use prism_dtu_common::OrgId;
    use prism_dtu_crowdstrike::CrowdstrikeClone;

    // Test vectors per ADR-036 §2.2: derived from real UUIDs, NOT "acme-corp"
    // DEMO_ORG_UUID_A: first 4 bytes [0xde, 0xad, 0xbe, 0xef] → slug = "deadbeef"
    // DEMO_ORG_UUID_B: first 4 bytes [0xca, 0xfe, 0xba, 0xbe] → slug = "cafebabe"
    let org_a: OrgId = OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);
    let org_b: OrgId = OrgId([
        0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    // Construct clones directly so we can inspect state (Box<dyn BehavioralClone> is opaque).
    // CompromisedEndpoint produces non-empty device records (disjointness test requires non-empty).
    let clone_a = CrowdstrikeClone::new_with_seed(
        100,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_a,
    );
    let clone_b = CrowdstrikeClone::new_with_seed(
        200,
        prism_dtu_common::Archetype::CompromisedEndpoint,
        org_b,
    );

    // Extract device IDs from both clones' generated_devices.
    let ids_a: std::collections::HashSet<String> = clone_a
        .state
        .generated_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    let ids_b: std::collections::HashSet<String> = clone_b
        .state
        .generated_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert!(
        !ids_a.is_empty(),
        "INV-DISTINCT-DATA-001: clone_a (seed=100, org=deadbeef) must have non-empty \
         generated_devices"
    );
    assert!(
        !ids_b.is_empty(),
        "INV-DISTINCT-DATA-001: clone_b (seed=200, org=cafebabe) must have non-empty \
         generated_devices"
    );

    // CRITICAL: verify set-disjointness (ids_A ∩ ids_B = ∅).
    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 VIOLATED: device IDs from seed=100/deadbeef and seed=200/cafebabe \
         must be disjoint; intersection={intersection:?} ({} IDs in common). \
         Both sets use 'dev-{{org_slug}}-{{seed}}-{{n}}' format which guarantees uniqueness.",
        intersection.len()
    );

    // Also verify canonical ID format: all ids_a start with "dev-deadbeef-100-".
    let expected_a_prefix = "dev-deadbeef-100-";
    for id in &ids_a {
        assert!(
            id.starts_with(expected_a_prefix),
            "clone_a device_id '{id}' must start with '{expected_a_prefix}' (ADR-036 §2.2)"
        );
    }

    // All ids_b start with "dev-cafebabe-200-".
    let expected_b_prefix = "dev-cafebabe-200-";
    for id in &ids_b {
        assert!(
            id.starts_with(expected_b_prefix),
            "clone_b device_id '{id}' must start with '{expected_b_prefix}' (ADR-036 §2.2)"
        );
    }

    // Integration-level smoke: build_clone_pairs also works with these configs.
    let config_a = make_single_cs_config(100, Some(DEMO_ORG_UUID_A), "default");
    let config_b = make_single_cs_config(200, Some(DEMO_ORG_UUID_B), "default");
    let pairs_a = build_clone_pairs(&config_a).expect("build_clone_pairs config_a must succeed");
    let pairs_b = build_clone_pairs(&config_b).expect("build_clone_pairs config_b must succeed");
    assert_eq!(
        pairs_a.len(),
        1,
        "config_a must produce exactly 1 clone pair (crowdstrike)"
    );
    assert_eq!(
        pairs_b.len(),
        1,
        "config_b must produce exactly 1 clone pair (crowdstrike)"
    );
    assert_eq!(
        pairs_a[0].name, "crowdstrike",
        "First pair must be crowdstrike for config_a"
    );
    assert_eq!(
        pairs_b[0].name, "crowdstrike",
        "First pair must be crowdstrike for config_b"
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
///
/// Resolution of ambiguity RG-6 (BC-2.06.018 PC-4):
/// When org_id = None (default config), build_clone_pairs uses the static-JSON new() path.
/// The new() constructor is unchanged — byte-identical to pre-Story-A behavior.
/// E-DEMO-004 fires only when fixture_set maps to non-HealthyOtEnvironment AND org_id is None.
/// fixture_set="default" → HealthyOtEnvironment, so no E-DEMO-004 even with no org_id.
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

    let pairs = result.unwrap();
    // Verify the expected number of pairs for default config (all 6 enabled).
    // Default config has: crowdstrike, claroty, cyberint, armis, threatintel, nvd
    assert_eq!(
        pairs.len(),
        6,
        "Default config must produce 6 clone pairs (all enabled by default). \
         Got: {}. BC-2.06.018 postcondition 4: static-JSON path must be preserved.",
        pairs.len()
    );

    // Verify all expected clone names are present.
    let names: Vec<&str> = pairs.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"crowdstrike"),
        "crowdstrike clone must be present"
    );
    assert!(names.contains(&"armis"), "armis clone must be present");
    assert!(names.contains(&"claroty"), "claroty clone must be present");
    assert!(
        names.contains(&"cyberint"),
        "cyberint clone must be present"
    );
    assert!(
        names.contains(&"threatintel"),
        "threatintel clone must be present"
    );
    assert!(names.contains(&"nvd"), "nvd clone must be present");
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
#[cfg(feature = "fixture-gen")]
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

    for fixture_set in &valid_fixture_sets {
        let config = make_single_cs_config(42, Some(DEMO_ORG_UUID_A), fixture_set);
        let result = build_clone_pairs(&config);
        assert!(
            result.is_ok(),
            "build_clone_pairs with fixture_set='{}' and valid org_id must succeed; \
             got Err: {}. INV-FIXTURE-SET-ARCHETYPE-MAP-001: all 8 canonical values must map \
             to valid archetypes.",
            fixture_set,
            result
                .as_ref()
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default()
        );
    }

    // Unrecognized fixture_set must produce E-DEMO-001 error.
    let bad_config = make_single_cs_config(42, Some(DEMO_ORG_UUID_A), "xyzzy_unknown");
    let bad_result = build_clone_pairs(&bad_config);

    assert!(
        bad_result.is_err(),
        "build_clone_pairs with fixture_set='xyzzy_unknown' must return Err (E-DEMO-001); \
         got Ok(_). INV-FIXTURE-SET-ARCHETYPE-MAP-001 + INV-CONSTRUCTION-TIME-FAILURE-001."
    );
    let err_msg = match bad_result {
        Ok(_) => panic!("expected Err but got Ok"),
        Err(e) => e.to_string(),
    };
    assert!(
        err_msg.contains("E-DEMO-001"),
        "Error must contain 'E-DEMO-001'; got '{}'",
        err_msg
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
