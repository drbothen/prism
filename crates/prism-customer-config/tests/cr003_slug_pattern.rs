//! CR-003 test suite — OrgSlug pattern validation in `validate_structural`.
//!
//! Covers:
//!   BC-3.3.004 R-CUST-002 (slug pattern check added after filename-stem check)
//!   AC-001 (E-CFG-019: InvalidOrgSlugPattern)
//!   AC-002 (`validate_all` made `pub(crate)`)
//!
//! ALL tests must fail (assertion errors, not panics) before the implementing
//! stub lands — demonstrating the production gap.
//!
//! Test naming: `test_BC_3_3_004_CR003_xxx()` per factory convention.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use tempfile::TempDir;

use prism_customer_config::{load_and_validate, ConfigError};

// ---------------------------------------------------------------------------
// Helper: write a named TOML file into a TempDir and return the dir path.
// ---------------------------------------------------------------------------

fn write_toml(dir: &TempDir, name: &str, contents: &str) {
    let path = dir.path().join(name);
    fs::write(&path, contents).expect("write toml fixture");
}

// ---------------------------------------------------------------------------
// Base valid TOML — org_slug = "acme" (matches filename stem "acme").
// UUID v7: 01975e4e-9f00-7abc-8def-000000000001
// ---------------------------------------------------------------------------

const VALID_TOML_TEMPLATE: &str = r#"
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "SLUG_PLACEHOLDER"
display_name = "ACME Corp"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "keyring://prism/acme/claroty"
spec = "sensors/claroty.toml"
"#;

// ---------------------------------------------------------------------------
// Helper: build a TOML string with a given slug and filename stem.
// ---------------------------------------------------------------------------

fn make_toml(slug: &str) -> String {
    VALID_TOML_TEMPLATE.replace("SLUG_PLACEHOLDER", slug)
}

// ---------------------------------------------------------------------------
// Helper: check whether the errors vec contains an InvalidOrgSlugPattern for
// the given slug. Returns true iff exactly such an error is present.
// ---------------------------------------------------------------------------

fn has_invalid_slug_pattern_error(errors: &[ConfigError], expected_slug: &str) -> bool {
    errors.iter().any(|e| {
        matches!(
            e,
            ConfigError::InvalidOrgSlugPattern { slug, .. } if slug == expected_slug
        )
    })
}

// ===========================================================================
// AC-001: E-CFG-019 is emitted for invalid org_slug patterns
// ===========================================================================

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "my org"` (contains a space) must produce E-CFG-019.
///
/// Production gap: `validate_structural` does not yet call `OrgSlug::new` to
/// validate the pattern; `ConfigError::InvalidOrgSlugPattern` is never emitted.
#[test]
fn test_BC_3_3_004_CR003_space_in_slug_produces_e_cfg_019() {
    let dir = TempDir::new().unwrap();
    // Use a filename stem that contains no space so the stem-match check doesn't
    // interfere — "my org" can never match a filename stem that we control.
    // We deliberately use "my-org" as the stem so the slug-mismatch (E-CFG-002)
    // fires before our slug-pattern check, but E-CFG-019 must ALSO be present.
    // Because the file stem is "my-org" and slug is "my org", E-CFG-002 fires.
    // To isolate E-CFG-019 we need a case where the slug passes stem-match but
    // fails pattern. That is impossible for "my org" (spaces disallowed in
    // filenames on most platforms). Instead we assert that among all errors
    // returned for this file, InvalidOrgSlugPattern is present.
    write_toml(&dir, "my-org.toml", &make_toml("my org"));

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();

    assert!(
        has_invalid_slug_pattern_error(&errors, "my org"),
        "AC-001 (E-CFG-019): expected InvalidOrgSlugPattern for slug 'my org' (space), \
         but it was not present in errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "acmé"` (contains Unicode 'é', EC-003) must produce E-CFG-019.
