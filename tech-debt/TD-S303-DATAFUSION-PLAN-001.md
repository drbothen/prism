# TD-S303-DATAFUSION-PLAN-001: DataFusion logical plan elision in explain

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

The S-3.03 story spec (§Tasks line 88-90) and original architecture narrative
called for using DataFusion `SessionContext::create_logical_plan()` against
schema-only MemTables to produce a `datafusion_logical_plan: String` field
in `ExplainResult`. This was elided in the implementation.

## Current Behavior

`ExecutionPlan.post_fetch_operations` is an AST-derived list of
human-readable operation strings (e.g. "GROUP BY 2 column(s)", "LIMIT 10").
This satisfies the BC-2.11.010 postcondition (showing post-materialization
operations without executing the query) without the DataFusion ceremony.

No `datafusion_logical_plan` field exists in `ExplainResult`.

## Required Fix (if user-required)

1. Add `datafusion_logical_plan: Option<String>` to `ExecutionPlan`.
2. Register schema-only `MemTable`s for each detected sensor source.
3. Call `SessionContext::create_logical_plan(&expanded_query)` to obtain the
   `LogicalPlan`.
4. Format via `format!("{}", plan.display_indent())` and store in the field.

Alternatively, translate the AST-derived operations list into a DataFusion
logical plan without running the full session context path.

## Deferred In

S-3.03 adversarial local pass-1 (I-LOCAL-001). The BC postcondition is met
by the AST-derived list; full DataFusion plan visibility is deferred until
it becomes a user-visible requirement.

## References

- BC-2.11.010 Invariants (DataFusion plan elision note, v1.4)
- `crates/prism-query/src/explain.rs` — `post_fetch_operations_from_ast()`
- S-3.03 story v1.6 (task §1 elision note)
