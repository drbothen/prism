//! RED gate tests for F-CSD-P9-001 (HIGH) — Harness CrowdStrike clone POST route parity.
//!
//! # Finding summary (F-CSD-P9-001)
//!
//! The standalone DTU (`prism-dtu-crowdstrike`) registers both GET and POST on
//! `/devices/entities/devices/v2` after the D-1650 fix (branch
//! `fix/csdevices-empty-pipeline`). The harness sibling clone
//! (`crates/prism-dtu-harness/src/clones/crowdstrike.rs`) registers GET-only on
//! that path in BOTH router builders:
//!
//! - `build_crowdstrike_router`         (~line 1158): GET only
//! - `build_crowdstrike_network_router` (~line 1385): GET only
//!
//! A harness-driven spec pipeline issuing the POST step would receive HTTP 405
//! Method Not Allowed → silent 0-row result. This violates BC-2.16.013
//! INV-HARNESS-ROUTE-PARITY.
//!
//! # BC anchors
//!
//! - BC-2.16.013 INV-HARNESS-ROUTE-PARITY — harness clone routers must mirror
//!   standalone DTU route surface
//! - D-1650 architect ratification §Contract Part 2 (post_host_details handler spec)
//! - ADR-028 §D1, §D5 (DTU route must precede TOML spec change)
//!
//! # Red Gate (BC-5.38.001) — Tests 1-4
//!
//! ALL four tests MUST FAIL before implementation. The POST route is not registered
//! in either harness router; every POST returns HTTP 405 Method Not Allowed.
//!
//! Expected failure message for each test:
//!   "assertion `left == right` failed: expected <N>, got 405"
//!
//! If any test passes before the harness router is patched, the test is suspect —
//! flag for spec-reviewer review.
//!
//! # What each test asserts (desired behavior after fix)
//!
//! - `test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_200_with_bearer`
//!   → 200 with `resources` array; currently FAILS (405)
//! - `test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_400_on_empty_ids`
//!   → 400; currently FAILS (405)
//! - `test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_401_without_auth`
//!   → 401; currently FAILS (405, method-check fires before auth check)
//! - `test_BC_2_16_013_F_CSD_P9_001_harness_network_post_host_details_reachable`
//!   → 200 with `resources` array in network-mode router; currently FAILS (405)
//!
//! # Network-mode router note
//!
//! `build_crowdstrike_router` (logical) and `build_crowdstrike_network_router`
//! (network) are separate functions with independent `.route(...)` calls at the
//! same path (~lines 1158 and 1385). Both must be patched. Test 4 locks the
//! network-mode builder independently so that a patch to one router that omits
//! the other is caught before merge.
//!
//! # Idiom
//!
//! Tests use the reqwest-over-TcpListener idiom established in
//! `tests/harness_clone_parity_test.rs`. Each test spins up a real Harness
//! (which starts a real axum server on an ephemeral 127.0.0.1 port) and sends
//! HTTP requests via reqwest.
//!
//! CrowdStrike harness auth model: `check_bearer_auth` accepts ANY non-empty
//! Bearer token (not token-validated, unlike Armis). The missing-bearer case
//! returns 401.

// Allow test-file conventions used across all harness tests.
#![allow(clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// Shared helpers
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
// Test 1: POST with valid auth and known IDs returns 200 + non-empty resources
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY — `build_crowdstrike_router` MUST
// include POST /devices/entities/devices/v2 with a handler that mirrors
// `get_host_details` but accepts JSON body `{"ids": [...]}`.
//
// CrowdStrike harness auth model: any non-empty Bearer token is accepted.
// No session-id supplied → direct lookup; all requested IDs are returned.
//
// Red Gate failure mode:
//   POST /devices/entities/devices/v2 → 405 Method Not Allowed
//   (no POST handler in build_crowdstrike_router)
// ============================================================================

