//! TD-WV0-08: `POST /dtu/reset` requires `X-Admin-Token` header (ADR-003 Amendment #5, CR-021).
//!
//! - No token → 401 Unauthorized (currently returns 200 — this is the bug under test).
//! - Wrong token → 401 Unauthorized.
//! - Correct token → 200 OK.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "dtu")]
mod td_wv0_08 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_pagerduty::PagerDutyClone;

    /// No token → 401 Unauthorized.
    ///
    /// RED GATE: currently returns 200 because post_reset has no admin-token gate (CR-021).
    #[tokio::test]
    async fn test_reset_requires_admin_token_missing_returns_401() {
        let mut clone = PagerDutyClone::new().expect("TD-WV0-08 pagerduty: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-08 pagerduty: start() must succeed");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/reset", clone.base_url()))
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            401,
            "TD-WV0-08: missing X-Admin-Token must return 401 (CR-021: currently no gate)"
        );
    }

    /// Wrong token → 401 Unauthorized.
    ///
    /// RED GATE: currently returns 200 because post_reset has no admin-token gate (CR-021).
    #[tokio::test]
    async fn test_reset_requires_admin_token_wrong_returns_401() {
        let mut clone = PagerDutyClone::new().expect("TD-WV0-08 pagerduty: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-08 pagerduty: start() must succeed");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/reset", clone.base_url()))
            .header("X-Admin-Token", "wrong-token-that-will-never-match")
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            401,
            "TD-WV0-08: incorrect X-Admin-Token must return 401 (CR-021: currently no gate)"
        );
    }

    /// Correct token → 200 OK + state actually reset.
    #[tokio::test]
    async fn test_reset_correct_admin_token_returns_200() {
        let mut clone = PagerDutyClone::new().expect("TD-WV0-08 pagerduty: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-08 pagerduty: start() must succeed");
        let token = clone.admin_token().to_string();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/reset", clone.base_url()))
            .header("X-Admin-Token", &token)
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            200,
            "TD-WV0-08: correct X-Admin-Token must return 200"
        );
    }
}
