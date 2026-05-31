#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-8: Reset semantics (rewritten for ADR-031 §D3-a).
//!
//! Given `reset()` is called (via POST /dtu/reset), then:
//! - All alert statuses revert to "open"
//! - Access-token allowlist is cleared (old tokens become invalid)
//! - Re-configuring a new access_token is required for subsequent authenticated requests
//!
//! Rewritten per S-DTU-CYBERINT-AUTH-FIDELITY-001 Task 11: uses
//! `Cookie: access_token=demo-key` instead of login + cyberint_session.

#[cfg(feature = "dtu")]
mod ac_8 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    const DEMO_TOKEN: &str = "demo-access-key";

    async fn start() -> (CyberintClone, String, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-8: new must succeed");
        clone.start().await.expect("AC-8: start must succeed");
        let base_url = clone.base_url();
        let admin_token = clone.admin_token().to_string();
        clone
            .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
            .await
            .expect("AC-8: configure access_token must succeed");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, admin_token, client)
    }

    /// After reset, alert statuses revert to "open".
    #[tokio::test]
    async fn ac_8_reset_reverts_alert_status_to_open() {
        let (mut clone, base_url, admin_token, client) = start().await;
        let cookie = format!("access_token={DEMO_TOKEN}");

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
        let before_body: serde_json::Value = before_reset
            .json()
            .await
            .expect("AC-8: before-reset body must be JSON");
        assert_eq!(
            before_body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-8: pre-reset status must be acknowledged"
        );

        // Reset via POST /dtu/reset.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("AC-8: POST /dtu/reset must not error");
        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "AC-8: /dtu/reset must return 200"
        );
        let reset_body: serde_json::Value = reset_resp
            .json()
            .await
            .expect("AC-8: reset body must be JSON");
        assert_eq!(
            reset_body["status"].as_str().unwrap_or(""),
            "ok",
            "AC-8: /dtu/reset must return {{status: ok}}"
        );

        // After reset, the allowlist is cleared — re-configure a new access_token.
        clone
            .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
            .await
            .expect("AC-8: re-configure access_token after reset must succeed");

        // Alert status must be back to "open".
        let after_reset = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-010"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
            .send()
            .await
            .expect("AC-8: GET after reset must succeed");
        assert_eq!(
            after_reset.status().as_u16(),
            200,
            "AC-8: GET after reset must return 200"
        );
        let after_body: serde_json::Value = after_reset
            .json()
            .await
            .expect("AC-8: after-reset body must be JSON");
        assert_eq!(
            after_body["status"].as_str().unwrap_or(""),
            "open",
            "AC-8: alert status must revert to 'open' after reset"
        );
    }

    /// After reset, the old access_token is invalidated — returns 401.
    ///
    /// ADR-031 §D3-a: reset_all() clears the access_token_allowlist.
    #[tokio::test]
    async fn ac_8_reset_clears_session_store_old_token_rejected() {
        let (_clone, base_url, admin_token, client) = start().await;
        let old_cookie = format!("access_token={DEMO_TOKEN}");

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
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // Old token must now return 401 (allowlist was cleared by reset).
        let after = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &old_cookie)
            .send()
            .await
            .expect("AC-8: post-reset request must not error");
        assert_eq!(
            after.status().as_u16(),
            401,
            "AC-8: old access_token must be rejected after reset (allowlist cleared by reset_all)"
        );
    }

    /// After reset, re-configuring a new access_token grants access.
    ///
    /// Replaces the old "new login required after reset" test: under the access_token
    /// model, reset clears the allowlist, and `configure({"access_token": ...})` re-provisions it.
    #[tokio::test]
    async fn ac_8_new_login_required_after_reset() {
        let (mut clone, base_url, admin_token, client) = start().await;

        // Reset.
        client
            .post(format!("{base_url}/dtu/reset"))
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // Re-configure a new access_token after reset.
        let new_token = "demo-access-key-after-reset";
        clone
            .configure(serde_json::json!({"access_token": new_token}))
            .await
            .expect("AC-8: configure after reset must succeed");

        // New token should grant access.
        let access = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={new_token}"))
            .send()
            .await
            .expect("AC-8: request with new token must not error");
        assert_eq!(
            access.status().as_u16(),
            200,
            "AC-8: new access_token after reset must grant access (HTTP 200)"
        );
    }

    /// Reset also reverts closed alerts back to "open".
    #[tokio::test]
    async fn ac_8_reset_reverts_closed_alert_to_open() {
        let (mut clone, base_url, admin_token, client) = start().await;
        let cookie = format!("access_token={DEMO_TOKEN}");

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
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("AC-8: reset must succeed");

        // Re-configure token after reset.
        clone
            .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
            .await
            .expect("AC-8: re-configure after reset must succeed");

        // Check alert status is now "open".
        let get_resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-012"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
            .send()
            .await
            .expect("AC-8: GET after reset must not error");
        assert_eq!(get_resp.status().as_u16(), 200);

        let body: serde_json::Value = get_resp
            .json()
            .await
            .expect("AC-8: closed-then-reset body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "open",
            "AC-8: closed alert must revert to 'open' after reset"
        );

        // PATCH should now succeed (alert is no longer closed).
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-012/status"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
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
