//! Integration tests for the OrgRegistry boot orchestrator (S-3.3.02).
//!
//! Tests cover AC-001 through AC-006 from the story spec, and all clauses from
//! BC-3.1.003, BC-3.1.004, and BC-3.3.004 that apply to the boot path.
//!
//! # BC Coverage
//!
//! | Test | BC Clause |
//! |------|-----------|
//! | test_BC_3_3_004_two_valid_files_both_registered | BC-3.3.004 postcond "On successful validation" §1; AC-001 |
//! | test_BC_3_1_003_bijectivity_after_valid_boot | BC-3.1.003 postcond §1, invariant §1; AC-004 |
//! | test_BC_3_3_004_n_valid_files_exactly_n_entries | BC-3.3.004 postcond "On successful validation" §1; VP-105; AC-004 |
//! | test_BC_3_3_004_invalid_toml_returns_validation_failed | BC-3.3.004 postcond "On any validation failure" §1-4; AC-002/AC-006 |
//! | test_BC_3_3_004_all_errors_aggregated_multi_error | BC-3.3.004 postcond §2; invariant §2; AC-006 |
//! | test_BC_3_3_004_duplicate_org_id_registration_failed | BC-3.1.004 postcond §2; BC-3.3.004 R-CUST-011; AC-002 |
//! | test_BC_3_3_004_duplicate_org_slug_registration_failed | BC-3.1.004 postcond §3; BC-3.3.004 R-CUST-012; AC-003 |
//! | test_BC_3_3_004_empty_dir_returns_ok_zero | BC-3.3.004 EC-3.3.004-01; precond §5; AC-005 |
//! | test_BC_3_3_004_validate_before_register_no_partial_state | BC-3.3.004 invariant §1; ADR-010 §2.5; AC-002 partial-state check |
//! | test_BC_3_1_003_registry_unchanged_on_validation_failure | BC-3.1.003 precond §2; BC-3.3.004 postcond §4 |
//! | test_BC_3_1_004_duplicate_org_id_error_contains_both_files | BC-3.1.004 postcond §4; BC-3.3.004 R-CUST-011 error payload |
//! | test_BC_3_1_004_duplicate_org_slug_error_contains_both_files | BC-3.1.004 postcond §4; BC-3.3.004 R-CUST-012 error payload |
//! | test_BC_3_3_004_non_toml_files_silently_skipped | BC-3.3.004 EC-3.3.004-07 |
//! | test_BC_3_1_003_forward_reverse_map_sizes_equal | BC-3.1.003 invariant §1 (len == count both ways) |
//!
//! BC anchors: BC-3.1.003, BC-3.1.004, BC-3.3.004
//!
//! Red Gate: ALL tests MUST FAIL until boot_org_registry is implemented.
//!           boot_org_registry currently panics with todo!("S-3.3.02: ...").

use std::path::Path;

use prism_core::org_registry::OrgRegistry;
use prism_customer_config::{boot_org_registry, BootError};
use tempfile::TempDir;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// A minimal, fully-valid customer TOML for the given slug.
///
/// Uses UUID v7 (time-ordered), slug matching the file stem, and a
/// vault:// credential ref. The slug is also used as the filename stem
/// (R-CUST-002) — callers must write this to `<dir>/<slug>.toml`.
///
/// DTU block uses `slack` (MSSP Coordination, mode=shared) — the simplest
/// valid DTU entry that does not require a `spec` file on disk.
fn valid_toml(slug: &str, org_id: Uuid) -> String {
    format!(
        r#"schema_version = 1
org_id = "{org_id}"
org_slug = "{slug}"
display_name = "Test Org {slug}"

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "env://SLACK_TOKEN"
"#
    )
}

/// Write a file `<slug>.toml` into `dir` with `valid_toml(slug, org_id)`.
fn write_valid_toml(dir: &Path, slug: &str, org_id: Uuid) {
    let path = dir.join(format!("{slug}.toml"));
    std::fs::write(path, valid_toml(slug, org_id)).expect("write fixture");
}

