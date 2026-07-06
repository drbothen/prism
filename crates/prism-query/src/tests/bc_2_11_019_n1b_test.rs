//! Tests for S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B — BC-2.11.019.
//!
//! Finding N1-B: the plan-time enrichment gate (E-QUERY-039) must fire when a query
//! references an enrichment function name that is not a registered per-field UDF name.
//! `PrismError::EnrichUdfNotFound` and `EnrichUdfNotFoundDetails` are implemented in
//! `prism-core/src/error.rs`; `check_enrich_udf_availability` in `engine.rs` gates
//! all query modes (pipe, SQL, SqlPipe).
//!
//! Tests assert:
//! 1. Pipe-mode query `FROM cyberint_alerts | enrich threat_intel(iocs_value)` where
//!    `threat_intel` is an infusion_id (NOT a registered per-field UDF name) returns
//!    `Err(PrismError::EnrichUdfNotFound(_))` with Display containing "E-QUERY-039".
//! 2. SQL-mode `SELECT nvd(iocs_value) FROM cyberint_alerts` returns the same error.
//! 3. Gate ordering: table gate fires before the enrich gate (E-QUERY-037 before E-QUERY-039).
//! 4. Supplementary: `available_infusions` sorted, `did_you_mean` Levenshtein logic,
//!    wired-but-empty registry, C1+C2 GROUP BY / ORDER BY / JOIN ON coverage.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_019_n1b_infusion_id_as_udf_name | AC-N1B | BC-2.11.019 |
//! | test_bc_2_11_019_n1b_sql_path_infusion_id_as_udf_name | AC-N1B | BC-2.11.019 |

use std::sync::Arc;

use crate::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    table_registry::TableRegistry,
};
use prism_core::error::PrismError;
use prism_spec_engine::{
    infusion::{InfusionField, InfusionSpec, InfusionType},
    InfusionRegistry,
};

// ── Test fixture helpers ──────────────────────────────────────────────────────

/// Minimal no-op credential store for unit tests that don't exercise auth.
/// Mirrors the `NoopCs` pattern used in `alias_wiring_tests` in engine.rs.
struct NoopCs;

