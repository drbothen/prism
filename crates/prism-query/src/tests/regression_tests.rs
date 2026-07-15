//! Regression tests for PR-127 review findings.
//!
//! These tests were written FIRST (TDD Red Gate), then the fixes were implemented.
//!
//! # Findings covered
//! | Finding | Tests |
//! |---------|-------|
//! | B-2: SQL/pipe AST depth check missing | test_BC_2_11_006_sql_and_chain_depth_65_rejected, etc. |
//! | B-4: walk_sql_statement irrefutable let | test_walk_sql_statement_select_variant_traversed |
//! | B-5: env-var override has no min floor | test_BC_2_11_006_env_query_size_zero_clamped_to_default, etc. |
//! | B-6: backslash in string literal treated literally | test_BC_2_11_002_string_literal_backslash_treated_literally |
//! | B-7: SQL_KEYWORDS case-sensitivity bypass | test_BC_2_11_003_alias_titlecase_keyword_rejected |
//! | B-8: Unbounded IN list | test_BC_2_11_006_in_list_1025_items_rejected, etc. |
//! | B-9: Error messages echo user input verbatim | test_error_message_truncates_long_user_input |
//!
//! Story: S-3.01 | PR-127 | BC-2.11.006 | DI-019

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::assertions_on_constants,
    // F-LOW-002 / OBS-002: regression tests are sanctioned direct callers of
    // parse_filter, parse_pipe, parse_sql — they test post-parse depth checks
    // in isolation (bypassing PrismQlParser::parse intentionally).
    clippy::disallowed_methods,
    unused_imports
)]

use crate::{
    ast::Ast,
    filter_parser::{parse_filter, PrismQlParser},
    pipe_parser::parse_pipe,
    security::{
        effective_list_items_limit, effective_nesting_depth_limit, effective_pipe_stage_limit,
        effective_query_size_limit, effective_regex_pattern_length_limit, MAX_SAFE_LIST_ITEMS,
        MAX_SAFE_NESTING_DEPTH, MAX_SAFE_PIPE_STAGES, MAX_SAFE_QUERY_SIZE,
        MAX_SAFE_REGEX_PATTERN_LEN, MIN_SAFE_LIST_ITEMS, MIN_SAFE_NESTING_DEPTH,
        MIN_SAFE_PIPE_STAGES, MIN_SAFE_QUERY_SIZE, MIN_SAFE_REGEX_PATTERN_LEN,
        PRISM_MAX_LIST_ITEMS, PRISM_MAX_NESTING_DEPTH, PRISM_MAX_QUERY_SIZE,
    },
    sql_parser::parse_sql,
    tests::util::run_with_deep_stack,
    ParseError,
};

// ─────────────────────────────────────────────────────────────────────────────
// B-2: AST depth check missing for SQL and pipe modes
// ─────────────────────────────────────────────────────────────────────────────

/// B-2: SQL WHERE with 65 right-nested parenthesised ANDs must be rejected by
/// the post-parse AST depth check in `parse_sql`.
///
/// When `parse_sql` is called directly (bypassing `PrismQlParser::parse`),
/// the pre-parse `check_paren_depth` guard is NOT applied. The post-parse
/// `check_sql_query_nesting_depth` call (added by B-2 fix) must catch this.
///
/// Structure: WHERE (a1=1 AND (a2=2 AND (a3=3 AND ... (a65=65)...)))
/// Each level of parens adds 1 to the AST depth.
///
/// Traces: B-2, BC-2.11.006, DI-019, EC-002
#[test]
fn test_BC_2_11_006_sql_and_chain_depth_65_rejected() {
    // Wrapped in 8MB-stack thread: 65-deep chumsky recursion can SIGBUS on macOS
    // aarch64 with `[profile.dev] debug = "line-tables-only"` (commit 931f3c6f).
    // See triage report and tests/util.rs for full rationale.
    run_with_deep_stack(|| {
        // Build a right-nested paren AND expression that exceeds depth 64.
        // Each parenthesised sub-expression adds 1 to the nesting depth.
        // 65 levels of nesting exceed PRISM_MAX_NESTING_DEPTH (64).
        let mut inner = "a65 = 65".to_string();
        for i in (1..65).rev() {
            inner = format!("(a{i} = {i} AND {inner})");
        }
        let input = format!("SELECT * FROM src WHERE {inner}");

        // Call parse_sql directly to bypass the pre-parse paren_depth check.
        // The post-parse check_sql_query_nesting_depth must still catch this.
        let result = parse_sql(&input);
        assert!(
            result.is_err(),
            "B-2: SQL WHERE with 65-deep nested ANDs must be rejected by post-parse depth check; got Ok"
        );
        let errs = result.unwrap_err();
        let msg = errs[0].message.clone();
        assert!(
            msg.contains("E-QUERY-003"),
            "B-2: error must contain E-QUERY-003, got: {msg}"
        );
    });
}

/// B-2: SQL WHERE with mixed AND/OR forcing 65-deep nesting must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_sql_or_mix_depth_65_rejected() {
    // Wrapped in 8MB-stack thread: 65-deep chumsky recursion can SIGBUS on macOS
    // aarch64 with `[profile.dev] debug = "line-tables-only"`. See tests/util.rs.
    run_with_deep_stack(|| {
        // Alternate AND/OR to force deep nesting: a1=1 OR (a2=2 AND (a3=3 OR ...))
        // Use paren groups to ensure real depth (paren check fires at 65 parens).
        // We'll use deeply right-nested parens that exceed the limit.
        // Each paren pair adds 1 to the paren counter. With 65 pairs we exceed 64.
        let mut query = "SELECT * FROM src WHERE ".to_string();
        // Build 65 opening parens, each containing a comparison
        for i in 0..65 {
            query.push_str(&format!("(a{i} = {i} OR "));
        }
        query.push_str("z = 0");
        for _ in 0..65 {
            query.push(')');
        }

        // This should fail either at paren_depth check or at nesting depth check
        let result = PrismQlParser::parse(&query);
        assert!(
            result.is_err(),
            "B-2: deeply nested SQL OR/AND must be rejected; got Ok"
        );
    });
}

/// B-2: SQL with deep IN (SELECT ... WHERE ... IN (SELECT ...)) subquery chain
/// must be rejected by the post-parse AST depth check in `parse_sql`.
///
/// When `parse_sql` is called directly (bypassing `PrismQlParser::parse`),
/// the pre-parse `check_paren_depth` guard is NOT applied. The post-parse
/// `check_sql_query_nesting_depth` call (added by B-2 fix) traverses into
/// Predicate::InSubquery and must catch excessive subquery nesting.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_sql_subquery_depth_65_rejected() {
    // Wrapped in 8MB-stack thread: 65-deep chumsky recursion can SIGBUS on macOS
    // aarch64 with `[profile.dev] debug = "line-tables-only"`. See tests/util.rs.
    run_with_deep_stack(|| {
        // Build a nested IN subquery chain that exceeds the nesting depth limit.
        // check_sql_query_nesting_depth recursively checks InSubquery, so each
        // nested SELECT adds 1 to the depth counter. With 65 levels, it exceeds
        // PRISM_MAX_NESTING_DEPTH (64).
        //
        // Note: we call parse_sql directly to bypass the pre-parse paren_depth
        // check (which counts lexical paren chars, not AST subquery depth).
        let mut inner = "SELECT * FROM s WHERE x = 1".to_string();
        for i in 0..65 {
            inner = format!("SELECT * FROM s{i} WHERE f IN ({inner})");
        }
        // The outermost query: SELECT * FROM src WHERE field IN (...)
        let query = format!("SELECT * FROM src WHERE field IN ({inner})");

        // Call parse_sql directly to test the post-parse depth check.
        let result = parse_sql(&query);
        assert!(
            result.is_err(),
            "B-2: deeply nested IN-subquery chain (65 levels) must be rejected by post-parse depth check; got Ok"
        );
        let errs = result.unwrap_err();
        let msg = errs[0].message.clone();
        assert!(
            msg.contains("E-QUERY-003"),
            "B-2: error must contain E-QUERY-003, got: {msg}"
        );
    });
}

/// B-2: Pipe `where` with 65 chained NOT predicates must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_pipe_where_not_chain_depth_65_rejected() {
    // Wrapped in 8MB-stack thread: 65-deep chumsky recursion can SIGBUS on macOS
    // aarch64 with `[profile.dev] debug = "line-tables-only"`. See tests/util.rs.
    run_with_deep_stack(|| {
        // Build: src | where NOT NOT NOT ... (65 NOTs) x = 1
        let nots = "NOT ".repeat(65);
        let input = format!("src | where {nots}x = 1");

        let result = PrismQlParser::parse(&input);
        assert!(
            result.is_err(),
            "B-2: pipe where with 65 chained NOTs must be rejected; got Ok"
        );
        let errs = result.unwrap_err();
        let msg = errs[0].message.clone();
        assert!(
            msg.contains("E-QUERY-003"),
            "B-2: error must contain E-QUERY-003, got: {msg}"
        );
    });
}