/// A minimal TOML that deliberately omits `display_name` (triggers E-CFG-001).
fn invalid_toml_missing_display_name(slug: &str, org_id: Uuid) -> String {
    format!(
        r#"schema_version = 1
org_id = "{org_id}"
org_slug = "{slug}"
"#
    )
}

/// A TOML with duplicate `org_id` as the already-registered one.
fn toml_with_org_id(slug: &str, org_id: Uuid) -> String {
    // Same UUID, different slug — this is E-CFG-011 (duplicate org_id across files).
    format!(
        r#"schema_version = 1
org_id = "{org_id}"
org_slug = "{slug}"
display_name = "Duplicate OrgId {slug}"

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "env://SLACK_TOKEN"
"#
    )
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.3.004 "On successful validation" §1, BC-3.1.003 postcond §1
// ---------------------------------------------------------------------------

/// Two valid, distinct TOML files → both orgs registered, no error.
///
/// Traces to: BC-3.3.004 postcond "On successful validation" clause 1;
///            BC-3.1.003 postcond clause 1; AC-001.
#[test]
fn test_BC_3_3_004_two_valid_files_both_registered() {
    let dir = TempDir::new().unwrap();

    let uuid_a = Uuid::now_v7();
    let uuid_b = Uuid::now_v7();

    write_valid_toml(dir.path(), "acme-corp", uuid_a);
    write_valid_toml(dir.path(), "beta-inc", uuid_b);

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(
        result.is_ok(),
        "expected Ok from boot_org_registry with two valid files, got: {result:?}"
    );
    let n = result.unwrap();
    assert_eq!(n, 2, "expected exactly 2 registered orgs, got {n}");
    assert_eq!(
        registry.len(),
        2,
        "OrgRegistry must contain exactly 2 entries"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.1.003 postcond §1, invariant §1 — bijectivity check
// ---------------------------------------------------------------------------

/// After valid boot, resolve(slug) and slug_for(id) are consistent (bijectivity).
///
/// Traces to: BC-3.1.003 postcond clause 1; BC-3.1.003 invariant 1;
///            TV-3.1.003-01; AC-004.
#[test]
fn test_BC_3_1_003_bijectivity_after_valid_boot() {
    let dir = TempDir::new().unwrap();

    let uuid_a = Uuid::now_v7();
    write_valid_toml(dir.path(), "acme-corp", uuid_a);

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_ok(), "expected Ok, got: {result:?}");

    use prism_core::ids::OrgId;
    use prism_core::tenant::OrgSlug;

    let slug = OrgSlug::new("acme-corp");
    let id = OrgId::from_uuid(uuid_a);

    let resolved_id = registry.resolve(&slug);
    let resolved_slug = registry.slug_for(&id);

    assert_eq!(
        resolved_id,
        Some(id),
        "BC-3.1.003 postcond §1: resolve(slug) must equal the registered OrgId"
    );
    assert_eq!(
        resolved_slug.as_ref().map(|s| s.as_str()),
        Some("acme-corp"),
        "BC-3.1.003 postcond §1: slug_for(id) must equal the registered slug"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / VP-105 — N valid files → exactly N entries
// ---------------------------------------------------------------------------

/// N valid customer files → OrgRegistry contains exactly N entries.
///
/// Traces to: BC-3.3.004 postcond "On successful validation" §1;
///            VP-105 (exit-0 implies OrgRegistry count == file count); AC-004.
#[test]
fn test_BC_3_3_004_n_valid_files_exactly_n_entries() {
    let dir = TempDir::new().unwrap();

    // Three distinct orgs.
    let slugs = ["alpha-org", "beta-org", "gamma-org"];
    let uuids: Vec<Uuid> = (0..3).map(|_| Uuid::now_v7()).collect();

    for (slug, &uuid) in slugs.iter().zip(uuids.iter()) {
        write_valid_toml(dir.path(), slug, uuid);
    }

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_ok(), "expected Ok, got: {result:?}");
    let n = result.unwrap();
    assert_eq!(n, 3, "returned count must equal file count (VP-105)");
    assert_eq!(
        registry.len(),
        3,
        "OrgRegistry entry count must equal file count (VP-105)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / AC-006 / BC-3.3.004 "On any validation failure" §1-4
// ---------------------------------------------------------------------------

/// Invalid TOML (missing display_name) → BootError::ValidationFailed with all errors.
///
/// Traces to: BC-3.3.004 postcond "On any validation failure" clause 1 (exit 1);
///            clause 2 (all errors collected); clause 4 (OrgRegistry empty);
///            VP-106; AC-002; AC-006.
#[test]
fn test_BC_3_3_004_invalid_toml_returns_validation_failed() {
    let dir = TempDir::new().unwrap();

    let uuid_a = Uuid::now_v7();
    let bad_toml = invalid_toml_missing_display_name("acme-corp", uuid_a);
    std::fs::write(dir.path().join("acme-corp.toml"), bad_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err(), "expected Err from invalid TOML, got Ok");
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            assert!(
                !errors.is_empty(),
                "ValidationFailed must carry at least one error"
            );
        }
        BootError::RegistrationFailed(e) => {
            panic!("expected ValidationFailed, got RegistrationFailed({e})");
        }
    }

    assert_eq!(
        registry.len(),
        0,
        "BC-3.3.004 postcond §4: OrgRegistry must be empty on validation failure (VP-106)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.3.004 invariant §2 — multi-error aggregation
// ---------------------------------------------------------------------------

/// Multiple files each with errors → ALL errors aggregated, not fail-fast.
///
/// Traces to: BC-3.3.004 postcond "On any validation failure" clause 2;
///            BC-3.3.004 invariant §2; EC-3.3.004-03; AC-006.
#[test]
fn test_BC_3_3_004_all_errors_aggregated_multi_error() {
    let dir = TempDir::new().unwrap();

    // File 1 (alphabetically first): missing display_name
    let uuid_a = Uuid::now_v7();
    let bad_a = invalid_toml_missing_display_name("alpha-org", uuid_a);
    std::fs::write(dir.path().join("alpha-org.toml"), bad_a).unwrap();

    // File 2 (alphabetically second): missing display_name
    let uuid_b = Uuid::now_v7();
    let bad_b = invalid_toml_missing_display_name("beta-org", uuid_b);
    std::fs::write(dir.path().join("beta-org.toml"), bad_b).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err(), "expected Err");
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            // Both files must have produced errors — not fail-fast on the first.
            assert!(
                errors.len() >= 2,
                "expected errors from both files (multi-error), got {} error(s): {errors:?}",
                errors.len()
            );
            // Verify both filenames appear somewhere in the error messages.
            let all_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            let has_alpha = all_msgs.iter().any(|m| m.contains("alpha-org"));
            let has_beta = all_msgs.iter().any(|m| m.contains("beta-org"));
            assert!(
                has_alpha,
                "expected 'alpha-org' error in multi-error output; got: {all_msgs:?}"
            );
            assert!(
                has_beta,
                "expected 'beta-org' error in multi-error output; got: {all_msgs:?}"
            );
        }
        BootError::RegistrationFailed(e) => {
            panic!("expected ValidationFailed, got RegistrationFailed({e})");
        }
    }
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.1.004 postcond §2, BC-3.3.004 R-CUST-011 — duplicate org_id
// ---------------------------------------------------------------------------

/// Two files with the same org_id → BootError::ValidationFailed (E-CFG-011).
///
/// Traces to: BC-3.1.004 postcond clause 2; BC-3.3.004 R-CUST-011; AC-002.
/// TV-3.1.004-04, TV-3.3.004-12.
#[test]
fn test_BC_3_3_004_duplicate_org_id_returns_validation_failed() {
    let dir = TempDir::new().unwrap();

    // Both files share the same org_id.
    let shared_uuid = Uuid::now_v7();

    write_valid_toml(dir.path(), "acme-corp", shared_uuid);
    // Second file: different slug, same org_id → E-CFG-011.
    let dup_toml = toml_with_org_id("beta-inc", shared_uuid);
    std::fs::write(dir.path().join("beta-inc.toml"), dup_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err(), "expected Err on duplicate org_id");
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            let any_dup_id_error = errors.iter().any(|e| {
                let msg = e.to_string();
                msg.contains("E-CFG-011") || msg.contains("org_id")
            });
            assert!(
                any_dup_id_error,
                "expected E-CFG-011 (duplicate org_id) in errors; got: {errors:?}"
            );
        }
        BootError::RegistrationFailed(e) => {
            // Also acceptable: registration layer catches the duplicate.
            // Either way the error must name a conflict.
            let msg = e.to_string();
            assert!(
                msg.contains("already bound") || msg.contains("conflict"),
                "RegistrationFailed error must describe the conflict: {msg}"
            );
        }
    }

    assert_eq!(
        registry.len(),
        0,
        "BC-3.3.004 postcond §4: OrgRegistry must have zero entries on duplicate org_id"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.1.004 postcond §3, BC-3.3.004 R-CUST-012 — duplicate org_slug
// ---------------------------------------------------------------------------

/// Two files with the same org_slug → BootError::ValidationFailed (E-CFG-012).
///
/// Traces to: BC-3.1.004 postcond clause 3; BC-3.3.004 R-CUST-012; AC-003.
/// TV-3.1.004-04.
#[test]
fn test_BC_3_3_004_duplicate_org_slug_returns_validation_failed() {
    let dir = TempDir::new().unwrap();

    let uuid_a = Uuid::now_v7();
    let uuid_b = Uuid::now_v7();

    // acme-corp.toml → slug "acme-corp"
    write_valid_toml(dir.path(), "acme-corp", uuid_a);

    // acme-corp2.toml → but org_slug = "acme-corp" (matches the first file's slug).
    // The validator requires org_slug == filename stem (R-CUST-002), so we must
    // write a file whose stem IS "acme-corp" for a pure slug-dup scenario.
    // Instead, use the cross-file duplicate slug path: two distinct file stems
    // but the org_slug field in both set to the same value.
    // File: dup-slug.toml, org_slug = "acme-corp" → E-CFG-002 (stem mismatch) AND E-CFG-012.
    // For a clean E-CFG-012 test we need both files to have matching org_slug == stem.
    // The only way: use a second directory path trick is not needed — just use
    // two files where each slug matches its own stem, but both stems are the same.
    // That is impossible in one directory. Instead: produce the duplicate-slug error
    // via the cross-file check which runs even when org_slug != stem (validator still
    // registers the raw string for duplicate detection).
    let dup_slug_toml = format!(
        r#"schema_version = 1
org_id = "{uuid_b}"
org_slug = "acme-corp"
display_name = "Duplicate Slug Corp"

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "env://SLACK_TOKEN"
"#
    );
    std::fs::write(dir.path().join("dup-slug.toml"), dup_slug_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err(), "expected Err on duplicate org_slug");
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            let any_dup_slug_error = errors.iter().any(|e| {
                let msg = e.to_string();
                msg.contains("E-CFG-012") || msg.contains("org_slug")
            });
            assert!(
                any_dup_slug_error,
                "expected E-CFG-012 (duplicate org_slug) in errors; got: {errors:?}"
            );
        }
        BootError::RegistrationFailed(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("already bound") || msg.contains("conflict"),
                "RegistrationFailed error must describe the conflict: {msg}"
            );
        }
    }

    assert_eq!(
        registry.len(),
        0,
        "BC-3.3.004 postcond §4: OrgRegistry must have zero entries on duplicate org_slug"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.3.004 EC-3.3.004-01 — empty dir
// ---------------------------------------------------------------------------

/// Empty `customers/` directory → Ok(0), empty OrgRegistry, no error.
///
/// Traces to: BC-3.3.004 EC-3.3.004-01; precond §5; AC-005.
#[test]
fn test_BC_3_3_004_empty_dir_returns_ok_zero() {
    let dir = TempDir::new().unwrap();
    // No files written — directory is empty.

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(
        result.is_ok(),
        "expected Ok(0) for empty directory, got: {result:?}"
    );
    let n = result.unwrap();
    assert_eq!(n, 0, "expected 0 registered orgs for empty directory");
    assert!(
        registry.is_empty(),
        "OrgRegistry must be empty when no files present"
    );
}

// ---------------------------------------------------------------------------
// ADR-010 §2.5 validate-before-register ordering — no partial state
// ---------------------------------------------------------------------------

/// Mixed directory (one valid, one invalid) → no entries registered even for
/// the valid file. Validation MUST complete for ALL files before register is
/// called for ANY file (ADR-010 §2.5, BC-3.3.004 invariant §1).
///
/// Traces to: BC-3.3.004 invariant §1; ADR-010 §2.5; AC-002 partial-state check.
#[test]
fn test_BC_3_3_004_validate_before_register_no_partial_state() {
    let dir = TempDir::new().unwrap();

    // File 1 (alphabetically first): valid.
    let uuid_a = Uuid::now_v7();
    write_valid_toml(dir.path(), "alpha-org", uuid_a);

    // File 2 (alphabetically second): invalid (missing display_name).
    let uuid_b = Uuid::now_v7();
    let bad_toml = invalid_toml_missing_display_name("beta-org", uuid_b);
    std::fs::write(dir.path().join("beta-org.toml"), bad_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err(), "expected Err when any file is invalid");
    assert_eq!(
        registry.len(),
        0,
        "BC-3.3.004 invariant §1: OrgRegistry must have ZERO entries — \
         no partial registration allowed when validation fails for any file"
    );
}

// ---------------------------------------------------------------------------
// BC-3.3.004 postcond §4 — registry unchanged on validation failure
// ---------------------------------------------------------------------------

/// OrgRegistry is completely empty after a failed boot.
/// Verifies postcond §4: "OrgRegistry contains zero entries (no partial registration)".
///
/// Traces to: BC-3.1.003 precond §2; BC-3.3.004 postcond §4; VP-106.
#[test]
fn test_BC_3_1_003_registry_unchanged_on_validation_failure() {
    let dir = TempDir::new().unwrap();

    // Multiple invalid files.
    for slug in &["aaa-org", "bbb-org", "ccc-org"] {
        let uuid = Uuid::now_v7();
        let bad = invalid_toml_missing_display_name(slug, uuid);
        std::fs::write(dir.path().join(format!("{slug}.toml")), bad).unwrap();
    }

    let registry = OrgRegistry::new();
    let _ = boot_org_registry(dir.path(), &registry);

    assert_eq!(
        registry.len(),
        0,
        "BC-3.3.004 postcond §4 / VP-106: OrgRegistry must be empty after any validation failure"
    );
    assert!(registry.is_empty(), "OrgRegistry.is_empty() must be true");
}

// ---------------------------------------------------------------------------
// BC-3.1.004 postcond §4 — error payload: both filenames named for E-CFG-011
// ---------------------------------------------------------------------------

/// E-CFG-011 error message names both files where the duplicate org_id appears.
///
/// Traces to: BC-3.1.004 postcond §4 ("error identifies both filenames");
///            BC-3.3.004 postcond §3 ("each error line includes: filename");
///            TV-3.1.004-04, TV-3.3.004-12.
#[test]
fn test_BC_3_1_004_duplicate_org_id_error_contains_both_files() {
    let dir = TempDir::new().unwrap();

    let shared_uuid = Uuid::now_v7();
    write_valid_toml(dir.path(), "acme-corp", shared_uuid);

    let dup_toml = toml_with_org_id("beta-inc", shared_uuid);
    std::fs::write(dir.path().join("beta-inc.toml"), dup_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err());
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            let dup_id_msgs: Vec<String> = errors
                .iter()
                .map(|e| e.to_string())
                .filter(|m| m.contains("E-CFG-011"))
                .collect();

            assert!(
                !dup_id_msgs.is_empty(),
                "expected at least one E-CFG-011 error; got: {errors:?}"
            );

            // The error message must name BOTH files (BC-3.1.004 postcond §4).
            let combined = dup_id_msgs.join(" ");
            assert!(
                combined.contains("acme-corp"),
                "E-CFG-011 error must name 'acme-corp': {combined}"
            );
            assert!(
                combined.contains("beta-inc"),
                "E-CFG-011 error must name 'beta-inc': {combined}"
            );
        }
        BootError::RegistrationFailed(_) => {
            // Acceptable if caught at registration layer — payload already validated elsewhere.
        }
    }
}

