//! Harness RED gate tests for F-CSD-P29-006 — detection_detail() full TOML field coverage.
//!
//! # Finding summary (F-CSD-P29-006)
//!
//! The harness `detection_detail()` helper generates detection records with only
//! 4 fields (`detection_id`, `status`, `severity`, nested `device{}`). The
//! crowdstrike.sensor.toml `detections` table declares 10+ columns; when the
//! harness omits those fields the spec-engine normalizes NULL for them in
//! demo and pipeline test scenarios — the same class of defect previously closed
//! for `host_detail()` (devices-table 6/6 TOML coverage, OBS-1).
//!
//! # BC anchors
//!
//! - BC-2.16.013 INV-HARNESS-ROUTE-PARITY (extended to schema field parity per SAP-2)
//! - crowdstrike.sensor.toml `detections` table columns: `detection_id`, `created_timestamp`,
//!   `status`, `severity`, `device_id` (root), `tactic`, `technique`, `behaviors_ioc_type`,
//!   `behaviors_ioc_value`, `behaviors_ioc_source`, `behaviors_ioc_description`
//! - Architect IN-SCOPE-FIX ruling 2026-07-11
//!
//! # Red Gate (BC-5.38.001)
//!
//! `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` MUST FAIL before
//! `detection_detail()` is updated. Expected failure: one or more assertions on
//! absent fields (`created_timestamp`, `tactic`, `technique`, `device_id` at root,
//! `behaviors` array and its IOC sub-fields) will panic with "assertion failed" on
//! the absent field. The test that exercises `behaviors` will fail first because
//! `.as_array().expect(...)` panics on a missing/null field — this is load-bearing.
//!
//! # Idiom
//!
//! Tests use the reqwest-over-TcpListener idiom established in
//! `tests/f_csd_p9_001_harness_post_host_details.rs`. Each test spins up a real
//! Harness on an ephemeral 127.0.0.1 port and drives it via real HTTP.
//!
//! CrowdStrike harness auth model: `oauth_token` returns `"dtu-fake-cs-token"` for
//! any POST to `/oauth2/token`; `check_bearer_auth` accepts ANY non-empty Bearer.

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
// Test: F-CSD-P29-006 — detection_detail() full TOML field coverage
//
// Exercises the full detection query pipeline through the harness:
//   POST /oauth2/token          → Bearer token
//   GET  /detects/queries/detects/v1  → detection IDs (bearer auth)
//   POST /detects/entities/summaries/GET/v1  → detection records
//
// Then asserts all 10+ TOML-declared detections columns are present in the
// first resource record returned by get_detection_summaries() / detection_detail().
//
// Red Gate failure mode:
//   Fields absent from detection_detail() at HEAD cause assertion failures.
//   First to fail: `behaviors.as_array().expect(...)` panics (field absent).
//   Others: `is_string()` / `is_number()` return false on JSON null/missing.
// ============================================================================

