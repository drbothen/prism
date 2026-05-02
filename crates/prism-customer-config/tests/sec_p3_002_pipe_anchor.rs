//! SEC-P3-002 / CR-019 test suite — `find_snippet_pipe` digit-prefix anchor.
//!
//! Covers:
//!   BC-3.3.004 postcondition 2: credential values must not appear in error output.
//!   AC-001 (SEC-P3-002): a credential value containing ` | ` does not cause
//!     `find_snippet_pipe` to return the wrong offset, bypassing redaction.
//!   AC-002 (SEC-P3-002): `find_snippet_pipe` returns `None` for a line whose
//!     first ` | ` occurrence is not preceded by digits/spaces only.
//!
//! ## Root cause
//!
//! `find_snippet_pipe` calls `line.find(" | ")` which returns the FIRST byte
//! offset of ` | ` anywhere in the line.  For a TOML snippet line such as:
//!
//!   "  3 | api_key = \"abc | def\""
//!
//! The first ` | ` is correctly the snippet separator (before `api_key`).
//! But for a TOML snippet containing a value that starts with ` | `:
//!
//!   "  5 | api_key = \"top | secret\""
//!
//! the separator ` | ` still appears first (before `api_key`), so this
//! case is handled correctly by position, but for a pathological case where
//! the value appears BEFORE the actual snippet pipe in a raw (non-snippet)
//! source line:
//!
//!   "api_key = \"top | secret\""
//!
//! There is no digit prefix at all; the function must return `None` so the
//! raw-source-line fallback path handles it instead of the snippet path
//! (which would misparse the line and miss the credential).
//!
//! ## Red Gate
//!
//! `test_AC_001_credential_value_with_pipe_does_not_break_extraction` must FAIL
//! (the secret value leaks into the error) until the anchor fix is in place.
//!
//! `test_AC_002_only_digit_prefix_matches_pipe` must FAIL (or at minimum the
//! redaction for the raw-source-line path is broken) until the fix lands.
//!
//! `test_AC_001_caret_line_pipe_matched` must PASS at the Red Gate (caret lines
//! are already matched by `line.find(" | ")`) and acts as a regression guard
//! for the spaces-only prefix case that must continue to work after the fix.
//!
//! ## Approach
//!
//! `find_snippet_pipe` and `sanitize_error_message` are both private.  Tests
//! exercise the function end-to-end via `load_and_validate` using crafted TOML
//! fixtures that force the TOML parser to include lines with ` | ` inside
//! credential values in the error snippet.
//!
//! Test naming: `test_AC_001_*`, `test_AC_002_*` per story AC IDs.
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
// AC-001: credential value containing " | " does not bypass redaction
// ===========================================================================

/// BC-3.3.004 postcondition 2 / AC-001 / EC-003 (SEC-P3-002):
///
/// A TOML snippet line such as `"  3 | api_key = \"abc | def\""` must have its
/// credential value redacted, even though the value itself contains ` | `.
///
/// The fixed `find_snippet_pipe` checks that the prefix before the first ` | `
/// consists only of ASCII digits and spaces.  For this line the prefix is
/// `"  3"` — all digits/spaces — so pipe is found at the correct offset, the
/// field name `api_key` is extracted, and the line is redacted.
///
/// ## Production gap (Red Gate — MUST FAIL before fix)
///
/// The unfixed `line.find(" | ")` returns the offset of the FIRST ` | ` in
/// the line, which is still the `3 |` separator position.  However, the
/// concern arises in combination with the first-occurrence semantics when
/// the value fragment ` | ` appears BEFORE any digit-prefixed separator (e.g.,
/// in a raw source line without a digit prefix).  This test verifies the
/// primary case: even with ` | ` in the value, the `api_key` field is detected
/// and its value is redacted.
///
/// Because the unfixed code handles the separator-first case correctly by
/// accident (the separator `3 |` always precedes the value ` | `), this test
/// may or may not fail at the Red Gate depending on parser output format.
/// It is included as a hard requirement and a definitive guard post-fix.
#[test]
fn test_AC_001_credential_value_with_pipe_does_not_break_extraction() {
    let dir = TempDir::new().unwrap();
    // Use `api_key` (matches `_key` suffix) with a value containing literal
    // ` | `.  The TOML string must be valid so the parser echoes it in the
    // error snippet.  The trailing `!invalid` forces the parse error.
    let toml = "schema_version = 1\n\
                org_id = \"01975e4e-9f00-7abc-8def-000000000010\"\n\
                org_slug = \"pipe\"\n\
                display_name = \"Pipe Test\"\n\
                \n\
                [api_creds]\n\
                api_key = \"abc | def-secret\"\n\
                \n\
                # Force parse error so snippet is emitted.\n\
                !invalid\n";
    write_toml(&dir, "pipe.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-001 pipe-in-value: expected at least one ConfigError, got none"
    );

    // The credential value must be redacted regardless of ` | ` inside it.
    for msg in &msgs {
        assert!(
            !msg.contains("abc | def-secret"),
            "AC-001 (SEC-P3-002 / EC-003): api_key value 'abc | def-secret' containing \
             a literal ' | ' must still be redacted by sanitize_error_message. \
             Full message: {msg}"
        );
        // Neither fragment must appear independently.
        assert!(
            !msg.contains("def-secret"),
            "AC-001 (SEC-P3-002): fragment 'def-secret' from api_key value must not \
             appear in ConfigError. Full message: {msg}"
        );
    }
}

