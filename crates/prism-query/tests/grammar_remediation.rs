//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Areas A, B, and D.
//!
//! Area A — BC-2.11.020 SqlPipe grammar + FORBID-BOTH invariant (ADR-043).
//! Area B — BC-2.11.021 Temporal grammar (NOW(), INTERVAL, timestamp arithmetic, ADR-044).
//! Area D — BC-2.11.023 Three-mode correctness + mode-bridge diagnostic + D7 shared predicate grammar.
//!
//! Red Gate tests: 9 (Areas A×3, B×2, D×4).
//!
//! Every test asserts SPEC behavior against todo!()-body stubs or unimplemented
//! production code paths. All tests MUST FAIL before the implementer writes any code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_imports
)]

use prism_core::error::PrismError;
use prism_query::{
    ast::{Ast, PipeStage, SqlStatement},
    PrismQlParser,
};

// ─── Area A: BC-2.11.020 — SQL→Pipe composition (ADR-043) ────────────────────

/// AC-001 / BC-2.11.020 postcondition 1.
///
/// Parse `SELECT * FROM crowdstrike_detections | enrich threat_score(src_ip) | limit 10`
/// and assert the result is `Ast::SqlPipe(_)` with `stages.len() == 2`.
///
/// Red Gate: `PrismQlParser::parse` does not yet parse SQL→Pipe composition;
/// the query currently parse-errors at `|` (Chumsky sees unexpected `|` in SQL mode).
/// The `assert!(result.is_ok())` fails RED because parse returns Err.
#[test]
fn test_bc_2_11_020_sqlpipe_ast_round_trip() {
    let query = "SELECT * FROM crowdstrike_detections | enrich threat_score(src_ip) | limit 10";
    let ast = PrismQlParser::parse(query)
        .expect("BC-2.11.020: SQL→Pipe query must parse successfully (not return Err)");

    // Assert variant is SqlPipe.
    assert!(
        matches!(ast, Ast::SqlPipe(_)),
        "BC-2.11.020: expected Ast::SqlPipe(_), got {:?}",
        ast
    );

    let Ast::SqlPipe(ref spq) = ast else {
        unreachable!("guarded above")
    };
    assert_eq!(
        spq.stages.len(),
        2,
        "BC-2.11.020: expected 2 stages (enrich + limit), got {}",
        spq.stages.len()
    );
}

/// AC-002 / BC-2.11.020 FORBID-BOTH invariant (ADR-043 §C, E-QUERY-040).
///
/// `SELECT * FROM t LIMIT 5 | enrich fn(x) | limit 3` — both SQL LIMIT and pipe `| limit`
/// present. Assert `Err(PrismError::RedundantRowLimit { sql_limit: 5, pipe_limit: 3 })`.
///
/// Red Gate: `plan_sqlpipe_query` is a `todo!()` stub; calling it panics.
/// The test panics on the `todo!()` inside `plan_sqlpipe_query`, which counts as RED
/// (not a compile error, not a passing test).
#[test]
fn test_bc_2_11_020_forbid_both_dual_limit_e_query_040() {
    // The query must parse first (FORBID-BOTH fires at plan time, not parse time).
    // On develop today, `|` in SQL mode fails to parse — so this test also fails
    // at the `expect` if parse returns Err.  Either failure is RED (not compile error).
    let query = "SELECT * FROM t LIMIT 5 | enrich fn(x) | limit 3";
    let ast = PrismQlParser::parse(query).expect(
        "BC-2.11.020 AC-002: SQL→Pipe query must parse successfully before plan-time check",
    );

    assert!(
        matches!(ast, Ast::SqlPipe(_)),
        "BC-2.11.020 AC-002: expected Ast::SqlPipe(_), got {:?}",
        ast
    );

    let Ast::SqlPipe(ref spq) = ast else {
        unreachable!()
    };

    // Call the plan-time stub — panics on todo!() → RED.
    let plan_result = prism_query::plan_sqlpipe_query(spq);

    match plan_result {
        Err(PrismError::RedundantRowLimit {
            sql_limit,
            pipe_limit,
        }) => {
            assert_eq!(sql_limit, 5, "BC-2.11.020: sql_limit must be 5");
            assert_eq!(pipe_limit, 3, "BC-2.11.020: pipe_limit must be 3");
            // Verify verbatim E-QUERY-040 message (POL-24 — full pedagogical text).
            let err = PrismError::RedundantRowLimit {
                sql_limit,
                pipe_limit,
            };
            let msg = err.to_string();
            assert!(
                msg.contains("E-QUERY-040"),
                "BC-2.11.020: error message must contain 'E-QUERY-040'; got: {msg}"
            );
            assert!(
                msg.contains("PrismQL requires exactly one row cap"),
                "BC-2.11.020 POL-24: error message must contain verbatim \
                 'PrismQL requires exactly one row cap'; got: {msg}"
            );
            assert!(
                msg.contains("place a single `| limit` at the end"),
                "BC-2.11.020 POL-24: error message must contain verbatim \
                 'place a single `| limit` at the end'; got: {msg}"
            );
        }
        other => panic!(
            "BC-2.11.020 AC-002: expected Err(PrismError::RedundantRowLimit {{sql_limit:5, pipe_limit:3}}), got: {:?}",
            other
        ),
    }
}