#[async_trait::async_trait]
impl prism_credentials::CredentialStore for NoopCs {
    async fn get(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, PrismError> {
        Ok(None)
    }
    async fn set(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
        _v: secrecy::SecretString,
    ) -> Result<(), PrismError> {
        Ok(())
    }
    async fn delete(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
    async fn list(
        &self,
        _t: &prism_core::OrgSlug,
    ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError> {
        Ok(vec![])
    }
    async fn exists(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
}

/// Build a `TableRegistry` with `cyberint_alerts` registered.
fn make_cyberint_table_registry() -> Arc<TableRegistry> {
    use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

    let registry = Arc::new(TableRegistry::new());
    let spec = SensorSpec::new(
        "cyberint",
        "Cyberint sensor",
        AuthType::ApiKey,
        "https://api.cyberint.com",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&spec)
        .expect("register cyberint must not fail");
    registry
}

/// Build an `InfusionRegistry` with `threat_intel` infusion
/// (infusion_id = `threat_intel`, per-field UDF names: `threat_score`,
/// `threat_is_known_malicious`, `threat_sources`).
///
/// NOTE: `threat_intel` is the infusion_id — NOT a UDF name. The registered UDF names
/// are the per-field names. A query using `enrich threat_intel(col)` calls a name
/// that is NOT in `udf_to_infusion` (only the per-field names are keys there).
fn make_threat_intel_infusion_registry() -> Arc<InfusionRegistry> {
    let registry = InfusionRegistry::new();
    let spec = InfusionSpec::new(
        "threat_intel",
        "ThreatIntel enrichment",
        InfusionType::LocalLookup,
        vec![
            InfusionField::new("threat_score", "iocs_value_first", "string", "float64"),
            InfusionField::new(
                "threat_is_known_malicious",
                "iocs_value_first",
                "string",
                "bool",
            ),
            InfusionField::new("threat_sources", "iocs_value_first", "string", "string"),
        ],
        "/dev/null",
    );
    registry
        .load_spec(spec)
        .expect("threat_intel spec must load");
    Arc::new(registry)
}

/// Build a `QueryEngine` with:
/// - `cyberint_alerts` in the `TableRegistry`
/// - `threat_intel` infusion (per-field UDFs: `threat_score`, etc.) in `InfusionRegistry`
fn make_test_engine_threat_intel() -> QueryEngine {
    let registry = make_cyberint_table_registry();
    let infusion_registry = make_threat_intel_infusion_registry();

    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
    .with_infusion_registry(infusion_registry)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.11.019 AC-N1B — pipe path Red Gate test.
///
/// A pipe-mode query `FROM cyberint_alerts | enrich threat_intel(iocs_value)` where
/// `threat_intel` is an infusion_id (NOT a UDF name — the registered UDF names are
/// `threat_score`, `threat_is_known_malicious`, `threat_sources`) must return
/// `Err(PrismError::EnrichUdfNotFound(_))` at plan time.
///
/// Observable assertion: the error Display contains "E-QUERY-039".
/// Negative assertion: the error Display does NOT contain "Internal error" / "E-INT-001".
/// Positive assertion: the error Display contains at least one registered UDF name
/// (proving `available_infusions` is populated).
///
/// Load-bearing: removing `check_enrich_udf_availability` from the pipe-mode path in
/// `engine.rs` makes this test fail (no E-QUERY-039 produced).
#[tokio::test]
async fn test_bc_2_11_019_n1b_infusion_id_as_udf_name() {
    let engine = make_test_engine_threat_intel();

    // threat_intel is an infusion_id — NOT a per-field UDF name.
    // Registered UDF names: threat_score, threat_is_known_malicious, threat_sources.
    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich threat_intel(iocs_value)",
            QueryOptions::default(),
        )
        .await;

    // Must be an error (not Ok).
    assert!(
        result.is_err(),
        "BC-2.11.019 AC-N1B: query with unregistered enrichment UDF name 'threat_intel' \
         must return Err (regression: the plan-time E-QUERY-039 gate was removed). \
         Got Ok result."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary assertion: error must report E-QUERY-039.
    assert!(
        display.contains("E-QUERY-039"),
        "BC-2.11.019 AC-N1B: error Display must contain 'E-QUERY-039' (EnrichUdfNotFound). \
         Got: {display}"
    );

    // Negative control: must NOT be an opaque internal error.
    assert!(
        !display.contains("Internal error") && !display.contains("E-INT-001"),
        "BC-2.11.019 AC-N1B: error must NOT be an opaque 'Internal error' / 'E-INT-001'. \
         Got: {display}"
    );

    // available_infusions populated: at least one registered per-field UDF name in display.
    let has_registered_udf = display.contains("threat_score")
        || display.contains("threat_is_known_malicious")
        || display.contains("threat_sources");
    assert!(
        has_registered_udf,
        "BC-2.11.019 AC-N1B: error Display must include registered per-field UDF names \
         (available_infusions must be non-empty). \
         Expected one of: threat_score, threat_is_known_malicious, threat_sources. \
         Got: {display}"
    );
}

/// BC-2.11.019 AC-N1B — SQL path Red Gate test.
///
/// A SQL-mode query using an unregistered enrichment function name `nvd`
/// (which is an infusion_id, not a per-field UDF name) must return
/// `Err(PrismError::EnrichUdfNotFound(_))` at plan time.
///
/// Load-bearing: removing `check_enrich_udf_availability` from the SQL-mode path in
/// `engine.rs` makes this test fail (no E-QUERY-039 produced).
#[tokio::test]
async fn test_bc_2_11_019_n1b_sql_path_infusion_id_as_udf_name() {
    // Build engine with nvd infusion (per-field UDFs: cvss_base_score, cvss_severity, cvss_vector)
    let registry = InfusionRegistry::new();
    let nvd_spec = InfusionSpec::new(
        "nvd",
        "NVD CVSS enrichment",
        InfusionType::LocalLookup,
        vec![
            InfusionField::new("cvss_base_score", "cve_id", "string", "float64"),
            InfusionField::new("cvss_severity", "cve_id", "string", "string"),
            InfusionField::new("cvss_vector", "cve_id", "string", "string"),
        ],
        "/dev/null",
    );
    registry.load_spec(nvd_spec).expect("nvd spec must load");

    let table_registry = make_cyberint_table_registry();
    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(table_registry)
    .with_infusion_registry(Arc::new(registry));

    // `nvd` is an infusion_id — NOT a per-field UDF name.
    // SQL path: ScalarFunc::Unknown("nvd") in projection.
    let result = engine
        .execute(
            "SELECT nvd(iocs_value) FROM cyberint_alerts LIMIT 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "BC-2.11.019 AC-N1B SQL path: query with unregistered enrichment UDF 'nvd' in \
         SQL SELECT must return Err (regression: the plan-time E-QUERY-039 gate was removed). \
         Got Ok result."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "BC-2.11.019 AC-N1B SQL path: error Display must contain 'E-QUERY-039'. \
         Got: {display}"
    );

    assert!(
        !display.contains("Internal error") && !display.contains("E-INT-001"),
        "BC-2.11.019 AC-N1B SQL path: error must NOT be opaque internal error. \
         Got: {display}"
    );
}

// ── S-DEMO-FIDELITY-REMEDIATION-001 regression tests ─────────────────────────
// These tests are GREEN-GATE: they pass after the fix-burst and must remain green.
// TD-VSDD-059: load-bearing tests, not paper-fix closures.

/// HIGH-001 regression guard — gate ordering: table gate fires BEFORE enrich gate.
///
/// BC-2.11.019 §Gate ordering (enrich-LAST): E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
///
/// A query referencing a non-existent table AND an invalid enrichment function name
/// MUST return E-QUERY-037 (TableNotAvailable), NOT E-QUERY-039 (EnrichUdfNotFound).
/// The table gate must fire first and short-circuit before the enrich gate runs.
///
/// Regression: prior to HIGH-001 fix, enrich gate ran FIRST and returned E-QUERY-039
/// even when the table itself didn't exist.
#[tokio::test]
async fn test_high001_gate_ordering_table_error_before_enrich_error() {
    let engine = make_test_engine_threat_intel();

    // `nonexistent_table` is not registered in the TableRegistry.
    // `badname` is also not a valid per-field UDF.
    // Table gate must fire first → E-QUERY-037, NOT E-QUERY-039.
    let result = engine
        .execute(
            "FROM nonexistent_table | enrich badname(col)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "HIGH-001 regression: query with unknown table + unknown enrich must fail"
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-037"),
        "HIGH-001 regression: gate ordering must be table FIRST (E-QUERY-037), \
         not enrich (E-QUERY-039). Got: {display}"
    );

    assert!(
        !display.contains("E-QUERY-039"),
        "HIGH-001 regression: E-QUERY-039 must NOT appear when table gate should fire first. \
         Got: {display}"
    );
}

/// HIGH-003 regression guard — collect_unknown_scalar_from_predicate wiring check.
///
/// BC-2.11.019 §Precondition 1(b): the enrich gate must scan BOTH SELECT projections
/// AND the WHERE clause for `ScalarFunc::Unknown` names.
///
/// NOTE on WHERE-clause coverage: The PrismQL SQL parser's WHERE predicate grammar
/// (`build_predicate_parser`) does not include scalar function call syntax — a query like
/// `WHERE badudf(col) = 1` is rejected as E-QUERY-001 (parse error) before the enrich
/// gate runs. `collect_unknown_scalar_from_predicate` is defensive code for programmatic
/// AST construction (future parser extensions, macro-generated queries).
///
/// The direct unit tests for `collect_unknown_scalar_from_predicate` live in
/// `engine::enrich_gate_where_clause_unit_tests` in `engine.rs`, which construct
/// `Predicate::Compare { lhs: Expr::FuncCall(ScalarFunc::Unknown) }` directly.
///
/// This integration test verifies the end-to-end wiring: a SQL SELECT with an unknown
/// UDF in the projection (parseable path) must trigger E-QUERY-039, confirming that
/// `collect_unknown_scalar_from_expr` is correctly called from the Ast::Sql arm.
///
/// Regression: prior to HIGH-003 fix, the collect helpers were nested inside
/// `check_enrich_udf_availability` (not module-level), blocking direct unit testing.
/// After HIGH-003, both helpers are module-level private functions with direct unit tests.
#[tokio::test]
async fn test_high003_sql_select_unknown_scalar_triggers_enrich_error() {
    let engine = make_test_engine_threat_intel();

    // `badudf` is not a registered per-field UDF name.
    // It appears in the SELECT projection — the parseable path for SQL-mode enrichment.
    // This verifies the Ast::Sql arm in check_enrich_udf_availability calls
    // collect_unknown_scalar_from_expr for SELECT items.
    let result = engine
        .execute(
            "SELECT badudf(iocs_value) FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "HIGH-003 wiring check: SQL SELECT with unknown UDF 'badudf' in projection must fail. \
         Got Ok — collect_unknown_scalar_from_expr not wired for SELECT items."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "HIGH-003 wiring check: SQL SELECT with unknown UDF must trigger E-QUERY-039. \
         Got: {display}"
    );

    assert!(
        display.contains("badudf"),
        "HIGH-003 wiring check: error message must name the unregistered function 'badudf'. \
         Got: {display}"
    );
}

/// MED-001 regression guard — `available_infusions` in E-QUERY-039 is sorted + deduped.
///
/// error-taxonomy.md §E-QUERY-039 specifies the `available_infusions` field
/// MUST be sorted (lexicographic ascending). Without the sort fix, the field is built
/// from `HashSet` iteration order (non-deterministic), causing:
/// 1. Non-deterministic error messages (flaky tests, non-reproducible output).
/// 2. Drift from the E-QUERY-038 sibling which already sorts `available_columns`
///    (`check_column_availability` OBS-FRESH-1 fix).
///
/// Test protocol:
/// 1. Build an engine with 3 infusion UDFs registered in reverse-lex order:
///    `["zzz_score", "mmm_count", "aaa_flag"]` (names that have a clear sorted order).
/// 2. Execute a query with an unregistered UDF name to trigger E-QUERY-039.
/// 3. Extract the `available_infusions` field from the error details.
/// 4. Assert it matches the sorted order: `["aaa_flag", "mmm_count", "zzz_score"]`.
///
/// Load-bearing: this test fails if the sort/dedup is removed from
/// `check_enrich_udf_availability` (MED-001 fix in S-DEMO-FIDELITY-REMEDIATION-001).
#[tokio::test]
async fn test_med001_available_infusions_sorted_in_e_query_039_error() {
    use prism_core::error::PrismError;

    // Build a registry with 3 infusion specs whose per-field names are deliberately
    // inserted in REVERSE-LEX order to verify the sort fix is load-bearing.
    let infusion_registry = InfusionRegistry::new();

    // Per-field names in reverse-lex: "zzz_score", "mmm_count", "aaa_flag"
    // (the sorted result should be: ["aaa_flag", "mmm_count", "zzz_score"])
    for name in ["zzz_score", "mmm_count", "aaa_flag"] {
        let spec = InfusionSpec::new(
            name,
            "test infusion",
            InfusionType::LocalLookup,
            vec![InfusionField::new(name, "input_col", "string", "string")],
            "/dev/null",
        );
        infusion_registry
            .load_spec(spec)
            .unwrap_or_else(|e| panic!("MED-001: load_spec for '{name}' must not fail: {e}"));
    }

    let table_registry = make_cyberint_table_registry();
    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(table_registry)
    .with_infusion_registry(Arc::new(infusion_registry));

    // Execute query with an unregistered UDF name to trigger E-QUERY-039.
    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich unknown_udf(some_col)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-001: query with unregistered UDF must return Err; got Ok"
    );

    let err = result.unwrap_err();
    match err {
        PrismError::EnrichUdfNotFound(ref details) => {
            // MED-001 assertion: available_infusions must be sorted lexicographically.
            let infusions = &details.available_infusions;
            let mut expected_sorted = infusions.clone();
            expected_sorted.sort();
            expected_sorted.dedup();
            assert_eq!(
                infusions, &expected_sorted,
                "MED-001: available_infusions in E-QUERY-039 must be sorted (lexicographic). \
                 Got: {infusions:?}. Expected sorted: {expected_sorted:?}. \
                 This fails if sort+dedup is missing from check_enrich_udf_availability."
            );
            // Verify the sort produces the expected order.
            assert_eq!(
                infusions.as_slice(),
                &["aaa_flag", "mmm_count", "zzz_score"],
                "MED-001: available_infusions must be sorted ['aaa_flag', 'mmm_count', 'zzz_score']. \
                 Got: {infusions:?}"
            );
        }
        other => panic!("MED-001: expected PrismError::EnrichUdfNotFound, got: {other:?}"),
    }
}

/// HIGH-1 regression guard — SqlPipe SQL head scalar bypass.
///
/// BC-2.11.019 §Precondition 1(b): the enrich gate must scan BOTH pipe stages AND
/// the SQL head (SELECT projection + WHERE clause) for `ScalarFunc::Unknown` names.
///
/// Prior to the HIGH-1 fix, the `Ast::SqlPipe(spq)` arm in `check_enrich_udf_availability`
/// ONLY scanned `spq.stages` for `PipeStage::Enrich`. A SqlPipe query with an unknown
/// scalar in the SQL head projection (e.g. `SELECT cvss(col) FROM t | limit 10`) bypassed
/// the E-QUERY-039 gate, flowed to the emitter → DataFusion → opaque E-INT-001 (-32000).
///
/// Gate ordering verified: 037 (table) passes (cyberint_alerts is registered) →
/// 038 (column) passes (no resolved_spec_map in test mode → fail-open) →
/// 039 (enrich) FIRES on `cvss` which is NOT a registered per-field UDF name.
///
/// Load-bearing (HIGH-1 fix): prior to the fix, this test returned E-INT-001 (or Ok)
/// instead of E-QUERY-039 because the SqlPipe arm did not scan `spq.head.select.items`.
/// Post-fix: the SqlPipe arm scans the SQL head SELECT items via
/// `collect_unknown_scalar_from_expr`, catching `cvss` as an unknown scalar.
#[tokio::test]
async fn test_high1_sqlpipe_head_unknown_scalar_fires_e_query_039() {
    use prism_core::error::PrismError;

    // Engine has:
    //   - cyberint_alerts in TableRegistry (037 passes)
    //   - threat_intel infusion: per-field UDFs are threat_score, threat_is_known_malicious,
    //     threat_sources. `cvss` is NOT a registered UDF name.
    //   - No resolved_spec_map (038 fails open in test mode)
    let engine = make_test_engine_threat_intel();

    // SqlPipe query: SQL head has `cvss(iocs_value)` in SELECT projection.
    // `iocs_value` is a real column in the fixture table; `cvss` is an unknown scalar.
    // The `| limit 10` pipe stage has no enrich stage — all enrichment is in the SQL head.
    let result = engine
        .execute(
            "SELECT cvss(iocs_value) FROM cyberint_alerts | LIMIT 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "HIGH-1: SqlPipe query with unknown scalar 'cvss' in SQL head projection must return Err. \
         Prior to fix, the SqlPipe arm only scanned pipe stages (not the SQL head) → \
         the gate was bypassed → DataFusion emitted opaque E-INT-001. Got Ok."
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::EnrichUdfNotFound(ref details) => {
            // Primary: the unknown scalar name must be reported.
            assert_eq!(
                details.infusion, "cvss",
                "HIGH-1: infusion field must be 'cvss' (the unknown scalar). Got: {:?}",
                details.infusion
            );
            // available_infusions must list the registered per-field UDF names.
            let has_threat_intel_udfs = details
                .available_infusions
                .contains(&"threat_score".to_string())
                || details
                    .available_infusions
                    .contains(&"threat_is_known_malicious".to_string())
                || details
                    .available_infusions
                    .contains(&"threat_sources".to_string());
            assert!(
                has_threat_intel_udfs,
                "HIGH-1: available_infusions must include registered per-field UDF names. \
                 Got: {:?}",
                details.available_infusions
            );
        }
        other => {
            let display = format!("{other}");
            panic!(
                "HIGH-1: expected PrismError::EnrichUdfNotFound (E-QUERY-039) for SqlPipe head \
                 unknown scalar 'cvss'. Got: {other:?} | display: {display}. \
                 If display contains 'E-INT-001'/'Internal error', the HIGH-1 fix is missing \
                 (SqlPipe head not scanned in check_enrich_udf_availability)."
            );
        }
    }
}

/// EC-11-059 — wired-but-empty InfusionRegistry MUST fire E-QUERY-039 with available_infusions=[].
///
/// BC-2.11.019 §EC-11-059: When the infusion subsystem is wired (`Some(registry)`) but
/// contains zero loaded specs, any query using `enrich` MUST return E-QUERY-039 with
/// `available_infusions = []` (empty Vec) and `did_you_mean = None`.
///
/// This guards against the MED-1 regression where:
///   `if registered_names.is_empty() { return Ok(()); }`
/// silently passed an enrich query through to DataFusion, producing an opaque E-INT-001
/// instead of the actionable E-QUERY-039 with available_infusions=[].
///
/// Distinction from `registry == None` (subsystem not wired, legitimately skip gate):
///   - `None` registry → skip gate (test/MVP deployment without enrichment subsystem)
///   - `Some(empty)` registry → wired, zero infusions → MUST fire E-QUERY-039 ([])
///
/// Spec canonical test vector "no-infusions" (BC-2.11.019 §payload):
///   available_infusions: [] (always present, empty Vec)
///   did_you_mean: None (absent — no candidates within Levenshtein 3 of empty set)
#[tokio::test]
async fn test_ec_11_059_wired_empty_registry_fires_e_query_039_with_empty_available() {
    use prism_core::error::PrismError;

    // Construct engine with Some(InfusionRegistry::new()) — wired but ZERO load_spec calls.
    let table_registry = make_cyberint_table_registry();
    let empty_infusion_registry = Arc::new(InfusionRegistry::new());
    // Confirm no specs were loaded (guard against test-fixture drift).
    assert!(
        empty_infusion_registry.udf_descriptors().is_empty(),
        "EC-11-059 fixture: InfusionRegistry must have zero descriptors for this test"
    );

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(table_registry)
    .with_infusion_registry(empty_infusion_registry);

    // cyberint_alerts is registered — table gate passes. Only enrich gate fires.
    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich threat_score(iocs_value)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "EC-11-059: wired-but-empty InfusionRegistry + enrich query must return Err. \
         MED-1 regression: prior code returned Ok (skipped gate when is_empty). \
         Got Ok — the is_empty skip was not removed."
    );

    let err = result.unwrap_err();

    // Assert it is specifically EnrichUdfNotFound (not an opaque internal error).
    match err {
        PrismError::EnrichUdfNotFound(ref details) => {
            // available_infusions MUST be empty Vec (not missing — always present per §payload).
            assert_eq!(
                details.available_infusions,
                Vec::<String>::new(),
                "EC-11-059: available_infusions must be [] (empty Vec) when registry has zero specs. \
                 Got: {:?}",
                details.available_infusions
            );

            // did_you_mean MUST be None — no candidates to compute Levenshtein against.
            assert_eq!(
                details.did_you_mean, None,
                "EC-11-059: did_you_mean must be None when available_infusions is empty. \
                 Got: {:?}",
                details.did_you_mean
            );

            // infusion name in error must match the queried name.
            assert_eq!(
                details.infusion, "threat_score",
                "EC-11-059: infusion field must reflect the queried name 'threat_score'. \
                 Got: {:?}",
                details.infusion
            );
        }
        other => {
            let display = format!("{other}");
            panic!(
                "EC-11-059: expected PrismError::EnrichUdfNotFound, got: {other:?}. \
                 Display: {display}. \
                 If display contains 'E-INT-001' or 'Internal error', \
                 the MED-1 is_empty skip was not removed."
            );
        }
    }
}

// ── C1+C2 fix: E-QUERY-039 gate covers JOIN ON / GROUP BY / ORDER BY ──────────
//
// S-DEMO-FIDELITY-REMEDIATION-001 C1+C2: `check_enrich_udf_availability` previously
// scanned only SELECT + WHERE for `ScalarFunc::Unknown` names.  JOIN ON, GROUP BY, and
// ORDER BY positions were not walked, so an unregistered UDF in those positions bypassed
// the gate and reached DataFusion as an opaque E-INT-001.
//
// The fix introduces `collect_unknown_scalars_from_sql_query` (a canonical shared
// helper) that walks ALL scalar positions in a `SqlQuery`, and replaces the per-arm
// inline walks in `check_enrich_udf_availability` with calls to this helper.
//
// Note on parser reach: PrismQL's SQL parser accepts `func(col)` syntax in GROUP BY
// and ORDER BY but NOT in JOIN ON (which uses `col = col` equality, not function calls,
// in its grammar).  However the AST struct allows any `Expr` in `Join.on`, and
// programmatic/macro-generated AST construction (test fixtures, future parser
// extensions) can place `Expr::FuncCall(ScalarFunc::Unknown)` there.  The fix covers
// all AST positions so the gate is correct by construction regardless of parser version.
// Tests that exercise the JOIN ON and GROUP BY positions use the `check_enrich_udf_availability`
// function directly via AST fixtures (not the parser) to avoid parser grammar coupling.
//
// Integration tests (Sql + SqlPipe) via the execute() path cover the SELECT/pipe positions
// that are reachable by the live parser.  Unit tests cover JOIN ON + GROUP BY + ORDER BY
// via direct function calls.

/// C1 unit — `collect_unknown_scalars_from_sql_query` finds unknown scalar in GROUP BY.
///
/// Constructs a SqlQuery programmatically with `badudf(col)` as a GROUP BY expression.
/// Asserts the name is collected, proving the GROUP BY walk in the canonical helper is live.
///
/// Load-bearing (TD-VSDD-059): removing the GROUP BY walk from
/// `collect_unknown_scalars_from_sql_query` makes this test fail.
#[test]
fn test_c1_collect_unknown_scalar_from_sql_query_group_by() {
    use crate::ast::{
        Expr, FieldPath, FromClause, FuncCall, ScalarFunc, SelectClause, SourceRef, SourceRefKind,
        SqlQuery,
    };

    let func_expr = Expr::FuncCall(FuncCall::Scalar {
        func: ScalarFunc::Unknown("badudf".to_string()),
        args: vec![Expr::Field(FieldPath::new(vec!["col".to_string()]))],
    });

    let sq = SqlQuery {
        select: SelectClause::new(vec![]),
        from: FromClause::new(SourceRef {
            raw: "crowdstrike_alerts".to_string(),
            kind: SourceRefKind::Custom,
        }),
        joins: vec![],
        where_: None,
        group_by: vec![func_expr],
        having: None,
        order_by: vec![],
        limit: None,
    };

    let mut out = Vec::new();
    crate::engine::collect_unknown_scalars_from_sql_query_test_only(&sq, &mut out);

    assert_eq!(
        out,
        vec!["badudf".to_string()],
        "C1: collect_unknown_scalars_from_sql_query must collect ScalarFunc::Unknown \
         in GROUP BY expressions. Got: {out:?}"
    );
}

/// C1 unit — `collect_unknown_scalars_from_sql_query` finds unknown scalar in ORDER BY.
///
/// Constructs a SqlQuery programmatically with `badudf(col)` as an ORDER BY expression.
/// Asserts the name is collected, proving the ORDER BY walk is live.
///
/// Load-bearing (TD-VSDD-059): removing the ORDER BY walk makes this test fail.
#[test]
fn test_c1_collect_unknown_scalar_from_sql_query_order_by() {
    use crate::ast::{
        Expr, FieldPath, FromClause, FuncCall, OrderExpr, ScalarFunc, SelectClause, SortDirection,
        SourceRef, SourceRefKind, SqlQuery,
    };

    let func_expr = Expr::FuncCall(FuncCall::Scalar {
        func: ScalarFunc::Unknown("rankerudf".to_string()),
        args: vec![Expr::Field(FieldPath::new(vec!["severity".to_string()]))],
    });

    let sq = SqlQuery {
        select: SelectClause::new(vec![]),
        from: FromClause::new(SourceRef {
            raw: "crowdstrike_alerts".to_string(),
            kind: SourceRefKind::Custom,
        }),
        joins: vec![],
        where_: None,
        group_by: vec![],
        having: None,
        order_by: vec![OrderExpr {
            expr: func_expr,
            direction: SortDirection::Asc,
        }],
        limit: None,
    };

    let mut out = Vec::new();
    crate::engine::collect_unknown_scalars_from_sql_query_test_only(&sq, &mut out);

    assert_eq!(
        out,
        vec!["rankerudf".to_string()],
        "C1: collect_unknown_scalars_from_sql_query must collect ScalarFunc::Unknown \
         in ORDER BY expressions. Got: {out:?}"
    );
}

/// C2 unit — `collect_unknown_scalars_from_sql_query` finds unknown scalar in JOIN ON.
///
/// Constructs a SqlQuery programmatically with a JOIN whose ON condition is
/// `badjoinudf(col) = other_col` (Expr::FuncCall in the on field).
/// Asserts the name is collected, proving the JOIN ON walk is live.
///
/// Note: the PrismQL parser's JOIN grammar uses `col = col` equality in ON, so this
/// position is exercised here via direct AST construction (not parser).
///
/// Load-bearing (TD-VSDD-059): removing the JOIN ON walk makes this test fail.
#[test]
fn test_c2_collect_unknown_scalar_from_sql_query_join_on() {
    use crate::ast::{
        Expr, FieldPath, FromClause, FuncCall, Join, JoinKind, ScalarFunc, SelectClause, SourceRef,
        SourceRefKind, SqlQuery,
    };

    let join_on_expr = Expr::FuncCall(FuncCall::Scalar {
        func: ScalarFunc::Unknown("badjoinudf".to_string()),
        args: vec![Expr::Field(FieldPath::new(vec!["x".to_string()]))],
    });

    let sq = SqlQuery {
        select: SelectClause::new(vec![]),
        from: FromClause::new(SourceRef {
            raw: "crowdstrike_alerts".to_string(),
            kind: SourceRefKind::Custom,
        }),
        joins: vec![Join {
            kind: JoinKind::Inner,
            source: SourceRef {
                raw: "cyberint_alerts".to_string(),
                kind: SourceRefKind::Custom,
            },
            alias: None,
            on: join_on_expr,
        }],
        where_: None,
        group_by: vec![],
        having: None,
        order_by: vec![],
        limit: None,
    };

    let mut out = Vec::new();
    crate::engine::collect_unknown_scalars_from_sql_query_test_only(&sq, &mut out);

    assert_eq!(
        out,
        vec!["badjoinudf".to_string()],
        "C2: collect_unknown_scalars_from_sql_query must collect ScalarFunc::Unknown \
         in JOIN ON expressions. Got: {out:?}"
    );
}

/// C1+C2 integration — SQL mode GROUP BY unknown scalar triggers E-QUERY-039.
///
/// `SELECT severity, COUNT(*) FROM cyberint_alerts GROUP BY badudf(severity)` where
/// `badudf` is not a registered UDF must return E-QUERY-039, NOT opaque E-INT-001.
///
/// Load-bearing: fails if GROUP BY walk is removed from
/// `collect_unknown_scalars_from_sql_query`.
#[tokio::test]
async fn test_c1_sql_group_by_unknown_scalar_triggers_e_query_039() {
    let engine = make_test_engine_threat_intel();

    // GROUP BY clause with an unknown scalar function — triggers the GROUP BY walk fix.
    let result = engine
        .execute(
            "SELECT severity, COUNT(*) FROM cyberint_alerts GROUP BY badudf(severity)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "C1: SQL GROUP BY with unknown scalar 'badudf' must return Err (E-QUERY-039). \
         Prior to C1/C2 fix, GROUP BY was not walked → gate bypass → opaque E-INT-001. \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "C1: SQL GROUP BY unknown scalar must trigger E-QUERY-039. Got: {display}"
    );
    assert!(
        !display.contains("E-INT-001") && !display.contains("Internal error"),
        "C1: error must NOT be opaque E-INT-001. Got: {display}"
    );
    assert!(
        display.contains("badudf"),
        "C1: error must name the unregistered function 'badudf'. Got: {display}"
    );
}

/// C1+C2 integration — SQL mode ORDER BY unknown scalar triggers E-QUERY-039.
///
/// `SELECT severity FROM cyberint_alerts ORDER BY rankerudf(severity)` must return
/// E-QUERY-039, NOT opaque E-INT-001.
///
/// Load-bearing: fails if ORDER BY walk is removed from
/// `collect_unknown_scalars_from_sql_query`.
#[tokio::test]
async fn test_c1_sql_order_by_unknown_scalar_triggers_e_query_039() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT severity FROM cyberint_alerts ORDER BY rankerudf(severity)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "C1: SQL ORDER BY with unknown scalar 'rankerudf' must return Err (E-QUERY-039). \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "C1: SQL ORDER BY unknown scalar must trigger E-QUERY-039. Got: {display}"
    );
    assert!(
        !display.contains("E-INT-001") && !display.contains("Internal error"),
        "C1: error must NOT be opaque E-INT-001. Got: {display}"
    );
    assert!(
        display.contains("rankerudf"),
        "C1: error must name the unregistered function 'rankerudf'. Got: {display}"
    );
}

/// OBS-2 — ENGINE-LEVEL `did_you_mean = Some(...)` from strsim + lexicographic tie-break.
///
/// BC-2.11.019 §EC-11-059 specifies `did_you_mean` carries the closest registered
/// UDF name within Levenshtein distance 3.  The strsim + lexicographic tie-break computation
/// lives in `check_enrich_udf_availability` in `engine.rs`.
///
/// Prior coverage: existing tests only assert `did_you_mean = None` (empty registry,
/// EC-11-059) or do not assert the field at all.  No test exercised the `Some(...)` path
/// — the strsim computation was a dead code path from a testing perspective.
///
/// Query: `FROM cyberint_alerts | enrich threat_scor(iocs_value)`.
/// `threat_scor` has Levenshtein distance 1 from registered name `threat_score` — within
/// the Levenshtein-3 threshold — so the engine MUST set `did_you_mean = Some("threat_score")`.
///
/// Load-bearing (TD-VSDD-059): removing the strsim computation from
/// `check_enrich_udf_availability` makes this test fail with `did_you_mean = None`.
#[tokio::test]
async fn test_obs2_did_you_mean_some_from_strsim_levenshtein_within_threshold() {
    use prism_core::error::PrismError;

    // Engine has threat_intel infusion with per-field UDFs:
    //   threat_score, threat_is_known_malicious, threat_sources
    // `threat_scor` is Levenshtein distance 1 from `threat_score` — within threshold 3.
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich threat_scor(iocs_value)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "OBS-2: query with near-miss UDF name 'threat_scor' must return Err (E-QUERY-039). \
         Got Ok."
    );

