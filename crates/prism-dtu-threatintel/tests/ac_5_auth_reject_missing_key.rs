#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-5: Given both key query param and Authorization header absent,
// the response is HTTP 401 {"error": "missing API key", "code": 401}
// mapping to E-INFUSION-AUTH-001.
//
// Expected Red Gate failure: the stub auth check is implemented and returns
// the correct 401 shape. However, this test also asserts that the numeric
// "code" field in the body equals 401 as an integer (not a string), and that
// a second request with only a whitespace-only Authorization Bearer also gets 401.
// The whitespace-bearer edge case is not handled by the stub (a "Bearer " prefix
// with trailing spaces would pass the `v.len() > 7` check), causing the second
// assertion to fail.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

#[tokio::test]
async fn ac_5_missing_api_key_returns_401_with_error_body() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-5: start() must succeed");

    let base = clone.base_url();
    let client = build_test_client();

    // Case 1: no key param, no Authorization header.
    let resp = client
        .get(format!("{base}/v3/ip/8.8.8.8"))
        .send()
        .await
        .expect("AC-5: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-5: missing API key must return HTTP 401"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-5: 401 response must be valid JSON");

    let error_msg = body
        .get("error")
        .and_then(|v| v.as_str())
        .expect("AC-5: 401 body must contain 'error' string field");

    assert_eq!(
        error_msg, "missing API key",
        "AC-5: error message must be 'missing API key' (E-INFUSION-AUTH-001)"
    );

    let code = body
        .get("code")
        .and_then(|v| v.as_u64())
        .expect("AC-5: 401 body must contain numeric 'code' field");

    assert_eq!(code, 401, "AC-5: 'code' field must be 401 (numeric)");

    // Case 2: Authorization header with empty bearer token (whitespace only).
    // A real implementation must reject "Bearer   " (only spaces after prefix).
    let resp2 = client
        .get(format!("{base}/v3/ip/8.8.8.8"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .expect("AC-5: empty-bearer request must reach server");

    assert_eq!(
        resp2.status().as_u16(),
        401,
        "AC-5: empty Bearer token (only spaces) must return HTTP 401"
    );
}
