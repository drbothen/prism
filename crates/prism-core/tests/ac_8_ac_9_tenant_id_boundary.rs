//! AC-8: 64-char valid string is accepted (boundary: exactly 64 is valid).
//! AC-9: 65-char valid string is rejected (boundary: 65 is invalid).

use prism_core::OrgSlug;

/// AC-8: exactly 64 valid characters must be accepted.
#[test]
fn test_ac8_tenant_id_64_chars_valid() {
    // 64 'a' characters — exactly at the boundary
    let s = "a".repeat(64);
    let result = OrgSlug::new(&s);
    assert!(
        result.is_ok(),
        "64-character string must be accepted (boundary valid), got: {result:?}"
    );
    assert_eq!(result.unwrap().as_str(), s.as_str());
}

/// AC-9: exactly 65 valid characters must be rejected.
#[test]
fn test_ac9_tenant_id_65_chars_rejected() {
    // 65 'a' characters — one over the boundary
    let s = "a".repeat(65);
    let result = OrgSlug::new(&s);
    assert!(
        result.is_err(),
        "65-character string must be rejected (boundary invalid)"
    );
}

/// Additional boundary: 63-char string is accepted.
#[test]
fn test_ac8_tenant_id_63_chars_valid() {
    let s = "b".repeat(63);
    let result = OrgSlug::new(&s);
    assert!(result.is_ok(), "63-character string must be accepted");
}

/// Additional boundary: 100-char string is rejected.
#[test]
fn test_ac9_tenant_id_100_chars_rejected() {
    let s = "c".repeat(100);
    let result = OrgSlug::new(&s);
    assert!(result.is_err(), "100-character string must be rejected");
}
