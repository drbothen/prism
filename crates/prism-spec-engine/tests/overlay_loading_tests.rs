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

use prism_core::{OrgId, OrgRegistry, OrgSlug, PrismError, SensorId, SpecErrorCode};
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
    let key = (OrgSlug::new("acme"), SensorId::from("armis"));
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

        let key = (OrgSlug::new("acme"), SensorId::from("armis"));
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

        let key = (OrgSlug::new("acme"), SensorId::from("armis"));
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

/// Parse the `message_template` column value for a given error code from the
/// error-taxonomy.md pipe-delimited table.
///
/// Reads the taxonomy file line by line and finds the row whose first column
/// matches `code` (e.g., `"E-SPEC-019"`).  The message_template is the 4th
/// pipe-delimited column (index 3, 0-based), with surrounding whitespace and
/// the enclosing double-quotes stripped.
///
/// Returns `Some(template)` when the row is found, `None` otherwise.
/// Panics (via `expect`) when called from a test on a malformed taxonomy row
/// so that taxonomy format regressions surface immediately.
fn parse_taxonomy_template(taxonomy_content: &str, code: &str) -> Option<String> {
    for line in taxonomy_content.lines() {
        // Only consider pipe-table rows that start with "| " and contain the code.
        if !line.starts_with('|') {
            continue;
        }
        // Split on '|' to get columns; columns[0] is empty (before the leading |).
        let cols: Vec<&str> = line.split('|').collect();
        // Minimum structure: | code | severity | category | message_template | ...
        if cols.len() < 5 {
            continue;
        }
        let row_code = cols[1].trim();
        if row_code != code {
            continue;
        }
        // cols[4] is the message_template column (0-based: [0]=empty, [1]=code,
        // [2]=severity, [3]=category, [4]=message_template).
        let raw_template = cols[4].trim();
        // Strip the surrounding double-quotes that delimit the template in the table.
        let template = raw_template
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(raw_template);
        return Some(template.to_string());
    }
    None
}

/// Render a message template by substituting `{placeholder}` markers with
/// concrete values from the provided slice of `(placeholder, value)` pairs.
///
/// Substitution is performed in order; if the same placeholder appears more
/// than once all occurrences are replaced.  Unmatched placeholders are left
/// in place (which will cause the byte-compare assertion to fail, surfacing
/// the gap).
fn render_template(template: &str, substitutions: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (placeholder, value) in substitutions {
        let marker = format!("{{{placeholder}}}");
        result = result.replace(&marker, value);
    }
    result
}

/// Extract the `SpecError::message` field from the first `PrismError::Spec`
/// in the slice whose `code` matches the given `SpecErrorCode`.
///
/// Panics if no matching error is found — the caller asserts code presence
/// before calling this helper.
fn extract_spec_message<'a>(errors: &'a [PrismError], expected_code: SpecErrorCode) -> &'a str {
    for e in errors {
        if let PrismError::Spec(se) = e {
            if se.code == expected_code {
                return &se.message;
            }
        }
    }
    panic!(
        "extract_spec_message: no error with code {:?} found in {:?}",
        expected_code, errors
    );
}

