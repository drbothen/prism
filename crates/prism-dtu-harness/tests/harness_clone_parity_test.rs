//! Harness Clone Route Parity — S-DEMO-HARNESS-CLONE-PARITY-001 Red Gate
//!
//! Covers:
//!   BC-2.16.013 INV-HARNESS-ROUTE-PARITY
//!
//! # Red Gate
//!
//! ALL tests in this file MUST FAIL before implementation. The two new routes do
//! not exist yet — requests will 404. If any test passes before implementation,
//! flag it for spec-reviewer review.
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern throughout (Factory TDD spec).
//!
//! # Idiom
//!
//! Tests use the reqwest-over-TcpListener idiom established in
//! `tests/logical_isolation_test.rs`. No tower::ServiceExt::oneshot — tower is not
//! a prism-dtu-harness dependency. Each test spins up a real Harness (which starts
//! real axum servers on ephemeral 127.0.0.1 ports) and sends HTTP requests via
//! reqwest.
//!
//! # Token acquisition
//!
//! Armis tests obtain the clone's REAL admin token via
//! `harness.admin_token_for(slug, DtuType::Armis)`. An arbitrary "Bearer test-token"
//! yields 401 (present-but-wrong token mismatch), NOT 200. This is the #1 Red Gate
//! trap documented in C-3 / AC-001.
//!
//! Claroty tests use any non-empty Bearer token (Claroty's check_bearer_auth
//! accepts ANY non-empty bearer, returning 401 only on missing/empty bearer).

// Allow test-file conventions used across all harness tests.
#![allow(clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// Shared test helpers
// ============================================================================

/// Build a reqwest Client with a 10-second timeout.
///
/// All test HTTP clients must use an explicit timeout (CR-003 precedent).
fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("test client build must succeed")
}

/// Get the SocketAddr for a given (slug, dtu_type) in the harness.
///
/// Panics if not found — used only in tests where the endpoint is known to exist.
fn get_addr(
    harness: &prism_dtu_harness::Harness,
    slug: &str,
    dtu_type: DtuType,
) -> std::net::SocketAddr {
    harness
        .endpoint_for(slug, dtu_type)
        .unwrap_or_else(|| panic!("no endpoint for slug={slug:?} dtu_type={dtu_type:?}"))
}

// ============================================================================
// AC-001 — Armis harness clone registers GET /api/v1/search
//
// test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY:
//   armis::router() MUST include GET /api/v1/search
//   Armis auth model: 403 on missing/invalid Bearer
//
// Red Gate failure mode:
//   GET /api/v1/search → 404 (route not yet registered)
// ============================================================================

/// AC-001: Armis harness clone GET /api/v1/search — 200 with real admin token, 403 without Bearer.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — armis::router() MUST include
/// GET /api/v1/search; Armis auth model: 403 on missing/malformed Bearer)
///
/// Red Gate: GET /api/v1/search → 404 (route not registered).
#[tokio::test]
async fn test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "test-tenant", DtuType::Armis);
    let client = test_client();

    // Obtain the real admin token — MUST use harness.admin_token_for(), not a fixed string.
    // An arbitrary "Bearer test-token" yields 401 (token mismatch), not 200 (C-3 / AC-001).
    let real_token = harness
        .admin_token_for("test-tenant", DtuType::Armis)
        .expect("Armis admin token must be in harness")
        .to_owned();

    // Sub-test 1: 200 with the real admin token.
    let resp_with_token = client
        .get(format!("http://{addr}/api/v1/search"))
        .bearer_auth(&real_token)
        .send()
        .await
        .expect("HTTP GET /api/v1/search must not fail at transport level");

    assert_eq!(
        resp_with_token.status().as_u16(),
        200,
        "GET /api/v1/search with real admin token must return HTTP 200 (AC-001, INV-HARNESS-ROUTE-PARITY). \
         Red Gate: route not registered → currently 404."
    );

    // Sub-test 2: 403 with no Authorization header (Armis auth model — missing Bearer = 403).
    let resp_no_auth = client
        .get(format!("http://{addr}/api/v1/search"))
        .send()
        .await
        .expect("HTTP GET /api/v1/search (no auth) must not fail at transport level");

    assert_eq!(
        resp_no_auth.status().as_u16(),
        403,
        "GET /api/v1/search with no Authorization header must return HTTP 403 (AC-001, Armis auth model). \
         Red Gate: route not registered → currently 404."
    );
}

