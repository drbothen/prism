---
document_type: adr
adr_id: "ADR-048"
title: "PrismQL HAVING/WHERE Predicate Grammar Divergence — Aggregate-Function Predicate LHS in HAVING"
status: accepted
date: "2026-06-28"
accepted_date: "2026-06-29"
version: "1.6"
modified: "2026-07-13"
producer: architect
subsystems_affected: [SS-11]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-DEMO-FIDELITY-REMEDIATION-001, DEFECT-PQL-FNCALL-LHS-001]
related_adrs: [ADR-041, ADR-043, ADR-046, ADR-003, ADR-052]
related_bcs: [BC-2.11.016, BC-2.11.003]
locked_decisions:
  - OD-1: ratified by user decision 2026-06-29 (chose Option A — extend HAVING grammar, keep WHERE divergence)
  - OD-2: PERCENTILE excluded from HAVING aggregate predicate grammar — resolved as accepted technical scope decision
  - OD-3: HAVING policy for non-ADR-048-D.2 aggregate functions (stddev, variance, corr, median, etc.) — permit as shipped (Option i); parse via fn_call_comparison fallthrough as FuncCall::Scalar(Unknown); architect decision 2026-07-13 (DEFECT-PQL-FNCALL-LHS-001 pass-2 F-PQLFN-P2-MED-001)
  - OD-4: Parser-level AGGREGATE_FUNC_NAMES blocklist in fn_call_comparison removed; plan-time DATAFUSION_BUILTIN_AGGREGATE_NAMES gate in check_enrich_udf_availability is the sole enforcement point; architect decision 2026-07-13 (DEFECT-PQL-FNCALL-LHS-001 pass-2 F-PQLFN-P2-MED-002)
  - OD-5: SQL WHERE predicate fn-call positions added to predicate_fncall_names coverage so DATAFUSION_BUILTIN_AGGREGATE_NAMES gate covers all non-HAVING predicate positions; architect decision 2026-07-13 (DEFECT-PQL-FNCALL-LHS-001 pass-2 F-PQLFN-P2-HIGH-001)
  - OD-6: DML WHERE predicate fn-call positions added to predicate_fncall_names coverage (sixth gated position); cross-mode consistency over intentional out-of-scope deferral; architect decision 2026-07-13 (DEFECT-PQL-FNCALL-LHS-001 pass-7 F-PQLFN-P7-LOW-002)
wiring_deferred_to: null
open_decisions: []
---

# ADR-048: PrismQL HAVING/WHERE Predicate Grammar Divergence — Aggregate-Function Predicate LHS in HAVING

## Status

ACCEPTED v1.6 (2026-07-13). F-PQLFN-P7-LOW-002 DML WHERE gate extension: §D.7.1 table
extended with Position 6 (SQL DML WHERE); §D.7.5 new (arm shape, implementation scope,
required tests). §D.6 enumeration updated to include SQL DML WHERE. §D.3 and §D.7.2
count references updated from five to six. Related Architecture Nodes updated. OD-6
locked: DML WHERE added to predicate_fncall_names coverage for cross-mode consistency.
Adjudication rationale: branch-introduced regression (fn_call_comparison now in
build_predicate_parser, which build_delete_parser and build_update_parser both bind) turned
pre-branch parse-time E-QUERY-001 into post-branch SILENT EMPTY SUCCESS; extending the
gate restores meaningful errors with zero risk (DML execution no-ops to Ok(vec![])).

ACCEPTED v1.5 (2026-07-13). F-PQLFN-P5-LOW-001 citation extension: §D.2 empirical claim
("percentile absent from `DATAFUSION_BUILTIN_FUNCTION_NAMES`") now anchored to all three
union-member registries (scalar ∪ aggregate ∪ window). v1.4 only cited the aggregate-registry
absence test. Two new executed locks added at commit bb23f143 complete scalar and window
arm coverage. §D.2 Mechanism 2 citation now names all five executed checks: 3 absence locks
(aggregate + scalar + window) + 2 presence controls. §D.7.3 cross-reference updated to
cite F-PQLFN-P5-LOW-001.

