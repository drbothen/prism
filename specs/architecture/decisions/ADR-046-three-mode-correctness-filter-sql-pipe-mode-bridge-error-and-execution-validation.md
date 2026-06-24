---
document_type: adr
adr_id: "ADR-046"
title: "Three-Mode Correctness — Filter / SQL / Pipe Mode-Bridge Error and Execution Validation"
status: accepted
date: "2026-06-24"
version: "1.0"
producer: architect
subsystems_affected: [SS-11, SS-10]
supersedes: null
superseded_by: null
amends: ADR-041
anchor_stories: []
related_adrs: [ADR-041, ADR-043, ADR-044, ADR-045]
related_bcs: [BC-2.11.002, BC-2.11.003, BC-2.11.004, BC-2.11.001]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-046: Three-Mode Correctness — Filter / SQL / Pipe Mode-Bridge Error and Execution Validation

## Status

ACCEPTED v1.2 (2026-06-24). Architect decision following grammar usability audit
(GRAMMAR-001, GRAMMAR-014, GRAMMAR-015, GRAMMAR-016) and pre-flight audit
(BLOCKER-002, BLOCKER-003). Amends ADR-041 by specifying the mode-bridge error
architecture and Filter mode execution validation requirement. Human ratifications
recorded 2026-06-24: HRG-4/MAJOR-001 ruling is Path B — `list_capabilities` consults
the authoritative `OrgRegistry` for org existence; D7 mode composition boundary ruling —
Filter is bare-predicate sugar, only SQL→Pipe composes, pipe is the canonical execution
model (see §Decision D7 and §Changelog).

---

## Context

PrismQL has three parse modes: Filter, SQL, and Pipe. As of current HEAD:

- **Filter mode** (`severity='HIGH'`) — parses correctly (PARSE_OK confirmed by live
  probe), but end-to-end execution is UNVALIDATED and it is UNDOCUMENTED in
  `prismql://reference`. No TDD tests exercise the full Filter→execution path.
- **SQL mode** (`SELECT … FROM … WHERE … LIMIT`) — parses and executes correctly;
  documented; tested.
- **Pipe mode** (`FROM … | where … | enrich fn(col) | limit N`) — parses and executes
  correctly after the enrichment pivot (PR #200 merged); partially documented (enrichment
  absent, being added by 001-C).

**Mode-mixing errors are the most critical usability failures.** GRAMMAR-001 and
GRAMMAR-014 describe the same root: when a user writes `SELECT … FROM t | limit N`, SQL
mode's parser encounters `|` and emits a Chumsky token-expectation dump that is useless
("found `|` expected `AS`, `as`, `WHERE`, `where`, … or end of input"). The error never
says the two true facts: (a) `|` is not valid in SQL mode, (b) to add pipe stages, start
with `FROM` or switch to the composed form (ADR-043).

**Symmetric issue:** when a user writes `FROM t | WHERE severity = 'HIGH' | limit 3`
using the SQL-clause form of `WHERE` (uppercase, expected to be a clause rather than a
stage), the pipe parser actually ACCEPTS it because `WHERE` is case-insensitive and
`where` is a pipe stage keyword. But this creates a subtler problem: the query parses
but the user's mental model ("uppercase WHERE is a SQL clause, lowercase `| where` is a
pipe stage") is not enforced. The reference should document case-insensitivity explicitly
(GRAMMAR-018).

**Filter mode execution gap:** The grammar audit found that Filter mode parses correctly
(`severity = 'HIGH'` → PARSE_OK as `Ast::Filter`) but the execution path in `engine.rs`
for `Ast::Filter` is not covered by integration tests that verify actual row results are
returned. The BC for filter mode (BC-2.11.002) requires execution, not just parsing.

**BLOCKER-003 (prompts hang):** `query_tutorial` and `investigate_host` prompts hang
indefinitely. These are synchronous render functions (`render_query_tutorial`,
`render_investigate_host` in `prompts.rs`) that return immediately. The hang is therefore
in the `PromptRouter` dispatch layer or the `#[prompt_handler]` macro expansion, not in
the render functions themselves. The architect's root-cause hypothesis: the rmcp 1.7
`PromptRouter::new_dyn` closure captures the argument map by reference, and the dispatch
machinery may await on something that blocks when required arguments (`hostname` for
`investigate_host`, or both `client_id` + optional `goal` for `query_tutorial`) are not
present in a specific pattern. This requires implementer investigation of the
`#[prompt_handler]` macro expansion before a fix can be specified.

---

## Decision