/// BC-2.16.013 / F-CSD-P9-001: harness CrowdStrike clone POST
/// /devices/entities/devices/v2 with valid Bearer must return HTTP 200 with a
/// non-empty `resources` array whose first entry has a `device_id` field.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — build_crowdstrike_router MUST include
/// POST /devices/entities/devices/v2 mirroring the standalone DTU post_host_details)
///
/// Red Gate: POST /devices/entities/devices/v2 → 405 (route not registered).
#[tokio::test]
async fn test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_200_with_bearer() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "acme-corp", DtuType::CrowdStrike);
    let client = test_client();

    // CrowdStrike harness auth: any non-empty Bearer token is accepted.
    // No session-id → direct lookup; harness returns a record for every requested ID.
    let body = serde_json::json!({ "ids": ["test-device-001", "test-device-002"] });

    let resp = client
        .post(format!("http://{addr}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer test-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 200,
        "BC-2.16.013 / F-CSD-P9-001: harness POST /devices/entities/devices/v2 with \
         valid Bearer must return HTTP 200; got {status}. \
         RED: currently 405 — POST not registered in build_crowdstrike_router."
    );

    let body_json: serde_json::Value = resp
        .json()
        .await
        .expect("POST response body must be valid JSON");

    let resources = body_json["resources"].as_array().expect(
        "BC-2.16.013 / F-CSD-P9-001: POST /devices/entities/devices/v2 response must \
             contain a 'resources' array",
    );

    assert!(
        !resources.is_empty(),
        "BC-2.16.013 / F-CSD-P9-001: POST with known IDs must return non-empty \
         resources; got empty array. \
         Harness returns a record for every requested ID (no-session direct-lookup path)."
    );

    // Shape check: first resource must carry a device_id field.
    let first = &resources[0];
    assert!(
        first.get("device_id").is_some(),
        "BC-2.16.013 / F-CSD-P9-001: resource records must have a 'device_id' field; \
         got: {first}"
    );
}

// ============================================================================
// Test 2: POST with empty ids body returns 400
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY — post_host_details handler must guard
// against empty ids array (mirrors standalone defect_csdevices_post_host_details
// Test 2 and `get_detection_summaries` precedent in the harness clone itself).
//
// Red Gate failure mode:
//   POST /devices/entities/devices/v2 → 405 (no handler; guard cannot fire)
// ============================================================================

/// BC-2.16.013 / F-CSD-P9-001: harness CrowdStrike clone POST
/// /devices/entities/devices/v2 with empty `ids` array must return HTTP 400.
///
/// Mirrors the `get_detection_summaries` empty-ids guard already present in the
/// harness clone (~line 566) and the standalone post_host_details guard.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — empty-ids guard must be present in
/// the POST handler, consistent with detection-summaries POST precedent)
///
/// Red Gate: POST /devices/entities/devices/v2 → 405 (no handler to evaluate guard).
#[tokio::test]
async fn test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_400_on_empty_ids() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "acme-corp", DtuType::CrowdStrike);
    let client = test_client();

    let body = serde_json::json!({ "ids": [] });

    let resp = client
        .post(format!("http://{addr}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer test-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 400,
        "BC-2.16.013 / F-CSD-P9-001: harness POST /devices/entities/devices/v2 with \
         empty ids must return HTTP 400; got {status}. \
         RED: currently 405 — no POST handler to evaluate the empty-ids guard."
    );

    let body_json: serde_json::Value = resp.json().await.expect("400 response must be valid JSON");

    let errors = body_json["errors"].as_array().expect(
        "BC-2.16.013 / F-CSD-P9-001: 400 response must contain an 'errors' array \
             (mirrors get_detection_summaries empty-ids response shape in the harness clone)",
    );

    assert_eq!(
        errors[0]["code"].as_u64(),
        Some(400),
        "BC-2.16.013 / F-CSD-P9-001: errors[0].code must be 400; got: {body_json}"
    );
}

// ============================================================================
// Test 3: POST without Authorization header returns 401
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY — check_bearer_auth must fire in the
// POST handler before any payload processing (same contract as get_host_details
// and get_detection_summaries in the harness clone).
//
// CrowdStrike harness auth model: missing/empty bearer → 401 (check_bearer_auth
// returns Some(401) when the Authorization header is absent or the token part is
// empty after "Bearer " stripping).
//
// Red Gate failure mode:
//   POST /devices/entities/devices/v2 → 405 (Axum method-not-allowed fires
//   before any handler code, including auth checks)
// ============================================================================

/// BC-2.16.013 / F-CSD-P9-001: harness CrowdStrike clone POST
/// /devices/entities/devices/v2 without Authorization header must return HTTP 401.
///
/// `check_bearer_auth` must be the first guard in the POST handler (consistent with
/// all other auth-gated handlers in the harness clone: oauth_token, get_host_details,
/// get_detection_summaries, list_host_ids, list_detection_ids).
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — CrowdStrike harness auth model:
/// missing/empty bearer → 401)
///
/// Red Gate: POST /devices/entities/devices/v2 → 405 (Axum method-not-allowed
/// fires before handler auth check).
#[tokio::test]
async fn test_BC_2_16_013_F_CSD_P9_001_harness_post_host_details_401_without_auth() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "acme-corp", DtuType::CrowdStrike);
    let client = test_client();

    let body = serde_json::json!({ "ids": ["test-device-001"] });

    // No Authorization header — check_bearer_auth must reject with 401.
    let resp = client
        .post(format!("http://{addr}/devices/entities/devices/v2"))
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 (no auth) must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 401,
        "BC-2.16.013 / F-CSD-P9-001: harness POST /devices/entities/devices/v2 without \
         Authorization header must return HTTP 401; got {status}. \
         RED: currently 405 — Axum method-not-allowed fires before handler auth check."
    );
}

