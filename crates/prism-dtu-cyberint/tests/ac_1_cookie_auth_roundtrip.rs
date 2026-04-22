//! AC-1: Cookie auth round-trip.
//!
//! Given `POST /login` with any body, the response is HTTP 200 with a `Set-Cookie`
//! header containing a `cyberint_session` token. Subsequent requests that include
//! this cookie receive HTTP 200.
//!
//! This test also covers EC-003: calling POST /login twice yields two distinct tokens,
//! both valid until reset().

#[cfg(feature = "dtu")]
mod ac_1 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    /// Helper: start a clone and return (clone, base_url, cookie-jar client).
    async fn start_clone() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-1: CyberintClone::new() must succeed");
        clone.start().await.expect("AC-1: start() must succeed");
        let base_url = clone.base_url();
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("AC-1: build reqwest client");
        (clone, base_url, client)
    }

    /// AC-1 primary: POST /login returns 200 with Set-Cookie header.
    #[tokio::test]
    async fn ac_1_login_returns_200_with_set_cookie_header() {
        let (_clone, base_url, client) = start_clone().await;

        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-1: POST /login must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-1: POST /login must return HTTP 200"
        );

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("AC-1: Set-Cookie header must be present on login response");
        let cookie_str = set_cookie.to_str().expect("AC-1: Set-Cookie must be ASCII");
        assert!(
            cookie_str.contains("cyberint_session="),
            "AC-1: Set-Cookie must contain cyberint_session=, got: {cookie_str}"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("AC-1: login response must be valid JSON");
        assert_eq!(
            body["message"].as_str().unwrap_or(""),
            "Login successful",
            "AC-1: login body must contain message: Login successful"
        );
    }

    /// AC-1 secondary: after login, authenticated GET /api/v1/alerts with cookie returns 200.
    #[tokio::test]
    async fn ac_1_authenticated_request_after_login_returns_200() {
        let (_clone, base_url, client) = start_clone().await;

        // Perform login — cookie_store(true) will capture the Set-Cookie.
        let login_resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-1: login request must succeed");
        assert_eq!(
            login_resp.status().as_u16(),
            200,
            "AC-1: login must return 200"
        );

        // Extract session token from Set-Cookie to use manually (cookie_store may not
        // persist across different reqwest::Client instances, so we also test manually).
        let set_cookie = login_resp
            .headers()
            .get("set-cookie")
            .expect("AC-1: Set-Cookie must be present")
            .to_str()
            .expect("AC-1: Set-Cookie must be ASCII");
        // Parse token: "cyberint_session=<token>; Path=/; HttpOnly"
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-1: Set-Cookie must contain cyberint_session=");

        // Use token in Cookie header directly.
        let no_jar_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build bare client");

        let alerts_resp = no_jar_client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-1: GET /api/v1/alerts with valid cookie must not error");

        assert_eq!(
            alerts_resp.status().as_u16(),
            200,
            "AC-1: GET /api/v1/alerts with valid session cookie must return HTTP 200"
        );
    }

    /// EC-003: calling POST /login twice yields two distinct tokens, both valid.
    #[tokio::test]
    async fn ac_1_ec_003_two_logins_yield_distinct_valid_tokens() {
        let (_clone, base_url, _) = start_clone().await;

        let bare = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp1 = bare
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("EC-003: first login must succeed");
        let cookie1 = resp1
            .headers()
            .get("set-cookie")
            .expect("EC-003: Set-Cookie on first login")
            .to_str()
            .unwrap()
            .to_owned();

        let resp2 = bare
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("EC-003: second login must succeed");
        let cookie2 = resp2
            .headers()
            .get("set-cookie")
            .expect("EC-003: Set-Cookie on second login")
            .to_str()
            .unwrap()
            .to_owned();

        assert_ne!(
            cookie1, cookie2,
            "EC-003: two logins must produce distinct session tokens"
        );

        // Extract both tokens and verify both are accepted.
        let token1 = cookie1
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("EC-003: parse token1")
            .to_owned();
        let token2 = cookie2
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("EC-003: parse token2")
            .to_owned();

        for (label, token) in [("token1", token1.as_str()), ("token2", token2.as_str())] {
            let r = bare
                .get(format!("{base_url}/api/v1/alerts"))
                .header("Cookie", format!("cyberint_session={token}"))
                .send()
                .await
                .expect("EC-003: request with {label} must send");
            assert_eq!(
                r.status().as_u16(),
                200,
                "EC-003: {label} must be a valid session token (got {} instead of 200)",
                r.status()
            );
        }
    }
}
