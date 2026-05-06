//! Write parser tests — S-3.06 PrismQL write parser extensions.
//!
//! Tests cover all Acceptance Criteria (AC-1 through AC-8) and Edge Cases
//! (EC-11-060 through EC-11-065) defined in S-3.06.
//!
//! All test bodies use `todo!()` per BC-5.38.001 Red Gate discipline.
//! These tests are RED by design — they will pass once the implementation
//! in S-3.06 is complete.
//!
//! # Test injection pattern (Story dev notes)
//! Tests use `HashSet<String>` as a `WriteVerbSource` for test-injectable
//! verb sets, avoiding the need for a fully initialized `WriteEndpointRegistry`.
//!
//! Story: S-3.06 | BC-2.11.004

use std::collections::HashSet;

use prism_query::write_ast::{DmlOperation, WriteNode};
use prism_query::write_verb_registry::{WriteVerbRegistry, WriteVerbSource};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a `WriteVerbRegistry` from a fixed set of verb strings for tests.
fn test_registry(verbs: &[&str]) -> WriteVerbRegistry {
    todo!("S-3.06 — test_registry helper")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-1: pipe mode happy path — single write verb, no args
// ─────────────────────────────────────────────────────────────────────────────

/// AC-1: `FROM crowdstrike_hosts | where last_seen < 7d | contain` →
/// `PipeQuery` with `write = Some(WriteNode { verb: "contain", args: [], ... })`
#[test]
fn test_ac1_pipe_write_no_args() {
    todo!("S-3.06 — test_ac1_pipe_write_no_args")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-6: pipe mode happy path — write verb with args
// ─────────────────────────────────────────────────────────────────────────────

/// AC-6: `FROM crowdstrike_hosts | where zone = "OT" | tag key="review" value="pending"` →
/// `WriteNode.args` contains two `WriteArg` entries.
#[test]
fn test_ac6_pipe_write_with_args() {
    todo!("S-3.06 — test_ac6_pipe_write_with_args")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-2 / EC-11-060: write stage in non-terminal position → E-QUERY-024
// ─────────────────────────────────────────────────────────────────────────────

/// AC-2: `FROM crowdstrike_hosts | contain | where severity >= 3` → `E-QUERY-024`
#[test]
fn test_ac2_write_stage_not_terminal() {
    todo!("S-3.06 — test_ac2_write_stage_not_terminal")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-3 / EC-11-061: SQL DML targeting prism_* table → E-QUERY-010
// ─────────────────────────────────────────────────────────────────────────────

/// AC-3: `UPDATE prism_alerts SET status = 'resolved'` → `E-QUERY-010`
#[test]
fn test_ac3_internal_table_write_protected() {
    todo!("S-3.06 — test_ac3_internal_table_write_protected")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-4 / EC-11-064: write verb in filter mode → parse error
// ─────────────────────────────────────────────────────────────────────────────

/// AC-4: `severity_id >= 4 | contain` in filter mode → parse error (writes not allowed)
#[test]
fn test_ac4_filter_mode_write_rejected() {
    todo!("S-3.06 — test_ac4_filter_mode_write_rejected")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-5 / EC-11-063: unknown verb in terminal position → E-QUERY-023
// ─────────────────────────────────────────────────────────────────────────────

/// AC-5: `FROM crowdstrike_hosts | where x = 1 | nonexistent_verb` → `E-QUERY-023`
/// with suggestion list of available verbs for `crowdstrike` sensor.
#[test]
fn test_ac5_unknown_verb_suggestion() {
    todo!("S-3.06 — test_ac5_unknown_verb_suggestion")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-7 / EC-11-062: DELETE FROM without WHERE → E-QUERY-022
// ─────────────────────────────────────────────────────────────────────────────

/// AC-7: `DELETE FROM armis_device_tags` (no WHERE) → `E-QUERY-022`
#[test]
fn test_ac7_delete_without_where() {
    todo!("S-3.06 — test_ac7_delete_without_where")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-8: SQL INSERT INTO … SELECT → DmlNode::InsertInto
// ─────────────────────────────────────────────────────────────────────────────

/// AC-8: `INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM
/// crowdstrike_hosts WHERE last_seen < 7d LIMIT 10` → `DmlNode::InsertInto`
#[test]
fn test_ac8_insert_into_select() {
    todo!("S-3.06 — test_ac8_insert_into_select")
}

// ─────────────────────────────────────────────────────────────────────────────
// Story task §7: DELETE FROM with WHERE → DmlNode::Delete
// ─────────────────────────────────────────────────────────────────────────────

/// `DELETE FROM armis_device_tags WHERE device_id = '123'` → `DmlNode::Delete`
#[test]
fn test_delete_from_with_where() {
    todo!("S-3.06 — test_delete_from_with_where")
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-11-065: empty verb registry → E-QUERY-023 for any terminal identifier
// ─────────────────────────────────────────────────────────────────────────────

/// EC-11-065: `WriteVerbRegistry` initialized with empty set → any terminal
/// identifier in pipe position produces `E-QUERY-023`.
#[test]
fn test_ec11_065_empty_registry_unknown_verb() {
    todo!("S-3.06 — test_ec11_065_empty_registry_unknown_verb")
}

// ─────────────────────────────────────────────────────────────────────────────
// Read-only pipeline — no write stage → PipeQuery.write == None
// ─────────────────────────────────────────────────────────────────────────────

/// A read-only pipeline (no write verb) produces `PipeQuery.write = None`.
/// This verifies that S-3.06 extensions do not break existing S-3.01 pipelines.
#[test]
fn test_read_only_pipeline_write_none() {
    todo!("S-3.06 — test_read_only_pipeline_write_none")
}

// ─────────────────────────────────────────────────────────────────────────────
// UPDATE statement → DmlNode::Update
// ─────────────────────────────────────────────────────────────────────────────

/// `UPDATE armis_devices SET status = 'quarantined' WHERE device_id = '42'` →
/// `DmlNode::Update` with one assignment and a WHERE filter.
#[test]
fn test_update_with_where() {
    todo!("S-3.06 — test_update_with_where")
}

/// `UPDATE armis_devices SET status = 'resolved'` (no WHERE) → `E-QUERY-022`
#[test]
fn test_update_without_where() {
    todo!("S-3.06 — test_update_without_where")
}

// ─────────────────────────────────────────────────────────────────────────────
// source_sensor extraction from SourceRef
// ─────────────────────────────────────────────────────────────────────────────

/// Verifies that `WriteNode.source_sensor` is correctly extracted from the
/// source stage: `crowdstrike_hosts` → `Some("crowdstrike")`.
#[test]
fn test_source_sensor_extracted_from_source_ref() {
    todo!("S-3.06 — test_source_sensor_extracted_from_source_ref")
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-021 corpus extension: write verb sequences and DML inputs must not panic
// ─────────────────────────────────────────────────────────────────────────────

/// VP-021 corpus extension: parse write verb sequence — must not panic.
/// Extends fuzz corpus as documented in S-3.06 §Verification Properties.
#[test]
fn test_vp021_write_verb_sequence_no_panic() {
    todo!("S-3.06 — test_vp021_write_verb_sequence_no_panic")
}

/// VP-021 corpus extension: parse DML inputs — must not panic.
#[test]
fn test_vp021_dml_inputs_no_panic() {
    todo!("S-3.06 — test_vp021_dml_inputs_no_panic")
}

/// VP-021 corpus extension: filter-mode write injection attempt — must not panic.
#[test]
fn test_vp021_filter_mode_write_injection_no_panic() {
    todo!("S-3.06 — test_vp021_filter_mode_write_injection_no_panic")
}
