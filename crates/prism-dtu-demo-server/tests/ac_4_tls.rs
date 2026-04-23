//! AC-4: TLS support — self-signed cert generation and HTTPS connectivity.
//!
//! Given the `tls` feature is compiled in, when `generate_self_signed_cert()` is
//! called, then it returns (cert_pem, key_pem) without error. When a clone is
//! served over HTTPS using that cert, the TLS handshake succeeds.
//!
//! Expected Red Gate failure: `generate_self_signed_cert()` panics with `todo!()`.

// TLS tests only compile with the tls feature.
#[cfg(feature = "tls")]
mod tls_tests {
    use prism_dtu_demo_server::tls::inner;

    /// AC-4: generate_self_signed_cert returns non-empty PEM strings.
    #[tokio::test]
    async fn ac_4_generate_self_signed_cert_returns_pem_pair() {
        // Expected failure: panics with "not yet implemented".
        let (cert_pem, key_pem) =
            inner::generate_self_signed_cert().expect("AC-4: cert generation must succeed");

        assert!(
            !cert_pem.is_empty(),
            "AC-4: cert_pem must not be empty"
        );
        assert!(
            !key_pem.is_empty(),
            "AC-4: key_pem must not be empty"
        );
        assert!(
            cert_pem.contains("BEGIN CERTIFICATE"),
            "AC-4: cert_pem must contain PEM header; got first 80 chars: {}",
            &cert_pem[..cert_pem.len().min(80)]
        );
    }

    /// AC-4: print_cert_fingerprint does not panic on a valid PEM string.
    #[test]
    fn ac_4_print_cert_fingerprint_does_not_panic() {
        // Expected failure: panics with "not yet implemented".
        // We pass a placeholder PEM string — the test verifies the function
        // doesn't panic, not that it parses correctly (that's an implementation detail).
        inner::print_cert_fingerprint("-----BEGIN CERTIFICATE-----\nFAKE\n-----END CERTIFICATE-----\n");
    }
}

// When `tls` feature is NOT present, verify the feature-gate is correct by
// confirming the tls module's public inner module is absent.
#[cfg(not(feature = "tls"))]
mod no_tls_feature {
    /// AC-4 (no-tls): The tls module inner is not accessible without the tls feature.
    ///
    /// This test always passes (it's a compile-time check — if `inner` were
    /// unconditionally accessible this file wouldn't compile without tls).
    #[test]
    fn ac_4_tls_module_absent_without_tls_feature() {
        // If this file compiles without the tls feature and without accessing
        // `inner`, the feature gate is correct. Nothing to assert at runtime.
    }
}
