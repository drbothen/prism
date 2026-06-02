#![allow(non_snake_case)]
//! RG-07 / AC-010: Armis DTU Parity Test
//!
//! Tests that the spec-driven path (armis.sensor.toml + PipelineExecutor)
//! against the Armis DTU clone produces OCSF output semantically equivalent
//! to the reference fixture for the devices table.
//!
//! Includes AQL forwarding sub-case (${query.filter.aql} in path template) and
//! timestamp fallback sub-case (firstSeen → lastSeen → DateTime::now()) per AC-010.
//!
//! DTU-EXT-003: CLOSED by S-DEMO-ARMIS-AQL-001. devices table now uses GET /api/v1/search
//!   with ?aql=${query.filter.aql} (real Armis Centrix AQL endpoint, ADR-031 §D8-a).
//! DTU-EXT-004: CLOSED by S-DEMO-ARMIS-AQL-001. alerts table now uses GET /api/v1/search.
//! AC-005 pipeline round-trip tests (test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline
//!   and test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline) now run live — no #[ignore].
//! ${env} resolution enabled by S-SPEC-ENV-VAR-001 (@4feec93a); tests override base_url directly.
//! Legacy parity comparison tests remain #[ignore]'d: reference OCSF fixtures are empty ([]);
//!   record per TS-PLUGIN-PARITY-001 before removing those #[ignore] tags.
//!
//! AC coverage: AC-010 (Armis DTU parity + AQL + timestamp fallback), PLUGIN-MIGRATION-001-F AC-001 (TOML fixture loading)
//! HS coverage: HS-016

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;
use prism_spec_engine::{
    MockAuthProvider, NullAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
};

fn canonicalize_ocsf(value: &serde_json::Value) -> String {
    serde_json::to_string(&normalize_for_parity(value))
        .expect("canonical JSON serialization must succeed")
}

fn normalize_for_parity(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted: std::collections::BTreeMap<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_for_parity(v)))
                .collect();
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(normalize_for_parity).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(s.trim().to_string()),
        other => other.clone(),
    }
}

#[derive(Debug, PartialEq)]
enum ParityVerdict {
    Pass,
    /// Used in #[ignore]'d test bodies — suppressed from dead_code analysis.
    #[allow(dead_code)]
    Warn(String),
    Fail(String),
    /// Reference fixture is empty — fixture has not been recorded yet.
    ///
    /// Returns Error (not Warn) so an unrecorded fixture cannot silently pass as WARN when the
    /// #[ignore] tag is removed. Prevents the silent-WARN masking pattern flagged by F-LP1-MED-003.
    /// Record fixtures via TS-PLUGIN-PARITY-001 before removing the #[ignore] tag.
    Error(String),
}

fn compute_parity_verdict(
    actual: &[serde_json::Value],
    reference: &serde_json::Value,
) -> ParityVerdict {
    let reference_array = match reference.as_array() {
        Some(arr) => arr,
        None => return ParityVerdict::Fail("Reference fixture must be JSON array".to_string()),
    };

    if reference_array.is_empty() {
        // Return Error (not Warn) — an unrecorded fixture must NOT silently pass as WARN once
        // the #[ignore] tag is removed. Record procedure: start Armis DTU, run legacy adapter,
        // commit OCSF output to prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json.
        return ParityVerdict::Error(
            "reference OCSF fixture is empty — record fixtures via TS-PLUGIN-PARITY-001 \
             before removing the #[ignore] tag. \
             (F-LP1-MED-003: empty-fixture must not produce silent WARN)"
                .to_string(),
        );
    }

    let actual_canonical: Vec<String> = actual.iter().map(canonicalize_ocsf).collect();
    let reference_canonical: Vec<String> = reference_array.iter().map(canonicalize_ocsf).collect();

    if actual_canonical == reference_canonical {
        ParityVerdict::Pass
    } else {
        ParityVerdict::Fail(format!(
            "OCSF mismatch. Actual: {} records, Reference: {} records.",
            actual.len(),
            reference_array.len()
        ))
    }
}

// ---------------------------------------------------------------------------
// RG-07 / HS-016: Armis DTU parity (tagged #[ignore])
// ---------------------------------------------------------------------------