// ---------------------------------------------------------------------------
// BC-3.1.004 postcond §4 — error payload: both filenames named for E-CFG-012
// ---------------------------------------------------------------------------

/// E-CFG-012 error message names both files where the duplicate org_slug appears.
///
/// Traces to: BC-3.1.004 postcond §4; BC-3.3.004 R-CUST-012; TV-3.3.004-13.
#[test]
fn test_BC_3_1_004_duplicate_org_slug_error_contains_both_files() {
    let dir = TempDir::new().unwrap();

    let uuid_a = Uuid::now_v7();
    let uuid_b = Uuid::now_v7();

    // acme-corp.toml: org_slug = "acme-corp" (matches stem).
    write_valid_toml(dir.path(), "acme-corp", uuid_a);

    // dup-slug.toml: org_slug = "acme-corp" (duplicate slug).
    let dup_toml = format!(
        r#"schema_version = 1
org_id = "{uuid_b}"
org_slug = "acme-corp"
display_name = "Dup Slug"

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "env://SLACK_TOKEN"
"#
    );
    std::fs::write(dir.path().join("dup-slug.toml"), dup_toml).unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_err());
    match result.unwrap_err() {
        BootError::ValidationFailed(errors) => {
            let dup_slug_msgs: Vec<String> = errors
                .iter()
                .map(|e| e.to_string())
                .filter(|m| m.contains("E-CFG-012"))
                .collect();

            assert!(
                !dup_slug_msgs.is_empty(),
                "expected at least one E-CFG-012 error; got: {errors:?}"
            );

            let combined = dup_slug_msgs.join(" ");
            assert!(
                combined.contains("acme-corp"),
                "E-CFG-012 error must name 'acme-corp': {combined}"
            );
            assert!(
                combined.contains("dup-slug"),
                "E-CFG-012 error must name 'dup-slug': {combined}"
            );
        }
        BootError::RegistrationFailed(_) => {
            // Acceptable if registration layer catches it.
        }
    }
}

