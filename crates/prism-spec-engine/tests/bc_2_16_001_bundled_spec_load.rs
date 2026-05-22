#![allow(non_snake_case)]
//! RG-01 / AC-006: BC-2.16.001 — Bundled Sensor Spec File Loading
//!
//! Tests that all 4 production TOML sensor spec files are discovered, parsed,
//! and descriptors produced by SpecLoader::load_all() when pointed at
//! crates/prism-sensors/specs/.
//!
//! Red Gate discipline (BC-5.38.001): ALL tests in this file MUST FAIL before
//! the implementer completes Tasks 3–6 (filling out the TOML specs) and Tasks
//! 11–12 (ESpec017 + filename-stem check). Failure reason: the bundled specs
//! are skeleton stubs; the spec_parser will either fail to parse them or the
//! descriptors will not match the expected counts until the implementer finishes.
//!
//! Test naming follows BC-based convention (CLAUDE.md §Conventions):
//!   test_BC_2_16_001_<descriptive>()
//! and for RG anchors:
//!   test_RG_01_<descriptive>()
//!
//! AC coverage: AC-001, AC-002, AC-003, AC-004, AC-006
//! HS coverage: (validation path exercised by HS-017)

use prism_spec_engine::spec_parser::{AuthType, SpecLoader};

// ---------------------------------------------------------------------------
// Path helpers — tests exercise the production load path (not fixtures).
// ---------------------------------------------------------------------------

/// Return the absolute path to crates/prism-sensors/specs/.
///
/// Uses CARGO_MANIFEST_DIR (the prism-spec-engine crate root) to compute the
/// path, ensuring the test works regardless of the working directory.
/// CARGO_MANIFEST_DIR = .../crates/prism-spec-engine
/// Target: .../crates/prism-sensors/specs
fn bundled_specs_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // Navigate: crates/prism-spec-engine → workspace_root → crates/prism-sensors/specs
    std::path::Path::new(manifest_dir)
        .join("../prism-sensors/specs")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(manifest_dir).join("../prism-sensors/specs"))
        .to_string_lossy()
        .to_string()
}

/// Return the absolute path to a file in crates/prism-sensors/specs/.
fn bundled_spec_path(filename: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../prism-sensors/specs")
        .join(filename)
}

// ---------------------------------------------------------------------------
// RG-01 / AC-006: All 4 bundled specs load at boot
// ---------------------------------------------------------------------------

/// RG-01 / AC-006 / BC-2.16.001 §Postconditions 1-3:
/// SpecLoader::load_all() pointed at crates/prism-sensors/specs/ discovers
/// all 4 bundled TOML spec files, parses them without errors, and produces
/// SensorTableDescriptor entries for each table.
///
/// RED GATE: Fails until the 4 TOML specs are production-grade (Tasks 3-6)
/// and SpecLoader::load_all() emits no parse errors for them.
#[test]
fn test_BC_2_16_001_loads_4_bundled_specs_at_boot() {
    let loader = SpecLoader::new(bundled_specs_dir());
    let (descriptors, errors) = loader.load_all();

    // All 4 specs must load without errors (no E-SPEC-001 or E-SPEC-009).
    assert!(
        errors.is_empty(),
        "Expected zero load errors but got {} error(s): {:?}",
        errors.len(),
        errors
    );

    // Collect sensor_ids from descriptors (format: "{sensor_id}.{table_name}")
    let sensor_ids: std::collections::HashSet<String> = descriptors
        .iter()
        .map(|d| d.table_name.split('.').next().unwrap_or("").to_string())
        .collect();

    // All 4 sensors must be present.
    for expected_sensor in &["crowdstrike", "claroty", "cyberint", "armis"] {
        assert!(
            sensor_ids.contains(*expected_sensor),
            "Expected sensor '{}' in loaded descriptors but not found. \
             Loaded sensors: {:?}",
            expected_sensor,
            sensor_ids
        );
    }

    // CrowdStrike must have exactly 3 tables (detections, devices, incidents — AC-001).
    let crowdstrike_tables: Vec<&str> = descriptors
        .iter()
        .filter(|d| d.table_name.starts_with("crowdstrike."))
        .map(|d| d.table_name.as_str())
        .collect();
    assert_eq!(
        crowdstrike_tables.len(),
        3,
        "CrowdStrike must have exactly 3 tables (detections, devices, incidents) \
         per AC-001 — got {:?}",
        crowdstrike_tables
    );

    // Claroty must have >= 2 tables (alerts, audit_logs — AC-002).
    let claroty_table_count = descriptors
        .iter()
        .filter(|d| d.table_name.starts_with("claroty."))
        .count();
    assert!(
        claroty_table_count >= 2,
        "Claroty must have >= 2 tables per AC-002 — got {}",
        claroty_table_count
    );

    // Cyberint must have exactly 2 tables (alerts, incidents — AC-003).
    let cyberint_table_count = descriptors
        .iter()
        .filter(|d| d.table_name.starts_with("cyberint."))
        .count();
    assert_eq!(
        cyberint_table_count, 2,
        "Cyberint must have exactly 2 tables (alerts, incidents) per AC-003 — got {}",
        cyberint_table_count
    );

    // Armis must have exactly 2 tables (devices, alerts — AC-004).
    let armis_table_count = descriptors
        .iter()
        .filter(|d| d.table_name.starts_with("armis."))
        .count();
    assert_eq!(
        armis_table_count, 2,
        "Armis must have exactly 2 tables (devices, alerts) per AC-004 — got {}",
        armis_table_count
    );

    // Virtual fields check: each descriptor must have at least the data columns
    // (sensor and source virtual fields are injected by the query layer, not
    // by SpecLoader::load_all(); verify at minimum that columns is non-empty).
    for descriptor in &descriptors {
        assert!(
            !descriptor.columns.is_empty(),
            "Table '{}' must have at least one column in its descriptor",
            descriptor.table_name
        );
    }
}

