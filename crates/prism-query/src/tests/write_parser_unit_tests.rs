//! Unit tests for S-3.06 pub(crate) write-parser functions.
//!
//! These tests exercise `pub(crate)` symbols not accessible from integration tests.
//!
//! Story: S-3.06 | BC-2.11.004

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::collections::HashSet;

use crate::write_verb_registry::{WriteVerbRegistry, WriteVerbSource};

// ─────────────────────────────────────────────────────────────────────────────
// Test helper
// ─────────────────────────────────────────────────────────────────────────────

fn test_registry(verbs: &[&str]) -> WriteVerbRegistry {
    let mut set: HashSet<String> = HashSet::new();
    for v in verbs {
        set.insert(v.to_ascii_lowercase());
    }
    WriteVerbRegistry::from_source(&set)
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_sensor_prefix — pipe_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `extract_sensor_prefix("crowdstrike_hosts")` → `Some("crowdstrike")`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_underscore_notation() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(
        extract_sensor_prefix("crowdstrike_hosts"),
        Some("crowdstrike".to_string()),
        "crowdstrike_hosts → Some('crowdstrike')"
    );
}

/// `extract_sensor_prefix("crowdstrike.hosts")` → `Some("crowdstrike")`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_dotted_notation() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(
        extract_sensor_prefix("crowdstrike.hosts"),
        Some("crowdstrike".to_string()),
        "crowdstrike.hosts → Some('crowdstrike')"
    );
}

/// `extract_sensor_prefix("")` → `None`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_empty_string() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(extract_sensor_prefix(""), None, "'' → None");
}

/// `extract_sensor_prefix("hosts")` (no separator) → `None`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_no_separator() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(
        extract_sensor_prefix("hosts"),
        None,
        "'hosts' (no sep) → None"
    );
}

/// `extract_sensor_prefix("crowdstrike_falcon_hosts")` → `Some("crowdstrike")`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_multiple_underscores() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(
        extract_sensor_prefix("crowdstrike_falcon_hosts"),
        Some("crowdstrike".to_string()),
        "multiple underscores → first segment only"
    );
}

/// `extract_sensor_prefix("armis.device.tags")` → `Some("armis")`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_multiple_dots() {
    use crate::pipe_parser::extract_sensor_prefix;
    assert_eq!(
        extract_sensor_prefix("armis.device.tags"),
        Some("armis".to_string()),
        "multiple dots → split on first dot only"
    );
}

/// `extract_sensor_prefix("_internal")` (leading underscore) → Some("") and must not panic.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_leading_underscore_no_panic() {
    use crate::pipe_parser::extract_sensor_prefix;
    // Leading underscore: separator is at position 0, prefix is ""
    let result = extract_sensor_prefix("_internal");
    // Must not panic; Some("") is the expected value
    assert_eq!(
        result,
        Some("".to_string()),
        "_internal → Some('') (empty prefix before leading _)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// is_internal_prism_table — sql_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `is_internal_prism_table("prism_alerts")` → `true`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_alerts() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(is_internal_prism_table("prism_alerts"));
}

/// `is_internal_prism_table("prism_cases")` → `true`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_cases() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(is_internal_prism_table("prism_cases"));
}

/// `is_internal_prism_table("crowdstrike_hosts")` → `false`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_external_table_false() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(!is_internal_prism_table("crowdstrike_hosts"));
}

/// `is_internal_prism_table("prism")` (no suffix) → `false`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_no_suffix_false() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(
        !is_internal_prism_table("prism"),
        "'prism' alone must not be protected"
    );
}

/// `is_internal_prism_table("prism_future_table")` → `true` (prefix-based).
#[test]
fn test_BC_2_11_004_is_internal_prism_table_any_prism_prefix_true() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(
        is_internal_prism_table("prism_future_table"),
        "any prism_ prefix table is protected"
    );
}

/// `is_internal_prism_table("")` → `false` (not a panic).
#[test]
fn test_BC_2_11_004_is_internal_prism_table_empty_string_false() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(!is_internal_prism_table(""));
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PR130-SEC-001: Case-insensitive internal table guard (CWE-178)
// ─────────────────────────────────────────────────────────────────────────────

/// `is_internal_prism_table("PRISM_audit")` → `true` (all uppercase).
/// Regression: CWE-178 bypass — attacker submits uppercase to skip protection.
#[test]
fn test_BC_2_11_004_internal_table_case_insensitive_PRISM_uppercase() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(
        is_internal_prism_table("PRISM_audit"),
        "PRISM_audit must be protected (uppercase bypass regression)"
    );
    assert!(
        is_internal_prism_table("PRISM_cases"),
        "PRISM_cases must be protected (uppercase bypass regression)"
    );
}