// ============================================================================
// AC-001 addendum — F-P2-LOW-001 coverage gap closure
//
// test_BC_2_16_013_armis_harness_search_401_on_wrong_token
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY:
//   Armis auth model: check_bearer_auth(&headers, &state.admin_token)
//   - Missing/no bearer  → 403
//   - Present but WRONG  → 401  ← this is the C-3 "#1 Red Gate trap" case
//   - Correct admin token → 200
//
// F-P2-LOW-001: the AC-001 test above covered only cases 1 and 3.
// A regression that downgraded Armis search to the Claroty "accept any
// non-empty bearer" model would have silently passed the existing test.
// This test closes that false-pass vulnerability.
//
// Load-bearing (TD-VSDD-059): FAILS if auth gate mutated to any-bearer model.
// ============================================================================

/// AC-001 / F-P2-LOW-001: Armis harness search → 401 on present-but-wrong token.
///
/// Armis check_bearer_auth returns 401 when the Authorization header is present
/// and non-empty but does NOT match the clone's admin_token. This is distinct
/// from the 403 missing-bearer case and from the Claroty "accept any bearer" model.
///
/// Token used: "definitely-not-the-admin-token" — guaranteed not to equal the
/// harness-generated admin token so the test is deterministic and non-trivially
/// passing.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — Armis auth model, C-3 trap)
#[tokio::test]
async fn test_BC_2_16_013_armis_harness_search_401_on_wrong_token() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "test-tenant", DtuType::Armis);
    let client = test_client();

    // Confirm the harness actually issues an admin token distinct from our wrong token,
    // so the test cannot accidentally pass due to token-value collision.
    let real_token = harness
        .admin_token_for("test-tenant", DtuType::Armis)
        .expect("Armis admin token must be in harness")
        .to_owned();
    assert_ne!(
        real_token, "definitely-not-the-admin-token",
        "test invariant: wrong-token literal must differ from real admin token"
    );

    // Present-but-wrong bearer → 401 (token mismatch, not missing).
    // This distinguishes the Armis model (exact-match required) from the Claroty
    // model (any non-empty bearer accepted). A regression to the Claroty model
    // would return 200 here and fail this assertion (load-bearing per TD-VSDD-059).
    let resp = client
        .get(format!("http://{addr}/api/v1/search"))
        .bearer_auth("definitely-not-the-admin-token")
        .send()
        .await
        .expect("HTTP GET /api/v1/search (wrong token) must not fail at transport level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "GET /api/v1/search with a present-but-wrong bearer token must return HTTP 401 \
         (AC-001 / F-P2-LOW-001, Armis auth model: token mismatch → 401, NOT 200 or 403). \
         If this returns 200, the auth gate has regressed to the Claroty any-bearer model. \
         If this returns 403, the harness is conflating missing-bearer with wrong-bearer."
    );
}

// ============================================================================
// AC-002 — Armis harness clone AQL routing has structural parity with standalone
//
// test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY:
//   Armis search response envelope: {"data": {"results": [...], "total": N}}
//   in:devices → $.data.results is a non-empty array
//   in:alerts  → $.data.results contains alert records (not device records)
//
// C-7: Structural parity only — NOT field-for-field byte equality with standalone.
// Standalone uses typed DeviceRecord/AlertRecord + time-window filtering;
// harness serves raw Vec<Value> from DEVICES_FIXTURE / ALERTS_FIXTURE without those.
//
// Red Gate failure mode:
//   GET /api/v1/search?aql=in:devices → 404 (route not yet registered)
// ============================================================================

