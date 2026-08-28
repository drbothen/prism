---
document_type: adr
adr_id: "ADR-060"
title: "LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables"
status: ACCEPTED
date: "2026-08-26"
modified: "2026-08-27"
version: "1.6"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-11, SS-16]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-ENGINE-LIMIT-EARLY-STOP-001
related_adrs: [ADR-028, ADR-033]
related_bcs: [BC-2.16.002, BC-2.16.015, BC-2.01.010]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-060: LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables

## Status

ACCEPTED v1.5 (2026-08-27) — Temporal-exemption soundness redesign (§D8.9): `is_pushed_temporal_predicate` replaces `is_purely_temporal_predicate`; `Ast::Filter` + `PipeStage::Where` unconditionally SUPPRESS in `has_client_side_where`; `expr_contains_aggregate_or_window` catch-all `_ => false` → `_ => true`; `any_early_stopped` truncation-signal chain added (§D8.9). v1.4: Subsystem-anchoring correction: SS-11 + SS-07 added. v1.3: Comprehensive plan-shape surface audit. §D8.7 closes F-R12-CRIT-001
(aggregate recursion gap) and F-R12-HIGH-001 (JOIN not suppressed), plus six additional gaps
discovered by exhaustive grammar enumeration: ORDER BY aggregate escapes Condition A; Condition G
was based on `where_filters` (equality push-down map) which is always empty for `Ast::Filter` mode
and `Ast::Pipe` stages, and misses non-equality client-side predicates (CONTAINS, BETWEEN, etc.);
`PipeStage::Tail` not suppressed; `FuncCall::Window` not suppressed; no conservative default
posture for unknown AST/PipeStage variants. Gate redesigned with complete condition set A–J plus
conservative allowlist default. Signature change: `where_filters` parameter removed (gate performs
its own AST inspection). All out-of-grammar shapes documented. Verdict: surface is bounded and
complete — no deferral recommended. v1.2: §D8.7 plan-shape gate, Conditions A–G.
v1.1: §D8.1 prose correction. v1.0: initial D8 LIMIT-aware early-stop.

---

## Context

### Defect Evidence

Live monroe validation of S-CLAROTY-VULNS-001 revealed that a query
`SELECT * FROM claroty_vulnerabilities | LIMIT 1` downloads the FULL dataset (5000+
vulnerability records across multiple pages) before DataFusion applies the LIMIT clause,
consistently exceeding the 30s query budget (E-QUERY-004).

**Note on DEFECT-1 (ADR-059, WITHDRAWN):** ADR-059 is WITHDRAWN (D-2312: the h2
flow-control window hypothesis was falsified by live wire evidence; no transport change was
applied). DEFECT-2 (this ADR) is **independent** — the LIMIT over-fetching defect exists
regardless of h2 transport behavior. The original framing that implied DEFECT-2 was observed
"even after ADR-059 was applied" was imprecise and is corrected here: both defects were
investigated in the same S-CLAROTY-VULNS-001 live session, but DEFECT-2 does not depend on
DEFECT-1 having been resolved first.

Root cause: `PipelineExecutor::execute_impl` fetches ALL pages until the API signals pagination
exhaustion or the 10K DI-019 cap is hit. There is no mechanism to stop fetching when the
accumulated record count satisfies the query's LIMIT. DataFusion applies its LIMIT operator on
the final materialized record batch, too late to prevent excessive HTTP fetches.

Concretely: `claroty_vulnerabilities` page_size = 1000 rows, ~1.1 MB/page. `LIMIT 1` requires
1 record but triggers 5+ HTTP requests (~5.5 MB total). At the per-page h2 fix latency
(estimated 5-10s/page for large pages), the 30s budget is easily exceeded.

### Atomicity Reconciliation

**CRITICAL:** Before specifying the fix, the "atomic" language in existing contracts must be
adjudicated to determine compatibility.

**BC-2.16.015 §Error Cases uses "atomic-fail" in two rows:**
- `E-SENSOR-001` (HTTP non-200): "the entire fetch returns the structured error … no partial/accumulated pages are returned (atomic-fail; Option-A fail-fast)"
- `E-SPEC-018` (timestamp parse failure): "the `?` discards the entire accumulated result — the fetch fails atomically and NO partial pages are returned"

**BC-2.16.002 §Postconditions states:**
- "Partial-record discard on mid-pipeline HTTP failure: ALL records accumulated from prior successfully-completed steps are discarded … This is the 'all-or-nothing' semantic"

**Ruling — "Atomic" means all-or-nothing on HTTP ERROR, not "must fetch the entire dataset"**

The "atomic" guarantee is an ERROR-PATH invariant: when the pipeline fails mid-pagination
(HTTP non-200, network timeout, parse error), the partial result is discarded and an `Err()`
is returned. This prevents misleading partial data from reaching OCSF mappers.

LIMIT-aware early-stop is NOT an error path. It is a deliberate, successful, non-error early
exit driven by query semantics. The pipeline successfully fetches some number of COMPLETE pages,
accumulates enough records to satisfy the LIMIT, and returns those records to DataFusion. No
`Err()` is returned; no data is discarded.

Evidence that this interpretation is correct:

1. **DI-019 precedent**: the existing 10K truncation (AC-8) already halts pagination before the
   full dataset is returned, sets `truncated: true`, and returns a valid `PipelineResult`. This
   is exactly the same pattern — non-error early stop — and has never been considered a
   violation of the atomicity guarantee.

2. **Textual scope**: both atomic-fail citations appear in `§Error Cases` tables, not in
   `§Postconditions` or `§Invariants`. Their scope is explicitly about failure behavior.

3. **Design intent**: the "all-or-nothing" postcondition in BC-2.16.002 provides the rationale:
   "partial PipelineResult could mislead downstream OCSF mappers into producing schema-mismatched
   rows." This risk does not apply when early-stopping at a COMPLETE page boundary on the SUCCESS
   path — each page is fully received and parsed; no row is partially constructed.

**Compatibility verdict:** LIMIT push-down early-stop is COMPATIBLE with the existing atomicity
guarantee. BCs do not need to weaken the error-path atomic invariant; they need only add a new
postcondition describing the success-path early-stop behavior.

### Sort-Order and Fan-Out

**Sort-order:** When a query includes `ORDER BY`, DataFusion applies sorting post-fetch on the
accumulated records. LIMIT push-down may return only the first N pages (in API-declared order),
which may not be the globally-sorted first N records. This is INTENTIONAL: the engine cannot
sort across pages it has not fetched. Consumers wanting globally-sorted top-N results MUST either:
(a) omit LIMIT and use ORDER BY + explicit LIMIT post-sort, or
(b) ensure the sensor API supports server-side ORDER BY (declared via future TOML `sort_by`).
This limitation is documented as a BC postcondition.

