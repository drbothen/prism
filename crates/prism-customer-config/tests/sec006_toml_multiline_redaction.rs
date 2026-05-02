//! SEC-006 test suite — multi-line TOML credential redaction in
//! `sanitize_error_message`.
//!
//! Covers:
//!   BC-3.3.001 Invariant — no credential value in error messages
//!   AC-005 (multi-line TOML string credential redaction)
//!   EC-006 (multi-line TOML credential snippet must not reveal secret)
//!
//! ALL tests must fail (assertion errors) before the implementing stub lands.
//!
//! ## Approach
//!
//! `sanitize_error_message` is private; these tests exercise it end-to-end via
//! `load_and_validate`. A TOML file that:
//!   1. Uses a triple-quoted multi-line credential field value, AND
//!   2. Contains a deliberately invalid field to force a parse error that echoes
//!      a TOML snippet in the error message
//!
//! ...must return a `ConfigError` whose `Display` string does not contain the
//! raw secret value.
//!
//! ## Production gap
//!
//! `sanitize_error_message` processes snippet lines line-by-line. For a
//! single-line credential (`bearer_token = "abc123"`), the entire assignment
//! appears on one snippet line and is redacted correctly. For a triple-quoted
//! multi-line value, the opening line is `bearer_token = """` (the field
//! assignment without the value), and the secret content appears on SUBSEQUENT
//! lines of the snippet — which are NOT matched by the current redaction logic
//! because they do not start with a `field_name = ` pattern.
//!
//! Test naming: `test_BC_3_3_001_SEC006_xxx()` per factory convention.
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
// AC-005: multi-line TOML credential values are redacted from error messages
// ===========================================================================

/// BC-3.3.001 Invariant / AC-005 / EC-006:
/// A TOML file with `password = """\nmy-secret-value\n"""` in a parse-error
/// context must not expose `my-secret-value` in the resulting `ConfigError`
/// message.
///
/// ## Production gap
///
/// `sanitize_error_message` redacts `password = """` but leaves the value
/// `my-secret-value` on the continuation line unredacted. The TOML parser
/// echoes all lines of a multi-line value in the error snippet; the raw secret
/// therefore leaks into the Display string of the resulting `ConfigError`.
#[test]
fn test_BC_3_3_001_SEC006_multiline_password_not_in_error_message() {
    let dir = TempDir::new().unwrap();
    // This TOML has a triple-quoted multi-line `password` field with a known
    // secret value, plus an intentionally invalid field (`invalid_field!`) to
    // force the TOML parser to emit an error message containing a snippet.
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = "ACME Corp"

# Multi-line credential that MUST be redacted in error messages.
[data_with_credentials]
password = """
my-secret-value
line2-of-secret
"""

# This invalid field forces a TOML parse / deserialization error so the
# parser echoes a snippet containing the multi-line value above.
invalid_field_bang! = true
"#;
    write_toml(&dir, "acme.toml", toml);

    // The file will fail to parse or validate; we need an error.
    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    // The raw secret must not appear in any error message.
    for msg in &msgs {
        assert!(
            !msg.contains("my-secret-value"),
            "AC-005 / EC-006 (SEC-006): ConfigError must not contain the raw multi-line \
             password value 'my-secret-value'. \
             Full message: {msg}"
        );
        assert!(
            !msg.contains("line2-of-secret"),
            "AC-005 / EC-006 (SEC-006): ConfigError must not contain the continuation line \
             'line2-of-secret' from the multi-line password. \
             Full message: {msg}"
        );
    }
}

/// BC-3.3.001 Invariant / AC-005:
/// A TOML file with `bearer_token = """\nabc123\n"""` (multi-line token) in a
/// parse-error context must not expose `abc123` in the error message.
///
/// ## Production gap
///
/// Same as above: the `abc123` continuation line is not matched by the current
/// per-line credential pattern check and leaks into the error display.
#[test]
fn test_BC_3_3_001_SEC006_multiline_bearer_token_not_in_error_message() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = "ACME Corp"

[dtu_credentials]
bearer_token = """
abc123
my-bearer-secret
"""

