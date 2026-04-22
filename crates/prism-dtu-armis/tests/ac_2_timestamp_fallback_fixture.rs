// AC-2: Given `fixtures/devices.json` contains device `"d-001"` with `last_seen: null`
// and `first_seen: "2024-01-15T10:00:00Z"`, When the device detail is returned by the DTU,
// Then the response JSON contains `last_seen: null` and a non-null `first_seen` —
// exercising Prism's timestamp fallback path in the TOML spec.
//
// Also verifies device `"d-002"` has both timestamps populated (contrast case).
//
// Red Gate: these tests assert specific field values from the fixture.
// They will fail if the fixture file is malformed, if device records are missing,
// or if the route handler strips null fields.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_2_device_d001_has_null_last_seen_and_non_null_first_seen() {
    let mut clone = ArmisClone::new().expect("AC-2: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-2: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Fetch the full device list and find d-001.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-2: GET /api/v1/devices must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-2: GET /api/v1/devices must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-2: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-2: data.devices must be an array");

    // Find device d-001 in the response.
    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-2: device d-001 must be present in fixture response"));

    // d-001 must have last_seen: null (mandatory timestamp-fallback fixture requirement).
    assert!(
        d001["last_seen"].is_null(),
        "AC-2: device d-001 must have last_seen: null for timestamp fallback test, got: {:?}",
        d001["last_seen"]
    );

    // d-001 must have a non-null first_seen (the fallback timestamp).
    assert!(
        !d001["first_seen"].is_null(),
        "AC-2: device d-001 must have non-null first_seen for fallback path"
    );

    let first_seen = d001["first_seen"]
        .as_str()
        .expect("AC-2: first_seen must be a string");

    assert_eq!(
        first_seen,
        "2024-01-15T10:00:00Z",
        "AC-2: d-001 first_seen must be '2024-01-15T10:00:00Z'"
    );
}

#[tokio::test]
async fn ac_2_device_d002_has_both_timestamps_populated() {
    // Contrast case: d-002 must have both last_seen and first_seen.
    let mut clone = ArmisClone::new().expect("AC-2 contrast: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-2 contrast: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-2 contrast: GET /api/v1/devices must succeed");

    let body: serde_json::Value = resp.json().await.expect("AC-2 contrast: body must be JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-2 contrast: data.devices must be array");

    let d002 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-002"))
        .unwrap_or_else(|| panic!("AC-2 contrast: device d-002 must be present"));

    // d-002 must have a non-null last_seen.
    assert!(
        !d002["last_seen"].is_null(),
        "AC-2 contrast: d-002 must have non-null last_seen"
    );

    // d-002 must also have a non-null first_seen.
    assert!(
        !d002["first_seen"].is_null(),
        "AC-2 contrast: d-002 must have non-null first_seen"
    );
}

#[tokio::test]
async fn ac_2_device_risk_endpoint_returns_risk_score() {
    // The risk endpoint also returns device data (risk_score field from fixture).
    let mut clone = ArmisClone::new().expect("AC-2 risk: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-2 risk: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-2 risk: GET /api/v1/devices/d-001/risk must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-2 risk: d-001 risk endpoint must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-2 risk: body must be valid JSON");

    assert_eq!(
        body["data"]["device_id"].as_str().unwrap_or(""),
        "d-001",
        "AC-2 risk: data.device_id must be 'd-001'"
    );

    assert!(
        body["data"]["risk_score"].is_number(),
        "AC-2 risk: data.risk_score must be a number"
    );

    assert!(
        body["data"]["risk_factors"].is_array(),
        "AC-2 risk: data.risk_factors must be an array"
    );
}

#[tokio::test]
async fn ec_002_risk_endpoint_returns_404_for_unknown_device() {
    // EC-002: GET /api/v1/devices/{device_id}/risk for device_id not in fixture → HTTP 404.
    let mut clone = ArmisClone::new().expect("EC-002: ArmisClone::new() must succeed");
    clone.start().await.expect("EC-002: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-NONEXISTENT-9999/risk"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-002: GET risk for unknown device must be sent");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "EC-002: risk endpoint for unknown device must return HTTP 404"
    );

    let body: serde_json::Value = resp.json().await.expect("EC-002: body must be valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "EC-002: 404 response must include an error field"
    );
    assert!(
        error.contains("not found"),
        "EC-002: error message must indicate 'not found', got: {error:?}"
    );
}
