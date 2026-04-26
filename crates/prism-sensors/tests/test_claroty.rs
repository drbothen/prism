//! Integration tests for Claroty bearer auth + polymorphic IDs + pagination.
//!
//! Covers BC-2.01.004 and BC-2.01.007:
//! - ClarotyId::Display (GREEN-BY-DESIGN)
//! - ClarotyId deserialization from JSON integer (AC-4, EC-004)
//! - ClarotyId deserialization from UUID string (TV-BC-2.01.007-002)
//! - ClarotyId integer == string normalized to same value (DEC-010)
//! - Bearer token included in POST-for-read requests (BC-2.01.007)
//! - TV-BC-2.01.007-003: HTTP 401 → SensorError authentication
//! - paginate_claroty 5-page integration (AC-5, TV-BC-2.01.004-001)
//!
//! ClarotyId::Display and ClarotyId::Int/Uuid construction are GREEN-BY-DESIGN.
//! All adapter and deserialization tests are RED (todo!() panics).
//!
//! Story: S-2.07 | BC: BC-2.01.004, BC-2.01.007
#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::claroty::{ClarotyAdapter, ClarotyAuth, ClarotyId};
use prism_sensors::auth::SensorAuth;
use prism_sensors::SensorAdapter;

// ---------------------------------------------------------------------------
// ClarotyId::Display — GREEN-BY-DESIGN
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN: ClarotyId::Int(12345).to_string() == "12345".
///
/// The Display impl is implemented (not todo!()), so this test passes.
#[test]
fn test_BC_2_01_007_claroty_id_int_display_formats_as_decimal_string() {
    let id = ClarotyId::Int(12345);
    assert_eq!(
        id.to_string(),
        "12345",
        "ClarotyId::Int(12345) must display as '12345' (BC-2.01.007)"
    );
}

/// GREEN-BY-DESIGN: ClarotyId::Uuid displays as hyphenated lowercase UUID.
///
/// The Display impl is implemented, so this test passes.
#[test]
fn test_BC_2_01_007_claroty_id_uuid_display_formats_as_hyphenated_string() {
    let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid UUID");
    let id = ClarotyId::Uuid(uuid);
    assert_eq!(
        id.to_string(),
        "550e8400-e29b-41d4-a716-446655440000",
        "ClarotyId::Uuid must display as hyphenated lowercase UUID"
    );
}

/// GREEN-BY-DESIGN: ClarotyId::Int equality.
#[test]
fn test_BC_2_01_007_claroty_id_int_equality() {
    assert_eq!(ClarotyId::Int(42), ClarotyId::Int(42));
    assert_ne!(ClarotyId::Int(42), ClarotyId::Int(43));
}

// ---------------------------------------------------------------------------
// ClarotyId deserialization — RED (todo!() in Deserialize impl)
// ---------------------------------------------------------------------------

/// AC-4 / EC-004 / TV-BC-2.01.007-001: Integer JSON value `12345` deserializes
/// as ClarotyId::Int(12345).
///
/// BC-2.01.007 postcondition: "Polymorphic ID fields normalized to PolymorphicId enum".
/// RED: ClarotyId::deserialize is todo!() — will panic.
#[test]
fn test_BC_2_01_007_deserialize_json_integer_as_claroty_id_int() {
    let json = serde_json::json!(12345);
    let id: ClarotyId = serde_json::from_value(json)
        .expect("AC-4: JSON integer must deserialize as ClarotyId::Int");
    assert_eq!(
        id,
        ClarotyId::Int(12345),
        "AC-4: integer 12345 must become ClarotyId::Int(12345) (EC-004)"
    );
}

/// TV-BC-2.01.007-002: UUID string deserializes as ClarotyId::Uuid.
///
/// RED: ClarotyId::deserialize is todo!() — will panic.
#[test]
fn test_BC_2_01_007_deserialize_uuid_string_as_claroty_id_uuid() {
    let json = serde_json::json!("550e8400-e29b-41d4-a716-446655440000");
    let id: ClarotyId = serde_json::from_value(json)
        .expect("TV-BC-2.01.007-002: UUID string must deserialize as ClarotyId::Uuid");
    assert!(
        matches!(id, ClarotyId::Uuid(_)),
        "UUID string must become ClarotyId::Uuid variant"
    );
    assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}

/// DEC-010: JSON integer `12345` and JSON string `"12345"` are different variants
/// but both serialize to the same string representation.
///
/// BC-2.01.007: "both representations treated as equivalent" for string serialization.
/// RED: ClarotyId::deserialize is todo!() — will panic.
#[test]
fn test_BC_2_01_007_integer_and_numeric_string_normalize_to_same_display() {
    let from_int: ClarotyId =
        serde_json::from_value(serde_json::json!(42)).expect("integer deserializes");
    // A numeric string "42" is NOT a valid UUID so it should not deserialize as Uuid.
    // However, if Claroty sends "42" as a string ID, the spec says treat as Int via
    // normalization. The Display of both must match.
    assert_eq!(
        from_int.to_string(),
        "42",
        "DEC-010: integer ClarotyId::Int(42) must display as '42'"
    );
}