**Fan-out (multi-sensor queries):** Each sensor pipeline in a fan-out executes independently.
LIMIT push-down applies per-pipeline independently. Each pipeline fetches the minimum pages to
satisfy the LIMIT. DataFusion applies the global LIMIT across the combined fan-out results. This
means each pipeline may return up to `LIMIT` records, and the combined result before DataFusion
trim may have up to `(sensor_count × LIMIT)` records — acceptable for the query planner.

---

## Decision

**D8 — PipelineExecutor SHALL stop fetching additional pages once accumulated records satisfy
the query's LIMIT (early-stop pagination)**

### D8.1 — LIMIT threading via FetchContext

**Gating precondition (§D8.7):** The `fetch_limit` that flows into `QueryParams.limit` is set
to 0 whenever the plan is classified as "reducing" by `ast_is_reducing_plan`. When
`fetch_limit = 0`, `QueryParams.limit = 0`, and `FetchContext::early_stop_limit = None`, so
early-stop does not fire. The threading described below applies only when the plan-shape gate
permits early-stop (i.e., `ast_is_reducing_plan` returns `false`).

A new `early_stop_limit: Option<usize>` field is added to `FetchContext`. This field is distinct
from the `query.limit` entry in `query_filters` (which is for TOML path_template interpolation,
e.g., CrowdStrike `DetectionListParams.limit`). The two are independent:
- `query_filters["query.limit"]` = limit value to inject INTO the sensor API request URL/body
- `early_stop_limit` = limit on how many records prism will accumulate before stopping pagination

`FetchContext::new()` gains a parameter `early_stop_limit: Option<usize>`. Callers
(`spec_driven_adapter.rs`) read `QueryParams.limit: u64` — the pre-extracted query LIMIT field
already present on `QueryParams` before `FetchContext` is constructed; no DataFusion physical-plan
inspection is required. Callers pass `Some(params.limit as usize)` when `params.limit > 0`, or
`None` when `params.limit == 0` (meaning no LIMIT was specified in the query). The behavior is
unchanged when no LIMIT is present.

### D8.2 — Check point in execute_impl

After each complete page is accumulated (immediately after the DI-019 truncation check), the
pagination loop adds:

```rust
if let Some(limit) = context.early_stop_limit {
    if all_records.len() >= limit {
        break 'steps;
    }
}
```

This check fires only after a COMPLETE page has been received and its records appended to
`all_records`. It does NOT fire mid-page. The page atomicity guarantee is preserved: either
the entire page arrives (and is accumulated), or a fetch error discards everything.

### D8.3 — Post-break semantics

When early-stop fires (not DI-019 cap), the `truncated` flag is NOT set. Instead,
`PipelineResult.early_stopped = true` is set (§D8.9). The pipeline returns a valid
`PipelineResult` with `truncated: false` and `early_stopped: true` containing at most
`limit + (page_size - 1)` records. DataFusion applies the precise LIMIT on this result.
The implementer MUST NOT set `truncated: true` for LIMIT early-stop — `truncated` is
semantically reserved for capacity-exceeded conditions (DI-019), not for query-driven early
stops. The `early_stopped` signal propagates to engine Step 6 where it contributes to the
`is_truncated` formula (§D8.9).

### D8.4 — Applicable pagination modes

LIMIT early-stop applies to both `PaginationConfig::OffsetLimit` and `PaginationConfig::CursorToken`
pagination modes. It does NOT apply to `PaginationConfig::None` (single-page fetch; no loop to
terminate early) or to the 10K DI-019 cap (which remains unchanged and fires before D8 when
applicable).

### D8.5 — Sort-order and ORDER BY documentation

The LIMIT early-stop postcondition in BC-2.16.002 (and relevant table BCs) MUST include: "When
ORDER BY is combined with LIMIT in the absence of server-side sort support, the engine returns
the first N records in API-declared order, which may not be the globally sorted top N. Consumers
requiring globally sorted top-N MUST omit LIMIT or ensure the sensor API returns data in the
desired sort order."

### D8.6 — timeout_secs overlay wiring: deferred to story 3

The `timeout_secs` overlay field is accepted but emits `overlay.timeout_secs_ignored` (WARN in
`overlay.rs`). Wiring it to the reqwest client requires threading the overlay timeout through
`ResolvedSensorSpec` → caller → `FetchContext` (or creating a per-org client cache with the
configured timeout). This is architecturally independent of D8 and adds complexity to
`FetchContext` that would blur the D8 change. Deferring to a separate story
`S-ENGINE-TIMEOUT-OVERLAY-WIRE-001`. Architectural direction for that story: the caller
(`spec_driven_adapter.rs`) reads `resolved_spec.provenance.timeout_secs_from_overlay` and, when
`true`, constructs a fresh reqwest client via a variant of `build_http_client_with_custom_timeout`
parameterized by the overlay timeout. The PipelineExecutor receives the correctly-configured
client; no change to `FetchContext` needed.

### D8.7 — Plan-Shape Gate for Early-Stop Suppression (v1.3 — Comprehensive Audit)

#### Problem

The MCP tool layer always sets `options.limit` to a non-zero value (default 25, user-supplied
otherwise). `run_materialization_pipeline` was deriving `fetch_limit` unconditionally from
`options.limit` without consulting the AST plan shape. This caused early-stop to fire for
reducing queries, curtailing the raw multi-page fetch BEFORE DataFusion applies the reducing
operator.

Concrete regressions (F-R11-CRIT-001): `SELECT COUNT(*) FROM claroty_vulnerabilities` returned
approximately one page worth of records instead of the true total; `GROUP BY severity` computed
group counts from a single page; queries with non-push-down WHERE predicates under-returned rows
after DataFusion filtered.

Round-12 fresh-context adversarial review found two additional reachable corruption paths
(F-R12-CRIT-001, F-R12-HIGH-001). The comprehensive audit documented in v1.3 enumerated every
grammar-expressible plan shape and found six additional gaps, documented below.

#### Out-of-Grammar Shapes (Not Gated — Confirmed by Code Inspection)

The following shapes are NOT expressible in PrismQL as of this ADR and therefore require no gate
condition. Each was explicitly checked against `ast.rs`, `sql_parser.rs`, and `pipe_parser.rs`:

| Shape | Expressible? | Notes |
|-------|-------------|-------|
| UNION / INTERSECT / EXCEPT | No | Not in grammar; no `SetOp` AST node |
| CTE (WITH clause) | No | Not in grammar; noted in `SqlPipeQuery` comment as S-3.06 future |
| FROM subquery / derived table | No | `FromClause.source` is `SourceRef`, not `SqlQuery` |
| OFFSET | No | `SqlQuery` has no `offset` field |
| Correlated subquery in FROM | No | Same as FROM subquery |

