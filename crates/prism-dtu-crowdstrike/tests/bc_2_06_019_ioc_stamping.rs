//! Red Gate tests for S-DEMO-ENRICHMENT-PIVOT-003 — CrowdStrike IOC stamping.
//!
//! Covers:
//!   Test 6: `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns`
//!
//! NOTE: Test 5 (`test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped`)
//! exercises `make_detection_with_ioc()` which is `pub(crate)` and cannot be called
//! from an external test file. That test is written as an in-crate `#[cfg(test)]` unit
//! test in `crates/prism-dtu-crowdstrike/src/generator.rs`.
//!
//! Story: S-DEMO-ENRICHMENT-PIVOT-003
//! Traces to:
//!   BC-2.06.019 v1.12 PC-4 — CrowdStrike detections IOC stamp in behaviors[] array
//!   SAP-2 — CrowdStrike detection TOML columns must match generator.rs key set
//!   AC-005 — CrowdStrike detections TOML spec declares behaviors[] IOC columns
//!
//! FAIL mode (Red Gate):
//!   Test 6: `crowdstrike.sensor.toml` does NOT yet contain behaviors[] IOC columns
//!     → assertion fails.
//!
//! Run:
//!   cargo test -p prism-dtu-crowdstrike --features fixture-gen \
//!       --test bc_2_06_019_ioc_stamping -- --nocapture

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

// ---------------------------------------------------------------------------
// Test 6 — CrowdStrike detection TOML spec has behaviors[] IOC columns
// ---------------------------------------------------------------------------

/// Test 6 — BC-2.06.019 v1.12 PC-4 + SAP-2: the CrowdStrike sensor TOML spec must
/// declare columns for the `behaviors[]` IOC fields stamped by `make_detection()`.
///
/// Expected columns (per AC-005):
///   `behaviors[].ioc_type`
///   `behaviors[].ioc_value`
///   `behaviors[].ioc_source`
///   `behaviors[].ioc_description`
///
/// These column names correspond to the JSON keys stamped in `behaviors[0]` by
/// `make_detection_with_ioc()` in `src/generator.rs` (AC-004 scope).
///
/// SAP-2 parity note (U19): CrowdStrike detection records are untyped `serde_json::Value`.
/// The SAP-2 check reads `src/generator.rs` `make_detection_with_ioc()` return value AND
/// `fixtures/detections-detail.json`, NOT types.rs structs (no typed Detection struct exists).
/// This test verifies the TOML spec side of the parity.
///
/// FAIL mode: neither `sensors/crowdstrike.sensor.toml` nor
///   `crates/prism-sensors/specs/crowdstrike.sensor.toml` currently contains these
///   behaviors[] IOC columns → at least one assertion fails.
///
/// BC-2.06.019 v1.12 PC-4, AC-005, SAP-2.
/// Red Gate test plan #6 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let toml_paths = [
        workspace_root.join("sensors/crowdstrike.sensor.toml"),
        workspace_root.join("crates/prism-sensors/specs/crowdstrike.sensor.toml"),
    ];

    // Column names MUST match the JSON keys stamped by make_detection_with_ioc()
    // in src/generator.rs (AC-004). The TOML column names ARE the JSON key names.
    let required_columns = [
        "behaviors[].ioc_type",
        "behaviors[].ioc_value",
        "behaviors[].ioc_source",
        "behaviors[].ioc_description",
    ];

    for toml_path in &toml_paths {
        let content = std::fs::read_to_string(toml_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read crowdstrike sensor TOML at {:?}: {e}. \
                 The file must exist after this story ships.",
                toml_path
            )
        });

        for col in &required_columns {
            assert!(
                content.contains(col),
                "BC-2.06.019 v1.12 PC-4 / AC-005 / SAP-2: crowdstrike.sensor.toml at {:?} \
                 must declare column '{}' (behaviors[] IOC field stamped by \
                 make_detection_with_ioc() in src/generator.rs). \
                 This column is absent — TOML spec update is required as part of \
                 S-DEMO-ENRICHMENT-PIVOT-003. \
                 SAP-2: column in TOML with no generator equivalent = P1 CRITICAL; \
                 generator key with no TOML column = MEDIUM (missing coverage).",
                toml_path,
                col
            );
        }
    }
}
