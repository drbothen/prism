#![allow(non_snake_case)]
//! Red Gate tests for S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Per-Org Sensor Endpoint
//! Overlay Loading (ADR-029).
//!
//! ALL tests in this file call stubbed `OverlayLoader` functions (all bodies are
//! `todo!()`) and MUST FAIL until the implementer fills in the real logic.
//! This is the Red Gate — DO NOT pre-implement any business logic in the stubs.
//!
//! Test names correspond 1:1 to the AC Red Gate test names in the story spec
//! §Acceptance Criteria (lines 183–341) and the story §Tasks list (lines 356–364).
//!
//! BC trace:
//!   AC-001 + AC-006 → BC-2.06.012 (overlay discovery + merge + backcompat)
//!   AC-002 (3 tests) → BC-2.06.013 (scalar-only enforcement)
//!   AC-003           → BC-2.06.014 (fanout identity resolution)
//!   AC-004           → BC-2.06.015 (OrgRegistry cross-validation)
//!   AC-005           → BC-2.06.016 (error taxonomy + SpecErrorCode variants)
//!   AC-007           → BC-2.06.012 §Canonical Test Vectors (two-org same-sensor)
//!
//! Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
//! BCs: BC-2.06.012 through BC-2.06.016

use std::collections::HashMap;

use prism_core::{OrgId, OrgRegistry, OrgSlug, PrismError, SpecErrorCode};
use prism_spec_engine::overlay::OverlayLoader;
use prism_spec_engine::spec_parser::SensorSpec;

// ---------------------------------------------------------------------------
// Test helpers — shared TYPE spec TOML and builder helpers
// ---------------------------------------------------------------------------

/// Canonical Armis TYPE spec TOML used as the base across all overlay tests.
///
/// Provides a minimal but valid TYPE spec that the overlay can extend.
/// Tables schema is intentionally non-trivial (2 columns) so the invariant
/// INV-OVL-001 (schema must be identical in merged result) is observable.
const ARMIS_TYPE_SPEC_TOML: &str = r#"
sensor_id = "armis"
name = "Armis Centrix (TYPE spec)"
auth_type = "bearer_static"
base_url = "https://armis.default.example.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory_info"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.columns]]
  name = "ip_address"
  column_type = "string"

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/devices"
  response_path = "$.data"
  variables_produced = []
"#;

/// Parse the canonical Armis TYPE spec into a `SensorSpec`.
///
/// Panics if the TOML fails to parse — this is test setup, not production code.
fn armis_type_spec() -> SensorSpec {
    use prism_spec_engine::spec_parser::SpecLoader;
    let result = SpecLoader::parse(ARMIS_TYPE_SPEC_TOML);
    result.expect("Armis TYPE spec helper TOML must parse without errors")
}

/// Build a `HashMap<String, SensorSpec>` containing only the Armis TYPE spec.
fn type_specs_with_armis() -> HashMap<String, SensorSpec> {
    let mut map = HashMap::new();
    map.insert("armis".to_string(), armis_type_spec());
    map
}

/// Build a `HashMap<String, SensorSpec>` with no entries (for E-SPEC-019 tests).
fn empty_type_specs() -> HashMap<String, SensorSpec> {
    HashMap::new()
}

/// Build an `OrgRegistry` with only `acme` registered.
fn registry_with_acme() -> OrgRegistry {
    let registry = OrgRegistry::new();
    let acme_id = OrgId::new();
    registry
        .register(OrgSlug::new("acme"), acme_id)
        .expect("registering acme must succeed");
    registry
}

/// Build an `OrgRegistry` with `acme` and `contoso` registered.
fn registry_with_acme_and_contoso() -> OrgRegistry {
    let registry = OrgRegistry::new();
    registry
        .register(OrgSlug::new("acme"), OrgId::new())
        .expect("registering acme must succeed");
    registry
        .register(OrgSlug::new("contoso"), OrgId::new())
        .expect("registering contoso must succeed");
    registry
}