When any of the above are added to the grammar, the first implementation step MUST classify
them for the gate before they are reachable at runtime. The conservative default posture (see
below) ensures that unknown variants suppress early-stop as a safety net.

#### Complete Shape Classification Table

Every expressible plan shape classified as SUPPRESS (early-stop off) or PERMIT (early-stop on):

| Shape | Classification | Rationale / Condition |
|-------|---------------|----------------------|
| Bare projection: `SELECT * FROM t LIMIT N` | PERMIT | No reducing op; first N rows are semantically correct result |
| Projection + ORDER BY + LIMIT | PERMIT | §D8.5 accepted: returns rows in API-declared order, not globally sorted. ORDER BY does not change row count |
| Projection + temporal-only WHERE + LIMIT | PERMIT | Temporal pred fully server-side via ADR-033 T1; no client-side filter post-fetch |
| Projection + non-temporal equality WHERE + LIMIT | SUPPRESS | Condition G: client-side DataFusion filter; curtailing fetch under-returns rows |
| Projection + non-temporal non-equality WHERE (CONTAINS, BETWEEN, IN-list, CIDR, Regex, etc.) | SUPPRESS | Condition G revised: all non-temporal predicates are client-side |
| Filter mode with non-temporal predicate + LIMIT | SUPPRESS | Condition G revised: Filter mode was NOT covered by old `where_filters` check |
| Pipe where non-temporal predicate + LIMIT | SUPPRESS | Condition G revised: Pipe stages were NOT covered by old `where_filters` check |
| SQL aggregate in SELECT: `SELECT COUNT(*), MAX(x)` | SUPPRESS | Condition A: `FuncCall::Aggregate` in select items |
| SQL aggregate in ORDER BY: `SELECT * ORDER BY MAX(x)` | SUPPRESS | Condition A revised: aggregate in ORDER BY implicitly groups all rows; early-stop corrupts aggregate |
| GROUP BY (with or without visible aggregate) | SUPPRESS | Condition B: GROUP BY deduplicates and groups across full dataset |
| SELECT DISTINCT | SUPPRESS | Condition C: de-duplication requires full dataset scan |
| HAVING clause (any predicate) | SUPPRESS | Condition D: HAVING implies post-aggregation filtering; full aggregation input required |
| SQL JOIN (INNER/LEFT/RIGHT/FULL/CROSS) | SUPPRESS | Condition H: JOIN inputs independently fetched; early-stopping either input truncates join |
| SqlPipe head with SQL JOIN | SUPPRESS | Condition H: same reasoning as SQL JOIN |
| Pipe stats stage | SUPPRESS | Condition E: aggregation requires full dataset |
| Pipe dedup stage | SUPPRESS | Condition F: deduplication requires full dataset scan |
| Pipe tail stage | SUPPRESS | Condition I: selecting last N rows requires seeing all rows; early-stop severs the tail |
| Pipe join stage (currently errors; future-proofed) | SUPPRESS | Condition J: JOIN input truncation same as SQL JOIN |
| `FuncCall::Window {}` in SELECT/ORDER BY | SUPPRESS | Condition A revised: window functions compute over partitioned frames; requires full frame materialization |
| Aggregate nested inside scalar UDF arg: `severity_label(max(x))` | SUPPRESS | Condition A revised: recursion into `FuncCall::Scalar::args` required |
| InSubquery in WHERE: `WHERE f IN (SELECT ...)` | SUPPRESS | Condition G revised: IN-subquery check is client-side; early-stop under-returns matches |
| Pipe fields stage (column projection/exclusion) | PERMIT | Row count unchanged; projection is row-preserving |
| Pipe enrich stage | PERMIT | Enrichment adds columns per row; row count unchanged; applied post-fetch per row |
| Pipe sort stage | PERMIT | §D8.5: same reasoning as SQL ORDER BY; row count unchanged |
| Pipe head/limit stage | PERMIT | Explicit row limit; early-stop correctly bounds fetch |
| Pipe where temporal predicate | PERMIT | Temporal pred is server-side; no client-side filter |
| SQL DML (INSERT/UPDATE/DELETE) | SUPPRESS | Default posture: DML uses `write_pipeline.rs`, not `run_materialization_pipeline`; gate result is irrelevant but must safely return SUPPRESS for any path that reaches it |
| Unknown/future `Ast` variant | SUPPRESS | Conservative default: `_ => true` catch-all |
| Unknown/future `PipeStage` variant | SUPPRESS | Conservative default: stage loop falls through to SUPPRESS |

#### Enforcement Site

`materialization.rs::run_materialization_pipeline` — at the `fetch_limit` derivation, BEFORE
fan-out targets are constructed. The gate is a guard on the single `fetch_limit` binding.

Subsystem scope: SS-11 (Query Execution) owns `run_materialization_pipeline` — the `fetch_limit`
derivation and `ast_is_reducing_plan` call site. SS-07 (Adapter Pagination & Response Cache) owns
`execute_impl` — the per-page early-stop check (§D8.2) — and the response-cache-key coherence
path where `fetch_limit` is the cache-key limit component (§D8.8).

Note: `where_filters` (the `FilterMap` from `extract_push_down_filters_as_map`) is no longer
passed to `ast_is_reducing_plan`. The gate performs its own AST inspection for client-side
predicate detection. `where_filters` continues to be computed and used for push-down and cache
key derivation; it is simply not forwarded to the gate.

#### Gate Function Signature (v1.3)

```
pub(crate) fn ast_is_reducing_plan(ast: &Ast) -> bool
```

The `where_filters: &FilterMap` parameter present in v1.2 is REMOVED. The gate was never
correctly reading Filter-mode or Pipe-mode predicates via `extract_push_down_filters_as_map`
(that function only processes `Ast::Sql` and `Ast::SqlPipe` head WHERE; it returns an empty map
for Filter and Pipe modes). The gate must perform its own AST walk via `has_client_side_where`.

#### Supporting Function: `expr_contains_aggregate_or_window`

This replaces the v1.2 `expr_contains_aggregate` function. The name change signals the expanded
scope: window functions are now also detected and cause suppression.

```
fn expr_contains_aggregate_or_window(expr: &Expr) -> bool
```

**Returns `true` for:**
- `Expr::FuncCall(FuncCall::Aggregate { .. })` — direct aggregate call
- `Expr::FuncCall(FuncCall::Window { .. })` — window function stub (S-3.06); full frame
  required regardless of field count in stub

