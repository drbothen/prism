//! S-DEMO-CLAROTY-TRAILING-SLASH-001 — Red Gate: Trailing-Slash Route Fidelity
//!
//! These three tests exercise the gap documented in ADR-031 §D8-b and BC-2.16.013
//! §Postconditions §1: the Claroty xDome API uses trailing slashes on all POST-for-read
//! endpoints, but prism-dtu-claroty's router registers routes WITHOUT trailing slashes
//! and has no `NormalizePathLayer` middleware.
//!
//! **Red Gate assertion:** All three tests MUST FAIL (HTTP 404) before the implementer
//! adds `tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash()` at the
//! OUTER SERVICE level in `clone.rs`. After the fix they must pass (HTTP 200).
//!
//! **Both serve paths requirement (FIX-1 / FIX-6 from v1.3):**
//! These tests exercise the non-TLS serve path (plain `axum::serve`), which is the path
//! `ClarotyClone::start()` uses in test contexts. The implementer MUST apply
//! `NormalizePathLayer::trim_trailing_slash().layer(router)` + fully-qualified
//! `ServiceExt::<axum::extract::Request>::into_make_service(app)` at BOTH serve sites
//! in `clone.rs` (~line 168 TLS `into_make_service` and ~line 192 non-TLS `axum::serve`).
//! If only the non-TLS path is patched, these tests pass but the TLS path ships broken.
//! If only the TLS path is patched, these tests continue to fail, which is the correct
//! Red Gate signal.
//!
//! # Behavioral Contracts
//!
//! | BC | Version | Role |
//! |----|---------|------|
//! | BC-2.16.013 | v1.25 | §Postconditions §1 — claroty.sensor.toml trailing-slash path_template form + normalize_path middleware requirement for prism-dtu-claroty |
//!
//! # Acceptance Criteria Coverage
//!
//! | Test | AC |
//! |------|----|
//! | `test_claroty_trailing_slash_alerts_returns_200`     | AC-001 |
//! | `test_claroty_trailing_slash_devices_returns_200`    | AC-002 |
//! | `test_claroty_trailing_slash_audit_log_get_returns_200` | AC-003 |
//! | `test_BC_2_16_013_no_slash_alerts_still_returns_200`     | AC-005 (regression guard EC-001) |
//! | `test_BC_2_16_013_no_slash_devices_still_returns_200`    | AC-005 (regression guard EC-001) |
//! | `test_BC_2_16_013_tags_route_with_slash_still_works`     | AC-005 (intentional tags trailing-slash route) |
//! | `test_BC_2_16_013_dtu_health_trailing_slash_returns_200`  | EC-003 |
//! | `test_BC_2_16_013_trailing_slash_alerts_missing_auth_returns_401` | EC-002 |
#![allow(clippy::unwrap_used, clippy::expect_used)]
// Test function names match BC naming convention (BC-NNN identifiers use PascalCase segments).
#![allow(non_snake_case)]

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

// ---------------------------------------------------------------------------
// Shared helper — mirrors the pattern from ac_1_devices_list.rs and edge_cases.rs.
//
// Uses ClarotyClone::new() + clone.start() (non-TLS plain HTTP path).
// This is the path `axum::serve(listener, router)` at ~line 192 of clone.rs.
// The Red Gate tests fail here because no NormalizePathLayer wraps the outer
// service — the router 404s on trailing-slash paths before routing resolves.
// ---------------------------------------------------------------------------

/// Start a fresh ClarotyClone on an ephemeral port and return (clone, base_url).
async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

// ============================================================================
// Red Gate Tests — AC-001, AC-002, AC-003
// These MUST FAIL (HTTP 404) before the implementer adds NormalizePathLayer.
// After the fix they MUST pass (HTTP 200).
// ============================================================================

