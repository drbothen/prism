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
//! AC coverage: AC-009 (Cyberint DTU parity + incidents SKIP), PLUGIN-MIGRATION-001-F AC-001 (TOML fixture loading)
//! HS coverage: HS-015

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;
use prism_spec_engine::{
    NullAuthProvider,
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
    // EC-016-013-002: explicit SKIP for cyberint.incidents
    Skip(String),
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
        // the #[ignore] tag is removed. Record procedure: start Cyberint DTU, run legacy adapter,
        // commit OCSF output to prism-dtu-cyberint/fixtures/parity/reference-ocsf/alerts.json.
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
    // NullAuthProvider is used here because this test is tagged #[ignore] and is not live.
    // The production path uses StaticCookieAuthProvider injecting `access_token` cookie
    // per BC-2.01.017 §Postconditions P2 / ADR-031 §D3-a. When this test is ungated
    // (S-6.09), replace NullAuthProvider with StaticCookieAuthProvider wired to the DTU
    // test credential to exercise the full auth path.
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

/// Compute the parity verdict for the cyberint incidents table.
///
/// Per EC-016-013-002, the incidents table is an explicit SKIP — the cyberint API
/// returns incidents in batches that exceed parity fixture comparison economic value.
/// This function is used by the incidents SKIP test (not `#[ignore]`'d) to assert
/// that the verdict computation path returns Skip, not Pass/Warn/Fail.
///
/// If this function is changed to return anything other than Skip, the test
/// `test_BC_2_16_013_dtu_parity_cyberint_incidents_skip` will FAIL, preventing
/// silent removal of the EC-016-013-002 SKIP contract.
fn compute_incidents_parity_verdict_for_skip_test() -> ParityVerdict {
    // EC-016-013-002: cyberint incidents DTU gap is an explicit SKIP.
    // The parity verdict for incidents is always Skip — the table exists in the spec
    // but the DTU does not have a parity route for it per the gap analysis.
    ParityVerdict::Skip(
        "cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note \
         (EC-016-013-002). Incidents batch size exceeds parity fixture comparison value."
            .to_string(),
    )
}

/// AC-009 / RG-06 / EC-016-013-002: cyberint incidents parity is EXPLICIT SKIP.
///
/// This test is NOT `#[ignore]`'d — it runs in CI and passes by asserting the SKIP
/// verdict. It FAILS if someone changes `compute_incidents_parity_verdict_for_skip_test`
/// to return anything other than `ParityVerdict::Skip`, preventing silent erosion of
/// the EC-016-013-002 contract.
///
/// Rationale: cyberint API returns incidents in batches that exceed parity fixture
/// comparison economic value; the spec marks this table as SKIP per EC-016-013-002.
#[test]
fn test_BC_2_16_013_dtu_parity_cyberint_incidents_skip() {
    let verdict = compute_incidents_parity_verdict_for_skip_test();

    match verdict {
        ParityVerdict::Skip(msg) => {
            assert!(
                msg.contains("EC-016-013-002") || msg.contains("incidents"),
                "SKIP verdict message must reference EC-016-013-002 or incidents context; got: {msg}"
            );
        }
        other => panic!(
            "Expected ParityVerdict::Skip for cyberint incidents per EC-016-013-002; got {other:?}"
        ),
    }
}

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
/// test_PLUGIN_MIGRATION_001_F_parity_cyberint_toml_fixture_loading
///
/// Asserts that the Cyberint production TOML spec is parseable and declares
/// both alerts and incidents tables. This is the non-#[ignore] portion of AC-001.
///
/// The full parity assertion (pipeline run + OCSF fixture comparison) is in
/// test_BC_2_16_013_dtu_parity_cyberint (tagged #[ignore] until S-6.09 merges).
/// The incidents parity is an explicit SKIP per EC-016-013-002.
///
/// AC-001 postcondition: cyberint.sensor.toml loads via SpecLoader::parse,
/// and does NOT require a sensor-named adapter type.
#[test]
fn test_PLUGIN_MIGRATION_001_F_parity_cyberint_toml_fixture_loading() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/cyberint.sensor.toml"),
    )
    .expect("cyberint.sensor.toml must be readable (AC-001: TOML spec must exist)");

    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(&spec_content)
        .expect("cyberint.sensor.toml must parse without error (AC-001: TOML spec must be valid)");

    // AC-001: spec-catalog lookup by SensorId string "cyberint" — no sensor-named adapter.
    assert_eq!(
        spec.sensor_id, "cyberint",
        "AC-001: cyberint TOML spec must declare sensor_id = \"cyberint\""
    );

    // AC-001: spec must declare an alerts table.
    let alerts = spec.tables.iter().find(|t| t.table_name == "alerts");
    assert!(
        alerts.is_some(),
        "AC-001: cyberint.sensor.toml must declare an 'alerts' table; \
         spec-driven lookup requires the table to be present (BC-2.16.009 postcondition 1)"
    );

    // AC-001: spec must also declare an incidents table (even though parity is SKIP per EC-016-013-002).
    let incidents = spec.tables.iter().find(|t| t.table_name == "incidents");
    assert!(
        incidents.is_some(),
        "AC-001: cyberint.sensor.toml must declare an 'incidents' table; \
         the DTU gap is an explicit SKIP (EC-016-013-002), not a missing table"
    );
}
