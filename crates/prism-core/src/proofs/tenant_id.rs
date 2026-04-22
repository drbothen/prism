//! VP-001: Kani proofs for TenantId validation invariants.
//!
//! Proof method: Kani (bounded model checking), strings up to length 8.
//! Run with: `cargo kani --proof verify_tenant_id_validation`
//!
//! All harnesses are gated with `#[cfg(kani)]` — zero effect on non-kani builds.

#[cfg(kani)]
mod proofs {
    use crate::tenant::TenantId;

    /// VP-001 Proof 1: empty string is always rejected.
    #[kani::proof]
    fn proof_empty_string_rejected() {
        let result = TenantId::new("");
        assert!(result.is_err(), "TenantId::new(\"\") must return Err");
    }

    /// VP-001 Proof 2: 65-character string of valid chars is always rejected (length > 64).
    #[kani::proof]
    fn proof_65_chars_rejected() {
        let s = "a".repeat(65);
        let result = TenantId::new(&s);
        assert!(result.is_err(), "TenantId::new(65 chars) must return Err");
    }

    /// VP-001 Proof 3: string containing '/' is always rejected (invalid character).
    #[kani::proof]
    fn proof_slash_rejected() {
        let result = TenantId::new("valid/path");
        assert!(result.is_err(), "TenantId::new with '/' must return Err");
    }

    /// VP-001 Proof 4: "acme_corp-01" is always accepted (valid input).
    #[kani::proof]
    fn proof_valid_input_accepted() {
        let result = TenantId::new("acme_corp-01");
        assert!(result.is_ok(), "TenantId::new(\"acme_corp-01\") must return Ok");
    }

    /// VP-001 Full harness: for all bounded inputs, result matches expected validity.
    ///
    /// Sourced verbatim from vp-001-tenant-id-validation.md proof harness skeleton.
    #[kani::proof]
    #[kani::unwind(9)] // length 8 + 1
    fn verify_tenant_id_validation() {
        let len: usize = kani::any();
        kani::assume(len <= 8);
        let bytes: [u8; 8] = kani::any();
        let s = std::str::from_utf8(&bytes[..len]);
        if let Ok(input) = s {
            let result = TenantId::new(input);
            let is_valid = !input.is_empty()
                && input
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
            if is_valid {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }
}