/// AC-003 / BC-2.11.020 invariant — additive: pure SQL / pure Pipe modes unchanged.
///
/// `SELECT * FROM t LIMIT 5` → `Ast::Sql(_)` (NOT SqlPipe).
/// `FROM t | where severity = 'HIGH'` → `Ast::Pipe(_)` (NOT SqlPipe).
///
/// This test PASSES on develop today (pure modes already work) and is a regression
/// guard for after SqlPipe grammar is added. It is listed as a Red Gate test in the
/// story because incorrect mode-detection in Area A could accidentally route pure SQL
/// to SqlPipe. It will pass both before and after implementation — that is correct
/// per the BC additive invariant. POL-16: does NOT assert on todo!() panics.
#[test]
fn test_bc_2_11_020_pure_modes_unchanged() {
    // Pure SQL: must parse as Ast::Sql, never Ast::SqlPipe.
    let sql = "SELECT * FROM t LIMIT 5";
    let ast = PrismQlParser::parse(sql).expect("BC-2.11.020: pure SQL must parse");
    assert!(
        matches!(ast, Ast::Sql(_)),
        "BC-2.11.020: pure SQL must parse as Ast::Sql, not {:?}",
        ast
    );
    assert!(
        !matches!(ast, Ast::SqlPipe(_)),
        "BC-2.11.020: pure SQL must NOT parse as Ast::SqlPipe"
    );

    // Pure Pipe: must parse as Ast::Pipe, never Ast::SqlPipe.
    let pipe = "FROM t | where severity = 'HIGH'";
    let pipe_ast = PrismQlParser::parse(pipe).expect("BC-2.11.020: pure Pipe must parse");
    assert!(
        matches!(pipe_ast, Ast::Pipe(_)),
        "BC-2.11.020: pure Pipe must parse as Ast::Pipe, not {:?}",
        pipe_ast
    );
    assert!(
        !matches!(pipe_ast, Ast::SqlPipe(_)),
        "BC-2.11.020: pure Pipe must NOT parse as Ast::SqlPipe"
    );
}

// ─── Area B: BC-2.11.021 — Temporal grammar (ADR-044) ────────────────────────

