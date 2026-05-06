//! Unit tests for S-3.06 pub(crate) write-parser functions.
//!
//! These tests live in src/tests/ (not tests/) because they exercise
//! `pub(crate)` symbols that are not accessible from integration tests:
//! - `extract_sensor_prefix` (pipe_parser)
//! - `is_internal_prism_table` (sql_parser)
//! - `check_unbounded_write` (sql_parser)
//! - `reject_write_verbs_in_filter` (filter_parser)
//! - `parse_pipe_with_write` (pipe_parser)
//! - `parse_sql_dml` (sql_parser)
//!
//! All tests are RED by design (BC-5.38.001 Red Gate discipline):
//! each test body ends in `todo!()` and will panic until the
//! corresponding implementation lands in S-3.06.
//!
//! Story: S-3.06 | BC-2.11.004

#![allow(unused_imports, unused_variables, dead_code)]

use std::collections::HashSet;

use crate::write_verb_registry::{WriteVerbRegistry, WriteVerbSource};

// ─────────────────────────────────────────────────────────────────────────────
// extract_sensor_prefix — pipe_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `extract_sensor_prefix("crowdstrike_hosts")` → `Some("crowdstrike")`.
/// The canonical underscore-delimited source table splits on `_`.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_underscore_notation() {
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix('crowdstrike_hosts') must return Some('crowdstrike')"
    )
}

/// `extract_sensor_prefix("crowdstrike.hosts")` → `Some("crowdstrike")`.
/// Dotted notation is also split at the first separator.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_dotted_notation() {
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix('crowdstrike.hosts') must return Some('crowdstrike')"
    )
}

/// `extract_sensor_prefix("")` → `None`.
/// Empty string has no sensor prefix.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_empty_string() {
    todo!("S-3.06 gap-fill unit — extract_sensor_prefix('') must return None")
}

/// `extract_sensor_prefix("hosts")` (no separator) → `None`.
/// A name with no `_` or `.` separator cannot produce a prefix.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_no_separator() {
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix('hosts') with no separator must return None"
    )
}

/// `extract_sensor_prefix("crowdstrike_falcon_hosts")` (multiple `_`) →
/// `Some("crowdstrike")`. Only the first segment before the first `_` is the prefix.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_multiple_underscores() {
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix with multiple underscores returns only first segment"
    )
}

/// `extract_sensor_prefix("armis.device.tags")` (multiple dots) →
/// `Some("armis")`. Splits on the first dot only.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_multiple_dots() {
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix with multiple dots splits on first dot only"
    )
}

/// `extract_sensor_prefix("_internal")` (leading underscore) →
/// Some("") or None — documents the boundary behaviour for leading separators.
/// The implementation MUST NOT panic on this input.
#[test]
fn test_BC_2_11_004_extract_sensor_prefix_leading_underscore_no_panic() {
    use crate::pipe_parser::extract_sensor_prefix;
    // Regardless of the return value, must not panic.
    let _ = extract_sensor_prefix("_internal");
    todo!(
        "S-3.06 gap-fill unit — extract_sensor_prefix with leading underscore must not panic; document return value"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// is_internal_prism_table — sql_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `is_internal_prism_table("prism_alerts")` → `true`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_alerts() {
    todo!("S-3.06 gap-fill unit — is_internal_prism_table('prism_alerts') must return true")
}

/// `is_internal_prism_table("prism_cases")` → `true`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_cases() {
    todo!("S-3.06 gap-fill unit — is_internal_prism_table('prism_cases') must return true")
}

/// `is_internal_prism_table("crowdstrike_hosts")` → `false`.
/// A sensor table that merely contains `prism` in a different position is NOT protected.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_external_table_false() {
    todo!("S-3.06 gap-fill unit — is_internal_prism_table('crowdstrike_hosts') must return false")
}

/// `is_internal_prism_table("prism")` → `false`.
/// The table named `prism` alone (no underscore suffix) is not an internal table.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_prism_no_suffix_false() {
    todo!("S-3.06 gap-fill unit — is_internal_prism_table('prism') (no suffix) must return false")
}

/// `is_internal_prism_table("prism_future_table")` → `true`.
/// Any `prism_` prefix table (including unknown future tables) is protected.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_any_prism_prefix_true() {
    todo!(
        "S-3.06 gap-fill unit — is_internal_prism_table('prism_future_table') must return true (prefix-based check)"
    )
}