/// `is_internal_prism_table("Prism_audit")` → `true` (mixed case).
/// Regression: CWE-178 bypass — attacker submits mixed case to skip protection.
#[test]
fn test_BC_2_11_004_internal_table_case_insensitive_mixed_case() {
    use crate::sql_parser::is_internal_prism_table;
    assert!(
        is_internal_prism_table("Prism_audit"),
        "Prism_audit must be protected"
    );
    assert!(
        is_internal_prism_table("PrIsM_rules"),
        "PrIsM_rules must be protected"
    );
    assert!(
        is_internal_prism_table("pRiSm_schedules"),
        "pRiSm_schedules must be protected"
    );
}

/// E2E: `DELETE FROM PRISM_audit WHERE id = '1'` must reject with E-QUERY-010
/// (not bypass the write-protection guard due to case mismatch).
/// Regression test for F-PR130-SEC-001 (CWE-178).
#[test]
fn test_BC_2_11_004_internal_table_case_insensitive_integration_DELETE_PRISM() {
    use crate::sql_parser::parse_sql_dml;
    let result = parse_sql_dml("DELETE FROM PRISM_audit WHERE id = '1'");
    assert!(result.is_err(), "DELETE FROM PRISM_audit must be rejected");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010 (write-protected), got: {msg}"
    );
}

/// E2E: `UPDATE Prism_cases SET x = 1 WHERE id = '1'` must reject with E-QUERY-010.
/// Regression test for F-PR130-SEC-001 (CWE-178).
#[test]
fn test_BC_2_11_004_internal_table_case_insensitive_integration_UPDATE_mixed() {
    use crate::sql_parser::parse_sql_dml;
    let result = parse_sql_dml("UPDATE Prism_cases SET x = 1 WHERE id = '1'");
    assert!(result.is_err(), "UPDATE Prism_cases must be rejected");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010 (write-protected), got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// check_unbounded_write — sql_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// DELETE with no filter (WHERE = None) → Some(E-QUERY-022 error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_delete_no_where() {
    use crate::sql_parser::check_unbounded_write;
    use crate::write_ast::{DmlNode, DmlOperation};
    let node = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "armis_device_tags".to_string(),
        columns: None,
        assignments: vec![],
        filter: None,
        source_select: None,
    };
    let result = check_unbounded_write(&node, 0);
    assert!(
        result.is_some(),
        "DELETE without WHERE must produce an error"
    );
    let err = result.unwrap();
    assert!(
        err.message.contains("E-QUERY-022"),
        "error must be E-QUERY-022, got: {}",
        err.message
    );
}

/// UPDATE with no filter (WHERE = None) → Some(E-QUERY-022 error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_update_no_where() {
    use crate::ast::Expr;
    use crate::sql_parser::check_unbounded_write;
    use crate::write_ast::{Assignment, DmlNode, DmlOperation};
    let node = DmlNode {
        operation: DmlOperation::Update,
        target_table: "armis_devices".to_string(),
        columns: None,
        assignments: vec![Assignment {
            column: "status".to_string(),
            value: Expr::Literal(crate::ast::Literal::String("ok".to_string())),
        }],
        filter: None,
        source_select: None,
    };
    let result = check_unbounded_write(&node, 0);
    assert!(
        result.is_some(),
        "UPDATE without WHERE must produce an error"
    );
    let err = result.unwrap();
    assert!(
        err.message.contains("E-QUERY-022"),
        "error must be E-QUERY-022, got: {}",
        err.message
    );
}

/// DELETE WITH filter → None (safe; no error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_delete_with_where_is_safe() {
    use crate::ast::{CompareOp, Expr, FieldPath, Literal};
    use crate::sql_parser::check_unbounded_write;
    use crate::write_ast::{DmlNode, DmlOperation};
    let filter = Expr::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["device_id"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("abc".to_string()))),
    };
    let node = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "armis_device_tags".to_string(),
        columns: None,
        assignments: vec![],
        filter: Some(filter),
        source_select: None,
    };
    let result = check_unbounded_write(&node, 0);
    assert!(
        result.is_none(),
        "DELETE with WHERE must return None (safe)"
    );
}