    let err = result.unwrap_err();

    match err {
        PrismError::EnrichUdfNotFound(ref details) => {
            // Primary: infusion field must reflect the queried name.
            assert_eq!(
                details.infusion, "threat_scor",
                "OBS-2: infusion field must be 'threat_scor'. Got: {:?}",
                details.infusion
            );

            // OBS-2 core assertion: did_you_mean must be Some("threat_score") because
            // Levenshtein("threat_scor", "threat_score") = 1, which is within threshold 3.
            // This exercises the live strsim computation in check_enrich_udf_availability.
            assert_eq!(
                details.did_you_mean,
                Some("threat_score".to_string()),
                "OBS-2: did_you_mean must be Some(\"threat_score\") for near-miss 'threat_scor' \
                 (Levenshtein distance 1, threshold 3). \
                 Got: {:?}. \
                 Failure means strsim computation is missing or returning None for distance-1 input.",
                details.did_you_mean
            );
        }
        other => {
            let display = format!("{other}");
            panic!(
                "OBS-2: expected PrismError::EnrichUdfNotFound, got: {other:?}. \
                 Display: {display}"
            );
        }
    }
}

/// OBS-2b — query with a UDF name BEYOND Levenshtein 3 threshold returns `did_you_mean = None`.
///
/// Negative counterpart to OBS-2: `threat_xxxxx` has Levenshtein distance > 3 from ALL
/// registered UDF names, so `did_you_mean` MUST be `None`.
///
/// Load-bearing: if the threshold guard is removed (all distances trigger Some), this test fails.
#[tokio::test]
async fn test_obs2b_did_you_mean_none_when_beyond_levenshtein_threshold() {
    use prism_core::error::PrismError;

    let engine = make_test_engine_threat_intel();

    // "totally_unknown_udf" has Levenshtein distance >> 3 from all registered names:
    //   threat_score (distance ~12), threat_is_known_malicious (distance ~15), etc.
    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich totally_unknown_udf(iocs_value)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "OBS-2b: query with far-miss UDF name must return Err (E-QUERY-039). Got Ok."
    );

    let err = result.unwrap_err();

    match err {
        PrismError::EnrichUdfNotFound(ref details) => {
            assert_eq!(
                details.did_you_mean, None,
                "OBS-2b: did_you_mean must be None when no registered UDF is within \
                 Levenshtein 3 of 'totally_unknown_udf'. Got: {:?}",
                details.did_you_mean
            );
        }
        other => {
            let display = format!("{other}");
            panic!(
                "OBS-2b: expected PrismError::EnrichUdfNotFound, got: {other:?}. \
                 Display: {display}"
            );
        }
    }
}

