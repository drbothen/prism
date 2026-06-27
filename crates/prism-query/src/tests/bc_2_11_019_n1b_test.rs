//! Red Gate tests for S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B — BC-2.11.019 v1.2.
//!
//! Finding N1-B: E-QUERY-039 (EnrichUdfNotFound) gate does NOT yet exist.
//! `PrismError::EnrichUdfNotFound` variant and `EnrichUdfNotFoundDetails` struct
//! have ZERO workspace matches (verified 2026-06-26). The plan-time enrichment gate
//! in `engine.rs` must be implemented from scratch.
//!
//! # Test strategy for net-new variant (pre-implementation)
//!
//! Since `PrismError::EnrichUdfNotFound` does not exist yet, tests that try to
//! CONSTRUCT the variant would produce a compile error, blocking ALL tests in this
//! target. Per the story's Red Gate guidance (S1 relaxation v1.2): assert on the
//! OBSERVABLE error at the query boundary via `execute()` output.
//!
//! Tests assert:
//! 1. Query `FROM cyberint_alerts | enrich threat_intel(iocs_value)` where
//!    `threat_intel` is an infusion_id (NOT registered as a UDF name) returns Err(_)
//!    with a Display string containing "E-QUERY-039".
//! 2. The error is NOT an opaque internal error (Display does NOT contain "E-INT-001"
//!    / "Internal error").
//! 3. `available_infusions` is non-empty in the error details — verified by asserting
//!    that the Display string contains at least one of the registered per-field UDF names.
//! 4. SQL mode `SELECT nvd(iocs_value) FROM cyberint_alerts` returns Err(_)
//!    with Display containing "E-QUERY-039" (SQL path gate).
//!
//! RED GATE: Current engine has NO E-QUERY-039 gate. Queries with unregistered
//! enrichment function names either succeed (return empty data) or fail with a
//! DataFusion error — neither produces "E-QUERY-039" in the display string.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_019_n1b_infusion_id_as_udf_name | AC-N1B | BC-2.11.019 v1.2 |
//! | test_bc_2_11_019_n1b_sql_path_infusion_id_as_udf_name | AC-N1B | BC-2.11.019 v1.2 |

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
            InfusionField::new("threat_score", "iocs_value", "string", "float64"),
            InfusionField::new("threat_is_known_malicious", "iocs_value", "string", "bool"),
            InfusionField::new("threat_sources", "iocs_value", "string", "string"),
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

/// BC-2.11.019 v1.2 AC-N1B — pipe path Red Gate test.
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
/// RED GATE: Current engine has NO E-QUERY-039 plan-time enrichment gate. The query
/// either succeeds with empty results (fan-out finds no data for an unregistered
/// enrichment function) or fails with a DataFusion error — neither produces "E-QUERY-039".
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
         must return Err — current code has no plan-time E-QUERY-039 gate. \
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

/// BC-2.11.019 v1.2 AC-N1B — SQL path Red Gate test.
///
/// A SQL-mode query using an unregistered enrichment function name `nvd`
/// (which is an infusion_id, not a per-field UDF name) must return
/// `Err(PrismError::EnrichUdfNotFound(_))` at plan time.
///
/// RED GATE: Same as pipe path — no E-QUERY-039 gate exists in current engine.
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
         SQL SELECT must return Err — current code has no plan-time E-QUERY-039 gate. \
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
/// BC-2.11.019 v1.3 enrich-LAST ordering: E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
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
/// BC-2.11.019 v1.3 §Precondition 1(b): the enrich gate must scan BOTH SELECT projections
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
