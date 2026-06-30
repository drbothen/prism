//! BC-2.06.019 PC-4 / TV-019-009, TV-019-010
//!
//! test_BC_2_06_019_armis_primary_device_stage_visibility
//!
//! Traces to: BC-2.06.019 postcondition 4 / TV-019-009, TV-019-010
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//!
//! StageMask projection IS implemented in routes/devices.rs (stage_idx > 0 guard).
//! These tests verify the HTTP-level behavior of that projection at two stage-clock
//! positions.
//!
//! HTTP-level load-bearing assertion pattern (B-P1-02): this test starts a real
//! ArmisClone server and makes actual GET /api/v1/devices requests with stage-clock
//! control via scenario_start_secs placement relative to Utc::now().
//!
//! Stage clock control (spec'd mechanism ADR-036 §2.1):
//!   Handlers call current_stage_index(&timeline, Utc::now().timestamp()) per request.
//!   We control the stage by placing scenario_start_secs relative to Utc::now():
//!   - Stage 0: scenario_start_secs = now + 30  (elapsed clamped to 0s < 60s threshold;
//!              +30 offset gives 90-second execution budget — see timing comment below)
//!   - Stage 1: scenario_start_secs = now - 90  (elapsed ≈ 90s, in [60, 180))
//!   With stage_duration_secs default [60, 180, 360, 600].
//!
//! Timing budget rationale (stage 0 offset now+30 vs former now-10):
//!   Stage 1 activates when elapsed ≥ 60s. With start=now+30, elapsed at request time
//!   equals test_exec_time−30. For elapsed < 60s we need test_exec_time < 90s. Under
//!   full workspace load the server startup + HTTP round-trip took ≈ 55 s, which exceeds
//!   the former 50-second budget (now−10 → elapsed = test_exec_time + 10; for elapsed <
//!   60s → test_exec_time < 50s). Changing to now+30 raises the budget to 90s.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};