/// AC-001: POST /api/v1/alerts/ (trailing slash) must return HTTP 200 with alerts fixture.
///
/// Red Gate: currently returns HTTP 404 — the axum router registers the route as
/// `POST /api/v1/alerts` (no trailing slash) with no NormalizePathLayer in the
/// outer service. After the fix (NormalizePathLayer::trim_trailing_slash() wrapping
/// both serve sites), this returns 200.
///
/// Traces to: BC-2.16.013 §Postconditions §1.
#[tokio::test]
async fn test_claroty_trailing_slash_alerts_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request to /api/v1/alerts/ failed");

    let status = resp.status().as_u16();

    // Red Gate assertion: must NOT be 404 (route not found) or 301 (redirect).
    // Without NormalizePathLayer at the outer service, axum returns 404 here.
    assert_eq!(
        status, 200,
        "POST /api/v1/alerts/ must return HTTP 200 (trailing-slash normalised to /api/v1/alerts); \
         got {status} — NormalizePathLayer::trim_trailing_slash() is missing from the outer service in clone.rs"
    );

    // Also verify the response contains the alerts fixture data.
    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");
    assert!(
        body.get("alerts").is_some(),
        "response must contain `alerts` array; got: {body}"
    );
}

/// AC-002: POST /api/v1/devices/ (trailing slash) must return HTTP 200 with devices fixture.
///
/// Red Gate: currently returns HTTP 404 — the axum router registers the route as
/// `POST /api/v1/devices` (no trailing slash) with no NormalizePathLayer in the
/// outer service. After the fix, this returns 200.
///
/// Traces to: BC-2.16.013 §Postconditions §1.
#[tokio::test]
async fn test_claroty_trailing_slash_devices_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request to /api/v1/devices/ failed");

    let status = resp.status().as_u16();

    // Red Gate assertion: must NOT be 404 or 301.
    assert_eq!(
        status, 200,
        "POST /api/v1/devices/ must return HTTP 200 (trailing-slash normalised to /api/v1/devices); \
         got {status} — NormalizePathLayer::trim_trailing_slash() is missing from the outer service in clone.rs"
    );

    // Also verify the response contains the devices fixture data.
    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");
    assert!(
        body.get("devices").is_some(),
        "response must contain `devices` array; got: {body}"
    );
    let devices = body["devices"]
        .as_array()
        .expect("`devices` must be an array");
    assert_eq!(
        devices.len(),
        20,
        "fixture must contain exactly 20 devices; got {}",
        devices.len()
    );
}

/// AC-003: POST /api/v1/audit_log/get/ (trailing slash) must return HTTP 200.
///
/// Red Gate: currently returns HTTP 404 — the axum router registers the route as
/// `POST /api/v1/audit_log/get` (no trailing slash) with no NormalizePathLayer
/// in the outer service. After the fix, this returns 200.
///
/// S-DEMO-CLAROTY-AUDIT-DTU-001 merged develop@e1c632dc;
/// real audit_log handler available; trailing-slash normalization verified against production handler.
///
/// Traces to: BC-2.16.013 §Postconditions §1.
/// Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001 (develop@e1c632dc).
#[tokio::test]
async fn test_claroty_trailing_slash_audit_log_get_returns_200() {
    // S-DEMO-CLAROTY-AUDIT-DTU-001 merged develop@e1c632dc;
    // real audit_log handler available; trailing-slash normalization verified against production handler.
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/audit_log/get/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request to /api/v1/audit_log/get/ failed");

    let status = resp.status().as_u16();

    // Red Gate assertion: must NOT be 404 or 301.
    assert_eq!(
        status, 200,
        "POST /api/v1/audit_log/get/ must return HTTP 200 (trailing-slash normalised to /api/v1/audit_log/get); \
         got {status} — NormalizePathLayer::trim_trailing_slash() is missing from the outer service in clone.rs. \
         Note: S-DEMO-CLAROTY-AUDIT-DTU-001 merged develop@e1c632dc so the handler exists."
    );
}

// ============================================================================
// AC-005 Regression Guards — these MUST PASS before AND after the fix.
//
// Verifies that:
//   EC-001: existing no-trailing-slash routes still return 200 after middleware addition.
//   AC-005 (intentional tags trailing-slash route): the tags route still behaves correctly.
//
// These tests establish the backward-compat baseline. If the implementer adds
// NormalizePathLayer correctly (trim_trailing_slash = STRIP-ONLY), no-slash routes
// pass through unmodified and hit the registered route directly.
// ============================================================================

