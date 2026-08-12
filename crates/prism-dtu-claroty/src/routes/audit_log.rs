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
use uuid::Uuid;

use crate::{
    routes::devices::{check_bearer_auth, validate_org_id},
    state::ClarotyState,
    types::GetAuditLogBody,
};

/// `POST /api/v1/audit_log/get`
///
/// Returns synthetic audit log entries from `fixtures/audit-log.json`.
/// Response: `{"audit_log": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-002).
///
/// Gap-CL-006: DTU-side closure. `claroty.sensor.toml` `response_path = "$.audit_log"`.
///
/// # W3-FIX-SEC-001 (AC-001..AC-003)
///
/// When the clone was created with a real (non-nil) `instance_org_id`, the
/// `X-Org-Id` header is validated against `state.instance_org_id` after bearer
/// auth — matching the guard used by `list_devices`. Nil-org clones (created via
/// `ClarotyClone::new()`) skip header validation for backward compatibility with
/// callers that do not supply an `X-Org-Id` header.
pub async fn list_audit_logs(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetAuditLogBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // W3-FIX-SEC-001: org-isolation guard.
    //
    // When instance_org_id is non-nil (real org clone), validate the X-Org-Id
    // header matches. Nil-org clones (ClarotyClone::new()) skip this check for
    // backward compat — nil UUID is the sentinel meaning "no org constraint".
    let nil_org = prism_core::OrgId::from_uuid(Uuid::nil());
    if state.instance_org_id != nil_org {
        if let Err(err) = validate_org_id(&headers, state.instance_org_id) {
            return err;
        }
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
// Tests — BC-2.16.013 / S-DEMO-CLAROTY-AUDIT-DTU-001
//
// All tests exercise the fully-implemented list_audit_logs handler.
// list_audit_logs performs auth-first ordering: check_bearer_auth is called
// before fixture loading, so auth failures return 401 immediately without
// touching the fixture.
//
// AC traces:
//   test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries → AC-001, AC-003, AC-004
//   test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced                   → AC-002, EC-001, EC-002
//   test_BC_2_16_013_claroty_audit_logs_dtu_column_parity                   → AC-005 (SAP-2)
//   test_W3_FIX_SEC_001_claroty_audit_logs_org_mismatch_returns_401                       → SEC-001 (org-isolation guard, non-nil clone, mismatched org)
//   test_W3_FIX_SEC_001_claroty_audit_logs_missing_org_header_on_real_org_returns_401    → SEC-001 (org-isolation guard, non-nil clone, absent header)
//   test_W3_FIX_SEC_001_claroty_audit_logs_nil_org_no_header_returns_200                 → SEC-001 (nil-org clone backward-compat)
//   test_BC_2_16_013_claroty_audit_logs_pipeline_integration_ac_006         → AC-006 (deferred, #[ignore])
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use prism_core::OrgId;
    use prism_dtu_common::BehavioralClone;
    use serde_json::json;
    use uuid::Uuid;

    use crate::clone::ClarotyClone;
    use crate::types::ClarotyAuditLogEntry;

    /// Build a test reqwest client with a 10-second timeout.
    ///
    /// CR-003: All test clients use an explicit timeout to prevent hung tests.
    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("test client build must succeed")
    }

    /// Start a fresh nil-org `ClarotyClone` and return (clone, base_url).
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

    /// Start a `ClarotyClone` with a specific non-nil org identity.
    ///
    /// Used by SEC-001 org-mismatch tests. The clone enforces X-Org-Id validation
    /// because `instance_org_id` is non-nil.
    async fn start_clone_with_org(org_id: OrgId) -> (ClarotyClone, String) {
        let mut clone = ClarotyClone::with_org(org_id);
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
    /// GREEN: list_audit_logs is fully implemented — auth check passes, fixture loads,
    /// JSON envelope {"audit_log": [...], "total": N} is returned with HTTP 200.
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

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

        // AC-004: entries contain no obviously real PII — all user_display_name fields
        // use the synthetic example.com domain or demo identifiers.
        for (i, entry) in audit_log.iter().enumerate() {
            let user_display_name = entry
                .get("user_display_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            // Synthetic fixture user_display_name values use @example.com per Task 2 guidance.
            assert!(
                user_display_name.contains("example.com") || user_display_name.is_empty(),
                "entry[{i}] user_display_name must use synthetic example.com domain (AC-004, ADR-031 §D2); \
                 got {user_display_name:?}"
            );
        }
    }

    /// BC-2.16.013 postcondition §2 (auth enforcement).
    ///
    /// AC-002: POST /api/v1/audit_log/get without bearer → HTTP 401 with exact JSON body.
    /// EC-001: Missing Authorization header entirely → 401.
    /// EC-002: Authorization: Bearer  (empty token after space) → 401.
    ///
    /// GREEN: list_audit_logs checks auth first via check_bearer_auth before fixture loading.
    /// The canonical 401 body is `{"error": "missing or invalid Authorization header", "code": 401}`
    /// (POL-24), produced by check_bearer_auth in routes/devices.rs.
    /// Auth is verified before any other processing — fixture load only occurs after auth passes.
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

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

        // AC-002: 401 body must match the canonical literal from check_bearer_auth (POL-24):
        //   {"error": "missing or invalid Authorization header", "code": 401}
        let body_no_auth: serde_json::Value = resp_no_auth
            .json()
            .await
            .expect("401 response must have JSON body (AC-002)");
        assert_eq!(
            body_no_auth.get("error").and_then(|v| v.as_str()),
            Some("missing or invalid Authorization header"),
            "EC-001: 401 body `error` must be exactly \
             \"missing or invalid Authorization header\" (AC-002, POL-24); \
             got: {body_no_auth}"
        );
        assert_eq!(
            body_no_auth.get("code").and_then(|v| v.as_u64()),
            Some(401),
            "EC-001: 401 body must contain `code: 401`; got: {body_no_auth} (AC-002)"
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

        // EC-002: 401 body must match the canonical literal from check_bearer_auth (POL-24):
        //   {"error": "missing or invalid Authorization header", "code": 401}
        let body_empty_bearer: serde_json::Value = resp_empty_bearer
            .json()
            .await
            .expect("EC-002: 401 response must have JSON body (AC-002)");
        assert_eq!(
            body_empty_bearer.get("error").and_then(|v| v.as_str()),
            Some("missing or invalid Authorization header"),
            "EC-002: 401 body `error` must be exactly \
             \"missing or invalid Authorization header\" (AC-002, POL-24); \
             got: {body_empty_bearer}"
        );
        assert_eq!(
            body_empty_bearer.get("code").and_then(|v| v.as_u64()),
            Some(401),
            "EC-002: 401 body must contain `code: 401`; got: {body_empty_bearer} (AC-002)"
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
    ///   - id               → String  (column_type = "string")
    ///   - action           → String  (column_type = "string")
    ///   - user_display_name → String  (column_type = "string") — real xDome field
    ///   - category         → String  (column_type = "string") — real xDome field
    ///   - timestamp        → String  (column_type = "datetime", ISO 8601 per ADR-028 §D8)
    ///   - details          → String  (column_type = "string") — real xDome field
    ///   - username         → String  (column_type = "string") — real xDome field
    ///   - note             → Option<String>  (column_type = "string") — absent from some records
    ///
    /// `actor` and `resource` do NOT exist in the xDome API (LIVE-DRIFT-003).
    ///
    /// This test deserializes the fixture file into Vec<ClarotyAuditLogEntry> to exercise
    /// all 8 field paths simultaneously. A missing or mis-typed field causes a serde
    /// deserialization error → test fails with an informative message.
    ///
    /// GREEN (Part 1): Direct fixture deserialization verifies struct-field parity against
    /// the TOML column declarations. This is a compile-time + runtime structural check —
    /// if ClarotyAuditLogEntry has a missing or wrongly-typed field, serde fails here.
    ///
    /// GREEN (Part 2): HTTP round-trip boots the DTU and verifies the response
    /// `audit_log` array deserializes cleanly into Vec<ClarotyAuditLogEntry>. This
    /// exercises the full path: check_bearer_auth → fixture load → JSON envelope → client parse.
    #[tokio::test]
    async fn test_BC_2_16_013_claroty_audit_logs_dtu_column_parity() {
        // Part 1: Direct fixture deserialization (SAP-2 compile-time parity check).
        //
        // Deserializing into Vec<ClarotyAuditLogEntry> verifies:
        //   - All 8 required field names are present (id, action, user_display_name, category,
        //     timestamp, details, username, note)
        //   - All field types are compatible with the fixture JSON
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
                     This means a TOML column (id/action/user_display_name/category/timestamp/\
                     details/username/note) has no matching field in ClarotyAuditLogEntry — \
                     SAP-2 P1 CRITICAL (AC-005)"
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
            !first.user_display_name.is_empty(),
            "ClarotyAuditLogEntry.user_display_name (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.category.is_empty(),
            "ClarotyAuditLogEntry.category (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.timestamp.is_empty(),
            "ClarotyAuditLogEntry.timestamp (column_type=datetime, ISO 8601) must be non-empty (AC-005)"
        );
        assert!(
            !first.details.is_empty(),
            "ClarotyAuditLogEntry.details (column_type=string) must be non-empty (AC-005)"
        );
        assert!(
            !first.username.is_empty(),
            "ClarotyAuditLogEntry.username (column_type=string) must be non-empty (AC-005)"
        );
        // note is Option<String> — presence not required on every entry, only checked type

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
        // Exercises the full path: check_bearer_auth → fixture load → JSON envelope → client parse.
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

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
        // Fields checked: id, action, user_display_name, category, timestamp, details, username, note.
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

    /// W3-FIX-SEC-001 — org-isolation guard (non-nil clone, mismatched org).
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request whose
    /// `X-Org-Id` header does not match `instance_org_id` must return HTTP 401.
    ///
    /// Mirrors the sibling guard in `list_devices` (W3-FIX-SEC-001).
    ///
    /// Verifies:
    /// - Valid bearer + mismatched X-Org-Id → 401 `{"error": "org_id mismatch: ..."}`
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_audit_logs_org_mismatch_returns_401() {
        // Instance org: a real, non-nil UUID.
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        // A different org UUID sent by the caller.
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            .header("X-Org-Id", caller_org.to_string())
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for org-mismatch test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + mismatched X-Org-Id must return HTTP 401 (W3-FIX-SEC-001)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — org-isolation guard: non-nil clone + absent X-Org-Id header → 401.
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request that omits the
    /// `X-Org-Id` header entirely must return HTTP 401. `validate_org_id` treats an
    /// absent header as a mismatch (AC-003: missing header → 401).
    ///
    /// This closes the org-isolation test matrix for `list_audit_logs`:
    ///   test_W3_FIX_SEC_001_claroty_audit_logs_org_mismatch_returns_401      → non-nil + MISMATCH → 401
    ///   test_W3_FIX_SEC_001_claroty_audit_logs_missing_org_header_on_real_org_returns_401 → non-nil + ABSENT → 401 (this test)
    ///   test_W3_FIX_SEC_001_claroty_audit_logs_nil_org_no_header_returns_200 → nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_audit_logs_missing_org_header_on_real_org_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for missing-header test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + absent X-Org-Id header must return HTTP 401 (W3-FIX-SEC-001 AC-003)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (W3-FIX-SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "W3-FIX-SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 backward-compat: nil-org clone without X-Org-Id header must return 200.
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel meaning
    /// "no org constraint". Callers that do not supply `X-Org-Id` must not be rejected.
    ///
    /// This test verifies the guard does NOT fire for nil-org clones, preserving
    /// the fidelity_validator route-11 (401-without-auth, nil-org, no X-Org-Id) behavior.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_audit_logs_nil_org_no_header_returns_200() {
        // Nil-org clone — no org enforcement.
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/audit_log/get"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for nil-org backward-compat test");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "nil-org clone without X-Org-Id header must return HTTP 200 (SEC-001 backward-compat)"
        );
    }

    /// AC-006 — spec-driven pipeline integration: `FROM claroty_audit_logs LIMIT 10`
    /// → `extract_at_path("$.audit_log")`.
    ///
    /// Deferred to S-DEMO-002 (prism-bin full-boot wiring not yet available).
    /// Per SID-1, this stub is present with an explicit blocking story cited.
    ///
    /// // DTU-EXT: requires prism-bin full-boot; ungated after S-DEMO-002 merges.
    #[tokio::test]
    #[ignore = "AC-006: requires prism-bin full-boot wiring; ungated after S-DEMO-002 merges"]
    async fn test_BC_2_16_013_claroty_audit_logs_pipeline_integration_ac_006() {
        // This test will exercise the full pipeline:
        //   PrismQL: `FROM claroty_audit_logs LIMIT 10`
        //   → sensor adapter → DTU POST /api/v1/audit_log/get
        //   → extract_at_path("$.audit_log")
        //   → DataFusion table → query result rows
        //
        // Blocked on: prism-bin full-boot wiring (S-DEMO-002).
        // When unblocked: start a full prism binary, execute the PrismQL query,
        // assert at least 5 rows returned with columns id, action, actor, timestamp, resource.
        todo!("implement in S-DEMO-002: full prism-bin boot + PrismQL pipeline integration")
    }
}