/// Write `content` to `<dir>/<path>`, creating parent directories as needed.
fn write_file(dir: &std::path::Path, rel_path: &str, content: &str) {
    let target = dir.join(rel_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(&target, content).expect("write file");
}

// ---------------------------------------------------------------------------
// AC-001: Overlay file discovery and scalar merge
// BC-2.06.012 postconditions 1–7
// ---------------------------------------------------------------------------

/// Red Gate test for AC-001 (BC-2.06.012).
///
/// Scenario: customers/acme/armis.sensor.toml overlay with
/// `base_url = "https://armis.acme-corp.io"`.
///
/// Asserts:
/// - `OverlayLoader::load_overlays` returns a non-empty result map with a
///   `ResolvedSensorSpec` keyed at `(OrgSlug::new("acme"), "armis")`.
/// - The resolved spec's `base_url` is `"https://armis.acme-corp.io"` (from overlay).
/// - The resolved spec's tables are identical to the TYPE spec (INV-OVL-001).
/// - `provenance.base_url_from_overlay` is `true` (BC-2.06.012 postcondition — merged
///   spec carries provenance metadata for which fields came from overlay).
/// - No errors in `result.errors`.
///
/// RED GATE: Panics with "not yet implemented" because `OverlayLoader::load_overlays`
/// body is `todo!()`.
#[test]
fn test_BC_2_06_012_overlay_discovered_and_merged() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    write_file(
        dir.path(),
        "customers/acme/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
"#,
    );

    let type_specs = type_specs_with_armis();
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // No validation errors expected for a valid overlay.
    assert!(
        result.errors.is_empty(),
        "Valid overlay must produce no errors; got: {:?}",
        result.errors
    );

    // The resolved map must contain an entry for (acme, armis).
    let key = (OrgSlug::new("acme"), "armis".to_string());
    assert!(
        result.resolved.contains_key(&key),
        "Resolved map must contain key (acme, armis); keys present: {:?}",
        result.resolved.keys().collect::<Vec<_>>()
    );

    let resolved = &result.resolved[&key];

    // BC-2.06.012 postcondition: base_url comes from overlay.
    assert_eq!(
        resolved.spec.base_url, "https://armis.acme-corp.io",
        "ResolvedSensorSpec base_url must be from overlay, not TYPE spec default"
    );

    // INV-OVL-001: tables schema must be identical to TYPE spec.
    let type_spec = armis_type_spec();
    assert_eq!(
        resolved.spec.tables.len(),
        type_spec.tables.len(),
        "Resolved spec must have same number of tables as TYPE spec (INV-OVL-001)"
    );
    assert_eq!(
        resolved.spec.tables[0].table_name, type_spec.tables[0].table_name,
        "Table name must be unchanged from TYPE spec (INV-OVL-001)"
    );

    // INV-OVL-002: auth_type is immutable.
    assert_eq!(
        resolved.spec.auth_type, type_spec.auth_type,
        "auth_type must be unchanged from TYPE spec (INV-OVL-002)"
    );

    // BC-2.06.012: provenance metadata present.
    assert!(
        resolved.provenance.base_url_from_overlay,
        "provenance.base_url_from_overlay must be true when overlay sets base_url"
    );

    // instance_id must be "armis@acme" per BC-2.06.012 postcondition.
    assert_eq!(
        resolved.instance_id, "armis@acme",
        "instance_id must be '{{sensor_id}}@{{org_slug}}' — expected 'armis@acme'"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (part 1): [[tables]] in overlay → E-SPEC-021
// BC-2.06.013 §Failure path — [[tables]] present
// ---------------------------------------------------------------------------

/// Red Gate test for AC-002 / BC-2.06.013 — [[tables]] forbidden path.
///
/// Scenario: overlay file contains a `[[tables]]` block (schema override attempt).
///
/// Asserts:
/// - `OverlayLoader::validate_overlay_toml` returns `Err(Vec<PrismError>)` containing
///   at least one error with `SpecErrorCode::ESpec021`.
/// - `result.errors` from `load_overlays` (alternate driver) would contain `ESpec021`.
///
/// RED GATE: Panics with "not yet implemented" because both
/// `OverlayLoader::validate_overlay_toml` and `OverlayLoader::load_overlays` are
/// `todo!()`.
#[test]
fn test_BC_2_06_013_tables_in_overlay_rejects_with_e_spec_021() {
    let overlay_toml = r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"

[[tables]]
table_name = "forbidden_override"
ocsf_class = "device_inventory_info"

  [[tables.columns]]
  name = "id"
  column_type = "string"
"#;

    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::validate_overlay_toml(
        overlay_toml,
        "customers/acme/armis.sensor.toml",
        "armis",
        "acme",
        &type_specs,
    );

    assert!(
        result.is_err(),
        "Overlay containing [[tables]] must be rejected (BC-2.06.013 INV-SCALAR-004)"
    );

    let errors = result.unwrap_err();
    let has_e_spec_021 = errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec021)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_021,
        "[[tables]] in overlay must produce E-SPEC-021 error (BC-2.06.016 §Error Catalog); \
         actual errors: {:?}",
        errors
    );
}

