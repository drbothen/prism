---
document_type: adr
adr_id: "ADR-043"
title: "True SQL→Pipe Composition — SELECT … FROM t | stage Head-Lowers to Pipe Source"
status: accepted
date: "2026-06-24"
version: "1.2"
modified: "2026-06-25"
producer: architect
subsystems_affected: [SS-11]
supersedes: null
superseded_by: null
amends: null
anchor_stories: []
related_adrs: [ADR-041, ADR-003]
related_bcs: [BC-2.11.003, BC-2.11.004]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-043: True SQL→Pipe Composition — `SELECT … FROM t | stage` Head-Lowers to Pipe Source

## Status

ACCEPTED v1.2 (2026-06-25). Architect decision following grammar usability audit
`.factory/research/prismql-grammar-usability-audit-2026-06-24.md` (GRAMMAR-001,
GRAMMAR-009, GRAMMAR-014) and human direction to implement full Option 2 (true
SQL→pipe composition). Human ratification recorded 2026-06-24: HRG-1 ruling is
FORBID-BOTH (see §Decision D4 and §Changelog).

---

## Context

PrismQL currently has two entirely separate parse grammars selected by first-token
heuristic: SQL mode (`SELECT … FROM … WHERE … LIMIT`) and Pipe mode (`FROM … | stage |
stage`). These are disjoint Chumsky parsers — once a query starts with `SELECT`, it is
committed to SQL mode for its entire length, and the pipe `|` operator is illegal
anywhere in it. Symmetrically, SQL clauses (`WHERE`, `ORDER BY`, `LIMIT`) are not valid
as pipe stages (though they parse in pipe mode because the keywords are also pipe stage
names in lowercase; the confusion is in casing/expectation, not in acceptance).

The headline defect: `SELECT * FROM t | enrich fn(col) | limit 5` — the form documented
in `prismql://reference` line 137 and the form every SQL-literate analyst intuitively
reaches for — produces PARSE_ERR at offset 28 (`found '|'`) because SQL mode has no `|`
production. The enrichment feature (the product's flagship capability) lives exclusively
in Pipe mode, but the entry form analysts reach for is SQL mode. This is the root cause
of RUNBOOK-DRIFT-001, GRAMMAR-001, GRAMMAR-009, and GRAMMAR-014.

The grammar audit identified two resolution paths:
- **Option 1 (Mode-bridge error only):** Keep grammars disjoint; emit a pedagogical
  mode-bridge error when SQL mode hits `|`, with a `normalized_pql` rewrite. Minimum-
  viable fix, cheap, does NOT make the reference truthful.
- **Option 2 (True composition):** Allow pipe stages to follow a SQL `SELECT … FROM`
  head by "lowering" the SQL head into a virtual pipe source, making `SELECT … FROM t |
  stage` truly parse and execute. This converts the reference's documented (but broken)
  form into the truth.

The human has directed Option 2 (true SQL→pipe composition). This ADR captures the
architecture of that choice.

The existing `pipe_sql_emitter.rs` (`crates/prism-query/src/pipe_sql_emitter.rs`) handles
the current Pipe→SQL lowering for the DataFusion execution path. The SQL head→pipe source
lowering is the converse direction and leverages the same execution substrate.

---

## Decision

We implement true SQL→pipe composition in the PrismQL grammar and execution pipeline:

**D1 — Grammar extension:** The parser is extended to recognise a "SQL head" form: a
`SELECT … FROM … [WHERE …] [GROUP BY …] [HAVING …] [ORDER BY …] [LIMIT …]` preamble
followed by one or more `| pipe_stage` suffixes. This is implemented in `filter_parser.rs`
`parse_with_limits` / `is_pipe_mode` by adding a new detection branch: if the first token
is `SELECT` AND the token stream contains an unquoted `|` followed by a pipe stage
keyword, the query is dispatched to a new `parse_sql_pipe` combinator rather than
`parse_sql`.

**D2 — AST representation:** The `Ast` enum receives a new variant:
`SqlPipe(SqlPipeQuery)`, where `SqlPipeQuery { head: SqlQuery, stages: Vec<PipeStage> }`.
`SqlQuery` is the existing SQL mode AST node (BC-2.11.003). `PipeStage` is the existing
pipe mode stage type (BC-2.11.004). The `#[non_exhaustive]` attribute on `Ast` means
downstream crates already have a wildcard arm and will compile without change.

**D3 — Execution lowering:** The execution engine's `Ast` match arm (currently in
`engine.rs`) handles `Ast::SqlPipe(sq)` by: (a) executing the `SqlQuery` head as a
normal SQL query producing an Arrow `RecordBatch` result set, then (b) passing that result
set through the `PipeStage` sequence exactly as `Ast::Pipe` does today. This reuses the
entire existing pipe execution path (including the `enrich` UDF dispatch and `stats`/
`dedup`/`fields` pipeline) with zero new execution logic.