/// B-2: Pipe `where` with deeply nested parens exceeding depth 64 must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_pipe_where_subquery_depth_65_rejected() {
    // Wrapped in 8MB-stack thread: 65-deep chumsky recursion can SIGBUS on macOS
    // aarch64 with `[profile.dev] debug = "line-tables-only"`. See tests/util.rs.
    run_with_deep_stack(|| {
        // Use 65 layers of parenthesized predicates: (((... x = 1 ...)))
        let mut inner = "x = 1".to_string();
        for _ in 0..65 {
            inner = format!("({inner})");
        }
        let input = format!("src | where {inner}");

        let result = PrismQlParser::parse(&input);
        assert!(
            result.is_err(),
            "B-2: pipe where with 65 paren depth must be rejected; got Ok"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// B-4: walk_sql_statement irrefutable let on #[non_exhaustive] enum
// ─────────────────────────────────────────────────────────────────────────────

/// B-4: walk_sql_statement must traverse a Select variant without panicking.
///
/// After the fix, walk_sql_statement uses `match` instead of irrefutable `let`.
/// This test verifies the Select arm still works correctly post-fix.
///
/// Traces: B-4
#[test]
fn test_walk_sql_statement_select_variant_traversed() {
    use crate::{
        ast::{FieldPath, Literal},
        visit::{walk_ast, Visitor},
    };

    struct FieldCounter(usize);
    impl Visitor for FieldCounter {
        fn visit_field(&mut self, _f: &FieldPath) {
            self.0 += 1;
        }
    }

    let ast =
        PrismQlParser::parse("SELECT a FROM src WHERE b = 1").expect("simple SELECT must parse");
    let mut counter = FieldCounter(0);
    walk_ast(&mut counter, &ast);
    assert!(
        counter.0 >= 1,
        "B-4: walk_sql_statement must visit at least one field in a Select query"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// B-5: env-var override has no min floor
// ─────────────────────────────────────────────────────────────────────────────

/// B-5: Setting PRISM_MAX_QUERY_SIZE=0 must be clamped to the safe minimum (not 0).
///
/// After the fix, effective_query_size_limit() returns MIN_SAFE_QUERY_SIZE when
/// the env var is 0, preventing bypass of the size guard.
///
/// Traces: B-5, BC-2.11.006, EC-001
#[test]
fn test_BC_2_11_006_env_query_size_zero_clamped_to_default() {
    // Set env var to 0 — must be clamped to minimum safe value (>= 1024)
    std::env::set_var("PRISM_MAX_QUERY_SIZE", "0");
    let limit = effective_query_size_limit();
    // Clean up immediately to avoid poisoning other tests
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");

    assert!(
        limit >= 1024,
        "B-5: PRISM_MAX_QUERY_SIZE=0 must be clamped to at least 1024 bytes, got {limit}"
    );
    assert!(
        limit <= PRISM_MAX_QUERY_SIZE,
        "B-5: clamped limit must not exceed the default ({PRISM_MAX_QUERY_SIZE}), got {limit}"
    );
}

/// B-5: Setting PRISM_MAX_NESTING_DEPTH to an excessive value (e.g., 99999) must
/// be clamped to the safe maximum (MAX_SAFE_NESTING_DEPTH = 256).
///
/// Traces: B-5, BC-2.11.006, EC-002
#[test]
fn test_BC_2_11_006_env_nesting_depth_excessive_clamped_to_max() {
    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "99999");
    let limit = effective_nesting_depth_limit();
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert!(
        limit <= 256,
        "B-5: PRISM_MAX_NESTING_DEPTH=99999 must be clamped to at most 256, got {limit}"
    );
    assert!(
        limit >= 8,
        "B-5: clamped nesting depth must be at least 8, got {limit}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// B-6: Backslash in string literals treated literally
// ─────────────────────────────────────────────────────────────────────────────

/// B-6: PrismQL string literals are RAW — no backslash escape sequences.
/// The string `'a\b'` must parse as the literal 4-character string `a\b`
/// (a, backslash, b), not as `a` followed by a bell character.
///
/// Traces: B-6, BC-2.11.002
#[test]
fn test_BC_2_11_002_string_literal_backslash_treated_literally() {
    let input = r"field = 'a\b'";
    let result = parse_filter(input);
    let fe = result.expect("B-6: filter with backslash in string literal must parse");

    use crate::ast::{Expr, Literal, Predicate};
    match &fe.predicate {
        Predicate::Compare { rhs, .. } => match rhs.as_ref() {
            Expr::Literal(Literal::String(s)) => {
                assert_eq!(
                    s, r"a\b",
                    "B-6: backslash must be treated as a literal character, not an escape sequence"
                );
                assert_eq!(s.len(), 3, "B-6: 'a\\b' must be 3 chars (a, backslash, b)");
            }
            other => panic!("B-6: expected Literal::String, got {:?}", other),
        },
        other => panic!("B-6: expected Predicate::Compare, got {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// B-7: SQL_KEYWORDS case-sensitivity bypass
// ─────────────────────────────────────────────────────────────────────────────

/// B-7: `SELECT a FROM t Where` — 'Where' (titlecase) must be rejected as alias
/// since it is a case-insensitive match for the keyword WHERE.
///
/// Before the fix, SQL_KEYWORDS.contains(&s) would miss "Where" because only
/// "WHERE" and "where" were in the list. After the fix, case-insensitive
/// comparison prevents aliases matching any case variant of SQL keywords.
///
/// Traces: B-7, BC-2.11.003
#[test]
fn test_BC_2_11_003_alias_titlecase_keyword_rejected() {
    // "Where" as a bare alias should be rejected (case-insensitive keyword check)
    let input = "SELECT a FROM t Where";
    let result = parse_sql(input);
    assert!(
        result.is_err(),
        "B-7: 'Where' (titlecase) used as alias must be rejected; got Ok"
    );
}

/// B-7: `SELECT a AS Select FROM t` — 'Select' (titlecase) as explicit alias
/// must also be rejected.
///
/// Traces: B-7, BC-2.11.003
#[test]
fn test_BC_2_11_003_as_alias_titlecase_keyword_rejected() {
    let input = "SELECT a AS Select FROM t";
    let result = parse_sql(input);
    assert!(
        result.is_err(),
        "B-7: 'Select' (titlecase) used as AS alias must be rejected; got Ok"
    );
}

/// B-7: Mixed-case keyword `sElEcT` must also be rejected as alias.
///
/// Traces: B-7, BC-2.11.003
#[test]
fn test_BC_2_11_003_alias_mixed_case_keyword_rejected() {
    let input = "SELECT a FROM t sElEcT";
    let result = parse_sql(input);
    assert!(
        result.is_err(),
        "B-7: 'sElEcT' (mixed case) used as alias must be rejected; got Ok"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// B-8: Unbounded IN list and separated_by repetitions
// ─────────────────────────────────────────────────────────────────────────────

/// B-8: An IN list with 1025 items must be rejected (limit is 1024).
///
/// Traces: B-8, BC-2.11.006, E-QUERY-002
#[test]
fn test_BC_2_11_006_in_list_1025_items_rejected() {
    // Build: field IN (1, 2, 3, ..., 1025)
    let values: Vec<String> = (1..=1025).map(|i| i.to_string()).collect();
    let in_clause = values.join(", ");
    let input = format!("field IN ({in_clause})");

    let result = PrismQlParser::parse(&input);
    assert!(
        result.is_err(),
        "B-8: IN list with 1025 items must be rejected; got Ok"
    );
    let errs = result.unwrap_err();
    let msg = errs[0].message.clone();
    assert!(
        msg.contains("E-QUERY-003") || msg.contains("E-QUERY-002") || msg.contains("list"),
        "B-8: error must mention query limit, got: {msg}"
    );
}

/// B-8: An ORDER BY clause with 1025 fields must be rejected (limit is 1024).
///
/// Traces: B-8, BC-2.11.006
#[test]
fn test_BC_2_11_006_order_by_1025_items_rejected() {
    // Build: SELECT * FROM src ORDER BY f1, f2, ..., f1025
    let fields: Vec<String> = (1..=1025).map(|i| format!("f{i}")).collect();
    let order_clause = fields.join(", ");
    let input = format!("SELECT * FROM src ORDER BY {order_clause}");

    // This query may be very large — first check size limit won't block it
    // prematurely (1025 fields * ~4 chars avg = ~5000 bytes, well under 64KB).
    assert!(
        input.len() < PRISM_MAX_QUERY_SIZE,
        "B-8: test input must be under max query size to isolate the list limit"
    );

    let result = PrismQlParser::parse(&input);
    assert!(
        result.is_err(),
        "B-8: ORDER BY with 1025 items must be rejected; got Ok"
    );
    let errs = result.unwrap_err();
    let msg = errs[0].message.clone();
    assert!(
        msg.contains("E-QUERY-003") || msg.contains("E-QUERY-002") || msg.contains("list"),
        "B-8: error must mention query limit, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// B-9: Error messages echo arbitrary user input verbatim
// ─────────────────────────────────────────────────────────────────────────────

/// B-9: Submitting a 10KB invalid CIDR string must produce an error message
/// that is less than 500 bytes (user input is truncated in error output).
///
/// Before the fix, CidrLiteral::new formats the full user string into the
/// error message. After the fix, a truncation helper caps echo at 200 bytes.
///
/// Traces: B-9, BC-2.11.006
#[test]
fn test_error_message_truncates_long_user_input() {
    // Construct a 10KB "CIDR" string that is not a valid CIDR.
    // Use something that passes string literal parsing but fails CIDR validation.
    let bad_cidr = "x".repeat(10_240);
    let input = format!("field IN CIDR '{bad_cidr}'");

    // The query is > 64KB? No: 10240 + overhead is ~10260 bytes, under 65536.
    // So it will pass size check and reach CIDR validation.
    assert!(
        input.len() < PRISM_MAX_QUERY_SIZE,
        "B-9: test input must be under max query size"
    );

    let result = PrismQlParser::parse(&input);
    // It must fail (bad CIDR), but the error message must be short.
    assert!(result.is_err(), "B-9: invalid CIDR must produce an error");
    let errs = result.unwrap_err();
    let msg = &errs[0].message;
    assert!(
        msg.len() < 500,
        "B-9: error message must be < 500 bytes after truncation, got {} bytes: {}",
        msg.len(),
        &msg[..msg.len().min(100)]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-LOW-001: walk_predicate must visit RecoveryError as an explicit leaf
// ─────────────────────────────────────────────────────────────────────────────

/// F-LOW-001: `walk_predicate` must handle `Predicate::RecoveryError` via
/// an explicit arm (not fall-through catch-all), and must invoke
/// `visit_predicate` on it exactly once without panicking.
///
/// A visitor that counts `visit_predicate` calls must see exactly 1 call
/// for a root `Predicate::RecoveryError` (the root visit call from
/// `walk_filter_expr` / `walk_pipe_stage`, plus the dispatch through
/// `visit_predicate` -> `walk_predicate`).
///
/// Traces: F-LOW-001, S-3.01
#[test]
fn test_walk_predicate_visits_recovery_error_as_leaf() {
    use crate::{
        ast::{FilterExpr, Predicate, SourceRef},
        visit::{walk_filter_expr, Visitor},
    };

    /// Visitor that counts how many times `visit_predicate` is called.
    struct PredicateCounter(usize);
    impl Visitor for PredicateCounter {
        fn visit_predicate(&mut self, p: &Predicate) {
            self.0 += 1;
            // Call the default walk to exercise the walk_predicate dispatch.
            crate::visit::walk_predicate(self, p);
        }
    }

    // Build a FilterExpr whose predicate is a RecoveryError sentinel.
    let fe = FilterExpr {
        source: SourceRef::from_raw("crowdstrike.detections"),
        predicate: Predicate::RecoveryError,
    };

    let mut counter = PredicateCounter(0);
    walk_filter_expr(&mut counter, &fe);

    assert_eq!(
        counter.0, 1,
        "F-LOW-001: walk_predicate must visit Predicate::RecoveryError exactly once as a leaf; got {} visits",
        counter.0
    );
}

/// F-LOW-001: Walking a `Predicate::RecoveryError` nested inside a
/// `Predicate::Logical` must visit it exactly once (leaf, no further descent).
///
/// Traces: F-LOW-001, S-3.01
#[test]
fn test_walk_predicate_recovery_error_inside_logical_visited_once() {
    use crate::{
        ast::{FieldPath, LogicalOp, Predicate, Span},
        visit::{walk_predicate, Visitor},
    };

    struct PredicateCounter(usize);
    impl Visitor for PredicateCounter {
        fn visit_predicate(&mut self, p: &Predicate) {
            self.0 += 1;
            crate::visit::walk_predicate(self, p);
        }
    }

    // Logical { AND: [RecoveryError, RecoveryError] }
    let logical = Predicate::Logical {
        op: LogicalOp::And,
        predicates: vec![Predicate::RecoveryError, Predicate::RecoveryError],
    };

    let mut counter = PredicateCounter(0);
    walk_predicate(&mut counter, &logical);

    // The outer Logical calls visit_predicate on each child => 2 RecoveryError visits.
    // The outer Logical itself is not counted here (walk_predicate is called directly
    // on it, not via visit_predicate).
    assert_eq!(
        counter.0, 2,
        "F-LOW-001: two RecoveryError children of Logical must each be visited once; got {}",
        counter.0
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-LOW-003: PRISM_MAX_LIST_ITEMS lacks env-var override (sibling coverage gap)
// ─────────────────────────────────────────────────────────────────────────────

/// F-LOW-003: Setting PRISM_MAX_LIST_ITEMS=0 must be clamped to the safe minimum.
///
/// All sibling effective_*_limit() functions have min/max clamping.
/// effective_list_items_limit() must mirror that pattern.
///
/// Traces: F-LOW-003, BC-2.11.006
#[test]
fn test_BC_2_11_006_env_list_items_zero_clamped() {
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "0");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert!(
        limit >= 16,
        "F-LOW-003: PRISM_MAX_LIST_ITEMS=0 must be clamped to at least MIN_SAFE_LIST_ITEMS (16), got {limit}"
    );
    assert!(
        limit <= PRISM_MAX_LIST_ITEMS,
        "F-LOW-003: clamped list items limit must not exceed default ({PRISM_MAX_LIST_ITEMS}), got {limit}"
    );
}

/// F-LOW-003: Setting PRISM_MAX_LIST_ITEMS to an excessive value (e.g., 99999) must
/// be clamped to the safe maximum (MAX_SAFE_LIST_ITEMS = 16384).
///
/// Traces: F-LOW-003, BC-2.11.006
#[test]
fn test_BC_2_11_006_env_list_items_excessive_clamped() {
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "99999");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert!(
        limit <= 16_384,
        "F-LOW-003: PRISM_MAX_LIST_ITEMS=99999 must be clamped to at most MAX_SAFE_LIST_ITEMS (16384), got {limit}"
    );
    assert!(
        limit >= 16,
        "F-LOW-003: clamped list items limit must be at least 16, got {limit}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-LOW-002: Limits must be snapshotted once per parse() call
// ─────────────────────────────────────────────────────────────────────────────

/// F-LOW-002: `ParseLimits::snapshot()` must capture all effective limit values
/// and the same instance used across all security guards within one `parse()` call.
///
/// This test verifies that:
/// 1. `ParseLimits::snapshot()` exists and produces a struct.
/// 2. The snapshot captures the effective values at the moment of the call.
/// 3. After snapshot, changing the env var does NOT change the snapshotted values.
///
/// Traces: F-LOW-002, BC-2.11.006
#[test]
fn test_parse_limits_snapshot_is_immutable_after_capture() {
    use crate::security::ParseLimits;

    // Set env vars to known values before snapshot.
    std::env::set_var("PRISM_MAX_QUERY_SIZE", "8192");
    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "12");
    std::env::set_var("PRISM_MAX_PIPE_STAGES", "5");
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "128");
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "64");

    let limits = ParseLimits::snapshot();

    // Now change env vars after snapshot — the snapshot must not change.
    std::env::set_var("PRISM_MAX_QUERY_SIZE", "99999999");
    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "255");
    std::env::set_var("PRISM_MAX_PIPE_STAGES", "200");
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "65000");
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "10000");

    // Clean up.
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    // Snapshotted values must reflect what was set BEFORE the snapshot.
    assert_eq!(
        limits.query_size, 8192,
        "F-LOW-002: snapshot must capture query_size=8192, got {}",
        limits.query_size
    );
    assert_eq!(
        limits.nesting_depth, 12,
        "F-LOW-002: snapshot must capture nesting_depth=12, got {}",
        limits.nesting_depth
    );
    assert_eq!(
        limits.pipe_stages, 5,
        "F-LOW-002: snapshot must capture pipe_stages=5, got {}",
        limits.pipe_stages
    );
    assert_eq!(
        limits.regex_pattern, 128,
        "F-LOW-002: snapshot must capture regex_pattern=128, got {}",
        limits.regex_pattern
    );
    assert_eq!(
        limits.list_items, 64,
        "F-LOW-002: snapshot must capture list_items=64, got {}",
        limits.list_items
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-HIGH-001: ParseLimits snapshot propagates to ALL 9 guards (race-free)
// ─────────────────────────────────────────────────────────────────────────────
//
// Serialise env-var mutation in these tests to prevent cross-contamination when
// running under `cargo test` (which runs tests in parallel on the same process).
// Each test acquires the lock, sets vars, takes the snapshot, resets vars, then
// calls the guard — all while holding the lock.

use std::sync::Mutex;

// Global mutex for tests that mutate env vars.
// All env-var-sensitive tests MUST acquire this before touching env vars.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// F-HIGH-001: `check_predicate_nesting_depth_with` uses the snapshotted
/// `nesting_depth` limit, not the current env-var value.
///
/// Protocol:
/// 1. Set PRISM_MAX_NESTING_DEPTH=8 (MIN floor).
/// 2. Snapshot → limits.nesting_depth = 8.
/// 3. Change PRISM_MAX_NESTING_DEPTH=64 (default) — post-snapshot.
/// 4. Build a predicate at depth 9 (exceeds snapshotted 8, below new 64).
/// 5. Call check_predicate_nesting_depth_with → must reject (used snapshot value 8).
///
/// Traces: F-HIGH-001, BC-2.11.006
#[test]
fn test_parse_limits_snapshot_propagates_to_predicate_depth_guard() {
    use crate::{
        ast::{FieldPath, Literal, Predicate, Span},
        security::ParseLimits,
    };

    let _guard = ENV_MUTEX.lock().unwrap();

    // Snapshot with nesting_depth = 8 (MIN_SAFE floor).
    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "8");
    let limits = ParseLimits::snapshot();
    // Change env var after snapshot — guard must still use snapshotted 8.
    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "64");
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert_eq!(
        limits.nesting_depth, 8,
        "F-HIGH-001: snapshot nesting_depth must be 8 (MIN_SAFE), got {}",
        limits.nesting_depth
    );

    // Build a 10-deep chain of Predicate::Not (depth 0..9 from root; call at depth=9).
    // With limit=8, depth 9 > 8 must be rejected.
    fn make_not_chain(depth: u32) -> Predicate {
        if depth == 0 {
            Predicate::Compare {
                lhs: Box::new(crate::ast::Expr::Literal(Literal::Integer(1))),
                op: crate::ast::CompareOp::Eq,
                rhs: Box::new(crate::ast::Expr::Literal(Literal::Integer(1))),
                case_insensitive: false,
            }
        } else {
            Predicate::Not(Box::new(make_not_chain(depth - 1)))
        }
    }

    // 9 NOTs around a leaf → root call at depth 0, reaching depth 9 at the leaf.
    let pred = make_not_chain(9);
    let result = limits.check_predicate_nesting_depth_with(&pred, 0);
    assert!(
        result.is_err(),
        "F-HIGH-001: depth-9 predicate must be rejected by snapshotted limit 8; got Ok"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("E-QUERY-003"),
        "F-HIGH-001: error must contain E-QUERY-003, got: {err}"
    );
}

/// F-HIGH-001: `check_pipe_stage_count_with` uses the snapshotted
/// `pipe_stages` limit, not the current env-var value.
///
/// Traces: F-HIGH-001, BC-2.11.006
#[test]
fn test_parse_limits_snapshot_propagates_to_pipe_stage_guard() {
    use crate::{
        ast::{FieldPath, PipeQuery, PipeStage, SourceRef},
        security::ParseLimits,
    };

    let _guard = ENV_MUTEX.lock().unwrap();

    // Snapshot with pipe_stages = MIN_SAFE_PIPE_STAGES (4 after OBS-001 fix).
    std::env::set_var("PRISM_MAX_PIPE_STAGES", "4");
    let limits = ParseLimits::snapshot();
    // Change env var to 32 after snapshot — guard must still use 4.
    std::env::set_var("PRISM_MAX_PIPE_STAGES", "32");
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");

    assert_eq!(
        limits.pipe_stages, 4,
        "F-HIGH-001: snapshot pipe_stages must be 4, got {}",
        limits.pipe_stages
    );

    // 5 stages > snapshotted limit 4 → must reject.
    let stages: Vec<PipeStage> = (0..5)
        .map(|_| {
            PipeStage::Fields(crate::ast::FieldsStage {
                fields: vec![],
                include: true,
            })
        })
        .collect();

    let result = limits.check_pipe_stage_count_with(&stages);
    assert!(
        result.is_err(),
        "F-HIGH-001: 5 stages must be rejected by snapshotted limit 4; got Ok"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("E-QUERY-003"),
        "F-HIGH-001: error must contain E-QUERY-003, got: {err}"
    );
}

/// F-HIGH-001: `check_list_length_with` uses the snapshotted `list_items`
/// limit, not the current env-var value.
///
/// Traces: F-HIGH-001, BC-2.11.006
#[test]
fn test_parse_limits_snapshot_propagates_to_list_size_guard() {
    use crate::security::ParseLimits;

    let _guard = ENV_MUTEX.lock().unwrap();

    // Snapshot with list_items = 16 (MIN_SAFE floor).
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "16");
    let limits = ParseLimits::snapshot();
    // Change env var to 1024 after snapshot — guard must still use 16.
    std::env::set_var("PRISM_MAX_LIST_ITEMS", "1024");
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert_eq!(
        limits.list_items, 16,
        "F-HIGH-001: snapshot list_items must be 16, got {}",
        limits.list_items
    );

    // 17 items > snapshotted 16 → must reject.
    let result = limits.check_list_length_with(17, "IN list");
    assert!(
        result.is_err(),
        "F-HIGH-001: 17-item list must be rejected by snapshotted limit 16; got Ok"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("E-QUERY-003"),
        "F-HIGH-001: error must contain E-QUERY-003, got: {err}"
    );
}

/// F-HIGH-001: `ParseLimits::current_regex_limit()` returns the thread-local
/// snapshotted value when `install_thread_local` has been called.
///
/// This verifies that `RegexLiteral::new` (which calls `current_regex_limit()`)
/// uses the snapshot rather than re-reading the env var during parsing.
///
/// Traces: F-HIGH-001, BC-2.11.006
#[test]
fn test_parse_limits_thread_local_regex_limit_uses_snapshot() {
    use crate::security::ParseLimits;

    let _guard = ENV_MUTEX.lock().unwrap();

    // Set limit to 64 (MIN_SAFE floor), snapshot, install thread-local.
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "64");
    let limits = ParseLimits::snapshot();
    limits.install_thread_local();

    // Change env var after install — current_regex_limit must still return 64.
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "1024");
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    let current = ParseLimits::current_regex_limit();
    ParseLimits::clear_thread_local();

    assert_eq!(
        current, 64,
        "F-HIGH-001: current_regex_limit() must return snapshotted 64, got {current}"
    );
}

/// F-HIGH-001: `PrismQlParser::parse` enforces the regex limit from the snapshot
/// even if the env var is changed between the snapshot and the actual parse.
///
/// We test this indirectly: set PRISM_MAX_REGEX_PATTERN_LEN=64, then parse a
/// regex pattern of exactly 65 bytes. Without snapshot propagation, if the env
/// var is reset to 1024 before `RegexLiteral::new` runs, the pattern would be
/// accepted. With snapshot propagation via thread_local, it must be rejected.
///
/// Traces: F-HIGH-001, BC-2.11.006
#[test]
fn test_parse_limits_snapshot_propagates_to_regex_pattern_guard() {
    use crate::{ast::RegexLiteral, security::ParseLimits};

    let _guard = ENV_MUTEX.lock().unwrap();

    // Install a snapshot with regex_pattern = 64 (MIN_SAFE).
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "64");
    let limits = ParseLimits::snapshot();
    limits.install_thread_local();

    // Immediately reset env var to 1024 — without thread_local propagation,
    // RegexLiteral::new would read 1024 and accept the 65-byte pattern.
    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "1024");
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    // A 65-byte pattern exceeds the snapshotted limit of 64.
    let pattern = "a".repeat(65);
    let result = RegexLiteral::new(&pattern);
    ParseLimits::clear_thread_local();

    assert!(
        result.is_err(),
        "F-HIGH-001: 65-byte regex pattern must be rejected by snapshotted limit 64; got Ok"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("E-QUERY-003"),
        "F-HIGH-001: error must contain E-QUERY-003, got: {err}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-MEDIUM-002: Thread-local cleared on panic unwind (Drop guard)
// ─────────────────────────────────────────────────────────────────────────────

/// F-MEDIUM-002: `PrismQlParser::parse` uses a Drop guard so the thread-local
/// `ParseLimits` snapshot is cleared even when the inner parser panics.
///
/// We verify this by:
/// 1. Manually installing a thread-local snapshot (simulating the state at the
///    moment of a panic mid-parse).
/// 2. Running `catch_unwind` over a closure that panics after the guard drops.
/// 3. Asserting that `current_regex_limit()` returns the env-var fallback after
///    unwinding — confirming the thread-local was cleared.
///
/// Additionally, `PrismQlParser::parse` itself is panic-safe: a forced panic
/// during parsing leaves no thread-local residue.
///
/// Traces: F-MEDIUM-002, BC-2.11.006
#[test]
fn test_thread_local_cleared_on_panic() {
    use crate::{
        filter_parser::ThreadLocalGuard,
        security::{ParseLimits, PRISM_MAX_REGEX_PATTERN_LEN},
    };

    let _guard = ENV_MUTEX.lock().unwrap();

    // Ensure no thread-local is installed before we start.
    ParseLimits::clear_thread_local();

    // Part 1: Verify the PRODUCTION Drop guard clears on panic unwind.
    //
    // This test imports `crate::filter_parser::ThreadLocalGuard` — the same type
    // used inside `PrismQlParser::parse` — rather than defining a local copy.
    // If someone deletes or breaks the production guard, this test will fail.
    // (F-MEDIUM-001: local copy would be a false-negative.)
    //
    // Install a sentinel value (99 — below MIN_SAFE, but we set it directly
    // to test the guard, not the clamp logic).
    let mut sentinel = ParseLimits::snapshot();
    sentinel.regex_pattern = 99;
    ParseLimits::install_thread_local(&sentinel);

    // Confirm it's installed.
    assert_eq!(
        ParseLimits::current_regex_limit(),
        99,
        "F-MEDIUM-002: sentinel must be installed before panic"
    );

    // catch_unwind — production ThreadLocalGuard drops on unwind, clearing the thread-local.
    let _ = std::panic::catch_unwind(|| {
        let _drop_guard = ThreadLocalGuard;
        panic!("forced panic to test production Drop guard");
    });

    // Thread-local must be None after the panic unwind (guard ran on drop).
    // current_regex_limit() falls back to env-var path when no snapshot installed.
    let after_panic = ParseLimits::current_regex_limit();
    assert_eq!(
        after_panic, PRISM_MAX_REGEX_PATTERN_LEN,
        "F-MEDIUM-002: thread-local must be cleared after panic unwind; \
         got {after_panic}, expected default {PRISM_MAX_REGEX_PATTERN_LEN}"
    );

    // Part 2: PrismQlParser::parse is itself panic-safe — no thread-local residue.
    //
    // Normal parsing of valid/invalid input must not leave a thread-local installed.
    // (The Drop guard in PrismQlParser::parse handles this.)
    let _ = crate::filter_parser::PrismQlParser::parse("host = \"example.com\"");
    let _ = crate::filter_parser::PrismQlParser::parse("INVALID @@@ QUERY");

    let after_parse = ParseLimits::current_regex_limit();
    assert_eq!(
        after_parse, PRISM_MAX_REGEX_PATTERN_LEN,
        "F-MEDIUM-002: thread-local must be None after PrismQlParser::parse returns; \
         got {after_parse}"
    );
}

/// F-MEDIUM-001: Production `ThreadLocalGuard` Drop clears the thread-local on normal drop.
///
/// This test constructs the production `ThreadLocalGuard` from `filter_parser`
/// (not a local copy), installs a snapshot, then drops the guard and verifies
/// the thread-local is cleared. If the production guard is deleted or its Drop
/// impl is broken, this test fails — a local-copy test would not catch that.
///
/// Traces: F-MEDIUM-001, F-MEDIUM-002, BC-2.11.006
#[test]
fn test_production_thread_local_guard_clears_on_drop() {
    use crate::{
        filter_parser::ThreadLocalGuard,
        security::{ParseLimits, PRISM_MAX_REGEX_PATTERN_LEN},
    };

    let _env_guard = ENV_MUTEX.lock().unwrap();

    // Clear any residue from prior tests.
    ParseLimits::clear_thread_local();

    // Step 1: Install a sentinel snapshot (regex_pattern = 42).
    let mut sentinel = ParseLimits::snapshot();
    sentinel.regex_pattern = 42;
    ParseLimits::install_thread_local(&sentinel);

    // Verify the thread-local reports the sentinel value (not the env-var fallback).
    assert_eq!(
        ParseLimits::current_regex_limit(),
        42,
        "F-MEDIUM-001: sentinel must be visible before guard drop"
    );

    // Step 2: Construct and explicitly drop the production ThreadLocalGuard.
    {
        let _guard = ThreadLocalGuard;
        // _guard drops here — Drop calls ParseLimits::clear_thread_local().
    }

    // Step 3: Thread-local must now be None; current_regex_limit falls back to env-var default.
    let after_drop = ParseLimits::current_regex_limit();
    assert_eq!(
        after_drop, PRISM_MAX_REGEX_PATTERN_LEN,
        "F-MEDIUM-001: production ThreadLocalGuard::drop must clear thread-local; \
         got {after_drop}, expected default {PRISM_MAX_REGEX_PATTERN_LEN}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-LOW-001: Boundary tests for all clamp pairs — MIN-1, MIN, MAX, MAX+1
// ─────────────────────────────────────────────────────────────────────────────

/// F-LOW-001: PRISM_MAX_QUERY_SIZE=1023 (MIN-1) must clamp UP to MIN_SAFE_QUERY_SIZE (1024).
///
/// Traces: F-LOW-001, BC-2.11.006, EC-001
#[test]
fn test_clamp_query_size_below_min_clamped_up() {
    use crate::security::{effective_query_size_limit, MIN_SAFE_QUERY_SIZE};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_QUERY_SIZE", "1023");
    let limit = effective_query_size_limit();
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");

    assert_eq!(
        limit, MIN_SAFE_QUERY_SIZE,
        "F-LOW-001: PRISM_MAX_QUERY_SIZE=1023 (MIN-1) must clamp to {MIN_SAFE_QUERY_SIZE}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_QUERY_SIZE=1024 (MIN exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006, EC-001
#[test]
fn test_clamp_query_size_at_min_accepted() {
    use crate::security::{effective_query_size_limit, MIN_SAFE_QUERY_SIZE};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_QUERY_SIZE", "1024");
    let limit = effective_query_size_limit();
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");

    assert_eq!(
        limit, 1024,
        "F-LOW-001: PRISM_MAX_QUERY_SIZE=1024 (MIN exact) must be accepted as 1024, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_QUERY_SIZE=1048576 (MAX exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006, EC-001
#[test]
fn test_clamp_query_size_at_max_accepted() {
    use crate::security::{effective_query_size_limit, MAX_SAFE_QUERY_SIZE};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_QUERY_SIZE", "1048576");
    let limit = effective_query_size_limit();
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");

    assert_eq!(
        limit,
        MAX_SAFE_QUERY_SIZE,
        "F-LOW-001: PRISM_MAX_QUERY_SIZE=1048576 (MAX exact) must be accepted as {MAX_SAFE_QUERY_SIZE}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_QUERY_SIZE=1048577 (MAX+1) must clamp DOWN to MAX_SAFE_QUERY_SIZE (1048576).
///
/// Traces: F-LOW-001, BC-2.11.006, EC-001
#[test]
fn test_clamp_query_size_above_max_clamped_down() {
    use crate::security::{effective_query_size_limit, MAX_SAFE_QUERY_SIZE};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_QUERY_SIZE", "1048577");
    let limit = effective_query_size_limit();
    std::env::remove_var("PRISM_MAX_QUERY_SIZE");

    assert_eq!(
        limit, MAX_SAFE_QUERY_SIZE,
        "F-LOW-001: PRISM_MAX_QUERY_SIZE=1048577 (MAX+1) must clamp to {MAX_SAFE_QUERY_SIZE}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_NESTING_DEPTH=7 (MIN-1) must clamp UP to MIN_SAFE_NESTING_DEPTH (8).
///
/// Traces: F-LOW-001, BC-2.11.006, EC-002
#[test]
fn test_clamp_nesting_depth_below_min_clamped_up() {
    use crate::security::{effective_nesting_depth_limit, MIN_SAFE_NESTING_DEPTH};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "7");
    let limit = effective_nesting_depth_limit();
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert_eq!(
        limit, MIN_SAFE_NESTING_DEPTH,
        "F-LOW-001: PRISM_MAX_NESTING_DEPTH=7 (MIN-1) must clamp to {MIN_SAFE_NESTING_DEPTH}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_NESTING_DEPTH=8 (MIN exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006, EC-002
#[test]
fn test_clamp_nesting_depth_at_min_accepted() {
    use crate::security::effective_nesting_depth_limit;

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "8");
    let limit = effective_nesting_depth_limit();
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert_eq!(
        limit, 8,
        "F-LOW-001: PRISM_MAX_NESTING_DEPTH=8 (MIN exact) must be accepted as 8, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_NESTING_DEPTH=256 (MAX exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006, EC-002
#[test]
fn test_clamp_nesting_depth_at_max_accepted() {
    use crate::security::{effective_nesting_depth_limit, MAX_SAFE_NESTING_DEPTH};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "256");
    let limit = effective_nesting_depth_limit();
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert_eq!(
        limit, MAX_SAFE_NESTING_DEPTH,
        "F-LOW-001: PRISM_MAX_NESTING_DEPTH=256 (MAX exact) must be accepted as {MAX_SAFE_NESTING_DEPTH}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_NESTING_DEPTH=257 (MAX+1) must clamp DOWN to MAX_SAFE_NESTING_DEPTH (256).
///
/// Traces: F-LOW-001, BC-2.11.006, EC-002
#[test]
fn test_clamp_nesting_depth_above_max_clamped_down() {
    use crate::security::{effective_nesting_depth_limit, MAX_SAFE_NESTING_DEPTH};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_NESTING_DEPTH", "257");
    let limit = effective_nesting_depth_limit();
    std::env::remove_var("PRISM_MAX_NESTING_DEPTH");

    assert_eq!(
        limit, MAX_SAFE_NESTING_DEPTH,
        "F-LOW-001: PRISM_MAX_NESTING_DEPTH=257 (MAX+1) must clamp to {MAX_SAFE_NESTING_DEPTH}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_LIST_ITEMS=15 (MIN-1) must clamp UP to MIN_SAFE_LIST_ITEMS (16).
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_list_items_below_min_clamped_up() {
    use crate::security::{effective_list_items_limit, MIN_SAFE_LIST_ITEMS};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_LIST_ITEMS", "15");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert_eq!(
        limit, MIN_SAFE_LIST_ITEMS,
        "F-LOW-001: PRISM_MAX_LIST_ITEMS=15 (MIN-1) must clamp to {MIN_SAFE_LIST_ITEMS}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_LIST_ITEMS=16 (MIN exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_list_items_at_min_accepted() {
    use crate::security::effective_list_items_limit;

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_LIST_ITEMS", "16");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert_eq!(
        limit, 16,
        "F-LOW-001: PRISM_MAX_LIST_ITEMS=16 (MIN exact) must be accepted as 16, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_LIST_ITEMS=16384 (MAX exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_list_items_at_max_accepted() {
    use crate::security::{effective_list_items_limit, MAX_SAFE_LIST_ITEMS};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_LIST_ITEMS", "16384");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert_eq!(
        limit, MAX_SAFE_LIST_ITEMS,
        "F-LOW-001: PRISM_MAX_LIST_ITEMS=16384 (MAX exact) must be accepted as {MAX_SAFE_LIST_ITEMS}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_LIST_ITEMS=16385 (MAX+1) must clamp DOWN to MAX_SAFE_LIST_ITEMS (16384).
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_list_items_above_max_clamped_down() {
    use crate::security::{effective_list_items_limit, MAX_SAFE_LIST_ITEMS};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_LIST_ITEMS", "16385");
    let limit = effective_list_items_limit();
    std::env::remove_var("PRISM_MAX_LIST_ITEMS");

    assert_eq!(
        limit, MAX_SAFE_LIST_ITEMS,
        "F-LOW-001: PRISM_MAX_LIST_ITEMS=16385 (MAX+1) must clamp to {MAX_SAFE_LIST_ITEMS}, got {limit}"
    );
}

/// F-LOW-001 / OBS-001: PRISM_MAX_PIPE_STAGES=3 (MIN-1 after OBS-001 floor=4)
/// must clamp UP to MIN_SAFE_PIPE_STAGES (4).
///
/// Traces: F-LOW-001, OBS-001, BC-2.11.006
#[test]
fn test_clamp_pipe_stages_below_min_clamped_up() {
    use crate::security::{effective_pipe_stage_limit, MIN_SAFE_PIPE_STAGES};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_PIPE_STAGES", "3");
    let limit = effective_pipe_stage_limit();
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");

    assert_eq!(
        limit, MIN_SAFE_PIPE_STAGES,
        "F-LOW-001/OBS-001: PRISM_MAX_PIPE_STAGES=3 (MIN-1) must clamp to {MIN_SAFE_PIPE_STAGES}, got {limit}"
    );
}

/// F-LOW-001 / OBS-001: PRISM_MAX_PIPE_STAGES=4 (MIN exact after OBS-001 floor=4)
/// must be accepted as-is.
///
/// Traces: F-LOW-001, OBS-001, BC-2.11.006
#[test]
fn test_clamp_pipe_stages_at_min_accepted() {
    use crate::security::effective_pipe_stage_limit;

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_PIPE_STAGES", "4");
    let limit = effective_pipe_stage_limit();
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");

    assert_eq!(
        limit, 4,
        "F-LOW-001/OBS-001: PRISM_MAX_PIPE_STAGES=4 (MIN exact) must be accepted as 4, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_PIPE_STAGES=256 (MAX exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_pipe_stages_at_max_accepted() {
    use crate::security::{effective_pipe_stage_limit, MAX_SAFE_PIPE_STAGES};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_PIPE_STAGES", "256");
    let limit = effective_pipe_stage_limit();
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");

    assert_eq!(
        limit, MAX_SAFE_PIPE_STAGES,
        "F-LOW-001: PRISM_MAX_PIPE_STAGES=256 (MAX exact) must be accepted as {MAX_SAFE_PIPE_STAGES}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_PIPE_STAGES=257 (MAX+1) must clamp DOWN to MAX_SAFE_PIPE_STAGES (256).
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_pipe_stages_above_max_clamped_down() {
    use crate::security::{effective_pipe_stage_limit, MAX_SAFE_PIPE_STAGES};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_PIPE_STAGES", "257");
    let limit = effective_pipe_stage_limit();
    std::env::remove_var("PRISM_MAX_PIPE_STAGES");

    assert_eq!(
        limit, MAX_SAFE_PIPE_STAGES,
        "F-LOW-001: PRISM_MAX_PIPE_STAGES=257 (MAX+1) must clamp to {MAX_SAFE_PIPE_STAGES}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=63 (MIN-1) must clamp UP to
/// MIN_SAFE_REGEX_PATTERN_LEN (64).
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_regex_pattern_below_min_clamped_up() {
    use crate::security::{effective_regex_pattern_length_limit, MIN_SAFE_REGEX_PATTERN_LEN};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "63");
    let limit = effective_regex_pattern_length_limit();
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    assert_eq!(
        limit, MIN_SAFE_REGEX_PATTERN_LEN,
        "F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=63 (MIN-1) must clamp to {MIN_SAFE_REGEX_PATTERN_LEN}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=64 (MIN exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_regex_pattern_at_min_accepted() {
    use crate::security::effective_regex_pattern_length_limit;

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "64");
    let limit = effective_regex_pattern_length_limit();
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    assert_eq!(
        limit, 64,
        "F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=64 (MIN exact) must be accepted as 64, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=65536 (MAX exact) must be accepted as-is.
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_regex_pattern_at_max_accepted() {
    use crate::security::{effective_regex_pattern_length_limit, MAX_SAFE_REGEX_PATTERN_LEN};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "65536");
    let limit = effective_regex_pattern_length_limit();
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    assert_eq!(
        limit, MAX_SAFE_REGEX_PATTERN_LEN,
        "F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=65536 (MAX exact) must be accepted as {MAX_SAFE_REGEX_PATTERN_LEN}, got {limit}"
    );
}

/// F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=65537 (MAX+1) must clamp DOWN to
/// MAX_SAFE_REGEX_PATTERN_LEN (65536).
///
/// Traces: F-LOW-001, BC-2.11.006
#[test]
fn test_clamp_regex_pattern_above_max_clamped_down() {
    use crate::security::{effective_regex_pattern_length_limit, MAX_SAFE_REGEX_PATTERN_LEN};

    let _guard = ENV_MUTEX.lock().unwrap();

    std::env::set_var("PRISM_MAX_REGEX_PATTERN_LEN", "65537");
    let limit = effective_regex_pattern_length_limit();
    std::env::remove_var("PRISM_MAX_REGEX_PATTERN_LEN");

    assert_eq!(
        limit, MAX_SAFE_REGEX_PATTERN_LEN,
        "F-LOW-001: PRISM_MAX_REGEX_PATTERN_LEN=65537 (MAX+1) must clamp to {MAX_SAFE_REGEX_PATTERN_LEN}, got {limit}"
    );
}

/// P5-02 (QRY cascade, error-taxonomy.md §E-QUERY-003 / ADR-038 §P5-02): security-limit
/// errors carry exactly ONE "E-QUERY-003" prefix in their Display output.
///
/// Pre-split, security limits routed through `QueryExecutionFailed` with an
/// embedded "E-QUERY-003: " prefix in `detail`, producing a double-prefixed
/// display (the code token appeared twice, wrapped in the generic
/// execution-error text). The dedicated `QuerySecurityLimitExceeded` variant
/// supplies the single canonical prefix via its Display impl; emission sites
/// no longer embed it.
///
/// Traces: P5-02, BC-2.11.006, error-taxonomy.md §E-QUERY-003/§E-QUERY-034
#[test]
fn test_p5_02_security_limit_display_has_exactly_one_e_query_003_prefix() {
    use crate::security::{check_query_size, PRISM_MAX_QUERY_SIZE};

    let oversized = "a".repeat(PRISM_MAX_QUERY_SIZE + 1);
    let err = check_query_size(&oversized)
        .expect_err("P5-02: oversized query must be rejected by check_query_size");
    assert!(
        matches!(
            err,
            prism_core::error::PrismError::QuerySecurityLimitExceeded { .. }
        ),
        "P5-02: security-limit rejection must use QuerySecurityLimitExceeded, got: {err:?}"
    );

    let display = err.to_string();
    assert!(
        display.starts_with("E-QUERY-003: "),
        "P5-02: Display must start with the canonical E-QUERY-003 prefix, got: {display}"
    );
    assert_eq!(
        display.matches("E-QUERY-003").count(),
        1,
        "P5-02: Display must contain exactly ONE E-QUERY-003 token (no double prefix), got: {display}"
    );
    assert!(
        !display.contains("query execution error"),
        "P5-02: security-limit Display must not route through the generic \
         E-QUERY-034 execution-error wrapper, got: {display}"
    );
}

/// P6-01: `map_datafusion_memory_error` non-`ResourcesExhausted` fallback arm must
/// NOT embed the phrase "query execution error" in `detail`.
///
/// The `QueryExecutionFailed` Display impl is:
///   `"E-QUERY-034: query execution error: {detail}"`
///
/// Prior to the fix the fallback arm set:
///   `detail: "query execution error: <redacted; see server logs>"`
/// producing the double-phrase stutter:
///   `"E-QUERY-034: query execution error: query execution error: <redacted; see server logs>"`
///
/// After the fix the detail is simply `"<redacted; see server logs>"`, rendering:
///   `"E-QUERY-034: query execution error: <redacted; see server logs>"`
/// — exactly ONE occurrence of "query execution error" in the Display output.
///
/// Traces: P6-01, BC-2.11.006, error-taxonomy.md §E-QUERY-034
#[test]
fn test_p6_01_map_datafusion_memory_error_fallback_display_has_exactly_one_query_execution_error_phrase(
) {
    use crate::memory::map_datafusion_memory_error;
    use datafusion::error::DataFusionError;

    // A generic DataFusion error that is NOT ResourcesExhausted — exercises the
    // fallback arm that produces QueryExecutionFailed.
    let generic_err = DataFusionError::Plan("test plan error".to_string());

    let err = map_datafusion_memory_error(generic_err, 200 * 1024 * 1024);

    assert!(
        matches!(
            err,
            prism_core::error::PrismError::QueryExecutionFailed { .. }
        ),
        "P6-01: non-ResourcesExhausted DataFusion error must map to QueryExecutionFailed, got: {err:?}"
    );

    let display = err.to_string();

    // The display MUST start with the E-QUERY-034 prefix.
    assert!(
        display.starts_with("E-QUERY-034:"),
        "P6-01: Display must start with E-QUERY-034 prefix, got: {display}"
    );

    // There must be exactly ONE occurrence of "query execution error" — no stutter.
    let count = display.matches("query execution error").count();
    assert_eq!(
        count, 1,
        "P6-01: Display must contain exactly ONE occurrence of 'query execution error' (no double-phrase stutter); \
         got {count} occurrence(s): {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P3-CRIT-NEW-001 regression tests — char-boundary panic in
// `parse_interval_duration_str` (split_at(s.len()-1) on multi-byte input)
//
// `parse_interval_duration_str` does `s.split_at(s.len() - 1)` as its first
// split operation.  When the last character of the INTERVAL content is a
// multi-byte UTF-8 sequence, `s.len() - 1` lands inside that sequence — a
// byte index that is NOT a char boundary — causing an unconditional panic.
//
// Reachable from all three public parse modes (filter / SQL WHERE / pipe
// where) via `build_temporal_rhs_parser`'s `interval_content` parser, which
// accepts `none_of('\'').repeated()` (any non-quote, including multi-byte).
//
// These tests MUST panic (or otherwise fail) BEFORE the fix and pass after.
// Each calls `PrismQlParser::parse` with a real INTERVAL expression so that
// the production parse path — including the grammar-level parser and
// `parse_interval_duration_str` — is exercised.
// ---------------------------------------------------------------------------

/// F-P3-CRIT-NEW-001a: INTERVAL with a single trailing 2-byte multi-byte char.
///
/// `é` (U+00E9) is 2 UTF-8 bytes. `s.split_at(s.len() - 1) = split_at(1)` lands
/// inside the 2-byte sequence → char-boundary panic in the unpatched code.
/// Post-fix: must return `Err` with a structured E-QUERY-001 message, never panic.
#[test]
fn test_f_p3_crit_new_001a_interval_trailing_multibyte_no_panic() {
    // `é` is U+00E9: two bytes, so s.len()-1 == 1 which is NOT a char boundary.
    let query = "event_time > INTERVAL 'é'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-NEW-001a: PrismQlParser::parse panicked on INTERVAL with trailing 2-byte char 'é'"
    );
    // Post-fix: must be Err with a structured message (not a panic-converted Ok).
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-NEW-001a: expected Err for invalid INTERVAL 'é', got Ok"
            );
            let errs = parse_result.unwrap_err();
            let msg = errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            assert!(
                msg.contains("E-QUERY-001"),
                "F-P3-CRIT-NEW-001a: error must contain E-QUERY-001, got: {msg}"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

/// F-P3-CRIT-NEW-001b: INTERVAL inside NOW() arithmetic with multi-byte content.
///
/// `café` ends with `é` (U+00E9, 2 bytes). `s.len()-1` lands inside that byte
/// sequence → panic.  Post-fix: must return `Err`, never panic.
#[test]
fn test_f_p3_crit_new_001b_interval_in_now_arithmetic_multibyte_no_panic() {
    let query = "event_time > NOW() - INTERVAL 'café'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-NEW-001b: PrismQlParser::parse panicked on NOW() - INTERVAL 'café'"
    );
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-NEW-001b: expected Err for invalid INTERVAL 'café', got Ok"
            );
            let errs = parse_result.unwrap_err();
            let msg = errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            assert!(
                msg.contains("E-QUERY-001"),
                "F-P3-CRIT-NEW-001b: error must contain E-QUERY-001, got: {msg}"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

/// F-P3-CRIT-NEW-001c: INTERVAL with a 3-byte trailing multi-byte char.
///
/// `€` (U+20AC) is 3 UTF-8 bytes. Both `s.len()-1` (2) and `s.len()-2` (1) are
/// NOT char boundaries → panic.  Post-fix: must return `Err`, never panic.
#[test]
fn test_f_p3_crit_new_001c_interval_trailing_3byte_multibyte_no_panic() {
    // `€` is U+20AC: 3 bytes. None of len-1, len-2 are char boundaries.
    let query = "event_time > INTERVAL '24€'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-NEW-001c: PrismQlParser::parse panicked on INTERVAL with 3-byte trailing char '€'"
    );
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-NEW-001c: expected Err for invalid INTERVAL '24€', got Ok"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

// ---------------------------------------------------------------------------
// F-P3-FRESH-CRIT-001 regression tests — numeric-magnitude overflow panics in
// `parse_interval_duration_str` (Site 1) and `inject_now_expr` (Site 2).
//
// VP-021 ("PrismQL parser: never panics on arbitrary input") is violated by:
//   Site 1: `value as i64` silently wraps when value > i64::MAX; then
//            `chrono::Duration::{seconds,minutes,hours,days}(wrapped_i64)` panics
//            when the resulting milliseconds representation overflows i64.
//   Site 2: `ts.instant - offset` / `ts.instant + offset` in `inject_now_expr`
//            panic on DateTime range overflow even when the Duration itself is
//            in-bounds (e.g. huge-but-representable seconds → underflows DateTime).
//
// chrono 0.4.44 bounds (verified from source + Cargo.lock):
//   max days  = 106_751_991_167       (overflow: 106_751_991_168)
//   max hours = 2_562_047_788_015     (overflow: 2_562_047_788_016)
//   max mins  = 153_722_867_280_912   (overflow: 153_722_867_280_913)
//   max secs  = 9_223_372_036_854_775 (overflow: 9_223_372_036_854_776)
//
// i64 cast-wrap: any u64 value > i64::MAX (9_223_372_036_854_775_807)
// wraps on `as i64` to a NEGATIVE value, which also violates Duration invariants.
//
// Each test calls `std::panic::catch_unwind` so that a panic does not abort the
// test binary — it shows up as Err from catch_unwind, which the test then fails
// with a descriptive assertion message (instead of a SIGABRT / test crash).
// ---------------------------------------------------------------------------

/// F-P3-FRESH-CRIT-001a: INTERVAL '106751991168d' — days magnitude exceeds
/// chrono::TimeDelta max. Must return Err(E-QUERY-001), never panic.
#[test]
fn test_f_p3_fresh_crit_001a_interval_days_overflow_no_panic() {
    // 106_751_991_168 days = max_days + 1; Duration::days panics on this.
    let query = "event_time > INTERVAL '106751991168d'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001a: PrismQlParser::parse panicked on INTERVAL days magnitude overflow"
    );
    let parse_result = result.unwrap();
    assert!(
        parse_result.is_err(),
        "F-P3-FRESH-CRIT-001a: expected Err for overflowing INTERVAL days, got Ok"
    );
    let errs = parse_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001a: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-FRESH-CRIT-001b: INTERVAL '2562047788016h' — hours magnitude exceeds
/// chrono::TimeDelta max. Must return Err(E-QUERY-001), never panic.
#[test]
fn test_f_p3_fresh_crit_001b_interval_hours_overflow_no_panic() {
    // 2_562_047_788_016 hours = max_hours + 1
    let query = "event_time > INTERVAL '2562047788016h'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001b: PrismQlParser::parse panicked on INTERVAL hours magnitude overflow"
    );
    let parse_result = result.unwrap();
    assert!(
        parse_result.is_err(),
        "F-P3-FRESH-CRIT-001b: expected Err for overflowing INTERVAL hours, got Ok"
    );
    let errs = parse_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001b: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-FRESH-CRIT-001c: INTERVAL '153722867280913m' — minutes magnitude exceeds
/// chrono::TimeDelta max. Must return Err(E-QUERY-001), never panic.
#[test]
fn test_f_p3_fresh_crit_001c_interval_minutes_overflow_no_panic() {
    // 153_722_867_280_913 minutes = max_minutes + 1
    let query = "event_time > INTERVAL '153722867280913m'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001c: PrismQlParser::parse panicked on INTERVAL minutes magnitude overflow"
    );
    let parse_result = result.unwrap();
    assert!(
        parse_result.is_err(),
        "F-P3-FRESH-CRIT-001c: expected Err for overflowing INTERVAL minutes, got Ok"
    );
    let errs = parse_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001c: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-FRESH-CRIT-001d: INTERVAL '9223372036854776s' — seconds magnitude exceeds
/// chrono::TimeDelta max. Must return Err(E-QUERY-001), never panic.
#[test]
fn test_f_p3_fresh_crit_001d_interval_seconds_overflow_no_panic() {
    // 9_223_372_036_854_776s = max_secs + 1; still fits in u64 and i64
    let query = "event_time > INTERVAL '9223372036854776s'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001d: PrismQlParser::parse panicked on INTERVAL seconds magnitude overflow"
    );
    let parse_result = result.unwrap();
    assert!(
        parse_result.is_err(),
        "F-P3-FRESH-CRIT-001d: expected Err for overflowing INTERVAL seconds, got Ok"
    );
    let errs = parse_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001d: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-FRESH-CRIT-001e: INTERVAL with value > i64::MAX (u64 overflow cast-wrap).
///
/// A u64 value of i64::MAX + 1 (= 9_223_372_036_854_775_808) cast to i64 wraps to
/// -9_223_372_036_854_775_808, which makes Duration::seconds(-i64::MAX-1) panic.
/// Must return Err(E-QUERY-001), never panic.
#[test]
fn test_f_p3_fresh_crit_001e_interval_value_exceeds_i64_max_no_panic() {
    // 9_223_372_036_854_775_808 = i64::MAX + 1: wraps to i64::MIN when cast via `as i64`
    let query = "event_time > INTERVAL '9223372036854775808s'";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001e: PrismQlParser::parse panicked on INTERVAL value > i64::MAX"
    );
    let parse_result = result.unwrap();
    assert!(
        parse_result.is_err(),
        "F-P3-FRESH-CRIT-001e: expected Err for INTERVAL value > i64::MAX, got Ok"
    );
    let errs = parse_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001e: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-FRESH-CRIT-001f: plan-time DateTime underflow via `inject_now_expr`.
///
/// `NOW() - INTERVAL '9223372036854775s'` (max representable Duration in seconds)
/// produces a valid Duration but when subtracted from the current DateTime (near
/// Unix epoch + some years) will underflow the DateTime range.
/// Must propagate a structured Err from `parse_and_plan`, never panic.
#[test]
fn test_f_p3_fresh_crit_001f_plan_time_datetime_underflow_no_panic() {
    // max_secs = 9_223_372_036_854_775s is within chrono::Duration bounds.
    // Subtracting this from a DateTime near "now" (year ~2026) will underflow the
    // DateTime representable range (min ~year -262,000), causing a panic in the
    // non-checked `ts.instant - offset` path.
    let query = "event_time > NOW() - INTERVAL '9223372036854775s'";
    let result = std::panic::catch_unwind(|| crate::parse_and_plan(query));
    assert!(
        result.is_ok(),
        "F-P3-FRESH-CRIT-001f: parse_and_plan panicked on DateTime underflow from max-Duration seconds"
    );
    let plan_result = result.unwrap();
    assert!(
        plan_result.is_err(),
        "F-P3-FRESH-CRIT-001f: expected Err for DateTime underflow, got Ok"
    );
    let errs = plan_result.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("E-QUERY-001"),
        "F-P3-FRESH-CRIT-001f: error must contain E-QUERY-001, got: {msg}"
    );
}

/// F-P3-CRIT-NEW-001d: Existing ASCII INTERVAL behavior must be preserved after fix.
///
/// `'24h'`, `'7d'`, `'1h'` are valid; `'bogus'` must be Err; `'999s'` must succeed.
/// This test guards against regressions introduced while fixing the char-boundary bug.
#[test]
fn test_f_p3_crit_new_001d_interval_ascii_behavior_preserved() {
    // Valid ASCII intervals must still parse to Ok(Ast).
    for valid in &[
        "event_time > INTERVAL '24h'",
        "event_time > INTERVAL '7d'",
        "event_time > INTERVAL '999s'",
        "event_time > INTERVAL '30m'",
    ] {
        let r = PrismQlParser::parse(valid);
        assert!(
            r.is_ok(),
            "F-P3-CRIT-NEW-001d: valid interval '{valid}' must parse Ok, got: {r:?}"
        );
    }

    // Invalid ASCII intervals must be Err (not panic).
    for invalid in &[
        "event_time > INTERVAL 'bogus'",
        "event_time > INTERVAL ''",
        "event_time > INTERVAL 'x'",
    ] {
        let result = std::panic::catch_unwind(|| PrismQlParser::parse(invalid));
        assert!(
            result.is_ok(),
            "F-P3-CRIT-NEW-001d: invalid interval '{invalid}' panicked (must return Err instead)"
        );
        if let Ok(r) = result {
            assert!(
                r.is_err(),
                "F-P3-CRIT-NEW-001d: invalid interval '{invalid}' must be Err, got Ok"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// F-P3-CRIT-001 integration smoke tests — char-boundary safety in the
// `is_enrich_missing_column_at` enrich-error path via PrismQlParser::parse.
//
// Root cause (fixed in error_recovery.rs): `rfind(|c: char| !c.is_alphanumeric() && c != '_')`
// returns the BYTE START of the matched char. For multibyte chars (e.g. `»` U+00BB = 2
// bytes, `—` U+2014 = 3 bytes), `i + 1` lands INSIDE the UTF-8 sequence →
// the subsequent byte-slice panics "byte index N is not a char boundary".
//
// Unit RED-gate tests live in error_recovery.rs tests module (direct calls with
// offset=input.len(), which expose the panic).  These integration tests verify
// the FULL parse path also produces Err without panic.
//
// Note on integration path: Chumsky's error offset for `"FROM t | enrich a»b"` is
// 17 (start of `»`), making prefix = "FROM t | enrich a" (ASCII). The unit tests in
// error_recovery.rs are the RED-gate because they call is_enrich_missing_column_at
// with offset=input.len() directly, exposing the multibyte char in the prefix.
// These integration tests serve as post-fix smoke tests confirming Err + no panic
// on the full PrismQlParser::parse path.
// ---------------------------------------------------------------------------

/// F-P3-CRIT-001a: pipe mode smoke test — `»` (U+00BB, 2 bytes) in enrich input.
///
/// `FROM t | enrich a»b` must return `Err` without panic via the full parse path.
/// (Unit RED-gate: error_recovery::tests::test_f_p3_crit_001_unit_a_two_byte_separator_no_panic)
#[test]
fn test_f_p3_crit_001a_enrich_multibyte_separator_2byte_no_panic() {
    let query = "FROM t | enrich a\u{00BB}b";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-001a: PrismQlParser::parse panicked on enrich with 2-byte separator '»'"
    );
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-001a: expected Err for invalid enrich 'a»b', got Ok"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

/// F-P3-CRIT-001b: pipe mode smoke test — `—` (U+2014 EM DASH, 3 bytes) in enrich input.
///
/// `FROM t | enrich x—y` must return `Err` without panic.
/// (Unit RED-gate: error_recovery::tests::test_f_p3_crit_001_unit_b_three_byte_separator_no_panic)
#[test]
fn test_f_p3_crit_001b_enrich_multibyte_separator_3byte_no_panic() {
    let query = "FROM t | enrich x\u{2014}y";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-001b: PrismQlParser::parse panicked on enrich with 3-byte separator '—'"
    );
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-001b: expected Err for invalid enrich 'x—y', got Ok"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

/// F-P3-CRIT-001c: SqlPipe mode smoke test — `»` (U+00BB) in enrich input via
/// the SqlPipe path (filter_parser.rs:485).
///
/// `SELECT * FROM t | enrich a»b` must return `Err` without panic.
/// (Unit RED-gate: error_recovery::tests::test_f_p3_crit_001_unit_a_two_byte_separator_no_panic)
#[test]
fn test_f_p3_crit_001c_enrich_multibyte_sqlpipe_path_no_panic() {
    let query = "SELECT * FROM t | enrich a\u{00BB}b";
    let result = std::panic::catch_unwind(|| PrismQlParser::parse(query));
    assert!(
        result.is_ok(),
        "F-P3-CRIT-001c: PrismQlParser::parse panicked on SqlPipe enrich with 2-byte separator '»'"
    );
    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "F-P3-CRIT-001c: expected Err for invalid SqlPipe enrich 'a»b', got Ok"
            );
        }
        Err(_) => unreachable!("caught unwind should have been Ok — panic was caught above"),
    }
}

/// F-P3-CRIT-001d: ASCII enrich behavior must be preserved after the fix.
///
/// `FROM t | enrich threat_score` must still produce the actionable
/// "enrich requires a column argument" guidance message (GRAMMAR-005).
#[test]
fn test_f_p3_crit_001d_enrich_ascii_behavior_preserved() {
    // Missing column — must be Err with enrich guidance message.
    let missing_col = "FROM t | enrich threat_score";
    let r = PrismQlParser::parse(missing_col);
    assert!(
        r.is_err(),
        "F-P3-CRIT-001d: expected Err for enrich without column, got Ok"
    );
    let errs = r.unwrap_err();
    let msg = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        msg.contains("enrich requires a column argument"),
        "F-P3-CRIT-001d: expected enrich guidance message, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PQLFN-PR11-OBS-001: regex-length and EC-004 canonical wrapped Display locks
// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.006 v1.20: two-layer MCP-observable Display is canonical for in-perimeter
// semantic parse errors wrapping security-limit inner codes.
//
// Defect 1 (regex span level): regex_match try_map was placed at the SEQUENCE
// level — span.start = field_path offset (36 for `hostname`).  Chumsky choice()
// error-merging uses the highest span.start; cidr_match's "expected keyword 'IN'"
// error at span.start=45 (the `M` of `MATCHES`) outcompeted the E-QUERY-003
// error at span.start=36, suppressing the real cause.
//
// Defect 2 (EC-004 dead code): source_ref segment grammar ([a-zA-Z0-9_]+ sep '.')
// excluded '/', '\', and adjacent '.' at the lexical level, so the try_map EC-004
// check was unreachable.  Path-traversal inputs failed with a structural error
// instead of the canonical EC-004 message.
//
// Fix reference: DEFECT-PQL-FNCALL-LHS-001 fix-burst-42.
// ─────────────────────────────────────────────────────────────────────────────

/// F-PQLFN-PR11-OBS-001 **RED → GREEN** — regex-length canonical wrapped Display lock.
///
/// Query: `FROM crowdstrike_detections | where hostname MATCHES "<1025 'a's>"`
///
/// After the fix (try_map placed at LITERAL level in `regex_match`):
/// - `regex_literal = string_val.padded().try_map(…)` span.start = 53 (opening `"`)
/// - 53 > cidr_match's span.start ≈ 45 → E-QUERY-003 wins in Chumsky choice()
/// - Canonical MCP-observable Display (BC-2.11.006 v1.20):
///   `"E-QUERY-001: query parse error at offset 53: E-QUERY-003: regex pattern
///    length 1025 bytes exceeds maximum allowed 1024 bytes"`
///
/// **RED state** (current try_map at sequence level):
/// - span.start = 36 (field_path `hostname`) for regex_match; cidr_match wins at ≈45
/// - Error is "expected keyword 'IN'" at wrong offset (not E-QUERY-003)
///
/// Offset derivation (query = `FROM crowdstrike_detections | where hostname MATCHES "…"`):
///   F(0)…M(3) (4)c(5)…s(26) (27)|(28) (29)w(30)…e(34) (35)h(36)…e(43) (44)M(45)…S(51) (52)"(53)
///
/// Traces: F-PQLFN-PR11-OBS-001; BC-2.11.006 v1.20; DEFECT-PQL-FNCALL-LHS-001.
#[test]
fn test_f_pqlfn_pr11_obs_001_regex_length_wrapped_display_lock() {
    let pattern = "a".repeat(1025);
    let query = format!("FROM crowdstrike_detections | where hostname MATCHES \"{pattern}\"");

    let result = PrismQlParser::parse(&query);
    assert!(
        result.is_err(),
        "regex_length_display_lock: must parse as Err"
    );

    let errs = result.unwrap_err();
    assert!(
        !errs.is_empty(),
        "regex_length_display_lock: must have at least one error"
    );
    let first = &errs[0];

    // Simulate ADR-048 §D.7.2 materialization.rs detail computation:
    let detail = if first.semantic {
        let msg = &first.message;
        msg.strip_prefix("E-QUERY-001: ").unwrap_or(msg).to_string()
    } else {
        first.to_string()
    };
    let wrapped = format!(
        "E-QUERY-001: query parse error at offset {}: {}",
        first.offset, detail
    );

    // BC-2.11.006 v1.20 canonical two-layer form (Option B ratified):
    let expected = "E-QUERY-001: query parse error at offset 53: \
         E-QUERY-003: regex pattern length 1025 bytes exceeds maximum allowed 1024 bytes";
    assert_eq!(
        wrapped, expected,
        "F-PQLFN-PR11-OBS-001 regex length: canonical wrapped Display mismatch.\n\
         RED state: cidr_match wins at span.start≈45 (wrong — 'expected IN' error).\n\
         GREEN state: try_map at literal level gives span.start=53 > 45 → E-QUERY-003 wins."
    );
}

/// F-PQLFN-PR11-OBS-001 **RED → GREEN** — EC-004 SourceRef traversal canonical wrapped Display lock.
///
/// Query: `FROM crowdstrike/../detections | where x = "1"`
///
/// After the fix (widened source_ref grammar — wide capture + EC-004 try_map):
/// - source_ref captures `crowdstrike/../detections` at offset 5 (after `FROM `)
/// - EC-004 fires at span.start = 5
/// - Canonical MCP-observable Display (BC-2.11.006 v1.20):
///   `"E-QUERY-001: query parse error at offset 5: EC-004: SourceRef contains
///    path traversal characters ('..', '/', '\')"`
///
/// **RED state** (current narrow segment grammar):
/// - source_ref grammar `[a-zA-Z0-9_]+` sep-by `.` captures only `crowdstrike`
///   (stops at `/`); EC-004 is unreachable dead code; error is structural (wrong offset)
///
/// Traces: F-PQLFN-PR11-OBS-001; BC-2.11.006 v1.20 EC-004; DEFECT-PQL-FNCALL-LHS-001.
#[test]
fn test_f_pqlfn_pr11_obs_001_sourceref_traversal_wrapped_display_lock() {
    // Source `crowdstrike/../detections` starts at offset 5 (after `FROM `).
    let query = "FROM crowdstrike/../detections | where x = \"1\"";

    let result = PrismQlParser::parse(query);
    assert!(result.is_err(), "ec_004_display_lock: must parse as Err");

    let errs = result.unwrap_err();
    assert!(
        !errs.is_empty(),
        "ec_004_display_lock: must have at least one error"
    );
    let first = &errs[0];

    // Simulate ADR-048 §D.7.2 materialization.rs detail computation:
    let detail = if first.semantic {
        let msg = &first.message;
        msg.strip_prefix("E-QUERY-001: ").unwrap_or(msg).to_string()
    } else {
        first.to_string()
    };
    let wrapped = format!(
        "E-QUERY-001: query parse error at offset {}: {}",
        first.offset, detail
    );

    // BC-2.11.006 v1.20 canonical two-layer form (Option B ratified):
    let expected = "E-QUERY-001: query parse error at offset 5: \
         EC-004: SourceRef contains path traversal characters ('..', '/', '\\')";
    assert_eq!(
        wrapped, expected,
        "F-PQLFN-PR11-OBS-001 EC-004: canonical wrapped Display mismatch.\n\
         RED state: narrow segment grammar truncates to 'crowdstrike' → structural error at \
         wrong offset (EC-004 unreachable dead code).\n\
         GREEN state: wide capture + EC-004 try_map fires at span.start=5."
    );
}