// ---------------------------------------------------------------------------
// AC-002 (part 2): unrecognized field in overlay → E-SPEC-023
// BC-2.06.013 §Failure path — unrecognized field
// ---------------------------------------------------------------------------

/// Red Gate test for AC-002 / BC-2.06.013 — unrecognized scalar field path.
///
/// Scenario: overlay file contains `auth_type = "bearer_static"` which is a
/// forbidden field (schema immutability — INV-OVL-002; any field not in the
/// allowed set triggers E-SPEC-023 per BC-2.06.013).
///
/// Asserts:
/// - `OverlayLoader::validate_overlay_toml` returns `Err` containing `ESpec023`.
/// - Error message includes the field name `"auth_type"`.
///
/// RED GATE: Panics with "not yet implemented".
#[test]
fn test_BC_2_06_013_unrecognized_field_rejects_with_e_spec_023() {
    let overlay_toml = r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
auth_type   = "oauth2_client_credentials"
"#;

    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::validate_overlay_toml(
        overlay_toml,
        "customers/acme/armis.sensor.toml",
        "armis",
        "acme",
        &type_specs,
    );

    assert!(
        result.is_err(),
        "Overlay with forbidden field 'auth_type' must be rejected (BC-2.06.013)"
    );

    let errors = result.unwrap_err();
    let has_e_spec_023 = errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec023)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_023,
        "Unrecognized field 'auth_type' in overlay must produce E-SPEC-023 \
         (BC-2.06.016 §Error Catalog); actual errors: {:?}",
        errors
    );

    // BC-2.06.016 message template: must include the offending field name.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
    let field_named_in_error = error_messages.iter().any(|m| m.contains("auth_type"));
    assert!(
        field_named_in_error,
        "E-SPEC-023 error message must include the field name 'auth_type'; \
         messages: {:?}",
        error_messages
    );
}

// ---------------------------------------------------------------------------
// AC-002 (part 3): wrong instance_id → E-SPEC-020
// BC-2.06.013 §Failure path — instance_id convention mismatch
// ---------------------------------------------------------------------------

/// Red Gate test for AC-002 / BC-2.06.013 — instance_id convention mismatch path.
///
/// Scenario: overlay file in `customers/acme/armis.sensor.toml` has
/// `instance_id = "armis@wrongorg"` — does not match expected `"armis@acme"`.
///
/// Asserts:
/// - `OverlayLoader::validate_overlay_toml` returns `Err` containing `ESpec020`.
/// - Error message includes the actual value and the expected value (BC-2.06.016).
///
/// RED GATE: Panics with "not yet implemented".
#[test]
fn test_BC_2_06_013_wrong_instance_id_rejects_with_e_spec_020() {
    let overlay_toml = r#"
extends     = "armis"
instance_id = "armis@wrongorg"
base_url    = "https://armis.acme-corp.io"
"#;

    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::validate_overlay_toml(
        overlay_toml,
        "customers/acme/armis.sensor.toml",
        "armis",
        "acme",
        &type_specs,
    );

    assert!(
        result.is_err(),
        "Overlay with instance_id 'armis@wrongorg' in customers/acme/ must be rejected \
         (BC-2.06.013 — expected 'armis@acme')"
    );

    let errors = result.unwrap_err();
    let has_e_spec_020 = errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec020)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_020,
        "instance_id mismatch must produce E-SPEC-020 (BC-2.06.016 §Error Catalog); \
         actual errors: {:?}",
        errors
    );

    // BC-2.06.016 E-SPEC-020 message template: must reference the expected value.
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
    let expected_value_present = error_messages.iter().any(|m| m.contains("armis@acme"));
    assert!(
        expected_value_present,
        "E-SPEC-020 error message must include the expected instance_id 'armis@acme'; \
         messages: {:?}",
        error_messages
    );
}

