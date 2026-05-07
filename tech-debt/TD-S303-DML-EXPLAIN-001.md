# TD-S303-DML-EXPLAIN-001: DML AST arm silently handled in explain

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

`explain()` accepts DML queries (INSERT, UPDATE, DELETE) through the parser
because `Ast::Sql(SqlStatement::Dml(_))` falls through the `_ => {}` arm
in `extract_sources_from_ast` and the `_ => vec![]` arm in `predicates_from_ast`.

The result is a valid `ExplainResult` with empty `sensors_to_query` and no
post-fetch operations — identical to a query with no sensor sources. The
`estimated_cost.summary` will be the "No sensor sources identified" message.

This is misleading: a DML statement should either be rejected at the explain
boundary or produce a warning indicating that DML is a write operation not
supported by the explain tool.

## Required Fix

In the `Ast::Sql(SqlStatement::Dml(_))` match arm of `extract_sources_from_ast`
(or in `explain()` itself after the parse step), emit a warning entry into
`estimated_cost.warnings`:

```rust
if let Ast::Sql(SqlStatement::Dml(_)) = &ast {
    warnings.push(
        "DML statement detected (INSERT/UPDATE/DELETE). explain_query reports \
         the plan for read queries only; DML is not executed and produces no \
         sensor plan. Use the write tool for DML operations.".to_string()
    );
}
```

## Deferred In

S-3.03 adversarial local pass-1 (O-LOCAL-003). Low priority — DML via
explain is an unlikely user error, and the empty result is safe (no side
effects).

## References

- BC-2.11.010 — `explain_query` MCP Tool
- `crates/prism-query/src/explain.rs` — `extract_sources_from_ast()` Dml arm
- `crates/prism-query/src/ast.rs` — `SqlStatement::Dml`