/// AC-005 / EC-001 regression guard: POST /api/v1/alerts (no trailing slash) must still return 200.
///
/// trim_trailing_slash() is STRIP-ONLY: it strips inbound /alerts/ → /alerts.
/// Requests WITHOUT a trailing slash pass through unmodified and hit the existing
/// no-slash route directly. This guard catches any regression where normalize_path
/// is mis-applied (e.g., with append_trailing_slash() which would ADD slashes and
/// break this route).
///
/// This test MUST PASS both before and after the normalize_path fix.
///
/// Traces to: BC-2.16.013 §Postconditions §1 — backward compatibility.
#[tokio::test]
async fn test_BC_2_16_013_no_slash_alerts_still_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request to /api/v1/alerts failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /api/v1/alerts (no trailing slash) must still return 200 after normalize_path addition; \
         trim_trailing_slash() is STRIP-ONLY and must not affect no-slash requests"
    );

    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");
    assert!(
        body.get("alerts").is_some(),
        "response must contain `alerts` array; got: {body}"
    );
}

/// AC-005 / EC-001 regression guard: POST /api/v1/devices (no trailing slash) must still return 200.
///
/// Same reasoning as above for the devices route.
///
/// This test MUST PASS both before and after the normalize_path fix.
///
/// Traces to: BC-2.16.013 §Postconditions §1 — backward compatibility.
#[tokio::test]
async fn test_BC_2_16_013_no_slash_devices_still_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request to /api/v1/devices failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /api/v1/devices (no trailing slash) must still return 200 after normalize_path addition"
    );

    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");
    assert!(
        body.get("devices").is_some(),
        "response must contain `devices` array; got: {body}"
    );
}

/// AC-005 (intentional tags trailing-slash route) regression guard: the intentional POST /api/v1/devices/:id/tags/ route
/// (WITH trailing slash — clone.rs ~line 128) must still work correctly.
///
/// This is a critical guard. The story spec (AC-005) explicitly identifies this route as
/// at-risk: `trim_trailing_slash()` will strip inbound `.../tags/` → `.../tags`, which
/// MISSES the registered `/api/v1/devices/:device_id/tags/` route (404). The implementer
/// MUST either:
///   (a) drop the trailing slash from the registered tags route (preferred per story spec), OR
///   (b) document why existing tag tests still pass after middleware addition.
///
/// This guard asserts the tags route continues to return HTTP 201 after the fix.
/// It verifies the implementer chose option (a): the registered route is updated to
/// `/api/v1/devices/:device_id/tags` (no trailing slash) so inbound `.../tags/` is
/// stripped to `.../tags` and matches.
///
/// This test MUST PASS both before (route registered with trailing slash, no normalize_path
/// — inbound `.../tags/` hits the registered `.../tags/` route directly) and after (route
/// registered without trailing slash, inbound `.../tags/` stripped to `.../tags` and matches).
///
/// Traces to: BC-2.16.013 §Postconditions §1 — normalize_path MUST NOT break existing routes.
#[tokio::test]
async fn test_BC_2_16_013_tags_route_with_slash_still_works() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // The tags route uses a trailing slash in the inbound request — this is the
    // existing test pattern from ac_3_tag_add_persists.rs and harness_tests.rs.
    let resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "trailing-slash-guard", "tag_value": "true"}))
        .send()
        .await
        .expect("request to /api/v1/devices/asset-001/tags/ failed");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "POST /api/v1/devices/asset-001/tags/ must return HTTP 201; \
         after normalize_path addition, the registered tags route must be updated to \
         `/api/v1/devices/:device_id/tags` (no trailing slash) so trim_trailing_slash() \
         strips the inbound request to match it"
    );

    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");
    assert_eq!(
        body["device_id"], "asset-001",
        "tags response must include device_id; got: {body}"
    );
    assert_eq!(
        body["tag_key"], "trailing-slash-guard",
        "tags response must include tag_key; got: {body}"
    );
    assert_eq!(
        body["status"], "added",
        "tags response status must be `added`; got: {body}"
    );
}

// ============================================================================
// EC-003 Coverage — GET /dtu/health with and without trailing slash
//
// The NormalizePathLayer is applied at the OUTER SERVICE level, which means it
// runs before axum's router resolves the path. This test proves that the control-
// plane `/dtu/health` route is reachable even when the caller appends a trailing
// slash — the layer strips `/dtu/health/` → `/dtu/health` before routing.
//
// `dtu_health()` returns 200 {"status":"ok"} unconditionally (no auth required).
// ============================================================================