/// C1+C2 integration — SqlPipe mode GROUP BY unknown scalar triggers E-QUERY-039.
///
/// `SELECT severity, COUNT(*) FROM cyberint_alerts GROUP BY badudf(severity) | LIMIT 10`
/// must return E-QUERY-039 (the GROUP BY is in the SqlPipe HEAD).
///
/// Load-bearing: fails if GROUP BY walk is removed from the SqlPipe arm of
/// `check_enrich_udf_availability`.
#[tokio::test]
async fn test_c1_sqlpipe_group_by_unknown_scalar_triggers_e_query_039() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT severity, COUNT(*) FROM cyberint_alerts GROUP BY badudf(severity) | LIMIT 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "C1: SqlPipe GROUP BY with unknown scalar 'badudf' must return Err (E-QUERY-039). \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "C1: SqlPipe GROUP BY unknown scalar must trigger E-QUERY-039. Got: {display}"
    );
    assert!(
        display.contains("badudf"),
        "C1: error must name the unregistered function 'badudf'. Got: {display}"
    );
}

// ── F-PJL1-HIGH-001: DataFusion built-in scalar exclusion tests ──────────────
//
// BC-2.11.019 §Gate firing condition: "fire E-QUERY-039 ONLY for a name
// that is neither a DataFusion built-in scalar NOR a registered enrichment UDF."
//
// Previously the gate would fire E-QUERY-039 for DataFusion built-ins like
// lower(), upper(), coalesce(), etc. — a functional regression because these
// ARE resolvable by ctx.sql() (registered in build_session_context via
// SessionStateDefaults). With an infusion registry wired, any SQL like
// `SELECT lower(hostname) FROM crowdstrike_detections` incorrectly returned
// E-QUERY-039 before this fix.

