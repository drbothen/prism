//! TD-WV0-07: `/dtu/configure` requires `X-Admin-Token` header (ADR-003 Amendment #5).
//!
//! - No token → 401 Unauthorized.
//! - Wrong token → 401 Unauthorized.
//! - Correct token → 200 OK.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "dtu")]
mod td_wv0_07 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    /// No token → 401 Unauthorized.
    #[tokio::test]
    async fn configure_without_token_returns_401() {
        let mut clone = CyberintClone::new().expect("TD-WV0-07 cyberint: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-07 cyberint: start() must succeed");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/configure", clone.base_url()))
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            401,
            "TD-WV0-07: missing X-Admin-Token must return 401"
        );
    }

    /// Wrong token → 401 Unauthorized.
    #[tokio::test]
    async fn configure_with_wrong_token_returns_401() {
        let mut clone = CyberintClone::new().expect("TD-WV0-07 cyberint: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-07 cyberint: start() must succeed");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/configure", clone.base_url()))
            .header("X-Admin-Token", "wrong-token-that-will-never-match")
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            401,
            "TD-WV0-07: incorrect X-Admin-Token must return 401"
        );
    }

    /// Correct token → 200 OK.
    #[tokio::test]
    async fn configure_with_correct_token_returns_200() {
        let mut clone = CyberintClone::new().expect("TD-WV0-07 cyberint: new must succeed");
        clone
            .start()
            .await
            .expect("TD-WV0-07 cyberint: start() must succeed");
        let token = clone.admin_token().to_string();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/dtu/configure", clone.base_url()))
            .header("X-Admin-Token", &token)
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("request must succeed");

        assert_eq!(
            resp.status(),
            200,
            "TD-WV0-07: correct X-Admin-Token must return 200"
        );
    }
}
