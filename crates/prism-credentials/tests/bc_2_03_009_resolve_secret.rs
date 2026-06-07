//! Tests for BC-2.03.009: resolve_secret() for _FILE Env Var and K8s Secret Mount
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests pass (implementation complete).

#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::io::Write;

use prism_credentials::resolve_secret::resolve_secret;
use secrecy::ExposeSecret;
use tempfile::NamedTempFile;

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-001: _FILE env var set — reads file, strips trailing newline
// ---------------------------------------------------------------------------

/// BC-2.03.009 postcondition: `{NAME}_FILE` set → reads file, strips trailing newline.
#[test]
fn test_BC_2_03_009_file_env_var_reads_file_and_strips_newline() {
    // Write "abc\n" to a temp file
    let mut tmp = NamedTempFile::new().expect("tempfile");
    writeln!(tmp, "abc").expect("write"); // writes "abc\n"

    let file_path = tmp.path().to_str().unwrap().to_string();

    // Set env var to file path
    std::env::set_var("PRISM_TEST_KEY_FILE_001", &file_path);
    std::env::remove_var("PRISM_TEST_KEY_001");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_001", "PRISM_TEST_KEY_001");

    std::env::remove_var("PRISM_TEST_KEY_FILE_001");

    assert!(
        result.is_ok(),
        "resolve_secret should succeed, got: {result:?}"
    );
    let secret = result.unwrap();
    assert!(secret.is_some(), "should return Some(SecretString)");
    assert_eq!(
        secret.unwrap().expose_secret(),
        "abc",
        "trailing newline must be stripped"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-002: only direct env var set — uses it directly
// ---------------------------------------------------------------------------

/// BC-2.03.009 postcondition: `{NAME}` set (no _FILE) → returns that value.
#[test]
fn test_BC_2_03_009_direct_env_var_used_when_no_file_env() {
    std::env::remove_var("PRISM_TEST_KEY_FILE_002");
    std::env::set_var("PRISM_TEST_KEY_002", "directvalue");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_002", "PRISM_TEST_KEY_002");

    std::env::remove_var("PRISM_TEST_KEY_002");

    assert!(result.is_ok(), "got: {result:?}");
    let secret = result.unwrap();
    assert!(secret.is_some());
    assert_eq!(secret.unwrap().expose_secret(), "directvalue");
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-003: both set — file wins, debug log notes precedence
// ---------------------------------------------------------------------------

/// BC-2.03.009 EC-03-022: when both are set, file takes precedence.
#[test]
fn test_BC_2_03_009_file_wins_when_both_set() {
    let mut tmp = NamedTempFile::new().expect("tempfile");
    writeln!(tmp, "from_file").expect("write");
    let file_path = tmp.path().to_str().unwrap().to_string();

    std::env::set_var("PRISM_TEST_KEY_FILE_003", &file_path);
    std::env::set_var("PRISM_TEST_KEY_003", "direct_value_should_be_ignored");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_003", "PRISM_TEST_KEY_003");

    std::env::remove_var("PRISM_TEST_KEY_FILE_003");
    std::env::remove_var("PRISM_TEST_KEY_003");

    assert!(result.is_ok(), "got: {result:?}");
    let secret = result.unwrap();
    assert!(secret.is_some());
    assert_eq!(
        secret.unwrap().expose_secret(),
        "from_file",
        "file must take precedence over direct env var"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-004: _FILE points to nonexistent file — returns PrismError
// ---------------------------------------------------------------------------

/// BC-2.03.009 error case: nonexistent file path returns PrismError::Credential.
#[test]
fn test_BC_2_03_009_rejects_nonexistent_file_with_credential_error() {
    std::env::set_var(
        "PRISM_TEST_KEY_FILE_004",
        "/nonexistent/path/does-not-exist.txt",
    );
    std::env::remove_var("PRISM_TEST_KEY_004");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_004", "PRISM_TEST_KEY_004");

    std::env::remove_var("PRISM_TEST_KEY_FILE_004");

    assert!(
        result.is_err(),
        "nonexistent file must return Err, got: {result:?}"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("E-CRED") || msg.contains("does not exist") || msg.contains("nonexistent"),
        "error must reference file path and existence, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-005: neither env var set — returns None
// ---------------------------------------------------------------------------

/// BC-2.03.009 postcondition: neither set → returns Ok(None).
#[test]
fn test_BC_2_03_009_neither_set_returns_none() {
    std::env::remove_var("PRISM_TEST_KEY_FILE_005");
    std::env::remove_var("PRISM_TEST_KEY_005");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_005", "PRISM_TEST_KEY_005");

    assert!(
        result.is_ok(),
        "neither set should return Ok, got: {result:?}"
    );
    assert!(result.unwrap().is_none(), "neither set should return None");
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.009-006: _FILE points to a directory — returns PrismError
// ---------------------------------------------------------------------------

/// BC-2.03.009 error case: directory path returns PrismError with regular-file requirement.
#[test]
fn test_BC_2_03_009_rejects_directory_path_with_credential_error() {
    // /tmp always exists and is a directory
    std::env::set_var("PRISM_TEST_KEY_FILE_006", "/tmp");
    std::env::remove_var("PRISM_TEST_KEY_006");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_006", "PRISM_TEST_KEY_006");

    std::env::remove_var("PRISM_TEST_KEY_FILE_006");

    assert!(result.is_err(), "directory path must return Err");
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("directory") || msg.contains("regular file") || msg.contains("E-CRED"),
        "error must mention regular file requirement, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Invariant: resolved value is SecretString (never plain String)
// ---------------------------------------------------------------------------

/// BC-2.03.009 invariant: resolved values are loaded into SecretString.
/// Compile-time enforcement: the return type is `Option<SecretString>`.
#[test]
fn test_BC_2_03_009_invariant_resolved_value_is_secret_string() {
    let _: Result<Option<secrecy::SecretString>, prism_core::PrismError> =
        resolve_secret("PRISM_TEST_KEY_FILE_INV", "PRISM_TEST_KEY_INV");
}

// ---------------------------------------------------------------------------
// EC-03-023: _FILE points to empty file — returns empty secret
// ---------------------------------------------------------------------------

/// BC-2.03.009 EC-03-023: empty file resolves to an empty SecretString.
#[test]
fn test_BC_2_03_009_empty_file_resolves_to_empty_secret() {
    let tmp = NamedTempFile::new().expect("tempfile"); // creates empty file
    let file_path = tmp.path().to_str().unwrap().to_string();

    std::env::set_var("PRISM_TEST_KEY_FILE_007", &file_path);
    std::env::remove_var("PRISM_TEST_KEY_007");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_007", "PRISM_TEST_KEY_007");

    std::env::remove_var("PRISM_TEST_KEY_FILE_007");

    assert!(result.is_ok(), "empty file should succeed, got: {result:?}");
    let secret = result.unwrap();
    assert!(secret.is_some(), "empty file should return Some");
    assert_eq!(
        secret.unwrap().expose_secret(),
        "",
        "empty file should resolve to empty string"
    );
}

// ---------------------------------------------------------------------------
// RG-ECRED-003: file-I/O errors must emit E-CRED-005, NOT E-CRED-009
// (ADR-035 §D1 E-CRED-005; AC-003 + AC-010 of S-MAINT-ECRED-TAXONOMY-SYNC-001)
//
// RED GATE: Currently fails because resolve_secret.rs emits "E-CRED-009: ..."
// PASSES AFTER: implementer changes 3 inline reason strings from "E-CRED-009:" to "E-CRED-005:"
// ---------------------------------------------------------------------------

/// RG-ECRED-003: resolve_secret file-I/O failures emit canonical E-CRED-005, not E-CRED-009.
///
/// Exercises the missing-file path (AC-003 canonical test vector from ADR-035 §Blast-Radius).
/// Also guards that NONE of the three file-I/O sub-paths emit E-CRED-009 after migration.
///
/// Currently fails: resolve_secret.rs emits `"E-CRED-009: credential file does not exist..."`.
/// Passes after: string literal updated to `"E-CRED-005: credential file I/O error for '...':"`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_03_009_resolve_secret_file_io_emits_e_cred_005() {
    // --- Sub-case 1: missing file ---
    let missing_path = "/nonexistent/path/rg-ecred-003-missing.txt";
    std::env::set_var("RG_ECRED_003_FILE_A", missing_path);
    std::env::remove_var("RG_ECRED_003_DIRECT_A");

    let result_missing = resolve_secret("RG_ECRED_003_FILE_A", "RG_ECRED_003_DIRECT_A");

    std::env::remove_var("RG_ECRED_003_FILE_A");

    assert!(
        result_missing.is_err(),
        "missing file must return Err, got: {result_missing:?}"
    );
    let err_missing = result_missing.unwrap_err();
    let msg_missing = err_missing.to_string();

    // Must contain E-CRED-005 (canonical code for file-I/O per ADR-035 §D1)
    assert!(
        msg_missing.contains("E-CRED-005"),
        "missing-file error must contain 'E-CRED-005', got: {msg_missing:?}"
    );
    // Must NOT contain E-CRED-009 (old undeclared code being retired)
    assert!(
        !msg_missing.contains("E-CRED-009"),
        "missing-file error must NOT contain 'E-CRED-009' (old undeclared code), got: {msg_missing:?}"
    );

    // --- Sub-case 2: path is a directory ---
    std::env::set_var("RG_ECRED_003_FILE_B", "/tmp");
    std::env::remove_var("RG_ECRED_003_DIRECT_B");

    let result_dir = resolve_secret("RG_ECRED_003_FILE_B", "RG_ECRED_003_DIRECT_B");

    std::env::remove_var("RG_ECRED_003_FILE_B");

    assert!(
        result_dir.is_err(),
        "directory path must return Err, got: {result_dir:?}"
    );
    let err_dir = result_dir.unwrap_err();
    let msg_dir = err_dir.to_string();

    assert!(
        msg_dir.contains("E-CRED-005"),
        "directory-path error must contain 'E-CRED-005', got: {msg_dir:?}"
    );
    assert!(
        !msg_dir.contains("E-CRED-009"),
        "directory-path error must NOT contain 'E-CRED-009' (old undeclared code), got: {msg_dir:?}"
    );
}

// ---------------------------------------------------------------------------
// EC-03-024: file with multiple lines — only content up to first newline
// ---------------------------------------------------------------------------

/// BC-2.03.009 EC-03-024: trailing newline stripped; result is the content without trailing \n.
/// The BC says "trailing newlines stripped" — only trailing newline behavior is tested.
#[test]
fn test_BC_2_03_009_trailing_newline_stripped_from_file_content() {
    let mut tmp = NamedTempFile::new().expect("tempfile");
    writeln!(tmp, "mysecret").expect("write"); // one line with trailing newline

    let file_path = tmp.path().to_str().unwrap().to_string();
    std::env::set_var("PRISM_TEST_KEY_FILE_008", &file_path);
    std::env::remove_var("PRISM_TEST_KEY_008");

    let result = resolve_secret("PRISM_TEST_KEY_FILE_008", "PRISM_TEST_KEY_008");

    std::env::remove_var("PRISM_TEST_KEY_FILE_008");

    assert!(result.is_ok(), "got: {result:?}");
    let secret = result.unwrap().unwrap();
    assert_eq!(
        secret.expose_secret(),
        "mysecret",
        "trailing newline must be stripped"
    );
}