**D4 — Constraint: SQL head LIMIT and any row-capping pipe stage are mutually exclusive
(HRG-1 FORBID-BOTH ruling, 2026-06-24).** A composed query MAY NOT specify both a SQL
`LIMIT N` clause in the head AND a row-capping pipe stage in the tail. The row-capping
pipe-stage family is: `| limit M` or `| tail M` (both cap the result row count at the
pipe level). Doing so produces a plan-time pedagogical error:

```
E-QUERY-040: redundant row limit. This query caps rows in two places: a SQL `LIMIT N` in
the head and a `| limit M` (or `| tail M`) pipe stage. PrismQL requires exactly one row
cap. Remove the SQL `LIMIT N` and place a single `| limit` at the end of the pipeline
(recommended for composed queries), or use `LIMIT` only in pure SQL-mode queries.
```

This is a **plan-time rejection** (after parse, during logical planning), not a
parse-time error — the grammar accepts the construct so the error can be semantic and
specific rather than a generic syntax error. The error code E-QUERY-040 is allocated here;
the full error-taxonomy row (MCP mapping, `StructuredErrorFields.normalized_pql`, and BC
postcondition) is authored by the product-owner. The canonical form for a composed query
with a row cap is a single tail `| limit`, e.g. `SELECT cols FROM t | enrich fn(c) | limit N`.

**Rationale for FORBID-BOTH over pipe-wins:** Every cross-language precedent (GoogleSQL
pipe syntax, PRQL, KQL, Splunk SPL, Spark) structurally avoids placing two caps at the
same syntactic level — they compose caps across subquery boundaries where nesting makes
intent unambiguous. Prism's composed form deliberately places a SQL-clause `LIMIT` and a
row-capping pipe stage (`| limit` or `| tail`) at the same level with no nesting, which
is exactly the ambiguous construct
mature languages avoid by design. For a pedagogical DSL consumed by AI agents, a clear
plan-time error trains the correct idiom; silent composition (pipe-wins) trains agents
that redundant caps are acceptable. FORBID-BOTH is also the reversible choice:
forbid→permit is non-breaking; permit→forbid would be a breaking change.

The only acceptable future relaxation is **pipe-wins** (option b) — SQL head `LIMIT`
becomes an intermediate cap; pipe `| limit` is the final cap. This relaxation is
non-breaking and may be adopted in a later cycle if operational evidence shows the
pedagogical error is overly restrictive. **SQL-wins has zero precedent and contradicts
the pipe-operator design ethos; it must never be adopted.**

**D5 — Mode detection update:** `is_pipe_mode` in `filter_parser.rs` currently returns
`true` only if the query does NOT start with `SELECT` AND contains `|` + pipe-keyword. It
is extended to also return a new `SqlPipeMode` enum variant when the query starts with
`SELECT` AND contains `|` + pipe-keyword. The top-level `parse_with_limits` dispatches on
this tristate: `FilterMode` → `parse_filter`, `PipeMode` → `parse_pipe`, `SqlPipeMode`
→ `parse_sql_pipe`. (The current bool return is replaced with a `QueryMode` enum.)

**D6 — Existing SQL mode and Pipe mode are UNCHANGED.** Pure `SELECT … FROM … WHERE …
LIMIT` queries (no `|`) continue to parse and execute as `Ast::Sql`. Pure `FROM … |
stage` queries continue as `Ast::Pipe`. This change is strictly additive.

**D7 — Mode-bridge error on `|` in pure SQL mode.** Even with composition, a `SELECT …
FROM t` query that hits a bare `|` not followed by a pipe stage keyword (e.g.,
`SELECT a | b FROM t`) is still a SQL-mode error. The mode-bridge diagnostic from
ADR-043's companion error-improvement work (see ADR-044) fires here: "found `|` in SQL
mode — did you mean to use pipe stages? Pipe stages require `| stage_keyword`. See
`prismql://reference` for pipe mode syntax."

---

## Rationale

True SQL→pipe composition is the correct answer because:

1. **The reference already documents it as true.** `prismql://reference` line 137 shows
   `SELECT * FROM t | WHERE … | ORDER BY time DESC | LIMIT 10`. The reference is the
   contract between Prism and its analysts. Implementing the grammar that matches the
   already-documented contract is the production-grade path. Option 1 (mode-bridge only)
   leaves the reference a lie and requires a doc retraction.

