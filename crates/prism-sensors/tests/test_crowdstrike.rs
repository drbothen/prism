//! Integration tests for CrowdStrike OAuth2 + two-step fetch.
//!
//! Covers BC-2.01.005:
//! - AC-1: OAuth2 token endpoint called once, token cached, reused
//! - AC-2: 401 on first use triggers token refresh + retry
//! - TV-BC-2.01.005-001: 50 IDs fetched in one PostEntities batch
//! - TV-BC-2.01.005-002: 0 IDs → empty result
//! - TV-BC-2.01.005-003: 401 on token request → SensorError authentication
//! - TV-BC-2.01.005-004: 401 on PostEntities → transparent token refresh
//! - TV-BC-2.01.005-005: 1500 IDs → 2 PostEntities calls (batched at BATCH_SIZE)
//!
//! All adapter tests are RED (todo!() panics). CROWDSTRIKE_BATCH_SIZE constant check
//! is GREEN-BY-DESIGN.
//!
//! Story: S-2.07 | BC: BC-2.01.005
#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::crowdstrike::{
    CrowdStrikeAdapter, CrowdStrikeAuth, CROWDSTRIKE_BATCH_SIZE,
};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{OrgId, SensorAdapter};

/// Returns a stable test `OrgId` for adapter constructor migration (AC-006).
///
/// Integration tests cannot access `DEFAULT_ORG_ID_BYTES` (which is
/// `#[cfg(test)]` gated in the library and thus unavailable to external test
/// crates).  We use a fixed UUID byte array directly here with the same value.
///
/// BC-3.2.001 invariant 3: the `DEFAULT_ORG_ID_BYTES` sentinel is not used in
/// production paths — this integration test helper replicates its value.
fn test_org_id() -> OrgId {
    // Same bytes as DEFAULT_ORG_ID_BYTES in lib.rs (AC-006 migration constant)
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

// ---------------------------------------------------------------------------
// CROWDSTRIKE_BATCH_SIZE constant — GREEN-BY-DESIGN
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN: The batch size constant must be exactly 100 per the spec.
///
/// S-2.07 Dev Notes: "The batch size is a const CROWDSTRIKE_BATCH_SIZE: usize = 100".
/// BC-2.01.005 §EC-01-008.
#[test]
fn test_BC_2_01_005_crowdstrike_batch_size_is_100() {
    assert_eq!(
        CROWDSTRIKE_BATCH_SIZE, 100,
        "CROWDSTRIKE_BATCH_SIZE must be 100 per BC-2.01.005 §EC-01-008"
    );
}

// ---------------------------------------------------------------------------
// Helper: build CrowdStrikeAuth + CrowdStrikeAdapter pointing at a mock server.
// ---------------------------------------------------------------------------

fn make_auth(base_url: &str) -> CrowdStrikeAuth {
    CrowdStrikeAuth {
        client_id: "test-client-id".into(),
        client_secret: SecretString::new("test-client-secret".into()),
        cloud_region: base_url.to_string(), // tests pass raw URL as region override
    }
}

fn make_spec(table: &str) -> SensorSpec {
    #[allow(deprecated)]
    SensorSpec {
        org_id: prism_sensors::OrgId::new(),
        source_table: table.into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    }
}

// ---------------------------------------------------------------------------
// AC-1 — OAuth2 token acquired once, reused on subsequent calls
// ---------------------------------------------------------------------------

/// AC-1 / TV-BC-2.01.005-001: Valid credentials → OAuth2 token called exactly once;
/// QueryV2 returns 50 IDs; PostEntities fetches all 50 in one batch.
///
/// Verifies token caching: mock token endpoint expects exactly 1 call.

#[tokio::test]
async fn test_BC_2_01_005_oauth2_token_called_once_and_cached() {
    let server = MockServer::start().await;

    // Mock the OAuth2 token endpoint — must be called exactly once (AC-1)
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test-bearer-token-abc123",
            "token_type": "bearer",
            "expires_in": 1799,
        })))
        .expect(1)
        .named("oauth2_token_endpoint")
        .mount(&server)
        .await;

    // Mock QueryV2 returning 50 IDs
    let ids: Vec<String> = (0..50).map(|i| format!("alert-id-{i:04}")).collect();
    Mock::given(method("GET"))
        .and(path("/queries/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ids,
            "meta": { "total": 50 }
        })))
        .expect(1)
        .named("query_v2")
        .mount(&server)
        .await;

    // Mock PostEntities returning 50 full records (1 batch of 50 ≤ CROWDSTRIKE_BATCH_SIZE)
    let full_records: Vec<serde_json::Value> = (0..50)
        .map(|i| serde_json::json!({ "id": format!("alert-id-{i:04}"), "severity": "medium" }))
        .collect();
    Mock::given(method("POST"))
        .and(path("/entities/alerts/GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": full_records
        })))
        .expect(1)
        .named("post_entities_50")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "AC-1: valid credentials must return Ok(Vec<RecordBatch>); got: {result:?}"
    );
    let batches = result.expect("AC-1 requires Ok result");
    assert!(
        !batches.is_empty(),
        "AC-1: must return at least one RecordBatch for 50 records"
    );
}

/// AC-1 (second fetch): token already cached → token endpoint still called only once total.
///
/// Two sequential fetches must result in only one token acquisition across both calls.

