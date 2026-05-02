//! SEC-P3-001 test suite — inline-table TOML credential redaction bypass.
//!
//! Covers:
//!   BC-3.3.004 postcondition 2: all validation errors written to stderr;
//!     credential values must not appear in error output.
//!   AC-001 (SEC-P3-001): inline TOML table credential values are redacted.
//!   AC-002 (SEC-P3-001): nested credential in array table section is redacted.
//!   AC-003 (SEC-P3-001): non-credential inline field values remain visible.
//!
//! ## Root cause
//!
//! `sanitize_error_message` extracts only the leading field name before the
//! first ` = ` on each TOML snippet content line and checks
//! `is_credential_pattern` on that name.  For an inline-table line such as:
//!
//!   3 | credentials = { bearer_token = "my-secret", display_name = "ACME" }
//!
//! The extracted field name is `credentials`, which carries no credential suffix
//! and therefore passes the pattern check unredacted.  The inner `bearer_token`
//! is never examined.
//!
//! ## Red Gate
//!
//! `test_AC_001_inline_table_credentials_redacted` and
//! `test_AC_002_nested_credentials_in_array_table_redacted` must FAIL
//! (assertion error: secret value found in ConfigError message) until the
//! multi-position ` = ` scan loop is implemented in `sanitize_error_message`.
//!
//! `test_AC_003_non_credential_inline_field_visible` must PASS at the Red Gate —
//! it guards against over-redaction; non-credential fields must remain visible.
//!
//! ## Approach
//!
//! `sanitize_error_message` is private.  Tests exercise it end-to-end via
//! `load_and_validate` using TOML fixtures that:
//!   1. Contain inline-table credential fields with known secret values, AND
//!   2. Include a deliberately invalid field to force a TOML parse error that
//!      echoes a snippet containing the inline table in the error message.
//!
//! Test naming: `test_AC_001_*`, `test_AC_002_*`, `test_AC_003_*` per story AC IDs.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use tempfile::TempDir;

use prism_customer_config::{load_and_validate, ConfigError};

// ---------------------------------------------------------------------------
// Helper: write a named TOML file into a TempDir.
// ---------------------------------------------------------------------------

fn write_toml(dir: &TempDir, name: &str, contents: &str) {
    let path = dir.path().join(name);
    fs::write(&path, contents).expect("write toml fixture");
}

// ---------------------------------------------------------------------------
// Helper: collect all error Display strings from a ConfigError Vec.
// ---------------------------------------------------------------------------

fn error_messages(errors: &[ConfigError]) -> Vec<String> {
    errors.iter().map(|e| e.to_string()).collect()
}

// ===========================================================================
// AC-001: inline-table credential values are redacted
// ===========================================================================

/// BC-3.3.004 postcondition 2 / AC-001 / EC-001 (SEC-P3-001):
///
/// A TOML snippet line containing `credentials = { bearer_token = "my-secret" }`
/// must NOT expose the literal value `my-secret` in the resulting `ConfigError`
/// display string.
///
/// ## Production gap (Red Gate — MUST FAIL before fix)
///
/// The current `sanitize_error_message` extracts `credentials` as the leading
/// field name and calls `is_credential_pattern("credentials")` which returns
/// `false` (no `_token`/`_secret`/`_key`/`_password`/`_pass` suffix).  The
/// inner `bearer_token = "my-secret"` is never inspected.  The snippet line is
/// emitted verbatim, leaking `my-secret` into the error message.
///
/// After the fix, the multi-position scan loop detects `bearer_token` and
/// redacts the entire content line before it appears in the error message.
#[test]
fn test_AC_001_inline_table_credentials_redacted() {
    let dir = TempDir::new().unwrap();
    // This TOML uses an inline table that wraps a known secret under a
    // field name (`credentials`) without a credential suffix.
    // The `!invalid` line forces the TOML parser to echo a snippet.
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = "ACME Corp"

[sensor_auth]
credentials = { bearer_token = "my-secret-value", display_name = "ACME" }

# Force a TOML parse error so the parser echoes the snippet above.
!invalid
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    // At least one error must have been produced.
    assert!(
        !msgs.is_empty(),
        "AC-001: expected at least one ConfigError, got none"
    );

    // The raw secret must NOT appear in any error message.
    for msg in &msgs {
        assert!(
            !msg.contains("my-secret-value"),
            "AC-001 (SEC-P3-001): ConfigError must not contain the inline-table \
             bearer_token value 'my-secret-value'. \
             Full message: {msg}"
        );
    }
}

/// BC-3.3.004 postcondition 2 / AC-001 / EC-010 (SEC-P3-001):
///
/// When an inline table contains MULTIPLE credential fields
/// (`api_key = "a"`, `api_secret = "b"`), the entire content line must be
/// redacted (both secrets hidden) once the FIRST matching token is found.
///
/// ## Production gap (Red Gate — MUST FAIL before fix)
///
/// Current code does not scan inner ` = ` positions; both `api_key` and
/// `api_secret` values leak.  After the fix, matching `api_key` (the first
/// credential pattern token) triggers full-line redaction, hiding both values.
#[test]
fn test_AC_001_inline_table_multiple_credential_fields_both_redacted() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000002"
org_slug = "corp"
display_name = "Corp"

[integration]
creds = { api_key = "key-alpha", api_secret = "secret-beta" }