/// EC-003 coverage: GET /dtu/health/ (trailing slash) must return HTTP 200.
///
/// Proves that NormalizePathLayer normalises control-plane routes, not just
/// application routes.  `dtu_health()` is registered at `/dtu/health` (no
/// trailing slash); inbound `/dtu/health/` is stripped to `/dtu/health` by
/// the outer-service layer before the router resolves, so the handler fires.
///
/// Also asserts GET /dtu/health (no slash) returns 200 — baseline sanity.
///
/// Traces to: S-DEMO-CLAROTY-TRAILING-SLASH-001 §Edge Cases EC-003; BC-2.16.013 §Postconditions §1.
#[tokio::test]
async fn test_BC_2_16_013_dtu_health_trailing_slash_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // EC-003a: trailing slash is normalised → 200.
    let resp_slash = client
        .get(format!("{base_url}/dtu/health/"))
        .send()
        .await
        .expect("GET /dtu/health/ failed");

    assert_eq!(
        resp_slash.status().as_u16(),
        200,
        "GET /dtu/health/ must return HTTP 200 after NormalizePathLayer strips the trailing slash; \
         without the outer-service layer this would 404 because the router registers only /dtu/health"
    );

    let body_slash: serde_json::Value = resp_slash
        .json()
        .await
        .expect("GET /dtu/health/ response must be valid JSON");
    assert_eq!(
        body_slash["status"], "ok",
        "GET /dtu/health/ response must be {{\"status\":\"ok\"}}; got: {body_slash}"
    );

    // EC-003b: no-slash baseline — must also return 200 (regression guard).
    let resp_no_slash = client
        .get(format!("{base_url}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health failed");

    assert_eq!(
        resp_no_slash.status().as_u16(),
        200,
        "GET /dtu/health (no trailing slash) must still return HTTP 200"
    );

    let body_no_slash: serde_json::Value = resp_no_slash
        .json()
        .await
        .expect("GET /dtu/health response must be valid JSON");
    assert_eq!(
        body_no_slash["status"], "ok",
        "GET /dtu/health response must be {{\"status\":\"ok\"}}; got: {body_no_slash}"
    );
}

// ============================================================================
// EC-002 Coverage — POST /api/v1/alerts/ without auth → 401 (not 404)
//
// If NormalizePathLayer were mis-placed (e.g. via Router::layer which no-ops
// in axum 0.7 because routing happens before inner layers), the request would
// reach the router as `/api/v1/alerts/`, find no matching route, and 404.
// When the layer is correctly applied at the OUTER SERVICE level, the path is
// stripped to `/api/v1/alerts` BEFORE routing, the route matches, and the
// `list_alerts` handler fires.  That handler calls `check_bearer_auth` first;
// with no Authorization header it returns 401.
//
// 401 (not 404) therefore proves that:
//   (a) the path normalisation ran before routing resolved, AND
//   (b) the handler's auth check executed — the route was genuinely reached.
// ============================================================================

/// EC-002 coverage: POST /api/v1/alerts/ without Authorization header must return 401, not 404.
///
/// Proves that NormalizePathLayer is placed at the outer-service level (not via
/// Router::layer).  The strip happens before route resolution, so `/api/v1/alerts/`
/// becomes `/api/v1/alerts`, the handler fires, and the bearer-auth check rejects
/// the unauthenticated request with 401.
///
/// Traces to: S-DEMO-CLAROTY-TRAILING-SLASH-001 §Edge Cases EC-002; BC-2.16.013 §Postconditions §1.
#[tokio::test]
async fn test_BC_2_16_013_trailing_slash_alerts_missing_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Deliberately omit the Authorization header.
    let resp = client
        .post(format!("{base_url}/api/v1/alerts/"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /api/v1/alerts/ (no auth) failed");

    let status = resp.status().as_u16();

    // 404 here means NormalizePathLayer is NOT at the outer-service level — the router
    // sees the trailing slash and finds no matching route.
    // 401 means the layer ran, the route matched, and the auth check fired correctly.
    assert_eq!(
        status, 401,
        "POST /api/v1/alerts/ without Authorization must return 401 (auth check ran); \
         got {status} — if 404, NormalizePathLayer is mis-placed (Router::layer no-ops in axum 0.7); \
         NormalizePathLayer must wrap the outer service, not the inner router"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("401 response must have a JSON body");
    assert!(
        body.get("error").is_some(),
        "401 response must contain an `error` field; got: {body}"
    );
}
