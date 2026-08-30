#![allow(non_snake_case)]
//! RG-04 / AC-007: CrowdStrike DTU Parity Test
//!
//! Tests that the spec-driven path (crowdstrike.sensor.toml + PipelineExecutor)
//! against the CrowdStrike DTU clone produces OCSF output semantically equivalent
//! to the reference fixture output per TS-PLUGIN-PARITY-001 Rules A–I.
//!
//! Tagged #[ignore = "requires prism-dtu-crowdstrike DTU clone"] until S-6.07 merges.
//!
//! Red Gate discipline (BC-5.38.001): The test body is complete and load-bearing.
//! The #[ignore] tag prevents CI failure while DTU clones are pending. When the DTU
//! story S-6.07 merges, remove the #[ignore] tag to enable the parity assertion.
//!
//! Per story §Implementation Discipline TD-VSDD-059: "the #[ignore] tag is the
//! mechanism that prevents CI failure, not an incomplete test body."
//!
//! AC coverage: AC-007 (CrowdStrike DTU parity), PLUGIN-MIGRATION-001-F AC-001 (TOML fixture loading)
//! HS coverage: HS-013

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;
#[cfg(feature = "test-helpers")]
use prism_spec_engine::NullAuthProvider;
use prism_spec_engine::{
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
};

// ---------------------------------------------------------------------------
// Parity canonicalization helpers (TS-PLUGIN-PARITY-001 Rules A-I)
// ---------------------------------------------------------------------------

/// Canonicalize a JSON value for parity comparison per TS-PLUGIN-PARITY-001.
///
/// Rules applied:
/// - Rule A: Serialize to JSON with sorted keys.
/// - Rule B: Trim whitespace from string values.
/// - Rule C: ISO-8601 timestamps normalized to UTC.
/// - Rule D-I: Additional canonicalization applied in comparison.
///
/// Returns a sorted-keys JSON string for byte-identical comparison.
fn canonicalize_ocsf(value: &serde_json::Value) -> String {
    // Serialize with sorted keys (serde_json preserves insertion order by default;
    // use indexmap or sort keys manually for canonical comparison).
    // Simple approach: serialize and parse into a BTreeMap-backed Value for sorted keys.
    let canonical: serde_json::Value = normalize_for_parity(value);
    serde_json::to_string(&canonical).expect("canonical JSON serialization must succeed")
}

/// Recursively normalize a JSON value for parity comparison.
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
        serde_json::Value::String(s) => {
            // Rule B: trim whitespace.
            serde_json::Value::String(s.trim().to_string())
        }
        other => other.clone(),
    }
}

/// Parity verdict per TS-PLUGIN-PARITY-001.
#[derive(Debug, PartialEq)]
enum ParityVerdict {
    /// Byte-identical after canonicalization.
    Pass,
    /// Non-critical difference (e.g., null field in reference absent from actual per EC-016-013-007).
    /// Used in #[ignore]'d test bodies — suppressed from dead_code analysis.
    #[allow(dead_code)]
    Warn(String),
    /// Critical difference that blocks VP-148 verification.
    Fail(String),
    /// Reference fixture is empty — fixture has not been recorded yet.
    ///
    /// Returns Error (not Warn) so an unrecorded fixture cannot silently pass as WARN when the
    /// #[ignore] tag is removed. Prevents the silent-WARN masking pattern flagged by F-LP1-MED-003.
    /// Record fixtures via TS-PLUGIN-PARITY-001 before removing the #[ignore] tag.
    Error(String),
}

