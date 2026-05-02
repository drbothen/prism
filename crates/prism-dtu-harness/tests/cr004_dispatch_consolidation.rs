//! CR-004 test suite — exhaustive `match dtu_type` in `start_clone`.
//!
//! Covers:
//!   BC-3.5.001 postcondition 1 (exhaustive dispatch for all DtuType variants)
//!   BC-3.5.002 postcondition 1 (same dispatch path covers network mode)
//!   AC-003 (sequential if-chains replaced with exhaustive match)
//!   AC-007 (network-mode harness dispatch is also correct)
//!   EC-004 (new DtuType variant triggers compile error — tested via build contract)
//!
//! ## Production gap
//!
//! `builder.rs` dispatches via a `match` with a `_ =>` wildcard arm that silently
//! routes all non-CrowdStrike / non-Cyberint types through `start_clone()`.
//! Inside `start_clone()` (and `start_clone_network()`), sequential `if dtu_type ==`
//! chains handle Armis and Claroty; everything else falls through to a generic stub.
//!
//! In `IsolationMode::Network`, Armis is NOT currently dispatched to the Armis-specific
//! router — it falls through to the generic `start_clone_network()` which uses an
//! unauthenticated generic router. The Armis spec (AC-5) requires HTTP 403 for
//! requests that lack a `Bearer` token. A test that asserts 403 for network-mode
//! Armis reveals the gap.
//!
//! Test naming: `test_BC_3_5_001_CR004_xxx()` per factory convention.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use prism_dtu_harness::{DtuType, HarnessBuilder, IsolationMode};