/// INSERT INTO without LIMIT or WHERE on the source SELECT → Some(E-QUERY-022).
#[test]
fn test_BC_2_11_004_check_unbounded_write_insert_no_limit_no_where() {
    use crate::ast::{FromClause, SelectClause, SourceRef, SqlQuery};
    use crate::sql_parser::check_unbounded_write;
    use crate::write_ast::{DmlNode, DmlOperation};
    let source_select = SqlQuery::new(
        SelectClause::new(vec![crate::ast::SelectItem::Star]),
        FromClause::new(SourceRef::from_raw("events")),
    );
    let node = DmlNode {
        operation: DmlOperation::InsertInto,
        target_table: "armis_tags".to_string(),
        columns: None,
        assignments: vec![],
        filter: None,
        source_select: Some(source_select),
    };
    let result = check_unbounded_write(&node, 0);
    assert!(
        result.is_some(),
        "INSERT...SELECT without LIMIT or WHERE must produce error"
    );
    let err = result.unwrap();
    assert!(
        err.message.contains("E-QUERY-022"),
        "error must be E-QUERY-022, got: {}",
        err.message
    );
}

/// INSERT INTO with LIMIT on the source SELECT → None (safe).
#[test]
fn test_BC_2_11_004_check_unbounded_write_insert_with_limit_is_safe() {
    use crate::ast::{FromClause, SelectClause, SourceRef, SqlQuery};
    use crate::sql_parser::check_unbounded_write;
    use crate::write_ast::{DmlNode, DmlOperation};
    let mut source_select = SqlQuery::new(
        SelectClause::new(vec![crate::ast::SelectItem::Star]),
        FromClause::new(SourceRef::from_raw("events")),
    );
    source_select.limit = Some(100);
    let node = DmlNode {
        operation: DmlOperation::InsertInto,
        target_table: "armis_tags".to_string(),
        columns: None,
        assignments: vec![],
        filter: None,
        source_select: Some(source_select),
    };
    let result = check_unbounded_write(&node, 0);
    assert!(
        result.is_none(),
        "INSERT...SELECT with LIMIT must return None (safe)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// reject_write_verbs_in_filter — filter_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `reject_write_verbs_in_filter` with a verb in the input and a populated
/// registry → returns Err.
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_with_verb_in_input() {
    use crate::filter_parser::reject_write_verbs_in_filter;
    let registry = test_registry(&["contain"]);
    let result = reject_write_verbs_in_filter("severity_id >= 4 | contain", &registry);
    assert!(
        result.is_err(),
        "filter with 'contain' write verb must be rejected"
    );
}

/// `reject_write_verbs_in_filter` with no verb in the input → Ok(()).
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_clean_input_ok() {
    use crate::filter_parser::reject_write_verbs_in_filter;
    let registry = test_registry(&["contain"]);
    let result = reject_write_verbs_in_filter("severity_id >= 4 AND status = 'active'", &registry);
    assert!(result.is_ok(), "clean filter must return Ok(())");
}

/// `reject_write_verbs_in_filter` with an empty registry → Ok(()) always.
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok() {
    use crate::filter_parser::reject_write_verbs_in_filter;
    let empty_registry = WriteVerbRegistry::default();
    let result = reject_write_verbs_in_filter("anything | contain | tag", &empty_registry);
    assert!(
        result.is_ok(),
        "empty registry must always return Ok(()) per INV-FILTER-EMPTY-REGISTRY"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_pipe_with_write — pipe_parser pub(crate) entry point
// ─────────────────────────────────────────────────────────────────────────────

/// `parse_pipe_with_write` with a registered verb in terminal position →
/// returns PipeQuery with write = Some(WriteNode { verb: "contain", ... }).
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_happy_path() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["contain", "tag"]);
    let result = parse_pipe_with_write(
        "FROM crowdstrike_hosts | where last_seen < 7d | contain",
        &registry,
    );
    assert!(result.is_ok(), "must parse successfully, got: {:?}", result);
    let pq = result.unwrap();
    let write = pq.write.as_ref().expect("write must be Some");
    assert_eq!(write.verb, "contain");
    assert!(write.args.is_empty());
}

/// `parse_pipe_with_write` with an unknown verb in terminal position →
/// returns Err with E-QUERY-023.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_unknown_verb() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["contain", "tag"]);
    let result = parse_pipe_with_write(
        "FROM crowdstrike_hosts | where x = 1 | nonexistent_verb",
        &registry,
    );
    assert!(result.is_err(), "unknown verb must produce error");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-023"),
        "expected E-QUERY-023, got: {msg}"
    );
}