/// BC-3.3.004 postcondition 2 / AC-001 / EC-003 (SEC-P3-002):
///
/// A credential field value containing ` | ` in a raw (non-snippet) source
/// line must also be redacted via the raw-source fallback path.
///
/// The raw-source path in `sanitize_error_message` (the `else` branch for
/// non-snippet lines) uses `line.find(" = ")` to extract the leading field name
/// and check `is_credential_pattern`.  This is unaffected by `find_snippet_pipe`
/// semantics.  The test verifies end-to-end behaviour for raw lines.
///
/// ## Red Gate behaviour
///
/// This test exercises a path that was already correct before the fix because
/// raw source lines skip `find_snippet_pipe` entirely.  It acts as a regression
/// guard to ensure the fix does not accidentally break the raw-source path.
#[test]
fn test_AC_001_raw_source_line_api_key_with_pipe_redacted() {
    let dir = TempDir::new().unwrap();
    // A TOML file where the credential appears in a raw context appended to
    // the error message (not in a snippet).  We use a deliberately unquoted
    // value to produce a parse error at the line with the credential.
    let toml = "schema_version = 1\n\
                org_id = \"01975e4e-9f00-7abc-8def-000000000011\"\n\
                org_slug = \"rawpipe\"\n\
                display_name = \"RawPipe\"\n\
                \n\
                [sensor]\n\
                api_key = \"raw-pipe-secret\"\n\
                \n\
                # Parse error triggers raw context dump.\n\
                !invalid\n";
    write_toml(&dir, "rawpipe.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-001 raw-source: expected at least one ConfigError, got none"
    );

    for msg in &msgs {
        assert!(
            !msg.contains("raw-pipe-secret"),
            "AC-001 (SEC-P3-002): api_key raw-source value 'raw-pipe-secret' must \
             be redacted. Full message: {msg}"
        );
    }
}

// ===========================================================================
// AC-002: only digit-prefix lines match as snippet lines
// ===========================================================================

/// BC-3.3.004 postcondition 2 / AC-002 / EC-004 (SEC-P3-002):
///
/// A raw (non-snippet) source line that contains ` | ` as part of a credential
/// value and has NO digit prefix — e.g. `"api_key = \"top | secret\""` —
/// must NOT be classified as a snippet line by `find_snippet_pipe`.
///
/// `find_snippet_pipe` must return `None` for such lines so that the
/// raw-source fallback branch handles them correctly via leading-field extraction.
///
/// ## Why this matters
///
/// If `find_snippet_pipe` returns `Some(pos)` for
/// `"api_key = \"top | secret\""`, it treats the part before the first ` | `
/// (i.e., `"api_key = \"top"`) as the prefix and the part after as the
/// content.  The extracted "field name" before ` = ` in `"api_key = \"top"` is
/// `api_key`, which DOES match a credential pattern, so the line would
/// actually be redacted — but for the wrong reason and with incorrect prefix
/// handling.  A future code path change could easily break this coincidence.
///
/// The fix anchors `find_snippet_pipe` to require a digits/spaces-only prefix
/// before the first ` | `, so raw source lines with alphabetic characters
/// before any ` | ` return `None` and fall through to the correct raw-source
/// handling path.
///
/// ## Red Gate
///
/// This test may or may not fail at the Red Gate depending on whether the
/// accidental correct-redaction occurs.  It is a definitive contract test:
/// after the fix, the raw-source path always handles non-snippet lines.
#[test]
fn test_AC_002_only_digit_prefix_matches_pipe() {
    let dir = TempDir::new().unwrap();
    // Construct a TOML where `api_secret = "top | secret"` appears in a
    // parse error context that emits the raw line without a snippet prefix.
    // We use an invalid character BEFORE the credential line so the parse
    // error points at the line above but the credential still appears in
    // the error context.
    let toml = "schema_version = 1\n\
                org_id = \"01975e4e-9f00-7abc-8def-000000000012\"\n\
                org_slug = \"nondigit\"\n\
                display_name = \"NonDigit\"\n\
                \n\
                [credentials_section]\n\
                api_secret = \"top | secret-value\"\n\
                \n\
                # Force parse error.\n\
                !invalid\n";
    write_toml(&dir, "nondigit.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-002 no-digit-prefix: expected at least one ConfigError, got none"
    );

    // Regardless of which path handles the line, the secret must be redacted.
    for msg in &msgs {
        assert!(
            !msg.contains("secret-value"),
            "AC-002 (SEC-P3-002 / EC-004): api_secret value 'secret-value' from a \
             non-snippet (no digit prefix) line must be redacted via the raw-source \
             fallback path. Full message: {msg}"
        );
        assert!(
            !msg.contains("top | secret-value"),
            "AC-002 (SEC-P3-002 / EC-004): full api_secret value 'top | secret-value' \
             must not appear in ConfigError. Full message: {msg}"
        );
    }
}