**Recurses into sub-expressions for:**
- `Expr::FuncCall(FuncCall::Scalar { args, .. })` — recurse into every element of `args`
  (F-R12-CRIT-001 root cause: `severity_label(max(severity_id))` escaped detection because the
  outer `Scalar` was not recursed)
- `Expr::Compare { lhs, rhs, .. }` — recurse into both
- `Expr::Logical { lhs, rhs, .. }` — recurse into both
- `Expr::Not(inner)` — recurse
- `Expr::TimestampArithmetic { base, .. }` — recurse into `base`
- `Expr::InSubquery { .. }` — the subquery's SELECT is a separate `SqlQuery`; the outer `Expr`
  does not directly contain the subquery's aggregate. Return `false` here (the subquery's
  aggregation is independent of the outer plan's row count). The IN condition itself is caught
  by `has_client_side_where` via Condition G.

**Returns `false` (leaf, no recursion) for:**
`Expr::Literal`, `Expr::Field`, `Expr::VirtualField`, `Expr::Star`, `Expr::Now`,
`Expr::Interval`, `Expr::In { .. }` (literal values, no sub-expressions)

**Conservative catch-all:** `_ => true` for all unknown future `Expr` variants. Known
non-aggregate leaf variants (`Expr::Literal`, `Expr::Field`, `Expr::VirtualField`, `Expr::Star`,
`Expr::Now`, `Expr::Interval`, `Expr::In`) are enumerated explicitly returning `false`; unknown
or future `Expr` variants (e.g., a CASE expression) are treated as potentially-aggregate →
SUPPRESS. This extends the conservative-default posture to the Expr-recursion level: the
terminal arm MUST be `_ => true`, NOT `_ => false`. The prior v1.3 description erroneously
stated `_ => false` (leaf assumption); v1.5 corrects this per F-R14-LOW-001. For `FuncCall`
variants specifically, the catch-all is also `_ => true` (unknown function call types may be
aggregates; conservative suppression preferred over a false PERMIT). (Anchored:
S-ENGINE-LIMIT-EARLY-STOP-001 AC-007; correctness enforced by exhaustive explicit enumeration
of all known non-aggregate leaf Exprs returning `false` — any unlisted variant hits `_ => true`.)

#### Supporting Function: `has_client_side_where`

```
fn has_client_side_where(ast: &Ast, datetime_index_cols: &[&str]) -> bool
```

Returns `true` iff any WHERE-position predicate in the AST will be applied client-side by
DataFusion after fetching (i.e., is NOT guaranteed to be fully resolved server-side).

Only temporal range predicates on INDEX datetime columns with concrete `Literal::Timestamp` RHS,
as determined by `is_pushed_temporal_predicate(pred, datetime_index_cols)` (§D8.9), are
guaranteed server-side for `Ast::Sql` and `Ast::SqlPipe` head WHERE. **`Ast::Filter` predicates
and `Ast::Pipe / Ast::SqlPipe` pipe-stage WHERE predicates are ALWAYS client-side regardless of
predicate form** (v1.5 unconditional suppression; see arm descriptions below). All other
predicate forms — equality comparisons, IN lists, `InSubquery`, CONTAINS/STARTSWITH/ENDSWITH
(StringOp), BETWEEN, CIDR, Regex, Has, Missing, IsNull, Wildcard, and any logical combinations
— are client-side.

**AST-mode dispatch:**

- `Ast::Filter(f)`: returns `true` UNCONDITIONALLY for all filter-mode predicates, including
  purely temporal ones. `extract_time_bounds_from_predicate` (ADR-033 T1) does NOT process
  `Ast::Filter` mode — temporal predicates in filter-mode queries are evaluated client-side by
  DataFusion after the full fetch, not server-side. The v1.3 `!is_purely_temporal_predicate`
  check for this arm was UNSOUND and is removed in v1.5; closes F-R15-LENSA-CRIT-001
  (filter-mode path). Note: the v1.2 `where_filters` approach was also INCORRECT for this mode —
  `extract_push_down_filters_as_map` always returned an empty map for `Ast::Filter`.

- `Ast::Sql(SqlStatement::Select(sql))`: returns
  `sql.where_.as_ref().map(|p| !is_pushed_temporal_predicate(p, datetime_index_cols)).unwrap_or(false)`.
  When the WHERE clause is absent, returns `false` (no client-side filter). When present,
  PERMIT (`false`) only if `is_pushed_temporal_predicate` determines the whole predicate is
  a fully server-side temporal range on an INDEX datetime column with concrete `Literal::Timestamp`
  RHS. Note: the v1.2 `where_filters` approach was correct for the equality-predicate sub-case
  but missed non-equality client-side predicates (CONTAINS, BETWEEN, etc.).

- `Ast::Pipe(pipe)`: returns `true` UNCONDITIONALLY whenever any `PipeStage::Where(_)` is
  present in `pipe.stages`, regardless of predicate form. Pipe `| where` stages push NOTHING
  server-side; `PipeStage::Where` is removed from the PERMIT allow-list in v1.5. The v1.3
  `!is_purely_temporal_predicate(pred)` check for this arm was UNSOUND because `Ast::Pipe`
  predicates are never resolved server-side by `extract_time_bounds_from_predicate`; closes
  F-R15-LENSA-CRIT-001 (pipe-mode path). Note: the v1.2 `where_filters` approach was also
  INCORRECT — `extract_push_down_filters_as_map` always returned an empty map for `Ast::Pipe`.

- `Ast::SqlPipe(spq)`: returns `true` iff (`spq.head.where_` is present AND
  `!is_pushed_temporal_predicate(where_pred, datetime_index_cols)`) OR any `PipeStage::Where(_)`
  is present in `spq.stages` (pipe-WHERE stages are unconditionally suppressed in v1.5;
  see `Ast::Pipe` arm rationale above).

