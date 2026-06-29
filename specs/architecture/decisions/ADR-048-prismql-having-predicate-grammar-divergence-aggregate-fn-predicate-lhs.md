---
document_type: adr
adr_id: "ADR-048"
title: "PrismQL HAVING/WHERE Predicate Grammar Divergence — Aggregate-Function Predicate LHS in HAVING"
status: accepted
date: "2026-06-28"
accepted_date: "2026-06-29"
version: "1.1"
modified: "2026-06-29"
producer: architect
subsystems_affected: [SS-11]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-DEMO-FIDELITY-REMEDIATION-001]
related_adrs: [ADR-041, ADR-043, ADR-046, ADR-003]
related_bcs: [BC-2.11.016, BC-2.11.003]
locked_decisions:
  - OD-1: ratified by user decision 2026-06-29 (chose Option A — extend HAVING grammar, keep WHERE divergence)
  - OD-2: PERCENTILE excluded from HAVING aggregate predicate grammar — resolved as accepted technical scope decision
wiring_deferred_to: null
open_decisions: []
---

# ADR-048: PrismQL HAVING/WHERE Predicate Grammar Divergence — Aggregate-Function Predicate LHS in HAVING

## Status

ACCEPTED v1.1 (2026-06-29). Human ratification received 2026-06-29; both open decisions
resolved. Implementation verified converged across 4 adversarial passes in
S-DEMO-FIDELITY-REMEDIATION-001 LOCAL re-gate.

## Resolution

### OD-1 — HAVING/WHERE Grammar Divergence: RATIFIED

**Status:** Resolved. Decision: Option A accepted.

The human (user) explicitly ratified the HAVING/WHERE predicate grammar divergence on
2026-06-29 when presented with the F-PXL3-MED-002 two-option analysis (Option A: extend
HAVING grammar; Option B: correct BC to bare-column form). The user chose Option A —
extend the grammar so that `HAVING <agg>(col) op literal` is a valid production gated by
E-QUERY-038 column checks, while `WHERE <agg>(col) op literal` deliberately remains
E-QUERY-001 (pre-aggregation WHERE does not accept aggregate function LHS). This OD-1
ratification locks D.1 and D.6 of this ADR.

### OD-2 — PERCENTILE Exclusion: RESOLVED

**Status:** Resolved as accepted technical scope decision within the ratified Option A extension.

The 2-argument `PERCENTILE(field, p)` form is excluded from `build_agg_call_parser` in
the HAVING predicate grammar. In scope: `COUNT(*) / COUNT(field) / SUM / AVG / MIN / MAX /
DISTINCT_COUNT`. Rationale: `PERCENTILE(field, p)` creates parser grammar ambiguity in
the predicate context (comma inside the agg arg list is indistinguishable from a
predicate-list separator); it is also non-standard in HAVING predicates across major SQL
dialects. Analysts needing percentile-based HAVING may alias in SELECT and reference the
alias. This OD-2 resolution locks D.2 of this ADR.

### Implementation Verification (POL-15)

ADR-048's primary deliverable — the HAVING grammar extension in prism-query — is confirmed
reachable from a production binary. `prism-mcp` and `prism-bin` both depend on `prism-query`
via the standard workspace dependency graph. The `build_having_predicate_parser` and
`collect_predicate_columns` FuncCall arm are in the query execution path invoked for every
HAVING clause in a GROUP BY query. The grammar/gate ships as part of the live query
engine — POL-15 (runtime_wiring_required_for_accepted_adrs) is satisfied.

Four adversarial passes during S-DEMO-FIDELITY-REMEDIATION-001 LOCAL re-gate confirmed
no drift between the shipped code (`build_having_predicate_parser`, `collect_predicate_columns`
FuncCall arm, WHERE unchanged at `build_predicate_parser`) and the decisions D.1–D.6
recorded here.

## Context

### The Finding (F-PXL3-MED-002)

BC-2.11.016 v1.5 (E-QUERY-038 Column-Not-Found Plan-Time Gate) mandates that the query:

```sql
SELECT severity, count(*) FROM crowdstrike_alerts
GROUP BY severity HAVING count(typo_col) > 5
```