/// AC-004 / BC-2.11.021 postconditions — parse temporal expressions in all three modes
/// and verify planning-time constant injection.
///
/// Red Gate:
/// - `NOW()` and `INTERVAL` are not yet recognized grammar elements → parse returns Err.
/// - `prism_query::parse_and_plan` is a `todo!()` stub → panics.
/// Either failure is RED (not a compile error).
#[test]
fn test_bc_2_11_021_now_interval_parses_all_three_modes() {
    // SQL mode: `SELECT * FROM t WHERE timestamp > NOW() - INTERVAL '24h'`
    let sql = "SELECT * FROM t WHERE timestamp > NOW() - INTERVAL '24h'";
    let sql_result = PrismQlParser::parse(sql);
    assert!(
        sql_result.is_ok(),
        "BC-2.11.021: SQL-mode temporal query must parse; got errors: {:?}",
        sql_result
    );

    // Pipe mode: `FROM t | where timestamp > NOW() - INTERVAL '24h'`
    let pipe = "FROM t | where timestamp > NOW() - INTERVAL '24h'";
    let pipe_result = PrismQlParser::parse(pipe);
    assert!(
        pipe_result.is_ok(),
        "BC-2.11.021: Pipe-mode temporal query must parse; got errors: {:?}",
        pipe_result
    );

    // Filter mode: `timestamp > NOW() - INTERVAL '24h'`
    let filter = "timestamp > NOW() - INTERVAL '24h'";
    let filter_result = PrismQlParser::parse(filter);
    assert!(
        filter_result.is_ok(),
        "BC-2.11.021: Filter-mode temporal query must parse; got errors: {:?}",
        filter_result
    );

    // Planning-time substitution: parse_and_plan replaces Expr::Now with Literal::Timestamp.
    // This stub panics on todo!() → RED.
    let plan_result = prism_query::parse_and_plan(sql);
    assert!(
        plan_result.is_ok(),
        "BC-2.11.021: parse_and_plan must succeed (NOW() → timestamp constant injection); got: {:?}",
        plan_result
    );
}

/// AC-005 / BC-2.11.021 — three distinct E-QUERY-001 error cases.
///
/// (1) `NOW(1)` — NOW() takes no arguments.
/// (2) `NOW() + INTERVAL '1h'` — subtraction-only in v1.
/// (3) `INTERVAL 'bogus'` — invalid duration literal.
///
/// Red Gate: `NOW()` is not yet a grammar element. All three queries fall into
/// filter-mode parsing treating `NOW` as a field name, producing wrong-reason errors
/// or no error at all. The message-content assertions will fail.
#[test]
fn test_bc_2_11_021_now_error_cases() {
    // Case 1: NOW(1) — "NOW() takes no arguments".
    let case1_query = "SELECT * FROM t WHERE timestamp > NOW(1) - INTERVAL '1h'";
    let case1 = PrismQlParser::parse(case1_query);
    assert!(
        case1.is_err(),
        "BC-2.11.021: NOW(1) must be a parse error; unexpectedly got Ok"
    );
    let err_msgs1: Vec<String> = case1.unwrap_err().iter().map(|e| e.to_string()).collect();
    assert!(
        err_msgs1
            .iter()
            .any(|m| m.contains("NOW() takes no arguments")),
        "BC-2.11.021: error for NOW(1) must contain 'NOW() takes no arguments'; got: {err_msgs1:?}"
    );

    // Case 2: NOW() + INTERVAL — subtraction-only in v1.
    let case2_query = "SELECT * FROM t WHERE timestamp > NOW() + INTERVAL '1h'";
    let case2 = PrismQlParser::parse(case2_query);
    assert!(
        case2.is_err(),
        "BC-2.11.021: NOW() + INTERVAL must be a parse error; unexpectedly got Ok"
    );
    let err_msgs2: Vec<String> = case2.unwrap_err().iter().map(|e| e.to_string()).collect();
    assert!(
        err_msgs2.iter().any(|m| m.contains("subtraction") || m.contains("only")),
        "BC-2.11.021: error for NOW()+INTERVAL must mention subtraction-only constraint; got: {err_msgs2:?}"
    );

    // Case 3: INTERVAL 'bogus' — invalid duration literal.
    let case3_query = "SELECT * FROM t WHERE timestamp > NOW() - INTERVAL 'bogus'";
    let case3 = PrismQlParser::parse(case3_query);
    assert!(
        case3.is_err(),
        "BC-2.11.021: INTERVAL 'bogus' must be a parse error; unexpectedly got Ok"
    );
    let err_msgs3: Vec<String> = case3.unwrap_err().iter().map(|e| e.to_string()).collect();
    assert!(
        err_msgs3
            .iter()
            .any(|m| { m.contains("duration") || m.contains("INTERVAL") || m.contains("valid") }),
        "BC-2.11.021: error for INTERVAL 'bogus' must mention invalid duration; got: {err_msgs3:?}"
    );
}