**D1 — Mode-bridge error diagnostic on `|` in SQL mode.** The SQL mode parse failure
handler in `error_recovery.rs` is extended with a post-parse heuristic: if a SQL-mode
parse fails at a `|` token, the generic Chumsky expectation dump is replaced with a
structured mode-bridge error:

```
E-QUERY-001: parse error near '|': pipe stages are not valid after a SQL SELECT query.
To use pipe stages (enrich, where, limit, sort, stats, dedup, fields), start with:
  FROM <table> | where <predicate> | <stage> …
Or use the SQL+pipe composition form (if SELECT is needed):
  SELECT <cols> FROM <table> | <pipe_stage> …
See prismql://reference for the complete grammar.
normalized_pql: "FROM <table> | where <extracted_predicate> | limit <N>"
```

The `normalized_pql` field contains a best-effort rewrite of the user's SQL query into
pipe mode: extract the `FROM` table, the `WHERE` predicate (if present), and the `LIMIT`
(if present), and emit the pipe equivalent. This rewrite is best-effort and is clearly
labeled "suggested rewrite" — it is not guaranteed to be semantically identical
(e.g., if the SQL query had JOINs that have no pipe equivalent). If rewrite is not
possible, `normalized_pql` is omitted.

This diagnostic is implemented in `error_recovery::rich_to_parse_error` by adding a
post-processing step that checks whether the raw Chumsky error occurred at a `|` token
and the query started with `SELECT`.

**D2 — Symmetric mode-bridge error on SQL clause in pipe stage position.** If a pipe-mode
parse fails because an uppercase SQL clause keyword (`SELECT`, `ORDER BY`) appears in
stage position (not preceded by `|`), the error message is:
```
E-QUERY-001: parse error near '<keyword>': SQL clauses are not valid as pipe stages.
In pipe mode, use lowercase stage keywords: 'where', 'sort', 'limit', 'stats'.
Example: FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10
```

Note: `WHERE` and `LIMIT` actually parse in pipe mode because they are case-insensitive
and the parser accepts them as `where`/`limit` stages. D2 therefore only fires for
`SELECT` in pipe stage position (not preceded by `|`) and `ORDER BY` (which is not a
pipe stage keyword and causes a genuine parse error).

**D3 — `normalized_pql` field on ALL E-QUERY-001 errors where a rewrite is available.**
ADR-041 §L4 and GRAMMAR-016 note that `normalized_pql` is documented as an error field
but is absent in practice. This ADR mandates: the mode-bridge error (D1) MUST include
`normalized_pql` when a rewrite is possible. The existing E-QUERY-001 structured error
type (`ParseErrorDetails`) is extended with an `Option<String> normalized_pql` field.
The error-taxonomy row for E-QUERY-001 must be updated by the product-owner to document
this field.

**D4 — Filter mode end-to-end execution validation requirement.** The implementer story
for three-mode correctness MUST include at least two integration tests that exercise the
full `Ast::Filter → execute → rows returned` path:
- `test_filter_mode_simple_predicate`: `severity='HIGH'` returns rows matching the
  predicate from a mocked/DTU sensor source.
- `test_filter_mode_with_source`: `crowdstrike_detections | severity='HIGH'` (source-
  qualified filter) returns rows.

These tests must use the same `QueryEngine::execute` path as SQL and pipe mode tests —
not just `PrismQlParser::parse`. Until these tests exist, Filter mode execution is
UNVERIFIED even though parsing works.

**D5 — Case-insensitivity documentation (GRAMMAR-018).** The `prismql://reference`
reference (assembled per ADR-045) MUST include a "Case Sensitivity" note stating: all
PrismQL keywords are case-insensitive. House style: UPPER for SQL mode keywords, lower
for pipe stage names. This is a documentation-only change (no parser changes).

**D6 — `query_tutorial` and `investigate_host` prompt hang investigation.** The
implementer assigned to BLOCKER-003 MUST:
(a) Inspect the `#[prompt_handler]` macro expansion (cargo expand or debug tracing) to
   identify the blocking point.
(b) Determine whether the hang is in `PromptRouter::new_dyn` argument parsing, in the
   `PromptRoute` closure awaiting, or in the rmcp dispatch loop.
(c) The hypothesis (see Context above) is that the issue is NOT in the render functions
   (which are synchronous) but in the rmcp 1.7 `PromptRouter` dispatch when certain
   argument combinations are present. The fix is likely in the route registration or in
   how `PromptRoute::new_dyn` handles the async closure.

The architect does NOT specify the fix for BLOCKER-003 — it requires implementer
investigation. The BC for `query_tutorial` is BC-2.10.009 §query_tutorial; the
implementer must verify the fix makes that BC's postcondition true.