produces `E-QUERY-038` (column not found for `typo_col`) rather than a DataFusion
internal error. The BC's EC-11-046 edge case and the canonical test vector both cite
the aggregate-function-predicate LHS form `HAVING count(typo_col) > 5`.

The implementation delivered in S-DEMO-FIDELITY-REMEDIATION-001 (F-PWL1-LOW-001 / Position
6 HAVING walk) correctly added HAVING to `check_query_column_availability`. However, the
load-bearing test uses the bare-column form `HAVING typo_col > 5` rather than
`HAVING count(typo_col) > 5`, because the shared predicate grammar (shared between WHERE
and HAVING) does not accept aggregate function calls on the predicate LHS.

### Grammar Root Cause

The PrismQL predicate grammar (`build_predicate_parser` in `filter_parser.rs`) is shared
across all three query modes (filter, SQL WHERE, SQL HAVING) and accepts only these atom
forms:

```
atom := '(' predicate ')'
      | HAS field_path
      | MISSING field_path
      | field_path '=~' string
      | field_path MATCHES string
      | field_path IN CIDR string
      | field_path NOT IN '(' literal_list ')'
      | field_path IN '(' literal_list ')'
      | field_path BETWEEN literal AND literal
      | field_path IS [NOT] NULL
      | field_path string_op string
      | field_path cidr string
      | field_path LIKE literal
      | field_path compare_op rhs_expr      ← the Compare atom
```

Every atom has a `field_path` LHS. There is no `agg_func(field_path) compare_op literal`
production. The aggregate function call forms (`COUNT`, `SUM`, `AVG`, `MIN`, `MAX`,
`DISTINCT_COUNT`, `PERCENTILE`) exist only in the SQL expression parser
(`build_sql_expr_parser` in `sql_parser.rs`), which is used by SELECT projections,
GROUP BY, and ORDER BY — but NOT by the predicate parser.

Therefore `HAVING count(typo_col) > 5` fails at parse time with E-QUERY-001 (Chumsky
sees `count` as a bare field path, then hits `(` which is not a compare operator —
parse error). It never reaches the column gate.

### Why Shared Grammar Cannot Simply Gain Aggregate LHS

WHERE and HAVING share `build_predicate_parser`. If aggregate-function predicate LHS
is added to `build_predicate_parser`, then `WHERE count(col) > 5` also becomes
syntactically valid. This is semantically incorrect: WHERE is evaluated before
aggregation; aggregate functions in WHERE are undefined behavior in DataFusion and
standard SQL. DataFusion would produce an internal error at execution time if such a
query were executed, defeating the purpose of the pedagogical gate.

Two candidate solutions:

**Option A (selected):** Create a HAVING-specific predicate parser that extends the base
predicate parser with aggregate-function predicate LHS, while WHERE continues to use only
the base predicate parser. HAVING grammar diverges from WHERE grammar intentionally.

**Option B (rejected):** Accept the grammar limitation, correct BC-2.11.016 EC-11-046
and the canonical test vector to use the bare-column form. E-QUERY-001 already fires for
`HAVING count(typo_col) > 5` at parse time (before any sensor call), so the pedagogical
goal is partially met. However, `count(typo_col) > 5` is a genuinely useful and common
HAVING shape; producing E-QUERY-001 for it is a language usability gap. The production-grade
default biases toward making the language genuinely useful rather than patching the spec
to hide a deficiency.

## Decision

### D.1 — HAVING/WHERE Predicate Grammar Diverge Intentionally

PrismQL's WHERE clause and HAVING clause now use different predicate grammars:

- **WHERE predicate grammar:** the existing `build_predicate_parser` — unchanged. All
  WHERE predicates must have a `field_path` LHS. `WHERE count(col) > 5` remains E-QUERY-001.
- **HAVING predicate grammar:** a new `build_having_predicate_parser` that extends the
  base predicate parser with an additional atom family: aggregate-function predicate LHS.

This divergence is **intentional and documented**. Standard SQL HAVING is defined as a
post-aggregation filter that may reference aggregate expressions. WHERE is a pre-aggregation
filter that may not. Prism's grammar now reflects this semantic distinction.

### D.2 — Aggregate-Function Predicate LHS Atom Family (HAVING only)

The HAVING predicate grammar adds the following atom family:

