#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-3: Stateful alert status transition (rewritten for ADR-031 §D3-a).
//!
//! Given `PATCH /api/v1/alerts/{alert_id}/status` with `{"status": "acknowledged"}`,
//! the response is HTTP 200 AND subsequent `GET /api/v1/alerts/{alert_id}` returns
//! the alert with `status: "acknowledged"` — stateful transition persists.
//!
//! Rewritten per S-DTU-CYBERINT-AUTH-FIDELITY-001 Task 11: uses
//! `Cookie: access_token=demo-key` instead of login + cyberint_session.

#[cfg(feature = "dtu")]
mod ac_3 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    const DEMO_TOKEN: &str = "demo-access-key";

    async fn start_with_demo_token() -> (CyberintClone, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("AC-3: new must succeed");
        clone.start().await.expect("AC-3: start must succeed");
        let base_url = clone.base_url();
        clone
            .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
            .await
            .expect("AC-3: configure must succeed");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, client)
    }

    /// PATCH status returns 200 with updated alert_id + status.
    #[tokio::test]
    async fn ac_3_patch_status_returns_200_with_acknowledged() {
        let (_clone, base_url, client) = start_with_demo_token().await;

        let resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-001/status"))
            .header("Cookie", format!("access_token={DEMO_TOKEN}"))
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
        let (_clone, base_url, client) = start_with_demo_token().await;
        let cookie = format!("access_token={DEMO_TOKEN}");

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
        let (_clone, base_url, client) = start_with_demo_token().await;
        let cookie = format!("access_token={DEMO_TOKEN}");

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