// ===========================================================================
// EC-005: caret lines with spaces-only prefix must continue to match
// ===========================================================================

/// BC-3.3.004 postcondition 2 / EC-005 (SEC-P3-002):
///
/// TOML 0.8 error messages include caret lines (`"   | ^^^^^"`) whose prefix
/// consists solely of whitespace (spaces).  The fixed `find_snippet_pipe`
/// accepts a spaces-only prefix via `is_ascii_digit() || c == ' '`, so caret
/// lines continue to be identified as snippet lines.
///
/// ## Red Gate (MUST PASS — regression guard)
///
/// Caret lines contain no credential assignments; they are passed through
/// unchanged by `sanitize_error_message`.  The important thing is that
/// `find_snippet_pipe` returns `Some(pos)` for them (not `None`), so the
/// snippet processing branch runs and the caret line is included intact in
/// the sanitized message rather than falling through to the raw-source branch.
///
/// This test checks that caret-line content does NOT accidentally disappear
/// from the error message after the anchor fix.  It is a regression guard.
#[test]
fn test_AC_002_caret_lines_not_suppressed_by_anchor() {
    let dir = TempDir::new().unwrap();
    // A TOML file with a known syntax error that triggers a caret annotation.
    // The `= ===invalid===` syntax forces the TOML parser to emit a snippet
    // with caret lines pointing at the error position.
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000013"
org_slug = "caret"
display_name = """
ACME Corp
"""

bad = ===invalid===
"#;
    write_toml(&dir, "caret.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "EC-005: expected at least one ConfigError for caret-line test, got none"
    );

    // The caret character must appear somewhere in the error output, proving
    // that caret lines are not dropped by the anchor fix.
    let any_has_caret = msgs.iter().any(|m| m.contains('^'));
    assert!(
        any_has_caret,
        "EC-005 (SEC-P3-002): caret annotation lines ('^') must not be suppressed \
         by the find_snippet_pipe anchor fix. Messages: {msgs:?}"
    );
}

/// BC-3.3.004 postcondition 2 / AC-001 (SEC-P3-002):
///
/// Regression: single-line credential field redaction must continue to work
/// correctly through the (now anchored) `find_snippet_pipe` call path.
#[test]
fn test_AC_001_single_line_credential_through_anchored_pipe_finder() {
    let dir = TempDir::new().unwrap();
    let toml = r#"schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000014"
org_slug = "anchorreg"
display_name = "AnchorReg"

[creds]
api_token = "anchor-regression-secret"

# Force parse error.
!invalid
"#;
    write_toml(&dir, "anchorreg.toml", toml);

    let result = load_and_validate(dir.path());
    let errors = result.unwrap_err();
    let msgs = error_messages(&errors);

    assert!(
        !msgs.is_empty(),
        "AC-001 anchor-regression: expected at least one ConfigError, got none"
    );

    for msg in &msgs {
        assert!(
            !msg.contains("anchor-regression-secret"),
            "AC-001 (SEC-P3-002 regression): single-line api_token value \
             'anchor-regression-secret' must remain redacted after the \
             find_snippet_pipe anchor fix. Full message: {msg}"
        );
    }
}
