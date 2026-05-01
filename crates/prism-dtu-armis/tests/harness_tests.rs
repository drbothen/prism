//! Harness migration assertions — S-3.4.02 Red Gate
//!
//! Migrates all prism-dtu-armis tests to use `prism-dtu-harness::Harness`
//! instead of the single-tenant `ArmisClone::start()` pattern, and adds
//! multi-org logical isolation + network cross-credential 401 tests.
//!
//! ## Source tests (kept in place; migrated here in implementer phase)
//!
//! | Source file | Original test functions |
//! |-------------|------------------------|
//! | `ac_1_aql_capture_and_device_list.rs` | ac_1_get_devices_with_aql_returns_200_and_logs_aql, ac_1_post_devices_with_aql_body_returns_200_and_logs_aql, ec_001_aql_special_characters_stored_verbatim, ec_004_pagination_beyond_last_page_returns_empty_array, ac_1_devices_response_contains_pagination_fields |
//! | `ac_2_timestamp_fallback_fixture.rs` | ac_2_device_d001_has_null_last_seen_and_non_null_first_seen, ac_2_device_d002_has_both_timestamps_populated, ac_2_device_risk_endpoint_returns_risk_score, ec_002_risk_endpoint_returns_404_for_unknown_device |
//! | `ac_3_stateful_tag_add.rs` | ac_3_post_tag_returns_201_with_device_id_and_tag_key, ac_3_added_tag_appears_in_subsequent_device_query, ac_3_tag_endpoint_requires_bearer_auth_returns_403 |
//! | `ac_4_tag_delete.rs` | ac_4_delete_tag_returns_200_removed, ac_4_device_does_not_have_tag_after_delete, ec_003_delete_nonexistent_tag_returns_404, ac_4_delete_tag_endpoint_requires_bearer_auth |
//! | `ac_5_missing_bearer_403.rs` | ac_5_get_devices_without_auth_returns_403, ac_5_get_alerts_without_auth_returns_403, ac_5_get_device_activity_without_auth_returns_403, ac_5_get_device_risk_without_auth_returns_403, ac_5_empty_bearer_value_returns_403, ac_5_wrong_scheme_returns_403, ac_5_dtu_internal_endpoints_do_not_require_auth |
//! | `ac_6_rate_limit_429.rs` | ac_6_rate_limit_429_after_threshold_exceeded_via_configure, ac_6_rate_limit_allows_requests_before_threshold, ec_006_malformed_response_mode_returns_non_parseable_body |
//! | `fidelity_validator.rs` | fidelity_validator_passes |
//! | `reset_state_invariants.rs` | ac_story_7_reset_clears_tag_store_and_aql_log, ac_story_7_reset_does_not_remove_fixture_data, activity_endpoint_returns_200_with_activities_array, alerts_endpoint_returns_200_with_alerts_array, alerts_pagination_beyond_last_returns_empty_array, reset_clears_failure_mode_to_none |
//! | `td_wv0_04_configure_deny_unknown.rs` | configure_known_field_returns_200, configure_unknown_field_returns_400 |
//! | `td_wv0_07_configure_requires_admin_token.rs` | configure_without_token_returns_401, configure_with_wrong_token_returns_401, configure_with_correct_token_returns_200 |
//!
//! ## New tests (BC-3.5.001 + BC-3.5.002)
//!
//! - `test_BC_3_5_001_ac_multi_org_logical_isolation` (AC-003): 2-org Armis harness; pairwise-disjoint device sets.
//! - `test_BC_3_5_002_ac_network_cross_creds_401` (AC-004): Network isolation; cross-org credential mismatch → HTTP 401.
//!
//! # Red Gate
//!
//! ALL tests in this file MUST fail before implementation.
//! They fail because `HarnessBuilder::build()` stubs are not yet wired to start
//! the Armis clone — `endpoint_for` will return `None`, causing `expect()` panics.
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern (Factory TDD spec).
#![cfg(feature = "dtu")]
#![allow(clippy::expect_used, non_snake_case, dead_code, unused_imports)]

