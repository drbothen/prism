#![allow(non_snake_case)]
//! Red Gate tests for BC-2.16.012 -- PluginRegistry Dispatch Migration.
//!
//! Tests 8-12 of the S-PLUGIN-PREREQ-E Red Gate set (prism-spec-engine crate).
//!
//! | Test | Name                                                                         | AC   | Red Gate failure mode                              |
//! |------|------------------------------------------------------------------------------|------|----------------------------------------------------|
//! | 8    | test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch                | AC-7 | Hypothetical sensor parse fails OR grep hits       |
//! | 9    | test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike          | AC-8 | todo!() in validate_cross_composition panics       |
//! | 10   | test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint             | AC-8 | todo!() in validate_cross_composition panics       |
//! | 11   | test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty              | AC-8 | todo!() in validate_cross_composition panics       |
//! | 12   | test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis                | AC-8 | todo!() in validate_cross_composition panics       |
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-001/002/003

use std::path::PathBuf;

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// Canonical inline TOML fixtures -- one per built-in sensor.
// Each fixture uses the correct auth_type discriminator per ADR-026 section D3
// and includes the minimum fields required by SpecLoader::parse.
// ---------------------------------------------------------------------------

const CROWDSTRIKE_FIXTURE_TOML: &str = r#"
sensor_id = "crowdstrike"
name = "CrowdStrike Falcon"
auth_type = "oauth2_client_credentials"
base_url = "https://api.crowdstrike.com"
version = "1.0.0"

[[tables]]
table_name = "detections"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "detection_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_detections"
  method = "GET"
  path_template = "/detections/queries/detections/v2"
  response_path = "$.resources"
  variables_produced = []
"#;

const CYBERINT_FIXTURE_TOML: &str = r#"
sensor_id = "cyberint"
name = "Cyberint"
auth_type = "bearer_static"
base_url = "https://api.cyberint.com"
version = "1.0.0"

[[tables]]
table_name = "alerts"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "alert_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_alerts"
  method = "GET"
  path_template = "/v1/alerts"
  response_path = "$.data"
  variables_produced = []
"#;

const CLAROTY_FIXTURE_TOML: &str = r#"
sensor_id = "claroty"
name = "Claroty xDome"
auth_type = "cookie_roundtrip"
base_url = "https://portal.claroty.com"
version = "1.0.0"

[[tables]]
table_name = "assets"
ocsf_class = "device_inventory"

  [[tables.columns]]
  name = "asset_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_assets"
  method = "POST"
  path_template = "/api/2.0/assets/"
  response_path = "$.objects"
  variables_produced = []
"#;

const ARMIS_FIXTURE_TOML: &str = r#"
sensor_id = "armis"
name = "Armis Centrix"
auth_type = "api_key"
base_url = "https://integration.armis.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_devices"
  method = "GET"
  path_template = "/api/v1/search/"
  response_path = "$.data.results"
  variables_produced = []
"#;

// ---------------------------------------------------------------------------
// Test 8 -- test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch
// AC-7: Parsing a novel sensor name ("hypothetical_sensor") succeeds without
//       parse-time error. Also verifies spec_parser.rs contains no hardcoded
//       dispatch match arms for the four built-in sensor names.
//
// Pre-implementation failure mode: if spec_parser.rs has a hardcoded match
// arm that rejects unknown sensors with an error, this test detects it.
// The grep-gate sub-assertion verifies no hardcoded match arms remain.
// ---------------------------------------------------------------------------

