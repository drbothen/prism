// AC-6: When auth_mode=reject is configured via POST /dtu/configure, any request
// bearing an apiKey parameter returns HTTP 403 {"error": "Forbidden. apiKey not verified."}.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn ac_6_auth_mode_reject_returns_403_for_any_api_key() {
    let mut clone = NvdClone::new().expect("AC-6: NvdClone::new() must succeed");
    clone.start().await.expect("AC-6: start() must succeed");

    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = reqwest::Client::new();

    // Configure auth_mode=reject.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({ "auth_mode": "reject" }))
        .send()
        .await
        .expect("AC-6: POST /dtu/configure must be reachable");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-6: configure must return 200"
    );

    // Any request with apiKey must now get HTTP 403.
    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "any-key")])
        .send()
        .await
        .expect("AC-6: request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-6: auth_mode=reject must return HTTP 403 for any apiKey"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-6: body must be valid JSON");
    assert_eq!(
        body["error"].as_str().unwrap_or(""),
        "Forbidden. apiKey not verified.",
        "AC-6: error message must match canonical text"
    );
}

#[tokio::test]
async fn ac_6_auth_mode_reject_does_not_affect_unauthenticated_requests() {
    let mut clone = NvdClone::new().expect("AC-6b: NvdClone::new() must succeed");
    clone.start().await.expect("AC-6b: start() must succeed");

    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = reqwest::Client::new();

    // Configure auth_mode=reject.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({ "auth_mode": "reject" }))
        .send()
        .await
        .expect("AC-6b: configure must succeed");

    // Unauthenticated request (no apiKey) must NOT get 403 for auth rejection.
    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001")])
        .send()
        .await
        .expect("AC-6b: unauthenticated request must be sent");

    // Should be 200 (CVE found) — not a 403 from auth_mode=reject.
    assert_ne!(
        resp.status().as_u16(),
        403,
        "AC-6b: auth_mode=reject must only reject requests bearing an apiKey"
    );
}