```
agg_call := 'COUNT' '(' ('*' | field_path) ')'
          | 'SUM' '(' field_path ')'
          | 'AVG' '(' field_path ')'
          | 'MIN' '(' field_path ')'
          | 'MAX' '(' field_path ')'
          | 'DISTINCT_COUNT' '(' field_path ')'

having_agg_atom := agg_call compare_op rhs_expr
```

This produces: `Predicate::Compare { lhs: Expr::FuncCall(FuncCall::Aggregate { .. }),
op: CompareOp, rhs: Expr::Literal(..) }` — reusing the existing AST types with no new
variants required.

**PERCENTILE is excluded from HAVING predicate LHS.** Rationale: `PERCENTILE(field, p)`
is not a standard aggregate function for HAVING predicates in any major SQL dialect. Its
two-argument form creates grammar ambiguity (`PERCENTILE(field, 90) > 5` would require a
comma inside the agg arg list while the surrounding predicate also uses comma separators).
It is available in SELECT projections and GROUP BY via the SQL expression parser. Analysts
needing percentile-based HAVING can alias it in SELECT (`SELECT PERCENTILE(latency, 95) AS p95 ...`)
and reference the alias in HAVING. This is the standard SQL pattern.

**COUNT with field argument** (`count(field)`) is in scope. `count(*)` (star arg) is also
in scope. Both are common HAVING patterns.

### D.3 — Column Extraction from Aggregate-Function Predicate LHS

The `collect_predicate_columns` function (engine.rs) HAVING path (Position 6) must be
extended so that when `Predicate::Compare { lhs: Expr::FuncCall(..), .. }` is encountered,
the extractor recurses into the FuncCall args to extract any `Expr::Field` column references.

Specifically: the existing `Predicate::Compare` arm currently handles only `Expr::Field`
LHS. It must be extended to also handle `Expr::FuncCall` LHS by delegating to
`extract_field_paths_from_expr(lhs_expr, ...)` — the same recursive walker already used
by SELECT (Position 1), GROUP BY (Position 3), ORDER BY (Position 4), and JOIN ON
(Position 5). This extension is required to make `count(typo_col) > 5` in HAVING produce
E-QUERY-038 rather than silently passing the column gate.

**Important:** this change to `collect_predicate_columns` is not a WHERE/HAVING asymmetry
in the extractor — WHERE predicates will never have a FuncCall LHS because the WHERE
grammar doesn't produce them. The extractor change is therefore WHERE-safe: adding the
FuncCall arm to `collect_predicate_columns` cannot cause false positives in the WHERE
position because the WHERE grammar cannot produce a `Predicate::Compare` with
`Expr::FuncCall` LHS.

### D.4 — BC-2.11.016 v1.5 Status: No Change Required

Once the grammar and extractor are extended per D.2 and D.3, BC-2.11.016 v1.5's claim
in EC-11-046 and the canonical test vector "having-position" (`HAVING count(typo_col) > 5`
→ E-QUERY-038) becomes accurate. The BC does not need a version bump for this fix.

If the product-owner judges that BC-2.11.016 should record the HAVING/WHERE grammar
divergence as an explicit postcondition note, a minor prose addition to the §Description
or §Implementation Location section is acceptable but not required. The grammar divergence
is architecturally documented here.

### D.5 — COUNT(*) Behavior in HAVING Predicate Gate

`HAVING count(*) > 5` uses the star arg, producing `Predicate::Compare { lhs:
Expr::FuncCall(FuncCall::Aggregate { func: AggFunc::Count, args: [Expr::Star], .. }), .. }`.
`Expr::Star` has no column name, so the column gate produces no column to check — the
gate passes silently. This is correct: `count(*)` counts all rows and has no column
dependency. No false E-QUERY-038 fires.

### D.6 — WHERE Parity: Deliberately None

The WHERE grammar does not gain aggregate-function predicate LHS. `WHERE count(col) > 5`
remains E-QUERY-001. This is the correct behavior: WHERE is pre-aggregation. The
E-QUERY-001 error for this form is acceptable because it is a parse error that fires
immediately, never reaching any sensor. An LLM agent receiving E-QUERY-001 for that
form should be directed to use HAVING or restructure the query.

## Consequences

### Positive