/// BC-2.16.012 AC-7 / INV-SPEC-PARSER-OPEN-001/003: An unrecognized sensor
/// name ("hypothetical_sensor") is parsed generically -- no parse-time error.
///
/// Also verifies via grep-gate that spec_parser.rs contains no match arms
/// dispatching on hardcoded sensor names in non-comment production code
/// (per BC-2.16.012 postconditions + TV-BC-2.16.012-001).
///
/// Red Gate failure mode:
/// - If a hardcoded match arm rejects unknown sensors -> parse returns Err.
/// - Grep-gate fails if hardcoded name strings appear in dispatch match-arm context.
///
/// Story: S-PLUGIN-PREREQ-E AC-7 | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-001/003
#[test]
fn test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch() {
    // Minimal valid sensor TOML with a hypothetical (unregistered) sensor name.
    // Must parse without error post-migration (generic path).
    let hypothetical_toml = r#"
sensor_id = "hypothetical_sensor"
name = "Hypothetical Sensor"
auth_type = "api_key"
base_url = "https://api.hypothetical.example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "event_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch_events"
  method = "GET"
  path_template = "/v1/events"
  response_path = "$.data"
  variables_produced = []
"#;

    let spec = SpecLoader::parse(hypothetical_toml)
        .expect("BC-2.16.012 AC-7: parsing 'hypothetical_sensor' must succeed via generic path");

    assert_eq!(
        spec.sensor_id, "hypothetical_sensor",
        "sensor_id must survive parse unchanged"
    );

    // AC-7 also requires that the cross-composition validation path is exercised
    // for novel sensors -- the open dispatch must call validate_cross_composition.
    // This call panics pre-implementation (todo!()) to establish the Red Gate.
    let result = SpecLoader::validate_cross_composition(
        &spec.sensor_id,
        "api_key", // auth_type declared in the hypothetical TOML
        1,         // single credential_ref
        "api_key",
        "api_key",
    );
    assert!(
        result.is_ok(),
        "BC-2.16.012 AC-7: hypothetical_sensor open dispatch must produce a valid spec \
         that passes cross-composition validation; got: {:?}",
        result.err()
    );

    // Grep-gate: verify spec_parser.rs has no hardcoded sensor name dispatch arms
    // in executable code. Per BC-2.16.012 postconditions + INV-SPEC-PARSER-OPEN-001.
    let spec_parser_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("spec_parser.rs");

    let content =
        std::fs::read_to_string(&spec_parser_path).expect("spec_parser.rs must be readable");

    let hardcoded_names = [
        "\"crowdstrike\"",
        "\"cyberint\"",
        "\"claroty\"",
        "\"armis\"",
    ];
    let mut dispatch_hits: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Skip comment lines.
        if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }
        // Flag lines that look like dispatch match arms (contain both a sensor name
        // string literal AND a match/=> construct indicating dispatch context).
        for &name in &hardcoded_names {
            if line.contains(name) && (line.contains("=>") || line.contains("match")) {
                dispatch_hits.push(line.trim().to_string());
            }
        }
    }

    assert!(
        dispatch_hits.is_empty(),
        "BC-2.16.012 AC-7 / INV-SPEC-PARSER-OPEN-001: spec_parser.rs still contains \
         hardcoded sensor name match arms. Task 6 must replace these with open dispatch.\n\
         Lines with hardcoded dispatch:\n{}",
        dispatch_hits.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Tests 9-12 -- Behavioral equivalence for the four built-in sensor specs.
// AC-8: Parsing each built-in sensor TOML via the post-migration SpecLoader
//       must produce a valid SensorSpec AND survive validate_cross_composition.
//
// Pre-implementation failure mode: validate_cross_composition is todo!() --
// panics when called. Tests 9-12 fail RED until Task 6b is implemented.
// ---------------------------------------------------------------------------

/// BC-2.16.012 AC-8 / INV-SPEC-PARSER-OPEN-002: Parsing the CrowdStrike
/// canonical sensor spec TOML and validating cross-composition rules must succeed.
///
/// Red Gate failure mode: validate_cross_composition is todo!() -- panics.
/// Post-implementation: parse + validate passes for a valid CrowdStrike spec.
///
/// Story: S-PLUGIN-PREREQ-E AC-8 | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-002
#[test]
fn test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike() {
    let spec = SpecLoader::parse(CROWDSTRIKE_FIXTURE_TOML)
        .expect("crowdstrike canonical fixture must parse without error");

    assert_eq!(
        spec.sensor_id, "crowdstrike",
        "sensor_id must be 'crowdstrike'"
    );
    assert!(
        !spec.tables.is_empty(),
        "crowdstrike spec must have at least one table"
    );

    // Panics pre-implementation on todo!() in validate_cross_composition.
    // Post-implementation: must return Ok(()) for a valid spec (no cross-composition violation).
    //
    // Per BC-2.06.003 / ADR-032: oauth2_client_credentials requires exactly 2
    // credential_refs (client_id + client_secret).
    let result = SpecLoader::validate_cross_composition(
        &spec.sensor_id,
        "oauth2_client_credentials", // auth_type in set -- Rule A passes
        2, // 2 credential_refs (client_id + client_secret) -- Rule B passes (oauth2 allows 2)
        "oauth2_client_credentials", // expected shape
        "oauth2_client_credentials", // actual shape matches -- Rule C passes
    );
    assert!(
        result.is_ok(),
        "BC-2.16.012 AC-8: valid crowdstrike spec must pass cross-composition validation; \
         got: {:?}",
        result.err()
    );
}

/// BC-2.16.012 AC-8 / INV-SPEC-PARSER-OPEN-002: Parsing the Cyberint
/// canonical sensor spec TOML and validating cross-composition rules must succeed.
///
/// Red Gate failure mode: validate_cross_composition is todo!() -- panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-8 | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-002
#[test]
fn test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint() {
    let spec = SpecLoader::parse(CYBERINT_FIXTURE_TOML)
        .expect("cyberint canonical fixture must parse without error");

    assert_eq!(spec.sensor_id, "cyberint", "sensor_id must be 'cyberint'");
    assert!(
        !spec.tables.is_empty(),
        "cyberint spec must have at least one table"
    );

    // Panics pre-implementation on todo!() in validate_cross_composition.
    let result = SpecLoader::validate_cross_composition(
        &spec.sensor_id,
        "bearer_static", // auth_type in set -- Rule A passes
        1,
        "bearer_token",
        "bearer_token",
    );
    assert!(
        result.is_ok(),
        "BC-2.16.012 AC-8: valid cyberint spec must pass cross-composition validation; \
         got: {:?}",
        result.err()
    );
}

/// BC-2.16.012 AC-8 / INV-SPEC-PARSER-OPEN-002: Parsing the Claroty
/// canonical sensor spec TOML and validating cross-composition rules must succeed.
///
/// Red Gate failure mode: validate_cross_composition is todo!() -- panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-8 | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-002
#[test]
fn test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty() {
    let spec = SpecLoader::parse(CLAROTY_FIXTURE_TOML)
        .expect("claroty canonical fixture must parse without error");

    assert_eq!(spec.sensor_id, "claroty", "sensor_id must be 'claroty'");
    assert!(
        !spec.tables.is_empty(),
        "claroty spec must have at least one table"
    );

    // Panics pre-implementation on todo!() in validate_cross_composition.
    let result = SpecLoader::validate_cross_composition(
        &spec.sensor_id,
        "cookie_roundtrip", // auth_type in set -- Rule A passes
        1,
        "username+password",
        "username+password",
    );
    assert!(
        result.is_ok(),
        "BC-2.16.012 AC-8: valid claroty spec must pass cross-composition validation; \
         got: {:?}",
        result.err()
    );
}

/// BC-2.16.012 AC-8 / INV-SPEC-PARSER-OPEN-002: Parsing the Armis
/// canonical sensor spec TOML and validating cross-composition rules must succeed.
///
/// Red Gate failure mode: validate_cross_composition is todo!() -- panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-8 | BC: BC-2.16.012 | INV-SPEC-PARSER-OPEN-002
#[test]
fn test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis() {
    let spec = SpecLoader::parse(ARMIS_FIXTURE_TOML)
        .expect("armis canonical fixture must parse without error");

    assert_eq!(spec.sensor_id, "armis", "sensor_id must be 'armis'");
    assert!(
        !spec.tables.is_empty(),
        "armis spec must have at least one table"
    );

    // Panics pre-implementation on todo!() in validate_cross_composition.
    let result = SpecLoader::validate_cross_composition(
        &spec.sensor_id,
        "api_key", // auth_type in set -- Rule A passes
        1,
        "api_key",
        "api_key",
    );
    assert!(
        result.is_ok(),
        "BC-2.16.012 AC-8: valid armis spec must pass cross-composition validation; \
         got: {:?}",
        result.err()
    );
}
