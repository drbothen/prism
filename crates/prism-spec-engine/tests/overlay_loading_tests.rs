//! Red Gate tests for S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Per-Org Sensor Endpoint
//! Overlay Loading (ADR-029).
//!
//! ALL tests in this file call stubbed functions and MUST FAIL until the
//! implementer fills in the real logic.  This is the Red Gate — do NOT
//! pre-implement any business logic in the stubs.
//!
//! Test names correspond 1:1 to the AC Red Gate test names in the story spec.
//! Adding tests here without a corresponding AC is forbidden (SID-1 §5).
//!
//! Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
//! BCs: BC-2.06.012 through BC-2.06.016

/// Placeholder test — confirms this file compiles and the test binary
/// includes the overlay_loading_tests module.  This placeholder panics
/// with a clear message to confirm the Red Gate is held.
///
/// The real Red Gate tests (test_BC_2_06_012_overlay_discovered_and_merged,
/// etc.) will be authored by the test-writer in the NEXT step.  This
/// placeholder is INTENTIONALLY not a full AC test — it is a scaffolding
/// marker only.
#[test]
fn placeholder_red_gate_stub() {
    panic!(
        "S-CONFIG-MULTI-TENANT-OVERRIDE-001: overlay_loading_tests placeholder — \
         Red Gate tests must be authored by test-writer before implementation. \
         This panic confirms the Red Gate is held (BC-5.38.001)."
    );
}
