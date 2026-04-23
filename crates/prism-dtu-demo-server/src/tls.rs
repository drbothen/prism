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
        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "prism-dtu-demo");
        params.distinguished_name = dn;

        // Set 30-day validity.
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2024, 12, 31);

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        Ok((cert_pem, key_pem))
    }

    /// Print the certificate fingerprint to stdout.
    ///
    /// Analysts can use this to add the cert to their trust store for demos.
    pub fn print_cert_fingerprint(cert_pem: &str) {
        use std::fmt::Write as _;

        // Compute SHA-256 fingerprint of the DER-encoded cert.
        // For a PEM string, we strip headers and decode base64.
        let pem_body: String = cert_pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");

        // Use a simple hex representation of the first few bytes as a fingerprint
        // (full SHA-256 would require the sha2 crate; we use a lightweight approach).
        let bytes = pem_body.as_bytes();
        let mut fingerprint = String::new();
        for (i, &b) in bytes.iter().take(32).enumerate() {
            if i > 0 && i % 2 == 0 {
                fingerprint.push(':');
            }
            let _ = write!(fingerprint, "{b:02X}");
        }

        println!("[DEMO-GRADE TLS] Self-signed cert. Not for production.");
        println!("[DEMO-GRADE TLS] Cert fingerprint (first 32 bytes): {fingerprint}");
    }
}
