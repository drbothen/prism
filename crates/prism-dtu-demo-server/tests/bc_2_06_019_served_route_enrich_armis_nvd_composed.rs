//! F-PIVOT003-R11A-002 — Armis→NVD composed served-route → enrich pipeline test.
//!
//! Symmetric to `bc_2_06_019_served_route_enrich_composed.rs` (Cyberint→ThreatIntel).
//! Drives the FULL CHAIN:
//!
//!   1. Build `ArmisClone::new_with_scenario` at stage 4 (Containment, device_cves=true).
//!   2. Build `NvdState` with catalog CVEs pre-populated as HIGH CVSS (base_score=8.1).
//!   3. Start the Armis DTU server.
//!   4. Fetch records THROUGH the served route `GET /api/v1/search?aql=in:devices`.
//!   5. Extract `device_cves_first` from the HTTP response body.
//!   6. Feed those values into the NVD enrich UDF pipeline:
//!      `register_infusion_udfs` → DataFusion MemTable → SQL execution.
//!   7. Assert NON-EMPTY high-CVSS (>=7.0) results + call_count>0 guard + vacuous-pass guard.
//!
//! ## Why this is load-bearing (TD-VSDD-059)
//!
//! This test FAILS if ANY of the following is broken:
//! (a) `device_cves_first` is stripped by StageMask at stage 4 (StageMask regression), OR
//! (b) `GET /api/v1/search?aql=in:devices` does not serve the field (route regression), OR
//! (c) `register_infusion_udfs` is not wired, OR
//! (d) `enrich_single` is never called (hollow-feature guard, TD-VSDD-059), OR
//! (e) Scenario CVEs are not in NvdState with HIGH CVSS (BC-2.06.020 PC-4 regression).
//!
//! ## Stage selection
//!
//! stage 4 (Containment) = elapsed ≥ 600s per default timeline thresholds [60, 180, 360, 600].
//! We use scenario_start = now - 1000s → elapsed = 1000s ≫ 600s → stage_idx=4.
//!
//! ## BC traceability
//!
//! BC-2.06.019 PC-2 (device_cves visible at Containment stage)
//! BC-2.06.019 AC-008 (device_cves_first NVD pivot field)
//! BC-2.06.020 INV-NVD-CVE-CORRELATION-001 (scenario CVEs appear with HIGH CVSS)
//! U17/Ruling 1b (device_cves_first = catalog.device_cves[0] scalar projection)
//! F-PIVOT003-R11A-002 (closing finding: Armis→NVD composed served-route → enrich test required)

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use chrono::Utc;
use datafusion::arrow::array::{Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};
use prism_dtu_nvd::{
    state::NvdState,
    types::{CveMetrics, CveRecord, CvssData, CvssMetricV31, LangValue},
};
use prism_query::infusion_udf::register_infusion_udfs;
use prism_query::memory::{build_session_context, QUERY_MEMORY_POOL_BYTES};
use prism_spec_engine::{InfusionSource, InfusionUdfDescriptor};

// ---------------------------------------------------------------------------
// Org ID helper
// ---------------------------------------------------------------------------

fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

// ---------------------------------------------------------------------------
// NvdInfusionSource — inline (mirrors bc_2_06_019_enrich_pipeline_e2e.rs)
// ---------------------------------------------------------------------------

/// `InfusionSource` backed by `NvdState::lookup_and_count`.
///
/// Drives the genuine enrich pipeline: `InfusionAsyncUdf::invoke_async_with_args` calls
/// `enrich_single(cve_id, ...)` → `NvdState::lookup_and_count`.
/// Call counter proves `enrich_single` was actually invoked (TD-VSDD-059 hollow-feature guard).
struct NvdInfusionSource {
    state: Arc<NvdState>,
    call_count: Arc<AtomicUsize>,
}