// ============================================================================
// Test 4: Network-mode router POST reachability lock
//
// BC-2.16.013 INV-HARNESS-ROUTE-PARITY — BOTH `build_crowdstrike_router` AND
// `build_crowdstrike_network_router` are separate functions with independent
// `.route(...)` registrations at /devices/entities/devices/v2 (~lines 1158 and
// 1385). A patch to one that omits the other is a valid implementation gap.
//
// This test locks the network-mode router independently. It is a POST
// reachability smoke-test: if the network-mode router serves 200, the route
// is registered and the contract is met.
//
// Note: `build_crowdstrike_network_router` wraps the list-IDs routes with
// bearer guards but passes through the entities/details routes unguarded
// to the underlying handler. The POST handler will use the same
// `check_bearer_auth` mechanism as all other handlers; any non-empty Bearer
// is accepted (CrowdStrike harness auth model).
//
// Red Gate failure mode:
//   POST /devices/entities/devices/v2 (network mode) → 405
//   (no POST handler in build_crowdstrike_network_router)
// ============================================================================

/// BC-2.16.013 / F-CSD-P9-001: harness CrowdStrike clone in NETWORK mode POST
/// /devices/entities/devices/v2 with valid Bearer must return HTTP 200.
///
/// Locks `build_crowdstrike_network_router` independently from
/// `build_crowdstrike_router`: both router builders register `/devices/entities/devices/v2`
/// separately and both must receive the POST route addition.
///
/// (BC-2.16.013 INV-HARNESS-ROUTE-PARITY — C-4 style: BOTH routers must have the route)
///
/// Red Gate: POST /devices/entities/devices/v2 (network router) → 405
/// (no POST handler registered in build_crowdstrike_network_router).
#[tokio::test]
async fn test_BC_2_16_013_F_CSD_P9_001_harness_network_post_host_details_reachable() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
        })
        .build()
        .await
        .expect("network harness build must succeed");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::CrowdStrike)
        .expect("network mode endpoint must be present for acme-corp CrowdStrike");

    let client = test_client();

    // CrowdStrike harness auth: any non-empty Bearer token is accepted.
    let body = serde_json::json!({ "ids": ["net-device-001"] });

    let resp = client
        .post(format!("http://{addr}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer test-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 (network mode) must reach server");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 200,
        "BC-2.16.013 / F-CSD-P9-001: harness CrowdStrike (network_router) POST \
         /devices/entities/devices/v2 with valid Bearer must return HTTP 200; got {status}. \
         RED: currently 405 — POST not registered in build_crowdstrike_network_router. \
         Both routers must be patched independently."
    );

    let body_json: serde_json::Value = resp
        .json()
        .await
        .expect("POST network-mode response must be valid JSON");

    let resources = body_json["resources"].as_array().expect(
        "BC-2.16.013 / F-CSD-P9-001: network-mode POST response must contain \
             a 'resources' array",
    );

    assert!(
        !resources.is_empty(),
        "BC-2.16.013 / F-CSD-P9-001: network-mode POST with a device ID must return \
         a non-empty resources array; got empty. \
         Harness returns a record for every requested ID (direct-lookup path)."
    );
}

// ============================================================================
// Test 5: OBS-1 — harness host_detail() missing first_seen field (RED lock)
//
// OBS-1 (LOCAL adversary pass-10): The harness `host_detail()` helper
// (~line 264, crates/prism-dtu-harness/src/clones/crowdstrike.rs) generates
// device records without a `first_seen` field:
//
//   fn host_detail(device_id: &str, containment_status: &str) -> Value {
//       json!({
//           "device_id": device_id,
//           "hostname": format!("{device_id}.example.com"),
//           "platform_name": "Linux",
//           "os_version": "Ubuntu 22.04",
//           "status": "normal",
//           "containment_status": containment_status,
//           "last_seen": "2026-01-02T09:00:00Z",      ← present
//           "external_ip": "203.0.113.1",
//           "local_ip": "10.0.0.1",
//           "agent_version": "7.04.17706.0"
//       })                                              ← first_seen ABSENT
//   }
//
// The crowdstrike.sensor.toml declares `first_seen` as a `datetime` column:
//   [[tables.columns]]
//   name = "first_seen"
//   column_type = "datetime"
//   ocsf_field = "device.first_seen_time"
//
// The standalone DTU (prism-dtu-crowdstrike) emits `first_seen` in its device
// records (SAP-2 parity). The harness sibling omits it, breaking DTU↔harness
// parity and causing the spec-engine to emit NULL for `first_seen` when the
// harness is used in demo/test scenarios — including the fix/csdevices-empty-pipeline
// branch where datetime-type correctness (Tests 7, T4 spec-column datetime) was
// specifically validated.
//
// BC anchor: BC-2.16.013 INV-HARNESS-ROUTE-PARITY (extended to schema field parity).
//
// Red Gate: POST /devices/entities/devices/v2 with valid IDs returns a resource
//           record WITHOUT `first_seen`. Assertion below fails because
//           `resources[0].get("first_seen")` returns None.
//
// Fix: Add `"first_seen": "2026-01-01T00:00:00Z"` to host_detail() in
//      crates/prism-dtu-harness/src/clones/crowdstrike.rs (~line 264).
// ============================================================================