// ─── Area D: BC-2.11.023 — Three-mode correctness + mode-bridge (ADR-046) ───

/// AC-009 / BC-2.11.023 postcondition (ADR-046 §D1) — mode-bridge diagnostic (tightened v1.5).
///
/// `SELECT * FROM t | INVALID_KEYWORD` triggers the mode-bridge D1 heuristic.
/// Asserts ALL THREE required substrings from BC-2.11.023 §D1 (verbatim, POL-24):
///   (a) `(enrich, where, limit, sort, stats, dedup, fields)` — stage-keyword enumeration
///   (b) `1. SQL+pipe composition:` and `2. Pipe mode only:` — numbered alternatives
///   (c) `See prismql://reference for the complete grammar.` — reference pointer
/// AND a negative control: no raw Chumsky token dump (`expected one of`).
///
/// Red Gate: the previous message omitted (a), (b), and (c). Strengthened assertions
/// fail RED until the verbatim BC §D1 message is in place.
#[test]
fn test_bc_2_11_023_mode_bridge_d1_sql_pipe_diagnostic() {
    let query = "SELECT * FROM t | INVALID_KEYWORD";
    let errs = PrismQlParser::parse(query)
        .expect_err("BC-2.11.023: SQL query with bare | must be a parse error");

    let combined_message = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(" | ");

    // (a) Stage-keyword enumeration must be present verbatim (BC-2.11.023 §D1, POL-24).
    assert!(
        combined_message.contains("(enrich, where, limit, sort, stats, dedup, fields)"),
        "BC-2.11.023 AC-009(a): mode-bridge D1 message must contain \
         '(enrich, where, limit, sort, stats, dedup, fields)'; got: {combined_message}"
    );
    // (b) Numbered alternatives — SQL+pipe composition option.
    assert!(
        combined_message.contains("1. SQL+pipe composition:"),
        "BC-2.11.023 AC-009(b): mode-bridge D1 message must contain \
         '1. SQL+pipe composition:'; got: {combined_message}"
    );
    // (b) Numbered alternatives — Pipe mode only option.
    assert!(
        combined_message.contains("2. Pipe mode only:"),
        "BC-2.11.023 AC-009(b): mode-bridge D1 message must contain \
         '2. Pipe mode only:'; got: {combined_message}"
    );
    // (c) Reference pointer.
    assert!(
        combined_message.contains("See prismql://reference for the complete grammar."),
        "BC-2.11.023 AC-009(c): mode-bridge D1 message must contain \
         'See prismql://reference for the complete grammar.'; got: {combined_message}"
    );
    // Negative control: Must NOT be a raw Chumsky token dump.
    assert!(
        !combined_message.contains("expected one of"),
        "BC-2.11.023: mode-bridge D1 must NOT produce a raw Chumsky dump; got: {combined_message}"
    );
}

// AC-010 test (test_bc_2_11_023_normalized_pql_on_mode_bridge_error) is in
// crates/prism-mcp/tests/mcp_infrastructure.rs — it uses both prism_mcp::error_mapping
// and prism_query::PrismQlParser, and prism_mcp is not a dependency of prism-query.
// See mcp_infrastructure.rs for the test body.

