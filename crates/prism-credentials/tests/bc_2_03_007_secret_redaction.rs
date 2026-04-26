//! Tests for BC-2.03.007: Secret Redaction in Logs, Errors, and MCP Responses
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests pass (implementation complete).

use prism_credentials::secret::{dry_run_preview, Secret};
use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// TV-BC-2.03.007-001: Display returns "[REDACTED]"
// ---------------------------------------------------------------------------

/// BC-2.03.007 postcondition: `format!("{}", secret)` returns "[REDACTED]".
/// The actual value must NEVER appear in formatted output.
#[test]
fn test_BC_2_03_007_display_returns_redacted() {
    let secret = Secret::new("sk-12345".to_string());
    let display_output = format!("{secret}");
    assert_eq!(
        display_output, "[REDACTED]",
        "Display must return [REDACTED], got: {display_output:?}"
    );
    // Confirm actual value is not leaked
    assert!(
        !display_output.contains("sk-12345"),
        "Display must not contain the actual value"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.007-002: Debug returns "SecretString([REDACTED])"
// ---------------------------------------------------------------------------

/// BC-2.03.007 postcondition: `format!("{:?}", secret)` returns "SecretString([REDACTED])".
#[test]
fn test_BC_2_03_007_debug_returns_secret_string_redacted() {
    let secret = Secret::new("sk-12345".to_string());
    let debug_output = format!("{secret:?}");
    assert_eq!(
        debug_output, "SecretString([REDACTED])",
        "Debug must return SecretString([REDACTED]), got: {debug_output:?}"
    );
    assert!(
        !debug_output.contains("sk-12345"),
        "Debug must not contain the actual value"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.007-003: dry_run_preview with 8-char value
// ---------------------------------------------------------------------------

/// BC-2.03.007 postcondition (dry-run): 8-char value "abcdefyz" displays as "ab***yz".
#[test]
fn test_BC_2_03_007_dry_run_preview_8_chars() {
    let preview = dry_run_preview("abcdefyz");
    assert_eq!(
        preview, "ab***yz",
        "8-char dry-run preview should be 'ab***yz', got: {preview:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.007-004: dry_run_preview with 1-char value
// ---------------------------------------------------------------------------

/// BC-2.03.007 EC-03-017: 1-character credential displays as "***" (no char leakage).
#[test]
fn test_BC_2_03_007_dry_run_preview_one_char_returns_stars_only() {
    let preview = dry_run_preview("X");
    assert_eq!(
        preview, "***",
        "1-char dry-run preview must return '***' only, got: {preview:?}"
    );
}

// ---------------------------------------------------------------------------
// Edge case: short values (2–4 chars) also return "***"
// ---------------------------------------------------------------------------

/// BC-2.03.007 EC-03-017: values shorter than 5 chars return "***" only.
#[test]
fn test_BC_2_03_007_dry_run_preview_short_values_return_stars_only() {
    for short in &["a", "ab", "abc", "abcd"] {
        let preview = dry_run_preview(short);
        assert_eq!(
            preview, "***",
            "Short value {:?} must return '***', got: {preview:?}",
            short
        );
    }
}

// ---------------------------------------------------------------------------
// dry_run_preview: 5-char value
// ---------------------------------------------------------------------------

/// BC-2.03.007: 5-char value "abcde" displays as "ab***de" — boundary check.
#[test]
fn test_BC_2_03_007_dry_run_preview_five_chars() {
    let preview = dry_run_preview("abcde");
    assert_eq!(
        preview, "ab***de",
        "5-char dry-run preview should be 'ab***de', got: {preview:?}"
    );
}

// ---------------------------------------------------------------------------
// expose() returns the actual value
// ---------------------------------------------------------------------------

/// BC-2.03.007: `.expose()` is the only way to access the inner value.
#[test]
fn test_BC_2_03_007_expose_returns_actual_value() {
    let secret = Secret::new("sk-12345".to_string());
    let exposed = secret.expose();
    assert_eq!(exposed, "sk-12345", "expose() must return the actual value");
}

// ---------------------------------------------------------------------------
// Display and Debug never contain the real value regardless of content
// ---------------------------------------------------------------------------

/// BC-2.03.007 invariant DI-002: no format macro can leak the credential value.
#[test]
fn test_BC_2_03_007_invariant_no_format_macro_leaks_value() {
    let sensitive = "super-secret-password-12345";
    let secret = Secret::new(sensitive.to_string());

    let display = format!("{secret}");
    let debug = format!("{secret:?}");
    let display_alt = format!("credential={secret}");

    assert!(!display.contains(sensitive));
    assert!(!debug.contains(sensitive));
    assert!(!display_alt.contains(sensitive));
}

// ---------------------------------------------------------------------------
// Zeroize: Secret implements Zeroize
// ---------------------------------------------------------------------------

/// BC-2.03.007 / architecture constraint: Secret<T> implements Zeroize.
/// This test verifies the trait bound is satisfied — compile error = failure.
#[test]
fn test_BC_2_03_007_secret_implements_zeroize() {
    let mut secret = Secret::new("sk-12345".to_string());
    // If Secret<String> does not implement Zeroize, this won't compile.
    secret.zeroize();
}

// ---------------------------------------------------------------------------
// Empty string is also redacted
// ---------------------------------------------------------------------------

/// BC-2.03.007: empty string value is still redacted.
#[test]
fn test_BC_2_03_007_empty_string_is_redacted() {
    let secret = Secret::new(String::new());
    assert_eq!(format!("{secret}"), "[REDACTED]");
    assert_eq!(format!("{secret:?}"), "SecretString([REDACTED])");
}
