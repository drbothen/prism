//! AC-8: Reset semantics.
//!
//! Given `reset()` is called (via POST /dtu/reset), then:
//! - All alert statuses revert to "open"
//! - Session store is cleared (old tokens become invalid)
//! - A new login is required for subsequent authenticated requests

#[cfg(feature = "dtu")]
mod ac_8 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn start() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-8: new must succeed");
        clone.start().await.expect("AC-8: start must succeed");
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
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .unwrap()
            .to_owned()
    }

    /// After reset, alert statuses revert to "open".
    #[tokio::test]
    async fn ac_8_reset_reverts_alert_status_to_open() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Acknowledge an alert to change its status.
        client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-010/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-8: PATCH must succeed");

        // Verify it's acknowledged before reset.
        let before_reset = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-010"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-8: GET before reset must succeed");
        let before_body: serde_json::Value = before_reset.json().await.unwrap();
        assert_eq!(
            before_body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-8: pre-reset status must be acknowledged"
        );

        // Reset via POST /dtu/reset.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .send()
            .await
            .expect("AC-8: POST /dtu/reset must not error");
        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "AC-8: /dtu/reset must return 200"
        );
        let reset_body: serde_json::Value = reset_resp.json().await.unwrap();
        assert_eq!(
            reset_body["status"].as_str().unwrap_or(""),
            "ok",
            "AC-8: /dtu/reset must return {{status: ok}}"
        );

        // After reset, need a new login (old token is invalid).
        let new_token = login(&base_url, &client).await;
        let new_cookie = format!("cyberint_session={new_token}");

        // Alert status must be back to "open".
        let after_reset = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-010"))
            .header("Cookie", &new_cookie)
            .send()
            .await
            .expect("AC-8: GET after reset must succeed");
        assert_eq!(
            after_reset.status().as_u16(),
            200,
            "AC-8: GET after reset must return 200"
        );
        let after_body: serde_json::Value = after_reset.json().await.unwrap();
        assert_eq!(
            after_body["status"].as_str().unwrap_or(""),
            "open",
            "AC-8: alert status must revert to 'open' after reset"
        );
    }

    /// After reset, the old session token is invalidated — returns 401.
    #[tokio::test]
    async fn ac_8_reset_clears_session_store_old_token_rejected() {
        let (_clone, base_url, client) = start().await;
        let old_token = login(&base_url, &client).await;
        let old_cookie = format!("cyberint_session={old_token}");

        // Verify old token works before reset.
        let before = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &old_cookie)
            .send()
            .await
            .expect("AC-8: pre-reset request must not error");
        assert_eq!(
            before.status().as_u16(),
            200,
            "AC-8: old token must work before reset"
        );

        // Reset.
        client
            .post(format!("{base_url}/dtu/reset"))
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // Old token must now return 401.
        let after = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &old_cookie)
            .send()
            .await
            .expect("AC-8: post-reset request must not error");
        assert_eq!(
            after.status().as_u16(),
            401,
            "AC-8: old session token must be rejected after reset (session store cleared)"
        );
    }

    /// After reset, a new login is required and works.
    #[tokio::test]
    async fn ac_8_new_login_required_after_reset() {
        let (_clone, base_url, client) = start().await;

        // Login and reset.
        login(&base_url, &client).await;
        client
            .post(format!("{base_url}/dtu/reset"))
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // New login should succeed.
        let new_token = login(&base_url, &client).await;
        assert!(
            !new_token.is_empty(),
            "AC-8: new login after reset must return a valid token"
        );

        // New token should grant access.
        let access = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={new_token}"))
            .send()
            .await
            .expect("AC-8: request with new token must not error");
        assert_eq!(
            access.status().as_u16(),
            200,
            "AC-8: new token after reset must grant access (HTTP 200)"
        );
    }

    /// Reset also reverts closed alerts back to "open".
    #[tokio::test]
    async fn ac_8_reset_reverts_closed_alert_to_open() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Close an alert.
        client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-012/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-8: close must succeed");

        // Reset.
        client
            .post(format!("{base_url}/dtu/reset"))
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // Re-login and check alert status.
        let new_token = login(&base_url, &client).await;
        let get_resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-012"))
            .header("Cookie", format!("cyberint_session={new_token}"))
            .send()
            .await
            .expect("AC-8: GET after reset must not error");
        assert_eq!(get_resp.status().as_u16(), 200);

        let body: serde_json::Value = get_resp.json().await.unwrap();
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "open",
            "AC-8: closed alert must revert to 'open' after reset"
        );

        // PATCH should now succeed (alert is no longer closed).
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-012/status"))
            .header("Cookie", format!("cyberint_session={new_token}"))
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-8: PATCH after reset must not error");
        assert_eq!(
            patch_resp.status().as_u16(),
            200,
            "AC-8: PATCH after reset must succeed (alert is open again, not closed)"
        );
    }
}
