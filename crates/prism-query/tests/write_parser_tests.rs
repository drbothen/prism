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

#![allow(unused_imports, unused_variables, dead_code)]

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

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Filter-mode rejection per verb (E-QUERY-010 / AC-4 extensions)
//
// Story S-3.06 AC-4 requires that every registered write verb is rejected at
// grammar level when used in filter mode — not just a generic "contain".
// Each verb needs its own test so that a per-verb regression is visible.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-4 gap: `severity_id >= 4 | tag key="x"` in filter mode →
/// parse error indicating writes not permitted. Verifies the `tag` verb
/// (with args) is also rejected, not just bare verbs.
#[test]
fn test_BC_2_11_004_filter_mode_rejects_tag_verb_with_args() {
    todo!("S-3.06 gap-fill — filter mode must reject 'tag' verb with key=value args")
}

/// AC-4 gap: `severity_id >= 4 | acknowledge` in filter mode →
/// parse error. Verifies `acknowledge` verb (different from `contain`) is
/// also hard-rejected at grammar level.
#[test]
fn test_BC_2_11_004_filter_mode_rejects_acknowledge_verb() {
    todo!("S-3.06 gap-fill — filter mode must reject 'acknowledge' verb")
}

/// AC-4 gap: Write verb appearing as a bare identifier in a predicate
/// (e.g., `contain = 1`) must NOT trigger the write-rejection error —
/// it's a comparison against a field named `contain`, not a write stage.
/// The write rejection fires only when `|` precedes the verb.
#[test]
fn test_BC_2_11_004_filter_mode_field_named_contain_is_not_rejected() {
    todo!("S-3.06 gap-fill — field named 'contain' in predicate is valid; no write rejection")
}