use prism_dtu_common::{FidelityCheck, FidelityValidator};
use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// Helper: build a single-tenant Armis harness (logical isolation).
//
// Used by every migrated single-org test. The implementer wires this helper so
// that HarnessBuilder builds an actual running Armis clone and
// `harness.endpoint_for("test-tenant", DtuType::Armis)` returns the address.
// ============================================================================

/// Build a single-tenant Armis harness in Logical isolation mode.
///
/// Returns `(harness, base_url, admin_token)`.
async fn build_single_armis_harness(slug: &str) -> (prism_dtu_harness::Harness, String, String) {
    let harness = prism_dtu_harness::HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(slug, |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("HarnessBuilder::build() must succeed for single-tenant Armis harness");

    let addr = harness
        .endpoint_for(slug, DtuType::Armis)
        .unwrap_or_else(|| {
            panic!(
                "harness must expose endpoint for ({slug:?}, Armis) — \
                 HarnessBuilder not yet wired to start Armis clone (RED gate expected)"
            )
        });

    let base_url = format!("http://{addr}");
    let admin_token = harness
        .admin_token_for(slug, DtuType::Armis)
        .unwrap_or_else(|| panic!("harness must expose admin token for ({slug:?}, Armis)"))
        .to_owned();

    (harness, base_url, admin_token)
}

// ============================================================================
// AC-1 migrations — aql_capture_and_device_list
// (source: ac_1_aql_capture_and_device_list.rs)
// ============================================================================

/// AC-1: GET /api/v1/devices?aql=in:type=switch with Bearer → HTTP 200, AQL captured.
///
/// (BC-3.5.001 postcondition 1; AC-001; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_get_devices_with_aql_returns_200_and_logs_aql() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

/// AC-1 / EC-005: POST /api/v1/devices with AQL body → HTTP 200, AQL captured.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_post_devices_with_aql_body_returns_200_and_logs_aql() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

/// EC-001: AQL strings with special characters stored verbatim.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec001_aql_special_characters_stored_verbatim() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

/// EC-004: Pagination page beyond last returns empty devices array with correct total.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec004_pagination_beyond_last_page_returns_empty_array() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

/// AC-1: Response contains pagination fields (data.devices, data.total, data.page).
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_devices_response_contains_pagination_fields() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

// ============================================================================
// AC-2 migrations — timestamp_fallback_fixture
// (source: ac_2_timestamp_fallback_fixture.rs)
// ============================================================================

/// AC-2: Device d-001 has last_seen:null and non-null first_seen (timestamp fallback).
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_d001_has_null_last_seen_and_non_null_first_seen() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

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

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-2: device d-001 must be present in fixture response"));

    assert!(
        d001["last_seen"].is_null(),
        "AC-2: device d-001 must have last_seen: null for timestamp fallback test, got: {:?}",
        d001["last_seen"]
    );

    assert!(
        !d001["first_seen"].is_null(),
        "AC-2: device d-001 must have non-null first_seen for fallback path"
    );

    let first_seen = d001["first_seen"]
        .as_str()
        .expect("AC-2: first_seen must be a string");

    assert_eq!(
        first_seen, "2024-01-15T10:00:00Z",
        "AC-2: d-001 first_seen must be '2024-01-15T10:00:00Z'"
    );
}

/// AC-2 contrast: Device d-002 has both timestamps populated.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_d002_has_both_timestamps_populated() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

    assert!(
        !d002["last_seen"].is_null(),
        "AC-2 contrast: d-002 must have non-null last_seen"
    );

    assert!(
        !d002["first_seen"].is_null(),
        "AC-2 contrast: d-002 must have non-null first_seen"
    );
}

/// AC-2: Risk endpoint returns risk_score field.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_risk_endpoint_returns_risk_score() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-2 risk: body must be valid JSON");

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

/// EC-002: Risk endpoint returns 404 for unknown device.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec002_risk_endpoint_returns_404_for_unknown_device() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
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

// ============================================================================
// AC-3 migrations — stateful_tag_add
// (source: ac_3_stateful_tag_add.rs)
// ============================================================================

