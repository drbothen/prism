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
        &catalog,
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
        containment_stage2, "contained",
        "TV-019-011: at stage 2 (elapsed ≈ 200s, LateralMovement), primary device '{}' \
         containment_status must NOT be 'contained' (containment only visible at stage 4); \
         got '{}'. BC-2.06.019 PC-4 / AC-008. \
         StageMask projection (hosts.rs containment_status override) must apply at stage 2.",
        primary_id, containment_stage2
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
        &catalog,
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

/// RED GATE TEST 9 — test_BPRL_P4_02_detections_stage_guard_primary_device
///
/// BC-2.06.019 PC-4 / BPRL-P4-02 coverage-matrix extension (D-1109).
///
/// Detections route must mirror the hosts.rs stage-guard:
/// at stage 0 (Baseline, elapsed < 60s), detection records whose `device_id`
/// equals the PRIMARY scenario device MUST be withheld.
/// At stage 2 (elapsed ≈ 200s), the primary device is visible (mask.primary_device=true,
/// stage_idx > 0), so primary-device detections MUST appear.
///
/// FAIL mode (without detections stage guard):
/// The detections route currently returns ALL generated detections regardless of
/// stage. At stage 0, detection 0 has device_id == primary_device_id_cs (the
/// CompromisedEndpoint generator links detection[n] to device[n % dev_count],
/// so detection 0 → device 0 = primary). Without stage-guard filtering, detection 0
/// appears at stage 0 even though the primary device is withheld from hosts.rs —
/// narrative incoherence (a detection references a device that doesn't exist yet).
///
/// HTTP-level load-bearing test (BPRL-P4-02):
/// - Stage 0 (scenario_start = now - 10s): detection referencing primary device ABSENT.
/// - Stage 2 (scenario_start = now - 200s): detection referencing primary device PRESENT.
#[tokio::test]
async fn test_BPRL_P4_02_detections_stage_guard_primary_device() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_cs.clone();

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // Stage 0 server (scenario_start = now + 30s → elapsed ≈ D-30s, 90s budget)
    // At request time: current_stage_index returns 0 (Baseline).
    // BPRL-P4-02: primary device is NOT visible at stage 0 (hosts.rs stage_idx > 0 guard).
    // Detections referencing the primary device must ALSO be withheld at stage 0.
    // +30s compensates for CPU contention from plugin tests in full workspace runs.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed ≈ D-30s (stage 0 budget 90s)

    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));
    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage0 = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    );

    clone_stage0
        .start()
        .await
        .expect("stage-0 CrowdstrikeClone start must succeed");

    let base_url_stage0 = clone_stage0.base_url();
    let token_stage0 = clone_stage0.admin_token().to_owned();

    // GET /detects/queries/detects/v1 — fetch all detection IDs at stage 0.
    let resp0_ids = client
        .get(format!("{base_url_stage0}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token_stage0}"))
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 (stage 0) must reach the server");

    assert_eq!(
        resp0_ids.status().as_u16(),
        200,
        "Stage 0: GET /detects/queries/detects/v1 must return HTTP 200; got {}",
        resp0_ids.status().as_u16()
    );

    let body0_ids: serde_json::Value = resp0_ids
        .json()
        .await
        .expect("stage-0 detection IDs response must be JSON");
    let det_ids_stage0: Vec<String> = body0_ids["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    // POST /detects/entities/summaries/GET/v1 — fetch detection details at stage 0.
    // Only request if we got any IDs (gracefully handle empty list).
    if !det_ids_stage0.is_empty() {
        let resp0_detail = client
            .post(format!(
                "{base_url_stage0}/detects/entities/summaries/GET/v1"
            ))
            .header("Authorization", format!("Bearer {token_stage0}"))
            .json(&serde_json::json!({"ids": det_ids_stage0}))
            .send()
            .await
            .expect("POST /detects/entities/summaries/GET/v1 (stage 0) must reach the server");

        assert_eq!(
            resp0_detail.status().as_u16(),
            200,
            "Stage 0: POST /detects/entities/summaries/GET/v1 must return HTTP 200; got {}",
            resp0_detail.status().as_u16()
        );

        let body0_detail: serde_json::Value = resp0_detail
            .json()
            .await
            .expect("stage-0 detection details response must be JSON");
        let resources0 = body0_detail["resources"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        // BPRL-P4-02 / BC-2.06.019 PC-4: at stage 0, NO detection referencing the
        // primary device must appear (primary device not yet surfaced at Baseline).
        let primary_det_stage0 = resources0.iter().find(|rec| {
            rec.get("device_id")
                .and_then(|v| v.as_str())
                .map(|id| id == primary_id)
                .unwrap_or(false)
        });

        assert!(
            primary_det_stage0.is_none(),
            "BPRL-P4-02 / BC-2.06.019 PC-4: at stage 0 (Baseline, elapsed clamped to 0s), \
             no detection referencing primary device '{}' must appear; \
             got detection: {:?}. \
             StageMask projection (detections.rs stage_idx > 0 guard, mirroring hosts.rs) \
             must filter detections referencing the primary device at stage 0. \
             BC-2.06.019 PC-4 / BPRL-P4-02",
            primary_id,
            primary_det_stage0
        );
    }

    clone_stage0
        .stop()
        .await
        .expect("stage-0 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 2 server (scenario_start = now - 200s → elapsed ≈ 200s ≥ 180s, < 360s)
    // At request time: current_stage_index returns 2 (LateralMovement).
    // BPRL-P4-02: at stage 2 the primary device IS visible (mask.primary_device=true,
    // stage_idx=2 > 0). Detections referencing the primary device MUST appear.
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
        org,
        Arc::clone(&timeline_stage2),
        time_anchor_stage2,
        &catalog,
    );

    clone_stage2
        .start()
        .await
        .expect("stage-2 CrowdstrikeClone start must succeed");

    let base_url_stage2 = clone_stage2.base_url();
    let token_stage2 = clone_stage2.admin_token().to_owned();

    // GET /detects/queries/detects/v1 — fetch detection IDs at stage 2.
    let resp2_ids = client
        .get(format!("{base_url_stage2}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 (stage 2) must reach the server");

    assert_eq!(
        resp2_ids.status().as_u16(),
        200,
        "Stage 2: GET /detects/queries/detects/v1 must return HTTP 200; got {}",
        resp2_ids.status().as_u16()
    );

    let body2_ids: serde_json::Value = resp2_ids
        .json()
        .await
        .expect("stage-2 detection IDs response must be JSON");
    let det_ids_stage2: Vec<String> = body2_ids["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !det_ids_stage2.is_empty(),
        "Stage 2: detection ID list must be non-empty at LateralMovement stage; \
         primary device is visible (mask.primary_device=true, stage_idx=2 > 0). \
         BPRL-P4-02 / BC-2.06.019 PC-4"
    );

    // POST /detects/entities/summaries/GET/v1 — fetch detection details at stage 2.
    let resp2_detail = client
        .post(format!(
            "{base_url_stage2}/detects/entities/summaries/GET/v1"
        ))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .json(&serde_json::json!({"ids": det_ids_stage2}))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 (stage 2) must reach the server");

    assert_eq!(
        resp2_detail.status().as_u16(),
        200,
        "Stage 2: POST /detects/entities/summaries/GET/v1 must return HTTP 200; got {}",
        resp2_detail.status().as_u16()
    );

    let body2_detail: serde_json::Value = resp2_detail
        .json()
        .await
        .expect("stage-2 detection details response must be JSON");
    let resources2 = body2_detail["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // BPRL-P4-02 / BC-2.06.019 PC-4: at stage 2, at least one detection referencing
    // the primary device MUST appear (primary device visible since stage_idx=2 > 0).
    let primary_det_stage2 = resources2.iter().find(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id)
            .unwrap_or(false)
    });

    assert!(
        primary_det_stage2.is_some(),
        "BPRL-P4-02 / BC-2.06.019 PC-4: at stage 2 (LateralMovement, elapsed ≈ 200s), \
         at least one detection referencing primary device '{}' MUST appear \
         (primary device visible since mask.primary_device=true, stage_idx=2 > 0); \
         got resources: {:?}. BPRL-P4-02 / BC-2.06.019 PC-4",
        primary_id,
        resources2
    );

    clone_stage2
        .stop()
        .await
        .expect("stage-2 server stop must succeed");
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R2-001 load-bearing test
// ---------------------------------------------------------------------------

/// LOAD-BEARING TEST — F-PIVOT003-R2-001: CrowdStrike IOC stamping wired on production path.
///
/// `CrowdstrikeClone::new_with_scenario` MUST produce detection records where detection 0
/// (the detection linked to the primary contained device) carries `behaviors[0].ioc_value`
/// from `catalog.ioc_hashes[0]`.
///
/// This exercises the PRODUCTION CONSTRUCTOR path (`new_with_scenario`), NOT the generator
/// helper directly. It proves that the demo server, which calls `new_with_scenario`, will
/// serve IOC-stamped detections (AC-004 / BC-2.06.019 v1.13, F-PIVOT003-R2-001).
///
/// TD-VSDD-059: load-bearing — verifies production path, not the helper function.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_06_019_scenario_clone_detection_0_carries_ioc_value_from_catalog() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);

    // Require catalog.ioc_hashes non-empty for the test to be meaningful.
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "build_scenario_entity_catalog must produce a non-empty ioc_hashes slice; \
         got empty. BC-2.06.019 v1.13 F-PIVOT003-R2-001."
    );
    let expected_ioc_hash = catalog.ioc_hashes[0].clone();

    let now = chrono::Utc::now().timestamp();
    let start_secs: i64 = now - 10;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_secs,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(start_secs, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    // Call the PRODUCTION constructor (the same one harness.rs uses).
    let clone = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    );

    // Detection 0 is the detection linked to the primary device (device_ids[0 % dev_count]).
    // It MUST carry behaviors[0].ioc_value = catalog.ioc_hashes[0].
    let det_0 = clone.state.generated_detections.iter().find(|rec| {
        // Detection 0 has the lowest creation timestamp (or can be found by id pattern).
        // Safest: find the detection with device_id = primary device.
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == catalog.primary_device_id_cs.as_str())
            .unwrap_or(false)
    });

    assert!(
        det_0.is_some(),
        "AC-004 / F-PIVOT003-R2-001: no detection record linked to primary device '{}'; \
         generated_detections: {} total. The production constructor must generate detection 0 \
         linked to the primary device (device_ids[0 % dev_count]).",
        catalog.primary_device_id_cs,
        clone.state.generated_detections.len()
    );

    let det_0 = det_0.unwrap();
    let empty_vec: Vec<serde_json::Value> = vec![];
    let behaviors = det_0
        .get("behaviors")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);

    assert!(
        !behaviors.is_empty(),
        "AC-004 / F-PIVOT003-R2-001: detection linked to primary device must have a non-empty \
         'behaviors' array; got empty. Scenario path must stamp IOC via generate_with_scenario_iocs."
    );

    let ioc_value = behaviors[0].get("ioc_value").and_then(|v| v.as_str());

    assert_eq!(
        ioc_value,
        Some(expected_ioc_hash.as_str()),
        "AC-004 / F-PIVOT003-R2-001: detection 0 behaviors[0].ioc_value MUST equal \
         catalog.ioc_hashes[0] = '{}'; got {:?}. \
         The PRODUCTION constructor CrowdstrikeClone::new_with_scenario must call \
         generate_with_scenario_iocs, threading the catalog's IOC hashes.",
        expected_ioc_hash,
        ioc_value
    );

    // Also verify ioc_type is "hash_sha256" (BC-2.06.019 v1.13 algorithm-qualified token).
    let ioc_type = behaviors[0].get("ioc_type").and_then(|v| v.as_str());

    assert_eq!(
        ioc_type,
        Some("hash_sha256"),
        "AC-004 / BC-2.06.019 v1.13: behaviors[0].ioc_type MUST be 'hash_sha256' \
         (algorithm-qualified); got {:?}.",
        ioc_type
    );
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R7A-002 SERVED-ROUTE test — ioc_hashes StageMask enforcement
// ---------------------------------------------------------------------------