ACCEPTED v1.4 (2026-07-13). F-PQLFN-P4-MED-001 ADR correction: §D.2 PERCENTILE
post-blocklist-removal note retracted and replaced. ADR-048 v1.3 claimed "percentile IS
registered in DataFusion 53.1 `default_aggregate_functions()`" and "present in
`DATAFUSION_BUILTIN_FUNCTION_NAMES`" — both claims are FALSE, proven by executed tests
(module `datafusion_aggregate_registry_empirical_tests` in engine.rs, commit 524a9986).
The manual `names.insert("percentile")` in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` is
NECESSARY, not redundant. Corrected §D.2 PERCENTILE note and §D.7.3 cross-reference.
[process-gap]: v1.3 "empirical" claim cited metadata inference (UDAF names assumed from
documentation), not an executed test; "empirical" claims in spec artifacts must cite an
executed check (test name or command output — feeds S-7.02 codification).

ACCEPTED v1.3 (2026-07-13). Two findings from DEFECT-PQL-FNCALL-LHS-001 adversary pass-3
adjudicated: F-PQLFN-P3-LOW-001 (§D.3 "Important:" paragraph self-contradiction corrected —
WHERE grammar DOES produce FuncCall LHS post-D.7.2; WHERE-safety restated in terms of
extractor arg-recursion, not grammar impossibility) and F-PQLFN-P3-OBS-003 (PERCENTILE
post-blocklist-removal error surface documented — v1.3 empirical claim later proven false
in v1.4; see §D.2 v1.4 correction). No locked-decision changes; no new ODs.

ACCEPTED v1.2 (2026-07-13). Three findings from DEFECT-PQL-FNCALL-LHS-001 adversary pass-2
adjudicated and codified: F-PQLFN-P2-HIGH-001 (SQL WHERE aggregate-in-predicate regression),
F-PQLFN-P2-MED-001 (HAVING non-six-name aggregate AST tagging policy), F-PQLFN-P2-MED-002
(parser-level blocklist UX inconsistency). New §D.7 documents the unified plan-time gate.
§D.2 and §D.6 amended. Three new locked decisions (OD-3/OD-4/OD-5). Cross-ref ADR-052 v1.13.

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

**PERCENTILE post-blocklist-removal error surface (v1.4 correction — F-PQLFN-P4-MED-001):**
With the parser-level `AGGREGATE_FUNC_NAMES` blocklist removed (§D.7.2), `HAVING
percentile(x, 95) > 5` parses as `FuncCall::Scalar(Unknown("percentile"))` via
`fn_call_comparison` in `build_sql_predicate_parser` (the `base` branch of
`build_having_predicate_parser`), following the non-six-name aggregate path described
in §D.7.3. The error surface is governed by two separate mechanisms:

**Mechanism 1 — Aggregate gate (E-QUERY-001):** HAVING predicates are NOT walked into
`predicate_fncall_names`. The aggregate-in-predicate gate (D.7.1) does NOT fire.
**E-QUERY-001 does NOT fire** for `HAVING percentile(x, 95) > 5`.

**Mechanism 2 — E-QUERY-039 gate:** HAVING predicates ARE walked into `sql_unknown_names`
via position (f) of `collect_unknown_scalars_from_sql_query`. "percentile" is NOT in
`DATAFUSION_BUILTIN_FUNCTION_NAMES` — **DataFusion 53.1 does NOT register "percentile"
in any of the three union-member registries** (`DATAFUSION_BUILTIN_FUNCTION_NAMES` is
computed as scalar ∪ aggregate ∪ window; per the v1.4 [process-gap] rule, every
union-membership claim must be anchored to an executed check per registry arm). All five
covering tests live in module `datafusion_aggregate_registry_empirical_tests`, engine.rs:

- **Aggregate-registry absence** (commit 524a9986): `test_f_pqlfn_p4_med_001_percentile_absent_from_datafusion_53_1_aggregate_registry`
- **Scalar-registry absence** (commit bb23f143, F-PQLFN-P5-LOW-001): `test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_scalar_registry`
- **Window-registry absence** (commit bb23f143, F-PQLFN-P5-LOW-001): `test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_window_registry`
- **Presence control — `approx_percentile_cont` IS in registry** (commit 524a9986): `test_f_pqlfn_p4_med_001_approx_percentile_cont_present_in_datafusion_53_1_registry`
- **Presence control — `approx_distinct` IS in registry** (commit 524a9986): `test_f_pqlfn_p4_med_001_approx_distinct_present_in_datafusion_53_1_registry`

The `DATAFUSION_BUILTIN_FUNCTION_NAMES` filter therefore does NOT exclude "percentile" before
the E-QUERY-039 check. The E-QUERY-039 outcome is registry-dependent:
- **No infusion registry configured:** `check_enrich_udf_availability` returns early
  `Ok(())` when `registry` is `None` → E-QUERY-039 is skipped → query passes to
  DataFusion → DataFusion plan error (percentile not a DataFusion built-in aggregate).
- **Infusion registry configured:** "percentile" reaches the registered-UDF check →
  **E-QUERY-039 fires** (percentile is not in `DATAFUSION_BUILTIN_FUNCTION_NAMES` and
  not a registered enrichment UDF).

Confirmed by executed test `test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt`
(temporal_typing_tests.rs, commit 524a9986): engine has no infusion registry → result is
NOT `PrismError::QueryParseFailed` (E-QUERY-001) → aggregate gate correctly does not fire;
actual result is a DataFusion-level plan error.

**v1.3 false claim retracted:** v1.3 stated "percentile IS registered as the canonical name
for three aggregate UDAFs" and "present in `DATAFUSION_BUILTIN_FUNCTION_NAMES`". Both are
FALSE. The UDAF names (`approx_percentile_cont`, etc.) were assumed from DataFusion
documentation metadata without executing a check. The manual `names.insert("percentile")`
in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` is NECESSARY, not redundant — it ensures "percentile"
is gated in WHERE/predicate positions even though DataFusion 53.1 does not register it natively.

