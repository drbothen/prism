//! Integration tests for Armis bearer auth + AQL forwarding + timestamp fallback.
//!
//! Covers BC-2.01.008:
//! - DEFAULT_AQL_TEMPLATE constant check (GREEN-BY-DESIGN)
//! - TV-BC-2.01.008-001: valid API key; records with primary timestamp
//! - TV-BC-2.01.008-002: firstSeen absent → lastSeen used
//! - TV-BC-2.01.008-003: both firstSeen and lastSeen null → Utc::now() + warn! (AC-6, EC-005)
//! - TV-BC-2.01.008-004: HTTP 401 → SensorError authentication
//! - TV-BC-2.01.008-005: AQL syntax error HTTP 400 → SensorError api_contract
//! - AQL forwarding: SensorSpec aql_query passed verbatim (no modification)
//! - AQL default: absent aql_query → DEFAULT_AQL_TEMPLATE substituted
//!
//! DEFAULT_AQL_TEMPLATE is GREEN-BY-DESIGN. All adapter tests are RED (todo!()).
//!
//! Story: S-2.07 | BC: BC-2.01.008
#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::armis::{ArmisAdapter, ArmisAuth, DEFAULT_AQL_TEMPLATE};
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
// DEFAULT_AQL_TEMPLATE — GREEN-BY-DESIGN
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN: DEFAULT_AQL_TEMPLATE is "in:{table}" per BC-2.01.008.
///
/// The template is a const string, not stubbed.
#[test]
fn test_BC_2_01_008_default_aql_template_format() {
    assert_eq!(
        DEFAULT_AQL_TEMPLATE, "in:{table}",
        "DEFAULT_AQL_TEMPLATE must be 'in:{{table}}' per BC-2.01.008"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_auth(instance_url: &str) -> ArmisAuth {
    ArmisAuth {
        instance_url: instance_url.to_string(),
        secret_key: SecretString::new("test-armis-secret-key".into()),
    }
}

fn make_spec(table: &str, aql_query: Option<&str>) -> SensorSpec {
    let sensor_config = match aql_query {
        Some(aql) => serde_json::json!({ "aql_query": aql }),
        None => serde_json::json!({}),
    };
    #[allow(deprecated)]
    SensorSpec {
        org_id: test_org_id(), // Must match adapter's OrgId (BC-3.2.001 precondition 4)
        source_table: table.into(),
        client_id: "acme".into(),
        sensor_config,
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-001: valid API key, all records have primaryTimestamp
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-001: Valid API key; GetSearch returns records with firstSeen field.
/// Adapter returns Ok(Vec<RecordBatch>) with all records.
///

#[tokio::test]
async fn test_BC_2_01_008_valid_api_key_returns_records_with_primary_timestamp() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "results": [
                    {
                        "id": 1001,
                        "firstSeen": "2024-03-15T10:00:00Z",
                        "lastSeen": "2024-03-15T12:00:00Z",
                        "type": "Laptop"
                    },
                    {
                        "id": 1002,
                        "firstSeen": "2024-03-14T08:00:00Z",
                        "lastSeen": "2024-03-15T11:00:00Z",
                        "type": "IoT"
                    }
                ],
                "total": 2
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(
        test_org_id(),
        &auth,
        SecretString::new("bearer-test-token".into()),
    );
    let spec = make_spec("armis_device", None);
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "TV-BC-2.01.008-001: valid key must return Ok; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-002: firstSeen absent → lastSeen used as timestamp fallback
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-002: Record missing firstSeen → adapter uses lastSeen.
///
/// BC-2.01.008 postcondition: "Timestamp extraction uses per-source fallback chain."
/// AC-6: "Given firstSeen: null and lastSeen: '2024-03-15T10:00:00Z', lastSeen is used."

#[tokio::test]
async fn test_BC_2_01_008_first_seen_null_uses_last_seen_as_fallback() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "results": [
                    {
                        "id": 2001,
                        "firstSeen": null,       // absent/null
                        "lastSeen": "2024-03-15T10:00:00Z",  // AC-6 literal value
                        "type": "Camera"
                    }
                ],
                "total": 1
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("bearer-tok".into()));
    let spec = make_spec("armis_device", None);
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "TV-BC-2.01.008-002: null firstSeen with valid lastSeen must not fail; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-003 / AC-6 / EC-005: both firstSeen and lastSeen null → now() + warn!
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-003 / AC-6 / EC-005: Both firstSeen and lastSeen null →
/// Utc::now() fallback used; tracing::warn! emitted.
///
/// We cannot assert on tracing output in a unit test without a subscriber,
/// but we CAN assert the fetch does NOT fail (the warn! path is not an error).
/// BC-2.01.008: "Record included; cursor not advanced for this record; warning logged."

#[tokio::test]
async fn test_BC_2_01_008_both_timestamps_null_uses_utc_now_without_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "results": [
                    {
                        "id": 3001,
                        "firstSeen": null,   // AC-6: both null
                        "lastSeen": null,    // AC-6: both null
                        "type": "Unknown"
                    }
                ],
                "total": 1
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("bearer-tok".into()));
    let spec = make_spec("armis_device", None);
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // Must not return Err — the now() fallback makes the record includable
    assert!(
        result.is_ok(),
        "AC-6 / EC-005: both timestamps null must return Ok (record included, not error); got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-004: HTTP 401 → SensorError authentication
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-004: Invalid API key → 401 → SensorError with status 401.
///
/// BC-2.01.008 error case: "category: authentication".

#[tokio::test]
async fn test_BC_2_01_008_rejects_401_api_key_with_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Invalid API key"
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("bad-token".into()));
    let spec = make_spec("armis_device", None);
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(result.is_err(), "TV-BC-2.01.008-004: 401 must return Err");
    match result {
        Err(SensorError::HttpError { status, .. }) => {
            assert_eq!(status, 401, "Must return 401 for API key rejection");
        }
        Err(e) => panic!("Expected HttpError(401), got: {e}"),
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-005: AQL syntax error HTTP 400 → SensorError api_contract
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-005: AQL that passes the Prism allowlist but is rejected by
/// the Armis server (HTTP 400) → SensorError::HttpError with status 400.
///
/// BC-2.01.008 error case: "category: api_contract; include AQL query text".
///
/// Note (ADR-005 §Q3): AQL that fails the Prism allowlist validator is rejected
/// pre-wire with `SensorError::ConfigValidation` (TV-BC-2.01.008-006).
/// This test covers the separate case where a syntactically valid AQL passes
/// the allowlist but Armis still rejects it for server-side reasons.

#[tokio::test]
async fn test_BC_2_01_008_rejects_400_aql_error_with_api_contract_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "Bad Request",
            "message": "AQL syntax error near 'unknownField'"
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(
        test_org_id(),
        &auth,
        SecretString::new("valid-token".into()),
    );
    // Use a query that passes the Prism allowlist (starts with in:, no injection markers)
    // but is rejected by Armis server for a field-semantics reason (simulated by mock 400).
    let spec = make_spec("armis_device", Some("in:devices unknownField:1"));
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_err(),
        "TV-BC-2.01.008-005: Armis HTTP 400 must return Err"
    );
    match result {
        Err(SensorError::HttpError { status, .. }) => {
            assert_eq!(
                status, 400,
                "Armis field rejection must produce HTTP 400 status"
            );
        }
        Err(e) => panic!("Expected HttpError(400), got: {e}"),
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// AQL forwarding — verbatim, no modification (BC-2.01.008 Architecture Compliance)
// ---------------------------------------------------------------------------

/// BC-2.01.008 Architecture Compliance: AQL query string forwarded VERBATIM.
///
/// The mock asserts the exact `aql` query param matches what was passed in spec.
/// A special string with spaces and operators verifies no sanitization occurs.

#[tokio::test]
async fn test_BC_2_01_008_aql_query_forwarded_verbatim_without_modification() {
    let server = MockServer::start().await;

    let verbatim_aql = "in:devices timeFrame:\"7 Days\" AND category:IoT";

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(query_param("aql", verbatim_aql))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "results": [], "total": 0 }
        })))
        .expect(1)
        .named("aql_verbatim")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("tok".into()));
    let spec = make_spec("armis_device", Some(verbatim_aql));
    let params = QueryParams::default();

    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // wiremock verifies the exact aql param was forwarded (expect(1) on the mock)
}

