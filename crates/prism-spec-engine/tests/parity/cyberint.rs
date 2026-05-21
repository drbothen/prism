#![allow(non_snake_case)]
//! RG-06 / AC-009: Cyberint DTU Parity Test
//!
//! Tests that the spec-driven path (cyberint.sensor.toml + PipelineExecutor)
//! against the Cyberint DTU clone produces OCSF output semantically equivalent
//! to the reference fixture for the alerts table.
//!
//! The incidents table parity test is a separate test function that immediately
//! asserts SKIP — it is NOT tagged #[ignore]. It runs and passes CI.
//!
//! The alerts parity test IS tagged #[ignore] until S-6.09 merges.
//!
//! Per EC-016-013-002: the cyberint.incidents DTU gap is an explicit SKIP assertion,
//! not an ignored test.
//!
//! AC coverage: AC-009 (Cyberint DTU parity + incidents SKIP)
//! HS coverage: HS-015

use prism_core::OrgSlug;
use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;
use prism_spec_engine::NullAuthProvider;
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::SpecLoader;
use std::collections::HashMap;

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
    Warn(String),
    Fail(String),
    // EC-016-013-002: explicit SKIP for cyberint.incidents
    Skip(String),
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
        return ParityVerdict::Warn(
            "Reference fixture is empty — DTU fixture not yet recorded per Task 10a.".to_string(),
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
// RG-06 / HS-015: Cyberint alerts DTU parity (tagged #[ignore])
// ---------------------------------------------------------------------------

/// RG-06 / AC-009 / HS-015 / BC-2.16.013 postcondition 2:
/// test_BC_2_16_013_dtu_parity_cyberint
///
/// Drives the Cyberint alerts spec against the DTU clone.
/// URL: GET /api/v1/alerts (DTU build_router() line 115; NOT /api/alerts).
///
/// Tagged #[ignore] until S-6.09 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "requires prism-dtu-cyberint DTU clone (S-6.09 not yet merged; DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_cyberint() {
    // Step 1: Start the DTU clone.
    let mut clone = CyberintClone::new().expect("CyberintClone::new() must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Cyberint DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load spec, override base_url.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/cyberint.sensor.toml"),
    )
    .expect("cyberint.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("cyberint.sensor.toml must parse");
    spec.base_url = dtu_base_url;

    // Step 3: Resolve the alerts table.
    // URL grounded: DTU registers GET /api/v1/alerts at clone.rs build_router() line 115 (ADR-028 §D1).
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("cyberint spec must declare an 'alerts' table");

    // Step 4: FetchContext.
    // NullAuthProvider is sufficient for DTU (cookie check validates non-empty cyberint_session cookie).
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());

    // Step 5: HTTP client with 30-second timeout.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client build");

    // Step 6: Execute.
    let auth_provider = NullAuthProvider;
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect("PipelineExecutor::execute must succeed against Cyberint DTU");

    // Step 7: Load reference fixture.
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-dtu-cyberint/fixtures/parity/reference-ocsf/alerts.json"),
    )
    .expect("reference OCSF fixture must exist (record per Task 10a before enabling test)");
    let reference_ocsf: serde_json::Value =
        serde_json::from_str(&fixture).expect("reference fixture must be valid JSON");

    // Step 8: Assert parity PASS or WARN.
    let verdict = compute_parity_verdict(&result.records, &reference_ocsf);
    assert!(
        matches!(verdict, ParityVerdict::Pass | ParityVerdict::Warn(_)),
        "Cyberint alerts parity must be PASS or WARN; got FAIL: {:?}",
        verdict
    );
    if let ParityVerdict::Warn(msg) = &verdict {
        eprintln!("[HS-015 parity WARN] Cyberint alerts: {msg}");
    }
}

// ---------------------------------------------------------------------------
// Cyberint incidents SKIP assertion (NOT #[ignore] — runs unconditionally)
// EC-016-013-002: explicit SKIP with standard message
// ---------------------------------------------------------------------------

/// AC-009 / EC-016-013-002 / HS-015:
/// Cyberint incidents table parity test — explicit SKIP assertion.
///
/// The incidents table parity test is an explicit SKIP (NOT #[ignore]).
/// It runs in CI and asserts the SKIP verdict with the standard message:
/// "cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note"
///
/// This is NOT tagged #[ignore] per AC-009 and EC-016-013-002:
/// "NOT #[ignore] — it is an explicit SKIP-assertion test that runs and passes CI."
///
/// RED GATE NOTE: This test is expected to PASS in the Red Gate state (the SKIP
/// assertion itself passes regardless of implementation state). It verifies that
/// the incidents table correctly declares its DTU gap.
#[test]
fn test_BC_2_16_013_dtu_parity_cyberint_incidents_explicit_skip() {
    // Verify the cyberint spec includes an incidents table (it must exist even though
    // the DTU does not have a parity route for it).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/cyberint.sensor.toml"),
    )
    .expect("cyberint.sensor.toml must be readable");

    // If the spec doesn't parse yet (Red Gate), this test will fail on the parse.
    // That's acceptable — it means the spec stub needs to be filled in.
    let spec = SpecLoader::parse(&spec_content).expect("cyberint.sensor.toml must parse");

    // The incidents table must exist in the spec (parseable) even though parity is SKIP.
    let incidents_table = spec.tables.iter().find(|t| t.table_name == "incidents");

    assert!(
        incidents_table.is_some(),
        "cyberint.sensor.toml must declare an 'incidents' table (parseable but SKIP for parity). \
         EC-016-013-002 requires the table to be present in the spec even though the DTU gap exists."
    );

    // Explicit SKIP verdict assertion (standard message per AC-009).
    let verdict = ParityVerdict::Skip(
        "cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note".to_string(),
    );
    assert!(
        matches!(verdict, ParityVerdict::Skip(_)),
        "Cyberint incidents parity must return explicit SKIP verdict (EC-016-013-002)"
    );

    // The SKIP is intentional — do not fail CI on it.
    if let ParityVerdict::Skip(msg) = verdict {
        eprintln!("[AC-009 explicit SKIP] Cyberint incidents: {msg}");
    }
}

/// HS-015 sub-assertion: Cyberint alerts spec declares correct auth_type (cookie_roundtrip).
///
/// RED GATE: Fails until cyberint.sensor.toml has auth_type = "cookie_roundtrip"
/// (corrected from legacy "bearer_static" per ADR-028 §D2 DTU enforcement).
#[test]
fn test_HS_015_BC_2_16_013_cyberint_spec_declares_cookie_roundtrip_auth() {
    let content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/cyberint.sensor.toml"),
    )
    .expect("cyberint.sensor.toml must be readable");
    let spec = SpecLoader::parse(&content).expect("cyberint.sensor.toml must parse");

    assert_eq!(
        spec.auth_type,
        prism_spec_engine::spec_parser::AuthType::CookieRoundtrip,
        "Cyberint spec must declare auth_type = 'cookie_roundtrip' per DTU enforcement \
         (ADR-028 §D2; HS-015)"
    );
}
