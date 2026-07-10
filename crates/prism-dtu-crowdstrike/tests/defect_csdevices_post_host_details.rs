//! RED gate tests for DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Sub-defect 1 (DTU route).
//!
//! # Defect summary
//!
//! `GET /devices/entities/devices/v2` is the only registered verb on that path.
//! The ratified fix (D-1650 architect ratification 2026-07-10) adds a
//! `POST /devices/entities/devices/v2` handler (`post_host_details`) mirroring the
//! detections `get_detection_summaries` POST pattern.  Until that handler is wired,
//! every POST to that path returns HTTP 405 Method Not Allowed.
//!
//! # What each test asserts (desired behavior after fix)
//!
//! - `test_BC_DEFECT_CSDEVICES_001_post_host_details_returns_resources_for_known_ids`
//!   → 200 with `resources` array; currently FAILS (405)
//! - `test_BC_DEFECT_CSDEVICES_001_post_host_details_rejects_empty_ids_with_400`
//!   → 400; currently FAILS (405)
//! - `test_BC_DEFECT_CSDEVICES_001_post_host_details_rejects_missing_auth_with_401`
//!   → 401; currently FAILS (405, method check fires before auth check)
//! - `test_BC_DEFECT_CSDEVICES_001_post_host_details_enforces_org_id_guard`
//!   → 401; currently FAILS (405)
//!
//! # BC anchors
//!
//! - D-1650 architect ratification §Contract Part 2 (post_host_details handler spec)
//! - BC-2.06.018 / ADR-028 §D1, §D5 (DTU route must precede TOML spec change)
//! - Mirrors `get_detection_summaries` POST contract in `detections.rs`
//!
//! # Red Gate (BC-5.38.001)
//!
//! ALL tests in this file must FAIL before `post_host_details` is wired into
//! `mod.rs`. Failure mode: assertion `status == expected` fails because actual
//! status is 405 Method Not Allowed.

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]
#![cfg(feature = "dtu")]

use std::sync::Arc;

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::{CrowdstrikeClone, CrowdstrikeState};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a reqwest Client with a short timeout suitable for in-process DTU tests.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client must build for test")
}

/// Known fixture host IDs from `fixtures/hosts-ids.json` / `hosts-detail.json`.
/// These IDs exist in the static fixture so `get_host_details` (and the future
/// `post_host_details`) will return real records when no session filtering is active.
const FIXTURE_IDS: &[&str] = &["h-001", "h-002", "h-003"];

/// A deterministic, non-nil OrgId for clone A (used in guard tests).
fn org_a() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000AA")
            .expect("static uuid must parse"),
    )
}

/// A different non-nil OrgId for clone B (mismatched caller).
fn org_b() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000BB")
            .expect("static uuid must parse"),
    )
}

/// Start a `CrowdstrikeClone` with a specific `instance_org_id` active (non-nil
/// sentinel), so `validate_org_id` enforcement is active on all handlers.
async fn start_clone_with_org(org_id: OrgId) -> (CrowdstrikeClone, String) {
    let admin_token = uuid::Uuid::new_v4().to_string();
    let mut clone = CrowdstrikeClone::new();
    clone.state = Arc::new(CrowdstrikeState::with_admin_token_and_org(
        admin_token,
        org_id,
    ));
    clone
        .start()
        .await
        .expect("CrowdstrikeClone::start must succeed");
    let base_url = clone.base_url();
    (clone, base_url)
}

// ---------------------------------------------------------------------------
// Test 1: Happy path — POST with known IDs returns 200 + non-empty resources
//
// Asserts the desired post-fix behavior. Contract Part 2:
//   `post_host_details` must return HTTP 200 with
//   `{"resources": [{...device records...}]}` for a body of known IDs.
//
// RED: currently returns 405 (no POST handler registered on this path).
//      Failure message: "assertion `left == right` failed: expected 200, got 405"
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: POST /devices/entities/devices/v2 with known IDs must
/// return HTTP 200 with a non-empty `resources` array.
///
/// The body `{"ids": ["h-001", "h-002", "h-003"]}` matches fixture device IDs.
/// Since no session header is supplied the handler uses direct fixture lookup
/// (fidelity-probe path in the GET handler; same path in the POST handler).
///
/// RED: 405 Method Not Allowed (no POST handler). PASSES after post_host_details
/// is wired into mod.rs on `/devices/entities/devices/v2`.
#[tokio::test]
async fn test_BC_DEFECT_CSDEVICES_001_post_host_details_returns_resources_for_known_ids() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("CrowdstrikeClone::start must succeed");

    let base_url = clone.base_url();
    let client = http_client();

    let ids: Vec<serde_json::Value> = FIXTURE_IDS
        .iter()
        .map(|id| serde_json::Value::String(id.to_string()))
        .collect();
    let body = serde_json::json!({ "ids": ids });

    let resp = client
        .post(format!("{base_url}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 200,
        "BC-DEFECT-CSDEVICES-001: POST /devices/entities/devices/v2 with known IDs \
         must return 200; got {status}. \
         RED: currently 405 — no POST handler registered before fix."
    );

    let body_json: serde_json::Value = resp.json().await.expect("response body must be valid JSON");

    let resources = body_json["resources"]
        .as_array()
        .expect("BC-DEFECT-CSDEVICES-001: response must contain a 'resources' array");
    assert!(
        !resources.is_empty(),
        "BC-DEFECT-CSDEVICES-001: POST with known IDs must return non-empty resources; \
         got empty array. DTU fixture has records for h-001..h-003."
    );

    // Verify at least one resource has a device_id field (shape check).
    let first = &resources[0];
    assert!(
        first.get("device_id").is_some(),
        "BC-DEFECT-CSDEVICES-001: resource records must have a 'device_id' field; \
         got: {first}"
    );

    clone.stop().await.expect("clone.stop must succeed");
}