- `Ast::Sql(SqlStatement::Dml(_))` and `_ =>`: returns `false` (DML does not use this
  pipeline; unknown variants handled by the outer gate's `_ => true` catch-all).

**`is_pushed_temporal_predicate(pred: &Predicate, datetime_index_cols: &[&str]) -> bool`:**
Returns `true` (PERMIT early-stop for the calling WHERE clause) iff the predicate is fully
handled server-side by the ADR-033 T1 temporal push-down mechanism. Mirrors
`extract_time_bounds_from_predicate` exactly. Replaces the v1.3 `is_purely_temporal_predicate`
which unsoundly permitted `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic` (relative
time expressions evaluated post-fetch) and non-INDEX datetime columns.

**Returns `true` (PERMIT) iff ALL THREE preconditions hold:**
1. **Range operator:** `Gt | Ge | Lt | Le` — NOT `Eq` or `Ne`. Temporal equality predicates
   (`timestamp = X`) are not extractable by `extract_time_bounds_from_predicate` and remain
   client-side.
2. **LHS is an INDEX datetime column:** `Expr::Field(name)` where `name` appears in
   `datetime_index_cols` (columns declared `index: true` + `column_type = "Datetime"` in sensor
   TOML). Non-INDEX datetime columns are not pushed server-side.
3. **RHS is a concrete absolute timestamp:** `Expr::Literal(Literal::Timestamp)`. Relative
   expressions — `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic` — are evaluated
   by DataFusion after fetch, not by the server.

**`Predicate::Logical { op: AND, lhs, rhs }`:** Recurses — returns `true` only if BOTH
`is_pushed_temporal_predicate(lhs, datetime_index_cols)` AND
`is_pushed_temporal_predicate(rhs, datetime_index_cols)` are `true`. Models AND-combined
temporal ranges (e.g., `timestamp >= X AND timestamp < Y`) that `extract_time_bounds_from_predicate`
can fully push server-side.

**All other predicates return `false` (SUPPRESS):**
- Temporal equality (`Eq` operator): not range-extractable
- `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic`: relative; evaluated post-fetch
- LHS field not in `datetime_index_cols`: non-INDEX columns not pushed server-side
- `Predicate::Logical { op: OR, .. }`: OR-combined predicates not handled by
  `extract_time_bounds_from_predicate`; conservative suppression
- Any other predicate form: conservative suppression

#### Complete Condition Set

`ast_is_reducing_plan(ast: &Ast) -> bool` returns `true` (SUPPRESS) when ANY of the following
holds:

**Condition A — Aggregation or window function in SELECT items or ORDER BY expressions
(revised from v1.2; closes F-R12-CRIT-001):**
- Any `SelectItem::Expr { expr, .. }` in `select.items` (SQL) or `head.select.items` (SqlPipe)
  where `expr_contains_aggregate_or_window(expr)` returns `true`.
- Any `OrderExpr { expr, .. }` in `order_by` (SQL or SqlPipe head) where
  `expr_contains_aggregate_or_window(expr)` returns `true`.
  Rationale: `ORDER BY MAX(severity)` without GROUP BY performs a global aggregation; early-stop
  corrupts the result. ORDER BY uses the same `Expr` parser as SELECT, so aggregate calls ARE
  parseable in ORDER BY position.
- Applies to: `Ast::Sql(SqlStatement::Select(sql))` and `Ast::SqlPipe(spq)` (head).

**Condition B — GROUP BY (unchanged from v1.2):**
`sql.group_by.is_empty() == false` (or `spq.head.group_by`). GROUP BY groups across the full
dataset; early-stop yields incorrect group membership and counts.

**Condition C — DISTINCT (unchanged from v1.2):**
`sql.select.distinct == true` (or `spq.head.select.distinct`). De-duplication requires a full
dataset scan; early-stop produces false-unique results.

**Condition D — HAVING (unchanged from v1.2):**
`sql.having.is_some()` (or `spq.head.having`). HAVING always implies post-aggregation filtering.
Conservatively suppresses even `HAVING non_agg_expr` (which is unusual SQL), because HAVING is
semantically coupled to GROUP BY / aggregation and full-dataset evaluation.

**Condition E — Pipe Stats stage (unchanged from v1.2):**
Any `PipeStage::Stats(_)` in `pipe.stages` or `spq.stages`. Aggregation requires full dataset.

**Condition F — Pipe Dedup stage (unchanged from v1.2):**
Any `PipeStage::Dedup(_)` in `pipe.stages` or `spq.stages`. Deduplication requires full dataset.

**Condition G — Client-side WHERE predicate (revised from v1.2):**
`has_client_side_where(ast)` returns `true`. This replaces the insufficient
`!where_filters.is_empty()` check. The old check failed for three cases: (1) `Ast::Filter` mode
(always returned empty `where_filters`), (2) `Ast::Pipe` stages (always returned empty
`where_filters`), and (3) non-equality SQL WHERE predicates (CONTAINS, BETWEEN, IN-list, CIDR,
Regex, Has, Missing, etc.) that are client-side DataFusion filters but not equality predicates.
The new check directly inspects the AST predicate form.

ADR-033 cross-reference: temporal range predicates (extracted by
`extract_time_window_from_ast_from_query`) are fully server-side via T1 push-down and do not
suppress early-stop. All other predicate forms are treated as client-side until Wave 5
per-sensor push-down classification (ADR-033 extension) enables the engine to classify
individual equality predicates as server-side.

**Condition H — SQL JOIN (new; closes F-R12-HIGH-001):**
`!sql.joins.is_empty()` (or `!spq.head.joins.is_empty()`). Each table in a JOIN is an
independent fan-out target fetched separately. Early-stopping the raw fetch from any source
truncates that input before DataFusion applies the JOIN. The resulting joined set is computed
over incomplete inputs — missing rows from the truncated side that would have matched are
silently absent.
Applies to all JOIN kinds (`Inner`, `Left`, `Right`, `FullOuter`, `Cross`).

**Condition I — Pipe Tail stage (new):**
Any `PipeStage::Tail(_)` in `pipe.stages` or `spq.stages`. `| tail N` semantically selects the
LAST N rows of the dataset. Early-stop fetches only the first few pages, making the last N rows
of the fetched subset the last N of a truncated dataset — not the true tail of the full dataset.
Note: the current `pipe_sql_emitter.rs` lowers `PipeStage::Tail(N)` to `LIMIT N` (known semantic
gap §3.2). Under the lowered form, suppression is technically immaterial for the current behavior
(LIMIT N on partial or full dataset both give the first N rows). Condition I is specified for
correctness when Tail is properly implemented.

**Condition J — Pipe Join stage (new; closes F-R12-HIGH-001 for pipe mode):**
Any `PipeStage::Join(_)` in `pipe.stages` or `spq.stages`. Same reasoning as Condition H.
Note: `pipe_sql_emitter.rs` currently returns an error for `PipeStage::Join` (not yet supported,
ENRICH-4-C), so this condition is defensive and future-proof. When Pipe Join is implemented, the
gate MUST already suppress early-stop for it.

#### Conservative Default Posture (new in v1.3)

The `Ast` enum, `PipeStage` enum, and `FuncCall` enum are all `#[non_exhaustive]`. New variants
may be added without a compile error in the gate's `match` arm. The default posture is:

**SUPPRESS (return `true`) for any unknown or unclassified variant.**

Implementation:
- `ast_is_reducing_plan`: the outer `match ast { ... _ => true }` arm suppresses for any future
  `Ast` variant not explicitly listed.
- Pipe-stage scan loop: unknown `PipeStage` variants trigger suppression via `_ => return true`
  (or an allowlist of known-safe stages: `PipeStage::Where`, `PipeStage::Sort`,
  `PipeStage::Limit`, `PipeStage::Fields`, `PipeStage::Enrich` — any stage not in this list
  suppresses early-stop).
- `FuncCall` catch-all in `expr_contains_aggregate_or_window`: `FuncCall::_unknown => true`
  (unknown function calls may be aggregates; suppress conservatively).

**Rationale for conservative default (allowlist over denylist):** The cost of incorrect
SUPPRESS is degraded performance (full pagination instead of early-stop). The cost of incorrect
PERMIT is a silent correctness regression: `truncated=false` results computed over a partial
dataset with no signal to the consumer. Given the asymmetric cost, the gate defaults to SUPPRESS
for all uncertainty. Only shapes explicitly proven safe receive PERMIT.

#### ORDER BY Does NOT Suppress Early-Stop (§D8.5 Preserved — Unchanged)

`PipeStage::Sort` and `SqlQuery::order_by` alone (without an aggregate in the ORDER BY
expression) are NOT suppression conditions. This preserves the §D8.5 accepted limitation.

A bare projection with ORDER BY + LIMIT returns records in API-declared order within the fetched
subset, which may not be the globally sorted top N. This is the §D8.5 accepted trade-off.

See §Alternatives Alt-D for the rejected alternative of suppressing early-stop for ORDER BY.

IMPORTANT: `ORDER BY aggregate_fn(col)` WITHOUT GROUP BY DOES suppress early-stop via
Condition A (aggregate in ORDER BY). The non-suppression applies only to ORDER BY expressions
that contain no aggregate or window function.

#### Gate Application in `run_materialization_pipeline` (v1.3)

```
// Plan-shape gate (ADR-060 §D8.7): suppress early-stop for reducing plans.
// Note: where_filters is NOT passed — gate performs its own AST inspection.
let fetch_limit: u64 = if ast_is_reducing_plan(&ast) {
    0 // suppress: reducing plan needs full pagination for correctness
} else {
    options.limit.map(|l| l as u64).unwrap_or(0)
};
```

The `0` sentinel flows unchanged through the existing pipeline:
- `QueryParams.limit = 0` → `FetchContext::early_stop_limit = None` (per existing
  `if params.limit == 0 { None }` mapping in `spec_driven_adapter.rs`)
- `FetchContext::early_stop_limit = None` → early-stop check in `execute_impl` does not fire
- Full pagination proceeds up to the DI-019 10K cap, as before this story

### D8.8 — Single-Binding Coherence with Plan-Shape Gate

The existing SINGLE-BINDING COHERENCE invariant (comment in `run_materialization_pipeline`:
"this binding feeds BOTH the response-cache key derivation AND the fan-out target construction")
is preserved by the plan-shape gate.

When `fetch_limit = 0` (reducing plan), the response-cache key uses 0 as the limit component.
This means all reducing-plan queries with the same filters and time window share a cache entry
that holds the full dataset (fetched without early-stop). A `SELECT COUNT(*)` and a
`SELECT COUNT(*) | LIMIT 25` both receive `fetch_limit = 0` and share a cache entry — correct,
since both need the full dataset.

When `fetch_limit = N > 0` (non-reducing plan, gate permits early-stop), the cache key uses N.
Different LIMIT values produce different cache entries — correct, since `LIMIT 10` and `LIMIT 100`
may stop at different pages.

The v1.3 signature change (removal of `where_filters` parameter from `ast_is_reducing_plan`)
does not affect coherence. `where_filters` continues to be computed and used in the cache key
derivation; it is no longer forwarded to the gate function. The single `fetch_limit` binding
remains the sole source feeding both cache key and `QueryParams.limit`.

### D8.9 — `any_early_stopped` Truncation-Signal Propagation Chain and `datetime_index_cols` Threading

#### Motivation

When the §D8.2 early-stop `break 'steps` fires at the exact-limit boundary
(`all_records.len() == limit`, so `total_rows == limit`), the naive formula `total_rows > limit`
evaluates to `false`. Without a separate signal, engine Step 6 would emit `is_truncated: false`
— silently hiding that pagination was halted before dataset exhaustion. A consumer receiving
`is_truncated: false` at the exact-limit boundary has no signal that more data may exist.

#### `PipelineResult.early_stopped: bool`

When the §D8.2 `break 'steps` fires (early-stop, NOT DI-019), `PipelineResult.early_stopped = true`
is set. This field is DISTINCT from `truncated`: `truncated` signals DI-019 capacity overflow
(§D8.3 invariant: implementer MUST NOT set `truncated` on early-stop); `early_stopped` signals
a query-driven early exit at the limit boundary.

#### `FetchOutput` Return Type

`SensorAdapter::fetch` return type changes to carry the early-stop signal out of the per-sensor
pipeline and into the fan-out layer:

```rust
pub struct FetchOutput {
    pub batches: Vec<RecordBatch>,
    pub any_early_stopped: bool,
}
```

`any_early_stopped` is set from `PipelineResult.early_stopped` of the sensor's pipeline
execution.

#### Propagation Chain

The `any_early_stopped` signal propagates from the per-sensor level to engine.rs Step 6:

```
PipelineResult.early_stopped
  → FetchOutput.any_early_stopped
  → FanOutResult.any_early_stopped   (OR-combined across all sensors in the fan-out)
  → MaterializationOutput.any_early_stopped
  → engine.rs Step 6: is_truncated = (total_rows > limit) || any_early_stopped
```

#### `is_truncated` Formula at Step 6 (BC-2.11.001 EC-11-092)

```rust
let is_truncated = total_rows > limit || materialization_output.any_early_stopped;
```

When `total_rows == limit` (exact-limit boundary) AND `any_early_stopped = true`:
- `total_rows > limit` = false
- `any_early_stopped` = true
- Result: `is_truncated = true` — correctly signals to the MCP consumer that pagination was
  halted and more data may be available.

`total_available` is a LOWER BOUND when `any_early_stopped = true`: the true dataset size is
unknown because pagination was stopped before exhaustion.

#### Step 6 is the SOLE Owner of Tool-Level Cap (BC-2.11.001 EC-11-093)

`run_materialization_pipeline` MUST return the full filtered/aggregated result set to engine.rs
Step 6 WITHOUT applying a tool-level pre-cap. Engine.rs Step 6 reads the full pre-cap row count
from the materialization output, computes `total_available`, sets `is_truncated`, and then
applies the cap. A `truncate_result_to_limit` pre-cap inside `run_materialization_pipeline`
causes Step 6 to see the pre-capped count as `total_available`, silently producing
`is_truncated: false` when the unfiltered count exceeds the tool limit (F-R13-CRIT-001
prohibited behavior).

The `fetch_limit` binding controls ONLY the early-stop check in the pagination loop; it does
NOT authorize `run_materialization_pipeline` to cap the result set returned to Step 6.
(Anchored: RG-PSG-025 `test_psg_exact_limit_is_truncated_true`)

#### `datetime_index_cols` Threading

`has_client_side_where(ast, datetime_index_cols)` (§D8.7) receives `datetime_index_cols` from
`run_materialization_pipeline`. The caller derives `datetime_index_cols` from the resolved
sensor spec: the set of column names declared `index: true` AND `column_type = "Datetime"` in
the sensor TOML. These are the columns whose temporal range predicates ADR-033 T1 pushes
server-side. The parameter is passed through to `is_pushed_temporal_predicate` at the predicate
inspection level.

---

## Rationale

**Why stop at COMPLETE page boundaries:** Stopping mid-page would violate the atomicity
guarantee (partially-received page → partial records, potential schema mismatch). Stopping only
at complete-page boundaries preserves the invariant that every record in `all_records` was
fully received and parsed.

**Why DataFusion applies precise LIMIT post-fetch:** The engine cannot know which record within
the first overfull page satisfies the LIMIT exactly. Fetching one complete page and letting
DataFusion trim is the cleanest separation of concerns: the pipeline layer handles transport;
the query layer handles record-level selection.

**Why not push LIMIT into the API request body/URL for OffsetLimit sensors:** For Claroty
`vulnerabilities` (POST body injection, page_size = 1000), pushing `limit = 1` into the API
would fetch a 1-record page, which is a different API call with potentially different
server-side behavior. The canonical mechanism for single-record fetches is `LIMIT` at the
DataFusion layer. The page_size in the TOML is calibrated for efficient batched fetching.

---

## Consequences

### Positive
- `SELECT ... FROM claroty_vulnerabilities | LIMIT N` (small N) fetches only `ceil(N / 1000)`
  pages instead of the full dataset. `LIMIT 1` → 1 page (~1.1 MB, ~5s) instead of 5+ pages
  (~5.5 MB, >30s). Unblocks S-CLAROTY-VULNS-001 live green.
- Applies sensor-agnostically to ALL offset/limit and cursor-paginated tables in the engine.
  CrowdStrike, Armis, Cyberint sensor queries with LIMIT benefit automatically.
- No behavioral change when LIMIT is absent (`early_stop_limit = None`); full pagination
  proceeds as before.
- DI-019 10K cap is unchanged; it continues to fire as the outer safety net.

### Negative / Trade-offs
- `FetchContext::new()` signature expands by one parameter. All callers must be updated.
  Currently one production caller (`spec_driven_adapter.rs`). The `#[non_exhaustive]` on
  `FetchContext` prevents external struct-literal construction; the `new()` function change
  is a breaking change for downstream code using the public `new()` constructor. Acceptable
  given the engineering need.
- `ast_is_reducing_plan` signature changes from `(&Ast, &FilterMap) -> bool` to `(&Ast) -> bool`
  in v1.3. The call site in `run_materialization_pipeline` and any existing Red Gate tests that
  pass `where_filters` must be updated. The BC-2.16.002 postcondition text must be amended by
  the product owner to reflect the new signature and the revised Condition G.
- When LIMIT early-stop fires and the first page has 1000 rows but LIMIT is 1, DataFusion
  materializes 999 unnecessary records before discarding them. This is the irreducible cost
  of page-granularity stopping. For typical LIMIT values (5–100) against page_size = 1000,
  the overhead is negligible compared to avoiding the extra HTTP fetches.
- Queries combining ORDER BY + LIMIT (without aggregate in ORDER BY) do NOT return globally
  sorted top-N (documented in D8.5). This is an expected limitation of a federated query engine
  without server-side sort propagation.
- Queries with any non-temporal WHERE predicate (including non-equality forms like CONTAINS,
  BETWEEN, etc., plus ALL predicates in filter-mode and pipe-stage WHERE) have early-stop
  suppressed (§D8.7 Condition G revised). They paginate fully to the DI-019 10K cap. This is
  the correct safe scope until Wave 5 per-sensor push-down classification (ADR-033 extension)
  can identify which predicates are fully server-side. At that point, Condition G can be
  refined to exempt proven server-side predicates.
- Queries with SQL JOINs or Pipe Join stages also have early-stop suppressed (Conditions H, J).
  Both join partners paginate fully to DI-019, which matches pre-story behavior.
- Early-stop's performance benefit is scoped to bare projections with no WHERE clause (or
  temporal-only WHERE), no JOINs, no reducing operators: `SELECT [cols/*] FROM table LIMIT N`
  and its pipe-mode equivalents. This is the correct safe scope; the conservative gate ensures
  no silent correctness regressions for the SOC-analyst aggregation and filtered queries that
  are the v1 core use case.