/// RG-01 sub-assertion / AC-006 / BC-2.16.001 §Table Registration:
/// The 4 bundled specs produce descriptors with the canonical
/// "{sensor_id}.{table_name}" namespace (e.g., "crowdstrike.detections").
///
/// RED GATE: Fails until the 4 TOML specs are production-grade (Tasks 3-6).
#[test]
fn test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces() {
    let loader = SpecLoader::new(bundled_specs_dir());
    let (descriptors, errors) = loader.load_all();

    assert!(
        errors.is_empty(),
        "Load errors block namespace assertion: {:?}",
        errors
    );

    let table_names: std::collections::HashSet<&str> =
        descriptors.iter().map(|d| d.table_name.as_str()).collect();

    // Required canonical table names (AC-006 §Table Registration with DataFusion).
    let required_tables = [
        "crowdstrike.detections",
        "crowdstrike.devices",
        "crowdstrike.incidents",
        "claroty.alerts",
        "cyberint.alerts",
        "cyberint.incidents",
        "armis.devices",
        "armis.alerts",
    ];

    for required in &required_tables {
        assert!(
            table_names.contains(*required),
            "Required table '{}' not found in loaded descriptors. \
             Found: {:?}",
            required,
            table_names
        );
    }
}

/// RG-01 sub-assertion / AC-006 / BC-2.16.001 §Auth Type Resolution:
/// Each bundled spec declares the correct auth_type per DTU enforcement (ADR-028 §D2).
/// Parse each spec directly to verify auth_type fields.
///
/// RED GATE: Fails until the 4 TOML specs contain correct auth_type values.
#[test]
fn test_BC_2_16_001_bundled_specs_declare_correct_auth_types() {
    // CrowdStrike: oauth2_client_credentials (AC-001, AC-011).
    let cs_content = std::fs::read_to_string(bundled_spec_path("crowdstrike.sensor.toml"))
        .expect("crowdstrike.sensor.toml must exist");
    let cs_spec = SpecLoader::parse(&cs_content).expect("crowdstrike.sensor.toml must parse");
    assert_eq!(
        cs_spec.auth_type,
        AuthType::Oauth2ClientCredentials,
        "CrowdStrike must declare oauth2_client_credentials (AC-011)"
    );

    // Claroty: bearer_static — grounded from DTU (ADR-028 §D2); NOT cookie_roundtrip (AC-002, AC-011).
    let cl_content = std::fs::read_to_string(bundled_spec_path("claroty.sensor.toml"))
        .expect("claroty.sensor.toml must exist");
    let cl_spec = SpecLoader::parse(&cl_content).expect("claroty.sensor.toml must parse");
    assert_eq!(
        cl_spec.auth_type,
        AuthType::BearerStatic,
        "Claroty must declare bearer_static (DTU-grounded per ADR-028 §D2; AC-011)"
    );

    // Cyberint: cookie_roundtrip — grounded from DTU (ADR-028 §D2); NOT bearer_static (AC-003, AC-011).
    let cy_content = std::fs::read_to_string(bundled_spec_path("cyberint.sensor.toml"))
        .expect("cyberint.sensor.toml must exist");
    let cy_spec = SpecLoader::parse(&cy_content).expect("cyberint.sensor.toml must parse");
    assert_eq!(
        cy_spec.auth_type,
        AuthType::CookieRoundtrip,
        "Cyberint must declare cookie_roundtrip (DTU-grounded per ADR-028 §D2; AC-011)"
    );

    // Armis: bearer_static — grounded from DTU (ADR-028 §D2); NOT api_key (AC-004, AC-011).
    let ar_content = std::fs::read_to_string(bundled_spec_path("armis.sensor.toml"))
        .expect("armis.sensor.toml must exist");
    let ar_spec = SpecLoader::parse(&ar_content).expect("armis.sensor.toml must parse");
    assert_eq!(
        ar_spec.auth_type,
        AuthType::BearerStatic,
        "Armis must declare bearer_static (DTU-grounded per ADR-028 §D2; AC-011)"
    );
}

