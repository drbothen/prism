//! AC-2: OrgSlug::new("acme_corp-01") returns Ok and as_str() == "acme_corp-01".

use prism_core::OrgSlug;

/// AC-2: valid identifier round-trips through construction and as_str().
#[test]
fn test_ac2_tenant_id_valid_round_trip() {
    let input = "acme_corp-01";
    let result = OrgSlug::new(input);
    assert!(
        result.is_ok(),
        "valid identifier must be accepted, got: {result:?}"
    );

    let tenant_id = result.unwrap();
    assert_eq!(
        tenant_id.as_str(),
        input,
        "as_str() must return the original validated string"
    );
}

/// AC-2 supplement: single character valid identifier.
#[test]
fn test_ac2_tenant_id_single_char_valid() {
    let result = OrgSlug::new("a");
    assert!(result.is_ok(), "single valid char must be accepted");
    assert_eq!(result.unwrap().as_str(), "a");
}

/// AC-2 supplement: uppercase, digits, underscore, hyphen all valid.
#[test]
fn test_ac2_tenant_id_all_valid_char_classes() {
    let result = OrgSlug::new("Tenant_1-A");
    assert!(result.is_ok(), "mix of valid chars must be accepted");
}
