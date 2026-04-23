//! Optional TLS support for the demo harness.
//!
//! Only compiled when the `tls` feature is enabled.
//!
//! Uses `rcgen` to generate a self-signed ECDSA-P256 certificate valid for 1 year
//! from the current time, and serves clones over HTTPS via `axum-server` + `rustls`.
//!
//! # Warning
//!
//! This is demo-grade TLS only. The self-signed certificate is not trusted by
//! any CA. For stakeholder demos, add the printed cert fingerprint to your trust store.
//! Not for production use.

#[cfg(feature = "tls")]
pub mod inner {
    use anyhow::Result;

    /// Generate a self-signed ECDSA-P256 certificate valid for 1 year from now.
    ///
    /// Returns `(cert_pem, key_pem, cert_der)`.
    ///
    /// The DER bytes are provided so callers can compute the SHA-256 fingerprint
    /// without re-decoding the PEM.
    pub fn generate_self_signed_cert() -> Result<(String, String, Vec<u8>)> {
        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
        use time::OffsetDateTime;

        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "prism-dtu-demo");
        params.distinguished_name = dn;

        // Dynamic validity: now .. now + 365 days.
        // Using time::OffsetDateTime (transitive dep via rcgen).
        let now = OffsetDateTime::now_utc();
        let one_year = time::Duration::days(365);
        params.not_before = now;
        params.not_after = now + one_year;

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();
        // `cert.der()` returns a `CertificateDer<'static>` — deref to `[u8]`.
        let cert_der: Vec<u8> = cert.der().to_vec();

        Ok((cert_pem, key_pem, cert_der))
    }

    /// Print the certificate fingerprint to stdout.
    ///
    /// Computes `sha256:<lowercase-hex>` of the DER-encoded certificate bytes.
    /// Analysts can use this to add the cert to their trust store for demos.
    pub fn print_cert_fingerprint(cert_der: &[u8]) {
        use sha2::{Digest, Sha256};

        let digest = Sha256::digest(cert_der);
        let hex = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":");

        println!("[DEMO-GRADE TLS] Self-signed cert. Not for production.");
        println!("[DEMO-GRADE TLS] SHA-256 fingerprint (DER): sha256:{hex}");
    }

    /// Build an `axum_server` `RustlsConfig` from PEM cert + key strings.
    ///
    /// This is the only place in the demo server that touches `axum_server`.
    /// The config can be cloned cheaply (it wraps an `Arc`) and passed to
    /// `axum_server::bind_rustls`.
    pub async fn build_rustls_config(
        cert_pem: &str,
        key_pem: &str,
    ) -> Result<axum_server::tls_rustls::RustlsConfig> {
        axum_server::tls_rustls::RustlsConfig::from_pem(
            cert_pem.as_bytes().to_vec(),
            key_pem.as_bytes().to_vec(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build RustlsConfig from PEM: {}", e))
    }
}
