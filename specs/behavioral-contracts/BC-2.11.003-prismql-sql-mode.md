---
document_type: behavioral-contract
level: L3
version: "1.6"
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
- **ADR-052 D2 — Datetime column Arrow type:** `ColumnType::Datetime` sensor columns are registered in the DataFusion execution schema as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `DataType::Utf8`). Temporal predicates such as `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` compare typed Timestamp values against Timestamp columns. Bare string literals in datetime column comparisons are validated at plan time by Prism's literal pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness — date-only and offset-less forms are rejected with `Err(E-QUERY-041)` before DataFusion execution (arrow-cast 58.2.0 is lenient and would silently coerce these forms; see Error Cases). The same `chrono::DateTime::parse_from_rfc3339` strictness is applied at the sensor-boundary datetime parsing path — query-planner validation and sensor-boundary parsing use identical strictness.

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
| `E-QUERY-041` | A `Timestamp(Microsecond, UTC)` datetime column compared against a bare string literal that Prism's plan-time literal pre-validator (`chrono::DateTime::parse_from_rfc3339` strictness) rejects — e.g., `WHERE timestamp > '2026-06-24'` (date-only, no time component) or `WHERE timestamp > '2026-06-24T12:00:00'` (offset-less ISO form); detected before DataFusion sees the query (arrow-cast 58.2.0 is lenient and would silently coerce these to wrong values) | `"E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp. Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."` |
| `E-QUERY-001` | Syntax error in SQL | Error with position, context, and SQL syntax reference |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-006 | `SELECT *` with no WHERE clause | Valid query; returns all materialized events (up to `limit` parameter) |
| EC-11-007 | `GROUP BY` with aggregate functions | DataFusion handles aggregation; results include grouped fields + aggregate values |
| EC-11-008 | `ORDER BY` on a field not in `SELECT` | Valid per SQL semantics; DataFusion handles this correctly |
| EC-11-003-001 | `SELECT * FROM events WHERE timestamp > '2026-06-24'` (date-only bare string literal in datetime comparison) | `Err(E-QUERY-041)`: Prism plan-time pre-validator (`chrono::DateTime::parse_from_rfc3339` strictness) rejects `'2026-06-24'` (date-only form fails strict RFC-3339 parse) — use full RFC-3339 UTC form `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'` |
| EC-11-003-002 | `SELECT * FROM events WHERE timestamp > '2026-06-24T00:00:00Z'` (valid RFC-3339 UTC string literal in datetime comparison) | Valid; passes Prism plan-time pre-validator (`chrono::DateTime::parse_from_rfc3339` accepts full RFC-3339 with `Z` UTC offset) — proceeds to DataFusion without error |

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
| `SELECT * FROM events WHERE timestamp > '2026-06-24'` | `Err(E-QUERY-041)` Prism plan-time pre-validator rejects date-only string (use `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'`) | error |
| `SELECT * FROM events WHERE timestamp > '2026-06-24T00:00:00Z'` | Valid query; passes Prism plan-time pre-validator — full RFC-3339 UTC form accepted | happy-path |

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
| 1.6 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 v1.1 correction (remove-uncertainty PASS-1 amendments).** §Postconditions ADR-052 D2 bullet: corrected E-QUERY-041 detection mechanism from "DataFusion cannot implicitly cast" to Prism plan-time literal pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness — arrow-cast 58.2.0 is LENIENT (accepts date-only and offset-less strings via coercion); Prism must validate at parse/plan time before DataFusion sees the query. Added: same chrono strictness applies at sensor-boundary datetime parsing (AC-013 consistency). Postcondition example updated to `arrow_cast(...)` form. Error Cases E-QUERY-041: trigger condition corrected to "Prism plan-time pre-validator rejects" (not "DataFusion cannot cast"); offset-less ISO form example added. Edge cases EC-11-003-001/002: mechanism descriptions updated (chrono pre-validator, not DataFusion implicit cast). Test vectors: descriptions updated to reflect pre-validator language. |
| 1.5 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 amendment (ratified 2026-07-03).** §Postconditions: added ADR-052 D2 datetime column Arrow type assertion — `ColumnType::Datetime` registers as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `Utf8`); typed Timestamp-vs-Timestamp comparison; bare string literal cast failure returns `Err(E-QUERY-041)`. Error Cases: E-QUERY-041 `TemporalLiteralUnparseable` added. Edge Cases: EC-11-003-001 (date-only bare string → E-QUERY-041) and EC-11-003-002 (valid RFC-3339 UTC string → OK) added. Canonical Test Vectors: two E-QUERY-041 / RFC-3339 vectors added. inputs: ADR-052 file added. |
| 1.4 | pr-127-review-remediation | 2026-05-05 | product-owner | Added canonical Denied SQL Statement Prefixes section (~40 keywords across DML/DDL/TCL/DCL/procedural/utility/vendor categories). Updated Invariants to reference E-QUERY-002 for mutation rejection. Updated Error Cases table to use E-QUERY-002 for denylist hits. Expanded Canonical Test Vectors with 9 new denylist vectors including whitespace-normalization and identifier-vs-keyword boundary cases. Addresses Adv OBS-002 [process-gap] from PR-127 review. NOTE: implementer follow-up required — extend filter_parser.rs denylist from 7 to ~40 keywords per new Denied SQL Statement Prefixes table. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
