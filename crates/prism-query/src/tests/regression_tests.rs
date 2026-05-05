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
        effective_list_items_limit, effective_nesting_depth_limit, effective_query_size_limit,
        PRISM_MAX_LIST_ITEMS, PRISM_MAX_NESTING_DEPTH, PRISM_MAX_QUERY_SIZE,
    },
    sql_parser::parse_sql,
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
}

/// B-2: SQL WHERE with mixed AND/OR forcing 65-deep nesting must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_sql_or_mix_depth_65_rejected() {
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
}

/// B-2: Pipe `where` with 65 chained NOT predicates must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_pipe_where_not_chain_depth_65_rejected() {
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
}

/// B-2: Pipe `where` with deeply nested parens exceeding depth 64 must be rejected.
///
/// Traces: B-2, BC-2.11.006, DI-019
#[test]
fn test_BC_2_11_006_pipe_where_subquery_depth_65_rejected() {
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
    use crate::ast::{FieldPath, Literal};
    use crate::visit::{walk_ast, Visitor};

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
    use crate::ast::{FilterExpr, Predicate, SourceRef};
    use crate::visit::{walk_filter_expr, Visitor};

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
    use crate::ast::{FieldPath, LogicalOp, Predicate, Span};
    use crate::visit::{walk_predicate, Visitor};

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
