---
document_type: behavioral-contract
level: L3
version: "1.12"
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

# BC-2.11.004: PrismQL Pipe Mode Parsing

## Description

Pipe mode is the highest-priority query mode: it activates whenever the query contains a `|` operator outside string literals, overriding SQL or filter mode detection. The Chumsky parser produces a `PipeExpr` AST representing a linear chain of transformation stages applied left-to-right. Each stage (`where`, `sort`, `head`, `tail`, `stats`, `dedup`, `fields`) is translated to a DataFusion `DataFrame` API call in sequence. The 32-stage limit and security limits from DI-019 apply. Pipe mode is the recommended mode for multi-step analysis workflows.

S-3.06 extends pipe mode with write stages: a write stage (sensor-registered verb in terminal pipe position) routes to the write-parser extension. Write verb matching is case-insensitive (see §Write Verb Case Sensitivity below). Unbounded DML writes (DELETE/UPDATE without WHERE) are rejected with `E-QUERY-022` before any API call is made. When `WriteVerbRegistry` is empty, `reject_write_verbs_in_filter` always returns `Ok(())` — no false positives on read-only installations.

## Preconditions
- A query string contains `|` outside of string literals -- pipe mode has highest precedence in mode auto-detection (see BC-2.11.002 for full precedence rules). Pipe mode wins even if the query also starts with `SELECT` or `FROM`.
- The query string has passed the 64KB length check

## Postconditions
- The Chumsky parser produces a `PipeExpr` AST consisting of an optional source followed by a chain of pipe stages
- Supported pipe stages:
  - `where <filter_expr>` -- filter rows using the same filter grammar as filter mode
  - `sort <field> [asc|desc] [, <field> [asc|desc]]` -- sort results
  - `head <N>` -- take first N rows (equivalent to `LIMIT N`)
  - `tail <N>` -- take last N rows (reverse sort + limit + reverse)
  - `stats <agg_func> [by <field>]` -- aggregation (count, sum, avg, min, max) with optional GROUP BY
  - `dedup <field> [, <field>]` -- deduplicate by specified fields
  - `fields [+|-] <field> [, <field>]` -- include (`+`) or exclude (`-`) specific fields from output
- Each pipe stage is translated to a DataFusion `DataFrame` API call in sequence
- Pipe stages are applied left-to-right (first stage operates on the full dataset, each subsequent stage operates on the previous stage's output)
- **ADR-052 D2 — Datetime column Arrow type:** `ColumnType::Datetime` sensor columns are registered in the DataFusion execution schema as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `DataType::Utf8`). Temporal predicates in `| where` stages such as `| where timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` compare typed Timestamp values against Timestamp columns. The PrismQL parser accepts the 7 canonical offset-less date/datetime forms in the `is_date_like` Acceptance Set (full set defined in BC-2.11.021 §Error Cases and ADR-052 §D4; representative examples: `'2026-06-24'`, `'2026-06-24T12:00:00'`) as `Literal::RawTemporalLiteral` AST nodes (parse succeeds — no E-QUERY-001 at parse time). The plan-time AST walker `check_temporal_literals` resolves each `RawTemporalLiteral` in `| where` clauses with a four-arm dispatch: (1) vs `Timestamp(Microsecond, UTC)` (Datetime) column (bare `Field` LHS) → **E-QUERY-041**; (2) vs String/Utf8 column (bare `Field` LHS) → **COERCE** to `Literal::String(s)` and compare as ordinary string literal (SUCCESS — no error; byte-identical to pre-ADR-052 behavior; see Error Cases); (3) vs Integer/Float/Bool column (bare `Field` LHS) → **E-QUERY-002 (QueryTypeMismatch)**; (4) in `| where` comparison where LHS is a function or compound expression (non-`Field`) → **E-QUERY-042** (`TemporalLiteralInvalidPosition`, NonColumnLhsComparison — LHS type cannot be resolved at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions). **Note — `stats by` and `sort` positions (pipe-mode parse-time rejection):** Bare date-like literals in `stats <agg_func> by '2026-06-24'` and `sort '2026-06-24'` positions are rejected AT PARSE TIME with E-QUERY-001 (enhanced message per ADR-052 §D4 v1.10 Option (a): "pipe stats by expects column references (field names), not literal values") — the `check_temporal_literals` walker is never invoked for these positions; they are not walker-level arms. The `chrono::DateTime::parse_from_rfc3339` strictness is preserved at the sensor-boundary datetime parsing path (AC-013) — both the AST-walk plan-time gate and the sensor-boundary parser reject non-RFC-3339 datetime forms; mechanisms differ but the contract is preserved.