# Force parse error.
!invalid
"#;
    write_toml(&dir, "corp.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-001 multi-cred: expected at least one ConfigError, got none"
    );

    for msg in &msgs {
        assert!(
            !msg.contains("key-alpha"),
            "AC-001 (EC-010): 'key-alpha' (api_key value in inline table) must be \
             redacted. Full message: {msg}"
        );
        assert!(
            !msg.contains("secret-beta"),
            "AC-001 (EC-010): 'secret-beta' (api_secret value in inline table) must be \
             redacted. Full message: {msg}"
        );
    }
}

// ===========================================================================
// AC-002: nested credential in array-table section is redacted
// ===========================================================================

/// BC-3.3.004 postcondition 2 / AC-002 / EC-001 (SEC-P3-001):
///
/// A TOML snippet line inside an `[[array_table]]` section that contains an
/// inline-table credential assignment (e.g.
/// `config = { api_password = "pass123" }`) must be fully redacted.
///
/// ## Production gap (Red Gate — MUST FAIL before fix)
///
/// Same bypass as AC-001: leading field `config` has no credential suffix, so
/// the inner `api_password` is not detected and its value leaks.
#[test]
fn test_AC_002_nested_credentials_in_array_table_redacted() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000003"
org_slug = "beta"
display_name = "Beta Org"

[[dtu]]
type = "claroty"
config = { api_password = "pass123-secret", timeout_secs = 30 }

# Force parse error.
!invalid
"#;
    write_toml(&dir, "beta.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-002: expected at least one ConfigError, got none"
    );

    for msg in &msgs {
        assert!(
            !msg.contains("pass123-secret"),
            "AC-002 (SEC-P3-001): ConfigError must not contain the nested \
             api_password value 'pass123-secret' from inline table inside an \
             array table. Full message: {msg}"
        );
    }
}

// ===========================================================================
// AC-003: non-credential inline field values remain visible (no over-redaction)
// ===========================================================================

/// BC-3.3.004 postcondition 2 / AC-003 (SEC-P3-001):
///
/// A TOML snippet line containing only non-credential inline-table fields
/// (`settings = { display_name = "ACME", timeout_seconds = 30 }`) must NOT be
/// redacted.  Over-redaction degrades the usefulness of TOML parse error
/// diagnostics.
///
/// ## Red Gate (MUST PASS at Red Gate — guards against over-redaction)
///
/// The current code already leaves `display_name` unredacted because it sees
/// `settings` as the leading field and correctly does not match.  After the
/// fix, the multi-position scan must likewise find no credential-pattern token
/// among `display_name` and `timeout_seconds`, leaving the line visible.
///
/// This test is a regression guard: if the implementation blindly redacts all
/// lines containing ` = `, it will fail this assertion.
#[test]
fn test_AC_003_non_credential_inline_field_visible() {
    let dir = TempDir::new().unwrap();
    // `display_name` and `timeout_seconds` are not credential fields.
    // Their values must remain visible in the error message.
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000004"
org_slug = "gamma"
display_name = "Gamma Corp"

[ui_config]
settings = { display_name = "ACME Corp UI", timeout_seconds = 30 }

# Force parse error.
!invalid
"#;
    write_toml(&dir, "gamma.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-003: expected at least one ConfigError, got none"
    );

    // The non-credential display_name value must be visible — it is diagnostic
    // information that must not be over-redacted.
    let any_contains_value = msgs.iter().any(|m| m.contains("ACME Corp UI"));
    assert!(
        any_contains_value,
        "AC-003 (SEC-P3-001): non-credential field 'display_name' inline value \
         'ACME Corp UI' must NOT be redacted — it contains only diagnostic info. \
         Messages: {msgs:?}"
    );
}

/// BC-3.3.004 postcondition 2 / AC-003 (SEC-P3-001):
///
/// A TOML snippet line with a plain non-credential assignment (not inline table)
/// must continue to be visible after the multi-position scan is added.
///
/// Regression guard for the existing single-level non-credential behaviour.
#[test]
fn test_AC_003_single_level_non_credential_not_redacted() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000005"
org_slug = "delta"
display_name = "Delta Org"

[metadata]
org_description = "A well-known ACME subsidiary"

# Force parse error.
!invalid
"#;
    write_toml(&dir, "delta.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-003 regression: expected at least one ConfigError, got none"
    );

    let any_contains_value = msgs
        .iter()
        .any(|m| m.contains("A well-known ACME subsidiary"));
    assert!(
        any_contains_value,
        "AC-003 regression: plain non-credential field value \
         'A well-known ACME subsidiary' must NOT be redacted. \
         Messages: {msgs:?}"
    );
}

/// BC-3.3.004 postcondition 2 / AC-001 (SEC-P3-001):
///
/// Regression: existing single-line credential field redaction
/// (`bearer_token = "classic-secret"`) must continue to work correctly after
/// the inline-table multi-position scan is added.
#[test]
fn test_AC_001_single_line_credential_regression() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000006"
org_slug = "epsilon"
display_name = "Epsilon Corp"

[dtu_auth]
bearer_token = "classic-secret"

# Force parse error.
!invalid
"#;
    write_toml(&dir, "epsilon.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-001 regression: expected at least one ConfigError, got none"
    );

    for msg in &msgs {
        assert!(
            !msg.contains("classic-secret"),
            "AC-001 regression (SEC-P3-001): single-line bearer_token value \
             'classic-secret' must remain redacted after the inline-table scan \
             is added. Full message: {msg}"
        );
    }
}