- BC-2.11.016 v1.5 EC-11-046 claim (`HAVING count(typo_col) > 5` → E-QUERY-038) becomes
  true, satisfying the pedagogical gate for the most common HAVING pattern.
- LLM agents writing `HAVING count(col) > 5` will receive E-QUERY-038 with `did_you_mean`
  instead of a confusing DataFusion error.
- The grammar divergence accurately reflects standard SQL semantics (HAVING is
  post-aggregation; WHERE is not).

### Negative / Risks

- WHERE and HAVING are now parsed by different parser functions. Tests must explicitly
  guard against WHERE aggregate LHS silently becoming valid (see §Tests).
- `build_sql_parser` must pass `having_clause` the new HAVING-specific parser rather than
  the existing shared `predicate` parser. This is a one-line change in the HAVING clause
  assembly, but requires care to not affect the WHERE clause.
- The `collect_predicate_columns` FuncCall arm extension is technically a change to a
  function also used by WHERE position (Position 2). Empirically this is safe (WHERE
  grammar cannot produce FuncCall LHS) but must be documented and test-verified.

### Neutral

- The existing `test_BC_2_11_016_having_column_gate_typo_fires_e_query_038` test (which
  uses `HAVING typo_col > 5`) remains valid and load-bearing as a guard for the bare-column
  HAVING path. The new test for `HAVING count(typo_col) > 5` is ADDITIVE — it tests
  a different grammar production.
- `extract_field_paths_from_expr` is not changed. Only `collect_predicate_columns` gains
  a FuncCall arm.
- E-QUERY-037 and E-QUERY-039, which already walk HAVING via `extract_field_paths_from_expr`
  (not the predicate extractor), are unaffected by this change.

## Considered Alternatives

### Alt-1: Correct BC to bare-column form (Option B, rejected)

Correct BC-2.11.016 EC-11-046 to use `HAVING typo_col > 5` and accept E-QUERY-001 for
the aggregate form. Simpler; no grammar change needed. Rejected because `HAVING count(col) > 5`
is a common and legitimate HAVING shape; producing E-QUERY-001 for it is a language
usability gap. The production-grade default biases toward making the language genuinely
useful.

### Alt-2: Single unified grammar with aggregate LHS in both WHERE and HAVING (rejected)

Extend `build_predicate_parser` to accept aggregate LHS for both WHERE and HAVING. Simpler
implementation. Rejected because `WHERE count(col) > 5` is semantically invalid in SQL
and DataFusion; making it parse successfully would cause confusing DataFusion execution
errors rather than a clean parse-time rejection.

### Alt-3: Context flag on predicate parser (considered but rejected)

Pass a boolean `allow_aggregate_lhs: bool` flag into `build_predicate_parser`. Functionally
equivalent to Option A, but adds a parameter to a function used across three parsers.
A separate `build_having_predicate_parser` is cleaner because it has a clear, named purpose
and avoids conditional branching inside the parser combinator.

## Related Architecture Nodes

- `filter_parser.rs` `build_predicate_parser` — shared predicate grammar unchanged
- `sql_parser.rs` `build_sql_predicate_parser` — WHERE path unchanged; HAVING now uses
  `build_having_predicate_parser`
- `engine.rs` `collect_predicate_columns` — gains FuncCall arm in Compare branch
- `engine.rs` `extract_field_paths_from_expr` — unchanged
- BC-2.11.016 v1.5 §EC-11-046 — becomes accurate after implementation

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | F-PXL3-MED-002-adr-048 | 2026-06-28 | architect | Initial ADR — HAVING/WHERE predicate grammar divergence rationale, D.1–D.6, consequences, considered alternatives. Addresses F-PXL3-MED-002 root cause analysis. |
| 1.1 | adr-048-acceptance-S-DEMO-FIDELITY-REMEDIATION-001 | 2026-06-29 | architect | PROPOSED → ACCEPTED. OD-1 ratified by user decision 2026-06-29 (Option A: extend HAVING grammar, keep WHERE E-QUERY-001). OD-2 resolved: PERCENTILE excluded from HAVING predicate grammar as accepted technical scope decision. §Resolution section added. POL-15 confirmed satisfied. `locked_decisions` populated; `open_decisions` cleared. |
