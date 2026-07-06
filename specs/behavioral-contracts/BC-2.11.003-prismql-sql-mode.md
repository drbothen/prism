---
document_type: behavioral-contract
level: L3
version: "1.11"
status: active
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md", ".factory/specs/architecture/decisions/ADR-052-prismql-native-temporal-typing-utf8-to-arrow-timestamp.md"]
input-hash: "c36ec87"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.003: PrismQL SQL Mode Parsing

## Description

SQL mode activates when a query begins with `SELECT` or `FROM` and contains no `|` pipe operators outside string literals. The Chumsky parser produces a `SqlSelect` AST restricted to read-only SELECT statements against the unified `events` table only — mutations and DDL are rejected at parse time. The validated AST is translated to a DataFusion logical plan (or sanitized SQL string) for execution. Alias expansion and security limit validation run before DataFusion receives the plan.

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
- **ADR-052 D2 — Datetime column Arrow type:** `ColumnType::Datetime` sensor columns are registered in the DataFusion execution schema as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `DataType::Utf8`). Temporal predicates such as `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` compare typed Timestamp values against Timestamp columns. The PrismQL parser accepts the 7 canonical offset-less date/datetime forms in the `is_date_like` Acceptance Set (full set defined in BC-2.11.021 §Error Cases and ADR-052 §D4; representative examples: `'2026-06-24'`, `'2026-06-24T12:00:00'`) as `Literal::RawTemporalLiteral` AST nodes (parse succeeds — no E-QUERY-001 at parse time). The plan-time AST walker `check_temporal_literals` resolves each `RawTemporalLiteral` against the real schema column type with a four-way dispatch: (1) vs `Timestamp(Microsecond, UTC)` (Datetime) column → **E-QUERY-041**; (2) vs String/Utf8 column → **COERCE** to `Literal::String(s)` and compare as ordinary string literal (SUCCESS — no error; byte-identical to pre-ADR-052 behavior; see Error Cases); (3) vs Integer/Float/Bool column → **E-QUERY-002 (QueryTypeMismatch)**; (4) in comparison position where LHS is a function or compound expression (non-`Field`) → **E-QUERY-042** (`TemporalLiteralInvalidPosition`, NonColumnLhsComparison — LHS type cannot be resolved at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions); (5) in SELECT projection position (bare literal in SELECT list, no column type context) → **COERCE** to `Literal::String(s)` (SUCCESS — standard SQL `SELECT '2026-06-24'` returns the string constant; OBS-2 preserved); (6) in GROUP BY position → **E-QUERY-042** (`TemporalLiteralInvalidPosition`, GroupBy — grouping by a literal constant is a degenerate no-op); (7) in ORDER BY position → **E-QUERY-042** (`TemporalLiteralInvalidPosition`, OrderBy — ordering by a literal constant is a degenerate no-op). The `chrono::DateTime::parse_from_rfc3339` strictness is preserved at the sensor-boundary datetime parsing path (AC-013) — both the AST-walk plan-time gate and the sensor-boundary parser reject non-RFC-3339 datetime forms; mechanisms differ but the contract is preserved.

## Invariants
- DI-019: All security limits apply to the expanded query
- Only `SELECT` statements are permitted; any query whose first non-whitespace keyword (case-insensitive) appears in the Denied SQL Statement Prefixes set returns `Err(E-QUERY-002 mutation_rejected)`
- Subqueries are not supported in v1; nested `SELECT` in `WHERE` or `FROM` returns a parse error with explanation

## Denied SQL Statement Prefixes

The parser rejects any statement whose first non-whitespace token (case-insensitive prefix match) appears in the following canonical denylist. This denylist is defense-in-depth — the Chumsky grammar already rejects non-SELECT shapes, but the explicit prefix check ensures a clear, auditable error before grammar parsing begins.

Implementers MUST check all keywords in this table in `filter_parser.rs` (or equivalent module). The current implementation covers 7 keywords; the canonical set is ~40.

| Category | Denied Keywords |
|----------|----------------|
| DML mutations | INSERT, UPDATE, DELETE, MERGE, REPLACE, UPSERT, COPY |
| DDL | CREATE, DROP, ALTER, RENAME, TRUNCATE, COMMENT |
| TCL (Transaction Control) | COMMIT, ROLLBACK, SAVEPOINT, RELEASE, BEGIN, START |
| DCL (Data Control) | GRANT, REVOKE, DENY |
| Procedural | EXECUTE, CALL, DO, PERFORM |
| Diagnostic / utility | EXPLAIN, ANALYZE, VACUUM, LOCK, REINDEX, SET, SHOW, USE |
| Vendor extensions | PRAGMA, ATTACH, DETACH |