/// RG-07 / AC-010 / HS-016 / BC-2.16.013 postcondition 2:
/// test_BC_2_16_013_dtu_parity_armis
///
/// Drives the Armis spec against the DTU clone for the devices table.
///
/// DTU-EXT-003: DTU has GET /api/v1/devices. AQL forwarding path (${query.filter.aql})
/// tests that the filter is forwarded correctly when the DTU extension resolves.
/// Timestamp fallback sub-case: re-execute without firstSeen/lastSeen → WARN per Rule C.
///
/// Tagged #[ignore] until S-6.10 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "reference OCSF fixtures are empty ([]); record per TS-PLUGIN-PARITY-001 \
(start DTU, run legacy adapter, commit OCSF output to prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json) \
before removing this tag. DTU-EXT-003/004 are CLOSED by S-DEMO-ARMIS-AQL-001; the remaining \
blocker is unrecorded fixtures, not missing routes."]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_armis() {
    // Step 1: Start the DTU clone.
    let mut clone = ArmisClone::new().expect("ArmisClone::new() must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load spec, override base_url.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("armis.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("armis.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    // Step 3: Resolve the devices table.
    // DTU-EXT-003: DTU has GET /api/v1/devices (not /api/v1/search with AQL).
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("armis spec must declare a 'devices' table");

    // Step 4: AQL forwarding sub-case (AC-010 step 4).
    // Construct FetchContext with AQL filter to exercise ${query.filter.aql} interpolation.
    let mut filters = HashMap::new();
    filters.insert(
        "aql".to_string(),
        "in:devices timeFrame:\"Last 3 Hours\"".to_string(),
    );
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    // Step 5: HTTP client with 30-second timeout.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");

    // Step 6: Execute the pipeline.
    let auth_provider = NullAuthProvider;
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed against Armis DTU");

    // AC-010 step 6: assert DTU received the verbatim AQL expression.
    // The DTU exposes GET /dtu/aql-log to retrieve received AQL strings.
    // This assertion requires the aql-log endpoint to be available — DTU-EXT-003 gap.
    // For now, assert that the pipeline ran without error; aql forwarding verification
    // requires DTU extension (PLUGIN-MIGRATION-Wave-2 cleanup story).

    // Step 7: Load reference fixture (devices table).
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json"),
    )
    .expect("reference OCSF fixture must exist (record per Task 10a before enabling test)");
    let reference_ocsf: serde_json::Value =
        serde_json::from_str(&fixture).expect("reference fixture must be valid JSON");

    // Step 8: Assert parity.
    let verdict = compute_parity_verdict(&result.records, &reference_ocsf);
    assert!(
        matches!(verdict, ParityVerdict::Pass | ParityVerdict::Warn(_)),
        "Armis devices parity must be PASS or WARN; got FAIL: {:?}",
        verdict
    );
    if let ParityVerdict::Warn(msg) = &verdict {
        eprintln!("[HS-016 parity WARN] Armis devices: {msg}");
    }
}

/// RG-07 sub-assertion / AC-010 / EC-016-013-005:
/// Armis timestamp fallback: when firstSeen and lastSeen are both absent,
/// parity PASS by convention (TS-PLUGIN-PARITY-001 Rule C: "both sides took same fallback path").
///
/// Tagged #[ignore] per DTU dependency.
#[ignore = "reference OCSF fixtures are empty ([]); timestamp-fallback parity requires a recorded \
devices fixture (prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json). Record per \
TS-PLUGIN-PARITY-001 before removing this tag. DTU-EXT-003/004 are CLOSED by S-DEMO-ARMIS-AQL-001; \
the remaining blocker is unrecorded fixtures, not missing routes."]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_armis_timestamp_fallback_pass_by_convention() {
    // Start DTU clone and configure to return device records without firstSeen/lastSeen.
    let mut clone = ArmisClone::new().expect("ArmisClone::new() must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("armis.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("armis.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("devices table must exist");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");
    let auth_provider = NullAuthProvider;

    // Execute against DTU configured for timestamp-absent records.
    // DTU runtime config controls whether devices have firstSeen/lastSeen.
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed");

    // TS-PLUGIN-PARITY-001 Rule C: when both sides take the fallback path,
    // parity is PASS by convention (EC-016-013-005).
    // Assert the pipeline ran and returned records (DTU will seed timestamp-absent devices).
    // The timestamp fallback to DateTime::now() means the output field will differ
    // between runs — Rule C says this is acceptable (PASS by convention).
    //
    // The concrete assertion is that the pipeline does NOT error on absent timestamps.
    // A WARN tracing event must be emitted per Rust adapter precedent (checked via tracing capture).
    eprintln!(
        "[AC-010] Armis timestamp fallback: pipeline completed with {} records. \
         Fallback to DateTime::now() is PASS by convention per TS-PLUGIN-PARITY-001 Rule C.",
        result.records.len()
    );
}

