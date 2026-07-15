//! Red Gate tests for S-PRISMQL-NATIVE-TEMPORAL-TYPING-001:
//! E-QUERY-041 temporal literal pre-validator (ADR-052 D4 Option A).
//!
//! Tests RG-004, RG-005, RG-007 verify the plan-time AST-walk pre-validator
//! (`check_temporal_literals` in `materialization.rs`) that fires AFTER E-QUERY-037,
//! E-QUERY-038, E-QUERY-039 and BEFORE DataFusion execution.
//!
//! Gate ordering: E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-041 → DataFusion.
//!
//! # Red Gate pre-implementation failure
//! The `check_temporal_literals` body was `todo!()` in the stubs commit (9401a6ca).
//! ALL three tests (RG-004, RG-005, RG-007) panicked with "not yet implemented"
//! when `engine.execute(...)` reached the temporal gate call.
//!
//! # Post-implementation
//! - RG-004: returns `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24" })`
//! - RG-005: same as RG-004, pipe mode parity
//! - RG-007: the pre-validator passes (valid RFC-3339); query continues to DataFusion
//!   execution (which fails with sensor error since no real sensor is wired) but does
//!   NOT return `PrismError::TemporalLiteralUnparseable`.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_…_e_query_041_sql_mode_date_only_string | AC-005 | BC-2.11.021 EC-11-021-009 |
//! | test_…_e_query_041_pipe_mode_date_only_string | AC-005 | BC-2.11.004 EC-11-004-001 |
//! | test_…_valid_rfc3339_utc_string_not_rejected | AC-007 | BC-2.11.003; BC-2.11.004 |

use std::sync::Arc;

use crate::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    table_registry::TableRegistry,
};
use prism_core::error::PrismError;

// ── Test fixture helpers ──────────────────────────────────────────────────────

/// Minimal no-op credential store for unit tests that don't exercise auth.
/// Mirrors the `NoopCs` pattern from `bc_2_11_019_n1b_test.rs`.
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

/// Build a `TableRegistry` with sensor "test" / table "events" registered as
/// "test_events". Includes column specs:
///   - `timestamp: ColumnType::Datetime` — used by E-QUERY-041 schema-aware gate
///   - `hostname: ColumnType::String` — non-datetime column for negative-control tests
///
/// Gate ordering guarantee: E-QUERY-037 (table_check) passes because "test_events"
/// IS registered. E-QUERY-038 (column_check) validates known columns.
/// E-QUERY-039 (enrich gate) is skipped (no infusion_registry wired).
/// E-QUERY-041 (`check_temporal_literals`) is schema-aware: only rejects bad date
/// literals when the compared column is `ColumnType::Datetime`.
fn make_test_events_registry() -> Arc<TableRegistry> {
    use prism_core::ColumnType;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

    let registry = Arc::new(TableRegistry::new());
    let spec = SensorSpec::new(
        "test",
        "Test sensor",
        AuthType::ApiKey,
        "https://test.invalid",
        vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![
                // timestamp is a Datetime column — E-QUERY-041 must fire for bad literals
                ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                // hostname is a String column — E-QUERY-041 must NOT fire for hostname comparisons
                ColumnSpec::new("hostname", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&spec)
        .expect("register test sensor must not fail");
    registry
}

/// Build a `QueryEngine` wired with the "test_events" table registry and no infusion
/// registry. The engine gates fire in order: E-QUERY-037 → E-QUERY-038 (fail-open) →
/// E-QUERY-039 (skipped, no registry) → E-QUERY-041 (fully implemented).
fn make_test_engine() -> QueryEngine {
    let registry = make_test_events_registry();
    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

/// Build a `TableRegistry` with sensor "ghost_sensor" / table "devices" registered as
/// "ghost_sensor_devices". Includes a "timestamp" `ColumnType::Datetime` column.
///
/// Used by MED-1 tests for dotted external-source path verification: the registered
/// name is `ghost_sensor_devices` (not `ghost_sensor`). The `check_temporal_literals`
/// AST-walk uses `primary_table_from_ast` which translates the
/// `SourceRefKind::External { sensor, table }` form to `"{sensor}_{table}"`
/// (i.e., `ghost_sensor_devices`) for registry lookup.
fn make_ghost_sensor_devices_registry() -> Arc<TableRegistry> {
    use prism_core::ColumnType;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

    let registry = Arc::new(TableRegistry::new());
    let spec = SensorSpec::new(
        "ghost_sensor",
        "Ghost sensor for MED-1 dotted-source tests",
        AuthType::ApiKey,
        "https://ghost.invalid",
        vec![TableSpec::new_point_in_time(
            "devices",
            "security_finding",
            vec![
                ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                ColumnSpec::new("hostname", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&spec)
        .expect("register ghost_sensor must not fail");
    registry
}

/// Build a `QueryEngine` wired with the "ghost_sensor_devices" registry.
fn make_ghost_sensor_engine() -> QueryEngine {
    let registry = make_ghost_sensor_devices_registry();
    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// RG-004: SQL-mode date-only string literal must trigger E-QUERY-041 at plan time.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-06-24'`
///
/// # Red Gate pre-implementation failure
/// `check_temporal_literals` body was `todo!()` — `engine.execute(...)` panicked with
/// "not yet implemented: E-QUERY-041 temporal literal pre-validator".
///
/// # Post-implementation state (AC-005)
/// Returns `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24" })`.
/// The error MUST NOT be `PrismError::QueryParseFailed` (wrong gate) and MUST NOT be a
/// DataFusion error — it fires at Prism plan time, BEFORE DataFusion sees the query.
///
/// # Why load-bearing
/// Without this gate, `arrow-cast 58.2.0` would silently coerce `'2026-06-24'` to
/// midnight-local — producing a wrong temporal comparison with no error. The Prism
/// chrono pre-validator provides the only deterministic rejection (ADR-052 D4).
///
/// Traces to: BC-2.11.021 EC-11-021-009; BC-2.11.003 EC-11-003-001;
/// ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string() {
    let engine = make_test_engine();

    // Red Gate: check_temporal_literals was todo!() — panicked here.
    // Post-implementation: returns Err(TemporalLiteralUnparseable { value_prefix: "2026-06-24" }).
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be an error.
    assert!(
        result.is_err(),
        "RG-004: SQL query with date-only string literal '2026-06-24' \
         must return Err(E-QUERY-041) from the Prism plan-time chrono pre-validator. \
         Got Ok result."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary assertion: error must be E-QUERY-041 via TemporalLiteralUnparseable.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "RG-004: error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix starting with '2026-06-24'. Got: {err:?} (Display: {display})"
    );

    // Display must contain E-QUERY-041 code (per error-taxonomy.md §E-QUERY-041).
    assert!(
        display.contains("E-QUERY-041"),
        "RG-004: error Display must contain 'E-QUERY-041'. Got: {display}"
    );

    // Negative assertion: must NOT be a DataFusion error (fire at Prism plan time).
    assert!(
        !display.contains("Arrow error") && !display.contains("DataFusion"),
        "RG-004: E-QUERY-041 must fire at Prism plan time, NOT as a DataFusion/Arrow error. \
         Got: {display}"
    );
}

/// RG-005: Pipe-mode date-only string literal must trigger E-QUERY-041 at plan time.
///
/// Query: `FROM test_events | where timestamp > '2026-06-24'`
///
/// # Red Gate pre-implementation failure
/// Same `todo!()` panic as RG-004.
///
/// # Post-implementation state (AC-005)
/// Returns `Err(PrismError::TemporalLiteralUnparseable { .. })`.
/// BC-2.11.004 EC-11-004-001 specifies E-QUERY-041 in pipe `| where` stages —
/// parity with SQL mode is required.
///
/// # Why load-bearing (pipe-mode parity)
/// If E-QUERY-041 only fires in SQL mode, analysts using pipe syntax could bypass the
/// gate with the same date-only pattern, getting wrong results silently.
///
/// Traces to: BC-2.11.004 EC-11-004-001; ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string() {
    let engine = make_test_engine();

    // Red Gate: check_temporal_literals is todo!() — panics here.
    // Post-implementation: returns Err(TemporalLiteralUnparseable { .. }).
    let result = engine
        .execute(
            "FROM test_events | where timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-005: pipe-mode query with date-only string literal '2026-06-24' \
         must return Err(E-QUERY-041) from the Prism plan-time chrono pre-validator. \
         Got Ok result."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "RG-005: error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix starting with '2026-06-24' (pipe mode). Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "RG-005: error Display must contain 'E-QUERY-041'. Got: {display}"
    );
}

/// RG-007: A valid RFC-3339 UTC string must NOT be rejected by E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-06-24T00:00:00Z'`
///
/// # Red Gate pre-implementation failure
/// Same `todo!()` panic as RG-004 and RG-005 — `check_temporal_literals` panics for
/// ALL inputs before any validation logic exists.
///
/// # Post-implementation state (AC-007)
/// The pre-validator calls `chrono::DateTime::parse_from_rfc3339("2026-06-24T00:00:00Z")`,
/// which returns `Ok(...)` — the pre-validator passes through. The query continues to
/// DataFusion execution (fails with a sensor error since no real sensor is wired), but
/// does NOT return `PrismError::TemporalLiteralUnparseable`.
///
/// # Why load-bearing
/// E-QUERY-041 must fire ONLY for non-RFC-3339 forms. If it also rejects valid
/// RFC-3339 strings, existing analyst queries break silently. This negative-path test
/// guards against an overly-aggressive implementation.
///
/// Traces to: BC-2.11.003 §Valid accepted forms; BC-2.11.004 §Valid accepted;
/// ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected() {
    let engine = make_test_engine();

    // Red Gate: check_temporal_literals is todo!() — panics here for ALL inputs.
    // Post-implementation: pre-validator passes; query may fail with sensor/DataFusion error
    // but MUST NOT return PrismError::TemporalLiteralUnparseable.
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    // After implementation: the pre-validator PASSES for valid RFC-3339.
    // The query may succeed or fail (no real sensor), but must NOT be E-QUERY-041.
    match &result {
        Err(PrismError::TemporalLiteralUnparseable { value_prefix }) => {
            panic!(
                "RG-007: valid RFC-3339 UTC string '2026-06-24T00:00:00Z' must NOT trigger \
                 E-QUERY-041 — it is a well-formed timestamp. The pre-validator must only \
                 reject date-only and offset-less forms. Got E-QUERY-041 with \
                 value_prefix={value_prefix:?}."
            );
        }
        _ => {
            // Any other outcome (Ok, or a different Err) is acceptable.
            // The query is expected to fail at sensor execution (no real DTU wired),
            // but the failure must NOT be TemporalLiteralUnparseable.
        }
    }
}

/// MED-3 / HIGH-1: A non-datetime string column comparison must NOT trigger E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE hostname > 'server-a'`
///
/// `hostname` is registered as `ColumnType::String` in `make_test_events_registry`.
/// The value `'server-a'` is not a valid RFC-3339 timestamp but that is irrelevant —
/// it is a valid lexicographic string comparison and must reach DataFusion without
/// rejection from the temporal pre-validator.
///
/// # Red Gate pre-HIGH-1-fix failure (schema-blind behavior)
/// `check_temporal_literals` is schema-blind: it validates ALL string literals in
/// ordering comparisons, regardless of column type. `'server-a'` fails
/// `chrono::DateTime::parse_from_rfc3339("server-a")`, so the schema-blind implementation
/// returns `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "server-a" })`.
/// This test asserts NOT E-QUERY-041 — it FAILS against the schema-blind implementation,
/// which confirms the Red Gate property (TDD-first per MED-3).
///
/// # Post-HIGH-1-fix state
/// `check_temporal_literals` looks up `hostname` in the `TableRegistry` and finds
/// `ColumnType::String` → not a datetime column → skips RFC-3339 validation.
/// The query proceeds to DataFusion (which may fail with a sensor error, but NOT
/// E-QUERY-041).
///
/// Traces to: ADR-052 §D4 ("schema-aware"); BC-2.11.021 §Gate conditions.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_column_ordering_not_rejected() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname > 'server-a'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be E-QUERY-041 — 'server-a' is a valid string, not a malformed timestamp.
    if let Err(PrismError::TemporalLiteralUnparseable { value_prefix }) = &result {
        panic!(
            "MED-3/HIGH-1: string-column ordering comparison `WHERE hostname > 'server-a'` \
             must NOT trigger E-QUERY-041. The temporal pre-validator must only reject \
             non-RFC-3339 literals when the compared column is ColumnType::Datetime. \
             Got E-QUERY-041 with value_prefix={value_prefix:?}. \
             Fix: make check_temporal_literals schema-aware (HIGH-1)."
        );
    }
    // Any other result (Ok or a different Err) is acceptable for this test.
}

/// MED-1 Red Gate: dotted external-source pipe query with date-only literal MUST raise
/// E-QUERY-041.
///
/// Query: `FROM ghost_sensor.devices | where timestamp > '2026-06-24'`
///
/// Registered table: `ghost_sensor_devices` (sensor `ghost_sensor` + table `devices`).
///
/// # Red Gate pre-fix failure (Option-A AST-walk bug)
/// `check_temporal_literals` Ast::Pipe arm's `primary_table_from_ast` call
/// was not correctly handling the `SourceRefKind::External { sensor, table }` form,
/// returning `None` for table lookup → fail-open → E-QUERY-041 NOT raised → silent
/// wrong result.
///
/// # Post-fix state
/// `primary_table_from_ast` translates `SourceRefKind::External { sensor, table }`
/// to `"{sensor}_{table}"` (e.g., `"ghost_sensor_devices"`). The schema lookup succeeds
/// and `check_temporal_literals` fires E-QUERY-041.
///
/// Traces to: ADR-052 §D4; BC-2.11.021 EC-11-021-009; BC-2.11.004 EC-11-004-001.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_dotted_external_source_pipe_date_only_raises_e_query_041(
) {
    let engine = make_ghost_sensor_engine();

    let result = engine
        .execute(
            "FROM ghost_sensor.devices | where timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-1: dotted external-source pipe query with date-only literal '2026-06-24' \
         must return Err(E-QUERY-041). Got Ok result. \
         Root cause: check_temporal_literals AST-walk must translate SourceRefKind::External \
         'ghost_sensor.devices' → 'ghost_sensor_devices' for the registry lookup."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "MED-1: error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix starting with '2026-06-24'. Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "MED-1: error Display must contain 'E-QUERY-041'. Got: {display}"
    );
}

/// LOW-1 (negative): Filter-mode query with valid RFC-3339 literal must NOT raise E-QUERY-041.
///
/// Query: `test_events | timestamp > '2026-07-04T00:00:00Z'`
///
/// Uses underscore source form (same reasoning as the positive LOW-1 test above).
/// Valid RFC-3339 must pass through `check_temporal_literals` without triggering E-QUERY-041.
///
/// Traces to: BC-2.11.023; ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_filter_mode_valid_rfc3339_not_rejected() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "test_events | timestamp > '2026-07-04T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    if let Err(PrismError::TemporalLiteralUnparseable { value_prefix }) = &result {
        panic!(
            "LOW-1: filter-mode valid RFC-3339 '2026-07-04T00:00:00Z' must NOT trigger \
             E-QUERY-041. Got TemporalLiteralUnparseable with value_prefix={value_prefix:?}."
        );
    }
    // Any other outcome (Ok, or a different Err) is acceptable.
}

/// LOW-2 EC-006: Offset-less datetime string literal MUST raise E-QUERY-041 via the
/// Option-A `RawTemporalLiteral` AST-walk path.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00:00'`
///
/// `'2026-06-24T12:00:00'` starts with 4 digits → `classify_string_literal` attempts
/// `TimestampLiteral::new` → fails (no UTC offset) → parser emits `RawTemporalLiteral`.
/// `check_temporal_literals` walks the AST, finds `RawTemporalLiteral` compared
/// against the `timestamp` Datetime column → fires E-QUERY-041.
///
/// Traces to: ADR-052 §D4; BC-2.11.021 §Invalid forms (EC-006 "offset-less").
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_ec006_offset_less_datetime_raises_e_query_041() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "EC-006/LOW-2: offset-less datetime '2026-06-24T12:00:00' in datetime ordering \
         comparison must raise E-QUERY-041. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24T12:00:00")
        ),
        "EC-006/LOW-2: must be TemporalLiteralUnparseable{{ value_prefix: '2026-06-24T12:00:00' }}. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "EC-006/LOW-2: error Display must contain 'E-QUERY-041'. Got: {display}"
    );
}

/// LOW-1 grammar determination probe: does PrismQL admit literal-LHS ordering comparisons?
///
/// This test is a DESIGN PROBE, not a regression test. It documents whether
/// `'2026-06-24' < timestamp` (literal on LHS of ordering comparison) is accepted
/// by the PrismQL grammar in either filter mode or SQL mode.
///
/// # Grammar constraint
/// The `build_predicate_parser` in `filter_parser.rs` has the structure:
/// `field_path op rhs_expr`. The `lhs` is always a FieldPath (identifier), and the `rhs`
/// is a literal or temporal expression. A quoted string `'2026-06-24'` is NOT a valid
/// FieldPath, so the grammar CANNOT parse `'2026-06-24' < timestamp` in either filter
/// mode or SQL mode (which delegates to the same base predicate parser).
///
/// # Consequence for `check_temporal_literals` literal-LHS handling
/// The `RawTemporalLiteral`-on-LHS case (a literal as `lhs`, field as `rhs`) is STRUCTURALLY
/// UNREACHABLE: it can only fire if the PrismQL parser produces an AST where a raw temporal
/// literal appears as the left operand — but the grammar never does this (quoted strings are
/// rejected as LHS values at parse time). The case is unreachable under the current grammar.
///
/// # Consequence for `check_temporal_literals`
/// The `RawTemporalLiteral`-on-LHS case is structurally unreachable in the AST-walk
/// path as well: the walk resolves the column from the `Predicate::Compare` lhs/rhs
/// positions, expecting the field to be on the left. Since the grammar never produces
/// a `RawTemporalLiteral` on the LHS (it's always the RHS), this case is harmless but
/// should be documented.
///
/// # What happens if an analyst writes this in SQL to DataFusion?
/// `SELECT * FROM t WHERE '2026-06-24' < timestamp` — PrismQL parser rejects the
/// WHERE predicate (non-temporal parse error), `check_temporal_literals` returns `Ok(())`,
/// then DataFusion processes the query. DataFusion will likely fail with a type error
/// (string vs Timestamp) or implicit coerce — but NOT with analyst-friendly E-QUERY-041.
/// This is an accepted limitation of the best-effort gate (ADR-052 D4).
///
/// This test DOCUMENTS the grammar constraint by asserting that literal-LHS is rejected
/// at the PrismQL parse level (not a runtime error during temporal checking).
#[test]
fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_low1_grammar_rejects_literal_lhs_comparison() {
    use crate::filter_parser::PrismQlParser;

    // Filter mode: `'2026-06-24' < timestamp` — literal as LHS.
    // Expected: parse FAILS (non-temporal error — not a FieldPath as LHS).
    let filter_literal_lhs = "'2026-06-24' < timestamp";
    let filter_result = PrismQlParser::parse(filter_literal_lhs);
    assert!(
        filter_result.is_err(),
        "LOW-1 grammar probe: '2026-06-24' < timestamp must FAIL PrismQL parse (literal \
         is not a valid FieldPath LHS). If this passes, the grammar now admits literal-LHS \
         comparisons and the temporal walker's literal-LHS unreachability assumption \
         becomes a real gap requiring check_temporal_literals coverage."
    );

    // Verify the parse error is NOT an E-QUERY-001 temporal error (it's a grammar error).
    let errors = filter_result.unwrap_err();
    let is_temporal_error = errors.iter().any(|e| {
        e.message
            .contains("E-QUERY-001: invalid ISO-8601 timestamp")
    });
    assert!(
        !is_temporal_error,
        "LOW-1 grammar probe: parse error for literal-LHS must NOT be an E-QUERY-001 \
         temporal error. Expected a grammar error (FieldPath expected). Got temporal: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    // SQL mode: `SELECT * FROM t WHERE '2026-06-24' < timestamp`.
    // Expected: parse FAILS (same predicate grammar — literal not a FieldPath as LHS).
    let sql_literal_lhs = "SELECT * FROM t WHERE '2026-06-24' < timestamp";
    let sql_result = PrismQlParser::parse(sql_literal_lhs);
    assert!(
        sql_result.is_err(),
        "LOW-1 grammar probe: SQL WHERE '2026-06-24' < timestamp must FAIL PrismQL parse. \
         If this passes, the grammar now admits literal-LHS in SQL mode — both parse-fail \
         path and AST-ok path need updating for symmetry."
    );

    // Document: the temporal walker's literal-LHS case (lhs=Literal, rhs=Field) is
    // unreachable under the current grammar. This is intentional — see doc comment above.
}

// ── OBS-2: E-QUERY-041 equality/inequality extension ──────────────────────────

/// OBS-2 negative: valid RFC-3339 equality must NOT be rejected by E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE timestamp = '2026-07-04T00:00:00Z'`
///
/// The pre-validator must pass valid RFC-3339 strings through regardless of comparison
/// operator. The query will fail with a sensor error (no sensor wired) but must NOT
/// fail with `TemporalLiteralUnparseable`.
///
/// Traces to: BC-2.11.003 §Postconditions; ADR-052 §D4 valid-pass-through.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_obs2_equality_valid_rfc3339_not_rejected() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp = '2026-07-04T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    // The pre-validator must pass — the query may fail at execution (no sensor wired)
    // but must NOT be TemporalLiteralUnparseable.
    let is_temporal_rejection =
        matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. }));
    assert!(
        !is_temporal_rejection,
        "OBS-2 negative: valid RFC-3339 equality '2026-07-04T00:00:00Z' must NOT trigger \
         E-QUERY-041. The pre-validator must only reject non-RFC-3339 string literals. \
         Got: {result:?}"
    );
}

/// OBS-2 negative: non-datetime column equality must NOT be gated by E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE hostname = 'yesterday'`
///
/// `hostname` is a `ColumnType::String` column. String equality with any value is valid
/// and must not trigger E-QUERY-041, even for strings that look like dates.
///
/// Traces to: ADR-052 §D4 schema-aware, String columns exempt.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_obs2_string_column_equality_not_gated() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = 'yesterday'",
            QueryOptions::default(),
        )
        .await;

    let is_temporal_rejection =
        matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. }));
    assert!(
        !is_temporal_rejection,
        "OBS-2 negative: String column equality must NOT trigger E-QUERY-041 (schema-aware gate). \
         hostname is ColumnType::String — only Datetime columns are gated. Got: {result:?}"
    );
}

// ── F-LOCAL-LOW-1: pipe/filter-mode WITHOUT `FROM`, date-like literals ────────

/// F-LOCAL-LOW-1 Red Gate (c): Pipe-without-FROM plain-table date-only literal must raise
/// E-QUERY-041.
///
/// Query: `test_events | timestamp > '2026-06-24'`
///
/// The source is `test_events` (bare identifier, `SourceRefKind::Custom`) preceding the `|`
/// with NO `FROM` keyword. `'2026-06-24'` is date-like: the PrismQL parser's
/// `classify_string_literal` produces a `RawTemporalLiteral`. The Option-A
/// `check_temporal_literals` AST-walk resolves `test_events` via
/// `primary_table_from_ast` → `Ast::Pipe` → `SourceRefKind::Custom` → `source.raw`,
/// looks up `timestamp` in the `TableRegistry`, finds `ColumnType::Datetime`,
/// and fires E-QUERY-041.
///
/// Pre-fix (RED GATE): `check_temporal_literals` Ast::Pipe arm did not correctly handle
/// the `SourceRefKind::Custom` case → table lookup returned `None` → fail-open →
/// E-QUERY-001 (QueryParseFailed) propagated instead of E-QUERY-041.
///
/// Post-fix (GREEN): `primary_table_from_ast` correctly resolves `SourceRefKind::Custom`
/// → `"test_events"` → `ColumnType::Datetime` → E-QUERY-041 fires.
///
/// Traces to: ADR-052 §D4; F-LOCAL-LOW-1 adversary pass finding.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_f_local_low1_pipe_no_from_date_only_raises_e_query_041(
) {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "test_events | timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-LOCAL-LOW-1: pipe-without-FROM date-only literal '2026-06-24' must return \
         Err(E-QUERY-041). Got Ok. \
         Root cause: check_temporal_literals Ast::Pipe arm must resolve SourceRefKind::Custom \
         'test_events' via extract_primary_table_from_ast for the registry lookup."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "F-LOCAL-LOW-1: error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix starting with '2026-06-24'. Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "F-LOCAL-LOW-1: error Display must contain 'E-QUERY-041'. Got: {display}"
    );
}

/// F-LOCAL-LOW-1 negative: Pipe-without-FROM with valid RFC-3339 must NOT raise E-QUERY-041.
///
/// Query: `test_events | timestamp > '2026-07-04T00:00:00Z'`
///
/// `'2026-07-04T00:00:00Z'` is a valid RFC-3339 UTC timestamp. The PrismQL parser's
/// `classify_string_literal` calls `TimestampLiteral::new` (via `parse_from_rfc3339`),
/// which succeeds — emitting `Literal::Datetime`, not `RawTemporalLiteral`. Consequently
/// `check_temporal_literals` finds no `RawTemporalLiteral` nodes and E-QUERY-041 must NOT fire.
///
/// (Any other error — E-QUERY-001, sensor-not-found — is acceptable; only
/// `TemporalLiteralUnparseable` is forbidden here.)
///
/// Traces to: ADR-052 §D4; F-LOCAL-LOW-1 adversary pass finding.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_f_local_low1_pipe_no_from_valid_rfc3339_not_rejected(
) {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "test_events | timestamp > '2026-07-04T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    if let Err(PrismError::TemporalLiteralUnparseable { value_prefix }) = &result {
        panic!(
            "F-LOCAL-LOW-1 negative: valid RFC-3339 '2026-07-04T00:00:00Z' must NOT trigger \
             E-QUERY-041 in pipe-without-FROM mode. \
             Got TemporalLiteralUnparseable with value_prefix={value_prefix:?}."
        );
    }
    // Any other outcome (Ok or a different Err) is acceptable.
}

/// F-LOCAL-LOW-1 Red Gate (d): Pipe-without-FROM dotted-source date-only literal must raise
/// E-QUERY-041.
///
/// Query: `ghost_sensor.devices | timestamp > '2026-06-24'`
///
/// The source is `ghost_sensor.devices` (dotted External form, `SourceRefKind::External`)
/// preceding the `|` with NO `FROM` keyword. The Option-A `check_temporal_literals`
/// AST-walk uses `primary_table_from_ast` → `Ast::Pipe` → `SourceRefKind::External`
/// → `format!("{sensor}_{table}")` → `"ghost_sensor_devices"` to resolve the registered
/// table name. `timestamp` is `ColumnType::Datetime` → E-QUERY-041 fires.
///
/// Pre-fix (RED GATE): `check_temporal_literals` Ast::Pipe arm did not correctly handle
/// `SourceRefKind::External` dotted-source form → table lookup returned `None` → fail-open
/// → E-QUERY-001 propagated.
///
/// Post-fix (GREEN): `primary_table_from_ast` resolves `SourceRefKind::External`
/// → `"ghost_sensor_devices"` → E-QUERY-041 fires.
///
/// Traces to: ADR-052 §D4; F-LOCAL-LOW-1 adversary pass finding (dotted-source parity).
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_f_local_low1_pipe_no_from_dotted_source_date_only_raises_e_query_041(
) {
    let engine = make_ghost_sensor_engine();

    let result = engine
        .execute(
            "ghost_sensor.devices | timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-LOCAL-LOW-1(d): pipe-without-FROM dotted-source date-only literal '2026-06-24' \
         must return Err(E-QUERY-041). Got Ok. \
         Root cause: check_temporal_literals AST-walk must resolve SourceRefKind::External \
         'ghost_sensor.devices' → 'ghost_sensor_devices' and detect ColumnType::Datetime."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "F-LOCAL-LOW-1(d): error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix starting with '2026-06-24'. Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "F-LOCAL-LOW-1(d): error Display must contain 'E-QUERY-041'. Got: {display}"
    );
}

// ── Option-A typed-column fixture ────────────────────────────────────────────

/// Build a `TableRegistry` with sensor "metrics_sensor" / table "events" registered as
/// "metrics_sensor_events". Includes multiple column types for seven-arm dispatch tests:
///   - `timestamp_col: ColumnType::Datetime` — for E-QUERY-041 tests (Datetime arm)
///   - `label_col: ColumnType::String`       — for coercion tests (String/Utf8 arm)
///   - `count_col: ColumnType::Integer`      — for E-QUERY-002 type-mismatch tests
///   - `ratio_col: ColumnType::Float`        — for E-QUERY-002 type-mismatch tests
///   - `active_col: ColumnType::Boolean`     — for E-QUERY-002 type-mismatch tests
///
/// Used by RG-015/016/017 (Integer/Float/Bool type-mismatch) and RG-013/014 (coerce).
fn make_typed_columns_registry() -> Arc<TableRegistry> {
    use prism_core::ColumnType;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

    let registry = Arc::new(TableRegistry::new());
    let spec = SensorSpec::new(
        "metrics_sensor",
        "Typed column sensor for Option-A seven-arm dispatch tests",
        AuthType::ApiKey,
        "https://metrics.invalid",
        vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![
                ColumnSpec::new("timestamp_col", ColumnType::Datetime, None, vec![]),
                ColumnSpec::new("label_col", ColumnType::String, None, vec![]),
                ColumnSpec::new("count_col", ColumnType::Integer, None, vec![]),
                ColumnSpec::new("ratio_col", ColumnType::Float, None, vec![]),
                ColumnSpec::new("active_col", ColumnType::Boolean, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&spec)
        .expect("register metrics_sensor must not fail");
    registry
}

/// Build a `QueryEngine` wired with the "metrics_sensor_events" typed-column registry.
fn make_typed_columns_engine() -> QueryEngine {
    let registry = make_typed_columns_registry();
    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

// ── RG-011 (stub k): full RFC-3339 regression guard ──────────────────────────

/// RG-011 (stub k): Full RFC-3339 UTC timestamp MUST be emitted as `Literal::Timestamp`
/// (NOT `RawTemporalLiteral`) and MUST NOT trigger E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-07-03T00:00:00Z'`
///
/// # Pre-implementation state (Red Gate)
/// `check_temporal_literals` is `todo!()` → panics for ALL queries that reach it.
/// The valid RFC-3339 string parses successfully (no parse error) → the engine reaches
/// `check_temporal_literals` → PANIC → test FAILS. ✓
///
/// # Post-implementation state (ADR-052 §D4 Step 2)
/// `classify_string_literal("2026-07-03T00:00:00Z")`:
///   - `TimestampLiteral::new("2026-07-03T00:00:00Z")` → SUCCEEDS → `Literal::Timestamp`
///   - NOT `Literal::RawTemporalLiteral` (only date-like, non-RFC-3339 strings become RawTemporal)
/// `check_temporal_literals`: finds no `RawTemporalLiteral` in AST → returns `Ok(())`.
/// Query may fail with sensor error (no real sensor wired) but MUST NOT return
/// `PrismError::TemporalLiteralUnparseable`.
///
/// # Why load-bearing
/// Regression guard: most analyst queries use full RFC-3339 (injected by `NOW() - INTERVAL`).
/// Any parser change that accidentally classifies full RFC-3339 as `RawTemporalLiteral`
/// would break all temporal queries silently. This test catches that regression.
///
/// Traces to: BC-2.11.003 §Valid accepted forms; BC-2.11.021 §Postconditions;
/// ADR-052 §D4 Step 2.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_full_rfc3339_regression_guard() {
    let engine = make_test_engine();

    // Red Gate: check_temporal_literals is todo!() — panics for this query.
    // Post-implementation: check_temporal_literals finds no RawTemporalLiteral → passes.
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-07-03T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    // The temporal pre-validator MUST NOT reject a valid RFC-3339 UTC timestamp.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-011: valid RFC-3339 '2026-07-03T00:00:00Z' must NOT trigger E-QUERY-041. \
         check_temporal_literals must leave Literal::Timestamp nodes untouched. \
         Got: {result:?}"
    );
}

// ── RG-012 (stub m): offset-less T-sep datetime vs Datetime col ───────────────

/// RG-012 (stub m): Offset-less T-sep datetime string (form 2: `%Y-%m-%dT%H:%M:%S`)
/// against a `ColumnType::Datetime` column MUST trigger E-QUERY-041 at plan time.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00:00'`
///
/// # Pre-implementation state (Red Gate)
/// `classify_string_literal("2026-06-24T12:00:00")` → `looks_like_timestamp = true` →
/// `TimestampLiteral::new("2026-06-24T12:00:00")` → FAILS (no UTC offset) → parse ERROR →
/// `PrismError::QueryParseFailed`. The test asserts `TemporalLiteralUnparseable` → FAILS. ✓
///
/// # Post-implementation state (ADR-052 §D4 Steps 2-3)
/// `is_date_like("2026-06-24T12:00:00")` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")`
/// → SUCCEEDS → `Literal::RawTemporalLiteral("2026-06-24T12:00:00")`.
/// `check_temporal_literals` → `timestamp` column is `ColumnType::Datetime` → E-QUERY-041.
///
/// # Why load-bearing
/// Offset-less ISO datetime is the second most common malformed form in analyst queries.
/// The `is_date_like` heuristic MUST cover both `NaiveDate` (form 1) and `NaiveDateTime`
/// (forms 2-7) patterns to gate all date-like inputs.
///
/// Traces to: BC-2.11.021 EC-11-021-009; ADR-052 §D4 form 2.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_offset_less_datetime_col() {
    let engine = make_test_engine();

    // Red Gate: parse currently fails (E-QUERY-001 QueryParseFailed) for '2026-06-24T12:00:00'.
    // Post-implementation: parser emits RawTemporalLiteral; check_temporal_literals
    // returns TemporalLiteralUnparseable (E-QUERY-041).
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-012: offset-less T-sep '2026-06-24T12:00:00' vs Datetime col must return \
         Err(E-QUERY-041). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24T12:00:00")
        ),
        "RG-012: must be TemporalLiteralUnparseable with value_prefix '2026-06-24T12:00:00'. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "RG-012: display must contain 'E-QUERY-041'. Got: {display}"
    );
}

// ── RG-013 (stub n): String col coercion, date-only form ─────────────────────

/// RG-013 (stub n): Date-only literal (form 1) against a `ColumnType::String` column
/// MUST be coerced in-place to `Literal::String` — NOT rejected with E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE hostname = '2026-06-24'`
///
/// # Pre-implementation state (Red Gate)
/// `classify_string_literal("2026-06-24")` → `looks_like_timestamp = true` →
/// `TimestampLiteral::new("2026-06-24")` → FAILS → `PrismError::QueryParseFailed`.
/// The test asserts "NOT QueryParseFailed" → FAILS. ✓
///
/// # Post-implementation state (ADR-052 §D4 Step 3 coercion arm; RISK-5)
/// `is_date_like("2026-06-24")` → `true` (form 1) → `Literal::RawTemporalLiteral`.
/// `check_temporal_literals` resolves `hostname` → `ColumnType::String` → COERCE:
/// rewrites `RawTemporalLiteral("2026-06-24")` to `Literal::String("2026-06-24")` in-place.
/// Query proceeds; emitted SQL: `hostname = '2026-06-24'` (byte-identical to pre-ADR-052).
/// Result is Ok or sensor-level error, NOT `QueryParseFailed` or `TemporalLiteralUnparseable`.
///
/// # Why load-bearing (RISK-5 regression guard)
/// Without the coercion arm, every analyst query filtering on a date-like label string
/// (e.g., `WHERE log_date = '2026-06-24'`) against a String column would break. This is
/// a silent correctness regression that must not happen.
///
/// # FIX-3 (OBS-1) byte-identity assertion
/// The body also drives the coercion through the pipe SQL emitter and asserts the emitted
/// fragment is `hostname = '2026-06-24'` byte-identical to pre-ADR-052, making the claim
/// in the docstring load-bearing (not just absence-of-error).
///
/// Traces to: ADR-052 §D4 coercion arm (RISK-5 RESOLVED BY DESIGN);
/// BC-2.11.021 EC-11-021-013; BC-2.11.003 EC-11-003-001.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_date_only_succeeds() {
    let engine = make_test_engine();

    // Red Gate: parse currently fails (QueryParseFailed) for date-like '2026-06-24'.
    // Post-implementation: parser emits RawTemporalLiteral; check_temporal_literals
    // resolves hostname → String → COERCE; no parse error.
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Post-implementation: must NOT be a parse error (coercion succeeds at plan time).
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-013: date-only '2026-06-24' vs String col must NOT return QueryParseFailed. \
         Under Option-A the parser emits RawTemporalLiteral (not a parse error); \
         plan-time coercion rewrites to Literal::String. Got: {result:?}"
    );

    // Must NOT be E-QUERY-041 (String column is exempt from temporal gating).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-013: date-only '2026-06-24' vs String col must NOT return E-QUERY-041. \
         The coercion arm rewrites RawTemporalLiteral → Literal::String. Got: {result:?}"
    );

    // FIX-3 (OBS-1): Drive through the pipe SQL emitter to prove byte-identity.
    // Constructs the post-parse AST directly (RawTemporalLiteral in WHERE predicate),
    // applies check_temporal_literals (coerces to Literal::String), then
    // asserts the emitted SQL fragment is `hostname = '2026-06-24'`.
    {
        use crate::ast::{
            Ast, CompareOp, Expr, FieldPath, Literal, PipeQuery, PipeStage, Predicate, SourceRef,
        };
        use crate::materialization::check_temporal_literals;
        use crate::pipe_sql_emitter::pipe_to_executable_sql;

        let registry = make_test_events_registry();

        // Build Pipe-mode AST with RawTemporalLiteral in WHERE (simulates post-parse AST).
        let pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["hostname"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::RawTemporalLiteral(
                "2026-06-24".to_string(),
            ))),
            case_insensitive: false,
        };
        let mut ast = Ast::Pipe(PipeQuery::new(
            SourceRef::from_raw("test_events"),
            vec![PipeStage::Where(pred)],
        ));

        // Run check_temporal_literals: coerces RawTemporalLiteral("2026-06-24")
        // vs String column → Literal::String("2026-06-24") in-place.
        check_temporal_literals(&mut ast, Some(registry.as_ref()), false)
            .expect("RG-013 byte-identity: coercion must not fail for String column");

        // Extract PipeQuery after coercion and emit SQL via pipe_to_executable_sql.
        let pq = if let Ast::Pipe(pq) = ast {
            pq
        } else {
            panic!("RG-013 byte-identity: expected Pipe AST after coercion");
        };
        let sql = pipe_to_executable_sql(&pq, &Default::default())
            .expect("RG-013 byte-identity: emitter must succeed after coercion");

        // The emitted WHERE fragment must be byte-identical to pre-ADR-052 behavior.
        // The coerced Literal::String emits `'2026-06-24'`, not `arrow_cast(...)`.
        assert!(
            sql.contains("WHERE hostname = '2026-06-24'"),
            "RG-013 byte-identity: coerced String literal must emit \
             `hostname = '2026-06-24'` (byte-identical to pre-ADR-052). Got: {sql}"
        );
    }
}

// ── RG-014 (stub o): String col coercion, offset-less T-sep form ──────────────

/// RG-014 (stub o): Offset-less T-sep literal (form 2) against String col → COERCE.
///
/// Query: `SELECT * FROM test_events WHERE hostname = '2026-06-24T12:00:00'`
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24T12:00:00'`. Test asserts "NOT
/// QueryParseFailed" → FAILS. ✓
///
/// # Post-implementation state
/// `is_date_like("2026-06-24T12:00:00") = true` (form 2) → `RawTemporalLiteral`.
/// `check_temporal_literals` → `hostname` is String → COERCE → `Literal::String`.
/// Emitted SQL: `hostname = '2026-06-24T12:00:00'` (byte-identical to pre-ADR-052).
///
/// # FIX-3 (OBS-1) byte-identity assertion
/// The body also drives the coercion through the pipe SQL emitter and asserts the emitted
/// fragment is `hostname = '2026-06-24T12:00:00'` byte-identical to pre-ADR-052.
///
/// Traces to: ADR-052 §D4 coercion arm; BC-2.11.021 EC-11-021-013.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_offset_less_succeeds() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '2026-06-24T12:00:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-014: offset-less '2026-06-24T12:00:00' vs String col must NOT return \
         QueryParseFailed. Coercion must succeed at plan time. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-014: offset-less '2026-06-24T12:00:00' vs String col must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    // FIX-3 (OBS-1): Drive through the pipe SQL emitter to prove byte-identity.
    {
        use crate::ast::{
            Ast, CompareOp, Expr, FieldPath, Literal, PipeQuery, PipeStage, Predicate, SourceRef,
        };
        use crate::materialization::check_temporal_literals;
        use crate::pipe_sql_emitter::pipe_to_executable_sql;

        let registry = make_test_events_registry();

        // Build Pipe-mode AST with RawTemporalLiteral in WHERE (simulates post-parse AST).
        let pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["hostname"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::RawTemporalLiteral(
                "2026-06-24T12:00:00".to_string(),
            ))),
            case_insensitive: false,
        };
        let mut ast = Ast::Pipe(PipeQuery::new(
            SourceRef::from_raw("test_events"),
            vec![PipeStage::Where(pred)],
        ));

        // Run check_temporal_literals: coerces RawTemporalLiteral("2026-06-24T12:00:00")
        // vs String column → Literal::String("2026-06-24T12:00:00") in-place.
        check_temporal_literals(&mut ast, Some(registry.as_ref()), false)
            .expect("RG-014 byte-identity: coercion must not fail for String column");

        // Extract PipeQuery after coercion and emit SQL via pipe_to_executable_sql.
        let pq = if let Ast::Pipe(pq) = ast {
            pq
        } else {
            panic!("RG-014 byte-identity: expected Pipe AST after coercion");
        };
        let sql = pipe_to_executable_sql(&pq, &Default::default())
            .expect("RG-014 byte-identity: emitter must succeed after coercion");

        // The emitted WHERE fragment must be byte-identical to pre-ADR-052 behavior.
        assert!(
            sql.contains("WHERE hostname = '2026-06-24T12:00:00'"),
            "RG-014 byte-identity: coerced String literal must emit \
             `hostname = '2026-06-24T12:00:00'` (byte-identical to pre-ADR-052). Got: {sql}"
        );
    }
}

// ── RG-015 (stub p): Integer col type-mismatch ────────────────────────────────

/// RG-015 (stub p): Date-like literal against `ColumnType::Integer` column MUST return
/// E-QUERY-002 (NOT E-QUERY-041 — E-QUERY-041 is only for Datetime columns).
///
/// Query: `SELECT * FROM metrics_sensor_events WHERE count_col = '2026-06-24'`
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24'`. Test asserts "NOT QueryParseFailed"
/// → FAILS. ✓
///
/// # Post-implementation state (ADR-052 §D4 Step 3 third arm)
/// `is_date_like = true` → `RawTemporalLiteral`. `check_temporal_literals` resolves
/// `count_col` → `ColumnType::Integer` → returns E-QUERY-002 (type mismatch, NOT E-QUERY-041).
///
/// # Why load-bearing
/// The seven-arm dispatch must be exhaustive: Integer (and Float, Bool) columns must route
/// to E-QUERY-002, not E-QUERY-041. Incorrect routing would mislead the analyst with a
/// wrong error message ("cannot interpret as UTC timestamp" for a type that never holds
/// timestamps).
///
/// Traces to: ADR-052 §D4 Step 3 third arm; BC-2.11.021 §Postconditions.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_integer_col_date_like_e_query_002() {
    let engine = make_typed_columns_engine();

    // Red Gate: parse fails (QueryParseFailed) for '2026-06-24'.
    // Post-implementation: RawTemporalLiteral; count_col is Integer → E-QUERY-002 (QueryTypeMismatch),
    // not E-QUERY-041.
    let result = engine
        .execute(
            "SELECT * FROM metrics_sensor_events WHERE count_col = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-015: date-like '2026-06-24' vs Integer col must return an error. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Must NOT be a raw parse error (Option-A parser emits RawTemporalLiteral successfully).
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "RG-015: Integer col type-mismatch must NOT be a parse error (parse should succeed \
         under Option-A). Got: {display}"
    );

    // Must NOT be E-QUERY-041 (only Datetime columns trigger the temporal gate).
    assert!(
        !matches!(&err, PrismError::TemporalLiteralUnparseable { .. }),
        "RG-015: date-like vs Integer col must return E-QUERY-002, NOT E-QUERY-041. \
         Got: {display}"
    );

    // Must be QueryTypeMismatch (E-QUERY-002) — the structured type-mismatch variant.
    // ADR-052 §D4 v1.5 + BC-2.11.021 + error-taxonomy v2.12 all specify E-QUERY-002
    // for numeric/bool type-mismatch (distinguishes from E-QUERY-001 QueryParseFailed).
    assert!(
        matches!(&err, PrismError::QueryTypeMismatch { .. }),
        "RG-015: date-like vs Integer col must return QueryTypeMismatch (E-QUERY-002). \
         Got: {display}"
    );
}

// ── RG-016 (stub q): Float col type-mismatch ─────────────────────────────────

/// RG-016 (stub q): Date-like literal against `ColumnType::Float` column → E-QUERY-002.
///
/// Same dispatch pattern as RG-015 (arm 3 of the seven-arm dispatch) but for Float type.
///
/// Traces to: ADR-052 §D4 v1.10 arm (3); ADR-052 §D4 Step 3 third arm.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_float_col_date_like_e_query_002() {
    let engine = make_typed_columns_engine();

    let result = engine
        .execute(
            "SELECT * FROM metrics_sensor_events WHERE ratio_col = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-016: date-like '2026-06-24' vs Float col must return an error. Got Ok."
    );

    let err = result.unwrap_err();

    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "RG-016: Float col type-mismatch must NOT be a parse error. Got: {err:?}"
    );

    assert!(
        !matches!(&err, PrismError::TemporalLiteralUnparseable { .. }),
        "RG-016: date-like vs Float col must return E-QUERY-002, NOT E-QUERY-041. Got: {err:?}"
    );

    // Must be QueryTypeMismatch (E-QUERY-002) — the structured type-mismatch variant.
    assert!(
        matches!(&err, PrismError::QueryTypeMismatch { .. }),
        "RG-016: date-like vs Float col must return QueryTypeMismatch (E-QUERY-002). Got: {err:?}"
    );
}

// ── RG-017 (stub r): Boolean col type-mismatch ───────────────────────────────

/// RG-017 (stub r): Date-like literal against `ColumnType::Boolean` column → E-QUERY-002.
///
/// Same dispatch pattern as RG-015 (arm 3 of the seven-arm dispatch) but for Boolean type.
///
/// Traces to: ADR-052 §D4 v1.10 arm (3); ADR-052 §D4 Step 3 third arm.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_bool_col_date_like_e_query_002() {
    let engine = make_typed_columns_engine();

    let result = engine
        .execute(
            "SELECT * FROM metrics_sensor_events WHERE active_col = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-017: date-like '2026-06-24' vs Bool col must return an error. Got Ok."
    );

    let err = result.unwrap_err();

    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "RG-017: Bool col type-mismatch must NOT be a parse error. Got: {err:?}"
    );

    assert!(
        !matches!(&err, PrismError::TemporalLiteralUnparseable { .. }),
        "RG-017: date-like vs Bool col must return E-QUERY-002, NOT E-QUERY-041. Got: {err:?}"
    );

    // Must be QueryTypeMismatch (E-QUERY-002) — the structured type-mismatch variant.
    assert!(
        matches!(&err, PrismError::QueryTypeMismatch { .. }),
        "RG-017: date-like vs Bool col must return QueryTypeMismatch (E-QUERY-002). Got: {err:?}"
    );
}

// ── RG-018 (stub s): non-date-like stays String literal ───────────────────────

/// RG-018 (stub s): Non-date-like string literal MUST remain `Literal::String`
/// (NOT `RawTemporalLiteral`) — no temporal error emitted.
///
/// Query: `SELECT * FROM test_events WHERE hostname = 'not-a-date'`
///
/// # Pre-implementation state (Red Gate)
/// `'not-a-date'` does NOT start with 4 ASCII digits → `looks_like_timestamp = false` →
/// parse SUCCEEDS as `Literal::String`. The engine then reaches `check_temporal_literals`
/// (todo!()) → PANICS → test FAILS. ✓
///
/// # Post-implementation state
/// `is_date_like("not-a-date")` → `false` (no chrono pattern matches). Parser emits
/// `Literal::String("not-a-date")`. `check_temporal_literals` finds no
/// `RawTemporalLiteral` → returns `Ok(())`. No temporal error.
///
/// Traces to: ADR-052 §D4 Step 2 (heuristic negative case);
/// BC-2.11.003 §Non-date-like forms (EC-005).
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_date_like_stays_string_literal() {
    let engine = make_test_engine();

    // Red Gate: check_temporal_literals is todo!() — panics for all queries that reach it.
    // 'not-a-date' parses fine (not date-like) → the engine reaches check_temporal_literals
    // → PANIC → test FAILS. ✓
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = 'not-a-date'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be E-QUERY-041 (non-date-like strings must pass through without temporal gating).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-018: non-date-like 'not-a-date' must NOT trigger E-QUERY-041. \
         is_date_like must return false for this input. Got: {result:?}"
    );

    // Also check 'sensor-id-abc' — another typical non-date sensor identifier.
    let result2 = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = 'sensor-id-abc'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(&result2, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-018: non-date-like 'sensor-id-abc' must NOT trigger E-QUERY-041. Got: {result2:?}"
    );
}

// ── RG-019 (stub t): dotted source column resolution ─────────────────────────

/// RG-019 (stub t): Dotted-column reference `ghost_sensor_devices.timestamp`
/// must be resolved against the `ghost_sensor_devices` schema → Datetime col → E-QUERY-041.
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24'`. Test asserts
/// `TemporalLiteralUnparseable` → FAILS. ✓
///
/// # Post-implementation state
/// `check_temporal_literals` resolves the 2-segment FieldPath
/// `[ghost_sensor_devices, timestamp]` against the registered schema →
/// `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: ADR-052 §D4 Step 3 (dotted column resolved via schema, not text-split);
/// BC-2.11.021 EC-11-021-009.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_dotted_source_column_resolution() {
    let engine = make_ghost_sensor_engine();

    // ghost_sensor_devices.timestamp is ColumnType::Datetime
    let result = engine
        .execute(
            "SELECT * FROM ghost_sensor_devices WHERE ghost_sensor_devices.timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-019: dotted column 'ghost_sensor_devices.timestamp' vs date-only must return \
         Err(E-QUERY-041). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "RG-019: must be TemporalLiteralUnparseable with value_prefix '2026-06-24'. \
         check_temporal_literals must resolve ghost_sensor_devices.timestamp → \
         Datetime via schema (NOT string-split on '.'). Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "RG-019: display must contain 'E-QUERY-041'. Got: {display}"
    );
}

// ── RG-020 (stub u): qualified nested column → String col coercion ────────────

/// RG-020 (stub u): Qualified column reference `ghost_sensor_devices.hostname`
/// must be resolved against the `ghost_sensor_devices` schema → String col → COERCE.
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24'`. Test asserts "NOT QueryParseFailed"
/// → FAILS. ✓
///
/// # Post-implementation state
/// `check_temporal_literals` resolves 2-segment FieldPath `[ghost_sensor_devices, hostname]`
/// against the schema → `ColumnType::String` → COERCE → no E-QUERY-041.
/// The qualified column is looked up in the CORRECT source's schema (ghost_sensor_devices),
/// not collapsed to the last segment (`hostname`) and resolved in any arbitrary table.
///
/// Traces to: ADR-052 §D4 Step 3 (qualified column via schema map);
/// BC-2.11.021.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_qualified_nested_column_resolution() {
    let engine = make_ghost_sensor_engine();

    // ghost_sensor_devices.hostname is ColumnType::String → must COERCE, not E-QUERY-041
    let result = engine
        .execute(
            "SELECT * FROM ghost_sensor_devices WHERE ghost_sensor_devices.hostname = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-020: qualified String column with date-like literal must NOT return QueryParseFailed. \
         Coercion must succeed at plan time (NOT a parse error). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-020: qualified String column must be COERCED, NOT return E-QUERY-041. \
         ghost_sensor_devices.hostname is ColumnType::String. Got: {result:?}"
    );
}

// ── RG-021 (stub v): equality operator temporal gate ─────────────────────────

/// RG-021 (stub v): Date-like literal in an EQUALITY comparison against a
/// `ColumnType::Datetime` column MUST trigger E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE timestamp = '2026-06-24'`
///
/// Tests that the equality operator (`=`) is also gated by the temporal pre-validator,
/// not only ordering operators (`>`, `<`, `>=`, `<=`). Under Option-A, the seven-arm
/// dispatch fires for ALL comparison operators when the LHS column is Datetime.
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24'`. Test asserts
/// `TemporalLiteralUnparseable` → FAILS. ✓
///
/// # Post-implementation state
/// `RawTemporalLiteral("2026-06-24")` → `timestamp` is Datetime → E-QUERY-041 regardless
/// of whether the comparison operator is ordering or equality.
///
/// Traces to: error-taxonomy.md §E-QUERY-041 ("compared against a bare string literal" —
/// not operator-specific); ADR-052 §D4 Step 3.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_filter_pipe_syntax_e_query_041() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-021: equality `timestamp = '2026-06-24'` against Datetime col must return \
         Err(E-QUERY-041). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "RG-021: equality operator with date-only literal vs Datetime col must return \
         TemporalLiteralUnparseable. The temporal gate must fire for '=' (not only '>/<'). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-041"),
        "RG-021: display must contain 'E-QUERY-041'. Got: {display}"
    );
}

// ── RG-022 (stub w): Unicode input no-panic (VP-021) ─────────────────────────

/// RG-022 (stub w): Query containing multi-byte Unicode characters adjacent to a
/// date-like literal MUST NOT panic. Asserts E-QUERY-041 fires without byte-offset panic.
///
/// # VP-021 regression guard
/// The old text-scanner VP-021 violation caused raw byte-offset slicing on multi-byte
/// UTF-8 strings, triggering SIGBUS / panic on non-UTF-8-safe offsets. Under Option-A,
/// `check_temporal_literals` operates on already-parsed `String` values (valid
/// UTF-8 by construction). No raw byte-offset slicing ever occurs — VP-021 is eliminated
/// by construction.
///
/// # Pre-implementation state (Red Gate)
/// The parser fails for `'2026-06-24'` (date-like string → `QueryParseFailed`). The test
/// asserts `TemporalLiteralUnparseable` → FAILS. ✓
///
/// # Post-implementation state
/// `hostname = '日本語'` → `Literal::String("日本語")` (non-date-like, `looks_like_timestamp = false`).
/// `timestamp > '2026-06-24'` → `Literal::RawTemporalLiteral("2026-06-24")`.
/// `check_temporal_literals` → `timestamp` is Datetime → E-QUERY-041.
/// No panic (VP-021 satisfied by construction).
///
/// Traces to: VP-021 (never panics on multi-byte input); ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_unicode_input_no_panic() {
    let engine = make_test_engine();

    // Unicode in hostname value (multi-byte UTF-8) adjacent to date-like temporal literal.
    // The '日本語' characters are 3 bytes each in UTF-8; the old text-scanner VP-021 violation
    // would panic if it sliced at byte offsets around multi-byte chars in the query string.
    // Under Option-A, all operations are on already-parsed String (valid UTF-8) — no panic path.
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '\u{65e5}\u{672c}\u{8a9e}' AND timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // If we reach here without panic, VP-021 is satisfied.
    // Post-implementation: timestamp > '2026-06-24' → E-QUERY-041 (Datetime col).
    assert!(
        result.is_err(),
        "RG-022: Unicode query with date-only timestamp literal must return an error. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // The temporal literal '2026-06-24' against Datetime col must produce E-QUERY-041.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "RG-022: unicode query must produce E-QUERY-041 for date-only vs Datetime col \
         (VP-021: no panic on multi-byte Unicode input). Got: {err:?} (Display: {display})"
    );
}

// ── RG-023: projection-position RawTemporalLiteral → COERCE to Literal::String ────

/// RG-023 (OBS-2): `RawTemporalLiteral` in a projection (SELECT) position without a
/// comparison context MUST be COERCED to `Literal::String` (ADR-052 §D4 OBS-2).
///
/// The query `SELECT '2026-06-24' FROM test_events` succeeds — the date-like literal
/// is treated as a plain string constant when there is no column type to constrain it.
///
/// # Spec change (OBS-2, ratified 2026-07-05)
/// ADR-052 §D4 said: non-comparison position → E-QUERY-002 (QueryPlanFailed).
/// ADR-052 §D4 says: non-comparison position → COERCE to Literal::String.
/// `check_expr_temporal` coerces the bare `RawTemporalLiteral` in-place.
///
/// # Pre-implementation state (Red Gate for OBS-2)
/// Code returns E-QUERY-002 `QueryPlanFailed` for bare literal in SELECT.
/// Test asserts `result.is_ok()` → FAILS. ✓
///
/// # Post-implementation state
/// `check_expr_temporal` coerces `RawTemporalLiteral("2026-06-24")` → `Literal::String("2026-06-24")`.
/// Query continues; DataFusion executes `SELECT '2026-06-24' FROM test_events` normally.
/// Result: Ok(QueryResult { rows: [] }) — 0 rows (no real sensor in test engine), no error.
///
/// Traces to: ADR-052 §D4 OBS-2; BC-2.11.021 §Postconditions.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_position_coerces_to_string() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT '2026-06-24' FROM test_events",
            QueryOptions::default(),
        )
        .await;

    // OBS-2: must NOT be a parse error.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-023: '2026-06-24' in SELECT must NOT return QueryParseFailed under Option-A. \
         Got: {result:?}"
    );

    // OBS-2: must NOT be E-QUERY-041.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-023: RawTemporalLiteral in projection must NOT trigger E-QUERY-041. \
         Got: {result:?}"
    );

    // OBS-2: must NOT be E-QUERY-002 (QueryPlanFailed) — OBS-2 coercion must fire instead.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "RG-023: RawTemporalLiteral in SELECT projection must NOT return QueryPlanFailed \
         (E-QUERY-002) — OBS-2 requires coerce-to-Literal::String. \
         Got: {result:?}"
    );

    // OBS-2: query must SUCCEED — bare literal in SELECT is coerced to a string constant.
    assert!(
        result.is_ok(),
        "RG-023: SELECT '2026-06-24' FROM test_events must succeed (OBS-2 coerce). \
         Got Err: {result:?}"
    );
}

// ── RG-025 (stub z): E-QUERY-041 message format byte-identical (POL-24) ──────

/// RG-025 (stub z): The `Display` string of `PrismError::TemporalLiteralUnparseable`
/// for `value_prefix = "2026-06-24"` MUST match the POL-24 canonical template byte-for-byte.
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24'`. The test tries to unwrap
/// `TemporalLiteralUnparseable` → the result is not TemporalLiteralUnparseable →
/// assertion FAILS. ✓
///
/// # Post-implementation state (POL-24)
/// Triggers E-QUERY-041 with `value_prefix = "2026-06-24"` and asserts the Display string
/// matches the canonical message template from error-taxonomy.md §E-QUERY-041 exactly.
///
/// Traces to: error-taxonomy.md §E-QUERY-041 (POL-24 byte-for-byte message contract);
/// BC-2.11.001 (MCP contract); ADR-052 §D4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_message_format_byte_identical() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-025: '2026-06-24' vs Datetime col must return an error. Got Ok."
    );

    let err = result.unwrap_err();

    // First verify it IS the temporal error.
    assert!(
        matches!(&err, PrismError::TemporalLiteralUnparseable { .. }),
        "RG-025: must be TemporalLiteralUnparseable. Got: {err:?}"
    );

    // Then verify the exact Display format (POL-24 byte-for-byte).
    let display = format!("{err}");
    let expected = "E-QUERY-041: The value '2026-06-24' cannot be interpreted as a UTC timestamp. \
         Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only \
         and offset-less forms are not accepted. For relative time filters, use \
         NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h').";

    assert_eq!(
        display, expected,
        "RG-025 POL-24: E-QUERY-041 Display must match the canonical template EXACTLY \
         (byte-for-byte). Any deviation (extra whitespace, punctuation change, reordered \
         clauses) is an MCP contract break. \
         Expected:\n  {expected:?}\nGot:\n  {display:?}"
    );
}

// ── RG-026 (stub aa): form 3 — T-sep fractional vs Datetime ──────────────────

/// RG-026 (stub aa): Fractional-seconds T-sep literal (form 3: `%Y-%m-%dT%H:%M:%S%.f`)
/// vs `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: BC-2.11.021 EC-11-021-011; ADR-052 §D4 form 3.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_fractional_t_sep_datetime_col() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00:00.123'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-026: fractional T-sep '2026-06-24T12:00:00.123' vs Datetime must return E-QUERY-041. \
         Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24T12:00:00.123")
        ),
        "RG-026: must be TemporalLiteralUnparseable (form 3 fractional T-sep). Got: {err:?}"
    );
}

// ── RG-027 (stub ab): form 4 — T-sep no-seconds vs Datetime ──────────────────

/// RG-027 (stub ab): T-sep no-seconds literal (form 4: `%Y-%m-%dT%H:%M`)
/// vs `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: BC-2.11.021 EC-11-021-010; ADR-052 §D4 form 4.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_no_seconds_t_sep_datetime_col() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24T12:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-027: T-sep no-seconds '2026-06-24T12:00' vs Datetime must return E-QUERY-041. Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24T12:00")
        ),
        "RG-027: must be TemporalLiteralUnparseable (form 4 T-sep no-seconds). Got: {err:?}"
    );
}

// ── RG-028 (stub ac): form 5 — space-sep full-seconds vs Datetime ─────────────

/// RG-028 (stub ac): Space-sep full-seconds literal (form 5: `%Y-%m-%d %H:%M:%S`)
/// vs `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: BC-2.11.021 EC-11-021-012; ADR-052 §D4 form 5.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_full_seconds_datetime_col()
{
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24 12:00:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-028: space-sep full-seconds '2026-06-24 12:00:00' vs Datetime must return \
         E-QUERY-041. Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24 12:00:00")
        ),
        "RG-028: must be TemporalLiteralUnparseable (form 5 space-sep full-seconds). Got: {err:?}"
    );
}

// ── RG-029 (stub ad): form 6 — space-sep fractional vs Datetime ───────────────

/// RG-029 (stub ad): Space-sep fractional-seconds literal (form 6: `%Y-%m-%d %H:%M:%S%.f`)
/// vs `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: ADR-052 §D4 form 6.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_fractional_datetime_col() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24 12:00:00.500'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-029: space-sep fractional '2026-06-24 12:00:00.500' vs Datetime must return \
         E-QUERY-041. Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24 12:00:00.500")
        ),
        "RG-029: must be TemporalLiteralUnparseable (form 6 space-sep fractional). Got: {err:?}"
    );
}

// ── RG-030 (stub ae): form 7 — space-sep no-seconds vs Datetime ───────────────

/// RG-030 (stub ae): Space-sep no-seconds literal (form 7: `%Y-%m-%d %H:%M`)
/// vs `ColumnType::Datetime` → E-QUERY-041.
///
/// Traces to: ADR-052 §D4 form 7.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_no_seconds_datetime_col() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24 12:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-030: space-sep no-seconds '2026-06-24 12:00' vs Datetime must return E-QUERY-041. \
         Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24 12:00")
        ),
        "RG-030: must be TemporalLiteralUnparseable (form 7 space-sep no-seconds). Got: {err:?}"
    );
}

// ── RG-031 (stub af): space-sep form 5 vs String col → COERCE ────────────────

/// RG-031 (stub af): Space-sep full-seconds literal (form 5) against `ColumnType::String`
/// column MUST be coerced in-place — NOT E-QUERY-041.
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`) for `'2026-06-24 12:00:00'`. Test asserts "NOT
/// QueryParseFailed" → FAILS. ✓
///
/// # Post-implementation state (RISK-5 extension to space-sep family)
/// `is_date_like = true` (form 5) → `RawTemporalLiteral`. `check_temporal_literals`
/// → `hostname` is String → COERCE → `Literal::String("2026-06-24 12:00:00")`.
/// Emitted SQL: `hostname = '2026-06-24 12:00:00'` (byte-identical to pre-ADR-052).
///
/// Traces to: BC-2.11.021 EC-11-021-013; ADR-052 §D4 coercion arm.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_space_sep_succeeds() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '2026-06-24 12:00:00'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-031: space-sep '2026-06-24 12:00:00' vs String col must NOT return QueryParseFailed. \
         Coercion must succeed at plan time. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-031: space-sep form 5 vs String col must be COERCED, NOT E-QUERY-041. Got: {result:?}"
    );
}

// ── RG-032 (stub ag): unpadded date over-match vs Datetime (ACCEPTED BENIGN) ──

/// RG-032 (stub ag): Unpadded single-digit month/day `'2026-6-24'` against
/// `ColumnType::Datetime` → E-QUERY-041 (ACCEPTED BENIGN over-match).
///
/// `chrono::NaiveDate::parse_from_str("2026-6-24", "%Y-%m-%d")` SUCCEEDS because chrono
/// `%m`/`%d` accept single digits — this is the accepted over-match. No regex guard applied.
/// `is_date_like` returns `true` → `RawTemporalLiteral` → E-QUERY-041. The "use RFC-3339"
/// message is accurate and apt (unpadded forms are also non-RFC-3339).
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`). Test asserts `TemporalLiteralUnparseable` → FAILS. ✓
///
/// Traces to: BC-2.11.021 EC-11-021-014; ADR-052 §D4 over-match ACCEPTED BENIGN.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_unpadded_date_overmatch_datetime_col(
) {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-6-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-032: unpadded '2026-6-24' (over-match ACCEPTED BENIGN) vs Datetime must return \
         E-QUERY-041. Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-6-24")
        ),
        "RG-032: must be TemporalLiteralUnparseable for unpadded date. No regex guard applied. \
         Got: {err:?}"
    );
}

// ── FIX-2 regression: projection-position temporal literal with unregistered table ──

/// FIX-2 regression guard: `SELECT '2026-06-24' FROM <unregistered_table>` MUST return
/// E-QUERY-037 (TableNotAvailable), NOT E-QUERY-002 (QueryPlanFailed).
///
/// # Pre-fix state (RED gate)
/// The early temporal gate in `engine::execute` calls the FULL `check_temporal_literals`
/// walker, which includes `check_select_items_raw_temporal`. A bare `RawTemporalLiteral` in
/// SELECT position returns `Err(QueryPlanFailed)` (E-QUERY-002) immediately, BEFORE
/// `check_table_availability` (E-QUERY-037) fires. This is wrong — the more actionable
/// "table not found" error is silenced by the less useful "temporal literal without context".
///
/// # Post-fix state (GREEN)
/// The early gate uses `skip_projection: true`, so SELECT-item / GROUP BY / ORDER BY checks
/// are deferred to the in-pipeline `check_temporal_literals` pass (which only runs after
/// `check_table_availability` confirms the table exists). For an unregistered table,
/// `check_table_availability` fires first and returns E-QUERY-037.
///
/// # EC-013 preservation
/// The early gate still checks WHERE predicates (field-LHS, Datetime column), so dotted
/// external-source queries like `FROM ghost_sensor.devices WHERE timestamp > '2026-06-24'`
/// still return E-QUERY-041 before E-QUERY-037.
///
/// Traces to: ADR-052 §D4 Option A (early gate scoping); BC-2.11.019 §Gate ordering.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_fix2_projection_literal_unregistered_table_yields_e_query_037(
) {
    // "test_events" is registered; "unregistered_table" is NOT registered.
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT '2026-06-24' FROM unregistered_table",
            QueryOptions::default(),
        )
        .await;

    // Must be an error — unregistered table.
    assert!(
        result.is_err(),
        "FIX-2 regression: query against unregistered_table must return an error. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Must be E-QUERY-037 (table not found), NOT E-QUERY-002 (temporal literal without context).
    // Pre-fix: early gate fires E-QUERY-002 before check_table_availability can fire E-QUERY-037.
    assert!(
        display.contains("E-QUERY-037"),
        "FIX-2 regression: unregistered table + projection-position temporal literal must yield \
         E-QUERY-037 (not E-QUERY-002). Early gate must not pre-empt the table availability check. \
         Pre-fix failure: early gate calls full check_temporal_literals (including SELECT \
         item walker) which fires E-QUERY-002 for bare RawTemporalLiteral before E-QUERY-037 \
         fires. Fix: early gate must use skip_projection=true. Got: {display}"
    );
}

// ── RG-033 (stub ah): unpadded date over-match vs String col → COERCE ─────────

/// RG-033 (stub ah): Unpadded date `'2026-6-24'` against `ColumnType::String` col
/// → COERCE (same over-match disposition applied to coercion arm).
///
/// # Pre-implementation state (Red Gate)
/// Parse fails (`QueryParseFailed`). Test asserts "NOT QueryParseFailed" → FAILS. ✓
///
/// # Post-implementation state
/// `is_date_like("2026-6-24") = true` (over-match) → `RawTemporalLiteral`.
/// `check_temporal_literals` → `hostname` is String → COERCE → `Literal::String("2026-6-24")`.
/// Unpadded date labels are valid sensor identifiers in some APIs; coercion is correct.
///
/// Traces to: ADR-052 §D4 coercion arm + over-match disposition;
/// BC-2.11.021.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_unpadded_date_succeeds() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '2026-6-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "RG-033: unpadded '2026-6-24' vs String col must NOT return QueryParseFailed. \
         Coercion must succeed at plan time. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-033: unpadded over-match vs String col must be COERCED, NOT E-QUERY-041. Got: {result:?}"
    );
}

// ── MED-1: SqlPipe RawTemporalLiteral seven-arm dispatch coverage ────────────
//
// ADR-052 §D4 v1.10 column-typed arms (1)-(3) of the seven-arm dispatch
// (Datetime→E-QUERY-041 / String→COERCE / Integer|Float|Bool→E-QUERY-002) apply
// equally to SqlPipe head predicates.
// `check_temporal_literals` Ast::SqlPipe arm walks the head SELECT + WHERE +
// HAVING + JOIN ON + GROUP BY + ORDER BY positions plus each pipe stage.
//
// These tests exercise the SqlPipe code path which was previously zero-coverage.
// The implementation already handles SqlPipe; these tests are the load-bearing
// regression guards that prevent future removal of the SqlPipe arm.
// ─────────────────────────────────────────────────────────────────────────────

/// MED-1 SqlPipe Datetime col: date-only literal in SqlPipe head WHERE clause
/// against a `ColumnType::Datetime` column MUST trigger E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE timestamp > '2026-06-24' | limit 10`
///
/// The head `WHERE timestamp > '2026-06-24'` contains a `RawTemporalLiteral`
/// compared against the `timestamp` Datetime column.  `check_temporal_literals`
/// Ast::SqlPipe arm must walk this position and return `E-QUERY-041`.
///
/// Traces to: BC-2.11.021 §Postconditions; ADR-052 §D4 MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sqlpipe_datetime_col_date_only_raises_e_query_041(
) {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp > '2026-06-24' | limit 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-1 SqlPipe Datetime: SqlPipe head WHERE with date-only '2026-06-24' against \
         Datetime col must return Err(E-QUERY-041). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralUnparseable { value_prefix }
            if value_prefix.starts_with("2026-06-24")
        ),
        "MED-1 SqlPipe Datetime: error must be PrismError::TemporalLiteralUnparseable with \
         value_prefix '2026-06-24'. Got: {err:?} (Display: {display}). \
         If this returns a different error, check_temporal_literals Ast::SqlPipe arm may \
         not be walking the head WHERE clause."
    );

    assert!(
        display.contains("E-QUERY-041"),
        "MED-1 SqlPipe Datetime: error Display must contain 'E-QUERY-041'. Got: {display}"
    );

    assert!(
        !display.contains("Arrow error") && !display.contains("DataFusion"),
        "MED-1 SqlPipe Datetime: E-QUERY-041 must fire at Prism plan time, NOT as a \
         DataFusion error. Got: {display}"
    );
}

/// MED-1 SqlPipe String col: date-only literal in SqlPipe head WHERE clause
/// against a `ColumnType::String` column MUST be coerced (COERCE arm) —
/// NOT rejected with E-QUERY-041.
///
/// Query: `SELECT * FROM test_events WHERE hostname = '2026-06-24' | limit 10`
///
/// `hostname` is a String column.  `check_temporal_literals` must detect the
/// `RawTemporalLiteral` + String-column combination and coerce it to
/// `Literal::String("2026-06-24")`, allowing the query to proceed.
///
/// The query may fail at sensor execution (no real sensor), but it MUST NOT
/// fail with `E-QUERY-041` (that would mean the Datetime arm fired incorrectly
/// against a String column).
///
/// Traces to: BC-2.11.021 §Postconditions coerce arm; ADR-052 §D4 MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sqlpipe_string_col_date_only_coerce_succeeds() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE hostname = '2026-06-24' | limit 10",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be E-QUERY-041 — String column uses COERCE arm.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "MED-1 SqlPipe String col: date-only '2026-06-24' against String col MUST NOT \
         trigger E-QUERY-041 (coerce arm must fire). Got: {result:?}. \
         If this returns E-QUERY-041, check_temporal_literals Ast::SqlPipe arm is \
         incorrectly routing String-col comparisons to the Datetime gate."
    );

    // Must NOT be a parse error.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "MED-1 SqlPipe String col: date-only string must parse successfully under Option-A. \
         Got: {result:?}"
    );
}

/// MED-1 SqlPipe Integer col: date-only literal in SqlPipe head WHERE clause
/// against a `ColumnType::Integer` column MUST trigger E-QUERY-002 (QueryTypeMismatch).
///
/// Query: `SELECT * FROM metrics_sensor_events WHERE count_col = '2026-06-24' | limit 10`
///
/// `count_col` is an Integer column.  `check_temporal_literals` must detect the
/// `RawTemporalLiteral` + non-Datetime/non-String column combination and return
/// `E-QUERY-002` (type mismatch), NOT `E-QUERY-041` (temporal gate).
///
/// This is arm (3) of the seven-arm dispatch (ADR-052 §D4 v1.10), exercised via SqlPipe.
///
/// Traces to: ADR-052 §D4 Step 3 third arm; BC-2.11.021; ADR-052 §D4 MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sqlpipe_integer_col_date_only_raises_e_query_002(
) {
    let engine = make_typed_columns_engine();

    let result = engine
        .execute(
            "SELECT * FROM metrics_sensor_events WHERE count_col = '2026-06-24' | limit 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-1 SqlPipe Integer col: date-only '2026-06-24' against Integer col must return \
         an error. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Must NOT be E-QUERY-041 (Datetime arm must not fire for Integer col).
    assert!(
        !matches!(&err, PrismError::TemporalLiteralUnparseable { .. }),
        "MED-1 SqlPipe Integer col: E-QUERY-041 must NOT fire for Integer col — use \
         E-QUERY-002 (QueryTypeMismatch). Got: {display}"
    );

    // Must NOT be a parse error.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "MED-1 SqlPipe Integer col: must NOT be a parse error under Option-A. Got: {display}"
    );

    // Must be QueryTypeMismatch (E-QUERY-002).
    assert!(
        matches!(&err, PrismError::QueryTypeMismatch { .. }),
        "MED-1 SqlPipe Integer col: error must be QueryTypeMismatch (E-QUERY-002), not a \
         different PrismError variant. Got: {display}. \
         If check_temporal_literals Ast::SqlPipe arm does not handle the third arm (non-Datetime, \
         non-String) correctly, it may swallow the error or return E-QUERY-041."
    );
}

// ── RG-035: GROUP BY position RawTemporalLiteral → REJECT E-QUERY-042 (GroupBy) ─────────────

/// RG-035 (ADR-052 §D4 v1.10): `RawTemporalLiteral` in a GROUP BY position MUST be
/// REJECTED with E-QUERY-042 (GroupBy position).
///
/// `SELECT count(*) FROM test_events GROUP BY '2026-06-24'` — grouping by a bare literal
/// constant is a degenerate no-op (all rows map to the same group) and is almost always
/// an analyst mistake. ADR-052 §D4 v1.10 tightens OBS-2: GROUP BY and ORDER BY positions
/// now REJECT rather than coerce (SELECT projection continues to coerce — see RG-023).
///
/// # Implementation state (GREEN)
/// GROUP BY date-like literal → REJECT E-QUERY-042 (GroupBy); test asserts Err and PASSES.
/// `check_expr_temporal_pos(..., GroupBy)` returns
/// `Err(PrismError::TemporalLiteralInvalidPosition { position: GroupBy, value_prefix: "2026-06-24" })`.
///
/// Traces to: ADR-052 §D4 v1.10 arm (6); BC-2.11.021 §Error Cases; error-taxonomy.md
///            §E-QUERY-042 v2.14; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 F-MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_group_by_date_like_rejects_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Primary: must be E-QUERY-042 TemporalLiteralInvalidPosition with GroupBy.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "RG-035: GROUP BY '2026-06-24' must return E-QUERY-042 (GroupBy position). \
         ADR-052 §D4 v1.10 arm (6). Got: {result:?}"
    );

    // value_prefix must be the first ≤50 chars of the literal.
    if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
        assert_eq!(
            value_prefix, "2026-06-24",
            "RG-035: value_prefix must be '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // Must NOT be E-QUERY-041 (wrong code — GROUP BY position, not datetime-col comparison).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-035: GROUP BY position must NOT trigger E-QUERY-041 (that's for datetime-col \
         comparisons). Got: {result:?}"
    );

    // Must NOT succeed — OBS-2 coerce NO LONGER applies to GROUP BY (ADR-052 §D4 v1.10).
    assert!(
        result.is_err(),
        "RG-035: GROUP BY '2026-06-24' must FAIL (E-QUERY-042). OBS-2 coerce is \
         retired for GROUP BY/ORDER BY in ADR-052 §D4 v1.10. Got Ok: {result:?}"
    );
}

// ── RG-036: ORDER BY position RawTemporalLiteral → E-QUERY-042 (ADR-052 §D4 v1.10) ─────

/// RG-036 (ADR-052 §D4 v1.10): `RawTemporalLiteral` in an ORDER BY position MUST be
/// REJECTED with E-QUERY-042 (OrderBy position).
///
/// `SELECT * FROM test_events ORDER BY '2026-06-24'` — ordering by a bare literal constant
/// is a degenerate no-op (sort order on a constant is undefined) and is almost always an
/// analyst mistake. ADR-052 §D4 v1.10 tightens OBS-2 for ORDER BY positions.
///
/// # Implementation state (GREEN)
/// ORDER BY date-like literal → REJECT E-QUERY-042 (OrderBy); test asserts Err and PASSES.
/// `check_expr_temporal_pos(..., OrderBy)` returns
/// `Err(PrismError::TemporalLiteralInvalidPosition { position: OrderBy, value_prefix: "2026-06-24" })`.
///
/// Traces to: ADR-052 §D4 v1.10 arm (7); BC-2.11.021 §Error Cases; error-taxonomy.md
///            §E-QUERY-042 v2.14; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 F-MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_order_by_date_like_rejects_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events ORDER BY '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Primary: must be E-QUERY-042 TemporalLiteralInvalidPosition with OrderBy.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "RG-036: ORDER BY '2026-06-24' must return E-QUERY-042 (OrderBy position). \
         ADR-052 §D4 v1.10 arm (7). Got: {result:?}"
    );

    // value_prefix must be the first ≤50 chars of the literal.
    if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
        assert_eq!(
            value_prefix, "2026-06-24",
            "RG-036: value_prefix must be '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // Must NOT be E-QUERY-041 (wrong code — ORDER BY position, not datetime-col comparison).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "RG-036: ORDER BY position must NOT trigger E-QUERY-041 (that's for datetime-col \
         comparisons). Got: {result:?}"
    );

    // Must NOT succeed — OBS-2 coerce NO LONGER applies to ORDER BY (ADR-052 §D4 v1.10).
    assert!(
        result.is_err(),
        "RG-036: ORDER BY '2026-06-24' must FAIL (E-QUERY-042). OBS-2 coerce is \
         retired for GROUP BY/ORDER BY in ADR-052 §D4 v1.10. Got Ok: {result:?}"
    );
}

// ── LOW-2: DML SET unknown-column RawTemporalLiteral → coerce to String ──────

/// LOW-2 (OBS-1/LOW-2 fix-burst): `RawTemporalLiteral` in a DML SET assignment
/// value for a column whose type is UNKNOWN (not found in the registry) MUST be
/// coerced to `Literal::String` in-place by `check_temporal_literals`.
///
/// This exercises the `None | Some(_)` arm of the DML SET dispatch block.
/// Post-OBS-2, `check_expr_temporal`'s bare-`RawTemporalLiteral` arm COERCES to
/// `Literal::String` and returns `Ok(())` — the DML unknown-column arm must mirror
/// that behavior for consistency (ADR-052 §D4 OBS-2 defense-in-depth).
///
/// # Test approach (SID-1 compliant)
/// Calls `check_temporal_literals` directly with a manually constructed DML AST.
/// DML execution falls to `Ok(vec![])` pending S-3.06 wiring, so no end-to-end
/// DML test is possible; this unit test at the `check_temporal_literals` boundary
/// is the load-bearing regression guard for the coerce behavior.
///
/// Traces to: ADR-052 §D4 OBS-2 + LOW-2; BC-2.11.021.
#[test]
fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_low2_dml_set_unknown_col_coerces_to_string() {
    use crate::ast::{Ast, Expr, Literal, SqlStatement};
    use crate::materialization::check_temporal_literals;
    use crate::write_ast::{Assignment, DmlNode, DmlOperation};

    let registry = make_test_events_registry();

    // UPDATE test_events SET unknown_col = '2026-06-24'
    // `unknown_col` is NOT registered in the test registry — col_type resolves to None
    // → the `None | Some(_)` arm of the SET assignment dispatch fires.
    let dml = DmlNode {
        operation: DmlOperation::Update,
        target_table: "test_events".to_string(),
        columns: None,
        assignments: vec![Assignment {
            column: "unknown_col".to_string(),
            value: Expr::Literal(Literal::RawTemporalLiteral("2026-06-24".to_string())),
        }],
        // filter: None — UPDATE without WHERE is legal at the AST level; the parser
        // enforces the WHERE-required rule; we bypass the parser here to test the
        // temporal walker in isolation.
        filter: None,
        source_select: None,
    };
    let mut ast = Ast::Sql(SqlStatement::Dml(dml));

    let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);

    assert!(
        result.is_ok(),
        "LOW-2: DML SET RawTemporalLiteral with unknown column MUST NOT error \
         (unknown-type arm coerces to String per OBS-2). Got: {result:?}"
    );

    // Verify the value was coerced from RawTemporalLiteral → Literal::String in-place.
    if let Ast::Sql(SqlStatement::Dml(ref dml_out)) = ast {
        let assignment = &dml_out.assignments[0];
        assert!(
            matches!(
                &assignment.value,
                Expr::Literal(Literal::String(s)) if s == "2026-06-24"
            ),
            "LOW-2: DML SET value MUST be coerced from RawTemporalLiteral to \
             Literal::String(\"2026-06-24\"). Got: {:?}",
            assignment.value
        );
    } else {
        panic!("LOW-2: expected Ast::Sql(SqlStatement::Dml) after check_temporal_literals");
    }
}

// ── E-QUERY-042 tests: NonColumnLhsComparison ─────────────────────────────────

/// E-QUERY-042 (NonColumnLhsComparison): `WHERE lower(hostname) = '2026-06-24'` must
/// return E-QUERY-042 (NonColumnLhsComparison), NOT QueryPlanFailed / -32000.
///
/// Prior behavior (before ADR-052 §D4 v1.10): the non-Field LHS arm of
/// `check_expr_temporal` returned `Err(PrismError::QueryPlanFailed { ... })` —
/// an analyst-hostile INTERNAL_ERROR. This test verifies the migration to the
/// analyst-readable `-32602 INVALID_PARAMS` E-QUERY-042 error.
///
/// `lower(hostname)` is a `FuncCall::Scalar` expression — the walker cannot resolve
/// the LHS type at plan time. Silently coercing `'2026-06-24'` to `Literal::String`
/// would reintroduce RISK-1 for datetime-valued expressions like `to_timestamp(col)`.
///
/// # Pre-implementation state (Red Gate)
/// `check_expr_temporal`'s non-Field LHS arm returns `QueryPlanFailed`.
/// Test asserts `TemporalLiteralInvalidPosition(NonColumnLhsComparison)` → FAILS. ✓
///
/// # Post-implementation state
/// Non-Field LHS arm returns `Err(PrismError::TemporalLiteralInvalidPosition {
///     position: NonColumnLhsComparison, value_prefix: "2026-06-24" })`.
///
/// Traces to: ADR-052 §D4 v1.10 arm (4); error-taxonomy.md §E-QUERY-042 v2.14;
///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 F-LOW-1.
///
/// # Implementation note — why direct AST test, not engine.execute()
///
/// Post-DEFECT-PQL-FNCALL-LHS-001: `build_predicate_parser` (shared by pipe `| where`,
/// filter mode, and SQL WHERE via `build_sql_predicate_parser`) now accepts fn-call LHS
/// via the `fn_call_comparison` production. Queries like `WHERE lower(hostname) = '2026-06-24'`
/// now PARSE successfully (fn-call LHS is admitted) and reach the temporal walker, which
/// fires E-QUERY-042 (NonColumnLhsComparison). The end-to-end path for pipe `| where` is
/// covered by `test_BC_2_11_004_ec11_004_005_pipe_fncall_lhs_date_like_rejects_e_query_042`.
///
/// This synthetic-AST test remains as defense-in-depth for Expr-typed positions (JOIN ON,
/// SELECT projection, GROUP BY, ORDER BY) whose parsers still emit `field_path_to_expr`
/// LHS only and cannot produce fn-call LHS from user input. The synthetic AST exercises
/// the non-Field LHS arm of `check_expr_temporal` directly — `Expr::Now = '2026-06-24'`
/// in a SELECT item — verifying the E-QUERY-042 path without relying on the parser.
#[test]
fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_column_lhs_date_like_e_query_042() {
    use crate::ast::{
        Ast, CompareOp, Expr, FromClause, Literal, SelectClause, SelectItem, SourceRef,
        SourceRefKind, SqlQuery, SqlStatement,
    };
    use crate::materialization::check_temporal_literals;
    use prism_core::error::TemporalLiteralPosition;

    // Build: SELECT (NOW() = '2026-06-24') FROM test_events
    // Expr::Now is a non-Field LHS — not a FieldPath — so it cannot be resolved to a column
    // type. This is the canonical non-column-LHS scenario per ADR-052 §D4 v1.10 arm (4).
    let mut ast = Ast::Sql(SqlStatement::Select(SqlQuery {
        select: SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: Expr::Compare {
                    lhs: Box::new(Expr::Now), // non-Field LHS — triggers E-QUERY-042
                    op: CompareOp::Eq,
                    rhs: Box::new(Expr::Literal(Literal::RawTemporalLiteral(
                        "2026-06-24".to_string(),
                    ))),
                },
                alias: None,
            }],
        },
        from: FromClause {
            source: SourceRef {
                raw: "test_events".to_string(),
                kind: SourceRefKind::Custom,
            },
            alias: None,
        },
        joins: vec![],
        where_: None,
        group_by: vec![],
        having: None,
        order_by: vec![],
        limit: None,
    }));

    let result = check_temporal_literals(&mut ast, None, false);

    // Primary: must be E-QUERY-042 with NonColumnLhsComparison.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            })
        ),
        "E-QUERY-042: SELECT (NOW() = '2026-06-24') must return \
         TemporalLiteralInvalidPosition(NonColumnLhsComparison). \
         ADR-052 §D4 v1.10 arm (4). Got: {result:?}"
    );

    // value_prefix must be the first ≤50 chars of the literal.
    if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
        assert_eq!(
            value_prefix, "2026-06-24",
            "E-QUERY-042: value_prefix must be '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // Must NOT be QueryPlanFailed — old analyst-hostile -32000 behavior.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "E-QUERY-042: non-column-LHS comparison must NOT return QueryPlanFailed (-32000). \
         Must return TemporalLiteralInvalidPosition (-32602). Got: {result:?}"
    );
}

// ── E-QUERY-042 parser-driven HAVING end-to-end test ─────────────────────────

/// E-QUERY-042 (NonColumnLhsComparison): HAVING `max(timestamp) > '2026-06-24'`
/// must return E-QUERY-042 (NonColumnLhsComparison) through a real `engine.execute()` call.
///
/// The F-HIGH-1 fix (ADR-052 §D4 v1.10 arm (4)) adds E-QUERY-042 for non-Field LHS
/// comparisons. The complementary unit-level test (`test_...non_column_lhs_date_like_e_query_042`
/// in `temporal_typing_tests.rs` and `test_having_non_field_lhs_raw_temporal_fires_e_query_042_non_column_lhs_comparison`
/// in `materialization.rs`) exercises the `check_temporal_literals` function directly with
/// a hand-constructed AST. This test closes the gap by exercising the HAVING path
/// end-to-end through the real SQL parser and engine.
///
/// # How the path is reached
/// The HAVING parser (`build_having_predicate_parser` in `sql_parser.rs`) supports the
/// `agg_fn(col) op literal` form (ADR-048). Parsing
/// `HAVING max(timestamp) > '2026-06-24'` produces:
/// ```
/// Predicate::Compare {
///     lhs: Expr::FuncCall::Aggregate(Max(timestamp)),   // NOT Expr::Field
///     op:  Gt,
///     rhs: Expr::Literal(RawTemporalLiteral("2026-06-24")),
/// }
/// ```
/// The early `check_temporal_literals` gate (skip_projection=true, runs before E-QUERY-037)
/// walks the HAVING predicate, finds a non-Field LHS with a `RawTemporalLiteral` RHS,
/// and returns `E-QUERY-042 NonColumnLhsComparison` per ADR-052 §D4 v1.10 arm (4).
///
/// # MCP mapping
/// `PrismError::TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, .. }` maps
/// to JSON-RPC -32602 INVALID_PARAMS (not -32000 INTERNAL_ERROR).
/// Verified by the independent mapping test in `prism-mcp::error_mapping` (line ~3628).
///
/// Traces to: ADR-052 §D4 v1.10 arm (4); BC-2.11.003; error-taxonomy.md §E-QUERY-042 v2.14;
///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 MED-1.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_having_agg_date_only_raises_e_query_042_parser_driven(
) {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    // Full SQL query parsed through the real PrismQL parser.
    // GROUP BY hostname (String col) is valid; HAVING max(timestamp) > '2026-06-24'
    // uses the ADR-048 agg_fn(col) op literal HAVING form.
    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY hostname \
             HAVING max(timestamp) > '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be an error — the temporal gate fires at plan time.
    assert!(
        result.is_err(),
        "MED-1 e2e: HAVING max(timestamp) > '2026-06-24' must return Err(E-QUERY-042). \
         Got Ok. \
         Check: check_pred_raw_temporal HAVING arm, early gate in engine.rs (skip_projection=true)."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary: must be E-QUERY-042 NonColumnLhsComparison.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "MED-1 e2e: error must be PrismError::TemporalLiteralInvalidPosition \
         (NonColumnLhsComparison). ADR-052 §D4 v1.10 arm (4). Got: {err:?} (Display: {display})"
    );

    // value_prefix must be the date-only string.
    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "MED-1 e2e: value_prefix must start with '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // Must contain E-QUERY-042 code in the Display string.
    assert!(
        display.contains("E-QUERY-042"),
        "MED-1 e2e: error Display must contain 'E-QUERY-042'. Got: {display}"
    );

    // Must NOT contain DataFusion/Arrow errors — fires at Prism plan time (early gate).
    assert!(
        !display.contains("Arrow error") && !display.contains("DataFusion"),
        "MED-1 e2e: E-QUERY-042 must fire at Prism plan time (early gate), NOT as a \
         DataFusion/Arrow error. Got: {display}"
    );

    // Must NOT be QueryPlanFailed — the pre-ADR-052-v1.10 analyst-hostile -32000 behavior.
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "MED-1 e2e: HAVING non-column-LHS must NOT return QueryPlanFailed (-32000). \
         Must return TemporalLiteralInvalidPosition (-32602). Got: {err:?}"
    );
}

// ── E-QUERY-042: Pipe mode parse-time rejection (stats by / sort) ─────────────

/// Pipe `stats … by '2026-06-24'` MUST fail at parse time with a clear E-QUERY-001
/// message indicating that `stats by` expects a field name, not a literal value.
///
/// ADR-052 §D4 v1.10 option (a): pipe `stats … by` and `sort` keys accept ONLY
/// `FieldPath` — a quoted string literal is rejected at parse time by the chumsky
/// parser. The enhancement requires the error message to clearly say "field name,
/// not a literal value" rather than a generic chumsky positional error.
///
/// # Pre-implementation state (Red Gate)
/// `FROM t | stats count by '2026-06-24'` fails with generic parse error.
/// Test asserts the message contains "field name" OR "literal" → FAILS. ✓
///
/// # Post-implementation state
/// Parse error message includes "field name" or "literal value" per the enhanced
/// `rewrite_temporal_literal_in_pipe_key_position` rewriter.
///
/// Traces to: ADR-052 §D4 v1.10 option (a); BC-2.11.004 §Error Cases;
///            error-taxonomy.md §E-QUERY-042 v2.14 pipe-mode note;
///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 pipe-parse enhancement.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_stats_by_date_like_e_query_001() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "FROM test_events | stats count by '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be a parse error (E-QUERY-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "pipe stats by literal: must fail with QueryParseFailed (E-QUERY-001). \
         Got: {result:?}"
    );

    // Error message must be CLEAR — contain "field name" or "literal".
    if let Err(PrismError::QueryParseFailed { detail, .. }) = &result {
        assert!(
            detail.contains("field name") || detail.contains("literal"),
            "pipe stats by literal: error message must contain 'field name' or 'literal' \
             to guide the analyst. Got: {detail:?}"
        );
    }
}

/// Pipe `sort '2026-06-24'` MUST fail at parse time with a clear E-QUERY-001 message
/// indicating that `sort` expects a field name, not a literal value.
///
/// Counterpart to the `stats by` test above, for the sort stage.
///
/// # Pre-implementation state (Red Gate)
/// `FROM t | sort '2026-06-24'` fails with generic parse error.
/// Test asserts the message contains "field name" OR "literal" → FAILS. ✓
///
/// # Post-implementation state
/// Parse error message includes "field name" or "literal value" per the enhanced
/// `rewrite_temporal_literal_in_pipe_key_position` rewriter.
///
/// Traces to: ADR-052 §D4 v1.10 option (a); BC-2.11.004 §Error Cases;
///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 pipe-parse enhancement.
#[tokio::test]
async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sort_date_like_e_query_001() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "FROM test_events | sort '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be a parse error (E-QUERY-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "pipe sort literal: must fail with QueryParseFailed (E-QUERY-001). \
         Got: {result:?}"
    );

    // Error message must be CLEAR — contain "field name" or "literal".
    if let Err(PrismError::QueryParseFailed { detail, .. }) = &result {
        assert!(
            detail.contains("field name") || detail.contains("literal"),
            "pipe sort literal: error message must contain 'field name' or 'literal' \
             to guide the analyst. Got: {detail:?}"
        );
    }
}

// ── DEFECT-EQUERY042-GROUPBY-DEADARM-001: Literal::Timestamp in GROUP BY/ORDER BY ─────────

/// DEFECT-EQUERY042-GROUPBY-DEADARM-001 (RED): A full RFC-3339 UTC timestamp literal
/// in a GROUP BY clause MUST be rejected with E-QUERY-042 (GroupBy position).
///
/// `SELECT count(*) FROM test_events GROUP BY '2026-07-01T00:00:00Z'`
///
/// # Root cause of the dead arm
/// The parser's `classify_string_literal` (filter_parser.rs) uses a three-way dispatch:
///   1. RFC-3339 parse succeeds → `Literal::Timestamp(ts)`   ← this path for `'2026-07-01T00:00:00Z'`
///   2. `is_date_like(s)` true   → `Literal::RawTemporalLiteral(s)`
///   3. Otherwise                → `Literal::String(s)`
///
/// `check_expr_temporal_pos` (materialization.rs) dispatches only on
/// `Expr::Literal(Literal::RawTemporalLiteral(...))` — the first match arm.
/// `Literal::Timestamp` (full RFC-3339 forms) falls through to `_ => Ok(())`,
/// silently skipping the GroupBy arm. The SQL emitter then receives the
/// `Literal::Timestamp` and emits `arrow_cast('2026-07-01T00:00:00Z',
/// 'Timestamp(Microsecond, Some("UTC"))')` in the GROUP BY clause.
/// DataFusion receives this and either accepts silently (degenerate success)
/// or rejects with an analyst-hostile internal error — neither is E-QUERY-042.
///
/// # Spec mandate
/// error-taxonomy.md v2.14 §E-QUERY-042 (GroupBy):
///   "A date-like literal in a GROUP BY expression (`GROUP BY '2026-06-24'`) —
///    grouping by a bare literal constant is a degenerate no-op (every row maps to
///    the same group keyed on the constant), almost always an analyst mistake."
/// `'2026-07-01T00:00:00Z'` is a date-shaped literal. The spec prose says "date-like
/// literal" without restricting to `RawTemporalLiteral`; the arm (6) table entry says
/// "GROUP BY position bare literal → E-QUERY-042 (GroupBy)".
/// ADR-052 §D4 (v1.11) arm (6); BC-2.11.021 §Error Cases; BC-2.11.003 §Error Cases.
///
/// # Implementation state (RED)
/// `Literal::Timestamp` bypasses the GroupBy arm. The test asserts
/// `Err(PrismError::TemporalLiteralInvalidPosition { position: GroupBy, .. })`,
/// which is never emitted — the test FAILS (RED gate confirmed). ✓
///
/// After the fix: `check_expr_temporal_pos` must handle `Literal::Timestamp` in
/// GroupBy/OrderBy positions with the same REJECT semantics as `RawTemporalLiteral`.
///
/// Traces to: error-taxonomy.md §E-QUERY-042 v2.14; ADR-052 §D4 (v1.11) arm (6);
///            BC-2.11.021 §Error Cases; DEFECT-EQUERY042-GROUPBY-DEADARM-001.
#[tokio::test]
async fn test_DEFECT_EQUERY042_GROUPBY_DEADARM_001_group_by_rfc3339_timestamp_must_fire_e_query_042(
) {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    // '2026-07-01T00:00:00Z' is a full RFC-3339 UTC timestamp.
    // Parser: classify_string_literal → TimestampLiteral::new succeeds → Literal::Timestamp.
    // Current behavior (BUG): check_expr_temporal_pos GroupBy arm misses Literal::Timestamp.
    // Expected behavior (after fix): Err(TemporalLiteralInvalidPosition { GroupBy }).
    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY '2026-07-01T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    // Must fail — GROUP BY constant is a degenerate no-op analyst mistake (E-QUERY-042).
    assert!(
        result.is_err(),
        "DEFECT-001: GROUP BY '2026-07-01T00:00:00Z' must return Err(E-QUERY-042). \
         If Ok, the Literal::Timestamp GROUP BY arm is silently accepting a degenerate \
         query that should be rejected. Got: Ok"
    );

    // Primary assertion: must be E-QUERY-042 TemporalLiteralInvalidPosition with GroupBy.
    // This is the RED gate — currently fails because Literal::Timestamp bypasses the arm.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "DEFECT-001: GROUP BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy). \
         ADR-052 §D4 (v1.11) arm (6); error-taxonomy.md v2.14 E-QUERY-042 (GroupBy). \
         Current defect: Literal::Timestamp bypasses check_expr_temporal_pos GroupBy arm \
         (arm matches only RawTemporalLiteral). Got: {result:?}"
    );

    // value_prefix must contain the first ≤50 chars of the offending literal.
    if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
        assert!(
            value_prefix.starts_with("2026-07-01"),
            "DEFECT-001: value_prefix must start with '2026-07-01'. Got: {value_prefix:?}"
        );
    }

    // Must NOT be E-QUERY-041 (wrong error — GroupBy position, not datetime-col comparison).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "DEFECT-001: GROUP BY position must NOT trigger E-QUERY-041. \
         E-QUERY-041 is for datetime-col comparisons with bad literal form. \
         GroupBy temporal literal must give E-QUERY-042. Got: {result:?}"
    );

    // Must NOT be a QueryPlanFailed (-32000 internal error) — that is the pre-fix
    // analyst-hostile behavior this defect fix must eliminate.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "DEFECT-001: GROUP BY '2026-07-01T00:00:00Z' must NOT return QueryPlanFailed \
         (-32000 internal error). Must return E-QUERY-042 (-32602 INVALID_PARAMS). \
         Got: {result:?}"
    );
}

/// DEFECT-EQUERY042-GROUPBY-DEADARM-001 (RED): A full RFC-3339 UTC timestamp literal
/// in an ORDER BY clause MUST be rejected with E-QUERY-042 (OrderBy position).
///
/// `SELECT * FROM test_events ORDER BY '2026-07-01T00:00:00Z'`
///
/// # Root cause
/// Same dead-arm as the GROUP BY case: `classify_string_literal` produces
/// `Literal::Timestamp` for `'2026-07-01T00:00:00Z'`; `check_expr_temporal_pos` only
/// handles `RawTemporalLiteral` in the OrderBy arm, so `Literal::Timestamp` falls to
/// `_ => Ok(())`. The emitter sends `arrow_cast(...)` in ORDER BY to DataFusion.
///
/// # Spec mandate
/// error-taxonomy.md v2.14 §E-QUERY-042 (OrderBy):
///   "A date-like literal in an ORDER BY expression (`ORDER BY '2026-06-24'`) —
///    ordering by a bare literal constant is a degenerate no-op (sort order on a
///    constant is undefined), almost always an analyst mistake."
/// `'2026-07-01T00:00:00Z'` is a date-shaped literal that meets this criterion.
/// ADR-052 §D4 (v1.11) arm (7); BC-2.11.021 §Error Cases; BC-2.11.003 §Error Cases.
///
/// # Implementation state (RED)
/// `Literal::Timestamp` bypasses the OrderBy arm. Test asserts
/// `Err(TemporalLiteralInvalidPosition { OrderBy })` — FAILS (RED gate). ✓
///
/// Traces to: error-taxonomy.md §E-QUERY-042 v2.14; ADR-052 §D4 (v1.11) arm (7);
///            BC-2.11.021 §Error Cases; DEFECT-EQUERY042-GROUPBY-DEADARM-001.
#[tokio::test]
async fn test_DEFECT_EQUERY042_GROUPBY_DEADARM_001_order_by_rfc3339_timestamp_must_fire_e_query_042(
) {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    // '2026-07-01T00:00:00Z' is a full RFC-3339 UTC timestamp.
    // Parser: Literal::Timestamp (RFC-3339 fast path).
    // Current behavior (BUG): OrderBy arm in check_expr_temporal_pos misses Literal::Timestamp.
    // Expected behavior (after fix): Err(TemporalLiteralInvalidPosition { OrderBy }).
    let result = engine
        .execute(
            "SELECT * FROM test_events ORDER BY '2026-07-01T00:00:00Z'",
            QueryOptions::default(),
        )
        .await;

    // Must fail — ORDER BY constant is a degenerate no-op analyst mistake (E-QUERY-042).
    assert!(
        result.is_err(),
        "DEFECT-001: ORDER BY '2026-07-01T00:00:00Z' must return Err(E-QUERY-042). \
         If Ok, the Literal::Timestamp ORDER BY arm is silently accepting a degenerate \
         query that should be rejected. Got: Ok"
    );

    // Primary assertion: must be E-QUERY-042 TemporalLiteralInvalidPosition with OrderBy.
    // This is the RED gate — currently fails because Literal::Timestamp bypasses the arm.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "DEFECT-001: ORDER BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy). \
         ADR-052 §D4 (v1.11) arm (7); error-taxonomy.md v2.14 E-QUERY-042 (OrderBy). \
         Current defect: Literal::Timestamp bypasses check_expr_temporal_pos OrderBy arm \
         (arm matches only RawTemporalLiteral). Got: {result:?}"
    );

    // value_prefix must contain the first ≤50 chars of the offending literal.
    if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
        assert!(
            value_prefix.starts_with("2026-07-01"),
            "DEFECT-001: value_prefix must start with '2026-07-01'. Got: {value_prefix:?}"
        );
    }

    // Must NOT be E-QUERY-041 (wrong error — OrderBy position, not datetime-col comparison).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "DEFECT-001: ORDER BY position must NOT trigger E-QUERY-041. \
         E-QUERY-041 is for datetime-col comparisons. \
         OrderBy temporal literal must give E-QUERY-042. Got: {result:?}"
    );

    // Must NOT be a QueryPlanFailed (-32000 internal error).
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "DEFECT-001: ORDER BY '2026-07-01T00:00:00Z' must NOT return QueryPlanFailed \
         (-32000 internal error). Must return E-QUERY-042 (-32602 INVALID_PARAMS). \
         Got: {result:?}"
    );
}

/// DEFECT-EQUERY042-GROUPBY-DEADARM-001 (GREEN negative control): A date-only
/// `RawTemporalLiteral` compared against a `Datetime` column in WHERE must STILL
/// produce E-QUERY-041 after the Literal::Timestamp GroupBy/OrderBy fix.
///
/// `SELECT * FROM test_events WHERE timestamp = '2026-06-24'`
///
/// `'2026-06-24'` is a date-only form: `classify_string_literal` → `is_date_like` true
/// → `Literal::RawTemporalLiteral("2026-06-24")`. The fix only adds handling for
/// `Literal::Timestamp` in GroupBy/OrderBy positions — it must not disturb the
/// `RawTemporalLiteral` comparison path (arm 1: Datetime col → E-QUERY-041).
///
/// # Regression guard
/// If the fix accidentally widens the GroupBy/OrderBy arms to catch `Literal::Timestamp`
/// in WHERE comparisons, or otherwise perturbs the compare-path dispatch, this test
/// detects the regression. The WHERE path must remain on its existing E-QUERY-041 rail.
///
/// Traces to: error-taxonomy.md §E-QUERY-041 v2.14; ADR-052 §D4 v1.10 arm (1);
///            BC-2.11.021 §Error Cases; DEFECT-EQUERY042-GROUPBY-DEADARM-001 NC-1.
#[tokio::test]
async fn test_DEFECT_EQUERY042_GROUPBY_DEADARM_001_where_datetime_col_date_only_still_yields_e_query_041(
) {
    let engine = make_test_engine();

    // '2026-06-24' (date-only, RawTemporalLiteral) vs Datetime column → E-QUERY-041.
    // This path must be unaffected by any fix to the Literal::Timestamp GroupBy/OrderBy arms.
    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE timestamp = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "NC-1: WHERE timestamp = '2026-06-24' (date-only RawTemporalLiteral vs Datetime col) \
         must still return E-QUERY-041 (TemporalLiteralUnparseable) after the \
         DEFECT-EQUERY042-GROUPBY-DEADARM-001 fix. \
         ADR-052 §D4 v1.10 arm (1); error-taxonomy.md E-QUERY-041. Got: {result:?}"
    );
}

/// DEFECT-EQUERY042-GROUPBY-DEADARM-001 (GREEN negative control): A plain non-temporal
/// string literal in GROUP BY must NOT trigger E-QUERY-042 after the fix.
///
/// `SELECT count(*) FROM test_events GROUP BY 'not_a_date'`
///
/// `'not_a_date'` is neither an RFC-3339 timestamp nor a `is_date_like` match:
/// `classify_string_literal` → `Literal::String("not_a_date")`. The fix for
/// DEFECT-EQUERY042-GROUPBY-DEADARM-001 adds a GroupBy arm for `Literal::Timestamp`
/// — it must NOT catch `Literal::String` as well (no false E-QUERY-042).
///
/// Standard SQL allows `GROUP BY <string_constant>` (degenerate but valid); prism
/// does not intercept plain-string GROUP BY at plan time. Only date-shaped literals
/// (temporal constants that are almost certainly analyst mistakes) are rejected.
///
/// # Regression guard
/// Ensures the fix discriminates between:
///   - `Literal::Timestamp` (RFC-3339 temporal constant) → REJECT E-QUERY-042
///   - `Literal::String`    (non-temporal constant)      → pass through to DataFusion
///   - `Literal::RawTemporalLiteral` (date-only/offset-less) → already REJECT E-QUERY-042
///
/// Traces to: error-taxonomy.md §E-QUERY-042 v2.14 (GroupBy only catches date-shaped
/// literals); ADR-052 §D4; DEFECT-EQUERY042-GROUPBY-DEADARM-001 NC-2.
#[tokio::test]
async fn test_DEFECT_EQUERY042_GROUPBY_DEADARM_001_group_by_plain_string_no_false_e_query_042() {
    let engine = make_test_engine();

    // 'not_a_date' → Literal::String (not temporal) → no E-QUERY-042.
    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY 'not_a_date'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be E-QUERY-042 — plain string is not a date-shaped literal.
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "NC-2: GROUP BY 'not_a_date' (Literal::String) must NOT return E-QUERY-042. \
         The fix must discriminate temporal literals (Literal::Timestamp) from plain \
         strings (Literal::String). Got: {result:?}"
    );

    // Must NOT be E-QUERY-041 either (not a datetime-col comparison).
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "NC-2: GROUP BY 'not_a_date' must NOT return E-QUERY-041. \
         Got: {result:?}"
    );
}

// ── F-EQ42-P1-002: Sibling call-site coverage ────────────────────────────────────────────────
//
// LOCAL adversary pass-1 finding F-EQ42-P1-002 (MED): the existing DEFECT-EQUERY042 tests only
// cover the top-level `Ast::Sql(SqlStatement::Select)` call sites (SQL-mode SELECT GROUP BY /
// ORDER BY). `check_expr_temporal_pos` has 10 GROUP BY/ORDER BY call sites across 5 AST-path
// classes. The four classes below were uncovered: SqlPipe head, Predicate::InSubquery,
// Expr::InSubquery, and DML source_select. (Expr::InSubquery coverage is provided by the
// F-EQ42-P2-001 tests after this block; the 6 tests here cover the other three classes.)
//
// These tests are GREEN LOCKS — the Literal::Timestamp arm added in the DEADARM fix already
// covers all these paths. The tests exist to LOCK that coverage so a future refactor cannot
// accidentally introduce a regression.
//
// All 6 tests in this block use `make_test_engine()` (test_events registered, no adapters).
//
// Gate ordering: E-QUERY-037 (table gate) → E-QUERY-038 (col gate, fail-open for DML) →
// E-QUERY-039 (enrich gate, skipped) → check_temporal_literals (E-QUERY-042).
// For SqlPipe/DML: check_temporal_literals fires from the EARLY gate in engine.rs (line ~831,
// skip_projection=true, but SqlPipe GROUP BY / ORDER BY fire from the IN-PIPELINE pass;
// DML source_select GROUP BY / ORDER BY fire unconditionally in both passes since the DML arm
// has no skip_projection guard). For Predicate::InSubquery (WHERE clause): fires via
// check_pred_raw_temporal in the early gate (before E-QUERY-037). For Expr::InSubquery
// (SELECT projection): fires via check_select_items_raw_temporal in the in-pipeline pass
// (skip_projection=false, after E-QUERY-037/038) — see F-EQ42-P2-001 block below.

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a SqlPipe HEAD GROUP BY clause
/// fires E-QUERY-042 (TemporalLiteralPosition::GroupBy).
///
/// `SELECT count(*) FROM test_events GROUP BY '2026-07-01T00:00:00Z' | limit 10`
///
/// SqlPipe head GROUP BY is covered by `check_temporal_literals` (materialization.rs)
/// Ast::SqlPipe arm, lines ~2724-2730. This call site is distinct from the top-level
/// Ast::Sql(Select) arm covered by the DEFECT-EQUERY042 tests.
///
/// # Call-site path
/// Parser: `classify_string_literal('2026-07-01T00:00:00Z')` → `Literal::Timestamp`.
/// `check_temporal_literals` Ast::SqlPipe arm → `spq.head.group_by` walker →
/// `check_expr_temporal_pos(expr, ..., TemporalCheckPos::GroupBy)` →
/// `Expr::Literal(Literal::Timestamp)` + GroupBy → `Err(TemporalLiteralInvalidPosition::GroupBy)`.
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (6); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_sqlpipe_head_group_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY '2026-07-01T00:00:00Z' | limit 10",
            QueryOptions::default(),
        )
        .await;

    // Primary assertion: must be E-QUERY-042 with GroupBy position.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "F-EQ42-P1-002: SqlPipe head GROUP BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy). \
         ADR-052 §D4 (v1.11) arm (6); check_temporal_literals Ast::SqlPipe GROUP BY walker. \
         Got: {result:?}"
    );

    // Negative: must NOT be E-QUERY-041.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: SqlPipe GROUP BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    // Negative: must NOT be QueryPlanFailed (pre-fix analyst-hostile internal error).
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: SqlPipe GROUP BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a SqlPipe HEAD ORDER BY clause
/// fires E-QUERY-042 (TemporalLiteralPosition::OrderBy).
///
/// `SELECT count(*) FROM test_events ORDER BY '2026-07-01T00:00:00Z' | limit 10`
///
/// SqlPipe head ORDER BY is covered by `check_temporal_literals` (materialization.rs)
/// Ast::SqlPipe arm, lines ~2732-2739. Sibling of the GROUP BY call site above.
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (7); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_sqlpipe_head_order_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT count(*) FROM test_events ORDER BY '2026-07-01T00:00:00Z' | limit 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "F-EQ42-P1-002: SqlPipe head ORDER BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy). \
         ADR-052 §D4 (v1.11) arm (7); check_temporal_literals Ast::SqlPipe ORDER BY walker. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: SqlPipe ORDER BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: SqlPipe ORDER BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a subquery GROUP BY via
/// `WHERE hostname IN (SELECT hostname FROM test_events GROUP BY '<rfc3339>')` fires
/// E-QUERY-042 (TemporalLiteralPosition::GroupBy).
///
/// # Call-site path
/// `check_temporal_literals` Ast::Sql(Select) arm walks the outer WHERE predicate via
/// `check_pred_raw_temporal`. The predicate is `Predicate::InSubquery { subquery, .. }`;
/// `check_pred_raw_temporal` Predicate::InSubquery arm (materialization.rs ~line 3167) walks
/// `subquery.group_by` → `check_expr_temporal_pos(..., TemporalCheckPos::GroupBy)` →
/// `Literal::Timestamp` + GroupBy → E-QUERY-042.
///
/// Note: the column gate (`check_query_column_availability`) intentionally does NOT descend
/// into `Predicate::InSubquery` bodies (fail-open per BC-2.11.019 OBS-001). Only `hostname`
/// (the outer IN field) is checked; the subquery body is fail-open. This means
/// `check_temporal_literals` fires from the early gate in engine.rs (before E-QUERY-037).
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (6); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_subquery_in_where_group_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events \
             WHERE hostname IN \
               (SELECT hostname FROM test_events GROUP BY '2026-07-01T00:00:00Z')",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "F-EQ42-P1-002: Subquery GROUP BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy). \
         check_pred_raw_temporal Predicate::InSubquery arm walks subquery.group_by. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: Subquery GROUP BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: Subquery GROUP BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a subquery ORDER BY via
/// `WHERE hostname IN (SELECT hostname FROM test_events ORDER BY '<rfc3339>')` fires
/// E-QUERY-042 (TemporalLiteralPosition::OrderBy).
///
/// Sibling of the GROUP BY subquery test above; exercises the ORDER BY walker in the
/// `check_pred_raw_temporal` Predicate::InSubquery arm (materialization.rs ~line 3175).
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (7); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_subquery_in_where_order_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events \
             WHERE hostname IN \
               (SELECT hostname FROM test_events ORDER BY '2026-07-01T00:00:00Z')",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "F-EQ42-P1-002: Subquery ORDER BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy). \
         check_pred_raw_temporal Predicate::InSubquery arm walks subquery.order_by. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: Subquery ORDER BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: Subquery ORDER BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a DML `source_select` GROUP BY
/// fires E-QUERY-042 (TemporalLiteralPosition::GroupBy).
///
/// `INSERT INTO test_events (hostname) SELECT hostname FROM test_events GROUP BY '<rfc3339>' LIMIT 10`
///
/// # Call-site path
/// The DML arm in `check_temporal_literals` (materialization.rs ~line 2751) walks
/// `dml.source_select.group_by` via `check_expr_temporal_pos(..., TemporalCheckPos::GroupBy)`.
/// Unlike the SqlPipe and Sql(Select) arms, the DML arm has NO `skip_projection` guard —
/// it runs unconditionally in BOTH the early gate (engine.rs ~line 831, skip_projection=true)
/// and the in-pipeline pass (skip_projection=false). E-QUERY-042 fires from the early gate.
///
/// # Bounded write requirement
/// `check_unbounded_write` (sql_parser.rs) rejects `INSERT INTO ... SELECT` with no WHERE
/// and no LIMIT as E-QUERY-022 (unbounded write). `LIMIT 10` on the source SELECT satisfies
/// the bounded-write constraint, allowing the parse to proceed to `check_temporal_literals`.
///
/// # Gate ordering for DML
/// `check_query_column_availability` returns Ok(()) immediately for DML (fail-open,
/// engine.rs ~line 2633: `_ => return Ok(())`). Therefore `check_temporal_literals` is
/// the first gate that fires E-QUERY-042 — no column gate interference.
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (6); F-P4-LOW-1; error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_dml_source_select_group_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    // LIMIT 10 makes the INSERT bounded (avoids E-QUERY-022 unbounded-write rejection).
    let result = engine
        .execute(
            "INSERT INTO test_events (hostname) \
             SELECT hostname FROM test_events \
             GROUP BY '2026-07-01T00:00:00Z' LIMIT 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "F-EQ42-P1-002: DML source_select GROUP BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy). \
         DML arm in check_temporal_literals has no skip_projection guard — fires early. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: DML GROUP BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: DML GROUP BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P1-002 (GREEN lock): RFC-3339 timestamp literal in a DML `source_select` ORDER BY
/// fires E-QUERY-042 (TemporalLiteralPosition::OrderBy).
///
/// Sibling of the DML GROUP BY test above; exercises the ORDER BY walker in the
/// DML source_select arm (materialization.rs ~line 2838).
///
/// Traces to: F-EQ42-P1-002; ADR-052 §D4 (v1.11) arm (7); F-P4-LOW-1; error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P1_002_dml_source_select_order_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "INSERT INTO test_events (hostname) \
             SELECT hostname FROM test_events \
             ORDER BY '2026-07-01T00:00:00Z' LIMIT 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "F-EQ42-P1-002: DML source_select ORDER BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy). \
         DML arm in check_temporal_literals has no skip_projection guard — fires early. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-002: DML ORDER BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P1-002: DML ORDER BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

// ── F-EQ42-P2-001: Expr::InSubquery walker coverage ──────────────────────────────────────────
//
// LOCAL adversary pass-2 finding F-EQ42-P2-001 (LOW): the F-EQ42-P1-002 doc block enumerates
// "Expr::InSubquery" as one of the four covered classes, but no test exercises the
// `Expr::InSubquery` arm in `check_expr_temporal_pos` (materialization.rs ~3439-3478).
// The `subquery_in_where_*` tests above parse to `Predicate::InSubquery` (WHERE clause,
// check_pred_raw_temporal path), NOT `Expr::InSubquery` (expression context,
// check_expr_temporal_pos path).
//
// `Expr::InSubquery` is produced by `build_sql_expr_parser` (sql_parser.rs ~line 852)
// when `field IN (SELECT ...)` appears in an expression-context position: SELECT projection,
// JOIN ON, GROUP BY, or ORDER BY. The SELECT projection surface is the most natural:
//   `SELECT hostname IN (SELECT hostname FROM test_events GROUP BY '<rfc3339>') FROM test_events`
//
// Call-site path (SELECT projection → Expr::InSubquery walker):
//   Early gate (skip_projection=true): SELECT items SKIPPED — Expr::InSubquery NOT reached.
//   E-QUERY-037: test_events registered → PASS.
//   E-QUERY-038: outer `hostname` field checked (Expr::InSubquery.field); subquery body
//     is fail-open (engine.rs ~line 1975, OBS-001) → PASS.
//   In-pipeline check_temporal_literals (skip_projection=false):
//     check_select_items_raw_temporal
//     → check_expr_temporal(Expr::InSubquery { field: hostname, subquery: ... })
//     → check_expr_temporal_pos(Expr::InSubquery{...}, TemporalCheckPos::Other)
//     → Expr::InSubquery arm (~3439): check_expr_temporal_pos(subquery.group_by[i], GroupBy)
//     → Literal::Timestamp + GroupBy → Err(TemporalLiteralInvalidPosition::GroupBy).
//
// These are GREEN LOCKS — the arm was present but unexercised. Adding these tests closes
// the coverage gap so a future refactor that accidentally removes the group_by/order_by
// walker from the Expr::InSubquery arm is caught immediately.

/// F-EQ42-P2-001 (GREEN lock): RFC-3339 timestamp literal in the GROUP BY of a subquery
/// appearing as a SELECT projection expression fires E-QUERY-042 (GroupBy).
///
/// `SELECT hostname IN (SELECT hostname FROM test_events GROUP BY '2026-07-01T00:00:00Z') FROM test_events`
///
/// # Why this is Expr::InSubquery, not Predicate::InSubquery
/// `build_sql_expr_parser` (sql_parser.rs ~852) produces `Expr::InSubquery` when
/// `field IN (SELECT ...)` appears in an EXPRESSION position (SELECT item, JOIN ON, etc.).
/// `build_sql_predicate_parser` (sql_parser.rs ~609) produces `Predicate::InSubquery`
/// when the same construct appears in a PREDICATE position (WHERE, HAVING).
/// The existing `subquery_in_where_*` tests exercise `Predicate::InSubquery`; this test
/// exercises the distinct `Expr::InSubquery` arm in `check_expr_temporal_pos`.
///
/// # Call-site path
/// Parser: `hostname IN (SELECT hostname FROM test_events GROUP BY '2026-07-01T00:00:00Z')`
/// as a SELECT projection → `SelectItem::Expr { expr: Expr::InSubquery { field: hostname,
/// subquery: SELECT hostname FROM test_events GROUP BY Literal::Timestamp(..) }, .. }`.
/// In-pipeline check_temporal_literals (skip_projection=false):
///   `check_select_items_raw_temporal` → `check_expr_temporal(Expr::InSubquery{...})`
///   → `check_expr_temporal_pos(Expr::InSubquery{...}, TemporalCheckPos::Other)`
///   → `Expr::InSubquery` arm (materialization.rs ~3461): `check_expr_temporal_pos(expr,
///   sub_primary, registry, TemporalCheckPos::GroupBy)` for each `subquery.group_by` expr
///   → `Literal::Timestamp` + GroupBy → `Err(TemporalLiteralInvalidPosition::GroupBy)`.
///
/// Traces to: F-EQ42-P2-001; ADR-052 §D4 (v1.11) arm (6); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P2_001_expr_insubquery_group_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT hostname IN \
               (SELECT hostname FROM test_events GROUP BY '2026-07-01T00:00:00Z') \
             FROM test_events",
            QueryOptions::default(),
        )
        .await;

    // Primary assertion: must be E-QUERY-042 with GroupBy position.
    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::GroupBy,
                ..
            })
        ),
        "F-EQ42-P2-001: Expr::InSubquery GROUP BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy). \
         check_expr_temporal_pos Expr::InSubquery arm (~3461) walks subquery.group_by \
         via check_expr_temporal_pos(GroupBy). ADR-052 §D4 (v1.11) arm (6). \
         Got: {result:?}"
    );

    // Negative: must NOT be E-QUERY-041.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P2-001: Expr::InSubquery GROUP BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    // Negative: must NOT be QueryPlanFailed (pre-fix analyst-hostile internal error).
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P2-001: Expr::InSubquery GROUP BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

/// F-EQ42-P2-001 (GREEN lock): RFC-3339 timestamp literal in the ORDER BY of a subquery
/// appearing as a SELECT projection expression fires E-QUERY-042 (OrderBy).
///
/// `SELECT hostname IN (SELECT hostname FROM test_events ORDER BY '2026-07-01T00:00:00Z') FROM test_events`
///
/// Sibling of the GROUP BY test above; exercises the ORDER BY walker in the
/// `Expr::InSubquery` arm (materialization.rs ~3469-3476).
///
/// # Call-site path
/// Same as the GROUP BY test except the subquery has `ORDER BY Literal::Timestamp(..)`.
/// `check_expr_temporal_pos(Expr::InSubquery{...}, Other)` → Expr::InSubquery arm:
///   `check_expr_temporal_pos(&mut order_expr.expr, ..., TemporalCheckPos::OrderBy)` →
///   `Literal::Timestamp` + OrderBy → `Err(TemporalLiteralInvalidPosition::OrderBy)`.
///
/// Traces to: F-EQ42-P2-001; ADR-052 §D4 (v1.11) arm (7); error-taxonomy.md §E-QUERY-042.
#[tokio::test]
async fn test_F_EQ42_P2_001_expr_insubquery_order_by_timestamp_fires_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT hostname IN \
               (SELECT hostname FROM test_events ORDER BY '2026-07-01T00:00:00Z') \
             FROM test_events",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::OrderBy,
                ..
            })
        ),
        "F-EQ42-P2-001: Expr::InSubquery ORDER BY '2026-07-01T00:00:00Z' must return \
         E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy). \
         check_expr_temporal_pos Expr::InSubquery arm (~3469) walks subquery.order_by \
         via check_expr_temporal_pos(OrderBy). ADR-052 §D4 (v1.11) arm (7). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P2-001: Expr::InSubquery ORDER BY timestamp must NOT return E-QUERY-041. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-EQ42-P2-001: Expr::InSubquery ORDER BY timestamp must NOT return QueryPlanFailed. \
         Got: {result:?}"
    );
}

// ── F-EQ42-P1-003: inject_now interaction lock ────────────────────────────────────────────────
//
// LOCAL adversary pass-1 finding F-EQ42-P1-003 (MED): the DEADARM fix added a
// `Literal::Timestamp` arm to `check_expr_temporal_pos` for GROUP BY / ORDER BY positions.
// `inject_now` (lib.rs) folds `Expr::Now` and `TimestampArithmetic { base: Now }` into
// `Literal::Timestamp`. If `GROUP BY NOW()` or `ORDER BY NOW()` could reach `inject_now`,
// the folded `Literal::Timestamp` would fire E-QUERY-042 — changing query behavior.
//
// EMPIRICAL DETERMINATION: `GROUP BY NOW()` and `ORDER BY NOW()` parse as
// `FuncCall::Scalar { func: ScalarFunc::Unknown("NOW"), args: [] }` via the `scalar_call`
// atom in `build_sql_expr_parser` (sql_parser.rs). The `build_temporal_rhs_parser`
// (filter_parser.rs) which produces `Expr::Now` is used ONLY in WHERE/HAVING predicate RHS
// — it is NOT wired into `build_sql_expr_parser`. Therefore:
//
//   1. `GROUP BY NOW()` parses as FuncCall::Scalar, NOT as Expr::Now.
//   2. `inject_now` does NOT fold FuncCall::Scalar("NOW") — only Expr::Now and
//      TimestampArithmetic { base: Now }.
//   3. `check_expr_temporal_pos` GroupBy arm matches Literal::Timestamp; it does NOT match
//      FuncCall::Scalar — the FuncCall arm recurses into args (empty) and returns Ok(()).
//   4. E-QUERY-042 does NOT fire for `GROUP BY NOW()`.
//
// For `GROUP BY NOW() - INTERVAL '1h'`:
//   `build_sql_expr_parser` has no arithmetic operators (no `+`, `-`, `*`, `/`).
//   After `NOW()` is matched by `scalar_call`, the remaining `- INTERVAL '1h'` is trailing
//   unparsed content. The full SQL SELECT parser rejects this as a parse error.
//   So `GROUP BY NOW() - INTERVAL '1h'` → QueryParseFailed (grammar constraint).
//
// These tests LOCK the grammar behavior so any future grammar extension that adds NOW()
// as a temporal expression in GROUP BY / ORDER BY is flagged by test failures that require
// explicit BC updates.

/// F-EQ42-P1-003 (grammar lock): `GROUP BY NOW()` does NOT fire E-QUERY-042.
///
/// `NOW()` in a GROUP BY clause parses as `FuncCall::Scalar { func: ScalarFunc::Unknown("NOW"),
/// args: [] }` via the `scalar_call` atom in `build_sql_expr_parser`. This is NOT `Expr::Now`.
/// `inject_now` does not fold it. `check_expr_temporal_pos` GroupBy arm matches
/// `Literal::Timestamp` — not `FuncCall`. The inject_now → E-QUERY-042 interaction is
/// UNREACHABLE from the PrismQL grammar surface for GROUP BY / ORDER BY positions.
///
/// # If this test starts failing
/// A grammar extension may have made NOW() produce Expr::Now in GROUP BY/ORDER BY context,
/// enabling inject_now to fold it to Literal::Timestamp, which the new arm would reject.
/// Update BC-2.11.021, ADR-052 §D4, and this comment before changing the assertion.
///
/// Traces to: F-EQ42-P1-003; ADR-052 §D4 v1.10; inject_now (lib.rs).
#[tokio::test]
async fn test_F_EQ42_P1_003_group_by_now_func_does_not_fire_e_query_042() {
    let engine = make_test_engine();

    // NOW() in GROUP BY parses as FuncCall::Scalar("NOW") — not Expr::Now / Literal::Timestamp.
    // inject_now does NOT fold FuncCall::Scalar. check_expr_temporal_pos FuncCall arm
    // recurses into empty args → Ok(()). E-QUERY-042 must NOT fire.
    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY NOW()",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be E-QUERY-042 — inject_now→Literal::Timestamp path is unreachable for NOW()
    // in GROUP BY because build_sql_expr_parser uses scalar_call (not build_temporal_rhs_parser).
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "F-EQ42-P1-003: GROUP BY NOW() must NOT return E-QUERY-042. \
         NOW() parses as FuncCall::Scalar, not Expr::Now — inject_now does not fold it. \
         Grammar: build_sql_expr_parser uses scalar_call atom (not build_temporal_rhs_parser). \
         If this assertion fails, a grammar extension has changed NOW() behavior in GROUP BY. \
         Update BC-2.11.021 and ADR-052 §D4 before changing this assertion. \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-041 either.
    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-003: GROUP BY NOW() must NOT return E-QUERY-041. Got: {result:?}"
    );
}

/// F-EQ42-P1-003 (grammar lock): `ORDER BY NOW()` does NOT fire E-QUERY-042.
///
/// Sibling of the GROUP BY NOW() test above; same parser mechanics apply to ORDER BY.
/// `ORDER BY NOW()` → `FuncCall::Scalar` → inject_now skips → check_expr_temporal_pos
/// FuncCall arm recurses into empty args → Ok(()). E-QUERY-042 unreachable.
///
/// Traces to: F-EQ42-P1-003; ADR-052 §D4 v1.10; inject_now (lib.rs).
#[tokio::test]
async fn test_F_EQ42_P1_003_order_by_now_func_does_not_fire_e_query_042() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT count(*) FROM test_events ORDER BY NOW()",
            QueryOptions::default(),
        )
        .await;

    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "F-EQ42-P1-003: ORDER BY NOW() must NOT return E-QUERY-042. \
         NOW() parses as FuncCall::Scalar, not Expr::Now — inject_now does not fold it. \
         Grammar: build_sql_expr_parser uses scalar_call atom (not build_temporal_rhs_parser). \
         If this assertion fails, a grammar extension has changed NOW() behavior in ORDER BY. \
         Update BC-2.11.021 and ADR-052 §D4 before changing this assertion. \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
        "F-EQ42-P1-003: ORDER BY NOW() must NOT return E-QUERY-041. Got: {result:?}"
    );
}

/// F-EQ42-P1-003 (grammar lock): `GROUP BY NOW() - INTERVAL '1h'` produces a parse error,
/// not E-QUERY-042.
///
/// `build_sql_expr_parser` (sql_parser.rs) has no binary arithmetic operators. After `NOW()`
/// is consumed by the `scalar_call` atom, the remaining `- INTERVAL '1h'` is trailing
/// unparsed content. The SQL SELECT parser rejects this as a `QueryParseFailed` error.
///
/// This locks the grammar constraint: the `TimestampArithmetic { base: Now }` → inject_now →
/// `Literal::Timestamp` collapse path (which WOULD trigger E-QUERY-042) is unreachable for
/// GROUP BY / ORDER BY expressions because `build_sql_expr_parser` does not include
/// `build_temporal_rhs_parser` or arithmetic operators in its atoms.
///
/// # If this test starts failing
/// A grammar extension may have added arithmetic operators to `build_sql_expr_parser`,
/// enabling `NOW() - INTERVAL '1h'` to produce `TimestampArithmetic { base: Now, ... }` in
/// GROUP BY context. At that point `inject_now` would fold it to `Literal::Timestamp` and
/// `check_expr_temporal_pos` GroupBy arm would fire E-QUERY-042 (CORRECT behavior).
/// Update this assertion to expect E-QUERY-042, and update BC-2.11.021, ADR-052 §D4.
///
/// Traces to: F-EQ42-P1-003; ADR-052 §D4 v1.10; inject_now (lib.rs); build_sql_expr_parser.
#[tokio::test]
async fn test_F_EQ42_P1_003_group_by_now_minus_interval_is_parse_error() {
    let engine = make_test_engine();

    // NOW() - INTERVAL '1h' in GROUP BY: build_sql_expr_parser has no arithmetic operators.
    // scalar_call matches NOW(); remaining '- INTERVAL '1h'' is trailing → parse error.
    let result = engine
        .execute(
            "SELECT count(*) FROM test_events GROUP BY NOW() - INTERVAL '1h'",
            QueryOptions::default(),
        )
        .await;

    // Must be a parse failure — arithmetic continuation after NOW() is not in the grammar.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-EQ42-P1-003: GROUP BY NOW() - INTERVAL '1h' must return QueryParseFailed. \
         build_sql_expr_parser has no arithmetic operators; trailing '- INTERVAL' is \
         unparsed content that makes the full SELECT fail to parse. \
         If this assertion fails, the grammar now supports arithmetic in GROUP BY — \
         in that case E-QUERY-042 should fire (update assertion + BC-2.11.021 + ADR-052 §D4). \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-042 — no Literal::Timestamp reaches GROUP BY arm.
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "F-EQ42-P1-003: GROUP BY NOW() - INTERVAL '1h' must NOT return E-QUERY-042 \
         (query fails at parse, before check_temporal_literals runs). Got: {result:?}"
    );
}

// ── DEFECT-PQL-FNCALL-LHS-001: pipe `| where` fn-call LHS grammar extension ──
//
// Tests cover BC-2.11.004 v1.31 arm (4) (EC-11-004-005, EC-11-004-006) and the
// ADR-052 §D4 v1.12 architect-adjudicated Option A grammar extension.
//
// DEFECT: The pipe `| where` grammar (`build_predicate_parser` in `filter_parser.rs`)
// only admitted `field_path` as comparison LHS.  Queries like
// `lower(device_id) = '2026-06-24'` therefore FAILED AT PARSE TIME with a generic
// E-QUERY-001 (`QueryParseFailed`) and surfaced as `-32000 INTERNAL_ERROR` to callers,
// bypassing the analyst-friendly `-32602 INVALID_PARAMS` E-QUERY-042 path entirely.
//
// FIX (architect Option A): extend the pipe `| where` grammar with a
// `fn_call_comparison` production (FuncCall::Scalar LHS only) BEFORE `field_comparison`.
// Parse then SUCCEEDS; the plan-time `check_temporal_literals` arm (4) fires E-QUERY-042
// when RHS is a date-like literal, or the query passes to DataFusion when it is not.
//
// RED GATE: Tests 1-3 below assert the POST-FIX behavior.  In the unimplemented (RED)
// state they FAIL because the grammar rejects fn-call LHS at parse time and returns
// `QueryParseFailed` instead.  Test 4 is a scope guard that passes in both states.

/// Build a `TableRegistry` with sensor "crowdstrike" / table "detections" registered as
/// "crowdstrike_detections".  Columns:
///   - `device_id:  ColumnType::String`  — fn-call LHS tests (EC-11-004-005/006)
///   - `timestamp:  ColumnType::Datetime` — included for schema completeness
///   - `risk_score: ColumnType::Float`   — OBS-001/OBS-002 aggregate-in-where tests
///
/// Gate ordering guarantee: E-QUERY-037 passes (table IS registered).
/// E-QUERY-038 gate walks fn-call args for column validation (FuncCall arm of
/// `collect_predicate_columns`).  "no_such_col_xyz" is NOT in this registry,
/// so `lower(no_such_col_xyz) = 'active'` → E-QUERY-038 after the grammar fix.
fn make_crowdstrike_detections_registry() -> Arc<TableRegistry> {
    use prism_core::ColumnType;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

    let registry = Arc::new(TableRegistry::new());
    let spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike detections (test fixture — DEFECT-PQL-FNCALL-LHS-001)",
        AuthType::ApiKey,
        "https://crowdstrike.invalid",
        vec![TableSpec::new_point_in_time(
            "detections",
            "security_finding",
            vec![
                ColumnSpec::new("device_id", ColumnType::String, None, vec![]),
                ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                ColumnSpec::new("risk_score", ColumnType::Float, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&spec)
        .expect("register crowdstrike sensor (DEFECT-PQL-FNCALL-LHS-001 fixture) must not fail");
    registry
}

/// Build a `QueryEngine` wired with the "crowdstrike_detections" registry.
fn make_crowdstrike_detections_engine() -> QueryEngine {
    let registry = make_crowdstrike_detections_registry();
    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

/// EC-11-004-005 end-to-end: pipe `| where` with fn-call LHS and date-like RHS must
/// return E-QUERY-042 (NonColumnLhsComparison), NOT a generic parse error.
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) = '2026-06-24'`
///
/// # Red Gate pre-fix failure (the DEFECT)
/// `build_predicate_parser` only admits `field_path` as comparison LHS.  Parsing
/// `lower(device_id) = '2026-06-24'` encounters `lower(` and fails at parse time
/// with `PrismError::QueryParseFailed { .. }` (E-QUERY-001 offset error), which the
/// MCP layer surfaces as `-32000 INTERNAL_ERROR`.
/// This test asserts `TemporalLiteralInvalidPosition(NonColumnLhsComparison)` → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// Grammar extension (ADR-052 §D4 v1.12 Option A) adds `fn_call_comparison` production
/// (FuncCall::Scalar LHS) BEFORE `field_comparison` in `build_predicate_parser`.
/// Parse SUCCEEDS, producing:
///   `Predicate::Compare { lhs: Expr::FuncCall(FuncCall::Scalar), rhs: Literal::RawTemporalLiteral("2026-06-24"), .. }`
/// `check_temporal_literals` arm (4) detects non-Field LHS + RawTemporalLiteral RHS →
///   `Err(PrismError::TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: "2026-06-24" })`
/// MCP layer maps to `-32602 INVALID_PARAMS` (analyst-friendly).
///
/// # SAP-3 spec-arm reachability
/// This test enters from the PUBLIC parser surface (`engine.execute()`) per SAP-3
/// spec-arm reachability discipline.  It does NOT use a synthetic AST.
///
/// # SID-2 composed-output discipline
/// Asserts both the error variant AND the full composed Display message string.
///
/// Traces to: BC-2.11.004 v1.31 EC-11-004-005; ADR-052 §D4 v1.12 arm (4);
///            error-taxonomy.md §E-QUERY-042 v2.14 (POL-24 byte-verbatim).
#[tokio::test]
async fn test_BC_2_11_004_ec11_004_005_pipe_fncall_lhs_date_like_rejects_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    // RED GATE: grammar rejects `lower(device_id)` as comparison LHS → QueryParseFailed.
    // POST-FIX: grammar extension makes parse succeed; check_temporal_literals arm (4)
    //           fires → TemporalLiteralInvalidPosition(NonColumnLhsComparison).
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be an error.
    assert!(
        result.is_err(),
        "EC-11-004-005: `lower(device_id) = '2026-06-24'` in pipe | where must return \
         Err(E-QUERY-042). Got Ok. \
         Check: fn_call_comparison grammar production + check_temporal_literals arm (4)."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: must be E-QUERY-042 NonColumnLhsComparison.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "EC-11-004-005: error must be PrismError::TemporalLiteralInvalidPosition \
         (NonColumnLhsComparison). ADR-052 §D4 v1.12 arm (4). \
         RED failure: grammar currently returns QueryParseFailed (parse-time rejection). \
         Got: {err:?} (Display: {display})"
    );

    // value_prefix must be the first ≤50 chars of the offending literal.
    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "EC-11-004-005: value_prefix must start with '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // SID-2 composed-output discipline: assert full Display string, not just the variant.
    // POL-24 byte-verbatim anchor for E-QUERY-042 NonColumnLhsComparison.
    assert!(
        display.contains("E-QUERY-042: A date-like literal compared against a computed expression"),
        "EC-11-004-005: Display must contain the canonical E-QUERY-042 NonColumnLhsComparison \
         message prefix (error-taxonomy.md §E-QUERY-042 v2.14, POL-24 byte-verbatim). \
         Got: {display}"
    );

    // Must NOT be QueryParseFailed — that was the defect (-32000 INTERNAL_ERROR to callers).
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "EC-11-004-005: error must NOT be QueryParseFailed (-32000). \
         That was the pre-fix defect (DEFECT-PQL-FNCALL-LHS-001). \
         Post-fix: grammar parses fn-call LHS; E-QUERY-042 fires at plan time (-32602). \
         Got: {err:?}"
    );
}

/// EC-11-004-006: pipe `| where` with fn-call LHS and NON-date-like RHS must succeed
/// (no temporal interception, no parse error).
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) = 'active'`
///
/// # Red Gate pre-fix failure (the DEFECT)
/// `build_predicate_parser` rejects `lower(device_id)` at parse time with
/// `PrismError::QueryParseFailed` — the fn-call LHS production does not exist yet.
/// This test asserts NOT `QueryParseFailed` and NOT `TemporalLiteralInvalidPosition` →
/// FAILS because the actual error IS `QueryParseFailed`. ✓
///
/// # Post-fix state (GREEN)
/// Grammar extension: `lower(device_id) = 'active'` parses to
///   `Predicate::Compare { lhs: FuncCall::Scalar(...), rhs: Literal::String("active"), .. }`
/// `check_temporal_literals`: `'active'` is NOT in `is_date_like` Acceptance Set →
///   no `RawTemporalLiteral` emitted → walker returns `Ok(())`.
/// E-QUERY-038: `device_id` IS in schema → passes.
/// Query reaches DataFusion (fails with sensor-not-found or execution error — acceptable).
///
/// Traces to: BC-2.11.004 v1.31 EC-11-004-006; ADR-052 §D4 v1.12 Option A;
///            EC-11-004-006 "fn-call args walked by collect_predicate_columns FuncCall arm".
#[tokio::test]
async fn test_BC_2_11_004_ec11_004_006_pipe_fncall_lhs_non_date_like_rhs_succeeds() {
    use crate::{
        filter_parser::PrismQlParser, materialization::execute_against_session,
        memory::build_session_context,
    };

    // F-PQLFN-P18-MED-001 RED GATE (fix-burst 14):
    // Grammar is already extended (fn-call LHS parses). The active defect is in the emitter:
    // `pipe_sql_emitter::expr_to_sql` has no `Expr::FuncCall` arm — the catch-all fires:
    //   Err(QueryExecutionFailed { "Complex expression in pipe WHERE stage is not yet
    //   supported. Rewrite as SQL." })
    //
    // Use `execute_against_session` directly (bypasses run_materialization_pipeline step-6
    // early-return guard) to guarantee the emitter is reached and the defect is exercised.
    //
    // POST-FIX (GREEN): `Expr::FuncCall` arm added to `expr_to_sql` → `lower(device_id)`
    // emits `lower(device_id)` SQL → DataFusion executes → Ok (empty result set).
    let query = "FROM crowdstrike_detections | where lower(device_id) = 'active'";
    let ctx = build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
    let ast = PrismQlParser::parse(query)
        .expect("grammar fn-call LHS extension is already green — query must parse");
    let result = execute_against_session(&ctx, query, &ast, std::collections::HashMap::new()).await;

    // BC-2.11.004 EC-11-004-006 spec-promised outcome: fn-call LHS in pipe | where with
    // non-date-like RHS must reach DataFusion execution (Ok with 0 rows or sensor error).
    // RED: Err(QueryExecutionFailed { "Complex expression..." }) — emitter FuncCall arm missing.
    //
    // Diagnostic-first ordering (F-PQLFN-P19-OBS-001): the specific FuncCall catch-all check
    // runs first so its message fires when the exact failing arm triggers; the general is_ok()
    // below catches any other Err variant.
    assert!(
        !matches!(
            &result,
            Err(PrismError::QueryExecutionFailed { detail, .. })
            if detail.contains("Complex expression")
        ),
        "EC-11-004-006: must NOT be Err(QueryExecutionFailed {{ 'Complex expression...' }}). \
         The catch-all in pipe_sql_emitter::expr_to_sql emits this exact detail string. \
         Fix: add Expr::FuncCall arm to expr_to_sql (e.g. `lower(device_id)` → fn-call SQL). \
         EC-11-004-006 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
    assert!(
        result.is_ok(),
        "EC-11-004-006: `lower(device_id) = 'active'` in pipe | where must return Ok \
         (DataFusion execution, non-date-like RHS passes all plan gates). \
         Fires only when the Err is not QueryExecutionFailed{{Complex expression}} \
         (that case is diagnosed above). \
         EC-11-004-006 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
}

/// EC-11-004-006 (SqlPipe variant): SqlPipe `| where` fn-call LHS with NON-date-like
/// RHS must succeed — no temporal interception, no "Complex expression" error.
///
/// Query: `SELECT * FROM crowdstrike_detections | where lower(device_id) = 'active'`
///
/// # Red Gate pre-fix failure (F-PQLFN-P18-MED-001)
/// `pipe_sql_emitter::expr_to_sql` has no `Expr::FuncCall` arm.
/// `sqlpipe_to_executable_sql` → `apply_stage(PipeStage::Where)` → `apply_where` →
/// `predicate_to_datafusion_sql` → `expr_to_sql(FuncCall)` → catch-all →
/// Err(QueryExecutionFailed { "Complex expression in pipe WHERE stage is not yet
/// supported. Rewrite as SQL." }).
/// This test asserts `result.is_ok()` → FAILS on current HEAD. ✓
///
/// # Post-fix state (GREEN)
/// `Expr::FuncCall` arm added → `lower(device_id)` emits `lower(device_id)` SQL →
/// DataFusion executes → Ok (empty result set).
///
/// Traces to: BC-2.11.004 v1.31 EC-11-004-006; ADR-052 §D4 v1.12 Option A.
#[tokio::test]
async fn test_BC_2_11_004_ec11_004_006_sqlpipe_fncall_lhs_non_date_like_rhs_succeeds() {
    use crate::{
        filter_parser::PrismQlParser, materialization::execute_against_session,
        memory::build_session_context,
    };

    // F-PQLFN-P18-MED-001 RED GATE (SqlPipe variant):
    // `expr_to_sql` catch-all fires for Expr::FuncCall LHS in the SqlPipe | where stage.
    //
    // Use `execute_against_session` directly to bypass step-6 early-return guard
    // and guarantee the SqlPipe emitter path is exercised.
    let query = "SELECT * FROM crowdstrike_detections | where lower(device_id) = 'active'";
    let ctx = build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
    let ast = PrismQlParser::parse(query)
        .expect("grammar fn-call LHS extension is already green — query must parse");
    let result = execute_against_session(&ctx, query, &ast, std::collections::HashMap::new()).await;

    // BC-2.11.004 EC-11-004-006 spec-promised outcome (SqlPipe mode): must reach DataFusion.
    // RED: Err(QueryExecutionFailed { "Complex expression..." }) — emitter FuncCall arm missing.
    //
    // Diagnostic-first ordering (F-PQLFN-P19-OBS-001): the specific FuncCall catch-all check
    // runs first so its message fires when the exact failing arm triggers; the general is_ok()
    // below catches any other Err variant.
    assert!(
        !matches!(
            &result,
            Err(PrismError::QueryExecutionFailed { detail, .. })
            if detail.contains("Complex expression")
        ),
        "EC-11-004-006 (SqlPipe): must NOT be Err(QueryExecutionFailed {{ 'Complex expression...' }}). \
         The catch-all in pipe_sql_emitter::expr_to_sql emits this exact detail string. \
         Fix: add Expr::FuncCall arm to expr_to_sql. \
         EC-11-004-006 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
    assert!(
        result.is_ok(),
        "EC-11-004-006 (SqlPipe): `lower(device_id) = 'active'` in SqlPipe | where must \
         return Ok (DataFusion execution). \
         Fires only when the Err is not QueryExecutionFailed{{Complex expression}} \
         (that case is diagnosed above). \
         EC-11-004-006 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
}

/// EC-11-004-006 E-QUERY-038 interaction: fn-call args must be walked by
/// `collect_predicate_columns` FuncCall arm — nonexistent column inside fn-call → E-QUERY-038.
///
/// Query: `FROM crowdstrike_detections | where lower(no_such_col_xyz) = 'active'`
///
/// # Red Gate pre-fix failure (the DEFECT)
/// Grammar rejects `lower(no_such_col_xyz)` at parse time with `QueryParseFailed`.
/// Test asserts `ColumnNotFound` → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// Grammar extension: parse succeeds.
/// `collect_predicate_columns` FuncCall arm walks `lower(no_such_col_xyz)`, extracts
/// `no_such_col_xyz` as a column reference, checks against schema.
/// `crowdstrike_detections` has `[device_id, timestamp]` — `no_such_col_xyz` is absent →
/// E-QUERY-038 `ColumnNotFound { column: "no_such_col_xyz", table: "crowdstrike_detections" }`.
///
/// # SID-2 composed-output discipline
/// Asserts both the `ColumnNotFound` variant fields AND the full Display string.
///
/// Traces to: BC-2.11.004 v1.31 EC-11-004-006 (E-QUERY-038 interaction note);
///            BC-2.11.016 v1.25 (collect_predicate_columns FuncCall arm).
#[tokio::test]
async fn test_BC_2_11_004_ec11_004_006_pipe_fncall_lhs_nonexistent_col_e_query_038() {
    let engine = make_crowdstrike_detections_engine();

    // RED GATE: grammar rejects `lower(no_such_col_xyz)` at parse time → QueryParseFailed.
    // POST-FIX: grammar parses OK; collect_predicate_columns walks fn-call args;
    //           no_such_col_xyz not in crowdstrike_detections schema → E-QUERY-038.
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(no_such_col_xyz) = 'active'",
            QueryOptions::default(),
        )
        .await;

    // Must be an error.
    assert!(
        result.is_err(),
        "EC-11-004-006/E-QUERY-038: `lower(no_such_col_xyz) = 'active'` must return \
         Err(E-QUERY-038). Got Ok. \
         Check: collect_predicate_columns FuncCall arm; no_such_col_xyz not in schema."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: must be E-QUERY-038 ColumnNotFound.
    assert!(
        matches!(&err, PrismError::ColumnNotFound(ref d) if d.column == "no_such_col_xyz"),
        "EC-11-004-006/E-QUERY-038: error must be PrismError::ColumnNotFound with \
         column = 'no_such_col_xyz'. \
         RED failure: grammar currently returns QueryParseFailed (parse-time rejection). \
         Got: {err:?} (Display: {display})"
    );

    // SID-2 composed-output discipline: assert the full Display message, not just the variant.
    assert!(
        display.contains("E-QUERY-038"),
        "EC-11-004-006/E-QUERY-038: Display must contain 'E-QUERY-038'. Got: {display}"
    );
    assert!(
        display.contains("no_such_col_xyz"),
        "EC-11-004-006/E-QUERY-038: Display must contain 'no_such_col_xyz' \
         (the missing column name per E-QUERY-038 message template). Got: {display}"
    );

    // Table name must be crowdstrike_detections in the error payload.
    if let PrismError::ColumnNotFound(ref details) = err {
        assert_eq!(
            details.table, "crowdstrike_detections",
            "EC-11-004-006/E-QUERY-038: ColumnNotFoundDetails.table must be \
             'crowdstrike_detections'. Got: {:?}",
            details.table
        );
    }

    // Must NOT be QueryParseFailed — that was the pre-fix defect.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "EC-11-004-006/E-QUERY-038: error must NOT be QueryParseFailed. \
         Post-fix: grammar parses fn-call LHS; E-QUERY-038 fires at plan time. \
         Got: {err:?}"
    );
}

/// Scope guard — BC-2.11.004 §INV: aggregate fn-call must NOT become valid in pipe `| where`.
///
/// Query: `FROM crowdstrike_detections | where count(device_id) = 5`
///
/// Per ADR-052 §D4 v1.12 and ADR-048 D.3, the `fn_call_comparison` grammar extension
/// in `build_predicate_parser` is RESTRICTED to `FuncCall::Scalar` only.  Aggregate
/// fn-calls (`count`, `sum`, `avg`, `min`, `max`) remain invalid in pipe `| where`
/// and must continue to produce an error (parse error or appropriate plan-time gate).
/// The HAVING path (ADR-048 D.3) is SQL-mode only.
///
/// # State in both RED and GREEN
/// This test passes in BOTH RED and GREEN: the query errors in RED (grammar rejects all
/// fn-call LHS), and must also error in GREEN (grammar extension correctly restricts to
/// FuncCall::Scalar, rejecting aggregate fn-calls).
/// This is a scope-guard / regression test that catches accidental broadening of the
/// fn_call_comparison production to aggregate functions.
///
/// # Rationale (why not a RED-only test)
/// The value of this test is prospective: if the implementer accidentally allows
/// `count()` as a valid pipe `| where` LHS, this test fails in GREEN, preventing a
/// silent semantic regression.
///
/// Traces to: BC-2.11.004 v1.31 EC-11-004-006 scope note;
///            ADR-048 D.3 (HAVING aggregate functions are SQL-mode only);
///            ADR-052 §D4 v1.12 (FuncCall::Scalar only in fn_call_comparison).
#[tokio::test]
async fn test_BC_2_11_004_invariant_pipe_where_aggregate_fncall_remains_invalid() {
    let engine = make_crowdstrike_detections_engine();

    // Both RED and GREEN: `count(device_id) = 5` in pipe | where must error.
    // RED:   grammar rejects ALL fn-call LHS → QueryParseFailed.
    // GREEN: grammar extension allows FuncCall::Scalar only; `count` is aggregate →
    //        grammar rejects (or plan gate rejects) → error.
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where count(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Must NOT succeed — aggregate fn-call is NEVER valid in pipe | where.
    assert!(
        result.is_err(),
        "BC-2.11.004 §INV: `count(device_id) = 5` in pipe | where must return Err. \
         Aggregate fn-calls are invalid in | where (ADR-048 D.3: HAVING path is SQL-mode only). \
         The fn_call_comparison grammar extension must NOT inadvertently allow aggregate LHS. \
         Got Ok — scope has been incorrectly broadened beyond FuncCall::Scalar."
    );
}

// ── DEFECT-PQL-FNCALL-LHS-001 fix-burst 1 ────────────────────────────────────
//
// SAP-3 e2e locks (GREEN on arrival) + RED gates for adversary pass-1 findings:
//
//   F-PQLFN-P1-MED-002  SQL WHERE arm-4 e2e lock (SAP-3)
//   F-PQLFN-P1-MED-003  filter-mode arm-4 e2e locks
//   F-PQLFN-P1-MED-004  unknown scalar in pipe/filter/sqlpipe | where → E-QUERY-039 (RED)
//   F-PQLFN-P1-OBS-001  empty-arg edges: count() GREEN lock + foo() RED
//   F-PQLFN-P1-OBS-002  DataFusion-only aggregate (stddev) not in 7-name blocklist (RED)
//
// Engine fixture:
//   make_crowdstrike_detections_engine()          — no infusion registry (temporal tests)
//   make_crowdstrike_engine_with_empty_infusion() — empty InfusionRegistry (E-QUERY-039 tests)
//
// SAP-3 definition: every code path that processes a temporal literal must be exercised
// end-to-end through the public `engine.execute()` surface, not synthetic ASTs.

/// Build a `QueryEngine` wired with `crowdstrike_detections` AND an empty
/// `InfusionRegistry` (Some, but zero entries).
///
/// Required for E-QUERY-039 RED tests: the gate only fires when the registry is Some.
/// With registry = None, `check_enrich_udf_availability` returns Ok(()) immediately.
/// An empty (Some) registry means all unknown scalar names → E-QUERY-039 (after fix).
fn make_crowdstrike_engine_with_empty_infusion() -> QueryEngine {
    let registry = make_crowdstrike_detections_registry();
    let empty_infusion_registry = Arc::new(prism_spec_engine::InfusionRegistry::new());
    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
    .with_infusion_registry(empty_infusion_registry)
}

// ── F-PQLFN-P1-MED-002: SQL WHERE arm-4 end-to-end regression lock ───────────

/// F-PQLFN-P1-MED-002 (GREEN, SAP-3 e2e lock): SQL WHERE fn-call LHS with
/// date-like RHS must return E-QUERY-042 NonColumnLhsComparison.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE lower(device_id) = '2026-06-24'`
///
/// SAP-3 spec-arm reachability: the shared `build_predicate_parser` (filter_parser.rs)
/// supplies both pipe `| where` and SQL WHERE predicates.  The EC-11-003-007 test vector
/// is exercised here end-to-end through `engine.execute()` against the SQL WHERE arm,
/// confirming arm (4) (`check_temporal_literals` NonColumnLhsComparison) is reachable
/// from the SQL surface, not just the pipe surface.
///
/// # GREEN on arrival
/// The `fn_call_comparison` grammar extension (@7a56e53b) makes the SQL WHERE
/// `lower(device_id)` form parse successfully.  `check_temporal_literals` arm (4)
/// fires for the non-Field LHS + RawTemporalLiteral("2026-06-24") combination.
///
/// # SID-2 composed-output discipline
/// Asserts both the error variant AND the full composed Display message string
/// byte-verbatim per POL-24 (error-taxonomy.md §E-QUERY-042 NonColumnLhsComparison).
///
/// Traces to: BC-2.11.003 v1.12 EC-11-003-007; error-taxonomy.md §E-QUERY-042 v2.14;
///            ADR-052 §D4 v1.10 arm (4); SAP-3.
#[tokio::test]
async fn test_BC_2_11_003_ec11_003_007_sql_where_fncall_lhs_date_like_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE lower(device_id) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-002: SQL WHERE lower(device_id) = '2026-06-24' must return Err(E-QUERY-042). \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: E-QUERY-042 NonColumnLhsComparison.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "MED-002 (SAP-3): SQL WHERE fn-call LHS with date-like RHS must produce \
         PrismError::TemporalLiteralInvalidPosition(NonColumnLhsComparison). \
         The shared build_predicate_parser makes SQL WHERE arm-4 reachable (SAP-3). \
         Got: {err:?} (Display: {display})"
    );

    // value_prefix must be the first ≤50 chars of the offending literal.
    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "MED-002: value_prefix must start with '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    // SID-2: assert full composed Display message (POL-24 byte-verbatim).
    let expected_prefix =
        "E-QUERY-042: A date-like literal compared against a computed expression \
                           cannot be type-checked at plan time.";
    assert!(
        display.contains(expected_prefix),
        "MED-002 (SID-2/POL-24): Display must contain the canonical \
         E-QUERY-042 NonColumnLhsComparison message prefix. \
         error-taxonomy.md §E-QUERY-042 v2.14. \
         Got: {display}"
    );

    // Must NOT be QueryParseFailed — that was the pre-fix defect.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "MED-002: error must NOT be QueryParseFailed. \
         Post-fix: grammar parses fn-call LHS; E-QUERY-042 fires at plan time (-32602). \
         Got: {err:?}"
    );
}

// ── F-PQLFN-P1-MED-003: filter-mode arm-4 locks ──────────────────────────────

/// F-PQLFN-P1-MED-003a (GREEN, SAP-3 e2e lock): filter-mode fn-call LHS with
/// date-like RHS must return E-QUERY-042 NonColumnLhsComparison.
///
/// Query: `crowdstrike_detections | lower(device_id) = '2026-06-24'`
///
/// Filter mode (`Ast::Filter`) uses the same `build_predicate_parser` as pipe `| where`.
/// The `fn_call_comparison` production parses `lower(device_id)` as
/// `FuncCall::Scalar(Unknown("lower"), [device_id])`.  `check_temporal_literals`
/// arm (4) fires: non-Field LHS + RawTemporalLiteral("2026-06-24") →
/// E-QUERY-042 NonColumnLhsComparison.
///
/// # GREEN on arrival
/// @7a56e53b: fn_call_comparison in build_predicate_parser + check_temporal_literals
/// arm (4) already handle Ast::Filter predicates.
///
/// Traces to: BC-2.11.003 v1.12 EC-11-003-007 (filter-mode parity);
///            ADR-052 §D4 v1.10 arm (4); SAP-3.
#[tokio::test]
async fn test_BC_2_11_003_ec11_003_007_filter_fncall_lhs_date_like_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | lower(device_id) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-003a: filter-mode lower(device_id) = '2026-06-24' must return \
         Err(E-QUERY-042). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "MED-003a: filter-mode fn-call LHS date-like RHS must produce \
         TemporalLiteralInvalidPosition(NonColumnLhsComparison). \
         Got: {err:?} (Display: {display})"
    );

    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "MED-003a: value_prefix must start with '2026-06-24'. Got: {value_prefix:?}"
        );
    }

    assert!(
        display.contains("E-QUERY-042"),
        "MED-003a: Display must contain 'E-QUERY-042'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "MED-003a: must NOT be QueryParseFailed. Got: {err:?}"
    );
}

/// F-PQLFN-P1-MED-003b: filter-mode fn-call LHS with NON-date-like RHS must succeed.
///
/// Query: `crowdstrike_detections | lower(device_id) = 'active'`
///
/// # Red Gate pre-fix failure (F-PQLFN-P18-MED-001)
/// `pipe_sql_emitter::predicate_to_datafusion_sql` → `expr_to_sql(FuncCall)` → catch-all →
/// Err(QueryExecutionFailed { "Complex expression..." }).
/// The Filter arm wraps this as:
///   Err(QueryExecutionFailed { "filter SQL lowering failed: ... Complex expression ..." }).
/// This test asserts `result.is_ok()` → FAILS on current HEAD. ✓
///
/// # Post-fix state (GREEN)
/// `Expr::FuncCall` arm added → `lower(device_id)` emits SQL → DataFusion executes →
/// Ok (empty result set — no tables registered in the direct-path session).
///
/// Traces to: BC-2.11.003 v1.12 EC-11-003-007 (non-date-like passthrough);
///            ADR-052 §D4 v1.10 Option A; F-PQLFN-P18-MED-001.
#[tokio::test]
async fn test_BC_2_11_003_ec11_003_007_filter_fncall_lhs_non_date_rhs_not_rejected() {
    use crate::{
        filter_parser::PrismQlParser, materialization::execute_against_session,
        memory::build_session_context,
    };

    // F-PQLFN-P18-MED-001 RED GATE (fix-burst 14):
    // `predicate_to_datafusion_sql` calls `expr_to_sql(FuncCall)` which hits the catch-all.
    // The Filter arm wraps the inner error: "filter SQL lowering failed: ... Complex expression...".
    //
    // Use `execute_against_session` directly to bypass step-6 early-return guard.
    let query = "crowdstrike_detections | lower(device_id) = 'active'";
    let ctx = build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
    let ast = PrismQlParser::parse(query)
        .expect("grammar fn-call LHS extension is already green — query must parse");
    let result = execute_against_session(&ctx, query, &ast, std::collections::HashMap::new()).await;

    // BC-2.11.003 EC-11-003-007 spec-promised outcome: filter fn-call LHS with
    // non-date-like RHS must reach DataFusion (Ok).
    // RED: Err(QueryExecutionFailed { "filter SQL lowering failed: ... Complex expression ..." }).
    //
    // Diagnostic-first ordering (F-PQLFN-P19-OBS-001): the specific FuncCall catch-all check
    // runs first so its message fires when the exact failing arm triggers; the general is_ok()
    // below catches any other Err variant.
    assert!(
        !matches!(
            &result,
            Err(PrismError::QueryExecutionFailed { detail, .. })
            if detail.contains("Complex expression")
        ),
        "EC-11-003-007: must NOT be Err(QueryExecutionFailed {{ '...Complex expression...' }}). \
         The catch-all in pipe_sql_emitter::expr_to_sql fires and is wrapped by the \
         filter SQL lowering error. Fix: add Expr::FuncCall arm to expr_to_sql. \
         EC-11-003-007 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
    assert!(
        result.is_ok(),
        "EC-11-003-007: filter-mode `lower(device_id) = 'active'` must return Ok \
         (DataFusion execution). \
         Fires only when the Err is not QueryExecutionFailed{{Complex expression}} \
         (that case is diagnosed above). \
         EC-11-003-007 / F-PQLFN-P18-MED-001 / F-PQLFN-P19-OBS-001. Got: {result:?}"
    );
}

// ── F-PQLFN-P1-MED-004: unknown scalar in pipe/filter/sqlpipe | where ─────────
//
// RED tests — MUST FAIL against @7a56e53b.
//
// Gap: `check_enrich_udf_availability` walks:
//   Ast::Pipe   → PipeStage::Enrich only (NOT PipeStage::Where predicates)
//   Ast::Filter → `_ => {}` (entirely skipped)
//   Ast::SqlPipe → PipeStage::Enrich + SQL head (NOT SqlPipe PipeStage::Where)
//
// A `FuncCall::Scalar { func: ScalarFunc::Unknown("notafunc_xyz"), ... }` in a `| where`
// or filter predicate bypasses the gate and reaches DataFusion, which crashes with
// QueryPlanFailed / -32000 INTERNAL_ERROR instead of the analyst-friendly E-QUERY-039.
//
// The fix: extend the Ast::Pipe, Ast::Filter, and Ast::SqlPipe arms to also walk
// PipeStage::Where / filter / SqlPipe-stage predicates for ScalarFunc::Unknown names.
//
// Test design: requires empty infusion registry (Some) so the gate runs.
// With None, the gate is skipped entirely regardless of fix.

/// F-PQLFN-P1-MED-004 RED (1/3): pipe `| where` unknown scalar → E-QUERY-039.
///
/// Query: `FROM crowdstrike_detections | where notafunc_xyz(device_id) = 'active'`
///
/// # RED gate pre-fix failure (@7a56e53b)
/// `check_enrich_udf_availability` walks Ast::Pipe → PipeStage::Enrich only.
/// `notafunc_xyz` in PipeStage::Where predicate is not found → Ok(()).
/// Query proceeds to DataFusion → `notafunc_xyz` is not a registered UDF →
/// QueryPlanFailed (some kind of -32000 error).
/// Test asserts `EnrichUdfNotFound` → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// Gate also walks PipeStage::Where predicates for ScalarFunc::Unknown names.
/// Finds `notafunc_xyz` → not in DataFusion built-in set → not in empty registry →
/// E-QUERY-039 `EnrichUdfNotFound { infusion: "notafunc_xyz", available_infusions: [] }`.
///
/// Traces to: BC-2.11.019 v1.6 §Precondition 1(b); error-taxonomy.md §E-QUERY-039.
#[tokio::test]
async fn test_BC_2_11_019_med_004_pipe_where_unknown_scalar_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    // RED GATE: gate currently skips PipeStage::Where predicates → QueryPlanFailed.
    // POST-FIX: gate walks PipeStage::Where → notafunc_xyz not in registry → E-QUERY-039.
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where notafunc_xyz(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-004 (pipe): notafunc_xyz(device_id) = 'active' in | where must return \
         Err(E-QUERY-039). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: EnrichUdfNotFound with infusion="notafunc_xyz".
    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "notafunc_xyz"),
        "MED-004 (pipe): error must be PrismError::EnrichUdfNotFound \
         with infusion = 'notafunc_xyz'. \
         RED failure: gate currently skips | where predicates → QueryPlanFailed. \
         Got: {err:?} (Display: {display})"
    );

    // available_infusions must be empty (no infusions registered).
    if let PrismError::EnrichUdfNotFound(ref d) = err {
        assert!(
            d.available_infusions.is_empty(),
            "MED-004 (pipe): available_infusions must be [] (empty registry). \
             Got: {:?}",
            d.available_infusions
        );
    }

    // Display must contain "E-QUERY-039" and the infusion name.
    assert!(
        display.contains("E-QUERY-039"),
        "MED-004 (pipe): Display must contain 'E-QUERY-039'. Got: {display}"
    );
    assert!(
        display.contains("notafunc_xyz"),
        "MED-004 (pipe): Display must contain 'notafunc_xyz'. Got: {display}"
    );

    // Must NOT be QueryPlanFailed — that is the pre-fix defect.
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "MED-004 (pipe): error must NOT be QueryPlanFailed (-32000). \
         Post-fix: E-QUERY-039 fires before DataFusion execution. \
         Got: {err:?}"
    );
}

/// F-PQLFN-P1-MED-004 RED (2/3): filter-mode unknown scalar → E-QUERY-039.
///
/// Query: `crowdstrike_detections | notafunc_xyz(device_id) = 'active'`
///
/// # RED gate pre-fix failure (@7a56e53b)
/// `check_enrich_udf_availability` has `_ => {}` for Ast::Filter — skipped entirely.
/// `notafunc_xyz` reaches DataFusion → QueryPlanFailed.
/// Test asserts `EnrichUdfNotFound` → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// Ast::Filter arm added to walk filter predicates for ScalarFunc::Unknown names.
/// `notafunc_xyz` not in DataFusion built-in set → not in empty registry → E-QUERY-039.
///
/// Traces to: BC-2.11.019 v1.6 §Precondition 1 (filter-mode parity);
///            error-taxonomy.md §E-QUERY-039.
#[tokio::test]
async fn test_BC_2_11_019_med_004_filter_unknown_scalar_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    // RED GATE: Ast::Filter arm is `_ => {}` → gate skipped → QueryPlanFailed.
    // POST-FIX: Ast::Filter arm walks filter predicate FuncCalls → E-QUERY-039.
    let result = engine
        .execute(
            "crowdstrike_detections | notafunc_xyz(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-004 (filter): notafunc_xyz(device_id) = 'active' in filter mode must return \
         Err(E-QUERY-039). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "notafunc_xyz"),
        "MED-004 (filter): error must be PrismError::EnrichUdfNotFound \
         with infusion = 'notafunc_xyz'. \
         RED failure: Ast::Filter is currently skipped by the gate → QueryPlanFailed. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-039"),
        "MED-004 (filter): Display must contain 'E-QUERY-039'. Got: {display}"
    );
    assert!(
        display.contains("notafunc_xyz"),
        "MED-004 (filter): Display must contain 'notafunc_xyz'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "MED-004 (filter): error must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

/// F-PQLFN-P1-MED-004 RED (3/3): SqlPipe `| where` unknown scalar → E-QUERY-039.
///
/// Query: `SELECT device_id FROM crowdstrike_detections | where notafunc_xyz(device_id) = 'active'`
///
/// In SqlPipe (`Ast::SqlPipe`), `check_enrich_udf_availability` currently checks:
///   (a) PipeStage::Enrich stages — none present
///   (b) SQL head `collect_unknown_scalars_from_sql_query` — `notafunc_xyz` is in the
///       `| where` stage, NOT in the SQL head SELECT/WHERE/GROUP BY positions
///
/// So `notafunc_xyz` in the SqlPipe `| where` stage bypasses the gate → DataFusion → -32000.
///
/// # RED gate pre-fix failure (@7a56e53b)
/// Gate misses ScalarFunc::Unknown in SqlPipe PipeStage::Where predicates.
/// QueryPlanFailed emitted. Test asserts EnrichUdfNotFound → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// Ast::SqlPipe arm also walks PipeStage::Where predicates for ScalarFunc::Unknown.
/// `notafunc_xyz` found → not in registry → E-QUERY-039.
///
/// Traces to: BC-2.11.019 v1.6 §Precondition 1(b) (SQL-mode WHERE coverage);
///            error-taxonomy.md §E-QUERY-039.
#[tokio::test]
async fn test_BC_2_11_019_med_004_sqlpipe_where_unknown_scalar_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    // RED GATE: SqlPipe arm only checks PipeStage::Enrich + SQL head → misses | where.
    // POST-FIX: SqlPipe arm also walks PipeStage::Where predicates → E-QUERY-039.
    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections | where notafunc_xyz(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "MED-004 (sqlpipe): notafunc_xyz in SqlPipe | where must return \
         Err(E-QUERY-039). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "notafunc_xyz"),
        "MED-004 (sqlpipe): error must be PrismError::EnrichUdfNotFound \
         with infusion = 'notafunc_xyz'. \
         RED failure: SqlPipe arm misses PipeStage::Where predicates → QueryPlanFailed. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-039"),
        "MED-004 (sqlpipe): Display must contain 'E-QUERY-039'. Got: {display}"
    );
    assert!(
        display.contains("notafunc_xyz"),
        "MED-004 (sqlpipe): Display must contain 'notafunc_xyz'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "MED-004 (sqlpipe): error must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── F-PQLFN-P1-OBS-001: empty-arg edge cases ─────────────────────────────────

/// TM-15 (fix-burst 2 RED): empty-arg aggregate `count()` in pipe `| where` must
/// produce E-QUERY-001 with the ADR-048 D.3 canonical aggregate message.
///
/// Query: `FROM crowdstrike_detections | where count() = 5`
///
/// # Pre-fix-burst-2 mechanism (@5ce8bedc)
/// `count` is in the parser-level AGGREGATE_FUNC_NAMES blocklist (7-name list in
/// `build_predicate_parser`). `fn_call_comparison` fires a `try_map` guard for the
/// aggregate name. Chumsky backtracks to `field_comparison`, which parses `count`
/// as a bare field identifier then fails at `(` → `PrismError::QueryParseFailed`
/// with a "found '('" message — NOT the ADR-048 D.3 canonical message.
///
/// # Post-fix-burst-2 mechanism
/// AGGREGATE_FUNC_NAMES parser-level blocklist is REMOVED (ADR-048 v1.2 OD-4).
/// `fn_call_comparison` successfully parses `count()` as `FuncCall::Scalar(Unknown("count"))`.
/// The plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate intercepts it and fires
/// E-QUERY-001 with the canonical detail:
///   "'count' is an aggregate function; aggregate fn-calls are not valid in
///    WHERE/where predicates (use HAVING for post-aggregation filters, ADR-048 D.3)"
///
/// # RED at @5ce8bedc
/// The current "found '('" message does NOT contain "aggregate function" or "HAVING".
/// Assertions for those strings FAIL → this test is RED. ✓
///
/// Traces to: BC-2.11.004 v1.32 EC-11-004-006 scope note; ADR-048 v1.2 D.7 TM-15;
///            filter_parser.rs (AGGREGATE_FUNC_NAMES removal target);
///            engine.rs DATAFUSION_BUILTIN_AGGREGATE_NAMES gate.
#[tokio::test]
async fn test_BC_2_11_004_obs_001_pipe_where_empty_arg_count_blocked() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where count() = 5",
            QueryOptions::default(),
        )
        .await;

    // TM-15 assertion (1/4): must be E-QUERY-001 QueryParseFailed.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "TM-15: `count() = 5` in pipe | where must return \
         PrismError::QueryParseFailed (E-QUERY-001). \
         Aggregate fn-calls are not valid in WHERE/where predicates (ADR-048 D.3). \
         Got: {result:?}"
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // TM-15 assertion (2/4): SID-2 message contains fn name.
    assert!(
        display.contains("count"),
        "TM-15: Display must contain 'count' (the aggregate fn name). \
         RED: 'found '('' message from parser backtrack does not contain 'count'. \
         GREEN (after fix-burst-2): plan-time gate fires canonical D.3 message. \
         Got: {display}"
    );

    // TM-15 assertion (3/4): SID-2 message contains "aggregate function".
    assert!(
        display.contains("aggregate function"),
        "TM-15: Display must contain 'aggregate function' (ADR-048 D.3 canonical message). \
         RED: parser-backtrack 'found '('' message does not contain 'aggregate function'. \
         Got: {display}"
    );

    // TM-15 assertion (4/4): SID-2 message contains "HAVING" (canonical D.3 guidance).
    assert!(
        display.contains("HAVING"),
        "TM-15: Display must contain 'HAVING' (use HAVING for post-aggregation filters, ADR-048 D.3). \
         RED: 'found '('' backtrack message does not contain 'HAVING'. \
         Got: {display}"
    );
}

/// F-PQLFN-P1-OBS-001 RED (2/2): empty-arg unknown scalar in pipe `| where`
/// must yield E-QUERY-039, not QueryPlanFailed.
///
/// Query: `FROM crowdstrike_detections | where foo() = 'x'`
///
/// `foo` is NOT in AGGREGATE_FUNC_NAMES, so it parses as `ScalarFunc::Unknown("foo")`
/// with empty args (via `fn_call_comparison`).  After the MED-004 fix, the gate
/// walks PipeStage::Where predicates and finds `foo` → not in DataFusion built-in set
/// → not in empty registry → E-QUERY-039.
///
/// # RED gate pre-fix failure (@7a56e53b)
/// Gate only walks PipeStage::Enrich → `foo` in PipeStage::Where predicate is missed →
/// query proceeds to DataFusion → DataFusion fails (unknown function `foo`) → QueryPlanFailed.
/// Test asserts EnrichUdfNotFound → FAILS. ✓
///
/// # Post-fix state (GREEN)
/// MED-004 fix extends the walk to PipeStage::Where predicates.
/// `foo` found → not in registry → E-QUERY-039.
///
/// NOTE: This test is RED until the MED-004 implementer fix lands.
///
/// Traces to: BC-2.11.019 v1.6 §Precondition 1(b);
///            error-taxonomy.md §E-QUERY-039; F-PQLFN-P1-MED-004.
#[tokio::test]
async fn test_BC_2_11_019_obs_001_pipe_where_empty_arg_unknown_scalar_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    // RED GATE: gate currently misses PipeStage::Where predicates → QueryPlanFailed.
    // POST-FIX (MED-004): gate finds foo() → not in registry → E-QUERY-039.
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where foo() = 'x'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "OBS-001 RED: `foo() = 'x'` in pipe | where must return Err(E-QUERY-039). \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "foo"),
        "OBS-001 RED: error must be PrismError::EnrichUdfNotFound with infusion = 'foo'. \
         RED failure: gate currently skips PipeStage::Where → QueryPlanFailed. \
         Becomes GREEN after MED-004 fix. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-039"),
        "OBS-001 RED: Display must contain 'E-QUERY-039'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "OBS-001 RED: error must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── F-PQLFN-P1-OBS-002: DataFusion aggregate not in 7-name blocklist ─────────
//
// Originally RED against @7a56e53b; GREEN from fix-burst-1 (@5ce8bedc) onward.
//
// Gap (closed by fix-burst-1): AGGREGATE_FUNC_NAMES contained only 7 names:
//   [count, sum, avg, min, max, distinct_count, percentile]
//
// DataFusion has many more aggregate functions (stddev, variance, median, corr, etc.).
// When `stddev(risk_score)` appeared in pipe `| where`, `fn_call_comparison` parsed it
// as `ScalarFunc::Unknown("stddev")` (not in the 7-name blocklist). The query bypassed
// the aggregate guard, reached DataFusion → QueryPlanFailed / -32000 INTERNAL_ERROR.
//
// Fix-burst-1 added the plan-time DATAFUSION_BUILTIN_AGGREGATE_NAMES gate
// to `check_enrich_udf_availability`, which intercepts stddev → E-QUERY-001 canonical D.3
// message. This test is now a GREEN lock.
//
// TM-03/TM-09 (fix-burst-2 strengthening): adds "HAVING" assertion per SID-2
// composed-string discipline (ADR-048 v1.2 §D.7).

/// TM-03/TM-09 GREEN lock (strengthened from fix-burst-1): `stddev` in pipe `| where`
/// must produce E-QUERY-001 with the ADR-048 D.3 canonical aggregate message, including
/// "HAVING" guidance.
///
/// Query: `FROM crowdstrike_detections | where stddev(risk_score) = 5`
///
/// # GREEN at @5ce8bedc (fix-burst-1 closed OBS-002)
/// Plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate fires canonical detail:
///   "'stddev' is an aggregate function; aggregate fn-calls are not valid in
///    WHERE/where predicates (use HAVING for post-aggregation filters, ADR-048 D.3)"
/// The resulting Display contains "aggregate", "stddev", and "HAVING" → all assertions pass.
///
/// # Fix-burst-2 stability
/// After AGGREGATE_FUNC_NAMES parser-level blocklist removal (ADR-048 v1.2 OD-4):
/// - stddev was NEVER in the 7-name blocklist, so behavior is unchanged
/// - Plan-time gate continues to fire → test remains GREEN ✓
///
/// Traces to: filter_parser.rs (plan-time gate unchanged for stddev); ADR-048 v1.2 D.7 TM-03/09;
///            F-PQLFN-P1-OBS-002; BC-2.11.004 v1.32 §Postconditions.
#[tokio::test]
async fn test_BC_2_11_003_obs_002_pipe_where_stddev_not_in_blocklist_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where stddev(risk_score) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-03/09: stddev(risk_score) = 5 in pipe | where must return an error. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary assertion: must be a CONTROLLED plan-time error (E-QUERY-001 QueryParseFailed).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-03/09: stddev(risk_score) in pipe | where must return \
         PrismError::QueryParseFailed (E-QUERY-001). \
         Plan-time DATAFUSION_BUILTIN_AGGREGATE_NAMES gate must intercept stddev. \
         Got: {err:?} (Display: {display})"
    );

    // SID-2 composed-string assertion (1/3): message contains "aggregate".
    assert!(
        display.contains("aggregate"),
        "TM-03/09: Display must contain 'aggregate' (ADR-048 D.3 pattern). Got: {display}"
    );

    // SID-2 composed-string assertion (2/3): message contains fn name.
    assert!(
        display.contains("stddev"),
        "TM-03/09: Display must contain 'stddev' (specific aggregate fn name). Got: {display}"
    );

    // SID-2 composed-string assertion (3/3): message contains "HAVING" (D.3 guidance).
    assert!(
        display.contains("HAVING"),
        "TM-03/09: Display must contain 'HAVING' (use HAVING guidance, ADR-048 D.3). \
         Got: {display}"
    );

    // Must NOT be QueryPlanFailed — the pre-fix-burst-1 -32000 defect.
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-03/09: error must NOT be QueryPlanFailed (-32000 INTERNAL_ERROR). \
         Got: {err:?}"
    );
}

// ── F-PQLFN-P21-OBS-003: E-QUERY-001 aggregate gate must report truthful offset ────────────
//
// ADR-048 §D.7.2 Full Display form specifies `at offset {offset}` — the value must be
// truthful. Previously `check_enrich_udf_availability` hardcoded `offset: 0` for the
// aggregate-in-predicate gate, regardless of where the aggregate appears in the query.
//
// Fix-burst-16 threads the function-name span from `FuncCall::Scalar::span` (populated
// by filter_parser.rs `fn_call_comparison` via `map_with`) through the new
// `collect_unknown_scalar_offsets_from_predicate` into the E-QUERY-001 error.
//
// These tests assert that `offset` points at the first byte of the aggregate function
// name in the original query string.

/// F-PQLFN-P21-OBS-003: E-QUERY-001 aggregate gate reports truthful offset for a
/// `stddev` call appearing AFTER another valid fn-call in a pipe `| where` predicate.
///
/// Query: `FROM crowdstrike_detections | where lower(risk_score) = 'low' AND stddev(severity) > 10`
///
/// `stddev` appears mid-query; pre-fix the offset is always 0. Post-fix, offset must
/// equal `query.find("stddev")`.
///
/// # RED → GREEN
/// FAILS on pre-fix code: `offset == 0` (hardcoded), test panics on `assert_eq!(0, expected)`.
/// PASSES after fix: `offset == query.find("stddev").unwrap()` (actual source position).
///
/// Load-bearing (TD-VSDD-059): removing the span-threading in
/// `collect_unknown_scalar_offsets_from_predicate` causes this test to fail
/// (offset reverts to 0).
#[tokio::test]
async fn test_pqlfn_p21_obs003_aggregate_offset_nonzero_pipe_where() {
    let query =
        "FROM crowdstrike_detections | where lower(risk_score) = 'low' AND stddev(severity) > 10";
    let expected_offset = query.find("stddev").expect("stddev must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P21-OBS-003: E-QUERY-001 aggregate gate must report truthful \
                 offset pointing at 'stddev' in the original query string. \
                 Expected offset={expected_offset} (first byte of 'stddev'), got offset={offset}. \
                 Pre-fix: offset is always 0 (hardcoded). \
                 Fix: thread span from FuncCall::Scalar through \
                 collect_unknown_scalar_offsets_from_predicate."
            );
            assert!(
                offset > 0,
                "F-PQLFN-P21-OBS-003: offset must be > 0 for an aggregate appearing \
                 mid-query (not at byte 0). Got offset={offset}"
            );
        }
        other => panic!(
            "F-PQLFN-P21-OBS-003: expected QueryParseFailed (E-QUERY-001) for \
             stddev in pipe | where, got: {other:?}"
        ),
    }
}

/// F-PQLFN-P21-OBS-003: SQL WHERE path also reports truthful offset.
///
/// Query: `SELECT severity FROM crowdstrike_detections WHERE lower(host) = 'server' AND stddev(risk_score) > 5`
///
/// `stddev` is mid-query; offset must be non-zero and correct.
#[tokio::test]
async fn test_pqlfn_p21_obs003_aggregate_offset_nonzero_sql_where() {
    let query =
        "SELECT severity FROM crowdstrike_detections WHERE lower(host) = 'server' AND stddev(risk_score) > 5";
    let expected_offset = query.find("stddev").expect("stddev must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P21-OBS-003 SQL WHERE: E-QUERY-001 offset must point at 'stddev' \
                 (expected={expected_offset}, got={offset})"
            );
        }
        other => panic!("F-PQLFN-P21-OBS-003 SQL WHERE: expected QueryParseFailed, got: {other:?}"),
    }
}

// ── DEFECT-PQL-FNCALL-LHS-001 fix-burst 2: ADR-048 v1.2 §D.7 aggregate-gate matrix ─────────
//
// Implements ADR-048 v1.2 §D.7 "Unified Plan-Time Aggregate-in-Predicate Gate" TM-01..TM-18.
//
// TM-01:  existing (test_BC_2_11_004_invariant_pipe_where_aggregate_fncall_remains_invalid)
// TM-03/09: strengthened above (test_BC_2_11_003_obs_002_pipe_where_stddev_not_in_blocklist_e_query_001)
// TM-15: updated above (test_BC_2_11_004_obs_001_pipe_where_empty_arg_count_blocked)
//
// RED tests at @5ce8bedc (fail until fix-burst-2 implementation):
//   TM-06  SQL WHERE count        → E-QUERY-001 canonical (currently QueryPlanFailed -32000)
//   TM-07  SQL WHERE sum          → E-QUERY-001 canonical (currently QueryPlanFailed -32000)
//   TM-08  Pipe WHERE count(col)  → canonical D.3 message (currently backtrack "found '('")
//   TM-10  SqlPipe-head WHERE sum → E-QUERY-001 canonical (currently QueryPlanFailed -32000)
//   TM-14  SQL WHERE agg + date   → E-QUERY-001 not E-QUERY-042 (D.7.4 gate ordering)
//   TM-16  SQL WHERE stddev       → canonical D.3 message (currently QueryPlanFailed -32000)
//
// GREEN lock tests (pass at @5ce8bedc AND after fix-burst-2):
//   TM-02  Filter mode count      → is_err() (scope guard, both pre- and post-fix)
//   TM-04  Filter mode stddev     → canonical D.3 message (plan-time gate already covers)
//   TM-05  SqlPipe WHERE stddev   → canonical D.3 message (plan-time gate already covers)
//   TM-11  HAVING count(*) passes → NOT E-QUERY-001 (HAVING exempt, MED-001 permit)
//   TM-12  HAVING stddev passes   → NOT E-QUERY-001 (HAVING exempt, MED-001 permit)
//   TM-13  HAVING count(typo_col) → E-QUERY-038 column gate fires (not aggregate gate)
//
// fix-burst-4 additions (GREEN from arrival, F-PQLFN-P4-LOW-002):
//   TM-17  Pipe | where distinct_count → E-QUERY-001 D.3 (manual-insert lock for distinct_count)
//   TM-18  Pipe | where percentile    → E-QUERY-001 D.3 (manual-insert lock for percentile)
//
// fix-burst-4 additions (D.7.4 ordering discriminators, F-PQLFN-P4-LOW-001):
//   D.7.4-exec  pipe | where stddev + date → E-QUERY-001 not E-QUERY-042 (execute path)
//   D.7.4-sched pipe | where stddev + date → E-QUERY-001 not E-QUERY-042 (execute_scheduled path)
//
// fix-burst-4 additions (F-PQLFN-P4-MED-001 HAVING e2e lock):
//   HAVING percentile(risk_score, 95) > 5 → NOT E-QUERY-001 (HAVING exempt)
//
// LOW-001 (F-PQLFN-P2-LOW-001): BC-2.11.004 v1.32 canonical scope limits
//   nested fn-call args:  upper(trim(device_id)) = 'active'  → QueryParseFailed (GREEN lock)
//   IEQ-with-fn-call:     lower(device_id) IEQ 'active'      → QueryParseFailed (GREEN lock)
//
// LOW-002 (F-PQLFN-P2-LOW-002): nested-predicate walk coverage
//   AND:  device_id = 'x' AND notafunc_xyz(risk_score) = 5   → E-QUERY-039 (GREEN lock)
//   NOT:  NOT (notafunc_xyz(risk_score) = 5)                  → E-QUERY-039 (GREEN lock)
//
// Engine fixtures:
//   make_crowdstrike_detections_engine()          — no infusion registry
//   make_crowdstrike_engine_with_empty_infusion() — empty InfusionRegistry (E-QUERY-039 tests)

// ── TM-02: Filter mode count → error (scope guard) ───────────────────────────

/// TM-02 GREEN lock: aggregate fn-call in filter mode must produce an error.
///
/// Query: `crowdstrike_detections | count(device_id) = 5`
///
/// Scope guard analogous to TM-01 (pipe WHERE invariant). ADR-048 D.7 covers all seven
/// predicate positions; filter mode is one of them. This test pins that aggregate fn-calls
/// are rejected in filter mode in both pre- and post-fix-burst-2 states.
///
/// # State in both pre- and post-fix-burst-2
/// RED (parser backtracks → QueryParseFailed) and GREEN (plan-time gate →
/// QueryParseFailed) both produce Err. The assertion is variant-only (is_err()).
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-02; BC-2.11.004 v1.32 §Postconditions.
#[tokio::test]
async fn test_BC_2_11_004_tm_02_filter_mode_count_aggregate_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | count(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-02: `count(device_id) = 5` in filter mode must return Err. \
         Aggregate fn-calls are not valid in filter mode predicates (ADR-048 D.7.1). \
         Got Ok."
    );
}

// ── TM-04: Filter mode stddev → canonical D.3 message (GREEN lock) ───────────

/// TM-04 GREEN lock: `stddev` in filter mode must produce E-QUERY-001 with canonical
/// ADR-048 D.3 message. Plan-time gate covers filter mode from fix-burst-1 onward.
///
/// Query: `crowdstrike_detections | stddev(risk_score) = 5`
///
/// At @5ce8bedc: stddev NOT in AGGREGATE_FUNC_NAMES → fn_call_comparison succeeds →
/// Ast::Filter predicate → plan-time gate (DATAFUSION_BUILTIN_AGGREGATE_NAMES) fires →
/// canonical D.3 message. GREEN lock. ✓
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-04 (filter mode position); BC-2.11.004 v1.32.
#[tokio::test]
async fn test_BC_2_11_004_tm_04_filter_mode_stddev_canonical_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | stddev(risk_score) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-04: stddev(risk_score) = 5 in filter mode must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-04: must be QueryParseFailed (E-QUERY-001). Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-04: Display must contain 'aggregate function' (ADR-048 D.3). Got: {display}"
    );

    assert!(
        display.contains("stddev"),
        "TM-04: Display must contain 'stddev'. Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-04: Display must contain 'HAVING' (D.3 guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-04: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-05: SqlPipe WHERE stddev → canonical D.3 message (GREEN lock) ─────────

/// TM-05 GREEN lock: `stddev` in SqlPipe `| where` must produce E-QUERY-001 with
/// canonical ADR-048 D.3 message. Plan-time gate covers SqlPipe WHERE from fix-burst-1.
///
/// Query: `SELECT * FROM crowdstrike_detections | where stddev(risk_score) = 5`
///
/// Uses `SELECT *` so that `risk_score` is accessible in the `| where` stage projection
/// (a `SELECT device_id FROM ...` query would hide `risk_score` → E-QUERY-038 fires instead).
///
/// At @5ce8bedc: stddev NOT in AGGREGATE_FUNC_NAMES → fn_call_comparison succeeds →
/// SqlPipe stage PipeStage::Where predicate → plan-time gate fires → canonical message.
/// GREEN lock. ✓
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-05 (SqlPipe WHERE position); BC-2.11.019 v1.7.
#[tokio::test]
async fn test_BC_2_11_019_tm_05_sqlpipe_where_stddev_canonical_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections | where stddev(risk_score) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-05: stddev(risk_score) = 5 in SqlPipe | where must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-05: must be QueryParseFailed (E-QUERY-001). Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-05: Display must contain 'aggregate function' (ADR-048 D.3). Got: {display}"
    );

    assert!(
        display.contains("stddev"),
        "TM-05: Display must contain 'stddev'. Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-05: Display must contain 'HAVING' (D.3 guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-05: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-06: SQL WHERE count → E-QUERY-001 canonical message (RED) ─────────────

/// TM-06 RED: `count` in SQL WHERE must fire E-QUERY-001 with canonical ADR-048 D.3
/// message. Locks the HIGH-001 SQL WHERE regression fix.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE count(device_id) > 5`
///
/// # RED at @5ce8bedc
/// SQL WHERE is NOT in predicate_fncall_names (engine only walks Pipe/Filter/SqlPipe
/// stage WHERE predicates — not the SQL head WHERE). `count` goes to sql_unknown_names
/// via `collect_unknown_scalars_from_sql_query`, but is filtered by
/// DATAFUSION_BUILTIN_FUNCTION_NAMES (count is a built-in aggregate). Query reaches
/// DataFusion → DataFusion rejects aggregate in WHERE context → QueryPlanFailed (-32000).
/// Test asserts QueryParseFailed → FAILS. ✓
///
/// # GREEN after fix-burst-2
/// SQL WHERE added to predicate_fncall_names scope (ADR-048 v1.2 D.6).
/// DATAFUSION_BUILTIN_AGGREGATE_NAMES gate fires → canonical D.3 message.
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-06; F-PQLFN-P2-HIGH-001; BC-2.11.019 v1.7.
#[tokio::test]
async fn test_BC_2_11_019_tm_06_sql_where_count_e_query_001_high001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE count(device_id) > 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-06: count(device_id) > 5 in SQL WHERE must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // RED assertion: must be QueryParseFailed (E-QUERY-001), NOT QueryPlanFailed (-32000).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-06 RED: SQL WHERE count must return QueryParseFailed (E-QUERY-001). \
         RED: SQL WHERE not yet in aggregate gate → QueryPlanFailed (-32000). \
         GREEN (fix-burst-2): SQL WHERE added to gate scope → canonical D.3 message. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-06: Display must contain 'aggregate function' (canonical D.3). Got: {display}"
    );

    assert!(
        display.contains("count"),
        "TM-06: Display must contain 'count' (fn name). Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-06: Display must contain 'HAVING' (D.3 guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-06: must NOT be QueryPlanFailed (-32000). RED: currently QueryPlanFailed. Got: {err:?}"
    );
}

// ── TM-07: SQL WHERE sum → E-QUERY-001 canonical message (RED) ───────────────

/// TM-07 RED: `sum` in SQL WHERE must fire E-QUERY-001. Locks HIGH-001 for a second
/// aggregate name to avoid single-name regression.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE sum(risk_score) > 10`
///
/// # RED at @5ce8bedc
/// Same path as TM-06: sum in DATAFUSION_BUILTIN_FUNCTION_NAMES → filtered from
/// sql_unknown_names → reaches DataFusion → QueryPlanFailed (-32000).
/// Test asserts QueryParseFailed → FAILS. ✓
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-07; F-PQLFN-P2-HIGH-001; BC-2.11.019 v1.7.
#[tokio::test]
async fn test_BC_2_11_019_tm_07_sql_where_sum_e_query_001_high001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE sum(risk_score) > 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-07: sum(risk_score) > 10 in SQL WHERE must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-07 RED: SQL WHERE sum must return QueryParseFailed (E-QUERY-001). \
         RED: SQL WHERE not in aggregate gate → QueryPlanFailed. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-07: Display must contain 'aggregate function'. Got: {display}"
    );

    assert!(
        display.contains("sum"),
        "TM-07: Display must contain 'sum' (fn name). Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-07: Display must contain 'HAVING'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-07: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-08: Pipe WHERE count(col) with args → canonical D.3 message (RED) ─────

/// TM-08 RED: `count(device_id)` (with arg) in pipe `| where` must produce E-QUERY-001
/// with the ADR-048 D.3 canonical aggregate message. Locks the MED-002 parser-backtrack
/// message fix.
///
/// Query: `FROM crowdstrike_detections | where count(device_id) = 5`
///
/// # RED at @5ce8bedc
/// `count` IS in AGGREGATE_FUNC_NAMES (7-name parser blocklist). `fn_call_comparison`
/// fires a `try_map` guard → Chumsky backtracks to `field_comparison` → `count` as field
/// path + `(device_id)` fails at `(` → QueryParseFailed with "found '('" message.
/// "found '('" does NOT contain "aggregate function" or "HAVING".
/// Test asserts those strings → FAILS. ✓
///
/// # GREEN after fix-burst-2
/// AGGREGATE_FUNC_NAMES parser-level blocklist removed (ADR-048 v1.2 OD-4).
/// `fn_call_comparison` parses count(device_id) as FuncCall::Scalar(Unknown("count")).
/// Plan-time DATAFUSION_BUILTIN_AGGREGATE_NAMES gate fires canonical D.3 message.
///
/// Note: TM-01 covers the same query with variant-only assertion (is_err()).
/// TM-08 adds the stronger SID-2 message assertions for the D.7 MED-002 closure.
///
/// Traces to: ADR-048 v1.2 §D.7.2 TM-08; F-PQLFN-P2-MED-002; BC-2.11.004 EC-11-082 (renumbered from EC-11-013 in v1.47; SR-006 collision with BC-2.11.005);
/// F-PQLFN-P3-OBS-002 (byte-verbatim POL-24 upgrade).
#[tokio::test]
async fn test_BC_2_11_004_tm_08_pipe_where_count_with_args_canonical_d3_message() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where count(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-08: count(device_id) = 5 in pipe | where must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-08: must be QueryParseFailed (E-QUERY-001). Got: {err:?} (Display: {display})"
    );

    // F-PQLFN-P10-OBS-001 / POL-24 byte-verbatim lock: assert the detail-only canonical
    // template from BC-2.11.004 EC-11-082 (renumbered from EC-11-013 in v1.47; SR-006 collision with BC-2.11.005) appears as an exact contiguous substring
    // of Display (ADR-048 D.7.2 de-prefix discipline: detail MUST NOT embed "E-QUERY-001:"
    // prefix — that prefix is emitted once by QueryParseFailed's #[error] format string).
    // One byte-verbatim lock here; other TM tests retain substring checks (defense-in-depth
    // diversity: one byte-verbatim lock + N substring locks per ADR-048 D.7).
    const CANONICAL_AGG_MSG: &str = "'count' is an aggregate function; \
        aggregate fn-calls are not valid in WHERE/where predicates \
        (use HAVING for post-aggregation filters, ADR-048 D.3)";
    assert!(
        display.contains(CANONICAL_AGG_MSG),
        "TM-08 F-PQLFN-P10-OBS-001: Display must contain the byte-verbatim detail-only \
         canonical template from BC-2.11.004 EC-11-082 (renumbered from EC-11-013 in v1.47; SR-006 collision with BC-2.11.005) (POL-24). \
         Expected contiguous substring: {CANONICAL_AGG_MSG:?}. \
         Got: {display}"
    );

    // F-PQLFN-P10-OBS-001 single-prefix regression lock: Display must contain EXACTLY
    // ONE "E-QUERY-001:" occurrence (ADR-048 D.7.2 single-prefix discipline).
    // Two occurrences would indicate the detail embeds the prefix again (double-prefix bug).
    assert!(
        display.matches("E-QUERY-001:").count() == 1,
        "TM-08 F-PQLFN-P10-OBS-001: Display must contain EXACTLY ONE 'E-QUERY-001:' \
         prefix (ADR-048 D.7.2 de-prefix discipline). Double-prefix indicates the \
         aggregate-gate detail embeds the code again. \
         Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-08: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-10: SqlPipe-head WHERE aggregate → E-QUERY-001 canonical message (RED) ─

/// TM-10 RED: aggregate in SqlPipe-head SQL WHERE must fire E-QUERY-001.
/// Locks the HIGH-001 fix for the SqlPipe-head WHERE position.
///
/// Query: `SELECT device_id FROM crowdstrike_detections WHERE sum(risk_score) = 10 | limit 5`
///
/// This is `Ast::SqlPipe` with head WHERE `sum(risk_score) = 10` and pipe stage `| limit 5`.
///
/// # RED at @5ce8bedc
/// For `Ast::SqlPipe`, the aggregate gate only walks `spq.stages` PipeStage::Where
/// predicates (NOT `spq.head.where_`). `sum` from the head WHERE goes to
/// `sql_unknown_names` via `collect_unknown_scalars_from_sql_query` but is filtered by
/// DATAFUSION_BUILTIN_FUNCTION_NAMES. Query reaches DataFusion → QueryPlanFailed (-32000).
/// Test asserts QueryParseFailed → FAILS. ✓
///
/// # GREEN after fix-burst-2
/// `spq.head.where_` added to predicate_fncall_names scope (ADR-048 v1.2 D.7.1 position 5).
/// Gate fires → canonical D.3 message.
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-10; F-PQLFN-P2-HIGH-001; BC-2.11.019 v1.7.
#[tokio::test]
async fn test_BC_2_11_019_tm_10_sqlpipe_head_where_aggregate_e_query_001_high001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections WHERE sum(risk_score) = 10 | limit 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-10: sum(risk_score) = 10 in SqlPipe-head WHERE must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-10 RED: SqlPipe-head WHERE sum must return QueryParseFailed (E-QUERY-001). \
         RED: spq.head.where_ not in aggregate gate → QueryPlanFailed (-32000). \
         GREEN (fix-burst-2): head WHERE added to gate scope → canonical D.3 message. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-10: Display must contain 'aggregate function'. Got: {display}"
    );

    assert!(
        display.contains("sum"),
        "TM-10: Display must contain 'sum'. Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-10: Display must contain 'HAVING'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-10: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-11/12/13: HAVING exemption GREEN locks (pin MED-001 permit ruling) ────

/// TM-11 GREEN lock: `HAVING count(*) > 5` must NOT fire E-QUERY-001.
/// HAVING is fully exempt from the aggregate-in-predicate gate (ADR-048 D.7.3).
///
/// Query: `SELECT device_id, count(*) FROM crowdstrike_detections GROUP BY device_id HAVING count(*) > 5`
///
/// count is in the HAVING six-name aggregate list → parsed as `FuncCall::Aggregate`
/// (via `build_agg_call_parser`). HAVING predicates are NOT walked by predicate_fncall_names.
/// No E-QUERY-001 fires. Result: execution error (no sensor backend) or Ok.
///
/// GREEN in both pre- and post-fix-burst-2 states — pins the MED-001 HAVING permit ruling.
///
/// Traces to: ADR-048 v1.2 §D.7.3 TM-11; F-PQLFN-P2-MED-001; BC-2.11.016 v1.6.
#[tokio::test]
async fn test_BC_2_11_016_tm_11_having_count_star_not_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id, count(*) FROM crowdstrike_detections \
             GROUP BY device_id HAVING count(*) > 5",
            QueryOptions::default(),
        )
        .await;

    // TM-11: HAVING count(*) must NOT fire aggregate-gate E-QUERY-001.
    // HAVING is exempt from the aggregate-in-predicate gate (ADR-048 D.7.3).
    // The result may be Ok or any non-E-QUERY-001 error (e.g., sensor not found at exec time).
    if let Err(ref e) = result {
        assert!(
            !matches!(e, PrismError::QueryParseFailed { .. }),
            "TM-11: HAVING count(*) > 5 must NOT fire E-QUERY-001 (aggregate gate). \
             HAVING is exempt per ADR-048 D.7.3 (MED-001 permit). Got: {e:?}"
        );

        // Additional guard: the error must not mention "aggregate function" in E-QUERY-001 context.
        let display = format!("{e}");
        assert!(
            !display.contains("aggregate function"),
            "TM-11: HAVING count(*) must not produce 'aggregate function' E-QUERY-001 message. \
             Got: {display}"
        );
    }
}

/// TM-12 GREEN lock: `HAVING stddev(risk_score) > 1.0` must NOT fire E-QUERY-001.
/// Specifically pins the MED-001 permit ruling for non-six-name aggregates in HAVING.
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id HAVING stddev(risk_score) > 1.0`
///
/// `stddev` is NOT in the HAVING six-name aggregate list → falls through to `base` predicate
/// parser → parses as `FuncCall::Scalar(Unknown("stddev"))` in HAVING position.
/// HAVING predicates are NOT walked by predicate_fncall_names → aggregate gate does NOT fire.
/// Result: execution error (no sensor backend) or Ok. Must NOT be E-QUERY-001.
///
/// ADR-048 v1.2 D.7.3: "stddev/variance/corr/median/etc. in HAVING parse as
/// `FuncCall::Scalar(Unknown)` — permitted."
///
/// GREEN in both pre- and post-fix-burst-2 states.
///
/// Traces to: ADR-048 v1.2 §D.7.3 TM-12; F-PQLFN-P2-MED-001; BC-2.11.016 v1.6.
#[tokio::test]
async fn test_BC_2_11_016_tm_12_having_stddev_non_six_name_not_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING stddev(risk_score) > 1.0",
            QueryOptions::default(),
        )
        .await;

    // TM-12: HAVING stddev must NOT fire aggregate-gate E-QUERY-001.
    // Non-six-name aggregates in HAVING are permitted via FuncCall::Scalar fallthrough
    // (ADR-048 v1.2 D.7.3 OD-3 MED-001 permit ruling).
    if let Err(ref e) = result {
        assert!(
            !matches!(e, PrismError::QueryParseFailed { .. }),
            "TM-12: HAVING stddev(risk_score) must NOT fire E-QUERY-001 (aggregate gate). \
             Non-six-name aggregates in HAVING are permitted (ADR-048 D.7.3 MED-001). \
             Got: {e:?}"
        );

        let display = format!("{e}");
        assert!(
            !display.contains("aggregate function"),
            "TM-12: HAVING stddev must not produce 'aggregate function' E-QUERY-001 message. \
             Got: {display}"
        );
    }
}

/// TM-13 GREEN lock: `HAVING count(typo_col_zzz) > 5` must fire E-QUERY-038 (column gate),
/// NOT E-QUERY-001 (aggregate gate). Pins the BC-2.11.016 HAVING column-gate behavior.
///
/// Query: `SELECT device_id, count(*) FROM crowdstrike_detections GROUP BY device_id HAVING count(typo_col_zzz) > 5`
///
/// `count(typo_col_zzz)` parses as `FuncCall::Aggregate(CountField(typo_col_zzz))`.
/// The HAVING column gate (`collect_predicate_columns` FuncCall arm) walks aggregate args
/// and extracts `typo_col_zzz`. `typo_col_zzz` is NOT in `crowdstrike_detections` schema
/// (device_id / timestamp / risk_score) → E-QUERY-038 ColumnNotFound.
///
/// The aggregate-in-predicate gate does NOT fire (HAVING is exempt per D.7.3).
/// E-QUERY-038 fires first at the plan-time column-existence check.
///
/// GREEN in both pre- and post-fix-burst-2 states (column gate independent of aggregate gate).
///
/// Traces to: ADR-048 v1.2 §D.7.3 TM-13; BC-2.11.016 v1.6 (HAVING column gate);
///            engine.rs `test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038`.
#[tokio::test]
async fn test_BC_2_11_016_tm_13_having_count_typo_col_fires_e_query_038_not_e_query_001() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id, count(*) FROM crowdstrike_detections \
             GROUP BY device_id HAVING count(typo_col_zzz) > 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-13: HAVING count(typo_col_zzz) must return Err (E-QUERY-038). Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary: must be ColumnNotFound (E-QUERY-038), NOT QueryParseFailed (E-QUERY-001).
    assert!(
        matches!(&err, PrismError::ColumnNotFound(ref d) if d.column == "typo_col_zzz"),
        "TM-13: HAVING count(typo_col_zzz) must fire E-QUERY-038 ColumnNotFound \
         (column gate, not aggregate gate). \
         Column gate must walk aggregate fn args in HAVING (BC-2.11.016). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-038"),
        "TM-13: Display must contain 'E-QUERY-038'. Got: {display}"
    );

    assert!(
        display.contains("typo_col_zzz"),
        "TM-13: Display must contain 'typo_col_zzz'. Got: {display}"
    );

    // Must NOT be E-QUERY-001 — the aggregate gate must NOT fire for HAVING.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-13: must NOT be QueryParseFailed (E-QUERY-001). \
         HAVING is exempt from aggregate gate (ADR-048 D.7.3). \
         Got: {err:?}"
    );
}

// ── TM-14: Gate ordering D.7.4 — SQL WHERE aggregate + date-like → E-QUERY-001 ─

/// TM-14 RED: aggregate fn-call in SQL WHERE with date-like RHS must fire E-QUERY-001
/// (aggregate gate), NOT E-QUERY-042 (temporal gate). ADR-048 v1.2 D.7.4 gate ordering.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE stddev(risk_score) = '2026-06-24'`
///
/// D.7.4: `check_enrich_udf_availability` (including aggregate gate) runs BEFORE
/// `check_temporal_literals` (ADR-052 §D4 arms). If both would fire:
///   - aggregate gate fires E-QUERY-001 first
///   - temporal gate never reached
///
/// # RED at @5ce8bedc
/// SQL WHERE not in predicate_fncall_names → aggregate gate doesn't fire →
/// query proceeds toward DataFusion → QueryPlanFailed (-32000).
/// Test asserts QueryParseFailed (not QueryPlanFailed) → FAILS. ✓
///
/// # GREEN after fix-burst-2
/// SQL WHERE added to predicate_fncall_names → aggregate gate fires E-QUERY-001 →
/// check_temporal_literals never reached → result is E-QUERY-001 (not E-QUERY-042). ✓
///
/// Traces to: ADR-048 v1.2 §D.7.4 TM-14; F-PQLFN-P2-HIGH-001; ADR-052 §D4.
#[tokio::test]
async fn test_BC_2_11_019_tm_14_sql_where_agg_date_like_e_query_001_not_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE stddev(risk_score) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-14: WHERE stddev(risk_score) = '2026-06-24' must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // TM-14 primary: must be E-QUERY-001 (aggregate gate fires first per D.7.4).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-14 RED: SQL WHERE stddev + date-like RHS must fire E-QUERY-001 (aggregate gate). \
         RED: SQL WHERE not in gate scope → QueryPlanFailed. \
         GREEN (fix-burst-2): aggregate gate fires before temporal gate (D.7.4). \
         Got: {err:?} (Display: {display})"
    );

    // Must NOT be E-QUERY-042 — the aggregate gate must fire BEFORE the temporal gate.
    assert!(
        !matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "TM-14: must NOT fire E-QUERY-042 NonColumnLhsComparison. \
         ADR-048 D.7.4: aggregate gate (E-QUERY-001) fires BEFORE temporal gate (E-QUERY-042). \
         Got: {err:?}"
    );

    // Must NOT be QueryPlanFailed (-32000).
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-14: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-16: SQL WHERE stddev → canonical D.3 message (RED) ────────────────────

/// TM-16 RED: `stddev` in SQL WHERE must produce E-QUERY-001 with canonical ADR-048 D.3
/// message. Locks the HIGH-001 fix for DataFusion-built-in-aggregate (non-seven-name) names.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE stddev(risk_score) = 5`
///
/// # RED at @5ce8bedc
/// `stddev` NOT in AGGREGATE_FUNC_NAMES (parser level). SQL WHERE not in
/// predicate_fncall_names. Goes to sql_unknown_names but filtered by
/// DATAFUSION_BUILTIN_FUNCTION_NAMES → reaches DataFusion → QueryPlanFailed (-32000).
/// Test asserts QueryParseFailed with canonical message → FAILS. ✓
///
/// # GREEN after fix-burst-2
/// SQL WHERE added to predicate_fncall_names. stddev IS in DATAFUSION_BUILTIN_AGGREGATE_NAMES
/// → gate fires canonical D.3 message.
///
/// Traces to: ADR-048 v1.2 §D.7.1 TM-16; F-PQLFN-P2-HIGH-001; BC-2.11.019 v1.7.
#[tokio::test]
async fn test_BC_2_11_019_tm_16_sql_where_stddev_canonical_d3_message() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE stddev(risk_score) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-16: WHERE stddev(risk_score) = 5 must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-16 RED: SQL WHERE stddev must return QueryParseFailed (E-QUERY-001). \
         RED: SQL WHERE not in aggregate gate → QueryPlanFailed (-32000). \
         GREEN (fix-burst-2): gate fires canonical D.3 message. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-16: Display must contain 'aggregate function' (canonical D.3). Got: {display}"
    );

    assert!(
        display.contains("stddev"),
        "TM-16: Display must contain 'stddev'. Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-16: Display must contain 'HAVING' (D.3 guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-16: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-17: Pipe | where distinct_count → E-QUERY-001 (manual-insert lock) ───────

/// TM-17 GREEN lock: `distinct_count` in pipe `| where` must fire E-QUERY-001 with
/// canonical ADR-048 D.3 message containing "distinct_count".
///
/// Query: `FROM crowdstrike_detections | where distinct_count(device_id) = 5`
///
/// "distinct_count" is NOT in DataFusion 53.1's `default_aggregate_functions()` registry
/// (EMPIRICALLY VERIFIED — F-PQLFN-P4-MED-001; DataFusion 53.1 uses "approx_distinct").
/// The manual `names.insert("distinct_count")` in `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
/// is what makes the plan-time aggregate gate fire for this name.
///
/// This test is a load-bearing lock for the manual insert: if the insert were removed,
/// "distinct_count" would fall through (no coverage), and E-QUERY-001 would NOT fire.
///
/// Traces to: F-PQLFN-P4-LOW-002; ADR-048 v1.2 D.7.1 D.7.6; BC-2.11.004 v1.33.
#[tokio::test]
async fn test_BC_2_11_004_tm_17_pipe_where_distinct_count_manual_insert_lock() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where distinct_count(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-17: `distinct_count(device_id) = 5` in pipe | where must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // TM-17 primary: must be E-QUERY-001 (aggregate gate fires, not E-QUERY-039).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-17: pipe | where distinct_count must return QueryParseFailed (E-QUERY-001). \
         The manual DATAFUSION_BUILTIN_AGGREGATE_NAMES insert for 'distinct_count' is the \
         mechanism — DataFusion 53.1 does NOT include 'distinct_count' in its registry \
         (uses 'approx_distinct' instead; EMPIRICALLY VERIFIED, F-PQLFN-P4-MED-001). \
         Without the manual insert, this test fails (no E-QUERY-001). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("distinct_count"),
        "TM-17: Display must contain 'distinct_count' (aggregate fn name, ADR-048 D.3). \
         Got: {display}"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-17: Display must contain 'aggregate function' (canonical D.3 message). \
         Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-17: Display must contain 'HAVING' (use HAVING guidance, ADR-048 D.3). \
         Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-17: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── TM-18: Pipe | where percentile → E-QUERY-001 (manual-insert lock) ────────

/// TM-18 GREEN lock: `percentile` in pipe `| where` must fire E-QUERY-001 with
/// canonical ADR-048 D.3 message containing "percentile".
///
/// Query: `FROM crowdstrike_detections | where percentile(risk_score, 95) = 5`
///
/// "percentile" is NOT in DataFusion 53.1's `default_aggregate_functions()` registry
/// (EMPIRICALLY VERIFIED — F-PQLFN-P4-MED-001; DataFusion 53.1 uses "approx_percentile_cont").
/// The manual `names.insert("percentile")` in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` is what
/// makes the plan-time aggregate gate fire for this name.
///
/// ADR-048 v1.3 claimed "percentile IS registered in default_aggregate_functions()" —
/// this is EMPIRICALLY FALSE (DataFusion 53.1). The manual insert is NECESSARY.
/// This test is a load-bearing lock for that insert.
///
/// Traces to: F-PQLFN-P4-LOW-002; ADR-048 v1.2 D.7.1 D.7.6; BC-2.11.004 v1.33.
#[tokio::test]
async fn test_BC_2_11_004_tm_18_pipe_where_percentile_manual_insert_lock() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where percentile(risk_score, 95) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "TM-18: `percentile(risk_score, 95) = 5` in pipe | where must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // TM-18 primary: must be E-QUERY-001 (aggregate gate fires, not E-QUERY-039).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "TM-18: pipe | where percentile must return QueryParseFailed (E-QUERY-001). \
         The manual DATAFUSION_BUILTIN_AGGREGATE_NAMES insert for 'percentile' is the \
         mechanism — DataFusion 53.1 does NOT include 'percentile' in its registry \
         (EMPIRICALLY VERIFIED, F-PQLFN-P4-MED-001; ADR-048 v1.3's contrary claim is FALSE). \
         Without the manual insert, this test fails (no E-QUERY-001). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("percentile"),
        "TM-18: Display must contain 'percentile' (aggregate fn name, ADR-048 D.3). \
         Got: {display}"
    );

    assert!(
        display.contains("aggregate function"),
        "TM-18: Display must contain 'aggregate function' (canonical D.3 message). \
         Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "TM-18: Display must contain 'HAVING' (use HAVING guidance, ADR-048 D.3). \
         Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "TM-18: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── EC-11-086: HAVING `percentile` → E-QUERY-001 (registry-independent) ─────────────────────
//
// ADR-048 v1.16 §D.2 amendment (F-PQLFN-PR3-LOW-001, BC-2.11.004 v1.48):
//
// `percentile` is excluded from `build_agg_call_parser` (ADR-048 §D.2 OD-2: two-arg grammar
// ambiguity). It therefore parses as `FuncCall::Scalar(Unknown("percentile"))` via
// `fn_call_comparison` in `build_sql_predicate_parser` (base branch of
// `build_having_predicate_parser`).
//
// Pre-v1.16 behavior (DO NOT REGRESS):
//   - HAVING is EXEMPT from the aggregate-gate `predicate_fncall_names` walk (ADR-048 §D.7.1);
//     "percentile" does NOT reach the E-QUERY-001 aggregate gate.
//   - "percentile" IS walked into `sql_unknown_names` via position (f) of
//     `collect_unknown_scalars_from_sql_query`.
//   - "percentile" is NOT in `DATAFUSION_BUILTIN_FUNCTION_NAMES` (not a DataFusion registry
//     built-in — proven by empirical tests at commit 524a9986 / bb23f143).
//   - With registry active (Some): "percentile" passes the DATAFUSION_BUILTIN_FUNCTION_NAMES
//     filter → not in registry → E-QUERY-039 fires ("enrichment infusion 'percentile' is not
//     registered") — a FALSE enrichment-registration suggestion misleading to LLM agents.
//   - With no registry (None): check_enrich_udf_availability returns Ok(()) at the
//     `let Some(registry) = registry else { return Ok(()) }` guard → no error → DataFusion
//     execution fails with QueryPlanFailed ("percentile" unknown to DataFusion).
//
// Post-v1.16 behavior (what the RED tests below assert):
//   `check_enrich_udf_availability` intercepts `name` ∈ `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
//   in HAVING position (f) BEFORE the infusion-registry lookup and fires E-QUERY-001 with
//   HAVING-specific guidance. This interception must run BEFORE the `let Some(registry) = ...`
//   guard so it fires even when no registry is configured (registry-INDEPENDENT).
//
// Tests (both RED until the implementation fix ships):
//   - test_BC_2_11_004_having_percentile_fires_e_query_001_no_registry  (variant b: no registry)
//   - test_BC_2_11_004_having_percentile_fires_e_query_001_with_registry (variant a: registry active)
//
// Stale-test note (ADR-048 v1.16 §D.2): `test_BC_2_11_016_tm_having_percentile_not_e_query_001_
// having_exempt` was anchored to the pre-v1.16 behavior ("result is NOT QueryParseFailed").
// That assertion is now WRONG — post-v1.16 the result IS QueryParseFailed (E-QUERY-001).
// This section replaces it with the two required replacement tests from ADR-048 §D.2.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// EC-11-086 (b) **RED** — registry-independence lock: `HAVING percentile(x, p) op value`
/// fires E-QUERY-001 with HAVING-specific guidance even when NO infusion registry is wired.
///
/// This is the direct replacement for the now-stale
/// `test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt`, which asserted the
/// pre-v1.16 behavior ("HAVING percentile must NOT fire E-QUERY-001"). Post-v1.16, E-QUERY-001
/// MUST fire.
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id
///         HAVING percentile(risk_score, 95) > 5`
///
/// Engine: `make_crowdstrike_detections_engine()` — no infusion registry (registry = None).
///
/// **Pre-fix failure path** (current code → RED):
/// - `percentile` in HAVING → `sql_unknown_names` via position (f) of
///   `collect_unknown_scalars_from_sql_query`
/// - HAVING is EXEMPT from the `predicate_fncall_names` aggregate gate (ADR-048 §D.7.1)
/// - `let Some(registry) = registry else { return Ok(()) }` guard fires → Ok(())
/// - Query proceeds to DataFusion execution → DataFusion cannot resolve `percentile` →
///   `QueryPlanFailed` (NOT `QueryParseFailed`)
/// - This test asserts `QueryParseFailed` → FAILS (RED). ✓
///
/// **Post-fix path** (after the v1.16 implementation):
/// - New `DATAFUSION_BUILTIN_AGGREGATE_NAMES` interception for HAVING position fires BEFORE
///   the `let Some(registry) = registry` guard → E-QUERY-001 regardless of registry state.
/// - `QueryParseFailed` with HAVING-specific message → this test PASSES (GREEN).
///
/// Canonical message (byte-verbatim per POL-24, ADR-048 §D.2):
/// `"E-QUERY-001: query parse error at offset {offset}: 'percentile' is a PrismQL aggregate
/// function; PERCENTILE is not directly supported in HAVING predicates — alias it in SELECT:
/// SELECT PERCENTILE(field, p) AS alias ... HAVING alias > threshold (ADR-048 D.3 OD-2)"`
///
/// Traces to: BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004;
///            F-PQLFN-PR3-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_having_percentile_fires_e_query_001_no_registry() {
    let engine = make_crowdstrike_detections_engine(); // no infusion registry (None)

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING percentile(risk_score, 95) > 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: must be E-QUERY-001 (QueryParseFailed), NOT QueryPlanFailed.
    // Pre-fix: registry=None → Ok(()) → DataFusion plan fails → QueryPlanFailed. RED.
    // Post-fix: HAVING DATAFUSION_BUILTIN_AGGREGATE_NAMES interception fires → QueryParseFailed.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-086 (b, no-registry): HAVING percentile(risk_score, 95) > 5 must fire \
         E-QUERY-001 (QueryParseFailed) with HAVING-specific guidance. \
         Pre-fix: registry=None → check_enrich_udf_availability returns Ok(()) → \
         DataFusion plan fails with QueryPlanFailed — NOT E-QUERY-001. \
         Post-fix: new DATAFUSION_BUILTIN_AGGREGATE_NAMES interception in HAVING position \
         fires BEFORE the registry-None guard → E-QUERY-001 (registry-INDEPENDENT). \
         (BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004) \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — the plan-time interception must precede DataFusion.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-086 (b, no-registry): must NOT be QueryPlanFailed. \
         Pre-fix: no-registry path lets the query through to DataFusion (plan fails). \
         Post-fix: E-QUERY-001 fires at plan time before DataFusion. Got: {result:?}"
    );

    // POL-24 message-text lock — HAVING-specific canonical message from ADR-048 §D.2.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains(
            "is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates"
        ),
        "EC-11-086 (b, no-registry): E-QUERY-001 display must contain HAVING-specific \
         guidance: \"is a PrismQL aggregate function; PERCENTILE is not directly supported \
         in HAVING predicates\" (ADR-048 §D.2 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("alias it in SELECT"),
        "EC-11-086 (b, no-registry): E-QUERY-001 display must contain alias guidance \
         \"alias it in SELECT\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("ADR-048 D.3 OD-2"),
        "EC-11-086 (b, no-registry): E-QUERY-001 display must contain ADR citation \
         \"ADR-048 D.3 OD-2\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'percentile'"),
        "EC-11-086 (b, no-registry): E-QUERY-001 display must contain quoted name \
         \"'percentile'\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
}

/// EC-11-086 (a) **RED** — load-bearing registry-active variant: `HAVING percentile(x, p)`
/// fires E-QUERY-001 (NOT E-QUERY-039) when an infusion registry is active.
///
/// This is the primary load-bearing RED test for F-PQLFN-PR3-LOW-001. The pre-fix code fires
/// E-QUERY-039 ("enrichment infusion 'percentile' is not registered") on registry-active
/// installations — a false enrichment-registration suggestion that misleads LLM agents into
/// thinking they need to register a 'percentile' enrichment function.
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id
///         HAVING percentile(risk_score, 95) > 5`
///
/// Engine: `make_crowdstrike_engine_with_empty_infusion()` — empty InfusionRegistry (Some, 0 entries).
///
/// **Pre-fix failure path** (current code → RED):
/// - `percentile` in HAVING → `sql_unknown_names` via position (f)
/// - NOT in `DATAFUSION_BUILTIN_FUNCTION_NAMES` (not a DataFusion registry built-in — F-PQLFN-P4-MED-001)
/// - Registry is Some([]) → "percentile" not in empty registry → E-QUERY-039 fires:
///   "enrichment infusion 'percentile' is not registered; available: []"
/// - This test asserts `QueryParseFailed` (E-QUERY-001) → FAILS (RED). ✓
///
/// **Post-fix path** (after the v1.16 implementation):
/// - New interception for `name` ∈ `DATAFUSION_BUILTIN_AGGREGATE_NAMES` in HAVING position fires
///   BEFORE the infusion-registry lookup → E-QUERY-001 with HAVING-specific message.
/// - E-QUERY-039 does NOT fire.
///
/// Canonical message (byte-verbatim per POL-24, ADR-048 §D.2):
/// `"E-QUERY-001: query parse error at offset {offset}: 'percentile' is a PrismQL aggregate
/// function; PERCENTILE is not directly supported in HAVING predicates — alias it in SELECT:
/// SELECT PERCENTILE(field, p) AS alias ... HAVING alias > threshold (ADR-048 D.3 OD-2)"`
///
/// Traces to: BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004;
///            F-PQLFN-PR3-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_having_percentile_fires_e_query_001_with_registry() {
    let engine = make_crowdstrike_engine_with_empty_infusion(); // registry active (Some, empty)

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING percentile(risk_score, 95) > 5",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed), NOT E-QUERY-039 (EnrichUdfNotFound).
    // Pre-fix: "percentile" not in DATAFUSION_BUILTIN_FUNCTION_NAMES, not in registry
    //          → EnrichUdfNotFound (E-QUERY-039). This assertion FAILS (RED). ✓
    // Post-fix: HAVING DATAFUSION_BUILTIN_AGGREGATE_NAMES interception fires first
    //           → QueryParseFailed (E-QUERY-001). Assertion PASSES.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-086 (a, registry-active): HAVING percentile(risk_score, 95) > 5 must fire \
         E-QUERY-001 (QueryParseFailed) with HAVING-specific guidance. \
         Pre-fix: registry-active installation fires E-QUERY-039 \
         ('enrichment infusion 'percentile' is not registered; available: []') — \
         a FALSE enrichment-registration suggestion misleading to LLM agents. \
         Post-fix: DATAFUSION_BUILTIN_AGGREGATE_NAMES interception in HAVING position fires \
         E-QUERY-001 BEFORE the registry lookup. \
         (BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004) \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-039 (the pre-fix regression).
    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-086 (a, registry-active): must NOT fire E-QUERY-039 (EnrichUdfNotFound). \
         'percentile' is a PrismQL aggregate keyword — E-QUERY-039 \
         ('enrichment infusion not registered') is a false suggestion. \
         The new HAVING-position interception must prevent E-QUERY-039 from firing. \
         Got: {result:?}"
    );

    // POL-24 message-text lock — HAVING-specific canonical message from ADR-048 §D.2.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains(
            "is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates"
        ),
        "EC-11-086 (a, registry-active): E-QUERY-001 display must contain HAVING-specific \
         guidance (ADR-048 §D.2 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("alias it in SELECT"),
        "EC-11-086 (a, registry-active): E-QUERY-001 display must contain alias guidance \
         \"alias it in SELECT\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("ADR-048 D.3 OD-2"),
        "EC-11-086 (a, registry-active): E-QUERY-001 display must contain ADR citation \
         \"ADR-048 D.3 OD-2\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'percentile'"),
        "EC-11-086 (a, registry-active): E-QUERY-001 display must contain quoted name \
         \"'percentile'\" (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    // Additional negative lock: display must NOT contain E-QUERY-039 message fragment.
    assert!(
        !err_display.contains("enrichment infusion"),
        "EC-11-086 (a, registry-active): E-QUERY-001 display must NOT contain E-QUERY-039 \
         message fragment 'enrichment infusion' — the pre-fix regression output was \
         \"E-QUERY-039: enrichment infusion 'percentile' is not registered; available: []\". \
         Got: {err_display:?}"
    );
}

/// EC-11-087 **GREEN lock** — `HAVING distinct_count(field)` succeeds (parses as
/// `FuncCall::Aggregate`, never reaches E-QUERY-039 gate).
///
/// `distinct_count` IS in `build_agg_call_parser`'s six-name list (ADR-048 §D.2 BNF:
/// `'DISTINCT_COUNT'` case-insensitive). `HAVING distinct_count(device_id) > 100` parses
/// as `Predicate::Compare { lhs: Expr::FuncCall(FuncCall::Aggregate(Distinct("device_id"))), ..}`
/// — NOT as `ScalarFunc::Unknown`. This AST node never reaches the `sql_unknown_names` walker
/// (which only collects `ScalarFunc::Unknown` nodes), so neither E-QUERY-039 nor the new
/// DATAFUSION_BUILTIN_AGGREGATE_NAMES HAVING interception fires.
///
/// This is the sibling contrast with EC-11-086 (percentile), establishing that the fix targets
/// only `ScalarFunc::Unknown` names in HAVING (which are NOT in `build_agg_call_parser`), not
/// valid aggregate AST nodes (which ARE in `build_agg_call_parser`).
///
/// **GREEN on arrival**: the `FuncCall::Aggregate` parse path for `distinct_count` has been
/// in place since S-DEMO-FIDELITY-REMEDIATION-001. If this test FAILS (E-QUERY-001 or
/// E-QUERY-039), the fix over-intercepts in HAVING position — that is a REAL DEFECT.
///
/// Note: the query may produce an execution error (no sensor adapter in test engine) — that
/// is acceptable. The assertions confirm only that NO parse-level or plan-level gate fires for
/// the `distinct_count` name in HAVING.
///
/// Traces to: BC-2.11.004 v1.48 EC-11-087; ADR-048 §D.2; F-PQLFN-PR3-LOW-001 sibling analysis.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_087_having_distinct_count_success() {
    let engine = make_crowdstrike_engine_with_empty_infusion(); // registry active — most stringent fixture

    let result = engine
        .execute(
            "SELECT device_id, COUNT(*) FROM crowdstrike_detections \
             GROUP BY device_id HAVING distinct_count(device_id) > 100",
            QueryOptions::default(),
        )
        .await;

    // Must NOT fire E-QUERY-001 (QueryParseFailed) — distinct_count parses as
    // FuncCall::Aggregate(Distinct), which is the correct HAVING aggregate AST form.
    // If the new HAVING interception accidentally fires for distinct_count, this fails.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-087 GREEN lock: HAVING distinct_count(device_id) > 100 must NOT fire \
         E-QUERY-001. `distinct_count` is in `build_agg_call_parser`'s six-name list; \
         it parses as FuncCall::Aggregate(Distinct), NOT ScalarFunc::Unknown. \
         The HAVING DATAFUSION_BUILTIN_AGGREGATE_NAMES interception must NOT fire for \
         proper aggregate AST nodes (BC-2.11.004 v1.48 EC-11-087, ADR-048 §D.2). \
         Got: {result:?}"
    );

    // Must NOT fire E-QUERY-039 (EnrichUdfNotFound) — distinct_count never reaches
    // the sql_unknown_names walker because it parses as FuncCall::Aggregate.
    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-087 GREEN lock: HAVING distinct_count must NOT fire E-QUERY-039. \
         `distinct_count` parses as FuncCall::Aggregate — the sql_unknown_names walker \
         only collects ScalarFunc::Unknown nodes; distinct_count never reaches the \
         E-QUERY-039 gate (BC-2.11.004 v1.48 EC-11-087). Got: {result:?}"
    );
    // (Result may be Ok or a non-parse error from the test engine — both acceptable.)
}

// ── F-PQLFN-P4-LOW-001: D.7.4 gate-ordering discriminator (pipe | where + temporal RHS) ──

/// F-PQLFN-P4-LOW-001 D.7.4 ordering discriminator (execute path):
/// `FROM crowdstrike_detections | where stddev(risk_score) = '2026-06-24'` must return
/// E-QUERY-001 (aggregate gate fires FIRST), NOT E-QUERY-042 (temporal gate).
///
/// This is a TRUE DISCRIMINATING test for D.7.4 gate ordering: the query has BOTH an
/// aggregate fn-call (`stddev`) AND a date-like RHS (`'2026-06-24'`). Two gates could
/// fire if both ran:
///   - Aggregate gate (`check_enrich_udf_availability` early call) → E-QUERY-001
///   - Temporal gate (`check_temporal_literals`) → E-QUERY-042 NonColumnLhsComparison
///
/// ADR-048 v1.2 §D.7.4: aggregate gate fires BEFORE temporal gate. The early
/// `check_enrich_udf_availability(query_str, None)` call in `execute_inner` is what
/// enforces this ordering. If that call were REMOVED, the temporal gate would fire
/// first and this test would FAIL with E-QUERY-042 — making it a true discriminator
/// that cannot be satisfied by any "later" gate call.
///
/// TM-14 covers the SQL WHERE position with the same combination; this test covers
/// the PIPE `| where` position, which is parsed via `fn_call_comparison` production
/// (different grammar path than SQL WHERE).
///
/// Traces to: F-PQLFN-P4-LOW-001; ADR-048 v1.2 §D.7.4; BC-2.11.004 v1.33.
#[tokio::test]
async fn test_BC_2_11_004_low_001_d74_pipe_where_agg_temporal_e_query_001_not_e_query_042_execute()
{
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where stddev(risk_score) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-P4-LOW-001 [execute]: stddev + date-like RHS in pipe | where must return Err. \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // PRIMARY: must be E-QUERY-001 (aggregate gate fires first per D.7.4).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-P4-LOW-001 [execute]: pipe | where stddev + date-like RHS must return \
         QueryParseFailed (E-QUERY-001). \
         D.7.4: early check_enrich_udf_availability fires BEFORE check_temporal_literals. \
         If E-QUERY-042: the early aggregate gate call is missing or bypassed. \
         Got: {err:?} (Display: {display})"
    );

    // Must NOT be E-QUERY-042 (temporal gate must NOT fire before aggregate gate).
    assert!(
        !matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "F-PQLFN-P4-LOW-001 [execute]: must NOT fire E-QUERY-042 NonColumnLhsComparison. \
         D.7.4: aggregate gate (E-QUERY-001) fires BEFORE temporal gate (E-QUERY-042). \
         Got: {err:?}"
    );

    // Must NOT be QueryPlanFailed (-32000).
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "F-PQLFN-P4-LOW-001 [execute]: must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );

    assert!(
        display.contains("stddev"),
        "F-PQLFN-P4-LOW-001 [execute]: Display must contain 'stddev'. Got: {display}"
    );
    assert!(
        display.contains("aggregate function"),
        "F-PQLFN-P4-LOW-001 [execute]: Display must contain 'aggregate function'. \
         Got: {display}"
    );
}

/// F-PQLFN-P4-LOW-001 D.7.4 ordering discriminator (execute_scheduled path):
/// `FROM crowdstrike_detections | where stddev(risk_score) = '2026-06-24'` must return
/// E-QUERY-001 via `execute_scheduled()`, NOT E-QUERY-042.
///
/// execute_scheduled_inner() has its own early `check_enrich_udf_availability(query_str, None)`
/// call that must fire BEFORE `check_temporal_literals`. If that call were removed from
/// execute_scheduled_inner, the temporal gate would fire first (E-QUERY-042) — making
/// this a true discriminator that cannot be satisfied by the later gate call.
///
/// Both execute() and execute_scheduled() must honor D.7.4 gate ordering symmetrically.
///
/// Traces to: F-PQLFN-P4-LOW-001; ADR-048 v1.2 §D.7.4; BC-2.11.004 v1.33.
#[tokio::test]
async fn test_BC_2_11_004_low_001_d74_pipe_where_agg_temporal_e_query_001_not_e_query_042_scheduled(
) {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let query = "FROM crowdstrike_detections | where stddev(risk_score) = '2026-06-24'";

    let scheduled_result = engine
        .execute_scheduled(query, None)
        .await
        .map(|(qr, _ctx)| qr);

    assert!(
        scheduled_result.is_err(),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: stddev + date-like RHS must return Err. \
         Got Ok."
    );

    let err = scheduled_result.unwrap_err();
    let display = format!("{err}");

    // PRIMARY: must be E-QUERY-001 (aggregate gate fires first in execute_scheduled_inner).
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: pipe | where stddev + date-like RHS must \
         return QueryParseFailed (E-QUERY-001). \
         execute_scheduled_inner has its own early check_enrich_udf_availability call \
         (D.7.4 symmetric ordering). If E-QUERY-042: that early call is missing. \
         Got: {err:?} (Display: {display})"
    );

    // Must NOT be E-QUERY-042.
    assert!(
        !matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: must NOT fire E-QUERY-042. \
         D.7.4: aggregate gate fires BEFORE temporal gate in execute_scheduled_inner. \
         Got: {err:?}"
    );

    // Must NOT be QueryPlanFailed (-32000).
    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: must NOT be QueryPlanFailed. Got: {err:?}"
    );

    assert!(
        display.contains("stddev"),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: Display must contain 'stddev'. Got: {display}"
    );
    assert!(
        display.contains("aggregate function"),
        "F-PQLFN-P4-LOW-001 [execute_scheduled]: Display must contain 'aggregate function'. \
         Got: {display}"
    );
}

// ── F-PQLFN-P2-LOW-001: BC-2.11.004 v1.32 canonical scope limits ─────────────
//
// LOW-001 tests pin two BC-2.11.004 v1.32 scope limits that are parse-time rejects
// at current HEAD (@5ce8bedc) — GREEN in both pre- and post-fix-burst-2 states.
//
// Limit 1 (nested fn-call args): `fn_call_arg` admits only `literal | field_path`.
//   Nested fn-calls (e.g., `upper(trim(device_id))`) fail to parse → QueryParseFailed.
//
// Limit 2 (IEQ/IIN/INE operators): fn_call_comparison only supports standard comparison
//   operators (=, !=, <, >, <=, >=). IEQ/IIN/INE are case-insensitive extensions that
//   are NOT wired into fn_call_comparison → `lower(device_id) IEQ 'active'` fails.

/// LOW-001 (1/2) GREEN lock: nested fn-call in fn_call_arg must be rejected.
///
/// Query: `FROM crowdstrike_detections | where upper(trim(device_id)) = 'active'`
///
/// `fn_call_arg` only accepts `literal | field_path`. When `trim(device_id)` appears as
/// an arg to `upper(...)`, it is not a field_path or literal → parser fails to match
/// the outer fn_call → either falls back to field_comparison (fails) or parse error.
/// Result: QueryParseFailed (E-QUERY-001) at parse time.
///
/// BC-2.11.004 v1.32 LOW-001 scope limit: "fn_call_arg admits `literal | field_path` only;
/// nested fn-calls not admitted → E-QUERY-001".
///
/// GREEN in both pre- and post-fix-burst-2 states (grammar limit is unchanged by fix-burst-2).
///
/// Traces to: BC-2.11.004 v1.32 LOW-001; ADR-048 v1.2 F-PQLFN-P2-LOW-001.
#[tokio::test]
async fn test_BC_2_11_004_low_001_nested_fncall_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where upper(trim(device_id)) = 'active'",
            QueryOptions::default(),
        )
        .await;

    // LOW-001: nested fn-call in arg position must fail to parse.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-001 (nested arg): upper(trim(device_id)) = 'active' must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         fn_call_arg only admits literal | field_path — nested fn-calls not admitted \
         (BC-2.11.004 v1.32 LOW-001). Got: {result:?}"
    );
}

/// LOW-001 (2/2) GREEN lock: IEQ operator does not compose with fn-call LHS.
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) IEQ 'active'`
///
/// `fn_call_comparison` handles only standard comparison operators (=, !=, <, >, <=, >=).
/// `IEQ` (case-insensitive equality) is NOT in the fn_call_comparison operator set →
/// parser fails to complete the fn_call_comparison production → falls back to
/// field_comparison → `lower` as field path + `(device_id)` fails at `(` →
/// QueryParseFailed (E-QUERY-001).
///
/// BC-2.11.004 v1.32 LOW-002: "IEQ/IIN/INE operators do NOT compose with fn-call LHS → E-QUERY-001".
///
/// GREEN in both pre- and post-fix-burst-2 states (IEQ wiring not in scope).
///
/// Traces to: BC-2.11.004 v1.32 LOW-002; ADR-048 v1.2 F-PQLFN-P2-LOW-001.
#[tokio::test]
async fn test_BC_2_11_004_low_001_ieq_operator_with_fncall_lhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) IEQ 'active'",
            QueryOptions::default(),
        )
        .await;

    // LOW-001 (IEQ): fn_call LHS + IEQ operator is not valid syntax → QueryParseFailed.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-001 (IEQ): lower(device_id) IEQ 'active' must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         IEQ does not compose with fn-call LHS (BC-2.11.004 v1.32 LOW-002). \
         Got: {result:?}"
    );
}

// ── F-PQLFN-P2-LOW-002: nested-predicate walk coverage ───────────────────────
//
// LOW-002 tests pin that `collect_unknown_scalar_from_predicate` recurses into
// nested predicates (Predicate::Logical AND/OR and Predicate::Not). These tests
// verify the MED-004 fix (pipe/filter/sqlpipe WHERE walk) also handles logical nesting.
//
// Both tests use `make_crowdstrike_engine_with_empty_infusion()` — E-QUERY-039 only
// fires when the InfusionRegistry is Some (non-None). `notafunc_xyz` is not a DataFusion
// built-in → not filtered → reaches E-QUERY-039 gate → EnrichUdfNotFound fires.
//
// GREEN in both pre- and post-fix-burst-2 states (predicate walk logic unchanged).

/// LOW-002 (1/2) GREEN lock: AND-nested fn-call in pipe WHERE predicate fires E-QUERY-039.
///
/// Query: `FROM crowdstrike_detections | where device_id = 'x' AND notafunc_xyz(risk_score) = 5`
///
/// The predicate is `Predicate::Logical(And, [Compare(device_id = 'x'), Compare(FuncCall("notafunc_xyz") = 5)])`.
/// `collect_unknown_scalar_from_predicate` recurses into `Predicate::Logical` →
/// finds `notafunc_xyz` → E-QUERY-039 fires.
///
/// Traces to: BC-2.11.019 v1.7; ADR-048 v1.2 F-PQLFN-P2-LOW-002;
///            engine.rs `test_high003_collect_unknown_scalar_in_not_predicate`.
#[tokio::test]
async fn test_BC_2_11_019_low_002_and_nested_predicate_notafunc_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where device_id = 'x' AND notafunc_xyz(risk_score) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "LOW-002 (AND): AND-nested notafunc_xyz must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "notafunc_xyz"),
        "LOW-002 (AND): AND-nested notafunc_xyz must fire E-QUERY-039 EnrichUdfNotFound. \
         collect_unknown_scalar_from_predicate must recurse into Predicate::Logical. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-039"),
        "LOW-002 (AND): Display must contain 'E-QUERY-039'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "LOW-002 (AND): must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

/// LOW-002 (2/2) GREEN lock: NOT-wrapped fn-call in pipe WHERE predicate fires E-QUERY-039.
///
/// Query: `FROM crowdstrike_detections | where NOT (notafunc_xyz(risk_score) = 5)`
///
/// The predicate is `Predicate::Not(Compare(FuncCall("notafunc_xyz") = 5))`.
/// `collect_unknown_scalar_from_predicate` recurses into `Predicate::Not` →
/// finds `notafunc_xyz` → E-QUERY-039 fires.
///
/// Companion to `test_high003_collect_unknown_scalar_in_not_predicate` in engine.rs
/// (unit test), this provides e2e coverage through `engine.execute()`.
///
/// Traces to: BC-2.11.019 v1.7; ADR-048 v1.2 F-PQLFN-P2-LOW-002;
///            engine.rs `test_high003_collect_unknown_scalar_in_not_predicate`.
#[tokio::test]
async fn test_BC_2_11_019_low_002_not_wrapped_predicate_notafunc_e_query_039() {
    let engine = make_crowdstrike_engine_with_empty_infusion();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where NOT (notafunc_xyz(risk_score) = 5)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "LOW-002 (NOT): NOT-wrapped notafunc_xyz must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::EnrichUdfNotFound(ref d) if d.infusion == "notafunc_xyz"),
        "LOW-002 (NOT): NOT-wrapped notafunc_xyz must fire E-QUERY-039 EnrichUdfNotFound. \
         collect_unknown_scalar_from_predicate must recurse into Predicate::Not. \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("E-QUERY-039"),
        "LOW-002 (NOT): Display must contain 'E-QUERY-039'. Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "LOW-002 (NOT): must NOT be QueryPlanFailed (-32000). Got: {err:?}"
    );
}

// ── F-PQLFN-P9-MED-002: DML WHERE arm-4 regression locks ─────────────────────
//
// ADR-052 v1.14 dispatch-table row 4 claims E-QUERY-042 arm-4
// (NonColumnLhsComparison) is reachable from DML WHERE:
//   `DELETE FROM t WHERE lower(col) = '2026-06-24'` hits this arm.
//
// Mechanism: `check_temporal_literals`'s `Ast::Sql(SqlStatement::Dml(dml))` arm
// (materialization.rs `check_pred_raw_temporal` call on `dml.filter`) fires
// `NonColumnLhsComparison` when the predicate LHS is a non-Field expr
// (FuncCall::Scalar) and the RHS is a RawTemporalLiteral.  The DML parser uses
// `build_predicate_parser()` (sql_parser.rs parse_sql_dml_with_limits), which
// includes the `fn_call_comparison` production added in DEFECT-PQL-FNCALL-LHS-001,
// so `lower(col)` parses as FuncCall::Scalar LHS and the temporal gate fires.
//
// These tests are GREEN on arrival — the mechanism already exists.
// They are load-bearing regression locks for the ADR-052 reachability spec claim.
//
// Engine entry point: `engine.execute(query, QueryOptions::default()).await` —
// identical to the SQL WHERE and filter-mode sibling tests (MED-002/MED-003).
//
// Gate ordering: early `check_temporal_literals` (skip_projection=true) fires
// BEFORE `check_table_availability`.  E-QUERY-042 propagates via `?` immediately;
// the engine never reaches the DML execution path (which returns Ok(vec![])).

/// F-PQLFN-P9-MED-002 (1/3) (GREEN lock): DML WHERE DELETE with fn-call LHS and
/// date-like RHS must return E-QUERY-042 NonColumnLhsComparison.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE lower(device_id) = '2026-06-24'`
///
/// # Path through the engine
/// 1. `PrismQlParser::parse` → `parse_dml_internal` → Ok(Ast::Sql(SqlStatement::Dml))
///    (`build_predicate_parser`'s `fn_call_comparison` production admits fn-call LHS).
/// 2. Early `check_temporal_literals` (skip_projection=true) — DML arm:
///    `dml.filter` → `check_pred_raw_temporal` →
///    non-Field LHS + RawTemporalLiteral("2026-06-24") →
///    `Err(TemporalLiteralInvalidPosition { NonColumnLhsComparison, .. })`.
/// 3. `?` propagates the error immediately; `check_table_availability` never fires.
///
/// # SID-2 composed-output discipline
/// Asserts both the error variant AND the canonical Display message prefix (POL-24).
///
/// Traces to: ADR-052 v1.14 dispatch-table row 4; BC-2.11.003 EC-11-003-007 (DML parity);
///            error-taxonomy.md §E-QUERY-042 v2.14; F-PQLFN-P9-MED-002.
#[tokio::test]
async fn test_dml_where_fncall_lhs_date_like_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE lower(device_id) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-P9-MED-002 (DELETE): lower(device_id) = '2026-06-24' in DML WHERE \
         must return Err(E-QUERY-042). \
         ADR-052 v1.14 dispatch-table row 4: DML WHERE arm-4 must be reachable. \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: E-QUERY-042 NonColumnLhsComparison.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "F-PQLFN-P9-MED-002 (DELETE): DML WHERE fn-call LHS with date-like RHS must produce \
         PrismError::TemporalLiteralInvalidPosition(NonColumnLhsComparison). \
         `check_temporal_literals` DML arm walks dml.filter via check_pred_raw_temporal; \
         non-Field LHS triggers arm (4) per ADR-052 v1.14 dispatch-table row 4. \
         Got: {err:?} (Display: {display})"
    );

    // value_prefix must be the first ≤50 chars of the offending literal.
    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "F-PQLFN-P9-MED-002 (DELETE): value_prefix must start with '2026-06-24'. \
             Got: {value_prefix:?}"
        );
    }

    // SID-2: assert canonical Display message prefix (POL-24 byte-verbatim).
    let expected_prefix =
        "E-QUERY-042: A date-like literal compared against a computed expression \
                           cannot be type-checked at plan time.";
    assert!(
        display.contains(expected_prefix),
        "F-PQLFN-P9-MED-002 (DELETE/SID-2/POL-24): Display must contain the canonical \
         E-QUERY-042 NonColumnLhsComparison message prefix. \
         error-taxonomy.md §E-QUERY-042 v2.14. \
         Got: {display}"
    );

    // Must NOT be QueryParseFailed — the DML parser now admits fn-call LHS.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-P9-MED-002 (DELETE): error must NOT be QueryParseFailed. \
         `build_predicate_parser` fn_call_comparison production parses lower(device_id); \
         E-QUERY-042 fires at plan time (-32602). Got: {err:?}"
    );
}

/// F-PQLFN-P9-MED-002 (2/3) (GREEN lock): DML WHERE UPDATE with fn-call LHS and
/// date-like RHS must return E-QUERY-042 NonColumnLhsComparison.
///
/// Query: `UPDATE crowdstrike_detections SET risk_score = 1 WHERE lower(device_id) = '2026-06-24'`
///
/// Sibling of the DELETE variant above — exercises the UPDATE DML operation path
/// through `check_temporal_literals`'s DML arm.  `dml.filter` is processed first
/// (line 3711 in materialization.rs); E-QUERY-042 fires via `?` before the SET
/// assignments are inspected.
///
/// The SET assignment `risk_score = 1` is a plain integer literal (not a
/// RawTemporalLiteral), so it does NOT contribute to the temporal error — the
/// error source is the WHERE predicate exclusively.
///
/// Traces to: ADR-052 v1.14 dispatch-table row 4; BC-2.11.003 EC-11-003-007 (DML parity);
///            error-taxonomy.md §E-QUERY-042 v2.14; F-PQLFN-P9-MED-002.
#[tokio::test]
async fn test_dml_where_update_fncall_lhs_date_like_e_query_042() {
    use prism_core::error::TemporalLiteralPosition;

    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "UPDATE crowdstrike_detections SET risk_score = 1 WHERE lower(device_id) = '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-P9-MED-002 (UPDATE): lower(device_id) = '2026-06-24' in DML WHERE \
         must return Err(E-QUERY-042). \
         ADR-052 v1.14 dispatch-table row 4: DML WHERE arm-4 must be reachable. \
         Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    // Primary variant assertion: E-QUERY-042 NonColumnLhsComparison.
    assert!(
        matches!(
            &err,
            PrismError::TemporalLiteralInvalidPosition {
                position: TemporalLiteralPosition::NonColumnLhsComparison,
                ..
            }
        ),
        "F-PQLFN-P9-MED-002 (UPDATE): DML WHERE fn-call LHS with date-like RHS must produce \
         PrismError::TemporalLiteralInvalidPosition(NonColumnLhsComparison). \
         dml.filter processed before SET assignments; arm (4) fires per ADR-052 v1.14. \
         Got: {err:?} (Display: {display})"
    );

    // value_prefix must be the first ≤50 chars of the offending literal.
    if let PrismError::TemporalLiteralInvalidPosition { value_prefix, .. } = &err {
        assert!(
            value_prefix.starts_with("2026-06-24"),
            "F-PQLFN-P9-MED-002 (UPDATE): value_prefix must start with '2026-06-24'. \
             Got: {value_prefix:?}"
        );
    }

    // Display must contain "E-QUERY-042" (matches sibling filter-mode assertion depth).
    assert!(
        display.contains("E-QUERY-042"),
        "F-PQLFN-P9-MED-002 (UPDATE): Display must contain 'E-QUERY-042'. Got: {display}"
    );

    // Must NOT be QueryParseFailed.
    assert!(
        !matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-P9-MED-002 (UPDATE): error must NOT be QueryParseFailed. \
         fn_call_comparison production parses lower(device_id) in UPDATE WHERE. \
         Got: {err:?}"
    );
}

/// F-PQLFN-P9-MED-002 (3/3) (GREEN lock): DML WHERE with fn-call LHS and
/// NON-date-like RHS must NOT produce E-QUERY-042.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE lower(device_id) = 'active'`
///
/// `'active'` is not in the `is_date_like` acceptance set; no `RawTemporalLiteral`
/// is emitted by the parser.  In `check_pred_raw_temporal`, `rhs` is
/// `Literal::String("active")` → `raw_val` is None → temporal gate does not fire
/// for the Compare predicate.  The query passes the plan gates and reaches the
/// DML execution path (returns Ok(vec![]) pending S-3.06 wiring).
///
/// Mirrors the filter-mode negative control
/// (`test_BC_2_11_003_ec11_003_007_filter_fncall_lhs_non_date_rhs_not_rejected`)
/// for DML WHERE parity.
///
/// Traces to: ADR-052 v1.14 dispatch-table row 4 (negative / passthrough case);
///            BC-2.11.003 EC-11-003-007 (non-date-like passthrough); F-PQLFN-P9-MED-002.
#[tokio::test]
async fn test_dml_where_fncall_lhs_non_date_rhs_not_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE lower(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    // BC-2.11.003 EC-11-003-007 / F-PQLFN-P9-MED-002 spec-promised outcome (DML mode):
    // DML `Ast::Sql(Dml)` falls to the `_ => Ok(Vec::new())` arm in
    // `execute_against_session_with_registry` — the emitter is never reached.
    // 'active' is not date-like → temporal gate passes. DML returns Ok(empty).
    // GREEN on current HEAD (DML no-op path bypasses the emitter defect).
    assert!(
        result.is_ok(),
        "F-PQLFN-P9-MED-002 (DELETE neg): `lower(device_id) = 'active'` in DML WHERE \
         must return Ok (DML no-op path — emitter is not reached for Ast::Sql(Dml)). \
         Got: {result:?}"
    );
}

// ── F-PQLFN-P10-OBS-002: fn-name identifier-start constraint ─────────────────

/// F-PQLFN-P10-OBS-002 (1/2): digit-leading fn-name must produce E-QUERY-001.
///
/// Query: `FROM crowdstrike_detections | where 123abc(device_id) = 5`
///
/// After the identifier-start constraint fix (filter_parser.rs fn_call_comparison):
/// - `fn_call_comparison` requires the first char to satisfy `is_ascii_alphabetic() || == '_'`.
/// - `1` (first char of `123abc`) is a digit, so `fn_call_comparison` fails to match.
/// - Chumsky backtracks to `field_comparison`, which parses `123abc` as a field path but
///   then fails at `(device_id)` (not a valid compare operator). All predicate alternatives fail.
/// - Result: `PrismError::QueryParseFailed` (E-QUERY-001).
///
/// Traces to: ADR-048 D.7.2 (fn-name identifier-start constraint); F-PQLFN-P10-OBS-002.
#[tokio::test]
async fn test_fncall_digit_leading_name_parse_error_obs_002() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where 123abc(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-P10-OBS-002: digit-leading fn-name '123abc(...)' must produce Err. Got Ok."
    );

    let err = result.unwrap_err();

    // Must be QueryParseFailed — digit-leading name is rejected at parse time.
    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-P10-OBS-002: '123abc(device_id) = 5' must return QueryParseFailed \
         (E-QUERY-001 parse error). fn_call_comparison identifier-start constraint \
         rejects digit-leading names at the grammar level. \
         Got: {err:?}"
    );
}

/// F-PQLFN-P10-OBS-002 (2/2): underscore-leading fn-name still parses as fn-call.
///
/// Query: `FROM crowdstrike_detections | where _abc(device_id) = 5`
///
/// After the identifier-start constraint fix:
/// - `_abc` starts with `_` which satisfies `is_ascii_alphabetic() || == '_'`.
/// - `fn_call_comparison` matches → `FuncCall::Scalar(Unknown("_abc"))`.
/// - `_abc` is NOT in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` → aggregate gate passes.
/// - No infusion registry configured → E-QUERY-039 check skipped (returns Ok()).
/// - Query passes plan gates and reaches execution (fails at sensor fan-out, not parse).
/// - Result: NOT QueryParseFailed.
///
/// This confirms the identifier-start fix is a strict narrowing (digit-leading rejected,
/// underscore-leading still admitted).
///
/// Traces to: ADR-048 D.7.2 (fn-name identifier-start constraint); F-PQLFN-P10-OBS-002.
#[tokio::test]
async fn test_fncall_underscore_leading_name_not_parse_error_obs_002() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where _abc(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be QueryParseFailed — `_abc` is a valid identifier start.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-P10-OBS-002: '_abc(device_id) = 5' must NOT return QueryParseFailed. \
         Underscore-leading names are valid identifier starts and must parse as fn-call LHS. \
         Got: {result:?}"
    );
}

// ── F-PQLFN-P21-OBS-004: LOW-005 fn-call-RHS negative regression locks ─────────
//
// BC-2.11.004 v1.41 LOW-005: "fn-call on RHS is not admitted — `rhs_expr` in
// `fn_call_comparison` and `field_comparison` accepts `temporal_rhs | literal` only;
// fn-call expressions are not a valid RHS alternative; `| where x = upper(y)` and
// `| where lower(x) = upper(y)` both fail at parse time with E-QUERY-001 (generic
// parse failure; no scope-limit citation)."
//
// These tests lock the current parse-time rejection behavior across all seven predicate
// positions so that a future extension of `rhs_expr` (to admit fn-call RHS) would
// require updating these tests explicitly rather than silently changing behavior.
//
// Tests are GREEN on arrival (locks on current rejection behavior).
//
// Surfaces covered (14 tests total, 2 per surface):
//   1. Pipe `| where`  (field_comparison LHS + fn_call_comparison LHS)
//   2. Filter mode     (field_comparison LHS + fn_call_comparison LHS)
//   3. SQL WHERE       (field_comparison LHS + fn_call_comparison LHS)
//   4. SqlPipe head WHERE
//   5. SqlPipe `| where` stage
//   6. DML WHERE (DELETE)
//   7. INSERT source_select WHERE (ADR-048 v1.13 §D.7.6, OD-7)
//
// Diagnostic-first assertion ordering (F-PQLFN-P19-OBS-001): specific Err variant
// (QueryParseFailed) asserted before any broad check; NOT-QueryPlanFailed guard follows.

// ── Surface 1: Pipe `| where` ─────────────────────────────────────────────────

/// LOW-005 (1/12) GREEN lock: pipe `| where` — field-comparison LHS, fn-call RHS.
///
/// Query: `FROM crowdstrike_detections | where device_id = upper(risk_score)`
///
/// `field_comparison`: LHS `device_id` → field_path ✓; `=` → compare_op ✓;
/// `upper(risk_score)` NOT in `rhs_expr` (temporal_rhs | literal only) → parse fails
/// → QueryParseFailed (E-QUERY-001).  The fn-call on RHS is the sole failure cause.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_pipe_where_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific variant before broad check.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (pipe | where, field LHS): `device_id = upper(risk_score)` must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only; fn-call is not a valid RHS \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — parse failure fires before plan-time gates.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (pipe | where, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time, not at plan time. Got: {result:?}"
    );
}

/// LOW-005 (2/12) GREEN lock: pipe `| where` — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) = upper(risk_score)`
///
/// `fn_call_comparison`: LHS `lower(device_id)` → FuncCall::Scalar ✓; `=` → compare_op ✓;
/// `upper(risk_score)` NOT in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
/// Chumsky backtracks to `field_comparison`; `lower` parsed as field_path, `(` is not a
/// compare_op → also fails.  Both alternatives fail → QueryParseFailed.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_pipe_where_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (pipe | where, fn-call LHS): `lower(device_id) = upper(risk_score)` must fail \
         to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only; fn-call RHS is not admitted \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (pipe | where, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── Surface 2: Filter mode ────────────────────────────────────────────────────

/// LOW-005 (3/12) GREEN lock: filter mode — field-comparison LHS, fn-call RHS.
///
/// Query: `crowdstrike_detections | device_id = upper(risk_score)`
///
/// Filter mode (`Ast::Filter`) uses the same `build_predicate_parser` as pipe `| where`.
/// `field_comparison`: `device_id` → field_path ✓; `=` → compare_op ✓;
/// `upper(risk_score)` NOT in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_filter_mode_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (filter mode, field LHS): `device_id = upper(risk_score)` must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         Filter mode shares build_predicate_parser; rhs_expr admits temporal_rhs | literal only \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (filter mode, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (4/12) GREEN lock: filter mode — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `crowdstrike_detections | lower(device_id) = upper(risk_score)`
///
/// Both `fn_call_comparison` (fn-call RHS mismatch) and `field_comparison` (open-paren
/// after field_path fails) alternatives fail → QueryParseFailed (E-QUERY-001).
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_filter_mode_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (filter mode, fn-call LHS): `lower(device_id) = upper(risk_score)` must fail \
         to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only; fn-call RHS not admitted \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (filter mode, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── Surface 3: SQL WHERE (Ast::Sql Select) ────────────────────────────────────

/// LOW-005 (5/12) GREEN lock: SQL WHERE — field-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE device_id = upper(risk_score)`
///
/// SQL WHERE predicate is parsed via `build_predicate_parser` (shared parser).
/// `field_comparison` LHS `device_id`; RHS `upper(risk_score)` NOT in `rhs_expr` →
/// parse fails → QueryParseFailed (E-QUERY-001).
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; BC-2.11.003 EC-11-003-007 (SQL WHERE parity);
///            F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sql_where_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SQL WHERE, field LHS): `device_id = upper(risk_score)` must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         SQL WHERE uses build_predicate_parser; rhs_expr admits temporal_rhs | literal only \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SQL WHERE, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (6/12) GREEN lock: SQL WHERE — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)`
///
/// `fn_call_comparison` LHS `lower(device_id)` ✓; RHS `upper(risk_score)` NOT in
/// `rhs_expr` → parse fails.  `field_comparison` fallback also fails. → QueryParseFailed.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; BC-2.11.003 EC-11-003-007 (SQL WHERE parity);
///            F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sql_where_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SQL WHERE, fn-call LHS): `lower(device_id) = upper(risk_score)` must fail \
         to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only; fn-call RHS not admitted \
         (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SQL WHERE, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── Surface 4: SqlPipe head WHERE ─────────────────────────────────────────────

/// LOW-005 (7/12) GREEN lock: SqlPipe head WHERE — field-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE device_id = upper(risk_score) | limit 10`
///
/// SqlPipe head SELECT … WHERE predicate is parsed via `build_predicate_parser`.
/// fn-call on RHS is not in `rhs_expr` → parse fails at the head WHERE clause →
/// QueryParseFailed (E-QUERY-001) before any SqlPipe stage is processed.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sqlpipe_head_where_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE device_id = upper(risk_score) | limit 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SqlPipe head WHERE, field LHS): `device_id = upper(risk_score)` in SqlPipe \
         head WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only (BC-2.11.004 v1.41 LOW-005). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SqlPipe head WHERE, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (8/12) GREEN lock: SqlPipe head WHERE — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score) | limit 10`
///
/// Two-sided fn-call in SqlPipe head WHERE.  `fn_call_comparison` LHS ✓; RHS
/// `upper(risk_score)` NOT in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sqlpipe_head_where_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections \
             WHERE lower(device_id) = upper(risk_score) | limit 10",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SqlPipe head WHERE, fn-call LHS): `lower(device_id) = upper(risk_score)` \
         in SqlPipe head WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only (BC-2.11.004 v1.41 LOW-005). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SqlPipe head WHERE, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── Surface 5: SqlPipe `| where` stage ───────────────────────────────────────

/// LOW-005 (9/12) GREEN lock: SqlPipe `| where` stage — field-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections | where device_id = upper(risk_score)`
///
/// SqlPipe pipe-stage `| where` uses `build_predicate_parser` (shared, six-caller).
/// fn-call on RHS is not in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
/// Parse fails before any SqlPipe execution path is reached.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sqlpipe_where_stage_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections | where device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SqlPipe | where stage, field LHS): `device_id = upper(risk_score)` in \
         SqlPipe | where stage must fail to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only (BC-2.11.004 v1.41 LOW-005). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SqlPipe | where stage, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (10/12) GREEN lock: SqlPipe `| where` stage — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `SELECT * FROM crowdstrike_detections | where lower(device_id) = upper(risk_score)`
///
/// Two-sided fn-call in SqlPipe `| where` stage.  `fn_call_comparison` LHS
/// `lower(device_id)` ✓; RHS `upper(risk_score)` NOT in `rhs_expr` → parse fails.
/// All predicate alternatives fail → QueryParseFailed (E-QUERY-001).
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_sqlpipe_where_stage_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections | where lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (SqlPipe | where stage, fn-call LHS): `lower(device_id) = upper(risk_score)` \
         in SqlPipe | where stage must fail to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only (BC-2.11.004 v1.41 LOW-005). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (SqlPipe | where stage, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── Surface 6: DML WHERE (DELETE) ─────────────────────────────────────────────

/// LOW-005 (11/14) GREEN lock: DML WHERE — field-comparison LHS, fn-call RHS.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE device_id = upper(risk_score)`
///
/// DML WHERE predicate is parsed via `build_delete_parser` which delegates to
/// `build_predicate_parser` (ADR-048 v1.6 OD-6 §D.7.5).  fn-call on RHS is not
/// in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
/// The engine never reaches the DML execution path.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_dml_where_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (DML WHERE, field LHS): `device_id = upper(risk_score)` in DELETE WHERE \
         must fail to parse (QueryParseFailed / E-QUERY-001). \
         build_delete_parser uses build_predicate_parser; rhs_expr admits temporal_rhs | literal \
         only (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (DML WHERE, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (12/14) GREEN lock: DML WHERE — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)`
///
/// Two-sided fn-call in DELETE WHERE.  `fn_call_comparison` LHS `lower(device_id)` ✓;
/// RHS `upper(risk_score)` NOT in `rhs_expr` → parse fails.  `field_comparison`
/// fallback also fails at `(device_id)` position. → QueryParseFailed (E-QUERY-001).
///
/// Negative contrast with `test_dml_where_fncall_lhs_date_like_e_query_042` (which
/// uses a literal RHS and succeeds at parse time): the sole difference here is that
/// the RHS is itself a fn-call, which `rhs_expr` does not admit.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P21-OBS-004.
#[tokio::test]
async fn test_BC_2_11_004_low_005_dml_where_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (DML WHERE, fn-call LHS): `lower(device_id) = upper(risk_score)` in DELETE \
         WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         build_delete_parser uses build_predicate_parser; rhs_expr admits temporal_rhs | literal \
         only; fn-call RHS not admitted (BC-2.11.004 v1.41 LOW-005). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (DML WHERE, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time, not at plan time. Got: {result:?}"
    );
}

// ── Surface 7: INSERT source_select WHERE (ADR-048 v1.13 §D.7.6, OD-7) ────────

/// LOW-005 (13/14) GREEN lock: INSERT source_select WHERE — field-comparison LHS, fn-call RHS.
///
/// Query: `INSERT INTO crowdstrike_detections (device_id) SELECT device_id FROM crowdstrike_detections WHERE device_id = upper(risk_score)`
///
/// INSERT source_select WHERE is Position 7 (ADR-048 v1.13 §D.7.6).  The WHERE
/// clause of the embedded SELECT is parsed via `build_sql_predicate_parser` →
/// `build_predicate_parser`; `rhs_expr` admits `temporal_rhs | literal` only.
/// `upper(risk_score)` is not in `rhs_expr` → parse fails → QueryParseFailed (E-QUERY-001).
///
/// GREEN on arrival: shared parser = shared restriction.  If this test fails, the
/// INSERT source_select WHERE path does not route through `build_predicate_parser`
/// (or `rhs_expr` has been extended without updating LOW-005 locks) — REAL DEFECT.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P33-LOW-001; ADR-048 v1.13 §D.7.6.
#[tokio::test]
async fn test_BC_2_11_004_low_005_insert_source_select_where_field_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "INSERT INTO crowdstrike_detections (device_id) SELECT device_id \
             FROM crowdstrike_detections WHERE device_id = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (INSERT source_select WHERE, field LHS): `device_id = upper(risk_score)` in \
         INSERT source_select WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         build_insert_parser embeds build_sql_parser for the SELECT; rhs_expr admits \
         temporal_rhs | literal only; fn-call RHS not admitted (BC-2.11.004 v1.41 LOW-005, \
         ADR-048 v1.13 §D.7.6, F-PQLFN-P33-LOW-001). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (INSERT source_select WHERE, field LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

/// LOW-005 (14/14) GREEN lock: INSERT source_select WHERE — fn-call-comparison LHS, fn-call RHS.
///
/// Query: `INSERT INTO crowdstrike_detections (device_id) SELECT device_id FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)`
///
/// Two-sided fn-call in INSERT source_select WHERE.  `fn_call_comparison` LHS
/// `lower(device_id)` ✓; RHS `upper(risk_score)` NOT in `rhs_expr` → parse fails.
/// `field_comparison` fallback also fails at `(device_id)` position. → QueryParseFailed.
///
/// Negative contrast with a fn-call LHS + literal RHS (which succeeds at parse time):
/// the sole difference here is that the RHS is itself a fn-call, which `rhs_expr` does not admit.
///
/// GREEN on arrival: shared parser = shared restriction (INSERT source_select WHERE,
/// ADR-048 v1.13 §D.7.6).  If this test fails — REAL DEFECT.
///
/// Traces to: BC-2.11.004 v1.41 LOW-005; F-PQLFN-P33-LOW-001; ADR-048 v1.13 §D.7.6.
#[tokio::test]
async fn test_BC_2_11_004_low_005_insert_source_select_where_fncall_lhs_fncall_rhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "INSERT INTO crowdstrike_detections (device_id) SELECT device_id \
             FROM crowdstrike_detections WHERE lower(device_id) = upper(risk_score)",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-005 (INSERT source_select WHERE, fn-call LHS): `lower(device_id) = upper(risk_score)` \
         in INSERT source_select WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         rhs_expr admits temporal_rhs | literal only; fn-call RHS not admitted \
         (BC-2.11.004 v1.41 LOW-005, ADR-048 v1.13 §D.7.6, F-PQLFN-P33-LOW-001). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-005 (INSERT source_select WHERE, fn-call LHS): must NOT be QueryPlanFailed. \
         fn-call RHS must be rejected at parse time. Got: {result:?}"
    );
}

// ── F-PQLFN-P22-MED-001: SqlPipe stages offset truthfulness ──────────────────
//
// ADR-048 §D.7.2 mandates that E-QUERY-001 aggregate-gate errors report the
// byte offset of the offending token in the ORIGINAL query string.
//
// `parse_sqlpipe_internal` splits the query at `split_offset` and passes
// `stages_str = &input[split_offset..]` to the stage parser.  Spans captured
// by `fn_call_comparison` inside the stage parser are relative to `stages_str`,
// NOT the original `input` — so reported offset = stage-relative position
// (wrong) instead of `split_offset + stage-relative position` (correct).
//
// Pre-fix: a `stddev` at position 45 in the original query appears at position
//   8 in `stages_str = "| where stddev(risk_score) = 5"` → offset reported as 8.
// Post-fix: offset reported as 45 (truthful ADR-048 §D.7.2).
//
// Fix mechanism: post-parse span shift — after the stage parser returns, walk
// every `PipeStage::Where` predicate tree and add `split_offset` to each
// `FuncCall::Scalar::span.{start,end}`.  Only `FuncCall::Scalar::span` is
// shifted; `FieldPath::span` is intentionally NOT shifted because no production
// code reads `FieldPath::span.start` for error-offset reporting (engine.rs only
// calls `collect_unknown_scalar_offsets_from_*` which extracts `FuncCall::Scalar::span`).
// Verified: `grep -n 'span\.start' crates/prism-query/src/` returns exactly one
// site — engine.rs collect_unknown_scalar_offsets_from_expr (F-PQLFN-P22-MED-001).
//
// Multi-stage case: the stage parser parses ALL stages from the single `stages_str`
// in one parse call, so `split_offset` is a uniform shift across all stages.
//
// Traces to: ADR-048 §D.7.2; F-PQLFN-P22-MED-001; BC-2.11.019.

/// F-PQLFN-P22-MED-001 (1/3): SqlPipe `| where` stage reports truthful offset.
///
/// Query: `SELECT * FROM crowdstrike_detections | where stddev(risk_score) = 5`
///
/// `split_offset` = 37 (position of `|`).
/// `stddev` in `stages_str = "| where stddev(risk_score) = 5"` → stage-relative = 8.
/// Absolute position of `stddev` in original query = 45.
///
/// # RED → GREEN
/// FAILS before fix: reported offset = 8 (stage-relative), expected 45 (absolute).
/// PASSES after fix: `shift_scalar_spans_in_stages` adds split_offset (37) to each
///   FuncCall::Scalar span captured during the stages parse → offset = 45.
///
/// Load-bearing (TD-VSDD-059): removing `shift_scalar_spans_in_stages` call in
/// `parse_sqlpipe_internal` reverts this test to failure (offset 8 ≠ 45).
#[tokio::test]
async fn test_pqlfn_p22_med001_aggregate_offset_sqlpipe_where_stage() {
    let query = "SELECT * FROM crowdstrike_detections | where stddev(risk_score) = 5";
    let expected_offset = query.find("stddev").expect("stddev must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P22-MED-001 (stages): E-QUERY-001 aggregate gate must report truthful \
                 offset pointing at 'stddev' in the ORIGINAL query string. \
                 Expected offset={expected_offset} (absolute), got offset={offset}. \
                 Pre-fix: fn_call_comparison spans are relative to stages_str \
                 (`&input[split_offset..]`), so offset = stage-relative position (wrong). \
                 Fix: shift FuncCall::Scalar spans by split_offset after stage parse \
                 (parse_sqlpipe_internal — F-PQLFN-P22-MED-001, ADR-048 §D.7.2)."
            );
            assert!(
                offset > 0,
                "F-PQLFN-P22-MED-001 (stages): offset must be > 0 for 'stddev' \
                 that does not start at byte 0. Got offset={offset}"
            );
        }
        other => panic!(
            "F-PQLFN-P22-MED-001 (stages): expected QueryParseFailed (E-QUERY-001) for \
             stddev in SqlPipe | where stage, got: {other:?}"
        ),
    }
}

/// F-PQLFN-P22-MED-001 (2/3): SqlPipe-HEAD WHERE already reports truthful offset.
///
/// Query: `SELECT device_id FROM crowdstrike_detections WHERE sum(risk_score) = 10 | limit 5`
///
/// `sql_head_str = input[..split_offset].trim_end()` starts at position 0 of `input`.
/// `build_sql_predicate_parser` delegates to `build_predicate_parser()` (filter_parser),
/// whose `fn_call_comparison` uses `map_with` to capture spans.  Since the head string
/// starts at byte 0, head spans ARE already absolute (no shift needed).
///
/// `sum` is at byte 51 in both `sql_head_str` and the original query.
///
/// # GREEN lock (no fix needed for this path)
/// PASSES before AND after the fix: head spans are already correct.
/// The fix's `shift_scalar_spans_in_stages` only touches `spq.stages`, not `spq.head`.
///
/// Traces to: ADR-048 §D.7.2; F-PQLFN-P22-MED-001 head-path verdict.
#[tokio::test]
async fn test_pqlfn_p22_med001_aggregate_offset_sqlpipe_head_where() {
    let query = "SELECT device_id FROM crowdstrike_detections WHERE sum(risk_score) = 10 | limit 5";
    let expected_offset = query.find("sum").expect("sum must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P22-MED-001 (head WHERE): E-QUERY-001 must report truthful offset \
                 pointing at 'sum' in the original query. \
                 Expected offset={expected_offset}, got offset={offset}. \
                 sql_head_str starts at position 0 of original input, so head spans are \
                 already absolute — no shift required (ADR-048 §D.7.2)."
            );
        }
        other => panic!(
            "F-PQLFN-P22-MED-001 (head WHERE): expected QueryParseFailed (E-QUERY-001) \
             for sum in SqlPipe head WHERE, got: {other:?}"
        ),
    }
}

/// F-PQLFN-P22-MED-001 (3/3): Multi-stage SqlPipe — second `| where` stage
/// aggregate also reports truthful offset.
///
/// Query: `SELECT * FROM crowdstrike_detections | where lower(device_id) = 'x' | where stddev(risk_score) > 5`
///
/// Two `| where` stages; `stddev` appears in the SECOND stage.
/// `split_offset` = 37 (position of first `|`).
/// `stages_str` = `"| where lower(device_id) = 'x' | where stddev(risk_score) > 5"`.
/// `stddev` in `stages_str` → stage-relative = 39.
/// Absolute position of `stddev` in original query = 76.
///
/// The stage parser processes both stages in a single parse call from `stages_str`,
/// so the shift is uniform across all stages.
///
/// # RED → GREEN
/// FAILS before fix: offset = 39 (stage-relative), expected 76 (absolute).
/// PASSES after fix: `shift_scalar_spans_in_stages` shifts ALL `FuncCall::Scalar`
///   spans by `split_offset` (37) → second-stage stddev offset = 76.
///
/// Load-bearing (TD-VSDD-059): this test ensures the fix handles multi-stage
/// queries, not just single-stage.
#[tokio::test]
async fn test_pqlfn_p22_med001_aggregate_offset_sqlpipe_where_second_stage() {
    let query = "SELECT * FROM crowdstrike_detections | where lower(device_id) = 'x' | where stddev(risk_score) > 5";
    let expected_offset = query.find("stddev").expect("stddev must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P22-MED-001 (second stage): E-QUERY-001 must report truthful offset \
                 pointing at 'stddev' in the ORIGINAL query string. \
                 Expected offset={expected_offset} (absolute), got offset={offset}. \
                 Pre-fix: stage-relative = 39, but 'stddev' is at absolute byte 76. \
                 Fix: shift_scalar_spans_in_stages applies uniform split_offset shift \
                 across ALL stages (F-PQLFN-P22-MED-001, ADR-048 §D.7.2)."
            );
            assert!(
                offset > 0,
                "F-PQLFN-P22-MED-001 (second stage): offset must be > 0. Got offset={offset}"
            );
        }
        other => panic!(
            "F-PQLFN-P22-MED-001 (second stage): expected QueryParseFailed (E-QUERY-001) \
             for stddev in second SqlPipe | where stage, got: {other:?}"
        ),
    }
}

// ── F-PQLFN-P23-LOW-001: BC-2.11.004 v1.41 LOW-002 LIKE fn-call-LHS lock ────────
//
// BC-2.11.004 v1.41 §Canonical Test Vectors (~line 141) documents:
//   `FROM crowdstrike_detections | where lower(host) LIKE '%server%'`
//   → Err(E-QUERY-001) [scope-limit LOW-002]
//
// `like_match` production in `build_predicate_parser` wires `field_path` on the LHS;
// `fn_call_comparison` is the SOLE production admitting fn-call LHS (compare_op set:
// =, !=, <, >, <=, >=). LIKE is not in compare_op → fn_call_comparison fails to match
// → parser backtracks to field_comparison → `lower` parsed as field_path, `(` is not
// a compare_op → field_comparison rejects → like_match tries `field_path LIKE ...`
// → `lower(device_id)` is not a valid field_path token sequence → like_match rejects
// → all alternatives exhausted → QueryParseFailed (E-QUERY-001, generic parse failure;
// no scope-limit citation in the message text per BC-2.11.004 v1.41 LOW-002).
//
// The test uses `device_id` (a schema column in the test fixture) instead of `host`
// (the BC canonical example) because the parse-time failure occurs before schema
// validation — any column name produces the same result. Using `device_id` follows
// the pattern of sibling tests in this file.
//
// LOW-002 coverage decision (F-PQLFN-P23-LOW-001): all 14 non-compose productions
// share the same enforcement mechanism — grammar omission (only `fn_call_comparison`
// admits fn-call LHS; all other productions hard-wire `field_path` on the LHS).
// One production per distinct operator class provides adequate representative coverage.
// `IEQ` is already tested in `test_BC_2_11_004_low_001_ieq_operator_with_fncall_lhs_rejected`
// (ieq_compare family). `LIKE` covers the like_match family. Together they span two
// distinct operator classes. Adding individual tests for all remaining 12 productions
// (MATCHES/=~, IN CIDR, NOT IN, IIN, IN, BETWEEN, IS NULL, HAS, MISSING,
// CONTAINS/STARTSWITH/ENDSWITH, CIDR, INE) would add test bulk with zero additional
// defect-detection value — the failure path is identical (field_path required on LHS
// in each production). LIKE alone suffices as the canonical representative for
// non-compare-op operator families (BC vector prescribed, F-PQLFN-P23-LOW-001).

/// LOW-002 LIKE GREEN lock: `like_match` production does not admit fn-call LHS.
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) LIKE '%server%'`
///
/// `fn_call_comparison` admits fn-call LHS only for the standard compare_op set
/// (`=`, `!=`, `<`, `>`, `<=`, `>=`). `LIKE` is handled by the separate `like_match`
/// production which requires `field_path` on the LHS. The parse fails before any
/// schema or plan-time checks are reached → QueryParseFailed (E-QUERY-001, generic
/// parse failure; no scope-limit citation in the message per BC-2.11.004 LOW-002).
///
/// Traces to: BC-2.11.004 v1.41 §Canonical Test Vectors LOW-002 LIKE vector;
///            F-PQLFN-P23-LOW-001; ADR-048 §D.7 fn_call_comparison scope limits.
#[tokio::test]
async fn test_BC_2_11_004_low_002_like_with_fncall_lhs_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) LIKE '%server%'",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-002 (LIKE): lower(device_id) LIKE '%server%' must fail to parse \
         (QueryParseFailed / E-QUERY-001). \
         like_match production requires field_path on LHS; fn-call LHS is not admitted \
         (BC-2.11.004 v1.41 LOW-002 canonical LIKE vector, F-PQLFN-P23-LOW-001). \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — parse failure fires before plan-time gates.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-002 (LIKE): must NOT be QueryPlanFailed. \
         fn-call LHS with LIKE must be rejected at parse time, not at plan time. \
         Got: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// LOW-006 — Reserved-keyword exclusion in `fn_call_comparison`
// (BC-2.11.004 v1.42, F-PQLFN-P26-OBS-002)
//
// `fn_call_comparison` must reject fn-call names that match any of the 20
// PrismQL predicate-level reserved keywords (case-insensitive):
//   NOT, AND, OR, IN, IIN, IEQ, INE, IS, BETWEEN, LIKE, CIDR, MATCHES, HAS,
//   MISSING, CONTAINS, ICONTAINS, STARTSWITH, ISTARTSWITH, ENDSWITH, IENDSWITH.
//
// Mechanism: `.validate()` at the end of `fn_call_comparison` emits a
// `Rich::custom` error (NOT a try_map backtrack) so the keyword message
// survives Chumsky choice() error-priority mechanics (see §Summary rationale
// below for NOT-backtrack analysis).
//
// Pre-fix state (Red Gate): keyword-shaped fn-names parse as FuncCall::Scalar
// via `fn_call_comparison`; plan time: DataFusion "Invalid function '<NAME>'"
// → QueryPlanFailed.  Tests 1–5 assert QueryParseFailed and are RED.
//
// Post-fix state: `.validate()` emits keyword error; all seven parse surfaces
// that share `build_predicate_parser` see non-empty errors → QueryParseFailed.
//
// Tests 6–7 are positive-guard locks (already GREEN before fix; regression
// prevention for legitimate fn-call and spaced NOT forms).
// ─────────────────────────────────────────────────────────────────────────────

/// LOW-006 (1/7) **RED**: pipe `| where` — `NOT` as fn-call name.
///
/// Query: `FROM crowdstrike_detections | where NOT(device_id) = 5`
///
/// BC-2.11.004 v1.42 canonical LOW-006 test vector.
///
/// Pre-fix: `kw("NOT").ignore_then(not.clone())` (not_pred arm-1) fails because
/// `(device_id)` alone is not a valid predicate; Chumsky backtracks to atom →
/// `fn_call_comparison` reads "NOT" + `(device_id)` + `= 5` → FuncCall::Scalar("NOT")
/// → plan time: DataFusion "Invalid function 'NOT'" → `QueryPlanFailed`.
///
/// Post-fix: `fn_call_comparison` emits keyword error via `.validate()` (not a
/// `try_map` backtrack, so the error is not lost to choice() error-priority);
/// `parse_pipe_with_limits` sees non-empty errors → `QueryParseFailed` with
/// message `"E-QUERY-001: 'NOT' is a PrismQL keyword and cannot be used as a
/// function name"`.
///
/// **Implemented GREEN** (fix-burst 20, commit 1a07a5f9): keyword exclusion was
/// introduced in `fn_call_comparison` via `.validate()` — this test was RED before
/// that fix-burst and has been GREEN since.
///
/// Traces to: BC-2.11.004 v1.42 §Canonical Test Vectors LOW-006 canonical vector;
///            F-PQLFN-P26-OBS-002; ADR-048 §D.7.
#[tokio::test]
async fn test_BC_2_11_004_low_006_pipe_keyword_not_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, pipe): `NOT(device_id) = 5` must fail to parse \
         (QueryParseFailed / E-QUERY-001). `NOT` is a PrismQL reserved keyword \
         and cannot be used as a fn-call name in `fn_call_comparison` \
         (BC-2.11.004 v1.42 LOW-006 canonical vector, F-PQLFN-P26-OBS-002). \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection must fire at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, pipe): must NOT be QueryPlanFailed. \
         Keyword fn-name rejection must fire at parse time, before plan-time gates. \
         Got: {result:?}"
    );

    // POL-24 message-text lock: error detail must cite the keyword-message template
    // and the specific quoted keyword name (BC-2.11.004 LOW-006).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, pipe): error message must contain keyword-message template substring \
         'is a PrismQL keyword and cannot be used as a function name' \
         (BC-2.11.004 LOW-006, POL-24 message-text lock). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, pipe): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24 message-text lock). Got: {err_display:?}"
    );
}

/// LOW-006 (2/7) **RED**: pipe `| where` — `CONTAINS` as fn-call name.
///
/// Query: `FROM crowdstrike_detections | where CONTAINS(device_id) = 5`
///
/// `CONTAINS` is in the 21-keyword reserved list (it is the `string_op_match`
/// operator in filter grammar).  The `string_op_match` production expects
/// `field_path CONTAINS literal` form; it fails on `CONTAINS(device_id)` because
/// "CONTAINS" as field_path is followed by `(`, not the keyword — leaving
/// `fn_call_comparison` to match.
///
/// Pre-fix: `fn_call_comparison` reads "CONTAINS" + `(device_id)` + `= 5` →
/// FuncCall::Scalar("CONTAINS") → plan time: "Invalid function" → `QueryPlanFailed`.
///
/// Post-fix: keyword exclusion → `QueryParseFailed`.
///
/// **Implemented GREEN** (fix-burst 20, commit 1a07a5f9): keyword exclusion was
/// introduced in `fn_call_comparison` — this test was RED before fix-burst 20.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 (CONTAINS in 21-keyword list, NULL added v1.48 EC-11-085);
///            F-PQLFN-P26-OBS-002.
#[tokio::test]
async fn test_BC_2_11_004_low_006_pipe_keyword_contains_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where CONTAINS(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (CONTAINS, pipe): `CONTAINS(device_id) = 5` must fail to parse \
         (QueryParseFailed / E-QUERY-001). `CONTAINS` is a PrismQL reserved keyword; \
         fn-call use must be rejected at parse time \
         (BC-2.11.004 v1.42 LOW-006, F-PQLFN-P26-OBS-002). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (CONTAINS, pipe): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (CONTAINS, pipe): error message must contain keyword-message template substring \
         (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'CONTAINS'"),
        "LOW-006 (CONTAINS, pipe): error message must contain quoted keyword name \"'CONTAINS'\" \
         (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
}

/// LOW-006 (3/7) **RED**: pipe `| where` — `not` (lowercase) as fn-call name.
///
/// Query: `FROM crowdstrike_detections | where not(device_id) = 5`
///
/// The keyword check in LOW-006 is case-insensitive (`eq_ignore_ascii_case`).
/// Lowercase `not` is NOT intercepted by `kw("NOT")` in not_pred (the `kw()` helper
/// in filter_parser.rs performs a case-sensitive ASCII comparison by convention);
/// therefore `fn_call_comparison` reaches "not" first and currently parses it as a
/// function name.
///
/// Pre-fix: FuncCall::Scalar("not") → plan time: "Invalid function 'not'" →
/// `QueryPlanFailed`.
///
/// Post-fix: case-insensitive keyword check rejects "not" → `QueryParseFailed`.
///
/// **Implemented GREEN** (fix-burst 20, commit 1a07a5f9): case-insensitive exclusion
/// (`eq_ignore_ascii_case`) was introduced — this test was RED before fix-burst 20.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 (`eq_ignore_ascii_case` requirement);
///            F-PQLFN-P26-OBS-002.
#[tokio::test]
async fn test_BC_2_11_004_low_006_pipe_keyword_lowercase_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where not(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (not lowercase, pipe): `not(device_id) = 5` must fail to parse \
         (QueryParseFailed / E-QUERY-001). Keyword exclusion is case-insensitive \
         (`eq_ignore_ascii_case`); lowercase `not` must be rejected like `NOT` \
         (BC-2.11.004 v1.42 LOW-006, F-PQLFN-P26-OBS-002). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (not lowercase, pipe): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    // POL-24 message-text lock: lowercase input preserved in message (`'not'`, not `'NOT'`)
    // because `func_name` captures the original case from the query string (BC-2.11.004 LOW-006).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (not lowercase, pipe): error message must contain keyword-message template \
         substring (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'not'"),
        "LOW-006 (not lowercase, pipe): error message must contain quoted keyword name \"'not'\" \
         (original-case preservation — func_name from grammar slice, BC-2.11.004 LOW-006, POL-24). \
         Got: {err_display:?}"
    );
}

/// LOW-006 (4/7) **RED**: SQL WHERE surface — `NOT` as fn-call name.
///
/// Query: `SELECT device_id FROM crowdstrike_detections WHERE NOT(device_id) = 5`
///
/// `build_predicate_parser` is shared by six callers (BC-2.11.004 §Postconditions
/// shared-parser scope).  SQL WHERE uses it via `build_sql_predicate_parser`.
/// The keyword exclusion in `fn_call_comparison` therefore applies to the SQL
/// WHERE surface without any additional wiring.
///
/// Pre-fix: FuncCall::Scalar("NOT") in WHERE → "Invalid function" → `QueryPlanFailed`.
///
/// Post-fix: parse-time keyword rejection → `QueryParseFailed`.
///
/// **Implemented GREEN** (fix-burst 20, commit 1a07a5f9): shared-parser keyword
/// exclusion introduced in `fn_call_comparison` — this test was RED before fix-burst 20.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 shared-parser scope;
///            BC-2.11.003 EC-11-003-007; ADR-048 v1.6 OD-6 §D.7.5.
#[tokio::test]
async fn test_BC_2_11_004_low_006_sql_where_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections WHERE NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, SQL WHERE): `NOT(device_id) = 5` in SQL WHERE must fail to parse \
         (QueryParseFailed / E-QUERY-001). `build_predicate_parser` is shared by SQL WHERE \
         via `build_sql_predicate_parser`; keyword exclusion in `fn_call_comparison` applies \
         (BC-2.11.004 v1.42 LOW-006 shared-parser scope). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, SQL WHERE): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, SQL WHERE): error message must contain keyword-message template substring \
         (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, SQL WHERE): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
}

/// LOW-006 (5/7) **RED**: SqlPipe `| where` stage — `NOT` as fn-call name.
///
/// Query: `SELECT * FROM crowdstrike_detections | where NOT(device_id) = 5`
///
/// SqlPipe `| where` stages are parsed by `build_pipe_stages_parser` in
/// `pipe_parser.rs`, which calls `build_predicate_parser()` directly (same
/// shared base as all other surfaces).  The keyword exclusion propagates.
///
/// Pre-fix: FuncCall::Scalar("NOT") in SqlPipe stage → "Invalid function" →
/// `QueryPlanFailed`.
///
/// Post-fix: parse-time keyword rejection → `QueryParseFailed`.
///
/// **Implemented GREEN** (fix-burst 20, commit 1a07a5f9): shared-parser keyword
/// exclusion introduced — this test was RED before fix-burst 20.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 shared-parser scope (SqlPipe `| where`);
///            F-PQLFN-P26-OBS-002.
#[tokio::test]
async fn test_BC_2_11_004_low_006_sqlpipe_stage_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections | where NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, SqlPipe | where): `NOT(device_id) = 5` in SqlPipe `| where` stage \
         must fail to parse (QueryParseFailed / E-QUERY-001). \
         `build_pipe_stages_parser` uses `build_predicate_parser` directly; keyword \
         exclusion in `fn_call_comparison` applies (BC-2.11.004 v1.42 LOW-006). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, SqlPipe | where): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, SqlPipe | where): error message must contain keyword-message template \
         substring (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, SqlPipe | where): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24). Got: {err_display:?}"
    );
}

/// LOW-006 (6/7) GREEN lock: `NOT (space) predicate` still parses after fix.
///
/// Query: `FROM crowdstrike_detections | where NOT (device_id = 'windows')`
///
/// `NOT` with a SPACE before the parenthesised predicate is handled by not_pred
/// arm-1 (`kw("NOT").padded().ignore_then(not.clone())`), which matches
/// `NOT ` then parses `(device_id = 'windows')` as a parenthesised predicate.
/// `fn_call_comparison` is NEVER tried for this form — the choice is resolved by
/// arm-1 before reaching atom.
///
/// The LOW-006 keyword gate in `fn_call_comparison` therefore does NOT affect
/// `NOT (pred)` syntax — it only applies when the identifier is IMMEDIATELY
/// followed by `(` with no intervening whitespace that changes the parse path.
///
/// Lock: this test is GREEN before and after the fix.  Any regression in `NOT`
/// space-form parsing would appear here.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 positive guard; F-PQLFN-P26-OBS-002.
#[tokio::test]
async fn test_BC_2_11_004_low_006_not_space_predicate_positive_guard() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where NOT (device_id = 'windows')",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be a parse error — the spaced `NOT (pred)` form is valid PrismQL.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 positive guard (NOT space): `NOT (device_id = 'windows')` must NOT \
         fail to parse. The LOW-006 keyword gate applies only in `fn_call_comparison` \
         (identifier immediately followed by `(`); the space-form is resolved by \
         not_pred arm-1 before atom is tried \
         (BC-2.11.004 v1.42 LOW-006, F-PQLFN-P26-OBS-002). Got: {result:?}"
    );
}

/// LOW-006 (7/7) GREEN lock: `lower()` (non-keyword fn-call) still parses after fix.
///
/// Query: `FROM crowdstrike_detections | where lower(device_id) = 'abc'`
///
/// `lower` is NOT in the 21-keyword reserved list; it is a valid DataFusion
/// built-in scalar function name.  The LOW-006 keyword gate in `fn_call_comparison`
/// must NOT reject it — only the 20 PrismQL predicate-level reserved keywords are
/// excluded.
///
/// Lock: this test is GREEN before and after the fix.  Any over-rejection (e.g.,
/// matching `lower` accidentally) would appear here.
///
/// Traces to: BC-2.11.004 v1.42 LOW-006 positive guard; F-PQLFN-P26-OBS-002;
///            ADR-048 §D.7 fn_call_comparison scope.
#[tokio::test]
async fn test_BC_2_11_004_low_006_lower_fn_call_positive_guard() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) = 'abc'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be a parse error — `lower` is not a reserved keyword.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 positive guard (lower): `lower(device_id) = 'abc'` must NOT fail to \
         parse. `lower` is not in the 21-keyword reserved list; `fn_call_comparison` \
         must admit it (BC-2.11.004 v1.42 LOW-006, F-PQLFN-P26-OBS-002). \
         Got: {result:?}"
    );
}

/// F-PQLFN-P27-MED-001: SqlPipe `| where` stage keyword-rejection offset is truthful.
///
/// Query: `SELECT * FROM crowdstrike_detections | where NOT(device_id) = 5`
///
/// `split_offset` = 37 (position of `|` in the original query).
/// `stages_str = "| where NOT(device_id) = 5"`.
/// `NOT` in `stages_str` → stage-relative offset = 8.
/// `NOT` in original query → absolute offset = 45.
///
/// The LOW-006 keyword gate in `fn_call_comparison` emits `Rich::custom` with span
/// `func_span.start..func_span.end` where `func_span.start = 8` (stage-relative).
/// `rich_to_parse_error` captures `err.span().start = 8` as `ParseError.offset`.
/// This error is returned by `parse_sqlpipe_internal` via the `stage_errs` early-return
/// path **WITHOUT** any offset shift.
///
/// Pre-fix: `PrismError::QueryParseFailed { offset: 8 }` — stage-relative, wrong.
/// Post-fix: `parse_sqlpipe_internal` shifts all `stage_errs` offsets by `split_offset`
///   (37) before returning → `PrismError::QueryParseFailed { offset: 45 }` — absolute.
///
/// Parallel to the `shift_scalar_spans_in_stages` shift (success path, F-PQLFN-P22-MED-001),
/// but for the error path (F-PQLFN-P27-MED-001, ADR-048 §D.7.2).
///
/// # RED → GREEN
/// FAILS before fix: offset = 8 (stage-relative), expected 45 (absolute).
/// PASSES after fix: shift applied in the `stage_errs` early-return path → offset = 45.
///
/// Load-bearing (TD-VSDD-059): removing the shift call in `parse_sqlpipe_internal`'s
/// stage-error return path reverts this test to failure (offset 8 ≠ 45).
///
/// Traces to: F-PQLFN-P27-MED-001; ADR-048 §D.7.2 truthful-offset principle;
///            BC-2.11.004 v1.42 LOW-006 (keyword fn-name rejection).
#[tokio::test]
async fn test_pqlfn_p27_med001_sqlpipe_stage_keyword_error_offset_truthful() {
    let query = "SELECT * FROM crowdstrike_detections | where NOT(device_id) = 5";
    let expected_offset = query.find("NOT").expect("NOT must be in query");

    let engine = make_crowdstrike_detections_engine();
    let result = engine.execute(query, QueryOptions::default()).await;

    match result {
        Err(PrismError::QueryParseFailed { offset, .. }) => {
            assert_eq!(
                offset, expected_offset,
                "F-PQLFN-P27-MED-001: LOW-006 keyword-rejection error must report \
                 truthful (absolute) offset pointing at 'NOT' in the ORIGINAL query. \
                 Expected offset={expected_offset} (absolute), got offset={offset}. \
                 Pre-fix: the stage_errs early-return path in parse_sqlpipe_internal \
                 does NOT shift errors by split_offset — Rich::custom span is \
                 stage-relative (stages_str position), so offset = 8 instead of 45. \
                 Fix: shift all stage_errs offsets by split_offset (37) before returning, \
                 parallel to shift_scalar_spans_in_stages on the success path \
                 (F-PQLFN-P27-MED-001, ADR-048 §D.7.2)."
            );
            assert!(
                offset > 0,
                "F-PQLFN-P27-MED-001: offset must be > 0 for 'NOT' \
                 that does not start at byte 0 of the original query. \
                 Got offset={offset}"
            );
        }
        other => panic!(
            "F-PQLFN-P27-MED-001: expected QueryParseFailed (E-QUERY-001) for \
             LOW-006 keyword fn-name rejection in SqlPipe | where stage, got: {other:?}"
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PQLFN-P27-MED-002: LOW-006 coverage for three previously-uncovered surfaces
//
// BC-2.11.004 §Postconditions SHARED-PARSER SCOPE names 7 parse surfaces that
// share `build_predicate_parser`.  Fix-burst 20 (commit 1a07a5f9) shipped tests
// for pipe `| where`, SQL WHERE, and SqlPipe `| where`; THREE surfaces lacked
// LOW-006 coverage (fix-burst 27):
//   (A) filter-mode root predicate (Ast::Filter path)
//   (B) SQL HAVING (build_having_predicate_parser fallthrough to base predicate)
//   (C) SQL DML WHERE (build_delete_parser / build_update_parser)
// Surface (D) INSERT source_select WHERE (ADR-048 v1.13 §D.7.6) added fix-burst 25.
//
// Because the keyword exclusion gate lives in `fn_call_comparison` inside
// `build_predicate_parser`, which IS shared by all seven surfaces, the following
// tests are expected to be GREEN on arrival (shared parser = shared gate).
// If any test FAILS, that is a REAL DEFECT (the gate does not reach this
// surface) — it must not be papered over.
//
// Each test also carries POL-24 message-text locks for the keyword-message
// template and quoted keyword name (F-PQLFN-P27-MED-003 coverage extension).
// ─────────────────────────────────────────────────────────────────────────────

/// LOW-006 (surface A): filter-mode root predicate — `NOT` as fn-call name.
///
/// Query: `crowdstrike_detections | NOT(device_id) = 5`
///
/// Filter mode is activated when the query contains `|` outside string literals
/// but lacks a `FROM` or `SELECT` prefix (BC-2.11.002 mode precedence).  The
/// root predicate is parsed by `build_predicate_parser` directly — the same
/// shared parser that enforces LOW-006.
///
/// Expected behavior: keyword exclusion fires in `fn_call_comparison` via
/// `.validate()` → `QueryParseFailed` (E-QUERY-001).  The analyst likely
/// intended `NOT (device_id = 5)` (not-predicate spaced form).
///
/// GREEN on arrival: shared parser = shared gate.  If this test fails, the
/// filter-mode root-predicate path is NOT wired through `build_predicate_parser`
/// and that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-006 (shared-parser scope, filter-mode surface);
///            F-PQLFN-P27-MED-002; F-PQLFN-P27-MED-003 (POL-24 message lock).
#[tokio::test]
async fn test_BC_2_11_004_low_006_filter_mode_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, filter mode): `NOT(device_id) = 5` in filter-mode root predicate \
         must fail to parse (QueryParseFailed / E-QUERY-001). `NOT` is a PrismQL reserved \
         keyword; `fn_call_comparison` keyword gate applies to this surface via \
         `build_predicate_parser` (BC-2.11.004 LOW-006 shared-parser scope, \
         F-PQLFN-P27-MED-002). Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, filter mode): must NOT be QueryPlanFailed. \
         Keyword fn-name rejection must fire at parse time (BC-2.11.004 LOW-006). \
         Got: {result:?}"
    );

    // POL-24 message-text lock: keyword-message template and quoted keyword name
    // (BC-2.11.004 LOW-006, F-PQLFN-P27-MED-003).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, filter mode): error message must contain keyword-message template \
         substring 'is a PrismQL keyword and cannot be used as a function name' \
         (BC-2.11.004 LOW-006, POL-24 message-text lock, F-PQLFN-P27-MED-003). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, filter mode): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). Got: {err_display:?}"
    );
}

/// LOW-006 (surface B): SQL HAVING — `NOT` as fn-call name.
///
/// Query: `SELECT count(*) FROM crowdstrike_detections GROUP BY device_id HAVING NOT(device_id) = 5`
///
/// `build_having_predicate_parser` tries the `agg_comparison` arm first (COUNT /
/// SUM / AVG / MIN / MAX / DISTINCT_COUNT).  `NOT` is not an aggregate function →
/// `agg_comparison` fails → falls through to `base` = `build_sql_predicate_parser`
/// → `build_predicate_parser` → `fn_call_comparison` → LOW-006 keyword exclusion
/// → `QueryParseFailed` (E-QUERY-001).
///
/// GREEN on arrival: shared parser = shared gate.  If this test fails, the HAVING
/// fallthrough to the base predicate is NOT reaching `fn_call_comparison` and
/// that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-006 (shared-parser scope, SQL HAVING surface);
///            ADR-048 D.3 (HAVING grammar); F-PQLFN-P27-MED-002;
///            F-PQLFN-P27-MED-003 (POL-24 message lock).
#[tokio::test]
async fn test_BC_2_11_004_low_006_sql_having_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT count(*) FROM crowdstrike_detections GROUP BY device_id HAVING NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, SQL HAVING): `NOT(device_id) = 5` in HAVING must fail to parse \
         (QueryParseFailed / E-QUERY-001). `NOT` is not an aggregate fn so \
         `agg_comparison` fails; fallthrough to `base` predicate reaches \
         `fn_call_comparison` keyword exclusion (BC-2.11.004 LOW-006 shared-parser \
         scope, ADR-048 D.3, F-PQLFN-P27-MED-002). Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, SQL HAVING): must NOT be QueryPlanFailed. \
         Keyword fn-name rejection must fire at parse time (BC-2.11.004 LOW-006). \
         Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006, F-PQLFN-P27-MED-003).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, SQL HAVING): error message must contain keyword-message template \
         substring (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, SQL HAVING): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). Got: {err_display:?}"
    );
}

/// LOW-006 (surface C): SQL DML WHERE — `NOT` as fn-call name.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE NOT(device_id) = 5`
///
/// `build_delete_parser` delegates the WHERE predicate to `build_predicate_parser`
/// (ADR-048 v1.6 OD-6 §D.7.5).  The keyword exclusion in `fn_call_comparison`
/// therefore applies to the DML WHERE surface without any additional wiring.
///
/// GREEN on arrival: shared parser = shared gate.  If this test fails, the DML
/// WHERE path is NOT routed through `build_predicate_parser` / `fn_call_comparison`
/// and that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-006 (shared-parser scope, SQL DML WHERE surface);
///            ADR-048 v1.6 OD-6 §D.7.5; F-PQLFN-P27-MED-002;
///            F-PQLFN-P27-MED-003 (POL-24 message lock).
#[tokio::test]
async fn test_BC_2_11_004_low_006_dml_where_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, DML WHERE): `NOT(device_id) = 5` in DELETE WHERE must fail to parse \
         (QueryParseFailed / E-QUERY-001). `build_delete_parser` uses `build_predicate_parser` \
         for the WHERE clause; keyword exclusion in `fn_call_comparison` applies \
         (BC-2.11.004 LOW-006 shared-parser scope, ADR-048 v1.6 OD-6 §D.7.5, \
         F-PQLFN-P27-MED-002). Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, DML WHERE): must NOT be QueryPlanFailed. \
         Keyword fn-name rejection must fire at parse time (BC-2.11.004 LOW-006). \
         Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006, F-PQLFN-P27-MED-003).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, DML WHERE): error message must contain keyword-message template \
         substring (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, DML WHERE): error message must contain quoted keyword name \"'NOT'\" \
         (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). Got: {err_display:?}"
    );
}

/// LOW-006 (surface D): INSERT source_select WHERE — `NOT` as fn-call name.
///
/// Query: `INSERT INTO crowdstrike_detections (device_id) SELECT device_id FROM crowdstrike_detections WHERE NOT(device_id) = 5`
///
/// INSERT source_select WHERE is Position 7 (ADR-048 v1.13 §D.7.6).  The SELECT's WHERE
/// clause parses via `build_sql_predicate_parser` → `build_predicate_parser`;
/// keyword exclusion in `fn_call_comparison` therefore applies to this surface.
///
/// GREEN on arrival: shared parser = shared keyword gate.  If this test FAILS, that is
/// a REAL DEFECT (the gate does not reach the INSERT source_select WHERE surface).
///
/// POL-24 message-text locks: error must contain the canonical keyword-message template
/// and the quoted keyword name "'NOT'" (F-PQLFN-P27-MED-003).
///
/// Traces to: BC-2.11.004 LOW-006 (shared-parser scope, INSERT source_select WHERE surface);
///            ADR-048 v1.13 §D.7.6; F-PQLFN-P33-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_low_006_insert_source_select_where_keyword_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "INSERT INTO crowdstrike_detections (device_id) SELECT device_id \
             FROM crowdstrike_detections WHERE NOT(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Diagnostic-first: specific Err variant before broad check (F-PQLFN-P19-OBS-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-006 (NOT, INSERT source_select WHERE): `NOT(device_id) = 5` in INSERT source_select \
         WHERE must fail to parse (QueryParseFailed / E-QUERY-001). \
         `build_insert_parser` embeds `build_sql_parser`; keyword exclusion in `fn_call_comparison` \
         applies via `build_predicate_parser` (BC-2.11.004 LOW-006 shared-parser scope, \
         ADR-048 v1.13 §D.7.6, F-PQLFN-P33-LOW-001). Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-006 (NOT, INSERT source_select WHERE): must NOT be QueryPlanFailed. \
         Keyword fn-name rejection must fire at parse time (BC-2.11.004 LOW-006). \
         Got: {result:?}"
    );

    // POL-24 message-text lock (BC-2.11.004 LOW-006, F-PQLFN-P27-MED-003).
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "LOW-006 (NOT, INSERT source_select WHERE): error message must contain keyword-message \
         template substring (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "LOW-006 (NOT, INSERT source_select WHERE): error message must contain quoted keyword \
         name \"'NOT'\" (BC-2.11.004 LOW-006, POL-24, F-PQLFN-P27-MED-003). \
         Got: {err_display:?}"
    );
}

// ── LOW-007 — Star-arg scope limit: `fn_call_arg` admits `literal | field_path` only
// (BC-2.11.004 LOW-007, F-PQLFN-P31-OBS-001)
//
// `Expr::Star` (`*`) is not admissible as a fn-call argument in predicate position —
// `| where count(*) = 5` fails closed at parse time with a generic E-QUERY-001.
// `fn_call_arg` in `fn_call_comparison` admits `literal | field_path` only; `*` is
// neither (BC-2.11.004 LOW-007).
//
// The aggregate gate's canonical HAVING-redirect message (ADR-048 §D.7.2) applies
// only to parseable forms like `count() = 5` / `count(col) = 5`; `count(*)` fails
// before the aggregate gate is reached.
//
// These tests are GREEN on arrival: `fn_call_arg` never matched `*`; the tests
// document already-correct behavior as load-bearing lock tests (TD-VSDD-059).
// If any surface unexpectedly ACCEPTS `count(*) = 5`, that is a REAL DEFECT.
//
// Coverage: 7 shared-parser surfaces (BC-2.11.004 §Postconditions SHARED-PARSER SCOPE).
// ──────────────────────────────────────────────────────────────────────────────────────

/// LOW-007 (1/7) GREEN lock: pipe `| where` — `count(*)` star-arg rejected at parse time.
///
/// Query: `FROM crowdstrike_detections | where count(*) = 5`
///
/// `fn_call_arg` in `fn_call_comparison` admits `literal | field_path` only.
/// `*` is neither — the delimited arg-list fails when it encounters `*` as
/// the first token after `(`, leaving `)` expected but `*` found.
///
/// Expected: `QueryParseFailed` (E-QUERY-001) at parse time.
/// Must NOT be `QueryPlanFailed` — the aggregate gate only fires on parseable
/// forms (`count() = 5`, `count(col) = 5`); `count(*)` never reaches it.
///
/// GREEN on arrival. If this test fails, `fn_call_arg` has been extended to
/// admit `*` without architectural adjudication — that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit); F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_pipe_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (pipe | where): `count(*) = 5` must fail to parse (QueryParseFailed / \
         E-QUERY-001). `fn_call_arg` admits `literal | field_path` only; `*` is neither \
         (BC-2.11.004 LOW-007 star-arg scope limit). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (pipe | where): must NOT be QueryPlanFailed. Star-arg rejection fires \
         at parse time before the aggregate gate (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (2/7) GREEN lock: filter mode — `count(*)` star-arg rejected at parse time.
///
/// Query: `crowdstrike_detections | count(*) = 5`
///
/// Filter mode source-qualified form: source `crowdstrike_detections`, `|` separator,
/// predicate `count(*) = 5`. `build_predicate_parser` parses the predicate;
/// `fn_call_arg` does not admit `*` → `QueryParseFailed`.
///
/// GREEN on arrival. If this test fails, filter-mode root-predicate path does not
/// route through `fn_call_comparison` / `fn_call_arg` — that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, filter-mode surface);
///            F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_filter_mode_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (filter mode): `count(*) = 5` in filter-mode root predicate must fail \
         to parse (QueryParseFailed / E-QUERY-001). `fn_call_arg` admits `literal | \
         field_path` only; `*` is neither (BC-2.11.004 LOW-007, filter-mode surface). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (filter mode): must NOT be QueryPlanFailed. Star-arg rejection fires \
         at parse time (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (3/7) GREEN lock: SQL WHERE — `count(*)` star-arg rejected at parse time.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE count(*) = 5`
///
/// `build_sql_predicate_parser` calls `build_predicate_parser`; `fn_call_arg` does
/// not admit `*` → SQL WHERE parse fails → `QueryParseFailed`.
///
/// GREEN on arrival. If this test fails, SQL WHERE is not routing through
/// `fn_call_comparison` / `fn_call_arg` — that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, SQL WHERE surface);
///            F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_sql_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (SQL WHERE): `count(*) = 5` in SQL WHERE must fail to parse \
         (QueryParseFailed / E-QUERY-001). `build_predicate_parser` shared parser \
         via `build_sql_predicate_parser`; `fn_call_arg` admits `literal | field_path` \
         only (BC-2.11.004 LOW-007 star-arg scope limit). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (SQL WHERE): must NOT be QueryPlanFailed. Star-arg rejection fires \
         at parse time (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (4/7) GREEN lock: SqlPipe head WHERE — `count(*)` star-arg rejected.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE count(*) = 5 | limit 5`
///
/// `parse_sqlpipe_internal` splits at the first pipe stage `|`; the SQL head
/// `SELECT * FROM crowdstrike_detections WHERE count(*) = 5` is parsed via
/// `build_predicate_parser` which does not admit `*` in `fn_call_arg`.
/// The SqlPipe head parse fails → `QueryParseFailed`.
///
/// Distinct from the SqlPipe `| where` stage test (5/6): this exercises the SQL
/// head portion of a SqlPipe query, not a pipe-stage WHERE.
///
/// GREEN on arrival. If this test fails, the SqlPipe head WHERE path has diverged
/// from the shared predicate parser — that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, SqlPipe head WHERE surface);
///            F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_sqlpipe_head_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE count(*) = 5 | limit 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (SqlPipe head WHERE): `count(*) = 5` in SqlPipe SQL-head WHERE must \
         fail to parse (QueryParseFailed / E-QUERY-001). `parse_sqlpipe_internal` SQL \
         head uses `build_predicate_parser`; `fn_call_arg` admits `literal | field_path` \
         only (BC-2.11.004 LOW-007 star-arg scope limit). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (SqlPipe head WHERE): must NOT be QueryPlanFailed. Star-arg rejection \
         fires at parse time (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (5/7) GREEN lock: SqlPipe `| where` stage — `count(*)` star-arg rejected.
///
/// Query: `SELECT * FROM crowdstrike_detections | where count(*) = 5`
///
/// `build_pipe_stages_parser` uses `build_predicate_parser` directly for `| where`
/// stage predicates (shared parser). `fn_call_arg` does not admit `*`
/// → pipe-stage predicate parse fails → `QueryParseFailed`.
///
/// GREEN on arrival. If this test fails, `build_pipe_stages_parser` is not
/// routing through `fn_call_comparison` / `fn_call_arg` — that is a REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, SqlPipe `| where` surface);
///            F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_sqlpipe_stage_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections | where count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (SqlPipe | where stage): `count(*) = 5` in SqlPipe pipe-stage WHERE \
         must fail to parse (QueryParseFailed / E-QUERY-001). `build_pipe_stages_parser` \
         uses `build_predicate_parser`; `fn_call_arg` admits `literal | field_path` only \
         (BC-2.11.004 LOW-007 star-arg scope limit). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (SqlPipe | where stage): must NOT be QueryPlanFailed. Star-arg \
         rejection fires at parse time (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (6/7) GREEN lock: DML WHERE — `count(*)` star-arg rejected at parse time.
///
/// Query: `DELETE FROM crowdstrike_detections WHERE count(*) = 5`
///
/// `build_delete_parser` delegates the WHERE predicate to `build_predicate_parser`
/// (ADR-048 v1.6 OD-6 §D.7.5). `fn_call_arg` does not admit `*`
/// → DML WHERE parse fails → `QueryParseFailed`.
///
/// GREEN on arrival. If this test fails, the DML WHERE path is not routing through
/// `build_predicate_parser` / `fn_call_comparison` / `fn_call_arg` — REAL DEFECT.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, SQL DML WHERE surface);
///            ADR-048 v1.6 OD-6 §D.7.5; F-PQLFN-P31-OBS-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_dml_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "DELETE FROM crowdstrike_detections WHERE count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (DML WHERE): `count(*) = 5` in DELETE WHERE must fail to parse \
         (QueryParseFailed / E-QUERY-001). `build_delete_parser` uses \
         `build_predicate_parser` for WHERE; `fn_call_arg` admits `literal | field_path` \
         only (BC-2.11.004 LOW-007 star-arg scope limit, ADR-048 v1.6 OD-6 §D.7.5). \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (DML WHERE): must NOT be QueryPlanFailed. Star-arg rejection fires \
         at parse time before the aggregate gate (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

/// LOW-007 (7/7) GREEN lock: INSERT source_select WHERE — `count(*)` star-arg rejected.
///
/// Query: `INSERT INTO crowdstrike_detections (device_id) SELECT device_id FROM crowdstrike_detections WHERE count(*) = 5`
///
/// INSERT source_select WHERE is Position 7 (ADR-048 v1.13 §D.7.6).  The SELECT's WHERE
/// clause parses via `build_sql_predicate_parser` → `build_predicate_parser`;
/// `fn_call_arg` admits `literal | field_path` only — `*` is neither.
/// The delimited arg-list fails when it encounters `*` after `(` → QueryParseFailed.
///
/// The aggregate gate's HAVING-redirect message applies only to parseable forms
/// like `count() = 5` / `count(col) = 5`; `count(*)` fails before the gate is reached.
///
/// GREEN on arrival: `fn_call_arg` never admitted `*`; shared parser = shared restriction.
/// If this test FAILS, `fn_call_arg` has been extended to admit `*` in the INSERT path —
/// that is a REAL DEFECT requiring architectural adjudication.
///
/// Traces to: BC-2.11.004 LOW-007 (star-arg scope limit, INSERT source_select WHERE surface);
///            ADR-048 v1.13 §D.7.6; F-PQLFN-P33-LOW-001.
#[tokio::test]
async fn test_BC_2_11_004_low_007_insert_source_select_where_star_arg_parse_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "INSERT INTO crowdstrike_detections (device_id) SELECT device_id \
             FROM crowdstrike_detections WHERE count(*) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "LOW-007 (INSERT source_select WHERE): `count(*) = 5` in INSERT source_select WHERE \
         must fail to parse (QueryParseFailed / E-QUERY-001). `fn_call_arg` in \
         `fn_call_comparison` admits `literal | field_path` only; `*` is neither \
         (BC-2.11.004 LOW-007 star-arg scope limit, ADR-048 v1.13 §D.7.6, \
         F-PQLFN-P33-LOW-001). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "LOW-007 (INSERT source_select WHERE): must NOT be QueryPlanFailed. Star-arg rejection \
         fires at parse time before the aggregate gate (BC-2.11.004 LOW-007). Got: {result:?}"
    );
}

// ── F-PQLFN-P28-OOS-001: SqlPipe `| sort '<literal>'` parity with pure-pipe ────────────────

/// SqlPipe `| sort '<date-like literal>'` MUST produce the same actionable E-QUERY-001
/// message as pure-pipe `| sort '<date-like literal>'` — BC-2.11.023 §Postconditions D2 parity (ADR-046 D2).
///
/// `SELECT * FROM test_events | sort '2026-06-24'` must fail with E-QUERY-001 containing
/// "field name" or "literal", identical to `FROM test_events | sort '2026-06-24'`.
///
/// # Root cause
/// `parse_sqlpipe_internal` (filter_parser.rs) applied only 2 of 3 pipe-error rewrites
/// (D2 + enrich), omitting `rewrite_temporal_literal_in_pipe_key_position`.  The SqlPipe
/// path therefore returned a generic Chumsky parse error without the analyst-readable
/// guidance (F-PQLFN-P28-OOS-001).
///
/// # Pre-implementation state (Red Gate — F-PQLFN-P28-OOS-001)
/// The message contains neither "field name" nor "literal" — the test FAILS (RED). ✓
///
/// # Post-implementation state
/// After adding `rewrite_temporal_literal_in_pipe_key_position(stages_str, errs)` to the
/// stage-error path in `parse_sqlpipe_internal` (before `shift_parse_error_offsets`),
/// the error message contains "field name" or "literal" for both routes.
///
/// Traces to: BC-2.11.023 §Postconditions D2 (mode-bridge diagnostic parity — ADR-046 D2); ADR-052 §D4 v1.10 option (a); F-PQLFN-P28-OOS-001.
#[tokio::test]
async fn test_f_pqlfn_p28_oos_001_sqlpipe_sort_literal_parity() {
    let engine = make_test_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events | sort '2026-06-24'",
            QueryOptions::default(),
        )
        .await;

    // Must be a parse error (E-QUERY-001).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "SqlPipe sort literal parity (F-PQLFN-P28-OOS-001): must fail with \
         QueryParseFailed (E-QUERY-001). Got: {result:?}"
    );

    // Error message MUST be actionable — contain "field name" or "literal"
    // (BC-2.11.023 §Postconditions D2 parity with pure-pipe `| sort '2026-06-24'` — ADR-046 D2).
    if let Err(PrismError::QueryParseFailed { detail, .. }) = &result {
        assert!(
            detail.contains("field name") || detail.contains("literal"),
            "SqlPipe sort literal parity (F-PQLFN-P28-OOS-001): error message must contain \
             'field name' or 'literal' to match pure-pipe behavior (BC-2.11.023 §Postconditions D2 — ADR-046 D2). \
             Got: {detail:?}"
        );
    }
}

// ── EC-11-085: NULL keyword rejection across all predicate surfaces ───────────────────────
//
// BC-2.11.004 v1.48 LOW-006 (21-keyword list, case-insensitive):
//   NOT, AND, OR, IN, IIN, IEQ, INE, IS, BETWEEN, LIKE, CIDR, MATCHES, HAS, MISSING,
//   CONTAINS, ICONTAINS, STARTSWITH, ISTARTSWITH, ENDSWITH, IENDSWITH, **NULL**
//
// `NULL` is added at v1.48 (DEFECT-PQL-FNCALL-LHS-001 fix-burst-36). The `fn_call_comparison`
// production in `build_predicate_parser` must now reject `NULL(...)` as a function name,
// firing E-QUERY-001 with message fragment:
//   "'NULL' is a PrismQL keyword and cannot be used as a function name"
// (BC-2.11.004 EC-11-085 canonical message, POL-24).
//
// The rejection must cover ALL three predicate surfaces:
//   (A) pipe `| where NULL(x) = 5`
//   (B) SQL WHERE `SELECT * FROM t WHERE NULL(x) = 5`
//   (C) filter mode `"crowdstrike_detections | NULL(x) = 5"` (sensor `|` predicate form)
//
// Pre-fix: `NULL` is NOT in the reserved-keyword blocklist inside `fn_call_comparison`.
//          `NULL(x)` parses successfully as a function call → proceeds to execution →
//          triggers E-QUERY-039 or DataFusion error (or Ok on no data). NOT E-QUERY-001.
//          Tests asserting E-QUERY-001 with the keyword message → FAIL (RED). ✓
//
// Post-fix: `fn_call_comparison` case-insensitively checks the function name against the
//           21-item LOW-006 list (including NULL) → E-QUERY-001 fires at parse time.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// EC-11-085 (A) **RED** — pipe `| where` surface: `NULL(x) = 5` fires E-QUERY-001
/// with keyword message.
///
/// Query: `FROM crowdstrike_detections | where NULL(device_id) = 5`
///
/// The `| where` stage uses `build_predicate_parser` → `fn_call_comparison`. Post-fix, the
/// 21-item LOW-006 keyword blocklist (BC-2.11.004 v1.48) includes NULL (case-insensitive),
/// so `fn_call_comparison` rejects `NULL(device_id)` before building the AST node.
///
/// **Pre-fix failure path** (current code → RED):
/// - `NULL` is NOT in the `fn_call_comparison` keyword blocklist → parses as
///   `ScalarFunc::Unknown("null")`/`ScalarFunc::Unknown("NULL")` → reaches execution →
///   fires E-QUERY-039 or a DataFusion error. NOT QueryParseFailed. Test FAILS (RED). ✓
///
/// **Post-fix path**:
/// - `fn_call_comparison` rejects `NULL` from the LOW-006 list → E-QUERY-001 fires with
///   "'NULL' is a PrismQL keyword and cannot be used as a function name".
///
/// Canonical message (byte-verbatim per POL-24, BC-2.11.004 EC-11-085):
/// `"E-QUERY-001: query parse error at offset {offset}: 'NULL' is a PrismQL keyword and
/// cannot be used as a function name"`
///
/// Traces to: BC-2.11.004 v1.48 EC-11-085 LOW-006; F-PQLFN-PR3-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_085_pipe_null_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where NULL(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed).
    // Pre-fix: NULL not in LOW-006 keyword blocklist → parses successfully → later error.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-085 (pipe | where): `NULL(device_id) = 5` must fire E-QUERY-001 \
         (QueryParseFailed). Pre-fix: NULL is not in the fn_call_comparison keyword blocklist \
         → parses as an unknown function → E-QUERY-039 or DataFusion error. Post-fix: NULL \
         added to LOW-006 21-keyword list → rejected at parse time. \
         (BC-2.11.004 v1.48 EC-11-085, F-PQLFN-PR3-LOW-001, POL-24) \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires before execution.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-085 (pipe | where): must NOT be QueryPlanFailed. Keyword gate fires at \
         parse time, before DataFusion plan. Got: {result:?}"
    );

    // Must NOT be E-QUERY-039 (the pre-fix regression where NULL parsed as an unknown function).
    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-085 (pipe | where): must NOT fire E-QUERY-039 (EnrichUdfNotFound). \
         Pre-fix: NULL(device_id) passes fn_call_comparison → reaches E-QUERY-039 check. \
         Post-fix: blocked before that gate by LOW-006. Got: {result:?}"
    );

    // POL-24 message-text lock — keyword-specific canonical message from BC-2.11.004 EC-11-085.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "EC-11-085 (pipe | where): E-QUERY-001 display must contain keyword message \
         \"is a PrismQL keyword and cannot be used as a function name\" \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NULL'"),
        "EC-11-085 (pipe | where): E-QUERY-001 display must contain quoted keyword name \
         \"'NULL'\" (BC-2.11.004 EC-11-085 canonical message, POL-24). \
         Got: {err_display:?}"
    );
}

/// EC-11-085 (B) **RED** — SQL WHERE surface: `NULL(x) = 5` in SELECT WHERE fires
/// E-QUERY-001 with keyword message.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE NULL(device_id) = 5`
///
/// SQL WHERE uses `build_sql_predicate_parser` (which delegates to `build_predicate_parser`
/// internally). Post-fix, the LOW-006 blocklist applies to this surface identically.
///
/// **Pre-fix failure path** → RED (same analysis as pipe variant above).
///
/// Traces to: BC-2.11.004 v1.48 EC-11-085 LOW-006 surface (b) SQL WHERE; F-PQLFN-PR3-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_085_sql_where_null_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE NULL(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-085 (SQL WHERE): `NULL(device_id) = 5` in SELECT WHERE must fire E-QUERY-001 \
         (QueryParseFailed). Pre-fix: NULL not in LOW-006 blocklist → parses → later error. \
         (BC-2.11.004 v1.48 EC-11-085, F-PQLFN-PR3-LOW-001, POL-24). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-085 (SQL WHERE): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-085 (SQL WHERE): must NOT fire E-QUERY-039. Got: {result:?}"
    );

    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "EC-11-085 (SQL WHERE): keyword message required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NULL'"),
        "EC-11-085 (SQL WHERE): quoted name \"'NULL'\" required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
}

/// EC-11-085 (C) **RED** — filter mode surface: `NULL(x) = 5` as sensor predicate fires
/// E-QUERY-001 with keyword message.
///
/// Query: `"crowdstrike_detections | NULL(device_id) = 5"` (sensor `|` predicate form —
/// filter mode, NOT pipe mode; the `|` separates sensor name from predicate expression).
///
/// Filter mode routes through `parse_filter_predicate` → `build_predicate_parser`.
/// Post-fix, the LOW-006 blocklist covers this surface identically.
///
/// **Pre-fix failure path** → RED (same analysis as pipe and SQL WHERE variants above).
///
/// Traces to: BC-2.11.004 v1.48 EC-11-085 LOW-006 surface (c) filter mode; F-PQLFN-PR3-LOW-001; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_085_filter_mode_null_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | NULL(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-085 (filter mode): `NULL(device_id) = 5` as sensor predicate must fire \
         E-QUERY-001 (QueryParseFailed). Pre-fix: NULL not in LOW-006 blocklist → parses. \
         (BC-2.11.004 v1.48 EC-11-085, F-PQLFN-PR3-LOW-001, POL-24). Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-085 (filter mode): must NOT be QueryPlanFailed. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-085 (filter mode): must NOT fire E-QUERY-039. Got: {result:?}"
    );

    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "EC-11-085 (filter mode): keyword message required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NULL'"),
        "EC-11-085 (filter mode): quoted name \"'NULL'\" required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
}

// ── F-PQLFN-PR3-OBS-001: SqlPipe-head-WHERE position 5 explicit lock ─────────────────────
//
// PR-LEVEL finding F-PQLFN-PR3-OBS-001 (OBS severity, fix-burst-36): the SqlPipe head-SELECT
// WHERE position (position 5 in ADR-048 §D.7 surface map) had no dedicated LOW-006 test
// covering the `SELECT ... WHERE keyword_fn(x) = value | pipe_stage` form.
//
// Position 5 is: `SELECT * FROM <table> WHERE <predicate> | <pipe_stages>`.
// The WHERE predicate in this position routes through `build_sql_predicate_parser` →
// `build_predicate_parser` → `fn_call_comparison`, identical to standalone SQL WHERE.
//
// This GREEN lock test confirms that position 5 fires E-QUERY-001 with the keyword message
// for `NOT(x) = 5` in the SqlPipe head-SELECT WHERE (NOT is one of the original 20 LOW-006
// keywords; NULL is the v1.48 addition). The existing LOW-006 NOT tests cover pipe and SQL
// WHERE but this test specifically covers the SqlPipe-with-trailing-pipe form.
//
// **GREEN on arrival**: `build_sql_predicate_parser` was already exercised for NOT by prior
// tests and `NOT` has been in the blocklist since BC-2.11.004 v1.0. If this test FAILS, the
// SqlPipe head-WHERE path does NOT share `build_predicate_parser` — that is a REAL DEFECT.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR3-OBS-001 **GREEN lock** — SqlPipe head-SELECT WHERE position 5:
/// `SELECT * FROM <table> WHERE NOT(x) = 5 | limit 10` fires E-QUERY-001 keyword message.
///
/// This is the explicit position-5 test requested by F-PQLFN-PR3-OBS-001 (BC-2.11.004 v1.48
/// LOW-006 surface completeness). `NOT` is in the original 20-keyword LOW-006 list (now 21 with NULL, BC-2.11.004 v1.48 EC-11-085); this form
/// adds the trailing `| limit 10` pipe stage to the query, exercising the SqlPipe head-WHERE
/// parse path (not plain SQL WHERE).
///
/// **GREEN on arrival**: the keyword gate in `build_predicate_parser` applies to all surfaces
/// that route through it, including the SqlPipe head-WHERE path. If this test FAILS, the
/// position-5 parser does NOT share `build_predicate_parser` — REAL DEFECT requiring routing
/// investigation.
///
/// Traces to: BC-2.11.004 v1.48 LOW-006 surface (e) SqlPipe-head-WHERE;
///            ADR-048 §D.7 position 5; F-PQLFN-PR3-OBS-001; POL-24.
#[tokio::test]
async fn test_f_pqlfn_pr3_obs_001_sqlpipe_head_where_keyword_not_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE NOT(device_id) = 5 | limit 10",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed).
    // GREEN on arrival: NOT has been in LOW-006 since BC-2.11.004 v1.0.
    // If this FAILS, position-5 SqlPipe head-WHERE is NOT routing through build_predicate_parser.
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-PR3-OBS-001 (SqlPipe head-WHERE position 5): \
         `SELECT * FROM crowdstrike_detections WHERE NOT(device_id) = 5 | limit 10` must fire \
         E-QUERY-001 (QueryParseFailed). NOT has been in LOW-006 since v1.0; this form tests \
         the SqlPipe head-WHERE parse path with trailing `| limit 10` pipe stage. \
         If FAILS: position-5 does NOT route through build_predicate_parser — REAL DEFECT. \
         (BC-2.11.004 v1.48 LOW-006, ADR-048 §D.7 position 5, F-PQLFN-PR3-OBS-001, POL-24) \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-PQLFN-PR3-OBS-001 (SqlPipe head-WHERE position 5): must NOT be QueryPlanFailed. \
         Keyword gate fires at parse time. Got: {result:?}"
    );

    // POL-24 message-text lock — keyword-specific canonical message.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "F-PQLFN-PR3-OBS-001 (SqlPipe head-WHERE position 5): E-QUERY-001 display must \
         contain keyword message \"is a PrismQL keyword and cannot be used as a function name\" \
         (BC-2.11.004 EC-11-085/LOW-006 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'NOT'"),
        "F-PQLFN-PR3-OBS-001 (SqlPipe head-WHERE position 5): E-QUERY-001 display must \
         contain quoted keyword \"'NOT'\" (POL-24). Got: {err_display:?}"
    );
}

// ── F-PQLFN-PR4-LOW-001: SqlPipe-head-HAVING EC-11-086 mirror tests (GREEN LOCK) ─────────
//
// The existing EC-11-086 tests (test_BC_2_11_004_having_percentile_fires_e_query_001_*)
// use plain `Ast::Sql(Select)` form (no pipe-tail stage). The `Ast::SqlPipe` head-HAVING
// arm of `check_enrich_udf_availability` (the `spq.head.having` walk — engine.rs lines
// 2047-2049, `collect_unknown_scalar_offsets_from_predicate` into `having_fncall_names`)
// had NO dedicated test until fix-burst-37.
//
// A SqlPipe query is produced by appending a pipe-stage tail to a SQL SELECT:
//   `SELECT ... GROUP BY ... HAVING ... | limit 10`
// This routes the query through `parse_sqlpipe_internal`, producing
// `Ast::SqlPipe { head: SqlQuery { ..., having: Some(...) }, stages: [Limit(10)] }`.
//
// The `spq.head.having` walk collects `ScalarFunc::Unknown` names from the head's HAVING
// predicate into `having_fncall_names`. The DATAFUSION_BUILTIN_AGGREGATE_NAMES gate then
// fires E-QUERY-001 for `percentile` BEFORE the registry guard — identical to plain SQL.
//
// These tests lock the SqlPipe code path. Reverting the `spq.head.having` walk causes:
//   - with_registry: `percentile` escapes the gate → E-QUERY-039 fires (not QueryParseFailed)
//   - no_registry:   `percentile` escapes the gate → registry-None guard fires → Ok(()) →
//                    DataFusion plan fails → QueryPlanFailed (not QueryParseFailed)
// ─────────────────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR4-LOW-001 (a) **GREEN LOCK** — SqlPipe-head-HAVING EC-11-086 mirror:
/// registry-active variant.
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id
///         HAVING percentile(risk_score, 95) > 5 | limit 10`
///
/// Engine: `make_crowdstrike_engine_with_empty_infusion()` — empty InfusionRegistry (Some, 0 entries).
///
/// The trailing `| limit 10` forces the parser to produce `Ast::SqlPipe` (not `Ast::Sql`).
/// The `spq.head.having` walk in `check_enrich_udf_availability` must intercept `percentile`
/// in the HAVING position of the SqlPipe head query and fire E-QUERY-001 with HAVING-specific
/// guidance — identical to the plain-SQL form (EC-11-086 variant a).
///
/// **Load-bearing:** reverting the `spq.head.having` walk causes pre-fix behavior:
///   - `percentile` NOT intercepted in SqlPipe head HAVING → escapes aggregate gate
///   - Registry active (Some, empty) → `percentile` not registered → E-QUERY-039 fires
///     ("enrichment infusion 'percentile' is not registered")
///   - This test asserts `QueryParseFailed` (E-QUERY-001) → FAILS.
///
/// Traces to: BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004;
///            F-PQLFN-PR4-LOW-001.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_086_sqlpipe_head_having_percentile_fires_e_query_001_with_registry()
{
    let engine = make_crowdstrike_engine_with_empty_infusion(); // registry active (Some, empty)

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING percentile(risk_score, 95) > 5 | limit 10",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed), NOT E-QUERY-039 (EnrichUdfNotFound).
    // Load-bearing: without spq.head.having walk, pre-fix path: `percentile` in SqlPipe
    // HAVING escapes DATAFUSION_BUILTIN_AGGREGATE_NAMES gate → E-QUERY-039 fires
    // ("enrichment infusion 'percentile' is not registered; available: []").
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): \
         `HAVING percentile(risk_score, 95) > 5 | limit 10` must fire E-QUERY-001 \
         (QueryParseFailed). Without spq.head.having walk: E-QUERY-039 fires \
         (false enrichment-registration suggestion). \
         (BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004; \
         F-PQLFN-PR4-LOW-001) Got: {result:?}"
    );

    // Must NOT be E-QUERY-039 — the load-bearing regression check.
    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): must NOT fire E-QUERY-039. \
         `percentile` in SqlPipe head HAVING must be intercepted by the spq.head.having walk \
         BEFORE the infusion-registry lookup. Got: {result:?}"
    );

    // POL-24 message-text lock — HAVING-specific canonical message from ADR-048 §D.2.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains(
            "is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates"
        ),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): E-QUERY-001 display must contain \
         HAVING-specific guidance (ADR-048 §D.2 canonical message, POL-24). \
         Got: {err_display:?}"
    );
    assert!(
        err_display.contains("alias it in SELECT"),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): E-QUERY-001 display must contain \
         alias guidance \"alias it in SELECT\" (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("ADR-048 D.3 OD-2"),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): E-QUERY-001 display must contain \
         ADR citation \"ADR-048 D.3 OD-2\" (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'percentile'"),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): E-QUERY-001 display must contain \
         input-verbatim quoted name \"'percentile'\" (ADR-048 §D.2, POL-24). \
         Got: {err_display:?}"
    );
    // Negative lock: display must NOT contain E-QUERY-039 message fragment.
    assert!(
        !err_display.contains("enrichment infusion"),
        "F-PQLFN-PR4-LOW-001 (a, SqlPipe, registry-active): E-QUERY-001 display must NOT \
         contain E-QUERY-039 fragment 'enrichment infusion' — this is the load-bearing \
         regression check for the spq.head.having walk (pre-fix regression output was \
         \"E-QUERY-039: enrichment infusion 'percentile' is not registered; available: []\"). \
         Got: {err_display:?}"
    );
}

/// F-PQLFN-PR4-LOW-001 (b) **GREEN LOCK** — SqlPipe-head-HAVING EC-11-086 mirror:
/// registry-independence (no registry) variant.
///
/// Same query but with NO infusion registry (registry = None). The SqlPipe form must
/// fire E-QUERY-001 registry-independently — the DATAFUSION_BUILTIN_AGGREGATE_NAMES
/// interception fires BEFORE the `let Some(registry) = registry else { return Ok(()) }` guard.
///
/// Engine: `make_crowdstrike_detections_engine()` — no infusion registry (None).
///
/// **Load-bearing:** reverting the `spq.head.having` walk causes pre-fix behavior:
///   - `percentile` in SqlPipe HAVING NOT intercepted → `let Some(registry) = ...` guard fires
///     (registry is None) → `Ok(())` → query proceeds to DataFusion plan → DataFusion cannot
///     resolve `percentile` → `QueryPlanFailed` (NOT `QueryParseFailed`)
///   - This test asserts `QueryParseFailed` → FAILS.
///
/// Traces to: BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004;
///            F-PQLFN-PR4-LOW-001.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_086_sqlpipe_head_having_percentile_fires_e_query_001_no_registry() {
    let engine = make_crowdstrike_detections_engine(); // no infusion registry (None)

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING percentile(risk_score, 95) > 5 | limit 10",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed) — registry-independent.
    // Without spq.head.having walk: registry-None path → Ok(()) → DataFusion plan fails
    // → QueryPlanFailed. This assertion FAILS (load-bearing). ✓
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): \
         `HAVING percentile(risk_score, 95) > 5 | limit 10` must fire E-QUERY-001 \
         (QueryParseFailed) registry-independently. Without spq.head.having walk: \
         registry=None → Ok(()) → DataFusion plan fails → QueryPlanFailed (NOT E-QUERY-001). \
         (BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; BC-2.11.019 v1.23 §OBS-004; \
         F-PQLFN-PR4-LOW-001) Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — the registry-independent HAVING interception must
    // precede DataFusion even when no registry is wired.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): must NOT be QueryPlanFailed. \
         The spq.head.having walk fires BEFORE the registry-None guard. \
         Got: {result:?}"
    );

    // POL-24 message-text lock — HAVING-specific canonical message from ADR-048 §D.2.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains(
            "is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates"
        ),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): E-QUERY-001 display must contain \
         HAVING-specific guidance (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("alias it in SELECT"),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): must contain \"alias it in SELECT\" \
         (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("ADR-048 D.3 OD-2"),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): must contain \"ADR-048 D.3 OD-2\" \
         (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("'percentile'"),
        "F-PQLFN-PR4-LOW-001 (b, SqlPipe, no-registry): must contain input-verbatim quoted \
         name \"'percentile'\" (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
}

// ── EC-11-085 case-variant locks (probe-6 hardening) ─────────────────────────────────────
//
// The existing EC-11-085 tests (A, B, C) use uppercase `NULL(device_id)` only. The
// `fn_call_comparison` keyword check uses `eq_ignore_ascii_case` (BC-2.11.004 v1.48 LOW-006
// — RESERVED_KEYWORDS list in filter_parser.rs `fn_call_comparison` `.validate()` arm),
// so lowercase `null(device_id)` and mixed-case `Null(device_id)` must also fire E-QUERY-001.
//
// The error message echoes the input name verbatim (`'{func_name}'` in the format string),
// so the quoted-name assertion in each test reflects the original input casing.
//
// These GREEN LOCK tests guard against a regression that would change `eq_ignore_ascii_case`
// to a case-sensitive equality (which would silently allow `null(...)` and `Null(...)` to
// parse as unknown function calls, reaching E-QUERY-039 or DataFusion execution).
// ─────────────────────────────────────────────────────────────────────────────────────────

/// EC-11-085 case-variant (lowercase) **GREEN LOCK** — `null(device_id) = 5` in pipe
/// `| where` fires E-QUERY-001 with keyword message.
///
/// Query: `FROM crowdstrike_detections | where null(device_id) = 5`
///
/// Input: lowercase `null` — `eq_ignore_ascii_case` on the LOW-006 RESERVED_KEYWORDS list
/// matches. Error message echoes input verbatim: quoted name is `'null'` (lowercase).
///
/// Traces to: BC-2.11.004 v1.48 EC-11-085 LOW-006 (case-insensitive `eq_ignore_ascii_case`);
///            BC-2.11.019 v1.23 §OBS-004; F-PQLFN-PR4-LOW-001 probe-6 hardening; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_085_pipe_lowercase_null_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where null(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-085 case-variant (lowercase): `null(device_id) = 5` must fire E-QUERY-001 \
         (QueryParseFailed). `fn_call_comparison` checks keywords case-insensitively \
         (eq_ignore_ascii_case) — lowercase 'null' must match NULL in the LOW-006 list. \
         (BC-2.11.004 v1.48 EC-11-085, BC-2.11.019 v1.23 §OBS-004, POL-24) \
         Got: {result:?}"
    );

    // Must NOT be QueryPlanFailed — keyword gate fires at parse time before execution.
    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-085 case-variant (lowercase): must NOT be QueryPlanFailed. \
         Keyword gate fires at parse time. Got: {result:?}"
    );

    // Must NOT be E-QUERY-039 — keyword rejection blocks before the enrich gate.
    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-085 case-variant (lowercase): must NOT fire E-QUERY-039. Got: {result:?}"
    );

    // POL-24 message-text lock — keyword-specific canonical message from BC-2.11.004 EC-11-085.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "EC-11-085 case-variant (lowercase): keyword message fragment required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
    // Input-verbatim echo: lowercase `null` input → quoted name is `'null'` (lowercase).
    assert!(
        err_display.contains("'null'"),
        "EC-11-085 case-variant (lowercase): error must quote the input-verbatim name \
         \"'null'\" (lowercase). The pre-existing EC-11-085 test uses uppercase 'NULL' for \
         uppercase input; this test confirms input-verbatim echo for lowercase input. \
         (BC-2.11.004 EC-11-085 canonical message, POL-24) Got: {err_display:?}"
    );
}

/// EC-11-085 case-variant (mixed case) **GREEN LOCK** — `Null(device_id) = 5` in pipe
/// `| where` fires E-QUERY-001 with keyword message.
///
/// Query: `FROM crowdstrike_detections | where Null(device_id) = 5`
///
/// Input: mixed-case `Null` — `eq_ignore_ascii_case` on the LOW-006 RESERVED_KEYWORDS list
/// matches. Error message echoes input verbatim: quoted name is `'Null'` (mixed case).
///
/// Traces to: BC-2.11.004 v1.48 EC-11-085 LOW-006 (case-insensitive `eq_ignore_ascii_case`);
///            BC-2.11.019 v1.23 §OBS-004; F-PQLFN-PR4-LOW-001 probe-6 hardening; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_085_pipe_mixedcase_null_as_fn_name_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where Null(device_id) = 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-085 case-variant (mixed): `Null(device_id) = 5` must fire E-QUERY-001 \
         (QueryParseFailed). `fn_call_comparison` keyword check is case-insensitive \
         (eq_ignore_ascii_case) — mixed-case 'Null' must match NULL in the LOW-006 list. \
         (BC-2.11.004 v1.48 EC-11-085, BC-2.11.019 v1.23 §OBS-004, POL-24) \
         Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
        "EC-11-085 case-variant (mixed): must NOT be QueryPlanFailed. \
         Keyword gate fires at parse time. Got: {result:?}"
    );

    assert!(
        !matches!(&result, Err(PrismError::EnrichUdfNotFound(_))),
        "EC-11-085 case-variant (mixed): must NOT fire E-QUERY-039. Got: {result:?}"
    );

    // POL-24 message-text lock — keyword-specific canonical message from BC-2.11.004 EC-11-085.
    let err_display = format!("{}", result.unwrap_err());
    assert!(
        err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
        "EC-11-085 case-variant (mixed): keyword message fragment required \
         (BC-2.11.004 EC-11-085 canonical message, POL-24). Got: {err_display:?}"
    );
    // Input-verbatim echo: mixed-case `Null` input → quoted name is `'Null'`.
    assert!(
        err_display.contains("'Null'"),
        "EC-11-085 case-variant (mixed): error must quote the input-verbatim name \
         \"'Null'\" (mixed case). \
         (BC-2.11.004 EC-11-085 canonical message, POL-24) Got: {err_display:?}"
    );
}

// ── F-PQLFN-PR4-OBS-002: Input-verbatim casing lock for HAVING percentile ────────────────
//
// BC-2.11.019 v1.23 §OBS-004 Convention note (F-PQLFN-PR4-OBS-002):
// The `'{name}'` placeholder in the HAVING-interception canonical message is INPUT-VERBATIM
// (engine.rs: `format!("'{name}' is a PrismQL aggregate function; ...")`).
//
//   Lowercase input `HAVING percentile(x, p) > v` → quoted name is `'percentile'`
//   Uppercase input `HAVING PERCENTILE(x, p) > v` → quoted name is `'PERCENTILE'`
//
// The `{name_upper}` occurrences in the guidance template body (e.g., "PERCENTILE is not
// directly supported ...; SELECT PERCENTILE(field, p)") are always uppercase regardless of
// input casing — those reference the PrismQL keyword form, not the input echo.
//
// The existing EC-11-086 tests use lowercase `percentile` input and check `'percentile'`.
// This test uses uppercase `PERCENTILE` input and checks `'PERCENTILE'`, locking the
// input-verbatim echo for the uppercase form.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR4-OBS-002 **GREEN LOCK** — Input-verbatim casing: `HAVING PERCENTILE(...)`
/// with uppercase input echoes `'PERCENTILE'` (uppercase) in the E-QUERY-001 message.
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id
///         HAVING PERCENTILE(risk_score, 95) > 5` (uppercase PERCENTILE — plain SQL form).
///
/// Per BC-2.11.019 v1.23 §OBS-004 Convention note (F-PQLFN-PR4-OBS-002): the `'{name}'`
/// prefix in the HAVING canonical message reflects the analyst's original input casing.
/// This test asserts that uppercase `PERCENTILE` input produces `'PERCENTILE'` (uppercase),
/// NOT the normalized lowercase form `'percentile'`.
///
/// **Load-bearing:** if the implementation normalizes the name before echo
/// (e.g., `name.to_lowercase()`), the message would quote `'percentile'` for uppercase
/// input → this test FAILS (regression detection).
///
/// Traces to: BC-2.11.019 v1.23 §OBS-004 (input-verbatim convention note, F-PQLFN-PR4-OBS-002);
///            BC-2.11.004 v1.48 EC-11-086; ADR-048 v1.16 §D.2; POL-24.
#[tokio::test]
async fn test_BC_2_11_004_ec_11_086_having_percentile_uppercase_input_verbatim() {
    // registry-independent: fires before the registry-None guard.
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING PERCENTILE(risk_score, 95) > 5",
            QueryOptions::default(),
        )
        .await;

    // Must be E-QUERY-001 (QueryParseFailed) — the DATAFUSION_BUILTIN_AGGREGATE_NAMES
    // interception fires registry-independently (before the registry-None guard).
    assert!(
        matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-PR4-OBS-002: HAVING PERCENTILE(...) > 5 (uppercase input) must fire \
         E-QUERY-001 (QueryParseFailed). DATAFUSION_BUILTIN_AGGREGATE_NAMES interception \
         fires registry-independently (before registry-None guard). \
         (BC-2.11.019 v1.23 §OBS-004, BC-2.11.004 v1.48 EC-11-086) Got: {result:?}"
    );

    let err_display = format!("{}", result.unwrap_err());

    // Input-verbatim lock: uppercase `PERCENTILE` input → quoted prefix must be `'PERCENTILE'`.
    // Engine.rs format: `"'{name}' is a PrismQL aggregate function; ..."` — `{name}` is
    // the raw name from the AST (input casing preserved). Post-fix: `'PERCENTILE'`.
    // If the implementation normalizes to lowercase first, this would be `'percentile'`
    // — regression.
    assert!(
        err_display.contains("'PERCENTILE'"),
        "F-PQLFN-PR4-OBS-002: E-QUERY-001 display must quote the input-verbatim uppercase \
         name \"'PERCENTILE'\" (BC-2.11.019 v1.23 §OBS-004 input-verbatim convention). \
         If the engine normalizes the name before echo, message would quote 'percentile' \
         instead — that is a regression against the input-verbatim contract. \
         Got: {err_display:?}"
    );

    // Standard template fragments (POL-24 message-text lock, ADR-048 §D.2).
    // Note: `{name_upper}` occurrences in the guidance body are always uppercase regardless
    // of input — e.g., "PERCENTILE is not directly supported" uses `{name_upper}` (template),
    // not the input echo.
    assert!(
        err_display.contains(
            "is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates"
        ),
        "F-PQLFN-PR4-OBS-002: E-QUERY-001 display must contain HAVING-specific guidance \
         (ADR-048 §D.2 canonical message, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("alias it in SELECT"),
        "F-PQLFN-PR4-OBS-002: E-QUERY-001 display must contain alias guidance \
         \"alias it in SELECT\" (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
    assert!(
        err_display.contains("ADR-048 D.3 OD-2"),
        "F-PQLFN-PR4-OBS-002: E-QUERY-001 display must contain ADR citation \
         \"ADR-048 D.3 OD-2\" (ADR-048 §D.2, POL-24). Got: {err_display:?}"
    );
}

// ── F-PQLFN-PR9-LOW-001: All-21-keyword rejection lock ───────────────────────────────────
//
// PR-LEVEL pass-9 mutation-reasoning identified that the existing LOW-006 / EC-11-085
// tests cover only four keywords (NOT, CONTAINS, not lowercase, NULL). Deleting ANY
// single entry from the 21-keyword `RESERVED_KEYWORDS` list in `fn_call_comparison`'s
// `.validate()` callback (mutation class M14) would leave that entry's rejection
// untested. This parameterized test covers all 21 entries in a single for-loop
// sweep, killing M14 for every entry.
//
// The pipe surface (`FROM t | where KW(col) = 5`) is used because it exercises the
// shared `build_predicate_parser` → `fn_call_comparison` `.validate()` path.
// All seven parse surfaces share the same `build_predicate_parser`; one surface
// provides complete M14 coverage.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR9-LOW-001 **GREEN LOCK** — All-21-keyword rejection: parameterized for loop
/// over the complete `RESERVED_KEYWORDS` list in `fn_call_comparison`.
///
/// For EACH of the 21 keywords, the query
/// `FROM crowdstrike_detections | where <KW>(device_id) = 5` (pipe surface) must produce
/// `QueryParseFailed` (E-QUERY-001) with the canonical message
/// `"'<KW>' is a PrismQL keyword and cannot be used as a function name"`.
///
/// The `build_predicate_parser` `.validate()` callback is shared by ALL seven parse
/// surfaces (pipe `| where`, SQL WHERE, filter mode, SqlPipe head-WHERE, DML positions —
/// ADR-048 §D.7.2). Testing via the pipe surface exercises the shared
/// `fn_call_comparison` `.validate()` path; any mutation that removes a single entry
/// from the 21-keyword `RESERVED_KEYWORDS` list would fail this test for that entry.
///
/// Length assertion: `RESERVED_KEYWORDS.len() == 21` documents the list size so a future
/// keyword addition forces test review (BC-2.11.004 v1.48 keyword count contract).
///
/// Kills mutation class M14 (removal of any entry from the `RESERVED_KEYWORDS` list in
/// `fn_call_comparison`'s `.validate()` callback) for all 21 entries.
///
/// Traces to: BC-2.11.004 v1.48 LOW-006 (21-keyword list); EC-11-085 (NULL, keyword #21);
///            F-PQLFN-PR9-LOW-001; ADR-048 §D.7.2; POL-24.
#[tokio::test]
async fn test_f_pqlfn_pr9_low_001_all_21_keyword_rejection_lock() {
    let engine = make_crowdstrike_detections_engine();

    // Canonical 21-keyword list from `fn_call_comparison` RESERVED_KEYWORDS
    // (filter_parser.rs, BC-2.11.004 v1.48). Keep in sync with filter_parser.rs.
    // The length assertion below documents the count — adding a keyword without
    // updating this test produces a compile-time list mismatch.
    const RESERVED_KEYWORDS: &[&str] = &[
        "NOT",
        "AND",
        "OR",
        "IN",
        "IIN",
        "IEQ",
        "INE",
        "IS",
        "BETWEEN",
        "LIKE",
        "CIDR",
        "MATCHES",
        "HAS",
        "MISSING",
        "CONTAINS",
        "ICONTAINS",
        "STARTSWITH",
        "ISTARTSWITH",
        "ENDSWITH",
        "IENDSWITH",
        "NULL",
    ];

    // Length assertion: documents list size (BC-2.11.004 v1.48 keyword count contract,
    // EC-11-085 NULL as keyword #21). A future keyword addition must update this count
    // and the RESERVED_KEYWORDS list above.
    assert_eq!(
        RESERVED_KEYWORDS.len(),
        21,
        "F-PQLFN-PR9-LOW-001: RESERVED_KEYWORDS length must be 21 \
         (BC-2.11.004 v1.48 keyword count contract, EC-11-085 NULL as #21). \
         If a new keyword was added, update this test and BC-2.11.004."
    );

    for kw in RESERVED_KEYWORDS {
        let query = format!("FROM crowdstrike_detections | where {kw}(device_id) = 5");

        let result = engine.execute(&query, QueryOptions::default()).await;

        // Primary assertion: must be QueryParseFailed, not Ok or a different error.
        assert!(
            matches!(&result, Err(PrismError::QueryParseFailed { .. })),
            "F-PQLFN-PR9-LOW-001 (keyword '{kw}'): \
             `FROM crowdstrike_detections | where {kw}(device_id) = 5` must produce \
             QueryParseFailed (E-QUERY-001). '{kw}' is a PrismQL reserved keyword; \
             `fn_call_comparison` `.validate()` must emit the keyword-rejection error \
             (BC-2.11.004 v1.48 LOW-006, F-PQLFN-PR9-LOW-001). \
             Kills mutation M14 (removal of '{kw}' from RESERVED_KEYWORDS). \
             Got: {result:?}"
        );

        // Must NOT be QueryPlanFailed — keyword rejection fires at parse time.
        assert!(
            !matches!(&result, Err(PrismError::QueryPlanFailed { .. })),
            "F-PQLFN-PR9-LOW-001 (keyword '{kw}'): must NOT be QueryPlanFailed. \
             Keyword rejection must fire at parse time, before plan-time gates \
             (BC-2.11.004 LOW-006, F-PQLFN-PR9-LOW-001). Got: {result:?}"
        );

        let err = result.unwrap_err();
        let err_display = format!("{err}");

        // POL-24 message-text lock: canonical keyword-rejection message fragment.
        assert!(
            err_display.contains("is a PrismQL keyword and cannot be used as a function name"),
            "F-PQLFN-PR9-LOW-001 (keyword '{kw}'): error message must contain \
             'is a PrismQL keyword and cannot be used as a function name' \
             (BC-2.11.004 LOW-006 canonical message, POL-24 message-text lock). \
             Got: {err_display:?}"
        );

        // POL-24 input-verbatim casing lock: the message must quote the keyword as entered.
        let quoted_kw = format!("'{kw}'");
        assert!(
            err_display.contains(&quoted_kw),
            "F-PQLFN-PR9-LOW-001 (keyword '{kw}'): error message must quote the keyword \
             as {quoted_kw} (input-verbatim casing, BC-2.11.004 LOW-006, POL-24). \
             Got: {err_display:?}"
        );
    }
}

// ── F-PQLFN-PR9-LOW-002: Uppercase/mixed-case aggregate-in-WHERE gate locks ──────────────
//
// PR-LEVEL pass-9 mutation-reasoning identified that all aggregate-in-WHERE tests
// (TM-14, TM-16, TM-17, TM-18 and their siblings) use lowercase function names.
// Deleting `.to_ascii_lowercase()` at engine.rs ~2185 in the `predicate_fncall_names`
// gate (mutation M13) changes the check from
//   `DATAFUSION_BUILTIN_AGGREGATE_NAMES.contains(&name_lower)`
// to
//   `DATAFUSION_BUILTIN_AGGREGATE_NAMES.contains(name)`  (using original case).
// Since the set stores lowercase keys, lowercase inputs still match — the mutation
// survives all existing tests. Uppercase/mixed-case inputs expose the mutation.
//
// Two tests added:
//   (a) SQL WHERE STDDEV (uppercase) — kills M13 for the SQL WHERE surface.
//   (b) Pipe | where Avg (mixed-case) — kills M13 for the pipe surface.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR9-LOW-002 (a) **GREEN LOCK** — Uppercase aggregate in SQL WHERE fires
/// E-QUERY-001 via the `predicate_fncall_names` gate.
///
/// Query: `SELECT * FROM crowdstrike_detections WHERE STDDEV(risk_score) > 5`
///
/// The gate normalizes via `.to_ascii_lowercase()` before checking
/// `DATAFUSION_BUILTIN_AGGREGATE_NAMES`: `STDDEV` → `stddev` → in set → E-QUERY-001.
/// Without `.to_ascii_lowercase()` (mutation M13): `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
/// .contains("STDDEV") is FALSE (set stores lowercase "stddev") → gate does not fire →
/// different error or Ok → this test FAILS → mutation exposed.
///
/// Kills mutation M13 (removal of `.to_ascii_lowercase()` in predicate_fncall_names gate,
/// engine.rs ~2185).
///
/// Traces to: BC-2.11.004 v1.48 (aggregate-gate case sensitivity); F-PQLFN-PR9-LOW-002;
///            ADR-048 v1.2 §D.7.1 TM-16 (SQL WHERE surface); BC-2.11.019 v1.23.
#[tokio::test]
async fn test_f_pqlfn_pr9_low_002_sql_where_stddev_uppercase_fires_aggregate_gate() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE STDDEV(risk_score) > 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): must return QueryParseFailed \
         (E-QUERY-001). STDDEV normalized to stddev via .to_ascii_lowercase() → aggregate \
         gate fires. Kills mutation M13 (.to_ascii_lowercase() removal). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): Display must contain \
         'aggregate function' (ADR-048 D.3 canonical). Got: {display}"
    );

    // Input-verbatim echo: uppercase STDDEV input → error quotes 'STDDEV'.
    assert!(
        display.contains("STDDEV"),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): Display must contain 'STDDEV' \
         (input-verbatim echo in canonical D.3 message, BC-2.11.019 v1.23 §OBS-004). \
         Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): Display must contain 'HAVING' \
         (ADR-048 D.3 use-HAVING guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "F-PQLFN-PR9-LOW-002 (SQL WHERE STDDEV uppercase): must NOT be QueryPlanFailed. \
         Aggregate gate fires before plan-time. Got: {err:?}"
    );
}

/// F-PQLFN-PR9-LOW-002 (b) **GREEN LOCK** — Mixed-case aggregate in pipe `| where`
/// fires E-QUERY-001 via the `predicate_fncall_names` gate.
///
/// Query: `FROM crowdstrike_detections | where Avg(risk_score) > 5`
///
/// The gate normalizes via `.to_ascii_lowercase()` before checking
/// `DATAFUSION_BUILTIN_AGGREGATE_NAMES`: `Avg` → `avg` → in set → E-QUERY-001.
/// Without `.to_ascii_lowercase()` (mutation M13): `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
/// .contains("Avg") is FALSE (set stores lowercase "avg") → gate does not fire →
/// different error or Ok → this test FAILS → mutation exposed.
///
/// All existing aggregate-in-WHERE pipe tests (e.g., TM-01, TM-17, TM-18) use lowercase
/// names; mutation M13 survives them. This test uses mixed-case `Avg` to expose M13.
///
/// Kills mutation M13 (removal of `.to_ascii_lowercase()` in predicate_fncall_names gate,
/// engine.rs ~2185).
///
/// Traces to: BC-2.11.004 v1.48 (aggregate-gate case sensitivity); F-PQLFN-PR9-LOW-002;
///            ADR-048 v1.2 §D.7.1 (pipe | where surface); BC-2.11.019 v1.23.
#[tokio::test]
async fn test_f_pqlfn_pr9_low_002_pipe_where_avg_mixed_case_fires_aggregate_gate() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "FROM crowdstrike_detections | where Avg(risk_score) > 5",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): must return Err. Got Ok."
    );

    let err = result.unwrap_err();
    let display = format!("{err}");

    assert!(
        matches!(&err, PrismError::QueryParseFailed { .. }),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): must return QueryParseFailed \
         (E-QUERY-001). 'Avg' normalized to 'avg' via .to_ascii_lowercase() → aggregate \
         gate fires. Kills mutation M13 (.to_ascii_lowercase() removal). \
         Got: {err:?} (Display: {display})"
    );

    assert!(
        display.contains("aggregate function"),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): Display must contain 'aggregate function' \
         (ADR-048 D.3 canonical). Got: {display}"
    );

    // Input-verbatim echo: mixed-case 'Avg' input → error quotes 'Avg'.
    assert!(
        display.contains("Avg"),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): Display must contain 'Avg' \
         (input-verbatim echo in canonical D.3 message, BC-2.11.019 v1.23 §OBS-004). \
         Got: {display}"
    );

    assert!(
        display.contains("HAVING"),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): Display must contain 'HAVING' \
         (ADR-048 D.3 use-HAVING guidance). Got: {display}"
    );

    assert!(
        !matches!(&err, PrismError::QueryPlanFailed { .. }),
        "F-PQLFN-PR9-LOW-002 (pipe Avg mixed-case): must NOT be QueryPlanFailed. \
         Aggregate gate fires before plan-time. Got: {err:?}"
    );
}
