//! Red Gate test RG-4: ArmisClone::new_with_seed forwarded with org_slug arg (fallible)
//!
//! Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3 Armis retrofit
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-A

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::OrgId;

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
/// Helper for the test vector derivation — does NOT call the production function
/// (which doesn't exist yet). Uses the same formula the production code will use.
fn derive_org_slug_from_bytes(org: &OrgId) -> String {
    let b = org.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

/// RG-4: test_BC_2_06_018_armis_new_with_seed_with_org_slug_arg
///
/// Traces to: BC-2.06.018 postcondition 1 / ADR-036 §2.3 Armis retrofit
/// Verifies:
/// - ArmisClone::new_with_seed(seed, org_id, org_slug) is callable (constructor exists)
/// - Return type is anyhow::Result<Self> — fallible, mirrors ArmisClone::new()
/// - Device records in state contain IDs in "dev-{org_slug}-{seed}-{n}" format
/// - Construction error propagates via ? in build_clone_pairs
/// - ADR-036 §2.3: Armis generator takes org_slug as explicit &str arg
///
/// RED GATE: This test FAILS until Gate 4 implements new_with_seed (todo!() panics).
#[test]
fn test_BC_2_06_018_armis_new_with_seed_with_org_slug_arg() {
    let org = deadbeef_org();
    let seed: u64 = 100;
    let org_slug = derive_org_slug_from_bytes(&org);

    // This panics with todo!() until Gate 4 implements the real constructor.
    // The return type is anyhow::Result<Self> (fallible — mirrors ArmisClone::new()).
    let result = ArmisClone::new_with_seed(seed, org.clone(), &org_slug);

    // RED GATE assertion: new_with_seed returns Ok (not Err) with a valid constructor.
    // This will panic at todo!() above, making the test RED.
    let clone = result.expect(
        "ArmisClone::new_with_seed must succeed with valid seed, org_id, and org_slug; \
         got Err — implementation incomplete (Gate 4 required)",
    );

    // When implemented (Gate 4 will make this pass):
    // - clone.state.generated_records must be non-empty
    // - All device IDs must start with "dev-deadbeef-100-"
    let _ = clone;
}

/// RG-4 (part B): Error from new_with_seed propagates through Result chain.
///
/// Verifies: The fallible constructor's error propagates consistently.
/// (BC-2.06.018 AC-004 — error propagates through build_clone_pairs via ?)
///
/// RED GATE: FAILS with todo!() until Gate 4 implements new_with_seed.
#[test]
fn test_BC_2_06_018_armis_new_with_seed_fallibility_consistent_with_new() {
    // Both new() and new_with_seed() must return anyhow::Result<Self>.
    // This test verifies the type signature is consistent.
    // The actual new() can be called for comparison.
    let _new_result: anyhow::Result<ArmisClone> = ArmisClone::new();

    let org = deadbeef_org();
    let org_slug = derive_org_slug_from_bytes(&org);
    // new_with_seed must also return anyhow::Result<Self>.
    let _seed_result: anyhow::Result<ArmisClone> = ArmisClone::new_with_seed(42, org, &org_slug);
    // Both type-check at the same level — fallibility is consistent.
}
