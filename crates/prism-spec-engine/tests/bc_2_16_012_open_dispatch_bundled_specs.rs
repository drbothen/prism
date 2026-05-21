#![allow(non_snake_case)]
//! RG-08 / AC-012: BC-2.16.012 — PluginRegistry Open Dispatch
//!
//! Tests that the 4 bundled specs flow through the open dispatch path
//! (INV-SPEC-PARSER-OPEN-001) and that spec_parser.rs contains no hardcoded
//! sensor name match arms in dispatch-context code.
//!
//! Red Gate discipline (BC-5.38.001): The grep gate test PASSES in the Red Gate
//! state (because no hardcoded names exist yet), but the spec-idempotency test
//! FAILS until the 4 bundled specs are production-grade and parse consistently.
//!
//! AC coverage: AC-012
//! HS coverage: (open dispatch verification)

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// RG-08 / AC-012: Open dispatch + grep gate
// ---------------------------------------------------------------------------

/// RG-08 / AC-012 / BC-2.16.012 INV-SPEC-PARSER-OPEN-001:
/// test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names
///
/// After this story merges, no hardcoded sensor name match arms for the 4
/// bundled sensors must appear in spec_parser.rs dispatch-context code.
///
/// The grep gate is the authoritative check. If this test fails, the implementer
/// introduced a hardcoded sensor name in the dispatch path.
///
/// RED GATE NOTE: This test is expected to PASS in the Red Gate state because
/// no implementation has been added yet. It remains as a regression gate that
/// MUST NOT be broken during implementation.
#[test]
fn test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names() {
    // Grep spec_parser.rs for hardcoded sensor names in dispatch-context code.
    // This is the canonical grep gate from AC-012.
    let spec_parser_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/spec_parser.rs"),
    )
    .expect("spec_parser.rs must be readable");

    // These sensor names must NOT appear as string literals in dispatch-context code.
    // Allowed: in TOML content parsed at runtime, not as hardcoded match arms.
    //
    // The implementer must not add patterns like: match sensor_id { "crowdstrike" => ... }
    // AC-012 specifies: grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"'
    // in dispatch-context code returns ZERO matches.
    //
    // The concrete assertion is below: check for match-arm syntax.

    // Count occurrences of hardcoded sensor name literals in the dispatch path.
    // The dispatch path is identified by context (match arms, if-else chains with sensor_id).
    // We use a simple heuristic: sensor_id literal + "=>" pattern.
    let dispatch_sensor_names = ["crowdstrike", "cyberint", "claroty", "armis"];
    for sensor_name in &dispatch_sensor_names {
        let match_arm_pattern = format!("\"{}\" =>", sensor_name);
        assert!(
            !spec_parser_src.contains(&match_arm_pattern),
            "Found hardcoded sensor name match arm '{}' in spec_parser.rs dispatch code. \
             BC-2.16.012 INV-SPEC-PARSER-OPEN-001 requires open dispatch only. \
             Remove the hardcoded match arm and use registry lookup instead.",
            match_arm_pattern
        );
    }
}