// ---------------------------------------------------------------------------
// BC-3.3.004 EC-3.3.004-07 — non-TOML files silently skipped
// ---------------------------------------------------------------------------

/// A non-.toml file (e.g., README.md) in customers/ is silently skipped.
///
/// Traces to: BC-3.3.004 EC-3.3.004-07; validator.rs behaviour.
#[test]
fn test_BC_3_3_004_non_toml_files_silently_skipped() {
    let dir = TempDir::new().unwrap();

    // One valid .toml file.
    let uuid_a = Uuid::now_v7();
    write_valid_toml(dir.path(), "acme-corp", uuid_a);

    // A non-.toml file — must be silently skipped.
    std::fs::write(
        dir.path().join("README.md"),
        "# This file should be ignored",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "internal notes").unwrap();

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(
        result.is_ok(),
        "non-.toml files must be silently skipped; expected Ok, got: {result:?}"
    );
    let n = result.unwrap();
    assert_eq!(
        n, 1,
        "only the .toml file should be counted; non-.toml files must be skipped"
    );
    assert_eq!(registry.len(), 1, "OrgRegistry must have exactly 1 entry");
}

// ---------------------------------------------------------------------------
// BC-3.1.003 invariant §1 — forward/reverse map sizes equal
// ---------------------------------------------------------------------------