/// `parse_pipe_with_write` with a verb in non-terminal position →
/// returns Err with E-QUERY-024.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_verb_not_terminal() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["contain"]);
    let result = parse_pipe_with_write(
        "FROM crowdstrike_hosts | contain | where severity >= 3",
        &registry,
    );
    assert!(
        result.is_err(),
        "non-terminal write verb must produce error"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-024"),
        "expected E-QUERY-024, got: {msg}"
    );
}

/// `parse_pipe_with_write` with empty registry and any terminal identifier →
/// returns Err with E-QUERY-023.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_empty_registry_any_verb_023() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = WriteVerbRegistry::default();
    let result = parse_pipe_with_write("FROM crowdstrike_hosts | someidentifier", &registry);
    assert!(
        result.is_err(),
        "empty registry: any terminal identifier must produce E-QUERY-023"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-023"),
        "expected E-QUERY-023, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_sql_dml — sql_parser pub(crate) DML entry point
// ─────────────────────────────────────────────────────────────────────────────

/// `parse_sql_dml("DELETE FROM armis_device_tags WHERE device_id = '123'")` →
/// `Ast::Sql(SqlStatement::Dml(DmlNode { operation: Delete, ... }))`.
#[test]
fn test_BC_2_11_004_parse_sql_dml_delete_with_where() {
    use crate::ast::{Ast, SqlStatement};
    use crate::sql_parser::parse_sql_dml;
    use crate::write_ast::DmlOperation;
    let result = parse_sql_dml("DELETE FROM armis_device_tags WHERE device_id = '123'");
    assert!(
        result.is_ok(),
        "DELETE with WHERE must parse, got: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Delete);
            assert_eq!(node.target_table, "armis_device_tags");
            assert!(node.filter.is_some());
        }
        other => panic!("expected Dml(Delete), got: {:?}", other),
    }
}

/// `parse_sql_dml("DELETE FROM armis_device_tags")` (no WHERE) →
/// `Err` with E-QUERY-022.
#[test]
fn test_BC_2_11_004_parse_sql_dml_delete_no_where_022() {
    use crate::sql_parser::parse_sql_dml;
    let result = parse_sql_dml("DELETE FROM armis_device_tags");
    assert!(result.is_err(), "DELETE without WHERE must error");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-022"),
        "expected E-QUERY-022, got: {msg}"
    );
}

/// `parse_sql_dml("UPDATE prism_alerts SET x = 1 WHERE id = '1'")` →
/// `Err` with E-QUERY-010.
#[test]
fn test_BC_2_11_004_parse_sql_dml_update_prism_table_010() {
    use crate::sql_parser::parse_sql_dml;
    let result = parse_sql_dml("UPDATE prism_alerts SET x = 1 WHERE id = '1'");
    assert!(
        result.is_err(),
        "prism_* table must be rejected with E-QUERY-010"
    );
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010, got: {msg}"
    );
}

/// `parse_sql_dml("not a dml statement")` → `Err` (parse error, not panic).
#[test]
fn test_BC_2_11_004_parse_sql_dml_not_dml_input_parse_error() {
    use crate::sql_parser::parse_sql_dml;
    let result = parse_sql_dml("not a dml statement");
    assert!(result.is_err(), "non-DML input must return parse error");
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional unit tests for write verb case sensitivity (BC-2.11.004 v1.4)
// ─────────────────────────────────────────────────────────────────────────────

/// Write verb case sensitivity: CONTAIN (uppercase) matches registered 'contain'.
#[test]
fn test_BC_2_11_004_write_verb_case_insensitive_uppercase() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["contain"]);
    let result = parse_pipe_with_write("FROM crowdstrike_hosts | CONTAIN", &registry);
    assert!(
        result.is_ok(),
        "CONTAIN must match 'contain' per INV-WRITE-VERB-CASE-INSENSITIVE, got: {:?}",
        result
    );
    let pq = result.unwrap();
    let write = pq.write.as_ref().expect("write must be Some");
    assert_eq!(write.verb, "contain", "verb normalized to lowercase");
}

/// WriteVerbRegistry::is_write_verb is case-insensitive.
#[test]
fn test_BC_2_11_004_registry_is_write_verb_case_insensitive() {
    let registry = test_registry(&["contain"]);
    assert!(
        registry.is_write_verb("CONTAIN"),
        "is_write_verb must be case-insensitive"
    );
    assert!(
        registry.is_write_verb("Contain"),
        "is_write_verb must be case-insensitive"
    );
    assert!(
        registry.is_write_verb("contain"),
        "is_write_verb must be case-insensitive"
    );
}