/// AC-3: POST tag returns 201 with device_id and tag_key.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_post_tag_returns_201_with_device_id_and_tag_key() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3: POST /api/v1/devices/d-001/tags/ must succeed");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "AC-3: POST tag must return HTTP 201"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-3: response body must be valid JSON");

    assert_eq!(
        body["device_id"].as_str().unwrap_or(""),
        "d-001",
        "AC-3: response device_id must be 'd-001'"
    );

    assert_eq!(
        body["tag_key"].as_str().unwrap_or(""),
        "ot-critical",
        "AC-3: response tag_key must be 'ot-critical'"
    );

    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "added",
        "AC-3: response status must be 'added'"
    );
}

/// AC-3: Added tag appears in subsequent device query.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_added_tag_appears_in_subsequent_device_query() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Add tag to device d-001.
    let tag_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3 state: POST tag must succeed");

    assert_eq!(
        tag_resp.status().as_u16(),
        201,
        "AC-3 state: POST tag must return 201"
    );

    // Now query devices and check d-001 has the tag.
    let devices_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-3 state: GET /api/v1/devices must succeed");

    assert_eq!(
        devices_resp.status().as_u16(),
        200,
        "AC-3 state: GET /api/v1/devices must return HTTP 200"
    );

    let body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("AC-3 state: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-3 state: data.devices must be array");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-3 state: d-001 must be in device list"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-3 state: d-001.tags must be an array");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        tag_values.contains(&"ot-critical"),
        "AC-3 state: d-001 must have 'ot-critical' tag after POST, got: {tag_values:?}"
    );
}

/// AC-3: Tag endpoint requires Bearer auth → 403 without auth.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_tag_endpoint_requires_bearer_auth_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3 auth: request without auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-3 auth: POST tag without auth must return HTTP 403 (not 401)"
    );
}

// ============================================================================
// AC-4 migrations — tag_delete
// (source: ac_4_tag_delete.rs)
// ============================================================================

/// AC-4: DELETE tag returns 200 with status "removed".
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_delete_tag_returns_200_removed() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // First add the tag.
    let add_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-4: POST tag must succeed");

    assert_eq!(
        add_resp.status().as_u16(),
        201,
        "AC-4: POST tag must return 201"
    );

    // Now delete it.
    let del_resp = client
        .delete(format!("{base_url}/api/v1/devices/d-001/tags/ot-critical"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4: DELETE /api/v1/devices/d-001/tags/ot-critical must succeed");

    assert_eq!(
        del_resp.status().as_u16(),
        200,
        "AC-4: DELETE tag must return HTTP 200"
    );

    let body: serde_json::Value = del_resp
        .json()
        .await
        .expect("AC-4: DELETE response body must be valid JSON");

    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "removed",
        "AC-4: DELETE response status must be 'removed'"
    );
}

/// AC-4: Device does not have tag after delete.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_device_does_not_have_tag_after_delete() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Add tag.
    client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "removable-tag" }))
        .send()
        .await
        .expect("AC-4 state: POST tag must succeed");

    // Delete it.
    client
        .delete(format!(
            "{base_url}/api/v1/devices/d-001/tags/removable-tag"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4 state: DELETE tag must succeed");

    // Query devices — d-001 must not have the tag.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4 state: GET /api/v1/devices must succeed");

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-4 state: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-4 state: data.devices must be array");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-4 state: d-001 must be in device list"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-4 state: d-001.tags must be array");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        !tag_values.contains(&"removable-tag"),
        "AC-4 state: d-001 must NOT have 'removable-tag' after DELETE, got: {tag_values:?}"
    );
}

/// EC-003: DELETE non-existent tag returns 404.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec003_delete_nonexistent_tag_returns_404() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/d-001/tags/never-added-tag"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-003: DELETE non-existent tag must be sent");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "EC-003: DELETE non-existent tag must return HTTP 404"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("EC-003: 404 response must be valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "EC-003: 404 response must include an error field"
    );
    assert!(
        error.contains("not found"),
        "EC-003: error must indicate tag not found, got: {error:?}"
    );
}

