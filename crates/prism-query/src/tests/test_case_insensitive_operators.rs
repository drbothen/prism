//! Red Gate tests for S-PRISMQL-CASE-INSENSITIVE-001.
//!
//! Covers RG-001 through RG-018, RG-022, RG-023, RG-024 (21 tests in this file).
//!
//! Red Gate discipline (BC-5.38.001): every test body asserts REAL behavior derived
//! from behavioral contracts and acceptance criteria. Tests fail before implementation
//! because the grammar has no IEQ/IIN/INE keywords and `predicate_to_datafusion_sql` +
//! `PqlNormalizer::normalize_predicate` have `todo!()` stubs for `case_insensitive: true`.
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
// RG-016: AC-020 — IEQ with non-string RHS → E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-016: `predicate_to_datafusion_sql` with `IEQ` and an integer literal RHS
/// must return an error.
///
/// The `lower()` function in DataFusion only accepts string columns; applying it
/// to an integer is a type error. The emitter must reject Integer RHS for
/// `case_insensitive: true` predicates.
///
/// Red Gate: PANICS — `predicate_to_datafusion_sql` hits `todo!()` for
/// `case_insensitive: true` before it can validate the RHS type.
/// Green Gate: PASSES — emitter returns Err when RHS is not a string literal.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IEQ/INE with non-string literal RHS".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001() {
    let pred = Predicate::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::Integer(42))),
        case_insensitive: true,
    };
    let result = predicate_to_datafusion_sql(&pred);
    assert!(
        result.is_err(),
        "RG-016: IEQ with integer literal RHS must return Err; got: {:?}",
        result.ok()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-017: AC-021 — IIN with empty list → E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-017: `predicate_to_datafusion_sql` with `IIN` and an empty values list
/// must return an error.
///
/// Empty `IN (...)` would produce invalid SQL; the emitter must enforce the
/// "IIN requires at least one value" invariant.
///
/// Red Gate: PANICS — `predicate_to_datafusion_sql` hits `todo!()` for
/// `case_insensitive: true` before the empty-list check.
/// Green Gate: PASSES — emitter returns Err for empty list.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IIN with empty membership list";
/// BC-2.11.024 v1.0 invariant "IIN requires at least one value".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001() {
    let pred = Predicate::In {
        field: FieldPath::new(["severity"]),
        values: vec![],
        negated: false,
        case_insensitive: true,
    };
    let result = predicate_to_datafusion_sql(&pred);
    assert!(
        result.is_err(),
        "RG-017: IIN with empty value list must return Err; got: {:?}",
        result.ok()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-018: AC-022 — IEQ on integer column → E-QUERY-002 QueryTypeMismatch
// ─────────────────────────────────────────────────────────────────────────────

/// RG-018: `severity_id IEQ 'high'` against a DataFusion schema where `severity_id`
/// is an integer column fails with E-QUERY-002 (QueryTypeMismatch), because
/// `lower()` is not applicable to non-string columns.
///
/// Red Gate: PANICS — `execute_against_session` calls `predicate_to_datafusion_sql`
/// which hits `todo!()` for `case_insensitive: true`.
/// Green Gate: DataFusion rejects `lower(severity_id)` applied to an INT column →
/// `execute_against_session` returns `Err`.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-002: IEQ/IIN/INE on non-string column".
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::Int64Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // IEQ on severity_id (integer column): lower(severity_id) is invalid in DataFusion
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

    // MemTable: severity_id as Int64 (not a string column)
    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity_id",
        DataType::Int64,
        true,
    )]));
    let severity_id_arr = Arc::new(Int64Array::from(vec![4i64, 2i64, 3i64])) as _;
    let batch =
        RecordBatch::try_new(schema, vec![severity_id_arr]).expect("RG-018: batch must build");

    let ctx = build_session_context(50 * 1024 * 1024).expect("RG-018: context must build");
    register_mem_table(&ctx, "detections", vec![batch]).expect("RG-018: MemTable must register");

    let result =
        execute_against_session(&ctx, "severity_id IEQ 'high'", &ast, HashMap::new()).await;

    assert!(
        result.is_err(),
        "RG-018: IEQ on integer severity_id column must fail (lower() not applicable to INT); \
         got success with rows: {:?}",
        result
            .ok()
            .map(|v| v.iter().map(|b| b.num_rows()).sum::<usize>())
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-022: AC-019 — GROUP BY severity produces at most 7 buckets after normalization
// ─────────────────────────────────────────────────────────────────────────────

/// RG-022: After adapter-boundary normalization, cross-sensor records with
/// `'High'` (CrowdStrike) and `'HIGH'` (Armis-like pre-norm) both normalize
/// to `'High'`, so `GROUP BY severity` yields exactly one `'High'` bucket with
/// 5 rows.
///
/// Red Gate: PANICS — `OcsfEnumMap::normalize_label` hits `todo!()`.
/// Green Gate: PASSES — normalization unifies case variants → 1 bucket.
///
/// Traces to: BC-2.02.013 v1.0 canonical test vector "GROUP BY severity cross-sensor";
/// EC-02-026; ADR-047 §Consequences "GROUP BY correct after normalization".
#[tokio::test]
async fn test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation() {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use prism_ocsf::OcsfEnumMap;

    use crate::materialization::{execute_against_session, register_mem_table};
    use crate::memory::build_session_context;

    // Simulate cross-sensor raw input: CrowdStrike 'High' x3, Armis-like 'HIGH' x2
    let raw_severities: Vec<&str> = vec!["High", "High", "High", "HIGH", "HIGH"];
    let enum_map = OcsfEnumMap::new();

    // Apply adapter-boundary normalization: each raw value → OCSF canonical caption
    let normalized_severities: Vec<&str> = raw_severities
        .iter()
        .map(|v| {
            enum_map
                .normalize_label("severity_id", v)
                .expect("RG-022: all severity values must normalize to canonical form")
        })
        .collect();

    // Build MemTable with normalized values
    let schema = Arc::new(Schema::new(vec![Field::new(
        "severity",
        DataType::Utf8,
        true,
    )]));
    let severity_arr = Arc::new(StringArray::from(normalized_severities)) as _;
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
        "RG-022: after normalization, GROUP BY severity must yield exactly 1 bucket \
         (all 5 rows as 'High'); got {total_buckets} bucket(s) — case fragmentation detected"
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