/// SERVED-ROUTE TEST — F-PIVOT003-R7A-002: IOC-bearing detections absent at stage 1
/// (mask.ioc_hashes=false), present at stage 2+ (mask.ioc_hashes=true).
///
/// BC-2.06.019 v1.13 PC-4: "ioc_hashes=false: detection records where
/// behaviors[].ioc_value matches catalog.ioc_hashes are withheld."
///
/// Detection 0 is the IOC-bearing detection (behaviors[0].ioc_value = ioc_hashes[0]).
/// It is linked to the primary device (device_ids[0 % dev_count]).
///
/// This test drives the ACTUAL HTTP route (GET /detects/queries/detects/v1 +
/// POST /detects/entities/summaries/GET/v1) at two stage-clock positions to
/// verify that routes/detections.rs enforces mask.ioc_hashes — NOT just the
/// data-layer generator test.
///
/// Stage clock control:
///   stage 1 (Recon):        scenario_start = now - 90s  → mask.ioc_hashes=false
///   stage 2 (LateralMovement): scenario_start = now - 200s → mask.ioc_hashes=true
///
/// Asserts:
///   stage 1: the IOC-bearing detection (detection 0) is ABSENT from the served
///     response (mask.ioc_hashes=false).
///   stage 2: the IOC-bearing detection (detection 0) IS present in the served
///     response (mask.ioc_hashes=true).
///
/// FAIL mode (without this fix): routes/detections.rs serves all generated detections
/// without ioc_hashes filtering → IOC-bearing detection IS present at stage 1 → assertion FAILS.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_06_019_crowdstrike_ioc_bearing_detection_stagemask_served_route() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);

    // Confirm catalog has IOC hashes so the test is non-vacuous.
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "F-PIVOT003-R7A-002 vacuous guard: catalog.ioc_hashes must be non-empty; \
         got empty — secondary RNG seeding failure."
    );
    let expected_ioc = catalog.ioc_hashes[0].clone();

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // Stage 1 server (scenario_start = now - 90s → elapsed ≈ 90s → stage 1 Recon)
    // At stage 1: mask.ioc_hashes=false → IOC-bearing detection MUST be absent.
    // Stage 1 mask: primary_device=true (stage_idx=1 > 0), ioc_hashes=false.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage1: i64 = now - 90; // elapsed ≈ 90s → stage 1

    let timeline_stage1 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));
    let time_anchor_stage1 = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage1 = CrowdstrikeClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage1),
        time_anchor_stage1,
        &catalog,
    );

    clone_stage1
        .start()
        .await
        .expect("stage-1 CrowdstrikeClone start must succeed");

    let base_url_stage1 = clone_stage1.base_url();
    let token_stage1 = clone_stage1.admin_token().to_owned();

    // Step 1: GET /detects/queries/detects/v1 — fetch all detection IDs at stage 1.
    let resp1_ids = client
        .get(format!("{base_url_stage1}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token_stage1}"))
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 (stage 1) must reach the server");

    assert_eq!(resp1_ids.status().as_u16(), 200);

    let body1_ids: serde_json::Value = resp1_ids
        .json()
        .await
        .expect("stage-1 detection IDs response must be JSON");
    let det_ids_stage1: Vec<String> = body1_ids["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    // Step 2: POST /detects/entities/summaries/GET/v1 — fetch detection details.
    // If det_ids_stage1 is empty, the IOC-bearing detection is already withheld — pass.
    if !det_ids_stage1.is_empty() {
        let resp1_detail = client
            .post(format!(
                "{base_url_stage1}/detects/entities/summaries/GET/v1"
            ))
            .header("Authorization", format!("Bearer {token_stage1}"))
            .json(&serde_json::json!({"ids": det_ids_stage1}))
            .send()
            .await
            .expect("POST summaries (stage 1) must reach the server");

        assert_eq!(resp1_detail.status().as_u16(), 200);

        let body1_detail: serde_json::Value = resp1_detail
            .json()
            .await
            .expect("stage-1 detection details must be JSON");
        let resources1 = body1_detail["resources"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        // Assert: NO detection has behaviors[].ioc_value in catalog.ioc_hashes.
        // BC-2.06.019 v1.13 PC-4 / F-PIVOT003-R7A-002.
        let ioc_bearing = resources1.iter().find(|rec| {
            rec.get("behaviors")
                .and_then(|v| v.as_array())
                .map(|behaviors| {
                    behaviors.iter().any(|b| {
                        b.get("ioc_value")
                            .and_then(|v| v.as_str())
                            .map(|val| val == expected_ioc.as_str())
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        assert!(
            ioc_bearing.is_none(),
            "F-PIVOT003-R7A-002 BC-2.06.019 v1.13 PC-4: at stage 1 (Recon, \
             mask.ioc_hashes=false), the IOC-bearing detection with \
             behaviors[].ioc_value='{}' MUST be absent from the served response; \
             found record: {:?}. \
             routes/detections.rs must withhold detections whose behaviors[].ioc_value \
             is in catalog.ioc_hashes when mask.ioc_hashes=false. \
             [SERVED-ROUTE enforcement — not just data-layer]",
            expected_ioc,
            ioc_bearing
        );
    }

    clone_stage1
        .stop()
        .await
        .expect("stage-1 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 2 server (scenario_start = now - 200s → elapsed ≈ 200s → stage 2 LateralMovement)
    // At stage 2: mask.ioc_hashes=true → IOC-bearing detection MUST be present.
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
        org,
        Arc::clone(&timeline_stage2),
        time_anchor_stage2,
        &catalog,
    );

    clone_stage2
        .start()
        .await
        .expect("stage-2 CrowdstrikeClone start must succeed");

    let base_url_stage2 = clone_stage2.base_url();
    let token_stage2 = clone_stage2.admin_token().to_owned();

    // Step 1: GET /detects/queries/detects/v1 — fetch detection IDs at stage 2.
    let resp2_ids = client
        .get(format!("{base_url_stage2}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 (stage 2) must reach the server");

    assert_eq!(resp2_ids.status().as_u16(), 200);

    let body2_ids: serde_json::Value = resp2_ids
        .json()
        .await
        .expect("stage-2 detection IDs response must be JSON");
    let det_ids_stage2: Vec<String> = body2_ids["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !det_ids_stage2.is_empty(),
        "F-PIVOT003-R7A-002 prereq: detection ID list must be non-empty at stage 2 \
         (primary device visible, mask.ioc_hashes=true). Got empty.",
    );

    // Step 2: POST summaries — fetch detection details at stage 2.
    let resp2_detail = client
        .post(format!(
            "{base_url_stage2}/detects/entities/summaries/GET/v1"
        ))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .json(&serde_json::json!({"ids": det_ids_stage2}))
        .send()
        .await
        .expect("POST summaries (stage 2) must reach the server");

    assert_eq!(resp2_detail.status().as_u16(), 200);

    let body2_detail: serde_json::Value = resp2_detail
        .json()
        .await
        .expect("stage-2 detection details must be JSON");
    let resources2 = body2_detail["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Assert: the IOC-bearing detection IS present (mask.ioc_hashes=true at stage 2).
    let ioc_bearing_stage2 = resources2.iter().find(|rec| {
        rec.get("behaviors")
            .and_then(|v| v.as_array())
            .map(|behaviors| {
                behaviors.iter().any(|b| {
                    b.get("ioc_value")
                        .and_then(|v| v.as_str())
                        .map(|val| val == expected_ioc.as_str())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    assert!(
        ioc_bearing_stage2.is_some(),
        "F-PIVOT003-R7A-002 BC-2.06.019 v1.13 PC-4: at stage 2 (LateralMovement, \
         mask.ioc_hashes=true), the IOC-bearing detection with \
         behaviors[].ioc_value='{}' MUST be present in the served response; \
         got {} detection records but none with this IOC value. \
         [SERVED-ROUTE enforcement]",
        expected_ioc,
        resources2.len()
    );

    clone_stage2
        .stop()
        .await
        .expect("stage-2 server stop must succeed");
}