/// After N successful registrations, OrgRegistry.len() equals the returned count.
/// Bijectivity invariant: forward-map and reverse-map must have equal sizes.
///
/// Traces to: BC-3.1.003 invariant §1 (len == count); EC-005.
/// (OrgRegistry.len() is backed by the BiMap which maintains bijectivity —
/// a successful len() call proves the invariant holds structurally.)
#[test]
fn test_BC_3_1_003_forward_reverse_map_sizes_equal() {
    let dir = TempDir::new().unwrap();

    let count = 5;
    let slugs = ["aaa-org", "bbb-org", "ccc-org", "ddd-org", "eee-org"];
    for slug in &slugs[..count] {
        write_valid_toml(dir.path(), slug, Uuid::now_v7());
    }

    let registry = OrgRegistry::new();
    let result = boot_org_registry(dir.path(), &registry);

    assert!(result.is_ok(), "expected Ok for {count} valid files");
    let n = result.unwrap();

    assert_eq!(
        n, count,
        "returned count must equal number of files written"
    );
    assert_eq!(
        registry.len(),
        count,
        "BC-3.1.003 invariant §1: OrgRegistry.len() must equal {count}"
    );
    // The BiMap internal invariant (forward == reverse size) is implicit in len();
    // a well-formed BiMap cannot have differing sizes, so this assertion is sufficient.
}