### Write Parser Extension (S-3.06)

- A write stage is recognized in terminal pipe position: `| <verb> <target> [args]` where `<verb>` is a sensor-registered write verb from `WriteVerbRegistry`
- Write stages are ONLY valid in terminal position (the last stage of a pipe); a write stage in non-terminal position returns `E-QUERY-024`
- Non-terminal pipe stages that contain write verbs are rejected with `E-QUERY-024` before DataFusion planning

#### Write Verb Case Sensitivity (Gap a)

Write verb matching is **case-insensitive**. `WriteVerbRegistry` normalizes verb names to lowercase on insert (via `from_source`) and on lookup (via `is_write_verb`). The parser calls `is_write_verb` on the lowercased form of the identifier seen in the query. This is consistent with PrismQL's SQL-style identifier conventions (SQL keywords are case-insensitive) and aligns with the SQL/filter parser convention established in S-3.01.

Examples:
- `| ISOLATE crowdstrike_hosts id = "abc"` — recognized (normalized to `isolate`)
- `| Isolate crowdstrike_hosts id = "abc"` — recognized (normalized to `isolate`)
- `| isolate crowdstrike_hosts id = "abc"` — canonical form

#### Unbounded Write Protection (Gap b)

DML write operations (DELETE, UPDATE, INSERT INTO ... SELECT) without a WHERE clause are rejected at parse time with `E-QUERY-022`. This prevents accidental mass-modification of sensor-managed data.

- `DELETE FROM <table>` (no WHERE) → `E-QUERY-022`
- `UPDATE <table> SET <col>=<val>` (no WHERE) → `E-QUERY-022`
- `INSERT INTO <table> SELECT ...` without LIMIT or WHERE on source SELECT → `E-QUERY-022`

The `--all` opt-in flag (or equivalent explicit bypass mechanism) may be defined per-sensor-spec to allow intentional unbounded operations — the BC requires that unbounded writes MUST be explicit, never the default.

#### Empty WriteVerbRegistry Behavior (Gap c)

When `WriteVerbRegistry` is empty (no sensor write endpoints registered), `reject_write_verbs_in_filter` always returns `Ok(())`. No false-positive errors are raised on read-only Prism installations or during test scenarios where write specs are not loaded. An empty registry means no write verbs exist to reject — filter parsing proceeds normally.

