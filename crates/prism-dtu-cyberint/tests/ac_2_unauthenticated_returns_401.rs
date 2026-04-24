#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-2: Unauthenticated requests to authenticated endpoints return HTTP 401.
//!
//! Given any request to `/api/v1/alerts` (or any other auth-guarded endpoint)
//! without a Cookie header, the response is HTTP 401
//! `{"error": "unauthorized", "code": 401}` — exercising Prism's E-AUTH-004 path.

#[cfg(feature = "dtu")]
mod ac_2 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn start() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-2: new must succeed");
        clone.start().await.expect("AC-2: start must succeed");
        let base_url = clone.base_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, client)
    }

    /// GET /api/v1/alerts with no Cookie returns 401 with correct error body.
    #[tokio::test]
    async fn ac_2_alerts_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .send()
            .await
            .expect("AC-2: request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: GET /api/v1/alerts without cookie must return HTTP 401"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("AC-2: 401 response must be valid JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "unauthorized",
            "AC-2: error field must be 'unauthorized'"
        );
        assert_eq!(
            body["code"].as_u64().unwrap_or(0),
            401,
            "AC-2: code field must be 401"
        );
    }

    /// POST /api/v1/alerts (alternate method) with no cookie returns 401.
    #[tokio::test]
    async fn ac_2_alerts_post_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .post(format!("{base_url}/api/v1/alerts"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-2: POST request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: POST /api/v1/alerts without cookie must return HTTP 401"
        );
    }

    /// GET /api/v1/alerts/{id} with no cookie returns 401.
    #[tokio::test]
    async fn ac_2_alert_detail_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-001"))
            .send()
            .await
            .expect("AC-2: request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: GET /api/v1/alerts/CYB-2024-001 without cookie must return HTTP 401"
        );
    }

    /// PATCH /api/v1/alerts/{id}/status with no cookie returns 401.
    #[tokio::test]
    async fn ac_2_patch_status_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-001/status"))
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-2: PATCH request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: PATCH /api/v1/alerts/id/status without cookie must return HTTP 401"
        );
    }

    /// POST /api/v1/alerts/{id}/close with no cookie returns 401.
    #[tokio::test]
    async fn ac_2_close_alert_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-001/close"))
            .send()
            .await
            .expect("AC-2: close request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: POST /api/v1/alerts/id/close without cookie must return HTTP 401"
        );
    }

    /// GET /api/v1/threat-intel with no cookie returns 401.
    #[tokio::test]
    async fn ac_2_threat_intel_no_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .send()
            .await
            .expect("AC-2: threat-intel request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: GET /api/v1/threat-intel without cookie must return HTTP 401"
        );
    }

    /// Empty Cookie header (not containing cyberint_session) returns 401.
    #[tokio::test]
    async fn ac_2_alerts_empty_cookie_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", "other_cookie=somevalue")
            .send()
            .await
            .expect("AC-2: request with wrong cookie must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: Cookie header without cyberint_session must return HTTP 401"
        );
    }

    /// Invalid (non-registered) session token returns 401.
    #[tokio::test]
    async fn ac_2_invalid_session_token_returns_401() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header(
                "Cookie",
                "cyberint_session=00000000-0000-0000-0000-000000000000",
            )
            .send()
            .await
            .expect("AC-2: request with invalid token must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: invalid session token must return HTTP 401"
        );
    }
}