/// Org ID with well-known first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// RED GATE TEST 7 — test_BC_2_06_019_armis_primary_device_stage_visibility
///
/// BC-2.06.019 PC-4 / TV-019-009, TV-019-010
///
/// HTTP-level load-bearing test (B-P1-02). Starts a real ArmisClone server and
/// makes GET /api/v1/devices requests at two controlled stage-clock positions.
///
/// Stage clock control: scenario_start_secs controls the elapsed time seen by the
/// handler. Stage 0 uses `now+30` (elapsed clamped to 0s, 90-second budget before
/// stage 1). Stage 1 uses `now-90` (elapsed ≈ 90s ∈ [60, 180)).
///
/// Asserts:
/// - At stage 0 (scenario_start = now + 30s): primary device ABSENT from response.
///   The primary device only appears from stage 1 onward (AC-007 / task spec).
/// - At stage 1 (scenario_start = now - 90s): primary device PRESENT in response;
///   lateral device IDs ABSENT (StageMask lateral_devices=false at stage 1).
///
/// StageMask projection is implemented in routes/devices.rs (`stage_idx > 0` guard
/// for primary device; `mask.lateral_devices` for lateral devices).
#[tokio::test]
async fn test_BC_2_06_019_armis_primary_device_stage_visibility() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // --- Stage 0 server (scenario_start = now + 30s → elapsed ≈ –30s clamped to 0s < 60s) ---
    // At request time the handler computes elapsed = now - start. By placing start 30 seconds
    // IN THE FUTURE the elapsed value (clamped to 0 per EC-019-003) is always < 60 s as long
    // as the server start + HTTP round-trip completes within 90 s. Stage 1 activates at 60 s;
    // with start = now+30 the test has a full 90-second execution budget before the assertion
    // would be threatened, compared to the previous 50-second budget (now-10).
    // Rationale: under full workspace load the test took 55 s (55 > 60 - 10 = 50) causing
    // a spurious stage-1 activation. current_stage_index() clamps negative elapsed to 0.
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0 (Baseline), 90s budget

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();
    let lateral_ids: Vec<String> = catalog.lateral_device_ids_armis.clone();

    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));

    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage0 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-0 server");

    clone_stage0
        .start()
        .await
        .expect("stage-0 server start must succeed");

    let base_url_stage0 = clone_stage0.base_url();
    let admin_token_stage0 = clone_stage0.admin_token().to_owned();

    let client = prism_dtu_common::build_test_client();

    // Stage 0: primary device MUST be absent from /api/v1/devices response.
    // FAIL: without StageMask projection, all generated records are served
    //       → primary device IS in the response → this assertion FAILS.
    let resp0 = client
        .get(format!("{base_url_stage0}/api/v1/devices"))
        .header("Authorization", format!("Bearer {admin_token_stage0}"))
        .send()
        .await
        .expect("GET /api/v1/devices (stage 0) must reach the server");

    assert_eq!(
        resp0.status().as_u16(),
        200,
        "Stage 0: GET /api/v1/devices must return HTTP 200; got {}",
        resp0.status().as_u16()
    );

    let body0: serde_json::Value = resp0.json().await.expect("stage-0 response must be JSON");
    let devices0: Vec<serde_json::Value> = body0["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let device_ids0: Vec<String> = devices0
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 AC-007 / TV-019-009: primary device must NOT appear at stage 0.
    // StageMask projection is implemented in routes/devices.rs (stage_idx > 0 guard).
    assert!(
        !device_ids0.contains(&primary_id),
        "TV-019-009: at stage 0 (elapsed < 60s), primary device '{}' must be ABSENT \
         from GET /api/v1/devices response; found it in {:?}. \
         StageMask projection (routes/devices.rs stage_idx > 0 guard) must filter \
         the primary device at stage 0. BC-2.06.019 PC-4 / AC-007",
        primary_id,
        device_ids0
    );

    clone_stage0
        .stop()
        .await
        .expect("stage-0 server stop must succeed");

    // --- Stage 1 server (scenario_start = now - 90s → elapsed ≈ 90s ≥ 60s, < 180s) ---
    // At request time the handler computes: elapsed = now - start ≈ 90s → stage 1 (Recon).
    let now = chrono::Utc::now().timestamp();
    let start_stage1: i64 = now - 90; // elapsed ≈ 90s → stage 1 (Recon)

    let timeline_stage1 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));

    let time_anchor_stage1 = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage1 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage1),
        time_anchor_stage1,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-1 server");

    clone_stage1
        .start()
        .await
        .expect("stage-1 server start must succeed");

    let base_url_stage1 = clone_stage1.base_url();
    let admin_token_stage1 = clone_stage1.admin_token().to_owned();

    // Stage 1: primary device MUST be present in /api/v1/devices response.
    // (Stage 1 mask: primary_device=true.)
    let resp1 = client
        .get(format!("{base_url_stage1}/api/v1/devices"))
        .header("Authorization", format!("Bearer {admin_token_stage1}"))
        .send()
        .await
        .expect("GET /api/v1/devices (stage 1) must reach the server");

    assert_eq!(
        resp1.status().as_u16(),
        200,
        "Stage 1: GET /api/v1/devices must return HTTP 200; got {}",
        resp1.status().as_u16()
    );

    let body1: serde_json::Value = resp1.json().await.expect("stage-1 response must be JSON");
    let devices1: Vec<serde_json::Value> = body1["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let device_ids1: Vec<String> = devices1
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 AC-007 / TV-019-010: primary device MUST appear at stage 1.
    // (Stage 1 mask: primary_device=true.)
    assert!(
        device_ids1.contains(&primary_id),
        "TV-019-010: at stage 1 (elapsed ≈ 90s ≥ 60s), primary device '{}' must be PRESENT \
         in GET /api/v1/devices response; found IDs: {:?}. \
         BC-2.06.019 PC-4 / AC-007 / TV-019-010",
        primary_id,
        device_ids1
    );

    // BC-2.06.019 AC-007 / TV-019-010: lateral device IDs must NOT appear at stage 1.
    // Stage 1 mask: lateral_devices=false.
    // FAIL: without StageMask projection, all generated records are served
    //       → lateral devices ARE in the response → this assertion FAILS.
    for lat_id in &lateral_ids {
        assert!(
            !device_ids1.contains(lat_id),
            "TV-019-010: at stage 1 (elapsed ≈ 90s), lateral device '{}' must be ABSENT \
             from GET /api/v1/devices response (StageMask lateral_devices=false at stage 1); \
             found it in {:?}. BC-2.06.019 PC-4 / AC-007 / TV-019-010 \
             [RED GATE: StageMask projection not implemented — lateral devices leak at stage 1]",
            lat_id,
            device_ids1
        );
    }

    clone_stage1
        .stop()
        .await
        .expect("stage-1 server stop must succeed");
}