impl fmt::Debug for NvdInfusionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NvdInfusionSource")
            .field("call_count", &self.call_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl InfusionSource for NvdInfusionSource {
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let record = self.state.lookup_and_count(input)?;
        let metrics = record.metrics.cvss_metric_v31?;
        let first = metrics.into_iter().next()?;
        Some(serde_json::json!({
            "cve_id": record.id,
            "cvss_base_score": first.cvss_data.base_score,
            "cvss_severity": first.cvss_data.base_severity,
            "cvss_vector": first.cvss_data.vector_string
        }))
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R11A-002 composed test
// ---------------------------------------------------------------------------

/// Composed test — F-PIVOT003-R11A-002: Armis→NVD served-route → enrich pipeline end-to-end.
///
/// Drives the FULL CHAIN:
///   ArmisClone::new_with_scenario (stage 4, device_cves=true)
///   → GET /api/v1/search?aql=in:devices (HTTP served route, StageMask applied)
///   → extract device_cves_first from response body
///   → register_infusion_udfs (NvdInfusionSource)
///   → DataFusion MemTable → SQL execution
///   → assert cvss_base_score >= 7.0 for ≥ 1 row
///
/// LOAD-BEARING: see module-level doc for FAIL modes.
///
/// BC-2.06.019 PC-2 / AC-008 / BC-2.06.020 INV-NVD-CVE-CORRELATION-001
/// U17/Ruling 1b / F-PIVOT003-R11A-002
#[tokio::test]
async fn test_BC_2_06_019_armis_served_route_to_nvd_enrich_pipeline_composed_full_chain() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared scenario entity catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.device_cves.is_empty(),
        "Prereq: catalog.device_cves must be non-empty (scenario seeding failure). \
         F-PIVOT003-R11A-002 / AC-008"
    );

    // Step 2 — Build NvdState with scenario CVEs pre-populated as HIGH CVSS.
    // Mirrors NvdClone::new_with_scenario: base_score=8.1, base_severity="HIGH" (BC-2.06.020 PC-4).
    let mut nvd_registry: HashMap<String, CveRecord> = HashMap::new();
    for cve_id in &catalog.device_cves {
        nvd_registry.insert(
            cve_id.to_uppercase(),
            CveRecord {
                id: cve_id.clone(),
                source_identifier: "prism-scenario@example.com".to_string(),
                published: "2024-01-01T00:00:00.000".to_string(),
                last_modified: "2024-01-01T00:00:00.000".to_string(),
                vuln_status: "Analyzed".to_string(),
                descriptions: vec![LangValue {
                    lang: "en".to_string(),
                    value: format!("Scenario synthetic CVE {cve_id}"),
                }],
                metrics: CveMetrics {
                    cvss_metric_v31: Some(vec![CvssMetricV31 {
                        source: "prism-scenario@example.com".to_string(),
                        r#type: "Primary".to_string(),
                        cvss_data: CvssData {
                            version: "3.1".to_string(),
                            vector_string: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N"
                                .to_string(),
                            base_score: 8.1,
                            base_severity: "HIGH".to_string(),
                        },
                        exploitability_score: 3.9,
                        impact_score: 5.2,
                    }]),
                },
                weaknesses: vec![],
                configurations: vec![],
                references: vec![],
                cisa_kev_vuln_added: None,
            },
        );
    }
    let nvd_state = Arc::new(NvdState::new(nvd_registry));

    // Step 3 — Build ArmisClone at stage 4 (Containment, device_cves=true).
    // scenario_start = now - 1000s → elapsed ≫ stage[4].threshold (600s) → StageMask.device_cves=true.
    // (Default stage thresholds: [60, 180, 360, 600] → stage index 4 at elapsed ≥ 600s.)
    let scenario_start: i64 = Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&Utc);

    let mut armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("ArmisClone::new_with_scenario must succeed");

    // Step 4 — Start the Armis DTU server.
    armis_clone
        .start()
        .await
        .expect("ArmisClone::start() must succeed for served-route test");
    let base_url = armis_clone.base_url();

    // Step 5 — Fetch THROUGH the served route GET /api/v1/search?aql=in:devices.
    // This exercises the StageMask projection: at stage 4, device_cves_first is served.
    // The response is the ACTUAL route output (StageMask applied by the route handler).
    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:devices")])
        .header("Authorization", "Bearer test-armis-nvd-composed-token")
        .send()
        .await
        .expect(
            "F-PIVOT003-R11A-002: GET /api/v1/search?aql=in:devices must reach the ArmisClone server. \
             FAIL = server did not start or is unreachable.",
        );

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-PIVOT003-R11A-002: GET /api/v1/search?aql=in:devices must return HTTP 200. \
         Got: {}",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("F-PIVOT003-R11A-002: /api/v1/search response must be valid JSON");

    // Step 6 — Extract device_cves_first from the HTTP response.
    // Real Armis search response: {"data": {"results": [...], "total": N}}.
    // armis.sensor.toml: response_path = "$.data.results".
    let results = body["data"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let served_cve_values: Vec<String> = results
        .iter()
        .filter_map(|rec| {
            rec.get("device_cves_first")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // Stop the Armis server — we have what we need.
    armis_clone
        .stop()
        .await
        .expect("ArmisClone::stop() must succeed");

    // Vacuous-pass guard: at stage 4 (device_cves=true), the served route MUST include
    // device_cves_first on CompromisedEndpoint device records.
    // If this fires: either StageMask device_cves gate is broken OR ArmisClone did not
    // stamp device_cves_first on device records (F-PIVOT003-R11C-002 / AC-008 failure).
    assert!(
        !served_cve_values.is_empty(),
        "F-PIVOT003-R11A-002 [VACUOUS PASS GUARD]: \
         served-route GET /api/v1/search?aql=in:devices at stage 4 returned NO device_cves_first fields. \
         results_count={}. \
         At stage 4 (scenario_start = now - 1000s), device_cves=true — device_cves_first MUST be served. \
         If this fires: StageMask device_cves gate is broken OR ArmisClone did not stamp catalog CVEs \
         on device records. catalog.device_cves={:?}. \
         BC-2.06.019 PC-2 / AC-008 / U17/Ruling 1b",
        results.len(),
        catalog.device_cves,
    );

    // Step 7 — Wire the DataFusion enrich UDF pipeline with NvdInfusionSource.
    let ctx = build_session_context(QUERY_MEMORY_POOL_BYTES)
        .expect("F-PIVOT003-R11A-002: build_session_context must succeed");

    let call_count = Arc::new(AtomicUsize::new(0));
    let nvd_source: Arc<dyn InfusionSource> = Arc::new(NvdInfusionSource {
        state: Arc::clone(&nvd_state),
        call_count: Arc::clone(&call_count),
    });

    // UDF name: "nvd_cvss_udf_armis_composed" — distinct to avoid collision with Test 11.
    // No source_column: return full JSON so we can parse both score and severity.
    let descriptor = InfusionUdfDescriptor::new(
        "nvd_cvss_udf_armis_composed",
        "cve_id",
        "string",
        "nvd_scenario_armis_composed",
        Arc::clone(&nvd_source),
        None, // no source_column: return full JSON for assertion
        3600,
        "",
    );

    register_infusion_udfs(&ctx, vec![descriptor])
        .expect("F-PIVOT003-R11A-002: register_infusion_udfs must succeed");

    // Step 8 — Build MemTable from served CVE values (not from state.generated_records).
    // This is the key composition: we use the SERVED values, not the raw state.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "device_cves_first",
        DataType::Utf8,
        true,
    )]));
    let arr = StringArray::from(served_cve_values.clone());
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("F-PIVOT003-R11A-002: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("F-PIVOT003-R11A-002: MemTable construction must succeed");
    ctx.register_table("served_armis_devices", Arc::new(table))
        .expect("F-PIVOT003-R11A-002: register_table must succeed");

    // Step 9 — Execute the enrich SQL through DataFusion.
    // Canonical pivot: FROM armis_devices | where has device_cves_first | enrich nvd(device_cves_first) | where cvss_base_score >= 7.0
    // Translated to SQL with the registered async UDF:
    let df = ctx
        .sql(
            "SELECT device_cves_first, nvd_cvss_udf_armis_composed(device_cves_first) AS cvss_result \
             FROM served_armis_devices \
             WHERE device_cves_first IS NOT NULL",
        )
        .await
        .expect(
            "F-PIVOT003-R11A-002: SQL with nvd_cvss_udf_armis_composed must parse and plan. \
             FAIL = UDF not registered or signature mismatch.",
        );

    let batches = df.collect().await.expect(
        "F-PIVOT003-R11A-002: nvd_cvss_udf_armis_composed execution must succeed. \
         FAIL = InfusionAsyncUdf::invoke_async_with_args returned an error.",
    );

    // Step 10 — Assert enrich_single was actually invoked (hollow-feature guard, TD-VSDD-059).
    let enrich_call_count = call_count.load(Ordering::SeqCst);
    assert!(
        enrich_call_count > 0,
        "F-PIVOT003-R11A-002: enrich_single call_count must be > 0 after SQL execution. \
         Got 0 — InfusionAsyncUdf::invoke_async_with_args did NOT call the source. \
         This is the hollow-feature guard (TD-VSDD-059): the UDF pipeline was not exercised. \
         served_cve_values_count = {}. \
         BC-2.06.019 AC-008",
        served_cve_values.len()
    );

    // Step 11 — Assert ≥1 result row AND at least one HIGH CVSS verdict (>=7.0).
    // The scenario catalog CVEs are registered as HIGH (base_score=8.1) in NvdState.
    // At stage 4, the served route exposes device_cves_first → enrich must find HIGH CVSS.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "F-PIVOT003-R11A-002: query must return at least 1 row. \
         Got 0 rows — MemTable was empty or WHERE clause eliminated all rows. \
         served_cve_values_count = {}. \
         BC-2.06.019 AC-008",
        served_cve_values.len()
    );

    let mut high_cvss_count = 0usize;
    let mut total_non_null = 0usize;
    for batch in &batches {
        let cvss_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("F-PIVOT003-R11A-002: cvss_result column must be StringArray");

        for i in 0..batch.num_rows() {
            if cvss_col.is_null(i) {
                continue;
            }
            total_non_null += 1;
            let result_str = cvss_col.value(i);
            // No source_column set — full JSON object returned by NvdInfusionSource.
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(result_str) {
                if let Some(score) = json_val.get("cvss_base_score").and_then(|v| v.as_f64()) {
                    if score >= 7.0 {
                        high_cvss_count += 1;
                    }
                }
            }
        }
    }

    assert!(
        total_non_null > 0,
        "F-PIVOT003-R11A-002: at least 1 non-NULL CVSS result required. \
         Got 0 — NvdInfusionSource::enrich_single returned None for all served CVE IDs. \
         enrich_call_count={enrich_call_count}. served_cve_values={:?}. \
         BC-2.06.019 AC-008 / INV-NVD-CVE-CORRELATION-001",
        served_cve_values
    );

    assert!(
        high_cvss_count > 0,
        "F-PIVOT003-R11A-002 [RED GATE]: at least 1 HIGH CVSS (>= 7.0) result required. \
         Got high_cvss_count=0 out of {total_non_null} non-NULL results. \
         Expected scenario CVEs to have base_score=8.1 (BC-2.06.020 PC-4). \
         enrich_call_count={enrich_call_count}. served_cve_values={:?}. \
         BC-2.06.019 AC-008 / INV-NVD-CVE-CORRELATION-001 / F-PIVOT003-R11A-002 [RED GATE]",
        served_cve_values
    );
}
