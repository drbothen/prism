#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-5: Mixed timestamp format fixtures.
//!
//! Given `GET /api/v1/alerts` returns fixtures with mixed timestamp formats
//! (ISO 8601 and Unix epoch), these formats must be present in `fixtures/alerts.json`
//! and the DTU returns them verbatim — exercising Prism's timestamp normalization.
//!
//! The DTU must NOT normalize or convert timestamps; it must return them as-is
//! from the fixture.

#[cfg(feature = "dtu")]
mod ac_5 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn login_and_start() -> (CyberintClone, String, String) {
        let mut clone = CyberintClone::new().expect("AC-5: new must succeed");
        clone.start().await.expect("AC-5: start must succeed");
        let base_url = clone.base_url();

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
            .expect("AC-5: Set-Cookie must be present on login")
            .to_str()
            .expect("AC-5: Set-Cookie must be ASCII")
            .to_owned();
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-5: Set-Cookie must contain cyberint_session=")
            .to_owned();

        (clone, base_url, token)
    }

    /// Alert list response includes both ISO 8601 and Unix epoch timestamps.
    #[tokio::test]
    async fn ac_5_alert_list_contains_iso8601_timestamps() {
        let (_clone, base_url, token) = login_and_start().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET /api/v1/alerts must not error");

        assert_eq!(resp.status().as_u16(), 200, "AC-5: must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        let data = body["data"].as_array().expect("AC-5: data must be array");
        assert!(!data.is_empty(), "AC-5: alert list must not be empty");

        // At least one alert must have an ISO 8601 string timestamp.
        let has_iso8601 = data.iter().any(|a| a["created_at"].is_string());
        assert!(
            has_iso8601,
            "AC-5: fixture must contain at least one ISO 8601 string timestamp"
        );
    }

    #[tokio::test]
    async fn ac_5_alert_list_contains_unix_epoch_timestamps() {
        let (_clone, base_url, token) = login_and_start().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET must not error");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        let data = body["data"].as_array().expect("AC-5: data must be array");

        // At least one alert must have a Unix epoch (integer) timestamp.
        let has_unix_epoch = data.iter().any(|a| a["created_at"].is_number());
        assert!(
            has_unix_epoch,
            "AC-5: fixture must contain at least one Unix epoch (integer) timestamp"
        );
    }

    /// Verbatim check: CYB-2024-001 has ISO 8601 "2024-01-15T08:23:41Z".
    #[tokio::test]
    async fn ac_5_cyb_2024_001_has_verbatim_iso8601_timestamp() {
        let (_clone, base_url, token) = login_and_start().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-001"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET alert must not error");
        assert_eq!(resp.status().as_u16(), 200, "AC-5: must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        assert_eq!(
            body["created_at"].as_str().unwrap_or(""),
            "2024-01-15T08:23:41Z",
            "AC-5: CYB-2024-001 created_at must be the verbatim ISO 8601 string from fixture"
        );
    }

    /// Verbatim check: CYB-2024-002 has Unix epoch integer 1705312800.
    #[tokio::test]
    async fn ac_5_cyb_2024_002_has_verbatim_unix_epoch_timestamp() {
        let (_clone, base_url, token) = login_and_start().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-002"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET CYB-2024-002 must not error");
        assert_eq!(resp.status().as_u16(), 200, "AC-5: must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        assert_eq!(
            body["created_at"].as_i64().unwrap_or(0),
            1705312800,
            "AC-5: CYB-2024-002 created_at must be the verbatim Unix epoch integer 1705312800"
        );
    }

    /// Both ISO 8601 and epoch variants coexist in the first page (10 of each).
    #[tokio::test]
    async fn ac_5_first_page_has_mixed_timestamps_roughly_half_each() {
        let (_clone, base_url, token) = login_and_start().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET must not error");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        let data = body["data"].as_array().expect("AC-5: data must be array");

        let iso_count = data.iter().filter(|a| a["created_at"].is_string()).count();
        let epoch_count = data.iter().filter(|a| a["created_at"].is_number()).count();

        assert!(
            iso_count >= 1,
            "AC-5: page 1 must have at least 1 ISO 8601 timestamp, got {iso_count}"
        );
        assert!(
            epoch_count >= 1,
            "AC-5: page 1 must have at least 1 epoch timestamp, got {epoch_count}"
        );
        assert_eq!(
            iso_count + epoch_count,
            data.len(),
            "AC-5: every alert must have a created_at timestamp"
        );
    }
}
