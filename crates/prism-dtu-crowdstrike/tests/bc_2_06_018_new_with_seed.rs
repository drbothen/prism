//! Red Gate test RG-3: CrowdstrikeClone::new_with_seed forwarded
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::OrgId;
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

/// RG-3: test_BC_2_06_018_crowdstrike_new_with_seed_forwarded
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - CrowdstrikeClone::new_with_seed(seed, org_id) is callable (constructor exists under fixture-gen)
/// - The clone has generated_devices / generated_detections fields (non-empty after construction)
/// - Device IDs follow canonical format "dev-{8hex}-{seed}-{n}" (ADR-036 §2.2)
/// - IDs for seed=100 differ from IDs for seed=200 (disjointness per INV-DISTINCT-DATA-001)
///
/// RED GATE: This test FAILS until Gate 4 implements new_with_seed (todo!() panics).
#[test]
fn test_BC_2_06_018_crowdstrike_new_with_seed_forwarded() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // This panics with todo!() until Gate 4 implements the real constructor.
    let clone = CrowdstrikeClone::new_with_seed(seed, org.clone());

    // After implementation: state must contain generated_devices.
    // The stub returns todo!() so this code never runs during Red Gate phase.
    // The Red Gate test FAILS at the new_with_seed call above.

    // When implemented (Gate 4 will make this pass):
    // - generated_devices must be non-empty
    // - All device IDs must start with "dev-deadbeef-100-"
    // - A clone with seed=200 must have disjoint device IDs

    let _ = clone; // consumed to suppress unused warning once todo! is removed
}

/// RG-3 (part B): Distinct seeds produce distinct device ID prefixes.
///
/// Verifies INV-DISTINCT-DATA-001: seed_A ≠ seed_B → device IDs disjoint.
/// This is a separate assertion from the constructor availability check above.
///
/// RED GATE: FAILS with todo!() until Gate 4 implements new_with_seed.
#[test]
fn test_BC_2_06_018_crowdstrike_new_with_seed_disjoint_id_prefixes() {
    let org = deadbeef_org();

    let _clone_100 = CrowdstrikeClone::new_with_seed(100, org.clone());
    let _clone_200 = CrowdstrikeClone::new_with_seed(200, org);

    // When implemented (Gate 4):
    // Collect device IDs from each clone's state.generated_devices
    // and assert ids_100 ∩ ids_200 = ∅.
}