/// RED GATE TEST 8 — test_BPRL_P4_02_armis_alerts_stage_guard_primary_device
///
/// BC-2.06.019 PC-4 / BPRL-P4-02 coverage-matrix extension (D-1109).
///
/// Armis alerts route (GET /api/v1/alerts) and search route (GET /api/v1/search?aql=in:alerts)
/// must mirror the devices.rs stage-guard:
/// at stage 0 (Baseline, elapsed < 60s), alert records whose `device_id` equals the PRIMARY
/// scenario device MUST be withheld.
/// At stage 2 (elapsed ≈ 200s), the primary device is visible (mask.primary_device=true,
/// stage_idx > 0), so primary-device alerts MUST appear.
///
/// FAIL mode (without alerts stage guard):
/// The alerts route currently returns ALL generated alerts regardless of stage. At stage 0,
/// alert 0 has device_id == primary_device_id_armis (generator links alert[n] to
/// dev-{slug}-{seed}-{n}, so alert 0 → device 0 = primary). Without stage-guard filtering,
/// alert 0 appears at stage 0 even though the primary device is withheld from devices.rs —
/// narrative incoherence (an alert references a device that doesn't exist yet).
///
/// HTTP-level load-bearing test (BPRL-P4-02, SID-1):
/// - Stage 0 (scenario_start = now + 30s → elapsed clamped to 0s < 60s): alert referencing primary device ABSENT.
/// - Stage 2 (scenario_start = now - 200s): alert referencing primary device PRESENT.
#[tokio::test]
async fn test_BPRL_P4_02_armis_alerts_stage_guard_primary_device() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // Stage 0 server (scenario_start = now + 30s → elapsed clamped to 0s < 60s)
    // At request time: current_stage_index returns 0 (Baseline).
    // BPRL-P4-02: primary device is NOT visible at stage 0 (devices.rs stage_idx > 0 guard).
    // Alerts referencing the primary device must ALSO be withheld at stage 0.
    // start = now+30 gives 90-second execution budget (vs previous 50 s with now-10).
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0, 90s budget

    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));
    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let clone_stage0 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-0 server");

    let mut clone_stage0 = clone_stage0;
    clone_stage0
        .start()
        .await
        .expect("stage-0 ArmisClone start must succeed");

    let base_url_stage0 = clone_stage0.base_url();
    let token_stage0 = clone_stage0.admin_token().to_owned();

    // GET /api/v1/alerts — fetch all alerts at stage 0.
    let resp0 = client
        .get(format!("{base_url_stage0}/api/v1/alerts"))
        .header("Authorization", format!("Bearer {token_stage0}"))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 0) must reach the server");

    assert_eq!(
        resp0.status().as_u16(),
        200,
        "Stage 0: GET /api/v1/alerts must return HTTP 200; got {}",
        resp0.status().as_u16()
    );

    let body0: serde_json::Value = resp0
        .json()
        .await
        .expect("stage-0 alerts response must be JSON");
    let alerts0 = body0["data"]["alerts"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // BPRL-P4-02 / BC-2.06.019 PC-4: at stage 0, NO alert referencing the primary device
    // must appear (primary device not yet surfaced at Baseline).
    let primary_alert_stage0 = alerts0.iter().find(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id)
            .unwrap_or(false)
    });

    assert!(
        primary_alert_stage0.is_none(),
        "BPRL-P4-02 / BC-2.06.019 PC-4: at stage 0 (Baseline, elapsed ≈ 10s), \
         no alert referencing primary device '{}' must appear at GET /api/v1/alerts; \
         got alert: {:?}. \
         [RED GATE: alerts route missing stage-guard for primary device — \
         serves alert at stage 0 before primary device is visible from devices.rs; \
         alerts route added to PC-4 coverage matrix per D-1109]",
        primary_id,
        primary_alert_stage0
    );

    clone_stage0
        .stop()
        .await
        .expect("stage-0 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 2 server (scenario_start = now - 200s → elapsed ≈ 200s ≥ 180s, < 360s)
    // At request time: current_stage_index returns 2 (LateralMovement).
    // BPRL-P4-02: at stage 2 the primary device IS visible (mask.primary_device=true,
    // stage_idx=2 > 0). Alerts referencing the primary device MUST appear.
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

    let mut clone_stage2 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage2),
        time_anchor_stage2,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-2 server");

    clone_stage2
        .start()
        .await
        .expect("stage-2 ArmisClone start must succeed");

    let base_url_stage2 = clone_stage2.base_url();
    let token_stage2 = clone_stage2.admin_token().to_owned();

    // GET /api/v1/alerts — fetch alerts at stage 2.
    let resp2 = client
        .get(format!("{base_url_stage2}/api/v1/alerts"))
        .header("Authorization", format!("Bearer {token_stage2}"))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 2) must reach the server");

    assert_eq!(
        resp2.status().as_u16(),
        200,
        "Stage 2: GET /api/v1/alerts must return HTTP 200; got {}",
        resp2.status().as_u16()
    );

    let body2: serde_json::Value = resp2
        .json()
        .await
        .expect("stage-2 alerts response must be JSON");
    let alerts2 = body2["data"]["alerts"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    assert!(
        !alerts2.is_empty(),
        "Stage 2: GET /api/v1/alerts must return non-empty alerts at LateralMovement stage; \
         primary device is visible (mask.primary_device=true, stage_idx=2 > 0). \
         BPRL-P4-02 / BC-2.06.019 PC-4"
    );

    // BPRL-P4-02 / BC-2.06.019 PC-4: at stage 2, at least one alert referencing the
    // primary device MUST appear (primary device visible since stage_idx=2 > 0).
    let primary_alert_stage2 = alerts2.iter().find(|rec| {
        rec.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id)
            .unwrap_or(false)
    });

    assert!(
        primary_alert_stage2.is_some(),
        "BPRL-P4-02 / BC-2.06.019 PC-4: at stage 2 (LateralMovement, elapsed ≈ 200s), \
         at least one alert referencing primary device '{}' MUST appear at GET /api/v1/alerts \
         (primary device visible since mask.primary_device=true, stage_idx=2 > 0); \
         got alerts: {:?}. BPRL-P4-02 / BC-2.06.019 PC-4",
        primary_id,
        alerts2
    );

    clone_stage2
        .stop()
        .await
        .expect("stage-2 server stop must succeed");
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R2-002 load-bearing test
// ---------------------------------------------------------------------------