**D7 — Mode Composition Boundary: Filter is sugar; only SQL→Pipe composes.**

The three parse modes are NOT equivalent in syntactic scope — they have a strict
expressiveness hierarchy, and only ONE syntactic composition path is supported:

1. **Filter mode is bare-predicate sugar.** Its expressive power is a strict subset of
   SQL and Pipe modes. A filter-mode predicate (`severity = 'HIGH'`) is semantically
   identical to a SQL `WHERE p` or a pipe `| where p`. Filter mode adds no capability
   that either of the other two modes lacks.

2. **The ONLY supported syntactic composition is SQL→Pipe** (per ADR-043 D1–D6). There
   is NO three-way composition (filter + SQL + pipe). Concatenating a bare filter
   predicate onto a `SELECT … FROM … | …` composed query is rejected at plan time:
   - If the predicate appears as a trailing bare term, it is either redundant
     (semantically identical to the WHERE/`| where` already present) or has no defined
     meaning adjacent to SELECT/FROM. Neither case should silently succeed.
   - The plan-time rejection message should redirect to the supported forms.

3. **Canonical mental model:** Pipe is the single underlying EXECUTION model. SQL and
   Filter are two sugar ENTRY syntaxes that lower into it. Filter mode lowers to the
   same predicate AST used by `WHERE` / `| where`. The predicate grammar is SHARED
   across all three modes — one predicate parser, three entry points. A predicate that
   parses in Filter mode parses identically in SQL `WHERE` and pipe `| where`.

4. **Requirements that follow from this model:**
   - One shared predicate grammar consumed by Filter, SQL, and Pipe modes.
   - A clean "graduation" path: a filter-mode query (`severity = 'HIGH'`) can be
     escalated to a pipe query by adding `FROM <table> | where <predicate> | …` — the
     predicate grammar is unchanged, only the entry syntax changes.
   - The mode-bridge error (D1) fires on a half-graduated query (e.g., a bare predicate
     adjacent to `SELECT` without a `|` prefix), directing the user to the correct form.

**Rationale:** Filter mode contributes nothing that SQL or Pipe cannot express. Collapsing
the conceptual model to "pipe is canonical execution; SQL and filter are syntactic sugar
that lower into it" kills the mode-mixing foot-gun at its root — analysts learn ONE
execution model with two alternative entry syntaxes rather than three separate languages
with different rules. The shared predicate grammar enforces this at the parser level.

**MAJOR-001 ruling — Path B: `list_capabilities` consults `OrgRegistry` (HRG-4,
2026-06-24).** The `client_exists()` check in `feature_flag.rs` is modified to consult
the authoritative `OrgRegistry` (populated from spec overlays per ADR-006/ADR-029) for
basic org existence, independently of `prism.toml` `[clients.*.capabilities]`
write-capability config entries. This decouples org-registration discovery from
write-capability provisioning, which is the correct semantic per ADR-006 (org existence
is determined by spec overlay registration, not by write-capability config).

**Path A (add `[clients.org-*]` entries to `prism.toml` or `demo-setup.sh`) is
explicitly rejected.** Path A would hard-code the client registration check to a
config-file side-channel that is not the authoritative source of truth for org
registration. It is a demo-expedient workaround that would paper over the architectural
misalignment and require re-visiting the same issue whenever a client is provisioned via
spec overlays without explicit `prism.toml` entries. Under the production-grade default,
the correct mechanism is implemented now.

**Implementation note:** The `FeatureFlagEvaluator` currently holds the
`client_capabilities` map (populated from `prism.toml` at construction). It needs access
to the `OrgRegistry` to implement Path B. The wiring is:
`FeatureFlagEvaluator::new(client_capabilities, org_registry: Arc<OrgRegistry>)` —
adding the `OrgRegistry` dependency follows the Arc-DI wiring contract (ADR-022 §C).

---

## Rationale

1. **Mode-bridge errors are the highest-leverage UX improvement with the lowest grammar
   change cost.** D1 and D2 require only changes to `error_recovery.rs`, not to any
   Chumsky grammar production. The grammar audit found this is the first thing analysts
   need when they cross the mode boundary — the current error is affirmatively unhelpful.

2. **`normalized_pql` was planned in ADR-041 but never implemented (GRAMMAR-016).** D3
   closes this gap. The field was documented as an error field; the error-taxonomy row
   for E-QUERY-001 references it; it needs to actually appear in the response.

3. **Filter mode is a BC-2.11.002 contract that must be execution-tested.** The BC
   specifies "Filter mode predicates are applied to the sensor data source." Parse-only
   testing does not satisfy this contract. D4 mandates the minimum tests that verify
   execution. This is a BC compliance gap, not a polish item.

