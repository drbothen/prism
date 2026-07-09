---
document_type: behavioral-contract
level: L3
version: "1.8"
status: active
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: 2026-07-09
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-043-true-sql-to-pipe-composition-select-from-t-stage-head-lowers-to-pipe-source.md"
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
---

# BC-2.11.020: SQL→Pipe Composition — `SqlPipe` AST Variant and FORBID-BOTH Dual-Limit Rule

## Description

When a PrismQL query begins with `SELECT … FROM <table>` and contains at least one `| <pipe_stage>` suffix (outside string literals), the parser dispatches to a new `parse_sql_pipe` combinator that produces an `Ast::SqlPipe(SqlPipeQuery)` node. Execution lowers the SQL head to a virtual pipe source, then applies the pipe stage sequence. A composed query MUST NOT specify both a SQL `LIMIT N` clause in the head and a row-capping pipe stage (`| limit M` OR `| tail M`) in the tail; doing so is a plan-time rejection with E-QUERY-040.

## Preconditions

- A query string begins with `SELECT` (case-insensitive) AND contains at least one unquoted `|` token followed by a pipe stage keyword (`where`, `sort`, `limit`, `head`, `tail`, `stats`, `dedup`, `fields`, `enrich`)
- The query string has passed the 64KB length check (BC-2.11.006)
- Mode detection (`is_pipe_mode` / `parse_with_limits`) has identified `SqlPipeMode` — the tristate extension of the former bool return from `is_pipe_mode` per ADR-043 D5

## Postconditions

- The Chumsky parser produces `Ast::SqlPipe(SqlPipeQuery)` where `SqlPipeQuery { head: SqlQuery, stages: Vec<PipeStage> }`
- `head` is the SQL preamble parsed as a standard `SqlQuery` (BC-2.11.003); `stages` is the ordered pipe stage sequence (BC-2.11.004)
- Execution: the executor's `Ast` match arm handles `Ast::SqlPipe(sq)` by (a) executing `sq.head` as a normal SQL query producing an Arrow `RecordBatch`, then (b) passing that result through `sq.stages` via the existing pipe stage execution path — zero new execution infrastructure
- Security: `parse_sql_pipe` applies the same `scan_inputs_audited` injection-defense pass, query-size check, and nesting-depth limit as `parse_sql` and `parse_pipe` (DI-019)
- **FORBID-BOTH rule (ADR-043 D4 / HRG-1):** If `sq.head.limit` is `Some(N)` AND `sq.stages` contains a row-capping pipe stage (`| limit M` OR `| tail M`), the planner returns `Err(PrismError::RedundantRowLimit { sql_limit: N, pipe_limit: M })` before any DataFusion execution. The check fires for `PipeStage::Limit(_) | PipeStage::Tail(_)` — both are row-capping operators. Message format: see E-QUERY-040 in error-taxonomy.
- When `normalized_pql` echo is present (BC-2.11.018), the `normalized_pql` field in the response carries the canonicalized pipe form of the composed query

## Invariants