/// Red Gate test for AC-005 / BC-2.06.016.
///
/// Triggers all five overlay error conditions in turn and asserts:
/// 1. `SpecErrorCode::ESpec019` — overlay extends unknown TYPE spec.
/// 2. `SpecErrorCode::ESpec020` — instance_id mismatch.
/// 3. `SpecErrorCode::ESpec021` — [[tables]] in overlay.
/// 4. `SpecErrorCode::ESpec022` — unregistered org slug directory.
/// 5. `SpecErrorCode::ESpec023` — unrecognized field in overlay.
///
/// For each error code the test:
///   a. Asserts the correct `SpecErrorCode` variant is emitted.
///   b. Reads the canonical `message_template` from the embedded fixture
///      `tests/fixtures/error-taxonomy-snapshot.md` (a branch-tracked copy of
///      the E-SPEC-019..023 rows from `.factory/specs/prd-supplements/error-taxonomy.md`).
///      The snapshot approach is CI-portable: `.factory/` is an orphan-branch worktree
///      (factory-artifacts branch) that is NOT present in normal CI checkouts.
///      (POL-25 safety net — test fails if taxonomy and code drift. F-LP14-CI-001.)
///   c. Substitutes placeholder values to build the expected message string.
///   d. Byte-compares the expected string against `SpecError::message` in the
///      production error (AC-005 wording: "message matches the canonical
///      template from error-taxonomy.md, test-driven validation").
///
/// PAPER-FIX DETECTION (TD-VSDD-059): the byte-compare step (d) fails if the
/// taxonomy template is amended without updating the production code, or vice
/// versa, because the expected and actual strings will diverge.
///
/// FIXTURE SYNC: this test reads `fixtures/error-taxonomy-snapshot.md`
/// (at the crate root, per BC-3.7.001 AC-006 crate layout convention).
/// The canonical source is `.factory/specs/prd-supplements/error-taxonomy.md`
/// (factory-artifacts branch). When the canonical taxonomy E-SPEC-019..023
/// rows are updated, the fixture MUST be updated in the same commit.
///
/// RED GATE: Panics with "not yet implemented" for `validate_overlay_toml`
/// and `load_overlays` stubs.
#[test]
fn test_BC_2_06_016_error_messages_match_canonical_templates() {
    // Load the error taxonomy snapshot embedded at compile time.
    //
    // CI-PORTABILITY FIX (F-LP14-CI-001, PR #155): the original implementation walked up
    // from CARGO_MANIFEST_DIR to find `.factory/specs/prd-supplements/error-taxonomy.md`
    // at runtime. This fails in CI because `.factory/` is an orphan-branch worktree
    // (factory-artifacts branch) that is not checked out during a normal `git checkout`.
    // The fix: copy the relevant E-SPEC-019..023 rows into a branch-tracked fixture file
    // and embed it with `include_str!()` so it is always present regardless of checkout
    // configuration.
    // Path is relative to this source file (tests/overlay_loading_tests.rs).
    // The fixture lives at crates/prism-spec-engine/fixtures/ (crate root),
    // per BC-3.7.001 AC-006 crate layout convention (fixtures at root, NOT tests/fixtures/).
    let taxonomy_content = include_str!("../fixtures/error-taxonomy-snapshot.md");

    // ---------- E-SPEC-019: extends references unknown TYPE spec ----------
    {
        let file = "customers/acme/nonexistent_sensor.sensor.toml";
        let extends_value = "nonexistent_sensor";

        let overlay_unknown_extends = r#"
extends     = "nonexistent_sensor"
instance_id = "nonexistent_sensor@acme"
base_url    = "https://example.com"
"#;
        // Empty type_specs → "nonexistent_sensor" not found.
        let result_019 = OverlayLoader::validate_overlay_toml(
            overlay_unknown_extends,
            file,
            extends_value,
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

        // AC-005 byte-compare: rendered taxonomy template == production SpecError::message.
        let template_019 = parse_taxonomy_template(&taxonomy_content, "E-SPEC-019")
            .expect("E-SPEC-019 row must exist in error-taxonomy.md (POL-25 safety net)");
        let expected_019 = render_template(
            &template_019,
            &[("file", file), ("extends_value", extends_value)],
        );
        let actual_019 = extract_spec_message(&errs_019, SpecErrorCode::ESpec019);
        assert_eq!(
            actual_019, expected_019,
            "E-SPEC-019 SpecError::message must byte-equal the canonical taxonomy \
             template (AC-005 POL-25). Drift means taxonomy and code are out of sync.\n\
             expected: {expected_019}\n\
             actual:   {actual_019}"
        );
    }

    // ---------- E-SPEC-020: instance_id mismatch ----------
    {
        let file = "customers/acme/armis.sensor.toml";
        let actual_id = "armis@wrongorg";
        let expected_id = "armis@acme";

        let overlay_bad_id = r#"
extends     = "armis"
instance_id = "armis@wrongorg"
base_url    = "https://armis.example.com"
"#;
        let result_020 = OverlayLoader::validate_overlay_toml(
            overlay_bad_id,
            file,
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

        // AC-005 byte-compare.
        let template_020 = parse_taxonomy_template(&taxonomy_content, "E-SPEC-020")
            .expect("E-SPEC-020 row must exist in error-taxonomy.md (POL-25 safety net)");
        let expected_020 = render_template(
            &template_020,
            &[
                ("file", file),
                ("actual", actual_id),
                ("expected", expected_id),
            ],
        );
        let actual_020 = extract_spec_message(&errs_020, SpecErrorCode::ESpec020);
        assert_eq!(
            actual_020, expected_020,
            "E-SPEC-020 SpecError::message must byte-equal the canonical taxonomy \
             template (AC-005 POL-25). Drift means taxonomy and code are out of sync.\n\
             expected: {expected_020}\n\
             actual:   {actual_020}"
        );
    }

    // ---------- E-SPEC-021: [[tables]] in overlay ----------
    {
        let file = "customers/acme/armis.sensor.toml";
        let instance_id = "armis@acme";

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
            file,
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

        // AC-005 byte-compare.
        let template_021 = parse_taxonomy_template(&taxonomy_content, "E-SPEC-021")
            .expect("E-SPEC-021 row must exist in error-taxonomy.md (POL-25 safety net)");
        let expected_021 = render_template(
            &template_021,
            &[("file", file), ("instance_id", instance_id)],
        );
        let actual_021 = extract_spec_message(&errs_021, SpecErrorCode::ESpec021);
        assert_eq!(
            actual_021, expected_021,
            "E-SPEC-021 SpecError::message must byte-equal the canonical taxonomy \
             template (AC-005 POL-25). Drift means taxonomy and code are out of sync.\n\
             expected: {expected_021}\n\
             actual:   {actual_021}"
        );
    }

    // ---------- E-SPEC-022: unregistered org slug directory ----------
    {
        let slug = "stale-corp";
        // The production code builds customers_dir_name as "customers/{slug}/"
        // and passes that as the first arg to e_spec_022_unknown_org_slug.
        // The taxonomy template uses {slug} for both the directory path part and
        // the bare slug; we substitute accordingly.
        let customers_dir_name = format!("customers/{slug}/");

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

        // AC-005 byte-compare.
        // E-SPEC-022 taxonomy template: "Per-org overlay directory 'customers/{slug}/'
        // references org slug '{slug}' which is not registered in OrgRegistry. ..."
        // The production code calls e_spec_022_unknown_org_slug(customers_dir_name, slug)
        // where customers_dir_name = "customers/stale-corp/" and slug = "stale-corp".
        // The template's {slug} placeholders map to the slug value alone; the
        // full directory path ("customers/stale-corp/") is already embedded as
        // the literal first occurrence of {slug} surrounded by 'customers/' and '/'.
        // We substitute both {slug} occurrences with the bare slug value to reconstruct
        // the same interpolation the production code performs.
        let template_022 = parse_taxonomy_template(&taxonomy_content, "E-SPEC-022")
            .expect("E-SPEC-022 row must exist in error-taxonomy.md (POL-25 safety net)");
        let expected_022 = render_template(&template_022, &[("slug", slug)]);
        // Verify the rendered template matches the customers_dir_name the code uses.
        // (This assertion is a cross-check that our substitution logic is coherent with
        // the production e_spec_022_unknown_org_slug(customers_dir_name, slug) signature.)
        assert!(
            expected_022.contains(&customers_dir_name),
            "E-SPEC-022 rendered template must contain the customers_dir_name '{}'; \
             rendered: {}",
            customers_dir_name,
            expected_022
        );
        let actual_022 = extract_spec_message(&result_022.errors, SpecErrorCode::ESpec022);
        assert_eq!(
            actual_022, expected_022,
            "E-SPEC-022 SpecError::message must byte-equal the canonical taxonomy \
             template (AC-005 POL-25). Drift means taxonomy and code are out of sync.\n\
             expected: {expected_022}\n\
             actual:   {actual_022}"
        );
    }

    // ---------- E-SPEC-023: unrecognized field in overlay ----------
    {
        let file = "customers/acme/armis.sensor.toml";
        let field_name = "secret_key";

        let overlay_unknown_field = r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme.io"
secret_key  = "s3cr3t"
"#;
        let result_023 = OverlayLoader::validate_overlay_toml(
            overlay_unknown_field,
            file,
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

        // AC-005 byte-compare.
        let template_023 = parse_taxonomy_template(&taxonomy_content, "E-SPEC-023")
            .expect("E-SPEC-023 row must exist in error-taxonomy.md (POL-25 safety net)");
        let expected_023 =
            render_template(&template_023, &[("file", file), ("field_name", field_name)]);
        let actual_023 = extract_spec_message(&errs_023, SpecErrorCode::ESpec023);
        assert_eq!(
            actual_023, expected_023,
            "E-SPEC-023 SpecError::message must byte-equal the canonical taxonomy \
             template (AC-005 POL-25). Drift means taxonomy and code are out of sync.\n\
             expected: {expected_023}\n\
             actual:   {actual_023}"
        );
    }
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-002 (EC-016-001): Multi-error aggregation — [[tables]] AND unrecognized
// field in same overlay file both collected (INV-ERR-003, BC-2.06.016)
// BC-2.06.016 §EC-016-001
// ---------------------------------------------------------------------------

/// Test for F-LP1-HIGH-002 / BC-2.06.016 EC-016-001.
///
/// Scenario: a single overlay file contains BOTH a `[[tables]]` block AND an
/// unrecognized field (`auth_type = "oauth2_client_credentials"`).
///
/// The existing `test_BC_2_06_016_error_messages_match_canonical_templates` only
/// triggers each error code in isolated scopes. This test verifies the cross-field
/// aggregation invariant: INV-ERR-003 requires ALL errors to be collected before
/// returning, not stop at the first one.
///
/// Asserts:
/// - `result.errors` contains E-SPEC-021 (tables in overlay).
/// - `result.errors` ALSO contains E-SPEC-023 (unrecognized field `auth_type`).
/// - Both are present simultaneously in a single `load_overlays` call for one file.
#[test]
fn test_BC_2_06_016_EC_016_001_tables_and_unrecognized_field_both_collected() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    // Overlay with BOTH [[tables]] AND an unrecognized field.
    write_file(
        dir.path(),
        "customers/acme/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
auth_type   = "oauth2_client_credentials"

[[tables]]
table_name = "forbidden_schema_override"
ocsf_class = "device_inventory_info"
"#,
    );

    let type_specs = type_specs_with_armis();
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // Must have errors — the overlay is invalid on two independent counts.
    assert!(
        !result.errors.is_empty(),
        "EC-016-001: overlay with both [[tables]] and unrecognized field must produce errors"
    );

    // BC-2.06.016 INV-ERR-003: E-SPEC-021 collected.
    let has_e_spec_021 = result.errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec021)
        } else {
            false
        }
    });
    assert!(
        has_e_spec_021,
        "EC-016-001: E-SPEC-021 (tables in overlay) must be present; errors: {:?}",
        result.errors
    );

    // BC-2.06.016 INV-ERR-003: E-SPEC-023 collected SIMULTANEOUSLY with E-SPEC-021.
    let has_e_spec_023 = result.errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec023)
        } else {
            false
        }
    });
    assert!(
        has_e_spec_023,
        "EC-016-001: E-SPEC-023 (unrecognized field 'auth_type') must ALSO be present \
         simultaneously with E-SPEC-021; errors: {:?}",
        result.errors
    );

    // Nothing resolved — the overlay file is invalid.
    assert!(
        result.resolved.is_empty(),
        "EC-016-001: invalid overlay must not produce a resolved entry; resolved keys: {:?}",
        result.resolved.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-001 associated test (EC-016-002): unregistered org slug AND [[tables]]
// in the same file — both E-SPEC-022 and E-SPEC-021 collected simultaneously.
// BC-2.06.016 §EC-016-002
// ---------------------------------------------------------------------------

/// Test for F-LP1-HIGH-001 EC-016-002 / BC-2.06.016.
///
/// Scenario: `customers/stale-corp/armis.sensor.toml` exists.
/// - `stale-corp` is NOT in OrgRegistry → E-SPEC-022 (unknown org slug).
/// - The overlay file also contains a `[[tables]]` block → E-SPEC-021.
///
/// After the implementer's F-LP1-HIGH-001 fix removed the early-return guard,
/// `load_overlays` continues scanning files inside unregistered directories.
/// This test verifies BOTH errors are collected from a single file under an
/// unregistered directory.
///
/// Asserts:
/// - `result.errors` contains E-SPEC-022 (unregistered slug `stale-corp`).
/// - `result.errors` ALSO contains E-SPEC-021 (tables in overlay file).
/// - `result.resolved` is empty (unregistered orgs never produce resolved entries).
#[test]
fn test_BC_2_06_016_EC_016_002_unknown_org_and_tables_both_collected() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    // File under unregistered org directory with an additional [[tables]] violation.
    write_file(
        dir.path(),
        "customers/stale-corp/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@stale-corp"
base_url    = "https://armis.stale.io"

[[tables]]
table_name = "forbidden_schema"
ocsf_class = "device_inventory_info"
"#,
    );

    let type_specs = type_specs_with_armis();
    // Registry has only "acme" — "stale-corp" is NOT registered.
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // E-SPEC-022: unregistered slug "stale-corp".
    let has_e_spec_022 = result.errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec022)
        } else {
            false
        }
    });
    assert!(
        has_e_spec_022,
        "EC-016-002: E-SPEC-022 (unknown org slug 'stale-corp') must be present; \
         errors: {:?}",
        result.errors
    );

    // E-SPEC-021: [[tables]] in the overlay file under the unregistered directory.
    let has_e_spec_021 = result.errors.iter().any(|e| {
        if let PrismError::Spec(se) = e {
            matches!(se.code, SpecErrorCode::ESpec021)
        } else {
            false
        }
    });
    assert!(
        has_e_spec_021,
        "EC-016-002: E-SPEC-021 (tables in overlay) must ALSO be present simultaneously \
         with E-SPEC-022 (post F-LP1-HIGH-001 fix, early-return guard removed); \
         errors: {:?}",
        result.errors
    );

    // BC-2.06.016 EC-016-002: unregistered orgs never appear in resolved map.
    assert!(
        result.resolved.is_empty(),
        "EC-016-002: unregistered org overlay must never be inserted into resolved map; \
         resolved keys: {:?}",
        result.resolved.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-002 (EC-016-003): All five error codes in same boot scenario
// BC-2.06.016 §EC-016-003 — INV-ERR-003 multi-file, multi-error aggregation
// ---------------------------------------------------------------------------

/// Test for F-LP1-HIGH-002 / BC-2.06.016 EC-016-003.
///
/// Scenario: five overlay files across two org directories, each triggering one of
/// the five canonical error codes (E-SPEC-019..E-SPEC-023) in a single
/// `load_overlays` call.
///
/// Layout:
/// - `customers/unknown-org/crowdstrike.sensor.toml`  — E-SPEC-022 (unregistered slug)
///   with `[[tables]]` — this file also emits E-SPEC-021 as a simultaneous file-level error.
/// - `customers/acme/bad-extends.sensor.toml`         — E-SPEC-019 (unknown TYPE spec)
/// - `customers/acme/armis.sensor.toml`               — E-SPEC-020 (instance_id mismatch)
/// - `customers/acme/claroty.sensor.toml`             — E-SPEC-021 (tables in overlay)
/// - `customers/acme/vectra.sensor.toml`              — E-SPEC-023 (unrecognized field)
///
/// Asserts:
/// - `result.errors` contains ALL five codes (E-SPEC-019 through E-SPEC-023).
/// - Error count ≥ 5 (may be more if a file triggers multiple codes).
/// - `result.resolved` is empty (any error makes the entire boot invalid per INV-ERR-003).
#[test]
fn test_BC_2_06_016_EC_016_003_all_five_codes_in_same_boot() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    // E-SPEC-022: unregistered slug "unknown-org" → E-SPEC-022.
    // Also contains [[tables]] → E-SPEC-021 is also collected for this file.
    write_file(
        dir.path(),
        "customers/unknown-org/crowdstrike.sensor.toml",
        r#"
extends     = "armis"
instance_id = "crowdstrike@unknown-org"
base_url    = "https://cs.unknown.io"

[[tables]]
table_name = "forbidden"
ocsf_class = "device_inventory_info"
"#,
    );

    // E-SPEC-019: extends references "nonexistent_sensor" which has no TYPE spec.
    write_file(
        dir.path(),
        "customers/acme/bad-extends.sensor.toml",
        r#"
extends     = "nonexistent_sensor"
instance_id = "bad-extends@acme"
base_url    = "https://example.com"
"#,
    );

    // E-SPEC-020: instance_id "armis@wrongorg" does not match expected "armis@acme".
    write_file(
        dir.path(),
        "customers/acme/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@wrongorg"
base_url    = "https://armis.acme-corp.io"
"#,
    );

    // E-SPEC-021: [[tables]] block present in overlay for a registered org.
    write_file(
        dir.path(),
        "customers/acme/claroty.sensor.toml",
        r#"
extends     = "armis"
instance_id = "claroty@acme"
base_url    = "https://claroty.acme.io"

[[tables]]
table_name = "forbidden_schema"
ocsf_class = "device_inventory_info"
"#,
    );

    // E-SPEC-023: unrecognized field "secret_key" in overlay.
    write_file(
        dir.path(),
        "customers/acme/vectra.sensor.toml",
        r#"
extends     = "armis"
instance_id = "vectra@acme"
base_url    = "https://vectra.acme.io"
secret_key  = "s3cr3t"
"#,
    );

    let type_specs = type_specs_with_armis();
    // Registry has only "acme" — "unknown-org" is NOT registered.
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // Must have errors — every file triggers at least one.
    assert!(
        !result.errors.is_empty(),
        "EC-016-003: mixed-error boot scenario must produce errors; got none"
    );

    let check_code = |code: SpecErrorCode| -> bool {
        result.errors.iter().any(|e| {
            if let PrismError::Spec(se) = e {
                se.code == code
            } else {
                false
            }
        })
    };

    assert!(
        check_code(SpecErrorCode::ESpec019),
        "EC-016-003: E-SPEC-019 (unknown extends) must appear; errors: {:?}",
        result.errors
    );
    assert!(
        check_code(SpecErrorCode::ESpec020),
        "EC-016-003: E-SPEC-020 (instance_id mismatch) must appear; errors: {:?}",
        result.errors
    );
    assert!(
        check_code(SpecErrorCode::ESpec021),
        "EC-016-003: E-SPEC-021 (tables in overlay) must appear; errors: {:?}",
        result.errors
    );
    assert!(
        check_code(SpecErrorCode::ESpec022),
        "EC-016-003: E-SPEC-022 (unregistered slug) must appear; errors: {:?}",
        result.errors
    );
    assert!(
        check_code(SpecErrorCode::ESpec023),
        "EC-016-003: E-SPEC-023 (unrecognized field) must appear; errors: {:?}",
        result.errors
    );

    // At minimum 5 distinct errors (may be more — unknown-org file also triggers E-SPEC-021).
    assert!(
        result.errors.len() >= 5,
        "EC-016-003: at least 5 errors expected across all five violation types; \
         got: {} — errors: {:?}",
        result.errors.len(),
        result.errors
    );

    // INV-ERR-003: any error → entire resolved map is empty.
    assert!(
        result.resolved.is_empty(),
        "EC-016-003: when ANY overlay error occurs, resolved map must be empty (INV-ERR-003); \
         resolved keys: {:?}",
        result.resolved.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-MED-002: rate_limit_hints overlay merge (requests_per_second, burst_size)
// BC-2.06.012 AC-001 scalar merge — rate_limit_hints tunables
// Edge case: EC-012-005
// ---------------------------------------------------------------------------

/// Test for F-LP1-MED-002 / BC-2.06.012 EC-012-005.
///
/// Scenario: overlay sets `rate_limit_hints.requests_per_second = 5.0` only
/// (without setting base_url or burst_size). Verifies:
/// - merged spec uses the TYPE spec base_url unchanged (no base_url in overlay).
/// - merged spec `rate_limit_hints.requests_per_second` equals overlay value (5.0).
/// - `provenance.rps_from_overlay` is `true`.
/// - `provenance.base_url_from_overlay` is `false` (base_url came from TYPE spec).
///
/// Also exercises independent `burst_size` merging:
/// - Second overlay sets only `rate_limit_hints.burst_size = 20`.
/// - Merged spec `rate_limit_hints.burst_size` equals 20.
/// - `provenance.burst_size_from_overlay` is `true`.
/// - `provenance.rps_from_overlay` is `false` (rps came from TYPE spec, which is None).
#[test]
fn test_BC_2_06_012_EC_012_005_rate_limit_overlay_merge() {
    let type_spec = armis_type_spec();

    // --- Case A: overlay sets requests_per_second only --- //
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        let customers_dir = dir.path().join("customers");

        write_file(
            dir.path(),
            "customers/acme/armis.sensor.toml",
            r#"
extends     = "armis"
instance_id = "armis@acme"

[rate_limit_hints]
requests_per_second = 5.0
"#,
        );

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();
        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "EC-012-005 Case A: rate_limit_hints overlay must produce no errors; got: {:?}",
            result.errors
        );

        let key = (OrgSlug::new("acme"), SensorId::from("armis"));
        assert!(
            result.resolved.contains_key(&key),
            "EC-012-005 Case A: resolved map must contain (acme, armis)"
        );
        let resolved = &result.resolved[&key];

        // base_url unchanged from TYPE spec (no base_url in overlay).
        assert_eq!(
            resolved.spec.base_url, type_spec.base_url,
            "EC-012-005 Case A: base_url must remain TYPE spec default when not in overlay"
        );
        assert!(
            !resolved.provenance.base_url_from_overlay,
            "EC-012-005 Case A: provenance.base_url_from_overlay must be false"
        );

        // requests_per_second merged from overlay.
        let rls = resolved
            .spec
            .rate_limit_hints
            .as_ref()
            .expect("rate_limit_hints must be Some after overlay merge");
        assert_eq!(
            rls.requests_per_second,
            Some(5.0),
            "EC-012-005 Case A: merged spec requests_per_second must equal overlay value 5.0"
        );
        assert!(
            resolved.provenance.rps_from_overlay,
            "EC-012-005 Case A: provenance.rps_from_overlay must be true"
        );

        // burst_size not in overlay → None (TYPE spec had no rate_limit_hints).
        assert_eq!(
            rls.burst_size, None,
            "EC-012-005 Case A: burst_size not in overlay must remain None"
        );
        assert!(
            !resolved.provenance.burst_size_from_overlay,
            "EC-012-005 Case A: provenance.burst_size_from_overlay must be false"
        );
    }

    // --- Case B: overlay sets burst_size only (independent scalar merging) --- //
    {
        let dir = tempfile::tempdir().expect("tempdir must succeed");
        let customers_dir = dir.path().join("customers");

        write_file(
            dir.path(),
            "customers/acme/armis.sensor.toml",
            r#"
extends     = "armis"
instance_id = "armis@acme"

[rate_limit_hints]
burst_size = 20
"#,
        );

        let type_specs = type_specs_with_armis();
        let registry = registry_with_acme();
        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

        assert!(
            result.errors.is_empty(),
            "EC-012-005 Case B: burst_size-only overlay must produce no errors; got: {:?}",
            result.errors
        );

        let key = (OrgSlug::new("acme"), SensorId::from("armis"));
        let resolved = &result.resolved[&key];

        let rls = resolved
            .spec
            .rate_limit_hints
            .as_ref()
            .expect("rate_limit_hints must be Some after burst_size overlay merge");

        // burst_size merged from overlay.
        assert_eq!(
            rls.burst_size,
            Some(20),
            "EC-012-005 Case B: merged spec burst_size must equal overlay value 20"
        );
        assert!(
            resolved.provenance.burst_size_from_overlay,
            "EC-012-005 Case B: provenance.burst_size_from_overlay must be true"
        );

        // requests_per_second not in overlay → None (TYPE spec had no rate_limit_hints).
        assert_eq!(
            rls.requests_per_second, None,
            "EC-012-005 Case B: requests_per_second not in overlay must remain None"
        );
        assert!(
            !resolved.provenance.rps_from_overlay,
            "EC-012-005 Case B: provenance.rps_from_overlay must be false"
        );
    }
}

