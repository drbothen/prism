// AC-1: Given `GET /api/v1/devices?aql=in:type%3Dswitch` with a valid Bearer token,
// Then the response is HTTP 200 with a `data.devices` array AND the received AQL string
// `"in:type=switch"` is logged in `GET /dtu/aql-log` (AQL capture works).
//
// Also covers:
//   EC-001 — AQL string with special characters stored verbatim (no parsing/escaping).
//   EC-004 — Pagination page beyond last → empty devices array with correct total.
//   EC-005 — Both GET and POST methods accepted for `/api/v1/devices`.
//
// Red Gate: these tests will fail until:
//   - Bearer auth middleware is wired for both GET and POST `/api/v1/devices`.
//   - AQL capture works for both GET query-param and POST body variants.
//   - `GET /dtu/aql-log` returns the captured strings.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_1_get_devices_with_aql_returns_200_and_logs_aql() {
    let mut clone = ArmisClone::new().expect("AC-1: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-1: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // GET /api/v1/devices?aql=in:type=switch with valid Bearer token.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("aql", "in:type=switch")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-1: GET /api/v1/devices request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: GET /api/v1/devices with valid Bearer must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-1: response body must be valid JSON");

    assert!(
        body["data"]["devices"].is_array(),
        "AC-1: response must contain data.devices array, got: {body}"
    );

    // Verify AQL was captured in the log.
    let aql_resp = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-1: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_resp.status().as_u16(),
        200,
        "AC-1: GET /dtu/aql-log must return HTTP 200"
    );

    let aql_body: serde_json::Value = aql_resp
        .json()
        .await
        .expect("AC-1: aql-log response must be valid JSON");

    let aql_strings = aql_body["aql_strings"]
        .as_array()
        .expect("AC-1: aql_strings must be an array");

    assert!(
        aql_strings
            .iter()
            .any(|s| s.as_str() == Some("in:type=switch")),
        "AC-1: AQL log must contain 'in:type=switch', got: {aql_strings:?}"
    );
}

#[tokio::test]
async fn ac_1_post_devices_with_aql_body_returns_200_and_logs_aql() {
    // EC-005: POST /api/v1/devices is also supported by Armis.
    // AQL in JSON body must be captured, not just query-param AQL.
    let mut clone = ArmisClone::new().expect("AC-1/EC-005: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-1/EC-005: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "aql": "in:type=plc" }))
        .send()
        .await
        .expect("AC-1/EC-005: POST /api/v1/devices must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1/EC-005: POST /api/v1/devices with valid Bearer must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-1/EC-005: body must be valid JSON");

    assert!(
        body["data"]["devices"].is_array(),
        "AC-1/EC-005: data.devices must be array, got: {body}"
    );

    // AQL from POST body must be captured.
    let aql_resp = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-1/EC-005: GET /dtu/aql-log must succeed");

    let aql_body: serde_json::Value = aql_resp
        .json()
        .await
        .expect("AC-1/EC-005: aql-log must be valid JSON");

    let aql_strings = aql_body["aql_strings"]
        .as_array()
        .expect("AC-1/EC-005: aql_strings must be array");

    assert!(
        aql_strings
            .iter()
            .any(|s| s.as_str() == Some("in:type=plc")),
        "AC-1/EC-005: AQL from POST body must be in log, got: {aql_strings:?}"
    );
}

#[tokio::test]
async fn ec_001_aql_special_characters_stored_verbatim() {
    // EC-001: AQL strings with special characters (<, >, =) are stored verbatim.
    let mut clone = ArmisClone::new().expect("EC-001: ArmisClone::new() must succeed");
    clone.start().await.expect("EC-001: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let special_aql = "risk_score>80 AND type=switch AND name<Z";

    let _resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("aql", special_aql)])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-001: GET request with special AQL must succeed");

    let aql_resp = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("EC-001: GET /dtu/aql-log must succeed");

    let aql_body: serde_json::Value = aql_resp
        .json()
        .await
        .expect("EC-001: aql-log body must be valid JSON");

    let aql_strings = aql_body["aql_strings"]
        .as_array()
        .expect("EC-001: aql_strings must be array");

    assert!(
        aql_strings.iter().any(|s| s.as_str() == Some(special_aql)),
        "EC-001: AQL with special chars must be stored verbatim; got: {aql_strings:?}"
    );
}

#[tokio::test]
async fn ec_004_pagination_beyond_last_page_returns_empty_array() {
    // EC-004: Page beyond last → empty devices array, correct total.
    let mut clone = ArmisClone::new().expect("EC-004: ArmisClone::new() must succeed");
    clone.start().await.expect("EC-004: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Fixture has 25 devices. Page 100 with size 25 should be empty.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("page", "100"), ("size", "25")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-004: GET /api/v1/devices page=100 must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-004: page beyond last must still return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-004: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("EC-004: data.devices must be array");

    assert!(
        devices.is_empty(),
        "EC-004: page beyond last must return empty devices array"
    );

    let total = body["data"]["total"]
        .as_u64()
        .expect("EC-004: data.total must be a number");

    assert!(
        total > 0,
        "EC-004: total must be > 0 (reflects true fixture count, not current page)"
    );
}

#[tokio::test]
async fn ac_1_devices_response_contains_pagination_fields() {
    // Verifies the full response shape: data.devices, data.total, data.page.
    let mut clone = ArmisClone::new().expect("AC-1: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-1: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-1: GET /api/v1/devices must succeed");

    let body: serde_json::Value = resp.json().await.expect("AC-1: body must be valid JSON");

    assert!(
        body["data"]["devices"].is_array(),
        "AC-1: data.devices must be present and be an array"
    );
    assert!(
        body["data"]["total"].is_number(),
        "AC-1: data.total must be present and be a number"
    );
    assert!(
        body["data"]["page"].is_number(),
        "AC-1: data.page must be present and be a number"
    );
}
