#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-3: Stateful alert status transition.
//!
//! Given `PATCH /api/v1/alerts/{alert_id}/status` with `{"status": "acknowledged"}`,
//! the response is HTTP 200 AND subsequent `GET /api/v1/alerts/{alert_id}` returns
//! the alert with `status: "acknowledged"` — stateful transition persists.

#[cfg(feature = "dtu")]
mod ac_3 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    /// Perform login and return the session token string.
    async fn login(base_url: &str, client: &reqwest::Client) -> String {
        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login must succeed");
        assert_eq!(resp.status().as_u16(), 200, "login must return 200");

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present on login")
            .to_str()
            .expect("Set-Cookie must be ASCII")
            .to_owned();
        set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("Set-Cookie must contain cyberint_session=")
            .to_owned()
    }

    async fn start() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-3: new must succeed");
        clone.start().await.expect("AC-3: start must succeed");
        let base_url = clone.base_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, client)
    }

    /// PATCH status returns 200 with updated alert_id + status.
    #[tokio::test]
    async fn ac_3_patch_status_returns_200_with_acknowledged() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;

        let resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-001/status"))
            .header("Cookie", format!("cyberint_session={token}"))
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-3: PATCH must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-3: PATCH status must return HTTP 200"
        );

        let body: serde_json::Value = resp.json().await.expect("AC-3: body must be JSON");
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-001",
            "AC-3: response must include alert_id"
        );
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-3: response status must be acknowledged"
        );
    }

    /// After PATCH, subsequent GET returns the updated status (state persists).
    #[tokio::test]
    async fn ac_3_status_persists_after_patch() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // PATCH to acknowledge
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-003/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-3: PATCH must not error");
        assert_eq!(
            patch_resp.status().as_u16(),
            200,
            "AC-3: PATCH must return 200"
        );

        // GET the alert and verify status is acknowledged
        let get_resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-003"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-3: GET must not error");
        assert_eq!(get_resp.status().as_u16(), 200, "AC-3: GET must return 200");

        let body: serde_json::Value = get_resp.json().await.expect("AC-3: GET body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-3: status must be acknowledged after PATCH (state must persist)"
        );
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-003",
            "AC-3: alert_id must match the patched alert"
        );
    }

    /// Alert list also reflects the updated status.
    #[tokio::test]
    async fn ac_3_alert_list_reflects_updated_status() {
        let (_clone, base_url, client) = start().await;
        let token = login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // PATCH CYB-2024-005 to acknowledged
        client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-005/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-3: PATCH must succeed");

        // GET list (no cursor — page 1 contains CYB-2024-005)
        let list_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-3: list request must not error");
        assert_eq!(
            list_resp.status().as_u16(),
            200,
            "AC-3: list must return 200"
        );

        let body: serde_json::Value = list_resp
            .json()
            .await
            .expect("AC-3: list body must be JSON");
        let data = body["data"]
            .as_array()
            .expect("AC-3: data must be an array");

        let alert = data
            .iter()
            .find(|a| a["alert_id"].as_str() == Some("CYB-2024-005"))
            .expect("AC-3: CYB-2024-005 must be in the list");
        assert_eq!(
            alert["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-3: alert list must show updated status for CYB-2024-005"
        );
    }
}