/// AC-4: DELETE tag endpoint requires Bearer auth → 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_delete_tag_endpoint_requires_bearer_auth() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/d-001/tags/some-tag"))
        .send()
        .await
        .expect("AC-4 auth: DELETE without auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-4 auth: DELETE tag without auth must return HTTP 403 (not 401)"
    );
}

// ============================================================================
// AC-5 migrations — missing_bearer_403
// (source: ac_5_missing_bearer_403.rs)
// ============================================================================

/// AC-5: GET /api/v1/devices without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_devices_without_auth_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/devices must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: missing Authorization header must return HTTP 403 (not 401)"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-5: 403 body must be valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "AC-5: 403 response must include 'error' field"
    );
    assert!(
        error.contains("bearer token") || error.contains("missing"),
        "AC-5: error must mention bearer token, got: {error:?}"
    );

    let code = body["code"].as_u64().unwrap_or(0);
    assert_eq!(
        code, 403,
        "AC-5: 403 response body 'code' field must be 403"
    );
}

/// AC-5: GET /api/v1/alerts without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_alerts_without_auth_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/alerts must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: GET /api/v1/alerts without auth must return HTTP 403"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: 403 body must be JSON");
    assert!(
        body["error"].is_string(),
        "AC-5: 403 body must have 'error' string field"
    );
    assert_eq!(
        body["code"].as_u64().unwrap_or(0),
        403,
        "AC-5: 403 body 'code' must be 403"
    );
}

/// AC-5: GET /api/v1/devices/d-001/activity without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_device_activity_without_auth_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/devices/d-001/activity must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: activity endpoint without auth must return HTTP 403"
    );
}

/// AC-5: GET /api/v1/devices/d-001/risk without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_device_risk_without_auth_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET risk must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: risk endpoint without auth must return HTTP 403"
    );
}

/// AC-5: "Authorization: Bearer " (empty token value) returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_empty_bearer_value_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .expect("AC-5 empty bearer: request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5 empty bearer: 'Bearer ' with no token must return HTTP 403"
    );
}

/// AC-5: "Authorization: Basic ..." (wrong scheme) returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_wrong_scheme_returns_403() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .send()
        .await
        .expect("AC-5 basic: request with Basic auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5 basic: Basic auth scheme must return HTTP 403 (Armis requires Bearer)"
    );
}

/// AC-5: DTU internal /dtu/* endpoints do not require auth.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_dtu_internal_endpoints_do_not_require_auth() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let health = client
        .get(format!("{base_url}/dtu/health"))
        .send()
        .await
        .expect("AC-5 dtu: GET /dtu/health must succeed");

    assert_eq!(
        health.status().as_u16(),
        200,
        "AC-5 dtu: GET /dtu/health must return 200 without auth"
    );

    let aql_log = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-5 dtu: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_log.status().as_u16(),
        200,
        "AC-5 dtu: GET /dtu/aql-log must return 200 without auth"
    );

    let reset = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-5 dtu: POST /dtu/reset must succeed");

    assert_eq!(
        reset.status().as_u16(),
        200,
        "AC-5 dtu: POST /dtu/reset must return 200 without auth"
    );
}

// ============================================================================
// AC-6 migrations — rate_limit_429
// (source: ac_6_rate_limit_429.rs)
// ============================================================================

/// AC-6: Rate limit 429 after threshold exceeded via /dtu/configure.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac6_rate_limit_429_after_threshold_exceeded_via_configure() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure failure injection: rate-limit after 0 successful requests.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 0,
            "retry_after_secs": 30
        }))
        .send()
        .await
        .expect("AC-6: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-6: POST /dtu/configure must return 200"
    );

    // Next request to vendor API must return 429.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-6: rate-limited request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-6: request after rate-limit exhaustion must return HTTP 429"
    );
}