/// LOAD-BEARING TEST — F-PIVOT003-R2-002: Armis device_cves_first wired on production path.
///
/// `ArmisClone::new_with_scenario` MUST produce device records where CompromisedEndpoint
/// asset records (identified by presence of `asset_id`) carry `device_cves_first` =
/// `catalog.device_cves[0]`.
///
/// This exercises the PRODUCTION CONSTRUCTOR path (`new_with_scenario`), NOT the generator
/// helper directly. It proves that the demo server, which calls `new_with_scenario`, will
/// serve CVE-stamped device records enabling the NVD pivot (AC-008 / BC-2.06.019 v1.13,
/// F-PIVOT003-R2-002).
///
/// TD-VSDD-059: load-bearing — verifies production path, not the helper function.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_06_019_scenario_clone_device_records_carry_device_cves_first() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);

    // Require catalog.device_cves non-empty for the test to be meaningful.
    assert!(
        !catalog.device_cves.is_empty(),
        "build_scenario_entity_catalog must produce a non-empty device_cves slice; \
         got empty. BC-2.06.019 v1.13 F-PIVOT003-R2-002."
    );
    let expected_cve = catalog.device_cves[0].clone();

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
    let clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed for device_cves_first test");

    // Find asset records (have `asset_id` field — discriminates from alert records).
    let asset_records: Vec<&serde_json::Value> = clone
        .state
        .generated_records
        .iter()
        .filter(|rec| rec.get("asset_id").is_some())
        .collect();

    assert!(
        !asset_records.is_empty(),
        "AC-008 / F-PIVOT003-R2-002: generated_records must contain at least one asset record \
         (with 'asset_id' field) for CompromisedEndpoint archetype; got 0. \
         The production constructor must invoke generate_with_scenario_cves."
    );

    // All asset records must carry device_cves_first = catalog.device_cves[0].
    let missing: Vec<&&serde_json::Value> = asset_records
        .iter()
        .filter(|rec| {
            rec.get("device_cves_first")
                .and_then(|v| v.as_str())
                .map(|v| v != expected_cve.as_str())
                .unwrap_or(true) // key absent → missing
        })
        .collect();

    assert!(
        missing.is_empty(),
        "AC-008 / F-PIVOT003-R2-002: {} of {} asset records are missing \
         'device_cves_first' = '{}'; these records will produce empty NVD pivot results. \
         ArmisClone::new_with_scenario MUST call generate_with_scenario_cves to stamp CVEs \
         on the production path (not just the helper function).",
        missing.len(),
        asset_records.len(),
        expected_cve
    );
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R7A-001 SERVED-ROUTE test — device_cves StageMask enforcement
// ---------------------------------------------------------------------------