## Invariants
- DI-019: Maximum 32 pipe stages enforced; exceeding returns structured error
- DI-019: Nesting depth within `where` expressions tracked against the 64 depth limit
- Each pipe stage produces a valid intermediate `DataFrame`; stage ordering errors (e.g., `head` before `stats`) are not compile errors -- DataFusion evaluates them in order
- **BC-2.11.004 §INV-FILTER-EMPTY-REGISTRY:** When `WriteVerbRegistry` is empty, `reject_write_verbs_in_filter` always returns `Ok(())`. No false positives on read-only installations or test environments without write verb specs loaded.
- **BC-2.11.004 §INV-WRITE-VERB-CASE-INSENSITIVE:** Write verb names are normalized to lowercase at registry-insert time and at lookup time. The parser never performs case-sensitive verb matching. `"ISOLATE"` and `"isolate"` are identical in the write parser.
- **BC-2.11.004 §INV-UNBOUNDED-WRITE-REJECTED:** DML operations without a WHERE clause (DELETE, UPDATE) or an INSERT INTO SELECT without LIMIT or WHERE are rejected at parse time before any API call. The rejection is not bypassable without an explicit opt-in mechanism defined in the sensor spec.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | Unknown pipe stage keyword | Error with list of supported stages |
| `E-QUERY-003` | More than 32 pipe stages | Structured error: "Query has N pipe stages (max 32). Simplify the query pipeline." |
| `E-QUERY-001` | `stats` with invalid aggregation function | Error listing supported aggregation functions |
| `E-QUERY-001` | `head` or `tail` with non-integer argument | Error: "head/tail requires a positive integer argument" |
| `E-QUERY-010` | Write stage targets an internal Prism table (`prism_audit`, `prism_metrics`, etc.) | Parse-time rejection: `"E-QUERY-010: write to internal table '{table}' is not permitted"` |
| `E-QUERY-022` | Unbounded DML write: DELETE or UPDATE without WHERE clause, or INSERT INTO SELECT without LIMIT or WHERE | `"E-QUERY-022: unbounded {verb} rejected — add a WHERE clause (or LIMIT for INSERT...SELECT) to scope the operation, or use explicit opt-in if provided by the sensor spec"` — rejected at parse time before any API call |
| `E-QUERY-023` | Unknown write verb in terminal pipe position | `"E-QUERY-023: unknown write verb '{verb}' for sensor '{sensor}'"` — includes suggestion list of available verbs from `WriteVerbRegistry::verbs_for_sensor` |
| `E-QUERY-024` | Write stage in non-terminal pipe position | `"E-QUERY-024: write stage must be in terminal pipe position"` — write stages cannot be followed by additional stages |
| `E-QUERY-041` | A `Timestamp(Microsecond, UTC)` datetime column compared against a date-like string literal in a `| where` stage (bare `Field` LHS). The PrismQL parser emits `Literal::RawTemporalLiteral` for any of the 7 canonical offset-less date/datetime forms in the `is_date_like` Acceptance Set (see BC-2.11.021 §Error Cases / ADR-052 §D4; e.g., `'2026-06-24'`, `'2026-06-24T12:00:00'`) (parse succeeds); the plan-time AST walker `check_temporal_literals` raises E-QUERY-041 upon resolving the column type as `Timestamp(Microsecond, UTC)`. **Does NOT fire for String/Utf8 columns** — `check_temporal_literals` coerces `RawTemporalLiteral` to `Literal::String(s)` for String columns in `| where` comparisons (SUCCESS, no error; `\| where string_col = '2026-06-24'` is valid — arm (2) of four-arm dispatch). **Does NOT fire for `stats by` or `sort` positions** — those positions reject at parse time with E-QUERY-001 (enhanced message per Option (a)); `check_temporal_literals` walker is never reached. **Does produce E-QUERY-042** for non-`Field` LHS comparisons in `| where` (arm (4) — see E-QUERY-042 row). | `"E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp. Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."` |
| `E-QUERY-042` | A date-like string literal in a `| where` comparison where the LHS is a function or compound expression (non-`Field`). The plan-time AST walker `check_temporal_literals` detects non-`Field` LHS → `TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: <first_50_chars> }`. LHS type unresolvable at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions. Example: `\| where lower(hostname) = '2026-06-24'`. **Does NOT fire** for `stats by` or `sort` literals — those are parse-time rejections (E-QUERY-001, not this error). | `"E-QUERY-042: A date-like literal compared against a computed expression cannot be type-checked at plan time. Compare against a bare datetime column using RFC-3339 (e.g., '2026-07-03T00:00:00Z'), against a string column using a non-date-shaped value, or wrap the expression in an explicit CAST."` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-009 | Pipe mode with no source prefix (starts with `| where ...`) | Valid; operates on the full materialized events table |
| EC-11-010 | `head 0` | Returns empty result set (valid but unusual) |
| EC-11-011 | `dedup` on a field with all unique values | Returns all rows (no deduplication occurs) |
| EC-11-012 | Multiple `where` stages in sequence | Valid; equivalent to AND-ing the conditions. Each `where` narrows the previous result. |
| EC-11-004-001 | `| where timestamp > '2026-06-24'` (date-only bare string literal in datetime `| where` predicate against a Datetime/Timestamp column) | `Err(E-QUERY-041)`: PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); plan-time walker `check_temporal_literals` resolves `timestamp` as `Timestamp(Microsecond, UTC)` → E-QUERY-041. Use full RFC-3339 UTC form `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'` |
| EC-11-004-002 | `| where timestamp > '2026-06-24T00:00:00Z'` (valid RFC-3339 UTC string literal in datetime `| where` predicate) | Valid; full RFC-3339 form parses directly to `Literal::Timestamp` (not `RawTemporalLiteral`) — `check_temporal_literals` does not intercept it; emitted as `arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` — proceeds to DataFusion without error |
| EC-11-004-003 | `| where report_date = '2026-06-24'` where `report_date` is a String/Utf8 column (not a Datetime column) | Valid — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")`; `check_temporal_literals` resolves `report_date` as `DataType::Utf8` (String column) → COERCE to `Literal::String("2026-06-24")`. Processed as a normal string comparison in the `| where` stage. No E-QUERY-041 emitted. Byte-identical to pre-ADR-052 behavior. |
| EC-11-004-004 | `FROM crowdstrike_detections \| stats count by '2026-06-24'` (temporal literal as GROUP BY literal in `stats` stage) | **FORBIDDEN — parse-time rejection.** The PrismQL parser detects a bare literal in the `stats by` position and rejects immediately with E-QUERY-001 (enhanced message per ADR-052 §D4 v1.10 Option (a)): `"E-QUERY-001: pipe stats by expects column references (field names), not literal values — '2026-06-24' looks like a date-shaped literal, not a column name. Grouping by a literal constant has no effect. Did you mean to reference a column, or to add a \| where filter before the stats stage?"`. The `check_temporal_literals` walker is never invoked. **F-MED-1 correction: prior v1.11 description was wrong** — this case does NOT coerce to success; it is a parse-time E-QUERY-001. |
| EC-11-004-005 | `FROM crowdstrike_detections \| where lower(hostname) = '2026-06-24'` (temporal literal in `| where` comparison where LHS is a function expression, not a bare `Field` column reference) | `Err(E-QUERY-042)` NonColumnLhsComparison — PrismQL parser emits `Literal::RawTemporalLiteral("2026-06-24")` (parse succeeds); `check_temporal_literals` walker detects non-`Field` LHS → arm (4) E-QUERY-042 `TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: "2026-06-24" }`. LHS type unresolvable at plan time. Closes prior `-32000 INTERNAL_ERROR` bug for function-expression LHS in pipe mode. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `| where severity = 'critical' \| stats count by _sensor` | Aggregate counts per sensor for critical events | happy-path |
| `| where severity = 'high' \| sort event_time desc \| head 10` | Top 10 recent high-severity events | happy-path |
| 33 pipe stages chained | `Err(E-QUERY-003)` pipe stage limit exceeded | error |
| `| stats invalid_func by severity` | `Err(E-QUERY-001)` invalid aggregation function | error |
| `| head 0` | Empty result set (valid) | edge-case |
| `FROM crowdstrike_detections \| where timestamp > '2026-06-24'` | `Err(E-QUERY-041)` — `check_temporal_literals` resolves `RawTemporalLiteral` vs Datetime column → E-QUERY-041 (use `'2026-06-24T00:00:00Z'` or `NOW() - INTERVAL 'Nh'`) | error |
| `FROM crowdstrike_detections \| where timestamp > '2026-06-24T00:00:00Z'` | Valid; parses to `Literal::Timestamp` (not `RawTemporalLiteral`); emitted as `arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` — no `RawTemporalLiteral` emitted, no E-QUERY-041 | happy-path |
| `FROM crowdstrike_detections \| where report_date = '2026-06-24'` (String/Utf8 column `report_date`) | Valid — `check_temporal_literals` coerces `RawTemporalLiteral` → `Literal::String("2026-06-24")`; processed as string comparison in `\| where` (no E-QUERY-041) | edge-case |
| `FROM crowdstrike_detections \| stats count by '2026-06-24'` (temporal literal as GROUP BY literal in `stats`) | `Err(E-QUERY-001)` — parse-time rejection (enhanced message per ADR-052 §D4 v1.10 Option (a)): `stats by` expects column references, not literal values. **F-MED-1 fix:** prior v1.11 vector was wrong (showed COERCE). | error |
| `FROM crowdstrike_detections \| where lower(hostname) = '2026-06-24'` (non-`Field` LHS in `\| where` comparison) | `Err(E-QUERY-042)` NonColumnLhsComparison — `check_temporal_literals` arm (4): non-`Field` LHS → E-QUERY-042 `TemporalLiteralInvalidPosition { position: NonColumnLhsComparison, value_prefix: "2026-06-24" }` | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
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
| 1.12 | adr-052-d4-v1.10-seven-arm-f-med-1 | 2026-07-05 | product-owner | **FIX 4 (F-MED-1 + ADR-052 §D4 v1.10 Option (a)): reclassify `stats by` / `sort` positions as parse-time rejections; add non-`Field` LHS E-QUERY-042.** §Postconditions ADR-052 D2 bullet: "four-way dispatch" → "four-arm dispatch" (pipes have no SQL-mode SELECT/GROUP BY/ORDER BY arms); arm (4) changed from `stats by / sort` non-comparison COERCE to non-`Field` LHS in `\| where` → E-QUERY-042 NonColumnLhsComparison; added explicit note that `stats by` and `sort` positions reject AT PARSE TIME with E-QUERY-001 (enhanced message, Option (a)) — walker is never reached. §Error Cases E-QUERY-041: "Does NOT fire" note updated — removed `stats by` / `sort` coerce claim; clarified those are parse-time rejections; added forward-ref to E-QUERY-042 for non-`Field` LHS. E-QUERY-042 row ADDED (NonColumnLhsComparison variant only for pipe mode; verbatim ADR-052 §D4 v1.10 message, POL-24). §Edge Cases: **EC-11-004-004 F-MED-1 RECLASSIFIED** from "Valid — arm (4) COERCE" to "FORBIDDEN — parse-time E-QUERY-001 with enhanced message"; EC-11-004-005 ADDED (`\| where lower(hostname) = '2026-06-24'` → E-QUERY-042 NonColumnLhsComparison). §Canonical Test Vectors: `stats count by '2026-06-24'` vector corrected from "COERCE → success" to "Err(E-QUERY-001) parse-time rejection"; `\| where lower(hostname)` vector ADDED (E-QUERY-042). |
| 1.11 | adr-052-d4-v1.8-four-way-dispatch | 2026-07-05 | product-owner | **HIGH-1: propagate ADR-052 §D4 v1.8 four-way dispatch (non-comparison coercion arm) to pipe-mode BC.** §Postconditions ADR-052 D2 bullet: `three-way dispatch` → `four-way dispatch`; added arm (4): non-comparison position (pipe stage function argument, `stats` GROUP BY literal, `sort` literal, `fields` literal — no column type in scope) → COERCE to `Literal::String(s)` (SUCCESS). §Error Cases E-QUERY-041: expanded "Does NOT fire" note to include non-comparison position (arm (4) of four-way dispatch). §Edge Cases: EC-11-004-004 ADDED (GROUP BY literal coerce → success in pipe mode). §Canonical Test Vectors: non-comparison-position coerce vector ADDED. |
| 1.10 | med-1-e-query-002-propagation | 2026-07-04 | product-owner | **MED-1 E-QUERY-001→E-QUERY-002 correction: numeric/bool temporal dispatch arm.** §Postconditions ADR-052 D2 bullet: corrected three-way dispatch arm (3) from "Integer/Float/Bool column → **E-QUERY-001**" to "**E-QUERY-002 (QueryTypeMismatch)**". Aligns to error-taxonomy.md v2.12 (E-QUERY-002 QueryTypeMismatch) and ADR-052 §D4 v1.5. Datetime→E-QUERY-041 and String/Utf8→COERCE arms UNCHANGED. All other §Error Cases E-QUERY-001 entries (unknown pipe stage keyword, invalid aggregation function, head/tail non-integer argument) are for distinct non-temporal conditions and are UNCHANGED. |
| 1.9 | ADR-052-d4-is-date-like-canonical-ref | 2026-07-04 | product-owner | Align `is_date_like` enumeration to ADR-052 §D4 v1.4 canonical 7-form set (reference, not re-enumeration). §Postconditions D2 bullet and §Error Cases E-QUERY-041 condition: replaced stale 2-form partial enumeration ("date-only and offset-less datetime") with reference to BC-2.11.021 §Error Cases / ADR-052 §D4 `is_date_like` Acceptance Set; two representative examples retained. Three-way dispatch (Datetime→E-QUERY-041; String/Utf8→coerce; numeric/bool→E-QUERY-001) unchanged. |
| 1.8 | ADR-052-d4-v1.3-bc-amendment | 2026-07-04 | product-owner | **ADR-052 §D4 v1.3 amendment (human-ratified 2026-07-04, Option A — lenient-parse-then-AST-walk + String-column coercion modification).** E-QUERY-041 detection mechanism redesigned from chrono plan-time pre-validator (v1.7) to `Literal::RawTemporalLiteral` AST node + `check_temporal_literals` plan-time walker. **Changes:** §Postconditions ADR-052 D2 bullet: detection description updated — parser emits `RawTemporalLiteral` (parse succeeds), `check_temporal_literals` three-way dispatch: (1) Datetime col → E-QUERY-041; (2) String/Utf8 col → COERCE to `Literal::String(s)` (SUCCESS, byte-identical to pre-ADR-052); (3) Integer/Float/Bool col → E-QUERY-001. §Error Cases E-QUERY-041: condition updated to AST-walk mechanism; added "Does NOT fire for String/Utf8 columns" note. §Edge Cases: EC-11-004-001 mechanism updated; EC-11-004-002 mechanism updated; EC-11-004-003 ADDED (String-column coercion → success in `| where` stage). §Canonical Test Vectors: E-QUERY-041 and RFC-3339 vector descriptions updated; String-column coercion vector ADDED. Message format (POL-24), column type assertion (D2), write parser extension (S-3.06), and security invariants UNCHANGED. RISK-5 eliminated by design. |
| 1.7 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 v1.1 correction (remove-uncertainty PASS-1 amendments).** §Postconditions ADR-052 D2 bullet: corrected E-QUERY-041 detection mechanism from "DataFusion cannot implicitly cast" to Prism plan-time literal pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness — arrow-cast 58.2.0 is LENIENT (accepts date-only and offset-less strings via coercion); Prism must validate at parse/plan time before DataFusion sees the query. Added: same chrono strictness applies at sensor-boundary datetime parsing (AC-013 consistency). Postcondition `| where` example updated to `arrow_cast(...)` form. Error Cases E-QUERY-041: trigger condition corrected to "Prism plan-time pre-validator rejects" (not "DataFusion cannot cast"); offset-less ISO form example added. Edge cases EC-11-004-001/002: mechanism descriptions updated (chrono pre-validator, not DataFusion implicit cast). Test vectors: descriptions updated to reflect pre-validator language. |
| 1.6 | ADR-052-bc-amendment-burst | 2026-07-03 | product-owner | **ADR-052 amendment (ratified 2026-07-03).** §Postconditions: added ADR-052 D2 datetime column Arrow type assertion — `ColumnType::Datetime` registers as `DataType::Timestamp(Microsecond, Some("UTC"))` (not `Utf8`); typed Timestamp-vs-Timestamp comparison in `| where` stages; bare string literal cast failure returns `Err(E-QUERY-041)`. Error Cases: E-QUERY-041 `TemporalLiteralUnparseable` added. Edge Cases: EC-11-004-001 (date-only bare string → E-QUERY-041) and EC-11-004-002 (valid RFC-3339 UTC string → OK) added. Canonical Test Vectors: two E-QUERY-041 / RFC-3339 vectors added. inputs: ADR-052 file added. |
| 1.5 | bundle-a.2.2 | 2026-05-08 | state-manager | POL-14 promotion: draft → active. S-3.06 flipped to merged (D-304 / Bundle A.2). |
| 1.4 | pre-impl-amendments | 2026-05-06 | product-owner | AMENDMENT 4 — three S-3.06 implementer gaps: (a) write verb case-insensitive matching specified (normalize to lowercase on insert+lookup, consistent with SQL conventions); (b) E-QUERY-022 added to Error Cases table (unbounded DML write rejected at parse time); (c) INV-FILTER-EMPTY-REGISTRY specified (empty WriteVerbRegistry → reject_write_verbs_in_filter always Ok(())). Also added E-QUERY-010/023/024 rows to Error Cases for completeness. Three new invariants added to §Invariants. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
