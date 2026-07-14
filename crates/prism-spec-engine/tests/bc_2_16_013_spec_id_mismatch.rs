#![allow(non_snake_case)]
//! RG-09 / HS-018: BC-2.16.001 — Sensor spec_id / Filename Mismatch Rejected
//!
//! Tests that SpecLoader::load_all() rejects a spec file where the sensor_id
//! value does not case-sensitively match the filename stem, emitting E-SPEC-017.
//!
//! Red Gate discipline (BC-5.38.001): ALL tests MUST FAIL before the implementer
//! completes Task 11 (SpecErrorCode::ESpec017 variant in prism-core/src/error.rs)
//! and Task 12 (filename-stem-vs-sensor_id check in SpecLoader::load_all()).
//!
//! CRITICAL: SpecLoader::parse(toml_input: &str) MUST NOT be used as the driver
//! for RG-09 — it cannot emit E-SPEC-017 because it accepts only TOML content
//! with no filename context. SpecLoader::load_all() is the correct driver.
//!
//! AC coverage: (implicit — spec validation correctness)
//! HS coverage: HS-018

use prism_core::{PrismError, SpecErrorCode};
use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// RG-09 / HS-018: sensor_id / filename mismatch rejected at load time
// ---------------------------------------------------------------------------

/// RG-09 / HS-018 / BC-2.16.001 §Error Conditions E-SPEC-017:
/// test_BC_2_16_001_RG_09_filename_stem_mismatch_emits_E_SPEC_017
///
/// Write a temp spec file crowdstrike.sensor.toml with sensor_id = "falcon"
/// to a temp dir, call SpecLoader::load_all(), assert returned errors contain
/// a SpecErrorCode::ESpec017 entry.
///
/// RED GATE: Fails until:
///   - Task 11: SpecErrorCode::ESpec017 variant added to prism-core/src/error.rs
///   - Task 12: filename-stem check added to SpecLoader::load_all()
///
/// Without Task 11, this test fails to compile (ESpec017 variant does not exist).
/// Without Task 12, load_all() does not emit E-SPEC-017 and the assertion fails.
#[test]
fn test_BC_2_16_001_RG_09_filename_stem_mismatch_emits_E_SPEC_017() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed");
    let dir_path = dir.path();

    // Write a spec file named crowdstrike.sensor.toml but with sensor_id = "falcon".
    // This is the canonical HS-018-01 mismatch case.
    std::fs::write(
        dir_path.join("crowdstrike.sensor.toml"),
        r#"
sensor_id = "falcon"
name = "CrowdStrike Falcon (mismatched ID)"
auth_type = "oauth2_client_credentials"
base_url = "https://api.crowdstrike.com"
version = "1.0.0"

[[tables]]
table_name = "detections"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/detects/queries/detects/v1"
  response_path = "$.resources"
  variables_produced = []
"#,
    )
    .expect("write mismatch spec");

    // Use SpecLoader::load_all() — NOT SpecLoader::parse() — because only load_all()
    // has the filename context required to detect the mismatch (BC-2.16.001 §Error Conditions).
    let loader = SpecLoader::new(dir_path.to_str().expect("valid path").to_string());
    let (descriptors, errors) = loader.load_all();

    // The mismatched spec must NOT produce a descriptor (it must be rejected entirely).
    assert!(
        descriptors.is_empty(),
        "Mismatched spec 'crowdstrike.sensor.toml' with sensor_id='falcon' must NOT \
         produce any descriptor; got {} descriptors: {:?}",
        descriptors.len(),
        descriptors
            .iter()
            .map(|d| &d.table_name)
            .collect::<Vec<_>>()
    );

    // The error collection must contain at least one E-SPEC-017 error.
    assert!(
        !errors.is_empty(),
        "Mismatched spec must produce at least one error (E-SPEC-017); got zero errors"
    );

    let has_e_spec_017 = errors.iter().any(|e| {
        if let PrismError::Spec(spec_err) = e {
            matches!(spec_err.code, SpecErrorCode::ESpec017)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_017,
        "Errors from mismatched spec must include SpecErrorCode::ESpec017; \
         actual errors: {:?}",
        errors
    );
}

/// HS-018-01 extended: Mismatch with a completely different sensor name.
/// sensor_id = "claroty" in a file named cyberint.sensor.toml must also be rejected.
///
/// RED GATE: Fails until Task 11 + Task 12 are complete.
#[test]
fn test_HS_018_BC_2_16_001_different_sensor_name_mismatch_rejected() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed");
    let dir_path = dir.path();

    // File name says cyberint, sensor_id says claroty.
    std::fs::write(
        dir_path.join("cyberint.sensor.toml"),
        r#"
sensor_id = "claroty"
name = "Wrong Sensor"
auth_type = "bearer_static"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "alerts"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/alerts"
  response_path = "$.data"
  variables_produced = []
"#,
    )
    .expect("write mismatch spec");

    let loader = SpecLoader::new(dir_path.to_str().expect("valid path").to_string());
    let (descriptors, errors) = loader.load_all();

    assert!(
        descriptors.is_empty(),
        "Mismatched spec (cyberint file, claroty ID) must produce no descriptors"
    );

    let has_e_spec_017 = errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec017)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_017,
        "Mismatch between file stem 'cyberint' and sensor_id 'claroty' must emit E-SPEC-017"
    );
}