/// AC-6: Rate limit allows requests before threshold.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac6_rate_limit_allows_requests_before_threshold() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure rate limit: allow 3 requests, then 429.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 3,
            "retry_after_secs": 30
        }))
        .send()
        .await
        .expect("AC-6 threshold: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-6 threshold: configure must return 200"
    );

    // First 3 requests must NOT be rate-limited (200 or 403, but not 429).
    for i in 1..=3_u32 {
        let resp = client
            .get(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-6 threshold: request {i} must succeed"));

        assert_ne!(
            resp.status().as_u16(),
            429,
            "AC-6 threshold: request {i} must NOT be rate-limited (within budget)"
        );
    }

    // 4th request must return 429.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-6 threshold: 4th request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-6 threshold: 4th request must return HTTP 429 (rate limit exceeded)"
    );
}

/// EC-006: MalformedResponse mode returns non-parseable body.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec006_malformed_response_mode_returns_non_parseable_body() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure malformed response mode.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "malformed_response"
        }))
        .send()
        .await
        .expect("EC-006: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "EC-006: configure must return 200"
    );

    // Request a device list — should get a malformed (non-JSON) response body.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-006: malformed response request must be sent");

    // The response will arrive (any status), but the body must not be valid JSON.
    let raw_bytes = resp
        .bytes()
        .await
        .expect("EC-006: raw body bytes must be readable");

    let parse_result = serde_json::from_slice::<serde_json::Value>(&raw_bytes);

    assert!(
        parse_result.is_err(),
        "EC-006: malformed response mode must produce a body that fails JSON parsing"
    );
}

// ============================================================================
// Fidelity validator migration
// (source: fidelity_validator.rs)
// AC-002: fidelity_validator reports checks_failed == 0 under harness.
// ============================================================================

/// AC-002: FidelityValidator reports checks_failed == 0 for all Armis endpoints
/// when the clone is hosted via HarnessBuilder.
///
/// The `base_url` must come from `harness.endpoint_for("test-tenant", DtuType::Armis)`,
/// not a hardcoded address.
///
/// (BC-3.5.001 precondition 3; AC-002; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac002_fidelity_validator_passes_under_harness() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;

    let checks = vec![
        // --- Unauthenticated vendor API endpoints: must return 403 ---
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/activity".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/risk".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/tags/".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"tag_key": "probe-tag"})),
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // --- DTU internal endpoints: no auth required ---
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        FidelityCheck {
            endpoint: "/dtu/aql-log".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["aql_strings".to_string()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;
    assert_eq!(
        report.checks_failed, 0,
        "fidelity failures: {:?}",
        report.failures
    );
}

// ============================================================================
// reset_state_invariants migration
// (source: reset_state_invariants.rs)
// AC-005: reset passes — state mutate + reset → clean state + all invariants hold.
// ============================================================================

/// AC-005: After state-mutating operation followed by reset, clone state is clean.
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_clears_tag_store_and_aql_log() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Step 1: Add a tag to d-001.
    let tag_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "pre-reset-tag" }))
        .send()
        .await
        .expect("AC-005: POST tag must succeed");

    assert_eq!(
        tag_resp.status().as_u16(),
        201,
        "AC-005: POST tag must return 201"
    );

    // Step 2: Send a device query with AQL to populate the AQL log.
    client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("aql", "in:type=switch")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-005: GET devices with AQL must succeed");

    // Verify AQL log is non-empty before reset.
    let aql_before = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-005: GET /dtu/aql-log must succeed");

    let aql_before_body: serde_json::Value = aql_before
        .json()
        .await
        .expect("AC-005: aql-log body must be JSON");

    let aql_before_strings = aql_before_body["aql_strings"]
        .as_array()
        .expect("AC-005: aql_strings must be array before reset");

    assert!(
        !aql_before_strings.is_empty(),
        "AC-005 pre-condition: AQL log must be non-empty before reset"
    );

    // Step 3: Call reset via POST /dtu/reset.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-005: POST /dtu/reset must succeed");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "AC-005: POST /dtu/reset must return HTTP 200"
    );

    let reset_body: serde_json::Value = reset_resp
        .json()
        .await
        .expect("AC-005: reset response must be valid JSON");

    assert_eq!(
        reset_body["status"].as_str().unwrap_or(""),
        "ok",
        "AC-005: reset response status must be 'ok'"
    );

    // Step 4: Verify AQL log is empty after reset.
    let aql_after = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-005: GET /dtu/aql-log after reset must succeed");

    let aql_after_body: serde_json::Value = aql_after
        .json()
        .await
        .expect("AC-005: aql-log body after reset must be JSON");

    let aql_after_strings = aql_after_body["aql_strings"]
        .as_array()
        .expect("AC-005: aql_strings after reset must be array");

    assert!(
        aql_after_strings.is_empty(),
        "AC-005: AQL log must be empty after reset, got: {aql_after_strings:?}"
    );

    // Step 5: Verify d-001 has no tags after reset.
    let devices_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-005: GET devices after reset must succeed");

    let devices_body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("AC-005: devices body after reset must be JSON");

    let devices = devices_body["data"]["devices"]
        .as_array()
        .expect("AC-005: data.devices must be array after reset");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-005: d-001 must be present after reset"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-005: d-001.tags must be array after reset");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        !tag_values.contains(&"pre-reset-tag"),
        "AC-005: d-001 must NOT have 'pre-reset-tag' after reset, got: {tag_values:?}"
    );
}