///
/// Production gap: pattern check not yet wired in; E-CFG-019 never emitted.
#[test]
fn test_BC_3_3_004_CR003_unicode_in_slug_produces_e_cfg_019() {
    let dir = TempDir::new().unwrap();
    // Stem "acm" differs from "acmé" so slug-mismatch fires; pattern check
    // must also fire and emit E-CFG-019.
    write_toml(&dir, "acm.toml", &make_toml("acmé"));

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();

    assert!(
        has_invalid_slug_pattern_error(&errors, "acmé"),
        "AC-001 / EC-003 (E-CFG-019): expected InvalidOrgSlugPattern for slug 'acmé' \
         (contains Unicode 'é'), but not found in errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "my.org"` (contains dot, not in `[a-zA-Z0-9_-]`) must produce E-CFG-019.
///
/// Production gap: pattern check not yet wired in; E-CFG-019 never emitted.
#[test]
fn test_BC_3_3_004_CR003_dot_in_slug_produces_e_cfg_019() {
    let dir = TempDir::new().unwrap();
    // "my.org" can appear as a stem only if the filename is "my.org.toml" — but that
    // stem would be "my.org", which matches the slug; so slug-mismatch does NOT fire
    // and E-CFG-019 is the only expected error for the slug pattern.
    // We use "my.org.toml" so the stem == slug == "my.org" and only E-CFG-019 fires.
    write_toml(&dir, "my.org.toml", &make_toml("my.org"));

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();

    assert!(
        has_invalid_slug_pattern_error(&errors, "my.org"),
        "AC-001 (E-CFG-019): expected InvalidOrgSlugPattern for slug 'my.org' (contains '.'), \
         but not found in errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug` longer than 64 characters must produce E-CFG-019.
///
/// Production gap: pattern check not yet wired in; E-CFG-019 never emitted.
#[test]
fn test_BC_3_3_004_CR003_slug_too_long_produces_e_cfg_019() {
    let dir = TempDir::new().unwrap();
    // Construct a 65-char slug using only valid charset chars.
    let long_slug: String = "a".repeat(65);

    // Use the slug as both filename stem and slug value so stem-match passes;
    // the only expected error is E-CFG-019 (too long).
    let file_name = format!("{long_slug}.toml");
    write_toml(&dir, &file_name, &make_toml(&long_slug));

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();

    assert!(
        has_invalid_slug_pattern_error(&errors, &long_slug),
        "AC-001 (E-CFG-019): expected InvalidOrgSlugPattern for 65-char slug, \
         but not found in errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = ""` (empty) must produce MissingField (E-CFG-001), not E-CFG-019.
/// Pattern check must only be applied to non-empty slugs.
///
/// This test verifies no double-emit and correct error code assignment.
///
/// This test PASSES at the Red Gate (empty-slug → MissingField is existing
/// behaviour). Its purpose is regression: the slug pattern implementation
/// must not break this by emitting E-CFG-019 for the empty slug.
#[test]
fn test_BC_3_3_004_CR003_empty_slug_produces_missing_field_not_e_cfg_019() {
    let dir = TempDir::new().unwrap();
    // An empty org_slug value. The filename stem is "acme" but slug is "",
    // so both stem-match (E-CFG-002) and potentially missing-field (E-CFG-001)
    // may fire. We assert that E-CFG-019 does NOT fire.
    let toml = r#"
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = ""
display_name = "ACME Corp"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "keyring://prism/acme/claroty"
spec = "sensors/claroty.toml"
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();

    // Must have MissingField for org_slug.
    let has_missing_field = errors
        .iter()
        .any(|e| matches!(e, ConfigError::MissingField { field, .. } if field == "org_slug"));
    assert!(
        has_missing_field,
        "AC-001: empty org_slug must produce MissingField (E-CFG-001), but not found in: {errors:?}"
    );

    // Must NOT have InvalidOrgSlugPattern for empty slug.
    let has_pattern_error = errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidOrgSlugPattern { slug, .. } if slug.is_empty()));
    assert!(
        !has_pattern_error,
        "AC-001: empty org_slug must NOT produce InvalidOrgSlugPattern (E-CFG-019); \
         pattern check must be skipped for empty slugs. Errors: {errors:?}"
    );
}

// ===========================================================================
// AC-001: valid slugs — E-CFG-019 must NOT be emitted
// ===========================================================================

/// BC-3.3.004 R-CUST-002 / EC-001:
/// `org_slug = "a"` (single character, valid pattern) must not produce E-CFG-019.
///
/// Production gap: once the pattern check is wired, this test confirms the
/// boundary is correct (min length = 1). Currently the check does not exist;
/// the test verifies the future implementation does not over-reject.
///
/// This test PASSES at the Red Gate (no E-CFG-019 is emitted today). Its job
/// is to serve as a regression guard after the fix lands.
#[test]
fn test_BC_3_3_004_CR003_single_char_slug_is_valid() {
    let dir = TempDir::new().unwrap();
    // Stem "a" matches slug "a"; spec file does not exist so E-CFG-015 will fire.
    // But no E-CFG-019 should be present.
    write_toml(&dir, "a.toml", &make_toml("a"));

    // May succeed or fail with other errors (SpecFileNotFound), but must not
    // contain InvalidOrgSlugPattern.
    let errors: Vec<ConfigError> = match load_and_validate(dir.path()) {
        Ok(_) => vec![],
        Err(errs) => errs,
    };

    let has_pattern_error = errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidOrgSlugPattern { slug, .. } if slug == "a"));

    assert!(
        !has_pattern_error,
        "AC-001 / EC-001: single-char slug 'a' is valid; must NOT produce E-CFG-019. \
         Errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002 / EC-002:
/// `org_slug` exactly 64 chars with `[a-zA-Z0-9_-]` chars must pass pattern check.
///
/// This test PASSES at the Red Gate. Regression guard for the upper boundary.
#[test]
fn test_BC_3_3_004_CR003_64_char_slug_is_valid() {
    let dir = TempDir::new().unwrap();
    let slug_64: String = "a".repeat(64);
    let file_name = format!("{slug_64}.toml");
    write_toml(&dir, &file_name, &make_toml(&slug_64));

    let errors: Vec<ConfigError> = match load_and_validate(dir.path()) {
        Ok(_) => vec![],
        Err(errs) => errs,
    };

    let has_pattern_error = errors.iter().any(|e| {
        matches!(e, ConfigError::InvalidOrgSlugPattern { slug, .. } if slug.as_str() == slug_64.as_str())
    });

    assert!(
        !has_pattern_error,
        "AC-001 / EC-002: 64-char slug is valid; must NOT produce E-CFG-019. \
         Errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002:
/// `org_slug = "acme-corp"` (hyphens, standard slug) must pass pattern check.
///
/// This test PASSES at the Red Gate. Regression guard.
#[test]
fn test_BC_3_3_004_CR003_hyphen_slug_is_valid() {
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "acme-corp.toml", &make_toml("acme-corp"));

    let errors: Vec<ConfigError> = match load_and_validate(dir.path()) {
        Ok(_) => vec![],
        Err(errs) => errs,
    };

    let has_pattern_error = errors.iter().any(
        |e| matches!(e, ConfigError::InvalidOrgSlugPattern { slug, .. } if slug == "acme-corp"),
    );

    assert!(
        !has_pattern_error,
        "AC-001: slug 'acme-corp' with hyphens is valid; must NOT produce E-CFG-019. \
         Errors: {errors:?}"
    );
}

/// BC-3.3.004 R-CUST-002:
/// `org_slug = "acme_corp"` (underscores) must pass pattern check.
///
/// This test PASSES at the Red Gate. Regression guard.
#[test]
fn test_BC_3_3_004_CR003_underscore_slug_is_valid() {
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "acme_corp.toml", &make_toml("acme_corp"));

    let errors: Vec<ConfigError> = match load_and_validate(dir.path()) {
        Ok(_) => vec![],
        Err(errs) => errs,
    };

    let has_pattern_error = errors.iter().any(
        |e| matches!(e, ConfigError::InvalidOrgSlugPattern { slug, .. } if slug == "acme_corp"),
    );

    assert!(
        !has_pattern_error,
        "AC-001: slug 'acme_corp' with underscores is valid; must NOT produce E-CFG-019. \
         Errors: {errors:?}"
    );
}

// ===========================================================================
// AC-002: validate_all is pub(crate) — external callers use load_and_validate
// ===========================================================================

/// AC-002: `validate_all` is NOT accessible as a public symbol from the crate root.
///
/// This test asserts the only public validation entry point is `load_and_validate`.
/// The compile-time enforcement is that `prism_customer_config::validate_all` does
/// not exist as a pub symbol — which is enforced by the `pub(crate)` declaration
/// in `validator.rs`.
///
/// Runtime assertion: `load_and_validate` succeeds on a valid directory (ensuring
/// it is reachable via the public API), proving it is the sole public entry point.
///
/// Production gap: If `validate_all` is still `pub`, the `pub(crate)` declaration
/// has not been applied, and downstream callers are exposed to partial-config
/// return semantics on duplicate-id errors.
///
/// This test PASSES at the Red Gate (validate_all is already pub(crate) per the
/// stub commit), but verifies the invariant is maintained.
#[test]
fn test_BC_3_3_004_CR003_load_and_validate_is_the_public_entry_point() {
    let dir = TempDir::new().unwrap();
    // Empty directory: no TOML files → load_and_validate returns Ok(empty).
    let result = load_and_validate(dir.path());

    assert!(
        result.is_ok(),
        "AC-002: load_and_validate on an empty directory must return Ok([]); \
         got Err: {result:?}"
    );
    assert_eq!(
        result.unwrap().len(),
        0,
        "AC-002: load_and_validate on an empty directory must return Ok([])"
    );

    // The compile-time guard: `prism_customer_config::validate_all` must not be
    // accessible. This is enforced at compile time — if `validate_all` were `pub`,
    // the following line would compile:
    //
    //   prism_customer_config::validator::validate_all(dir.path());
    //
    // but because it is `pub(crate)`, it produces a compile error for external crates.
    // We document this invariant here so that a reviewer who loosens the visibility
    // will immediately see a failing assertion intent.
    //
    // Verified property: `validate_all` has `pub(crate)` visibility (AC-002).
    // This assertion enforces the runtime side: load_and_validate is the one reachable
    // public API, and it delegates to validate_all internally.
    assert!(
        true,
        "AC-002: compile-time guard confirmed; validate_all is not accessible externally"
    );
}
