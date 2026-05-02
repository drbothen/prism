//! CR-003 stub test suite — OrgSlug pattern validation in `validate_structural`.
//!
//! Covers:
//!   BC-3.3.004 R-CUST-002 (slug pattern check added after filename-stem check)
//!   AC-001 (E-CFG-019: InvalidOrgSlugPattern)
//!   AC-002 (`validate_all` made `pub(crate)`)
//!
//! Every test body is `todo!("AC-NNN: <description>")`.
//! ALL tests MUST fail (Red Gate) before the implementing stub lands.
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

// ===========================================================================
// AC-001: E-CFG-019 is emitted for invalid org_slug patterns
// ===========================================================================

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "my org"` (contains a space) must produce E-CFG-019.
///
/// This test uses the canonical AC-001 example from the story spec.
#[test]
fn test_BC_3_3_004_CR003_space_in_slug_produces_e_cfg_019() {
    todo!("AC-001: org_slug='my org' (space) must produce ConfigError::InvalidOrgSlugPattern (E-CFG-019)")
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "acmé"` (contains Unicode 'é', EC-003) must produce E-CFG-019.
#[test]
fn test_BC_3_3_004_CR003_unicode_in_slug_produces_e_cfg_019() {
    todo!("AC-001 / EC-003: org_slug with Unicode character must produce ConfigError::InvalidOrgSlugPattern (E-CFG-019)")
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = "my.org"` (contains dot, not in `[a-zA-Z0-9_-]`) must produce E-CFG-019.
#[test]
fn test_BC_3_3_004_CR003_dot_in_slug_produces_e_cfg_019() {
    todo!("AC-001: org_slug containing '.' must produce ConfigError::InvalidOrgSlugPattern (E-CFG-019)")
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug` longer than 64 characters must produce E-CFG-019.
#[test]
fn test_BC_3_3_004_CR003_slug_too_long_produces_e_cfg_019() {
    todo!("AC-001: org_slug longer than 64 characters must produce ConfigError::InvalidOrgSlugPattern (E-CFG-019)")
}

/// BC-3.3.004 R-CUST-002 / AC-001:
/// `org_slug = ""` (empty) is already covered by the MissingField check, but
/// the slug pattern check must not panic or double-emit for the empty case.
/// Empty slug emits E-CFG-001 (MissingField); E-CFG-019 must NOT be emitted
/// (pattern check is only applied to non-empty slugs).
#[test]
fn test_BC_3_3_004_CR003_empty_slug_produces_missing_field_not_e_cfg_019() {
    todo!("AC-001: empty org_slug must produce MissingField (E-CFG-001), not E-CFG-019; no double-emit")
}

// ===========================================================================
// AC-001: valid slugs — E-CFG-019 must NOT be emitted
// ===========================================================================

/// BC-3.3.004 R-CUST-002 / EC-001:
/// `org_slug = "a"` (single character, valid pattern) must not produce E-CFG-019.
#[test]
fn test_BC_3_3_004_CR003_single_char_slug_is_valid() {
    todo!("AC-001 / EC-001: single-char org_slug must pass pattern check; no E-CFG-019 emitted")
}

/// BC-3.3.004 R-CUST-002 / EC-002:
/// `org_slug` exactly 64 chars with `[a-zA-Z0-9_-]` chars must pass pattern check.
#[test]
fn test_BC_3_3_004_CR003_64_char_slug_is_valid() {
    todo!("AC-001 / EC-002: 64-char slug with valid charset must pass pattern check; no E-CFG-019")
}

/// BC-3.3.004 R-CUST-002:
/// `org_slug = "acme-corp"` (hyphens, standard slug) must pass pattern check.
#[test]
fn test_BC_3_3_004_CR003_hyphen_slug_is_valid() {
    todo!("AC-001: org_slug with hyphens must pass pattern check; no E-CFG-019 emitted")
}

/// BC-3.3.004 R-CUST-002:
/// `org_slug = "acme_corp"` (underscores) must pass pattern check.
#[test]
fn test_BC_3_3_004_CR003_underscore_slug_is_valid() {
    todo!("AC-001: org_slug with underscores must pass pattern check; no E-CFG-019 emitted")
}

// ===========================================================================
// AC-002: validate_all is pub(crate) — external callers use load_and_validate
// ===========================================================================

/// AC-002: `validate_all` is NOT accessible as a public symbol from the crate root.
///
/// This test is a COMPILE-TIME guard: if `validate_all` is still `pub`, downstream
/// crates can call it directly and receive partial configs on duplicate-id error.
/// `load_and_validate` is the only public entry point.
///
/// Because this is a compile-time property, the body asserts it via the public API.
/// The real enforcement is that `prism_customer_config::validate_all` must not compile
/// from external crates.
#[test]
fn test_BC_3_3_004_CR003_load_and_validate_is_the_public_entry_point() {
    todo!("AC-002: load_and_validate must be the only public validation API; validate_all must be pub(crate)")
}
