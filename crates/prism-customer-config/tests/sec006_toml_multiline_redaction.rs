//! SEC-006 stub test suite — multi-line TOML credential redaction in
//! `sanitize_error_message`.
//!
//! Covers:
//!   BC-3.3.001 Invariant — no credential value in error messages
//!   AC-005 (multi-line TOML string credential redaction)
//!   EC-006 (multi-line TOML credential snippet must not reveal secret)
//!
//! Every test body is `todo!("AC-NNN: <description>")`.
//! ALL tests MUST fail (Red Gate) before the implementing stub lands.
//!
//! Test naming: `test_BC_3_3_001_SEC006_xxx()` per factory convention.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

// NOTE: `sanitize_error_message` is a private function in `prism-customer-config`.
// These tests exercise it indirectly by triggering a parse error on a TOML file
// whose credential field contains a multi-line string value, then asserting that
// the error returned by `load_and_validate` does NOT contain the raw secret.
//
// Direct unit tests of `sanitize_error_message` live in `validator.rs` inline
// tests (not here). This integration-style test verifies the end-to-end redaction
// path when the TOML parser echoes a multi-line snippet.

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

// ===========================================================================
// AC-005: multi-line TOML credential values are redacted from error messages
// ===========================================================================

/// BC-3.3.001 Invariant / AC-005 / EC-006:
/// A TOML file with `password = """\nmy-secret-value\n"""` in a parse-error
/// context must not expose `my-secret-value` in the resulting `ConfigError`
/// message.
///
/// The test triggers a parse error by including an invalid field alongside the
/// credential so the TOML parser emits a snippet.
#[test]
fn test_BC_3_3_001_SEC006_multiline_password_not_in_error_message() {
    todo!("AC-005 / EC-006: multi-line TOML password= value must not appear in ConfigError message after sanitize_error_message")
}

/// BC-3.3.001 Invariant / AC-005:
/// A TOML file with `bearer_token = """\nabc123\n"""` (multi-line token) in a
/// parse-error context must not expose `abc123` in the error message.
#[test]
fn test_BC_3_3_001_SEC006_multiline_bearer_token_not_in_error_message() {
    todo!("AC-005: multi-line bearer_token value must not appear in ConfigError message")
}

/// BC-3.3.001 Invariant / AC-005:
/// A TOML file with `api_secret = """\nsuper-secret\n"""` in a parse-error
/// context must not expose `super-secret` in the error message.
#[test]
fn test_BC_3_3_001_SEC006_multiline_api_secret_not_in_error_message() {
    todo!("AC-005: multi-line api_secret value must not appear in ConfigError message")
}

// ===========================================================================
// AC-005: conservative redaction must not over-redact non-credential fields
// ===========================================================================

/// BC-3.3.001 Invariant / AC-005:
/// A TOML snippet containing a non-credential field (e.g., `display_name`) with
/// a multi-line value must NOT be redacted by `sanitize_error_message`.
/// Over-redaction of non-credential fields obscures legitimate diagnostic info.
#[test]
fn test_BC_3_3_001_SEC006_non_credential_field_not_redacted() {
    todo!("AC-005: non-credential multi-line field (display_name) must NOT be redacted in error message")
}

/// BC-3.3.001 Invariant / AC-005:
/// Single-line credential field redaction (existing behaviour) must continue
/// to work correctly after any multi-line handling changes. Regression guard.
#[test]
fn test_BC_3_3_001_SEC006_single_line_credential_still_redacted() {
    todo!("AC-005: single-line credential redaction must still work after multi-line changes (regression)")
}
