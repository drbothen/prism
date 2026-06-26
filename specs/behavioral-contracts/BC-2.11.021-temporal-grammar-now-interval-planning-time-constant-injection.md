---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md"
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
---

# BC-2.11.021: Temporal Grammar — `NOW()` and `INTERVAL` Planning-Time Constant Injection

## Description

`NOW()` is a zero-argument temporal function recognized in the PrismQL expression grammar (shared by SQL mode, Pipe mode, and Filter mode). `INTERVAL 'Nh'` (or bare duration `INTERVAL 24h`) wraps an existing duration literal as an explicit interval expression. `NOW() - INTERVAL 'Nh'` / `NOW() - 24h` produces a `TimestampArithmetic` expression that is evaluated at query-plan time by injecting the current UTC timestamp as a `Literal::Timestamp` constant. The resulting constant is substituted before DataFusion execution, making `WHERE timestamp > NOW() - INTERVAL '1h'` a valid timestamp comparison.

## Preconditions

- A query string (in any mode — SQL, Pipe, or Filter) contains a temporal expression using `NOW()` and optionally `INTERVAL`
- The query string has passed the 64KB length check (BC-2.11.006)
- The executor's planning step has access to the current UTC timestamp (injected as query context)

## Postconditions

- **`Expr::Now` parsing:** The token sequence `NOW` `(` `)` (case-insensitive) in any expression position produces `Expr::Now`. Any argument to `NOW()` (e.g., `NOW(1)`) produces `Err(E-QUERY-001)`: "NOW() takes no arguments".
- **`Expr::Interval` parsing:** The token sequence `INTERVAL` followed by a quoted duration string (`'24h'`, `'7d'`, `'30s'`) OR a bare duration literal (`24h`, `7d`, `30s`) produces `Expr::Interval(Duration)`. The inner `Duration` type reuses the existing `ast::Literal::Duration` representation.
- **`Expr::TimestampArithmetic` parsing:** The expression `NOW() - <duration_expr>` where `<duration_expr>` is either `Expr::Interval` or a bare `Duration` literal produces `Expr::TimestampArithmetic { base: Box<Expr::Now>, op: Sub, offset: Duration }`. `NOW() + <duration>` is NOT supported in v1 and produces `Err(E-QUERY-001)`: "timestamp arithmetic only supports subtraction: use `NOW() - INTERVAL 'Nh'`".
- **Planning-time constant injection:** At planning time, `Expr::Now` (and any `TimestampArithmetic` whose `base` is `Expr::Now`) is evaluated using the query's execution timestamp (`DateTime<Utc>`) and replaced with a `Literal::Timestamp` constant before the logical plan is handed to DataFusion. DataFusion sees a concrete `WHERE timestamp > '2026-06-24T00:00:00Z'` comparison.
- **ADR-033 push-down benefit:** Once lowered to a `Literal::Timestamp`, the timestamp predicate is automatically recognized by ADR-033's T1 push-down heuristic in `pushdown.rs` and passed as `start_time`/`end_time` range hints to sensor adapters — no changes to `pushdown.rs` required.
- **`build_example_query` validity:** After this BC is implemented, `prism_describe.rs`'s `build_example_query` function generates `WHERE timestamp > NOW() - INTERVAL '1h'` and this query parses and plans successfully. No change to `build_example_query` is needed.

## Invariants