---

## Alternatives Considered

**Alt-A: Push LIMIT into OffsetLimit POST body as the page_size** — Rejected. Changing
`page_size` from 1000 to 1 is a different API semantic: the server may enforce minimum
page sizes, respond differently for tiny page requests, or charge differently per call.
The TOML page_size is a calibrated transport parameter; it should not be overridden by
query semantics.

**Alt-B: Two-level truncation (record-level early-stop)** — Rejected. Stopping mid-page
after appending N records from a page of M records would violate the page-atomicity guarantee.
All records from a received page are either kept or discarded together.

**Alt-C: Engine-level LIMIT annotation on sensor table scans** — Considered as an alternative
to FetchContext threading. DataFusion supports custom TableProvider with LIMIT pushdown hints.
Rejected for this iteration: it requires a more significant refactor of `SensorTableProvider`
and is architectural scope for a separate effort. FetchContext threading is the minimum viable
mechanism; the TableProvider approach is a future optimization.

**Alt-D: Suppress early-stop for ORDER BY + LIMIT** — Rejected. Suppressing early-stop whenever
`order_by` is non-empty would require a full-dataset scan before sorting, eliminating the
optimization for common "show recent N" queries. The §D8.5 accepted-limitation (results in
API-declared order, not globally sorted) is preferable for queries like
`SELECT * FROM alerts ORDER BY severity LIMIT 100` where the consumer wants any 100 alerts
sorted by severity, not specifically the globally-ranked top 100. Consumers needing globally
ranked top-N should use time-window predicates to bound the dataset server-side, or use ORDER BY
without LIMIT (accepting full-scan cost). ORDER BY is different from GROUP BY/aggregation: it
does not reduce the row count or change semantic correctness of a "return N rows" request —
it only changes which N rows are returned.

