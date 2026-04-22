//! Edge cases per story S-6.09 edge case catalog (EC-001 through EC-006).
//!
//! EC-001 and EC-004 are co-located in ac_4 and ac_6 respectively because they
//! are tightly bound to those AC flows. The remaining edge cases live here.

#[cfg(feature = "dtu")]
mod edge_cases {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn start() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("edge_cases: new must succeed");
        clone.start().await.expect("edge_cases: start must succeed");
        let base_url = clone.base_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, client)
    }

    async fn login(base_url: &str, client: &reqwest::Client) -> String {
        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login must succeed");
        resp.headers()
            .get("set-cookie")
            .expect("edge_cases: Set-Cookie must be present on login")
            .to_str()
            .expect("edge_cases: Set-Cookie must be ASCII")
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("edge_cases: Set-Cookie must contain cyberint_session=")
            .to_owned()
    }

    /// EC-002: GET /api/v1/alerts/{alert_id} for unknown alert_id returns HTTP 404
    /// with {"error": "alert not found"}.
    #[tokio::test]
    async fn ec_002_unknown_alert_id_returns_404() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts/NONEXISTENT-9999"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("EC-002: GET for unknown alert must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-002: unknown alert_id must return HTTP 404"
        );

        let body: serde_json::Value = resp.json().await.expect("EC-002: 404 body must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "alert not found",
            "EC-002: error message must be 'alert not found'"
        );
    }

    /// EC-002: PATCH /status for unknown alert_id also returns 404.
    #[tokio::test]
    async fn ec_002_patch_unknown_alert_returns_404() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;

        let resp = client
            .patch(format!("{base_url}/api/v1/alerts/NONEXISTENT-0000/status"))
            .header("Cookie", format!("cyberint_session={token}"))
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("EC-002: PATCH for unknown alert must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-002: PATCH on unknown alert_id must return HTTP 404"
        );
    }

    /// EC-002: POST /close for unknown alert_id returns 404.
    #[tokio::test]
    async fn ec_002_close_unknown_alert_returns_404() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/NONEXISTENT-1234/close"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("EC-002: close for unknown alert must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-002: POST /close on unknown alert_id must return HTTP 404"
        );
    }

    /// EC-005: Out-of-scope endpoint /api/v1/digital-risk/findings returns 404.
    #[tokio::test]
    async fn ec_005_out_of_scope_endpoint_returns_404() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/digital-risk/findings"))
            .send()
            .await
            .expect("EC-005: request to out-of-scope endpoint must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-005: /api/v1/digital-risk/findings must return HTTP 404 (not in scope)"
        );
    }

    /// EC-005: Another out-of-scope path also returns 404.
    #[tokio::test]
    async fn ec_005_another_out_of_scope_endpoint_returns_404() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/api/v1/indicators"))
            .send()
            .await
            .expect("EC-005: request must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-005: /api/v1/indicators must return HTTP 404 (not in DTU scope)"
        );
    }

    /// EC-006: auth_mode=reject causes all authenticated requests to return 401
    /// regardless of valid cookie.
    #[tokio::test]
    async fn ec_006_auth_mode_reject_returns_401_for_valid_cookie() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Verify token works before auth_mode change.
        let before = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: request before configure must not error");
        assert_eq!(
            before.status().as_u16(),
            200,
            "EC-006: valid cookie must work before auth_mode=reject"
        );

        // Set auth_mode=reject.
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({"auth_mode": "reject"}))
            .send()
            .await
            .expect("EC-006: configure must not error");
        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "EC-006: configure must return 200"
        );

        // All authenticated requests now return 401 regardless of cookie.
        let after = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: post-configure request must not error");
        assert_eq!(
            after.status().as_u16(),
            401,
            "EC-006: auth_mode=reject must return 401 even with a valid session cookie"
        );

        let body: serde_json::Value = after.json().await.expect("EC-006: 401 body must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "unauthorized",
            "EC-006: error field must be 'unauthorized'"
        );
    }

    /// EC-006: auth_mode=reject also applies to /api/v1/threat-intel.
    #[tokio::test]
    async fn ec_006_auth_mode_reject_applies_to_threat_intel() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({"auth_mode": "reject"}))
            .send()
            .await
            .expect("EC-006: configure must succeed");

        let resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: threat-intel request must not error");
        assert_eq!(
            resp.status().as_u16(),
            401,
            "EC-006: auth_mode=reject must return 401 for /api/v1/threat-intel too"
        );
    }

    /// EC-006: auth_mode=accept restores normal operation after reject.
    #[tokio::test]
    async fn ec_006_auth_mode_accept_restores_normal_operation() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Set reject, then restore accept.
        client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({"auth_mode": "reject"}))
            .send()
            .await
            .expect("EC-006: configure reject must not error");
        client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("EC-006: configure accept must not error");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: request after accept must not error");
        assert_eq!(
            resp.status().as_u16(),
            200,
            "EC-006: auth_mode=accept must restore normal operation (200)"
        );
    }

    /// DTU /dtu/health is always accessible without auth and returns {"status": "ok"}.
    #[tokio::test]
    async fn dtu_health_endpoint_accessible_without_auth() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .get(format!("{base_url}/dtu/health"))
            .send()
            .await
            .expect("health check must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "/dtu/health must return HTTP 200 without auth"
        );

        let body: serde_json::Value = resp.json().await.expect("health body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "ok",
            "/dtu/health must return {{\"status\": \"ok\"}}"
        );
    }

    /// Unknown JSON keys in POST /dtu/configure are silently ignored (ADR-002 §5).
    #[tokio::test]
    async fn dtu_configure_unknown_keys_silently_ignored() {
        let (_clone, base_url, client) = start().await;

        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({
                "unknown_field": "some_value",
                "another_unknown": 42
            }))
            .send()
            .await
            .expect("configure with unknown keys must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "POST /dtu/configure with unknown keys must return 200 (silently ignored)"
        );
    }

    /// Threat intel endpoint returns data and next_cursor fields.
    #[tokio::test]
    async fn threat_intel_returns_data_and_next_cursor_fields() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;

        let resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("GET /api/v1/threat-intel must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "/api/v1/threat-intel must return 200"
        );

        let body: serde_json::Value = resp.json().await.expect("body must be JSON");
        assert!(
            body.get("data").is_some(),
            "/api/v1/threat-intel must include 'data' field"
        );
        assert!(
            body.get("next_cursor").is_some(),
            "/api/v1/threat-intel must include 'next_cursor' field"
        );

        let data = body["data"].as_array().expect("data must be array");
        assert_eq!(
            data.len(),
            15,
            "/api/v1/threat-intel must return all 15 threat fixture items on first page"
        );
    }
}