/// EC-11-064 — DataFusion built-in `lower()` must NOT trigger E-QUERY-039.
///
/// Story v2.3 inventory name: `test_bc_2_11_019_n1b_builtin_passthrough_lower` (EC-11-064).
///
/// With an infusion registry wired and `cyberint_alerts` registered, a query
/// `SELECT lower(iocs_value) FROM cyberint_alerts` must succeed (not return
/// E-QUERY-039) because `lower` is a DataFusion built-in scalar that ctx.sql()
/// will resolve correctly.
///
/// Fail-before: the gate over-matched ScalarFunc::Unknown names and returned
/// E-QUERY-039 for `lower` (since the parser renders ALL unknown scalars as
/// ScalarFunc::Unknown). Pass-after: the gate skips names present in the
/// DataFusion built-in scalar function set.
///
/// Load-bearing: if DATAFUSION_BUILTIN_FUNCTION_NAMES is removed or the
/// built-in exclusion check is removed from check_enrich_udf_availability,
/// this test fails because `lower` returns E-QUERY-039.
#[tokio::test]
async fn test_bc_2_11_019_n1b_builtin_passthrough_lower() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT lower(iocs_value) FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    // Must NOT return E-QUERY-039.
    if let Err(ref e) = result {
        let display = format!("{e}");
        assert!(
            !display.contains("E-QUERY-039"),
            "EC-11-064: DataFusion built-in 'lower' must NOT trigger E-QUERY-039. \
             Got: {display}. Fix: add DataFusion built-in exclusion to \
             check_enrich_udf_availability (BC-2.11.019 v1.5)."
        );
    }
    // Note: the query may fail for other reasons (e.g. no sensor data at test time),
    // but it must NOT fail with E-QUERY-039. We only assert the absence of that error.
    // The test runner confirms no E-QUERY-039 in the error path; Ok is also acceptable.
}