/// AC-005: Reset does not remove fixture data (devices, alerts, activity remain).
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_does_not_remove_fixture_data() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Reset.
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-005 fixtures: POST /dtu/reset must succeed");

    // Devices must still be present.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-005 fixtures: GET devices must succeed after reset");

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-005 fixtures: body must be JSON");

    let total = body["data"]["total"]
        .as_u64()
        .expect("AC-005 fixtures: data.total must be a number");

    assert!(
        total >= 25,
        "AC-005 fixtures: all 25 fixture devices must be present after reset, got total={total}"
    );
}

/// AC-005: Activity endpoint returns 200 with activities array after harness build.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_activity_endpoint_returns_200_with_activities_array() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("activity: GET /api/v1/devices/d-001/activity must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "activity: endpoint must return HTTP 200 with valid Bearer"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("activity: body must be valid JSON");

    assert!(
        body["data"]["activities"].is_array(),
        "activity: data.activities must be present and be an array"
    );

    assert!(
        body["data"]["total"].is_number(),
        "activity: data.total must be present and be a number"
    );
}

/// AC-005: Alerts endpoint returns 200 with alerts array after harness build.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_alerts_endpoint_returns_200_with_alerts_array() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("alerts: GET /api/v1/alerts must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "alerts: endpoint must return HTTP 200 with valid Bearer"
    );

    let body: serde_json::Value = resp.json().await.expect("alerts: body must be valid JSON");

    assert!(
        body["data"]["alerts"].is_array(),
        "alerts: data.alerts must be present and be an array"
    );

    assert!(
        body["data"]["total"].is_number(),
        "alerts: data.total must be present and be a number"
    );

    let total = body["data"]["total"].as_u64().unwrap_or(0);
    assert!(
        total >= 12,
        "alerts: fixture must have at least 12 alerts, got total={total}"
    );
}

/// AC-005: Alerts pagination beyond last page returns empty array.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_alerts_pagination_beyond_last_returns_empty_array() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .query(&[("page", "999"), ("size", "25")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("alerts page: GET /api/v1/alerts page=999 must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "alerts page: page beyond last must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("alerts page: body must be JSON");

    let alerts = body["data"]["alerts"]
        .as_array()
        .expect("alerts page: data.alerts must be array");

    assert!(
        alerts.is_empty(),
        "alerts page: page beyond last must return empty alerts array"
    );

    let total = body["data"]["total"].as_u64().unwrap_or(0);
    assert!(
        total > 0,
        "alerts page: total must reflect fixture size even on empty page"
    );
}

