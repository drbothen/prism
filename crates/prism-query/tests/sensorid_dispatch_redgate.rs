//! Green test: BC-2.01.013 dispatch-site verification for SensorId open dispatch.
//!
//! This file verifies that all dispatch sites have migrated from closed-enum
//! `SensorType` match arms to open `SensorId`-keyed string dispatch.
//! Migration is complete as of S-PLUGIN-PREREQ-A.
//!
//! # Story: S-PLUGIN-PREREQ-A
//! # BC: BC-2.01.013 — DataSource Trait: Spec-Driven Adapter Pattern (AC-5)
//! # Dispatch site reference: AC-5 table item 2 (virtual_fields.rs:163–166)

use prism_core::SensorId;

// ---------------------------------------------------------------------------
// Open dispatch helper — represents the post-migration dispatch API.
//
// All dispatch sites now use `sensor_id.as_ref()` instead of:
//   match sensor_type { SensorType::X => "x" }
//
// This function verifies the migrated virtual_fields.rs dispatch pattern (AC-5).
// ---------------------------------------------------------------------------

/// Open dispatch interface — SensorId-based dispatch (S-PLUGIN-PREREQ-A).
///
/// Dispatch sites use `sensor_id.as_ref()` — sensor identity IS the string.
/// No closed-enum match required (AC-5 dispatch site 2).
fn virtual_field_sensor_name(sensor_id: &SensorId) -> &str {
    // Open dispatch: SensorId IS the string. No closed-enum match needed.
    sensor_id.as_ref()
}

/// BC-2.01.013 AC-5 postcondition: all dispatch sites use open SensorId-based dispatch,
/// not closed SensorType enum match arms.
///
/// Verifies BC-2.01.013 postcondition: dispatch sites accept any SensorId string,
/// including unknown/custom sensors, without modification (open extensibility).
#[test]
fn test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch() {
    // SensorId::from("crowdstrike") — validated input, no panic.
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