- DI-019: All security limits (64KB size, 64 nesting depth, 32 pipe stages) apply to the composed query; limits are applied across both the SQL head and the pipe stages collectively
- Pure SQL queries (no `|` outside string literals) continue to parse as `Ast::Sql` (BC-2.11.003) — UNCHANGED
- Pure pipe queries continue to parse as `Ast::Pipe` (BC-2.11.004) — UNCHANGED
- This is strictly additive: no existing `Ast::Sql` or `Ast::Pipe` behavior is modified
- The `#[non_exhaustive]` attribute on `Ast` ensures external downstream crates already have a wildcard arm and compile without change when `Ast::SqlPipe` is added
- **INV-FORBID-BOTH-PERMANENT:** The FORBID-BOTH ruling is permanent. SQL-wins (SQL head `LIMIT` takes precedence silently) is NEVER acceptable. The only acceptable future relaxation is pipe-wins (removing the restriction so both caps compose as `| limit` final cap wins), which is a non-breaking relaxation and may be adopted in a future cycle; it does NOT change this invariant retroactively.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-038` | Column referenced in the `Ast::SqlPipe` query (position 9: `\| where` stage predicates — same walking as position 8; and positions 10–14 for the pipe stage tail: `\| sort` keys, `\| stats ... by` grouping refs AND aggregate function argument field paths (checked before Stats replaces binding context), `\| fields` column refs, `\| enrich` input column (position 13 — checked before Enrich updates binding context), `\| dedup` field keys (position 14); positions 1–6 for the SQL head) not found in `TableRegistry` schema for the table and client (or not in the running derived-column binding context). **Derived-column binding rule applies to SqlPipe pipe stages:** Enrich stages union infusion output columns when resolvable (fail-open with `suspended = true` otherwise); Stats stages replace available with `{explicit_aliases ∪ by_fields}`; once suspended all remaining stages skip E-QUERY-038 (FP-001 false-positive prohibition). **SQLPIPE HEAD-PROJECTION BINDING RULE (BC-2.11.016 v1.14):** for `Ast::SqlPipe` stage positions (8–14), the initial `available` set is seeded from the HEAD SQL's projection output — not the raw schema — with three branches: (a) `SELECT *` → full raw schema (unchanged); (b) fully-explicit SELECT (no Star/TableStar items) → `{aliases} ∪ {bare Field names} ∪ {GROUP BY fields}`; any non-`Field` SELECT item without alias → `suspended := true` (fail-open; FP-001); (c) MIXED-STAR SELECT (at least one Star/TableStar AND at least one explicit item) → `schema_columns ∪ {aliases from explicit items} ∪ {bare Field names of un-aliased explicit bare-Field items} ∪ {GROUP BY fields}`; if any explicit non-`Field` item lacks an alias → additionally `suspended := true` (FP-001 fail-open). Positions 1–6 run against raw schema unchanged. | `E-QUERY-038` with `column`, `available_columns` (org-scoped), `did_you_mean`; MCP `-32602 INVALID_PARAMS`. Gate fires at plan time — no sensor API call made. Full spec: BC-2.11.016 v1.14 (complete fourteen-position enumeration; derived-column binding rule; position-11 aggregate arg coverage; SQLPIPE HEAD-PROJECTION BINDING RULE for initial `available` set at stage positions, including MIXED-STAR branch (c)). |
| `E-QUERY-040` | SQL head `LIMIT N` AND a row-capping `| limit M` or `| tail M` pipe stage both present (e.g. `SELECT … LIMIT N … FROM t | limit M` or `SELECT … LIMIT N … FROM t | tail M`) | Plan-time pedagogical error: "E-QUERY-040: redundant row limit. This query caps rows in two places: a SQL `LIMIT {sql_limit}` in the head and a row-capping `| limit`/`| tail` pipe stage (cap: {pipe_limit}). PrismQL requires exactly one row cap. Remove the SQL `LIMIT {sql_limit}` and place a single `| limit` at the end of the pipeline (recommended for composed queries), or use `LIMIT` only in pure SQL-mode queries." MCP mapping: `-32602 INVALID_PARAMS` (caller-resolvable). `{pipe_limit}` is the integer M from the `| limit M` or `| tail M` pipe stage (whichever is present). |
| `E-QUERY-001` | SQL head has a syntax error (standard SQL parse error) | Delegates to BC-2.11.003 SQL mode parse error handling |
| `E-QUERY-001` | Unknown pipe stage keyword after `|` | Delegates to BC-2.11.004 pipe mode error handling |
| `E-QUERY-003` | Composed query exceeds 32 pipe stages in the tail | `"E-QUERY-003: pipe stage count {n} exceeds maximum allowed 32"` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-020-001 | `SELECT * FROM t \| enrich threat_score(ip)` — simplest composed query | Parses as `Ast::SqlPipe`; executes correctly; no LIMIT conflict |
| EC-11-020-002 | `SELECT * FROM t LIMIT 10 \| enrich threat_score(ip) \| limit 5` | `Err(E-QUERY-040)` — FORBID-BOTH rule fires; sql_limit=10, pipe_limit=5 |
| EC-11-020-003 | `SELECT * FROM t LIMIT 10 \| enrich threat_score(ip)` — SQL LIMIT, no pipe `\| limit` | Valid; SQL head `LIMIT 10` applies as the single row cap |
| EC-11-020-004 | `SELECT * FROM t \| enrich threat_score(ip) \| limit 5` — no SQL LIMIT, pipe `\| limit` | Valid; pipe `\| limit 5` applies as the single row cap |
| EC-11-020-005 | `SELECT * FROM t \| where severity = 'HIGH' \| sort time DESC \| limit 25` | Valid multi-stage composed query; no LIMIT conflict |
| EC-11-020-006 | `SELECT * FROM t` with no `|` following | Pure SQL mode — parses as `Ast::Sql` (BC-2.11.003), NOT as `Ast::SqlPipe` |
| EC-11-020-007 | `SELECT * FROM t \|` with `|` at end of input (no stage keyword) | Mode detection sees `|` but no pipe-stage keyword after it; falls through to mode-bridge error (BC-2.11.023) |
| EC-11-020-008 | `SELECT * FROM t LIMIT 10 \| enrich threat_score(ip) \| tail 5` — SQL LIMIT, `| tail` pipe stage | `Err(E-QUERY-040)` — FORBID-BOTH rule fires; sql_limit=10, pipe_limit=5 |
| EC-11-020-009 | `SELECT * FROM t \| enrich threat_score(ip) \| tail 5` — no SQL LIMIT, pipe `\| tail` | Valid; pipe `\| tail 5` applies as the single row cap |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT * FROM crowdstrike_detections \| enrich threat_score(src_ip) \| limit 10` | `Ast::SqlPipe` — executes SQL head then enrich+limit stages; returns enriched rows | happy-path |
| `SELECT severity, count(*) FROM crowdstrike_detections GROUP BY severity \| sort count DESC \| head 5` | `Ast::SqlPipe` — aggregate in SQL head, then sort+head stages | happy-path |
| `SELECT * FROM t LIMIT 5 \| enrich fn(x) \| limit 3` | `Err(E-QUERY-040)` with sql_limit=5 and pipe_limit=3 | error |
| `SELECT * FROM t LIMIT 5 \| enrich fn(x) \| tail 3` | `Err(E-QUERY-040)` with sql_limit=5 and pipe_limit=3 | error |
| `SELECT * FROM t` | `Ast::Sql` (not SqlPipe — no `\|`) | boundary |
| `FROM t \| enrich fn(x) \| limit 10` | `Ast::Pipe` (not SqlPipe — does not start with SELECT) | boundary |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-014 | Query security limits: rejects oversized queries | kani |
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Related BCs

