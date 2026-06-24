//! Filter mode end-to-end execution tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area D.
//!
//! BC-2.11.023 postcondition (AC-012): Filter mode must execute end-to-end via
//! `QueryEngine::execute`, not just parse. These tests are called by
//! `test_BC_2_11_023_filter_mode_end_to_end_execution` in `grammar_remediation.rs`.
//!
//! All test bodies call `todo!()` — the implementer writes the assertions.

/// Minimal filter mode execution: no explicit source, just a predicate expression.
/// Assert: `QueryEngine::execute("severity = 'high'", ...)` returns a non-error
/// `ExecuteResult` and the rows satisfy the predicate.
#[test]
fn test_filter_mode_simple_predicate() {
    todo!(
        "BC-2.11.023 AC-012: execute Filter mode query with bare predicate and assert result; \
         implementer wires QueryEngine::execute for Filter mode"
    )
}

/// Filter mode execution with explicit source: `crowdstrike.detections | severity = 'critical'`.
/// Assert: `QueryEngine::execute(...)` routes to the `crowdstrike` sensor, applies the
/// predicate, and returns rows where `severity == 'critical'`.
#[test]
fn test_filter_mode_with_source() {
    todo!(
        "BC-2.11.023 AC-012: execute Filter mode query with explicit sensor.table source \
         and assert predicate is applied; implementer wires source-qualified filter dispatch"
    )
}
