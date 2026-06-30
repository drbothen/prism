//! Red Gate test: BC-2.06.019 PC-4 — Cyberint alerts route StageMask projection
//!
//! Traces to: BC-2.06.019 postcondition 4
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B (updated AC-003 / S-DEMO-ENRICHMENT-PIVOT-003)
//! Finding: BPRL-P2-01
//!
//! FAIL mode (Red Gate): routes/alerts.rs fixture-gen path serves all records
//! without applying StageMask filtering. A synthetic alert record carrying an IOC
//! IP reference appears at stage 0 (where ioc_ips=false) — the assertion that it
//! is ABSENT fails because no filter is applied.
//!
//! Stage clock control: scenario_start_secs chosen to ensure a known stage at request time:
//!   Stage 0 (Baseline): scenario_start = now + 30s → elapsed clamped to 0s < 60s → ioc_ips=false
//!                        (90-second budget; EC-019-003 clamps negative elapsed to 0)
//!   Stage 3 (Exfil):    scenario_start = now - 400s → elapsed ≈ 400s ≥ 360s → ioc_ips=true
//!
//! BC-2.06.019 PC-4 alert-surface semantics (Cyberint):
//!   ioc_ips=false   → alert records whose `alert_data.ip` matches a catalog IOC IP are excluded
//!   ioc_ips=true    → those records appear in the response
//!
//! AC-003 (S-DEMO-ENRICHMENT-PIVOT-003): replaced `_ioc_value`/`_ioc_type` synthetic filter
//! with real-schema IOC field access (`ioc.value`, `iocs[].value`, `alert_data.ip/domain`).
//! Synthetic test records use `alert_data.ip` to carry the IOC IP reference — matching
//! the real Cyberint API schema and the `crate::types::AlertData` struct.
//!
//! Load-bearing assertion pattern (TD-VSDD-059): HTTP-level check via a real
//! CyberintClone server. The test injects a synthetic alert into generated_records
//! (post-construction state mutation) to produce a scenario where filtering MATTERS.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};
use prism_dtu_cyberint::CyberintClone;

/// Org ID with well-known first 4 bytes → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Build an auth cookie header value for the demo access token.
fn access_token_cookie(token: &str) -> String {
    format!("access_token={token}")
}