/// AC-005: POST /dtu/reset also resets failure mode to None.
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_clears_failure_mode_to_none() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Step 1: Configure rate-limit so all requests return 429.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 0,
            "retry_after_secs": 1
        }))
        .send()
        .await
        .expect("reset_failure_mode: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "reset_failure_mode: configure must return 200"
    );

    // Confirm rate-limit is active: vendor request returns 429.
    let rate_limited_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("reset_failure_mode: rate-limited request must be sent");

    assert_eq!(
        rate_limited_resp.status().as_u16(),
        429,
        "reset_failure_mode: pre-condition — request must be rate-limited before reset"
    );

    // Step 2: Reset — must clear failure mode.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset_failure_mode: POST /dtu/reset must succeed");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "reset_failure_mode: reset must return 200"
    );

    // Step 3: Verify vendor requests are no longer rate-limited.
    let normal_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("reset_failure_mode: post-reset request must be sent");

    assert_ne!(
        normal_resp.status().as_u16(),
        429,
        "reset_failure_mode: after reset, requests must NOT be rate-limited (failure_mode cleared)"
    );

    assert_eq!(
        normal_resp.status().as_u16(),
        200,
        "reset_failure_mode: after reset, device query must return HTTP 200"
    );
}

// ============================================================================
// TD migrations — configure endpoint
// (source: td_wv0_04_configure_deny_unknown.rs, td_wv0_07_configure_requires_admin_token.rs)
// ============================================================================

/// TD-WV0-04: /dtu/configure rejects unknown fields (deny_unknown_fields) — known field → 200.
///
/// (S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_known_field_returns_200() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(resp.status(), 200, "known field must return 200");
}

/// TD-WV0-04: /dtu/configure rejects unknown fields (deny_unknown_fields) — unknown field → 400.
///
/// (S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_unknown_field_returns_400() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"bogus": "val"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        400,
        "unknown field must return 400 Bad Request, not silently accept"
    );
}

/// TD-WV0-07: /dtu/configure without X-Admin-Token → 401.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_without_token_returns_401() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        401,
        "TD-WV0-07: missing X-Admin-Token must return 401"
    );
}

/// TD-WV0-07: /dtu/configure with wrong X-Admin-Token → 401.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_with_wrong_token_returns_401() {
    let (_harness, base_url, _admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", "wrong-token-that-will-never-match")
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        401,
        "TD-WV0-07: incorrect X-Admin-Token must return 401"
    );
}

/// TD-WV0-07: /dtu/configure with correct X-Admin-Token → 200.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_with_correct_token_returns_200() {
    let (_harness, base_url, admin_token) = build_single_armis_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        200,
        "TD-WV0-07: correct X-Admin-Token must return 200"
    );
}

// ============================================================================
// NEW: AC-003 — Multi-org logical isolation (BC-3.5.001 postcondition 2; TV-2)
//
// Build a 2-org logical harness ("test-tenant" + "other-tenant"), both Armis.
// Query each org; assert device sets are pairwise-disjoint.
// ============================================================================

