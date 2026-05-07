//! Unit tests for `explain_query` behavior.
//!
//! All tests in this module are RED-GATE stubs: they panic immediately with the
//! story AC text they validate. The implementer must replace `todo!()` with real
//! assertions once `explain()` is implemented.
//!
//! # BC References
//! - BC-2.11.010 — `explain_query` MCP Tool
//!
//! Story: S-3.03

// Stub phase: allow unused imports / dead code in test module.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::explain::{explain, ExplainOptions};
use prism_core::OrgSlug;

// ---------------------------------------------------------------------------
// AC-1: No sensor API calls on valid query
// ---------------------------------------------------------------------------

/// AC-1 (BC-2.11.010): Given a valid PrismQL query, When `explain_query` is
/// called, Then an `ExplainResult` is returned without issuing any sensor API
/// calls (verified by asserting zero calls to `AdapterRegistry`).
#[test]
fn test_ac1_explain_valid_query_no_sensor_calls() {
    todo!(
        "AC-1 (S-3.03 / BC-2.11.010): \
        explain_query returns ExplainResult for a valid query \
        WITHOUT issuing any sensor API calls. \
        Implementer: call explain() with a valid filter query, \
        verify Ok(ExplainResult) is returned, \
        and verify AdapterRegistry.fetch() was never invoked."
    );
}

// ---------------------------------------------------------------------------
// AC-2: Push-down predicates populated for REQUIRED columns
// ---------------------------------------------------------------------------

/// AC-2 (BC-2.11.010): Given a query with `WHERE severity_id >= 3` where
/// `severity_id` is a REQUIRED push-downable column, When explained, Then
/// `ExplainSource.api_filters_pushed` contains the translated filter for the
/// relevant source.
#[test]
fn test_ac2_pushdown_predicates_populated_for_required_column() {
    todo!(
        "AC-2 (S-3.03 / BC-2.11.010): \
        explain_query with WHERE severity_id >= 3 must populate \
        ExplainSource.api_filters_pushed with the translated filter string \
        for the relevant sensor source. \
        Implementer: use a sensor spec with severity_id as ColumnOptions::Required \
        and verify the push-down appears in execution_plan.sensors_to_query."
    );
}

// ---------------------------------------------------------------------------
// AC-3: Two sources → two ExplainSource entries
// ---------------------------------------------------------------------------

/// AC-3 (BC-2.11.010): Given a query targeting two sources
/// (`crowdstrike.detections` and `claroty.alerts`), When explained, Then
/// `ExplainResult.execution_plan.sensors_to_query` contains two `ExplainSource`
/// entries, one per source.
#[test]
fn test_ac3_multi_source_query_yields_two_explain_sources() {
    todo!(
        "AC-3 (S-3.03 / BC-2.11.010): \
        explain_query targeting crowdstrike.detections and claroty.alerts \
        must return ExplainResult.execution_plan.sensors_to_query with exactly 2 entries, \
        one for each source. \
        Implementer: construct a SQL or pipe query over both tables and \
        assert execution_plan.sensors_to_query.len() == 2."
    );
}

// ---------------------------------------------------------------------------
// AC-4: Syntactically invalid query → structured parse error
// ---------------------------------------------------------------------------

/// AC-4 (BC-2.11.010): Given a syntactically invalid PrismQL query, When
/// `explain_query` is called, Then a structured parse error is returned
/// (not a panic), and `ExplainResult` is not produced.
#[test]
fn test_ac4_invalid_query_returns_parse_error_not_panic() {
    todo!(
        "AC-4 (S-3.03 / BC-2.11.010): \
        explain_query with a syntactically invalid query (e.g., '<invalid>') \
        must return Err(PrismError::ParseError {{..}}) — never panic — \
        and must NOT return an Ok(ExplainResult). \
        Implementer: call explain() with an invalid query string and \
        assert matches!(result, Err(_))."
    );
}

// ---------------------------------------------------------------------------
// AC-5: clients: None → all configured clients, no fan-out
// ---------------------------------------------------------------------------

/// AC-5 (BC-2.11.010 / BC-2.11.011): Given a query with `clients: None`,
/// When explained, Then `ExplainResult` lists all configured client IDs
/// in the cost estimate / resolution context without any sensor fan-out
/// occurring.
#[test]
fn test_ac5_clients_none_lists_all_clients_without_fanout() {
    todo!(
        "AC-5 (S-3.03 / BC-2.11.010): \
        explain_query with ExplainOptions {{ clients: None, .. }} must resolve \
        all configured clients from the ClientRegistry and include them in the \
        ExplainResult (e.g., in CostEstimate.per_sensor_latency_ms keys or a \
        clients field) WITHOUT issuing any sensor API calls. \
        Implementer: construct a ClientRegistry with 3 clients, call explain(), \
        and verify all 3 appear in the result with zero sensor fetches."
    );
}

// ---------------------------------------------------------------------------
// EC-11-050: No sensor sources resolved → E-QUERY-001
// ---------------------------------------------------------------------------

