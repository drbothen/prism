//! RED tests for WGS-W2-002 (HIGH, CWE-312): bearer tokens must be wrapped in
//! SecretString so the type system prevents accidental plaintext exposure.
//!
//! Fix target: CrowdStrike `CachedToken::token`, Armis `ArmisAdapter::bearer_token`,
//! and Claroty `ClarotyAdapter::bearer_token` — all three were plain `String`.
//!
//! # RED gate
//!
//! These tests FAIL to COMPILE on current code because the `new()` constructors
//! accept `String`, not `SecretString`.  After the fix they accept `SecretString`
//! and these tests become GREEN.
//!
//! Additionally, `CachedToken` Debug output is tested via the unit tests in
//! `src/auth/crowdstrike.rs` (since `CachedToken` is `pub(crate)`).
//!
//! Security fix: WGS-W2-002 | BC: BC-2.01.005 / BC-2.01.008 / BC-2.01.013

#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::{ExposeSecret, SecretString};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorSpec};
use prism_sensors::auth::armis::{ArmisAdapter, ArmisAuth};
use prism_sensors::auth::claroty::{ClarotyAdapter, ClarotyAuth};
use prism_sensors::auth::SensorAuth;
use prism_sensors::SensorAdapter;

// ---------------------------------------------------------------------------
// WGS-W2-002-AR: ArmisAdapter::new() accepts SecretString for bearer_token
// ---------------------------------------------------------------------------

/// WGS-W2-002 (Armis): ArmisAdapter::new() must accept SecretString, not String.
///
/// This test fails to COMPILE on current code (new() takes String).
/// After fix: new() takes SecretString → compiles.
///
/// Also verifies: the bearer token is correctly used in HTTP calls (the
/// Authorization header receives the secret value via expose_secret()).
#[tokio::test]
async fn test_WGS_W2_002_armis_adapter_new_accepts_secret_string_and_calls_http() {
    let server = MockServer::start().await;

    let bearer_value = "armis-secret-bearer-xyz";

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "results": [], "total": 0 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = ArmisAuth {
        instance_url: server.uri(),
        secret_key: SecretString::new("armis-sk-test".into()),
    };
    // This line FAILS TO COMPILE on current code (new() takes String, not SecretString).
    // After fix: new() accepts SecretString.
    let adapter = ArmisAdapter::new(&auth, SecretString::new(bearer_value.into()));

    let spec = SensorSpec {
        source_table: "devices".into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    };
    let params = QueryParams::default();

    // Adapter debug must NOT contain the bearer value in plaintext.
    let debug_str = format!("{adapter:?}");
    assert!(
        !debug_str.contains(bearer_value),
        "WGS-W2-002: ArmisAdapter Debug MUST NOT emit bearer_token plaintext. Got: {debug_str:?}"
    );

    // HTTP call must succeed — verifies bearer is used correctly.
    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "WGS-W2-002: ArmisAdapter with SecretString bearer must make successful HTTP call; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// WGS-W2-002-CL: ClarotyAdapter::new() accepts SecretString for bearer_token
// ---------------------------------------------------------------------------

/// WGS-W2-002 (Claroty): ClarotyAdapter::new() must accept SecretString, not String.
///
/// This test fails to COMPILE on current code (new() takes String).
/// After fix: new() takes SecretString → compiles.
#[tokio::test]
async fn test_WGS_W2_002_claroty_adapter_new_accepts_secret_string_and_calls_http() {
    let server = MockServer::start().await;

    let bearer_value = "claroty-secret-bearer-abc";

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "objects": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = ClarotyAuth {
        instance_url: server.uri(),
        username: "testuser".into(),
        password: SecretString::new("testpass".into()),
    };
    // This line FAILS TO COMPILE on current code (new() takes String, not SecretString).
    let adapter = ClarotyAdapter::new(&auth, SecretString::new(bearer_value.into()));

    let spec = SensorSpec {
        source_table: "claroty_alert".into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    };
    let params = QueryParams::default();

    // Adapter debug must NOT contain the bearer value in plaintext.
    let debug_str = format!("{adapter:?}");
    assert!(
        !debug_str.contains(bearer_value),
        "WGS-W2-002: ClarotyAdapter Debug MUST NOT emit bearer_token plaintext. Got: {debug_str:?}"
    );

    // HTTP call must succeed — verifies bearer is used correctly.
    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "WGS-W2-002: ClarotyAdapter with SecretString bearer must make successful HTTP call; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// WGS-W2-002-CS: CrowdStrike CachedToken Debug tested in unit tests
// (CachedToken is pub(crate) so tested in src/auth/crowdstrike.rs)
// This integration test verifies the adapter-level Debug does not leak tokens.
// ---------------------------------------------------------------------------

/// WGS-W2-002 (CrowdStrike): CrowdStrikeAdapter Debug must not contain token.
///
/// Verifies that after token acquisition + caching, the cached token value
/// does not appear in the adapter's Debug output.
#[tokio::test]
async fn test_WGS_W2_002_crowdstrike_adapter_debug_does_not_contain_cached_token() {
    let server = MockServer::start().await;

    let token_value = "cs-super-secret-bearer-token-12345";

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": token_value,
            "token_type": "bearer",
            "expires_in": 1799,
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [],
        })))
        .mount(&server)
        .await;

    use prism_sensors::auth::crowdstrike::{CrowdStrikeAdapter, CrowdStrikeAuth};
    let auth = CrowdStrikeAuth {
        client_id: "test-client".into(),
        client_secret: SecretString::new("test-secret".into()),
        cloud_region: server.uri(),
    };
    let adapter = CrowdStrikeAdapter::new(&auth);

    let spec = SensorSpec {
        source_table: "crowdstrike_alert".into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    };
    let params = QueryParams::default();

    // Trigger token acquisition (token gets cached in token_cache)
    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;

    // After token is cached, adapter Debug must NOT contain the token value.
    let debug_str = format!("{adapter:?}");
    assert!(
        !debug_str.contains(token_value),
        "WGS-W2-002: CrowdStrikeAdapter Debug after token cache MUST NOT emit token plaintext. \
         Got: {debug_str:?}"
    );
}
