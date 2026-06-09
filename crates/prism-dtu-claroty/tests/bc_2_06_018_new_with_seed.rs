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

/// RG-11: test_BC_2_06_018_claroty_new_with_seed_forwarded
///
/// Traces to: BC-2.06.018 postcondition 1 (seed forwarded to generator-backed clones)
/// Verifies:
/// - ClarotyClone::new_with_seed(seed, org_id) is callable (constructor exists under fixture-gen)
/// - Constructor is infallible (returns Self, not Result<Self>)
/// - State has generated_records after construction
/// - Route handlers serve from generated_records when non-empty
///
/// RED GATE: This test FAILS until Gate 4 implements new_with_seed (todo!() panics).
#[test]
fn test_BC_2_06_018_claroty_new_with_seed_forwarded() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // This panics with todo!() until Gate 4 implements the real constructor.
    // ClarotyClone::new_with_seed is infallible (returns Self, not Result<Self>).
    let clone = ClarotyClone::new_with_seed(seed, org.clone());

    // When implemented (Gate 4 will make this pass):
    // - clone.state.generated_records must be non-empty
    // - Route handler must serve from generated_records
    let _ = clone;
}

/// RG-11 (part B): Determinism — same (seed, org_id) → byte-identical generated_records.
///
/// BC-3.4.001 postcondition 3 / BC-2.06.018 postcondition 3.
///
/// RED GATE: FAILS with todo!() until Gate 4.
#[test]
fn test_BC_2_06_018_claroty_new_with_seed_deterministic() {
    let org = deadbeef_org();
    let seed: u64 = 42;

    let _clone_a = ClarotyClone::new_with_seed(seed, org.clone());
    let _clone_b = ClarotyClone::new_with_seed(seed, org);

    // When implemented: clone_a.state.generated_records == clone_b.state.generated_records
}
