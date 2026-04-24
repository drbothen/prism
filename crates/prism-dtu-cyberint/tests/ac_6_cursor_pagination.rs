#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-6: Cursor pagination.
//!
//! Given `GET /api/v1/alerts` without cursor returns first page with `next_cursor`
//! set, then `GET /api/v1/alerts?cursor={next_cursor}` returns the second page
//! with `next_cursor: null`.
//!
//! Also covers EC-004: invalid cursor → DTU returns first page (cursor not found
//! → start from beginning).

#[cfg(feature = "dtu")]
mod ac_6 {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::CyberintClone;

    async fn start_with_token() -> (CyberintClone, String, String) {
        let mut clone = CyberintClone::new().expect("AC-6: new must succeed");
        clone.start().await.expect("AC-6: start must succeed");
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
            .expect("AC-6: Set-Cookie must be present on login")
            .to_str()
            .expect("AC-6: Set-Cookie must be ASCII")
            .to_owned();
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-6: Set-Cookie must contain cyberint_session=")
            .to_owned();

        (clone, base_url, token)
    }

    /// Page 1 has data and next_cursor set (non-null).
    #[tokio::test]
    async fn ac_6_page_1_has_data_and_next_cursor() {
        let (_clone, base_url, token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-6: GET page 1 must not error");
        assert_eq!(resp.status().as_u16(), 200, "AC-6: page 1 must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-6: body must be JSON");

        let data = body["data"].as_array().expect("AC-6: data must be array");
        assert!(
            !data.is_empty(),
            "AC-6: page 1 must contain alerts (got empty data array)"
        );

        assert!(
            !body["next_cursor"].is_null(),
            "AC-6: page 1 must include a non-null next_cursor"
        );
    }

    /// Page 2 (cursor = value from page 1) has data and next_cursor is null.
    #[tokio::test]
    async fn ac_6_page_2_has_data_and_null_next_cursor() {
        let (_clone, base_url, token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        // Fetch page 1 to get cursor.
        let page1_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-6: page 1 request must not error");
        let page1: serde_json::Value = page1_resp.json().await.expect("AC-6: page 1 must be JSON");
        let cursor = page1["next_cursor"]
            .as_str()
            .expect("AC-6: page 1 next_cursor must be a string")
            .to_owned();

        // Fetch page 2 using the cursor.
        let page2_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .query(&[("cursor", cursor.as_str())])
            .send()
            .await
            .expect("AC-6: page 2 request must not error");
        assert_eq!(
            page2_resp.status().as_u16(),
            200,
            "AC-6: page 2 must return 200"
        );

        let page2: serde_json::Value = page2_resp.json().await.expect("AC-6: page 2 must be JSON");

        let data2 = page2["data"]
            .as_array()
            .expect("AC-6: page 2 data must be array");
        assert!(
            !data2.is_empty(),
            "AC-6: page 2 must contain alerts (got empty data array)"
        );

        assert!(
            page2["next_cursor"].is_null(),
            "AC-6: page 2 must have next_cursor: null (no more pages), got: {:?}",
            page2["next_cursor"]
        );
    }

    /// Pages 1 and 2 together cover all 25 fixture alerts (20 + 5).
    #[tokio::test]
    async fn ac_6_both_pages_cover_all_fixture_alerts() {
        let (_clone, base_url, token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        let page1_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-6: page 1 must not error");
        let page1: serde_json::Value = page1_resp
            .json()
            .await
            .expect("AC-6: page 1 body must be JSON");
        let cursor = page1["next_cursor"]
            .as_str()
            .expect("AC-6: next_cursor must be a string")
            .to_owned();
        let count1 = page1["data"]
            .as_array()
            .expect("AC-6: page 1 data must be array")
            .len();

        let page2_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .query(&[("cursor", cursor.as_str())])
            .send()
            .await
            .expect("AC-6: page 2 must not error");
        let page2: serde_json::Value = page2_resp
            .json()
            .await
            .expect("AC-6: page 2 body must be JSON");
        let count2 = page2["data"]
            .as_array()
            .expect("AC-6: page 2 data must be array")
            .len();

        assert_eq!(
            count1 + count2,
            25,
            "AC-6: both pages combined must cover all 25 fixture alerts (page1={count1}, page2={count2})"
        );
    }

    /// EC-004: Invalid cursor → DTU returns first page.
    #[tokio::test]
    async fn ac_6_ec_004_invalid_cursor_returns_first_page() {
        let (_clone, base_url, token) = start_with_token().await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        let cookie = format!("cyberint_session={token}");

        // Fetch page 1 for comparison.
        let page1_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-6 EC-004: page 1 must not error");
        let page1: serde_json::Value = page1_resp
            .json()
            .await
            .expect("AC-6 EC-004: page 1 body must be JSON");
        let page1_ids: Vec<&str> = page1["data"]
            .as_array()
            .expect("AC-6 EC-004: page 1 data must be array")
            .iter()
            .map(|a| a["alert_id"].as_str().unwrap_or(""))
            .collect();

        // Fetch with an invalid cursor.
        let invalid_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .query(&[("cursor", "this-cursor-does-not-exist")])
            .send()
            .await
            .expect("AC-6 EC-004: invalid cursor request must not error");
        assert_eq!(
            invalid_resp.status().as_u16(),
            200,
            "AC-6 EC-004: invalid cursor must still return 200 (start from beginning)"
        );

        let invalid_body: serde_json::Value = invalid_resp
            .json()
            .await
            .expect("AC-6 EC-004: invalid cursor body must be JSON");
        let invalid_ids: Vec<&str> = invalid_body["data"]
            .as_array()
            .expect("EC-004: data must be array")
            .iter()
            .map(|a| a["alert_id"].as_str().unwrap_or(""))
            .collect();

        assert_eq!(
            page1_ids, invalid_ids,
            "AC-6 EC-004: invalid cursor must return the same first page as no cursor"
        );
    }
}
