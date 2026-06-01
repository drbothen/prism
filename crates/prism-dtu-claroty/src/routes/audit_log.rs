//! Route handler for the Claroty xDome audit log endpoint.
//!
//! `POST /api/v1/audit_log/get` — audit log list (Gap-CL-006 closure).
//!
//! Response envelope: `{"audit_log": [...], "total": N}`.
//! The key `audit_log` matches `response_path = "$.audit_log"` in
//! `claroty.sensor.toml`. Requires valid `Authorization: Bearer` header (AC-002).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};

use crate::{routes::devices::check_bearer_auth, state::ClarotyState, types::GetAuditLogBody};

/// `POST /api/v1/audit_log/get`
///
/// Returns synthetic audit log entries from `fixtures/audit-log.json`.
/// Response: `{"audit_log": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-002).
///
/// Gap-CL-006: DTU-side closure. `claroty.sensor.toml` `response_path = "$.audit_log"`.
pub async fn list_audit_logs(
    State(_state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetAuditLogBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // SAFETY: fixture files are bundled at build time; missing fixture is a build error, not runtime condition.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "audit-log")
        .expect("fixtures/audit-log.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let entries = raw
        .as_array()
        .expect("audit-log fixture must be a JSON array")
        .clone();
    let total = entries.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"audit_log": entries, "total": total})),
    )
}