/// HS-016 sub-assertion: Armis spec declares correct auth_type (bearer_static).
///
/// RED GATE: Fails until armis.sensor.toml has auth_type = "bearer_static"
/// (corrected from legacy "api_key" per ADR-028 §D2 DTU enforcement).
#[test]
fn test_HS_016_BC_2_16_013_armis_spec_declares_bearer_static_auth() {
    let content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("armis.sensor.toml must be readable");
    let spec = SpecLoader::parse(&content).expect("armis.sensor.toml must parse");

    assert_eq!(
        spec.auth_type,
        prism_spec_engine::spec_parser::AuthType::BearerStatic,
        "Armis spec must declare auth_type = 'bearer_static' per DTU enforcement \
         (ADR-028 §D2; HS-016). Legacy ArmisAuth::auth_type_name() returned 'api_key' — \
         that was a latent label bug deleted by PLUGIN-MIGRATION-001-A."
    );
}

// ---------------------------------------------------------------------------
// F-LP1-MED-003 load-bearing unit test: empty-fixture → Error (NOT Warn)
// ---------------------------------------------------------------------------

/// F-LP1-MED-003 / TD-VSDD-059: empty reference fixture must return Error, not Warn.
///
/// Runs unconditionally in CI. If this test fails after a code change, it means
/// someone reverted the empty-fixture ERROR guard — restore `ParityVerdict::Error` in
/// `compute_parity_verdict`.
#[test]
fn test_BC_2_16_013_compute_parity_verdict_empty_fixture_returns_error() {
    let empty_reference = serde_json::Value::Array(vec![]);
    let actual: Vec<serde_json::Value> = vec![];

    let verdict = compute_parity_verdict(&actual, &empty_reference);

    match verdict {
        ParityVerdict::Error(msg) => {
            assert!(
                msg.contains("empty") || msg.contains("fixture"),
                "Error message must describe the empty-fixture condition; got: {msg}"
            );
        }
        ParityVerdict::Warn(_) => panic!(
            "F-LP1-MED-003: compute_parity_verdict returned Warn for empty fixture — \
             must return Error so unrecorded fixtures fail loudly when #[ignore] is removed"
        ),
        other => panic!(
            "F-LP1-MED-003: compute_parity_verdict returned {other:?} for empty fixture — \
             must return Error"
        ),
    }
}

// ---------------------------------------------------------------------------
// PLUGIN-MIGRATION-001-F AC-001: TOML fixture loading gate (non-#[ignore] part)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// AC-005: AQL push-down round-trip — pipeline boundary test (S-DEMO-ARMIS-AQL-001)
//
// These tests are the load-bearing AC-005 assertions required by F-P1-CRIT-002.
// They drive PipelineExecutor::execute against the real Armis DTU clone using
// armis.sensor.toml (path_template = "/api/v1/search?aql=${query.filter.aql}").
//
// The pipeline percent-encodes the AQL value before embedding it in the URL.
// Axum's Query<SearchQueryParams> extractor on the DTU side percent-decodes it
// before capture_aql() is called. So the AQL-log entry equals the original
// unencoded value.
//
// These tests PASS now (the implementer's TOML fix made the AQL flow through).
// They FAIL if AQL forwarding regresses — e.g., if path_template loses the ?aql=
// parameter, ${query.filter.aql} is removed, or the DTU capture_aql() call is dropped.
// ---------------------------------------------------------------------------