/// EC-11-065 — DataFusion built-in `coalesce()` and other built-ins must NOT trigger E-QUERY-039.
///
/// Story v2.3 inventory name: `test_bc_2_11_019_n1b_builtin_passthrough_coalesce` (EC-11-065).
///
/// Covers: `coalesce` (the multi-arg form registered as a built-in DataFusion scalar),
/// `upper`, `length`, `abs`.
/// Each is a DataFusion built-in that the AST renders as ScalarFunc::Unknown before
/// ctx.sql() resolves it. With the fix, none trigger E-QUERY-039.
///
/// The coalesce assertion is the primary load-bearing one: `SELECT coalesce(iocs_value,
/// 'unknown') FROM cyberint_alerts` passes the E-QUERY-038 column gate (iocs_value is a
/// valid column in cyberint_alerts — or gate fails open in test mode) and reaches the
/// E-QUERY-039 gate, which MUST NOT fire because `coalesce` is in
/// `DATAFUSION_BUILTIN_FUNCTION_NAMES`.
///
/// NOTE: `cyberint_alerts` in the test fixture has an empty columns array, so the
/// E-QUERY-038 column check `fails open` (no resolved_spec_map in test mode) — the
/// coalesce query proceeds past gate 038 to gate 039, which is the gate under test.
///
/// Load-bearing (EC-11-065 / TD-VSDD-059):
/// - Removing `coalesce` from `DATAFUSION_BUILTIN_FUNCTION_NAMES` makes the coalesce
///   assertion fail with E-QUERY-039.
/// - Removing the entire built-in exclusion check makes ALL assertions fail.
#[tokio::test]
async fn test_bc_2_11_019_n1b_builtin_passthrough_coalesce() {
    let engine = make_test_engine_threat_intel();

    // Primary: coalesce with a valid column — must pass E-QUERY-039 gate (coalesce is built-in).
    // E-QUERY-038 fails open in test mode (no resolved_spec_map), so this reaches gate 039.
    let coalesce_result = engine
        .execute(
            "SELECT coalesce(iocs_value, 'unknown') FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    if let Err(ref e) = coalesce_result {
        let display = format!("{e}");
        assert!(
            !display.contains("E-QUERY-039"),
            "EC-11-065: DataFusion built-in 'coalesce' must NOT trigger E-QUERY-039. \
             Got: {display}. Fix: verify 'coalesce' is in DATAFUSION_BUILTIN_FUNCTION_NAMES \
             and the built-in exclusion check is active in check_enrich_udf_availability \
             (BC-2.11.019 v1.5). This test FAILS if coalesce is removed from the exclusion set."
        );
    }

    // Secondary: upper, length, abs — also DataFusion built-ins, must not fire E-QUERY-039.
    let builtin_queries = [
        ("upper", "SELECT upper(iocs_value) FROM cyberint_alerts"),
        ("length", "SELECT length(iocs_value) FROM cyberint_alerts"),
        ("abs", "SELECT abs(severity_score) FROM cyberint_alerts"),
    ];

    for (fn_name, query) in &builtin_queries {
        let result = engine.execute(query, QueryOptions::default()).await;

        if let Err(ref e) = result {
            let display = format!("{e}");
            assert!(
                !display.contains("E-QUERY-039"),
                "EC-11-065: DataFusion built-in '{fn_name}' must NOT trigger E-QUERY-039. \
                 Query: {query}. Got: {display}. \
                 Fix: add DataFusion built-in exclusion to check_enrich_udf_availability \
                 (BC-2.11.019 v1.5)."
            );
        }
    }
}