// ---------------------------------------------------------------------------
// AC-003: Instance identity resolution at fanout uses overlay base_url
// BC-2.06.014 postcondition Case A (overlay present) + Case B (no overlay)
// ---------------------------------------------------------------------------

/// Red Gate test for AC-003 / BC-2.06.014.
///
/// Scenario: `customers/acme/armis.sensor.toml` overlay sets
/// `base_url = "https://armis.acme-corp.io"`. After `OverlayLoader::load_overlays`
/// runs, the `ResolvedSensorSpec` at key `(acme, armis)` must carry the overlay
/// base_url (Case A — per-org endpoint routing).
///
/// Case B is tested via a separate overlay that sets no base_url (only extends +
/// instance_id). The merged result must fall back to the TYPE spec base_url.
///
/// Note: `SensorInstanceOverlay` is `#[non_exhaustive]` and cannot be constructed
/// directly in external test crates (the compile-fail gate at
/// `tests/external/non-exhaustive-violation/` enforces this). The production path
/// is `OverlayLoader::load_overlays` → `validate_overlay_toml` → `merge_overlay_onto_type_spec`,
/// which is the path under test here. The `merge_overlay_onto_type_spec` unit is
/// exercised indirectly through the full load path per SID-1 requirements.
///
/// Asserts (Case A):
/// - `result.resolved[(acme, armis)].spec.base_url == "https://armis.acme-corp.io"`.
/// - `result.resolved[(acme, armis)].provenance.base_url_from_overlay == true`.
/// - `tables` from TYPE spec (INV-FANOUT-003).
///
/// Asserts (Case B):
/// - `result.resolved[(acme, armis)].spec.base_url == "https://armis.default.example.com"`.
/// - `result.resolved[(acme, armis)].provenance.base_url_from_overlay == false`.
///
/// RED GATE: Panics with "not yet implemented" because `OverlayLoader::load_overlays`
/// and the merge path are `todo!()`.
#[test]
fn test_BC_2_06_014_resolved_spec_overlays_base_url() {
    let type_spec = armis_type_spec();

    // --- Case A: overlay sets base_url --- //
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        let customers_dir = dir.path().join("customers");

        write_file(
            dir.path(),
            "customers/acme/armis.sensor.toml",
            r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
"#,
        );

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();
        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "Case A: valid overlay must produce no errors; got: {:?}",
            result.errors
        );

        let key = (OrgSlug::new("acme"), "armis".to_string());
        assert!(
            result.resolved.contains_key(&key),
            "Case A: resolved map must contain (acme, armis)"
        );

        let resolved_a = &result.resolved[&key];

        // BC-2.06.014 Case A: overlay base_url used at HTTP dispatch.
        assert_eq!(
            resolved_a.spec.base_url, "https://armis.acme-corp.io",
            "Case A: resolved spec must use overlay base_url for HTTP dispatch (BC-2.06.014)"
        );

        // INV-FANOUT-003: tables schema from TYPE spec unchanged.
        assert_eq!(
            resolved_a.spec.tables.len(),
            type_spec.tables.len(),
            "Tables must be from TYPE spec, not affected by overlay (INV-FANOUT-003)"
        );

        // Provenance: base_url came from overlay.
        assert!(
            resolved_a.provenance.base_url_from_overlay,
            "provenance.base_url_from_overlay must be true for Case A"
        );
    }

    // --- Case B: overlay has no base_url (SaaS sensor / minimal overlay) --- //
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        let customers_dir = dir.path().join("customers");

        // Minimal overlay — only required fields, no base_url.
        // BC-2.06.014 Case B: no overlay base_url → TYPE spec base_url used.
        write_file(
            dir.path(),
            "customers/acme/armis.sensor.toml",
            r#"
extends     = "armis"
instance_id = "armis@acme"
"#,
        );

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();
        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "Case B: minimal overlay must produce no errors; got: {:?}",
            result.errors
        );

        let key = (OrgSlug::new("acme"), "armis".to_string());
        assert!(
            result.resolved.contains_key(&key),
            "Case B: minimal overlay must still produce a ResolvedSensorSpec entry"
        );

        let resolved_b = &result.resolved[&key];

        // BC-2.06.014 Case B: TYPE spec base_url used when overlay sets no base_url.
        assert_eq!(
            resolved_b.spec.base_url, "https://armis.default.example.com",
            "Case B: resolved spec must use TYPE spec base_url when overlay base_url absent"
        );

        // Provenance: base_url NOT from overlay in Case B.
        assert!(
            !resolved_b.provenance.base_url_from_overlay,
            "provenance.base_url_from_overlay must be false for Case B"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004: Unknown customers/<slug>/ directory aborts boot with E-SPEC-022
// BC-2.06.015 §Failure path — unregistered slug directory
// ---------------------------------------------------------------------------

/// Red Gate test for AC-004 / BC-2.06.015.
///
/// Scenario: `customers/unknown-org/armis.sensor.toml` exists on disk, but
/// `OrgRegistry` contains only `"acme"`. The `OverlayLoader` must detect that
/// `"unknown-org"` is not registered and emit `E-SPEC-022`.
///
/// Asserts:
/// - `OverlayLoader::load_overlays` returns `result.errors` containing at least
///   one error with `SpecErrorCode::ESpec022`.
/// - The error message includes `"unknown-org"` (BC-2.06.016 canonical template).
/// - The `result.resolved` map is empty (no overlays loaded when any slug fails).
///
/// RED GATE: Panics with "not yet implemented".
#[test]
fn test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    write_file(
        dir.path(),
        "customers/unknown-org/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@unknown-org"
base_url    = "https://armis.unknown.io"
"#,
    );

    let type_specs = type_specs_with_armis();
    // Registry has only "acme" — "unknown-org" is NOT registered.
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // BC-2.06.015 failure path: must produce E-SPEC-022 for unregistered slug.
    assert!(
        !result.errors.is_empty(),
        "Unregistered org slug directory must produce at least one error (BC-2.06.015)"
    );

    let has_e_spec_022 = result.errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec022)
        } else {
            false
        }
    });

    assert!(
        has_e_spec_022,
        "Unregistered slug 'unknown-org' must produce E-SPEC-022 (BC-2.06.016 §Error Catalog); \
         actual errors: {:?}",
        result.errors
    );

    // BC-2.06.016 E-SPEC-022 message template: must include the unrecognized slug.
    let error_messages: Vec<String> = result.errors.iter().map(|e| format!("{}", e)).collect();
    let slug_in_message = error_messages.iter().any(|m| m.contains("unknown-org"));
    assert!(
        slug_in_message,
        "E-SPEC-022 error message must include the unrecognized slug 'unknown-org'; \
         messages: {:?}",
        error_messages
    );

    // INV-SCALAR-003: a single invalid overlay fails the entire walk (no partial success).
    assert!(
        result.resolved.is_empty(),
        "When an unregistered slug is found, the resolved map must be empty (INV-SCALAR-003); \
         got: {:?} entries",
        result.resolved.len()
    );
}

