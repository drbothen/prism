//! F-PIVOT003-R10A-001 — Composed served-route → enrich pipeline test.
//!
//! This test closes the "seam" between served-route masking and enrichment UDF pipeline
//! by driving the FULL CHAIN in a single test:
//!
//!   1. Build `CyberintClone::new_with_scenario` at stage ≥ 3 (ioc_hashes=true).
//!   2. Build `ThreatIntelClone::new_with_scenario` with matching scenario catalog.
//!   3. Start the Cyberint DTU server.
//!   4. Fetch records THROUGH the served route `GET /api/v1/alerts`.
//!   5. Extract `iocs[].value` from the HTTP response body.
//!   6. Feed those values into the enrich UDF pipeline:
//!      `register_infusion_udfs` → DataFusion MemTable → SQL execution.
//!   7. Assert NON-EMPTY enriched results with `threat_is_known_malicious=true` verdict.
//!
//! This is the composed end-to-end test the adversary asked for at F-PIVOT003-R10A-001:
//! the canonical-pivot tests assert each link separately; THIS test drives the full chain.
//!
//! ## Why this is architecturally feasible
//!
//! The served-route output is `serde_json::Value` records. We extract IOC values from the
//! JSON body, build a MemTable, and run the DataFusion async UDF pipeline — the same
//! approach used in `bc_2_06_019_enrich_pipeline_e2e.rs` (Tests 10/11). The only
//! composition step is collecting IOC values from the HTTP response instead of from
//! `clone.state.generated_records` directly. This proves the StageMask filtering and
//! the enrichment pipeline are coherent end-to-end: at stage < 3, IOC records are
//! withheld and the MemTable would be empty; at stage ≥ 3, IOC records are served
//! and the enrich pipeline finds malicious verdicts.
//!
//! ## BC traceability
//!
//! BC-2.06.019 v1.13 PC-4 (Cyberint alerts IOC surface at stage ≥ 3)
//! BC-2.06.019 v1.13 AC-007 (iocs[].value canonical pivot field)
//! BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 (scenario IOCs resolve as Malicious)
//! F-PIVOT003-R10A-001 (closing finding: composed served-route → enrich test required)

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::{
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
use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
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
// ThreatIntelInfusionSource — inline (mirrors bc_2_06_019_enrich_pipeline_e2e.rs)
// ---------------------------------------------------------------------------

/// `InfusionSource` backed by `ThreatIntelState::lookup_fixture`.
///
/// Drives the genuine enrich pipeline: `InfusionAsyncUdf::invoke_async_with_args` calls
/// `enrich_single(ioc_value, ...)` → `ThreatIntelState::lookup_fixture`.
/// Call counter proves `enrich_single` was actually invoked (TD-VSDD-059 hollow-feature guard).
struct ThreatIntelInfusionSource {
    state: Arc<prism_dtu_threatintel::state::ThreatIntelState>,
    call_count: Arc<AtomicUsize>,
}

impl fmt::Debug for ThreatIntelInfusionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThreatIntelInfusionSource")
            .field("call_count", &self.call_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl InfusionSource for ThreatIntelInfusionSource {
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        use prism_dtu_threatintel::types::FixtureKey;
        let key = self.state.lookup_fixture(input)?;
        Some(match key {
            FixtureKey::Malicious => serde_json::json!({
                "lookup_value": input,
                "threat_score": 85,
                "threat_is_known_malicious": true,
                "threat_sources": ["greynoise", "abuseipdb"]
            }),
            FixtureKey::Benign => serde_json::json!({
                "lookup_value": input,
                "threat_score": 5,
                "threat_is_known_malicious": false,
                "threat_sources": ["greynoise"]
            }),
            FixtureKey::Unknown => serde_json::json!({
                "lookup_value": input,
                "threat_score": 0,
                "threat_is_known_malicious": false,
                "threat_sources": []
            }),
        })
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// F-PIVOT003-R10A-001 composed test
// ---------------------------------------------------------------------------

/// Composed test — F-PIVOT003-R10A-001: served-route → enrich pipeline end-to-end.
///
/// Drives the FULL CHAIN:
///   CyberintClone::new_with_scenario (stage ≥ 3, ioc_hashes=true)
///   → GET /api/v1/alerts (HTTP served route, StageMask applied)
///   → extract iocs[].value from response body
///   → register_infusion_udfs (ThreatIntelInfusionSource)
///   → DataFusion MemTable → SQL execution
///   → assert threat_is_known_malicious=true for ≥ 1 row
///
/// This closes the seam between:
///   - Stagemask tests (bc_2_06_019_ioc_stamping.rs Test 10): assert served-route
///     correctly filters/exposes IOC fields by stage.
///   - Enrich pipeline tests (bc_2_06_019_enrich_pipeline_e2e.rs Tests 10/11): assert
///     DataFusion UDF pipeline returns malicious verdicts from state.generated_records.
///
/// LOAD-BEARING: this test FAILS if:
/// (a) Served route withholds IOC records at stage ≥ 3 (StageMask regression), OR
/// (b) Served route does not include iocs[].value in the JSON response, OR
/// (c) `register_infusion_udfs` is not wired, OR
/// (d) `enrich_single` is never called (hollow-feature guard), OR
/// (e) Scenario IOCs are not registered as Malicious in ThreatIntelState.
///
/// BC-2.06.019 v1.13 PC-4 / AC-007 / BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001
/// F-PIVOT003-R10A-001
#[tokio::test]
async fn test_BC_2_06_019_served_route_to_enrich_pipeline_composed_full_chain() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared scenario entity catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "Prereq: catalog.ioc_hashes must be non-empty (scenario seeding failure). \
         F-PIVOT003-R10A-001 / AC-007"
    );

    // Step 2 — Build ThreatIntelClone with catalog IOCs pre-populated as Malicious.
    // new_with_scenario registers all catalog.ioc_hashes as FixtureKey::Malicious.
    let threatintel_clone = prism_dtu_threatintel::ThreatIntelClone::new_with_scenario(&catalog);

    // Step 3 — Build CyberintClone at stage ≥ 3 (ioc_hashes=true, Exfil stage).
    // scenario_start = now - 1000s → elapsed ≫ stage[3].threshold → StageMask.ioc_hashes=true.
    // (Default stage thresholds: [60, 180, 360, 600] → stage index ≥ 3 at elapsed ≥ 600s.)
    let scenario_start: i64 = Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&Utc);

    let mut cyberint_clone = prism_dtu_cyberint::CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("CyberintClone::new_with_scenario must succeed");

    // Register a known access_token so the alerts route accepts our requests.
    // Cyberint uses cookie auth (`access_token={token}`); new_with_scenario does not
    // register one by default so we inject it after construction.
    let access_token = "test-composed-pivot-token".to_owned();
    cyberint_clone
        .state
        .register_access_token(access_token.clone());

    // Step 4 — Start the Cyberint DTU server.
    cyberint_clone
        .start()
        .await
        .expect("CyberintClone::start() must succeed for served-route test");
    let base_url = cyberint_clone.base_url();

    // Step 5 — Fetch THROUGH the served route GET /api/v1/alerts.
    // This exercises the StageMask projection: at stage ≥ 3, IOC records are exposed.
    // The response is the ACTUAL route output (StageMask applied by the route handler).
    let client = prism_dtu_common::build_test_client();

    // Cookie header: `access_token={token}` (ADR-031 §D3-a, extract_access_token in alerts.rs).
    let cookie_header = format!("access_token={access_token}");

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", cookie_header)
        .send()
        .await
        .expect(
            "F-PIVOT003-R10A-001: GET /api/v1/alerts must reach the CyberintClone server. \
             FAIL = server did not start or is unreachable.",
        );

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-PIVOT003-R10A-001: GET /api/v1/alerts must return HTTP 200. \
         Got: {}",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("F-PIVOT003-R10A-001: /api/v1/alerts response must be valid JSON");

    // Step 6 — Extract iocs[].value from the HTTP response.
    // This mirrors the canonical PrismQL query field path: iocs[].value (AC-007).
    // The served-route JSON body has `data: [{alert records}]`.
    let data = body["data"].as_array().cloned().unwrap_or_default();

    let mut served_ioc_values: Vec<String> = Vec::new();
    for rec in &data {
        // iocs[] array form (canonical pivot field — BC-2.06.019 v1.13 AC-007).
        if let Some(iocs_arr) = rec.get("iocs").and_then(|v| v.as_array()) {
            for ioc_entry in iocs_arr {
                if let Some(val) = ioc_entry.get("value").and_then(|v| v.as_str()) {
                    served_ioc_values.push(val.to_owned());
                }
            }
        }
        // Also collect singleton ioc.value for backward-compat coverage.
        if let Some(val) = rec
            .get("ioc")
            .and_then(|ioc| ioc.get("value"))
            .and_then(|v| v.as_str())
        {
            served_ioc_values.push(val.to_owned());
        }
    }

    // Stop the Cyberint server — we have what we need.
    cyberint_clone
        .stop()
        .await
        .expect("CyberintClone::stop() must succeed");

    // Vacuous-pass guard: the served route MUST have returned IOC values at stage ≥ 3.
    // If this fires, the StageMask ioc_hashes=true gate is broken (not serving IOC records
    // at Exfil stage), which is a served-route regression independent of enrichment.
    assert!(
        !served_ioc_values.is_empty(),
        "F-PIVOT003-R10A-001 [VACUOUS PASS GUARD]: served-route GET /api/v1/alerts at stage ≥ 3 \
         returned NO iocs[].value entries. response alert_count={}. \
         At stage ≥ 3 (scenario_start = now - 1000s), ioc_hashes=true — IOC records MUST be \
         served. If this fires: StageMask ioc_hashes gate is broken OR CyberintClone did not \
         stamp catalog IOC hashes on alert records. catalog.ioc_hashes={:?}. \
         BC-2.06.019 v1.13 PC-4 / AC-007",
        data.len(),
        catalog.ioc_hashes,
    );

    // Step 7 — Wire the DataFusion enrich UDF pipeline with ThreatIntelInfusionSource.
    let ctx = build_session_context(QUERY_MEMORY_POOL_BYTES)
        .expect("F-PIVOT003-R10A-001: build_session_context must succeed");

    let call_count = Arc::new(AtomicUsize::new(0));
    let ti_source: Arc<dyn InfusionSource> = Arc::new(ThreatIntelInfusionSource {
        state: Arc::clone(&threatintel_clone.state),
        call_count: Arc::clone(&call_count),
    });

    let descriptor = InfusionUdfDescriptor::new(
        "threat_is_known_malicious_udf",
        "ip",
        "string",
        "threatintel_scenario_composed",
        Arc::clone(&ti_source),
        Some("threat_is_known_malicious".to_string()),
        3600,
        "",
    );

    register_infusion_udfs(&ctx, vec![descriptor])
        .expect("F-PIVOT003-R10A-001: register_infusion_udfs must succeed");

    // Step 8 — Build MemTable from served IOC values (not from state.generated_records).
    // This is the key composition: we use the SERVED values, not the raw state.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_value",
        DataType::Utf8,
        true,
    )]));
    let arr = StringArray::from(served_ioc_values.clone());
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("F-PIVOT003-R10A-001: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("F-PIVOT003-R10A-001: MemTable construction must succeed");
    ctx.register_table("served_cyberint_iocs", Arc::new(table))
        .expect("F-PIVOT003-R10A-001: register_table must succeed");

    // Step 9 — Execute the enrich SQL through DataFusion.
    // Canonical pivot: FROM cyberint_alerts | enrich threat_intel(iocs[].value) | where threat_is_known_malicious
    // Translated to SQL with the registered async UDF:
    let df = ctx
        .sql(
            "SELECT ioc_value, threat_is_known_malicious_udf(ioc_value) AS threat_verdict \
             FROM served_cyberint_iocs \
             WHERE ioc_value IS NOT NULL",
        )
        .await
        .expect(
            "F-PIVOT003-R10A-001: SQL with threat_is_known_malicious_udf must parse and plan. \
             FAIL = UDF not registered or signature mismatch.",
        );

    let batches = df.collect().await.expect(
        "F-PIVOT003-R10A-001: threat_is_known_malicious_udf execution must succeed. \
         FAIL = InfusionAsyncUdf::invoke_async_with_args returned an error.",
    );

    // Step 10 — Assert enrich_single was actually invoked (hollow-feature guard).
    let enrich_call_count = call_count.load(Ordering::SeqCst);
    assert!(
        enrich_call_count > 0,
        "F-PIVOT003-R10A-001: enrich_single call_count must be > 0 after SQL execution. \
         Got 0 — InfusionAsyncUdf::invoke_async_with_args did NOT call the source. \
         This is the hollow-feature guard (TD-VSDD-059): the UDF pipeline was not exercised. \
         served_ioc_values_count = {}. \
         BC-2.06.019 v1.13 AC-007",
        served_ioc_values.len()
    );

    // Step 11 — Assert ≥ 1 row and ≥ 1 Malicious verdict.
    // The scenario catalog IOC hashes are registered as Malicious in ThreatIntelState.
    // At stage ≥ 3, the served route exposes those IOC values → enrich must find Malicious.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "F-PIVOT003-R10A-001: query must return at least 1 row. \
         Got 0 rows — MemTable was empty or WHERE clause eliminated all rows. \
         served_ioc_values_count = {}. \
         BC-2.06.019 v1.13 AC-007",
        served_ioc_values.len()
    );

    let mut malicious_count = 0usize;
    let mut total_non_null = 0usize;
    for batch in &batches {
        let verdict_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("F-PIVOT003-R10A-001: threat_verdict column must be StringArray");
        for i in 0..batch.num_rows() {
            if !verdict_col.is_null(i) {
                total_non_null += 1;
                if verdict_col.value(i) == "true" {
                    malicious_count += 1;
                }
            }
        }
    }

    assert!(
        total_non_null > 0,
        "F-PIVOT003-R10A-001: at least 1 non-NULL verdict row required. \
         Got 0 — enrich_single returned None for all served IOC values. \
         enrich_call_count={enrich_call_count}. served_ioc_values={:?}. \
         BC-2.06.019 v1.13 AC-007 / INV-THREATINTEL-IOC-CORRELATION-001",
        served_ioc_values
    );

    assert!(
        malicious_count > 0,
        "F-PIVOT003-R10A-001 [RED GATE]: at least 1 Malicious verdict required. \
         Got malicious_count=0 out of {total_non_null} non-NULL verdicts. \
         Expected: served IOC values from GET /api/v1/alerts (stage ≥ 3) include \
         catalog.ioc_hashes[0] which ThreatIntelState maps to FixtureKey::Malicious. \
         enrich_call_count={enrich_call_count}. served_ioc_values={:?}. \
         BC-2.06.019 v1.13 AC-007 / INV-THREATINTEL-IOC-CORRELATION-001 / F-PIVOT003-R10A-001",
        served_ioc_values
    );
}