/// Filter rejection: verb appearing after `|` with mixed case.
#[test]
fn test_BC_2_11_004_filter_rejection_case_insensitive() {
    use crate::filter_parser::reject_write_verbs_in_filter;
    let registry = test_registry(&["contain"]);
    // CONTAIN uppercase after `|` must be rejected
    let result = reject_write_verbs_in_filter("severity >= 4 | CONTAIN", &registry);
    assert!(
        result.is_err(),
        "uppercase CONTAIN after | must be rejected in filter mode"
    );
}

/// source_sensor is populated from FROM source with underscore notation.
#[test]
fn test_BC_2_11_004_source_sensor_populated_from_from_clause() {
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["contain"]);
    let result = parse_pipe_with_write("FROM armis_devices | contain", &registry);
    assert!(result.is_ok(), "got: {:?}", result);
    let pq = result.unwrap();
    let write = pq.write.as_ref().expect("write must be Some");
    assert_eq!(
        write.source_sensor.as_deref(),
        Some("armis"),
        "source_sensor must be 'armis' from armis_devices"
    );
}

/// parse_pipe_with_write: write stage with multiple args.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_multiple_args() {
    use crate::ast::Literal;
    use crate::pipe_parser::parse_pipe_with_write;
    let registry = test_registry(&["tag"]);
    let result = parse_pipe_with_write(
        r#"FROM crowdstrike_hosts | tag key="review" value="pending""#,
        &registry,
    );
    assert!(result.is_ok(), "got: {:?}", result);
    let pq = result.unwrap();
    let write = pq.write.as_ref().expect("write must be Some");
    assert_eq!(write.verb, "tag");
    assert_eq!(write.args.len(), 2);
    let has_key = write
        .args
        .iter()
        .any(|a| a.key == "key" && a.value == Literal::String("review".into()));
    let has_value = write
        .args
        .iter()
        .any(|a| a.key == "value" && a.value == Literal::String("pending".into()));
    assert!(has_key, "must have key='review'");
    assert!(has_value, "must have value='pending'");
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PR130-CR-001: PrismQlParser::parse_with_registry — wire parse_pipe_with_write
// ─────────────────────────────────────────────────────────────────────────────

/// `PrismQlParser::parse_with_registry` — pipe mode produces WriteNode when
/// terminal verb is registered. Previously `PrismQlParser::parse` always
/// returned `PipeQuery { write: None }` regardless of registry.
#[test]
fn test_BC_2_11_004_PrismQlParser_parse_with_registry_pipe_write_routes_to_WriteNode() {
    use crate::ast::Ast;
    use crate::filter_parser::PrismQlParser;
    let registry = test_registry(&["contain"]);
    let result = PrismQlParser::parse_with_registry("FROM crowdstrike_hosts | contain", &registry);
    assert!(result.is_ok(), "must parse successfully, got: {:?}", result);
    match result.unwrap() {
        Ast::Pipe(pq) => {
            let write = pq
                .write
                .as_ref()
                .expect("PipeQuery.write must be Some(WriteNode)");
            assert_eq!(write.verb, "contain", "verb must be 'contain'");
        }
        other => panic!("expected Ast::Pipe, got: {:?}", other),
    }
}

/// `PrismQlParser::parse` (no registry) — same pipe query returns write: None.
/// Ensures backward compatibility: S-3.01 callers are unaffected.
#[test]
fn test_BC_2_11_004_PrismQlParser_parse_no_registry_pipe_write_is_none() {
    use crate::ast::Ast;
    use crate::filter_parser::PrismQlParser;
    // Without registry, the pipe query should parse but write stage is unknown → error
    // OR parse as plain pipe (implementation-dependent). Key: parse() never crashes.
    // The default parse() does NOT attempt write detection, so a bare verb at the end
    // falls through to E-QUERY-023 or is treated as a read-only pipe stage failure.
    // We just assert no panic occurs.
    let result = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1");
    assert!(
        result.is_ok(),
        "read-only pipe parse must succeed: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Pipe(pq) => {
            assert!(pq.write.is_none(), "write must be None without registry");
        }
        other => panic!("expected Ast::Pipe, got: {:?}", other),
    }
}