---

## Source / Origin

DEFECT-2 (S-CLAROTY-VULNS-001 live monroe validation, 2026-08-26). The `| LIMIT 1` query
exhausted the 30s query budget fetching the full `claroty_vulnerabilities` dataset. ADR-059
is WITHDRAWN (D-2312: h2 flow-control window hypothesis falsified; no transport change was
adopted); DEFECT-2 is independent — the LIMIT over-fetching defect was observed in isolation
and does not depend on any h2 fix. The DI-019 precedent (10K truncation as non-error early
stop) confirmed that page-boundary early stopping is consistent with the existing atomicity
contract.

**F-R11-CRIT-001** (LOCAL cascade round-11, 2026-08-26): early-stop was firing for reducing
queries (`SELECT COUNT(*)`, `GROUP BY severity`, WHERE-filtered projections) because
`fetch_limit` was derived unconditionally from `options.limit` (always non-zero on the MCP
path, default 25). The plan-shape gate in §D8.7 closes this regression.

**F-R12-CRIT-001** (LOCAL cascade round-12, 2026-08-26): `expr_contains_aggregate` did not
recurse into `FuncCall::Scalar` arguments. A query `SELECT severity_label(max(severity_id))
FROM t LIMIT 5` escaped Condition A. Closed by v1.3 (renamed to
`expr_contains_aggregate_or_window`; recursion into `FuncCall::Scalar::args` added).

