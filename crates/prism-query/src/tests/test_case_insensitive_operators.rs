//! Red Gate tests for S-PRISMQL-CASE-INSENSITIVE-001.
//!
//! Covers RG-001 through RG-018, RG-022, RG-023, RG-024 plus LOCAL-pass-2 tests
//! RG-018b, RG-018c, and F-P1-PIPE-TYPECHECK-GAP (24 tests in this file).
//!
//! Red Gate discipline (BC-5.38.001): every test body asserts REAL behavior derived
//! from behavioral contracts and acceptance criteria. Tests fail before implementation
//! because the grammar has no IEQ/IIN/INE keywords and `predicate_to_datafusion_sql` +
//! `PqlNormalizer::normalize_predicate` have `todo!()` stubs for `case_insensitive: true`.
//!
//! ## LOCAL-pass-2 findings (tests added / strengthened in this burst)
//!
//! The adversary identified three additional gaps in the pass-1 implementation at
//! HEAD face9b91:
//!
//! - **F-P1-OPERATOR-HARDCODE**: `execute_against_session` hardcodes `operator: "IEQ"`
//!   in the `QueryTypeMismatch` error regardless of whether the actual operator was
//!   INE or IIN. Tests RG-018 (strengthened) + RG-018b (INE) + RG-018c (IIN).
//!
//! - **AC-022 suggestion**: `PrismError::QueryTypeMismatch` has no `suggested_column`
//!   field, so the Display string does not include the BC-2.11.024 suggestion text.
//!   error-taxonomy v2.18 requires: `"; for label comparison, use the string column
//!   '<sibling>' with IEQ/IIN/INE instead"`. All three RG-018 variants assert this.
//!
//! - **F-P1-PIPE-TYPECHECK-GAP**: the pre-flight CI type check only exists in
//!   `execute_against_session::Ast::Filter`. The `Ast::Pipe` arm has no check and
//!   falls through to DataFusion, producing generic E-QUERY-034 instead of E-QUERY-002.
//!   Test `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_pipe_mode_e_query_002`.
//!
//! Behavioral contracts traced:
//!   BC-2.11.024 v1.0 — PrismQL IEQ/IIN/INE case-insensitive operators
//!   BC-2.11.002 v1.5 — filter-mode parsing (amended)
//!   BC-2.11.004 v1.13 — pipe-mode | where stage (amended)
//!   BC-2.11.018 v1.3 — normalized_pql echo (amended EC-11-057)
//!   BC-2.02.013 v1.0 — adapter-boundary OCSF enum-label normalization (RG-022)

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    // parse_filter and parse_pipe are disallowed-methods in production code; unit tests
    // are the sanctioned direct callers (same pattern as parser_tests.rs).
    clippy::disallowed_methods,
    clippy::panic,
    non_snake_case,
)]

use crate::ast::{
    Ast, CompareOp, Expr, FieldPath, FilterExpr, Literal, PqlNormalizer, Predicate, SourceRef,
};
use crate::filter_parser::{parse_filter, PrismQlParser};
use crate::pipe_sql_emitter::predicate_to_datafusion_sql;