4. **BLOCKER-003 is almost certainly a framework-level issue in rmcp 1.7.** The render
   functions themselves are pure synchronous functions that return immediately (verified
   by reading `render_query_tutorial` and `render_investigate_host` in `prompts.rs`).
   The hang MUST be in the dispatch machinery. This is the correct architectural
   diagnosis — the implementation must be verified by the implementer, not presumed.

---

## Consequences

### Positive

- Mode-mixing errors become the most helpful class of PQL errors, not the least.
- Filter mode becomes a first-class verified mode, not a parse-only curiosity.
- `normalized_pql` appears in errors for the first time, enabling agent self-correction.
- BLOCKER-003 is unblocked after investigation.

### Negative / Trade-offs

- `ParseErrorDetails` gains a new optional field `normalized_pql`; any consumers of the
  structured error response must handle it gracefully (additive, non-breaking).
- The mode-bridge rewrite (D1 `normalized_pql`) is best-effort and may produce incorrect
  rewrites for complex queries (JOINs, subqueries). The implementation must be conservative:
  only emit `normalized_pql` when the rewrite is unambiguous (simple `FROM/WHERE/LIMIT`
  cases).
- D6 requires implementer investigation time before a fix can be designed; this blocks
  BLOCKER-003 resolution until the root cause is established.

### Status as of v1.1 (2026-06-24)

ACCEPTED. HRG-4/MAJOR-001 ruling: Path B (`list_capabilities` consults `OrgRegistry`)
ratified 2026-06-24. BLOCKER-003 fix is blocked pending implementer investigation per D6.
Mode-bridge error (D1/D2/D3), Filter mode tests (D4), and MAJOR-001 Path B wiring are
implementable immediately.

---

## Alternatives Considered

- **Emit `normalized_pql` from the parser (Chumsky error recovery) rather than as a
  post-parse heuristic:** Rejected because Chumsky's `Rich` error type does not carry
  structural information about what a correct rewrite would look like — it only knows what
  tokens were expected. The post-parse heuristic in `error_recovery.rs` can inspect the
  original query string and derive the rewrite, which Chumsky cannot.

- **Add a pre-parse mode detection layer that rejects mode-mixed queries with a clear
  diagnostic before Chumsky runs:** Rejected because mode detection already exists in
  `is_pipe_mode` and the pre-parse layer would duplicate it. The correct place for the
  mode-bridge diagnostic is the post-parse error handler where we know both (a) which
  mode was selected and (b) where the parse failed.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.2 | 2026-06-24 | architect | D7 mode composition boundary ruling added: Filter is bare-predicate sugar (strict subset of SQL/Pipe); only SQL→Pipe syntactic composition is supported (ADR-043); pipe is the canonical execution model; SQL and Filter are sugar entry syntaxes that lower into it; shared predicate grammar across all three modes; three-way composition rejected; graduation path Filter→Pipe documented. |
| v1.1 | 2026-06-24 | architect | HRG ratification burst. HRG-4/MAJOR-001: Path B ratified; MAJOR-001 ruling section added to §Decision (list_capabilities consults OrgRegistry via Arc-DI wiring; Path A rejected). Status PROPOSED→ACCEPTED. |
| v1.0 | 2026-06-24 | architect | Initial draft. PROPOSED. No human ratification gate (all decisions within established spec), but MAJOR-001 fix path choice deferred for human selection. |

---

## Source / Origin

- Grammar usability audit: `.factory/research/prismql-grammar-usability-audit-2026-06-24.md`
  §GRAMMAR-001, §GRAMMAR-014, §GRAMMAR-015, §GRAMMAR-016, §GRAMMAR-018.
- Pre-flight audit: `.factory/research/demo-pre-flight-audit-2026-06-24.md`
  BLOCKER-002, BLOCKER-003, §1.4 three modes, §2.5 prompt behavior.
- `crates/prism-query/src/error_recovery.rs` — `rich_to_parse_error` (the post-parse
  error formatter that D1/D2 extend).
- `crates/prism-mcp/src/prompts.rs` lines 450-493 — `render_query_tutorial` and
  `render_investigate_host` (confirmed synchronous; hang is in dispatch layer).
- `crates/prism-mcp/src/prompts.rs` lines 247-300 — `PromptRoute::new_dyn` closures
  (candidate hang sites).
- ADR-041 §L4 — pedagogical E-QUERY-NNN self-correction loop (this ADR implements the
  mode-bridge component of that layer).