/// F-PJL1-HIGH-001 regression guard — a genuinely unknown name still triggers E-QUERY-039.
///
/// `bogus_enrich_fn` is neither a DataFusion built-in nor a registered infusion.
/// The gate must still fire E-QUERY-039 for it, proving the exclusion is additive
/// (skips built-ins only) and does not break the main gate behavior.
///
/// Load-bearing: if built-in exclusion incorrectly skips ALL names, this test
/// catches the regression.
#[tokio::test]
async fn test_f_pjl1_high001_non_builtin_unknown_still_triggers_e_query_039() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT bogus_enrich_fn(iocs_value) FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PJL1-HIGH-001 regression guard: non-builtin unknown 'bogus_enrich_fn' must \
         return Err (E-QUERY-039). Got Ok — built-in exclusion is too broad."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "F-PJL1-HIGH-001 regression guard: non-builtin unknown 'bogus_enrich_fn' must \
         trigger E-QUERY-039. Got: {display}"
    );
    assert!(
        display.contains("bogus_enrich_fn"),
        "F-PJL1-HIGH-001 regression guard: error must name the unregistered function \
         'bogus_enrich_fn'. Got: {display}"
    );
}

// ── F-PJL4-MED-001: SCHEDULED path gate-ordering discriminating test ──────────
//
// AC-H1 moved the capability gate (E-QUERY-011) AFTER 037/038/039 in
// execute_scheduled_inner. The existing `test_h1_gate_ordering_discriminating_table_fires_before_capability`
// tests both execute and execute_scheduled, but F-PJL4-MED-001 requires an
// additional test that specifically exercises only the SCHEDULED path with a
// query that triggers E-QUERY-037 (unregistered table) and would trigger
// E-QUERY-011 (prism_audit reference) if the ordering were reversed.

// ── F1 RED GATE: DataFusion built-in aggregate + window exclusion (BC-2.11.019 §F-PJL1-HIGH-001) ──
//
// BC-2.11.019 §F-PJL1-HIGH-001 amendment: SQL-mode E-QUERY-039 fires ONLY when the
// name is (a) not a PQL typed scalar variant, (b) NOT in scalar + aggregate + window
// built-ins, AND (c) not in InfusionRegistry.
//
// Current bug (F1, HIGH): DATAFUSION_BUILTIN_FUNCTION_NAMES is built ONLY from
// `SessionStateDefaults::default_scalar_functions()`. DataFusion built-in aggregate
// functions (stddev, median, variance, approx_distinct, etc.) and window functions
// (row_number, rank, dense_rank, etc.) parse as `ScalarFunc::Unknown` in PrismQL, but
// are NOT in the scalar-only exclusion set — causing them to falsely trigger E-QUERY-039.
//
// Fix: expand the LazyLock to union scalar + aggregate + window functions; rename to
// DATAFUSION_BUILTIN_FUNCTION_NAMES.
//
// Pipe-mode guard: the built-in exclusion applies to SQL-mode ONLY. `| enrich stddev(col)`
// still fires E-QUERY-039 because pipe-mode enrich is an explicit infusion directive.

/// EC-11-066 RED GATE — DataFusion built-in aggregate `stddev()` must NOT trigger E-QUERY-039.
///
/// Story v2.4 inventory name: EC-11-066 (BC-2.11.019 §F-PJL1-HIGH-001 amendment).
///
/// `SELECT stddev(severity_score) FROM cyberint_alerts` must NOT return E-QUERY-039.
/// `stddev` is a DataFusion built-in aggregate function (in `default_aggregate_functions()`),
/// NOT a scalar function. The current `DATAFUSION_BUILTIN_FUNCTION_NAMES` exclusion set
/// is scalar-only, so `stddev` parses as `ScalarFunc::Unknown("stddev")` and falsely
/// triggers E-QUERY-039 before `ctx.sql()` gets to resolve it correctly.
///
/// FAIL-BEFORE: exclusion set is scalar-only → stddev triggers E-QUERY-039.
/// PASS-AFTER: exclusion set includes aggregate functions (DATAFUSION_BUILTIN_FUNCTION_NAMES).
///
/// Load-bearing: removing `default_aggregate_functions()` from the exclusion union causes
/// this test to fail with E-QUERY-039.
#[tokio::test]
async fn test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT stddev(severity_score) FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    // Primary assertion: must NOT trigger E-QUERY-039.
    // The query may fail for other reasons (e.g. table has no data at test time,
    // column validation) but the enrich gate MUST NOT be the reason.
    if let Err(ref e) = result {
        let display = format!("{e}");
        assert!(
            !display.contains("E-QUERY-039"),
            "EC-11-066 RED GATE: DataFusion built-in aggregate 'stddev' must NOT trigger \
             E-QUERY-039 in SQL mode. Got: {display}. \
             Fix: expand DATAFUSION_BUILTIN_FUNCTION_NAMES to include \
             SessionStateDefaults::default_aggregate_functions() and rename to \
             DATAFUSION_BUILTIN_FUNCTION_NAMES (BC-2.11.019 v1.6 §F-PJL1-HIGH-001)."
        );
    }
    // Ok result is also acceptable (no E-QUERY-039 means the gate was correctly bypassed).
}

