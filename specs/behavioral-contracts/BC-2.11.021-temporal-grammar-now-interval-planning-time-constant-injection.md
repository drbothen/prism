---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: 2026-07-04
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
  - ".factory/specs/architecture/decisions/ADR-052-prismql-native-temporal-typing-utf8-to-arrow-timestamp.md"
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
- **Planning-time constant injection:** At planning time, `Expr::Now` (and any `TimestampArithmetic` whose `base` is `Expr::Now`) is evaluated using the query's execution timestamp (`DateTime<Utc>`) and replaced with a `Literal::Timestamp` constant before the logical plan is handed to DataFusion. DataFusion sees a concrete `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` comparison against a `Timestamp(Microsecond, UTC)` column — a fully explicit typed comparison with no implicit coercion. (ADR-052 D3/D7: `Literal::Timestamp` emitter uses the explicit `arrow_cast(...)` form, NOT `TIMESTAMP '...'` — DataFusion 53.1.0 lowers `TIMESTAMP '...'` to naive `Timestamp(Nanosecond, None)`, making implicit coercion non-deterministic across DF versions; `arrow_cast` produces exactly `Timestamp(Microsecond, Some("UTC"))` matching the column type. The column is `DataType::Timestamp(Microsecond, Some("UTC"))` per ADR-052 D2.)
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
| `E-QUERY-041` | The PrismQL parser accepts the following offset-less date/datetime string literals as `Literal::RawTemporalLiteral` AST nodes (**parse succeeds for all 7 forms** — no E-QUERY-001 at parse time): date-only (`'2026-06-24'`); T-separator full seconds (`'2026-06-24T12:00:00'`); T-separator fractional seconds (`'2026-06-24T12:00:00.123'`); T-separator no seconds (`'2026-06-24T12:00'`); space-separator full seconds (`'2026-06-24 12:00:00'`); space-separator fractional seconds (`'2026-06-24 12:00:00.500'`); space-separator no seconds (`'2026-06-24 12:00'`). **Over-matched forms (ACCEPTED BENIGN per ADR-052 §D4):** unpadded digits (`'2026-6-24'`) and big/signed years (`'12345-06-24'`, `'-0044-03-15'`) also match `is_date_like` — against a Datetime/Timestamp col → E-QUERY-041 (the "use RFC-3339" message is accurate and apt); against String/Utf8 col → COERCE (correct); against numeric/bool col → E-QUERY-002 (QueryTypeMismatch) (correct). The plan-time validator `check_temporal_literals` walks the resolved AST with a seven-arm dispatch (ADR-052 §D4 v1.10): (1) for `RawTemporalLiteral` nodes in comparison position against a `Timestamp(Microsecond, UTC)` column (bare `Field` LHS), E-QUERY-041 is raised; (2) for `RawTemporalLiteral` nodes in comparison position against a String/Utf8 column (bare `Field` LHS), the node is rewritten in-place to `Literal::String(s)` and processing continues without error (byte-identical to pre-ADR-052 behavior); (3) for `RawTemporalLiteral` nodes in comparison position against Integer/Float/Bool columns (bare `Field` LHS), E-QUERY-002 (QueryTypeMismatch) is raised; (4) for `RawTemporalLiteral` nodes in comparison position where the LHS is a function or compound expression (non-`Field`), E-QUERY-042 (`TemporalLiteralInvalidPosition`, NonColumnLhsComparison) is raised — LHS type cannot be resolved at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions; (5) for `RawTemporalLiteral` nodes in SELECT projection position (bare literal in SELECT list, no column type context), the node is COERCED to `Literal::String(s)` — SUCCESS, no error emitted (standard SQL `SELECT '2026-06-24'` returns the string constant; OBS-2 human-ratified, ADR-052 §D4 v1.8); (6) for `RawTemporalLiteral` nodes in GROUP BY position, E-QUERY-042 (`TemporalLiteralInvalidPosition`, GroupBy) is raised — grouping by a literal constant is a degenerate no-op, almost always an analyst mistake; (7) for `RawTemporalLiteral` nodes in ORDER BY position, E-QUERY-042 (`TemporalLiteralInvalidPosition`, OrderBy) is raised — ordering by a literal constant is a degenerate no-op, almost always an analyst mistake. | `"E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp. Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."` |

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
| EC-11-021-009 | `WHERE timestamp > '2026-06-24'` (date-only bare string literal, no time component or UTC offset, compared against a `Timestamp(Microsecond, UTC)` column) — form 1 of 7 | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds via `%Y-%m-%d`); plan-time AST walker `check_temporal_literals` resolves column `timestamp` as `Timestamp(Microsecond, UTC)` (Datetime type) → raises E-QUERY-041. Use `NOW() - INTERVAL 'Nh'` or a full RFC-3339 UTC literal (e.g., `'2026-06-24T00:00:00Z'`) instead |
| EC-11-021-010 | `WHERE timestamp > '2026-06-24T12:00'` (T-separator, no-seconds — form 4 of `is_date_like`, compared against a `Timestamp(Microsecond, UTC)` column) | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24T12:00")` (parse succeeds via `%Y-%m-%dT%H:%M`); `check_temporal_literals` resolves column as `Timestamp(Microsecond, UTC)` → E-QUERY-041 |
| EC-11-021-011 | `WHERE timestamp > '2026-06-24T12:00:00.123'` (T-separator, fractional seconds — form 3 of `is_date_like`, compared against a `Timestamp(Microsecond, UTC)` column) | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24T12:00:00.123")` (parse succeeds via `%Y-%m-%dT%H:%M:%S%.f`); `check_temporal_literals` resolves column as `Timestamp(Microsecond, UTC)` → E-QUERY-041 |
| EC-11-021-012 | `WHERE timestamp > '2026-06-24 12:00:00'` (space-separator, full seconds — form 5 of `is_date_like`, compared against a `Timestamp(Microsecond, UTC)` column) | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24 12:00:00")` (parse succeeds via `%Y-%m-%d %H:%M:%S`); `check_temporal_literals` resolves column as `Timestamp(Microsecond, UTC)` → E-QUERY-041 |
| EC-11-021-013 | `WHERE string_col = '2026-06-24 12:00:00'` (space-separator form compared against a String/Utf8 column, not a Datetime column) | COERCE: `check_temporal_literals` rewrites `RawTemporalLiteral("2026-06-24 12:00:00")` in-place to `Literal::String("2026-06-24 12:00:00")`; query executes as ordinary string literal comparison — SUCCESS, no error emitted (byte-identical to pre-ADR-052 behavior; RISK-5 eliminated by design) |
| EC-11-021-014 | `WHERE timestamp > '2026-6-24'` (unpadded month/day — over-matched form, ACCEPTED BENIGN per ADR-052 §D4, compared against a `Timestamp(Microsecond, UTC)` column) | `Err(E-QUERY-041)`: `is_date_like` matches via `%Y-%m-%d` (chrono `%m`/`%d` accept single digits); `check_temporal_literals` resolves Datetime column → E-QUERY-041. "Use RFC-3339" message is accurate (unpadded forms are also non-RFC-3339). Unpadded-year/big-year/negative-year forms behave identically. Over-match is ACCEPTED BENIGN — no regex guard or year-width constraint is applied |
| EC-11-021-015 | `SELECT '2026-06-24' FROM t` (date-like literal in non-comparison SELECT projection position, no column type context — OBS-2 human-ratified behavior change, ADR-052 §D4 v1.8) | COERCE: `check_temporal_literals` rewrites `RawTemporalLiteral("2026-06-24")` in-place to `Literal::String("2026-06-24")` — SUCCESS, no error emitted. Standard SQL `SELECT '2026-06-24'` returns the string constant `2026-06-24`. Note: GROUP BY and ORDER BY literal positions now REJECT with E-QUERY-042 (see EC-11-021-016 and EC-11-021-017) — arm (5) SELECT projection coerce does NOT apply to GROUP BY or ORDER BY positions (ADR-052 §D4 v1.10). |
| EC-11-021-016 | `SELECT hostname FROM t GROUP BY '2026-06-24'` (date-like literal in GROUP BY position — ADR-052 §D4 v1.10 REJECT) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (GroupBy): `check_temporal_literals` encounters `RawTemporalLiteral("2026-06-24")` in GROUP BY position → arm (6) REJECT. Message: `"E-QUERY-042: GROUP BY expects a column reference, not a literal constant. '2026-06-24' is a date-shaped literal — grouping by a constant has no effect and is almost certainly a query mistake. Did you mean to reference a column name, or to add a WHERE filter before grouping?"` |
| EC-11-021-017 | `SELECT hostname FROM t ORDER BY '2026-06-24'` (date-like literal in ORDER BY position — ADR-052 §D4 v1.10 REJECT) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (OrderBy): `check_temporal_literals` encounters `RawTemporalLiteral("2026-06-24")` in ORDER BY position → arm (7) REJECT. Message: `"E-QUERY-042: ORDER BY expects a column reference, not a literal constant. '2026-06-24' is a date-shaped literal — ordering by a constant has no effect. Did you mean to reference a column name that contains this value?"` |
| EC-11-021-018 | `WHERE lower(hostname) = '2026-06-24'` (non-column-LHS comparison: function call LHS, date-like RHS — ADR-052 §D4 v1.10 REJECT, closes prior `-32000 INTERNAL_ERROR` bug) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (NonColumnLhsComparison): `check_temporal_literals` encounters `RawTemporalLiteral("2026-06-24")` as comparison RHS where LHS is `lower(hostname)` (function call, not bare `Field`) → arm (4) REJECT. Message: `"E-QUERY-042: A date-like literal compared against a computed expression cannot be type-checked at plan time. Compare against a bare datetime column using RFC-3339 (e.g., '2026-07-03T00:00:00Z'), against a string column using a non-date-shaped value, or wrap the expression in an explicit CAST."` Prior behavior: `QueryPlanFailed → -32000 INTERNAL_ERROR` (opaque, not analyst-actionable). |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT * FROM crowdstrike_detections WHERE timestamp > NOW() - INTERVAL '24h'` | Parse OK as `Ast::Sql`; plan-time injects current UTC - 24h as `Literal::Timestamp`; executes correctly | happy-path |
| `FROM crowdstrike_detections \| where timestamp > NOW() - INTERVAL '1h' \| limit 25` | Parse OK as `Ast::Pipe`; plan-time injection; executes | happy-path |
| `timestamp > NOW() - 7d` (Filter mode) | Parse OK as `Ast::Filter`; plan-time injection | happy-path |
| `SELECT * FROM t WHERE timestamp > NOW() + INTERVAL '1h'` | `Err(E-QUERY-001)` subtraction-only | error |
| `SELECT * FROM t WHERE timestamp > NOW(utc)` | `Err(E-QUERY-001)` no args to NOW() | error |
| `SELECT * FROM crowdstrike_detections WHERE timestamp > '2026-06-24'` | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 (use `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'`) | error |
| `SELECT * FROM crowdstrike_detections WHERE timestamp > '2026-06-24T12:00'` (T-sep, no-seconds, form 4) | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 | error |
| `SELECT * FROM crowdstrike_detections WHERE timestamp > '2026-06-24T12:00:00.123'` (T-sep fractional, form 3) | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 | error |
| `SELECT * FROM crowdstrike_detections WHERE timestamp > '2026-06-24 12:00:00'` (space-sep, form 5) | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 | error |
| `SELECT * FROM t WHERE string_col = '2026-06-24 12:00:00'` (space-sep form vs String/Utf8 column) | Parse OK; `check_temporal_literals` COERCEs `RawTemporalLiteral` → `Literal::String(...)`; executes as ordinary string comparison — SUCCESS, no E-QUERY error emitted | happy-path |
| `SELECT '2026-06-24' FROM t` (date-like literal in non-comparison projection position, no column type context) | Parse OK; `check_temporal_literals` COERCEs `RawTemporalLiteral` → `Literal::String("2026-06-24")` (no column type context → string constant); query executes — returns string constant `2026-06-24` — SUCCESS (OBS-2 human-ratified, ADR-052 §D4 v1.8) | happy-path |
| `SELECT hostname FROM t GROUP BY '2026-06-24'` (GROUP BY bare date literal) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (GroupBy) — arm (6): grouping by a literal constant is a degenerate no-op; analyst-facing INVALID_PARAMS | error |
| `SELECT hostname FROM t ORDER BY '2026-06-24'` (ORDER BY bare date literal) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (OrderBy) — arm (7): ordering by a literal constant is a degenerate no-op; analyst-facing INVALID_PARAMS | error |
| `SELECT * FROM t WHERE lower(hostname) = '2026-06-24'` (non-column-LHS function call, date-like RHS) | `Err(E-QUERY-042)` `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) — arm (4): LHS is a function call (`lower(hostname)`), not a bare `Field`; type cannot be resolved at plan time; MCP `-32602 INVALID_PARAMS` (was `-32000 INTERNAL_ERROR` prior to v1.10) | error |

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
| ADR traces | ADR-044 v1.0, ADR-033 (push-down benefits automatically), ADR-052 §D4 v1.10 (§D3 `arrow_cast` explicit emission — `TIMESTAMP '...'` produces `Nanosecond/None` in DF 53.1.0; §D7 ADR-044 §D4 supersession; §D4 v1.4 E-QUERY-041 detection: lenient-parse-then-AST-walk with 7-form `is_date_like` acceptance set — date-only, T-sep full/fractional/no-seconds, space-sep full/fractional/no-seconds; over-match forms unpadded/big/signed years ACCEPTED BENIGN; plan-time walker `check_temporal_literals` performs seven-arm dispatch (v1.10): (1) Datetime col (bare Field LHS) → E-QUERY-041; (2) String/Utf8 col (bare Field LHS) → COERCE to `Literal::String(s)` (SUCCESS, no error); (3) Integer/Float/Bool col (bare Field LHS) → E-QUERY-002 (QueryTypeMismatch); (4) non-Field LHS (function/expression) comparison → E-QUERY-042 (NonColumnLhsComparison); (5) SELECT projection → COERCE to `Literal::String(s)` (SUCCESS, OBS-2); (6) GROUP BY position → E-QUERY-042 (GroupBy); (7) ORDER BY position → E-QUERY-042 (OrderBy)) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | ADR-052-d4-v1.10-e-query-042-propagation | 2026-07-05 | product-owner | **ADR-052 §D4 v1.10: E-QUERY-042 propagated + four-way→seven-arm dispatch update.** §Error Cases E-QUERY-041 dispatch: updated four-way → seven-arm per ADR-052 §D4 v1.10. Former arm (4) "non-comparison position (projection/GROUP BY/ORDER BY) → COERCE" refined to: (4) non-`Field` LHS comparison → E-QUERY-042 (NonColumnLhsComparison); (5) SELECT projection → COERCE (SUCCESS, OBS-2 preserved — UNCHANGED); (6) GROUP BY → E-QUERY-042 (GroupBy); (7) ORDER BY → E-QUERY-042 (OrderBy). Arms (1)/(2)/(3) — Datetime→E-QUERY-041, String/Utf8→COERCE, Integer/Float/Bool→E-QUERY-002 — UNCHANGED. §Edge Cases: EC-11-021-015 updated to remove incorrect GROUP BY/ORDER BY coerce claim; GROUP BY/ORDER BY now explicitly noted as REJECT (E-QUERY-042). EC-11-021-016 ADDED (GROUP BY date literal → E-QUERY-042 GroupBy). EC-11-021-017 ADDED (ORDER BY date literal → E-QUERY-042 OrderBy). EC-11-021-018 ADDED (non-column-LHS comparison → E-QUERY-042 NonColumnLhsComparison; closes prior `-32000 INTERNAL_ERROR` bug for `WHERE lower(hostname) = '2026-06-24'`). §Canonical Test Vectors: three E-QUERY-042 vectors added (GROUP BY, ORDER BY, non-column-LHS). §Traceability ADR traces: updated to v1.10 seven-arm dispatch enumeration. |
| 1.6 | obs-2-non-comparison-coerce | 2026-07-04 | product-owner | **OBS-2 human-ratified: non-comparison-position RawTemporalLiteral coerces to Literal::String (success), align ADR-052 §D4 v1.8.** §Error Cases E-QUERY-041 dispatch: updated "three-way" → "four-way"; former arm (3) "Integer/Float/Bool columns or in non-comparison positions without a resolvable String context → E-QUERY-002" split into: (3) Integer/Float/Bool comparison position → E-QUERY-002 (QueryTypeMismatch) UNCHANGED; (4) non-comparison position (projection `SELECT '2026-06-24' FROM t`, GROUP BY constant, ORDER BY constant, or any literal position with no column type context) → COERCE to `Literal::String(s)`, SUCCESS (was E-QUERY-002 in v1.5, over-strict). §Edge Cases: added EC-11-021-015 (`SELECT '2026-06-24' FROM t` → SUCCESS string constant). §Canonical Test Vectors: added non-comparison projection vector. §Traceability ADR traces: updated to four-way dispatch with v1.8 non-comparison coercion arm. The three comparison arms (Datetime→E-QUERY-041; String/Utf8→COERCE; Integer/Float/Bool comparison→E-QUERY-002) are UNCHANGED. |
| 1.5 | med-1-e-query-002-propagation | 2026-07-04 | product-owner | **MED-1 E-QUERY-001→E-QUERY-002 correction: numeric/bool + non-comparison temporal dispatch arm.** §Error Cases E-QUERY-041 row: corrected two occurrences — "against numeric/bool col → E-QUERY-001 (correct)" → "E-QUERY-002 (QueryTypeMismatch) (correct)" and "(3) for `RawTemporalLiteral` nodes against Integer/Float/Bool columns ... E-QUERY-001 is raised" → "E-QUERY-002 (QueryTypeMismatch) is raised". §Traceability ADR traces: "Integer/Float/Bool col → E-QUERY-001" → "Integer/Float/Bool col → E-QUERY-002 (QueryTypeMismatch)". Aligns to error-taxonomy.md v2.12 (E-QUERY-002 QueryTypeMismatch with column/table/actual_type/operator fields) and ADR-052 §D4 v1.5. The Datetime→E-QUERY-041, String/Utf8→COERCE, and all E-QUERY-001 entries for NOW() syntax errors and unsupported `+` operator are UNCHANGED. |
| 1.4 | ADR-052-d4-v1.4-is-date-like-7-forms | 2026-07-04 | product-owner | **ADR-052 §D4 v1.4 is_date_like 7-form acceptance set (pre-TDD refinement).** §Error Cases E-QUERY-041 condition: expanded from generic "date-like string literal" to enumerate all 7 accepted `is_date_like` forms verbatim (date-only; T-sep full seconds; T-sep fractional; T-sep no-seconds; space-sep full seconds; space-sep fractional; space-sep no-seconds) using architect's authoritative block-quote text from ADR-052 §Recommended BC Amendments. Over-matched forms (unpadded digits via `%m`/`%d`, big/signed years via `%Y`) documented as ACCEPTED BENIGN inline — no regex guard or year-width constraint applied. Three-way dispatch language updated to architect's exact block-quote wording. §Edge Cases: added EC-11-021-010 (T-sep no-seconds vs Datetime → E-QUERY-041), EC-11-021-011 (T-sep fractional vs Datetime → E-QUERY-041), EC-11-021-012 (space-sep vs Datetime → E-QUERY-041), EC-11-021-013 (space-sep vs String/Utf8 → COERCE → SUCCESS), EC-11-021-014 (unpadded over-match → E-QUERY-041 ACCEPTED BENIGN); EC-11-021-009 description updated to note "form 1 of 7" and cite format string. §Canonical Test Vectors: added 4 new vectors for v1.4 forms (T-sep no-seconds → E-QUERY-041; T-sep fractional → E-QUERY-041; space-sep → E-QUERY-041; space-sep vs String → SUCCESS COERCE). §Traceability ADR traces: ADR-052 §D4 v1.3 reference updated to v1.4 with 7-form enumeration summary. |
| 1.3 | ADR-052-d4-v1.3-bc-amendment | 2026-07-04 | product-owner | **ADR-052 §D4 v1.3 amendment (human-ratified 2026-07-04, Option A — lenient-parse-then-AST-walk + String-column coercion modification).** E-QUERY-041 detection mechanism redesigned from chrono plan-time pre-validator (v1.2) to `Literal::RawTemporalLiteral` AST node + `check_temporal_literals` plan-time walker. **Changes:** §Error Cases E-QUERY-041 row: condition rewritten to describe three-way column-type dispatch in `check_temporal_literals` — (1) `RawTemporalLiteral` vs Datetime/Timestamp col → E-QUERY-041; (2) vs String/Utf8 col → COERCE to `Literal::String(s)` (SUCCESS, no error, byte-identical to pre-ADR-052); (3) vs Integer/Float/Bool col → E-QUERY-001. §Edge Cases EC-11-021-009: mechanism updated from "chrono pre-validator rejects" to "`check_temporal_literals` resolves Datetime column → E-QUERY-041". §Canonical Test Vectors: E-QUERY-041 test vector description updated. §Traceability ADR traces: updated to ADR-052 §D4 v1.3 with three-way dispatch description. Message format, invariants, postconditions, and ADR-033/ADR-044 traces UNCHANGED (D1–D3, D5–D8 unaffected). RISK-5 eliminated by design (String-column coercion arm). |
| 1.2 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 v1.1 correction (remove-uncertainty PASS-1 amendments).** §Postconditions Planning-time constant injection bullet: corrected from `TIMESTAMP '...'` form to explicit `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` form per ADR-052 v1.1 D3 — DataFusion 53.1.0 lowers `TIMESTAMP '...'` to `Timestamp(Nanosecond, None)` (not Microsecond/UTC); `arrow_cast` is the deterministic form. Error Cases E-QUERY-041: corrected mechanism from "DataFusion cannot implicitly cast" to Prism plan-time pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness — arrow-cast 58.2.0 is LENIENT (accepts date-only and offset-less strings); Prism must gate at plan time before DataFusion sees the query. Edge Cases EC-11-021-009: description updated to reference Prism pre-validator, not DataFusion cast. Test vector description updated. Traceability ADR traces: ADR-052 v1.0 → v1.1. |
| 1.1 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 amendment (ratified 2026-07-03).** §Postconditions Planning-time constant injection bullet updated: bare string literal `'...'` → typed `TIMESTAMP '...'` emission per ADR-052 D3; comparison now Timestamp-vs-Timestamp against `Timestamp(Microsecond, UTC)` column per ADR-052 D7/D3. Error Cases: E-QUERY-041 `TemporalLiteralUnparseable` added — fires when bare string literal in datetime comparison cannot be cast to `Timestamp(Microsecond, UTC)`. Edge Cases: EC-11-021-009 added. Test Vectors: E-QUERY-041 vector added. Traceability ADR traces: ADR-052 v1.0 added. inputs: ADR-052 file added. Invariants UNCHANGED (invariants concern `Literal::Timestamp.iso8601` RFC-3339 internal format, which is unchanged by ADR-052). |
| 1.0 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **POL-14 BC auto-promotion: draft → active.** Anchor story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 squash-merged via PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict) cascade on frozen HEAD 356e0573). `status: draft → active`. No behavioral change; frontmatter status field only. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-044 v1.0. Closes GRAMMAR-011. |