/// RED GATE TEST — test_BC_2_06_019_cyberint_alerts_stagemask_ioc_filter
///
/// BC-2.06.019 PC-4: Cyberint alerts route must apply StageMask filtering when
/// a scenario timeline is present. Alert records carrying an IOC reference
/// (real-schema `alert_data.ip` field per AC-003) that matches a catalog IOC
/// must be excluded when the corresponding IOC mask field is false.
///
/// AC-003 (S-DEMO-ENRICHMENT-PIVOT-003): synthetic records use `alert_data.ip`
/// (real Cyberint schema field) to reference the catalog IOC IP — not the retired
/// `_ioc_value` synthetic field.
///
/// Asserts:
///   Stage 0 (ioc_ips=false): IOC-referencing alert ABSENT from /api/v1/alerts response.
///   Stage 3 (ioc_ips=true):  IOC-referencing alert PRESENT in /api/v1/alerts response.
///
/// FAIL mode (without StageMask projection):
///   Stage 0 request returns the IOC-referencing alert → assertion "absent at stage 0" FAILS.
#[tokio::test]
async fn test_BC_2_06_019_cyberint_alerts_stagemask_ioc_filter() {
    let org = deadbeef_org();
    let seed: u64 = 42;
    let demo_token = "test-demo-token-bc-2-06-019".to_owned();

    // Build the scenario entity catalog to get the actual catalog IOC IPs.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_ips.is_empty(),
        "catalog.ioc_ips must be non-empty for the test to have a meaningful IOC reference"
    );
    let catalog_ioc_ip = catalog.ioc_ips[0].clone();

    // -------------------------------------------------------------------------
    // Stage 0 server: scenario_start = now + 30s → elapsed clamped to 0s < 60s
    // At stage 0 (Baseline): ioc_ips=false, ioc_domains=false, ioc_hashes=false.
    // 90-second execution budget (vs 50s with now-10). EC-019-003 clamps negative elapsed to 0.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0 (Baseline), 90s budget

    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[], // default thresholds: [60, 180, 360, 600]
    ));

    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage0 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    // Register the demo access token (required for all Cyberint auth routes).
    clone_stage0.state.register_access_token(demo_token.clone());

    // Inject a synthetic alert record that references the catalog IOC IP.
    // This is the load-bearing record: the route handler must EXCLUDE it when
    // ioc_ips=false and INCLUDE it when ioc_ips=true.
    //
    // AC-003 (S-DEMO-ENRICHMENT-PIVOT-003): uses real-schema `alert_data.ip` field
    // to carry the IOC IP reference. The route handler deserializes each alert as
    // `crate::types::Alert` and checks `alert_data.ip` against catalog IOC IPs.
    // The retired `_ioc_value`/`_ioc_type` synthetic fields are not used.
    //
    // Arc::get_mut requires refcount == 1. The clone was just constructed, state
    // is Arc<CyberintState> with refcount=1 at this point (before start()).
    {
        let state_mut = Arc::get_mut(&mut clone_stage0.state)
            .expect("Arc refcount must be 1 before server start — mutation is safe");
        state_mut.generated_records.push(serde_json::json!({
            "alert_id": "synthetic-ioc-alert-bc-2-06-019",
            "id": "synthetic-ioc-alert-bc-2-06-019",
            "ref_id": "REF-synthetic-bc-2-06-019",
            "environment": "production",
            "confidence": 95u64,
            "status": "open",
            "severity": "high",
            "severity_id": 4u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["synthetic-asset.example.com"],
            "title": "Synthetic IOC-Referencing Alert (BC-2.06.019 test)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Synthetic alert for BPRL-P2-01 StageMask test.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // AC-003: real-schema alert_data.ip carries the IOC IP reference.
            // Route handler checks alert_data.ip against catalog IOC IPs when ioc_ips=false.
            "alert_data": {
                "ip": catalog_ioc_ip.clone()
            }
        }));
    }

    clone_stage0
        .start()
        .await
        .expect("stage-0 server start must succeed");
    let base_url_stage0 = clone_stage0.base_url();

    let client = prism_dtu_common::build_test_client();

    // GET /api/v1/alerts at stage 0 (ioc_ips=false).
    let resp0 = client
        .get(format!("{base_url_stage0}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 0) must reach the server");

    assert_eq!(
        resp0.status().as_u16(),
        200,
        "Stage 0: GET /api/v1/alerts must return HTTP 200; got {}",
        resp0.status().as_u16()
    );

    let body0: serde_json::Value = resp0.json().await.expect("stage-0 response must be JSON");
    let data0 = body0["data"].as_array().cloned().unwrap_or_default();

    // Collect alert_ids in the stage-0 response.
    let alert_ids0: Vec<String> = data0
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 PC-4: at stage 0 (ioc_ips=false), the synthetic IOC-referencing
    // alert MUST be ABSENT from the response.
    //
    // FAIL mode: without StageMask projection, all alerts (including the synthetic
    // IOC-referencing one) are served → this assertion FAILS.
    assert!(
        !alert_ids0.contains(&"synthetic-ioc-alert-bc-2-06-019".to_owned()),
        "BC-2.06.019 PC-4 / BPRL-P2-01: at stage 0 (ioc_ips=false), alert \
         'synthetic-ioc-alert-bc-2-06-019' referencing catalog IOC IP '{}' \
         must be ABSENT from GET /api/v1/alerts response; found it in {:?}. \
         Route handler must apply StageMask filtering. \
         [RED GATE: StageMask projection not implemented in routes/alerts.rs]",
        catalog_ioc_ip,
        alert_ids0
    );

    clone_stage0
        .stop()
        .await
        .expect("stage-0 server stop must succeed");

    // -------------------------------------------------------------------------
    // Stage 3 server: scenario_start = now - 400s → elapsed ≈ 400s ≥ 360s
    // At stage 3 (Exfil): ioc_ips=true, ioc_domains=true, ioc_hashes=true
    // The IOC-referencing alert MUST be PRESENT.
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage3: i64 = now - 400;

    let timeline_stage3 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage3,
        &[], // default thresholds: [60, 180, 360, 600]
    ));

    let time_anchor_stage3 = chrono::DateTime::from_timestamp(start_stage3, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone_stage3 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage3),
        time_anchor_stage3,
        &catalog,
    )
    .expect("new_with_scenario for stage-3 must succeed");

    // Register access token and inject the synthetic IOC-referencing alert.
    // AC-003: use real-schema alert_data.ip (same as stage-0 clone above).
    clone_stage3.state.register_access_token(demo_token.clone());

    {
        let state_mut = Arc::get_mut(&mut clone_stage3.state)
            .expect("Arc refcount must be 1 before server start");
        state_mut.generated_records.push(serde_json::json!({
            "alert_id": "synthetic-ioc-alert-bc-2-06-019",
            "id": "synthetic-ioc-alert-bc-2-06-019",
            "ref_id": "REF-synthetic-bc-2-06-019",
            "environment": "production",
            "confidence": 95u64,
            "status": "open",
            "severity": "high",
            "severity_id": 4u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["synthetic-asset.example.com"],
            "title": "Synthetic IOC-Referencing Alert (BC-2.06.019 test)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Synthetic alert for BPRL-P2-01 StageMask test.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // AC-003: real-schema alert_data.ip carries the IOC IP reference.
            "alert_data": {
                "ip": catalog_ioc_ip.clone()
            }
        }));
    }

    clone_stage3
        .start()
        .await
        .expect("stage-3 server start must succeed");
    let base_url_stage3 = clone_stage3.base_url();

    // GET /api/v1/alerts at stage 3 (ioc_ips=true).
    let resp3 = client
        .get(format!("{base_url_stage3}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 3) must reach the server");

    assert_eq!(
        resp3.status().as_u16(),
        200,
        "Stage 3: GET /api/v1/alerts must return HTTP 200; got {}",
        resp3.status().as_u16()
    );

    let body3: serde_json::Value = resp3.json().await.expect("stage-3 response must be JSON");
    let data3 = body3["data"].as_array().cloned().unwrap_or_default();

    let alert_ids3: Vec<String> = data3
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // BC-2.06.019 PC-4: at stage 3 (ioc_ips=true), the synthetic IOC-referencing
    // alert MUST be PRESENT.
    assert!(
        alert_ids3.contains(&"synthetic-ioc-alert-bc-2-06-019".to_owned()),
        "BC-2.06.019 PC-4: at stage 3 (ioc_ips=true), alert \
         'synthetic-ioc-alert-bc-2-06-019' referencing catalog IOC IP '{}' \
         must be PRESENT in GET /api/v1/alerts response; found alert_ids: {:?}",
        catalog_ioc_ip,
        alert_ids3
    );

    clone_stage3
        .stop()
        .await
        .expect("stage-3 server stop must succeed");
}

