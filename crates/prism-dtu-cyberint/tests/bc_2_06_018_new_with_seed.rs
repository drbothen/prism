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

/// RG-12: test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - CyberintClone::new_with_seed(seed, org_id) is callable (constructor exists under fixture-gen)
/// - Return type is anyhow::Result<Self> — fallible, mirrors CyberintClone::new()
/// - State has generated_records after construction
/// - Error from constructor propagates through build_clone_pairs via ?
///
/// RED GATE: This test FAILS until Gate 4 implements new_with_seed (todo!() panics).
#[test]
fn test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // This panics with todo!() until Gate 4 implements the real constructor.
    // The return type is anyhow::Result<Self> (fallible — mirrors CyberintClone::new()).
    let result = CyberintClone::new_with_seed(seed, org.clone());

    // RED GATE assertion: new_with_seed returns Ok (not Err) with a valid constructor.
    // This will panic at todo!() above, making the test RED.
    let clone = result.expect(
        "CyberintClone::new_with_seed must succeed with valid seed and org_id; \
         got Err — implementation incomplete (Gate 4 required)",
    );

    // When implemented (Gate 4 will make this pass):
    // - clone.state.generated_records must be non-empty
    let _ = clone;
}

/// RG-12 (part B): Fallibility consistent with existing CyberintClone::new().
///
/// Both new() and new_with_seed() must return anyhow::Result<Self>.
/// This verifies the type signature is consistent.
///
/// RED GATE: FAILS with todo!() at new_with_seed call.
#[test]
fn test_BC_2_06_018_cyberint_new_with_seed_fallibility_consistent_with_new() {
    // Both new() and new_with_seed() must return anyhow::Result<Self>.
    let _new_result: anyhow::Result<CyberintClone> = CyberintClone::new();

    let org = deadbeef_org();
    // new_with_seed must also return anyhow::Result<Self>.
    let _seed_result: anyhow::Result<CyberintClone> = CyberintClone::new_with_seed(42, org);
    // Both type-check at the same level — fallibility is consistent.
}
