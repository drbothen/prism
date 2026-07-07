//! Red Gate test stubs for S-PRISMQL-CASE-INSENSITIVE-001.
//!
//! Covers RG-001 through RG-018, RG-022, RG-023, RG-024 (22 tests in this file).
//! All bodies are `todo!()` per BC-5.38.001 Red Gate discipline.
//!
//! Behavioral contracts traced:
//!   BC-2.11.024 v1.0 — PrismQL IEQ/IIN/INE case-insensitive operators
//!   BC-2.11.002 v1.5 — filter-mode parsing (amended)
//!   BC-2.11.004 v1.13 — pipe-mode | where stage (amended)
//!   BC-2.11.018 v1.3 — normalized_pql echo (amended EC-11-057)
//!
//! Self-Check Rule (BC-5.38.005 invariant 1):
//! "If I include this non-todo!() function body, will the test for this function
//! pass trivially without any implementer work?"
//! Applied to every function below — all answer YES, so all bodies are `todo!()`.

// ─────────────────────────────────────────────────────────────────────────────
// RG-001: AC-001 — IEQ parses to Predicate::Compare { case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-001: `severity IEQ 'high'` parses to `Predicate::Compare { op: Eq, case_insensitive: true }`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" IEQ row;
/// BC-2.11.002 v1.5 amendment.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-001: parse 'severity IEQ \\'high\\'' and assert \
         Predicate::Compare {{ op: CompareOp::Eq, case_insensitive: true }} (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-002: AC-002 — IIN parses to Predicate::In { case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-002: `status IIN ('open', 'new')` parses to `Predicate::In { case_insensitive: true }`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" IIN row;
/// BC-2.11.002 v1.5 amendment.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-002: parse 'status IIN (\\'open\\', \\'new\\')' and assert \
         Predicate::In {{ negated: false, case_insensitive: true }} (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-003: AC-003 — INE parses to Predicate::Compare { op: Ne, case_insensitive: true }
// ─────────────────────────────────────────────────────────────────────────────

/// RG-003: `severity INE 'informational'` parses to `Predicate::Compare { op: Ne, case_insensitive: true }`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "New operators" INE row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-003: parse 'severity INE \\'informational\\'' and assert \
         Predicate::Compare {{ op: CompareOp::Ne, case_insensitive: true }} (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-004: AC-004 — IEQ keyword is parsed case-insensitively (ieq/IEQ/Ieq identical)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-004: `severity ieq 'high'`, `severity IEQ 'high'`, `severity Ieq 'high'` all produce
/// structurally identical ASTs with `case_insensitive: true`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "Operators parsed case-insensitively via kw()".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-004: parse all 3 ieq/IEQ/Ieq variants, assert structural \
         AST equality with case_insensitive=true (BC-2.11.024 v1.0 kw() combinator invariant)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-005: AC-005 — IIN parses before IN — no prefix-match collision
// ─────────────────────────────────────────────────────────────────────────────

/// RG-005: `status IIN ('open')` parses without error and produces
/// `Predicate::In { case_insensitive: true }` (NOT case_insensitive: false which
/// would indicate IIN was consumed as IN + stray I).
///
/// Traces to: BC-2.11.024 v1.0 invariant "IIN requires at least one value";
/// risk_mitigation: IIN-before-IN combinator ordering.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-005: parse 'status IIN (\\'open\\')' — must succeed \
         with case_insensitive=true, NOT fail with parse error indicating IIN consumed as IN \
         (verifies IIN-before-IN combinator ordering, BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-006: AC-008 — IEQ emits lower(field) = lower('val')
// ─────────────────────────────────────────────────────────────────────────────

/// RG-006: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Eq, case_insensitive: true }`
/// emits `lower(severity) = lower('high')`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" IEQ row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-006: call predicate_to_datafusion_sql with \
         Predicate::Compare {{ op: Eq, case_insensitive: true, lhs: Field('severity'), \
         rhs: Literal('high') }}, assert output == \"lower(severity) = lower('high')\" \
         (BC-2.11.024 v1.0 lower() lowering table)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-007: AC-009 — INE emits lower(field) != lower('val')
// ─────────────────────────────────────────────────────────────────────────────

/// RG-007: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Ne, case_insensitive: true }`
/// emits `lower(severity) != lower('low')`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" INE row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-007: call predicate_to_datafusion_sql with \
         Predicate::Compare {{ op: Ne, case_insensitive: true }}, assert output == \
         \"lower(severity) != lower('low')\" (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-008: AC-010 — IIN emits lower(field) IN (lower('v1'), lower('v2'), ...)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-008: `predicate_to_datafusion_sql` for `Predicate::In { case_insensitive: true }`
/// emits `lower(severity) IN (lower('high'), lower('critical'))`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "DataFusion SQL lowering" IIN row.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-008: call predicate_to_datafusion_sql with \
         Predicate::In {{ case_insensitive: true, values: ['high', 'critical'] }}, assert \
         output == \"lower(severity) IN (lower('high'), lower('critical'))\" (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-009: AC-011 — case-sensitive = emits unchanged (no lower() wrapping)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-009: `predicate_to_datafusion_sql` for `Predicate::Compare { op: Eq, case_insensitive: false }`
/// emits `severity = 'High'` with NO `lower()` wrapping.
///
/// Regression guard: ensures the existing case-sensitive path is unmodified.
/// Traces to: BC-2.11.024 v1.0 postcondition "Case-sensitive operators unchanged".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-009: call predicate_to_datafusion_sql with \
         Predicate::Compare {{ op: Eq, case_insensitive: false, rhs: Literal('High') }}, \
         assert output == \"severity = 'High'\" — no lower() wrapping (BC-2.11.024 v1.0 \
         regression guard)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-010: AC-012 — IEQ execution matches rows regardless of casing
// ─────────────────────────────────────────────────────────────────────────────

/// RG-010: Given a DataFusion MemTable with `{severity: 'High'}`,
/// `severity IEQ 'high'` and `severity IEQ 'HIGH'` both return the row.
///
/// Traces to: BC-2.11.024 v1.0 canonical test vectors #1 and #2.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-010: create DataFusion MemTable with severity='High', \
         execute 'severity IEQ \\'high\\'' and 'severity IEQ \\'HIGH\\'', assert both return \
         the row (BC-2.11.024 v1.0 test vectors #1/#2)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-011: AC-013 — case-sensitive = returns 0 rows when casing differs
// ─────────────────────────────────────────────────────────────────────────────

/// RG-011: Given a MemTable with `{severity: 'High'}`,
/// `severity = 'high'` (case-sensitive) returns 0 rows.
///
/// Regression guard — confirms case-sensitive default is unchanged.
/// Traces to: BC-2.11.024 v1.0 canonical test vector #6 "regression-no-change".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-011: create DataFusion MemTable with severity='High', \
         execute 'severity = \\'high\\'' (case-sensitive), assert 0 rows returned \
         (BC-2.11.024 v1.0 test vector #6 regression guard)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-012: AC-013b — IEQ available in pipe-mode | where stage
// ─────────────────────────────────────────────────────────────────────────────

/// RG-012: Pipe-mode `FROM crowdstrike_detections | where severity IEQ 'high' | head 5`
/// parses and executes successfully.
///
/// Traces to: BC-2.11.004 v1.13 amendment (IEQ/IIN/INE in | where via shared grammar);
/// BC-2.11.024 v1.0 invariant "valid in filter mode and pipe-mode | where stages".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-012: parse and execute pipe-mode query \
         'FROM ... | where severity IEQ \\'high\\' | head 5', assert no parse error \
         and query executes (BC-2.11.004 v1.13 amendment, BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-013: AC-014 — normalized_pql reflects IEQ/IIN in uppercase canonical form
// ─────────────────────────────────────────────────────────────────────────────

/// RG-013: `severity ieq 'high'` (lowercase keyword) normalizes to `severity IEQ 'high'`
/// (uppercase canonical) in the `normalized_pql` response field.
///
/// Traces to: BC-2.11.018 v1.3 amendment EC-11-057;
/// BC-2.11.024 v1.0 postcondition "normalized_pql round-trip" uppercase invariant.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-013: parse 'severity ieq \\'high\\'' (lowercase), \
         normalize via PqlNormalizer, assert normalized string contains 'IEQ' (uppercase) \
         not 'ieq' (BC-2.11.018 v1.3 EC-11-057, BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-014: AC-015 — normalized_pql round-trip: parse → normalize → reparse → same AST
// ─────────────────────────────────────────────────────────────────────────────

/// RG-014: `severity IEQ 'high'` → parse to `ast_original` → normalize → reparse to `ast_reparsed`
/// → `ast_original == ast_reparsed`.
///
/// Traces to: BC-2.11.024 v1.0 postcondition "normalized_pql round-trip" invariant;
/// BC-2.11.018 v1.3 amendment (round-trip extended to IEQ/IIN/INE).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-014: parse 'severity IEQ \\'high\\'', normalize, \
         reparse, assert ast_original == ast_reparsed (BC-2.11.024 v1.0 round-trip invariant, \
         BC-2.11.018 v1.3)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-015: AC-025 — No panic: repeated IEQ does not panic (VP-021 regression guard)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-015: `severity IEQ 'high' AND severity IEQ 'high'` does not panic — must return
/// a valid query result or structured error.
///
/// Traces to: BC-2.11.024 v1.0 canonical test vector "fuzz-seed regression";
/// VP-021 (parser never panics).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-015: parse and process \
         'severity IEQ \\'high\\' AND severity IEQ \\'high\\'', assert no panic \
         (VP-021 regression guard, BC-2.11.024 v1.0 fuzz-seed)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-016: AC-020 — IEQ with non-string RHS → E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-016: `severity IEQ 42` (integer literal on RHS) is rejected with E-QUERY-001.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IEQ/INE with non-string literal RHS".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-016: parse 'severity IEQ 42', assert Err(E-QUERY-001) \
         with message indicating IEQ requires a string literal RHS (BC-2.11.024 v1.0 error case)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-017: AC-021 — IIN with empty list → E-QUERY-001
// ─────────────────────────────────────────────────────────────────────────────

/// RG-017: `severity IIN ()` (empty membership list) is rejected with E-QUERY-001.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-001: IIN with empty membership list";
/// BC-2.11.024 v1.0 invariant "IIN requires at least one value".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-017: parse 'severity IIN ()', assert Err(E-QUERY-001) \
         indicating IIN requires at least one value (BC-2.11.024 v1.0 invariant)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-018: AC-022 — IEQ on integer column → E-QUERY-002 QueryTypeMismatch
// ─────────────────────────────────────────────────────────────────────────────

/// RG-018: `severity_id IEQ 'high'` against an integer column returns E-QUERY-002 (QueryTypeMismatch),
/// because `lower()` is not applicable to non-string types.
///
/// Traces to: BC-2.11.024 v1.0 error case "E-QUERY-002: IEQ/IIN/INE on non-string column";
/// BC-2.11.024 v1.0 precondition "field referenced by IEQ/IIN/INE is a string-type column".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-018: execute 'severity_id IEQ \\'high\\'' against \
         a DataFusion schema where severity_id is an integer column, assert Err(E-QUERY-002 \
         QueryTypeMismatch) naming the field and suggesting the string column (BC-2.11.024 v1.0)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-022: AC-019 — GROUP BY severity produces at most 7 buckets after normalization
// ─────────────────────────────────────────────────────────────────────────────

/// RG-022: Simulated cross-sensor records — CrowdStrike `'High'` (3) + Armis-like `'HIGH'` (2)
/// pre-normalization — after adapter boundary normalization, `GROUP BY severity` yields
/// one `'High'` bucket with 5 rows, not two fragmented buckets.
///
/// Traces to: BC-2.02.013 v1.0 canonical test vector "GROUP BY severity cross-sensor";
/// EC-02-026; ADR-047 §Consequences "GROUP BY correct after normalization".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-022: create MemTable with severity rows from \
         multiple simulated sensors (CrowdStrike 'High' x3, Armis-like 'HIGH' x2 pre-norm), \
         apply adapter normalization, execute GROUP BY severity, assert 'High' bucket = 5 \
         (BC-2.02.013 v1.0, ADR-047 §Consequences)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-023: AC-023 — Grammar resource MCP resource includes IEQ/IIN/INE
// ─────────────────────────────────────────────────────────────────────────────

/// RG-023: The PrismQL grammar reference MCP resource (BC-2.11.022 / ADR-045 parity gate)
/// includes `IEQ`, `IIN`, and `INE` in the operator table.
///
/// Traces to: BC-2.11.024 v1.0 ADR-047 §D.4 discoverability;
/// BC-2.11.002 v1.5 amendment (IEQ/IIN/INE in operator table).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-023: locate and inspect the PrismQL grammar reference \
         MCP resource (BC-2.11.022 / ADR-045 parity gate), assert 'IEQ', 'IIN', 'INE' appear \
         in the operator table (ADR-047 §D.4, BC-2.11.002 v1.5)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-024: AC-024 — prism describe output includes IEQ example with OCSF casing note
// ─────────────────────────────────────────────────────────────────────────────

/// RG-024: `prism describe <sensor_table>` output for a table with a `severity` column
/// includes at least one IEQ example AND the OCSF casing note about Title-case storage.
///
/// Traces to: ADR-047 §D.4; BC-2.11.024 v1.0 architecture anchor "discoverability examples".
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-024: inspect prism describe output for a sensor table \
         with a severity column, assert the output contains an IEQ example AND the OCSF casing \
         note ('OCSF severity is stored as Title-case ... use IEQ/IIN') (ADR-047 §D.4)"
    )
}