/// OBS-1 / BC-2.16.013: harness CrowdStrike clone POST
/// /devices/entities/devices/v2 device records must include a non-null
/// `first_seen` field matching the TOML spec declaration.
///
/// # TOML spec source of truth
///
/// `crowdstrike.sensor.toml` (crowdstrike_devices table):
/// ```toml
/// [[tables.columns]]
/// name = "first_seen"
/// column_type = "datetime"
/// ocsf_field = "device.first_seen_time"
/// ```
///
/// # Defect
///
/// `host_detail()` in `crates/prism-dtu-harness/src/clones/crowdstrike.rs`
/// (~line 264) generates device fixture records that include `last_seen` but omit
/// `first_seen`. The standalone DTU (`prism-dtu-crowdstrike`) emits `first_seen`;
/// the harness sibling does not — breaking field parity (SAP-2).
///
/// When the harness is used in demo or pipeline scenarios, the spec-engine
/// normalizes the device rows with NULL for `first_seen`, silently corrupting
/// datetime-typed query results.
///
/// # Red Gate (BC-5.38.001)
///
/// At HEAD: `resources[0].get("first_seen")` is None → assertion fails.
/// Post-fix: `host_detail()` includes `"first_seen": "2026-01-01T00:00:00Z"` →
///   assertion passes.
#[tokio::test]
async fn test_BC_2_16_013_OBS_1_harness_post_host_details_first_seen_field_present() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = get_addr(&harness, "acme-corp", DtuType::CrowdStrike);
    let client = test_client();

    // CrowdStrike harness auth: any non-empty Bearer token is accepted.
    let body = serde_json::json!({ "ids": ["test-device-001"] });

    let resp = client
        .post(format!("http://{addr}/devices/entities/devices/v2"))
        .header("Authorization", "Bearer test-token")
        .json(&body)
        .send()
        .await
        .expect("POST /devices/entities/devices/v2 must reach server");

    // Pre-condition: route must be reachable (locked by Tests 1-4).
    // If this fails with 405, the harness POST handler hasn't been implemented yet.
    let status = resp.status().as_u16();
    assert_eq!(
        status, 200,
        "OBS-1 pre-condition: POST /devices/entities/devices/v2 must return 200 \
         (route reachability locked by Tests 1-4). got {status}"
    );

    let body_json: serde_json::Value = resp
        .json()
        .await
        .expect("POST response body must be valid JSON");

    let resources = body_json["resources"]
        .as_array()
        .expect("OBS-1: response must have a 'resources' array");

    assert!(
        !resources.is_empty(),
        "OBS-1: resources array must be non-empty for a known device ID"
    );

    let first = &resources[0];

    // PRIMARY RED GATE ASSERTION: first_seen must be present in the device record.
    // At HEAD: host_detail() omits first_seen → assertion fails.
    // Post-fix: host_detail() includes first_seen → assertion passes.
    assert!(
        first.get("first_seen").is_some(),
        "OBS-1 / BC-2.16.013: harness CrowdStrike device records must include a \
         `first_seen` field. \
         TOML spec: crowdstrike.sensor.toml declares `first_seen` as column_type = \"datetime\" \
         (ocsf_field = \"device.first_seen_time\"). The standalone DTU emits first_seen; \
         the harness host_detail() (~line 264, clones/crowdstrike.rs) omits it — \
         breaking field parity (SAP-2). \
         RED: `resources[0].get(\"first_seen\")` is None — field absent from host_detail() fixture. \
         Fix: add `\"first_seen\": \"2026-01-01T00:00:00Z\"` to host_detail() in \
         crates/prism-dtu-harness/src/clones/crowdstrike.rs. \
         got record: {first}"
    );

    // SECONDARY ASSERTION: first_seen must not be JSON null (the field must carry a value).
    // Guards against a fix that adds the key but sets it to null.
    assert!(
        !first["first_seen"].is_null(),
        "OBS-1 / BC-2.16.013: `first_seen` field must not be JSON null — \
         it must carry a valid ISO 8601 datetime string (e.g. '2026-01-01T00:00:00Z'). \
         Fix: set a non-null datetime value in host_detail() for first_seen. \
         got record: {first}"
    );
}