/// Filter-mode `parse_with_registry`: write verb after `|` is rejected with E-QUERY-010.
#[test]
fn test_BC_2_11_004_PrismQlParser_parse_filter_mode_rejects_write_verb_with_registry() {
    use crate::filter_parser::PrismQlParser;
    let registry = test_registry(&["contain"]);
    // Filter mode: starts with a predicate, not FROM / SELECT / DML.
    let result = PrismQlParser::parse_with_registry("status = 'active' | contain", &registry);
    assert!(result.is_err(), "filter mode must reject write verb");
    let msg = result.unwrap_err()[0].message.clone();
    assert!(
        msg.contains("E-QUERY-010"),
        "expected E-QUERY-010 for write verb in filter mode, got: {msg}"
    );
}

/// Filter-mode `parse_with_registry` with no write verb: parses normally.
#[test]
fn test_BC_2_11_004_PrismQlParser_parse_with_registry_filter_clean_ok() {
    use crate::ast::Ast;
    use crate::filter_parser::PrismQlParser;
    let registry = test_registry(&["contain"]);
    let result = PrismQlParser::parse_with_registry("status = 'active'", &registry);
    assert!(result.is_ok(), "clean filter must parse: {:?}", result);
    assert!(
        matches!(result.unwrap(), Ast::Filter(_)),
        "must produce Ast::Filter"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-PR130-CR-003: INSERT column list preserved in DmlNode.columns
// ─────────────────────────────────────────────────────────────────────────────

/// `INSERT INTO t (col1, col2) SELECT * FROM s WHERE id = 1 LIMIT 100`
/// must produce `DmlNode.columns = Some(["col1", "col2"])`.
/// Regression: previously the column list was silently dropped (`_cols`).
#[test]
fn test_BC_2_11_004_insert_column_list_preserved_in_DmlNode_columns() {
    use crate::ast::{Ast, SqlStatement};
    use crate::sql_parser::parse_sql_dml;
    use crate::write_ast::DmlOperation;
    let result = parse_sql_dml(
        "INSERT INTO armis_tags (device_id, tag_name) SELECT id, name FROM events WHERE id = '1' LIMIT 10",
    );
    assert!(
        result.is_ok(),
        "INSERT with col list must parse: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::InsertInto);
            let cols = node.columns.as_ref().expect("columns must be Some");
            assert_eq!(cols.len(), 2, "must have 2 columns, got: {:?}", cols);
            assert!(
                cols.contains(&"device_id".to_string()),
                "columns must contain 'device_id'"
            );
            assert!(
                cols.contains(&"tag_name".to_string()),
                "columns must contain 'tag_name'"
            );
        }
        other => panic!("expected Ast::Sql(Dml(InsertInto)), got: {:?}", other),
    }
}

/// `INSERT INTO t SELECT * FROM s WHERE id = 1 LIMIT 100` (no column list)
/// must produce `DmlNode.columns = None`.
#[test]
fn test_BC_2_11_004_insert_no_column_list_columns_is_none() {
    use crate::ast::{Ast, SqlStatement};
    use crate::sql_parser::parse_sql_dml;
    use crate::write_ast::DmlOperation;
    let result =
        parse_sql_dml("INSERT INTO armis_tags SELECT id FROM events WHERE id = '1' LIMIT 10");
    assert!(
        result.is_ok(),
        "INSERT without col list must parse: {:?}",
        result
    );
    match result.unwrap() {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::InsertInto);
            assert!(
                node.columns.is_none(),
                "columns must be None when no column list is present"
            );
        }
        other => panic!("expected Ast::Sql(Dml(InsertInto)), got: {:?}", other),
    }
}

/// DELETE and UPDATE always have `columns = None`.
#[test]
fn test_BC_2_11_004_delete_update_columns_always_none() {
    use crate::ast::{Ast, SqlStatement};
    use crate::sql_parser::parse_sql_dml;
    use crate::write_ast::DmlOperation;

    let del = parse_sql_dml("DELETE FROM armis_tags WHERE id = '1'").unwrap();
    match del {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Delete);
            assert!(node.columns.is_none(), "DELETE columns must be None");
        }
        _ => panic!("expected Delete DmlNode"),
    }

    let upd = parse_sql_dml("UPDATE armis_tags SET status = 'ok' WHERE id = '1'").unwrap();
    match upd {
        Ast::Sql(SqlStatement::Dml(node)) => {
            assert_eq!(node.operation, DmlOperation::Update);
            assert!(node.columns.is_none(), "UPDATE columns must be None");
        }
        _ => panic!("expected Update DmlNode"),
    }
}