2. **It eliminates the entire class of mode-mixing confusion.** The grammar audit
   (GRAMMAR-001, GRAMMAR-009, GRAMMAR-014) traces all mode-mixing issues to the hard
   boundary between SQL and pipe. True composition converts that boundary into a smooth
   on-ramp: SQL-literate analysts start with `SELECT … FROM t`, add `| enrich fn(col)`,
   and it works. No mode switch required.

3. **The execution architecture is already set up for it.** `pipe_sql_emitter.rs` emits
   Pipe→SQL; the converse SQL head→pipe execution reuses the exact same pipe stage
   execution infrastructure. There is no new execution substrate to build. The risk is
   in the parser extension (bounded to `filter_parser.rs` + new `parse_sql_pipe` combinator
   and `SqlPipeQuery` AST node), not in execution.

4. **It is the form analysts will reach for first.** Security analysts are SQL-literate
   (CrowdStrike Query Language, Splunk SPL, KQL all have SQL-like `SELECT/FROM`
   constructs). The intuitive enrichment query is `SELECT … FROM t | enrich fn(col)`.
   This is what they will type. Option 1 forces them to rewrite to `FROM t | enrich fn(col)`
   — a mode-switch that requires knowing Pipe mode exists, which is exactly the discovery
   gap the audit identified.

5. **BC-2.11.003 and BC-2.11.004 remain valid.** The `SqlQuery` and `PipeQuery` AST
   nodes are unchanged. The new `SqlPipeQuery` composes them at the AST level. Existing
   BCs for SQL mode queries and pipe mode queries are not violated; a new BC for the
   `SqlPipe` mode covers the composed form.

---

## Consequences

### Positive

- `SELECT … FROM t | enrich fn(col) | limit N` parses and executes correctly — the
  reference's documented example becomes true.
- The mode-mixing BLOCKER (GRAMMAR-001) is resolved at its root, not papered over.
- Analysts learn one unified mental model: SQL head optionally followed by pipe stages.
- The CI reference-round-trip gate (GRAMMAR-017) can now verify the reference's own
  examples parse correctly — a regression guard that was impossible before.

### Negative / Trade-offs

- Parser complexity increases: `is_pipe_mode` becomes a tristate `QueryMode` enum;
  `parse_with_limits` grows a third dispatch branch; a new `parse_sql_pipe` combinator is
  added to `filter_parser.rs`.
- The `Ast` enum gains a new variant `SqlPipe(SqlPipeQuery)`, requiring a new match arm
  in `engine.rs` (execution) and any other `Ast` match sites in `prism-query` — these are
  small, bounded changes given the `#[non_exhaustive]` attribute already enforces wildcard
  arms on external consumers.
- D4 (FORBID-BOTH — SQL `LIMIT` + row-capping pipe stage family) requires an E-QUERY-040
  plan-time rejection path and negative-gate tests in the CI round-trip suite (assert
  both `| limit` and `| tail` forms combined with a head `LIMIT N` fail with E-QUERY-040).
  The full error-taxonomy row is product-owner scope.
- A new BC for `Ast::SqlPipe` is required (product-owner deliverable).
- This is a grammar extension with security implications: the new `parse_sql_pipe` path
  must be covered by the same security checks as `parse_sql` and `parse_pipe` (size
  limits, depth limits, injection defense via `scan_inputs_audited`).

### Status as of v1.2 (2026-06-25)

ACCEPTED. HRG-1 ratified FORBID-BOTH (2026-06-24); HRG-2 confirmed Option 2 over
Option 1. Not yet implemented. The existing `Ast::Sql` and `Ast::Pipe` paths are
unaffected at current HEAD.

---

## Alternatives Considered

- **Option 1 — Mode-bridge error only (minimum viable fix):** Emit a pedagogical error
  when SQL mode hits `|`, with a `normalized_pql` rewrite to the equivalent pipe form.
  Rejected because: (a) the reference already documents the composed form as true —
  Option 1 requires retracting that documentation; (b) it puts the cognitive burden on
  the analyst to manually rewrite their intuitive query; (c) it leaves the mode-split as
  a permanent foot-gun. Option 1's mode-bridge error is still produced by ADR-046 as a
  complementary measure for the residual case where `|` appears in a non-composition
  context.

- **Option 3 — Deprecate SQL mode, make Pipe mode the only mode:** Remove `Ast::Sql`
  and `parse_sql` entirely; require all queries to use `FROM … |` form. Rejected because:
  (a) SQL mode is already implemented, tested, and works well for simple read queries;
  (b) forcing a mass rewrite of demo scripts, prompts, and test fixtures adds churn with
  zero new capability; (c) SQL-literate analysts find `SELECT … FROM …` more readable
  for simple projections; (d) breaking change with high blast radius.