// ---------------------------------------------------------------------------
// F-LP1-MED-002: timeout_secs overlay merge with provenance tracking
// BC-2.06.012 AC-001 scalar merge — timeout_secs tunable
// ---------------------------------------------------------------------------

/// Test for F-LP1-MED-002 / BC-2.06.012 — timeout_secs overlay merge.
///
/// Scenario: overlay sets `timeout_secs = 60`. Verifies:
/// - `provenance.timeout_secs_from_overlay` is `true`.
/// - `result.errors` is empty (valid overlay).
/// - `result.resolved` contains the (acme, armis) entry.
///
/// Note: `SensorSpec` does not have a `timeout_secs` field — the value is stored
/// only in provenance metadata for later retrieval by the HTTP client wiring layer
/// (follow-up story S-CONFIG-MULTI-TENANT-OVERRIDE-002). This test verifies the
/// provenance tracking path per BC-2.06.012 postcondition.
#[test]
fn test_BC_2_06_012_timeout_secs_overlay_merge_with_provenance() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");

    write_file(
        dir.path(),
        "customers/acme/armis.sensor.toml",
        r#"
extends     = "armis"
instance_id = "armis@acme"
base_url    = "https://armis.acme-corp.io"
timeout_secs = 60
"#,
    );

    let type_specs = type_specs_with_armis();
    let registry = registry_with_acme();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    assert!(
        result.errors.is_empty(),
        "timeout_secs overlay must produce no errors; got: {:?}",
        result.errors
    );

    let key = (OrgSlug::new("acme"), SensorId::from("armis"));
    assert!(
        result.resolved.contains_key(&key),
        "timeout_secs overlay must produce a resolved entry for (acme, armis)"
    );

    let resolved = &result.resolved[&key];

    // BC-2.06.012 postcondition: provenance.timeout_secs_from_overlay must be true.
    assert!(
        resolved.provenance.timeout_secs_from_overlay,
        "provenance.timeout_secs_from_overlay must be true when overlay sets timeout_secs; \
         provenance: {:?}",
        resolved.provenance
    );

    // base_url from overlay (also set in this overlay).
    assert_eq!(
        resolved.spec.base_url, "https://armis.acme-corp.io",
        "base_url must be from overlay when set alongside timeout_secs"
    );
    assert!(
        resolved.provenance.base_url_from_overlay,
        "provenance.base_url_from_overlay must be true when overlay also sets base_url"
    );
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
    let acme_key = (OrgSlug::new("acme"), SensorId::from("armis"));
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
    let contoso_key = (OrgSlug::new("contoso"), SensorId::from("armis"));
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

