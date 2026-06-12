//! Red Gate test 8: BC-2.06.019 PC-4 / TV-019-011
//!
//! test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only
//!
//! Traces to: BC-2.06.019 postcondition 4 / TV-019-011
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//!
//! FAIL mode (Red Gate): route handler in routes/hosts.rs does not implement
//! StageMask projection (BC-2.06.019 PC-4). The handler currently serves all
//! generated_devices regardless of stage. The CompromisedEndpoint primary device
//! is generated with `containment_status = "contained"` always. Without stage-mask
//! filtering that hides/overrides this at pre-containment stages, the stage-2
//! assertion (containment_status must NOT be "contained") FAILS because the raw
//! generated record is returned.
//!
//! HTTP-level load-bearing test (B-P1-02): starts two real CrowdstrikeClone servers
//! with different scenario_start_secs to control the stage at request time.
//!
//! Stage clock control (spec'd mechanism ADR-036 §2.1):
//!   Handlers call current_stage_index(&timeline, Utc::now().timestamp()) per request.
//!   We control the stage by placing scenario_start_secs in the past:
//!   - Stage 2: scenario_start_secs = now - 200  (elapsed ≈ 200s, in [180, 360))
//!   - Stage 4: scenario_start_secs = now - 700  (elapsed ≈ 700s ≥ 600s)
//!   With stage_duration_secs default [60, 180, 360, 600].
//!
//! Primary Red Gate failure:
//! Stage 2 request: raw generated record for primary device has
//!   containment_status = "contained". Without StageMask projection that overrides
//!   this to "normal" (or hides the "contained" state) at pre-containment stages,
//!   the assertion containment_status != "contained" FAILS.
//!
//! Route flow for generated-device path:
//!   GET /devices/queries/devices/v1 → IDs
//!   GET /devices/entities/devices/v2?ids=<primary_id> → device detail

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// Org ID with well-known first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// RED GATE TEST 8 — test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only
///
/// BC-2.06.019 PC-4 / TV-019-011
///
/// HTTP-level load-bearing test (B-P1-02). Starts two CrowdstrikeClone servers
/// with different scenario_start_secs to land at stage 2 and stage 4 respectively.
///
/// Asserts:
/// - At stage 2 (scenario_start = now - 200s): primary device's containment_status
///   is NOT "contained" in the HTTP response from GET /devices/entities/devices/v2.
///   (AC-008 / TV-019-011: containment_status must be "normal" at stage 2.)
/// - At stage 4 (scenario_start = now - 700s): primary device's containment_status
///   IS "contained" in the HTTP response.
///   (AC-008 / TV-019-011: the Containment stage makes it visible.)
///
/// FAIL mode (without StageMask projection):
/// Generated record always has containment_status = "contained" (generator sets this
/// for primary device). Without stage-mask override at pre-containment stages, the
/// stage-2 assertion ("must NOT be 'contained'") FAILS.
#[tokio::test]
async fn test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_cs.clone();

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // Stage 2 server (scenario_start = now - 200s → elapsed ≈ 200s ≥ 180s, < 360s)
    // At request time: current_stage_index returns 2 (LateralMovement).
    // AC-008: containment_status must NOT be "contained" at stage 2.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage2: i64 = now - 200; // elapsed ≈ 200s → stage 2

    let timeline_stage2 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage2,
        &[],
    ));
    let time_anchor_stage2 = chrono::DateTime::from_timestamp(start_stage2, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage2 = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage2),
        time_anchor_stage2,
    );

    clone_stage2
        .start()
        .await
        .expect("stage-2 CrowdstrikeClone start must succeed");

    let base_url_stage2 = clone_stage2.base_url();
    let token_stage2 = clone_stage2.admin_token().to_owned();

    // GET /devices/entities/devices/v2?ids=<primary_id>
    // (Direct fixture lookup — no X-DTU-Session-Id needed.)
    let resp2 = client
        .get(format!(
            "{base_url_stage2}/devices/entities/devices/v2?ids={primary_id}"
        ))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .send()
        .await
        .expect("GET /devices/entities/devices/v2 (stage 2) must reach the server");

    assert_eq!(
        resp2.status().as_u16(),
        200,
        "Stage 2: GET /devices/entities/devices/v2 must return HTTP 200; got {}",
        resp2.status().as_u16()
    );

    let body2: serde_json::Value = resp2.json().await.expect("stage-2 response must be JSON");
    let resources2 = body2["resources"].as_array().cloned().unwrap_or_default();

    // Primary device must be in the response at stage 2 (primary_device=true in mask).
    let primary_rec_stage2 = resources2.iter().find(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == &primary_id)
            .unwrap_or(false)
    });

    assert!(
        primary_rec_stage2.is_some(),
        "Stage 2: primary device '{}' must be present in GET /devices/entities/devices/v2 \
         response (primary_device=true in stage-2 mask); got resources: {:?}. \
         BC-2.06.019 PC-4 / TV-019-011",
        primary_id,
        resources2
    );

    // AC-008 / TV-019-011: at stage 2, containment_status must NOT be "contained".
    // FAIL: raw generated record has containment_status = "contained" (set by generator).
    // Without StageMask projection that overrides to "normal" at pre-containment stages,
    // this assertion FAILS.
    let containment_stage2 = primary_rec_stage2
        .and_then(|rec| rec.get("containment_status"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_ne!(
        containment_stage2,
        "contained",
        "TV-019-011: at stage 2 (elapsed ≈ 200s, LateralMovement), primary device '{}' \
         containment_status must NOT be 'contained' (containment only visible at stage 4); \
         got '{}'. BC-2.06.019 PC-4 / AC-008 \
         [RED GATE: StageMask projection not implemented — raw 'contained' record served at stage 2]",
        primary_id,
        containment_stage2
    );

    clone_stage2
        .stop()
        .await
        .expect("stage-2 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 4 server (scenario_start = now - 700s → elapsed ≈ 700s ≥ 600s)
    // At request time: current_stage_index returns 4 (Containment).
    // AC-008: containment_status MUST be "contained" at stage 4.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage4: i64 = now - 700; // elapsed ≈ 700s → stage 4

    let timeline_stage4 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage4,
        &[],
    ));
    let time_anchor_stage4 = chrono::DateTime::from_timestamp(start_stage4, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage4 = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage4),
        time_anchor_stage4,
    );

    clone_stage4
        .start()
        .await
        .expect("stage-4 CrowdstrikeClone start must succeed");

    let base_url_stage4 = clone_stage4.base_url();
    let token_stage4 = clone_stage4.admin_token().to_owned();

    let resp4 = client
        .get(format!(
            "{base_url_stage4}/devices/entities/devices/v2?ids={primary_id}"
        ))
        .header("Authorization", format!("Bearer {token_stage4}"))
        .send()
        .await
        .expect("GET /devices/entities/devices/v2 (stage 4) must reach the server");

    assert_eq!(
        resp4.status().as_u16(),
        200,
        "Stage 4: GET /devices/entities/devices/v2 must return HTTP 200; got {}",
        resp4.status().as_u16()
    );

    let body4: serde_json::Value = resp4.json().await.expect("stage-4 response must be JSON");
    let resources4 = body4["resources"].as_array().cloned().unwrap_or_default();

    let primary_rec_stage4 = resources4.iter().find(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == &primary_id)
            .unwrap_or(false)
    });

    assert!(
        primary_rec_stage4.is_some(),
        "Stage 4: primary device '{}' must be present in GET /devices/entities/devices/v2 \
         response (Containment stage: all mask fields true); got resources: {:?}. \
         BC-2.06.019 PC-4 / TV-019-011 / AC-008",
        primary_id,
        resources4
    );

    // AC-008 / TV-019-011: at stage 4, containment_status MUST be "contained".
    let containment_stage4 = primary_rec_stage4
        .and_then(|rec| rec.get("containment_status"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert_eq!(
        containment_stage4, "contained",
        "TV-019-011: at stage 4 (elapsed ≈ 700s, Containment), primary device '{}' \
         containment_status must be 'contained'; got '{}'. \
         BC-2.06.019 PC-4 / AC-008 / TV-019-011",
        primary_id, containment_stage4
    );

    clone_stage4
        .stop()
        .await
        .expect("stage-4 server stop must succeed");
}