/// AC-002: Armis harness clone AQL routing — structural parity with standalone.
///
/// in:devices → $.data.results non-empty, $.data.total numeric.
/// in:alerts  → $.data.results non-empty (alert records, not device records).
/// Envelope: {"data": {"results": [...], "total": N}}
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — C-7: structural parity)
///
/// Red Gate: GET /api/v1/search → 404 (route not registered).
#[tokio::test]
async fn test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "test-tenant", DtuType::Armis);
    let client = test_client();

    let real_token = harness
        .admin_token_for("test-tenant", DtuType::Armis)
        .expect("Armis admin token must be in harness")
        .to_owned();

    // Sub-test A: aql=in:devices → devices response.
    let resp_devices = client
        .get(format!("http://{addr}/api/v1/search?aql=in:devices"))
        .bearer_auth(&real_token)
        .send()
        .await
        .expect("HTTP GET /api/v1/search?aql=in:devices must not fail at transport level");

    assert_eq!(
        resp_devices.status().as_u16(),
        200,
        "GET /api/v1/search?aql=in:devices must return HTTP 200 (AC-002). \
         Red Gate: route not registered → currently 404."
    );

    let body_devices: serde_json::Value = resp_devices
        .json()
        .await
        .expect("response must be valid JSON (AC-002)");

    // Structural parity check (C-7): $.data.results must be a non-empty array.
    let results = body_devices
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.as_array())
        .expect(
            "GET /api/v1/search response must contain $.data.results array \
             (AC-002, INV-HARNESS-ROUTE-PARITY envelope: {\"data\": {\"results\": [...], \"total\": N}})"
        );

    assert!(
        !results.is_empty(),
        "GET /api/v1/search?aql=in:devices — $.data.results must be non-empty \
         (AC-002, C-7: structural parity with DEVICES_FIXTURE)"
    );

    // $.data.total must be numeric.
    let total = body_devices
        .get("data")
        .and_then(|d| d.get("total"))
        .and_then(|t| t.as_u64())
        .expect(
            "GET /api/v1/search response must contain $.data.total as numeric \
             (AC-002, INV-HARNESS-ROUTE-PARITY)",
        );

    assert!(
        total > 0,
        "$.data.total must be > 0 for in:devices AQL (AC-002); got {total}"
    );

    // Sub-test B: aql=in:alerts → alerts response (different entity than devices).
    // C-7: structural parity — results must be non-empty array; entity type differs.
    let resp_alerts = client
        .get(format!("http://{addr}/api/v1/search?aql=in:alerts"))
        .bearer_auth(&real_token)
        .send()
        .await
        .expect("HTTP GET /api/v1/search?aql=in:alerts must not fail at transport level");

    assert_eq!(
        resp_alerts.status().as_u16(),
        200,
        "GET /api/v1/search?aql=in:alerts must return HTTP 200 (AC-002). \
         Red Gate: route not registered → currently 404."
    );

    let body_alerts: serde_json::Value = resp_alerts
        .json()
        .await
        .expect("alerts response must be valid JSON (AC-002)");

    let alert_results = body_alerts
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.as_array())
        .expect(
            "GET /api/v1/search?aql=in:alerts response must contain $.data.results array \
             (AC-002, INV-HARNESS-ROUTE-PARITY)",
        );

    // Structural parity: alert results must be a non-empty array.
    // The harness discriminates in:alerts from in:devices at the handler level.
    assert!(
        !alert_results.is_empty(),
        "GET /api/v1/search?aql=in:alerts — $.data.results must be non-empty \
         (AC-002, C-7: structural parity with ALERTS_FIXTURE)"
    );

    // Discriminator correctness (structural): in:devices and in:alerts must return
    // different entity types. Device records carry `device_id`; alert records carry
    // `alert_id`. If the handler returns the same fixture for both AQL values, the
    // first-element comparison catches it.
    let first_device = results.first().cloned().unwrap_or(serde_json::Value::Null);
    let first_alert = alert_results
        .first()
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    assert_ne!(
        first_device, first_alert,
        "in:devices and in:alerts must return different entity types (AC-002, discriminator check). \
         If the same fixture is returned for both, the AQL discriminator is broken."
    );

    // Sub-test C: EC-004 both-present precedence — aql with BOTH in:devices AND in:alerts
    // present MUST return DEVICE records (devices take precedence over alerts).
    //
    // This is the load-bearing EC-004 assertion. The handler's discriminator is:
    //   return_alerts = aql.contains("in:alerts") && !aql.contains("in:devices")
    // A mutant that drops the `&& !aql.contains("in:devices")` guard would return
    // alert records for the both-present case (has `alert_id`, no top-level device ID
    // field as owner), and this assertion would FAIL — making the guard load-bearing.
    let resp_both = client
        .get(format!(
            "http://{addr}/api/v1/search?aql=in:devices%20in:alerts"
        ))
        .bearer_auth(&real_token)
        .send()
        .await
        .expect(
            "HTTP GET /api/v1/search?aql=in:devices%20in:alerts must not fail at transport level",
        );

    assert_eq!(
        resp_both.status().as_u16(),
        200,
        "GET /api/v1/search with both in:devices and in:alerts in AQL must return HTTP 200 \
         (AC-002, EC-004 both-present precedence)."
    );

    let body_both: serde_json::Value = resp_both
        .json()
        .await
        .expect("both-present response must be valid JSON (AC-002, EC-004)");

    let both_results = body_both
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.as_array())
        .expect(
            "GET /api/v1/search (both in:devices in:alerts) must contain $.data.results array \
             (AC-002, EC-004)",
        );

    assert!(
        !both_results.is_empty(),
        "GET /api/v1/search with both in:devices and in:alerts — $.data.results must be non-empty \
         (AC-002, EC-004 both-present precedence)"
    );

    // Sub-test C load-bearing discriminator: the first result must be a DEVICE record.
    //
    // `alert_id.is_none()` is the PRIMARY load-bearing discriminator: device records
    // have no `alert_id` field; alert records always carry `alert_id`. A mutant that
    // drops the `&& !aql.contains("in:devices")` guard returns alert fixture records
    // for the both-present case — those records have `alert_id` present, so
    // `alert_id.is_none()` fails. This directly catches the dropped-guard mutant.
    //
    // `device_id == "d-001"` is a SECONDARY positive discriminator: it checks the
    // entity-id value against the first device fixture entry, confirming DEVICE records
    // (not alert records) were returned. Alert records carry `device_id` too (as a
    // foreign-key reference, e.g. "d-002"), so `device_id.is_some()` alone is
    // non-discriminating — only the specific value check distinguishes the two.
    let first_both = both_results
        .first()
        .expect("already asserted non-empty; first() is safe here");

    assert!(
        first_both.get("alert_id").is_none(),
        "EC-004: GET /api/v1/search with both in:devices and in:alerts — $.data.results[0] must \
         NOT have an `alert_id` field. `alert_id.is_none()` is the load-bearing discriminator: \
         device records carry no `alert_id`; alert records always do. The dropped-guard mutant \
         (s.contains(\"in:alerts\") without the `&& !s.contains(\"in:devices\")` guard) returns \
         alert fixture records here, which carry `alert_id` — failing this assertion. \
         got: {first_both}"
    );

    assert_eq!(
        first_both.get("device_id").and_then(|v| v.as_str()),
        Some("d-001"),
        "EC-004: GET /api/v1/search with both in:devices and in:alerts — $.data.results[0] must \
         have device_id == \"d-001\" (first device fixture entry). Alert fixture records also carry \
         `device_id` as a foreign-key reference (e.g. \"d-002\"), so only the specific value \
         check distinguishes device records from alert records on this field. \
         got: {first_both}"
    );
}