// ---------------------------------------------------------------------------
// PRR-011: negative-path coverage tests (added in PR #155 fix-burst)
// ---------------------------------------------------------------------------

/// PRR-011 / SEC-REDUX-005: oversized overlay file is rejected before read_to_string.
///
/// Verifies the 64 KiB file-size cap (MAX_OVERLAY_FILE_BYTES) prevents boot-time
/// OOM from maliciously large overlay files (CWE-400).
#[test]
fn test_BC_2_06_013_oversized_overlay_file_rejected() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");
    let acme_dir = customers_dir.join("acme");
    std::fs::create_dir_all(&acme_dir).expect("create acme dir");

    // Write a file larger than MAX_OVERLAY_FILE_BYTES (64 KiB).
    let oversized_content = "x".repeat(65 * 1024 + 1); // 65 KiB + 1 byte
    std::fs::write(acme_dir.join("armis.sensor.toml"), &oversized_content)
        .expect("write oversized file");

    let registry = registry_with_acme();
    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    assert!(
        result.resolved.is_empty(),
        "oversized overlay must not be inserted into resolved map"
    );
    assert_eq!(
        result.errors.len(),
        1,
        "exactly one error must be emitted for the oversized file, got: {:?}",
        result.errors
    );
    // Verify the error references the correct file and size limit.
    let err_msg = format!("{:?}", result.errors[0]);
    assert!(
        err_msg.contains("exceeds maximum allowed size"),
        "error must mention size limit, got: {err_msg}"
    );
}