The "exclusion" in OD-2 and the statement "PERCENTILE is excluded from HAVING predicate LHS"
refer specifically to the absence of a first-class `FuncCall::Aggregate` AST node for
PERCENTILE in `build_agg_call_parser` — not to a plan-time rejection. The analyst-directed
SELECT-alias pattern remains recommended for cross-dialect portability.

**COUNT with field argument** (`count(field)`) is in scope. `count(*)` (star arg) is also
in scope. Both are common HAVING patterns.

**D.2 scope note (v1.2 amendment):** This list defines the aggregate functions for which
`build_agg_call_parser` emits an explicit `FuncCall::Aggregate` AST node in the HAVING
predicate grammar. It is NOT the complete list of aggregate functions permitted in HAVING
at runtime. Other DataFusion built-in aggregate functions (stddev, variance, corr, median,
approx_median, regr_*, bool_and, bool_or, etc.) are also valid in HAVING predicates: they
parse via the `fn_call_comparison` fallthrough in `build_sql_predicate_parser` (the `base`
branch of `build_having_predicate_parser`) and produce `FuncCall::Scalar(Unknown(name))`
AST nodes. DataFusion resolves these correctly at execution time via its aggregate function
registry. See §D.7 for the policy rationale (OD-3).

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

**Important (v1.3 amendment — F-PQLFN-P3-LOW-001):** post-D.7.2, the WHERE predicate
grammar DOES produce `Predicate::Compare` with `Expr::FuncCall` LHS. `fn_call_comparison`
in `build_predicate_parser` accepts any function call in predicate position — aggregate
names parse as `FuncCall::Scalar(Unknown(name))` and are caught by the plan-time
`DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate (D.7.1), but the grammar itself does not
prevent them from being produced. The earlier claim that "the WHERE grammar cannot
produce a `Predicate::Compare` with `Expr::FuncCall` LHS" was accurate before D.7.2
(when the parser-level `AGGREGATE_FUNC_NAMES` `try_map` guard blocked aggregate names
at parse time) but is self-contradicted by §D.7.1, which enumerates six predicate
positions — including SQL WHERE and SQL DML WHERE — that now feed FuncCall names to the
plan-time gate after parsing successfully.

The extractor change is WHERE-safe not because grammar impossibility prevents FuncCall
LHS in WHERE predicates (it does not), but because `extract_field_paths_from_expr`
recurses correctly into FuncCall args regardless of whether the call is an aggregate or
a scalar UDF. Column extraction operates on the args, not the function identity. No
false E-QUERY-038 fires result from the FuncCall arm in `collect_predicate_columns` for
WHERE positions — the column check correctly identifies the field arguments to any
function call appearing in predicate LHS position.

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

### D.6 — WHERE Aggregate Invariant: All DataFusion Built-in Aggregates Rejected (v1.2 restated)

The WHERE clause in every PrismQL mode (pipe `| where`, filter root, SqlPipe `| where`,
SQL WHERE, SqlPipe-head WHERE, **SQL DML WHERE**) does not accept aggregate-function
predicate LHS. `WHERE agg(col) op literal` is rejected with E-QUERY-001 for ALL DataFusion
built-in aggregate functions, enforced by the plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
gate in `check_enrich_udf_availability`. This invariant covers count, sum, avg, min, max,
distinct_count AND all extended aggregates (stddev, variance, corr, median, approx_median,
regr_*, array_agg, string_agg, bool_and, bool_or, etc.) — any name in DataFusion's
`SessionStateDefaults::default_aggregate_functions()` registry.

**SQL DML WHERE (v1.6 addition — OD-6):** `DELETE FROM t WHERE stddev(x) > 5` and
`UPDATE t SET col = val WHERE stddev(x) > 5` are rejected with the canonical E-QUERY-001
message. This is the sixth gated position (§D.7.1 Position 6, §D.7.5). DML WHERE
previously fell to `_ => {}` in `check_enrich_udf_availability`; post-v1.6 it is walked
into `predicate_fncall_names` by the `Ast::Sql(SqlStatement::Dml(dml))` arm.

**v1.2 restatement of enforcement mechanism:** The prior v1.1 text stated that WHERE
aggregate rejection was enforced by a parser-level `AGGREGATE_FUNC_NAMES` list in the
`fn_call_comparison` `try_map` guard. That list (count, sum, avg, min, max, distinct_count,
percentile) was incomplete and its error was swallowed by Chumsky backtrack in practice
(analysts received "found '('" instead of E-QUERY-001 — F-PQLFN-P2-MED-002).
Furthermore, SQL WHERE aggregate names reached `sql_unknown_names` and were filtered by
`DATAFUSION_BUILTIN_FUNCTION_NAMES` before the aggregate gate ran, so `WHERE stddev(x) = 5`
escaped to DataFusion and produced -32000 (F-PQLFN-P2-HIGH-001).

Post-v1.2 enforcement: the `AGGREGATE_FUNC_NAMES` `try_map` guard is REMOVED from
`fn_call_comparison`. The plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate in
`check_enrich_udf_availability` is the SOLE enforcement point. `predicate_fncall_names`
is extended to cover SQL WHERE predicate positions (see §D.7). The gate fires the
canonical "E-QUERY-001: '{name}' is an aggregate function; aggregate fn-calls are not
valid in pipe | where (use HAVING for post-aggregation filters, ADR-048 D.3)" message for
all covered positions.

An LLM agent receiving E-QUERY-001 for any aggregate-in-WHERE form should be directed
to use HAVING or restructure the query. The pre-aggregation semantic prohibition is
correct and intentional; this clause is the D.1 ratification applied across the full
aggregate namespace.

### D.7 — Unified Plan-Time Aggregate-in-Predicate Gate (v1.2 new)

This decision point codifies the DEFECT-PQL-FNCALL-LHS-001 pass-2 adjudication. It
supersedes the v1.1 informal description of the ADR-048 D.3 gate and replaces the
removed parser-level enforcement.

#### D.7.1 — Gate Scope

The `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate in `check_enrich_udf_availability` fires
E-QUERY-001 for any `ScalarFunc::Unknown(name)` appearing as a predicate-comparison LHS
when `name` ∈ `DATAFUSION_BUILTIN_AGGREGATE_NAMES`. Gate applies to ALL six predicate
positions that feed into `predicate_fncall_names`:

| Position | Collection method | Pre-v1.2 coverage | Post-v1.6 coverage |
|---|---|---|---|
| Pipe `| where` | `collect_unknown_scalar_from_predicate` on `PipeStage::Where` | YES | YES |
| Filter root | `collect_unknown_scalar_from_predicate` on `Ast::Filter` | YES | YES |
| SqlPipe `\| where` | `collect_unknown_scalar_from_predicate` on `Ast::SqlPipe` pipe stages | YES | YES |
| SQL WHERE | `collect_unknown_scalar_from_predicate` on `sq.where_` in `Ast::Sql` arm | NO (was in sql_unknown_names; DFBIAFN filter bypassed gate) | YES (v1.2) |
| SqlPipe-head WHERE | `collect_unknown_scalar_from_predicate` on `spq.head.where_` in `Ast::SqlPipe` arm | NO (was in sql_unknown_names via collect_unknown_scalars_from_sql_query) | YES (v1.2) |
| SQL DML WHERE | `collect_unknown_scalar_from_predicate` on `dml.filter` in `Ast::Sql(SqlStatement::Dml(dml))` arm | NO (fell to `_ => {}`; pre-branch: parser E-QUERY-001) | YES (v1.6, OD-6) |

**HAVING is explicitly exempt.** HAVING predicates are not walked into
`predicate_fncall_names`. They reach `sql_unknown_names` via
`collect_unknown_scalars_from_sql_query` position (f), where
`DATAFUSION_BUILTIN_FUNCTION_NAMES` filters all DataFusion-known functions (scalar,
aggregate, window) before the E-QUERY-039 check. This is correct: HAVING may legitimately
reference aggregate functions, and no E-QUERY-001 should fire there.