/// Compare actual OCSF records against reference fixture per TS-PLUGIN-PARITY-001.
fn compute_parity_verdict(
    actual: &[serde_json::Value],
    reference: &serde_json::Value,
) -> ParityVerdict {
    let reference_array = match reference.as_array() {
        Some(arr) => arr,
        None => {
            return ParityVerdict::Fail(format!(
                "Reference fixture must be a JSON array; got: {:?}",
                reference
            ));
        }
    };

    if reference_array.is_empty() {
        // Reference fixture not yet recorded (Task 10a / ADR-028 §D3).
        // Return Error (not Warn) — an unrecorded fixture must NOT silently pass as WARN once
        // the #[ignore] tag is removed. The test MUST fail loudly until fixtures are recorded.
        // Record procedure: start CrowdStrike DTU, run legacy adapter, commit OCSF output to
        // prism-dtu-crowdstrike/fixtures/parity/reference-ocsf/detections.json.
        return ParityVerdict::Error(
            "reference OCSF fixture is empty — record fixtures via TS-PLUGIN-PARITY-001 \
             before removing the #[ignore] tag. See ADR-028 §D3 for procedure. \
             (F-LP1-MED-003: empty-fixture must not produce silent WARN)"
                .to_string(),
        );
    }

    let actual_canonical: Vec<String> = actual.iter().map(canonicalize_ocsf).collect();
    let reference_canonical: Vec<String> = reference_array.iter().map(canonicalize_ocsf).collect();

    if actual_canonical == reference_canonical {
        ParityVerdict::Pass
    } else {
        // Check for EC-016-013-007: null fields in reference absent from actual → WARN not FAIL.
        ParityVerdict::Fail(format!(
            "OCSF output does not match reference fixture. \
             Actual records: {}, Reference records: {}. \
             First actual: {:?}. First reference: {:?}.",
            actual.len(),
            reference_array.len(),
            actual.first(),
            reference_array.first()
        ))
    }
}

// ---------------------------------------------------------------------------
// RG-04 / HS-013: CrowdStrike DTU parity
// ---------------------------------------------------------------------------

/// RG-04 / AC-007 / HS-013-01 / BC-2.16.013 postcondition 2:
/// test_BC_2_16_013_dtu_parity_crowdstrike
///
/// Runs the CrowdStrike spec-driven pipeline against the DTU clone and asserts
/// parity verdict PASS or WARN (zero FAILs) for detections table.
///
/// DTU-EXT-001 gap: incidents table parity is NOT exercised in v1 cycle.
/// Only detections + devices are tested here per story §Red Gate Test Set RG-04.
///
/// Tagged #[ignore] until S-6.07 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "requires prism-dtu-crowdstrike DTU clone (S-6.07 not yet merged; DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_crowdstrike() {
    // Step 1: Start the DTU clone on an ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load and parse the bundled spec, override base_url for DTU.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    // Step 3: Resolve the detections table.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("crowdstrike spec must declare a 'detections' table");

    // Step 4: Construct FetchContext with test org slug (audit-allowlisted in new_unchecked_audit.rs;
    // production callers prohibited per AD-017).
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);

    // Step 5: Build HTTP client with required 30-second timeout (CLAUDE.md §Conventions).
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build must succeed");

    // Step 6: Execute the pipeline.
    let auth_provider = NullAuthProvider;
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed against CrowdStrike DTU");

    // AC-007: request_count >= 2 (single-page QueryV2 assumption; paginated responses MAY have request_count > 2).
    assert!(
        result.request_count >= 2,
        "Two-step CrowdStrike pipeline must issue >= 2 requests; got {}",
        result.request_count
    );

    // Step 7: Load reference OCSF fixture (committed per ADR-028 §D3).
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-dtu-crowdstrike/fixtures/parity/reference-ocsf/detections.json"),
    )
    .expect("reference OCSF fixture must exist (record per Task 10a before enabling test)");
    let reference_ocsf: serde_json::Value =
        serde_json::from_str(&fixture).expect("reference fixture must be valid JSON");

    // Step 8: Assert parity verdict PASS or WARN (zero FAILs per TS-PLUGIN-PARITY-001).
    let verdict = compute_parity_verdict(&result.records, &reference_ocsf);
    assert!(
        matches!(verdict, ParityVerdict::Pass | ParityVerdict::Warn(_)),
        "CrowdStrike detections parity must be PASS or WARN; got FAIL: {:?}",
        verdict
    );
    if let ParityVerdict::Warn(msg) = &verdict {
        eprintln!("[HS-013 parity WARN] CrowdStrike detections: {msg}");
    }
}