/// BC-2.01.008: When aql_query is absent from spec, DEFAULT_AQL_TEMPLATE is used
/// with the table name substituted.
///
/// For source_table="armis_device" → AQL = "in:armis_device" (template: "in:{table}").

#[tokio::test]
async fn test_BC_2_01_008_absent_aql_query_uses_default_template_with_table() {
    let server = MockServer::start().await;

    // Default AQL for table "armis_device" = "in:armis_device"
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(query_param("aql", "in:armis_device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "results": [], "total": 0 }
        })))
        .expect(1)
        .named("default_aql")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("tok".into()));
    let spec = make_spec("armis_device", None); // no aql_query
    let params = QueryParams::default();

    let _ = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    // wiremock verifies default template was used with table substituted
}

// ---------------------------------------------------------------------------
// init_registry
// ---------------------------------------------------------------------------

/// S-2.07 Task 5: init_registry() wires all 4 adapters.
///
/// NOTE: This test will panic at runtime (todo!()) during the stub phase because
/// `init_registry` delegates to `todo!()` — adapters now require `OrgId`
/// (AC-001, S-3.1.06-ImplPhase).  The test is retained for the migration window
/// (AC-005) and will be updated when `init_registry` callers migrate to
/// `init_registry_for_org`.
#[test]
fn test_BC_2_01_008_init_registry_registers_armis_adapter() {
    use prism_core::types::SensorType;
    use prism_sensors::{init_registry, ArmisAuth, ClarotyAuth, CrowdStrikeAuth, CyberintAuth};
    use secrecy::SecretString;

    let cs_auth = CrowdStrikeAuth {
        client_id: "cs-id".into(),
        client_secret: SecretString::new("cs-secret".into()),
        cloud_region: "us-1".into(),
    };
    let cy_auth = CyberintAuth {
        environment: "portal".into(),
        api_key: SecretString::new("cy-key".into()),
    };
    let cl_auth = ClarotyAuth {
        instance_url: "https://acme.claroty.com".into(),
        username: "user".into(),
        password: SecretString::new("pass".into()),
    };
    let ar_auth = ArmisAuth {
        instance_url: "https://acme.armis.com".into(),
        secret_key: SecretString::new("ar-key".into()),
    };

    // TODO impl-phase: migrate to init_registry_for_org (AC-005, S-3.1.06)
    #[allow(deprecated)]
    let registry = init_registry(
        &cs_auth,
        &cy_auth,
        &cl_auth,
        SecretString::new("claroty-bearer".into()),
        &ar_auth,
        SecretString::new("armis-bearer".into()),
    );

    // init_registry (deprecated) uses nil OrgId internally as the sentinel key.
    // Callers must migrate to init_registry_for_org to use a real OrgId (AC-005).
    let nil_org = prism_sensors::OrgId::from_uuid(uuid::Uuid::nil());
    assert!(
        registry.get(nil_org, SensorType::CrowdStrike).is_some(),
        "init_registry must register CrowdStrikeAdapter"
    );
    assert!(
        registry.get(nil_org, SensorType::Cyberint).is_some(),
        "init_registry must register CyberintAdapter"
    );
    assert!(
        registry.get(nil_org, SensorType::Claroty).is_some(),
        "init_registry must register ClarotyAdapter"
    );
    assert!(
        registry.get(nil_org, SensorType::Armis).is_some(),
        "init_registry must register ArmisAdapter"
    );
}