// ============================================================================
// AC-003 — Claroty harness clone registers POST /api/v1/audit_log/get in both routers
//
// test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY:
//   claroty::router() MUST include POST /api/v1/audit_log/get
//   claroty::network_router() MUST also include POST /api/v1/audit_log/get
//   Claroty auth model: 401 on missing/invalid Bearer (NOT 403)
//
// C-4: BOTH router() and network_router() must be tested.
//
// Red Gate failure mode:
//   POST /api/v1/audit_log/get → 404 (route not yet registered in either router)
// ============================================================================

/// AC-003: Claroty harness clone POST /api/v1/audit_log/get — 200 with bearer, 401 without.
///
/// Covers logical mode (router()) and network mode (network_router()) — C-4.
/// Claroty auth model: 401 on missing/empty Bearer (NOT 403 — that's Armis only).
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — claroty::router() MUST include
/// POST /api/v1/audit_log/get; Claroty auth model: 401 on missing/invalid Bearer)
///
/// Red Gate: POST /api/v1/audit_log/get → 404 (route not registered).
#[tokio::test]
async fn test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without() {
    // Part A: Logical mode (router())
    let harness_logical = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("logical harness build must succeed");

    let addr_logical = get_addr(&harness_logical, "acme-corp", DtuType::Claroty);
    let client = test_client();

    // Sub-test A1: 200 with any non-empty Bearer (Claroty model — not token-validated).
    let resp_with_bearer = client
        .post(format!("http://{addr_logical}/api/v1/audit_log/get"))
        .header("Authorization", "Bearer any-non-empty-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect(
            "POST /api/v1/audit_log/get (logical, with bearer) must not fail at transport level",
        );

    assert_eq!(
        resp_with_bearer.status().as_u16(),
        200,
        "POST /api/v1/audit_log/get (logical router) with valid Bearer must return HTTP 200 \
         (AC-003, INV-HARNESS-ROUTE-PARITY). \
         Red Gate: route not registered → currently 404."
    );

    // Sub-test A2: 401 with no Authorization header (Claroty model — missing Bearer = 401).
    // NOTE: Claroty returns 401 (not 403). 403 is Armis's model (AC-003 spec note).
    let resp_no_auth = client
        .post(format!("http://{addr_logical}/api/v1/audit_log/get"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/audit_log/get (logical, no auth) must not fail at transport level");

    assert_eq!(
        resp_no_auth.status().as_u16(),
        401,
        "POST /api/v1/audit_log/get (logical router) with no Authorization must return HTTP 401 \
         (AC-003, Claroty auth model: 401 NOT 403). \
         Red Gate: route not registered → currently 404."
    );

    // Part B: Network mode (network_router()) — C-4 coverage.
    // The story spec requires the route in BOTH routers.
    let harness_network = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("network harness build must succeed");

    let addr_network = harness_network
        .endpoint_for("acme-corp", DtuType::Claroty)
        .expect("network mode endpoint must be present for acme-corp Claroty");

    // Sub-test B1: 200 with Bearer in network mode.
    let resp_network_with_bearer = client
        .post(format!("http://{addr_network}/api/v1/audit_log/get"))
        .header("Authorization", "Bearer any-non-empty-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect(
            "POST /api/v1/audit_log/get (network, with bearer) must not fail at transport level",
        );

    assert_eq!(
        resp_network_with_bearer.status().as_u16(),
        200,
        "POST /api/v1/audit_log/get (network_router) with valid Bearer must return HTTP 200 \
         (AC-003, C-4: BOTH routers must have the route). \
         Red Gate: route not registered in network_router() → currently 404."
    );

    // Sub-test B2: 401 with no Authorization in network mode.
    let resp_network_no_auth = client
        .post(format!("http://{addr_network}/api/v1/audit_log/get"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/audit_log/get (network, no auth) must not fail at transport level");

    assert_eq!(
        resp_network_no_auth.status().as_u16(),
        401,
        "POST /api/v1/audit_log/get (network_router) with no Authorization must return HTTP 401 \
         (AC-003, C-4: network_router() uses plain check_bearer_auth per sibling convention). \
         Red Gate: route not registered in network_router() → currently 404."
    );
}

// ============================================================================
// AC-004 — Claroty harness clone audit_log response matches standalone
//
// test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY:
//   Response envelope: {"audit_log": [...], "total": N}
//   audit_log is non-empty
//   Each entry has all 8 columns (real xDome API fields, Tier 0+1):
//     id, action, user_display_name, category, timestamp, details, username, note
//   `note` is nullable (null for entries without a note); all other columns are non-empty strings.
//   Fixture embedded via include_str! of prism-dtu-claroty fixtures/audit-log.json
//   NOT via prism_dtu_common::load_fixture (C-1: harness uses compile-time embed)
//
// Red Gate failure mode:
//   POST /api/v1/audit_log/get → 404 (route not yet registered)
// ============================================================================

/// AC-004: Claroty harness audit_log response — envelope matches standalone.
///
/// Envelope: {"audit_log": [...], "total": N}
/// audit_log non-empty; each entry has all 8 TOML-declared columns
/// (id/action/user_display_name/category/timestamp/details/username/note).
/// `note` is nullable; the 7 other columns are non-empty strings.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — Claroty audit_log response envelope)
///
/// Red Gate: POST /api/v1/audit_log/get → 404 (route not registered).
#[tokio::test]
async fn test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "acme-corp", DtuType::Claroty);
    let client = test_client();

    let resp = client
        .post(format!("http://{addr}/api/v1/audit_log/get"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/audit_log/get must not fail at transport level");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /api/v1/audit_log/get with valid Bearer must return HTTP 200 (AC-004). \
         Red Gate: route not registered → currently 404."
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("response must be valid JSON (AC-004)");

    // Envelope key check: must be "audit_log" (matches standalone $.audit_log response_path).
    let audit_log = body.get("audit_log").and_then(|v| v.as_array()).expect(
        "response must contain `audit_log` JSON array key matching \
             claroty.sensor.toml response_path=\"$.audit_log\" (AC-004, INV-HARNESS-ROUTE-PARITY)",
    );

    assert!(
        !audit_log.is_empty(),
        "audit_log array must be non-empty (AC-004, C-1: harness embeds prism-dtu-claroty \
         fixtures/audit-log.json via include_str!)"
    );

    // total field must match array length.
    let total = body
        .get("total")
        .and_then(|t| t.as_u64())
        .expect("response must contain `total` numeric field (AC-004)");

    assert_eq!(
        total as usize,
        audit_log.len(),
        "`total` must equal the length of the `audit_log` array (AC-004)"
    );

    // 8-column structural check: every entry must have all 8 TOML-declared columns
    // (real xDome API fields, Tier 0+1 update). `note` is nullable (null in some entries);
    // the remaining 7 columns are non-empty strings.
    // Columns: id, action, user_display_name, category, timestamp, details, username, note
    // (from claroty.sensor.toml audit_logs table, SAP-2 parity requirement).
    for (i, entry) in audit_log.iter().enumerate() {
        // Non-nullable columns: key must be present AND value must be non-empty string.
        let non_null_columns = [
            "id",
            "action",
            "user_display_name",
            "category",
            "timestamp",
            "details",
            "username",
        ];
        for col in &non_null_columns {
            let val = entry.get(col).and_then(|v| v.as_str()).unwrap_or("");
            assert!(
                !val.is_empty(),
                "audit_log[{i}] must have non-empty `{col}` column \
                 (AC-004, 8-column parity: id/action/user_display_name/category/\
                 timestamp/details/username/note per claroty.sensor.toml SAP-2 requirement)"
            );
        }
        // Nullable column: key must be present (value may be null or a non-empty string).
        assert!(
            entry.get("note").is_some(),
            "audit_log[{i}] must have `note` key present (AC-004, SAP-2: \
             key absent vs null is a BC-2.11.001 EC-11-079 wire-shape violation)"
        );
    }

    // Spot-check: first entry's timestamp must resemble an ISO 8601 datetime.
    // (ADR-028 §D8: datetime columns use ISO 8601 format)
    if let Some(first) = audit_log.first() {
        let ts = first
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            ts.contains('T'),
            "audit_log[0].timestamp must be ISO 8601 format (ADR-028 §D8); got {ts:?} (AC-004)"
        );
    }
}