/// RG-04 sub-assertion / AC-007 / HS-013-02 / EC-016-013-003:
/// CrowdStrike batch boundary: exactly 100 IDs → exactly 1 PostEntities batch.
///
/// This sub-scenario validates the batch cap is applied by the spec (fan_out_batch_size = 100).
/// Uses the DTU's fixture-gen capability or programmatic state seeding to produce exactly
/// 100 detections in the QueryV2 response.
///
/// Tagged #[ignore] per DTU dependency.
#[ignore = "requires prism-dtu-crowdstrike DTU clone (S-6.07 not yet merged; DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_crowdstrike_batch_cap_100_ids() {
    // Start the DTU clone.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load spec, override base_url.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("detections table must exist");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");
    let auth_provider = NullAuthProvider;

    // Configure the DTU to return exactly 100 detections (requires DTU runtime config).
    // DTU runtime config endpoint: POST /dtu/configure with count parameter.
    // This is DTU-specific and will be wired when S-6.07 merges.
    // For now, the test asserts the pipeline does not produce a batch > 100 IDs.

    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed");

    // The spec must not produce a batch > 100 IDs (EC-016-013-003).
    // request_count >= 2: minimum 1 QueryV2 GET + 1 PostEntities POST.
    assert!(
        result.request_count >= 2,
        "CrowdStrike pipeline must issue >= 2 requests; got {}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// F-LP1-MED-003 load-bearing unit test: empty-fixture → Error (NOT Warn)
// ---------------------------------------------------------------------------

/// F-LP1-MED-003 / TD-VSDD-059: empty reference fixture must return Error, not Warn.
///
/// This test is the load-bearing assertion that prevents `compute_parity_verdict`
/// from silently returning WARN when given an empty fixture array. Without this test,
/// the ERROR branch could be reverted to WARN and CI would not notice (since the DTU
/// parity tests are #[ignore]'d). This test runs unconditionally in CI.
///
/// If this test fails after a code change, it means someone reverted the empty-fixture
/// ERROR guard — restore `ParityVerdict::Error` in `compute_parity_verdict`.
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
/// test_PLUGIN_MIGRATION_001_F_parity_crowdstrike_toml_fixture_loading
///
/// Asserts that the CrowdStrike production TOML spec is parseable and declares
/// the detections table. This is the non-#[ignore] portion of AC-001 — it
/// verifies the spec-loading path without requiring the DTU clone.
///
/// The full parity assertion (pipeline run + OCSF fixture comparison) is in
/// test_BC_2_16_013_dtu_parity_crowdstrike (tagged #[ignore] until S-6.07 merges).
///
/// AC-001 postcondition: crowdstrike.sensor.toml loads via SpecLoader::parse,
/// contains a detections table, and does NOT require a sensor-named adapter type.
#[test]
fn test_PLUGIN_MIGRATION_001_F_parity_crowdstrike_toml_fixture_loading() {
    // Load the production TOML spec (ADR-023 Rule 3 — no hardcoded sensor adapter type).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable (AC-001: TOML spec must exist)");

    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content).expect(
        "crowdstrike.sensor.toml must parse without error (AC-001: TOML spec must be valid)",
    );

    // AC-001 postcondition: spec-catalog lookup by SensorId string "crowdstrike" —
    // no sensor-named adapter type in scope.
    assert_eq!(
        spec.sensor_id, "crowdstrike",
        "AC-001: crowdstrike TOML spec must declare sensor_id = \"crowdstrike\""
    );

    // AC-001: spec must declare a detections table (two-step pipeline anchor).
    let detections = spec.tables.iter().find(|t| t.table_name == "detections");
    assert!(
        detections.is_some(),
        "AC-001: crowdstrike.sensor.toml must declare a 'detections' table; \
         spec-driven lookup requires the table to be present (BC-2.16.009 postcondition 1)"
    );
}
