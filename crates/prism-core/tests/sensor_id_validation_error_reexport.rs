#![allow(non_snake_case)]
//! AC-7 Red Gate test: `SensorIdValidationError` crate-root re-export.
//!
//! BC-2.01.013 postcondition: the spec-driven sensor identifier surface exposes
//! a consistent error type at the crate root — ergonomic parity with `SensorId`
//! which is already re-exported at `prism_core::SensorId` per PREREQ-A.
//!
//! AC-7 resolves TD-S-PLUGIN-PREREQ-A-008 P3.
//!
//! RED GATE MECHANISM: `SensorIdValidationError` is currently accessible only via
//! `prism_core::sensor_id::SensorIdValidationError` (the module path). The
//! crate-root re-export (`pub use sensor_id::SensorIdValidationError;` in lib.rs)
//! does NOT exist yet.
//!
//! The test below verifies the EXPECTED postcondition by scanning lib.rs for the
//! re-export line and failing if it is absent. This allows the test FILE to compile
//! (avoiding blocking AC-6 and other prism-core tests) while still being RED until
//! AC-7 is implemented.
//!
//! A companion compile-fail assertion is in the doc-comment of the re-export line
//! itself once AC-7 is implemented (per story spec: "doctest on the re-export line").
//!
//! After AC-7 implementation:
//!   1. The re-export line `pub use sensor_id::SensorIdValidationError;` is added to lib.rs.
//!   2. This test PASSES (the scan finds the re-export).
//!   3. The doctest on the re-export line passes (`cargo test --doc -p prism-core`).

use prism_core::sensor_id::SensorIdValidationError;

/// test_AC_7_sensor_id_validation_error_reexport_at_crate_root
///
/// Scans `crates/prism-core/src/lib.rs` for the re-export line
/// `pub use sensor_id::SensorIdValidationError;`. Fails until the line is present.
///
/// RED GATE: the re-export does not exist in lib.rs yet. This test FAILS.
///
/// Traces to BC-2.01.013 postcondition: sensor identifier error type is accessible
/// at crate root, matching the ergonomic pattern established by `SensorId` in PREREQ-A.
#[test]
fn test_AC_7_sensor_id_validation_error_reexport_at_crate_root() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    let lib_rs_path = std::path::Path::new(&manifest_dir).join("src/lib.rs");

    let lib_content = std::fs::read_to_string(&lib_rs_path).unwrap_or_else(|e| {
        panic!("Failed to read {}: {}", lib_rs_path.display(), e);
    });

    // The re-export line must be present in lib.rs.
    // Acceptable forms (either works):
    let has_reexport = lib_content.contains("pub use sensor_id::SensorIdValidationError")
        || lib_content.contains("pub use sensor_id::{SensorId, SensorIdValidationError}")
        || lib_content.contains("pub use sensor_id::{SensorIdValidationError, SensorId}");

    assert!(
        has_reexport,
        "AC-7 RED GATE: `pub use sensor_id::SensorIdValidationError` not found in \
         prism-core/src/lib.rs.\n\
         IMPLEMENTATION NEEDED (AC-7): add `pub use sensor_id::SensorIdValidationError;` \
         to lib.rs alongside the existing `pub use sensor_id::SensorId;` re-export.\n\
         After adding, also add a doctest showing `use prism_core::SensorIdValidationError;` \
         compiles."
    );
}

/// Verify the module-path access works (baseline check — expected to PASS).
///
/// `prism_core::sensor_id::SensorIdValidationError` must be accessible.
/// This is the module path that works BEFORE AC-7 is implemented.
#[test]
fn test_AC_7_sensor_id_validation_error_module_path_accessible() {
    // Module path always works (no re-export needed for this).
    let err = prism_core::SensorId::try_from_str("").unwrap_err();
    // The error type is the same regardless of which path we use.
    let _: SensorIdValidationError = err;
}

/// Verify the error type has the expected variant for empty-string input.
///
/// This is a baseline test to ensure the type is functional (not just importable).
/// Expected to PASS before and after AC-7.
#[test]
fn test_AC_7_sensor_id_validation_error_empty_string_variant() {
    let err = prism_core::SensorId::try_from_str("").unwrap_err();
    match err {
        SensorIdValidationError::TooShort => {
            // Expected: empty string → TooShort variant (length 0).
        }
        other => {
            panic!(
                "Expected SensorIdValidationError::TooShort for empty input; got: {:?}",
                other
            );
        }
    }
}
