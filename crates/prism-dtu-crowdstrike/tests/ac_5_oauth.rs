//! AC-5: OAuth2 token endpoint returns fixed bearer token (S-6.07).
//!
//! Given `POST /oauth2/token` is called with any `client_credentials` body,
//! Then the response is HTTP 200 with `access_token: "dtu-fake-cs-token"`.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-5: POST /oauth2/token returns 200 with the static fake access token.
#[tokio::test]
async fn ac_5_oauth_token_returns_200_with_fake_cs_token() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-5: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-client-id",
            "client_secret": "test-client-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5: oauth token request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-5: POST /oauth2/token must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: body must be valid JSON");

    assert_eq!(
        body["access_token"].as_str().unwrap_or(""),
        "dtu-fake-cs-token",
        "AC-5: access_token must be 'dtu-fake-cs-token'"
    );
    assert_eq!(
        body["token_type"].as_str().unwrap_or("").to_lowercase(),
        "bearer",
        "AC-5: token_type must be 'bearer'"
    );
    assert_eq!(
        body["expires_in"].as_u64().unwrap_or(0),
        3600,
        "AC-5: expires_in must be 3600"
    );
}

/// AC-5: Token works for a subsequent authenticated request.
#[tokio::test]
async fn ac_5_token_from_oauth_works_on_authenticated_endpoint() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-5 use: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Get the token.
    let token_resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-id",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5 use: token request must reach server");

    assert_eq!(
        token_resp.status().as_u16(),
        200,
        "AC-5 use: token must be 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("AC-5 use: token body must be JSON");
    let token = token_body["access_token"]
        .as_str()
        .expect("AC-5 use: access_token must be string");

    // Use the token on an authenticated endpoint.
    let auth_resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("AC-5 use: authenticated request must reach server");

    assert_eq!(
        auth_resp.status().as_u16(),
        200,
        "AC-5 use: token obtained from /oauth2/token must authenticate successfully on detection endpoint"
    );
}

/// AC-5: reject mode returns 401 when auth_mode = "reject".
#[tokio::test]
async fn ac_5_oauth_reject_mode_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("AC-5 reject: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Configure auth_mode = "reject".
    clone
        .configure(serde_json::json!({"auth_mode": "reject"}))
        .await
        .expect("AC-5 reject: configure must succeed");

    let resp = client
        .post(format!("{base_url}/oauth2/token"))
        .json(&serde_json::json!({
            "client_id": "test-id",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .expect("AC-5 reject: token request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-5 reject: auth_mode='reject' must return HTTP 401 from token endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5 reject: body must be JSON");

    let errors = body["errors"]
        .as_array()
        .expect("AC-5 reject: errors must be array");
    assert!(
        !errors.is_empty(),
        "AC-5 reject: errors array must not be empty"
    );
    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        401,
        "AC-5 reject: error code must be 401"
    );
}
