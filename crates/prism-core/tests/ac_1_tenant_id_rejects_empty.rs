//! AC-1: TenantId::new("") returns Err(PrismError::InvalidTenantId) with "E-AUTH" in message.

use prism_core::{PrismError, TenantId};

/// AC-1: empty string is rejected with E-AUTH error code in message.
#[test]
fn test_ac1_tenant_id_rejects_empty_string() {
    let result = TenantId::new("");
    assert!(result.is_err(), "empty string must be rejected");

    let err = result.unwrap_err();
    // Verify it is the InvalidTenantId variant
    assert!(
        matches!(err, PrismError::InvalidTenantId { .. }),
        "expected PrismError::InvalidTenantId, got: {err:?}"
    );
    // Verify Display begins with E-AUTH error code
    let msg = format!("{err}");
    assert!(
        msg.contains("E-AUTH"),
        "error message must contain 'E-AUTH', got: {msg:?}"
    );
}

/// AC-1 edge: whitespace-only string is also rejected.
#[test]
fn test_ac1_tenant_id_rejects_whitespace_only() {
    let result = TenantId::new("   ");
    assert!(result.is_err(), "whitespace-only string must be rejected");
}

/// AC-1 edge: single space character is rejected.
#[test]
fn test_ac1_tenant_id_rejects_single_space() {
    let result = TenantId::new(" ");
    assert!(result.is_err(), "single space must be rejected");
}
