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
/// - `severity`     — string label {"Low","Medium","High","Critical"} per standalone DTU
///                    parity (F-CSD-P31-MED-001); PRESENT as integer 50 at HEAD —
///                    assertion flipped to is_string() → RED at HEAD
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
    //   - detection_id: string (REQUIRED) — GREEN
    //   - status: string — GREEN
    //
    // RED assertion (F-CSD-P31-MED-001): severity flipped to is_string().
    //   detection_detail() emits integer 50 → is_string() returns false → RED at HEAD.
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

    // severity: string label matching standalone DTU output.
    //
    // F-CSD-P31-MED-001: crowdstrike.sensor.toml declares column_type = "string" and
    // the standalone DTU generator (make_detection_with_ioc) emits string labels
    // {"Low","Medium","High","Critical"} (severity_id 1→"Low", 2→"Medium", 3→"High",
    // _→"Critical"). BC-2.16.013 INV-HARNESS-ROUTE-PARITY: harness must match standalone DTU.
    //
    // RED at HEAD: detection_detail() emits integer 50 → is_string() returns false.
    assert!(
        resource["severity"].is_string(),
        "F-CSD-P31-MED-001 RED: `severity` must be a string label matching standalone DTU \
         output. TOML: detections.severity, column_type = \"string\". \
         Standalone DTU (make_detection_with_ioc): severity_id 1→\"Low\", 2→\"Medium\", \
         3→\"High\", _→\"Critical\". BC-2.16.013 INV-HARNESS-ROUTE-PARITY: harness shape \
         must match standalone DTU. detection_detail() currently emits integer 50 — fix \
         must change to a string label. \
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

// ============================================================================
// Tests: F-CSD-P30-OBS-003 — detection device_id must be a real host-pool ID
//
// Exercises the same full detection summary pipeline as F-CSD-P29-006 plus the
// device list pipeline:
//   POST /oauth2/token          → Bearer token
//   GET  /detects/queries/detects/v1  → detection IDs
//   POST /detects/entities/summaries/GET/v1  → detection records
//   GET  /devices/queries/devices/v1  → host pool device IDs
//
// Red Gate failure mode (both tests):
//   detection_detail() at HEAD emits "device_id": "placeholder" (root and nested).
//   "placeholder" is not in the generate_host_ids() output — harness host pool
//   contains only "h-{org_slug}-{seed}-{index}" IDs.
//
// Test 1 fails first at assertion (a): assert_ne!(device_id, "placeholder") →
// assertion fails because device_id IS "placeholder".
//
// Test 2 fails at the intersection assertion: detection device_ids = {"placeholder"},
// devices = {"h-*", ...}, intersection = ∅ → assert!(!intersection.is_empty()) fails.
// ============================================================================

/// F-CSD-P30-OBS-003: harness CrowdStrike detection_detail() must embed a real
/// host-pool ID in device_id (root and nested device.device_id), not "placeholder".
///
/// Drives the full detection summary pipeline via HTTP routes, then fetches the
/// device pool via list_host_ids and asserts membership for every detection resource:
///
///   (a) root device_id != "placeholder"
///   (b) root device_id starts with "h-"  (generate_host_ids format: h-{org_slug}-{seed}-{index:03})
///   (c) root device_id is a member of the host pool returned by list_host_ids
///   (d) nested device.device_id equals root device_id (consistency invariant)
///
/// # Architect ruling
///
/// F-CSD-P30-OBS-003 architect Option A 2026-07-11: detection_detail() signature
/// must receive det_index + org_slug + seed and compute
/// generate_host_ids(org_slug, seed)[det_index % host_pool.len()] to embed the
/// mapped host ID into both root "device_id" and nested "device.device_id".
///
/// # BC anchors
///
/// - F-CSD-P30-OBS-003 (architect Option A ruling 2026-07-11)
/// - BC-2.16.013 INV-HARNESS-ROUTE-PARITY
/// - generate_host_ids (modulo mapping from det_index to host pool)
///
/// # Red Gate (BC-5.38.001)
///
/// At HEAD, detection_detail() emits "device_id": "placeholder" at root and nested.
/// "placeholder" fails:
///   (a) assert_ne!(device_id, "placeholder") — device_id IS "placeholder"
///   (b) device_id.starts_with("h-") — "placeholder" does not start with "h-"
///   (c) device_pool.contains(device_id) — "placeholder" is not in generate_host_ids output
#[tokio::test]
async fn test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder() {
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
    // Step 1: OAuth token.
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
        "F-CSD-P30-OBS-003 pre-condition: POST /oauth2/token must return HTTP 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("POST /oauth2/token response must be valid JSON");

    let access_token = token_body["access_token"]
        .as_str()
        .expect("POST /oauth2/token must return access_token string");

    // -------------------------------------------------------------------------
    // Step 2: Fetch detection IDs via list_detection_ids.
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
        "F-CSD-P30-OBS-003 pre-condition: GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let ids_body: serde_json::Value = ids_resp
        .json()
        .await
        .expect("GET /detects/queries/detects/v1 response must be valid JSON");

    let detection_ids: Vec<&str> = ids_body["resources"]
        .as_array()
        .expect("GET /detects/queries/detects/v1 must return a 'resources' array")
        .iter()
        .take(5)
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        !detection_ids.is_empty(),
        "F-CSD-P30-OBS-003 pre-condition: harness must generate at least one detection ID"
    );

    // -------------------------------------------------------------------------
    // Step 3: Fetch detection summaries via get_detection_summaries.
    // -------------------------------------------------------------------------
    let summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": detection_ids }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 must reach server");

    assert_eq!(
        summaries_resp.status().as_u16(),
        200,
        "F-CSD-P30-OBS-003 pre-condition: POST /detects/entities/summaries/GET/v1 must \
         return HTTP 200"
    );

    let summaries_body: serde_json::Value = summaries_resp
        .json()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 response must be valid JSON");

    let resources = summaries_body["resources"]
        .as_array()
        .expect("POST /detects/entities/summaries/GET/v1 must return a 'resources' array");

    assert!(
        !resources.is_empty(),
        "F-CSD-P30-OBS-003 pre-condition: resources must be non-empty for the requested IDs"
    );

    // -------------------------------------------------------------------------
    // Step 4: Fetch the device ID pool via list_host_ids.
    //
    // generate_host_ids("acme-corp", 42) produces HOST_COUNT=30 IDs in format
    // "h-acme-corp-42-{index:03}". These are the IDs the devices table exposes.
    // -------------------------------------------------------------------------
    let devices_resp = client
        .get(format!("http://{addr}/devices/queries/devices/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .expect("GET /devices/queries/devices/v1 must reach server");

    assert_eq!(
        devices_resp.status().as_u16(),
        200,
        "F-CSD-P30-OBS-003 pre-condition: GET /devices/queries/devices/v1 must return HTTP 200"
    );

    let devices_body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("GET /devices/queries/devices/v1 response must be valid JSON");

    let device_pool: std::collections::HashSet<String> = devices_body["resources"]
        .as_array()
        .expect("GET /devices/queries/devices/v1 must return a 'resources' array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !device_pool.is_empty(),
        "F-CSD-P30-OBS-003 pre-condition: host pool from list_host_ids must be non-empty"
    );

    // -------------------------------------------------------------------------
    // Step 5: Assert device_id properties for every detection resource.
    // -------------------------------------------------------------------------
    for (i, resource) in resources.iter().enumerate() {
        let device_id = resource["device_id"].as_str().unwrap_or("");

        // (a) Must not be the literal "placeholder" string.
        //
        // F-CSD-P30-OBS-003 RED: detection_detail() at HEAD hardcodes
        // "device_id": "placeholder". The implementer fix (architect Option A
        // 2026-07-11) must replace "placeholder" with
        // generate_host_ids(org_slug, seed)[det_index % host_pool.len()].
        assert_ne!(
            device_id, "placeholder",
            "F-CSD-P30-OBS-003 RED (a): detection resources[{i}] root device_id must not \
             be the literal \"placeholder\" string. \
             detection_detail() currently hardcodes \"placeholder\"; architect Option A \
             2026-07-11 requires embedding a real host ID from generate_host_ids via \
             det_index modulo mapping. \
             got resource: {resource}"
        );

        // (b) Must start with "h-" (generate_host_ids format: h-{org_slug}-{seed}-{index:03}).
        //
        // F-CSD-P30-OBS-003 RED: "placeholder" does not start with "h-".
        assert!(
            device_id.starts_with("h-"),
            "F-CSD-P30-OBS-003 RED (b): detection resources[{i}] root device_id must start \
             with \"h-\" (generate_host_ids format: h-{{org_slug}}-{{seed}}-{{index:03}}). \
             \"placeholder\" fails this check. Fix: embed real host ID from generate_host_ids. \
             got device_id: {device_id:?}, resource: {resource}"
        );

        // (c) Must be a member of the actual host pool from list_host_ids.
        //
        // F-CSD-P30-OBS-003 RED: "placeholder" is not in the generate_host_ids() output.
        // Root cause of DEFECT-CSDEVICES-EMPTY-PIPELINE-001: PrismQL JOIN on
        // detections.device_id = devices.device_id produces 0 rows because the
        // detection side emits "placeholder" while the devices side emits "h-*" IDs
        // from generate_host_ids.
        assert!(
            device_pool.contains(device_id),
            "F-CSD-P30-OBS-003 RED (c): detection resources[{i}] root device_id must be a \
             member of the host pool from list_host_ids (generate_host_ids output). \
             \"placeholder\" is not in the pool — this is the root cause of \
             DEFECT-CSDEVICES-EMPTY-PIPELINE-001: PrismQL JOIN detections.device_id = \
             devices.device_id yields 0 rows. Fix: detection_detail() must embed a real \
             host ID via det_index %% host_pool.len() (architect Option A 2026-07-11). \
             got device_id: {device_id:?}, pool sample (first 5): {:?}",
            device_pool.iter().take(5).collect::<Vec<_>>()
        );

        // (d) Nested device.device_id must equal root device_id (consistency invariant).
        //
        // Architect Option A specifies both root and nested fields receive the same
        // mapped host ID. At HEAD both are "placeholder" (equal but wrong); after the
        // fix both must be the same real host ID from generate_host_ids modulo mapping.
        // BC-2.16.013 INV-HARNESS-ROUTE-PARITY: harness shape must match standalone DTU.
        let nested_device_id = resource["device"]["device_id"].as_str().unwrap_or("");
        assert_eq!(
            nested_device_id, device_id,
            "F-CSD-P30-OBS-003: detection resources[{i}] nested device.device_id must equal \
             root device_id. Both fields must carry the same mapped host ID from \
             generate_host_ids modulo mapping (architect Option A 2026-07-11). \
             BC-2.16.013 INV-HARNESS-ROUTE-PARITY. \
             got root device_id: {device_id:?}, nested device.device_id: {nested_device_id:?}, \
             resource: {resource}"
        );
    }
}

/// F-CSD-P30-OBS-003 JOIN-fidelity lock: the intersection of detections device_ids
/// with devices device_ids must be non-empty after the fix.
///
/// This test is the JOIN-fidelity lock for DEFECT-CSDEVICES-EMPTY-PIPELINE-001.
/// A non-empty intersection is the minimum requirement for PrismQL JOIN queries
/// (SELECT * FROM detections JOIN devices ON detections.device_id = devices.device_id)
/// to produce at least one row. When the intersection is empty, all such JOINs
/// return 0 rows regardless of query predicates.
///
/// The test drives the harness to collect:
///   - detection_device_ids: root device_id values from get_detection_summaries
///   - device_ids: resource IDs from list_host_ids
/// and asserts that their intersection is non-empty.
///
/// # Architect ruling
///
/// F-CSD-P30-OBS-003 architect Option A 2026-07-11: see
/// test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder.
/// generate_host_ids modulo mapping guarantees >= 1 detection maps to a host pool
/// member, so the intersection is non-empty after the fix.
///
/// # BC anchors
///
/// - F-CSD-P30-OBS-003 (architect Option A ruling 2026-07-11)
/// - BC-2.16.013 INV-HARNESS-ROUTE-PARITY
/// - generate_host_ids (modulo mapping guarantees non-empty intersection per org)
///
/// # Red Gate (BC-5.38.001)
///
/// At HEAD, all detection resources carry device_id = "placeholder".
/// The devices table hosts only "h-*" IDs from generate_host_ids.
/// intersection = {"placeholder"} ∩ {"h-acme-corp-42-*"} = ∅ →
/// assert!(!intersection.is_empty(), ...) fails.
#[tokio::test]
async fn test_F_CSD_P30_OBS_003_detection_device_ids_join_devices_nonempty() {
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
    // Step 1: OAuth token.
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
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: POST /oauth2/token must return HTTP 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("POST /oauth2/token response must be valid JSON");

    let access_token = token_body["access_token"]
        .as_str()
        .expect("POST /oauth2/token must return access_token string");

    // -------------------------------------------------------------------------
    // Step 2: Collect ALL detection device_ids from get_detection_summaries.
    //
    // Fetches all detection IDs (DETECTION_COUNT=20) then resolves their
    // device_id fields — mirrors the full JOIN left-hand side.
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
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: GET /detects/queries/detects/v1 \
         must return HTTP 200"
    );

    let ids_body: serde_json::Value = ids_resp
        .json()
        .await
        .expect("GET /detects/queries/detects/v1 response must be valid JSON");

    let all_detection_ids: Vec<&str> = ids_body["resources"]
        .as_array()
        .expect("GET /detects/queries/detects/v1 must return a 'resources' array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        !all_detection_ids.is_empty(),
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: harness must generate detection IDs"
    );

    let summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": all_detection_ids }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 must reach server");

    assert_eq!(
        summaries_resp.status().as_u16(),
        200,
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: POST /detects/entities/summaries/GET/v1 \
         must return HTTP 200"
    );

    let summaries_body: serde_json::Value = summaries_resp
        .json()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 response must be valid JSON");

    let detection_resources = summaries_body["resources"]
        .as_array()
        .expect("detection summaries must have resources array");

    // Collect the set of unique device_ids across all detection records.
    let detection_device_ids: std::collections::HashSet<String> = detection_resources
        .iter()
        .filter_map(|r| r["device_id"].as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !detection_device_ids.is_empty(),
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: at least one detection must have \
         a device_id field"
    );

    // -------------------------------------------------------------------------
    // Step 3: Collect ALL device IDs from list_host_ids (JOIN right-hand side).
    //
    // generate_host_ids("acme-corp", 42) produces HOST_COUNT=30 IDs; a single
    // request with default limit=100 returns the full pool.
    // -------------------------------------------------------------------------
    let devices_resp = client
        .get(format!("http://{addr}/devices/queries/devices/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .expect("GET /devices/queries/devices/v1 must reach server");

    assert_eq!(
        devices_resp.status().as_u16(),
        200,
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: GET /devices/queries/devices/v1 \
         must return HTTP 200"
    );

    let devices_body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("GET /devices/queries/devices/v1 response must be valid JSON");

    let all_device_ids: std::collections::HashSet<String> = devices_body["resources"]
        .as_array()
        .expect("GET /devices/queries/devices/v1 must return a 'resources' array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        !all_device_ids.is_empty(),
        "F-CSD-P30-OBS-003 join-fidelity pre-condition: host pool from list_host_ids must \
         be non-empty"
    );

    // -------------------------------------------------------------------------
    // Step 4: Assert the JOIN intersection is non-empty.
    //
    // F-CSD-P30-OBS-003 RED: at HEAD detection_device_ids = {"placeholder"},
    // all_device_ids = {"h-acme-corp-42-001", ..., "h-acme-corp-42-030"}.
    // intersection = ∅ → assertion FAILS.
    //
    // After fix (architect Option A): detection_device_ids contains at least one
    // "h-acme-corp-42-*" member from generate_host_ids modulo mapping, so
    // intersection is non-empty and this test passes.
    // -------------------------------------------------------------------------
    let intersection: std::collections::HashSet<&String> = detection_device_ids
        .iter()
        .filter(|id| all_device_ids.contains(*id))
        .collect();

    assert!(
        !intersection.is_empty(),
        "F-CSD-P30-OBS-003 RED JOIN-fidelity: the intersection of detections device_ids \
         and devices device_ids must be non-empty. An empty intersection means every \
         PrismQL JOIN query (SELECT * FROM detections JOIN devices ON \
         detections.device_id = devices.device_id) returns 0 rows — the silent \
         empty-pipeline failure mode of DEFECT-CSDEVICES-EMPTY-PIPELINE-001. \
         At HEAD detection_detail() emits device_id = \"placeholder\" for every \
         detection; the devices pool contains only generate_host_ids output (\"h-*\" IDs); \
         intersection = ∅. Fix: detection_detail() must embed a real host ID via \
         generate_host_ids modulo mapping (architect Option A 2026-07-11). \
         detection_device_ids: {:?} \
         device_ids sample (first 5): {:?}",
        detection_device_ids,
        all_device_ids.iter().take(5).collect::<Vec<_>>()
    );
}

// ============================================================================
// Test: F-CSD-P31-MED-001 — detection severity must be a string label
//
// Drives the same 3-step HTTP pipeline (token → IDs → summaries) and asserts:
//   (a) severity is a JSON string (not integer)
//   (b) severity value is one of {"Low","Medium","High","Critical"} — the label
//       set emitted by make_detection_with_ioc in the standalone DTU generator
//
// Red Gate failure mode:
//   detection_detail() at HEAD emits "severity": 50 (integer).
//   Assertion (a): is_string() returns false on integer 50 → FAILS.
//   Assertion (b): never reached (panics at (a) first).
// ============================================================================

/// F-CSD-P31-MED-001: harness CrowdStrike `detection_detail()` must emit `severity`
/// as a string label matching standalone DTU output, not an integer score.
///
/// # Parity reference
///
/// The standalone DTU generator (`make_detection_with_ioc`) converts `severity_id`
/// to a string label: 1→"Low", 2→"Medium", 3→"High", _→"Critical".
/// `crowdstrike.sensor.toml` declares `detections.severity` with
/// `column_type = "string"`. The harness `detection_detail()` diverged by emitting
/// the integer score 50 — a surface-and-defer anti-pattern (the comment in P29-006
/// rationalized the divergence instead of fixing it). F-CSD-P31-MED-001 closes
/// the divergence.
///
/// # BC anchors
///
/// - F-CSD-P31-MED-001 (severity type parity, standalone-DTU-as-parity-reference)
/// - BC-2.16.013 INV-HARNESS-ROUTE-PARITY
/// - make_detection_with_ioc (standalone DTU generator, severity label set)
///
/// # Red Gate (BC-5.38.001)
///
/// At HEAD, `detection_detail()` emits `"severity": 50` (integer).
/// Both assertions FAIL:
///   (a) is_string() → false (50 is a number, not a string)
///   (b) valid label check → never reached (panics at (a) first)
#[tokio::test]
async fn test_F_CSD_P31_MED_001_detection_severity_is_string_label_matching_standalone_dtu() {
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
    // Step 1: OAuth bearer token.
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
        "F-CSD-P31-MED-001 pre-condition: POST /oauth2/token must return HTTP 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("POST /oauth2/token response must be valid JSON");

    let access_token = token_body["access_token"]
        .as_str()
        .expect("POST /oauth2/token must return access_token string");

    // -------------------------------------------------------------------------
    // Step 2: Fetch detection IDs.
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
        "F-CSD-P31-MED-001 pre-condition: GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let ids_body: serde_json::Value = ids_resp
        .json()
        .await
        .expect("GET /detects/queries/detects/v1 response must be valid JSON");

    let detection_ids: Vec<&str> = ids_body["resources"]
        .as_array()
        .expect("GET /detects/queries/detects/v1 must return a 'resources' array")
        .iter()
        .take(5)
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        !detection_ids.is_empty(),
        "F-CSD-P31-MED-001 pre-condition: harness must generate at least one detection ID"
    );

    // -------------------------------------------------------------------------
    // Step 3: Fetch detection summaries.
    // -------------------------------------------------------------------------
    let summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": detection_ids }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 must reach server");

    assert_eq!(
        summaries_resp.status().as_u16(),
        200,
        "F-CSD-P31-MED-001 pre-condition: POST /detects/entities/summaries/GET/v1 must \
         return HTTP 200"
    );

    let summaries_body: serde_json::Value = summaries_resp
        .json()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 response must be valid JSON");

    let resources = summaries_body["resources"]
        .as_array()
        .expect("POST /detects/entities/summaries/GET/v1 must return a 'resources' array");

    assert!(
        !resources.is_empty(),
        "F-CSD-P31-MED-001 pre-condition: resources must be non-empty"
    );

    // -------------------------------------------------------------------------
    // Assertions — checked against the first returned detection resource.
    // -------------------------------------------------------------------------
    let resource = &resources[0];

    // (a) severity must be a string (not integer).
    //
    // F-CSD-P31-MED-001 RED (a): detection_detail() at HEAD emits "severity": 50.
    // is_string() returns false on integer → assertion FAILS.
    assert!(
        resource["severity"].is_string(),
        "F-CSD-P31-MED-001 RED (a): `severity` must be a string, not an integer. \
         TOML: detections.severity, column_type = \"string\". \
         Standalone DTU (make_detection_with_ioc): emits string labels, not integer scores. \
         BC-2.16.013 INV-HARNESS-ROUTE-PARITY: harness shape must match standalone DTU. \
         detection_detail() currently emits integer 50. Fix: emit a string label. \
         got resource: {resource}"
    );

    // (b) severity value must be one of the standalone DTU label set.
    //
    // F-CSD-P31-MED-001 RED (b): never reached at HEAD (panics at (a) first).
    // After fix: verifies the implementer chose a label from the correct set.
    let severity = resource["severity"]
        .as_str()
        .expect("F-CSD-P31-MED-001: severity is string (checked above)");

    let valid_labels = ["Low", "Medium", "High", "Critical"];
    assert!(
        valid_labels.contains(&severity),
        "F-CSD-P31-MED-001 RED (b): `severity` value must be one of \
         {{\"Low\",\"Medium\",\"High\",\"Critical\"}} — the label set from \
         make_detection_with_ioc (severity_id 1→\"Low\", 2→\"Medium\", 3→\"High\", \
         _→\"Critical\"). BC-2.16.013 INV-HARNESS-ROUTE-PARITY: harness labels must \
         match standalone DTU. \
         got severity: {severity:?}, resource: {resource}"
    );
}