/// RG-08 sub-assertion / AC-012:
/// Parsing crowdstrike.sensor.toml twice produces byte-identical SensorSpec structs
/// with a NON-EMPTY table set. Behavioral equivalence is preserved across multiple
/// parse calls, and the spec must be production-grade (not a skeleton stub).
///
/// RED GATE: Fails until the crowdstrike.sensor.toml spec is production-grade
/// (Tasks 3-5) and SpecLoader::parse() is deterministic for it.
///
/// STRENGTHENED (Part B): Added N > 0 precondition to ensure skeleton stubs
/// (tables = []) cause this test to fail — idempotency over an empty table set
/// is trivially true but meaningless as a Red Gate check.
#[test]
fn test_BC_2_16_012_spec_parse_is_idempotent_for_crowdstrike() {
    let content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");

    let spec1 =
        SpecLoader::parse(&content).expect("crowdstrike.sensor.toml must parse on first call");
    let spec2 =
        SpecLoader::parse(&content).expect("crowdstrike.sensor.toml must parse on second call");

    // STRENGTHENED: Idempotency assertion is meaningful ONLY when the spec has
    // production-grade content (N > 0 tables). Skeleton stubs (tables = []) produce
    // N = 0, which fails this precondition and correctly blocks the Red Gate.
    //
    // RED GATE: Fails here until implementer Task 3 adds [[tables]] to crowdstrike.sensor.toml.
    assert!(
        spec1.tables.len() > 0,
        "CrowdStrike spec must have > 0 tables for idempotency to be a meaningful Red Gate check. \
         Skeleton stub tables = [] produces 0 tables — implementer Task 3 must fill in \
         the 3 production tables (detections, devices, incidents) per AC-001 \
         (BC-2.16.012 INV-SPEC-PARSER-OPEN-001 idempotency gate)."
    );

    // Behavioral equivalence: same sensor_id, auth_type, tables, version.
    assert_eq!(
        spec1.sensor_id, spec2.sensor_id,
        "sensor_id must be identical across two parse calls"
    );
    assert_eq!(
        spec1.auth_type, spec2.auth_type,
        "auth_type must be identical across two parse calls"
    );
    assert_eq!(
        spec1.tables.len(),
        spec2.tables.len(),
        "tables count must be identical across two parse calls"
    );
    assert_eq!(
        spec1.version, spec2.version,
        "version must be identical across two parse calls"
    );

    // Check each table name is preserved.
    for (t1, t2) in spec1.tables.iter().zip(spec2.tables.iter()) {
        assert_eq!(
            t1.table_name, t2.table_name,
            "table_name '{}' must be identical across two parse calls",
            t1.table_name
        );
        assert_eq!(
            t1.steps.len(),
            t2.steps.len(),
            "steps count for table '{}' must be identical across two parse calls",
            t1.table_name
        );
    }
}

/// AC-012 regression: The open dispatch path handles all 4 bundled specs.
/// No spec should fail to parse due to a missing dispatch branch.
/// Each spec must also expose >= 1 table after implementer Tasks 3-6 fill it.
///
/// RED GATE: Fails until all 4 specs are production-grade.
///
/// STRENGTHENED (Part B): Added >= 1 table assertion per sensor so skeleton
/// stubs (tables = []) cause this test to fail — parsing a skeleton successfully
/// is not sufficient to verify open dispatch produces usable table registrations.
#[test]
fn test_BC_2_16_012_open_dispatch_handles_all_4_bundled_sensors() {
    let specs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../prism-sensors/specs");
    let specs = [
        ("crowdstrike", specs_dir.join("crowdstrike.sensor.toml")),
        ("claroty", specs_dir.join("claroty.sensor.toml")),
        ("cyberint", specs_dir.join("cyberint.sensor.toml")),
        ("armis", specs_dir.join("armis.sensor.toml")),
    ];

    for (sensor_id, path) in &specs {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));

        // If open dispatch is working, all 4 must parse without dispatch-related errors.
        let result = SpecLoader::parse(&content);
        assert!(
            result.is_ok(),
            "Spec '{}' must parse via open dispatch path (INV-SPEC-PARSER-OPEN-001). \
             Dispatch failure would indicate a hardcoded sensor name check. \
             Error: {:?}",
            sensor_id,
            result.err()
        );

        let spec = result.unwrap();
        assert_eq!(
            spec.sensor_id.as_str(),
            *sensor_id,
            "Parsed sensor_id '{}' must match expected '{}'",
            spec.sensor_id,
            sensor_id
        );

        // STRENGTHENED: Open dispatch is only meaningful when the spec exposes
        // >= 1 table. Skeleton stubs (tables = []) parse successfully but produce
        // zero tables, which means the open dispatch path has nothing to register.
        //
        // RED GATE: Fails here for each sensor until implementer Tasks 3-6
        // add production [[tables]] entries.
        assert!(
            !spec.tables.is_empty(),
            "Sensor '{}' must expose >= 1 table via open dispatch path (INV-SPEC-PARSER-OPEN-001). \
             Skeleton stub tables = [] produces 0 tables — implementer Tasks 3-6 must fill in \
             production-grade table definitions so the dispatch path has table registrations \
             to forward (BC-2.16.012 AC-012).",
            sensor_id
        );
    }
}