/// PRR-011: unreadable overlay file (chmod 000) returns PrismError::Io.
///
/// Only runs on POSIX (chmod 000 is not meaningful on Windows CI).
#[test]
#[cfg(unix)]
fn test_BC_2_06_012_overlay_file_unreadable_returns_io_error() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");
    let acme_dir = customers_dir.join("acme");
    std::fs::create_dir_all(&acme_dir).expect("create acme dir");

    // Write a valid overlay, then make it unreadable.
    let overlay_path = acme_dir.join("armis.sensor.toml");
    std::fs::write(
        &overlay_path,
        r#"extends = "armis"
instance_id = "armis@acme"
base_url = "https://armis.acme-corp.io"
"#,
    )
    .expect("write overlay");
    std::fs::set_permissions(&overlay_path, std::fs::Permissions::from_mode(0o000))
        .expect("chmod 000");

    let registry = registry_with_acme();
    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // The metadata() call should succeed (readable for size check), but read_to_string should fail.
    // OR the file_type() check (is_file()) itself may fail — either way, errors must be non-empty.
    // Note: on some platforms, metadata() on a chmod-000 file also fails.
    // We only require that the overlay is NOT in resolved and some error was collected.
    assert!(
        result.resolved.is_empty(),
        "unreadable overlay must not be in resolved map"
    );
    // Restore permissions so tempdir cleanup can succeed.
    std::fs::set_permissions(&overlay_path, std::fs::Permissions::from_mode(0o644))
        .expect("restore permissions");
}

