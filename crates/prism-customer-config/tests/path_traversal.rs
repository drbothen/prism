//! Path traversal regression tests for W3-FIX-SEC-003.
//!
//! Covers:
//!   AC-001 — `..` traversal rejected with E-CFG-018 (BC-3.3.004 postcondition on failure)
//!   AC-002 — Absolute paths rejected with E-CFG-018 (BC-3.3.004 postcondition on failure)
//!   AC-003 — Relative paths within the tree pass (BC-3.3.004 postcondition on success)
//!   AC-004 — Symlink escaping rejected with E-CFG-018 (BC-3.3.004 postcondition on failure)
//!
//! Red Gate phase 2: all tests MUST FAIL with real assertion errors before implementation
//! lands.  The stub validate_spec_path returns Ok(PathBuf::new()) unconditionally; rejection
//! tests fail because they get Ok instead of Err(SpecPathTraversal), and acceptance tests
//! fail because the returned PathBuf is empty (not the expected canonical path).
//!
//! Test naming convention: `test_BC_S_SS_NNN_xxx` per VSDD TDD protocol.
//! BC-3.3.004 = BC-3.3.004 → S=3, SS=3 (using section 3.3), NNN=004.
//! AC-NNN prefix indicates the acceptance criterion being exercised.

use prism_customer_config::error::ConfigError;
use prism_customer_config::load_and_validate;
use prism_customer_config::validator::validate_spec_path;
use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: build a minimal on-disk customers directory structure for tests.
//
// Layout:
//   <tmp>/
//     customers/
//       acme.toml          ← synthetic "config file" (the config_path argument)
//       sensors/
//         claroty.toml     ← valid spec file within the tree
// ---------------------------------------------------------------------------
fn setup_tree() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tmp dir");
    let customers = tmp.path().join("customers");
    let sensors = customers.join("sensors");
    fs::create_dir_all(&sensors).expect("create sensors dir");
    // Write a dummy valid spec file.
    fs::write(sensors.join("claroty.toml"), b"# stub spec").expect("write claroty.toml");
    // The "config file" path (does not need to exist on disk for path checks).
    let config_path = customers.join("acme.toml");
    (tmp, config_path)
}

// ---------------------------------------------------------------------------
// AC-001: `..` traversal rejected with E-CFG-018
//
// Traces to BC-3.3.004 postcondition on failure: validator must emit E-CFG-018
// and the error message must reference the spec path.
//
// EC-001 canonical vector: spec = "../../../../etc/passwd"
// Pre-join `..` check must fire before any filesystem I/O.
// ---------------------------------------------------------------------------

/// AC-001 / EC-001: dotdot components in spec_path must yield E-CFG-018.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_001_relative_path_traversal_rejected_with_e_cfg_018() {
    let (_tmp, config_path) = setup_tree();
    let result = validate_spec_path(&config_path, "../../../../etc/passwd");
    match result {
        Err(ConfigError::SpecPathTraversal { spec_path, .. }) => {
            assert_eq!(spec_path, "../../../../etc/passwd");
        }
        Ok(_path) => panic!("expected E-CFG-018 error for dotdot traversal, got Ok"),
        Err(other) => panic!("expected E-CFG-018, got: {other}"),
    }
}

/// AC-001 / EC-004: single `..` that stays within the repo tree must still be rejected.
/// The `..` component itself is sufficient for rejection, regardless of canonical destination.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_001_single_dotdot_always_rejected() {
    let (_tmp, config_path) = setup_tree();
    let result = validate_spec_path(&config_path, "../other_customer/sensors/claroty.toml");
    match result {
        Err(ConfigError::SpecPathTraversal { .. }) => {}
        Ok(_path) => panic!("expected E-CFG-018 for `..` component, got Ok"),
        Err(other) => panic!("expected E-CFG-018, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// AC-002: Absolute paths rejected with E-CFG-018
//
// Traces to BC-3.3.004 postcondition on failure.
// Spec paths must always be relative to the config file's parent directory.
// ---------------------------------------------------------------------------

/// AC-002: `/etc/passwd` (Unix absolute path) must yield E-CFG-018.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_002_absolute_path_rejected() {
    let (_tmp, config_path) = setup_tree();
    let result = validate_spec_path(&config_path, "/etc/passwd");
    match result {
        Err(ConfigError::SpecPathTraversal { spec_path, .. }) => {
            assert_eq!(spec_path, "/etc/passwd");
        }
        Ok(_path) => panic!("expected E-CFG-018 for absolute path, got Ok"),
        Err(other) => panic!("expected E-CFG-018, got: {other}"),
    }
}

/// AC-002: A second absolute path (`/tmp/evil.toml` on Unix) must also yield E-CFG-018.
/// On Unix, `Path::is_absolute()` reliably detects `/`-prefixed paths.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_002_absolute_path_root_slash_rejected() {
    let (_tmp, config_path) = setup_tree();
    // Use a clearly absolute path on the current platform.
    let abs = if cfg!(unix) {
        "/tmp/evil.toml"
    } else {
        "C:\\evil.toml"
    };
    let result = validate_spec_path(&config_path, abs);
    // On non-Unix the path might not be detected as absolute; only assert on Unix.
    if cfg!(unix) {
        match result {
            Err(ConfigError::SpecPathTraversal { .. }) => {}
            Ok(_path) => panic!("expected E-CFG-018 for absolute path on Unix, got Ok"),
            Err(other) => panic!("expected E-CFG-018, got: {other}"),
        }
    }
}

