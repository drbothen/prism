//! AC-3: OrgSlug::new("../etc") returns Err — dot character is outside allowed set.

use prism_core::OrgSlug;

/// AC-3: path traversal sequence rejected due to '.' and '/'.
#[test]
fn test_ac3_tenant_id_rejects_path_traversal() {
    let result = OrgSlug::new("../etc");
    assert!(result.is_err(), "path traversal string must be rejected");
}

/// AC-3 edge: dot alone is rejected.
#[test]
fn test_ac3_tenant_id_rejects_dot() {
    let result = OrgSlug::new(".");
    assert!(result.is_err(), "dot character must be rejected");
}

/// AC-3 edge: slash alone is rejected.
#[test]
fn test_ac3_tenant_id_rejects_slash() {
    let result = OrgSlug::new("/");
    assert!(result.is_err(), "slash character must be rejected");
}

/// AC-3 edge: null byte is rejected.
#[test]
fn test_ac3_tenant_id_rejects_null_byte() {
    let result = OrgSlug::new("tenant\0id");
    assert!(result.is_err(), "null byte must be rejected");
}

/// AC-3 edge: at-sign is rejected.
#[test]
fn test_ac3_tenant_id_rejects_at_sign() {
    let result = OrgSlug::new("user@domain");
    assert!(result.is_err(), "@ character must be rejected");
}
