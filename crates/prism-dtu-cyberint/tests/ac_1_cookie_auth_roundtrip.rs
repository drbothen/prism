#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-1: Cookie auth round-trip (rewritten for ADR-031 §D3-a static-token model).
//!
//! Rewritten per S-DTU-CYBERINT-AUTH-FIDELITY-001 Task 11:
//! The real Cyberint API uses `Cookie: access_token={api_key}` directly —
//! no `POST /login` step exists. The DTU is corrected to match.
//!
//! Pattern:
//! - Register demo token via `configure({ access_token: "demo-access-key" })`
//! - Send `Cookie: access_token=demo-access-key` with each authenticated request
//! - ADR-031 §D3-a: `cyberint_session` cookie name is permanently superseded.

#[cfg(feature = "dtu")]
mod ac_1 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    const DEMO_TOKEN: &str = "demo-access-key";

    /// Start a clone with a pre-registered demo access token.
    /// Returns (clone, base_url, client).
    async fn start_with_demo_token() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-1: CyberintClone::new() must succeed");
        clone.start().await.expect("AC-1: start() must succeed");
        let base_url = clone.base_url();

        // Register the demo access_token via configure (AC-004 pattern).
        clone
            .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
            .await
            .expect("AC-1: configure must succeed");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("AC-1: build reqwest client");
        (clone, base_url, client)
    }

    /// AC-1 primary: authenticated request with access_token cookie returns 200.
    ///
    /// The real Cyberint API accepts `Cookie: access_token={api_key}` directly.
    /// No login step. This test verifies the corrected DTU matches that behavior.
    ///
    /// Replaces the old `ac_1_login_returns_200_with_set_cookie_header` test which
    /// tested the incorrect POST /login → cyberint_session flow (ADR-031 §D1-b).
    #[tokio::test]
    async fn ac_1_login_returns_200_with_set_cookie_header() {
        // NOTE: This test name is retained for traceability with the story AC-001
        // Red Gate test. The test now verifies that access_token cookie auth works
        // (replacing the removed POST /login route).
        let (_clone, base_url, client) = start_with_demo_token().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
            .send()
            .await
            .expect("AC-1: GET /api/v1/alerts with access_token cookie must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-1: GET /api/v1/alerts with 'Cookie: access_token=demo-access-key' \
             MUST return HTTP 200 — DTU corrected to accept access_token cookie \
             (ADR-031 §D3-a). Got {} instead.",
            resp.status().as_u16()
        );
    }

    /// AC-1 secondary: authenticated request with valid access_token returns 200.
    ///
    /// Replaces the old post-login authenticated request test.
    #[tokio::test]
    async fn ac_1_authenticated_request_after_login_returns_200() {
        let (_clone, base_url, client) = start_with_demo_token().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
            .send()
            .await
            .expect("AC-1: authenticated GET /api/v1/alerts must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-1: GET /api/v1/alerts with valid access_token cookie must return HTTP 200; \
             got {}",
            resp.status().as_u16()
        );
    }

    /// EC-003 (rewritten): registering two distinct tokens yields two independently valid tokens.
    ///
    /// The allowlist model permits multiple valid tokens concurrently.
    /// Replaces the old "two logins → two distinct session tokens" test.
    #[tokio::test]
    async fn ac_1_ec_003_two_logins_yield_distinct_valid_tokens() {
        let mut clone = CyberintClone::new().expect("EC-003: CyberintClone::new() must succeed");
        clone.start().await.expect("EC-003: start() must succeed");
        let base_url = clone.base_url();

        // Register two distinct tokens.
        let token1 = "ec-003-token-one";
        let token2 = "ec-003-token-two";

        clone
            .configure(serde_json::json!({"access_token": token1}))
            .await
            .expect("EC-003: configure token1 must succeed");
        clone
            .configure(serde_json::json!({"access_token": token2}))
            .await
            .expect("EC-003: configure token2 must succeed");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("EC-003: build client");

        // Both tokens must be independently valid.
        for (label, token) in [("token1", token1), ("token2", token2)] {
            let r = client
                .get(format!("{base_url}/api/v1/alerts"))
                .header("Cookie", format!("access_token={token}"))
                .send()
                .await
                .expect("EC-003: request with {label} must send");
            assert_eq!(
                r.status().as_u16(),
                200,
                "EC-003: {label} must be a valid access token (got {} instead of 200) \
                 — the allowlist model supports multiple concurrent valid tokens \
                 (ADR-031 §D3-a rule 3).",
                r.status()
            );
        }

        // The two tokens are distinct.
        assert_ne!(token1, token2, "EC-003: two distinct tokens must differ");
    }
}