/// AC-003: 2-org Armis logical harness returns pairwise-disjoint device sets.
///
/// "test-tenant" devices must not appear in "other-tenant" response and vice versa.
///
/// (BC-3.5.001 postcondition 2; TV-2; AC-003; S-3.4.02 Task 3; VP-122, VP-123)
#[tokio::test]
async fn test_BC_3_5_001_ac_multi_org_logical_isolation() {
    // Build a 2-org Armis harness in Logical isolation.
    let harness = prism_dtu_harness::HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .with_customer_overrides("other-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("2-org logical harness build must succeed");

    // Resolve endpoints for both orgs.
    let addr_a = harness
        .endpoint_for("test-tenant", DtuType::Armis)
        .unwrap_or_else(|| {
            panic!(
                "harness must expose endpoint for (test-tenant, Armis) — \
                 HarnessBuilder not yet wired to start Armis clone (RED gate expected)"
            )
        });

    let addr_b = harness
        .endpoint_for("other-tenant", DtuType::Armis)
        .unwrap_or_else(|| {
            panic!(
                "harness must expose endpoint for (other-tenant, Armis) — \
                 HarnessBuilder not yet wired to start Armis clone (RED gate expected)"
            )
        });

    // Verify addresses are distinct (BC-3.5.001 Invariant 3).
    assert_ne!(
        addr_a, addr_b,
        "BC-3.5.001 Invariant 3: each (org, dtu_type) pair must bind a distinct port"
    );

    let base_a = format!("http://{addr_a}");
    let base_b = format!("http://{addr_b}");
    let client = reqwest::Client::new();

    // Fetch all devices from org-A.
    let resp_a = client
        .get(format!("{base_a}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-003: GET /api/v1/devices for test-tenant must succeed");

    assert_eq!(
        resp_a.status().as_u16(),
        200,
        "AC-003: test-tenant device list must return HTTP 200"
    );

    let body_a: serde_json::Value = resp_a
        .json()
        .await
        .expect("AC-003: test-tenant response must be valid JSON");

    let devices_a = body_a["data"]["devices"]
        .as_array()
        .expect("AC-003: test-tenant data.devices must be array");

    let ids_a: std::collections::HashSet<String> = devices_a
        .iter()
        .filter_map(|d| d["device_id"].as_str().map(|s| s.to_owned()))
        .collect();

    // Fetch all devices from org-B.
    let resp_b = client
        .get(format!("{base_b}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-003: GET /api/v1/devices for other-tenant must succeed");

    assert_eq!(
        resp_b.status().as_u16(),
        200,
        "AC-003: other-tenant device list must return HTTP 200"
    );

    let body_b: serde_json::Value = resp_b
        .json()
        .await
        .expect("AC-003: other-tenant response must be valid JSON");

    let devices_b = body_b["data"]["devices"]
        .as_array()
        .expect("AC-003: other-tenant data.devices must be array");

    let ids_b: std::collections::HashSet<String> = devices_b
        .iter()
        .filter_map(|d| d["device_id"].as_str().map(|s| s.to_owned()))
        .collect();

    // Assert non-empty: each org must have at least one device.
    assert!(
        !ids_a.is_empty(),
        "AC-003: test-tenant must have at least one device"
    );
    assert!(
        !ids_b.is_empty(),
        "AC-003: other-tenant must have at least one device"
    );

    // Assert pairwise-disjoint: no device_id may appear in both orgs.
    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "AC-003: device sets must be pairwise-disjoint (BC-3.5.001 postcondition 2); \
         shared ids: {intersection:?}"
    );
}

// ============================================================================
// NEW: AC-004 — Network isolation cross-credential 401 (BC-3.5.002 postcondition 2; TV-3)
//
// Build a 2-org Network harness. Use org-A's credentials against org-B's endpoint.
// Expect HTTP 401.
// ============================================================================

/// AC-004: 2-org Network harness; cross-org credential mismatch returns HTTP 401.
///
/// Org-A's Bearer token sent to org-B's Armis clone endpoint must be rejected
/// with HTTP 401 (network isolation enforced).
///
/// (BC-3.5.002 postcondition 2; TV-3; AC-004; S-3.4.02 Task 4; VP-125, VP-126, VP-127)
#[tokio::test]
async fn test_BC_3_5_002_ac_network_cross_creds_401() {
    // Build a 2-org Armis Network harness.
    let harness = prism_dtu_harness::HarnessBuilder::new()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .with_customer_overrides("other-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("2-org network harness build must succeed");

    // Resolve the endpoint address for org-B (other-tenant).
    let addr_b = harness
        .endpoint_for("other-tenant", DtuType::Armis)
        .unwrap_or_else(|| {
            panic!(
                "harness must expose endpoint for (other-tenant, Armis) — \
                 HarnessBuilder::build_network not yet wired (RED gate expected)"
            )
        });

    // Obtain org-A's admin token (test-tenant Bearer credential).
    let token_a = harness
        .admin_token_for("test-tenant", DtuType::Armis)
        .unwrap_or_else(|| {
            panic!(
                "harness must expose admin token for (test-tenant, Armis) — \
                 build_network not yet populating admin_tokens (RED gate expected)"
            )
        })
        .to_owned();

    let base_b = format!("http://{addr_b}");
    let client = reqwest::Client::new();

    // Send org-A's Bearer token to org-B's endpoint.
    // Network-mode clone validates the Bearer token; a foreign token must return 401.
    let resp = client
        .get(format!("{base_b}/api/v1/devices"))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("AC-004: request with cross-org token to other-tenant endpoint must be sent");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-004: cross-org credential mismatch must return HTTP 401 \
         (BC-3.5.002 postcondition 2; VP-126)"
    );
}