/// `is_internal_prism_table("")` → `false` (not a panic).
/// Empty string does not start with `prism_`.
#[test]
fn test_BC_2_11_004_is_internal_prism_table_empty_string_false() {
    todo!("S-3.06 gap-fill unit — is_internal_prism_table('') must return false (not panic)")
}

// ─────────────────────────────────────────────────────────────────────────────
// check_unbounded_write — sql_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// DELETE with no filter (WHERE = None) → Some(E-QUERY-022 error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_delete_no_where() {
    use crate::write_ast::{DmlNode, DmlOperation};
    let node = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "armis_device_tags".to_string(),
        assignments: vec![],
        filter: None,
        source_select: None,
    };
    todo!(
        "S-3.06 gap-fill unit — check_unbounded_write for DELETE without WHERE must return Some(E-QUERY-022)"
    )
}

/// UPDATE with no filter (WHERE = None) → Some(E-QUERY-022 error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_update_no_where() {
    use crate::ast::Expr;
    use crate::write_ast::{Assignment, DmlNode, DmlOperation};
    let node = DmlNode {
        operation: DmlOperation::Update,
        target_table: "armis_devices".to_string(),
        assignments: vec![Assignment {
            column: "status".to_string(),
            value: Expr::Literal(crate::ast::Literal::String("ok".to_string())),
        }],
        filter: None,
        source_select: None,
    };
    todo!(
        "S-3.06 gap-fill unit — check_unbounded_write for UPDATE without WHERE must return Some(E-QUERY-022)"
    )
}

/// DELETE WITH filter → None (safe; no error).
#[test]
fn test_BC_2_11_004_check_unbounded_write_delete_with_where_is_safe() {
    use crate::ast::{CompareOp, Expr, FieldPath, Literal};
    use crate::write_ast::{DmlNode, DmlOperation};
    // DmlNode.filter is Option<Expr> (see write_ast.rs) — build a Compare Expr.
    let filter = Expr::Compare {
        lhs: Box::new(Expr::Field(FieldPath::new(["device_id"]))),
        op: CompareOp::Eq,
        rhs: Box::new(Expr::Literal(Literal::String("abc".to_string()))),
    };
    let node = DmlNode {
        operation: DmlOperation::Delete,
        target_table: "armis_device_tags".to_string(),
        assignments: vec![],
        filter: Some(filter),
        source_select: None,
    };
    todo!(
        "S-3.06 gap-fill unit — check_unbounded_write for DELETE with WHERE must return None (safe)"
    )
}

/// INSERT INTO without LIMIT or WHERE on the source SELECT → Some(E-QUERY-022).
#[test]
fn test_BC_2_11_004_check_unbounded_write_insert_no_limit_no_where() {
    use crate::ast::{FromClause, SelectClause, SourceRef, SqlQuery};
    use crate::write_ast::{DmlNode, DmlOperation};
    // Build a source SELECT with no WHERE and no LIMIT.
    let source_select = SqlQuery::new(
        SelectClause::new(vec![crate::ast::SelectItem::Star]),
        FromClause::new(SourceRef::from_raw("events")),
    );
    let node = DmlNode {
        operation: DmlOperation::InsertInto,
        target_table: "armis_tags".to_string(),
        assignments: vec![],
        filter: None,
        source_select: Some(source_select),
    };
    todo!(
        "S-3.06 gap-fill unit — check_unbounded_write for INSERT...SELECT without LIMIT or WHERE must return Some(E-QUERY-022)"
    )
}