- `NOW()` is always evaluated at **plan time**, not execution time (no per-row evaluation, no DataFusion UDF registration)
- `Expr::Now` produces a UTC timestamp; the resulting `Literal::Timestamp` is RFC-3339 formatted with UTC offset
- Duration arithmetic is subtraction-only in v1; `NOW() + duration` is a plan-time parse error
- The expression grammar extension is shared across all three modes (Filter, SQL, Pipe) — one expression parser, three entry points
- `Expr::Interval` is syntactic sugar that wraps the already-implemented `Literal::Duration` — no new runtime type is introduced

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | `NOW()` with one or more arguments (e.g., `NOW(1)`) | `"E-QUERY-001: NOW() takes no arguments. Use NOW() with no parenthesized arguments."` |
| `E-QUERY-001` | `NOW() + INTERVAL '1h'` (addition, not subtraction) | `"E-QUERY-001: timestamp arithmetic only supports subtraction: use NOW() - INTERVAL 'Nh'. Future addition support is planned."` |
| `E-QUERY-001` | `INTERVAL` without a valid duration literal following it (e.g., `INTERVAL 'bogus'`) | `"E-QUERY-001: INTERVAL requires a duration literal (e.g., INTERVAL '24h', INTERVAL '7d', INTERVAL '30s'). Found: '{found}'."` |
| `E-QUERY-001` | `NOW()` in a non-expression context (e.g., as a table name) | Standard syntax error at position |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-021-001 | `WHERE timestamp > NOW() - INTERVAL '24h'` in SQL mode | Valid; lowers to timestamp constant comparison at plan time |
| EC-11-021-002 | `WHERE timestamp > NOW() - 24h` (bare duration form) | Valid (same as EC-11-021-001; bare duration form accepted) |
| EC-11-021-003 | `\| where timestamp > NOW() - INTERVAL '1h'` in Pipe mode | Valid; same planning-time injection applies |
| EC-11-021-004 | `timestamp > NOW() - INTERVAL '1h'` in Filter mode | Valid; Filter mode uses the same expression parser |
| EC-11-021-005 | `NOW() - INTERVAL '0s'` | Valid; evaluates to the current timestamp (effectively `NOW()`) |
| EC-11-021-006 | `NOW() - INTERVAL '52w'` (52 weeks) | Valid; large durations are accepted as long as they parse as `Duration` |
| EC-11-021-007 | `NOW() + INTERVAL '24h'` | `Err(E-QUERY-001)`: subtraction-only in v1 |
| EC-11-021-008 | `NOW(tz='UTC')` | `Err(E-QUERY-001)`: no arguments accepted |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT * FROM crowdstrike_detections WHERE timestamp > NOW() - INTERVAL '24h'` | Parse OK as `Ast::Sql`; plan-time injects current UTC - 24h as `Literal::Timestamp`; executes correctly | happy-path |
| `FROM crowdstrike_detections \| where timestamp > NOW() - INTERVAL '1h' \| limit 25` | Parse OK as `Ast::Pipe`; plan-time injection; executes | happy-path |
| `timestamp > NOW() - 7d` (Filter mode) | Parse OK as `Ast::Filter`; plan-time injection | happy-path |
| `SELECT * FROM t WHERE timestamp > NOW() + INTERVAL '1h'` | `Err(E-QUERY-001)` subtraction-only | error |
| `SELECT * FROM t WHERE timestamp > NOW(utc)` | `Err(E-QUERY-001)` no args to NOW() | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Related BCs

- **BC-2.11.003** (related — SQL mode expression grammar extended): `NOW()` is usable in SQL mode's WHERE clause
- **BC-2.11.004** (related — Pipe mode expression grammar extended): `NOW()` is usable in Pipe mode's `| where` stage
- **BC-2.11.002** (related — Filter mode expression grammar extended): `NOW()` is usable in bare Filter mode predicates
- **BC-2.11.022** (related — reference generation): the `prismql://reference` Datetime Arithmetic section documents `NOW()` and `INTERVAL`; BC-2.11.022's CI gate validates that section's examples parse through this grammar

## Architecture Anchors

- `crates/prism-query/src/ast.rs` — `Expr` enum (add `Now`, `Interval(Duration)`, `TimestampArithmetic` variants)
- `crates/prism-query/src/filter_parser.rs` or equivalent expression parser — `build_expr_parser` (shared by SQL, Pipe, Filter modes)
- `crates/prism-query/src/lib.rs` — `inject_now` / `inject_now_sql_query` / `inject_now_pipe_stage` / `inject_now_predicate` / `inject_now_expr` (planning-time `now` constant injection; no `plan_query` function exists)
- ADR-044: Temporal Grammar — `NOW()` and `INTERVAL`/Relative-Duration Literals

## Story Anchor

TBD

## VP Anchors

VP-021 (fuzz)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC governs the expression grammar of the PrismQL parser within the ephemeral query engine. CAP-015 defines the three query modes (filter, SQL, pipe) and their execution semantics; temporal operators (`NOW()`, `INTERVAL`) are expression-level extensions to those modes. |
| L2 Invariants | DI-019 |
| Priority | P0 |
| Closes findings | GRAMMAR-011 (NOW() + INTERVAL grammar implementation) |
| ADR traces | ADR-044 v1.0, ADR-033 (push-down benefits automatically) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-044 v1.0. Closes GRAMMAR-011. |
