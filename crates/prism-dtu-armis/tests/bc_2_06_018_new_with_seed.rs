//! Red Gate test RG-4: ArmisClone::new_with_seed canonical 3-arg form (fallible)
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3 Armis retrofit
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{Archetype, OrgId};

/// Golden test vector: org bytes [0xde, 0xad, 0xbe, 0xef, ...] → org_slug = "deadbeef".
///
/// NOTE: This is `prism_dtu_common::OrgId` ([u8; 16]-backed), NOT `prism_core::OrgId`.
/// ADR-036 §2.2: "Any test using 'dev-acme-...' is incorrect."
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Derive canonical org_slug from OrgId bytes (formula: hex of first 4 bytes).
///
/// Helper for the test vector derivation — uses the same formula as production code.
fn derive_org_slug_from_bytes(org: &OrgId) -> String {
    let b = org.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

/// RG-4: test_BC_2_06_018_armis_new_with_seed_canonical_3arg
///
/// Traces to: BC-2.06.018 postcondition 1 / ADR-036 v2.2 canonical 3-arg form
/// Verifies:
/// - ArmisClone::new_with_seed(seed, archetype, org_id) is callable (canonical 3-arg, no org_slug arg)
/// - Return type is anyhow::Result<Self> — fallible, mirrors ArmisClone::new()
/// - state.generated_records is non-empty after construction
/// - Device records in state contain IDs in "dev-{org_slug}-{seed}-{n}" format
///   (org_slug derived internally by new_with_seed from org_id bytes)
///
/// LOAD-BEARING: assertions inspect the constructed clone's state.generated_records.
/// A broken impl (empty generated_records) causes this test to FAIL.
#[test]
fn test_BC_2_06_018_armis_new_with_seed_canonical_3arg() {
    let org = deadbeef_org();
    let seed: u64 = 100;
    let org_slug = derive_org_slug_from_bytes(&org);

    // Canonical 3-arg form: org_slug derived internally (ADR-036 v2.2).
    let result = ArmisClone::new_with_seed(seed, Archetype::CompromisedEndpoint, org.clone());

    let clone = result.expect(
        "ArmisClone::new_with_seed must succeed with valid seed, archetype, and org_id; \
         got Err — implementation incomplete",
    );

    // (a) generated_records must be non-empty (BC-2.06.018 postcondition 1).
    let generated = &clone.state.generated_records;
    assert!(
        !generated.is_empty(),
        "ArmisClone::new_with_seed must populate state.generated_records; got empty vec. \
         BC-2.06.018 postcondition 1 violated."
    );

    // (b) Device records (those with "asset_id" containing the org_slug) must be present.
    let expected_slug_prefix = format!("dev-{}-{}-", org_slug, seed);
    let device_records: Vec<&serde_json::Value> = generated
        .iter()
        .filter(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.starts_with(&expected_slug_prefix))
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !device_records.is_empty(),
        "state.generated_records must contain device records with asset_id starting with \
         '{expected_slug_prefix}'; found 0 such records among {} total records. \
         ADR-036 §2.2 canonical format violated.",
        generated.len()
    );
}

/// RG-4 (part B): Error from new_with_seed propagates through Result chain.
///
/// Verifies: The fallible constructor's error propagates consistently.
/// (BC-2.06.018 AC-004 — error propagates through build_clone_pairs via ?)
///
/// LOAD-BEARING: Both type signatures must be anyhow::Result<Self>.
#[test]
fn test_BC_2_06_018_armis_new_with_seed_fallibility_consistent_with_new() {
    // Both new() and new_with_seed() must return anyhow::Result<Self>.
    // This test verifies the type signature is consistent.
    let _new_result: anyhow::Result<ArmisClone> = ArmisClone::new();

    let org = deadbeef_org();
    // new_with_seed must also return anyhow::Result<Self> (canonical 3-arg form, ADR-036 v2.2).
    let result: anyhow::Result<ArmisClone> =
        ArmisClone::new_with_seed(42, Archetype::CompromisedEndpoint, org);
    assert!(
        result.is_ok(),
        "ArmisClone::new_with_seed must succeed with valid inputs; got Err: {:?}",
        result.err()
    );
    // Both type-check at the same level — fallibility is consistent.
}

/// RG-4 (part C): Two distinct seeds produce disjoint device ID sets.
///
/// Verifies INV-DISTINCT-DATA-001: seed_A ≠ seed_B → device asset_ids disjoint.
///
/// LOAD-BEARING: directly inspects generated_records from both clones.
#[test]
fn test_BC_2_06_018_armis_new_with_seed_disjoint_ids() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug_from_bytes(&org);

    // Canonical 3-arg form: org_slug derived internally (ADR-036 v2.2).
    let clone_100 = ArmisClone::new_with_seed(100, Archetype::CompromisedEndpoint, org.clone())
        .expect("new_with_seed(100) must succeed");
    let clone_200 = ArmisClone::new_with_seed(200, Archetype::CompromisedEndpoint, org)
        .expect("new_with_seed(200) must succeed");

    let ids_100: std::collections::HashSet<String> = clone_100
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .filter(|s| s.contains(&format!("{org_slug}-100-")))
        .collect();

    let ids_200: std::collections::HashSet<String> = clone_200
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .filter(|s| s.contains(&format!("{org_slug}-200-")))
        .collect();

    assert!(
        !ids_100.is_empty(),
        "clone_100 generated_records must contain records with seed=100 in asset_id"
    );
    assert!(
        !ids_200.is_empty(),
        "clone_200 generated_records must contain records with seed=200 in asset_id"
    );

    let intersection: std::collections::HashSet<&String> = ids_100.intersection(&ids_200).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001: asset_ids for seed=100 and seed=200 must be disjoint; \
         intersection={intersection:?}"
    );
}