**Rejection behavior:** Any match returns `Err(E-QUERY-002)` with the message:
> "Only SELECT queries are supported. Prism is a read-only query engine. Denied keyword: `<keyword>`."

**Match semantics:** Case-insensitive, whitespace-normalized, prefix-of-first-token match. A query starting with `  insert ` (leading spaces) must be caught. A query starting with `INSERTED_AT` (identifier containing a denied prefix) must NOT be caught — match is on the full first token, not a substring.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | `FROM` clause references table other than `events` | Error: "PrismQL queries operate on the unified 'events' table. Use sensor/client filters instead of separate tables." |
| `E-QUERY-002` | First non-whitespace keyword matches Denied SQL Statement Prefixes | Error: "Only SELECT queries are supported. Prism is a read-only query engine. Denied keyword: `<keyword>`." |
| `E-QUERY-001` | Subquery detected | Error: "Subqueries are not supported. Use pipe mode for multi-stage operations." |
| `E-QUERY-041` | A `Timestamp(Microsecond, UTC)` datetime column compared against a date-like string literal in SQL mode (bare `Field` LHS). The PrismQL parser emits `Literal::RawTemporalLiteral` for any of the 7 canonical offset-less date/datetime forms in the `is_date_like` Acceptance Set (see BC-2.11.021 §Error Cases / ADR-052 §D4; e.g., `'2026-06-24'`, `'2026-06-24T12:00:00'`) (parse succeeds); the plan-time AST walker `check_temporal_literals` raises E-QUERY-041 upon resolving the column type as `Timestamp(Microsecond, UTC)`. **Does NOT fire for String/Utf8 columns or in SELECT projection position** — `check_temporal_literals` coerces `RawTemporalLiteral` to `Literal::String(s)` for String columns (SUCCESS, no error; `WHERE string_col = '2026-06-24'` is valid — arm (2) of seven-arm dispatch) and for literals in SELECT projection position with no column type context (arm (5) of seven-arm dispatch). GROUP BY and ORDER BY positions now REJECT with E-QUERY-042 (not coerce). Non-`Field` LHS comparisons REJECT with E-QUERY-042 (arm (4)). | `"E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp. Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."` |
| `E-QUERY-042` | A date-like string literal in a structural/positional context where `check_temporal_literals` rejects rather than coerces: (a) GROUP BY position (`GROUP BY '2026-06-24'` — grouping by a literal constant is a degenerate no-op); (b) ORDER BY position (`ORDER BY '2026-06-24'` — ordering by a literal constant is a degenerate no-op); (c) non-`Field` LHS comparison (`WHERE lower(hostname) = '2026-06-24'` — LHS type unresolvable at plan time). | GroupBy: `"E-QUERY-042: GROUP BY expects a column reference, not a literal constant. '<first_50_chars>' is a date-shaped literal — grouping by a constant has no effect and is almost certainly a query mistake. Did you mean to reference a column name, or to add a WHERE filter before grouping?"` OrderBy: `"E-QUERY-042: ORDER BY expects a column reference, not a literal constant. '<first_50_chars>' is a date-shaped literal — ordering by a constant has no effect. Did you mean to reference a column name that contains this value?"` NonColumnLhsComparison: `"E-QUERY-042: A date-like literal compared against a computed expression cannot be type-checked at plan time. Compare against a bare datetime column using RFC-3339 (e.g., '2026-07-03T00:00:00Z'), against a string column using a non-date-shaped value, or wrap the expression in an explicit CAST."` |
| `E-QUERY-001` | Syntax error in SQL | Error with position, context, and SQL syntax reference |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-006 | `SELECT *` with no WHERE clause | Valid query; returns all materialized events (up to `limit` parameter) |
| EC-11-007 | `GROUP BY` with aggregate functions | DataFusion handles aggregation; results include grouped fields + aggregate values |
| EC-11-008 | `ORDER BY` on a field not in `SELECT` | Valid per SQL semantics; DataFusion handles this correctly |
| EC-11-003-001 | `SELECT * FROM events WHERE timestamp > '2026-06-24'` (date-only bare string literal in datetime comparison against a Datetime/Timestamp column) | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); plan-time walker `check_temporal_literals` resolves `timestamp` as `Timestamp(Microsecond, UTC)` → E-QUERY-041. Use full RFC-3339 UTC form `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'` |
| EC-11-003-002 | `SELECT * FROM events WHERE timestamp > '2026-06-24T00:00:00Z'` (valid RFC-3339 UTC string literal in datetime comparison) | Valid; full RFC-3339 form parses directly to `Literal::Timestamp` (not `RawTemporalLiteral`) — `check_temporal_literals` does not intercept it; emitted as `arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` — proceeds to DataFusion without error |
| EC-11-003-003 | `SELECT * FROM events WHERE report_date = '2026-06-24'` where `report_date` is a String/Utf8 column (not a Datetime column) | Valid — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")`; `check_temporal_literals` resolves `report_date` as `DataType::Utf8` (String column) → COERCE to `Literal::String("2026-06-24")`. Processed as a normal string comparison. No E-QUERY-041 emitted. Byte-identical to pre-ADR-052 behavior. |
| EC-11-003-004 | `SELECT '2026-06-24', * FROM events` (temporal literal in bare SELECT projection — no column type context) | Valid — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")`; `check_temporal_literals` recognizes SELECT projection position with no column type context → arm (5) COERCE to `Literal::String("2026-06-24")` (OBS-2 preserved). Processed as constant string in projection. No E-QUERY-041 emitted. |
| EC-11-003-005 | `SELECT severity, count(*) FROM events GROUP BY '2026-06-24'` (temporal literal in GROUP BY position) | `Err(E-QUERY-042)` GroupBy — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); `check_temporal_literals` walker detects GROUP BY position → arm (6) E-QUERY-042 `TemporalLiteralInvalidPosition { position: GroupBy, value_prefix: "2026-06-24" }`. Grouping by a literal constant is a degenerate no-op; query is rejected. |
| EC-11-003-006 | `SELECT * FROM events ORDER BY '2026-06-24'` (temporal literal in ORDER BY position) | `Err(E-QUERY-042)` OrderBy — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); `check_temporal_literals` walker detects ORDER BY position → arm (7) E-QUERY-042 `TemporalLiteralInvalidPosition { position: OrderBy, value_prefix: "2026-06-24" }`. Ordering by a literal constant is a degenerate no-op; query is rejected. |
| EC-11-003-007 | `SELECT * FROM events WHERE lower(hostname) = '2026-06-24'` (temporal literal in comparison where LHS is a function expression, not a bare `Field` column reference) | `Err(E-QUERY-042)` NonColumnLhsComparison — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); `check_temporal_literals` walker detects non-`Field` LHS → arm (4) E-QUERY-042 `TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: "2026-06-24" }`. LHS type unresolvable at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions. Closes prior `-32000 INTERNAL_ERROR` bug for function-expression LHS. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT severity, count(*) FROM events GROUP BY severity` | Aggregate rows grouped by severity | happy-path |
| `SELECT * FROM events WHERE severity = 'critical'` | Filtered event rows | happy-path |
| `INSERT INTO events VALUES (...)` | `Err(E-QUERY-002)` mutation rejected (INSERT is in DML denylist) | error |
| `UPDATE events SET x = 1` | `Err(E-QUERY-002)` mutation rejected (UPDATE is in DML denylist) | error |
| `DELETE FROM events` | `Err(E-QUERY-002)` mutation rejected (DELETE is in DML denylist) | error |
| `MERGE INTO events USING ...` | `Err(E-QUERY-002)` mutation rejected (MERGE is in DML denylist) | error |
| `GRANT SELECT ON events TO user` | `Err(E-QUERY-002)` mutation rejected (GRANT is in DCL denylist) | error |
| `ROLLBACK` | `Err(E-QUERY-002)` mutation rejected (ROLLBACK is in TCL denylist) | error |
| `VACUUM events` | `Err(E-QUERY-002)` mutation rejected (VACUUM is in utility denylist) | error |
| `PRAGMA table_info(events)` | `Err(E-QUERY-002)` mutation rejected (PRAGMA is in vendor denylist) | error |
| `  insert INTO events VALUES (...)` | `Err(E-QUERY-002)` mutation rejected (leading whitespace normalized before match) | error |
| `INSERTED_AT > 0` | `Err(E-QUERY-001)` syntax error (INSERTED_AT is an identifier, not the INSERT keyword) | error |
| `SELECT * FROM alerts` | `Err(E-QUERY-001)` non-events table rejected | error |
| `SELECT * FROM events WHERE (SELECT count(*) FROM events) > 5` | `Err(E-QUERY-001)` subquery rejected | error |
| `SELECT * FROM events WHERE timestamp > '2026-06-24'` | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 (use `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'`) | error |
| `SELECT * FROM events WHERE timestamp > '2026-06-24T00:00:00Z'` | Valid; parses to `Literal::Timestamp` (not `RawTemporalLiteral`); emitted as `arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` — no `RawTemporalLiteral` emitted, no E-QUERY-041 | happy-path |
| `SELECT * FROM events WHERE report_date = '2026-06-24'` (String/Utf8 column `report_date`) | Valid — `check_temporal_literals` coerces `RawTemporalLiteral` → `Literal::String("2026-06-24")`; processed as string comparison (no E-QUERY-041) | edge-case |
| `SELECT '2026-06-24', * FROM events` (temporal literal in bare SELECT projection — no column type context) | Valid — `check_temporal_literals` arm (5) SELECT projection → COERCE to `Literal::String("2026-06-24")` (OBS-2 preserved); no E-QUERY-041 | edge-case |
| `SELECT severity, count(*) FROM events GROUP BY '2026-06-24'` (temporal literal in GROUP BY position) | `Err(E-QUERY-042)` GroupBy — `check_temporal_literals` arm (6): GROUP BY position → E-QUERY-042 `TemporalLiteralInvalidPosition { position: GroupBy, value_prefix: "2026-06-24" }` | error |
| `SELECT * FROM events ORDER BY '2026-06-24'` (temporal literal in ORDER BY position) | `Err(E-QUERY-042)` OrderBy — `check_temporal_literals` arm (7): ORDER BY position → E-QUERY-042 `TemporalLiteralInvalidPosition { position: OrderBy, value_prefix: "2026-06-24" }` | error |
| `SELECT * FROM events WHERE lower(hostname) = '2026-06-24'` (non-`Field` LHS comparison) | `Err(E-QUERY-042)` NonColumnLhsComparison — `check_temporal_literals` arm (4): non-`Field` LHS → E-QUERY-042 `TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: "2026-06-24" }` | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-014 | Query security limits: rejects oversized queries | kani |
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.11 | adr-052-d4-v1.10-seven-arm-dispatch | 2026-07-05 | product-owner | **FIX 3: propagate ADR-052 §D4 v1.10 seven-arm dispatch to SQL-mode BC.** §Postconditions ADR-052 D2 bullet: arm (4) split into four new arms — (4) non-`Field` LHS comparison → E-QUERY-042 NonColumnLhsComparison; (5) SELECT projection → COERCE (OBS-2 preserved); (6) GROUP BY → E-QUERY-042 GroupBy; (7) ORDER BY → E-QUERY-042 OrderBy. §Error Cases E-QUERY-041: "Does NOT fire" note updated — removed GROUP BY / ORDER BY from coerce claim (those now produce E-QUERY-042); added SELECT projection arm (5) and explicit statement that GROUP BY / ORDER BY now REJECT. E-QUERY-042 row ADDED (three sub-cases: GroupBy, OrderBy, NonColumnLhsComparison) with verbatim ADR-052 §D4 v1.10 message templates (POL-24). §Edge Cases: EC-11-003-004 arm reference updated (4) → (5) "SELECT projection" label; EC-11-003-005 ADDED (GROUP BY → E-QUERY-042); EC-11-003-006 ADDED (ORDER BY → E-QUERY-042); EC-11-003-007 ADDED (non-`Field` LHS → E-QUERY-042 NonColumnLhsComparison, closes prior `-32000 INTERNAL_ERROR` bug). §Canonical Test Vectors: projection-coerce vector updated (arm (4) → arm (5)); three E-QUERY-042 vectors ADDED (GROUP BY, ORDER BY, non-`Field` LHS). |
| 1.10 | adr-052-d4-v1.8-four-way-dispatch | 2026-07-05 | product-owner | **HIGH-1: propagate ADR-052 §D4 v1.8 four-way dispatch (non-comparison coercion arm) to SQL-mode BC.** §Postconditions ADR-052 D2 bullet: `three-way dispatch` → `four-way dispatch`; added arm (4): non-comparison position (bare SELECT projection, GROUP BY, ORDER BY, function argument — no column type in scope) → COERCE to `Literal::String(s)` (SUCCESS). §Error Cases E-QUERY-041: expanded "Does NOT fire" note to include non-comparison position (arm (4) of four-way dispatch). §Edge Cases: EC-11-003-004 ADDED (bare SELECT projection coerce → success). §Canonical Test Vectors: non-comparison-position coerce vector ADDED. |
| 1.9 | med-1-e-query-002-propagation | 2026-07-04 | product-owner | **MED-1 E-QUERY-001→E-QUERY-002 correction: numeric/bool temporal dispatch arm.** §Postconditions ADR-052 D2 bullet: corrected three-way dispatch arm (3) from "Integer/Float/Bool column → **E-QUERY-001**" to "**E-QUERY-002 (QueryTypeMismatch)**". Aligns to error-taxonomy.md v2.12 (E-QUERY-002 QueryTypeMismatch) and ADR-052 §D4 v1.5. Datetime→E-QUERY-041 and String/Utf8→COERCE arms UNCHANGED. All other §Error Cases E-QUERY-001 entries (non-events table, subquery, SQL syntax error) are for distinct conditions and are UNCHANGED. |
| 1.8 | ADR-052-d4-is-date-like-canonical-ref | 2026-07-04 | product-owner | Align `is_date_like` enumeration to ADR-052 §D4 v1.4 canonical 7-form set (reference, not re-enumeration). §Postconditions D2 bullet and §Error Cases E-QUERY-041 condition: replaced stale 2-form partial enumeration ("date-only and offset-less datetime") with reference to BC-2.11.021 §Error Cases / ADR-052 §D4 `is_date_like` Acceptance Set; two representative examples retained. Three-way dispatch (Datetime→E-QUERY-041; String/Utf8→coerce; numeric/bool→E-QUERY-001) unchanged. |
| 1.7 | ADR-052-d4-v1.3-bc-amendment | 2026-07-04 | product-owner | **ADR-052 §D4 v1.3 amendment (human-ratified 2026-07-04, Option A — lenient-parse-then-AST-walk + String-column coercion modification).** E-QUERY-041 detection mechanism redesigned from chrono plan-time pre-validator (v1.6) to `Literal::RawTemporalLiteral` AST node + `check_temporal_literals` plan-time walker. **Changes:** §Postconditions ADR-052 D2 bullet: detection description updated — parser emits `RawTemporalLiteral` (parse succeeds), `check_temporal_literals` three-way dispatch: (1) Datetime col → E-QUERY-041; (2) String/Utf8 col → COERCE to `Literal::String(s)` (SUCCESS, byte-identical to pre-ADR-052); (3) Integer/Float/Bool col → E-QUERY-001. §Error Cases E-QUERY-041: condition updated to AST-walk mechanism; added "Does NOT fire for String/Utf8 columns" note. §Edge Cases: EC-11-003-001 mechanism updated; EC-11-003-002 mechanism updated; EC-11-003-003 ADDED (String-column coercion → success). §Canonical Test Vectors: E-QUERY-041 and RFC-3339 vector descriptions updated; String-column coercion vector ADDED. Message format (POL-24), column type assertion (D2), and security denylist UNCHANGED. RISK-5 eliminated by design. |
| 1.6 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 v1.1 correction (remove-uncertainty PASS-1 amendments).** §Postconditions ADR-052 D2 bullet: corrected E-QUERY-041 detection mechanism from "DataFusion cannot implicitly cast" to Prism plan-time literal pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness — arrow-cast 58.2.0 is LENIENT (accepts date-only and offset-less strings via coercion); Prism must validate at parse/plan time before DataFusion sees the query. Added: same chrono strictness applies at sensor-boundary datetime parsing (AC-013 consistency). Postcondition example updated to `arrow_cast(...)` form. Error Cases E-QUERY-041: trigger condition corrected to "Prism plan-time pre-validator rejects" (not "DataFusion cannot cast"); offset-less ISO form example added. Edge cases EC-11-003-001/002: mechanism descriptions updated (chrono pre-validator, not DataFusion implicit cast). Test vectors: descriptions updated to reflect pre-validator language. |
| 1.5 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 amendment (ratified 2026-07-03).** §Postconditions: added ADR-052 D2 datetime column Arrow type assertion — `ColumnType::Datetime` registers as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `Utf8`); typed Timestamp-vs-Timestamp comparison; bare string literal cast failure returns `Err(E-QUERY-041)`. Error Cases: E-QUERY-041 `TemporalLiteralUnparseable` added. Edge Cases: EC-11-003-001 (date-only bare string → E-QUERY-041) and EC-11-003-002 (valid RFC-3339 UTC string → OK) added. Canonical Test Vectors: two E-QUERY-041 / RFC-3339 vectors added. inputs: ADR-052 file added. |
| 1.4 | pr-127-review-remediation | 2026-05-05 | product-owner | Added canonical Denied SQL Statement Prefixes section (~40 keywords across DML/DDL/TCL/DCL/procedural/utility/vendor categories). Updated Invariants to reference E-QUERY-002 for mutation rejection. Updated Error Cases table to use E-QUERY-002 for denylist hits. Expanded Canonical Test Vectors with 9 new denylist vectors including whitespace-normalization and identifier-vs-keyword boundary cases. Addresses Adv OBS-002 [process-gap] from PR-127 review. NOTE: implementer follow-up required — extend filter_parser.rs denylist from 7 to ~40 keywords per new Denied SQL Statement Prefixes table. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
