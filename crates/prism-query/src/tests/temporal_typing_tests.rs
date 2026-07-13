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
    let engine = make_crowdstrike_detections_engine();

    // RED GATE: grammar rejects `lower(device_id)` → QueryParseFailed.
    // POST-FIX: grammar parses OK; 'active' is not date-like → no temporal interception;
    //           query passes to DataFusion (may fail with sensor error, not parse/plan error).
    let result = engine
        .execute(
            "FROM crowdstrike_detections | where lower(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be a parse error — the grammar extension must make this parseable.
    // RED failure: currently returns QueryParseFailed (grammar defect).
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "EC-11-004-006: `lower(device_id) = 'active'` in pipe | where must NOT return \
         QueryParseFailed. Post-fix: grammar extension parses fn-call LHS; 'active' has \
         no temporal intercept; query proceeds to DataFusion. \
         RED failure: grammar still rejects fn-call LHS → QueryParseFailed. \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-042 — 'active' is not in the is_date_like Acceptance Set;
    // no RawTemporalLiteral is emitted; check_temporal_literals does not intercept.
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "EC-11-004-006: `lower(device_id) = 'active'` must NOT return E-QUERY-042. \
         'active' is not date-like → no RawTemporalLiteral → temporal gate passes. \
         Got: {result:?}"
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

/// F-PQLFN-P1-MED-003b (GREEN, SAP-3 e2e lock): filter-mode fn-call LHS with
/// NON-date-like RHS must NOT be rejected.
///
/// Query: `crowdstrike_detections | lower(device_id) = 'active'`
///
/// `'active'` is not in the `is_date_like` Acceptance Set; no `RawTemporalLiteral`
/// is emitted; `check_temporal_literals` returns Ok(()).  The query passes the plan
/// gates and proceeds to DataFusion (may fail with a sensor error, but must NOT
/// produce E-QUERY-042 or E-QUERY-001/QueryParseFailed).
///
/// Traces to: BC-2.11.003 v1.12 EC-11-003-007 (non-date-like passthrough);
///            ADR-052 §D4 v1.10 Option A; SAP-3.
#[tokio::test]
async fn test_BC_2_11_003_ec11_003_007_filter_fncall_lhs_non_date_rhs_not_rejected() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "crowdstrike_detections | lower(device_id) = 'active'",
            QueryOptions::default(),
        )
        .await;

    // Must NOT be a parse error — grammar extension makes fn-call LHS parseable.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "MED-003b: filter-mode lower(device_id) = 'active' must NOT return \
         QueryParseFailed. 'active' is not date-like — temporal gate passes. \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-042 — 'active' is not a date-like literal.
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "MED-003b: filter-mode lower(device_id) = 'active' must NOT return \
         E-QUERY-042. 'active' has no RawTemporalLiteral — arm (4) does not fire. \
         Got: {result:?}"
    );

    // Any other outcome (Ok or a different sensor/execution error) is acceptable.
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
/// E-QUERY-001 with the canonical message:
///   "E-QUERY-001: 'count' is an aggregate function; aggregate fn-calls are not
///    valid in pipe | where (use HAVING for post-aggregation filters, ADR-048 D.3)"
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
/// Plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate fires canonical message:
///   "E-QUERY-001: 'stddev' is an aggregate function; aggregate fn-calls are not
///    valid in pipe | where (use HAVING for post-aggregation filters, ADR-048 D.3)"
/// This message contains "aggregate", "stddev", and "HAVING" → all assertions pass.
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
/// Scope guard analogous to TM-01 (pipe WHERE invariant). ADR-048 D.7 covers all five
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
/// Traces to: ADR-048 v1.2 §D.7.2 TM-08; F-PQLFN-P2-MED-002; BC-2.11.004 v1.33 EC-11-013;
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

    // OBS-002 / POL-24 byte-verbatim lock: assert the complete canonical message template
    // from BC-2.11.004 v1.33 EC-11-013 appears as an exact contiguous substring of Display.
    // One byte-verbatim lock here; other TM tests retain substring checks (defense-in-depth
    // diversity: one byte-verbatim lock + N substring locks per ADR-048 D.7).
    const CANONICAL_AGG_MSG: &str = "E-QUERY-001: 'count' is an aggregate function; \
        aggregate fn-calls are not valid in WHERE/where predicates \
        (use HAVING for post-aggregation filters, ADR-048 D.3)";
    assert!(
        display.contains(CANONICAL_AGG_MSG),
        "TM-08 OBS-002: Display must contain the byte-verbatim canonical template from \
         BC-2.11.004 v1.33 EC-11-013 (POL-24). \
         Expected contiguous substring: {CANONICAL_AGG_MSG:?}. \
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

// ── F-PQLFN-P4-MED-001 HAVING e2e lock ───────────────────────────────────────

/// F-PQLFN-P4-MED-001 HAVING e2e lock: `HAVING percentile(risk_score, 95) > 5`
/// must NOT fire E-QUERY-001 (HAVING is exempt from aggregate-in-predicate gate).
///
/// Query: `SELECT device_id FROM crowdstrike_detections GROUP BY device_id
///         HAVING percentile(risk_score, 95) > 5`
///
/// HAVING is fully exempt from the aggregate-in-predicate gate (ADR-048 D.7.3).
/// "percentile" in HAVING parses as `FuncCall::Scalar(Unknown("percentile"))` — the
/// non-six-name fallthrough path (ADR-048 D.7.3 OD-3 MED-001 permit). HAVING predicates
/// are NOT walked by `predicate_fncall_names`, so the aggregate gate does not fire.
///
/// The manual `names.insert("percentile")` in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` only
/// applies to WHERE/predicate positions. This test confirms it does NOT trigger in HAVING.
///
/// The actual result (DataFusion plan error since "percentile" is not a DataFusion
/// built-in aggregate and HAVING is passed through unmodified) is locked here.
///
/// Traces to: F-PQLFN-P4-MED-001 HAVING e2e lock; ADR-048 v1.3 D.7.3; BC-2.11.016 v1.6.
#[tokio::test]
async fn test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt() {
    let engine = make_crowdstrike_detections_engine();

    let result = engine
        .execute(
            "SELECT device_id FROM crowdstrike_detections \
             GROUP BY device_id HAVING percentile(risk_score, 95) > 5",
            QueryOptions::default(),
        )
        .await;

    // HAVING is exempt from the aggregate-in-predicate gate (ADR-048 D.7.3).
    // The result may be Ok or any non-E-QUERY-001 error (DataFusion plan failure, etc.).
    if let Err(ref e) = result {
        assert!(
            !matches!(e, PrismError::QueryParseFailed { .. }),
            "F-PQLFN-P4-MED-001 HAVING e2e lock: HAVING percentile(risk_score, 95) > 5 \
             must NOT fire E-QUERY-001 (aggregate gate). \
             HAVING is exempt per ADR-048 D.7.3 (MED-001 permit). \
             The manual insert for 'percentile' in DATAFUSION_BUILTIN_AGGREGATE_NAMES \
             only covers WHERE/predicate positions, not HAVING. \
             Got: {e:?}"
        );

        let display = format!("{e}");
        assert!(
            !display.contains("aggregate function"),
            "F-PQLFN-P4-MED-001 HAVING e2e: display must NOT contain 'aggregate function' \
             (E-QUERY-001 aggregate gate message). HAVING is exempt. Got: {display}"
        );
    }
    // If Ok: no assertion needed — HAVING percentile passed through to DataFusion.
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

    // Must NOT be a parse error — `build_predicate_parser` fn_call_comparison admits fn-call LHS.
    assert!(
        !matches!(&result, Err(PrismError::QueryParseFailed { .. })),
        "F-PQLFN-P9-MED-002 (DELETE neg): lower(device_id) = 'active' in DML WHERE must NOT \
         return QueryParseFailed. 'active' is not date-like — temporal gate passes. \
         Got: {result:?}"
    );

    // Must NOT be E-QUERY-042 — 'active' is not a date-like literal.
    assert!(
        !matches!(
            &result,
            Err(PrismError::TemporalLiteralInvalidPosition { .. })
        ),
        "F-PQLFN-P9-MED-002 (DELETE neg): lower(device_id) = 'active' in DML WHERE must NOT \
         return E-QUERY-042. 'active' has no RawTemporalLiteral — arm (4) does not fire. \
         Got: {result:?}"
    );

    // Any other outcome (Ok(vec![]) from the DML no-op path, or a different error) is
    // acceptable.  The DML execution path returns Ok(vec![]) pending S-3.06 wiring.
}
