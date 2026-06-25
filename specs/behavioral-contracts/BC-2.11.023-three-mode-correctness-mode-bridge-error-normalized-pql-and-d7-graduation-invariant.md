---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: "2026-06-24"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
---

# BC-2.11.023: Three-Mode Correctness — Mode-Bridge Error, `normalized_pql`, and D7 Graduation Invariant

## Description

PrismQL has three parse-entry modes (Filter, SQL, Pipe) that share a single underlying execution model (Pipe is canonical execution; Filter and SQL are syntactic sugar that lower into it). This BC governs: (1) the mode-bridge diagnostic emitted when a user mixes modes in an unsupported way (SQL mode hits `|`, SQL clause keyword in pipe position); (2) the `normalized_pql: Option<String>` field on `StructuredErrorFields` (in `crates/prism-mcp/src/error_mapping.rs`) that carries a best-effort pipe-mode rewrite in the structured MCP error response; (3) the D7 shared-predicate-grammar invariant (Filter mode predicate grammar is exactly the `WHERE`/`| where` predicate grammar); (4) Filter mode end-to-end execution validation.

## Preconditions

- A query has been dispatched to `parse_with_limits` and mode detection has run
- For mode-bridge diagnostics: either (a) SQL mode parse failed at a `|` token, OR (b) pipe-mode parse failed at a SQL clause keyword (`SELECT`, `ORDER BY`) in stage position
- For Filter mode execution: a query has been classified as `Ast::Filter` and passed to the executor

## Postconditions

### Mode-bridge diagnostic (ADR-046 D1 — SQL mode hits `|`)

When a SQL-mode parse fails at a `|` token (i.e., the query starts with `SELECT` and the parser encounters an unquoted `|` that is NOT a valid SQL→Pipe composition trigger per BC-2.11.020), the generic Chumsky expectation dump is REPLACED with:

```
E-QUERY-001: parse error near '|': pipe stages are not valid after a SQL SELECT query in SQL mode.
To use pipe stages (enrich, where, limit, sort, stats, dedup, fields), use one of:
  1. SQL+pipe composition:  SELECT <cols> FROM <table> | <pipe_stage> …
  2. Pipe mode only:        FROM <table> | where <predicate> | <stage> …
See prismql://reference for the complete grammar.
```

The `normalized_pql` field in `ParseErrorDetails` is set to a best-effort pipe-mode rewrite when it is unambiguous (simple `FROM/WHERE/LIMIT` cases). If rewrite is not possible (JOINs, subqueries, complex projections), `normalized_pql` is `None`.

### Mode-bridge diagnostic (ADR-046 D2 — SQL clause keyword in pipe stage position)

When a pipe-mode parse fails because an uppercase SQL clause keyword (`SELECT`, `ORDER BY`) appears in stage position (not preceded by `|`), the error message is:

```
E-QUERY-001: parse error near '<keyword>': SQL clauses are not valid as pipe stages.
In pipe mode, use lowercase stage keywords: 'where', 'sort', 'limit', 'stats'.
Example: FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10
```

Note: `WHERE` and `LIMIT` (uppercase) already parse in pipe mode because keywords are case-insensitive. D2 fires specifically when `SELECT` appears in pipe stage position or `ORDER BY` appears in stage position.

### `normalized_pql` field on `StructuredErrorFields` (ADR-046 D3)

`StructuredErrorFields` (the `#[non_exhaustive]` structured MCP error payload in `crates/prism-mcp/src/error_mapping.rs`) gains a new optional field `normalized_pql: Option<String>`. The rewrite string is computed in `prism-query`'s error-recovery path (`crates/prism-query/src/error_recovery.rs`) and propagated to `StructuredErrorFields` when mapping `PrismError::QueryParseFailed` in `error_mapping.rs` — because `prism-query` MUST NOT depend on `prism-mcp` (dependency-direction rule). This field:
- Is populated on mode-bridge errors (D1) when a best-effort pipe rewrite is available
- Is populated on any `E-QUERY-001` parse error where a canonical rewrite is derivable from the query string
- Is `None` when no rewrite is derivable or when the error is not a mode-bridge error
- MUST appear in the structured MCP error response's `StructuredErrorFields` JSON payload when set
- Serializes via `#[serde(skip_serializing_if = "Option::is_none")]` — absent from JSON when `None`
- Adding this field does NOT change the `ci.yml` non-exhaustive `EXPECTED` count because `StructuredErrorFields` is already `#[non_exhaustive]` (new field on existing type, not a new type)