/// AC-011 (wrapped) / BC-2.11.023 + BC-2.11.002 — filter-mode end-to-end parse.
///
/// This test verifies parse-level behavior (compilation of the execute path
/// is in `filter_mode.rs`).  It asserts bare predicates parse as `Ast::Filter`
/// and source-qualified predicates parse as `Ast::Filter` with a non-empty source.
///
/// Red Gate: on develop today these PASS (filter parse is implemented).
/// This test is a regression guard ensuring the filter-mode parse path is
/// not broken when the SqlPipe grammar is added in Area A.
#[test]
fn test_bc_2_11_023_filter_mode_end_to_end_execution() {
    // Bare predicate: must parse as Ast::Filter.
    let simple = "severity = 'HIGH'";
    let ast = PrismQlParser::parse(simple)
        .expect("BC-2.11.023: bare predicate must parse as Ast::Filter");
    assert!(
        matches!(ast, Ast::Filter(_)),
        "BC-2.11.023: bare predicate must be Ast::Filter; got {:?}",
        ast
    );

    // Source-qualified predicate: must parse as Ast::Filter with source set.
    let with_source = "crowdstrike.detections | severity = 'HIGH'";
    let ast2 = PrismQlParser::parse(with_source)
        .expect("BC-2.11.023: source-qualified predicate must parse as Ast::Filter");
    assert!(
        matches!(ast2, Ast::Filter(_)),
        "BC-2.11.023: source-qualified predicate must be Ast::Filter; got {:?}",
        ast2
    );
    let Ast::Filter(ref fe) = ast2 else {
        unreachable!()
    };
    assert!(
        !fe.source.raw.is_empty(),
        "BC-2.11.023: source-qualified filter must have a non-empty source reference"
    );
}

// ─── Area C: GRAMMAR-005/015 — Enrich parse-error guidance (AC-022 / AC-025) ──

/// AC-022 / GRAMMAR-005 — `enrich` without column argument in a simple pipe.
///
/// `FROM t | enrich threat_score` — `threat_score` is the infusion name but the
/// required `(<column>)` argument is absent.  The parse error MUST contain the
/// actionable guidance:
///   `enrich requires a column argument: | enrich <infusion>(<column>)`
///
/// RED GATE: currently Chumsky emits a raw token-expectation dump
/// ("expected '('"). The substring assertion below fails RED.
///
/// Mental-deletion proof: removing the enrich-guidance rewrite in error_recovery.rs
/// reverts to raw Chumsky output, causing the `enrich requires a column argument`
/// assertion to fail.
#[test]
fn test_bc_2_11_grammar005_enrich_missing_column_arg_guidance() {
    let query = "FROM t | enrich threat_score";
    let errs = PrismQlParser::parse(query)
        .expect_err("AC-022: 'FROM t | enrich threat_score' must be a parse error");

    let combined = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(" | ");

    assert!(
        combined.contains("enrich requires a column argument"),
        "AC-022: error must contain 'enrich requires a column argument'; got: {combined}"
    );
    assert!(
        combined.contains("| enrich <infusion>(<column>)"),
        "AC-022: error must contain '| enrich <infusion>(<column>)' example; got: {combined}"
    );
}

/// AC-025 / GRAMMAR-015 — `enrich` without column argument in a multi-stage pipeline.
///
/// `FROM t | where severity = 'HIGH' | enrich threat_score` — the `enrich` stage
/// appears in position 2 (after a `where` stage).  The same actionable guidance
/// MUST appear regardless of pipeline position.
///
/// RED GATE: same as AC-022 — raw Chumsky dump today, not guided message.
///
/// Mental-deletion proof: removing the enrich-guidance rewrite causes the substring
/// assertion to fail for pipelines with a preceding where stage.
#[test]
fn test_bc_2_11_grammar015_enrich_missing_column_arg_multi_stage_guidance() {
    let query = "FROM t | where severity = 'HIGH' | enrich threat_score";
    let errs = PrismQlParser::parse(query)
        .expect_err("AC-025: 'FROM t | where … | enrich threat_score' must be a parse error");

    let combined = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(" | ");

    assert!(
        combined.contains("enrich requires a column argument"),
        "AC-025: error must contain 'enrich requires a column argument' in multi-stage pipeline; got: {combined}"
    );
    assert!(
        combined.contains("| enrich <infusion>(<column>)"),
        "AC-025: error must contain '| enrich <infusion>(<column>)' example in multi-stage pipeline; got: {combined}"
    );
}

