//! Red Gate test RG-3: CrowdstrikeClone::new_with_seed forwarded
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{Archetype, OrgId};
use prism_dtu_crowdstrike::CrowdstrikeClone;

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

/// Derive canonical org_slug from the first 4 bytes of OrgId (ADR-036 §2.2).
fn derive_org_slug(org: &OrgId) -> String {
    let b = org.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

/// RG-3: test_BC_2_06_018_crowdstrike_new_with_seed_forwarded
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - CrowdstrikeClone::new_with_seed(seed, archetype, org_id) is callable (constructor exists under fixture-gen)
/// - state.generated_devices is non-empty after construction
/// - All device IDs follow canonical format "dev-{org_slug}-{seed}-{n}" (ADR-036 §2.2)
/// - state.generated_detections is non-empty after construction
///
/// LOAD-BEARING: assertions directly inspect the constructed clone's state.
/// A broken impl (empty generated_devices) causes this test to FAIL.
#[test]
fn test_BC_2_06_018_crowdstrike_new_with_seed_forwarded() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);
    let seed: u64 = 100;

    // CompromisedEndpoint: 50 device records + 20 detection records (non-empty, verifiable).
    let clone = CrowdstrikeClone::new_with_seed(seed, Archetype::CompromisedEndpoint, org.clone());

    // (a) generated_devices must be non-empty (BC-2.06.018 postcondition 1).
    let generated_devices = &clone.state.generated_devices;
    assert!(
        !generated_devices.is_empty(),
        "CrowdstrikeClone::new_with_seed must populate state.generated_devices; got empty vec. \
         BC-2.06.018 postcondition 1 violated."
    );

    // (b) Every device record must have device_id starting with "dev-{org_slug}-{seed}-".
    let expected_prefix = format!("dev-{}-{}-", org_slug, seed);
    for (i, rec) in generated_devices.iter().enumerate() {
        let device_id = rec
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("generated_devices[{i}] missing 'device_id' field: {rec}"));
        assert!(
            device_id.starts_with(&expected_prefix),
            "generated_devices[{i}].device_id must start with '{expected_prefix}'; \
             got '{device_id}'. ADR-036 §2.2 canonical format violated."
        );
    }

    // (c) generated_detections must also be non-empty.
    let generated_detections = &clone.state.generated_detections;
    assert!(
        !generated_detections.is_empty(),
        "CrowdstrikeClone::new_with_seed must populate state.generated_detections; got empty vec. \
         BC-2.06.018 postcondition 1 violated."
    );
}

/// RG-3 (part B): Distinct seeds produce disjoint device ID sets.
///
/// Verifies INV-DISTINCT-DATA-001: seed_A ≠ seed_B → device IDs disjoint.
///
/// LOAD-BEARING: directly inspects generated_devices from both clones and asserts
/// set intersection is empty. A broken impl (both serving same static fixtures)
/// would NOT satisfy this test if both seeds also generated the same IDs.
#[test]
fn test_BC_2_06_018_crowdstrike_new_with_seed_disjoint_id_prefixes() {
    let org = deadbeef_org();
    let org_slug = derive_org_slug(&org);

    let clone_100 =
        CrowdstrikeClone::new_with_seed(100, Archetype::CompromisedEndpoint, org.clone());
    let clone_200 = CrowdstrikeClone::new_with_seed(200, Archetype::CompromisedEndpoint, org);

    let ids_100: std::collections::HashSet<String> = clone_100
        .state
        .generated_devices
        .iter()
        .filter_map(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    let ids_200: std::collections::HashSet<String> = clone_200
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
        !ids_100.is_empty(),
        "clone_100 generated_devices must be non-empty (seed=100)"
    );
    assert!(
        !ids_200.is_empty(),
        "clone_200 generated_devices must be non-empty (seed=200)"
    );

    // IDs for seed=100 use prefix "dev-deadbeef-100-{n}"; seed=200 uses "dev-deadbeef-200-{n}".
    // They are structurally disjoint by seed embedding.
    let intersection: std::collections::HashSet<&String> = ids_100.intersection(&ids_200).collect();
    assert!(
        intersection.is_empty(),
        "INV-DISTINCT-DATA-001 violated: device IDs for seed=100 and seed=200 must be disjoint; \
         intersection={intersection:?}. org_slug={org_slug}"
    );

    // Also verify both sets use the correct org_slug prefix.
    for id in &ids_100 {
        assert!(
            id.contains(&org_slug),
            "seed=100 device_id '{id}' must contain org_slug '{org_slug}'"
        );
    }
    for id in &ids_200 {
        assert!(
            id.contains(&org_slug),
            "seed=200 device_id '{id}' must contain org_slug '{org_slug}'"
        );
    }
}
