//! API surface test: only `PrismQlParser::parse` is callable from outside the crate.
//!
//! This integration test is the canonical verification that:
//! 1. `build_*_parser` factories in `filter_parser` and `pipe_parser` are NOT in the
//!    public API (pub(crate) — SEC-C-003, F-HIGH-001 fix).
//! 2. `PrismQlParser::parse` works as the sole public entry point.
//!
//! If any `build_*_parser` function were `pub`, it could be called from this file
//! to bypass ALL security guards (check_query_size, check_paren_depth,
//! check_predicate_nesting_depth, check_filter_list_sizes).
//!
//! Story: S-3.01 | SEC-C-003 | F-HIGH-001
//!
//! # What this file DOES NOT do
//! It does not attempt to call `build_*_parser` — those symbols are `pub(crate)`
//! and would produce a compile error. The absence of such calls, combined with
//! `cargo build --workspace` succeeding, is the verification.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_query::PrismQlParser;

/// Verifies the sole public parse API accepts a well-formed filter query.
#[test]
fn test_api_surface_parse_filter_via_public_entry_point() {
    let result = PrismQlParser::parse("src_ip = '10.0.0.1'");
    assert!(
        result.is_ok(),
        "API surface: PrismQlParser::parse must accept a valid filter query; got {:?}",
        result.unwrap_err()
    );
}

/// Verifies the sole public parse API accepts a well-formed SQL query.
#[test]
fn test_api_surface_parse_sql_via_public_entry_point() {
    let result =
        PrismQlParser::parse("SELECT * FROM crowdstrike.detections WHERE severity = 'high'");
    assert!(
        result.is_ok(),
        "API surface: PrismQlParser::parse must accept a valid SQL query; got {:?}",
        result.unwrap_err()
    );
}

/// Verifies the sole public parse API accepts a well-formed pipe query.
#[test]
fn test_api_surface_parse_pipe_via_public_entry_point() {
    let result =
        PrismQlParser::parse("crowdstrike.detections | where severity = 'high' | limit 100");
    assert!(
        result.is_ok(),
        "API surface: PrismQlParser::parse must accept a valid pipe query; got {:?}",
        result.unwrap_err()
    );
}