/// F-CSD-P29-006: harness CrowdStrike clone `detection_detail()` must emit all
/// TOML-declared detections columns so the spec-engine can normalize non-NULL
/// values in demo and pipeline test scenarios.
///
/// # Devices-table precedent
///
/// `host_detail()` was fixed under OBS-1 to emit all crowdstrike_devices TOML
/// columns (`device_id`, `hostname`, `platform_name`, `os_version`, `status`,
/// `first_seen`, `last_seen`, `external_ip`, `local_ip`, `containment_status`,
/// `agent_version`). This test enforces the equivalent contract for
/// `detection_detail()` on the crowdstrike_detections table.
///
/// # Contract source
///
/// crowdstrike.sensor.toml `detections` table (key columns asserted here):
/// - `detection_id` — string, REQUIRED (currently PRESENT — passes GREEN)
/// - `status`       — string (currently PRESENT — passes GREEN)
/// - `severity`     — integer in fixture (currently PRESENT — passes GREEN)
/// - `created_timestamp` — datetime (currently ABSENT — RED)
/// - `device_id`    — string at ROOT (currently ABSENT at root; only nested
///                    under `device.device_id` — RED for root assertion)
/// - `tactic`       — string, ocsf_field = "attack.tactic.name" (currently ABSENT — RED)
/// - `technique`    — string, ocsf_field = "attack.technique.name" (currently ABSENT — RED)
/// - `behaviors_ioc_type`        via source_path "$.behaviors[*].ioc_type" (ABSENT — RED)
/// - `behaviors_ioc_value`       via source_path "$.behaviors[*].ioc_value" (ABSENT — RED)
/// - `behaviors_ioc_source`      via source_path "$.behaviors[*].ioc_source" (ABSENT — RED)
/// - `behaviors_ioc_description` via source_path "$.behaviors[*].ioc_description" (ABSENT — RED)
///
/// # Architect ruling
///
/// F-CSD-P29-006 declared IN-SCOPE-FIX by architect 2026-07-11.
///
/// # Red Gate (BC-5.38.001)
///
/// ALL absent-field assertions below MUST FAIL before `detection_detail()` is
/// updated. If any assertion passes before the fix, that field was already present
/// and the test is non-trivially correct. If all pass without any code change,
/// flag for spec-reviewer: the behavior already existed.
#[tokio::test]
async fn test_F_CSD_P29_006_detection_detail_full_toml_field_coverage() {
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

    // -------------------------------------------------------------------------
    // Step 1: Obtain OAuth bearer token from POST /oauth2/token.
    //
    // The harness oauth_token handler accepts any POST and returns
    // {"access_token": "dtu-fake-cs-token", "token_type": "bearer", "expires_in": 3600}.
    // Uses the real CrowdStrike client_credentials form-encoded flow so the test
    // mirrors production pipeline token acquisition.
    // -------------------------------------------------------------------------
    let token_resp = client
        .post(format!("http://{addr}/oauth2/token"))
        .form(&[
            ("client_id", "test-client-id"),
            ("client_secret", "test-client-secret"),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await
        .expect("POST /oauth2/token must reach server");

    assert_eq!(
        token_resp.status().as_u16(),
        200,
        "F-CSD-P29-006 pre-condition: POST /oauth2/token must return HTTP 200; \
         got {}",
        token_resp.status().as_u16()
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("POST /oauth2/token response must be valid JSON");

    let access_token = token_body["access_token"]
        .as_str()
        .expect("POST /oauth2/token must return access_token string");

    // -------------------------------------------------------------------------
    // Step 2: Get detection IDs from GET /detects/queries/detects/v1.
    //
    // CrowdStrike auth model: any non-empty Bearer is accepted.
    // Response: {"resources": [<detection_id>, ...], "meta": {...}}
    // -------------------------------------------------------------------------
    let ids_resp = client
        .get(format!("http://{addr}/detects/queries/detects/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .expect("GET /detects/queries/detects/v1 must reach server");

    assert_eq!(
        ids_resp.status().as_u16(),
        200,
        "F-CSD-P29-006 pre-condition: GET /detects/queries/detects/v1 must return \
         HTTP 200; got {}",
        ids_resp.status().as_u16()
    );

    let ids_body: serde_json::Value = ids_resp
        .json()
        .await
        .expect("GET /detects/queries/detects/v1 response must be valid JSON");

    let detection_ids = ids_body["resources"]
        .as_array()
        .expect("GET /detects/queries/detects/v1 must return a 'resources' array");

    assert!(
        !detection_ids.is_empty(),
        "F-CSD-P29-006 pre-condition: harness must generate at least one detection \
         ID per org (generate_detection_ids seeds by org_slug)"
    );

    // Collect up to 5 IDs for the summaries request (avoid over-fetching in tests).
    let ids_vec: Vec<&str> = detection_ids
        .iter()
        .take(5)
        .filter_map(|v| v.as_str())
        .collect();

    // -------------------------------------------------------------------------
    // Step 3: POST to /detects/entities/summaries/GET/v1 with the returned IDs.
    //
    // Response: {"resources": [<detection_detail(id)>, ...]}
    // Each resource is produced by detection_detail() in the harness clone.
    // -------------------------------------------------------------------------
    let summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": ids_vec }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 must reach server");

    assert_eq!(
        summaries_resp.status().as_u16(),
        200,
        "F-CSD-P29-006 pre-condition: POST /detects/entities/summaries/GET/v1 must \
         return HTTP 200; got {}",
        summaries_resp.status().as_u16()
    );

    let summaries_body: serde_json::Value = summaries_resp
        .json()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 response must be valid JSON");

    let resources = summaries_body["resources"].as_array().expect(
        "POST /detects/entities/summaries/GET/v1 must return a 'resources' array \
             (get_detection_summaries produces one detection_detail() record per requested ID)",
    );

    assert!(
        !resources.is_empty(),
        "F-CSD-P29-006 pre-condition: resources must be non-empty for the IDs \
         returned by list_detection_ids (no session-id → direct lookup path in \
         get_detection_summaries)"
    );

    let resource = &resources[0];

    // =========================================================================
    // Field coverage assertions — all aligned to crowdstrike.sensor.toml
    // `detections` table column declarations.
    //
    // GREEN assertions (fields already present in detection_detail() at HEAD):
    // =========================================================================

    // detection_id: string, REQUIRED (crowdstrike.sensor.toml options = ["REQUIRED"]).
    assert!(
        resource["detection_id"].is_string(),
        "F-CSD-P29-006: `detection_id` must be a string (REQUIRED). \
         TOML: detections.detection_id, column_type = \"string\", options = [\"REQUIRED\"]. \
         got record: {resource}"
    );

    // status: string.
    assert!(
        resource["status"].is_string(),
        "F-CSD-P29-006: `status` must be a string. \
         TOML: detections.status, column_type = \"string\". \
         got record: {resource}"
    );

    // severity: number (fixture emits integer 50; TOML declares column_type = \"string\"
    // but the harness fixture uses a numeric value consistent with the real API).
    assert!(
        resource["severity"].is_number(),
        "F-CSD-P29-006: `severity` must be a number. \
         TOML: detections.severity, column_type = \"string\" (real CrowdStrike API returns \
         integer severity score 1-100; harness fixture preserves numeric type). \
         got record: {resource}"
    );

    // =========================================================================
    // RED gate assertions — fields ABSENT from detection_detail() at HEAD.
    // ALL assertions below MUST FAIL before detection_detail() is fixed.
    // =========================================================================

    // created_timestamp: top-level datetime string.
    //
    // TOML: detections.created_timestamp, column_type = "datetime", options = ["INDEX"].
    // The spec-engine uses created_timestamp for FQL time-window push-down
    // (extract_time_window_from_ast, ADR-033 T1, BC-2.01.013). A NULL value
    // blocks time-range predicate push-down in harness-driven scenarios.
    //
    // RED: detection_detail() emits only detection_id/status/severity/device{}.
    // created_timestamp is absent from the fixture → resource["created_timestamp"].is_string()
    // returns false.
    assert!(
        resource["created_timestamp"].is_string(),
        "F-CSD-P29-006 RED: `created_timestamp` must be a top-level string (ISO 8601 datetime). \
         TOML: detections.created_timestamp, column_type = \"datetime\", options = [\"INDEX\"]. \
         The spec-engine reads created_timestamp for FQL time-window push-down \
         (ADR-033 T1 / extract_time_window_from_ast). detection_detail() omits this field — \
         spec-engine normalizes NULL, blocking time-range push-down in harness scenarios. \
         Fix: add `\"created_timestamp\": \"2026-01-01T00:00:00Z\"` to detection_detail(). \
         got record: {resource}"
    );

    // tactic: top-level string.
    //
    // TOML: detections.tactic, column_type = "string", ocsf_field = "attack.tactic.name".
    // NULL in harness → OCSF normalization emits NULL for attack.tactic.name.
    //
    // RED: detection_detail() does not include tactic.
    assert!(
        resource["tactic"].is_string(),
        "F-CSD-P29-006 RED: `tactic` must be a top-level string field. \
         TOML: detections.tactic, column_type = \"string\", ocsf_field = \"attack.tactic.name\". \
         detection_detail() omits tactic — spec-engine normalizes NULL for OCSF attack tactic. \
         Fix: add `\"tactic\": \"Discovery\"` (or equivalent) to detection_detail(). \
         got record: {resource}"
    );

    // technique: top-level string.
    //
    // TOML: detections.technique, column_type = "string", ocsf_field = "attack.technique.name".
    // NULL in harness → OCSF normalization emits NULL for attack.technique.name.
    //
    // RED: detection_detail() does not include technique.
    assert!(
        resource["technique"].is_string(),
        "F-CSD-P29-006 RED: `technique` must be a top-level string field. \
         TOML: detections.technique, column_type = \"string\", ocsf_field = \"attack.technique.name\". \
         detection_detail() omits technique — spec-engine normalizes NULL for OCSF attack technique. \
         Fix: add `\"technique\": \"File and Directory Discovery\"` (or equivalent) to detection_detail(). \
         got record: {resource}"
    );

    // device_id: top-level string (NOT nested under device{}).
    //
    // TOML: detections.device_id, column_type = "string", ocsf_field = "device.uid".
    // The spec-engine reads $.device_id from the resource root (no source_path override
    // on this column in crowdstrike.sensor.toml). detection_detail() only embeds
    // device_id under `device.device_id`; the root-level field is absent.
    //
    // RED: resource["device_id"] is JSON null/missing → is_string() returns false.
    assert!(
        resource["device_id"].is_string(),
        "F-CSD-P29-006 RED: `device_id` must be present at the TOP-LEVEL of the resource. \
         TOML: detections.device_id, column_type = \"string\" (no source_path — spec-engine \
         reads $.device_id from the resource root). Currently detection_detail() only embeds \
         device_id inside `device.device_id`; the root-level field is absent — spec-engine \
         normalizes NULL for device_id. \
         Fix: add top-level `\"device_id\": <value>` to detection_detail() (distinct from \
         the nested `device` object). \
         got record: {resource}"
    );

    // behaviors: non-empty array at root.
    //
    // TOML: behaviors_ioc_type/ioc_value/ioc_source/ioc_description all use
    // source_path = "$.behaviors[*].ioc_*". The spec-engine walks the `behaviors`
    // array at root to extract IOC fields. Without a non-empty behaviors array,
    // all four IOC columns normalize to NULL/empty-list.
    //
    // RED: detection_detail() has no `behaviors` key → .as_array().expect() panics.
    let behaviors = resource["behaviors"].as_array().expect(
        "F-CSD-P29-006 RED: `behaviors` must be a non-empty array at the resource root. \
             TOML: behaviors_ioc_type/ioc_value/ioc_source/ioc_description all declare \
             source_path = \"$.behaviors[*].ioc_*\" — the spec-engine walks this array. \
             detection_detail() omits `behaviors` entirely; all four IOC columns normalize \
             to NULL/empty-list in harness-driven scenarios. \
             Fix: add `\"behaviors\": [{\"ioc_type\": ..., \"ioc_value\": ..., \
             \"ioc_source\": ..., \"ioc_description\": ...}]` to detection_detail().",
    );

    assert!(
        !behaviors.is_empty(),
        "F-CSD-P29-006 RED: `behaviors` array must be non-empty — spec-engine \
         source_path expressions ($.behaviors[*].ioc_*) require at least one entry. \
         Fix: include at least one behavior object in the behaviors array in detection_detail(). \
         got record: {resource}"
    );

    let first_behavior = &behaviors[0];

    // behaviors[0].ioc_type: string.
    //
    // TOML: behaviors_ioc_type, source_path = "$.behaviors[*].ioc_type".
    // Real values: hash_sha256, hash_md5, domain, filename, registry_key.
    //
    // RED: behaviors absent at HEAD → never reached (panics on .as_array() above).
    // Post-partial-fix (behaviors added but ioc_type absent): this assertion fires.
    assert!(
        first_behavior["ioc_type"].is_string(),
        "F-CSD-P29-006 RED: behaviors[0].ioc_type must be a string. \
         TOML: behaviors_ioc_type column, source_path = \"$.behaviors[*].ioc_type\". \
         Example values: hash_sha256, hash_md5, domain, filename, registry_key. \
         got behavior: {first_behavior}"
    );

    // behaviors[0].ioc_source: string.
    //
    // TOML: behaviors_ioc_source, source_path = "$.behaviors[*].ioc_source".
    //
    // RED: absent at HEAD.
    assert!(
        first_behavior["ioc_source"].is_string(),
        "F-CSD-P29-006 RED: behaviors[0].ioc_source must be a string. \
         TOML: behaviors_ioc_source column, source_path = \"$.behaviors[*].ioc_source\". \
         Example values: \"catalog\", \"crowdstrike\". \
         got behavior: {first_behavior}"
    );

    // behaviors[0].ioc_description: string.
    //
    // TOML: behaviors_ioc_description, source_path = "$.behaviors[*].ioc_description".
    //
    // RED: absent at HEAD.
    assert!(
        first_behavior["ioc_description"].is_string(),
        "F-CSD-P29-006 RED: behaviors[0].ioc_description must be a string. \
         TOML: behaviors_ioc_description column, source_path = \"$.behaviors[*].ioc_description\". \
         Example value: \"scenario IOC\". \
         got behavior: {first_behavior}"
    );

    // behaviors[0].ioc_value: KEY must EXIST (value may legitimately be null).
    //
    // TOML: behaviors_ioc_value, source_path = "$.behaviors[*].ioc_value".
    // The real CrowdStrike API returns null for ioc_value when no IOC value is
    // associated with the behavior. We assert KEY PRESENCE, not string type.
    //
    // NOTE: Do NOT assert is_string() here — null is a valid ioc_value.
    //
    // RED: absent at HEAD.
    assert!(
        first_behavior.get("ioc_value").is_some(),
        "F-CSD-P29-006 RED: behaviors[0] must have the `ioc_value` KEY present \
         (value may be null — null is a valid ioc_value per the real CrowdStrike API). \
         TOML: behaviors_ioc_value column, source_path = \"$.behaviors[*].ioc_value\". \
         We assert KEY PRESENCE only (not string type) because null ioc_value is expected \
         for non-hash IOC types. \
         got behavior: {first_behavior}"
    );
}
