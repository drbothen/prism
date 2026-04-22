//! AC-4: Irreversible close semantics.
//!
//! Given `POST /api/v1/alerts/{alert_id}/close`, the response is HTTP 200 with
//! `status: "closed"` AND subsequent `PATCH` for acknowledge returns HTTP 400
//! `{"error": "alert already closed"}`.
//!
//! Also covers EC-001: closing an already-closed alert returns 400 (idempotent check).

#[cfg(feature = "dtu")]
mod ac_4 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn start() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-4: new must succeed");
        clone.start().await.expect("AC-4: start must succeed");
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
        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present")
            .to_str()
            .expect("Set-Cookie must be ASCII")
            .to_owned();
        set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("parse token")
            .to_owned()
    }

    /// POST /close returns 200 with status: "closed".
    #[tokio::test]
    async fn ac_4_close_alert_returns_200_with_closed_status() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-002/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: POST /close must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-4: POST /close must return HTTP 200"
        );

        let body: serde_json::Value = resp.json().await.expect("AC-4: close body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "closed",
            "AC-4: close response must include status: closed"
        );
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-002",
            "AC-4: close response must include the alert_id"
        );
    }

    /// After close, GET returns status: "closed".
    #[tokio::test]
    async fn ac_4_get_after_close_returns_closed_status() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-004/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: close must succeed");

        let get_resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-004"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: GET must not error");
        assert_eq!(get_resp.status().as_u16(), 200, "AC-4: GET must return 200");

        let body: serde_json::Value = get_resp.json().await.expect("AC-4: body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "closed",
            "AC-4: GET after close must return status: closed"
        );
    }

    /// After close, PATCH acknowledge returns 400 {"error": "alert already closed"}.
    #[tokio::test]
    async fn ac_4_patch_after_close_returns_400_already_closed() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Close first.
        client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-006/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: close must succeed");

        // Now try to acknowledge the closed alert.
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-006/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-4: PATCH after close must not error");

        assert_eq!(
            patch_resp.status().as_u16(),
            400,
            "AC-4: PATCH on closed alert must return HTTP 400"
        );

        let body: serde_json::Value = patch_resp
            .json()
            .await
            .expect("AC-4: 400 response must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "alert already closed",
            "AC-4: error message must be 'alert already closed'"
        );
    }

    /// EC-001: closing an already-closed alert also returns 400.
    #[tokio::test]
    async fn ac_4_ec_001_close_already_closed_returns_400() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // First close.
        let first_close = client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-008/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-001: first close must not error");
        assert_eq!(
            first_close.status().as_u16(),
            200,
            "EC-001: first close must return 200"
        );

        // Second close on the same alert.
        let second_close = client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-008/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-001: second close must not error");
        assert_eq!(
            second_close.status().as_u16(),
            400,
            "EC-001: closing an already-closed alert must return HTTP 400"
        );

        let body: serde_json::Value = second_close
            .json()
            .await
            .expect("EC-001: 400 body must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "alert already closed",
            "EC-001: error field must be 'alert already closed'"
        );
    }
}