// ===========================================================================
// AC-003: every DtuType variant starts a clone successfully via start_clone
// ===========================================================================

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Claroty` dispatches to the Claroty-specific clone path.
/// `HarnessBuilder` start succeeds and the clone returns HTTP 200 for POST device list.
///
/// This test PASSES at the Red Gate (Claroty dispatch via `start_clone` → Claroty
/// router is already wired). Regression guard: must not break after exhaustive match
/// refactor.
#[tokio::test]
async fn test_BC_3_5_001_CR004_claroty_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: Claroty clone must start without error via the dispatch in start_clone");

    // Claroty legacy endpoint: GET /assets/v1/assets (returns JSON with "assets" key).
    // This endpoint is Claroty-specific — the generic stub does NOT have this key shape.
    let addr = harness
        .endpoint_for("acme-corp", DtuType::Claroty)
        .expect("Claroty endpoint must be registered");
    let url = format!("http://{addr}/assets/v1/assets");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Claroty clone must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::Claroty must dispatch to Claroty router and return HTTP 200 \
         for GET /assets/v1/assets"
    );

    let body: serde_json::Value = resp.json().await.expect("response must be valid JSON");
    assert!(
        body.get("assets").is_some(),
        "AC-003: Claroty clone must return an 'assets' key in the response body, \
         confirming dispatch to the Claroty-specific router (not the generic stub)"
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Armis` dispatches to the Armis-specific clone path.
/// Armis clone REQUIRES a Bearer auth header — missing auth returns HTTP 403 (AC-5).
///
/// This test PASSES at the Red Gate (Armis dispatch via `start_clone` → Armis
/// router is already wired in Logical mode). Regression guard.
#[tokio::test]
async fn test_BC_3_5_001_CR004_armis_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: Armis clone must start without error");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Armis)
        .expect("Armis endpoint must be registered");

    // Armis-specific: GET /api/v1/devices WITHOUT auth → 403 (not 200 or 401).
    // This verifies the Armis router (not the generic stub) was dispatched.
    let url = format!("http://{addr}/api/v1/devices");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Armis clone must succeed at transport level");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-003: DtuType::Armis must dispatch to Armis-specific router which returns \
         HTTP 403 for missing Bearer auth (AC-5). If 200 is returned, the generic \
         stub was dispatched instead — the exhaustive match is not wired correctly."
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::CrowdStrike` dispatches through the CrowdStrike-specific clone path.
/// CrowdStrike clone starts and the OAuth token endpoint is reachable.
///
/// This test PASSES at the Red Gate (CrowdStrike has an explicit arm in `builder.rs`).
/// Regression guard.
#[tokio::test]
async fn test_BC_3_5_001_CR004_crowdstrike_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: CrowdStrike clone must start without error via explicit match arm");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::CrowdStrike)
        .expect("CrowdStrike endpoint must be registered");

    // CrowdStrike health endpoint.
    let url = format!("http://{addr}/dtu/health");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to CrowdStrike /dtu/health must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::CrowdStrike must start via its explicit match arm and \
         respond HTTP 200 on /dtu/health"
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Cyberint` dispatches through the Cyberint-specific clone path.
///
/// This test PASSES at the Red Gate (Cyberint has an explicit arm in `builder.rs`).
/// Regression guard.
#[tokio::test]
async fn test_BC_3_5_001_CR004_cyberint_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Cyberint];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: Cyberint clone must start without error via explicit match arm");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Cyberint)
        .expect("Cyberint endpoint must be registered");

    let url = format!("http://{addr}/dtu/health");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Cyberint /dtu/health must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::Cyberint must start via its explicit match arm and \
         respond HTTP 200 on /dtu/health"
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::Slack` dispatches through the Slack-specific clone
/// path and exposes the `/dtu/received-payloads` endpoint (Slack-specific, not generic).
///
/// ## Production gap
///
/// `builder.rs` routes Slack through `_ => start_clone()`. Inside `start_clone`,
/// `build_router_for_type` dispatches Slack to the Slack router. So the current
/// dispatch IS correct for Slack in Logical mode. This test is a regression guard:
/// the exhaustive match refactor must not break this path.
#[tokio::test]
async fn test_BC_3_5_001_CR004_slack_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: Slack clone must start without error");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Slack)
        .expect("Slack endpoint must be registered");

    // Slack-specific endpoint that does NOT exist on the generic stub.
    let url = format!("http://{addr}/dtu/received-payloads");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Slack /dtu/received-payloads must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::Slack must dispatch to Slack-specific router which exposes \
         /dtu/received-payloads (HTTP 200). A 404 means the generic stub was dispatched \
         instead — the exhaustive match is not correctly handling Slack."
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::PagerDuty` dispatches through the PagerDuty
/// clone path and exposes PagerDuty-specific endpoints.
///
/// Regression guard: exhaustive match refactor must preserve PagerDuty dispatch.
#[tokio::test]
async fn test_BC_3_5_001_CR004_pagerduty_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: PagerDuty clone must start without error");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::PagerDuty)
        .expect("PagerDuty endpoint must be registered");

    let url = format!("http://{addr}/dtu/health");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to PagerDuty /dtu/health must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::PagerDuty must dispatch to its clone path and return \
         HTTP 200 on /dtu/health"
    );
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::Jira` dispatches through the Jira clone path.
///
/// Regression guard.
#[tokio::test]
async fn test_BC_3_5_001_CR004_jira_clone_starts_via_exhaustive_match() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-003: Jira clone must start without error");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Jira)
        .expect("Jira endpoint must be registered");

    let url = format!("http://{addr}/dtu/health");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Jira /dtu/health must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: DtuType::Jira must dispatch to its clone path and return HTTP 200 on /dtu/health"
    );
}

// ===========================================================================
// AC-007: network-mode harness dispatch is also correct (BC-3.5.002)
// ===========================================================================

/// BC-3.5.002 postcondition 1 / AC-007:
/// In `IsolationMode::Network`, `DtuType::Armis` MUST dispatch to the
/// Armis-specific router. The Armis spec requires HTTP 403 for requests
/// that have no `Authorization: Bearer` header (AC-5).
///
/// ## Production gap
///
/// In Network mode, `build_network()` dispatches Armis via `_ => start_clone_network()`
/// which uses the generic network router. The generic network router ALLOWS unauthenticated
/// access (returns HTTP 200 for GET requests with no auth header). The Armis-specific
/// router requires Bearer auth and returns 403 for missing auth.
///
/// This test FAILS at the Red Gate: the generic network router returns HTTP 200 for
/// an unauthenticated GET /api/v1/devices request, but the test asserts HTTP 403
/// (the correct Armis behavior).
#[tokio::test]
async fn test_BC_3_5_002_CR004_armis_network_mode_dispatch_is_correct() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("AC-007: Armis network-mode harness build must succeed");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Armis)
        .expect("Armis network endpoint must be registered");

    // GET /api/v1/devices WITHOUT Bearer auth.
    // Armis-specific router: MUST return HTTP 403 (missing Bearer token → Forbidden).
    // Generic network router: returns HTTP 200 (unauthenticated reads allowed).
    let url = format!("http://{addr}/api/v1/devices");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to Armis network clone must succeed at transport level");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-007 (production gap): DtuType::Armis in IsolationMode::Network must dispatch to \
         the Armis-specific router which returns HTTP 403 for missing Bearer auth (AC-5). \
         HTTP 200 indicates the generic network stub was dispatched — the exhaustive match \
         must also be applied to the network-mode dispatch path in build_network()."
    );
}

/// BC-3.5.002 postcondition 1 / AC-007:
/// In `IsolationMode::Network`, `DtuType::CrowdStrike` dispatches through the
/// CrowdStrike-specific network path without error.
///
/// This test PASSES at the Red Gate (CrowdStrike has an explicit arm in `build_network()`).
/// Regression guard.
#[tokio::test]
async fn test_BC_3_5_002_CR004_crowdstrike_network_mode_dispatch_is_correct() {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike];
            spec.seed = 42;
        })
        .build()
        .await
        .expect(
            "AC-007: CrowdStrike network-mode harness build must succeed \
                 via its explicit match arm in build_network()",
        );

    let addr = harness
        .endpoint_for("acme-corp", DtuType::CrowdStrike)
        .expect("CrowdStrike network endpoint must be registered");

    let url = format!("http://{addr}/dtu/health");
    let resp = reqwest::get(&url)
        .await
        .expect("HTTP GET to CrowdStrike network /dtu/health must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-007: DtuType::CrowdStrike in IsolationMode::Network must start via its \
         explicit match arm and respond HTTP 200 on /dtu/health"
    );
}