// ============================================================================
// Test: F-CSD-P31-OBS-001 — detection device_id must be stable across batch subsets
//
// Drives the 3-step HTTP pipeline to collect the full detection ID list (shuffled,
// deterministic order via shuffle_by_seed), then:
//   1. Requests summaries for the FULL ID set → builds {detection_id → device_id} map.
//   2. Requests summaries for a STRICT SUBSET (3rd and 5th IDs from the full list,
//      i.e., indices [2] and [4]) → builds {detection_id → device_id} map.
//   3. Asserts each detection_id maps to the SAME device_id in both responses.
//
// Red Gate failure mode:
//   detection_detail() at HEAD derives det_index from BATCH ENUMERATION POSITION
//   (the `i` counter from `.enumerate()` in get_detection_summaries). When the same
//   detection_id appears at position 2 in the full set but position 0 in the subset,
//   it receives a different det_index → different device_id via generate_host_ids
//   modulo mapping. Assertion fails because host_ids[2 % 30] ≠ host_ids[0 % 30].
// ============================================================================

/// F-CSD-P31-OBS-001: `detection_detail()` device_id mapping must be stable across
/// batch subsets — the same detection_id must map to the same device_id regardless
/// of which subset of IDs is requested in the POST body.
///
/// # Root cause
///
/// `get_detection_summaries` currently derives `det_index` from the batch enumeration
/// position (the `i` counter from `.enumerate()`). When a subset of IDs is requested,
/// each ID's position in the subset differs from its position in the full set →
/// different `det_index` → different host ID via `generate_host_ids` modulo mapping.
///
/// # Fix target
///
/// The implementer will change `det_index` to parse the trailing integer from the
/// detection_id format `det-{org_slug}-{seed}-{NNN}` (e.g., "det-acme-42-003" → 3),
/// making the device_id mapping deterministic and independent of batch composition.
///
/// # BC anchors
///
/// - F-CSD-P31-OBS-001 (det_index batch-subset stability)
/// - BC-2.16.013 INV-HARNESS-ROUTE-PARITY
/// - generate_host_ids (host pool for device_id derivation)
/// - generate_detection_ids (detection ID format: det-{org_slug}-{seed}-{NNN:03})
///
/// # Red Gate (BC-5.38.001)
///
/// At HEAD, batch-enumeration det_index causes device_id instability:
///   Full set: detection_ids[2] → det_index=2 → host_ids[2 % HOST_COUNT]
///   Subset [detection_ids[2], detection_ids[4]]: detection_ids[2] → det_index=0 →
///     host_ids[0 % HOST_COUNT]
/// host_ids[0] ≠ host_ids[2] (all generate_host_ids entries are distinct) →
/// assertion fails.
#[tokio::test]
async fn test_F_CSD_P31_OBS_001_detection_device_id_stable_across_batch_subsets() {
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
    // Step 1: OAuth bearer token.
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
        "F-CSD-P31-OBS-001 pre-condition: POST /oauth2/token must return HTTP 200"
    );

    let token_body: serde_json::Value = token_resp
        .json()
        .await
        .expect("POST /oauth2/token response must be valid JSON");

    let access_token = token_body["access_token"]
        .as_str()
        .expect("POST /oauth2/token must return access_token string");

    // -------------------------------------------------------------------------
    // Step 2: Fetch ALL detection IDs (deterministic shuffled order via
    // shuffle_by_seed — same order on every call for a given seed).
    //
    // generate_detection_ids produces DETECTION_COUNT=20 IDs in format
    // det-{org_slug}-{seed}-{NNN:03}. The server returns them shuffled by seed.
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
        "F-CSD-P31-OBS-001 pre-condition: GET /detects/queries/detects/v1 must return HTTP 200"
    );

    let ids_body: serde_json::Value = ids_resp
        .json()
        .await
        .expect("GET /detects/queries/detects/v1 response must be valid JSON");

    let all_ids: Vec<String> = ids_body["resources"]
        .as_array()
        .expect("GET /detects/queries/detects/v1 must return a 'resources' array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect();

    assert!(
        all_ids.len() >= 5,
        "F-CSD-P31-OBS-001 pre-condition: harness must generate at least 5 detection IDs \
         (DETECTION_COUNT=20); need indices [2] and [4] for subset test. \
         got {} IDs",
        all_ids.len()
    );

    // -------------------------------------------------------------------------
    // Step 3: Request FULL set summaries.
    //
    // POST all IDs in shuffled order. Under current behavior:
    //   all_ids[i] → det_index = i (batch enumeration position)
    // Builds: full_map = {detection_id → device_id}.
    // -------------------------------------------------------------------------
    let all_ids_ref: Vec<&str> = all_ids.iter().map(|s| s.as_str()).collect();

    let full_summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": all_ids_ref }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 (full set) must reach server");

    assert_eq!(
        full_summaries_resp.status().as_u16(),
        200,
        "F-CSD-P31-OBS-001 pre-condition: full-set POST must return HTTP 200"
    );

    let full_body: serde_json::Value = full_summaries_resp
        .json()
        .await
        .expect("full-set summaries response must be valid JSON");

    let full_resources = full_body["resources"]
        .as_array()
        .expect("full-set summaries must have resources array");

    assert_eq!(
        full_resources.len(),
        all_ids.len(),
        "F-CSD-P31-OBS-001 pre-condition: full-set response must return one resource per \
         requested ID"
    );

    // Build {detection_id → device_id} map from full-set response.
    // Resources are returned in the same order as the posted IDs.
    let full_map: std::collections::HashMap<String, String> = full_resources
        .iter()
        .filter_map(|r| {
            let det_id = r["detection_id"].as_str()?.to_owned();
            let dev_id = r["device_id"].as_str()?.to_owned();
            Some((det_id, dev_id))
        })
        .collect();

    assert_eq!(
        full_map.len(),
        all_ids.len(),
        "F-CSD-P31-OBS-001 pre-condition: full_map must have one entry per detection ID \
         (requires detection_id and device_id fields in each resource)"
    );

    // -------------------------------------------------------------------------
    // Step 4: Pick a STRICT SUBSET — 3rd and 5th IDs (indices [2] and [4]).
    //
    // The subset intentionally skips index [0] and [1] so that under the current
    // batch-enumeration behavior the subset's indices (0, 1) differ from the
    // full-set indices (2, 4), producing different device_ids.
    // -------------------------------------------------------------------------
    let subset_ids: Vec<&str> = vec![all_ids[2].as_str(), all_ids[4].as_str()];

    // -------------------------------------------------------------------------
    // Step 5: Request SUBSET summaries.
    //
    // POST [all_ids[2], all_ids[4]]. Under current behavior:
    //   all_ids[2] → det_index=0 (was 2 in full set) → different device_id
    //   all_ids[4] → det_index=1 (was 4 in full set) → different device_id
    // -------------------------------------------------------------------------
    let subset_summaries_resp = client
        .post(format!("http://{addr}/detects/entities/summaries/GET/v1"))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "ids": subset_ids }))
        .send()
        .await
        .expect("POST /detects/entities/summaries/GET/v1 (subset) must reach server");

    assert_eq!(
        subset_summaries_resp.status().as_u16(),
        200,
        "F-CSD-P31-OBS-001 pre-condition: subset POST must return HTTP 200"
    );

    let subset_body: serde_json::Value = subset_summaries_resp
        .json()
        .await
        .expect("subset summaries response must be valid JSON");

    let subset_resources = subset_body["resources"]
        .as_array()
        .expect("subset summaries must have resources array");

    assert_eq!(
        subset_resources.len(),
        2,
        "F-CSD-P31-OBS-001 pre-condition: subset response must return exactly 2 resources"
    );

    // Build {detection_id → device_id} map from subset response.
    let subset_map: std::collections::HashMap<String, String> = subset_resources
        .iter()
        .filter_map(|r| {
            let det_id = r["detection_id"].as_str()?.to_owned();
            let dev_id = r["device_id"].as_str()?.to_owned();
            Some((det_id, dev_id))
        })
        .collect();

    assert_eq!(
        subset_map.len(),
        2,
        "F-CSD-P31-OBS-001 pre-condition: subset_map must have 2 entries \
         (requires detection_id and device_id fields in each resource)"
    );

    // -------------------------------------------------------------------------
    // Step 6: Assert stability — each detection_id in the subset maps to the
    // SAME device_id in both the full-set and subset responses.
    //
    // F-CSD-P31-OBS-001 RED: at HEAD, batch-enumeration shifts det_index:
    //   full_map[all_ids[2]] = host_ids[2 % HOST_COUNT]
    //   subset_map[all_ids[2]] = host_ids[0 % HOST_COUNT]
    // host_ids[2] ≠ host_ids[0] → assertion fails.
    //
    // After fix (parse trailing NNN from detection_id):
    //   Both maps use the same stable det_index derived from the ID's own integer →
    //   both device_ids are equal → assertion passes.
    // -------------------------------------------------------------------------
    for det_id in &[&all_ids[2], &all_ids[4]] {
        let full_device = full_map
            .get(det_id.as_str())
            .unwrap_or_else(|| panic!("F-CSD-P31-OBS-001: {det_id:?} must be in full_map"));
        let subset_device = subset_map
            .get(det_id.as_str())
            .unwrap_or_else(|| panic!("F-CSD-P31-OBS-001: {det_id:?} must be in subset_map"));

        assert_eq!(
            subset_device, full_device,
            "F-CSD-P31-OBS-001 RED: detection_id {det_id:?} maps to device_id \
             {subset_device:?} in the subset request but {full_device:?} in the \
             full-set request. Batch-enumeration det_index shifts when a subset is \
             requested — the same ID gets a different position in the subset than in \
             the full set, producing a different device_id via generate_host_ids \
             modulo mapping. Fix: parse det_index from the trailing integer of the \
             detection_id format det-{{org_slug}}-{{seed}}-{{NNN}} so the mapping is \
             stable regardless of batch composition. \
             F-CSD-P31-OBS-001; BC-2.16.013 INV-HARNESS-ROUTE-PARITY."
        );
    }
}
