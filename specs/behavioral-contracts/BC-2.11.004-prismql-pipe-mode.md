---
document_type: behavioral-contract
level: L3
version: "1.5"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
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

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-009 | Pipe mode with no source prefix (starts with `| where ...`) | Valid; operates on the full materialized events table |
| EC-11-010 | `head 0` | Returns empty result set (valid but unusual) |
| EC-11-011 | `dedup` on a field with all unique values | Returns all rows (no deduplication occurs) |
| EC-11-012 | Multiple `where` stages in sequence | Valid; equivalent to AND-ing the conditions. Each `where` narrows the previous result. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `| where severity = 'critical' \| stats count by _sensor` | Aggregate counts per sensor for critical events | happy-path |
| `| where severity = 'high' \| sort event_time desc \| head 10` | Top 10 recent high-severity events | happy-path |
| 33 pipe stages chained | `Err(E-QUERY-003)` pipe stage limit exceeded | error |
| `| stats invalid_func by severity` | `Err(E-QUERY-001)` invalid aggregation function | error |
| `| head 0` | Empty result set (valid) | edge-case |

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
| 1.5 | bundle-a.2.2 | 2026-05-08 | state-manager | POL-14 promotion: draft → active. S-3.06 flipped to merged (D-304 / Bundle A.2). |
| 1.4 | pre-impl-amendments | 2026-05-06 | product-owner | AMENDMENT 4 — three S-3.06 implementer gaps: (a) write verb case-insensitive matching specified (normalize to lowercase on insert+lookup, consistent with SQL conventions); (b) E-QUERY-022 added to Error Cases table (unbounded DML write rejected at parse time); (c) INV-FILTER-EMPTY-REGISTRY specified (empty WriteVerbRegistry → reject_write_verbs_in_filter always Ok(())). Also added E-QUERY-010/023/024 rows to Error Cases for completeness. Three new invariants added to §Invariants. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