/// EC-11-065 gap: Filter mode with an empty registry — no verbs registered.
/// Any `| identifier` sequence should still be handled gracefully (either
/// rejected as unknown or accepted as non-write; no panic).
#[test]
fn test_BC_2_11_004_filter_mode_empty_registry_no_panic() {
    todo!("S-3.06 gap-fill — filter mode with empty registry: no panic on any input")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Internal prism table protection — all known tables (E-QUERY-010)
//
// AC-3 / EC-11-061: every `prism_*` table must be protected. The stub-architect
// tests only cover `prism_alerts`. Each internal table needs its own test.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-3 gap: `DELETE FROM prism_cases WHERE id = '1'` →
/// `E-QUERY-010` (prism_cases is write-protected).
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_cases() {
    todo!("S-3.06 gap-fill — DELETE FROM prism_cases must return E-QUERY-010")
}

/// AC-3 gap: `INSERT INTO prism_rules (id) SELECT id FROM alerts LIMIT 1` →
/// `E-QUERY-010` (prism_rules is write-protected).
#[test]
fn test_BC_2_11_004_internal_table_insert_prism_rules() {
    todo!("S-3.06 gap-fill — INSERT INTO prism_rules must return E-QUERY-010")
}

/// AC-3 gap: `UPDATE prism_schedules SET active = true WHERE id = '1'` →
/// `E-QUERY-010` (prism_schedules is write-protected).
#[test]
fn test_BC_2_11_004_internal_table_update_prism_schedules() {
    todo!("S-3.06 gap-fill — UPDATE prism_schedules must return E-QUERY-010")
}

/// AC-3 gap: `DELETE FROM prism_audit WHERE ts < '2026-01-01T00:00:00Z'` →
/// `E-QUERY-010` (prism_audit is write-protected).
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_audit() {
    todo!("S-3.06 gap-fill — DELETE FROM prism_audit must return E-QUERY-010")
}

/// AC-3 gap: `DELETE FROM prism_aliases WHERE alias = 'old'` →
/// `E-QUERY-010` (prism_aliases is write-protected).
#[test]
fn test_BC_2_11_004_internal_table_delete_prism_aliases() {
    todo!("S-3.06 gap-fill — DELETE FROM prism_aliases must return E-QUERY-010")
}

/// AC-3 gap: Arbitrary `prism_` prefix tables (not in the known list) must
/// also be rejected — the check is prefix-based, not an allowlist of known tables.
/// e.g., `DELETE FROM prism_unknown_future_table WHERE id = '1'` → `E-QUERY-010`.
#[test]
fn test_BC_2_11_004_internal_table_unknown_prism_prefix() {
    todo!("S-3.06 gap-fill — any prism_* table must return E-QUERY-010, not just known ones")
}

/// AC-3 gap: Table name `prism` (no underscore suffix) must NOT be rejected —
/// the write-protection prefix is `prism_`, not `prism`.
/// e.g., `DELETE FROM prism WHERE id = '1'` should NOT produce E-QUERY-010.
#[test]
fn test_BC_2_11_004_table_named_prism_no_underscore_is_allowed() {
    todo!("S-3.06 gap-fill — 'prism' alone (no underscore) is not a protected table name")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Unbounded write protection (E-QUERY-022) — INSERT path
//
// AC-7 / EC-11-062 stubs only cover DELETE. INSERT INTO SELECT without LIMIT
// or WHERE needs coverage per Story §SQL mode parser.
// ─────────────────────────────────────────────────────────────────────────────

/// Unbounded INSERT gap: `INSERT INTO armis_tags (id) SELECT id FROM events`
/// with no WHERE and no LIMIT → `E-QUERY-022` (unbounded write).
#[test]
fn test_BC_2_11_004_insert_select_without_limit_or_where_is_unbounded() {
    todo!("S-3.06 gap-fill — INSERT INTO...SELECT without LIMIT or WHERE returns E-QUERY-022")
}

/// Unbounded INSERT safe: `INSERT INTO armis_tags (id) SELECT id FROM events WHERE active = true`
/// has a WHERE on the source SELECT → should parse successfully (no E-QUERY-022).
#[test]
fn test_BC_2_11_004_insert_select_with_where_is_bounded() {
    todo!("S-3.06 gap-fill — INSERT INTO...SELECT with WHERE on source SELECT is allowed")
}

/// Unbounded INSERT safe: `INSERT INTO armis_tags (id) SELECT id FROM events LIMIT 500`
/// has a LIMIT on the source SELECT → should parse successfully (no E-QUERY-022).
#[test]
fn test_BC_2_11_004_insert_select_with_limit_is_bounded() {
    todo!("S-3.06 gap-fill — INSERT INTO...SELECT with LIMIT on source SELECT is allowed")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: WriteVerbRegistry trait impls — HashSet<String> path
//
// Story dev notes: `HashSet<String>` implements `WriteVerbSource` for test
// injection. Each method needs a dedicated test.
// ─────────────────────────────────────────────────────────────────────────────

/// WriteVerbSource for HashSet: `is_registered_verb` returns true for a verb
/// in the set and false for one that is not.
#[test]
fn test_BC_2_11_004_hashset_verb_source_is_registered_verb() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    verbs.insert("tag".to_string());
    todo!(
        "S-3.06 gap-fill — HashSet<String>::is_registered_verb must return true for 'contain', false for 'acknowledge'"
    )
}

/// WriteVerbSource for HashSet: `all_verbs` returns all verbs in the set.
#[test]
fn test_BC_2_11_004_hashset_verb_source_all_verbs() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    verbs.insert("tag".to_string());
    todo!("S-3.06 gap-fill — HashSet<String>::all_verbs must return both 'contain' and 'tag'")
}

/// WriteVerbSource for HashSet: `verbs_for_sensor` returns all verbs
/// (test sets are not sensor-partitioned, per Story dev notes).
#[test]
fn test_BC_2_11_004_hashset_verb_source_verbs_for_sensor() {
    let mut verbs: HashSet<String> = HashSet::new();
    verbs.insert("contain".to_string());
    todo!("S-3.06 gap-fill — HashSet<String>::verbs_for_sensor returns all verbs for any sensor")
}

/// WriteVerbRegistry from HashSet source: is_write_verb returns true for
/// registered verbs and false for unregistered verbs.
#[test]
fn test_BC_2_11_004_registry_is_write_verb() {
    todo!(
        "S-3.06 gap-fill — WriteVerbRegistry::is_write_verb must return true for registered verbs"
    )
}

/// WriteVerbRegistry from HashSet source: is_empty() returns false when
/// verbs are registered, true only when empty.
#[test]
fn test_BC_2_11_004_registry_is_empty_populated_vs_default() {
    // WriteVerbRegistry::default() creates an empty registry.
    let empty_registry = WriteVerbRegistry::default();
    assert!(
        empty_registry.is_empty(),
        "default WriteVerbRegistry must be empty"
    );
    todo!("S-3.06 gap-fill — populated WriteVerbRegistry::is_empty() must return false")
}

/// WriteVerbRegistry: all_verbs() returns all verbs that were loaded from the source.
#[test]
fn test_BC_2_11_004_registry_all_verbs_matches_source() {
    todo!("S-3.06 gap-fill — WriteVerbRegistry::all_verbs must iterate all loaded verbs")
}

/// WriteVerbRegistry: verbs_for_sensor returns empty slice for unknown sensor.
#[test]
fn test_BC_2_11_004_registry_verbs_for_sensor_unknown_returns_empty() {
    todo!("S-3.06 gap-fill — WriteVerbRegistry::verbs_for_sensor for unknown sensor returns empty")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: DML statement parsing — positive and negative per operation type
// ─────────────────────────────────────────────────────────────────────────────

/// DML UPDATE gap: `UPDATE crowdstrike_hosts SET status = 'contained', priority = 'high'
/// WHERE device_id = 'abc'` → `DmlNode::Update` with two assignments.
/// Verifies multi-assignment UPDATE parsing.
#[test]
fn test_BC_2_11_004_update_multiple_assignments() {
    todo!("S-3.06 gap-fill — UPDATE with multiple SET assignments parses correctly")
}

/// DML INSERT gap: malformed INSERT (missing SELECT) →
/// parse error (not a panic). Verifies error path of INSERT parser.
#[test]
fn test_BC_2_11_004_insert_missing_select_is_parse_error() {
    todo!("S-3.06 gap-fill — INSERT INTO without SELECT produces parse error, not panic")
}

/// DML DELETE gap: malformed DELETE (missing FROM) →
/// parse error (not a panic). Verifies grammar catches missing keyword.
#[test]
fn test_BC_2_11_004_delete_missing_from_is_parse_error() {
    todo!("S-3.06 gap-fill — DELETE without FROM produces parse error, not panic")
}

/// DML UPDATE gap: malformed UPDATE (missing WHERE, missing SET) →
/// parse error (not a panic, not an unbounded-write error — it's malformed syntax).
#[test]
fn test_BC_2_11_004_update_malformed_no_set_clause() {
    todo!("S-3.06 gap-fill — UPDATE without SET clause produces parse error (not panic)")
}

/// DML: verify target_table field is exactly the parsed table name
/// (no trimming artifacts, no case mutation) for INSERT INTO.
#[test]
fn test_BC_2_11_004_dml_node_target_table_preserved_exactly() {
    todo!("S-3.06 gap-fill — DmlNode.target_table must equal exactly the input table name")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Error constructor message content (E-QUERY-010/022/023/024)
//
// The ParseError constructors are `todo!()` stubs. Tests must verify the
// correct error code prefix appears in the message after implementation.
// ─────────────────────────────────────────────────────────────────────────────

/// E-QUERY-010 constructor: message must contain the error code and the
/// table name.
#[test]
fn test_BC_2_11_004_error_010_message_contains_code_and_table() {
    todo!(
        "S-3.06 gap-fill — ParseError::internal_table_write_protected message must contain E-QUERY-010 and table name"
    )
}

/// E-QUERY-022 constructor: message must contain the error code and a
/// suggestion to add WHERE or LIMIT.
#[test]
fn test_BC_2_11_004_error_022_message_contains_code_and_suggestion() {
    todo!(
        "S-3.06 gap-fill — ParseError::unbounded_write message must contain E-QUERY-022 and WHERE/LIMIT suggestion"
    )
}

/// E-QUERY-023 constructor: message must contain the error code, the
/// unrecognised verb, and the available-verb suggestion list.
#[test]
fn test_BC_2_11_004_error_023_message_contains_code_verb_and_suggestions() {
    todo!(
        "S-3.06 gap-fill — ParseError::unknown_write_verb message must contain E-QUERY-023, the verb, and suggestion list"
    )
}

/// E-QUERY-024 constructor: message must contain the error code, the verb
/// that appeared in non-terminal position, and the pipeline position.
#[test]
fn test_BC_2_11_004_error_024_message_contains_code_verb_and_position() {
    todo!(
        "S-3.06 gap-fill — ParseError::write_stage_not_terminal message must contain E-QUERY-024, verb, and position"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Pipe mode write verb — additional edge cases
// ─────────────────────────────────────────────────────────────────────────────

/// Multiple write stages: `FROM crowdstrike_hosts | contain | tag key="x"` →
/// `E-QUERY-024`. Both verbs are registered; the second one is non-terminal
/// (i.e., there are stages after `contain`). Must not silently produce a
/// double-write AST.
#[test]
fn test_BC_2_11_004_two_write_verbs_in_sequence_rejected() {
    todo!("S-3.06 gap-fill — two consecutive write verbs must produce E-QUERY-024")
}

/// Write verb case sensitivity: if verb registry is case-sensitive, then
/// `FROM crowdstrike_hosts | CONTAIN` (uppercase) must not match the
/// registered lowercase verb `contain` and should produce E-QUERY-023.
/// Documents the contract (case policy must be consistent).
#[test]
fn test_BC_2_11_004_pipe_write_verb_case_sensitivity_policy() {
    todo!(
        "S-3.06 gap-fill — write verb matching is case-sensitive: 'CONTAIN' != 'contain' → E-QUERY-023 or success (document which)"
    )
}

/// Write stage with no source prefix: `| where x = 1 | contain` (starts
/// with `|`, no FROM) — should parse if EC-11-009 no-source pipelines are
/// supported, with `source_sensor = None` in the WriteNode.
#[test]
fn test_BC_2_11_004_write_stage_no_source_prefix_sensor_is_none() {
    todo!("S-3.06 gap-fill — pipeline with no FROM source: WriteNode.source_sensor must be None")
}

/// Write stage immediately after FROM (no intermediate stages):
/// `FROM crowdstrike_hosts | contain` → valid; no stages, write = Some(WriteNode).
#[test]
fn test_BC_2_11_004_write_stage_no_intermediate_stages() {
    todo!("S-3.06 gap-fill — FROM source | write_verb with no intermediate stages is valid")
}

/// Write stage with integer literal arg: `FROM hosts | tag priority=42` →
/// `WriteArg { key: "priority", value: Literal::Integer(42) }`.
/// Tests that non-string literal args are parsed correctly.
#[test]
fn test_BC_2_11_004_write_arg_integer_literal() {
    todo!("S-3.06 gap-fill — write arg with integer literal value must parse correctly")
}

/// Write stage with boolean literal arg: `FROM hosts | tag critical=true` →
/// `WriteArg { key: "critical", value: Literal::Bool(true) }`.
#[test]
fn test_BC_2_11_004_write_arg_boolean_literal() {
    todo!("S-3.06 gap-fill — write arg with boolean literal value must parse correctly")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Query size / security guard applies to write queries
//
// Story: "S-3.01's security.rs query size check (PRISM_MAX_QUERY_SIZE) applies
// to write queries identically — no exemptions."
// ─────────────────────────────────────────────────────────────────────────────

/// A write query that exceeds PRISM_MAX_QUERY_SIZE (64KB) must be rejected
/// with E-QUERY-003 (size limit), not silently parsed.
#[test]
fn test_BC_2_11_004_oversized_write_query_rejected_before_parse() {
    use prism_query::PrismQlParser;
    // Construct a write query that exceeds 64KB.
    let giant_table: String = "a".repeat(66_000);
    let oversized = format!("DELETE FROM {} WHERE id = '1'", giant_table);
    let result = PrismQlParser::parse(&oversized);
    todo!(
        "S-3.06 gap-fill — write query > 64KB must return Err with E-QUERY-003, got: {:?}",
        result
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: VP-021 corpus extension — additional panic-safety seeds
//
// Story: "extend fuzz corpus to include write verb sequences, DML statements,
// and filter-mode write injection attempts." Min 5 representative seeds.
// ─────────────────────────────────────────────────────────────────────────────

/// VP-021 corpus seed 4: `FROM crowdstrike_hosts | where x = 1 | tag key="v"` —
/// single verb with one write arg. Must not panic.
#[test]
fn test_vp021_corpus_seed_single_verb_with_one_arg() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | tag key=\"v\"");
    todo!("S-3.06 gap-fill VP-021 corpus seed 4: write verb with arg must not panic")
}

/// VP-021 corpus seed 5: `UPDATE armis_devices SET status = 'ok' WHERE id = '1'` —
/// well-formed UPDATE. Must not panic.
#[test]
fn test_vp021_corpus_seed_update_with_where() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("UPDATE armis_devices SET status = 'ok' WHERE id = '1'");
    todo!("S-3.06 gap-fill VP-021 corpus seed 5: UPDATE with WHERE must not panic")
}

/// VP-021 corpus seed 6: `DELETE FROM armis_device_tags` (no WHERE) —
/// unbounded write that should return E-QUERY-022. Must not panic.
#[test]
fn test_vp021_corpus_seed_delete_without_where() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("DELETE FROM armis_device_tags");
    todo!("S-3.06 gap-fill VP-021 corpus seed 6: DELETE without WHERE returns error, not panic")
}

/// VP-021 corpus seed 7: `UPDATE prism_alerts SET x = 1` — internal table
/// write attempt (E-QUERY-010 expected). Must not panic.
#[test]
fn test_vp021_corpus_seed_internal_table_attempt() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("UPDATE prism_alerts SET x = 1");
    todo!("S-3.06 gap-fill VP-021 corpus seed 7: internal table write must error, not panic")
}

/// VP-021 corpus seed 8: `| nonexistent_verb_xyz` — malformed pipe with
/// unknown verb (E-QUERY-023 expected). Must not panic.
#[test]
fn test_vp021_corpus_seed_malformed_verb() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("FROM crowdstrike_hosts | nonexistent_verb_xyz");
    todo!("S-3.06 gap-fill VP-021 corpus seed 8: unknown verb must error, not panic")
}

/// VP-021 corpus seed 9: `INSERT INTO x VALUES (1)` — SQL INSERT without SELECT
/// (malformed per the grammar). Must not panic.
#[test]
fn test_vp021_corpus_seed_insert_values_not_select() {
    use prism_query::PrismQlParser;
    let _ = PrismQlParser::parse("INSERT INTO x VALUES (1)");
    todo!("S-3.06 gap-fill VP-021 corpus seed 9: INSERT VALUES (not SELECT) must not panic")
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Perimeter compliance — new symbols must be pub(crate) not pub
//
// BC-2.11.006 INV-SEC-PERIMETER-001: parser sub-functions introduced by S-3.06
// must never be `pub`. These compile-fail-style tests use the public API to
// verify the feature works, while the perimeter gate enforces visibility.
//
// Note: actual `pub` enforcement is tested by the
// `tests/external/perimeter-violation/` compile-fail crate (PR #127).
// These tests verify behavioral correctness via the public entry point.
// ─────────────────────────────────────────────────────────────────────────────

/// Perimeter test: write queries must be reachable via `PrismQlParser::parse`
/// (the sole public entry point). Verifying that the public API routes to
/// the write parser path without exposing `parse_pipe_with_write` directly.
#[test]
fn test_BC_2_11_004_write_query_reachable_via_public_entry_point() {
    use prism_query::PrismQlParser;
    // A write query through the sole public entry point must not panic.
    // The result may be Ok or Err depending on mode detection and registry
    // state, but it MUST NOT panic.
    let result = PrismQlParser::parse("FROM crowdstrike_hosts | where x = 1 | contain");
    todo!(
        "S-3.06 gap-fill perimeter — public PrismQlParser::parse routes write queries correctly; got: {:?}",
        result
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// GAP-FILL: Property-based test — write AST round-trip (proptest)
//
// Story: "add a proptest verifying that random valid AST nodes round-trip
// through format → parse and produce equivalent ASTs."
//
// The round-trip uses WriteNode's Display/format implementation (to be added
// by the implementer) and verifies that re-parsing produces an equivalent AST.
// ─────────────────────────────────────────────────────────────────────────────

/// Proptest: arbitrary write verb + no-arg write node round-trips through
/// format → parse to produce an equivalent WriteNode.
///
/// Strategy: generate random verb strings from the test registry, format them
/// into a query string, and verify the re-parsed WriteNode matches the original.
/// Exercises the invariant: parse(format(node)) == node (semantic equivalence).
#[test]
fn test_BC_2_11_004_proptest_write_node_roundtrip() {
    todo!(
        "S-3.06 gap-fill proptest — random WriteNode round-trips through format→parse must be semantically equivalent"
    )
}
