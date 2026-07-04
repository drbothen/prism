---
document_type: adr
adr_id: "ADR-044"
title: "Temporal Grammar — NOW() and INTERVAL/Relative-Duration Literals in PrismQL"
status: proposed
date: "2026-06-24"
version: "1.0"
producer: architect
subsystems_affected: [SS-11]
supersedes: null
superseded_by: "ADR-052 (§D4 only — planning-time injection as ISO-8601 string; §D1–D3, §D5–D7 remain valid)"
amends: null
anchor_stories: []
related_adrs: [ADR-003, ADR-033, ADR-041, ADR-043]
related_bcs: [BC-2.11.003, BC-2.11.004, BC-2.11.001]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-044: Temporal Grammar — `NOW()` and `INTERVAL`/Relative-Duration Literals in PrismQL

## Status

PROPOSED v1.0 (2026-06-24). Architect decision following grammar usability audit
`.factory/research/prismql-grammar-usability-audit-2026-06-24.md` (GRAMMAR-011) and
human direction to implement `NOW()` + relative-duration, not delete the documented
syntax. Human ratification not required — the human explicitly chose implementation.

**PARTIALLY SUPERSEDED by ADR-052 v1.1 (2026-07-03) — §D4 only.**

§D4 (planning-time constant injection as ISO-8601 string comparison) is superseded
by **ADR-052 v1.1**. After ADR-052: sensor `datetime` columns register as
`Timestamp(Microsecond, Some("UTC"))` rather than `Utf8`; `Literal::Timestamp`
emission changes from a bare quoted string (`'...'`) to an explicit
`arrow_cast('<rfc3339>', 'Timestamp(Microsecond, Some("UTC"))')` call (DataFusion
53.1.0 verified: `TIMESTAMP '...'` produces `Timestamp(Nanosecond, None)`, requiring
the explicit form); and string literals compared against datetime columns are
pre-validated at plan time by `chrono::DateTime::parse_from_rfc3339` (strict RFC-3339,
not arrow-cast's lenient form) with E-QUERY-041 raised on rejection.

§D1 (`Expr::Now` variant), §D2 (`INTERVAL` syntax), §D3 (`TimestampArithmetic` AST),
§D5 (`build_example_query` fix), §D6 (reference accuracy), and §D7 (SQL mode + Pipe
mode both support `NOW()`) remain **fully valid and unchanged**.

---

## Context

The PrismQL reference (`prismql://reference` §Datetime Arithmetic) and the `prism_describe`
tool's `build_example_query` function both document and generate queries using
`NOW() - INTERVAL '24h'` syntax. The grammar audit confirmed that this syntax does NOT
exist anywhere in the parser (`grep` across `sql_parser.rs`, `filter_parser.rs`,
`ast.rs` → zero matches for `NOW` or `INTERVAL`). Every query that uses this form
produces PARSE_ERR.

This means:
- The machine-generated per-table example query (the primary "here's how to query this"
  signal every analyst sees on `prism_describe`) is invalid syntax.
- The reference's Datetime Arithmetic section is entirely fictional.
- Every analyst or LLM agent who follows the documented time-filter pattern will fail.

The audit identified two resolution paths:
1. **Implement `NOW()` + relative-duration** (Path 1) — the production-grade path.
2. **Delete the docs + fix `build_example_query`** (Path 2) — the retraction path.

The human directed Path 1. This ADR defines the grammar extension.

The existing grammar already has `Duration` literals (`30s`, `24h`, `7d` — these ARE
implemented in `ast.rs` as `Literal::Duration` and are used by `pipe_parser.rs`). The
`TimestampLiteral` (strict RFC-3339 with required offset) is also implemented. `NOW()` is
a new addition; `INTERVAL 'Nh'` is a new syntax form that complements the already-present
bare duration literal.

---

## Decision

**D1 — `NOW()` as a new zero-argument function call in the expression grammar.** The
`Expr` type in `ast.rs` gains a new variant: `Expr::Now`. This is parsed by the existing
expression parser (used by both SQL mode and Pipe mode) when it encounters the token
sequence `NOW` `(` `)` (case-insensitive). No arguments are accepted; `NOW(arg)` is a
parse error ("NOW() takes no arguments"). Execution: `Expr::Now` evaluates to the current
UTC timestamp at query-plan time (not at execution time), consistent with the existing
ephemeral-query model. The timestamp is a `DateTime<Utc>` injected by the executor as a
planning-time constant.

**D2 — `INTERVAL 'Nh'` as a new duration expression.** The `Expr` type gains a new
variant: `Expr::Interval(Duration)`. The `Duration` inner type reuses the existing
`ast::Literal::Duration` value (which already represents `30s`, `24h`, `7d`, `7w`).
The INTERVAL keyword accepts either the SQL string form `INTERVAL '24h'` or the bare
duration literal form `INTERVAL 24h`. Both forms are accepted by the parser and produce
identical AST nodes. This is documented in the reference with both forms shown.

**D3 — `NOW() - INTERVAL 'Nh'` / `NOW() - 24h` as a `TimestampArithmetic` expression.**
The expression grammar gains an arithmetic rule: `NOW() - duration_expr` (where
`duration_expr` is either `INTERVAL '...'` or a bare `Duration` literal) produces
`Expr::TimestampArithmetic { base: Box<Expr::Now>, op: Sub, offset: Duration }`. This is
the only supported temporal arithmetic form in v1 — `NOW() + duration` is a parse error
("timestamp arithmetic only supports subtraction: use `NOW() - INTERVAL 'Nh'`").

**D4 — Execution semantics: planning-time constant injection.** `Expr::Now` is evaluated
at query-plan time (when the executor calls `plan_query`), not lazily during DataFusion
execution. The resulting `DateTime<Utc>` is substituted as a `Literal::Timestamp`
constant in the logical plan before DataFusion receives it. This is consistent with the
ephemeral query model (a single `prism start` session does not have a long-lived server
that re-evaluates `NOW()` on each row) and eliminates DataFusion UDF registration
complexity. The `plan_query` step already receives query context (including the execution
timestamp) that can carry the `now` value.

**D5 — `build_example_query` fix.** After this ADR lands, the `NOW() - INTERVAL '1h'`
example in `build_example_query` (`prism_describe.rs` line 450) becomes valid syntax.
No change to `build_example_query` is needed (it already generates the right form; this
ADR makes the parser accept it).

**D6 — Reference Datetime Arithmetic section becomes accurate.** The existing section in
`pql_reference.md` already shows the correct forms. No reference doc changes are needed
for the Datetime Arithmetic section (the doc is ahead of the implementation; this ADR
closes the gap). The reference update story for GRAMMAR-011 (doc bucket) is reduced to:
(a) verifying the reference examples round-trip through the new parser (the CI gate from
GRAMMAR-017), and (b) adding the bare duration literal alternative (`NOW() - 24h`
alongside `NOW() - INTERVAL '24h'`).

**D7 — Scope: SQL mode and Pipe mode both support `NOW()`.** The expression parser is
shared between SQL mode and Pipe mode (both call `build_predicate_parser` which calls
`build_expr_parser`). `Expr::Now` and `Expr::Interval` are expression-level additions
and therefore available in both modes without separate parser changes.

---

## Rationale

1. **The grammar already has `Duration` literals.** Implementing `NOW()` + `INTERVAL` is
   incremental work on an existing foundation, not a new feature. The `Duration` AST type
   exists; the INTERVAL production is a parser rule that wraps it; `NOW()` is a zero-arg
   function. This is bounded, low-risk grammar work.

2. **The alternative (deletion) is deeply unproductive.** Path 2 requires: (a) deleting
   the Datetime Arithmetic section from the reference, (b) changing `build_example_query`
   to not show time-filtered examples, (c) accepting that Prism has no time-filter syntax
   for the most common security analyst pattern ("show me detections from the last 24h").
   This is worse than the status quo and contradicts the production-grade default.

3. **`NOW() - INTERVAL '24h'` is the single most natural security analyst time filter.**
   CrowdStrike Query Language, Splunk SPL, and KQL all support relative-time operators.
   An analyst from any of these platforms will reach for this form reflexively. The grammar
   audit identified it as the zero-implementation-with-high-documentation cost (GRAMMAR-011).

4. **Planning-time constant injection (D4) is the correct execution model.** PrismQL
   queries are ephemeral (each `query` tool call spawns a fresh DataFusion `SessionContext`
   per ADR-002). Evaluating `NOW()` at plan time and injecting it as a `Literal::Timestamp`
   constant is exact, deterministic, and works with the existing DataFusion SQL execution
   path without UDF registration. The `SessionContext::sql()` path used by the pipe
   execution engine (`scoping/pipe-execution-engine-design.md`) receives SQL with the
   constant already substituted, so DataFusion sees `WHERE timestamp > '2026-06-24T00:00:00Z'`
   and applies it as a normal timestamp comparison.

5. **`ADR-033` (push-down time-window extraction) already handles timestamp predicates.**
   The pre-fan-out heuristic (T1) in `pushdown.rs` extracts time-window predicates from
   the WHERE clause to pass as `start_time`/`end_time` range hints to sensor adapters.
   Once `NOW() - INTERVAL '24h'` is lowered to a `Literal::Timestamp` constant at plan
   time, ADR-033's T1 extractor will recognise it as a timestamp comparison and extract
   it as a time-window hint — no changes to `pushdown.rs` required.

---

## Consequences

### Positive

- `build_example_query` generates valid PQL — the primary per-table discovery hint stops
  being a hallucination.
- The reference Datetime Arithmetic section becomes accurate.
- Security analysts can express the most common time filter naturally.
- ADR-033 push-down gains `NOW()` queries automatically once they are lowered to constants.

### Negative / Trade-offs

- `ast.rs` grows two new `Expr` variants (`Now`, `Interval`, `TimestampArithmetic`).
  These are expression-level additions; the `#[non_exhaustive]` attribute on `Expr` means
  downstream crates already have wildcard arms.
- The `plan_query` step must inject the `now` constant. If the executor path does not
  currently thread a `now` timestamp through to the expression evaluator, a small wiring
  change is needed. The implementer must audit the plan-to-execution path to find the
  right injection point.
- `NOW() + duration` (future offset) is intentionally NOT supported in v1. If analysts
  need it, a follow-up story adds it. This constraint must be documented in the reference.

### Status as of v1.0 (2026-06-24)

PROPOSED. Implementation gated on this ADR's acceptance by the human. No `NOW()`
support exists in the grammar at current HEAD (`develop@acc6722c`).

---

## Alternatives Considered

- **Path 2 — Delete the Datetime Arithmetic section and fix `build_example_query` to
  emit only parseable forms (absolute RFC-3339 timestamps or no time filter):** Rejected
  because: (a) it regresses the product — no time-filter syntax is a major usability gap;
  (b) it makes `prism_describe` less useful (time-filtered examples are the most useful
  examples for security workflows); (c) the `Duration` literal type already exists, making
  Path 1 low-cost.

- **Server-side UDF for `NOW()` in DataFusion:** Register `NOW` as a DataFusion scalar
  UDF that returns the current UTC timestamp at execution time. Rejected because: (a) the
  ephemeral-session model means each query gets a fresh `SessionContext` anyway, so
  planning-time and execution-time `NOW()` evaluations are identical in practice; (b) UDF
  registration adds complexity and a DataFusion dependency surface; (c) constant-injection
  at plan time is simpler, more testable (the constant is visible in the logical plan), and
  compatible with ADR-033's T1 push-down without additional changes.

---

## Source / Origin

- Grammar usability audit: `.factory/research/prismql-grammar-usability-audit-2026-06-24.md`
  §GRAMMAR-011, §3 Bucket 1 (fix-grammar/code).
- `crates/prism-query/src/ast.rs` — existing `Literal::Duration`, `TimestampLiteral`.
- `crates/prism-mcp/src/pql_reference.md` §Datetime Arithmetic (lines 77-96) — doc that
  this ADR makes truthful.
- `crates/prism-mcp/src/tools/prism_describe.rs` line 450 — `build_example_query`
  generating the broken `NOW() - INTERVAL '1h'` form.
- `crates/prism-query/src/pushdown.rs` — ADR-033 T1 push-down (will benefit automatically).