/// HS-018 / RG-09 (regression net — NOT a Red Gate per BC-5.38.001):
///
/// FALSE-POSITIVE GUARD for the E-SPEC-017 (filename-stem-vs-sensor_id) validator added
/// by implementer Tasks 11+12. A spec with `sensor_id = "crowdstrike"` in a file named
/// `crowdstrike.sensor.toml` MUST NOT trigger E-SPEC-017 — the validator only fires on
/// actual mismatches.
///
/// Discipline: this is a regression test, NOT a Red Gate. Red Gate tests (BC-5.38.001)
/// must FAIL before the implementer makes them pass; this test PASSES in both the
/// Red Gate state (no validator exists yet, so no false-positive is possible) AND in the
/// implemented state (validator correctly distinguishes match from mismatch). The actual
/// Red Gate tests in this file are tests #1 and #2 above — those reference
/// `SpecErrorCode::ESpec017` which fails to compile until Task 11 lands, and asserts the
/// mismatch error is emitted which fails until Task 12 adds the filename-stem check.
///
/// POLICY 16 alignment: this test is NOT an inverted-polarity test. It does not assert
/// that production code panics with a stub message. It asserts the correct-input case
/// (matching filename stem and sensor_id) produces no E-SPEC-017 error.
#[test]
fn test_HS_018_BC_2_16_001_correct_filename_sensor_id_pair_no_error() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed");
    let dir_path = dir.path();

    // Correct pairing: crowdstrike.sensor.toml + sensor_id = "crowdstrike".
    std::fs::write(
        dir_path.join("crowdstrike.sensor.toml"),
        r#"
sensor_id = "crowdstrike"
name = "CrowdStrike Falcon"
auth_type = "oauth2_client_credentials"
base_url = "https://api.crowdstrike.com"
version = "1.0.0"

[[tables]]
table_name = "detections"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/detects/queries/detects/v1"
  response_path = "$.resources"
  variables_produced = []
"#,
    )
    .expect("write correct spec");

    let loader = SpecLoader::new(dir_path.to_str().expect("valid path").to_string());
    let (descriptors, errors) = loader.load_all();

    // No E-SPEC-017 error for correct pairing.
    let has_e_spec_017 = errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec017)
        } else {
            false
        }
    });

    assert!(
        !has_e_spec_017,
        "Correct sensor_id/filename pairing must NOT emit E-SPEC-017 (false positive prevention). \
         Errors: {:?}",
        errors
    );

    // Must produce at least one descriptor (the detections table).
    assert!(
        !descriptors.is_empty(),
        "Correct sensor_id/filename pairing must produce descriptors; got none"
    );
}