### D7 shared-predicate-grammar invariant

The predicate grammar is shared across all three modes:
- A predicate that parses in Filter mode (`severity = 'HIGH'`) parses identically in SQL `WHERE severity = 'HIGH'` and pipe `| where severity = 'HIGH'`
- There is exactly ONE predicate parser (`build_predicate_parser`) consumed by Filter, SQL, and Pipe parsers — no mode-specific predicate grammar extensions
- The **graduation path**: a filter-mode query (`severity = 'HIGH'`) can be escalated to a pipe query by adding `FROM <table> | where <predicate> | …` — the predicate grammar is unchanged, only the entry syntax changes

### Filter mode end-to-end execution validation (ADR-046 D4)

The implementer story MUST include at minimum:
- `test_filter_mode_simple_predicate`: executes `severity='HIGH'` as `Ast::Filter` against a mocked/DTU sensor source and verifies rows matching the predicate are returned
- `test_filter_mode_with_source`: executes `crowdstrike_detections | severity='HIGH'` (source-qualified filter) and verifies rows returned

These tests MUST use `QueryEngine::execute`, not just `PrismQlParser::parse`. Until these tests exist, Filter mode execution is UNVERIFIED.

## Invariants

- `StructuredErrorFields.normalized_pql` is ALWAYS `Option<String>` — never a required field; absent from JSON when not applicable (non-breaking addition to existing `#[non_exhaustive]` struct)
- Mode-bridge rewrites in `normalized_pql` are best-effort; the field is clearly labeled "suggested rewrite" in documentation
- Three-way composition (filter + SQL + pipe simultaneously) is NOT supported; the plan-time rejection directs to the two supported forms (pure SQL, pure Pipe, or SQL→Pipe composition per BC-2.11.020)
- Filter mode is bare-predicate sugar: its expressive power is a strict subset of SQL and Pipe modes — it adds no new capability
- **INV-SHARED-PREDICATE-GRAMMAR:** One predicate grammar, three entry points. No predicate operator valid in Filter mode is invalid in SQL `WHERE` or pipe `| where`, and vice versa.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` (mode-bridge, D1) | SQL mode parse fails at `\|` token AND query is NOT a valid SQL→Pipe composition trigger | Mode-bridge message (see Postconditions §D1); `normalized_pql` set if rewrite possible |
| `E-QUERY-001` (mode-bridge, D2) | `SELECT` appears in pipe stage position (not preceded by `\|`) | Pipe-mode error with mode-bridge guidance (see Postconditions §D2) |
| `E-QUERY-001` (standard) | Any other parse error in any mode | Standard Chumsky-derived error with position and context |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-023-001 | `SELECT * FROM t WHERE severity = 'HIGH' LIMIT 10` with no `\|` | Pure SQL mode — parses as `Ast::Sql`; no mode-bridge error |
| EC-11-023-002 | `SELECT * FROM t LIMIT 5 SORT time DESC` | SQL mode parse error (SORT is not valid SQL); NOT a mode-bridge error (no `\|` token) — standard syntax error |
| EC-11-023-003 | `SELECT * FROM t \| enrich fn(col)` | Valid SQL→Pipe composition (BC-2.11.020); no mode-bridge error |
| EC-11-023-004 | `SELECT * FROM t \| INVALID_KEYWORD` | SQL mode with `\|` not followed by pipe-stage keyword — mode-bridge error D1 fires; `normalized_pql` may be None |
| EC-11-023-005 | `severity = 'HIGH'` (Filter mode) | Valid filter mode; no mode-bridge; Filter→Sql graduation: `SELECT * FROM t WHERE severity = 'HIGH'` |
| EC-11-023-006 | `severity = 'HIGH'` graduated to pipe | `FROM t \| where severity = 'HIGH'` — same predicate grammar, different entry |
| EC-11-023-007 | Filter mode `severity = 'HIGH'` with execution | Executes as `Ast::Filter`; rows matching `severity = 'HIGH'` are returned (D4 validation required) |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT * FROM t \| unknown_stage` | `Err(E-QUERY-001)` with mode-bridge message mentioning SQL→Pipe form; `normalized_pql` may be set | error/mode-bridge |
| `SELECT * FROM t LIMIT 5` (pure SQL, no `\|`) | `Ok(Ast::Sql)` — NOT a mode-bridge scenario | happy-path |
| `severity = 'HIGH'` (Filter) | `Ok(Ast::Filter)` | happy-path |
| `severity = 'HIGH'` in SQL WHERE: `SELECT * FROM t WHERE severity = 'HIGH'` | `Ok(Ast::Sql)` — same predicate grammar | happy-path |
| `severity = 'HIGH'` in pipe: `FROM t \| where severity = 'HIGH'` | `Ok(Ast::Pipe)` — same predicate grammar | happy-path |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Related BCs

