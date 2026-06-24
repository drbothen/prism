//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Areas A, B, and D.
//!
//! Area A — BC-2.11.020 SqlPipe grammar + FORBID-BOTH invariant (ADR-043).
//! Area B — BC-2.11.021 Temporal grammar (NOW(), INTERVAL, timestamp arithmetic, ADR-044).
//! Area D — BC-2.11.023 Three-mode correctness + mode-bridge diagnostic + D7 shared predicate grammar.
//!
//! All test bodies call `todo!()` — the implementer writes the assertions.
//! Compilation of this file confirms all referenced public types / functions exist.
//!
//! Red Gate tests: 9 (Areas A×3, B×2, D×4).

// ─── Area A: BC-2.11.020 — SQL→Pipe composition (ADR-043) ────────────────────

/// AC-001 / BC-2.11.020 postcondition 1: parse `SELECT * FROM t | where severity = 'high' | limit 10`
/// and assert the resulting `Ast::SqlPipe` has `head.from.source.raw == "t"` and the stages
/// contain a `PipeStage::Where` followed by `PipeStage::Limit(10)`.
#[test]
fn test_bc_2_11_020_sqlpipe_ast_round_trip() {
    todo!(
        "BC-2.11.020 AC-001: parse SQL→Pipe query and assert Ast::SqlPipe shape; \
         implementer writes chumsky grammar rule and assertion"
    )
}

/// AC-002 / BC-2.11.020 FORBID-BOTH invariant (ADR-043 §C, E-QUERY-040):
/// parse `SELECT * FROM t LIMIT 5 | limit 10` and assert the planner returns
/// `PrismError::RedundantRowLimit { sql_limit: 5, pipe_limit: 10 }`.
#[test]
fn test_bc_2_11_020_forbid_both_dual_limit_e_query_040() {
    todo!(
        "BC-2.11.020 AC-002: trigger FORBID-BOTH invariant and assert PrismError::RedundantRowLimit; \
         implementer wires planner gate"
    )
}

/// AC-003 / BC-2.11.020 invariant — pure SQL / pure Pipe modes must continue to parse
/// correctly and must NOT be assigned the SqlPipe variant.
/// Assert: `SELECT * FROM t LIMIT 5` → `Ast::Sql(..)`, not `Ast::SqlPipe(..)`.
#[test]
fn test_bc_2_11_020_pure_modes_unchanged() {
    todo!(
        "BC-2.11.020 AC-003: assert pure SQL and pure Pipe queries are NOT assigned Ast::SqlPipe; \
         implementer verifies parser dispatch"
    )
}

// ─── Area B: BC-2.11.021 — Temporal grammar (ADR-044) ────────────────────────

/// AC-004 / BC-2.11.021 postcondition — parse `NOW()`, `INTERVAL '7 days'`, and
/// `timestamp > NOW() - INTERVAL '7 days'` in Filter, SQL, and Pipe modes.
/// Assert: the resulting `Expr` tree contains `Expr::Now`, `Expr::Interval`,
/// and `Expr::TimestampArithmetic` respectively.
#[test]
fn test_bc_2_11_021_now_interval_parses_all_three_modes() {
    todo!(
        "BC-2.11.021 AC-004: parse temporal expressions in all three modes and assert AST variants; \
         implementer writes chumsky parser extension + planning-time injection"
    )
}

/// AC-005 / BC-2.11.021 — error cases: assert three E-QUERY-001 parse-failure cases:
/// (1) `INTERVAL '7'` (unit missing), (2) `INTERVAL 'days'` (count missing),
/// (3) `NOW() - NOW()` (subtracting two timestamps is undefined).
#[test]
fn test_bc_2_11_021_now_error_cases() {
    todo!(
        "BC-2.11.021 AC-005: assert E-QUERY-001 for malformed INTERVAL and \
         timestamp-minus-timestamp; implementer wires grammar error cases"
    )
}

// ─── Area D: BC-2.11.023 — Three-mode correctness + mode-bridge (ADR-046) ───

/// AC-010 / BC-2.11.023 postcondition (ADR-046 §D1): trigger mode-bridge D1 error by
/// submitting a query that parses in one mode but is routed to another mode, and assert
/// the `StructuredErrorFields.normalized_pql` field is populated with the canonical form.
#[test]
fn test_bc_2_11_023_mode_bridge_d1_sql_pipe_diagnostic() {
    todo!(
        "BC-2.11.023 AC-010: trigger mode-bridge D1 error and assert normalized_pql present \
         in StructuredErrorFields; implementer wires mode-bridge detection"
    )
}

/// AC-011 / BC-2.11.023 postcondition — normalized_pql field on StructuredErrorFields
/// is a `Some(String)` for D1 mode-bridge errors and `None` for all other error types.
#[test]
fn test_bc_2_11_023_normalized_pql_on_mode_bridge_error() {
    todo!(
        "BC-2.11.023 AC-011: assert normalized_pql is Some on D1 error and None on non-D1 errors; \
         implementer wires StructuredErrorFields population"
    )
}

/// AC-012 / BC-2.11.023 invariant — Filter mode executes end-to-end via `QueryEngine::execute`.
/// Calls `test_filter_mode_simple_predicate` and `test_filter_mode_with_source` as sub-cases.
#[test]
fn test_bc_2_11_023_filter_mode_end_to_end_execution() {
    todo!(
        "BC-2.11.023 AC-012: execute Filter mode query end-to-end and assert result shape; \
         implementer uses test_filter_mode_simple_predicate and test_filter_mode_with_source"
    )
}

/// AC-013 / BC-2.11.023 D7 invariant — shared predicate grammar: assert that `severity = 'high'`
/// parses identically in Filter mode, in SQL WHERE, and in Pipe `| where` stage — all
/// produce the same `Predicate` tree.
#[test]
fn test_bc_2_11_023_d7_shared_predicate_grammar() {
    todo!(
        "BC-2.11.023 AC-013: parse shared predicate in Filter, SQL WHERE, and Pipe | where and \
         assert equal Predicate trees; implementer verifies D7 shared-grammar invariant"
    )
}