// ---------------------------------------------------------------------------
// AC-003: Relative paths within the tree pass
//
// Traces to BC-3.3.004 postcondition on success.
// Existing R-CUST-015 behavior for non-existent files is unchanged.
// ---------------------------------------------------------------------------

/// AC-003 / EC-002: `sensors/claroty.toml` (no `..`, within tree, file exists) passes
/// and returns the canonical (absolute, symlink-resolved) path to the spec file.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_003_relative_within_tree_passes() {
    let (_tmp, config_path) = setup_tree();
    // The sensors/claroty.toml file was created by setup_tree().
    let customers_dir = config_path.parent().unwrap();
    let expected_canonical = customers_dir
        .join("sensors/claroty.toml")
        .canonicalize()
        .expect("sensors/claroty.toml must exist for this test");

    let result = validate_spec_path(&config_path, "sensors/claroty.toml");
    match result {
        Ok(canonical_path) => {
            assert_eq!(
                canonical_path, expected_canonical,
                "validate_spec_path must return the canonical path to the spec file; \
                 got {canonical_path:?}, expected {expected_canonical:?}"
            );
        }
        Err(e) => panic!("expected Ok for valid relative path, got: {e}"),
    }
}

/// AC-003 / EC-002: `./sensors/claroty.toml` (leading `./`, no `..`) also passes
/// and returns the same canonical path as the bare relative form.
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_AC_003_dot_prefix_relative_within_tree_passes() {
    let (_tmp, config_path) = setup_tree();
    let customers_dir = config_path.parent().unwrap();
    let expected_canonical = customers_dir
        .join("sensors/claroty.toml")
        .canonicalize()
        .expect("sensors/claroty.toml must exist for this test");

    let result = validate_spec_path(&config_path, "./sensors/claroty.toml");
    match result {
        Ok(canonical_path) => {
            assert_eq!(
                canonical_path, expected_canonical,
                "validate_spec_path must return the canonical path for `./`-prefixed spec; \
                 got {canonical_path:?}, expected {expected_canonical:?}"
            );
        }
        Err(e) => panic!("expected Ok for `./`-prefixed valid path, got: {e}"),
    }
}

// ---------------------------------------------------------------------------
// AC-004: Symlink escaping rejected (post-join canonicalize boundary check)
//
// Traces to BC-3.3.004 postcondition on failure (EC-003).
// A symlink within the customers dir that points outside must be rejected.
// This test is skipped on platforms that do not support symlinks (e.g., some
// Windows CI environments without the SeCreateSymbolicLinkPrivilege).
// ---------------------------------------------------------------------------