// ---------------------------------------------------------------------------
// AC-005: Error taxonomy — SpecErrorCode variants ESpec019..ESpec023 exist
//         and canonical message templates match BC-2.06.016
// BC-2.06.016 §Error Catalog
// ---------------------------------------------------------------------------

/// Red Gate test for AC-005 / BC-2.06.016.
///
/// Triggers all five overlay error conditions in turn and asserts:
/// 1. `SpecErrorCode::ESpec019` — overlay extends unknown TYPE spec.
/// 2. `SpecErrorCode::ESpec020` — instance_id mismatch.
/// 3. `SpecErrorCode::ESpec021` — [[tables]] in overlay.
/// 4. `SpecErrorCode::ESpec022` — unregistered org slug directory.
/// 5. `SpecErrorCode::ESpec023` — unrecognized field in overlay.
///
/// For each, asserts that the error code matches the canonical BC-2.06.016
/// §Error Catalog code (INV-ERR-001: all are FATAL/broken/validation).
///
/// RED GATE: Panics with "not yet implemented" for `validate_overlay_toml`
/// and `load_overlays` stubs.
#[test]
fn test_BC_2_06_016_error_messages_match_canonical_templates() {
    // ---------- E-SPEC-019: extends references unknown TYPE spec ----------
    {
        let overlay_unknown_extends = r#"
extends     = "nonexistent_sensor"
instance_id = "nonexistent_sensor@acme"
base_url    = "https://example.com"
"#;
        // Empty type_specs → "nonexistent_sensor" not found.
        let result_019 = OverlayLoader::validate_overlay_toml(
            overlay_unknown_extends,
            "customers/acme/nonexistent_sensor.sensor.toml",
            "nonexistent_sensor",
            "acme",
            &empty_type_specs(),
        );
        assert!(
            result_019.is_err(),
            "E-SPEC-019: overlay extending unknown TYPE spec must be rejected"
        );
        let errs_019 = result_019.unwrap_err();
        let has_019 = errs_019.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                matches!(se.code, SpecErrorCode::ESpec019)
            } else {
                false
            }
        });
        assert!(
            has_019,
            "E-SPEC-019 must be emitted when extends references unknown TYPE spec; \
             got: {:?}",
            errs_019
        );
        // BC-2.06.016: message includes the extends value.
        let msg_contains_extends = errs_019
            .iter()
            .any(|e| format!("{}", e).contains("nonexistent_sensor"));
        assert!(
            msg_contains_extends,
            "E-SPEC-019 message must include the extends value 'nonexistent_sensor'"
        );
    }

    // ---------- E-SPEC-020: instance_id mismatch ----------
    {
        let overlay_bad_id = r#"
extends     = "armis"
instance_id = "armis@wrongorg"
base_url    = "https://armis.example.com"
"#;
        let result_020 = OverlayLoader::validate_overlay_toml(
            overlay_bad_id,
            "customers/acme/armis.sensor.toml",
            "armis",
            "acme",
            &type_specs_with_armis(),
        );
        assert!(
            result_020.is_err(),
            "E-SPEC-020: instance_id mismatch must be rejected"
        );
        let errs_020 = result_020.unwrap_err();
        let has_020 = errs_020.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                matches!(se.code, SpecErrorCode::ESpec020)
            } else {
                false
            }
        });
        assert!(
            has_020,
            "E-SPEC-020 must be emitted for instance_id mismatch; got: {:?}",
            errs_020
        );
    }

    // ---------- E-SPEC-021: [[tables]] in overlay ----------
    {
        let overlay_with_tables = r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme.io"

[[tables]]
table_name = "forbidden"
ocsf_class = "device_inventory_info"
"#;
        let result_021 = OverlayLoader::validate_overlay_toml(
            overlay_with_tables,
            "customers/acme/armis.sensor.toml",
            "armis",
            "acme",
            &type_specs_with_armis(),
        );
        assert!(
            result_021.is_err(),
            "E-SPEC-021: [[tables]] in overlay must be rejected"
        );
        let errs_021 = result_021.unwrap_err();
        let has_021 = errs_021.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                matches!(se.code, SpecErrorCode::ESpec021)
            } else {
                false
            }
        });
        assert!(
            has_021,
            "E-SPEC-021 must be emitted when [[tables]] present in overlay; got: {:?}",
            errs_021
        );
    }

    // ---------- E-SPEC-022: unregistered org slug directory ----------
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let customers_dir = dir.path().join("customers");

        write_file(
            dir.path(),
            "customers/stale-corp/armis.sensor.toml",
            r#"
extends     = "armis"
instance_id = "armis@stale-corp"
base_url    = "https://armis.stale.io"
"#,
        );

        // Registry has only "acme"; "stale-corp" is unregistered.
        let result_022 = OverlayLoader::load_overlays(
            &customers_dir,
            &type_specs_with_armis(),
            &registry_with_acme(),
        );

        let has_022 = result_022.errors.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                matches!(se.code, SpecErrorCode::ESpec022)
            } else {
                false
            }
        });
        assert!(
            has_022,
            "E-SPEC-022 must be emitted for unregistered slug directory 'stale-corp'; \
             errors: {:?}",
            result_022.errors
        );
        // BC-2.06.016 E-SPEC-022 message includes the slug.
        let msg_has_slug = result_022
            .errors
            .iter()
            .any(|e| format!("{}", e).contains("stale-corp"));
        assert!(
            msg_has_slug,
            "E-SPEC-022 message must include the unrecognized slug 'stale-corp'"
        );
    }

    // ---------- E-SPEC-023: unrecognized field in overlay ----------
    {
        let overlay_unknown_field = r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme.io"
secret_key  = "s3cr3t"
"#;
        let result_023 = OverlayLoader::validate_overlay_toml(
            overlay_unknown_field,
            "customers/acme/armis.sensor.toml",
            "armis",
            "acme",
            &type_specs_with_armis(),
        );
        assert!(
            result_023.is_err(),
            "E-SPEC-023: overlay with unrecognized field 'secret_key' must be rejected"
        );
        let errs_023 = result_023.unwrap_err();
        let has_023 = errs_023.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                matches!(se.code, SpecErrorCode::ESpec023)
            } else {
                false
            }
        });
        assert!(
            has_023,
            "E-SPEC-023 must be emitted for unrecognized field 'secret_key'; got: {:?}",
            errs_023
        );
        // BC-2.06.016 E-SPEC-023 message includes the field name.
        let msg_has_field = errs_023
            .iter()
            .any(|e| format!("{}", e).contains("secret_key"));
        assert!(
            msg_has_field,
            "E-SPEC-023 message must include the unrecognized field name 'secret_key'; \
             messages: {:?}",
            errs_023
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
        );
    }
}

