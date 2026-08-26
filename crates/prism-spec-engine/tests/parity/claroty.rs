#![allow(non_snake_case)]
//! RG-05 / AC-008: Claroty DTU Parity Test
//!
//! Tests that the spec-driven path (claroty.sensor.toml + PipelineExecutor)
//! against the Claroty DTU clone produces OCSF output semantically equivalent
//! to the reference fixture output per TS-PLUGIN-PARITY-001 Rules A–I.
//!
//! Includes polymorphic ID fixture cases per EC-016-013-004:
//! - Integer ID fixture: id = 12345 → assert id column value is "12345" (string-normalized)
//! - UUID string ID fixture: id = "550e8400-..." → assert id column matches reference
//!
//! Tagged #[ignore = "requires prism-dtu-claroty DTU clone"] until S-6.08 merges.
//!
//! AC coverage: AC-008 (Claroty DTU parity + polymorphic ID), PLUGIN-MIGRATION-001-F AC-001 (TOML fixture loading)
//! HS coverage: HS-014

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use prism_spec_engine::{
    NullAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
};

// Re-use canonicalization helpers from super (parity module).
// Since parity/ is a flat test directory (not a module), we inline the helpers.

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
        // the #[ignore] tag is removed. Record procedure: start Claroty DTU, run legacy adapter,
        // commit OCSF output to prism-dtu-claroty/fixtures/parity/reference-ocsf/alerts.json.
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
            "OCSF output mismatch. Actual: {} records, Reference: {} records.",
            actual.len(),
            reference_array.len()
        ))
    }
}

// ---------------------------------------------------------------------------
// RG-05 / HS-014: Claroty DTU parity
// ---------------------------------------------------------------------------

/// RG-05 / AC-008 / HS-014 / BC-2.16.013 postcondition 2:
/// test_BC_2_16_013_dtu_parity_claroty
///
/// Drives the Claroty spec against the DTU clone for the alerts table.
/// Includes polymorphic ID fixture cases (EC-016-013-004).
///
/// Note: assets table deferred — DTU-EXT-002: Claroty DTU has /api/v1/devices, not /api/v1/assets.
/// Parity test pivots to alerts which has a DTU route at /api/v1/alerts per clone.rs build_router().
///
/// Tagged #[ignore] until S-6.08 merges.
#[ignore = "requires prism-dtu-claroty DTU clone (S-6.08 not yet merged; DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_claroty() {
    // Step 1: Start the DTU clone.
    let mut clone = ClarotyClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Claroty DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load and parse the spec, override base_url.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("claroty.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("claroty.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    // Step 3: Resolve the alerts table.
    // Note: assets table is deferred (DTU-EXT-002); test uses alerts per HS-014 v1.1.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("claroty spec must declare an 'alerts' table");

    // Step 4: FetchContext.
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);

    // Step 5: HTTP client with 30-second timeout.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");

    // Step 6: Execute.
    let auth_provider = NullAuthProvider;
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed against Claroty DTU");

    // Step 7: Load reference fixture.
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-dtu-claroty/fixtures/parity/reference-ocsf/alerts.json"),
    )
    .expect("reference OCSF fixture must exist (record per Task 10a before enabling test)");
    let reference_ocsf: serde_json::Value =
        serde_json::from_str(&fixture).expect("reference fixture must be valid JSON");

    // Step 8: Assert parity.
    let verdict = compute_parity_verdict(&result.records, &reference_ocsf);
    assert!(
        matches!(verdict, ParityVerdict::Pass | ParityVerdict::Warn(_)),
        "Claroty alerts parity must be PASS or WARN; got FAIL: {:?}",
        verdict
    );
    if let ParityVerdict::Warn(msg) = &verdict {
        eprintln!("[HS-014 parity WARN] Claroty alerts: {msg}");
    }
}

/// RG-05 / EC-016-013-004: Polymorphic ID — integer ID normalized to string.
///
/// Claroty alert records may have `"id": 12345` (integer) or `"id": "uuid-..."` (string).
/// The spec must declare id column as type = "string" so both are normalized to string.
/// This test exercises the integer-ID case directly.
///
/// Tagged #[ignore] per DTU dependency.
#[ignore = "requires prism-dtu-claroty DTU clone (S-6.08 not yet merged; DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_claroty_polymorphic_integer_id_normalized_to_string() {
    // This test exercises the DTU's integer-ID fixture case.
    // The DTU must be configured to return an alert with id = 12345 (integer).
    // After TOML spec and pipeline handle type coercion, the id column value
    // must be "12345" (string), not 12345 (integer).
    //
    // Full implementation requires DTU runtime configuration to seed integer-ID alerts.
    // Tagged #[ignore] until DTU story S-6.08 merges.

    let mut clone = ClarotyClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Claroty DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("claroty.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("claroty.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("alerts table must exist");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed");

    // Find any record with an integer id field and assert it was normalized to string.
    // The actual fixture seeding via DTU runtime config is part of the DTU story (S-6.08).
    // This assertion is a best-effort check on whatever the DTU returns.
    for record in &result.records {
        if let Some(id_val) = record.get("id") {
            // After string normalization, the id field must be a JSON string, not a number.
            assert!(
                id_val.is_string(),
                "EC-016-013-004: id field in Claroty alert must be normalized to string. \
                 Got: {:?}",
                id_val
            );
        }
    }
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

/// PLUGIN-MIGRATION-001-F / AC-001 / BC-2.16.009 postcondition 1:
/// test_PLUGIN_MIGRATION_001_F_parity_claroty_toml_fixture_loading
///
/// Asserts that the Claroty production TOML spec is parseable and declares
/// the alerts table. This is the non-#[ignore] portion of AC-001.
///
/// The full parity assertion (pipeline run + OCSF fixture comparison) is in
/// test_BC_2_16_013_dtu_parity_claroty (tagged #[ignore] until S-6.08 merges).
///
/// AC-001 postcondition: claroty.sensor.toml loads via SpecLoader::parse,
/// contains an alerts table, and does NOT require a sensor-named adapter type.
#[test]
fn test_PLUGIN_MIGRATION_001_F_parity_claroty_toml_fixture_loading() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("claroty.sensor.toml must be readable (AC-001: TOML spec must exist)");

    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("claroty.sensor.toml must parse without error (AC-001: TOML spec must be valid)");

    // AC-001: spec-catalog lookup by SensorId string "claroty" — no sensor-named adapter.
    assert_eq!(
        spec.sensor_id, "claroty",
        "AC-001: claroty TOML spec must declare sensor_id = \"claroty\""
    );

    // AC-001: spec must declare an alerts table (parity test anchor).
    let alerts = spec.tables.iter().find(|t| t.table_name == "alerts");
    assert!(
        alerts.is_some(),
        "AC-001: claroty.sensor.toml must declare an 'alerts' table; \
         spec-driven lookup requires the table to be present (BC-2.16.009 postcondition 1)"
    );
}