/// SERVED-ROUTE TEST — F-PIVOT003-R7A-001: device_cves_first absent at stages 0-3,
/// present at stage 4.
///
/// BC-2.06.019 v1.13 PC-4: "device_cves=false: CVE-related enrichment fields on device
/// records are omitted."  The `device_cves_first` field is the scalar CVE projection
/// (U17/Ruling 1b) stamped by `generate_with_scenario_cves`.
///
/// This test drives the ACTUAL HTTP route (`GET /api/v1/devices`) at two stage-clock
/// positions to verify that `routes/devices.rs` enforces `mask.device_cves` before
/// serving records — NOT just the data-layer generator test that checks
/// `state.generated_records`.
///
/// Stage clock control (ADR-036 §2.1):
///   stage 1 (Recon):      scenario_start = now - 90s   → mask.device_cves=false
///   stage 4 (Containment): scenario_start = now - 700s  → mask.device_cves=true
///
/// Asserts:
///   - stage 1: primary device record returned by GET /api/v1/devices does NOT have
///     the `device_cves_first` key.
///   - stage 4: primary device record returned by GET /api/v1/devices DOES have the
///     `device_cves_first` key (the NVD pivot field is now visible).
///
/// FAIL mode (without this fix): routes/devices.rs serves all generated records
/// unfiltered by mask.device_cves → device_cves_first IS present at stage 1 → assertion FAILS.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_06_019_armis_device_cves_first_stagemask_served_route() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();

    // Confirm catalog has CVEs so the test is non-vacuous.
    assert!(
        !catalog.device_cves.is_empty(),
        "F-PIVOT003-R7A-001 vacuous guard: catalog.device_cves must be non-empty; \
         got empty — secondary RNG seeding failure."
    );

    let client = prism_dtu_common::build_test_client();

    // -------------------------------------------------------------------------
    // Stage 1 server (scenario_start = now - 90s → elapsed ≈ 90s → stage 1 Recon)
    // At stage 1: mask.device_cves=false → device_cves_first MUST be absent.
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

    let mut clone_stage1 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage1),
        time_anchor_stage1,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-1 server");

    clone_stage1
        .start()
        .await
        .expect("stage-1 server start must succeed");

    let base_url_stage1 = clone_stage1.base_url();
    let token_stage1 = clone_stage1.admin_token().to_owned();

    let resp1 = client
        .get(format!("{base_url_stage1}/api/v1/devices"))
        .header("Authorization", format!("Bearer {token_stage1}"))
        .send()
        .await
        .expect("GET /api/v1/devices (stage 1) must reach the server");

    assert_eq!(resp1.status().as_u16(), 200);

    let body1: serde_json::Value = resp1.json().await.expect("stage-1 response must be JSON");
    let devices1: Vec<serde_json::Value> = body1["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // At stage 1, primary device IS present (mask.primary_device=true, stage_idx > 0).
    // Find the primary device record and assert device_cves_first is ABSENT.
    let primary_record_stage1 = devices1.iter().find(|r| {
        r.get("asset_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id.as_str())
            .unwrap_or(false)
    });

    assert!(
        primary_record_stage1.is_some(),
        "F-PIVOT003-R7A-001 prereq: primary device '{}' must be present at stage 1 \
         (mask.primary_device=true, stage_idx=1 > 0); not found in {:?}",
        primary_id,
        devices1
            .iter()
            .filter_map(|r| r.get("asset_id").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
    );

    let primary1 = primary_record_stage1.unwrap();
    assert!(
        primary1.get("device_cves_first").is_none(),
        "F-PIVOT003-R7A-001 BC-2.06.019 v1.13 PC-4: at stage 1 (Recon, mask.device_cves=false), \
         device_cves_first MUST be absent from GET /api/v1/devices response; \
         found it with value {:?}. \
         routes/devices.rs must strip device_cves_first when !mask.device_cves. \
         [SERVED-ROUTE enforcement — not just data-layer]",
        primary1.get("device_cves_first")
    );

    clone_stage1
        .stop()
        .await
        .expect("stage-1 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 4 server (scenario_start = now - 700s → elapsed ≈ 700s → stage 4 Containment)
    // At stage 4: mask.device_cves=true → device_cves_first MUST be present.
    // Default thresholds: [60, 180, 360, 600] → stage 4 activates at 600s.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage4: i64 = now - 700; // elapsed ≈ 700s > 600s threshold → stage 4

    let timeline_stage4 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage4,
        &[],
    ));
    let time_anchor_stage4 = chrono::DateTime::from_timestamp(start_stage4, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage4 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage4),
        time_anchor_stage4,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-4 server");

    clone_stage4
        .start()
        .await
        .expect("stage-4 server start must succeed");

    let base_url_stage4 = clone_stage4.base_url();
    let token_stage4 = clone_stage4.admin_token().to_owned();

    let resp4 = client
        .get(format!("{base_url_stage4}/api/v1/devices"))
        .header("Authorization", format!("Bearer {token_stage4}"))
        .send()
        .await
        .expect("GET /api/v1/devices (stage 4) must reach the server");

    assert_eq!(resp4.status().as_u16(), 200);

    let body4: serde_json::Value = resp4.json().await.expect("stage-4 response must be JSON");
    let devices4: Vec<serde_json::Value> = body4["data"]["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // At stage 4, primary device IS present (mask.primary_device=true, stage_idx > 0).
    let primary_record_stage4 = devices4.iter().find(|r| {
        r.get("asset_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id.as_str())
            .unwrap_or(false)
    });

    assert!(
        primary_record_stage4.is_some(),
        "F-PIVOT003-R7A-001 prereq: primary device '{}' must be present at stage 4 \
         (mask.primary_device=true, stage_idx=4 > 0); not found in response",
        primary_id
    );

    let primary4 = primary_record_stage4.unwrap();
    assert!(
        primary4.get("device_cves_first").is_some(),
        "F-PIVOT003-R7A-001 BC-2.06.019 v1.13 PC-4: at stage 4 (Containment, \
         mask.device_cves=true), device_cves_first MUST be present in GET /api/v1/devices \
         response; found record {:?}. \
         routes/devices.rs must serve device_cves_first when mask.device_cves=true.",
        primary4
    );

    clone_stage4
        .stop()
        .await
        .expect("stage-4 server stop must succeed");
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R8C-001 SERVED-ROUTE tests — search.rs device branch
// ---------------------------------------------------------------------------
//
// The canonical `from armis.devices` query path in armis.sensor.toml:
//   path_template = "/api/v1/search?aql=${query.filter.aql}"
//   response_path = "$.data.results"
//
// The `devices` table's path_template routes through GET /api/v1/search, NOT
// GET /api/v1/devices.  ALL StageMask guards that apply to device records
// must be enforced on the search.rs device branch (the real query path),
// not only on devices.rs (a sibling endpoint the adapter does NOT call for
// device table queries).
//
// F-PIVOT003-R7 applied the `device_cves_first` strip only to devices.rs;
// F-PIVOT003-R8C-001 applies the same strip (and the entity-visibility guards)
// to search.rs, which is the route the query engine actually hits.

/// SERVED-ROUTE TEST — F-PIVOT003-R8C-001 (primary guard): StageMask primary/lateral
/// entity visibility guards are enforced on GET /api/v1/search?aql=in:devices
/// (the canonical armis.devices query path).
///
/// Asserts:
///   - stage 0: primary device ABSENT from search results (stage_idx == 0 guard).
///   - stage 1: primary device PRESENT, lateral devices ABSENT (mask.lateral_devices=false).
///
/// FAIL mode (before fix): search.rs device branch served all records without scenario
/// sub-path → primary device leaked at stage 0.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();
    let lateral_ids: Vec<String> = catalog.lateral_device_ids_armis.clone();

    let client = prism_dtu_common::build_test_client();

    // ---- Stage 0 server (scenario_start = now + 30s → elapsed clamped to 0s < 60s, 90s budget) ----
    // Placing start 30s in the future gives a 90-second execution budget before stage 1 activates
    // (vs 50s with now-10). current_stage_index clamps negative elapsed to 0 (EC-019-003).
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0 (Baseline), 90s budget
    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));
    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage0 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-0 server");
    clone_stage0
        .start()
        .await
        .expect("stage-0 server start must succeed");

    let base_url0 = clone_stage0.base_url();
    let token0 = clone_stage0.admin_token().to_owned();

    // GET /api/v1/search?aql=in:devices — the CANONICAL armis.devices table query path.
    let resp0 = client
        .get(format!("{base_url0}/api/v1/search?aql=in:devices"))
        .header("Authorization", format!("Bearer {token0}"))
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices (stage 0) must reach the server");

    assert_eq!(
        resp0.status().as_u16(),
        200,
        "F-PIVOT003-R8C-001: GET /api/v1/search?aql=in:devices (stage 0) must return 200"
    );

    let body0: serde_json::Value = resp0
        .json()
        .await
        .expect("stage-0 search response must be JSON");
    let results0: Vec<serde_json::Value> = body0["data"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let ids0: Vec<String> = results0
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 PC-4 / F-PIVOT003-R8C-001: primary device MUST be absent at stage 0.
    assert!(
        !ids0.contains(&primary_id),
        "F-PIVOT003-R8C-001: at stage 0 (elapsed clamped to 0s < 60s), primary device '{}' \
         MUST be ABSENT from GET /api/v1/search?aql=in:devices (canonical table query path); \
         found it in {:?}. \
         search.rs device branch must apply StageMask projection (stage_idx > 0 guard). \
         [This is the route armis.devices table path_template hits — NOT /api/v1/devices]",
        primary_id,
        ids0
    );

    clone_stage0
        .stop()
        .await
        .expect("stage-0 server stop must succeed");

    // ---- Stage 1 server (elapsed ≈ 90s → stage 1 Recon) ----
    let now = chrono::Utc::now().timestamp();
    let start_stage1: i64 = now - 90;
    let timeline_stage1 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));
    let time_anchor_stage1 = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage1 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage1),
        time_anchor_stage1,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-1 server");
    clone_stage1
        .start()
        .await
        .expect("stage-1 server start must succeed");

    let base_url1 = clone_stage1.base_url();
    let token1 = clone_stage1.admin_token().to_owned();

    let resp1 = client
        .get(format!("{base_url1}/api/v1/search?aql=in:devices"))
        .header("Authorization", format!("Bearer {token1}"))
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices (stage 1) must reach the server");

    assert_eq!(resp1.status().as_u16(), 200);
    let body1: serde_json::Value = resp1
        .json()
        .await
        .expect("stage-1 search response must be JSON");
    let results1: Vec<serde_json::Value> = body1["data"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let ids1: Vec<String> = results1
        .iter()
        .filter_map(|rec| {
            rec.get("asset_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // At stage 1: primary device MUST be present (mask.primary_device=true, stage_idx=1>0).
    assert!(
        ids1.contains(&primary_id),
        "F-PIVOT003-R8C-001: at stage 1 (elapsed ≈ 90s), primary device '{}' \
         MUST be PRESENT in GET /api/v1/search?aql=in:devices; found IDs: {:?}. \
         BC-2.06.019 PC-4 / F-PIVOT003-R8C-001",
        primary_id,
        ids1
    );

    // At stage 1: lateral devices MUST be absent (mask.lateral_devices=false at Recon).
    for lat_id in &lateral_ids {
        assert!(
            !ids1.contains(lat_id),
            "F-PIVOT003-R8C-001: at stage 1 (Recon), lateral device '{}' MUST be ABSENT \
             from GET /api/v1/search?aql=in:devices (mask.lateral_devices=false at stage 1); \
             found it in {:?}. BC-2.06.019 PC-4 / F-PIVOT003-R8C-001",
            lat_id,
            ids1
        );
    }

    clone_stage1
        .stop()
        .await
        .expect("stage-1 server stop must succeed");
}

/// SERVED-ROUTE TEST — F-PIVOT003-R8C-001 (device_cves guard): device_cves_first is
/// absent at stage 1 (mask.device_cves=false) and present at stage 4 (mask.device_cves=true)
/// on the CANONICAL armis.devices query path: GET /api/v1/search?aql=in:devices.
///
/// BC-2.06.019 v1.13 PC-4 / F-PIVOT003-R8C-001: the `device_cves_first` strip must be
/// applied in search.rs (the route path_template points to), not only in devices.rs.
///
/// FAIL mode (before fix): search.rs device branch missing device_cves_first strip →
/// CVE field leaked at stages 0-3 via the canonical query path even though the
/// NVD pivot must not return results until stage 4.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_F_PIVOT003_R8C_001_search_device_cves_first_stagemask_served_route() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let primary_id = catalog.primary_device_id_armis.clone();

    assert!(
        !catalog.device_cves.is_empty(),
        "F-PIVOT003-R8C-001 vacuous guard: catalog.device_cves must be non-empty"
    );

    let client = prism_dtu_common::build_test_client();

    // ---- Stage 1 server (elapsed ≈ 90s → stage 1 Recon, mask.device_cves=false) ----
    let now = chrono::Utc::now().timestamp();
    let start_stage1: i64 = now - 90;
    let timeline_stage1 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage1,
        &[],
    ));
    let time_anchor_stage1 = chrono::DateTime::from_timestamp(start_stage1, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage1 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage1),
        time_anchor_stage1,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-1 server");
    clone_stage1
        .start()
        .await
        .expect("stage-1 server start must succeed");

    let base_url1 = clone_stage1.base_url();
    let token1 = clone_stage1.admin_token().to_owned();

    let resp1 = client
        .get(format!("{base_url1}/api/v1/search?aql=in:devices"))
        .header("Authorization", format!("Bearer {token1}"))
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices (stage 1) must reach server");

    assert_eq!(resp1.status().as_u16(), 200);
    let body1: serde_json::Value = resp1
        .json()
        .await
        .expect("stage-1 search response must be JSON");
    let results1: Vec<serde_json::Value> = body1["data"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Find the primary device record in results.
    let primary1 = results1.iter().find(|r| {
        r.get("asset_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id.as_str())
            .unwrap_or(false)
    });

    // Primary device must be present at stage 1 (mask.primary_device=true, stage_idx=1>0).
    assert!(
        primary1.is_some(),
        "F-PIVOT003-R8C-001 device_cves prereq: primary device '{}' must be present \
         at stage 1 in GET /api/v1/search?aql=in:devices; not found in results",
        primary_id
    );

    let primary1_rec = primary1.unwrap();

    // device_cves_first MUST be absent at stage 1 (mask.device_cves=false at Recon).
    assert!(
        primary1_rec.get("device_cves_first").is_none(),
        "F-PIVOT003-R8C-001 / BC-2.06.019 v1.13 PC-4: at stage 1 (Recon, mask.device_cves=false), \
         device_cves_first MUST be absent from GET /api/v1/search?aql=in:devices results; \
         found value {:?}. \
         search.rs device branch must apply the device_cves_first strip \
         (mirrors devices.rs paginate_devices). The NVD pivot must not return results \
         until stage 4 — leaking device_cves_first early breaks this guarantee.",
        primary1_rec.get("device_cves_first")
    );

    clone_stage1
        .stop()
        .await
        .expect("stage-1 server stop must succeed");

    // ---- Stage 4 server (elapsed ≈ 700s → stage 4 Containment, mask.device_cves=true) ----
    let now = chrono::Utc::now().timestamp();
    let start_stage4: i64 = now - 700;
    let timeline_stage4 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage4,
        &[],
    ));
    let time_anchor_stage4 = chrono::DateTime::from_timestamp(start_stage4, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage4 = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org,
        Arc::clone(&timeline_stage4),
        time_anchor_stage4,
        &catalog,
    )
    .expect("new_with_scenario must succeed for stage-4 server");
    clone_stage4
        .start()
        .await
        .expect("stage-4 server start must succeed");

    let base_url4 = clone_stage4.base_url();
    let token4 = clone_stage4.admin_token().to_owned();

    let resp4 = client
        .get(format!("{base_url4}/api/v1/search?aql=in:devices"))
        .header("Authorization", format!("Bearer {token4}"))
        .send()
        .await
        .expect("GET /api/v1/search?aql=in:devices (stage 4) must reach server");

    assert_eq!(resp4.status().as_u16(), 200);
    let body4: serde_json::Value = resp4
        .json()
        .await
        .expect("stage-4 search response must be JSON");
    let results4: Vec<serde_json::Value> = body4["data"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let primary4 = results4.iter().find(|r| {
        r.get("asset_id")
            .and_then(|v| v.as_str())
            .map(|id| id == primary_id.as_str())
            .unwrap_or(false)
    });

    assert!(
        primary4.is_some(),
        "F-PIVOT003-R8C-001 device_cves stage 4 prereq: primary device '{}' must be present \
         at stage 4 in GET /api/v1/search?aql=in:devices; not found in results",
        primary_id
    );

    let primary4_rec = primary4.unwrap();

    // device_cves_first MUST be present at stage 4 (mask.device_cves=true at Containment).
    assert!(
        primary4_rec.get("device_cves_first").is_some(),
        "F-PIVOT003-R8C-001 / BC-2.06.019 v1.13 PC-4: at stage 4 (Containment, mask.device_cves=true), \
         device_cves_first MUST be present in GET /api/v1/search?aql=in:devices results; \
         found record {:?}. The NVD pivot must be enabled at stage 4.",
        primary4_rec
    );

    clone_stage4
        .stop()
        .await
        .expect("stage-4 server stop must succeed");
}
