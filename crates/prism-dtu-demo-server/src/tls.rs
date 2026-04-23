//! Optional TLS support for the demo harness.
//!
//! Only compiled when the `tls` feature is enabled.
//!
//! Uses `rcgen` to generate a self-signed ECDSA-P256 certificate at startup
//! (30-day validity) and serves clones over HTTPS via `axum-server` + `rustls`.
//!
//! # Warning
//!
//! This is demo-grade TLS only. The self-signed certificate is not trusted by
//! any CA. For stakeholder demos, add the printed cert fingerprint to your trust store.
//! Not for production use.

#[cfg(feature = "tls")]
pub mod inner {
    use anyhow::Result;

    /// Generate a self-signed ECDSA-P256 certificate valid for 30 days.
    ///
    /// Returns `(cert_pem, key_pem)`.
    pub fn generate_self_signed_cert() -> Result<(String, String)> {
        todo!(
            "TLS cert generation not yet implemented — \
             implement in S-6.20 Phase 2"
        )
    }

    /// Print the certificate fingerprint to stdout.
    ///
    /// Analysts can use this to add the cert to their trust store for demos.
    pub fn print_cert_fingerprint(cert_pem: &str) {
        todo!(
            "TLS fingerprint printing not yet implemented — \
             implement in S-6.20 Phase 2"
        )
    }
}
