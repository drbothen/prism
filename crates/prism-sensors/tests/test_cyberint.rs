//! Integration tests for Cyberint cookie-based auth + timestamp parsing.
//!
//! Covers BC-2.01.006:
//! - TV-BC-2.01.006-001: valid access_token cookie; record parsed; cursor set
//! - TV-BC-2.01.006-004: HTTP 401 cookie rejection → SensorError authentication
//! - Cookie-based auth: POST /login sets session cookie used for subsequent requests
//! - 401 re-authentication: re-login + retry once
//! - Multi-format timestamp parsing (see test_timestamp.rs for unit tests)
//!
//! All adapter tests are RED (todo!() panics from CyberintAdapter::new).
//!
//! Story: S-2.07 | BC: BC-2.01.006
#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::cyberint::{CyberintAdapter, CyberintAuth};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{OrgId, SensorAdapter};

/// Returns a stable test `OrgId` for adapter constructor migration (AC-006).
///
/// Same value as `DEFAULT_ORG_ID_BYTES` in lib.rs; duplicated here because
/// `#[cfg(test)]` items in the library are not accessible from external
/// integration test crates.
fn test_org_id() -> OrgId {
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_auth(environment: &str) -> CyberintAuth {
    CyberintAuth {
        environment: environment.to_string(),
        api_key: SecretString::new("test-api-key-secret".into()),
    }
}

fn make_spec() -> SensorSpec {
    #[allow(deprecated)]
    SensorSpec {
        org_id: test_org_id(), // Must match adapter's OrgId (BC-3.2.001 precondition 4)
        source_table: "cyberint_alert".into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.006-001: valid access_token cookie → fetch succeeds with record
// ---------------------------------------------------------------------------

/// TV-BC-2.01.006-001: Adapter logs in via POST /login, receives Set-Cookie,
/// then uses session cookie on subsequent API call.
///
/// BC-2.01.006 postcondition: "All Cyberint API requests include the access_token
/// cookie header."

#[tokio::test]
async fn test_BC_2_01_006_login_sets_cookie_used_for_data_request() {
    let server = MockServer::start().await;

    // POST /login: sets session cookie
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Set-Cookie",
                    "access_token=session-abc123; Path=/; HttpOnly",
                )
                .set_body_json(serde_json::json!({ "status": "ok" })),
        )
        .expect(1)
        .named("login")
        .mount(&server)
        .await;

    // Data endpoint: verifies Cookie header is present
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .and(header_exists("Cookie"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "alert-001",
                    "created_at": "2024-03-15T10:00:00Z",
                    "severity": "high"
                }
            ],
            "total": 1
        })))
        .expect(1)
        .named("data_endpoint")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CyberintAdapter::new(test_org_id(), &auth);
    let spec = make_spec();
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "TV-BC-2.01.006-001: valid login must succeed; got: {result:?}"
    );
}

/// BC-2.01.006: login() is called exactly once at first fetch; cookie is reused on
/// second fetch (not re-logged in unless 401).
///

#[tokio::test]
async fn test_BC_2_01_006_login_called_once_cookie_reused_on_second_fetch() {
    let server = MockServer::start().await;

    // Login must be called only once across two fetches
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Set-Cookie", "access_token=session-xyz; Path=/; HttpOnly")
                .set_body_json(serde_json::json!({ "status": "ok" })),
        )
        .expect(1)
        .named("login_once")
        .mount(&server)
        .await;

    // Data endpoint called twice
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "total": 0
        })))
        .expect(2)
        .named("data_twice")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CyberintAdapter::new(test_org_id(), &auth);
    let spec = make_spec();
    let params = QueryParams::default();

    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // wiremock verifies login called exactly once
}

// ---------------------------------------------------------------------------
// 401 re-authentication: detect 401 → re-login → retry once
// ---------------------------------------------------------------------------

/// BC-2.01.006 §cookie refresh: On a 401 response from the data endpoint, the
/// adapter re-authenticates (re-POSTs to /login) and retries the request once.
///
/// Login is called twice (initial + re-auth). Data endpoint is called twice
/// (401 + successful retry).

#[tokio::test]
async fn test_BC_2_01_006_401_triggers_relogin_and_retry() {
    let server = MockServer::start().await;

    // Login is called twice: initial + re-auth after 401
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Set-Cookie",
                    "access_token=refreshed-session; Path=/; HttpOnly",
                )
                .set_body_json(serde_json::json!({ "status": "ok" })),
        )
        .expect(2)
        .named("login_twice")
        .mount(&server)
        .await;

    // First data request returns 401 (consumed after 1 use)
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "cookie_expired"
        })))
        .up_to_n_times(1)
        .expect(1)
        .named("data_401")
        .mount(&server)
        .await;

    // Retry after re-auth succeeds
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{ "id": "alert-002", "created_at": "2024-03-15T12:00:00Z" }],
            "total": 1
        })))
        .expect(1)
        .named("data_retry_ok")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CyberintAdapter::new(test_org_id(), &auth);
    let spec = make_spec();
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "BC-2.01.006: 401 + re-auth + retry must ultimately succeed; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.006-004: HTTP 401 on login → SensorError authentication
// ---------------------------------------------------------------------------

/// TV-BC-2.01.006-004: Login endpoint returns 401 → SensorError with status 401.
///
/// BC-2.01.006 error case: "category: authentication, suggestion: Verify
/// Cyberint access_token in credential store."

#[tokio::test]
async fn test_BC_2_01_006_rejects_login_401_with_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "invalid_credentials"
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CyberintAdapter::new(test_org_id(), &auth);
    let spec = make_spec();
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_err(),
        "TV-BC-2.01.006-004: 401 on login must return Err"
    );
    match result {
        Err(SensorError::HttpError { status, .. }) => {
            assert_eq!(
                status, 401,
                "Authentication failure must produce status 401"
            );
        }
        Err(e) => panic!("Expected HttpError(401), got: {e}"),
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// Timestamp parsing integration: Unix epoch in response parsed correctly
// AC-3 / EC-002
// ---------------------------------------------------------------------------

/// AC-3 / EC-002: Response with Unix epoch timestamp "1710500000" →
/// adapter parses it via parse_timestamp() and returns a valid RecordBatch.
///
/// The exact DateTime value is tested in test_timestamp.rs; here we verify
/// the adapter does not return an error when encountering the Unix epoch format.

#[tokio::test]
async fn test_BC_2_01_006_unix_epoch_timestamp_in_response_parsed_without_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Set-Cookie", "access_token=tok; Path=/")
                .set_body_json(serde_json::json!({ "status": "ok" })),
        )
        .mount(&server)
        .await;

    // Response with Unix epoch timestamp (AC-3 literal value)
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "alert-epoch",
                    "created_at": "1710500000",  // AC-3 literal value
                    "severity": "high"
                }
            ],
            "total": 1
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = CyberintAdapter::new(test_org_id(), &auth);
    let spec = make_spec();
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "AC-3: Unix epoch timestamp must not cause fetch to fail; got: {result:?}"
    );
}