**JOIN ON, GROUP BY, ORDER BY, SELECT projection** are not in scope for the
aggregate-in-predicate gate. These positions are handled by other gates (E-QUERY-042 for
degenerate GROUP BY / ORDER BY via ADR-052, DataFusion for valid SELECT aggregates).
Aggregate functions in JOIN ON are not gated and reach DataFusion; that edge case is
out of scope for this fix.

#### D.7.2 — Single Message Source (MED-002 closure)

The `fn_call_comparison` production in `build_predicate_parser` (`filter_parser.rs`) no
longer contains the `AGGREGATE_FUNC_NAMES` `try_map` guard. That guard was unreliable:
Chumsky backtrack swallowed the rejection error, producing "found '('" instead of the
E-QUERY-001 message when an aggregate name was encountered. Removing it means:

- Aggregate function names in predicate positions parse successfully as
  `FuncCall::Scalar(Unknown(name))` via `fn_call_comparison`
- The plan-time gate in `check_enrich_udf_availability` catches the name and emits
  the canonical E-QUERY-001 message: "'{name}' is an aggregate function; aggregate
  fn-calls are not valid in pipe | where (use HAVING for post-aggregation filters,
  ADR-048 D.3)"
- All six predicate positions receive an identical, helpful message — single message
  source, consistent analyst/LLM UX

#### D.7.3 — HAVING Non-Six-Name Aggregate Policy (MED-001 adjudication)

`HAVING stddev(x) > 5`, `HAVING variance(col) > 100`, `HAVING corr(a, b) > 0.5` etc.
are legitimate standard SQL HAVING predicates. These names are not in `build_agg_call_parser`
(which handles only the six named functions per §D.2). Post-removal of the parser-level
blocklist, these names parse via `fn_call_comparison` in `build_sql_predicate_parser`
(the `base` branch of `build_having_predicate_parser`) as:

```
Predicate::Compare {
    lhs: Expr::FuncCall(FuncCall::Scalar { func: ScalarFunc::Unknown("stddev"), args }),
    op, rhs
}
```

This AST tagging is semantically imprecise (the function is an aggregate at execution
time, not a scalar) but has no runtime correctness impact: DataFusion resolves the name
via its aggregate function registry regardless of how PrismQL tagged it. The
`collect_predicate_columns` FuncCall arm (D.3) correctly recurses into the args for
E-QUERY-038 column checking.

**Rationale for Option (i) — permit as shipped:**
- Rejecting `HAVING stddev(x) > 5` would be a usability regression for valid standard SQL
- No existing code pattern-matches `FuncCall::Aggregate` specifically in a HAVING context
  to make routing decisions that would be confused by `FuncCall::Scalar`
- Extending `build_agg_call_parser` to all DataFusion aggregate names would require a
  dynamic, runtime-query-constructed parser from `DATAFUSION_BUILTIN_AGGREGATE_NAMES`
  (significantly higher implementation complexity with no correctness benefit)
- The misclassification is internal to the AST; the HAVING plan-time result is correct

**HAVING names remain exempt from the aggregate-in-predicate gate** (D.7.1): `stddev`
in a HAVING predicate goes to `sql_unknown_names`, is filtered by
`DATAFUSION_BUILTIN_FUNCTION_NAMES` (DataFusion 53.1 built-in — `stddev` IS registered),
and never triggers E-QUERY-039. DataFusion resolves it correctly. No E-QUERY-001 fires.
**PERCENTILE does NOT follow this same path** — "percentile" is ABSENT from
`DATAFUSION_BUILTIN_FUNCTION_NAMES` (DataFusion 53.1 has no "percentile" built-in); see
§D.2 PERCENTILE post-blocklist-removal note (v1.4 correction F-PQLFN-P4-MED-001,
v1.5 extension F-PQLFN-P5-LOW-001 — union-membership claim now anchored to all three
registry arms: aggregate-absence 524a9986, scalar-absence bb23f143, window-absence
bb23f143, plus two presence controls 524a9986) for the verified DataFusion 53.1 behavior
and registry-dependent E-QUERY-039 outcome.

#### D.7.4 — Gate Ordering vs ADR-052

The `check_enrich_udf_availability` gate (which includes the D.7 aggregate gate) runs
BEFORE `check_temporal_literals` (ADR-052 §D4 plan-time walker) in the query execution
pipeline. Gate ordering for a predicate position:

```
E-QUERY-001 (parse) → check_enrich_udf_availability (E-QUERY-039 + D.7 aggregate gate)
  → check_query_column_availability (E-QUERY-038)
  → check_temporal_literals (ADR-052 §D4, E-QUERY-041/042)
  → DataFusion
```

If `WHERE stddev(x) = '2026-06-24'` is submitted (aggregate name + date-like RHS):
the D.7 aggregate gate fires E-QUERY-001 first. `check_temporal_literals` (ADR-052 arm 5
NonColumnLhsComparison) is never reached. No gate conflict.

#### D.7.5 — DML WHERE Gate Extension (v1.6 — F-PQLFN-P7-LOW-002)

**Finding root cause:** `build_delete_parser` and `build_update_parser` in `sql_parser.rs`
both bind `build_predicate_parser()` for their WHERE clause. The DEFECT-PQL-FNCALL-LHS-001
branch added `fn_call_comparison` to `build_predicate_parser`, which means DML WHERE now
accepts fn-call LHS. Pre-branch, `DELETE FROM t WHERE stddev(x) > 5` produced E-QUERY-001
at parse time (fn-call LHS not in grammar). Post-branch, the query parses as
`FuncCall::Scalar(Unknown("stddev"))` — but `check_enrich_udf_availability` fell to
`_ => {}` for `Ast::Sql(SqlStatement::Dml(_))` (line comment: "DML has no enrichment
syntax"), so the aggregate gate never fired and the analyst received SILENT EMPTY SUCCESS
(DML materialization no-ops to `Ok(vec![])`). This is cross-mode inconsistency: same
construct produces E-QUERY-001 in SELECT WHERE, silence in DML WHERE.

**Adjudication: Option A — extend the gate.**

Rationale for Option A over Option B (explicit out-of-scope deferral with story anchor):
- The inconsistency is a regression **introduced by this branch** — Option B would document
  around a defect we created.
- The gate IS invoked for DML queries at both `execute_inner` call sites (line 815 registry=None,
  line 918 registry=Some). The gap is solely the `_ => {}` arm.
- DML execution currently no-ops to `Ok(vec![])` — the gate addition cannot break any
  currently working DML query.
- Silent EMPTY SUCCESS is strictly worse than the prior parse-time E-QUERY-001.
- The arm shape is 4 lines — same pattern as the SELECT arm without the
  `collect_unknown_scalars_from_sql_query` call (DML has no SELECT projection, GROUP BY,
  ORDER BY, or HAVING positions to walk).

**Implementation scope for implementer:**

Add the following arm to the `match &ast` block in `check_enrich_udf_availability`
(engine.rs), in place of or before the existing `_ => {}` arm:

```rust
// DML WHERE: walk filter predicate into predicate_fncall_names.
// DML has no SELECT/GROUP BY/ORDER BY/HAVING positions — only the WHERE predicate
// is walked. Post-branch, build_predicate_parser (used by build_delete_parser and
// build_update_parser) accepts fn-call LHS via fn_call_comparison; without this arm
// the aggregate gate silently passes DML WHERE aggregates. (ADR-048 §D.7.5, OD-6)
Ast::Sql(SqlStatement::Dml(dml)) => {
    if let Some(pred) = &dml.filter {
        collect_unknown_scalar_from_predicate(pred, &mut predicate_fncall_names);
    }
}
```

E-QUERY-039 coverage is provided by the existing `predicate_fncall_names` → `sql_unknown_names`
fold at line 1979 of engine.rs — no separate `collect_unknown_scalars_from_sql_query`
call is needed.

**Required tests** (named after F-PQLFN-P7-LOW-002; add to engine.rs test module):

| Test name | Query | Expected |
|---|---|---|
| `test_f_pqlfn_p7_low_002_delete_where_aggregate_fires_e_query_001` | `DELETE FROM t WHERE stddev(x) > 5` | E-QUERY-001 (aggregate gate, no registry needed) |
| `test_f_pqlfn_p7_low_002_update_where_aggregate_fires_e_query_001` | `UPDATE t SET col = 1 WHERE variance(x) > 100` | E-QUERY-001 (aggregate gate, no registry needed) |
| `test_f_pqlfn_p7_low_002_delete_where_unknown_udf_fires_e_query_039` | `DELETE FROM t WHERE badudf(col) = 1` (with registry, badudf not registered) | E-QUERY-039 |
| `test_f_pqlfn_p7_low_002_update_where_unknown_udf_fires_e_query_039` | `UPDATE t SET col = 1 WHERE badudf(x) = 1` (with registry, badudf not registered) | E-QUERY-039 |

Note: `DELETE FROM t WHERE badudf(col) = 1` with **no registry** is NOT required to fire
E-QUERY-039 — when no registry is configured `check_enrich_udf_availability` returns
`Ok(())` early (line 1966), and the DML no-op produces `Ok(vec![])`. This matches the
behavior of unknown UDFs in SELECT WHERE with no registry (consistent).

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

- `filter_parser.rs` `build_predicate_parser` — `fn_call_comparison` production added (DEFECT-PQL-FNCALL-LHS-001); `AGGREGATE_FUNC_NAMES` `try_map` guard REMOVED (v1.2 D.7.2)
- `filter_parser.rs` `fn_call_comparison` — no longer contains parser-level aggregate blocklist; plan-time gate is the sole enforcement
- `sql_parser.rs` `build_sql_predicate_parser` — WHERE path uses `build_predicate_parser`; HAVING uses `build_having_predicate_parser`
- `sql_parser.rs` `build_having_predicate_parser` — `agg_comparison.or(base)` structure; `base` includes `fn_call_comparison` (D.7.3 passthrough for non-six-name aggregates)
- `sql_parser.rs` `build_agg_call_parser` — handles COUNT/DISTINCT_COUNT/SUM/AVG/MIN/MAX; PERCENTILE excluded (OD-2)
- `engine.rs` `collect_predicate_columns` — gains FuncCall arm in Compare branch (D.3)
- `engine.rs` `extract_field_paths_from_expr` — unchanged
- `engine.rs` `check_enrich_udf_availability` — `predicate_fncall_names` vec now populated from SQL WHERE (`sq.where_`) in `Ast::Sql` arm and SqlPipe-head WHERE (`spq.head.where_`) in `Ast::SqlPipe` arm (v1.2 D.7.1 NEW); DML WHERE (`dml.filter`) in `Ast::Sql(SqlStatement::Dml(dml))` arm (v1.6 D.7.5 NEW, OD-6)
- `engine.rs` `DATAFUSION_BUILTIN_AGGREGATE_NAMES` — sole aggregate gate; gated against `predicate_fncall_names`; HAVING names exempt
- `engine.rs` `collect_unknown_scalars_from_sql_query` — unchanged (still walks WHERE via position (b) into `sql_unknown_names`; harmless duplicate for WHERE names that survive the aggregate gate; NOT called for DML — DML has no projection/GROUP BY/HAVING positions)
- `sql_parser.rs` `build_delete_parser` / `build_update_parser` — both bind `build_predicate_parser()` for WHERE clause; post-DEFECT-PQL-FNCALL-LHS-001 branch, fn_call_comparison is in `build_predicate_parser`, so DML WHERE accepts fn-call LHS; D.7.5 gate extension ensures aggregate names are caught plan-time
- BC-2.11.016 v1.5 §EC-11-046 — accurate after implementation (D.4)

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | F-PXL3-MED-002-adr-048 | 2026-06-28 | architect | Initial ADR — HAVING/WHERE predicate grammar divergence rationale, D.1–D.6, consequences, considered alternatives. Addresses F-PXL3-MED-002 root cause analysis. |
| 1.1 | adr-048-acceptance-S-DEMO-FIDELITY-REMEDIATION-001 | 2026-06-29 | architect | PROPOSED → ACCEPTED. OD-1 ratified by user decision 2026-06-29 (Option A: extend HAVING grammar, keep WHERE E-QUERY-001). OD-2 resolved: PERCENTILE excluded from HAVING predicate grammar as accepted technical scope decision. §Resolution section added. POL-15 confirmed satisfied. `locked_decisions` populated; `open_decisions` cleared. |
| 1.2 | adr-048-v1.2-DEFECT-PQL-FNCALL-LHS-001-pass2-adjudication | 2026-07-13 | architect | DEFECT-PQL-FNCALL-LHS-001 pass-2 adjudication of F-PQLFN-P2-HIGH-001, F-PQLFN-P2-MED-001, F-PQLFN-P2-MED-002. New §D.7 (unified plan-time gate, HAVING policy, gate ordering vs ADR-052). §D.2 scope note: non-six-name aggregates parse via fn_call_comparison as FuncCall::Scalar(Unknown) — intentional. §D.6 restated: WHERE aggregate invariant covers FULL DATAFUSION_BUILTIN_AGGREGATE_NAMES (not just 7-name parser list); parser-level AGGREGATE_FUNC_NAMES blocklist removed from fn_call_comparison; plan-time gate is sole enforcement; SQL WHERE predicate positions added to predicate_fncall_names. OD-3/OD-4/OD-5 locked. cross-ref ADR-052 added to related_adrs. |
| 1.3 | adr-048-v1.3-DEFECT-PQL-FNCALL-LHS-001-pass3-adjudication | 2026-07-13 | architect | DEFECT-PQL-FNCALL-LHS-001 pass-3 adjudication of F-PQLFN-P3-LOW-001 and F-PQLFN-P3-OBS-003. §D.3 "Important:" paragraph corrected: post-D.7.2 the WHERE grammar DOES produce FuncCall LHS via fn_call_comparison; WHERE-safety derives from extract_field_paths_from_expr arg-recursion, not grammar impossibility. §D.2 PERCENTILE post-blocklist-removal note added (claim later retracted in v1.4 — see v1.4 row). §D.7.3 PERCENTILE cross-reference added. No locked-decision changes. |
| 1.4 | adr-048-v1.4-DEFECT-PQL-FNCALL-LHS-001-pass4-correction | 2026-07-13 | architect | [process-gap] F-PQLFN-P4-MED-001 correction. §D.2 PERCENTILE note retracted: v1.3 claimed "percentile IS registered in DataFusion 53.1 default_aggregate_functions()" and "present in DATAFUSION_BUILTIN_FUNCTION_NAMES" — BOTH FALSE. Proven by executed tests: test_f_pqlfn_p4_med_001_percentile_absent_from_datafusion_53_1_aggregate_registry (engine.rs datafusion_aggregate_registry_empirical_tests, commit 524a9986) and test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt (temporal_typing_tests.rs, commit 524a9986). Corrected §D.2: E-QUERY-001 does NOT fire (HAVING exempt); E-QUERY-039 outcome is registry-dependent (no registry → Ok() early return → DataFusion plan error; with registry → E-QUERY-039 fires). manual names.insert("percentile") in DATAFUSION_BUILTIN_AGGREGATE_NAMES is NECESSARY (DataFusion 53.1 has no "percentile" built-in). §D.7.3 PERCENTILE cross-reference corrected. [process-gap]: "empirical" claims in spec artifacts must cite an EXECUTED check (test name or command output), not inferred metadata — feeds S-7.02 codification. |
| 1.5 | adr-048-v1.5-DEFECT-PQL-FNCALL-LHS-001-pass5-citation-extension | 2026-07-13 | architect | F-PQLFN-P5-LOW-001 citation extension. §D.2 Mechanism 2 empirical citation for "percentile absent from DATAFUSION_BUILTIN_FUNCTION_NAMES" extended from aggregate-registry-only (v1.4) to all three union-member registries. Added scalar-registry absence lock (test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_scalar_registry, bb23f143) and window-registry absence lock (test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_window_registry, bb23f143). Five executed checks now cited: aggregate-absence (524a9986) + scalar-absence (bb23f143) + window-absence (bb23f143) + presence-control approx_percentile_cont (524a9986) + presence-control approx_distinct (524a9986). §D.7.3 cross-reference updated to cite F-PQLFN-P5-LOW-001 and enumerate all three registry arms with commit anchors. No locked-decision changes; no new ODs. |
| 1.6 | adr-048-v1.6-DEFECT-PQL-FNCALL-LHS-001-pass7-dml-gate-extension | 2026-07-13 | architect | F-PQLFN-P7-LOW-002 adjudication: DML WHERE added as sixth gated predicate position. Root cause: build_delete_parser and build_update_parser both bind build_predicate_parser; branch extension of build_predicate_parser with fn_call_comparison turned pre-branch parse-time E-QUERY-001 into post-branch SILENT EMPTY SUCCESS for DML WHERE aggregate fn-calls. Ruling: Option A (extend gate) over Option B (explicit out-of-scope deferral) — regression introduced by this branch, gate invocation already touches DML at both execute_inner call sites, DML no-ops so zero risk. New §D.7.5 documents arm shape (Ast::Sql(SqlStatement::Dml(dml)) if let Some(pred) = &dml.filter → collect_unknown_scalar_from_predicate), four required tests. §D.7.1 table extended (Position 6: SQL DML WHERE). §D.6 enumeration updated to include SQL DML WHERE. §D.3 and §D.7.2 count references updated from "five" to "six". Related Architecture Nodes updated. OD-6 locked. |