/// Supplementary test: non-IOC-referencing alerts are NOT filtered by IOC masks.
///
/// At stage 0 (ioc_ips=false), alerts without `alert_data.ip` (AC-003 real-schema
/// IOC field) referencing a catalog IP must still appear.
/// This validates that the filter is selective (only IOC-referencing records are
/// excluded) and does not suppress the entire alerts surface.
///
/// AC-003 (S-DEMO-ENRICHMENT-PIVOT-003): IOC-referencing alert uses `alert_data.ip`
/// (real-schema). Non-IOC alert has no `alert_data.ip` entry (or has an empty ip).
///
/// BC-2.06.019 PC-4: "ioc_hashes/ioc_ips/ioc_domains=false → alert and detection
/// records REFERENCING those catalog IOCs are excluded" — non-referencing alerts pass.
#[tokio::test]
async fn test_BC_2_06_019_cyberint_non_ioc_alerts_not_filtered() {
    let org = deadbeef_org();
    let seed: u64 = 42;
    let demo_token = "test-demo-token-non-ioc-filter".to_owned();

    // Stage 0: scenario_start = now + 30s → elapsed clamped to 0s < 60s → ioc_ips=false.
    // IOC-referencing alerts excluded, others pass. 90-second budget. EC-019-003.
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0 (Baseline), 90s budget

    let catalog = build_scenario_entity_catalog(seed, &org);
    let catalog_ioc_ip = catalog.ioc_ips[0].clone();

    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));

    let time_anchor = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    clone.state.register_access_token(demo_token.clone());

    // Inject two synthetic alerts: one with IOC reference, one without.
    {
        let state_mut =
            Arc::get_mut(&mut clone.state).expect("Arc refcount must be 1 before server start");

        // Alert WITH IOC reference — must be excluded at stage 0.
        // AC-003: uses real-schema alert_data.ip (not retired _ioc_value/_ioc_type).
        state_mut.generated_records.push(serde_json::json!({
            "alert_id": "ioc-referencing-alert",
            "id": "ioc-referencing-alert",
            "ref_id": "REF-ioc-ref",
            "environment": "production",
            "confidence": 90u64,
            "status": "open",
            "severity": "high",
            "severity_id": 4u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["asset.example.com"],
            "title": "IOC-referencing alert",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "IOC-referencing.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // AC-003: real-schema alert_data.ip carries the IOC IP reference.
            "alert_data": {
                "ip": catalog_ioc_ip.clone()
            }
        }));

        // Alert WITHOUT IOC reference — must pass through at stage 0.
        // AC-003: no alert_data.ip field (or alert_data entirely absent) → passes filter.
        state_mut.generated_records.push(serde_json::json!({
            "alert_id": "non-ioc-alert",
            "id": "non-ioc-alert",
            "ref_id": "REF-non-ioc",
            "environment": "production",
            "confidence": 80u64,
            "status": "open",
            "severity": "medium",
            "severity_id": 2u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Malware",
            "type": "malware_distribution",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["other-asset.example.com"],
            "title": "Non-IOC alert",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "No IOC reference.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // No alert_data.ip field — must NOT be filtered (not an IOC-referencing alert).
        }));
    }

    clone.start().await.expect("server start must succeed");
    let base_url = clone.base_url();

    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts must reach the server");

    assert_eq!(resp.status().as_u16(), 200);

    let body: serde_json::Value = resp.json().await.expect("response must be JSON");
    let data = body["data"].as_array().cloned().unwrap_or_default();

    let alert_ids: Vec<String> = data
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // IOC-referencing alert must be ABSENT at stage 0.
    assert!(
        !alert_ids.contains(&"ioc-referencing-alert".to_owned()),
        "BC-2.06.019 PC-4: at stage 0 (ioc_ips=false), 'ioc-referencing-alert' \
         referencing catalog IOC IP must be ABSENT; found in {:?}",
        alert_ids
    );

    // Non-IOC alert must be PRESENT at stage 0 (filter is selective, not blanket).
    assert!(
        alert_ids.contains(&"non-ioc-alert".to_owned()),
        "BC-2.06.019 PC-4: at stage 0, 'non-ioc-alert' (no IOC reference) \
         must remain PRESENT; not found in {:?}",
        alert_ids
    );

    clone.stop().await.expect("server stop must succeed");
}