// ─────────────────────────────────────────────────────────────────────────────
// RG-001: AC-001 — IEQ parses to Predicate::Compare { case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-001: `severity IEQ 'high'` parses to `Predicate::Compare { op: Eq, case_insensitive: true }`.
///
/// Red Gate: FAILS — `parse_filter` returns `Err` because the grammar has no IEQ keyword.
/// Green Gate: PASSES once IEQ is added to the Chumsky predicate combinator.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" IEQ row;
/// BC-2.11.002 v1.5 amendment.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true() {
    let result = parse_filter("severity IEQ 'high'");
    assert!(
        result.is_ok(),
        "RG-001: 'severity IEQ \\'high\\'' must parse successfully; got: {:?}",
        result.err()
    );
    let filter = result.unwrap();
    match filter.predicate {
        Predicate::Compare {
            op,
            case_insensitive,
            ref rhs,
            ..
        } => {
            assert_eq!(op, CompareOp::Eq, "RG-001: IEQ must map to CompareOp::Eq");
            assert!(
                case_insensitive,
                "RG-001: IEQ must set case_insensitive=true"
            );
            assert_eq!(
                **rhs,
                Expr::Literal(Literal::String("high".to_owned())),
                "RG-001: RHS must be Literal::String(\"high\")"
            );
        }
        other => panic!("RG-001: expected Predicate::Compare, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-002: AC-002 — IIN parses to Predicate::In { case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-002: `status IIN ('open', 'new')` parses to `Predicate::In { case_insensitive: true }`.
///
/// Red Gate: FAILS — grammar has no IIN keyword.
/// Green Gate: PASSES once IIN is added.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" IIN row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true() {
    let result = parse_filter("status IIN ('open', 'new')");
    assert!(
        result.is_ok(),
        "RG-002: 'status IIN (\\'open\\', \\'new\\')' must parse; got: {:?}",
        result.err()
    );
    let filter = result.unwrap();
    match filter.predicate {
        Predicate::In {
            negated,
            case_insensitive,
            ref values,
            ..
        } => {
            assert!(!negated, "RG-002: IIN must not be negated");
            assert!(
                case_insensitive,
                "RG-002: IIN must set case_insensitive=true"
            );
            assert_eq!(values.len(), 2, "RG-002: IIN list must have 2 values");
            assert_eq!(
                values[0],
                Literal::String("open".to_owned()),
                "RG-002: first value must be 'open'"
            );
            assert_eq!(
                values[1],
                Literal::String("new".to_owned()),
                "RG-002: second value must be 'new'"
            );
        }
        other => panic!("RG-002: expected Predicate::In, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-003: AC-003 — INE parses to Predicate::Compare { op: Ne, case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-003: `severity INE 'informational'` parses to `Predicate::Compare { op: Ne, case_insensitive: true }`.
///
/// Red Gate: FAILS — grammar has no INE keyword.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" INE row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true() {
    let result = parse_filter("severity INE 'informational'");
    assert!(
        result.is_ok(),
        "RG-003: 'severity INE \\'informational\\'' must parse; got: {:?}",
        result.err()
    );
    let filter = result.unwrap();
    match filter.predicate {
        Predicate::Compare {
            op,
            case_insensitive,
            ..
        } => {
            assert_eq!(op, CompareOp::Ne, "RG-003: INE must map to CompareOp::Ne");
            assert!(
                case_insensitive,
                "RG-003: INE must set case_insensitive=true"
            );
        }
        other => panic!("RG-003: expected Predicate::Compare, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-004: AC-004 — IEQ keyword parsed case-insensitively (ieq/IEQ/Ieq identical)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-004: `severity ieq 'high'`, `severity IEQ 'high'`, `severity Ieq 'high'` produce
/// structurally identical ASTs with `case_insensitive: true`.
///
/// Red Gate: FAILS — none of the three forms parse (grammar missing IEQ keyword).
/// Green Gate: all three parse to the same `Predicate::Compare { case_insensitive: true }`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "Operators parsed case-insensitively via kw()".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing() {
    let lower = parse_filter("severity ieq 'high'");
    let upper = parse_filter("severity IEQ 'high'");
    let mixed = parse_filter("severity Ieq 'high'");

    assert!(
        lower.is_ok(),
        "RG-004: lowercase 'ieq' must parse; got: {:?}",
        lower.err()
    );
    assert!(
        upper.is_ok(),
        "RG-004: uppercase 'IEQ' must parse; got: {:?}",
        upper.err()
    );
    assert!(
        mixed.is_ok(),
        "RG-004: mixed 'Ieq' must parse; got: {:?}",
        mixed.err()
    );

    let lower_pred = lower.unwrap().predicate;
    let upper_pred = upper.unwrap().predicate;
    let mixed_pred = mixed.unwrap().predicate;

    assert_eq!(
        lower_pred, upper_pred,
        "RG-004: ieq and IEQ must produce identical predicates"
    );
    assert_eq!(
        upper_pred, mixed_pred,
        "RG-004: IEQ and Ieq must produce identical predicates"
    );

    // All three must have case_insensitive: true
    match lower_pred {
        Predicate::Compare {
            case_insensitive, ..
        } => {
            assert!(
                case_insensitive,
                "RG-004: all IEQ variants must set case_insensitive=true"
            );
        }
        other => panic!("RG-004: expected Compare, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-005: AC-005 — IIN parses before IN — no prefix-match collision
// ─────────────────────────────────────────────────────────────────────────────

/// RG-005: `status IIN ('open')` parses with `case_insensitive: true` — NOT
/// `case_insensitive: false`, which would indicate IIN was consumed as bare IN.
///
/// Red Gate: FAILS — grammar has no IIN keyword.
/// Green Gate: PASSES once IIN is added before IN in the combinator ordering.
///
/// Traces to: BC-2.11.024 v1.0 invariant "IIN requires at least one value";
/// risk_mitigation: IIN-before-IN combinator ordering.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision() {
    let result = parse_filter("status IIN ('open')");
    assert!(
        result.is_ok(),
        "RG-005: 'status IIN (\\'open\\')' must parse without error; got: {:?}",
        result.err()
    );
    let filter = result.unwrap();
    match filter.predicate {
        Predicate::In {
            case_insensitive: true,
            negated: false,
            ref values,
            ..
        } => {
            assert_eq!(
                values.len(),
                1,
                "RG-005: single-element IIN list must have 1 value"
            );
            assert_eq!(
                values[0],
                Literal::String("open".to_owned()),
                "RG-005: value must be 'open'"
            );
        }
        Predicate::In {
            case_insensitive: false,
            ..
        } => panic!(
            "RG-005: IIN was parsed as case-sensitive IN — IIN-before-IN combinator ordering violated"
        ),
        other => panic!(
            "RG-005: expected Predicate::In {{ case_insensitive: true }}, got {:?}",
            other
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-006: AC-008 — IEQ emits lower(field) = lower('val')
// ─────────────────────────────────────────────────────────────────────────────

/// RG-006: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Eq, case_insensitive: true }`
/// emits `lower(severity) = lower('high')`.
///
/// Red Gate: PANICS — `predicate_to_datafusion_sql` hits `todo!()` for `case_insensitive: true`.
/// Green Gate: PASSES once the emitter lowers IEQ to `lower()` pattern.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" IEQ row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower() {
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: true,
    };
    let sql = predicate_to_datafusion_sql(&pred).expect("RG-006: IEQ emitter must not return Err");
    assert_eq!(
        sql, "lower(severity) = lower('high')",
        "RG-006: IEQ must emit lower(field) = lower('val') pattern"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-007: AC-009 — INE emits lower(field) != lower('val')
// ─────────────────────────────────────────────────────────────────────────────

/// RG-007: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Ne, case_insensitive: true }`
/// emits `lower(severity) != lower('low')`.
///
/// Red Gate: PANICS — hits `todo!()` for `case_insensitive: true`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" INE row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower() {
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Ne,
        rhs: Box::new(Expr::Literal(Literal::String("low".to_owned()))),
        case_insensitive: true,
    };
    let sql = predicate_to_datafusion_sql(&pred).expect("RG-007: INE emitter must not return Err");
    assert_eq!(
        sql, "lower(severity) != lower('low')",
        "RG-007: INE must emit lower(field) != lower('val') pattern"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-008: AC-010 — IIN emits lower(field) IN (lower('v1'), lower('v2'), ...)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-008: `predicate_to_datafusion_sql` for `Predicate::In { case_insensitive: true, values: ["high", "critical"] }`
/// emits `lower(severity) IN (lower('high'), lower('critical'))`.
///
/// Red Gate: PANICS — hits `todo!()` for `case_insensitive: true` in the `In` arm.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" IIN row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list() {
    let pred = Predicate::In {
        field: FieldPath::new(["severity"]),
        values: vec![
            Literal::String("high".to_owned()),
            Literal::String("critical".to_owned()),
        ],
        negated: false,
        case_insensitive: true,
    };
    let sql = predicate_to_datafusion_sql(&pred).expect("RG-008: IIN emitter must not return Err");
    assert_eq!(
        sql, "lower(severity) IN (lower('high'), lower('critical'))",
        "RG-008: IIN must emit lower(field) IN (lower('v1'), lower('v2')) pattern"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-009: AC-011 — case-sensitive = emits unchanged (no lower() wrapping)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-009: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Eq, case_insensitive: false }`
/// emits `severity = 'High'` with NO `lower()` wrapping.
///
/// Red Gate: PASSES — the `case_insensitive: false` branch has no `todo!()`.
/// This is a regression guard: if the `false` branch is accidentally broken, this fails.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "Case-sensitive operators unchanged".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping() {
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("High".to_owned()))),
        case_insensitive: false,
    };
    let sql = predicate_to_datafusion_sql(&pred)
        .expect("RG-009: case-sensitive = must emit SQL without error");
    assert_eq!(
        sql, "severity = 'High'",
        "RG-009: case-sensitive = must NOT wrap with lower()"
    );
    assert!(
        !sql.contains("lower"),
        "RG-009: case-sensitive = output must not contain lower(); got: {sql:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-010: AC-012 — IEQ execution matches rows regardless of casing
// ─────────────────────────────────────────────────────────────────────────────

/// RG-010: DataFusion MemTable with `{severity: 'High', 'Low', 'Medium'}`;
/// `severity IEQ 'high'` matches the 'High' row (returns 1 row), and
/// `severity IEQ 'HIGH'` also matches the 'High' row.
///
/// Red Gate: PANICS — `execute_against_session` calls `predicate_to_datafusion_sql`
/// which hits `todo!()` for the IEQ `case_insensitive: true` predicate.
/// Green Gate: both IEQ queries return exactly 1 row.
///
/// Traces to: BC-2.11.024 v1.0 canonical test vectors #1 and #2.
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Build the IEQ predicate AST directly: severity IEQ 'high'
    let ieq_pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: true,
    };
    let ast_ieq_lower = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: ieq_pred,
    });

    // Also test IEQ 'HIGH' (all-caps query, Title-case data)
    let ieq_pred_upper = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("HIGH".to_owned()))),
        case_insensitive: true,
    };
    let ast_ieq_upper = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: ieq_pred_upper,
    });

    // MemTable: 3 rows with severity values 'High', 'Low', 'Medium'
    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity",
        DataType::Utf8,
        true,
    )]));
    let severity_arr = Arc::new(StringArray::from(vec!["High", "Low", "Medium"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_arr]).expect("RG-010: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-010: context must build");
    register_mem_table(&ctx, "detections", vec![batch.clone()])
        .expect("RG-010: MemTable must register");

    // IEQ 'high' must match 'High' row
    let result_lower =
        execute_against_session(&ctx, "severity IEQ 'high'", &ast_ieq_lower, HashMap::new())
            .await
            .expect("RG-010: IEQ 'high' execution must not error");

    let rows_lower: usize = result_lower.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        rows_lower, 1,
        "RG-010: severity IEQ 'high' must match the 'High' row; got {rows_lower} rows"
    );

    // Re-register (context is consumed per execution in filter fan-out).
    // Build a new context for the second assertion.
    let ctx2 = build_session_context(50 * 1024 * 1024).expect("RG-010: context2 must build");
    register_mem_table(&ctx2, "detections", vec![batch])
        .expect("RG-010: MemTable must register for ctx2");

    // IEQ 'HIGH' must also match 'High' row
    let result_upper =
        execute_against_session(&ctx2, "severity IEQ 'HIGH'", &ast_ieq_upper, HashMap::new())
            .await
            .expect("RG-010: IEQ 'HIGH' execution must not error");

    let rows_upper: usize = result_upper.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        rows_upper, 1,
        "RG-010: severity IEQ 'HIGH' must also match the 'High' row; got {rows_upper} rows"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-011: AC-013 — case-sensitive = returns 0 rows when casing differs
// ─────────────────────────────────────────────────────────────────────────────

/// RG-011: DataFusion MemTable with `{severity: 'High'}`;
/// `severity = 'high'` (case-sensitive) returns 0 rows.
///
/// Red Gate: PASSES — `case_insensitive: false` has no `todo!()`. This is a
/// regression guard: the existing `=` operator must remain case-sensitive.
///
/// Traces to: BC-2.11.024 v1.0 canonical test vector #6 "regression-no-change".
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Case-sensitive = predicate: severity = 'high' (queries for lowercase, data is Title-case)
    let cs_pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: false,
    };
    let ast = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: cs_pred,
    });

    // MemTable: severity = 'High' (Title-case, OCSF canonical)
    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity",
        DataType::Utf8,
        true,
    )]));
    let severity_arr = Arc::new(StringArray::from(vec!["High"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_arr]).expect("RG-011: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-011: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-011: MemTable must register");

    let result = execute_against_session(&ctx, "severity = 'high'", &ast, HashMap::new())
        .await
        .expect("RG-011: case-sensitive = execution must not error");

    let total: usize = result.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total, 0,
        "RG-011: case-sensitive severity = 'high' must return 0 rows when data is 'High'; \
         got {total} rows (indicates = became case-insensitive — regression)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-012: AC-013b — IEQ available in pipe-mode | where stage
// ─────────────────────────────────────────────────────────────────────────────

/// RG-012: Pipe-mode `crowdstrike_detections | where severity IEQ 'high' | head 5`
/// parses successfully.
///
/// Red Gate: FAILS — `PrismQlParser::parse` returns `Err` because IEQ is not in grammar.
/// Green Gate: PASSES once IEQ is added to the shared predicate combinator used by
/// both filter mode and pipe | where stages.
///
/// Traces to: BC-2.11.004 v1.13 amendment (IEQ/IIN/INE in | where via shared grammar);
/// BC-2.11.024 v1.0 invariant "valid in filter mode and pipe-mode | where stages".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage() {
    let query = "crowdstrike_detections | where severity IEQ 'high' | head 5";
    let result = PrismQlParser::parse(query);
    assert!(
        result.is_ok(),
        "RG-012: pipe-mode 'where severity IEQ \\'high\\'' must parse; got: {:?}",
        result.err()
    );

    // Verify the parsed AST is pipe mode with a Where stage containing IEQ
    match result.unwrap() {
        Ast::Pipe(ref pq) => {
            let has_ieq_where = pq.stages.iter().any(|stage| {
                use crate::ast::PipeStage;
                matches!(
                    stage,
                    PipeStage::Where(Predicate::Compare {
                        case_insensitive: true,
                        ..
                    })
                )
            });
            assert!(
                has_ieq_where,
                "RG-012: pipe AST must contain a Where stage with case_insensitive=true predicate"
            );
        }
        other => panic!("RG-012: expected Ast::Pipe, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-013: AC-014 — normalized_pql reflects IEQ in uppercase canonical form
// ─────────────────────────────────────────────────────────────────────────────

/// RG-013: `severity ieq 'high'` (lowercase keyword) normalizes to a form containing
/// `IEQ` (uppercase canonical) in `normalized_pql`.
///
/// Red Gate: FAILS — `parse_filter("severity ieq 'high'")` returns `Err` (grammar missing).
/// Green Gate: PASSES once grammar + normalizer IEQ branch are implemented.
///
/// Traces to: BC-2.11.018 v1.3 amendment EC-11-057;
/// BC-2.11.024 v1.0 postcondition "normalized_pql round-trip" uppercase invariant.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase() {
    let result = parse_filter("severity ieq 'high'");
    assert!(
        result.is_ok(),
        "RG-013: lowercase 'severity ieq \\'high\\'' must parse; got: {:?}",
        result.err()
    );
    let pred = result.unwrap().predicate;
    let normalized = PqlNormalizer::normalize_predicate_pub(&pred);

    assert!(
        normalized.contains("IEQ"),
        "RG-013: normalized form must contain uppercase 'IEQ'; got: {normalized:?}"
    );
    assert!(
        !normalized.contains("ieq"),
        "RG-013: normalized form must NOT contain lowercase 'ieq'; got: {normalized:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-014: AC-015 — normalized_pql round-trip: parse → normalize → reparse → same AST
// ─────────────────────────────────────────────────────────────────────────────

/// RG-014: `severity IEQ 'high'` → parse → normalize → reparse → AST equals original.
///
/// Red Gate: FAILS — parse step returns Err.
/// Green Gate: PASSES once grammar + normalizer are implemented.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "normalized_pql round-trip" invariant;
/// BC-2.11.018 v1.3 amendment (round-trip extended to IEQ/IIN/INE).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality() {
    let original = parse_filter("severity IEQ 'high'");
    assert!(
        original.is_ok(),
        "RG-014: 'severity IEQ \\'high\\'' must parse; got: {:?}",
        original.err()
    );
    let pred_original = original.unwrap().predicate;

    let normalized = PqlNormalizer::normalize_predicate_pub(&pred_original);
    assert!(
        !normalized.is_empty(),
        "RG-014: normalized form must not be empty"
    );

    let reparsed = parse_filter(&normalized);
    assert!(
        reparsed.is_ok(),
        "RG-014: normalized form must reparse without error; form={normalized:?}, err={:?}",
        reparsed.err()
    );
    let pred_reparsed = reparsed.unwrap().predicate;

    assert_eq!(
        pred_original, pred_reparsed,
        "RG-014: parse → normalize → reparse must yield identical predicate ASTs"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-015: AC-025 — No panic: repeated IEQ does not panic (VP-021 regression guard)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-015: `severity IEQ 'high' AND severity IEQ 'high'` parses without panic.
///
/// Red Gate: FAILS — parse_filter returns Err (IEQ not in grammar).
/// Green Gate: PASSES — parser handles repeated IEQ, returns Ok with a Logical::And predicate.
///
/// Traces to: BC-2.11.024 v1.0 canonical test vector "fuzz-seed regression";
/// VP-021 (parser never panics).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic() {
    let result = parse_filter("severity IEQ 'high' AND severity IEQ 'high'");
    assert!(
        result.is_ok(),
        "RG-015: repeated IEQ in AND must parse without error or panic; got: {:?}",
        result.err()
    );
    // Both halves must have case_insensitive: true
    match result.unwrap().predicate {
        Predicate::Logical { ref predicates, .. } => {
            for p in predicates {
                match p {
                    Predicate::Compare {
                        case_insensitive, ..
                    } => {
                        assert!(
                            case_insensitive,
                            "RG-015: every IEQ in AND must have case_insensitive=true"
                        );
                    }
                    other => panic!("RG-015: expected Compare in AND children, got {:?}", other),
                }
            }
        }
        other => panic!("RG-015: expected Logical::And at root, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-016: AC-020 — IEQ with non-string RHS → parse-time E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-016: parsing `severity IEQ 42` must produce a PARSE-TIME E-QUERY-001 error
/// carrying the verbatim BC-2.11.024 message about string-literal requirement.
///
/// The old test called `predicate_to_datafusion_sql` (the SQL emitter) directly —
/// that function already handles this case (paper-fix). The REAL pipeline gap is at
/// the parser layer: the parser must emit E-QUERY-001 when it sees a non-string RHS
/// after IEQ, so operators produce structured errors before any SQL is generated.
///
/// Red Gate: FAILS — `parse_filter("severity IEQ 42")` returns a generic Chumsky
/// error ("found '4', expected '\"'") without the E-QUERY-001 code or BC message.
/// Green Gate: PASSES once the IEQ parser branch emits ParseError containing
/// "E-QUERY-001" for non-string RHS.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IEQ/INE with non-string RHS";
/// AC-020.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001() {
    // BC-2.11.024: the PARSER must reject integer RHS for IEQ with a structured code.
    let errors = parse_filter("severity IEQ 42").expect_err(
        "RG-016: 'severity IEQ 42' must fail to parse — non-string RHS is invalid for IEQ",
    );
    let all_msgs: String = errors
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "RG-016: parse error for 'severity IEQ 42' must carry 'E-QUERY-001' \
         per BC-2.11.024; parser currently emits a generic Chumsky error without \
         the structured code; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-017: AC-021 — IIN with empty list → parse-time E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-017: parsing `severity IIN ()` must produce a PARSE-TIME E-QUERY-001 error
/// carrying the verbatim BC-2.11.024 message about the membership-list requirement.
///
/// The old test called `predicate_to_datafusion_sql` with an empty `values: vec![]`
/// AST node directly — that emitter check was already implemented (paper-fix). The
/// REAL pipeline gap is at the parser layer: the parser must emit E-QUERY-001 when
/// it sees an empty `()` list after IIN.
///
/// Red Gate: FAILS — `parse_filter("severity IIN ()")` returns a generic Chumsky
/// error ("found ')', expected string literal") without the E-QUERY-001 code.
/// Green Gate: PASSES once the IIN parser branch emits ParseError containing
/// "E-QUERY-001" for an empty membership list.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IIN with empty membership list";
/// AC-021.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001() {
    // BC-2.11.024: the PARSER must reject an empty IIN list with a structured code.
    let errors = parse_filter("severity IIN ()").expect_err(
        "RG-017: 'severity IIN ()' must fail to parse — empty membership list is invalid",
    );
    let all_msgs: String = errors
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "RG-017: parse error for 'severity IIN ()' must carry 'E-QUERY-001' \
         per BC-2.11.024; parser currently emits a generic Chumsky error without \
         the structured code; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-018: AC-022 — IEQ on integer column → E-QUERY-002 QueryTypeMismatch
// ─────────────────────────────────────────────────────────────────────────────

/// RG-018: `severity_id IEQ 'high'` against a schema where `severity_id` is Int64
/// must produce E-QUERY-002 `QueryTypeMismatch`, carrying the offending column name
/// AND suggesting the corresponding string column "severity".
///
/// A generic E-QUERY-034 `QueryExecutionFailed` (DataFusion "lower() type error") is
/// NOT acceptable — BC-2.11.024 requires a structured, schema-aware type-check that
/// fires BEFORE DataFusion execution and names the string-sibling column.
///
/// Red Gate: FAILS — current code emits generic E-QUERY-034 (DataFusion error from
/// `lower(severity_id)` at execution time). The `contains("E-QUERY-002")` assertion
/// fails because the structured pre-flight check is not yet wired.
/// Green Gate: PASSES once the IEQ execution path performs a pre-DataFusion type check
/// and emits `QueryTypeMismatch` with column info and string column suggestion.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-002: IEQ/IIN/INE on non-string column";
/// AC-022.
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Build the AST for "severity_id IEQ 'high'" directly (IEQ parser produces
    // Parse error for non-string RHS after RG-016 is implemented; use the AST
    // path to exercise the execution-layer type check independently).
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity_id"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: true,
    };
    let ast = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: pred,
    });

    // Schema has BOTH the integer column ("severity_id") AND the string sibling
    // ("severity"), so the query engine can introspect and produce a suggestion.
    let schema = Arc::new(Schema::new(vec![
        Field::new("severity_id", DataType::Int64, true),
        Field::new("severity", DataType::Utf8, true),
    ]));
    let severity_id_arr = Arc::new(Int64Array::from(vec![4i64, 2i64, 3i64])) as _;
    let severity_arr = Arc::new(StringArray::from(vec!["High", "Low", "Medium"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_id_arr, severity_arr])
        .expect("RG-018: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-018: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-018: MemTable must register");

    let result =
        execute_against_session(&ctx, "severity_id IEQ 'high'", &ast, HashMap::new()).await;

    // ALL assertions FAIL before implementation (current error is E-QUERY-034):
    let err = result.expect_err(
        "RG-018: severity_id IEQ 'high' on Int64 column must fail with E-QUERY-002; got Ok",
    );
    let err_str = err.to_string();

    // (1) Must be structured E-QUERY-002 QueryTypeMismatch, not E-QUERY-034 DataFusion error
    assert!(
        err_str.contains("E-QUERY-002"),
        "RG-018: must produce E-QUERY-002 (QueryTypeMismatch), \
         not E-QUERY-034 (generic DataFusion execution error); got: {err_str:?}"
    );
    // (2) Must name the offending integer column
    assert!(
        err_str.contains("severity_id"),
        "RG-018: E-QUERY-002 must name the offending column 'severity_id'; got: {err_str:?}",
    );
    // (3) BC-2.11.024 string column suggestion: error must reference "severity" as the
    // corresponding string alternative. The implementer introspects the table schema
    // to find the sibling string column ("severity" when the integer column is "severity_id").
    assert!(
        err_str.contains("severity"),
        "RG-018: E-QUERY-002 must suggest the corresponding string column 'severity'; \
         got: {err_str:?}"
    );
    // (4) error-taxonomy v2.18 AC-022 suggestion text:
    // PrismError::QueryTypeMismatch needs `suggested_column: Option<String>` and the
    // Display (b1 variant, when Some) must append:
    // "; for label comparison, use the string column '<sibling>' with IEQ/IIN/INE instead"
    //
    // Currently FAILS — `QueryTypeMismatch` has no `suggested_column` field and the
    // Display string ends after `'{operator}'` with no suggestion appended.
    assert!(
        err_str.contains(
            "for label comparison, use the string column 'severity' with IEQ/IIN/INE instead"
        ),
        "RG-018: E-QUERY-002 Display must include suggestion per error-taxonomy v2.18: \
         '...for label comparison, use the string column \\'severity\\' with IEQ/IIN/INE instead'; \
         PrismError::QueryTypeMismatch is missing suggested_column: Option<String> \
         and the Display does not append the suggestion; got: {err_str:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-022: AC-019 — GROUP BY severity produces 1 bucket after pipeline normalization
// ─────────────────────────────────────────────────────────────────────────────

/// RG-022: Cross-sensor records with `'High'` (CrowdStrike) and `'HIGH'` (Armis-like)
/// are run through `OcsfNormalizer::normalize_with_mappers` (the BC-2.02.013 v1.1
/// F-CRIT-001 insertion point). After normalization, both become `'High'`, so
/// `GROUP BY severity` yields exactly 1 bucket with 5 rows.
///
/// The old test pre-normalized manually via `OcsfEnumMap::normalize_label` in the
/// test body (paper-fix). The REAL pipeline gap is in `normalize_with_mappers`:
/// it currently returns the DynamicMessage unchanged, so `'HIGH'` stays `'HIGH'`
/// and GROUP BY sees 2 distinct buckets instead of 1.
///
/// Red Gate: FAILS — `normalize_with_mappers` has no normalization wired.
/// The stub mapper sets `severity = "HIGH"` on the DynamicMessage, which is
/// returned unchanged. GROUP BY yields 2 buckets → assertion `== 1` fails.
/// Green Gate: PASSES once `normalize_with_mappers` applies `OcsfEnumMap::normalize_label`
/// to the severity field, converting `"HIGH"` → `"High"` before returning.
///
/// Traces to: BC-2.02.013 v1.1 F-CRIT-001 insertion point; AC-019;
/// EC-02-026; ADR-047 §Consequences "GROUP BY correct after normalization".
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use prism_core::PrismError;
    use prism_ocsf::{OcsfNormalizer, SensorMapper};
    use prost_reflect::{DynamicMessage, ReflectMessage, Value as ProtoValue};
    use serde_json::{Map, Value};

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Stub mapper: transfers "severity" from raw JSON into the DynamicMessage.
    // Simulates what the real CrowdStrike adapter does at the sensor boundary.
    struct SeverityStubMapper;
    impl SensorMapper for SeverityStubMapper {
        fn sensor_id(&self) -> &'static str {
            "crowdstrike"
        }
        fn record_types(&self) -> &'static [&'static str] {
            &["detection"]
        }
        fn map(
            &self,
            _record_type: &str,
            raw: &Value,
            msg: &mut DynamicMessage,
            _extensions: &mut Map<String, Value>,
        ) -> Result<String, PrismError> {
            if let Some(s) = raw.get("severity").and_then(|v| v.as_str()) {
                // Guard against panic if descriptor is missing the field (defensive only;
                // "severity" exists in the real OCSF DetectionFinding descriptor).
                if msg.descriptor().get_field_by_name("severity").is_some() {
                    msg.set_field_by_name("severity", ProtoValue::String(s.to_owned()));
                }
            }
            Ok("stub-id".to_owned())
        }
    }

    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStubMapper)]);

    // Raw cross-sensor input: CrowdStrike 'High' x3, Armis-like 'HIGH' x2.
    // After pipeline normalization both MUST become 'High' (canonical OCSF caption).
    let raws = [
        serde_json::json!({"severity": "High"}),
        serde_json::json!({"severity": "High"}),
        serde_json::json!({"severity": "High"}),
        serde_json::json!({"severity": "HIGH"}),
        serde_json::json!({"severity": "HIGH"}),
    ];

    let mut normalized_severities: Vec<String> = Vec::new();
    for raw in raws {
        let (msg, _) = normalizer
            .normalize_with_mappers("crowdstrike", "detection", raw)
            .expect(
                "RG-022: normalize_with_mappers must not return Err \
                 (OcsfDescriptorNotFound means ocsf-proto-gen is not installed; \
                  this test requires a populated OCSF descriptor pool)",
            );
        // Extract the (potentially normalized) severity field from the DynamicMessage
        let sev = msg
            .get_field_by_name("severity")
            .map(|v| match v.into_owned() {
                ProtoValue::String(s) => s,
                other => format!("<non-string: {other:?}>"),
            })
            .unwrap_or_default();
        normalized_severities.push(sev);
    }

    // Build MemTable from the pipeline output (not from manually pre-normalized values)
    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity",
        DataType::Utf8,
        true,
    )]));
    let sev_refs: Vec<&str> = normalized_severities.iter().map(|s| s.as_str()).collect();
    let severity_arr = Arc::new(StringArray::from(sev_refs)) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_arr]).expect("RG-022: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-022: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-022: MemTable must register");

    let sql = "SELECT severity, count(*) AS cnt FROM detections GROUP BY severity";
    let sql_ast = PrismQlParser::parse(sql).expect("RG-022: GROUP BY query must parse");
    let result = execute_against_session(&ctx, sql, &sql_ast, HashMap::new())
        .await
        .expect("RG-022: GROUP BY must execute");

    let total_buckets: usize = result.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_buckets, 1,
        "RG-022: after pipeline normalization, GROUP BY severity must yield exactly 1 bucket \
         (all 5 rows as 'High'); got {total_buckets} bucket(s) \
         — normalize_with_mappers is not canonicalizing severity casing yet"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-018b: AC-022 operator fidelity — INE sibling
// ─────────────────────────────────────────────────────────────────────────────

/// RG-018b: `severity_id INE 'high'` on an Int64 column → E-QUERY-002 where
/// `operator` = `"INE"` (NOT the hardcoded `"IEQ"` that pass-1 emits).
/// Also asserts the suggestion text from error-taxonomy v2.18.
///
/// Red Gate: FAILS for two reasons at HEAD face9b91:
///   (1) `operator` field hardcoded `"IEQ"` → `err_str.contains("INE")` fails.
///   (2) `suggested_column` not in `QueryTypeMismatch` → suggestion text absent.
///
/// Green Gate: PASSES once the type-check gate correctly propagates the actual
/// operator string and `QueryTypeMismatch.suggested_column` is wired.
///
/// Traces to: error-taxonomy v2.18 `suggested_column` + operator fidelity;
/// BC-2.11.024 v1.0 AC-022; F-P1-OPERATOR-HARDCODE.
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_ine_integer_column_e_query_002_ine_operator() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Predicate: severity_id INE 'high' — CompareOp::Ne + case_insensitive: true
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity_id"]))),
        op: CompareOp::Ne,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: true,
    };
    let ast = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: pred,
    });

    // Schema has severity_id (Int64) + severity (String) so the type check can
    // introspect and produce the "severity" string-column suggestion.
    let schema = Arc::new(Schema::new(vec![
        Field::new("severity_id", DataType::Int64, true),
        Field::new("severity", DataType::Utf8, true),
    ]));
    let severity_id_arr = Arc::new(Int64Array::from(vec![4i64])) as _;
    let severity_arr = Arc::new(StringArray::from(vec!["High"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_id_arr, severity_arr])
        .expect("RG-018b: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-018b: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-018b: MemTable must register");

    let result =
        execute_against_session(&ctx, "severity_id INE 'high'", &ast, HashMap::new()).await;

    let err = result
        .expect_err("RG-018b: severity_id INE 'high' on Int64 must fail with E-QUERY-002; got Ok");
    let err_str = err.to_string();

    // (1) Must be E-QUERY-002
    assert!(
        err_str.contains("E-QUERY-002"),
        "RG-018b: must produce E-QUERY-002 (QueryTypeMismatch) for INE on Int64; got: {err_str:?}"
    );
    // (2) Must name the offending column
    assert!(
        err_str.contains("severity_id"),
        "RG-018b: E-QUERY-002 must name 'severity_id'; got: {err_str:?}"
    );
    // (3) Operator must be INE — not hardcoded IEQ.
    // Currently FAILS: the type-check gate in execute_against_session::Ast::Filter
    // hardcodes `operator: "IEQ"` regardless of the actual Compare op. The error
    // reads `...operator 'IEQ'...` when it should read `...operator 'INE'...`.
    assert!(
        err_str.contains("INE"),
        "RG-018b: E-QUERY-002 operator field must say 'INE' for an INE (Ne+case_insensitive) \
         predicate, not the hardcoded 'IEQ'; got: {err_str:?}"
    );
    // (4) Suggestion text — error-taxonomy v2.18
    // Currently FAILS: QueryTypeMismatch has no suggested_column field.
    assert!(
        err_str.contains(
            "for label comparison, use the string column 'severity' with IEQ/IIN/INE instead"
        ),
        "RG-018b: E-QUERY-002 must include suggestion per error-taxonomy v2.18; got: {err_str:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-018c: AC-022 operator fidelity — IIN sibling
// ─────────────────────────────────────────────────────────────────────────────

/// RG-018c: `severity_id IIN ('high', 'critical')` on an Int64 column → E-QUERY-002
/// where `operator` = `"IIN"` (NOT the hardcoded `"IEQ"`).
/// Also asserts the suggestion text from error-taxonomy v2.18.
///
/// Red Gate: FAILS for two reasons at HEAD face9b91:
///   (1) `operator` field hardcoded `"IEQ"` → `err_str.contains("IIN")` fails.
///   (2) `suggested_column` not in `QueryTypeMismatch` → suggestion text absent.
///
/// Traces to: error-taxonomy v2.18 `suggested_column` + operator fidelity;
/// BC-2.11.024 v1.0 AC-022; F-P1-OPERATOR-HARDCODE.
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_integer_column_e_query_002_iin_operator() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Predicate: severity_id IIN ('high', 'critical') — Predicate::In + case_insensitive: true
    let pred = Predicate::In {
        field: FieldPath::new(["severity_id"]),
        values: vec![
            Literal::String("high".to_owned()),
            Literal::String("critical".to_owned()),
        ],
        negated: false,
        case_insensitive: true,
    };
    let ast = Ast::Filter(FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: pred,
    });

    let schema = Arc::new(Schema::new(vec![
        Field::new("severity_id", DataType::Int64, true),
        Field::new("severity", DataType::Utf8, true),
    ]));
    let severity_id_arr = Arc::new(Int64Array::from(vec![4i64, 5i64])) as _;
    let severity_arr = Arc::new(StringArray::from(vec!["High", "Critical"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_id_arr, severity_arr])
        .expect("RG-018c: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-018c: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-018c: MemTable must register");

    let result = execute_against_session(
        &ctx,
        "severity_id IIN ('high', 'critical')",
        &ast,
        HashMap::new(),
    )
    .await;

    let err = result.expect_err(
        "RG-018c: severity_id IIN ('high','critical') on Int64 must fail with E-QUERY-002; got Ok",
    );
    let err_str = err.to_string();

    // (1) Must be E-QUERY-002
    assert!(
        err_str.contains("E-QUERY-002"),
        "RG-018c: must produce E-QUERY-002 (QueryTypeMismatch) for IIN on Int64; got: {err_str:?}"
    );
    // (2) Must name the offending column
    assert!(
        err_str.contains("severity_id"),
        "RG-018c: E-QUERY-002 must name 'severity_id'; got: {err_str:?}"
    );
    // (3) Operator must be IIN — not hardcoded IEQ.
    // Currently FAILS: hardcoded `operator: "IEQ"` in the QueryTypeMismatch construction.
    assert!(
        err_str.contains("IIN"),
        "RG-018c: E-QUERY-002 operator field must say 'IIN' for an IIN (In+case_insensitive) \
         predicate, not the hardcoded 'IEQ'; got: {err_str:?}"
    );
    // (4) Suggestion text — error-taxonomy v2.18
    // Currently FAILS: QueryTypeMismatch has no suggested_column field.
    assert!(
        err_str.contains(
            "for label comparison, use the string column 'severity' with IEQ/IIN/INE instead"
        ),
        "RG-018c: E-QUERY-002 must include suggestion per error-taxonomy v2.18; got: {err_str:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-P1-PIPE-TYPECHECK-GAP: pipe-mode IEQ on integer column → E-QUERY-002
// ─────────────────────────────────────────────────────────────────────────────

/// F-P1-PIPE-TYPECHECK-GAP: `detections | where severity_id IEQ 'high'` against an
/// Int64 `severity_id` column must return structured E-QUERY-002 (QueryTypeMismatch),
/// NOT the generic E-QUERY-034 (QueryExecutionFailed).
///
/// The pre-flight CI type check exists ONLY in `execute_against_session::Ast::Filter`.
/// The `Ast::Pipe` arm calls `pipe_to_executable_sql` then runs DataFusion SQL directly.
/// DataFusion's `lower(severity_id)` on Int64 fails at planning time and gets mapped to
/// `PrismError::QueryExecutionFailed` (E-QUERY-034), not `QueryTypeMismatch` (E-QUERY-002).
///
/// Red Gate: FAILS — the Ast::Pipe arm has no CI column-type pre-flight check.
/// The error from `execute_against_session` is E-QUERY-034, not E-QUERY-002, so
/// `err_str.contains("E-QUERY-002")` fails.
///
/// Green Gate: PASSES once the same CI type check from the Filter arm is also applied
/// in the Pipe arm (and SqlPipe arm if applicable) before SQL lowering.
///
/// SID-1 compliance: this is an in-process unit test driving the production code path
/// without any external dependencies. No #[ignore] needed — MemTable is fully in-memory.
///
/// Traces to: BC-2.11.024 v1.0 AC-022; BC-2.11.004 v1.13 (pipe | where);
/// F-P1-PIPE-TYPECHECK-GAP (adversary finding, LOCAL pass-2).
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_pipe_mode_e_query_002() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::ast::{PipeQuery, PipeStage};
    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Construct Ast::Pipe directly (grammar does not yet have IEQ, so parser can't be used).
    // This is the same pattern as RG-018 / RG-010 which construct Ast::Filter directly.
    // The test exercises the execution-layer type check independently of the parser.
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity_id"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("high".to_owned()))),
        case_insensitive: true,
    };
    let ast = Ast::Pipe(PipeQuery::new(
        SourceRef::from_raw("detections"),
        vec![PipeStage::Where(pred)],
    ));

    // Schema: severity_id (Int64) + severity (String), same as the filter-mode RG-018 test.
    let schema = Arc::new(Schema::new(vec![
        Field::new("severity_id", DataType::Int64, true),
        Field::new("severity", DataType::Utf8, true),
    ]));
    let severity_id_arr = Arc::new(Int64Array::from(vec![4i64, 2i64, 3i64])) as _;
    let severity_arr = Arc::new(StringArray::from(vec!["High", "Low", "Medium"])) as _;
    let batch = RecordBatch::try_new(schema, vec![severity_id_arr, severity_arr])
        .expect("F-P1-PIPE-TYPECHECK-GAP: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024)
        .expect("F-P1-PIPE-TYPECHECK-GAP: context must build");
    register_mem_table(&ctx, "detections", vec![batch])
        .expect("F-P1-PIPE-TYPECHECK-GAP: MemTable must register");

    let result = execute_against_session(
        &ctx,
        "detections | where severity_id IEQ 'high'",
        &ast,
        HashMap::new(),
    )
    .await;

    // Must return Err — a CI operator on Int64 is always invalid.
    let err = result.expect_err(
        "F-P1-PIPE-TYPECHECK-GAP: severity_id IEQ 'high' on Int64 in pipe mode must return Err",
    );
    let err_str = err.to_string();

    // Must be structured E-QUERY-002 QueryTypeMismatch, NOT generic E-QUERY-034.
    //
    // Currently FAILS: the Ast::Pipe arm of execute_against_session has no CI type check.
    // `pipe_to_executable_sql` emits `SELECT * FROM detections WHERE lower(severity_id) = lower('high')`.
    // DataFusion rejects lower() on Int64 → PrismError::QueryExecutionFailed → E-QUERY-034.
    // The assertion `contains("E-QUERY-002")` fails because the error says E-QUERY-034.
    assert!(
        err_str.contains("E-QUERY-002"),
        "F-P1-PIPE-TYPECHECK-GAP: pipe mode must produce E-QUERY-002 (QueryTypeMismatch) \
         for IEQ on an Int64 column, NOT the generic E-QUERY-034; \
         the Ast::Pipe arm of execute_against_session is missing the CI column-type \
         pre-flight check that exists in the Ast::Filter arm; got: {err_str:?}"
    );
    // Must name the offending column.
    assert!(
        err_str.contains("severity_id"),
        "F-P1-PIPE-TYPECHECK-GAP: E-QUERY-002 must name the offending column 'severity_id'; \
         got: {err_str:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-023: AC-023 — Grammar completeness proxy (IEQ/IIN/INE parseable)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-023: `IEQ`, `IIN`, and `INE` are parseable keywords (grammar completeness proxy).
///
/// NOTE: The authoritative test for the MCP `REFERENCE_EXAMPLES` table is in
/// `crates/prism-mcp/tests/reference_content.rs` — direct access from prism-query
/// tests is blocked by the circular dependency (prism-mcp → prism-query). The
/// implementer must add IEQ/IIN/INE entries to `REFERENCE_EXAMPLES` in
/// `crates/prism-mcp/src/resources.rs` AND extend
/// `crates/prism-mcp/tests/reference_content.rs` to assert their presence.
///
/// Red Gate: FAILS — parse_filter returns Err for all three operators (not in grammar).
/// Green Gate: PASSES — all three operators parse successfully.
///
/// Traces to: BC-2.11.024 v1.0 ADR-047 §D.4 discoverability;
/// BC-2.11.002 v1.5 amendment (IEQ/IIN/INE in operator table).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine() {
    let ieq = parse_filter("severity IEQ 'high'");
    let iin = parse_filter("status IIN ('open', 'new')");
    let ine = parse_filter("severity INE 'informational'");

    assert!(
        ieq.is_ok(),
        "RG-023: IEQ must be a parseable keyword; got: {:?}",
        ieq.err()
    );
    assert!(
        iin.is_ok(),
        "RG-023: IIN must be a parseable keyword; got: {:?}",
        iin.err()
    );
    assert!(
        ine.is_ok(),
        "RG-023: INE must be a parseable keyword; got: {:?}",
        ine.err()
    );

    // All three must produce case_insensitive: true predicates
    match ieq.unwrap().predicate {
        Predicate::Compare {
            case_insensitive, ..
        } => {
            assert!(
                case_insensitive,
                "RG-023: IEQ must have case_insensitive=true"
            );
        }
        other => panic!("RG-023: IEQ expected Compare, got {:?}", other),
    }
    match iin.unwrap().predicate {
        Predicate::In {
            case_insensitive, ..
        } => {
            assert!(
                case_insensitive,
                "RG-023: IIN must have case_insensitive=true"
            );
        }
        other => panic!("RG-023: IIN expected In, got {:?}", other),
    }
    match ine.unwrap().predicate {
        Predicate::Compare {
            case_insensitive, ..
        } => {
            assert!(
                case_insensitive,
                "RG-023: INE must have case_insensitive=true"
            );
        }
        other => panic!("RG-023: INE expected Compare, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-024: AC-024 — Normalized IEQ form is discoverability-quality (describe proxy)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-024: `PqlNormalizer::normalize_predicate_pub` on an IEQ predicate emits
/// the uppercase canonical `IEQ` form, confirming it is documentation-quality for
/// use in `prism describe` output.
///
/// NOTE: The authoritative test for the `prism describe` output and the OCSF casing
/// note lives in `crates/prism-mcp/tests/` — direct access from prism-query tests
/// is blocked by the circular dependency (prism-mcp → prism-query). The implementer
/// must ensure `build_reference_content` and the describe handler include at least
/// one IEQ example and the OCSF Title-case note per ADR-047 §D.4.
///
/// Red Gate: PANICS — `PqlNormalizer::normalize_predicate_pub` hits `todo!()` for
/// `case_insensitive: true`.
/// Green Gate: PASSES — emits form containing uppercase "IEQ".
///
/// Traces to: ADR-047 §D.4; BC-2.11.024 v1.0 "discoverability examples".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example() {
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("High".to_owned()))),
        case_insensitive: true,
    };
    let normalized = PqlNormalizer::normalize_predicate_pub(&pred);
    assert!(
        normalized.contains("IEQ"),
        "RG-024: normalized IEQ predicate must contain uppercase 'IEQ' for \
         discoverability/describe use; got: {normalized:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// LOCAL-pass-3: BC-2.11.024 v1.1 — SQL-mode parse-time rejection of IEQ/IIN/INE
//
// BC-2.11.024 v1.1 contract (product-owner, 2026-07-07):
//   IEQ/IIN/INE are filter+pipe-mode only.  In raw SQL mode (query begins with
//   SELECT) they MUST be rejected at PARSE TIME with a structured E-QUERY-001
//   error — NOT the opaque E-QUERY-034/QueryExecutionFailed that DataFusion
//   currently produces.  Verbatim message:
//
//   E-QUERY-001: parse error near '{operator}': case-insensitive operators
//   (IEQ/IIN/INE) are not supported in SQL mode.  Use filter mode
//   (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM
//   crowdstrike_detections | where severity IEQ 'high') instead.
//
//   ({operator} = the encountered keyword IEQ/IIN/INE, uppercased.)
//
// Current behavior at HEAD 41aa21d9:
//   PrismQlParser::parse("SELECT * FROM t WHERE severity IEQ 'high'") returns
//   Ok(Ast::Sql(...)) — parse succeeds because `build_predicate_parser` (shared
//   between filter mode and the SQL WHERE clause parser) now includes IEQ/IIN/INE.
//   The downstream DataFusion execution then produces E-QUERY-034.
//
// Red Gate for tests 1-3: FAILS at `expect_err(...)` because parse returns Ok.
//   The `expect_err` call panics with the Ast::Sql value, failing the test.
// Green Gate for tests 1-3: PASSES once parse_select_mode (or parse_sql_internal)
//   detects CI operators in the WHERE clause and emits the verbatim E-QUERY-001
//   SQL-mode rejection message before returning.
//
// Red Gate for test 4: PASSES NOW — filter and pipe mode IEQ already work
//   (implemented in pass-1/pass-2).  This is a no-regression lock.
// ═════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_ieq_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SQL-mode IEQ parse-time rejection.
///
/// `SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'`
/// must be rejected at parse time by `PrismQlParser::parse` with:
///   - error code `E-QUERY-001`
///   - guidance substring `"not supported in SQL mode"`
///   - operator name `"IEQ"` in the error message
///
/// Red Gate (HEAD 41aa21d9): FAILS — `PrismQlParser::parse` returns
/// `Ok(Ast::Sql(...))` because `build_predicate_parser` (shared between
/// filter and SQL WHERE clause parsers) accepts IEQ without a SQL-mode guard.
/// `expect_err` panics on the Ok value, failing the test.
///
/// Green Gate: PASSES once `parse_select_mode` (or `parse_sql_internal`) emits
/// the verbatim BC-2.11.024 v1.1 SQL-mode rejection ParseError.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; LOCAL-pass-3.
#[test]
fn test_BC_2_11_024_sql_mode_ieq_rejected() {
    let result =
        PrismQlParser::parse("SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'");
    let err = result.expect_err(
        "BC-2.11.024 v1.1: PrismQlParser::parse must reject IEQ in SQL-mode (SELECT …) \
         at parse time with E-QUERY-001; currently returns Ok(Ast::Sql(…)) — \
         the SQL-mode CI operator parse-time rejection gate is not yet implemented \
         (LOCAL-pass-3 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: SQL-mode IEQ rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: SQL-mode IEQ rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IEQ"),
        "BC-2.11.024 v1.1: SQL-mode IEQ rejection must name the operator 'IEQ' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_iin_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SQL-mode IIN parse-time rejection.
///
/// `SELECT * FROM crowdstrike_detections WHERE severity IIN ('high', 'low')`
/// must be rejected at parse time with E-QUERY-001 naming the operator `"IIN"`.
///
/// Red Gate (HEAD 41aa21d9): FAILS — `PrismQlParser::parse` returns
/// `Ok(Ast::Sql(...))`.  `expect_err` panics on the Ok value.
///
/// Green Gate: PASSES once the SQL-mode CI operator guard is implemented.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; LOCAL-pass-3.
#[test]
fn test_BC_2_11_024_sql_mode_iin_rejected() {
    let result = PrismQlParser::parse(
        "SELECT * FROM crowdstrike_detections WHERE severity IIN ('high', 'low')",
    );
    let err = result.expect_err(
        "BC-2.11.024 v1.1: PrismQlParser::parse must reject IIN in SQL-mode (SELECT …) \
         at parse time with E-QUERY-001; currently returns Ok(Ast::Sql(…)) — \
         the SQL-mode CI operator parse-time rejection gate is not yet implemented \
         (LOCAL-pass-3 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: SQL-mode IIN rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: SQL-mode IIN rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IIN"),
        "BC-2.11.024 v1.1: SQL-mode IIN rejection must name the operator 'IIN' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_ine_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SQL-mode INE parse-time rejection.
///
/// `SELECT * FROM crowdstrike_detections WHERE severity INE 'high'`
/// must be rejected at parse time with E-QUERY-001 naming the operator `"INE"`.
///
/// Red Gate (HEAD 41aa21d9): FAILS — `PrismQlParser::parse` returns
/// `Ok(Ast::Sql(...))`.  `expect_err` panics on the Ok value.
///
/// Green Gate: PASSES once the SQL-mode CI operator guard is implemented.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; LOCAL-pass-3.
#[test]
fn test_BC_2_11_024_sql_mode_ine_rejected() {
    let result =
        PrismQlParser::parse("SELECT * FROM crowdstrike_detections WHERE severity INE 'high'");
    let err = result.expect_err(
        "BC-2.11.024 v1.1: PrismQlParser::parse must reject INE in SQL-mode (SELECT …) \
         at parse time with E-QUERY-001; currently returns Ok(Ast::Sql(…)) — \
         the SQL-mode CI operator parse-time rejection gate is not yet implemented \
         (LOCAL-pass-3 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: SQL-mode INE rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: SQL-mode INE rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("INE"),
        "BC-2.11.024 v1.1: SQL-mode INE rejection must name the operator 'INE' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_filter_and_pipe_ieq_still_execute  (regression guard)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — No-regression lock: filter mode and pipe | where IEQ
/// continue to parse and execute successfully after the SQL-mode rejection gate
/// is added.
///
/// Should PASS NOW (HEAD 41aa21d9): filter-mode and pipe-mode IEQ were fully
/// implemented in LOCAL-pass-1/pass-2.  This test locks in that behaviour so
/// that the pass-3 implementation cannot accidentally break the working modes.
///
/// Assertions:
///   1. `parse_filter("severity IEQ 'high'")` returns `Ok(FilterExpr { … })`.
///   2. Executing `Ast::Filter` against a MemTable {severity: ['High', 'Low']}
///      returns exactly 1 row.
///   3. `PrismQlParser::parse("crowdstrike_detections | where severity IEQ 'high'")`
///      returns `Ok(Ast::Pipe(…))`.
///   4. Executing `Ast::Pipe` against a MemTable {severity: ['High', 'Low']}
///      returns exactly 1 row.
///
/// Traces to: BC-2.11.024 v1.1 §Filter-mode and Pipe-mode; LOCAL-pass-3
/// regression guard; BC-2.11.002 v1.5; BC-2.11.004 v1.13.
#[tokio::test]
async fn test_BC_2_11_024_filter_and_pipe_ieq_still_execute() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // ── 1. Filter mode: severity IEQ 'high' ──────────────────────────────────
    //
    // Parse via `parse_filter` (already in grammar from LOCAL-pass-1).
    let filter_expr = parse_filter("severity IEQ 'high'").expect(
        "regression guard: filter mode 'severity IEQ \\'high\\'' must parse OK; \
         IEQ was implemented in LOCAL-pass-1 — unexpected parse failure \
         indicates a pass-3 regression in the filter grammar",
    );
    let filter_ast = Ast::Filter(filter_expr);

    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity",
        DataType::Utf8,
        true,
    )]));
    let severity_arr = Arc::new(StringArray::from(vec!["High", "Low"])) as _;
    let batch = RecordBatch::try_new(schema.clone(), vec![severity_arr]).expect("batch must build");

    let ctx =
        build_session_context(50 * 1024 * 1024).expect("regression guard: context must build");
    register_mem_table(&ctx, "detections", vec![batch])
        .expect("regression guard: filter MemTable must register");

    let filter_result =
        execute_against_session(&ctx, "severity IEQ 'high'", &filter_ast, HashMap::new())
            .await
            .expect(
                "regression guard: filter mode 'severity IEQ \\'high\\'' must execute \
         successfully — pass-3 must not break the working filter-mode execution path",
            );

    let filter_rows: usize = filter_result.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        filter_rows, 1,
        "regression guard: filter 'severity IEQ \\'high\\'' must match exactly 1 row \
         from {{severity: ['High', 'Low']}}; got {filter_rows} rows"
    );

    // ── 2. Pipe mode: crowdstrike_detections | where severity IEQ 'high' ─────
    //
    // Parse via `PrismQlParser::parse` (pipe mode IEQ works from LOCAL-pass-1).
    let pipe_ast = PrismQlParser::parse("crowdstrike_detections | where severity IEQ 'high'")
        .expect(
            "regression guard: pipe mode 'crowdstrike_detections | where severity IEQ \\'high\\'' \
             must parse OK; IEQ in | where was implemented in LOCAL-pass-1 — \
             unexpected parse failure indicates a pass-3 regression in the pipe grammar",
        );

    let severity_arr2 = Arc::new(StringArray::from(vec!["High", "Low"])) as _;
    let batch2 = RecordBatch::try_new(schema, vec![severity_arr2]).expect("batch2 must build");

    let ctx2 =
        build_session_context(50 * 1024 * 1024).expect("regression guard: context2 must build");
    register_mem_table(&ctx2, "crowdstrike_detections", vec![batch2])
        .expect("regression guard: pipe MemTable must register");

    let pipe_result = execute_against_session(
        &ctx2,
        "crowdstrike_detections | where severity IEQ 'high'",
        &pipe_ast,
        HashMap::new(),
    )
    .await
    .expect(
        "regression guard: pipe 'crowdstrike_detections | where severity IEQ \\'high\\'' \
         must execute successfully — pass-3 must not break the working pipe-mode path",
    );

    let pipe_rows: usize = pipe_result.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        pipe_rows, 1,
        "regression guard: pipe '| where severity IEQ \\'high\\'' must match exactly 1 row \
         from {{severity: ['High', 'Low']}}; got {pipe_rows} rows"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// LOCAL-pass-4: BC-2.11.024 v1.1 — SQL-mode CI-operator rejection walker
//               must cover HAVING and IN-subquery WHERE positions.
//
// F-HIGH-001 (adversary finding, LOCAL pass-3):
//   `detect_ci_operator_in_predicate` in sql_parser.rs is invoked only on
//   `sq.where_` — it is NOT called on `sq.having`, and it has no
//   `Predicate::InSubquery` arm.  As a result:
//
//   a) `HAVING severity IEQ 'high'` bypasses the gate → parse returns Ok →
//      the query reaches DataFusion and produces an opaque E-QUERY-034.
//
//   b) `WHERE col IN (SELECT col FROM t WHERE severity IEQ 'high')` bypasses
//      the gate → the InSubquery arm hits `_ => None` → the inner subquery's
//      CI operator is silently ignored → parse returns Ok → DataFusion
//      receives the malformed plan.
//
// Current behaviour at HEAD 1379b572 for every test in this section:
//   `PrismQlParser::parse(query)` returns `Ok(Ast::Sql(...))` or
//   `Ok(Ast::SqlPipe(...))`.  `expect_err(...)` panics on the Ok value,
//   causing the test to FAIL — which is the intended Red Gate.
//
// Green Gate for tests 1-5: PASSES once `parse_sql_with_limits` (a) also
//   invokes `detect_ci_operator_in_predicate` on `sq.having`, and (b)
//   `detect_ci_operator_in_predicate` gains a `Predicate::InSubquery` arm
//   that recurses into `subquery.where_` (and `subquery.having`).
// ═════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_ieq_in_having_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SQL-mode HAVING clause with IEQ must be rejected at parse time.
///
/// `SELECT severity, count(*) FROM crowdstrike_detections GROUP BY severity
///  HAVING severity IEQ 'high'` must be rejected with:
///   - error code `E-QUERY-001`
///   - guidance substring `"not supported in SQL mode"`
///   - operator name `"IEQ"` in the error message
///
/// Red Gate (HEAD 1379b572): FAILS.
///   `parse_sql_with_limits` calls `detect_ci_operator_in_predicate` only on
///   `sq.where_`.  `sq.having` is never checked.  The query parses to
///   `Ok(Ast::Sql(...))` and `expect_err` panics on the Ok value.
///
/// Green Gate: PASSES once `parse_sql_with_limits` also invokes the walker
///   on `sq.having` after the existing `sq.where_` check.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; F-HIGH-001 (LOCAL-pass-3);
/// LOCAL-pass-4.
#[test]
fn test_BC_2_11_024_sql_mode_ieq_in_having_rejected() {
    let query = "SELECT severity, count(*) FROM crowdstrike_detections \
                 GROUP BY severity HAVING severity IEQ 'high'";
    let result = PrismQlParser::parse(query);
    let err = result.expect_err(
        "BC-2.11.024 v1.1 F-HIGH-001: PrismQlParser::parse must reject IEQ in a SQL-mode \
         HAVING clause with E-QUERY-001; currently returns Ok(Ast::Sql(…)) because \
         parse_sql_with_limits does not invoke detect_ci_operator_in_predicate on sq.having \
         (LOCAL-pass-4 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: HAVING IEQ rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: HAVING IEQ rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IEQ"),
        "BC-2.11.024 v1.1: HAVING IEQ rejection must name the operator 'IEQ' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_ieq_in_subquery_where_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — IEQ inside an IN-subquery WHERE must be rejected at parse time.
///
/// `SELECT severity FROM crowdstrike_detections
///  WHERE severity IN (SELECT severity FROM other_table WHERE severity IEQ 'high')`
/// must be rejected with:
///   - error code `E-QUERY-001`
///   - guidance substring `"not supported in SQL mode"`
///   - operator name `"IEQ"` in the error message
///
/// Red Gate (HEAD 1379b572): FAILS.
///   The outer WHERE predicate is `Predicate::InSubquery { subquery: SqlQuery { ... } }`.
///   `detect_ci_operator_in_predicate` has no `InSubquery` arm — it falls through to
///   `_ => None` and returns `None`.  The inner subquery's CI operator is invisible to
///   the walker.  The query parses to `Ok(Ast::Sql(...))` and `expect_err` panics.
///
/// Green Gate: PASSES once `detect_ci_operator_in_predicate` gains a
///   `Predicate::InSubquery { subquery, .. }` arm that recurses into
///   `subquery.where_` (and `subquery.having`).
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; F-HIGH-001 (LOCAL-pass-3);
/// LOCAL-pass-4.
#[test]
fn test_BC_2_11_024_sql_mode_ieq_in_subquery_where_rejected() {
    let query = "SELECT severity FROM crowdstrike_detections \
                 WHERE severity IN (SELECT severity FROM other_table WHERE severity IEQ 'high')";
    let result = PrismQlParser::parse(query);
    let err = result.expect_err(
        "BC-2.11.024 v1.1 F-HIGH-001: PrismQlParser::parse must reject IEQ inside an \
         IN-subquery WHERE clause with E-QUERY-001; currently returns Ok(Ast::Sql(…)) because \
         detect_ci_operator_in_predicate has no Predicate::InSubquery arm and the inner \
         subquery's CI operator is invisible to the walker (LOCAL-pass-4 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: IN-subquery IEQ rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: IN-subquery IEQ rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IEQ"),
        "BC-2.11.024 v1.1: IN-subquery IEQ rejection must name the operator 'IEQ' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_iin_in_having_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SQL-mode HAVING clause with IIN must be rejected at parse time.
///
/// `SELECT severity, count(*) FROM crowdstrike_detections GROUP BY severity
///  HAVING severity IIN ('high', 'critical')` must be rejected with:
///   - error code `E-QUERY-001`
///   - guidance substring `"not supported in SQL mode"`
///   - operator name `"IIN"` in the error message
///
/// Red Gate (HEAD 1379b572): FAILS.
///   Same root cause as `test_BC_2_11_024_sql_mode_ieq_in_having_rejected`:
///   `sq.having` is never checked.  The query parses to `Ok(Ast::Sql(...))`.
///
/// Green Gate: PASSES once `parse_sql_with_limits` checks `sq.having`.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; F-HIGH-001 (LOCAL-pass-3);
/// LOCAL-pass-4.
#[test]
fn test_BC_2_11_024_sql_mode_iin_in_having_rejected() {
    let query = "SELECT severity, count(*) FROM crowdstrike_detections \
                 GROUP BY severity HAVING severity IIN ('high', 'critical')";
    let result = PrismQlParser::parse(query);
    let err = result.expect_err(
        "BC-2.11.024 v1.1 F-HIGH-001: PrismQlParser::parse must reject IIN in a SQL-mode \
         HAVING clause with E-QUERY-001; currently returns Ok(Ast::Sql(…)) because \
         parse_sql_with_limits does not invoke detect_ci_operator_in_predicate on sq.having \
         (LOCAL-pass-4 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: HAVING IIN rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: HAVING IIN rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IIN"),
        "BC-2.11.024 v1.1: HAVING IIN rejection must name the operator 'IIN' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sqlpipe_head_having_ieq_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — SqlPipe head HAVING clause with IEQ must be rejected at parse time.
///
/// `SELECT severity, count(*) FROM crowdstrike_detections GROUP BY severity
///  HAVING severity IEQ 'high' | limit 10`
/// must be rejected with:
///   - error code `E-QUERY-001`
///   - guidance substring `"not supported in SQL mode"`
///   - operator name `"IEQ"` in the error message
///
/// Reachability: `parse_sqlpipe_internal` (filter_parser.rs) splits on the
/// unquoted `|` before `limit`, passes the SQL head
/// `SELECT severity, count(*) FROM crowdstrike_detections GROUP BY severity
///  HAVING severity IEQ 'high'` through `parse_sql_with_limits`.
/// The HAVING check added by the F-HIGH-001 fix fires here because
/// `parse_sql_with_limits` is shared between pure SQL and SqlPipe SQL heads.
///
/// Red Gate (HEAD 1379b572): FAILS.
///   `parse_sql_with_limits` does not check `sq.having`.  The SQL head parses
///   to `Ok(SqlQuery { having: Some(Predicate::Compare { case_insensitive: true }) })`.
///   The stages parse to `Ok([PipeStage::Limit(10)])`.  The combined result is
///   `Ok(Ast::SqlPipe(...))`.  `expect_err` panics on the Ok value.
///
/// Green Gate: PASSES once `parse_sql_with_limits` checks `sq.having`.
///   The same code path that fixes the pure-SQL HAVING case also fixes this.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; F-HIGH-001 (LOCAL-pass-3);
/// LOCAL-pass-4.
#[test]
fn test_BC_2_11_024_sqlpipe_head_having_ieq_rejected() {
    // `| limit 10` triggers SqlPipe routing via `is_sqlpipe_mode` (PIPE_STAGE_KEYWORDS
    // includes "limit").  The SQL head is everything before the `|`.
    let query = "SELECT severity, count(*) FROM crowdstrike_detections \
                 GROUP BY severity HAVING severity IEQ 'high' | limit 10";
    let result = PrismQlParser::parse(query);
    let err = result.expect_err(
        "BC-2.11.024 v1.1 F-HIGH-001: PrismQlParser::parse must reject IEQ in a SqlPipe \
         SQL-head HAVING clause with E-QUERY-001; currently returns Ok(Ast::SqlPipe(…)) because \
         parse_sql_with_limits does not check sq.having (LOCAL-pass-4 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: SqlPipe HAVING IEQ rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: SqlPipe HAVING IEQ rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IEQ"),
        "BC-2.11.024 v1.1: SqlPipe HAVING IEQ rejection must name the operator 'IEQ' \
         in the error message; got: {all_msgs:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_sql_mode_ieq_in_nested_subquery_rejected
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.024 v1.1 — IEQ inside a doubly-nested IN-subquery WHERE must be rejected.
///
/// `SELECT col FROM t WHERE col IN
///   (SELECT col FROM t2 WHERE col IN
///     (SELECT col FROM t3 WHERE col IEQ 'val'))`
///
/// must be rejected with `E-QUERY-001`.
///
/// This test locks in that `detect_ci_operator_in_predicate` recurses through
/// arbitrary `Predicate::InSubquery` nesting depth, not just one level.
///
/// Red Gate (HEAD 1379b572): FAILS.
///   `detect_ci_operator_in_predicate` has no `Predicate::InSubquery` arm.
///   Both nesting levels are invisible to the walker.  The query parses to
///   `Ok(Ast::Sql(...))`.
///
/// Green Gate: PASSES once `detect_ci_operator_in_predicate` gains an
///   `InSubquery` arm that recurses through `subquery.where_` and
///   `subquery.having` — the recursion naturally handles any nesting depth.
///
/// Traces to: BC-2.11.024 v1.1 §SQL-Mode Rejection; F-HIGH-001 (LOCAL-pass-3);
/// LOCAL-pass-4.
#[test]
fn test_BC_2_11_024_sql_mode_ieq_in_nested_subquery_rejected() {
    // Doubly-nested: outermost WHERE → InSubquery → WHERE → InSubquery → WHERE IEQ
    let query = "SELECT col FROM t \
                 WHERE col IN (SELECT col FROM t2 \
                   WHERE col IN (SELECT col FROM t3 WHERE col IEQ 'val'))";
    let result = PrismQlParser::parse(query);
    let err = result.expect_err(
        "BC-2.11.024 v1.1 F-HIGH-001: PrismQlParser::parse must reject IEQ at any \
         IN-subquery nesting depth with E-QUERY-001; currently returns Ok(Ast::Sql(…)) because \
         detect_ci_operator_in_predicate has no Predicate::InSubquery arm and the recursive \
         CI operator is never reached (LOCAL-pass-4 fix-burst target)",
    );
    let all_msgs: String = err
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        all_msgs.contains("E-QUERY-001"),
        "BC-2.11.024 v1.1: nested-subquery IEQ rejection must carry 'E-QUERY-001'; \
         got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("not supported in SQL mode"),
        "BC-2.11.024 v1.1: nested-subquery IEQ rejection must contain \
         'not supported in SQL mode'; got: {all_msgs:?}"
    );
    assert!(
        all_msgs.contains("IEQ"),
        "BC-2.11.024 v1.1: nested-subquery IEQ rejection must name the operator 'IEQ' \
         in the error message; got: {all_msgs:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// LOCAL-pass-5: F-MED-001 — negated + case_insensitive IN silently drops negation
// ═════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_negated_case_insensitive_in_returns_query_plan_failed
// ─────────────────────────────────────────────────────────────────────────────

/// F-MED-001 (LOCAL pass-5): `predicate_to_datafusion_sql` for
/// `Predicate::In { negated: true, case_insensitive: true }` must return
/// `Err(PrismError::QueryPlanFailed)`.
///
/// It must NOT silently emit a positive `lower(field) IN (...)` SQL fragment by
/// dropping the `negated` flag — that would silently invert query semantics.
///
/// ## The defect at HEAD b2e3892c
///
/// `pipe_sql_emitter.rs` `Predicate::In` arm:
///
/// ```rust
/// if *case_insensitive {
///     // … validates values …
///     return Ok(format!("lower({field_sql}) IN ({})", vals.join(", ")));
/// }
/// ```
///
/// The `if *case_insensitive` branch never inspects `negated`.  A predicate
/// with `negated: true, case_insensitive: true` silently emits a positive
/// `lower(severity) IN (lower('high'))` SQL fragment, returning `Ok(_)`.
///
/// ## Red Gate reason (HEAD b2e3892c)
///
/// `predicate_to_datafusion_sql` returns `Ok("lower(severity) IN (lower('high'))")`.
/// `assert!(result.is_err(), ...)` panics because the result is `Ok`, not `Err`.
///
/// ## Green Gate
///
/// PASSES once the `if *case_insensitive` branch checks `negated` and returns
/// `Err(PrismError::QueryPlanFailed { detail: "..." })` for the combination.
/// This combination has no grammar-reachable parser production (there is no
/// `NIIN` / `NOT IIN` operator in PQL); encountering it in the emitter is an
/// AST invariant violation that warrants a plan-failure error rather than
/// silent semantic inversion.
///
/// ## Traces
///
/// BC-2.11.024 §Invariants (IIN cannot be negated via grammar); LOCAL adversary
/// pass-5 finding F-MED-001.
#[test]
fn test_BC_2_11_024_negated_case_insensitive_in_returns_query_plan_failed() {
    use prism_core::PrismError;

    let pred = Predicate::In {
        field: FieldPath::new(["severity"]),
        values: vec![Literal::String("high".to_owned())],
        negated: true,
        case_insensitive: true,
    };
    let result = predicate_to_datafusion_sql(&pred);
    assert!(
        result.is_err(),
        "F-MED-001: Predicate::In {{ negated: true, case_insensitive: true }} must return \
         Err(QueryPlanFailed) — the negation flag must not be silently dropped to produce \
         a positive-IN SQL fragment; currently returns Ok (silently drops negated=true); \
         got Ok: {:?}",
        result.as_ref().unwrap()
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, PrismError::QueryPlanFailed { .. }),
        "F-MED-001: error variant must be QueryPlanFailed (AST invariant violation: \
         negated+case_insensitive IN has no grammar production), not another variant; \
         got: {:?}",
        err
    );
}