- **BC-2.11.002** (amends — Filter mode execution): this BC's D4 execution-validation mandate is an additional obligation on top of BC-2.11.002's parse-only coverage
- **BC-2.11.003** (related — SQL mode): D1 mode-bridge fires when SQL mode parse fails at `|`
- **BC-2.11.004** (related — Pipe mode): D2 mode-bridge fires when pipe mode encounters SQL clause keywords in stage position
- **BC-2.11.020** (depends on — SQL→Pipe composition): valid `SELECT … FROM t | <stage>` forms do NOT trigger the mode-bridge; BC-2.11.020 governs those cases

## Architecture Anchors

- `crates/prism-query/src/error_recovery.rs` — `rich_to_parse_error` (D1/D2 post-parse heuristic); also produces the `normalized_pql` rewrite STRING for mode-bridge errors
- `crates/prism-mcp/src/error_mapping.rs` — `StructuredErrorFields` struct (add `normalized_pql: Option<String>` field with `#[serde(skip_serializing_if = "Option::is_none")]`; populate from the `prism-query` rewrite string in the `QueryParseFailed` mapping arm). NOTE: there is NO `ParseErrorDetails` type in `prism-query/src/error.rs` — the MCP-facing structured error payload is `StructuredErrorFields` in `prism-mcp`. `prism-query` MUST NOT depend on `prism-mcp`. (D-1110 correction)
- `crates/prism-query/src/engine.rs` — `Ast::Filter` execution match arm (D4 tests target this path)
- ADR-046: Three-Mode Correctness

## Story Anchor

TBD

## VP Anchors

VP-021 (fuzz)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC governs the error-recovery layer and mode-detection behavior of the PrismQL parser within the ephemeral query engine. CAP-015 explicitly describes the three query modes and their auto-detection behavior; this BC specifies the mode-bridge diagnostics emitted when modes are mixed incorrectly and the structural invariant that Filter is syntactic sugar. |
| L2 Invariants | DI-019 |
| Priority | P0 |
| Closes findings | GRAMMAR-014 (mode-mixing parse errors dump raw Chumsky list), GRAMMAR-016 (`normalized_pql` field documented but absent), ADR-046 D4 (Filter mode end-to-end execution unvalidated), ADR-046 D7 (shared-predicate-grammar invariant) |
| ADR traces | ADR-046 v1.2 D1–D5, D7 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001-spec-sync | 2026-06-24 | product-owner | OBS-2 spec-drift correction (D-1110 ratified reality). Renamed `ParseErrorDetails` → `StructuredErrorFields` throughout; updated location from `prism-query/src/error.rs` to `prism-mcp/src/error_mapping.rs`. Added dependency-direction rationale (prism-query MUST NOT depend on prism-mcp — rewrite STRING computed in error_recovery.rs, FIELD carried on StructuredErrorFields). Clarified non-exhaustive EXPECTED count is unchanged (new field on existing type). Updated Description, Postconditions §D3, Invariants, and Architecture Anchors. Behavioral contract semantics (normalized_pql MUST appear in the structured MCP error response on D1 mode-bridge errors) are unchanged. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-046 v1.2. Closes GRAMMAR-014, GRAMMAR-016, ADR-046 D4/D7. |