**F-R12-HIGH-001** (LOCAL cascade round-12, 2026-08-26): SQL JOINs and Pipe Join stages were
not suppression conditions. `SELECT * FROM a JOIN b ON a.id = b.id LIMIT 5` erroneously
permitted early-stop, truncating the join inputs. Closed by v1.3 (Conditions H and J).

**v1.3 comprehensive audit (2026-08-27)**: Human-directed exhaustive grammar enumeration
found six additional gaps: Condition A did not scan ORDER BY expressions; Condition G was
based on `where_filters` (always empty for Filter/Pipe modes, and missing non-equality
SQL predicates); PipeStage::Tail not suppressed; FuncCall::Window not suppressed; no
conservative default posture. All closed in v1.3.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.6 | 2026-08-27 | architect | MED-001 (F-R16-P1-MED-001): ADR-059 citation reframe — §Context §Defect Evidence and §Source/Origin clauses corrected. ADR-059 is WITHDRAWN (hypothesis falsified, D-2312); DEFECT-2 (this ADR) is independent and was observed in isolation. The prior framing implied DEFECT-2 was contingent on DEFECT-1 being applied first; that is false — the LIMIT over-fetching defect exists regardless of h2 transport behavior. No behavioral decision changes. |
| 1.5 | 2026-08-27 | architect | Temporal-exemption soundness redesign (§D8.9): `is_pushed_temporal_predicate(pred, datetime_index_cols: &[&str])` replaces `is_purely_temporal_predicate`; mirrors `extract_time_bounds_from_predicate` (ADR-033 T1) exactly — requires range op (Gt/Ge/Lt/Le) + LHS in `datetime_index_cols` (INDEX datetime col) + RHS `Expr::Literal(Literal::Timestamp)`. `Ast::Filter` unconditionally SUPPRESS in `has_client_side_where` (closes F-R15-LENSA-CRIT-001 filter-mode path). `PipeStage::Where` unconditionally SUPPRESS in `has_client_side_where` (closes F-R15-LENSA-CRIT-001 pipe-mode path). `expr_contains_aggregate_or_window` catch-all corrected: `_ => false` (stale) → `_ => true` (conservative SUPPRESS; per F-R14-LOW-001). `datetime_index_cols: &[&str]` param threaded through `has_client_side_where` and `is_pushed_temporal_predicate`. §D8.9 `any_early_stopped` truncation-signal propagation chain: `PipelineResult.early_stopped` → `FetchOutput { batches, any_early_stopped }` → `FanOutResult.any_early_stopped` → `MaterializationOutput.any_early_stopped` → engine Step 6 `is_truncated = (total_rows > limit) \|\| any_early_stopped` (closes F-R15-LENSA-HIGH-001 exact-limit boundary). |
| 1.4 | 2026-08-27 | architect | Subsystem-anchoring correction (F-R13-LENSC-HIGH-001): SS-11 (Query Execution) and SS-07 (Adapter Pagination & Response Cache) added to `subsystems_affected`. SS-11 owns `prism-query::materialization.rs` — the `fetch_limit` derivation and plan-shape gate enforcement site (§D8.7). SS-07 owns `execute_impl` — the per-page early-stop check (§D8.2) — and the response-cache-key coherence path where `fetch_limit` is the cache-key limit component (§D8.8). No behavioral change; frontmatter correction only. |
| 1.3 | 2026-08-27 | architect | §D8.7 comprehensive plan-shape surface audit. Closes F-R12-CRIT-001 (aggregate recursion gap: `expr_contains_aggregate_or_window` now recurses into `FuncCall::Scalar` args and detects `FuncCall::Window`). Closes F-R12-HIGH-001 (SQL JOIN → Condition H; Pipe Join stage → Condition J). Six additional gaps closed: Condition A extended to scan `order_by` expressions; Condition G redesigned — replaced `where_filters` (equality push-down map, always empty for Filter/Pipe modes) with `has_client_side_where()` covering all four AST modes and all non-temporal predicate forms; Condition I added (PipeStage::Tail); conservative default posture added (`_ => true` catch-all for unknown AST/PipeStage variants). Signature change: `where_filters` parameter removed — gate performs its own AST inspection. Out-of-grammar shapes documented (UNION/INTERSECT/EXCEPT, CTEs, FROM subquery, OFFSET: not gated). Complete shape-classification table added. §D8.7 replaced in full; §D8.8 coherence note updated for new signature; §Consequences updated; §Source updated. |
| 1.2 | 2026-08-26 | architect | §D8.7 plan-shape gate: closes F-R11-CRIT-001. Suppresses early-stop (`fetch_limit=0`) for reducing plans (aggregation, GROUP BY, DISTINCT, HAVING, Stats, Dedup, non-temporal WHERE). §D8.1 annotated with gating precondition. §D8.8 single-binding coherence clarification. §Consequences and §Alternatives updated. |
| 1.1 | 2026-08-26 | architect | §D8.1 prose correction: LIMIT is read from `QueryParams.limit: u64` (pre-extracted; 0 = no limit), not from DataFusion physical-plan inspection. Behavioral decision D8 unchanged. |
| 1.0 | 2026-08-26 | architect | Initial — D8 LIMIT-aware early-stop pagination, atomicity reconciliation ruling, timeout_secs deferral. |