// ---------------------------------------------------------------------------
// AC-006: Backwards compatibility — no customers/ directory → zero overlays
// BC-2.06.012 postcondition: absent customers/ dir → zero entries; boot succeeds
// Edge cases: EC-012-001 (absent dir) + EC-012-002 (.gitkeep only)
// ---------------------------------------------------------------------------

/// Red Gate test for AC-006 / BC-2.06.012 (EC-012-001 + EC-012-002).
///
/// Scenario A (EC-012-001): sensor_specs_dir has no `customers/` subdirectory at all.
/// Scenario B (EC-012-002): `customers/.gitkeep` present but no subdirectories.
///
/// Asserts for both scenarios:
/// - `OverlayLoader::load_overlays` returns `result.resolved` that is empty.
/// - `result.errors` is empty (no errors emitted — boot continues normally).
///
/// RED GATE: Panics with "not yet implemented".
#[test]
fn test_BC_2_06_012_backcompat_no_customers_dir_uses_type_spec_only() {
    // --- Scenario A: customers/ directory entirely absent ---
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        // Intentionally do NOT create a customers/ subdirectory.
        let customers_dir = dir.path().join("customers");

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();

        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "EC-012-001: absent customers/ dir must produce no errors; got: {:?}",
            result.errors
        );
        assert!(
            result.resolved.is_empty(),
            "EC-012-001: absent customers/ dir must produce zero ResolvedSensorSpec entries; \
             got: {}",
            result.resolved.len()
        );
    }

    // --- Scenario B: customers/.gitkeep only (no subdirectories) ---
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        // Create customers/.gitkeep — a plain file, NOT a subdirectory.
        write_file(dir.path(), "customers/.gitkeep", "");
        let customers_dir = dir.path().join("customers");

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();

        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "EC-012-002: customers/.gitkeep only must produce no errors; got: {:?}",
            result.errors
        );
        assert!(
            result.resolved.is_empty(),
            "EC-012-002: customers/.gitkeep only must produce zero ResolvedSensorSpec entries; \
             got: {}",
            result.resolved.len()
        );
    }
}