/// Test: iocs[].value IOC hash filtering by StageMask.
///
/// AC-003 (S-DEMO-ENRICHMENT-PIVOT-003): the real-schema filter uses `iocs[].value`
/// (typed `Alert.iocs` array) to extract hash IOC values. At stage 0 (ioc_hashes=false),
/// alert records with `iocs[].value` matching a catalog hash must be WITHHELD.
/// At stage 2 (ioc_hashes=true), those records must be PRESENT.
///
/// This test replaces the retired `_ioc_value`/`_ioc_type` fail-closed test (BPRL-P3-OBS-1)
/// which tested the old synthetic field mechanism. The real-schema approach uses typed
/// `Alert.iocs` deserialization — records carrying catalog hash IOCs in `iocs[].value`
/// are withheld when `ioc_hashes=false` (AC-003 real-schema filter in routes/alerts.rs).
///
/// BC-2.06.019 PC-4: ioc_hashes=false → alerts referencing catalog IOC hashes are excluded.
#[tokio::test]
async fn test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld() {
    let org = deadbeef_org();
    let seed: u64 = 42;
    let demo_token = "test-demo-token-iocs-hash-filter".to_owned();

    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "catalog.ioc_hashes must be non-empty for this test"
    );
    let catalog_hash = catalog.ioc_hashes[0].clone();

    // Stage 0 (Baseline): scenario_start = now + 30s → elapsed clamped to 0s < 60s.
    // ioc_hashes=false → alert with iocs[].value = catalog hash must be ABSENT.
    // 90-second budget. EC-019-003 clamps negative elapsed to 0.
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now + 30; // elapsed clamped to 0s → stage 0 (Baseline), 90s budget

    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let mut clone = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    clone.state.register_access_token(demo_token.clone());

    {
        let state_mut =
            Arc::get_mut(&mut clone.state).expect("Arc refcount must be 1 before server start");

        // Alert with iocs[].value = catalog hash — must be withheld at stage 0 (ioc_hashes=false).
        // AC-003: real-schema `iocs` array is checked by the route handler.
        state_mut.generated_records.push(serde_json::json!({
            "alert_id": "iocs-hash-alert",
            "id": "iocs-hash-alert",
            "ref_id": "REF-iocs-hash",
            "environment": "production",
            "confidence": 85u64,
            "status": "open",
            "severity": "high",
            "severity_id": 4u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["asset.example.com"],
            "title": "Alert with iocs[] hash IOC reference",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "AC-003 real-schema iocs[].value filter test.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // AC-003: real-schema `iocs` array carries the hash IOC value.
            // route handler checks iocs[].value against catalog IOC hashes.
            "iocs": [{"type": "hash_sha256", "value": catalog_hash.clone()}]
        }));
    }

    clone.start().await.expect("server start must succeed");
    let base_url = clone.base_url();

    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts must reach the server");

    assert_eq!(resp.status().as_u16(), 200);

    let body: serde_json::Value = resp.json().await.expect("response must be JSON");
    let data = body["data"].as_array().cloned().unwrap_or_default();

    let alert_ids: Vec<String> = data
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // At stage 0 (ioc_hashes=false), the iocs-hash alert must be WITHHELD.
    // AC-003 real-schema filter: `iocs[].value` = catalog hash → withheld when ioc_hashes=false.
    assert!(
        !alert_ids.contains(&"iocs-hash-alert".to_owned()),
        "BC-2.06.019 PC-4 / AC-003: at stage 0 (ioc_hashes=false), alert 'iocs-hash-alert' \
         with iocs[].value = catalog hash '{}' must be WITHHELD; found in {:?}. \
         Route handler must check iocs[].value against catalog IOC hashes.",
        catalog_hash,
        alert_ids
    );

    clone.stop().await.expect("server stop must succeed");
}