// ---------------------------------------------------------------------------
// Test 2: Empty ids body returns 400
//
// Contract Part 2 (mirrors detections.rs get_detection_summaries):
//   POST with `{"ids": []}` must return HTTP 400
//   `{"errors": [{"code": 400, "message": "ids array must not be empty"}]}`
//
// RED: currently returns 405 (handler doesn't exist yet, no guard can fire).
//      Failure message: "expected 400, got 405"
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: POST with empty `ids` array must return HTTP 400.
///
/// Mirrors the detections POST guard: `if body.ids.is_empty() { return 400 }`.
///
/// RED: 405 Method Not Allowed. PASSES after empty-ids guard is implemented in
/// `post_host_details`.
#[tokio::test]
async fn test_BC_DEFECT_CSDEVICES_001_post_host_details_rejects_empty_ids_with_400() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("CrowdstrikeClone::start must succeed");

    let base_url = clone.base_url();
    let client = http_client();

    let body = serde_json::json!({ "ids": [] });

    let resp = client
        .post(format!("{base_url}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 400,
        "BC-DEFECT-CSDEVICES-001: POST with empty ids must return 400; got {status}. \
         RED: currently 405 — no POST handler to evaluate the guard."
    );

    let body_json: serde_json::Value = resp
        .json()
        .await
        .expect("error response must be valid JSON");
    let errors = body_json["errors"]
        .as_array()
        .expect("400 response must contain an 'errors' array");
    assert_eq!(
        errors[0]["code"].as_u64(),
        Some(400),
        "BC-DEFECT-CSDEVICES-001: error code in body must be 400; got: {body_json}"
    );

    clone.stop().await.expect("clone.stop must succeed");
}

// ---------------------------------------------------------------------------
// Test 3: Missing Authorization header returns 401
//
// Contract Part 2 (same auth check as get_host_details / check_auth):
//   POST without Bearer token must return HTTP 401.
//
// RED: 405 (Axum method-not-allowed fires before any handler code, including
//      auth checks). Failure message: "expected 401, got 405"
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: POST without Authorization header must return HTTP 401.
///
/// Auth check (`check_auth`) must fire before any payload processing, matching
/// the behavior of `get_host_details` and `get_detection_summaries`.
///
/// RED: 405 Method Not Allowed. PASSES after auth guard is implemented in
/// `post_host_details` (same `check_auth` call as the GET handler).
#[tokio::test]
async fn test_BC_DEFECT_CSDEVICES_001_post_host_details_rejects_missing_auth_with_401() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("CrowdstrikeClone::start must succeed");

    let base_url = clone.base_url();
    let client = http_client();

    let body = serde_json::json!({ "ids": ["h-001"] });

    // No Authorization header — auth check must reject with 401.
    let resp = client
        .post(format!("{base_url}/devices/entities/devices/v2"))
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 401,
        "BC-DEFECT-CSDEVICES-001: POST without Authorization must return 401; got {status}. \
         RED: currently 405 — Axum method-not-allowed fires before handler auth check."
    );

    clone.stop().await.expect("clone.stop must succeed");
}

// ---------------------------------------------------------------------------
// Test 4: X-Org-Id guard enforced on POST (mismatched org → 401)
//
// Contract Part 2 (mirrors W3-FIX-SEC-001 / validate_org_id in get_host_details):
//   When instance_org_id is set on the clone, POST requests with a mismatched
//   (or absent) X-Org-Id header must return HTTP 401.
//
// RED: 405. PASSES after validate_org_id is called inside post_host_details.
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: POST with mismatched X-Org-Id must return HTTP 401
/// when the clone's instance_org_id is non-nil (W3-FIX-SEC-001 org guard).
///
/// Uses `start_clone_with_org(org_a())` so the guard is active, then sends a
/// request with `X-Org-Id: <org_b uuid>` (different org).
///
/// RED: 405 Method Not Allowed. PASSES after `validate_org_id` is wired into
/// `post_host_details` (matching the GET handler's existing guard).
#[tokio::test]
async fn test_BC_DEFECT_CSDEVICES_001_post_host_details_enforces_org_id_guard() {
    let (mut clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let body = serde_json::json!({ "ids": ["h-001"] });

    // X-Org-Id header contains org_b UUID — does not match org_a instance.
    let resp = client
        .post(format!("{base_url}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-Org-Id", org_b().to_string())
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 401,
        "BC-DEFECT-CSDEVICES-001: POST with mismatched X-Org-Id must return 401; got {status}. \
         RED: currently 405 — no POST handler to enforce validate_org_id."
    );

    clone.stop().await.expect("clone.stop must succeed");
}