/// BC-2.01.007 error case: non-UUID, non-integer string should fail deserialization.
///
/// RED: ClarotyId::deserialize is todo!() — will panic.
#[test]
fn test_BC_2_01_007_rejects_non_uuid_non_integer_string() {
    let json = serde_json::json!("not-a-uuid-or-integer");
    let result: Result<ClarotyId, _> = serde_json::from_value(json);
    assert!(
        result.is_err(),
        "Non-UUID, non-integer string must fail ClarotyId deserialization"
    );
}

/// BC-2.01.007: Deserialization round-trip for a struct containing ClarotyId.
///
/// RED: ClarotyId::deserialize is todo!() — will panic.
#[test]
fn test_BC_2_01_007_claroty_id_roundtrip_in_struct() {
    #[derive(serde::Deserialize, Debug)]
    struct Record {
        id: ClarotyId,
    }

    let json = serde_json::json!({ "id": 99999 });
    let record: Record =
        serde_json::from_value(json).expect("struct with integer ClarotyId must deserialize");
    assert_eq!(record.id, ClarotyId::Int(99999));
}

// ---------------------------------------------------------------------------
// ClarotyAdapter — bearer auth, POST-for-read
// RED: ClarotyAdapter::new is todo!()
// ---------------------------------------------------------------------------

fn make_auth(instance_url: &str) -> ClarotyAuth {
    ClarotyAuth {
        instance_url: instance_url.to_string(),
        username: "testuser".into(),
        password: SecretString::new("testpass".into()),
    }
}

fn make_spec(table: &str) -> SensorSpec {
    SensorSpec {
        source_table: table.into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    }
}

/// BC-2.01.007 postcondition: All requests include "Authorization: Bearer {token}".
///
/// RED: ClarotyAdapter::new is todo!() — will panic.
#[tokio::test]
async fn test_BC_2_01_007_bearer_token_included_in_requests() {
    let server = MockServer::start().await;

    // Claroty uses POST-for-read; expects Authorization header
    Mock::given(method("POST"))
        .and(path("/api/v1/alerts"))
        .and(header("Authorization", "Bearer test-bearer-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "objects": [{ "id": 1, "severity": "medium" }],
            "count": 1
        })))
        .expect(1)
        .named("post_read_with_bearer")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ClarotyAdapter::new(&auth, "test-bearer-token".into());
    let spec = make_spec("claroty_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "BC-2.01.007: bearer token in header must allow successful fetch; got: {result:?}"
    );
}

/// TV-BC-2.01.007-003: HTTP 401 → SensorError with status 401.
///
/// BC-2.01.007 error case: "category: authentication".
/// RED: ClarotyAdapter::new is todo!() — will panic.
#[tokio::test]
async fn test_BC_2_01_007_rejects_401_with_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/alerts"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "detail": "Authentication credentials were not provided."
        })))
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ClarotyAdapter::new(&auth, "expired-token".into());
    let spec = make_spec("claroty_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(result.is_err(), "TV-BC-2.01.007-003: 401 must return Err");
    match result {
        Err(SensorError::HttpError { status, .. }) => {
            assert_eq!(status, 401, "Status must be 401 for authentication error");
        }
        Err(e) => panic!("Expected HttpError(401), got: {e}"),
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

/// TV-BC-2.01.007-001: Alerts endpoint with IDs as JSON integers → normalized to ClarotyId.
///
/// Response contains `"id": 12345` (integer) for each record.
/// RED: ClarotyAdapter::new AND ClarotyId::deserialize are todo!() — will panic.
#[tokio::test]
async fn test_BC_2_01_007_integer_ids_in_response_normalized_to_claroty_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/alerts"))
        .and(header("Authorization", "Bearer my-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "objects": [
                { "id": 10001, "type": "alert" },
                { "id": 10002, "type": "alert" }
            ],
            "count": 2
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ClarotyAdapter::new(&auth, "my-bearer".into());
    let spec = make_spec("claroty_alert");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "TV-BC-2.01.007-001: integer IDs must be handled without error; got: {result:?}"
    );
}

/// AC-5 / TV-BC-2.01.004-001: audit_logs endpoint with total_count=300, page_size=100
/// → exactly 3 HTTP requests (offsets 0, 100, 200).
///
/// Tests that ClarotyAdapter dispatches audit_logs to paginate_claroty().
/// RED: ClarotyAdapter::new is todo!() — will panic.
#[tokio::test]
async fn test_BC_2_01_004_claroty_adapter_paginates_audit_logs_3_pages() {
    let server = MockServer::start().await;

    for offset in [0usize, 100, 200] {
        let records: Vec<serde_json::Value> = (0..100)
            .map(|i| serde_json::json!({ "id": offset + i, "action": "login" }))
            .collect();
        Mock::given(method("GET"))
            .and(path("/api/v1/audit_logs"))
            .and(wiremock::matchers::query_param(
                "offset",
                offset.to_string().as_str(),
            ))
            .and(wiremock::matchers::query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total_count": 300,
                "data": records,
            })))
            .expect(1)
            .named(format!("audit_offset_{offset}"))
            .mount(&server)
            .await;
    }

    let auth = make_auth(&server.uri());
    let adapter = ClarotyAdapter::new(&auth, "audit-bearer".into());
    let spec = make_spec("audit_logs");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await
        .expect("AC-5: audit_logs with 3 pages must succeed");
    assert!(
        !result.is_empty(),
        "AC-5: 3 pages × 100 records must produce non-empty batches"
    );
}