/// AC-006 / BC-2.16.001 §Partial Failure Isolation (DI-030):
/// An invalid spec file in the same directory does NOT prevent valid specs from loading.
/// Uses a tempdir to isolate.
///
/// RED GATE: Passes structurally today but is included so the implementer cannot
/// break DI-030 while adding Task 12 (filename-stem check). When Task 12 lands,
/// this test also exercises partial-failure isolation for E-SPEC-017.
#[test]
fn test_BC_2_16_001_invalid_spec_does_not_block_valid_specs() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed");
    let dir_path = dir.path();

    // Write a valid spec.
    std::fs::write(
        dir_path.join("valid.sensor.toml"),
        r#"
sensor_id = "valid"
name = "Valid Sensor"
auth_type = "bearer_static"
base_url = "https://valid.example.com"
version = "1.0.0"

[[tables]]
table_name = "items"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_items"
  method = "GET"
  path_template = "/api/v1/items"
  response_path = "$.items"
  variables_produced = []
"#,
    )
    .expect("write valid spec");

    // Write a malformed TOML file.
    std::fs::write(
        dir_path.join("malformed.sensor.toml"),
        "sensor_id = \"malformed\nthis is not valid toml [[[",
    )
    .expect("write malformed spec");

    let loader = SpecLoader::new(dir_path.to_str().expect("valid path").to_string());
    let (descriptors, errors) = loader.load_all();

    // The valid spec must produce a descriptor.
    assert!(
        descriptors.iter().any(|d| d.table_name == "valid.items"),
        "valid.items descriptor must be produced despite malformed sibling spec"
    );

    // The malformed spec must produce an error (DI-030 partial-failure isolation).
    assert!(
        !errors.is_empty(),
        "Malformed spec must produce at least one error (DI-030)"
    );
}

/// AC-006 sub-assertion / BC-2.16.001 §Empty-credential scenario (DEC-036):
/// An empty credential scenario (no client credentials configured) must not
/// produce an error from SpecLoader::load_all(). The spec loads successfully;
/// tables are registered; credential resolution failure is a runtime concern.
///
/// RED GATE: This test checks behavior that is already correct in SpecLoader::load_all(),
/// but it becomes a regression test once the implementer fills in the 4 bundled specs.
/// It fails today because the bundled specs are skeleton stubs that may not parse fully.
#[test]
fn test_BC_2_16_001_empty_credential_scenario_not_an_error() {
    // Point at a spec with no credential_refs declared — this is the empty-credential case.
    // Use the bundled crowdstrike spec (which has oauth2 auth but no credential_refs in skeleton).
    let cs_content = std::fs::read_to_string(bundled_spec_path("crowdstrike.sensor.toml"))
        .expect("crowdstrike.sensor.toml must exist");

    // SpecLoader::parse() succeeds even with no credential_refs (they default to empty).
    let spec = SpecLoader::parse(&cs_content)
        .expect("crowdstrike.sensor.toml must parse — empty credential_refs is not an error");

    // DEC-036: empty credential_refs is not an error at parse time.
    assert!(
        spec.credential_refs.is_empty(),
        "Skeleton crowdstrike spec must have empty credential_refs (DEC-036 empty-credential scenario)"
    );
}