/// AC-012 / BC-2.11.023 invariant D7 — shared predicate grammar.
///
/// Parse `severity = 'HIGH' AND risk_score > 50` in all three entry forms and assert
/// the predicate subtrees are equivalent (`PartialEq` on `Predicate`).
///
/// Red Gate: on develop today filter / pipe / SQL predicates share a parser,
/// so the `Predicate` trees are already equal and this test PASSES.
/// After Area B temporal grammar is added, any mode-specific predicate extension
/// in `build_predicate_parser` would break this invariant — this test catches that.
/// It is load-bearing as a regression guard.
#[test]
fn test_bc_2_11_023_d7_shared_predicate_grammar() {
    let predicate = "severity = 'HIGH' AND risk_score > 50";

    // Filter mode.
    let filter_ast =
        PrismQlParser::parse(predicate).expect("BC-2.11.023 D7: filter-mode predicate must parse");
    let Ast::Filter(ref fe) = filter_ast else {
        panic!("BC-2.11.023 D7: expected Ast::Filter; got {:?}", filter_ast)
    };

    // SQL WHERE mode.
    let sql_query = format!("SELECT * FROM t WHERE {predicate}");
    let sql_ast =
        PrismQlParser::parse(&sql_query).expect("BC-2.11.023 D7: SQL WHERE predicate must parse");
    let sql_where = if let Ast::Sql(SqlStatement::Select(ref sq)) = sql_ast {
        sq.where_.as_ref().expect("SQL must have WHERE clause")
    } else {
        panic!(
            "BC-2.11.023 D7: expected Ast::Sql(Select(_)); got {:?}",
            sql_ast
        )
    };

    // Pipe | where mode.
    let pipe_query = format!("FROM t | where {predicate}");
    let pipe_ast = PrismQlParser::parse(&pipe_query)
        .expect("BC-2.11.023 D7: Pipe | where predicate must parse");
    let pipe_where = if let Ast::Pipe(ref pq) = pipe_ast {
        pq.stages
            .iter()
            .find_map(|s| {
                if let PipeStage::Where(p) = s {
                    Some(p)
                } else {
                    None
                }
            })
            .expect("BC-2.11.023 D7: Pipe must have | where stage")
    } else {
        panic!("BC-2.11.023 D7: expected Ast::Pipe; got {:?}", pipe_ast)
    };

    // D7 invariant: all predicate trees must have the same semantic structure.
    //
    // Predicates carry `Span { start, end }` byte-offsets into their source query string.
    // In filter mode the predicate starts at offset 0; in SQL mode the WHERE clause
    // starts at a later offset (e.g. after `SELECT * FROM t WHERE `). Span values
    // therefore differ across modes even for identical predicate text.
    //
    // We strip span fields before comparing: serialize to JSON, remove all `"span"` keys,
    // then compare the span-free JSON trees.
    fn strip_spans(val: &mut serde_json::Value) {
        match val {
            serde_json::Value::Object(map) => {
                map.remove("span");
                for v in map.values_mut() {
                    strip_spans(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    strip_spans(v);
                }
            }
            _ => {}
        }
    }

    let mut filter_json = serde_json::to_value(&fe.predicate)
        .expect("BC-2.11.023 D7: filter predicate must serialize to JSON");
    let mut sql_json = serde_json::to_value(sql_where)
        .expect("BC-2.11.023 D7: SQL WHERE predicate must serialize to JSON");
    let mut pipe_json = serde_json::to_value(pipe_where)
        .expect("BC-2.11.023 D7: Pipe WHERE predicate must serialize to JSON");

    strip_spans(&mut filter_json);
    strip_spans(&mut sql_json);
    strip_spans(&mut pipe_json);

    assert_eq!(
        filter_json, sql_json,
        "BC-2.11.023 D7: filter-mode predicate must equal SQL WHERE predicate (shared grammar)"
    );
    assert_eq!(
        filter_json, pipe_json,
        "BC-2.11.023 D7: filter-mode predicate must equal Pipe | where predicate (shared grammar)"
    );
}
