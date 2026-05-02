#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-7: FailureMode::RateLimit via POST /dtu/configure.
//!
//! Given `FailureMode::RateLimit` configured via `POST /dtu/configure`, when the
//! threshold is exceeded, the response is HTTP 429 — maps to E-SENSOR-003.

#[cfg(feature = "dtu")]
mod ac_7 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    /// Returns (clone, base_url, session_token, admin_token).
    async fn start_with_token() -> (CyberintClone, String, String, String) {
        let mut clone = CyberintClone::new().expect("AC-7: new must succeed");
        clone.start().await.expect("AC-7: start must succeed");
        let base_url = clone.base_url();
        let admin_token = clone.admin_token().to_string();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let login_resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login must succeed");
        let set_cookie = login_resp
            .headers()
            .get("set-cookie")
            .expect("AC-7: Set-Cookie must be present on login")
            .to_str()
            .expect("AC-7: Set-Cookie must be ASCII")
            .to_owned();
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-7: Set-Cookie must contain cyberint_session=")
            .to_owned();

        (clone, base_url, token, admin_token)
    }

    /// Configure rate_limit_after=1; second request returns 429.
    #[tokio::test]
    async fn ac_7_rate_limit_429_after_threshold_exceeded() {
        let (_clone, base_url, token, admin_token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        // Configure rate limit: allow 1 request then 429.
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"rate_limit_after": 1}))
            .send()
            .await
            .expect("AC-7: POST /dtu/configure must not error");
        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "AC-7: configure must return 200"
        );

        // First request — should succeed (count becomes 1, threshold is 1, 1 > 1 is false).
        let first_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: first request must not error");
        assert_eq!(
            first_resp.status().as_u16(),
            200,
            "AC-7: first request (within threshold) must return 200"
        );

        // Second request — count becomes 2, 2 > 1 is true → 429.
        let second_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: second request must not error");
        assert_eq!(
            second_resp.status().as_u16(),
            429,
            "AC-7: second request past threshold must return HTTP 429"
        );

        let body: serde_json::Value = second_resp
            .json()
            .await
            .expect("AC-7: 429 body must be JSON");
        assert!(
            !body["error"].is_null(),
            "AC-7: HTTP 429 response must include an error field"
        );
    }

    /// Rate limit response body includes error field (maps to E-SENSOR-003).
    #[tokio::test]
    async fn ac_7_rate_limit_response_includes_error_field() {
        let (_clone, base_url, token, admin_token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        // Set rate limit to 0: every request triggers 429.
        client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"rate_limit_after": 0}))
            .send()
            .await
            .expect("AC-7: configure must not error");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: rate-limited request must not error");
        assert_eq!(
            resp.status().as_u16(),
            429,
            "AC-7: rate_limit_after=0 must immediately return 429 on first auth request"
        );

        let body: serde_json::Value = resp.json().await.expect("AC-7: body must be JSON");
        let error_msg = body["error"].as_str().unwrap_or("");
        assert!(
            !error_msg.is_empty(),
            "AC-7: HTTP 429 body must include a non-empty error message"
        );
    }

    /// After reset(), rate limit counter resets — requests succeed again.
    #[tokio::test]
    async fn ac_7_rate_limit_resets_after_dtu_reset() {
        let (_clone, base_url, token, admin_token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        // Configure limit=0, exhausting all new requests.
        client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"rate_limit_after": 0}))
            .send()
            .await
            .expect("AC-7 reset: configure must not error");

        // Verify it's rate-limited.
        let limited = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-7 reset: limited request must not error");
        assert_eq!(
            limited.status().as_u16(),
            429,
            "AC-7 reset: must be 429 before reset"
        );

        // Reset the DTU.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("AC-7 reset: POST /dtu/reset must not error");
        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "AC-7 reset: /dtu/reset must return 200"
        );

        // After reset, need to re-login (session store cleared by reset).
        let new_login = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-7 reset: re-login must succeed");
        let new_set_cookie = new_login
            .headers()
            .get("set-cookie")
            .expect("AC-7 reset: Set-Cookie must be present after re-login")
            .to_str()
            .expect("AC-7 reset: Set-Cookie must be ASCII")
            .to_owned();
        let new_token = new_set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-7 reset: Set-Cookie must contain cyberint_session=")
            .to_owned();

        // After reset, rate_limit_after is None → first request succeeds.
        let after_reset = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={new_token}"))
            .send()
            .await
            .expect("AC-7 reset: post-reset request must not error");
        assert_eq!(
            after_reset.status().as_u16(),
            200,
            "AC-7 reset: first request after reset must succeed (rate limit cleared)"
        );
    }

    /// Rate limit also applies to PATCH, POST /close, and GET /threat-intel.
    #[tokio::test]
    async fn ac_7_rate_limit_applies_to_threat_intel_endpoint() {
        let (_clone, base_url, token, admin_token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        // rate_limit_after=0: first authenticated request returns 429.
        client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"rate_limit_after": 0}))
            .send()
            .await
            .expect("AC-7: configure must not error");

        let resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: threat-intel rate limit request must not error");
        assert_eq!(
            resp.status().as_u16(),
            429,
            "AC-7: rate limit must apply to /api/v1/threat-intel"
        );
    }
}