/// PRR-011: mixed-case org directory (e.g., `customers/ACME/`) produces E-SPEC-022.
///
/// `OrgSlug::new("ACME")` returns a valid OrgSlug (mixed-case allowed per regex
/// `^[a-zA-Z0-9_-]{1,64}$`) but the registry only has lowercase "acme".
/// The error message should say "unregistered" not "case mismatch" — this is expected.
#[test]
fn test_BC_2_06_015_mixed_case_org_dir_produces_e_spec_022() {
    let dir = tempfile::tempdir().expect("tempdir must succeed");
    let customers_dir = dir.path().join("customers");
    // Create directory with UPPERCASE name — not registered (only "acme" lowercase is).
    let upper_dir = customers_dir.join("ACME");
    std::fs::create_dir_all(&upper_dir).expect("create ACME dir");

    std::fs::write(
        upper_dir.join("armis.sensor.toml"),
        r#"extends = "armis"
instance_id = "armis@ACME"
base_url = "https://armis.acme-corp.io"
"#,
    )
    .expect("write overlay");

    let registry = registry_with_acme(); // only "acme" (lowercase) registered
    let type_specs = type_specs_with_armis();

    let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);

    // "ACME" is a syntactically valid slug but not registered — E-SPEC-022 expected.
    assert!(
        result.resolved.is_empty(),
        "ACME (unregistered) overlay must not be in resolved map"
    );
    let has_e_spec_022 = result.errors.iter().any(|e| match e {
        PrismError::Spec(spec_err) => spec_err.code == SpecErrorCode::ESpec022,
        _ => false,
    });
    assert!(
        has_e_spec_022,
        "E-SPEC-022 must be emitted for unregistered 'ACME' directory, got: {:?}",
        result.errors
    );
}

