---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.003: PrismQL SQL Mode Parsing

## Preconditions
- A query string has been classified as SQL mode by the mode auto-detection precedence (see BC-2.11.002 for full precedence rules):
  - Query starts with `SELECT` (case-insensitive) and does not contain `|` outside string literals, OR
  - Query starts with `FROM` (case-insensitive) and does not contain `|` outside string literals
- If the query contains `|` outside string literals, pipe mode takes precedence regardless of leading keywords
- The query string has passed the 64KB length check

## Postconditions
- The Chumsky parser produces a `SqlSelect` AST with:
  - `projections`: list of field names or `*`, aggregate functions (`count`, `sum`, `avg`, `min`, `max`)
  - `from`: always `events` (the unified OCSF table); other table names are rejected
  - `where_clause`: optional `FilterExpr` (same grammar as filter mode)
  - `group_by`: optional list of field names
  - `order_by`: optional list of `(field, direction)` pairs
  - `limit`: optional integer
- The parsed AST undergoes security validation (alias expansion, field resolution, nesting depth)
- The validated AST is reconstructed as a sanitized SQL string and passed to DataFusion's built-in SQL parser for execution
- Alternatively, the AST may be directly translated to DataFusion `LogicalPlan` via the `DataFrame` API
- Alias references within the `WHERE` clause are expanded before parsing
- The `FROM events` table name is mandatory and refers to the unified OCSF materialized table

## Invariants
- DI-019: All security limits apply to the expanded query
- Only `SELECT` statements are permitted; `INSERT`, `UPDATE`, `DELETE`, `DROP`, and DDL are rejected at parse time with a clear error
- Subqueries are not supported in v1; nested `SELECT` in `WHERE` or `FROM` returns a parse error with explanation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | `FROM` clause references table other than `events` | Error: "PrismQL queries operate on the unified 'events' table. Use sensor/client filters instead of separate tables." |
| `E-QUERY-001` | SQL contains mutation statement (INSERT, UPDATE, etc.) | Error: "Only SELECT queries are supported. Prism is a read-only query engine." |
| `E-QUERY-001` | Subquery detected | Error: "Subqueries are not supported. Use pipe mode for multi-stage operations." |
| `E-QUERY-001` | Syntax error in SQL | Error with position, context, and SQL syntax reference |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-006 | `SELECT *` with no WHERE clause | Valid query; returns all materialized events (up to `limit` parameter) |
| EC-11-007 | `GROUP BY` with aggregate functions | DataFusion handles aggregation; results include grouped fields + aggregate values |
| EC-11-008 | `ORDER BY` on a field not in `SELECT` | Valid per SQL semantics; DataFusion handles this correctly |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Priority | P0 |
