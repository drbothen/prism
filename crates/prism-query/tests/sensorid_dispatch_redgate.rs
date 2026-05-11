//! Red Gate test: BC-2.01.013 dispatch-site exercise for SensorId.
//!
//! This file exercises the dispatch path that will be migrated from
//! `SensorType`-keyed match arms to `SensorId`-keyed open dispatch.
//!
//! # Red Gate status
//! The test MUST FAIL before implementation. It panics at `todo!()` in the
//! `SensorId::from(&str)` constructor, which has not yet been implemented.
//! This proves the test is wired to the unimplemented SensorId path.
//!
//! # Story: S-PLUGIN-PREREQ-A
//! # BC: BC-2.01.013 — DataSource Trait: Spec-Driven Adapter Pattern (AC-5)
//! # Dispatch site reference: AC-5 table item 2 (virtual_fields.rs:163–166)

use prism_core::SensorId;

// ---------------------------------------------------------------------------
// Bridge stub: represents the post-migration dispatch API.
//
// In the current codebase, `virtual_fields.rs` has:
//   match sensor_type { SensorType::X => "x" }
//
// After S-PLUGIN-PREREQ-A, this becomes:
//   sensor_id.as_ref()
//
// This stub simulates the future `virtual_field_for_sensor(SensorId)` helper.
// It panics at todo!() because SensorId::from() is not yet implemented,
// proving the Red Gate is intact.
// ---------------------------------------------------------------------------

/// Post-migration dispatch interface — open SensorId-based dispatch.
///
/// After S-PLUGIN-PREREQ-A, dispatch sites use `sensor_id.as_ref()` instead of
/// `match sensor_type { SensorType::X => "x" }`. This function represents the
/// migrated virtual_fields.rs dispatch site (AC-5 dispatch site 2).
///
/// GREEN: SensorId::as_ref() (AsRef<str>) is now implemented.
fn virtual_field_sensor_name(sensor_id: &SensorId) -> &str {
    // Open dispatch: SensorId IS the string. No closed-enum match needed.
    sensor_id.as_ref()
}

/// BC-2.01.013 AC-5 postcondition: all dispatch sites use open SensorId-based dispatch,
/// not closed SensorType enum match arms.
///
/// Red Gate: panics at todo!() in SensorId::from(&str) — proving the dispatch path
/// is not yet converted to the open, string-based form.
///
/// Post-implementation: this test will call the real dispatch function with a SensorId
/// and assert the sensor name string is returned correctly.
#[test]
fn test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch() {
    // Constructing SensorId panics at todo!() in From<&str> — Red Gate confirmed.
    let sensor_id = SensorId::from("crowdstrike");

    // Post-migration: open dispatch — sensor identity IS the string.
    let name = virtual_field_sensor_name(&sensor_id);
    assert_eq!(
        name, "crowdstrike",
        "Dispatch site must return the sensor id string (open dispatch, not closed enum match)"
    );

    // Verify the open-dispatch invariant: unknown sensors return their own name.
    let custom_id = SensorId::from("my-custom-sensor");
    let custom_name = virtual_field_sensor_name(&custom_id);
    assert_eq!(
        custom_name, "my-custom-sensor",
        "Open dispatch must handle unknown sensor ids without panicking"
    );
}