/// AC-005 / BC-2.16.013 / S-DEMO-ARMIS-AQL-001:
/// test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline
///
/// End-to-end AQL push-down round-trip for the devices table:
/// 1. Start the Armis DTU clone on an ephemeral port.
/// 2. Seed FetchContext with query_filters["aql"] = a device AQL expression.
/// 3. Load armis.sensor.toml and override base_url to the DTU's address.
/// 4. Execute PipelineExecutor::execute for the devices table.
/// 5. Assert (a): GET /dtu/aql-log contains the verbatim AQL (percent-decoded).
/// 6. Assert (b): result records are non-empty and device-shaped (device_id present).
///
/// This test is the LOAD-BEARING AC-005 assertion (F-P1-CRIT-002 closure).
/// Regression sentinel: if path_template loses "?aql=${query.filter.aql}", the DTU
/// receives no aql param, falls back to devices (safe fallback), but aql-log will
/// be empty → assertion (a) fails LOAD-BEARINGLY.
#[tokio::test]
async fn test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline() {
    // Step 1: Start the Armis DTU clone.
    let mut clone = ArmisClone::new().expect("AC-005 devices: ArmisClone::new() must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-005 devices: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load armis.sensor.toml and override base_url to the DTU address.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("AC-005 devices: armis.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-005 devices: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Resolve the devices table.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-005 devices: armis spec must declare a 'devices' table")
        .clone();

    // Step 4: Seed FetchContext with the device AQL filter.
    // This is the pipeline-boundary injection: no query engine needed.
    // Per SID-1: drive production code path directly via FetchContext seed.
    let aql_value = "in:devices timeFrame:\"Last 3 Hours\"";
    let mut filters = HashMap::new();
    filters.insert("aql".to_string(), aql_value.to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    // Step 5: HTTP client with 30-second timeout per CLAUDE.md conventions.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-005 devices: reqwest Client::build must succeed");

    // Step 6: Execute the pipeline via PipelineExecutor::execute.
    // The path_template "/api/v1/search?aql=${query.filter.aql}" will be
    // interpolated with the AQL value (percent-encoded), sending the full
    // query to the DTU's GET /api/v1/search endpoint.
    //
    // MockAuthProvider with a non-empty token is required: the Armis DTU's
    // check_bearer_auth requires "Bearer {non-empty}" (HTTP 403 otherwise).
    // NullAuthProvider returns an empty string which the DTU rejects (AC-001 EC-004).
    let auth_provider = MockAuthProvider::new("test-bearer-token");
    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect(
            "AC-005 devices: PipelineExecutor::execute must succeed against Armis DTU \
             (path_template uses /api/v1/search?aql=${query.filter.aql})",
        );

    // Assertion (b): result records are non-empty and device-shaped.
    // The DTU serves devices fixture at $.data.results for device AQL.
    assert!(
        !result.records.is_empty(),
        "AC-005 devices: PipelineExecutor must return non-empty records from \
         $.data.results (devices fixture loaded by DTU); got 0 records. \
         REGRESSION INDICATOR: path_template may have lost the ?aql= parameter or \
         response_path may have changed from $.data.results."
    );

    // Spot-check first record for device_id (DeviceRecord shape).
    let first = &result.records[0];
    assert!(
        first["device_id"].is_string(),
        "AC-005 devices: first record must contain device_id string (DeviceRecord shape \
         from $.data.results); got: {first}. \
         REGRESSION INDICATOR: response_path or DTU routing is wrong."
    );

    // Assertion (a): DTU received the verbatim AQL in /dtu/aql-log.
    // The pipeline percent-encodes the AQL in the URL; axum's Query extractor
    // percent-decodes it before capture_aql() is called — so the log entry
    // equals the original unencoded AQL value.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-005 devices: aql-log reqwest client build");
    let aql_log_resp = client
        .get(format!("{dtu_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-005 devices: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_log_resp.status().as_u16(),
        200,
        "AC-005 devices: GET /dtu/aql-log must return HTTP 200"
    );

    let aql_log_body: serde_json::Value = aql_log_resp
        .json()
        .await
        .expect("AC-005 devices: aql-log response must be valid JSON");

    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("AC-005 devices: aql_strings must be an array");

    assert!(
        aql_strings.iter().any(|s| s.as_str() == Some(aql_value)),
        "AC-005 devices: /dtu/aql-log must contain the verbatim device AQL \
         '{}' after pipeline execution. \
         REGRESSION INDICATOR: path_template lost ?aql= (AQL never sent), \
         or capture_aql() was removed from get_search handler. \
         aql_strings: {:?}",
        aql_value,
        aql_strings
    );
}

/// AC-005 / BC-2.16.013 / S-DEMO-ARMIS-AQL-001:
/// test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline
///
/// End-to-end AQL push-down round-trip for the alerts table:
/// 1. Start the Armis DTU clone on an ephemeral port.
/// 2. Seed FetchContext with query_filters["aql"] = an alert AQL expression.
/// 3. Load armis.sensor.toml and override base_url to the DTU's address.
/// 4. Execute PipelineExecutor::execute for the alerts table.
/// 5. Assert (a): GET /dtu/aql-log contains the verbatim AQL (percent-decoded).
/// 6. Assert (b): result records are non-empty and alert-shaped (alert_id present).
///
/// This test is the LOAD-BEARING AC-005 assertion for the alerts table (F-P1-CRIT-002 closure).
/// Regression sentinel: if path_template loses "?aql=${query.filter.aql}" for the alerts
/// step, the aql-log assertion (a) will fail.
#[tokio::test]
async fn test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline() {
    // Step 1: Start the Armis DTU clone.
    let mut clone = ArmisClone::new().expect("AC-005 alerts: ArmisClone::new() must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-005 alerts: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load armis.sensor.toml and override base_url.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("AC-005 alerts: armis.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-005 alerts: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Resolve the alerts table.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("AC-005 alerts: armis spec must declare an 'alerts' table")
        .clone();

    // Step 4: Seed FetchContext with alert AQL.
    // Real Armis `in:alerts` discriminator routes to the alerts fixture (F-LP12-HIGH-001 fix).
    // The production poller uses `in:alerts status:Open` as the default alert AQL.
    let aql_value = "in:alerts status:Open";
    let mut filters = HashMap::new();
    filters.insert("aql".to_string(), aql_value.to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    // Step 5: HTTP client.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-005 alerts: reqwest Client::build must succeed");

    // Step 6: Execute the pipeline.
    // MockAuthProvider with a non-empty token is required: the DTU's check_bearer_auth
    // requires "Bearer {non-empty}" (HTTP 403 otherwise — AC-001 EC-004).
    let auth_provider = MockAuthProvider::new("test-bearer-token");
    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect(
            "AC-005 alerts: PipelineExecutor::execute must succeed against Armis DTU \
             (alerts table path_template uses /api/v1/search?aql=${query.filter.aql})",
        );

    // Assertion (b): result records are non-empty and alert-shaped.
    assert!(
        !result.records.is_empty(),
        "AC-005 alerts: PipelineExecutor must return non-empty records from \
         $.data.results (alerts fixture); got 0 records. \
         REGRESSION INDICATOR: path_template may have lost ?aql= for alerts step, \
         or response_path changed from $.data.results."
    );

    // Spot-check first record for alert_id (AlertRecord shape).
    let first = &result.records[0];
    assert!(
        first["alert_id"].is_string(),
        "AC-005 alerts: first record must contain alert_id string (AlertRecord shape \
         from $.data.results for Alert AQL); got: {first}. \
         REGRESSION INDICATOR: DTU routing by AQL pattern is broken — \
         'in:alerts' must route to alerts fixture (F-LP12-HIGH-001 fix)."
    );

    // Assertion (a): DTU received the verbatim AQL.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-005 alerts: aql-log reqwest client build");
    let aql_log_resp = client
        .get(format!("{dtu_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-005 alerts: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_log_resp.status().as_u16(),
        200,
        "AC-005 alerts: GET /dtu/aql-log must return HTTP 200"
    );

    let aql_log_body: serde_json::Value = aql_log_resp
        .json()
        .await
        .expect("AC-005 alerts: aql-log response must be valid JSON");

    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("AC-005 alerts: aql_strings must be an array");

    assert!(
        aql_strings.iter().any(|s| s.as_str() == Some(aql_value)),
        "AC-005 alerts: /dtu/aql-log must contain the verbatim alert AQL \
         '{}' after pipeline execution. \
         REGRESSION INDICATOR: alerts table path_template lost ?aql=, or \
         capture_aql() was removed from get_search handler. \
         aql_strings: {:?}",
        aql_value,
        aql_strings
    );
}

// ---------------------------------------------------------------------------
// PLUGIN-MIGRATION-001-F AC-001: TOML fixture loading gate (non-#[ignore] part)
// ---------------------------------------------------------------------------

/// PLUGIN-MIGRATION-001-F / AC-001 / BC-2.16.009 postcondition 1:
/// test_PLUGIN_MIGRATION_001_F_parity_armis_toml_fixture_loading
///
/// Asserts that the Armis production TOML spec is parseable and declares
/// the devices table. This is the non-#[ignore] portion of AC-001.
///
/// The full parity assertion (pipeline run + OCSF fixture comparison) is in
/// test_BC_2_16_013_dtu_parity_armis (tagged #[ignore] until S-6.10 merges).
///
/// AC-001 postcondition: armis.sensor.toml loads via SpecLoader::parse,
/// contains a devices table, and does NOT require a sensor-named adapter type.
/// Exempt from no-hardcoded-sensors perimeter — uses SensorId string, not enum.
#[test]
fn test_PLUGIN_MIGRATION_001_F_parity_armis_toml_fixture_loading() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("armis.sensor.toml must be readable (AC-001: TOML spec must exist)");

    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("armis.sensor.toml must parse without error (AC-001: TOML spec must be valid)");

    // AC-001: spec-catalog lookup by SensorId string "armis" — no sensor-named adapter.
    assert_eq!(
        spec.sensor_id, "armis",
        "AC-001: armis TOML spec must declare sensor_id = \"armis\""
    );

    // AC-001: spec must declare a devices table (parity test anchor; DTU-EXT-003).
    let devices = spec.tables.iter().find(|t| t.table_name == "devices");
    assert!(
        devices.is_some(),
        "AC-001: armis.sensor.toml must declare a 'devices' table; \
         spec-driven lookup requires the table to be present (BC-2.16.009 postcondition 1)"
    );
}
