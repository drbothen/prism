//! BC-2.06.019 + BC-2.06.020 — genuine end-to-end enrich pipeline tests.
//!
//! **F-PIVOT003-R3-001 closure** — these tests execute the ACTUAL PrismQL enrich pipeline
//! (DataFusion `SessionContext` + `InfusionAsyncUdf` + `register_infusion_udfs`) against
//! scenario-seeded DTU clone data, proving that the enrichment UDF wiring is non-hollow.
//!
//! ## What makes these tests genuine (not data-layer assertions)
//!
//! The tests wire an `InfusionSource` implementation that delegates to the real
//! DTU clone state (`ThreatIntelState::lookup_fixture` / `NvdState::lookup_and_count`),
//! register it as a DataFusion async scalar UDF via `register_infusion_udfs`, build a
//! `MemTable` from scenario clone `generated_records`, and execute a SQL query through
//! the full DataFusion execution engine. The UDF calls `InfusionAsyncUdf::invoke_async_with_args`
//! which in turn calls `enrich_single` on the source — NOT manually calling lookup functions.
//!
//! ## Architectural note: why SQL not PrismQL pipe
//!
//! The PrismQL `| enrich` pipe stage is translated to SQL at the materialization layer
//! (`execute_against_session` / `Ast::Pipe` branch) but the ephemeral materialization pipeline
//! (`Ast::Filter | Ast::Pipe`) returns pre-collected batches directly WITHOUT executing SQL
//! (see `materialization.rs` line 874). DataFusion async UDFs are only invoked when the query
//! is executed as SQL against the `SessionContext`. Therefore, genuine E2E proof requires
//! constructing the `SessionContext` + MemTable + SQL path directly — which is exactly what
//! `prism-query`'s existing `bc_2_19_001_plugin_udf_registration_test.rs` does for unit-level
//! UDF testing, and what this test does at the scenario-data level.
//!
//! ## Tests
//!
//! - `test_BC_2_06_019_enrich_pipeline_e2e_threatintel_pivot_executes_udf_and_returns_malicious`:
//!   AC-007 — executes `threat_is_known_malicious_udf(ioc_value)` SQL against Cyberint alert
//!   records flattened from `iocs[].value`. Asserts ≥1 row returns a Malicious verdict JSON.
//!
//! - `test_BC_2_06_019_enrich_pipeline_e2e_nvd_pivot_executes_udf_and_returns_high_cvss`:
//!   AC-008 — executes `nvd_cvss_udf(device_cves_first)` SQL against Armis device records.
//!   Asserts ≥1 row returns a HIGH CVSS JSON with `cvss_base_score >= 7.0`.
//!
//! ## BC traceability
//!
//! AC-007: BC-2.06.019 / BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001
//! AC-008: BC-2.06.019 PC-2 / BC-2.06.020 INV-NVD-CVE-CORRELATION-001
//!
//! ## TD-VSDD-059 (paper-fix guard)
//!
//! These tests FAIL if:
//! (a) `register_infusion_udfs` is not wired (UDF not found in SessionContext), OR
//! (b) `InfusionAsyncUdf::invoke_async_with_args` is a stub that returns None/empty, OR
//! (c) The `InfusionSource::enrich_single` is never called (hollow feature), OR
//! (d) The MemTable contains no rows with IOC/CVE values (AC-002/AC-008 data-layer failure),
//!     which would cause the assertions to fire a vacuous-pass guard.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::collections::HashMap;
use std::fmt;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use chrono::Utc;
use datafusion::arrow::array::{Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, OrgId,
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
// ThreatIntelInfusionSource — wraps ThreatIntelState, implements InfusionSource
// ---------------------------------------------------------------------------

/// `InfusionSource` implementation backed by `ThreatIntelState::lookup_fixture`.
///
/// Drives the genuine enrich pipeline: `InfusionAsyncUdf::invoke_async_with_args` calls
/// `enrich_single(ioc_value, ...)` which delegates to `ThreatIntelState::lookup_fixture`.
/// Returns the DTU fixture response JSON for Malicious, Benign, or Unknown keys.
///
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
        let key = self.state.lookup_fixture(input)?;
        // Return the fixture response shape matching DTU routes/lookup.rs ip_fixture_response
        // (or hash_fixture_response for hash IOCs — same field names, different key).
        use prism_dtu_threatintel::types::FixtureKey;
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
// NvdInfusionSource — wraps NvdState, implements InfusionSource
// ---------------------------------------------------------------------------

/// `InfusionSource` implementation backed by `NvdState::lookup_and_count`.
///
/// Drives the genuine enrich pipeline: `InfusionAsyncUdf::invoke_async_with_args` calls
/// `enrich_single(cve_id, ...)` which delegates to `NvdState::lookup_and_count`.
/// Returns CVSS score JSON for known CVEs, `None` for unknown.
///
/// Call counter proves `enrich_single` was actually invoked (TD-VSDD-059 hollow-feature guard).
struct NvdInfusionSource {
    state: Arc<prism_dtu_nvd::state::NvdState>,
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
        // Extract CVSS v3.1 metrics from the CveRecord (BC-2.06.020 PC-4).
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
// Test 10 — AC-007 genuine E2E: ThreatIntel enrich UDF pipeline
// ---------------------------------------------------------------------------

/// Test 10 — AC-007 genuine end-to-end: ThreatIntel enrich UDF pipeline returns Malicious verdict.
///
/// **F-PIVOT003-R3-001 closure** — this test proves the PrismQL enrich pipeline executes the
/// real `InfusionAsyncUdf::invoke_async_with_args` → `InfusionSource::enrich_single` chain
/// against scenario-seeded Cyberint alert records.
///
/// Pipeline driven:
/// 1. `register_infusion_udfs` registers `ThreatIntelInfusionSource` as DataFusion async UDF.
/// 2. MemTable is built from Cyberint alert `generated_records`, flattening `iocs[].value`
///    array entries into individual `(ioc_value TEXT)` rows.
/// 3. SQL: `SELECT threat_is_known_malicious_udf(ioc_value) AS verdict FROM cyberint_alerts_flat`
///    executes through the DataFusion engine — the async UDF is called for each row.
/// 4. Assert: ≥1 verdict row is non-NULL AND the response JSON contains
///    `"threat_is_known_malicious":true` (scenario IOC → Malicious).
/// 5. Assert: enrich_single call_count > 0 (UDF was actually invoked — hollow-feature guard).
///
/// LOAD-BEARING: this test FAILS if:
/// (a) The UDF is not registered (`register_infusion_udfs` broken), OR
/// (b) `invoke_async_with_args` is a stub returning None/empty, OR
/// (c) `enrich_single` is never called (call_count == 0), OR
/// (d) The Cyberint clone produced no alert records with IOC values (AC-002 failure).
///
/// Traces to:
///   BC-2.06.019 (AC-007: iocs[].value canonical pivot field)
///   BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 (scenario IOCs resolve as Malicious)
///   F-PIVOT003-R3-001 (closing finding: genuine pipeline execution required)
#[tokio::test]
async fn test_BC_2_06_019_enrich_pipeline_e2e_threatintel_pivot_executes_udf_and_returns_malicious()
{
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared scenario entity catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "Test 10 prereq: catalog.ioc_hashes must be non-empty (scenario seeding failure). \
         F-PIVOT003-R3-001 / AC-007"
    );

    // Step 2 — Build ThreatIntelClone with scenario IOCs pre-populated as Malicious.
    let threatintel_clone = prism_dtu_threatintel::ThreatIntelClone::new_with_scenario(&catalog);

    // Step 3 — Build CyberintClone with scenario IOC stamps on alert records.
    let scenario_start: i64 = Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&Utc);

    let cyberint_clone = prism_dtu_cyberint::CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("CyberintClone::new_with_scenario must succeed");

    // Step 4 — Extract all IOC values from alert records' `iocs[].value` arrays.
    // This mirrors the iocs[].value array field-path extraction from BC-2.06.019 AC-007.
    // We flatten each ioc entry from the iocs[] array into individual rows for the MemTable.
    let mut ioc_values: Vec<String> = Vec::new();
    for rec in &cyberint_clone.state.generated_records {
        let is_alert = rec.get("_surface").and_then(|v| v.as_str()) == Some("alert");
        if !is_alert {
            continue;
        }
        // iocs[] array form (BC-2.06.019 AC-007 canonical field).
        if let Some(iocs_arr) = rec.get("iocs").and_then(|v| v.as_array()) {
            for ioc_entry in iocs_arr {
                if let Some(val) = ioc_entry.get("value").and_then(|v| v.as_str()) {
                    ioc_values.push(val.to_owned());
                }
            }
        }
        // Also collect singleton ioc.value for backward-compat coverage.
        if let Some(val) = rec
            .get("ioc")
            .and_then(|ioc| ioc.get("value"))
            .and_then(|v| v.as_str())
        {
            ioc_values.push(val.to_owned());
        }
    }

    assert!(
        !ioc_values.is_empty(),
        "Test 10 prereq: no IOC values found in Cyberint alert records (seed={seed}). \
         AC-002 must stamp catalog.ioc_hashes onto alert records via iocs[].value. \
         BC-2.06.019 AC-007 / F-PIVOT003-R3-001 [VACUOUS PASS GUARD]"
    );

    // Step 5 — Build the DataFusion SessionContext with the ThreatIntel enrichment UDF.
    let ctx = build_session_context(QUERY_MEMORY_POOL_BYTES)
        .expect("Test 10: build_session_context must succeed");

    let call_count = Arc::new(AtomicUsize::new(0));
    let ti_source: Arc<dyn InfusionSource> = Arc::new(ThreatIntelInfusionSource {
        state: Arc::clone(&threatintel_clone.state),
        call_count: Arc::clone(&call_count),
    });

    // UDF name: "threat_is_known_malicious_udf" — distinct from the field name to
    // avoid confusion with future DataFusion column names.
    // source_column = "threat_is_known_malicious" — projects the boolean field from the JSON object.
    let descriptor = InfusionUdfDescriptor::new(
        "threat_is_known_malicious_udf",
        "ip",
        "string",
        "threatintel_scenario",
        Arc::clone(&ti_source),
        Some("threat_is_known_malicious".to_string()),
        3600,
        "",
    );

    register_infusion_udfs(&ctx, vec![descriptor])
        .expect("Test 10: register_infusion_udfs must succeed");

    // Step 6 — Build MemTable from the IOC values.
    // Each row is one IOC value extracted from iocs[].value.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_value",
        DataType::Utf8,
        true,
    )]));
    let arr = StringArray::from(ioc_values.clone());
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("Test 10: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("Test 10: MemTable construction must succeed");
    ctx.register_table("cyberint_alerts_flat", Arc::new(table))
        .expect("Test 10: register_table must succeed");

    // Step 7 — Execute the enrich SQL query through DataFusion.
    // This is the canonical ThreatIntel pivot query equivalent (BC-2.06.019 AC-007):
    //   FROM cyberint_alerts | enrich threat_intel(iocs[].value) | where threat_is_known_malicious = true
    // Translated to SQL using the registered async UDF:
    let df = ctx
        .sql(
            "SELECT ioc_value, threat_is_known_malicious_udf(ioc_value) AS threat_verdict \
             FROM cyberint_alerts_flat \
             WHERE ioc_value IS NOT NULL",
        )
        .await
        .expect(
            "Test 10: SQL with threat_is_known_malicious_udf must parse and plan. \
             FAIL = UDF not registered or signature mismatch. \
             F-PIVOT003-R3-001",
        );

    let batches = df.collect().await.expect(
        "Test 10: threat_is_known_malicious_udf execution must succeed. \
             FAIL = InfusionAsyncUdf::invoke_async_with_args returned an error. \
             F-PIVOT003-R3-001",
    );

    // Step 8 — Assert enrich_single was actually invoked (hollow-feature guard).
    let enrich_call_count = call_count.load(Ordering::SeqCst);
    assert!(
        enrich_call_count > 0,
        "Test 10 F-PIVOT003-R3-001: enrich_single call_count must be > 0 after SQL execution. \
         Got 0 — InfusionAsyncUdf::invoke_async_with_args did NOT call the source. \
         This is the hollow-feature guard (TD-VSDD-059): the UDF pipeline was not exercised. \
         ioc_values_in_table = {}. \
         BC-2.06.019 AC-007 / F-PIVOT003-R3-001",
        ioc_values.len()
    );

    // Step 9 — Assert ≥1 result row AND at least one Malicious verdict.
    // The scenario IOC hashes are registered as Malicious in ThreatIntelState.
    // At least one IOC value in the alert records must be a catalog hash → verdict = true.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "Test 10 F-PIVOT003-R3-001: query must return at least 1 row. \
         Got 0 rows — MemTable was empty or WHERE clause eliminated all rows. \
         ioc_values_in_table = {}. \
         BC-2.06.019 AC-007",
        ioc_values.len()
    );

    // Collect all non-NULL verdict strings and check for Malicious.
    // The UDF returns a JSON string with threat_is_known_malicious projected.
    // For a Malicious IOC: source_column projection extracts `true` (bool → "true" string).
    let mut malicious_count = 0usize;
    let mut total_non_null = 0usize;
    for batch in &batches {
        let verdict_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Test 10: threat_verdict column must be StringArray");

        for i in 0..batch.num_rows() {
            if !verdict_col.is_null(i) {
                total_non_null += 1;
                let verdict = verdict_col.value(i);
                // source_column = "threat_is_known_malicious" projects the bool field.
                // InfusionAsyncUdf::project_value serializes bool true as the string "true".
                if verdict == "true" {
                    malicious_count += 1;
                }
            }
        }
    }

    assert!(
        total_non_null > 0,
        "Test 10 F-PIVOT003-R3-001: at least 1 non-NULL verdict row required. \
         Got 0 — enrich_single returned None for all IOCs (no catalog IOCs found in state). \
         enrich_call_count={enrich_call_count}. ioc_values={:?}. \
         BC-2.06.019 AC-007 / INV-THREATINTEL-IOC-CORRELATION-001",
        ioc_values
    );

    assert!(
        malicious_count > 0,
        "Test 10 F-PIVOT003-R3-001: at least 1 Malicious verdict required. \
         Got malicious_count=0 out of {total_non_null} non-NULL verdicts. \
         Expected scenario catalog IOC hash to resolve as Malicious via ThreatIntelState. \
         enrich_call_count={enrich_call_count}. ioc_values={:?}. \
         BC-2.06.019 AC-007 / INV-THREATINTEL-IOC-CORRELATION-001 / F-PIVOT003-R3-001 [RED GATE]",
        ioc_values
    );

    // Step 10 — F-PIVOT003-R7B-001: AC-007 conjunction — assert threat_score >= 75.
    // Register a second UDF using the same ThreatIntelInfusionSource, source_column = "threat_score".
    // ThreatIntelInfusionSource returns threat_score: 85 for Malicious keys (>= 75 per AC-007).
    // Sibling pattern: enrichment_pivot_002_tests.rs:341 / bc_2_06_020_enrichment_correlation.rs:388.
    let score_call_count = Arc::new(AtomicUsize::new(0));
    let ti_score_source: Arc<dyn InfusionSource> = Arc::new(ThreatIntelInfusionSource {
        state: Arc::clone(&threatintel_clone.state),
        call_count: Arc::clone(&score_call_count),
    });

    let score_descriptor = InfusionUdfDescriptor::new(
        "threat_score_udf",
        "ip",
        "string",
        "threatintel_scenario_score",
        Arc::clone(&ti_score_source),
        Some("threat_score".to_string()),
        3600,
        "",
    );

    register_infusion_udfs(&ctx, vec![score_descriptor])
        .expect("Test 10: register_infusion_udfs for threat_score_udf must succeed");

    let score_df = ctx
        .sql(
            "SELECT ioc_value, threat_score_udf(ioc_value) AS score_result \
             FROM cyberint_alerts_flat \
             WHERE ioc_value IS NOT NULL",
        )
        .await
        .expect(
            "Test 10 F-PIVOT003-R7B-001: SQL with threat_score_udf must parse and plan. \
             FAIL = UDF not registered or signature mismatch.",
        );

    let score_batches = score_df
        .collect()
        .await
        .expect("Test 10 F-PIVOT003-R7B-001: threat_score_udf execution must succeed.");

    // At least one row must have threat_score >= 75 (Malicious = 85).
    let mut score_above_threshold = 0usize;
    for batch in &score_batches {
        let score_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Test 10: score_result column must be StringArray");
        for i in 0..batch.num_rows() {
            if score_col.is_null(i) {
                continue;
            }
            let score_str = score_col.value(i);
            // source_column = "threat_score" projects the u64 field; serialized as a numeric string.
            if let Ok(score) = score_str.parse::<u64>() {
                if score >= 75 {
                    score_above_threshold += 1;
                }
            }
        }
    }

    assert!(
        score_above_threshold > 0,
        "Test 10 F-PIVOT003-R7B-001: at least 1 threat_score >= 75 result required. \
         Got score_above_threshold=0. ThreatIntelInfusionSource returns threat_score=85 for \
         Malicious keys (>= 75 per AC-007). \
         BC-2.06.019 AC-007 conjunction: threat_is_known_malicious=true AND threat_score >= 75. \
         BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / F-PIVOT003-R7B-001 [RED GATE]"
    );
}