// ---------------------------------------------------------------------------
// Red Gate tests — BC-2.16.013 / S-DEMO-CLAROTY-AUDIT-DTU-001
//
// All three tests MUST FAIL until the todo!() in list_audit_logs is implemented.
// Failure reason: tokio's test runner propagates the todo!() panic as a test failure.
//
// AC traces:
//   test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries → AC-001, AC-003, AC-004
//   test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced                   → AC-002, EC-001, EC-002
//   test_BC_2_16_013_claroty_audit_logs_dtu_column_parity                   → AC-005 (SAP-2)
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use prism_dtu_common::BehavioralClone;
    use serde_json::json;

    use crate::clone::ClarotyClone;
    use crate::types::ClarotyAuditLogEntry;

    /// Start a fresh `ClarotyClone` and return (clone, base_url).
    ///
    /// Used by the HTTP-level tests. The clone binds to an ephemeral port on
    /// 127.0.0.1 and is dropped (stopping the server) at end of the test.
    async fn start_clone() -> (ClarotyClone, String) {
        let mut clone = ClarotyClone::new();
        clone
            .start()
            .await
            .expect("ClarotyClone::start must succeed in test environment");
        let base_url = clone.base_url();
        (clone, base_url)
    }

    /// BC-2.16.013 postcondition §1 (DTU-Parity) + postcondition §2 (fixture-parity)
    /// + postcondition §3 (synthetic-fixture-data).
    ///
    /// AC-001: POST /api/v1/audit_log/get with valid bearer returns HTTP 200.
    /// AC-003: Response envelope contains `audit_log` key (matches response_path = "$.audit_log").
    /// AC-004: `audit_log` array is non-empty (≥ 5 synthetic entries; no real PII).
    ///
    /// Red Gate: MUST FAIL (todo!() panic) until list_audit_logs is implemented.
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries() {
        let (_clone, base_url) = start_clone().await;
        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await
            .expect("POST /api/v1/audit_log/get must not fail at transport level");

        // AC-001: route registered and reachable (not 404).
        assert_eq!(
            resp.status().as_u16(),
            200,
            "POST /api/v1/audit_log/get with valid bearer must return HTTP 200 (AC-001)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("response body must be valid JSON (AC-003)");

        // AC-003: envelope key matches response_path = "$.audit_log".
        let audit_log = body.get("audit_log").and_then(|v| v.as_array()).expect(
            "response must contain `audit_log` JSON array key matching \
                 claroty.sensor.toml response_path=\"$.audit_log\" (AC-003)",
        );

        // AC-004: at least 5 synthetic entries (fixture has exactly 5 per Task 2).
        assert!(
            audit_log.len() >= 5,
            "audit_log fixture must contain at least 5 synthetic entries; got {} (AC-004)",
            audit_log.len()
        );

        // AC-003 + AC-004: `total` field present and matches array length.
        let total = body
            .get("total")
            .and_then(|v| v.as_u64())
            .expect("response must contain `total` numeric field (AC-003)");
        assert_eq!(
            total as usize,
            audit_log.len(),
            "`total` must equal the length of the `audit_log` array (AC-003)"
        );

        // AC-004: entries contain no obviously real PII — all actor/resource fields
        // use the synthetic example.com domain or demo identifiers.
        for (i, entry) in audit_log.iter().enumerate() {
            let actor = entry
                .get("actor")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            // Synthetic fixture actors use @example.com per Task 2 guidance.
            assert!(
                actor.contains("example.com") || actor.is_empty(),
                "entry[{i}] actor must use synthetic example.com domain (AC-004, ADR-031 §D2); \
                 got {actor:?}"
            );
        }
    }

    /// BC-2.01.013 postcondition §2 (auth enforcement).
    ///
    /// AC-002: POST /api/v1/audit_log/get without bearer → HTTP 401 with exact JSON body.
    /// EC-001: Missing Authorization header entirely → 401.
    /// EC-002: Authorization: Bearer  (empty token after space) → 401.
    ///
    /// Red Gate: MUST FAIL (todo!() panic) until list_audit_logs is implemented.
    /// Note: the todo!() fires even before the auth check because check_bearer_auth
    /// result is currently ignored (stub drops the value).
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced() {
        let (_clone, base_url) = start_clone().await;
        let client = reqwest::Client::new();

        // EC-001: No Authorization header at all → 401.
        let resp_no_auth = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for auth test");

        assert_eq!(
            resp_no_auth.status().as_u16(),
            401,
            "missing Authorization header must return HTTP 401 (AC-002, EC-001)"
        );

        // AC-002: 401 body must match the canonical pattern from check_bearer_auth.
        let body_no_auth: serde_json::Value = resp_no_auth
            .json()
            .await
            .expect("401 response must have JSON body (AC-002)");
        assert!(
            body_no_auth.get("error").is_some(),
            "401 body must contain `error` field; got: {body_no_auth} (AC-002)"
        );
        assert_eq!(
            body_no_auth.get("code").and_then(|v| v.as_u64()),
            Some(401),
            "401 body must contain `code: 401`; got: {body_no_auth} (AC-002)"
        );

        // EC-002: Bearer header with empty token value → 401.
        let resp_empty_bearer = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer ")
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for empty-bearer test");

        assert_eq!(
            resp_empty_bearer.status().as_u16(),
            401,
            "Authorization: Bearer  (empty token) must return HTTP 401 (EC-002)"
        );

        // EC-003: malformed (non-JSON) request body with valid bearer → 200 (body ignored).
        // Exercises AC-003 permissive body handling.
        let resp_bad_body = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .body("not json at all!!!")
            .send()
            .await
            .expect("transport must not fail for bad-body test");

        // With valid auth, handler must return 200 (body is Option<Json<...>> — parse failure
        // yields None, fixture returned regardless per AC-003 / EC-003).
        assert_eq!(
            resp_bad_body.status().as_u16(),
            200,
            "malformed body with valid bearer must return HTTP 200 (EC-003)"
        );
    }

    /// BC-2.16.013 postcondition §2 (DTU-TOML-column-parity) — SAP-2 compile-time probe.
    ///
    /// AC-005: Every column declared in claroty.sensor.toml audit_logs table maps 1:1
    /// to a field in ClarotyAuditLogEntry with the correct Rust type:
    ///   - id        → String  (column_type = "string")
    ///   - action    → String  (column_type = "string")
    ///   - actor     → String  (column_type = "string")
    ///   - timestamp → String  (column_type = "datetime", ISO 8601 per ADR-028 §D8)
    ///   - resource  → String  (column_type = "string")
    ///
    /// This test deserializes the fixture file into Vec<ClarotyAuditLogEntry> to exercise
    /// all 5 field paths simultaneously. A missing or mis-typed field causes a serde
    /// deserialization error → test fails with an informative message.
    ///
    /// Red Gate: MUST FAIL (todo!() panic from list_audit_logs called indirectly)
    /// — the fixture deserialization itself does NOT call list_audit_logs, so the
    /// panic path is NOT through list_audit_logs.
    ///
    /// IMPORTANT: This test's Red Gate failure is by COMPILATION FAILURE or serde
    /// assertion, not by todo!(). If the struct fields are mis-named or wrong type,
    /// this test will fail with a serde error. If the struct is missing a field, the
    /// struct definition itself would not compile. The Red Gate here is structural.
    ///
    /// Additional HTTP-level enforcement: we also boot the DTU and verify the
    /// deserialized fixture round-trips cleanly through the HTTP response path.
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_column_parity() {
        // Part 1: Direct fixture deserialization (SAP-2 compile-time parity check).
        //
        // Deserializing into Vec<ClarotyAuditLogEntry> verifies:
        //   - All 5 required field names are present (id, action, actor, timestamp, resource)
        //   - All field types are compatible with the fixture JSON (all strings)
        // This exercises AC-005 without requiring the server to be running.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixture_path = format!("{manifest_dir}/fixtures/audit-log.json");
        let fixture_raw = std::fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
            panic!(
                "fixtures/audit-log.json must exist and be readable; error: {e} (AC-004, Task 2)"
            )
        });

        let entries: Vec<ClarotyAuditLogEntry> =
            serde_json::from_str(&fixture_raw).unwrap_or_else(|e| {
                panic!(
                    "fixtures/audit-log.json must deserialize into Vec<ClarotyAuditLogEntry>; \
                     serde error: {e}\n\
                     This means a TOML column (id/action/actor/timestamp/resource) has no \
                     matching field in ClarotyAuditLogEntry — SAP-2 P1 CRITICAL (AC-005)"
                )
            });

        // AC-004: fixture must have at least 5 entries.
        assert!(
            entries.len() >= 5,
            "audit-log.json must contain at least 5 synthetic entries; got {} (AC-004)",
            entries.len()
        );

        // AC-005: spot-check each required field is non-empty on the first entry.
        let first = &entries[0];
        assert!(
            !first.id.is_empty(),
            "ClarotyAuditLogEntry.id (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.action.is_empty(),
            "ClarotyAuditLogEntry.action (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.actor.is_empty(),
            "ClarotyAuditLogEntry.actor (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.timestamp.is_empty(),
            "ClarotyAuditLogEntry.timestamp (column_type=datetime, ISO 8601) must be non-empty (AC-005)"
        );
        assert!(
            !first.resource.is_empty(),
            "ClarotyAuditLogEntry.resource (column_type=string) must be non-empty (AC-005)"
        );

        // AC-005: timestamp must parse as ISO 8601 (ADR-028 §D8).
        // Use a simple check: must contain 'T' and end with 'Z' (or contain '+').
        assert!(
            first.timestamp.contains('T')
                && (first.timestamp.ends_with('Z') || first.timestamp.contains('+')),
            "ClarotyAuditLogEntry.timestamp must be ISO 8601 format (ADR-028 §D8); \
             got {:?} (AC-005)",
            first.timestamp
        );

        // Part 2: HTTP round-trip — boot the DTU and verify the response
        // `audit_log` array deserializes cleanly into Vec<ClarotyAuditLogEntry>.
        // This exercises the full path: handler → fixture load → JSON envelope → client parse.
        // Red Gate: this section panics via todo!() until list_audit_logs is implemented.
        let (_clone, base_url) = start_clone().await;
        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "column parity test: HTTP 200 required (AC-001)"
        );

        let body: serde_json::Value = resp.json().await.expect("body must be JSON");
        let audit_log_value = body
            .get("audit_log")
            .expect("response must have `audit_log` key (AC-003)");

        // Deserialize the HTTP response array into Vec<ClarotyAuditLogEntry>.
        // Fails if any column in the response doesn't match the struct (SAP-2).
        let response_entries: Vec<ClarotyAuditLogEntry> =
            serde_json::from_value(audit_log_value.clone()).unwrap_or_else(|e| {
                panic!(
                    "HTTP response `audit_log` array must deserialize into \
                     Vec<ClarotyAuditLogEntry>; serde error: {e} (AC-005)"
                )
            });

        assert!(
            !response_entries.is_empty(),
            "HTTP response audit_log must be non-empty (AC-004)"
        );
    }
}