#[tokio::test]
async fn test_BC_2_01_005_cached_token_reused_on_second_fetch() {
    let server = MockServer::start().await;

    // Token endpoint: must be called ONLY once across two adapter.fetch() calls
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "cached-token-xyz",
            "token_type": "bearer",
            "expires_in": 1799,
        })))
        .expect(1)
        .named("token_endpoint_single_call")
        .mount(&server)
        .await;

    // QueryV2 — called twice (once per fetch)
    Mock::given(method("GET"))
        .and(path("/queries/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [],
            "meta": { "total": 0 }
        })))
        .expect(2)
        .named("query_v2_twice")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    // First fetch
    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // Second fetch — token must be served from cache
    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // wiremock verifies exact call counts on server drop (token called only once)
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.005-002: QueryV2 returns 0 IDs → empty result
// ---------------------------------------------------------------------------

/// TV-BC-2.01.005-002: QueryV2 returns 0 IDs → fetch returns empty Vec<RecordBatch>.
///

#[tokio::test]
async fn test_BC_2_01_005_query_returns_zero_ids_yields_empty_result() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok",
            "token_type": "bearer",
            "expires_in": 1799,
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/queries/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [],
            "meta": { "total": 0 }
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await
        .expect("TV-BC-2.01.005-002: empty result must be Ok, not Err");
    assert_eq!(
        result.len(),
        0,
        "TV-BC-2.01.005-002: zero IDs from QueryV2 must yield empty Vec<RecordBatch>"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.005-003: OAuth2 401 → SensorError with category authentication
// ---------------------------------------------------------------------------

/// TV-BC-2.01.005-003: Invalid credentials → OAuth2 returns 401 →
/// SensorError::HttpError with status 401.
///
/// BC-2.01.005 error case: "category: authentication".

#[tokio::test]
async fn test_BC_2_01_005_rejects_oauth2_401_with_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errors": [{ "code": 401, "message": "invalid client credentials" }]
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_err(),
        "TV-BC-2.01.005-003: OAuth2 401 must return Err"
    );
    match result {
        Err(SensorError::HttpError { status, .. }) => {
            assert_eq!(status, 401, "Error status must be 401 (authentication)");
        }
        Err(e) => panic!("Expected HttpError(401), got: {e}"),
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// AC-2 / TV-BC-2.01.005-004: Token expires mid-fetch → transparent refresh
// ---------------------------------------------------------------------------

/// AC-2 / TV-BC-2.01.005-004: Token returns 401 on PostEntities call → token
/// refreshed transparently → PostEntities retried with new token.
///
/// Token endpoint must be called exactly twice (initial + refresh).
/// PostEntities must be called exactly twice (failed + retried).

#[tokio::test]
async fn test_BC_2_01_005_token_refresh_on_post_entities_401() {
    let server = MockServer::start().await;

    // First token request (consumed after 1 use)
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "first-token",
            "expires_in": 1799,
        })))
        .up_to_n_times(1)
        .expect(1)
        .named("token_first")
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/queries/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ["id-001"],
            "meta": { "total": 1 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // PostEntities returns 401 on first attempt (consumed after 1 use)
    Mock::given(method("POST"))
        .and(path("/entities/alerts/GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errors": [{ "code": 401, "message": "token expired" }]
        })))
        .up_to_n_times(1)
        .expect(1)
        .named("post_entities_401")
        .mount(&server)
        .await;

    // Second token request (refresh — fallback once first is consumed)
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "refreshed-token",
            "expires_in": 1799,
        })))
        .expect(1)
        .named("token_refresh")
        .mount(&server)
        .await;

    // PostEntities succeeds on retry with refreshed token
    Mock::given(method("POST"))
        .and(path("/entities/alerts/GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [{ "id": "id-001", "severity": "high" }]
        })))
        .expect(1)
        .named("post_entities_retry_ok")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "AC-2: token refresh must make fetch succeed; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.005-005: 1500 IDs → 2 PostEntities calls (100 batch size → 15 batches)
// Wait — story says batch at 100, BC says 1000. Spec is authoritative: S-2.07
// CROWDSTRIKE_BATCH_SIZE=100 (dev notes), BC says "1000 per batch" in EC-01-008.
// We use CROWDSTRIKE_BATCH_SIZE = 100. Test uses 150 IDs → 2 batches.
// ---------------------------------------------------------------------------

/// TV-BC-2.01.005-005 (adapted): 150 IDs from QueryV2 → 2 PostEntities batches
/// (100 + 50) since CROWDSTRIKE_BATCH_SIZE=100.
///
/// BC-2.01.005 §EC-01-008: "IDs batched into multiple PostEntities calls".

#[tokio::test]
async fn test_BC_2_01_005_150_ids_batch_into_two_post_entities_calls() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "batch-tok",
            "expires_in": 1799,
        })))
        .mount(&server)
        .await;

    // QueryV2 returns 150 IDs
    let ids: Vec<String> = (0..150).map(|i| format!("id-{i:03}")).collect();
    Mock::given(method("GET"))
        .and(path("/queries/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ids,
            "meta": { "total": 150 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // PostEntities is called twice (2 batches: 100 + 50)
    let batch1_records: Vec<serde_json::Value> = (0..100)
        .map(|i| serde_json::json!({ "id": format!("id-{i:03}") }))
        .collect();
    let batch2_records: Vec<serde_json::Value> = (100..150)
        .map(|i| serde_json::json!({ "id": format!("id-{i:03}") }))
        .collect();
    Mock::given(method("POST"))
        .and(path("/entities/alerts/GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": batch1_records
        })))
        .up_to_n_times(1)
        .expect(1)
        .named("batch1")
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/entities/alerts/GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": batch2_records
        })))
        .up_to_n_times(1)
        .expect(1)
        .named("batch2")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CrowdStrikeAdapter::new(test_org_id(), &auth);
    let spec = make_spec("crowdstrike_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "150 IDs across 2 batches must return Ok; got: {result:?}"
    );
    // wiremock verifies that PostEntities was called exactly 2 times
}
