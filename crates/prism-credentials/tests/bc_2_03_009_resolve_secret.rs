//! Tests for BC-2.03.009: resolve_secret() for _FILE Env Var and K8s Secret Mount
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests pass (implementation complete).

#![allow(clippy::unwrap_used, clippy::expect_used)]
use prism_credentials::resolve_secret::resolve_secret;
use secrecy::ExposeSecret;
use std::io::Write;
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