/// INSERT INTO with LIMIT on the source SELECT → None (safe).
#[test]
fn test_BC_2_11_004_check_unbounded_write_insert_with_limit_is_safe() {
    use crate::ast::{FromClause, SelectClause, SourceRef, SqlQuery};
    use crate::write_ast::{DmlNode, DmlOperation};
    let mut source_select = SqlQuery::new(
        SelectClause::new(vec![crate::ast::SelectItem::Star]),
        FromClause::new(SourceRef::from_raw("events")),
    );
    source_select.limit = Some(100);
    let node = DmlNode {
        operation: DmlOperation::InsertInto,
        target_table: "armis_tags".to_string(),
        assignments: vec![],
        filter: None,
        source_select: Some(source_select),
    };
    todo!(
        "S-3.06 gap-fill unit — check_unbounded_write for INSERT...SELECT with LIMIT must return None (safe)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// reject_write_verbs_in_filter — filter_parser pub(crate) helper
// ─────────────────────────────────────────────────────────────────────────────

/// `reject_write_verbs_in_filter` with a verb in the input and a populated
/// registry → returns Err with a write-rejection message.
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_with_verb_in_input() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    let _registry = WriteVerbRegistry::default(); // will be replaced by from_source in impl
    todo!(
        "S-3.06 gap-fill unit — reject_write_verbs_in_filter with 'contain' in input must return Err"
    )
}

/// `reject_write_verbs_in_filter` with no verb in the input →
/// returns Ok(()) (no rejection).
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_clean_input_ok() {
    let registry = WriteVerbRegistry::default();
    todo!(
        "S-3.06 gap-fill unit — reject_write_verbs_in_filter with no verb in input must return Ok(())"
    )
}

/// `reject_write_verbs_in_filter` with an empty registry →
/// returns Ok(()) regardless of input (no verbs to reject).
#[test]
fn test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok() {
    let empty_registry = WriteVerbRegistry::default();
    todo!(
        "S-3.06 gap-fill unit — reject_write_verbs_in_filter with empty registry must always return Ok(())"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_pipe_with_write — pipe_parser pub(crate) entry point
// ─────────────────────────────────────────────────────────────────────────────

/// `parse_pipe_with_write` with a registered verb in terminal position →
/// returns PipeQuery with write = Some(WriteNode { verb: "contain", ... }).
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_happy_path() {
    todo!(
        "S-3.06 gap-fill unit — parse_pipe_with_write with 'contain' in registry returns PipeQuery with write=Some"
    )
}

/// `parse_pipe_with_write` with an unknown verb in terminal position →
/// returns Err with E-QUERY-023.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_unknown_verb() {
    todo!(
        "S-3.06 gap-fill unit — parse_pipe_with_write with unregistered terminal verb returns Err(E-QUERY-023)"
    )
}

/// `parse_pipe_with_write` with a verb in non-terminal position →
/// returns Err with E-QUERY-024.
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_verb_not_terminal() {
    todo!(
        "S-3.06 gap-fill unit — parse_pipe_with_write with verb mid-pipeline returns Err(E-QUERY-024)"
    )
}

/// `parse_pipe_with_write` with an empty registry and any terminal identifier →
/// returns Err with E-QUERY-023 (EC-11-065).
#[test]
fn test_BC_2_11_004_parse_pipe_with_write_empty_registry_any_verb_023() {
    todo!(
        "S-3.06 gap-fill unit — parse_pipe_with_write with empty registry produces E-QUERY-023 for any terminal identifier"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_sql_dml — sql_parser pub(crate) DML entry point
// ─────────────────────────────────────────────────────────────────────────────

/// `parse_sql_dml("DELETE FROM armis_device_tags WHERE device_id = '123'")` →
/// `Ast::Sql(SqlStatement::Dml(DmlNode { operation: Delete, ... }))`.
#[test]
fn test_BC_2_11_004_parse_sql_dml_delete_with_where() {
    todo!(
        "S-3.06 gap-fill unit — parse_sql_dml for DELETE with WHERE returns Ast::Sql(Dml(Delete))"
    )
}

/// `parse_sql_dml("DELETE FROM armis_device_tags")` (no WHERE) →
/// `Err` with E-QUERY-022.
#[test]
fn test_BC_2_11_004_parse_sql_dml_delete_no_where_022() {
    todo!("S-3.06 gap-fill unit — parse_sql_dml for DELETE without WHERE returns Err(E-QUERY-022)")
}

/// `parse_sql_dml("UPDATE prism_alerts SET x = 1 WHERE id = '1'")` →
/// `Err` with E-QUERY-010.
#[test]
fn test_BC_2_11_004_parse_sql_dml_update_prism_table_010() {
    todo!("S-3.06 gap-fill unit — parse_sql_dml targeting prism_* table returns Err(E-QUERY-010)")
}

/// `parse_sql_dml("not a dml statement")` → `Err` with parse error (not panic).
#[test]
fn test_BC_2_11_004_parse_sql_dml_not_dml_input_parse_error() {
    todo!(
        "S-3.06 gap-fill unit — parse_sql_dml on non-DML input returns Err (parse error, not panic)"
    )
}