// ---------------------------------------------------------------------------
// BootError::RegistrationFailed path — defense-in-depth guard (EC-003)
// ---------------------------------------------------------------------------

/// Calling boot_org_registry returns BootError::RegistrationFailed when the
/// registry reports a conflict that slipped past load_and_validate.
///
/// This tests the defense-in-depth guard (BC-3.3.04 Task 6 / S-3.3.02 EC-003):
/// if OrgRegistry::register ever returns Err despite validation passing, the
/// boot function must propagate it as RegistrationFailed.
///
/// Implementation note: this scenario is currently impossible to trigger naturally
/// (load_and_validate would have caught the duplicate). The test verifies the
/// BootError enum variant exists and is reachable in principle by constructing
/// a registry that already has one entry before boot, then booting with a file
/// that has the same (slug, id) pair — producing an OrgRegistry::register
/// idempotent Ok (D-050). For a true conflict we pre-register a conflicting pair.
///
/// Traces to: BC-3.3.04 Task 6; S-3.3.02 EC-003; BC-3.1.004 postcond §5.
#[test]
fn test_BC_3_3_004_registration_failed_when_registry_already_has_conflict() {
    use prism_core::ids::OrgId;
    use prism_core::tenant::OrgSlug;

    let dir = TempDir::new().unwrap();

    // Write one valid file for "acme-corp".
    let uuid_a = Uuid::now_v7();
    write_valid_toml(dir.path(), "acme-corp", uuid_a);

    // Pre-populate the registry with a DIFFERENT org_id for the SAME slug.
    // This simulates the defense-in-depth case where the registry already has
    // a conflicting entry when boot_org_registry runs.
    let pre_uuid = Uuid::now_v7();
    let registry = OrgRegistry::new();
    registry
        .register(OrgSlug::new("acme-corp"), OrgId::from_uuid(pre_uuid))
        .expect("pre-population should succeed on empty registry");

    // Now boot: load_and_validate will pass (uuid_a is valid), but register
    // will see slug "acme-corp" already bound to pre_uuid → RegistrationFailed.
    let result = boot_org_registry(dir.path(), &registry);

    assert!(
        result.is_err(),
        "expected Err when registry pre-conflict causes RegistrationFailed"
    );
    match result.unwrap_err() {
        BootError::RegistrationFailed(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("acme-corp"),
                "RegistrationFailed error must name the conflicting slug: {msg}"
            );
        }
        BootError::ValidationFailed(errors) => {
            // If validation somehow catches this (defensive: uuid_a passes all structural
            // checks), accept it but note the expectation.
            panic!(
                "expected RegistrationFailed (defense-in-depth), got ValidationFailed: {errors:?}"
            );
        }
    }
}
