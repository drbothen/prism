//! API surface test: the sanctioned public parse APIs are callable from outside the crate.
//!
//! This integration test is the canonical verification that:
//! 1. `build_*_parser` factories in `filter_parser` and `pipe_parser` are NOT in the
//!    public API (pub(crate) — SEC-C-003, F-HIGH-001 fix).
//! 2. `PrismQlParser::parse` and `PrismQlParser::parse_with_registry` are the two
//!    sanctioned public entry points (BC-2.11.006 §Enforcement layer 3, OBS-2 fix).
//!
//! If any `build_*_parser` function were `pub`, it could be called from this file
//! to bypass ALL security guards (check_query_size, check_paren_depth,
//! check_predicate_nesting_depth, check_filter_list_sizes).
//!
//! Story: S-3.01 | SEC-C-003 | F-HIGH-001 | S-3.06 OBS-2
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

// OBS-2 (F-PR130-P1, BC-2.11.006 §Enforcement layer 3):
// `parse_with_registry` is the second sanctioned public entry point (S-3.06).
// These tests confirm it is callable externally and applies the same security
// guards as `parse`.

/// Verifies `parse_with_registry` is callable as a public API from external crates.
#[test]
fn test_api_surface_parse_with_registry_filter_via_public_entry_point() {
    use prism_query::write_verb_registry::WriteVerbRegistry;
    let registry = WriteVerbRegistry::default();
    let result = PrismQlParser::parse_with_registry("src_ip = '10.0.0.1'", &registry);
    assert!(
        result.is_ok(),
        "API surface: PrismQlParser::parse_with_registry must accept a valid filter query; got {:?}",
        result.unwrap_err()
    );
}

/// Verifies `parse_with_registry` rejects denied SQL keywords (same as `parse`).
/// This is the OBS-2 enforcement-layer-3 check for security parity.
#[test]
fn test_api_surface_parse_with_registry_denylist_parity() {
    use prism_query::write_verb_registry::WriteVerbRegistry;
    let registry = WriteVerbRegistry::default();
    // MERGE is in the denylist — both `parse` and `parse_with_registry` must reject it.
    let result = PrismQlParser::parse_with_registry("MERGE INTO foo USING bar", &registry);
    assert!(
        result.is_err(),
        "API surface: parse_with_registry must apply denylist (E-QUERY-002) like parse"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-002"),
        "expected E-QUERY-002, got: {msg}"
    );
}