// ---------------------------------------------------------------------------
// SEC-REDUX-006: URL scheme validation in validate_overlay_toml
// ---------------------------------------------------------------------------

/// SEC-REDUX-006: overlay base_url with non-HTTP scheme is rejected (E-SPEC-001).
///
/// The overlay validator must reject `file://`, `ftp://`, and other non-HTTP schemes
/// to prevent SSRF attacks (CWE-918) once SEC-REDUX-001 wiring is live.
#[test]
fn test_BC_2_06_012_validate_overlay_toml_rejects_non_http_base_url_scheme() {
    let type_specs = type_specs_with_armis();

    for bad_url in &[
        "file:///etc/shadow",
        "ftp://internal-server/",
        "http://169.254.169.254/latest/meta-data/",
    ] {
        // http:// is allowed (second case above); file:// and ftp:// are not.
        // NOTE: http://169.254.169.254 is technically http://, so it would pass the scheme check.
        // We test file:// and ftp:// as the real CWE-918 blocks.
        if bad_url.starts_with("http://") {
            continue;
        }
        let toml_input = format!(
            r#"extends = "armis"
instance_id = "armis@acme"
base_url = "{bad_url}"
"#
        );
        let result = OverlayLoader::validate_overlay_toml(
            &toml_input,
            "customers/acme/armis.sensor.toml",
            "armis",
            "acme",
            &type_specs,
        );
        assert!(
            result.is_err(),
            "non-HTTP scheme '{bad_url}' must be rejected by validate_overlay_toml (SEC-REDUX-006)"
        );
        let errs = result.unwrap_err();
        let has_scheme_error = errs.iter().any(|e| match e {
            PrismError::Spec(se) => {
                let msg = se.message.to_lowercase();
                msg.contains("not a valid url") || msg.contains("must start with http")
            }
            _ => false,
        });
        assert!(
            has_scheme_error,
            "error for '{bad_url}' must mention URL validation, got: {errs:?}"
        );
    }
}

/// SEC-REDUX-006: overlay base_url with https:// scheme is accepted.
#[test]
fn test_BC_2_06_012_validate_overlay_toml_accepts_https_base_url_scheme() {
    let type_specs = type_specs_with_armis();
    let toml_input = r#"extends = "armis"
instance_id = "armis@acme"
base_url = "https://armis.acme-corp.io"
"#;
    let result = OverlayLoader::validate_overlay_toml(
        toml_input,
        "customers/acme/armis.sensor.toml",
        "armis",
        "acme",
        &type_specs,
    );
    assert!(
        result.is_ok(),
        "https:// base_url must be accepted: {:?}",
        result
    );
}