# Force a parse error.
[[bad_table
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    for msg in &msgs {
        assert!(
            !msg.contains("abc123"),
            "AC-005 (SEC-006): 'abc123' from multi-line bearer_token must not appear in \
             ConfigError message. Full message: {msg}"
        );
        assert!(
            !msg.contains("my-bearer-secret"),
            "AC-005 (SEC-006): 'my-bearer-secret' from multi-line bearer_token continuation \
             must not appear in ConfigError message. Full message: {msg}"
        );
    }
}

/// BC-3.3.001 Invariant / AC-005:
/// A TOML file with `api_secret = """\nsuper-secret\n"""` in a parse-error
/// context must not expose `super-secret` in the error message.
///
/// ## Production gap
///
/// The `api_secret` field matches the `_secret` suffix credential pattern, so
/// the `api_secret = """` opening line is redacted. But `super-secret` on the
/// next line is passed through unredacted.
#[test]
fn test_BC_3_3_001_SEC006_multiline_api_secret_not_in_error_message() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = "ACME Corp"

[sensor_creds]
api_secret = """
super-secret
second-secret-line
"""

# Force deserialization error with an unknown field.
unknown_top_level = 42
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    for msg in &msgs {
        assert!(
            !msg.contains("super-secret"),
            "AC-005 (SEC-006): 'super-secret' from multi-line api_secret must not appear in \
             ConfigError message. Full message: {msg}"
        );
        assert!(
            !msg.contains("second-secret-line"),
            "AC-005 (SEC-006): 'second-secret-line' from multi-line api_secret continuation \
             must not appear in ConfigError message. Full message: {msg}"
        );
    }
}

// ===========================================================================
// AC-005: conservative redaction must not over-redact non-credential fields
// ===========================================================================

/// BC-3.3.001 Invariant / AC-005:
/// A TOML snippet containing a non-credential field (`display_name`) with a
/// multi-line value must NOT be redacted by `sanitize_error_message`.
/// Over-redaction of non-credential fields obscures legitimate diagnostic info.
///
/// ## Production gap (inverted — this test verifies no over-redaction)
///
/// After the fix, the multi-line redaction logic must be scoped strictly to
/// credential-named fields. Non-credential field values must remain visible.
///
/// This test PASSES at the Red Gate (no over-redaction exists today because
/// multi-line credential redaction is not yet implemented). It acts as a
/// regression guard: the implementation must not blindly redact all multi-line
/// continuation lines.
#[test]
fn test_BC_3_3_001_SEC006_non_credential_field_not_redacted() {
    let dir = TempDir::new().unwrap();
    // `display_name` is not a credential field. Its multi-line value must remain
    // visible in the error message (diagnostic information).
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = """
ACME Corporation
Headquarters: 123 Main St
"""

# Force a parse error.
bad = ===invalid===
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    // At least one error message must be present.
    assert!(
        !msgs.is_empty(),
        "AC-005: expected at least one ConfigError for invalid TOML, got none"
    );

    // The display_name value MUST still appear in the error message (not redacted).
    // We check that the diagnostic content is not stripped.
    // NOTE: "ACME Corporation" is the multi-line display_name value; it is NOT a
    // credential and must not be redacted.
    let any_contains_display_name = msgs.iter().any(|m| m.contains("ACME Corporation"));
    assert!(
        any_contains_display_name,
        "AC-005: non-credential field 'display_name' multi-line value 'ACME Corporation' \
         must NOT be redacted — it contains diagnostic information. \
         Messages: {msgs:?}"
    );
}

/// BC-3.3.001 Invariant / AC-005:
/// Single-line credential field redaction (existing behaviour) must continue
/// to work correctly after any multi-line handling changes. Regression guard.
///
/// ## Why this fails at the Red Gate
///
/// The `password = "single-line-secret"` value appears on a single snippet
/// line. The current code redacts it. After the multi-line fix, the same
/// single-line redaction must still work. This test PASSES at the Red Gate and
/// serves as a regression guard.
#[test]
fn test_BC_3_3_001_SEC006_single_line_credential_still_redacted() {
    let dir = TempDir::new().unwrap();
    // Single-line credential followed by an invalid field to trigger a parse error.
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme"
display_name = "ACME Corp"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "keyring://prism/acme/claroty"
spec = "sensors/claroty.toml"
password = "single-line-secret"

# Force a parse error to make the parser echo the snippet.
!invalid
"#;
    write_toml(&dir, "acme.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    for msg in &msgs {
        assert!(
            !msg.contains("single-line-secret"),
            "AC-005 regression: single-line password value 'single-line-secret' must remain \
             redacted after multi-line changes. Full message: {msg}"
        );
    }
}