/// EC-11-050 (S-3.03): Query with no sensor sources resolved → `E-QUERY-001`
/// returned; `ExplainResult` not produced.
#[test]
fn test_ec11_050_no_sources_resolved_returns_error() {
    todo!(
        "EC-11-050 (S-3.03): \
        explain_query for a query where source resolution yields no sensor sources \
        must return Err containing E-QUERY-001 and must NOT return ExplainResult. \
        Implementer: use a query referencing a non-existent sensor table and \
        assert the error kind matches E-QUERY-001."
    );
}

// ---------------------------------------------------------------------------
// EC-11-051: All predicates non-push-downable → post_filter_predicates populated
// ---------------------------------------------------------------------------

/// EC-11-051 (S-3.03): Query with all predicates non-push-downable →
/// `ExplainSource.api_filters_pushed` is empty; `post_filter_predicates`
/// contains all predicates.
#[test]
fn test_ec11_051_non_pushdown_predicates_go_to_post_filter() {
    todo!(
        "EC-11-051 (S-3.03): \
        explain_query with a WHERE clause over non-push-downable columns (ColumnOptions::Default) \
        must produce ExplainSource.api_filters_pushed == [] and \
        ExplainSource.post_filter_predicates containing all predicates. \
        Implementer: use a sensor spec with all columns as Default, \
        assert api_filters_pushed is empty and post_filter_predicates is non-empty."
    );
}

// ---------------------------------------------------------------------------
// EC-11-052: Multi-client, clients: None → all clients listed, no fan-out
// ---------------------------------------------------------------------------

/// EC-11-052 (S-3.03): Multi-client query with `clients: None` →
/// `ExplainResult` lists all configured client IDs without any sensor fan-out.
///
/// Note: overlaps with AC-5; this test focuses on the multi-client case with
/// an explicit assertion that no sensor I/O occurred.
#[test]
fn test_ec11_052_multi_client_none_no_fanout() {
    todo!(
        "EC-11-052 (S-3.03): \
        explain_query with clients: None against a ClientRegistry containing \
        multiple clients must list all clients in ExplainResult \
        WITHOUT triggering any sensor adapter fetch calls. \
        Implementer: spy/mock on AdapterRegistry to confirm zero fetch invocations."
    );
}

// ---------------------------------------------------------------------------
// EC-11-053: Query exceeding PRISM_MAX_QUERY_SIZE → parse error
// ---------------------------------------------------------------------------

/// EC-11-053 (S-3.03): Parser encounters query exceeding
/// `PRISM_MAX_QUERY_SIZE` → parse error returned before plan generation.
#[test]
fn test_ec11_053_oversized_query_returns_parse_error() {
    todo!(
        "EC-11-053 (S-3.03 / DI-019): \
        explain_query with a query string exceeding PRISM_MAX_QUERY_SIZE \
        must return a parse/security error BEFORE attempting plan generation. \
        Implementer: construct a query of 64KB+1 bytes and assert \
        matches!(result, Err(_)) with an appropriate security-limit error kind."
    );
}

// ---------------------------------------------------------------------------
// EC-11-025: Over materialization limit → warning, not error (BC-2.11.010)
// ---------------------------------------------------------------------------

/// EC-11-025 (BC-2.11.010): Explain a query that would exceed the 10K
/// materialization limit → Explain succeeds (not an error); `estimated_cost`
/// includes a warning that the estimated record count exceeds 10K and the
/// query would fail at execution time.
#[test]
fn test_ec11_025_over_materialization_limit_succeeds_with_warning() {
    todo!(
        "EC-11-025 (BC-2.11.010 / DI-019): \
        explain_query for a query whose estimated_row_count > 10,000 must \
        return Ok(ExplainResult) — not Err — and \
        ExplainResult.estimated_cost.warnings must be non-empty, \
        containing text indicating the query would fail at execution time. \
        Implementer: stub a sensor spec whose count_hint returns > 10_000 and \
        assert result.is_ok() and estimated_cost.warnings.len() > 0."
    );
}

// ---------------------------------------------------------------------------
// EC-11-026: Invalid field names → error with similar_fields suggestions
// ---------------------------------------------------------------------------

/// EC-11-026 (BC-2.11.010): Explain a query with invalid field names →
/// error with `similar_fields` suggestions.
#[test]
fn test_ec11_026_invalid_field_names_return_error_with_suggestions() {
    todo!(
        "EC-11-026 (BC-2.11.010): \
        explain_query with a WHERE clause referencing a field that does not \
        exist in the sensor schema must return Err containing field resolution \
        error details including a similar_fields suggestion list. \
        Implementer: use a schema that does not contain the queried field \
        and assert the error variant contains similar field suggestions."
    );
}

// ---------------------------------------------------------------------------
// DI-004: Audit entry emitted for every explain invocation
// ---------------------------------------------------------------------------

/// DI-004 (BC-2.11.010): An audit entry IS emitted for `explain_query`
/// invocations — it is an MCP tool invocation and must be audited for
/// SOC 2 compliance.
#[test]
fn test_di004_audit_entry_emitted_on_explain() {
    todo!(
        "DI-004 (BC-2.11.010): \
        Every explain_query invocation must emit an audit entry recording \
        the query, scoping parameters, and explain result summary. \
        Implementer: capture audit output (e.g., via a test AuditSink) and \
        assert an audit record is present after calling explain(). \
        The audit entry must be emitted even for queries that return errors."
    );
}