/// EC-11-067 RED GATE — DataFusion built-in window function `row_number()` must NOT
/// trigger E-QUERY-039.
///
/// Story v2.4 inventory name: EC-11-067 (BC-2.11.019 §F-PJL1-HIGH-001 amendment).
///
/// `SELECT row_number() FROM cyberint_alerts` must NOT return E-QUERY-039.
/// `row_number` is a DataFusion built-in window function (in `default_window_functions()`),
/// not a scalar. It parses as `ScalarFunc::Unknown("row_number")` in PrismQL's SQL parser
/// (PrismQL recognises only 7 aggregates as `FuncCall::Aggregate`; everything else is
/// `ScalarFunc::Unknown`). The current scalar-only exclusion set misses it.
///
/// FAIL-BEFORE: exclusion set is scalar-only → row_number triggers E-QUERY-039.
/// PASS-AFTER: exclusion set includes window functions (DATAFUSION_BUILTIN_FUNCTION_NAMES).
///
/// Load-bearing: removing `default_window_functions()` from the exclusion union causes
/// this test to fail with E-QUERY-039.
#[tokio::test]
async fn test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039() {
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "SELECT row_number() FROM cyberint_alerts",
            QueryOptions::default(),
        )
        .await;

    // Primary assertion: must NOT trigger E-QUERY-039.
    if let Err(ref e) = result {
        let display = format!("{e}");
        assert!(
            !display.contains("E-QUERY-039"),
            "EC-11-067 RED GATE: DataFusion built-in window function 'row_number' must NOT \
             trigger E-QUERY-039 in SQL mode. Got: {display}. \
             Fix: expand DATAFUSION_BUILTIN_FUNCTION_NAMES to include \
             SessionStateDefaults::default_window_functions() and rename to \
             DATAFUSION_BUILTIN_FUNCTION_NAMES (BC-2.11.019 v1.6 §F-PJL1-HIGH-001)."
        );
    }
    // Ok result is also acceptable.
}

/// F-PNL1 pipe-mode guard — `| enrich stddev(col)` in PIPE mode STILL fires E-QUERY-039.
///
/// BC-2.11.019 §F-PJL1-HIGH-001: the built-in exclusion applies to SQL-mode
/// `ScalarFunc::Unknown` paths ONLY. Pipe-mode `| enrich <name>(...)` is an explicit
/// enrichment directive — even if `<name>` is a DataFusion built-in aggregate name,
/// when used as a pipe enrich infusion and NOT in the `InfusionRegistry`, E-QUERY-039
/// MUST still fire (the analyst is trying to call an unregistered infusion, not a scalar).
///
/// This guards against over-broadening: if the built-in exclusion were incorrectly applied
/// to pipe-mode names (`pipe_enrich_names`), the gate would silently pass unregistered
/// pipe-mode enrich directives.
///
/// Load-bearing: if the fix applies the built-in exclusion to `pipe_enrich_names`,
/// this test fails (no E-QUERY-039 produced for `stddev` in pipe mode).
#[tokio::test]
async fn test_f_pnl1_pipe_mode_builtin_aggregate_still_fires_e_query_039() {
    // engine has threat_intel infusion (per-field UDFs: threat_score, etc.)
    // `stddev` is NOT a registered infusion name.
    let engine = make_test_engine_threat_intel();

    let result = engine
        .execute(
            "FROM cyberint_alerts | enrich stddev(severity_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PNL1 pipe-mode guard: `| enrich stddev(col)` must return Err(E-QUERY-039) \
         because 'stddev' is not a registered infusion name (pipe-mode enrich directives \
         are NOT subject to the DataFusion built-in exclusion). Got Ok — \
         the built-in exclusion was incorrectly applied to pipe-mode names."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        display.contains("E-QUERY-039"),
        "F-PNL1 pipe-mode guard: `| enrich stddev(...)` must trigger E-QUERY-039. \
         Got: {display}. The fix must NOT apply the DataFusion built-in exclusion to \
         pipe-mode enrich names (BC-2.11.019 v1.6 §F-PJL1-HIGH-001 scope of change)."
    );
}

/// F-PJL4-MED-001 — execute_scheduled SCHEDULED path: E-QUERY-037 fires BEFORE E-QUERY-011.
///
/// A scheduled query against an unregistered table (`ghost_sensor_alerts`) that also
/// references `prism_audit` must return E-QUERY-037 (TableNotAvailable), NOT
/// E-QUERY-011 (AuditTableAccessDenied).
///
/// This test exercises ONLY the `execute_scheduled` path (not `execute`),
/// making it a discriminating guard specifically for the scheduled gate ordering.
///
/// # Why this test is needed (F-PJL4-MED-001)
///
/// The existing `test_h1_gate_ordering_discriminating_table_fires_before_capability`
/// checks both paths in a single assertion. If the scheduled path ordering were
/// reverted (capability gate moved BEFORE table gate in execute_scheduled_inner),
/// that test would catch it — but only by comparing `sched_err` to a
/// `PrismError::TableNotAvailable` variant alongside the `execute_result` check.
///
/// This test is a DEDICATED scheduled-only guard: it fails if and only if the
/// scheduled path runs the capability gate before the table gate, making the
/// failure message unambiguous. A reviewer or CI run that sees this test failing
/// knows exactly which path is broken (scheduled, not interactive).
///
/// # Revert-verification
///
/// Moving `check_internal_table_capabilities` BEFORE `check_table_availability`
/// in `execute_scheduled_inner` would cause this test to return E-QUERY-011
/// instead of E-QUERY-037, and this test would FAIL.
///
/// # BC reference
/// BC-2.11.019 / H1 fix (S-DEMO-FIDELITY-REMEDIATION-001 F-PJL4-MED-001).
#[tokio::test]
async fn test_f_pjl4_med001_scheduled_path_table_gate_fires_before_capability_gate() {
    use crate::table_registry::TableRegistry;
    use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

    // Build engine with a wired TableRegistry containing only `armis`.
    // This activates the E-QUERY-037 gate for any non-armis table.
    let armis_spec = SensorSpec::new(
        "armis",
        "Armis sensor",
        AuthType::ApiKey,
        "https://example.com",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        Vec::new(),
    );
    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&armis_spec)
        .expect("register armis must not fail");

    let engine = crate::engine::QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        crate::engine::QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(Arc::clone(&registry));

    // Discriminating query for the SCHEDULED path:
    //   `ghost_sensor_alerts` — NOT registered → E-QUERY-037 (table gate)
    //   `prism_audit` via JOIN — requires AuditRead capability (absent in scheduled
    //                            system context `&[]`) → E-QUERY-011 (capability gate)
    //
    // Post-fix (table FIRST): scheduled path returns E-QUERY-037.
    // Pre-fix (capability FIRST in execute_scheduled_inner): scheduled path returns E-QUERY-011.
    let query = "SELECT * FROM ghost_sensor_alerts JOIN prism_audit ON id = id LIMIT 10";

    let scheduled_result = engine
        .execute_scheduled(query, None)
        .await
        .map(|(qr, _ctx)| qr);

    assert!(
        scheduled_result.is_err(),
        "F-PJL4-MED-001: execute_scheduled with unregistered table must return Err"
    );

    let sched_err = scheduled_result.unwrap_err();

    // DISCRIMINATING assertion: SCHEDULED path must return E-QUERY-037 (table gate fires
    // first), NOT E-QUERY-011 (capability gate).
    //
    // If execute_scheduled_inner still runs the capability gate FIRST (pre-fix ordering),
    // this assertion fails with AuditTableAccessDenied (E-QUERY-011).
    assert!(
        matches!(sched_err, PrismError::TableNotAvailable(_)),
        "F-PJL4-MED-001 DISCRIMINATING (SCHEDULED path only): execute_scheduled must return \
         TableNotAvailable (E-QUERY-037), NOT AuditTableAccessDenied (E-QUERY-011). \
         Got: {sched_err:?}. \
         If AuditTableAccessDenied: the H1 fix is incomplete for the scheduled path — \
         execute_scheduled_inner is running the capability gate BEFORE the table gate. \
         Fix: move check_internal_table_capabilities AFTER check_table_availability \
         in execute_scheduled_inner (BC-2.11.019 v1.5 / H1 fix)."
    );
}
