//! Red Gate test RG-12: CyberintClone::new_with_seed forwarded and fallible
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::OrgId;
use prism_dtu_cyberint::CyberintClone;

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

/// RG-12: test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - CyberintClone::new_with_seed(seed, org_id) is callable (constructor exists under fixture-gen)
/// - Return type is anyhow::Result<Self> — fallible, mirrors CyberintClone::new()
/// - state.generated_records is non-empty after construction
/// - Alert records contain alert_id with org_slug embedded
///
/// LOAD-BEARING: assertions inspect the constructed clone's state.generated_records.
/// A broken impl (empty generated_records) causes this test to FAIL.
#[test]
fn test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);
    let seed: u64 = 100;

    let result = CyberintClone::new_with_seed(seed, org.clone());

    let clone = result.expect(
        "CyberintClone::new_with_seed must succeed with valid seed and org_id; \
         got Err — implementation incomplete",
    );

    // (a) generated_records must be non-empty (BC-2.06.018 postcondition 1).
    let generated = &clone.state.generated_records;
    assert!(
        !generated.is_empty(),
        "CyberintClone::new_with_seed must populate state.generated_records; got empty vec. \
         BC-2.06.018 postcondition 1 violated."
    );

    // (b) Alert records must have alert_id containing the org_slug.
    // Cyberint generator produces alert records with alert_id format "alert-{org_slug}-{seed}-{n}".
    let alert_records_count = generated
        .iter()
        .filter(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.contains(&org_slug))
                .unwrap_or(false)
        })
        .count();

    assert!(
        alert_records_count > 0,
        "state.generated_records must contain alert records with org_slug '{org_slug}' in alert_id; \
         found 0 such records among {} total records. \
         ADR-036 §2.2 canonical format violated.",
        generated.len()
    );
}

/// RG-12 (part B): Fallibility consistent with existing CyberintClone::new().
///
/// Both new() and new_with_seed() must return anyhow::Result<Self>.
/// This verifies the type signature is consistent.
///
/// LOAD-BEARING: both constructor calls must succeed with valid inputs.
#[test]
fn test_BC_2_06_018_cyberint_new_with_seed_fallibility_consistent_with_new() {
    // Both new() and new_with_seed() must return anyhow::Result<Self>.
    let _new_result: anyhow::Result<CyberintClone> = CyberintClone::new();
    assert!(
        _new_result.is_ok(),
        "CyberintClone::new() must succeed; got Err: {:?}",
        _new_result.err()
    );

    let org = deadbeef_org();
    // new_with_seed must also return anyhow::Result<Self>.
    let seed_result: anyhow::Result<CyberintClone> = CyberintClone::new_with_seed(42, org);
    assert!(
        seed_result.is_ok(),
        "CyberintClone::new_with_seed must succeed with valid inputs; got Err: {:?}",
        seed_result.err()
    );
    // Both type-check at the same level — fallibility is consistent.
}

/// RG-12 (part C): Distinct seeds produce disjoint alert ID sets.
///
/// Verifies INV-DISTINCT-DATA-001.
///
/// LOAD-BEARING: directly inspects generated_records from both clones.
#[test]
fn test_BC_2_06_018_cyberint_new_with_seed_disjoint_ids() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);

    let clone_100 = CyberintClone::new_with_seed(100, org.clone()).expect("seed=100 must succeed");
    let clone_200 = CyberintClone::new_with_seed(200, org).expect("seed=200 must succeed");

    let alert_ids_100: std::collections::HashSet<String> = clone_100
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .filter(|s| s.contains(&format!("{org_slug}")))
        .collect();

    let alert_ids_200: std::collections::HashSet<String> = clone_200
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .filter(|s| s.contains(&format!("{org_slug}")))
        .collect();

    assert!(
        !alert_ids_100.is_empty(),
        "clone_100 generated_records must contain alert records with org_slug in alert_id"
    );
    assert!(
        !alert_ids_200.is_empty(),
        "clone_200 generated_records must contain alert records with org_slug in alert_id"
    );

    // Seed is embedded in the ID: "alert-{org_slug}-100-{n}" vs "alert-{org_slug}-200-{n}"
    let intersection: std::collections::HashSet<&String> =
        alert_ids_100.intersection(&alert_ids_200).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001: alert_ids for seed=100 and seed=200 must be disjoint; \
         intersection={intersection:?}. org_slug={org_slug}"
    );
}