/// AC-004 / EC-003: symlink within the customers dir pointing outside is rejected.
///
/// The pre-join check passes (no `..` in "evil_link.toml"); the post-join
/// `canonicalize()` resolves the symlink to `/etc/hosts`, whose canonical path
/// does not start with the canonical customers directory → E-CFG-018.
#[test]
#[allow(non_snake_case)]
#[cfg(unix)]
fn test_BC_3_3_004_AC_004_symlink_escape_rejected() {
    let (_tmp, config_path) = setup_tree();
    let customers = config_path.parent().unwrap();

    // Create a symlink: customers/evil_link.toml → /etc/hosts.
    // /etc/hosts exists on both Linux and macOS.
    let link_path = customers.join("evil_link.toml");
    std::os::unix::fs::symlink("/etc/hosts", &link_path).expect("create symlink for AC-004 test");

    let result = validate_spec_path(&config_path, "evil_link.toml");
    match result {
        Err(ConfigError::SpecPathTraversal { spec_path, .. }) => {
            assert_eq!(spec_path, "evil_link.toml");
        }
        Ok(_path) => panic!("expected E-CFG-018 for symlink escape, got Ok"),
        Err(other) => panic!("expected E-CFG-018, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// SEC-P2-002: Pre-join traversal check fires BEFORE existence check
//
// BC-3.3.004 CWE-22 invariant; story AC-007
// ---------------------------------------------------------------------------

/// SEC-P2-002: A traversal path targeting a NON-EXISTENT file must emit
/// `SpecPathTraversal` (E-CFG-018), not `SpecFileNotFound` (E-CFG-015).
///
/// Before the SEC-P2-002 fix, `validate_dtu_block` only called `validate_spec_path`
/// when `resolved.exists()` returned `true`. A traversal path like
/// `"../../../../etc/nonexistent"` would bypass the traversal check (file doesn't
/// exist → check skipped) and emit `SpecFileNotFound` instead.
///
/// After the fix, I/O-free pre-join checks (`..` scan, absolute path rejection)
/// fire BEFORE the existence check, so non-existent traversal targets are still
/// caught and emitted as `SpecPathTraversal`.
///
/// This test exercises the full validation pipeline via `validate_all` (not
/// `validate_spec_path` directly), which exercises the `validate_dtu_block` code path.
///
/// (BC-3.3.004 CWE-22 invariant; SEC-P2-002; story AC-007)
#[test]
#[allow(non_snake_case)]
fn test_BC_3_3_004_SEC_P2_002_traversal_nonexistent_target_still_logs_E_CFG_018() {
    let tmp = TempDir::new().expect("tmp dir");
    let customers_dir = tmp.path();

    // Write a minimal customer config file with a traversal path as the spec.
    // The target `../../../../etc/nonexistent` does NOT exist on disk.
    let config_toml = r#"
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "traversal-test"
display_name = "Traversal Test Org"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "keyring://prism/test/claroty"
spec = "../../../../etc/nonexistent"
"#;
    let config_path = customers_dir.join("traversal_test.toml");
    fs::write(&config_path, config_toml).expect("write traversal test config");

    let errors = load_and_validate(customers_dir).err().unwrap_or_default();

    // Must produce at least one error.
    assert!(
        !errors.is_empty(),
        "load_and_validate must produce an error for traversal to non-existent target (SEC-P2-002)"
    );

    // The error MUST be SpecPathTraversal (E-CFG-018), NOT SpecFileNotFound (E-CFG-015).
    let has_traversal_error = errors.iter().any(|e| {
        matches!(
            e,
            ConfigError::SpecPathTraversal { spec_path, .. }
            if spec_path.contains("nonexistent") || spec_path.contains("..")
        )
    });

    let has_not_found_error = errors
        .iter()
        .any(|e| matches!(e, ConfigError::SpecFileNotFound { .. }));

    assert!(
        has_traversal_error,
        "load_and_validate must emit SpecPathTraversal (E-CFG-018) for traversal to \
         non-existent target; errors found: {:?} (BC-3.3.004 CWE-22; SEC-P2-002 AC-007)",
        errors
    );

    assert!(
        !has_not_found_error,
        "load_and_validate must NOT emit SpecFileNotFound (E-CFG-015) when traversal \
         is detected — traversal check must fire before existence check; \
         errors found: {:?} (BC-3.3.004 CWE-22; SEC-P2-002 AC-007)",
        errors
    );
}

/// SEC-P2-002 (absolute path variant): An absolute path targeting a non-existent
/// file must also emit `SpecPathTraversal` (E-CFG-018), not `SpecFileNotFound`.
///
/// (BC-3.3.004 CWE-22 invariant; SEC-P2-002 AC-007)
#[test]
#[allow(non_snake_case)]
#[cfg(unix)]
fn test_BC_3_3_004_SEC_P2_002_absolute_path_nonexistent_target_emits_E_CFG_018() {
    let tmp = TempDir::new().expect("tmp dir");
    let customers_dir = tmp.path();

    let config_toml = r#"
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000002"
org_slug = "abs-path-test"
display_name = "Absolute Path Test"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "keyring://prism/test/claroty"
spec = "/etc/prism-nonexistent-test-file-99999.toml"
"#;
    let config_path = customers_dir.join("abs_path_test.toml");
    fs::write(&config_path, config_toml).expect("write absolute path test config");

    let errors = load_and_validate(customers_dir).err().unwrap_or_default();

    assert!(
        !errors.is_empty(),
        "load_and_validate must produce an error for absolute path to non-existent target (SEC-P2-002)"
    );

    let has_traversal_error = errors
        .iter()
        .any(|e| matches!(e, ConfigError::SpecPathTraversal { .. }));

    assert!(
        has_traversal_error,
        "load_and_validate must emit SpecPathTraversal (E-CFG-018) for absolute path \
         spec even when the target does not exist; errors: {:?} \
         (BC-3.3.004 CWE-22; SEC-P2-002 AC-007)",
        errors
    );
}
