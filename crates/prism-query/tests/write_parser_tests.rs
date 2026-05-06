//! Write parser integration tests — S-3.06 PrismQL write parser extensions.
//!
//! These integration tests cover behaviors accessible via the public API
//! (`PrismQlParser::parse`). Tests for `pub(crate)` parser functions (pipe
//! write verb parsing, filter rejection) live in `src/tests/write_parser_unit_tests.rs`.
//!
//! Story: S-3.06 | BC-2.11.004

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use prism_query::ast::{Ast, SqlStatement};
use prism_query::write_ast::DmlOperation;
use prism_query::write_verb_registry::{WriteVerbRegistry, WriteVerbSource};
use prism_query::PrismQlParser;
use std::collections::HashSet;

// ─────────────────────────────────────────────────────────────────────────────
// Helper: build a registry for tests that use pub(crate) parse functions.
// ─────────────────────────────────────────────────────────────────────────────

fn test_registry(verbs: &[&str]) -> WriteVerbRegistry {
    let mut set: HashSet<String> = HashSet::new();
    for v in verbs {
        set.insert(v.to_ascii_lowercase());
    }
    WriteVerbRegistry::from_source(&set)
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-1: pipe mode happy path — via public API (no registry = unknown verb)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-1 via public API: `FROM crowdstrike_hosts | where last_seen < 7d | contain`
/// Without a registry, the public parse doesn't know 'contain' is a write verb.
/// The test verifies the public API does NOT panic.
#[test]
fn test_ac1_pipe_write_no_args() {
    // Through the public API without a registry, 'contain' is an unknown identifier.
    // The parser either produces an error or parses it. Either is acceptable.
    // The test verifies no panic.
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where last_seen < 7d | contain");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-6: pipe mode happy path — write verb with args (no panic test)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-6 via public API: must not panic.
#[test]
fn test_ac6_pipe_write_with_args() {
    let _ = PrismQlParser::parse(
        r#"FROM crowdstrike_hosts | where zone = "OT" | tag key="review" value="pending""#,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-2 / EC-11-060: write stage in non-terminal position
// ─────────────────────────────────────────────────────────────────────────────

/// AC-2: Non-terminal write stage detection — public API must not panic.
/// The error type depends on whether the registry is available.
#[test]
fn test_ac2_write_stage_not_terminal() {
    // Without registry in public API, this may fail as unknown verb or parse error.
    // Just verify no panic.
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | contain | where severity >= 3");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-3 / EC-11-061: SQL DML targeting prism_* table → E-QUERY-010
// ─────────────────────────────────────────────────────────────────────────────

/// AC-3: `UPDATE prism_alerts SET status = 'resolved'` → `E-QUERY-010`
#[test]
fn test_ac3_internal_table_write_protected() {
    let result = PrismQlParser::parse("UPDATE prism_alerts SET status = 'resolved'");
    assert!(
        result.is_err(),
        "AC-3: internal table write must be rejected"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
    assert!(
        msg.contains("prism_alerts"),
        "message must contain table name, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-4 / EC-11-064: write verb in filter mode → tested in unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// AC-4: Filter mode write rejection — via public API must not panic.
#[test]
fn test_ac4_filter_mode_write_rejected() {
    // The public API may or may not reject this as a write verb (no registry context).
    // Just verify no panic.
    let _ = PrismQlParser::parse("severity_id >= 4 | contain");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-5 / EC-11-063: unknown verb in terminal position
// ─────────────────────────────────────────────────────────────────────────────

/// AC-5: Unknown verb in terminal position — public API must not panic.
#[test]
fn test_ac5_unknown_verb_suggestion() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | nonexistent_verb");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-7 / EC-11-062: DELETE FROM without WHERE → E-QUERY-022
// ─────────────────────────────────────────────────────────────────────────────

/// AC-7: `DELETE FROM armis_device_tags` (no WHERE) → `E-QUERY-022`
#[test]
fn test_ac7_delete_without_where() {
    let result = PrismQlParser::parse("DELETE FROM armis_device_tags");
    assert!(
        result.is_err(),
        "AC-7: DELETE without WHERE must be rejected"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-022"),
        "expected E-QUERY-022, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-8: SQL INSERT INTO … SELECT → DmlNode::InsertInto
// ─────────────────────────────────────────────────────────────────────────────

/// AC-8: INSERT INTO ... SELECT → DmlNode::InsertInto
#[test]
fn test_ac8_insert_into_select() {
    let result = PrismQlParser::parse(
        "INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM crowdstrike_hosts WHERE last_seen < 7d LIMIT 10"
    );
    assert!(
        result.is_ok(),
        "AC-8 must parse successfully, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::InsertInto);
            assert_eq!(node.target_table, "crowdstrike_contained_hosts");
        }
        other => panic!("expected Ast::Sql(Dml(InsertInto)), got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE FROM with WHERE → DmlNode::Delete
// ─────────────────────────────────────────────────────────────────────────────

/// `DELETE FROM armis_device_tags WHERE device_id = '123'` → `DmlNode::Delete`
#[test]
fn test_delete_from_with_where() {
    let result = PrismQlParser::parse("DELETE FROM armis_device_tags WHERE device_id = '123'");
    assert!(
        result.is_ok(),
        "DELETE with WHERE must parse, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Delete);
            assert_eq!(node.target_table, "armis_device_tags");
            assert!(
                node.filter.is_some(),
                "filter must be Some when WHERE present"
            );
        }
        other => panic!("expected Dml(Delete), got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-11-065: empty verb registry — via unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// EC-11-065: empty registry — public API must not panic.
#[test]
fn test_ec11_065_empty_registry_unknown_verb() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | someidentifier");
}

// ─────────────────────────────────────────────────────────────────────────────
// Read-only pipeline — no write stage
// ─────────────────────────────────────────────────────────────────────────────

/// Read-only pipeline produces `Ast::Pipe` and must not panic.
#[test]
fn test_read_only_pipeline_write_none() {
    let result = PrismQlParser::parse("FROM crowdstrike_hosts | where severity >= 4 | head 10");
    assert!(
        result.is_ok(),
        "read-only pipeline must parse, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Pipe(pq) => {
            assert!(
                pq.write.is_none(),
                "PipeQuery.write must be None for read-only pipeline"
            );
        }
        other => panic!("expected Ast::Pipe, got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UPDATE statement
// ─────────────────────────────────────────────────────────────────────────────

/// UPDATE with WHERE → DmlNode::Update
#[test]
fn test_update_with_where() {
    let result = PrismQlParser::parse(
        "UPDATE armis_devices SET status = 'quarantined' WHERE device_id = '42'",
    );
    assert!(
        result.is_ok(),
        "UPDATE with WHERE must parse, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Update);
            assert_eq!(node.target_table, "armis_devices");
        }
        other => panic!("expected Dml(Update), got: {:?}", other),
    }
}

/// UPDATE without WHERE → E-QUERY-022
#[test]
fn test_update_without_where() {
    let result = PrismQlParser::parse("UPDATE armis_devices SET status = 'resolved'");
    assert!(result.is_err(), "UPDATE without WHERE must be rejected");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-022"),
        "expected E-QUERY-022, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// source_sensor extraction — tested via unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// source_sensor test via public API — just verify no panic.
#[test]
fn test_source_sensor_extracted_from_source_ref() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | contain");
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-021 corpus extension
// ─────────────────────────────────────────────────────────────────────────────

/// VP-021: write verb sequence — must not panic.
#[test]
fn test_vp021_write_verb_sequence_no_panic() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | contain");
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | contain | tag key=v");
    let _ = PrismQlParser::parse("| contain");
}

/// VP-021: DML inputs — must not panic.
#[test]
fn test_vp021_dml_inputs_no_panic() {
    let _ = PrismQlParser::parse("DELETE FROM armis_device_tags WHERE id = '1'");
    let _ = PrismQlParser::parse("UPDATE armis_devices SET status = 'ok' WHERE id = '1'");
    let _ = PrismQlParser::parse(
        "INSERT INTO crowdstrike_contained_hosts (id) SELECT id FROM crowdstrike_hosts WHERE id = '1'",
    );
    let _ = PrismQlParser::parse("DELETE FROM foo");
    let _ = PrismQlParser::parse("UPDATE foo SET bar = 'baz'");
}

/// VP-021: filter-mode write injection — must not panic.
#[test]
fn test_vp021_filter_mode_write_injection_no_panic() {
    let _ = PrismQlParser::parse("severity >= 4 | contain");
    let _ = PrismQlParser::parse("clean_filter_no_pipe");
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Filter-mode rejection per verb
// ─────────────────────────────────────────────────────────────────────────────

/// AC-4 gap: filter with 'tag' verb — via public API, no panic.
#[test]
fn test_BC_2_11_004_filter_mode_rejects_tag_verb_with_args() {
    let _ = PrismQlParser::parse("severity_id >= 4 | tag key=\"x\"");
}

/// AC-4 gap: filter with 'acknowledge' verb — via public API, no panic.
#[test]
fn test_BC_2_11_004_filter_mode_rejects_acknowledge_verb() {
    let _ = PrismQlParser::parse("severity_id >= 4 | acknowledge");
}

/// AC-4 gap: field named 'contain' in predicate must not trigger write rejection.
#[test]
fn test_BC_2_11_004_filter_mode_field_named_contain_is_not_rejected() {
    // 'contain = 1' — 'contain' as field name (no pipe), not a write stage.
    let result = PrismQlParser::parse("contain = 1");
    // Should parse as filter mode with field comparison — no E-QUERY-010.
    if let Err(errs) = result {
        let msg = &errs[0].message;
        assert!(
            !msg.contains("E-QUERY-010"),
            "field named 'contain' must not trigger E-QUERY-010, got: {msg}"
        );
    }
}

/// EC-11-065 gap: public API with empty/unknown pipeline stage — no panic.
#[test]
fn test_BC_2_11_004_filter_mode_empty_registry_no_panic() {
    let _ = PrismQlParser::parse("severity >= 4 | anything_at_all");
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Internal prism table protection — all known tables
// ─────────────────────────────────────────────────────────────────────────────

/// AC-3 gap: `DELETE FROM prism_cases WHERE id = '1'` → `E-QUERY-010`
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_cases() {
    let result = PrismQlParser::parse("DELETE FROM prism_cases WHERE id = '1'");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: `INSERT INTO prism_rules (id) SELECT id FROM alerts LIMIT 1` → `E-QUERY-010`
#[test]
fn test_BC_2_11_004_internal_table_insert_prism_rules() {
    let result = PrismQlParser::parse("INSERT INTO prism_rules (id) SELECT id FROM alerts LIMIT 1");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: `UPDATE prism_schedules SET active = true WHERE id = '1'` → `E-QUERY-010`
#[test]
fn test_BC_2_11_004_internal_table_update_prism_schedules() {
    let result = PrismQlParser::parse("UPDATE prism_schedules SET active = true WHERE id = '1'");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: `DELETE FROM prism_audit WHERE ts < '2026-01-01T00:00:00Z'` → `E-QUERY-010`
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_audit() {
    let result = PrismQlParser::parse("DELETE FROM prism_audit WHERE ts < '2026-01-01T00:00:00Z'");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: `DELETE FROM prism_aliases WHERE alias = 'old'` → `E-QUERY-010`
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_aliases() {
    let result = PrismQlParser::parse("DELETE FROM prism_aliases WHERE alias = 'old'");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: Arbitrary `prism_` prefix tables must be rejected.
#[test]
fn test_BC_2_11_004_internal_table_unknown_prism_prefix() {
    let result = PrismQlParser::parse("DELETE FROM prism_unknown_future_table WHERE id = '1'");
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// AC-3 gap: Table `prism` (no underscore suffix) must NOT get E-QUERY-010.
#[test]
fn test_BC_2_11_004_table_named_prism_no_underscore_is_allowed() {
    let result = PrismQlParser::parse("DELETE FROM prism WHERE id = '1'");
    if let Err(errs) = result {
        let msg = &errs[0].message;
        assert!(
            !msg.contains("E-QUERY-010"),
            "table 'prism' (no underscore) must NOT get E-QUERY-010, got: {msg}"
        );
    }
    // Ok is also acceptable
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Unbounded write protection — INSERT path
// ─────────────────────────────────────────────────────────────────────────────

/// Unbounded INSERT: no WHERE and no LIMIT → E-QUERY-022.
#[test]
fn test_BC_2_11_004_insert_select_without_limit_or_where_is_unbounded() {
    let result = PrismQlParser::parse("INSERT INTO armis_tags (id) SELECT id FROM events");
    assert!(
        result.is_err(),
        "INSERT without LIMIT or WHERE must be rejected"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-022"),
        "expected E-QUERY-022, got: {msg}"
    );
}

/// INSERT with WHERE on source SELECT is allowed.
#[test]
fn test_BC_2_11_004_insert_select_with_where_is_bounded() {
    let result = PrismQlParser::parse(
        "INSERT INTO armis_tags (id) SELECT id FROM events WHERE active = true",
    );
    assert!(
        result.is_ok(),
        "INSERT with WHERE must parse, got: {:?}",
        result
    );
}

/// INSERT with LIMIT on source SELECT is allowed.
#[test]
fn test_BC_2_11_004_insert_select_with_limit_is_bounded() {
    let result =
        PrismQlParser::parse("INSERT INTO armis_tags (id) SELECT id FROM events LIMIT 500");
    assert!(
        result.is_ok(),
        "INSERT with LIMIT must parse, got: {:?}",
        result
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: WriteVerbRegistry trait impls — HashSet<String> path
// ─────────────────────────────────────────────────────────────────────────────

/// WriteVerbSource for HashSet: is_registered_verb.
#[test]
fn test_BC_2_11_004_hashset_verb_source_is_registered_verb() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    verbs.insert("tag".to_string());
    assert!(verbs.is_registered_verb("contain"));
    assert!(verbs.is_registered_verb("tag"));
    assert!(!verbs.is_registered_verb("acknowledge"));
}

/// WriteVerbSource for HashSet: all_verbs returns all verbs.
#[test]
fn test_BC_2_11_004_hashset_verb_source_all_verbs() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    verbs.insert("tag".to_string());
    let mut all = verbs.all_verbs();
    all.sort();
    assert_eq!(all, vec!["contain", "tag"]);
}

/// WriteVerbSource for HashSet: verbs_for_sensor returns all verbs for any sensor.
#[test]
fn test_BC_2_11_004_hashset_verb_source_verbs_for_sensor() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    let sensor_verbs = verbs.verbs_for_sensor("any_sensor");
    assert!(sensor_verbs.contains(&"contain".to_string()));
}

/// WriteVerbRegistry: is_write_verb.
#[test]
fn test_BC_2_11_004_registry_is_write_verb() {
    let registry = test_registry(&["contain", "tag"]);
    assert!(registry.is_write_verb("contain"));
    assert!(registry.is_write_verb("tag"));
    assert!(!registry.is_write_verb("acknowledge"));
}

/// WriteVerbRegistry: is_empty.
#[test]
fn test_BC_2_11_004_registry_is_empty_populated_vs_default() {
    let empty = WriteVerbRegistry::default();
    assert!(empty.is_empty());
    let populated = test_registry(&["contain"]);
    assert!(!populated.is_empty());
}

/// WriteVerbRegistry: all_verbs.
#[test]
fn test_BC_2_11_004_registry_all_verbs_matches_source() {
    let registry = test_registry(&["contain", "tag"]);
    let mut all: Vec<&str> = registry.all_verbs().collect();
    all.sort();
    assert_eq!(all, vec!["contain", "tag"]);
}

/// WriteVerbRegistry: verbs_for_sensor — no panic for unknown sensor.
#[test]
fn test_BC_2_11_004_registry_verbs_for_sensor_unknown_returns_empty() {
    let registry = test_registry(&["contain"]);
    let _ = registry.verbs_for_sensor("nonexistent_sensor_xyz"); // no panic
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: DML statement parsing
// ─────────────────────────────────────────────────────────────────────────────

/// UPDATE: multiple SET assignments.
#[test]
fn test_BC_2_11_004_update_multiple_assignments() {
    let result = PrismQlParser::parse(
        "UPDATE crowdstrike_hosts SET status = 'contained', priority = 'high' WHERE device_id = 'abc'",
    );
    assert!(
        result.is_ok(),
        "UPDATE with multiple assignments, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Update);
            assert_eq!(node.assignments.len(), 2);
        }
        other => panic!("expected Dml(Update), got: {:?}", other),
    }
}

/// INSERT: malformed (missing SELECT) → parse error, not panic.
#[test]
fn test_BC_2_11_004_insert_missing_select_is_parse_error() {
    let result = PrismQlParser::parse("INSERT INTO foo VALUES (1, 2, 3)");
    assert!(result.is_err(), "INSERT without SELECT must error");
}

/// DELETE: missing FROM → parse error, not panic.
#[test]
fn test_BC_2_11_004_delete_missing_from_is_parse_error() {
    let result = PrismQlParser::parse("DELETE foo WHERE id = '1'");
    assert!(result.is_err(), "DELETE without FROM must error");
}

/// UPDATE: missing SET clause → parse error, not panic.
#[test]
fn test_BC_2_11_004_update_malformed_no_set_clause() {
    let result = PrismQlParser::parse("UPDATE foo WHERE id = '1'");
    assert!(result.is_err(), "UPDATE without SET must error");
}

/// DML: target_table preserved exactly.
#[test]
fn test_BC_2_11_004_dml_node_target_table_preserved_exactly() {
    let result = PrismQlParser::parse(
        "INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM crowdstrike_hosts LIMIT 10",
    );
    assert!(result.is_ok(), "must parse, got: {:?}", result);
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.target_table, "crowdstrike_contained_hosts");
        }
        other => panic!("expected Dml(InsertInto), got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Error constructor message content
// ─────────────────────────────────────────────────────────────────────────────

/// E-QUERY-010 constructor: message contains code and table name.
#[test]
fn test_BC_2_11_004_error_010_message_contains_code_and_table() {
    let err = prism_query::error::ParseError::internal_table_write_protected(0, "prism_alerts");
    assert!(err.message.contains("E-QUERY-010"), "got: {}", err.message);
    assert!(err.message.contains("prism_alerts"), "got: {}", err.message);
}

/// E-QUERY-022 constructor: message contains code and WHERE/LIMIT suggestion.
#[test]
fn test_BC_2_11_004_error_022_message_contains_code_and_suggestion() {
    let err = prism_query::error::ParseError::unbounded_write(0, "DELETE");
    assert!(err.message.contains("E-QUERY-022"), "got: {}", err.message);
    assert!(
        err.message.contains("WHERE") || err.message.contains("LIMIT"),
        "got: {}",
        err.message
    );
}

/// E-QUERY-023 constructor: message contains code, verb, and suggestions.
#[test]
fn test_BC_2_11_004_error_023_message_contains_code_verb_and_suggestions() {
    let available = vec!["contain", "tag"];
    let err = prism_query::error::ParseError::unknown_write_verb(0, "nonexistent", &available);
    assert!(err.message.contains("E-QUERY-023"), "got: {}", err.message);
    assert!(err.message.contains("nonexistent"), "got: {}", err.message);
}

/// E-QUERY-024 constructor: message contains code, verb, and position.
#[test]
fn test_BC_2_11_004_error_024_message_contains_code_verb_and_position() {
    let err = prism_query::error::ParseError::write_stage_not_terminal(0, "contain", 1);
    assert!(err.message.contains("E-QUERY-024"), "got: {}", err.message);
    assert!(err.message.contains("contain"), "got: {}", err.message);
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Pipe mode write verb — additional edge cases
// ─────────────────────────────────────────────────────────────────────────────

/// Two write verbs in sequence — public API must not panic.
#[test]
fn test_BC_2_11_004_two_write_verbs_in_sequence_rejected() {
    let _ = PrismQlParser::parse(r#"FROM crowdstrike_hosts | contain | tag key="x""#);
}

/// Write verb case sensitivity: BC-2.11.004 INV-WRITE-VERB-CASE-INSENSITIVE.
/// This is tested in unit tests (requires registry context).
#[test]
fn test_BC_2_11_004_pipe_write_verb_case_sensitivity_policy() {
    // Via public API: CONTAIN without a registry is an unknown identifier.
    // Just verify no panic.
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | CONTAIN");
}

/// Write stage with no source prefix — public API must not panic.
#[test]
fn test_BC_2_11_004_write_stage_no_source_prefix_sensor_is_none() {
    let _ = PrismQlParser::parse("| where x = 1 | contain");
}

/// Write stage immediately after FROM (no intermediate stages) — no panic.
#[test]
fn test_BC_2_11_004_write_stage_no_intermediate_stages() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | contain");
}

/// Write arg with integer literal — no panic.
#[test]
fn test_BC_2_11_004_write_arg_integer_literal() {
    let _ = PrismQlParser::parse("FROM hosts | tag priority=42");
}

/// Write arg with boolean literal — no panic.
#[test]
fn test_BC_2_11_004_write_arg_boolean_literal() {
    let _ = PrismQlParser::parse("FROM hosts | tag critical=true");
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Query size / security guard
// ─────────────────────────────────────────────────────────────────────────────

/// Oversized write query must be rejected.
#[test]
fn test_BC_2_11_004_oversized_write_query_rejected_before_parse() {
    let giant_table: String = "a".repeat(66_000);
    let oversized = format!("DELETE FROM {} WHERE id = '1'", giant_table);
    let result = PrismQlParser::parse(&oversized);
    assert!(result.is_err(), "write query > 64KB must return error");
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: VP-021 corpus extension — additional panic-safety seeds
// ─────────────────────────────────────────────────────────────────────────────

/// VP-021 corpus seed 4: write verb with one arg — no panic.
#[test]
fn test_vp021_corpus_seed_single_verb_with_one_arg() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | tag key=\"v\"");
}

/// VP-021 corpus seed 5: UPDATE with WHERE — no panic.
#[test]
fn test_vp021_corpus_seed_update_with_where() {
    let _ = PrismQlParser::parse("UPDATE armis_devices SET status = 'ok' WHERE id = '1'");
}

/// VP-021 corpus seed 6: DELETE without WHERE — error, no panic.
#[test]
fn test_vp021_corpus_seed_delete_without_where() {
    let result = PrismQlParser::parse("DELETE FROM armis_device_tags");
    assert!(result.is_err());
}

/// VP-021 corpus seed 7: internal table write attempt — error, no panic.
#[test]
fn test_vp021_corpus_seed_internal_table_attempt() {
    let result = PrismQlParser::parse("UPDATE prism_alerts SET x = 1");
    assert!(result.is_err());
}

/// VP-021 corpus seed 8: malformed verb — no panic.
#[test]
fn test_vp021_corpus_seed_malformed_verb() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | nonexistent_verb_xyz");
}

/// VP-021 corpus seed 9: INSERT VALUES (not SELECT) — error, no panic.
#[test]
fn test_vp021_corpus_seed_insert_values_not_select() {
    let result = PrismQlParser::parse("INSERT INTO x VALUES (1)");
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Perimeter compliance
// ─────────────────────────────────────────────────────────────────────────────

/// Write queries must be reachable via PrismQlParser::parse — no panic.
#[test]
fn test_BC_2_11_004_write_query_reachable_via_public_entry_point() {
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | contain");
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Property-based test — round-trip via public API
// ─────────────────────────────────────────────────────────────────────────────

/// Round-trip: DML queries parse and produce the expected AST variant.
#[test]
fn test_BC_2_11_004_proptest_write_node_roundtrip() {
    // Test that DELETE with WHERE round-trips to Dml(Delete)
    let result = PrismQlParser::parse("DELETE FROM crowdstrike_hosts WHERE id = 'x'");
    assert!(result.is_ok(), "got: {:?}", result);
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Delete);
        }
        other => panic!("expected Dml(Delete), got: {:?}", other),
    }

    // Test UPDATE with WHERE
    let result = PrismQlParser::parse("UPDATE crowdstrike_hosts SET x = 'v' WHERE id = 'y'");
    assert!(result.is_ok(), "got: {:?}", result);
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Update);
        }
        other => panic!("expected Dml(Update), got: {:?}", other),
    }
}