// ---------------------------------------------------------------------------
// AC-007: Two-org overlays for same sensor produce distinct ResolvedSensorSpec entries
// BC-2.06.012 §Canonical Test Vectors (two-org same-sensor)
// Edge case: EC-012-006 (two orgs, same sensor, no interference)
// ---------------------------------------------------------------------------

/// Red Gate test for AC-007 / BC-2.06.012 §Canonical Test Vectors (two-org).
///
/// Scenario: Both `customers/acme/armis.sensor.toml` and
/// `customers/contoso/armis.sensor.toml` exist. The loader must produce two
/// independent `ResolvedSensorSpec` entries — one per `(org_slug, sensor_id)` pair.
///
/// Asserts:
/// - `result.resolved` contains exactly two entries: `(acme, armis)` and `(contoso, armis)`.
/// - `(acme, armis)` has `base_url = "https://armis.acme-corp.io"`.
/// - `(contoso, armis)` has `base_url = "https://armis.contoso.com"`.
/// - Both entries have identical `[[tables]]` schemas from the TYPE spec (INV-OVL-001).
/// - `result.errors` is empty.
///
/// Uses the fixture files at:
/// - `crates/prism-sensors/specs/customers/acme/armis.sensor.toml`
/// - `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml`
/// (content is also written inline for test isolation via tempdir)
///
/// RED GATE: Panics with "not yet implemented".
#[test]
fn test_S_CONFIG_MULTI_TENANT_OVERRIDE_001_007_two_org_overlays_produce_distinct_resolved_specs() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    // acme overlay — from BC-2.06.012 §Canonical Test Vectors (two-org same-sensor)
    write_file(
        dir.path(),
        "customers/acme/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
"#,
    );

    // contoso overlay — from BC-2.06.012 §Canonical Test Vectors (two-org same-sensor)
    write_file(
        dir.path(),
        "customers/contoso/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@contoso"
base_url    = "https://armis.contoso.com"
"#,
    );

    let type_specs = type_specs_with_armis();
    let registry = registry_with_acme_and_contoso();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // No errors expected for two valid overlays.
    assert!(
        result.errors.is_empty(),
        "Two-org overlay scenario must produce no errors; got: {:?}",
        result.errors
    );

    // Exactly two resolved entries (one per org).
    assert_eq!(
        result.resolved.len(),
        2,
        "Two-org overlay scenario must produce exactly 2 ResolvedSensorSpec entries; \
         got: {}",
        result.resolved.len()
    );

    // acme entry: overlay base_url.
    let acme_key = (OrgSlug::new("acme"), "armis".to_string());
    assert!(
        result.resolved.contains_key(&acme_key),
        "Resolved map must contain (acme, armis) entry"
    );
    let acme_resolved = &result.resolved[&acme_key];
    assert_eq!(
        acme_resolved.spec.base_url, "https://armis.acme-corp.io",
        "(acme, armis) resolved spec must use acme overlay base_url"
    );
    assert_eq!(
        acme_resolved.instance_id, "armis@acme",
        "(acme, armis) instance_id must be 'armis@acme'"
    );

    // contoso entry: overlay base_url (different from acme — EC-012-006 no interference).
    let contoso_key = (OrgSlug::new("contoso"), "armis".to_string());
    assert!(
        result.resolved.contains_key(&contoso_key),
        "Resolved map must contain (contoso, armis) entry"
    );
    let contoso_resolved = &result.resolved[&contoso_key];
    assert_eq!(
        contoso_resolved.spec.base_url, "https://armis.contoso.com",
        "(contoso, armis) resolved spec must use contoso overlay base_url"
    );
    assert_eq!(
        contoso_resolved.instance_id, "armis@contoso",
        "(contoso, armis) instance_id must be 'armis@contoso'"
    );

    // INV-OVL-001: schemas identical from TYPE spec for both orgs.
    let type_spec = armis_type_spec();
    assert_eq!(
        acme_resolved.spec.tables.len(),
        type_spec.tables.len(),
        "acme resolved spec tables must be identical to TYPE spec (INV-OVL-001)"
    );
    assert_eq!(
        contoso_resolved.spec.tables.len(),
        type_spec.tables.len(),
        "contoso resolved spec tables must be identical to TYPE spec (INV-OVL-001)"
    );
    assert_eq!(
        acme_resolved.spec.tables[0].table_name, contoso_resolved.spec.tables[0].table_name,
        "Both orgs must see the same table schema from TYPE spec (INV-OVL-001)"
    );

    // INV-FANOUT-004: resolving one org does not affect the other.
    assert_ne!(
        acme_resolved.spec.base_url, contoso_resolved.spec.base_url,
        "acme and contoso resolved base_urls must be independent (INV-FANOUT-004)"
    );
}
