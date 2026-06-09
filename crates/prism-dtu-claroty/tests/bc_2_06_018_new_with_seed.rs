//! Red Gate test RG-11: ClarotyClone::new_with_seed forwarded
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::OrgId;

/// Golden test vector: org bytes [0xde, 0xad, 0xbe, 0xef, ...] → org_slug = "deadbeef".
///
/// NOTE: This is `prism_dtu_common::OrgId` ([u8; 16]-backed), NOT `prism_core::OrgId`.
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Derive canonical org_slug from OrgId bytes (first 4 bytes as hex).
fn derive_org_slug(org: &OrgId) -> String {
    let b = org.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

/// RG-11: test_BC_2_06_018_claroty_new_with_seed_forwarded
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - ClarotyClone::new_with_seed(seed, org_id) is callable (constructor exists under fixture-gen)
/// - Constructor is infallible (returns Self, not Result<Self>)
/// - state.generated_records is non-empty after construction
/// - Device records contain IDs with "dev-{org_slug}-{seed}-" prefix
///
/// LOAD-BEARING: assertions inspect the constructed clone's state.generated_records.
/// A broken impl (empty generated_records) causes this test to FAIL.
#[test]
fn test_BC_2_06_018_claroty_new_with_seed_forwarded() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);
    let seed: u64 = 100;

    // ClarotyClone::new_with_seed is infallible (returns Self, not Result<Self>).
    let clone = ClarotyClone::new_with_seed(seed, org.clone());

    // (a) generated_records must be non-empty (BC-2.06.018 postcondition 1).
    let generated = &clone.state.generated_records;
    assert!(
        !generated.is_empty(),
        "ClarotyClone::new_with_seed must populate state.generated_records; got empty vec. \
         BC-2.06.018 postcondition 1 violated."
    );

    // (b) Device records must have device_id or asset_id starting with "dev-{org_slug}-{seed}-".
    let expected_prefix = format!("dev-{}-{}-", org_slug, seed);
    let device_records_count = generated
        .iter()
        .filter(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.starts_with(&expected_prefix))
                .unwrap_or(false)
        })
        .count();

    assert!(
        device_records_count > 0,
        "state.generated_records must contain device records with device_id starting with \
         '{expected_prefix}'; found 0 such records among {} total records. \
         ADR-036 §2.2 canonical format violated.",
        generated.len()
    );
}

/// RG-11 (part B): Determinism — same (seed, org_id) → byte-identical generated_records.
///
/// BC-3.4.001 postcondition 3 / BC-2.06.018 postcondition 3.
///
/// LOAD-BEARING: two clones with identical inputs must produce identical generated_records.
#[test]
fn test_BC_2_06_018_claroty_new_with_seed_deterministic() {
    let org = deadbeef_org();
    let seed: u64 = 42;

    let clone_a = ClarotyClone::new_with_seed(seed, org.clone());
    let clone_b = ClarotyClone::new_with_seed(seed, org);

    // When implemented: clone_a.state.generated_records == clone_b.state.generated_records
    assert_eq!(
        clone_a.state.generated_records, clone_b.state.generated_records,
        "ClarotyClone::new_with_seed with same (seed={seed}, org) must produce \
         byte-identical generated_records. BC-3.4.001 determinism invariant violated."
    );
}

/// RG-11 (part C): Distinct seeds produce disjoint device ID sets.
///
/// Verifies INV-DISTINCT-DATA-001.
///
/// LOAD-BEARING: directly inspects generated_records from both clones.
#[test]
fn test_BC_2_06_018_claroty_new_with_seed_disjoint_ids() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);

    let clone_100 = ClarotyClone::new_with_seed(100, org.clone());
    let clone_200 = ClarotyClone::new_with_seed(200, org);

    let ids_100: std::collections::HashSet<String> = clone_100
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
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
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .filter(|s| s.contains(&format!("{org_slug}-200-")))
        .collect();

    assert!(
        !ids_100.is_empty(),
        "clone_100 generated_records must contain records with seed=100 in device_id"
    );
    assert!(
        !ids_200.is_empty(),
        "clone_200 generated_records must contain records with seed=200 in device_id"
    );

    let intersection: std::collections::HashSet<&String> = ids_100.intersection(&ids_200).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001: device_ids for seed=100 and seed=200 must be disjoint; \
         intersection={intersection:?}. org_slug={org_slug}"
    );
}