- **BC-2.11.003** (composes with — SQL head is a `SqlQuery`): the SQL head of a `SqlPipeQuery` is parsed using the same rules as SQL mode
- **BC-2.11.004** (composes with — pipe stages are `PipeStage`): the stages in a `SqlPipeQuery` are the same `PipeStage` enum used by pure pipe mode
- **BC-2.11.023** (depends on — mode-bridge error): when `|` in SQL context is not a valid composition trigger, BC-2.11.023's mode-bridge diagnostic fires

## Architecture Anchors

- `crates/prism-query/src/filter_parser.rs` — `is_pipe_mode` / `parse_with_limits` (mode detection + dispatch)
- `crates/prism-query/src/ast.rs` — `Ast` enum (add `SqlPipe(SqlPipeQuery)` variant)
- `crates/prism-query/src/materialization.rs` — `execute_against_session` `Ast::SqlPipe` arm
- ADR-043: True SQL→Pipe Composition

## Story Anchor

TBD (story-writer assigns after decomposition)

## VP Anchors

VP-021 (fuzz), VP-014 (size limit)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC governs SQL→Pipe composition within the PrismQL query parser, which is the core of the ephemeral query engine. CAP-015 explicitly describes "Three query modes: filter, SQL, and pipe" and this BC defines the new composed `SqlPipe` mode that unifies SQL and pipe entry syntaxes. |
| L2 Invariants | DI-019 |
| Priority | P0 |
| Closes findings | GRAMMAR-001, GRAMMAR-009 (mode-mixing parse failure at `\|` in SQL mode), ADR-043 D4 (FORBID-BOTH dual-limit rule) |
| ADR traces | ADR-043 v1.2 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | FIX-IEQ-ERRPATH-001-ADV-FIX-P6-MED-002 | 2026-07-09 | product-owner | **ADV-FIX-P6-MED-002 POL-25 companion to BC-2.11.016 v1.14 — MIXED-STAR branch added to SQLPIPE HEAD-PROJECTION BINDING RULE.** §Error Cases E-QUERY-038 Condition column: SQLPIPE HEAD-PROJECTION BINDING RULE description expanded from two branches to three — (a) `SELECT *` → full raw schema (unchanged); (b) fully-explicit SELECT (no Star/TableStar) → `{aliases} ∪ {bare Field names} ∪ {GROUP BY fields}` (anonymous-item suspension unchanged); **(c) MIXED-STAR SELECT (at least one Star/TableStar AND at least one explicit item) → `schema_columns ∪ {aliases} ∪ {bare Field names} ∪ {GROUP BY fields}`; any anonymous explicit non-`Field` item without alias → additionally `suspended := true` (FP-001 fail-open).** BC anchor updated from v1.13 → v1.14 in both Condition and Behavior columns. `modified: 2026-07-09`. |
| 1.7 | FIX-IEQ-ERRPATH-001-ADV-FIX-P5-FP-001 | 2026-07-09 | product-owner | **ADV-FIX-P5-OBS-1 upgraded to FP-001 violation (POL-25 companion to BC-2.11.016 v1.13 — SQLPIPE HEAD-PROJECTION BINDING RULE).** §Error Cases E-QUERY-038 row: (1) **Condition column:** added SQLPIPE HEAD-PROJECTION BINDING RULE note — for `Ast::SqlPipe` stage positions (8–14), initial `available` is seeded from head projection output, not raw schema (`SELECT *` → full raw schema unchanged; explicit SELECT → `{aliases} ∪ {bare Field names} ∪ {GROUP BY fields}`; anonymous non-`Field` SELECT item without alias → `suspended := true` fail-open; positions 1–6 unaffected); (2) **Behavior column:** BC anchor updated from BC-2.11.016 v1.12 → v1.13; spec anchor note updated to reflect head-projection binding addition. `modified: 2026-07-09`. |
| 1.6 | FIX-IEQ-ERRPATH-001-ADV-FIX-P4-MED-002-OBS-001 | 2026-07-08 | product-owner | **ADV-FIX-P4-MED-002 + OBS-001 POL-25 closure.** §Error Cases E-QUERY-038 row: (1) **MED-002 (pin currency):** "BC-2.11.016 v1.9" BC anchor in Behavior column updated to BC-2.11.016 v1.12 — 3 versions stale (v1.9 → v1.12); (2) **OBS-001 (position-11 scope expansion):** position-11 description updated from "`\| stats ... by` grouping refs" to also cover aggregate function argument field paths (checked before Stats replaces binding context) — consistent with BC-2.11.016 v1.12 position-11 enumeration. `modified: 2026-07-08`. |
| 1.5 | FIX-IEQ-ERRPATH-001-grammar-keyword-correction-sort-by | 2026-07-08 | product-owner | **Grammar keyword correction: `\| sort by` → `\| sort` in §Error Cases E-QUERY-038 row (position 10).** Test-writer grammar verification confirmed no `by` keyword exists between `sort` and its field list in the PrismQL pipe parser. Fixed position 10 label in the E-QUERY-038 covered-positions list for `Ast::SqlPipe` pipe stage tail. BC anchor updated to BC-2.11.016 v1.9. |
| 1.4 | ADV-FIX-P2-CRIT-001-CRIT-002-HIGH-002-HIGH-003 | 2026-07-08 | product-owner | **ADV-FIX-P2-CRIT-001/002 + HIGH-002/003 closure (FIX-IEQ-ERRPATH-001 LOCAL pass-2) — POL-25 multi-cite propagation.** §Error Cases E-QUERY-038 row: (1) extended pipe stage tail positions from 10–12 to 10–14, adding (13) `\| enrich` input column and (14) `\| dedup` field keys; (2) added Derived-column binding rule note for SqlPipe pipe stages — same rule as pure Pipe mode; (3) BC anchor updated to BC-2.11.016 v1.8 (fourteen-position enumeration). `modified: 2026-07-08`. |
| 1.3 | FIX-IEQ-ERRPATH-001-grammar-keyword-correction | 2026-07-08 | product-owner | **Grammar keyword correction: `\| project` → `\| fields` in §Error Cases E-QUERY-038 row (position 12).** Test-writer grammar verification (fix-PR FIX-IEQ-ERRPATH-001) confirmed no `\| project` keyword exists in the PrismQL pipe parser; the projection stage keyword is `\| fields` (`PipeStage::Fields`). Fixed position 12 label in the E-QUERY-038 covered-positions list for `Ast::SqlPipe` pipe stage tail. |
| 1.2 | ADV-FIX-P1-HIGH-001-HIGH-002-DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 | 2026-07-08 | product-owner | **POL-25 multi-cite propagation (ADV-FIX-P1-HIGH-001 / HIGH-002 closure).** §Error Cases: added `E-QUERY-038` row — SqlPipe queries (`Ast::SqlPipe`) are now explicitly documented as subject to the twelve-position column-availability gate (BC-2.11.016 v1.6): SQL head positions 1–6 + `\| where` stage position 9 + pipe stage positions 10–12. Cross-reference to BC-2.11.016 v1.6 for full postcondition spec. `modified: 2026-07-08`. |
| 1.1 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **POL-14 BC auto-promotion: draft → active.** Anchor story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 squash-merged via PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict) cascade on frozen HEAD 356e0573). `status: draft → active`. No behavioral change; frontmatter status field only. |
| 1.1 | F-P2-HIGH-001-bc-sweep | 2026-06-25 | product-owner | **F-P2-HIGH-001 closure (POL-25 multi-cite propagation gap).** FORBID-BOTH rule extended from `\| limit M`-only to the full row-capping pipe-stage family (`\| limit M` OR `\| tail M`), matching error-taxonomy.md v2.00 E-QUERY-040 row (updated by S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 HIGH-1 fix-burst). Changes: (1) §Description updated to reference row-capping pipe stage family; (2) §Postconditions FORBID-BOTH bullet updated — condition now reads `PipeStage::Limit(_) \| PipeStage::Tail(_)`, neutral wording "a row-capping pipe stage (`\| limit M` OR `\| tail M`)"; (3) §Error Cases E-QUERY-040 row updated — condition covers both forms; message format updated to verbatim error-taxonomy v2.00 neutral wording "a row-capping `\| limit`/`\| tail` pipe stage (cap: {pipe_limit})"; `{pipe_limit}` field definition clarified; (4) EC-11-020-008 + EC-11-020-009 added (`\| tail` FORBID-BOTH and valid-`\| tail` edge cases); (5) `\| tail` E-QUERY-040 error test vector added to Canonical Test Vectors. Frontmatter: version 1.0→1.1, modified: 2026-06-25. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-043 v1.1 (FORBID-BOTH ratified). Closes GRAMMAR-001, GRAMMAR-009. Allocates E-QUERY-040 plan-time dual-limit rejection (error-taxonomy row authored in same burst). |