// ---------------------------------------------------------------------------
// Test 11 — AC-008 genuine E2E: NVD enrich UDF pipeline
// ---------------------------------------------------------------------------

/// Test 11 — AC-008 genuine end-to-end: NVD enrich UDF pipeline returns HIGH CVSS verdict.
///
/// **F-PIVOT003-R3-001 closure** — this test proves the PrismQL enrich pipeline executes the
/// real `InfusionAsyncUdf::invoke_async_with_args` → `InfusionSource::enrich_single` chain
/// against scenario-seeded Armis device records.
///
/// Pipeline driven:
/// 1. `register_infusion_udfs` registers `NvdInfusionSource` as DataFusion async UDF.
/// 2. MemTable is built from Armis device `generated_records` that carry `device_cves_first`.
/// 3. SQL: `SELECT nvd_cvss_udf(device_cves_first) AS cvss_result FROM armis_devices_flat`
///    executes through the DataFusion engine — the async UDF is called for each row.
/// 4. Assert: ≥1 result row contains `"cvss_base_score"` with value >= 7.0 (HIGH severity).
/// 5. Assert: enrich_single call_count > 0 (UDF was actually invoked — hollow-feature guard).
///
/// LOAD-BEARING: this test FAILS if:
/// (a) The UDF is not registered, OR
/// (b) `invoke_async_with_args` is a stub returning None/empty, OR
/// (c) `enrich_single` is never called (call_count == 0), OR
/// (d) The Armis clone produced no device records with device_cves_first (AC-008 failure).
///
/// Traces to:
///   BC-2.06.019 PC-2 (device_cves visible at Containment stage)
///   BC-2.06.020 INV-NVD-CVE-CORRELATION-001 (scenario CVEs appear with HIGH CVSS)
///   U17/Ruling 1b (device_cves_first = catalog.device_cves[0] scalar projection)
///   F-PIVOT003-R3-001 (closing finding: genuine pipeline execution required)
#[tokio::test]
async fn test_BC_2_06_019_enrich_pipeline_e2e_nvd_pivot_executes_udf_and_returns_high_cvss() {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_nvd::{
        state::NvdState,
        types::{CveMetrics, CveRecord, CvssData, CvssMetricV31, LangValue},
    };

    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared scenario entity catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.device_cves.is_empty(),
        "Test 11 prereq: catalog.device_cves must be non-empty (scenario seeding failure). \
         F-PIVOT003-R3-001 / AC-008"
    );

    // Step 2 — Build ArmisClone via the production constructor.
    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("ArmisClone::new_with_scenario must succeed");

    // Step 3 — Collect device records with device_cves_first.
    let device_cve_values: Vec<String> = armis_clone
        .state
        .generated_records
        .iter()
        .filter_map(|rec| {
            rec.get("device_cves_first")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert!(
        !device_cve_values.is_empty(),
        "Test 11 prereq: no device records with 'device_cves_first' found in ArmisClone \
         state.generated_records (seed={seed}). AC-008 / U17/Ruling 1b requires \
         ArmisClone::new_with_scenario to stamp device_cves_first on CompromisedEndpoint \
         asset records. catalog.device_cves={:?}. \
         BC-2.06.019 PC-2 / F-PIVOT003-R3-001 [VACUOUS PASS GUARD]",
        catalog.device_cves
    );

    // Step 4 — Build NvdState with scenario CVEs pre-populated as HIGH CVSS.
    // Mirrors NvdClone::new_with_scenario but without loading fixtures from disk.
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

    // Step 5 — Build the DataFusion SessionContext with the NVD enrichment UDF.
    let ctx = build_session_context(QUERY_MEMORY_POOL_BYTES)
        .expect("Test 11: build_session_context must succeed");

    let call_count = Arc::new(AtomicUsize::new(0));
    let nvd_source: Arc<dyn InfusionSource> = Arc::new(NvdInfusionSource {
        state: Arc::clone(&nvd_state),
        call_count: Arc::clone(&call_count),
    });

    // UDF name: "nvd_cvss_udf" — distinct to avoid confusion.
    // No source_column: returns the full JSON object so the test can parse both score and severity.
    let descriptor = InfusionUdfDescriptor::new(
        "nvd_cvss_udf",
        "cve_id",
        "string",
        "nvd_scenario",
        Arc::clone(&nvd_source),
        None, // no source_column: return full JSON for assertion
        3600,
        "",
    );

    register_infusion_udfs(&ctx, vec![descriptor])
        .expect("Test 11: register_infusion_udfs must succeed");

    // Step 6 — Build MemTable from device CVE values.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "device_cves_first",
        DataType::Utf8,
        true,
    )]));
    let arr = StringArray::from(device_cve_values.clone());
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("Test 11: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("Test 11: MemTable construction must succeed");
    ctx.register_table("armis_devices_flat", Arc::new(table))
        .expect("Test 11: register_table must succeed");

    // Step 7 — Execute the enrich SQL query through DataFusion.
    // This is the canonical NVD pivot query equivalent (BC-2.06.019 AC-008):
    //   FROM armis_devices | where has device_cves_first | enrich nvd(device_cves_first) | where cvss_base_score >= 7.0
    // Translated to SQL using the registered async UDF:
    let df = ctx
        .sql(
            "SELECT device_cves_first, nvd_cvss_udf(device_cves_first) AS cvss_result \
             FROM armis_devices_flat \
             WHERE device_cves_first IS NOT NULL",
        )
        .await
        .expect(
            "Test 11: SQL with nvd_cvss_udf must parse and plan. \
             FAIL = UDF not registered or signature mismatch. \
             F-PIVOT003-R3-001",
        );

    let batches = df.collect().await.expect(
        "Test 11: nvd_cvss_udf execution must succeed. \
             FAIL = InfusionAsyncUdf::invoke_async_with_args returned an error. \
             F-PIVOT003-R3-001",
    );

    // Step 8 — Assert enrich_single was actually invoked (hollow-feature guard).
    let enrich_call_count = call_count.load(Ordering::SeqCst);
    assert!(
        enrich_call_count > 0,
        "Test 11 F-PIVOT003-R3-001: enrich_single call_count must be > 0 after SQL execution. \
         Got 0 — InfusionAsyncUdf::invoke_async_with_args did NOT call the source. \
         This is the hollow-feature guard (TD-VSDD-059): the UDF pipeline was not exercised. \
         device_cve_values_in_table = {}. \
         BC-2.06.019 AC-008 / F-PIVOT003-R3-001",
        device_cve_values.len()
    );

    // Step 9 — Assert ≥1 result row AND at least one HIGH CVSS verdict.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "Test 11 F-PIVOT003-R3-001: query must return at least 1 row. \
         Got 0 rows — MemTable was empty or WHERE clause eliminated all rows. \
         device_cve_values_in_table = {}. \
         BC-2.06.019 AC-008",
        device_cve_values.len()
    );

    // Parse cvss_result JSON strings and verify >= 7.0 base_score.
    let mut high_cvss_count = 0usize;
    let mut total_non_null = 0usize;
    for batch in &batches {
        let cvss_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Test 11: cvss_result column must be StringArray");

        for i in 0..batch.num_rows() {
            if cvss_col.is_null(i) {
                continue;
            }
            total_non_null += 1;
            let result_str = cvss_col.value(i);
            // Parse the JSON result from the NvdInfusionSource.
            // No source_column projection was set — full JSON object is serialized.
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
        "Test 11 F-PIVOT003-R3-001: at least 1 non-NULL CVSS result required. \
         Got 0 — NvdInfusionSource::enrich_single returned None for all CVE IDs. \
         enrich_call_count={enrich_call_count}. device_cve_values={:?}. \
         BC-2.06.019 AC-008 / INV-NVD-CVE-CORRELATION-001",
        device_cve_values
    );

    assert!(
        high_cvss_count > 0,
        "Test 11 F-PIVOT003-R3-001: at least 1 HIGH CVSS (>= 7.0) result required. \
         Got high_cvss_count=0 out of {total_non_null} non-NULL results. \
         Expected scenario CVEs to have base_score=8.1 (BC-2.06.020 PC-4). \
         enrich_call_count={enrich_call_count}. device_cve_values={:?}. \
         BC-2.06.019 AC-008 / INV-NVD-CVE-CORRELATION-001 / F-PIVOT003-R3-001 [RED GATE]",
        device_cve_values
    );
}
