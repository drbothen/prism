//! TD-WV1-04: Failing tests for `DemoHarness::start_all` TLS wiring.
//!
//! # Acceptance criteria exercised
//!
//! AC-4  (harness layer):  `start_all` propagates `Option<Arc<RustlsConfig>>` to
//!         each clone's `start_on`.
//! AC-6:  `url_map()` returns `https://…` values when TLS is active.
//!
//! # Red Gate
//!
//! Both tests that call `start_all` with a TLS argument will FAIL TO COMPILE
//! against the current codebase because `DemoHarness::start_all` currently
//! accepts `(&DemoConfig)` only.
//!
//! After the TD-WV1-04 fix the signature becomes:
//!   `start_all(&mut self, config: &DemoConfig, tls: Option<Arc<RustlsConfig>>)`
//!
//! The test `td_wv1_04_harness_start_all_without_tls_serves_http` uses the new
//! signature with `None` and verifies the plain-HTTP path still works.  It
//! will also fail to compile until the signature is updated — both tests share
//! the same compile-time Red Gate.

#[cfg(feature = "tls")]
mod harness_tls_tests {
    use std::sync::Arc;

    use axum_server::tls_rustls::RustlsConfig;
    use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};
    use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};
    use prism_dtu_demo_server::tls::inner;

    /// Build a minimal `DemoConfig` with only crowdstrike + threatintel enabled,
    /// both on ephemeral ports.  Using only 2 clones keeps test startup time low.
    fn two_clone_config() -> DemoConfig {
        DemoConfig {
            harness: Default::default(),
            clones: ClonesConfig {
                crowdstrike: CloneConfig {
                    enabled: true,
                    port: 0,
                    ..Default::default()
                },
                claroty: CloneConfig {
                    enabled: false,
                    ..Default::default()
                },
                cyberint: CloneConfig {
                    enabled: false,
                    ..Default::default()
                },
                armis: CloneConfig {
                    enabled: false,
                    ..Default::default()
                },
                threatintel: CloneConfig {
                    enabled: true,
                    port: 0,
                    ..Default::default()
                },
                nvd: CloneConfig {
                    enabled: false,
                    ..Default::default()
                },
            },
        }
    }

    /// Build an HTTPS-capable reqwest client that accepts self-signed certs.
    fn https_client() -> reqwest::Client {
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("reqwest HTTPS client must build")
    }

    // ---------------------------------------------------------------------------
    // TD-WV1-04-001
    // ---------------------------------------------------------------------------

    /// TD-WV1-04: `DemoHarness::start_all` propagates `RustlsConfig` to all clones
    /// and each clone serves HTTPS on `/dtu/health`.
    ///
    /// # Red Gate
    ///
    /// Fails to compile: `start_all` does not accept a second `tls` argument.
    /// After the fix, `start_all(&mut self, config, Some(Arc::new(rustls_cfg)))`
    /// must wire TLS into every clone's `start_on` call, and `url_map()` must
    /// return `https://…` values.
    #[tokio::test]
    async fn td_wv1_04_harness_start_all_with_tls_serves_https_on_all_clones() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let (cert_pem, key_pem, _cert_der) =
            inner::generate_self_signed_cert().expect("cert generation must succeed");
        let rustls_cfg: RustlsConfig = inner::build_rustls_config(&cert_pem, &key_pem)
            .await
            .expect("RustlsConfig must build");

        let config = two_clone_config();
        let pairs = build_clone_pairs(&config).expect("build_clone_pairs must succeed");
        let mut harness = DemoHarness::new(pairs);

        // RED GATE: `start_all` currently takes `(&DemoConfig)` only.
        // After the fix it takes `(&DemoConfig, Option<Arc<RustlsConfig>>)`.
        harness
            .start_all(&config, Some(Arc::new(rustls_cfg)))
            .await
            .expect("TD-WV1-04: start_all with TLS must succeed");

        // Each clone's URL in the url_map must start with https://.
        let url_map = harness.url_map();
        assert_eq!(
            url_map.len(),
            2,
            "TD-WV1-04: url_map must contain exactly 2 entries (crowdstrike + threatintel)"
        );

        let client = https_client();

        for (name, url) in &url_map {
            // AC-6: url_map values must be https:// when TLS is active.
            assert!(
                url.starts_with("https://"),
                "TD-WV1-04: url_map[{name}] must start with https://; got: {url}"
            );

            // Verify the clone is actually serving HTTPS on /dtu/health.
            let health_url = format!("{url}/dtu/health");
            let resp = client
                .get(&health_url)
                .send()
                .await
                .expect("TD-WV1-04: HTTPS GET to {name}/dtu/health must succeed");

            assert_eq!(
                resp.status(),
                200,
                "TD-WV1-04: {name}/dtu/health over HTTPS must return 200"
            );

            let body: serde_json::Value = resp.json().await.expect("response must be JSON");
            assert_eq!(
                body["status"], "ok",
                "TD-WV1-04: {name}/dtu/health body must be {{\"status\":\"ok\"}}"
            );
        }

        harness.stop_all().await;
    }

    // ---------------------------------------------------------------------------
    // TD-WV1-04-002
    // ---------------------------------------------------------------------------

    /// TD-WV1-04: `DemoHarness::start_all` with `None` tls still serves plain HTTP.
    ///
    /// Verifies backward compatibility: when the `tls` argument is `None`, clones
    /// bind via plain `axum::serve` and `url_map()` returns `http://…` values.
    ///
    /// # Red Gate
    ///
    /// Fails to compile: same compile error as the TLS variant above — `start_all`
    /// does not accept a second argument in the current code.
    ///
    /// Once the new signature lands, this test is expected to PASS (it verifies the
    /// None/plain-HTTP path is backward compatible).
    #[tokio::test]
    async fn td_wv1_04_harness_start_all_without_tls_serves_http() {
        let config = two_clone_config();
        let pairs = build_clone_pairs(&config).expect("build_clone_pairs must succeed");
        let mut harness = DemoHarness::new(pairs);

        // RED GATE (compile): start_all does not accept a second argument yet.
        harness
            .start_all(&config, None)
            .await
            .expect("TD-WV1-04: start_all without TLS must succeed");

        let url_map = harness.url_map();
        assert_eq!(url_map.len(), 2, "url_map must contain 2 entries");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("reqwest client must build");

        for (name, url) in &url_map {
            // Without TLS, url_map values must still use http://.
            assert!(
                url.starts_with("http://"),
                "TD-WV1-04: url_map[{name}] must start with http:// when TLS is None; got: {url}"
            );

            let health_url = format!("{url}/dtu/health");
            let resp = client
                .get(&health_url)
                .send()
                .await
                .expect("TD-WV1-04: HTTP GET to {name}/dtu/health must succeed");

            assert_eq!(
                resp.status(),
                200,
                "TD-WV1-04: {name}/dtu/health over HTTP must return 200"
            );
        }

        harness.stop_all().await;
    }
}