- **D4 alternative: pipe-wins (SQL head LIMIT = intermediate cap, pipe `| limit` = final
  cap, both silently composed):** This is the behavior observed across GoogleSQL pipe
  syntax, PRQL, KQL, Splunk SPL, and Spark — all model caps as compositional unary
  operators. It was the architect's original recommendation (v1.0) and was demoted by
  HRG-1 human ratification. It remains the **only acceptable future relaxation** of the
  FORBID-BOTH ruling: if operational evidence shows FORBID-BOTH is overly restrictive,
  pipe-wins may be adopted without breaking existing valid queries (forbid→permit is
  non-breaking). No other LIMIT-precedence alternative is acceptable — SQL-wins has zero
  precedent in any researched language and is permanently ruled out.

---

## Human Ratification Gate — CLOSED (2026-06-24)

Both gates ratified by the human on 2026-06-24.

**HRG-1 — SQL head LIMIT + row-capping pipe stage interaction (D4): RATIFIED — FORBID-BOTH.**
A composed query MAY NOT specify both a SQL `LIMIT` clause in the head and a row-capping
pipe stage (`| limit M` or `| tail M`) in the tail; doing so is a plan-time pedagogical
error (E-QUERY-040). This overrides the architect's original recommendation (pipe-wins).
See D4 and §Alternatives Considered for the complete rationale. Research basis:
`.factory/research/prismql-composition-and-reference-research-2026-06-24.md` (HRG-1
analysis; GoogleSQL pipe syntax / PRQL / Kusto / Splunk / Spark cross-language verdict).

**HRG-2 — Confirmation of Option 2 over Option 1: RATIFIED — Option 2 confirmed.**
The human directed full SQL→pipe composition (Option 2) during the audit review session.
Formalized here for traceability. Implementation may proceed.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.2 | 2026-06-25 | architect | PR-LEVEL cascade fix burst closing F-P2-HIGH-001 + F-P2-LOW-001. **(F-P2-HIGH-001)** D4 FORBID-BOTH scope extended from `\| limit M` only to the full row-capping pipe-stage family: `\| limit M` or `\| tail M`. D4 heading, error message, rationale, consequences, and HRG-1 ratification text all updated to name the family. The rule's behavioral intent is unchanged (one row cap per composed query); this is a precision fix ensuring `\| tail` is explicitly covered alongside `\| limit`, matching the PR #203 HIGH-1 implementation and error-taxonomy.md v2.00. **(F-P2-LOW-001)** Replaced stale `ParseErrorDetails` reference in D4 consequence sentence with `StructuredErrorFields.normalized_pql` — there is no `ParseErrorDetails` type (D-1110); the MCP-facing structured error payload is `StructuredErrorFields` in `prism-mcp/src/error_mapping.rs` (BC-2.11.023 v1.2 corrected this in the BC; ADR-043's copy was the last remaining stale citation per POL-25 multi-cite sweep). |
| v1.1 | 2026-06-24 | architect | HRG ratification burst. HRG-1: FORBID-BOTH ruling recorded; D4 rewritten from pipe-wins to plan-time E-QUERY-040 rejection; pipe-wins demoted to Considered Alternatives as only acceptable future relaxation. HRG-2: Option 2 confirmed. Status PROPOSED→ACCEPTED. Research basis: `.factory/research/prismql-composition-and-reference-research-2026-06-24.md`. |
| v1.0 | 2026-06-24 | architect | Initial draft. PROPOSED. HRG-1 and HRG-2 pending human ratification. |

---

## Source / Origin

- Grammar usability audit: `.factory/research/prismql-grammar-usability-audit-2026-06-24.md`
  §GRAMMAR-001, §GRAMMAR-009, §GRAMMAR-014, §4 Recommendation.
- Pre-flight audit: `.factory/research/demo-pre-flight-audit-2026-06-24.md` BLOCKER-002,
  §2.5 enrichment, §1.4 three modes.
- ADR-041 §L1 primer (existing four-layer teaching surface; composition makes the L1
  primer truthful).
- `crates/prism-query/src/filter_parser.rs` — `is_pipe_mode`, `parse_with_limits` (mode
  detection entry point).
- `crates/prism-query/src/ast.rs` — `Ast` enum, `SqlQuery`, `PipeQuery`, `PipeStage`.
- `crates/prism-query/src/pipe_sql_emitter.rs` — existing Pipe→SQL lowering (the converse
  execution pattern this ADR mirrors).
- `crates/prism-mcp/src/pql_reference.md` line 137 — the broken `SELECT … | WHERE …`
  reference example that this ADR makes true.
