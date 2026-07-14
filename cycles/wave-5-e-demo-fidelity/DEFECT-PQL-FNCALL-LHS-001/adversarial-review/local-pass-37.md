---
pass: 37
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 9745372c
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: false
finding_count: 1
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: 29
fix_burst_commits: [0749f16e]
fix_burst_new_frozen_head: 0749f16e
---

# LOCAL Adversary Pass 37 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 9745372c (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — 1 MED finding
**CLEAN(PR-merge):** NO — 1 MED finding (MED blocks PR-merge gate per BC-5.39.001 §CLEAN(PR-merge): zero CRIT+HIGH+MED required)

**Streak:** stays 0/3 on frozen 9745372c; RESET to 0/3 on new frozen 0749f16e after fix-burst-29
**SAP-1:** PASS (zero net-new event_type emissions; fix-burst-29 is comment/docstring only)
**Status:** CLOSED — fix-burst-29 COMPLETE @0749f16e (D-1755 2026-07-14)

---

## Fix-burst-28 Closure Verification

Before enumerating new findings, this pass verified all fix-burst-28 closures against frozen 9745372c:

- **Docstring seven-walk table accuracy:** `check_enrich_udf_availability` 7-row table verified — all seven Position/Grammar Surface/Walk Path/Gate Function cells accurate against code arms. No invented surfaces; Positions 6/7 DML arms present and correct.
- **Offset truthfulness:** `collect_unknown_scalar_offsets_from_predicate` (the offsets variant) confirmed as the gate-positions function returning `Vec<(String, usize)>`; `collect_unknown_scalar_from_predicate` confirmed as the SQL scalar walk function returning `Vec<String>`. Two-variant disambiguation in ADR-048 v1.15 §D.7.1 footnote verified accurate.
- **Case-insensitive aggregate lookup:** `DataFusion` built-in name filtering in the aggregate gate confirmed to use case-insensitive comparison — `sql_unknown_names` walk correctly filtered.
- **SqlPipe-head absolute offsets:** `Ast::SqlPipe` head `where_` walk confirmed to produce absolute offsets into the predicate string (not relative to sub-expression start). Position 5 walk path verified accurate in docstring.
- **Two-variant walker coexistence:** Both `collect_unknown_scalar_offsets_from_predicate` and `collect_unknown_scalar_from_predicate` confirmed present in `engine.rs`; both actively used (offsets variant by aggregate gate, non-offset variant by enrichment gate scalar walk). No dead-code confusion introduced by fix-burst-28.
- **OD-citation sweep correctness:** Positions 1–5 comments confirmed to use only positional ordinals (no OD-N labels); OD-6/OD-7 citations retained where appropriate (correct numerical coincidence). Residual `OD-[1-5]` grep yielded 2 legitimate locked-decision references (non-positional context).
- **1653/1653 prism-query** confirmed GREEN at 9745372c baseline (established by fix-burst-27; fix-burst-28 was doc/comment sweep only — no test count change).

All five fix-burst-28 closures VERIFIED REAL. No false-close recurrence.

---

## Findings

### F-PQLFN-P37-MED-001 [MED][POL-22 truthfulness / TD-VSDD-059] — CLOSED @0749f16e (fix-burst-29: 6 sites corrected; D-1755)

**Severity:** MED
**Category:** POL-22 (specification truthfulness) / TD-VSDD-059 (paper-fix detection / walk-observability false claim) — inline regression-class comments at Position-4 and Position-5 in `engine.rs` and adjacent module docstrings claimed that removal of the `predicate_fncall_names` walk at those positions would cause `totally_unknown_udf` to pass the gate — i.e., that the walk is the sole mechanism preventing the gate from producing a false `Ok`. This claim is FALSE for Positions 4 and 5: `Ast::Sql(Select)` and `Ast::SqlPipe` arms both independently call `collect_unknown_scalars_from_sql_query` (via the `sql_unknown_names` path), which walks `WHERE` into `sql_unknown_names` without going through `predicate_fncall_names`. `totally_unknown_udf` is not a DataFusion built-in, so it survives the `sql_unknown_names` filter and fires E-QUERY-039 independently of the `predicate_fncall_names` walk.
**Status:** CLOSED — fix-burst-29 @0749f16e (6 sites corrected; 1653/1653 prism-query; D-1755 2026-07-14)

**Finding:** Fix-burst-28 corrected Position-N ordinal labeling across ~30 sites. During the sweep, six sites contained regression-class comments of the form "walk removal → false Ok (gate escape)" at Positions 4 and 5. Example:

```
// Position 4: SQL WHERE — walk-observable: removing predicate_fncall_names walk here
// causes totally_unknown_udf to pass the gate (false Ok / E-QUERY-039 escape)
```

This claim is false for Positions 4 (`Ast::Sql(SqlStatement::Select)`) and 5 (`Ast::SqlPipe` → head `where_`):

**Why the claim is false for Positions 4/5:**

`Ast::Sql(Select)` arms call both:
- (a) `collect_unknown_scalar_offsets_from_predicate` via the `predicate_fncall_names` walk (the aggregate gate path that checks DataFusion built-in names)
- (b) `collect_unknown_scalars_from_sql_query` via `sql_unknown_names` (the SQL scalar walk path that collects ALL function names from the SQL WHERE expression, independent of `predicate_fncall_names`)

When `totally_unknown_udf(x)=1` appears in a SQL WHERE clause:
- Path (b) adds `totally_unknown_udf` to `sql_unknown_names`
- `totally_unknown_udf` is not a DataFusion built-in → it is NOT filtered from `sql_unknown_names`
- `sql_unknown_names` is checked against available infusions → `totally_unknown_udf` is unknown → E-QUERY-039 fires

Therefore, even if the `predicate_fncall_names` walk (path a) is removed entirely from the Position-4/5 arms, `totally_unknown_udf` STILL fires E-QUERY-039 via path (b). The walk-removal does NOT produce a false `Ok`.

**What the `predicate_fncall_names` walk actually does at Positions 4/5:** It filters DataFusion BUILT-IN aggregate function names (e.g., `count`, `sum`, `avg`) FROM `sql_unknown_names` before the infusion lookup. These built-ins pass the aggregate gate, whereas `totally_unknown_udf` (not a built-in) is never filtered. The `predicate_fncall_names` walk serves the AGGREGATE gate exclusively at Positions 4/5.

**TRUE walk locks (positions where walk removal genuinely produces false Ok):**
- **TM-06** (`test_BC_2_11_019_sql_where_count_passes_gate`): Position 4, `count(x)` — a DataFusion built-in. If `predicate_fncall_names` walk is removed, `count` stays in `sql_unknown_names` → E-QUERY-039 fires INCORRECTLY (false negative on a valid aggregate). This is the true walk-observable regression class for Position 4.
- **TM-07** (`test_BC_2_11_019_sql_where_sum_passes_gate`): Position 4, `sum(x)` — same mechanism.
- **TM-10** (`test_BC_2_11_019_sqlpipe_head_aggregate_passes_gate`): Position 5, SqlPipe-head aggregate — same mechanism.

**Positions that ARE genuinely walk-observable for E-QUERY-039:** Positions 1 (`Ast::Pipe` → `PipeStage::Where`), 2 (`Ast::Filter`), 3 (`Ast::SqlPipe` → `SqlPipeStage::Where`), 6 (`Ast::Sql(Dml)` → `dml.filter` DML WHERE), 7 (`Ast::Sql(Dml)` → `dml.source_select.where_` INSERT source_select WHERE). These arms do NOT call `collect_unknown_scalars_from_sql_query` — the `predicate_fncall_names` walk is the SOLE mechanism for the enrich-UDF gate at these positions.

**Severity rationale:** MED because: (1) a developer reading the Position-4/5 comments would incorrectly conclude that the `predicate_fncall_names` walk is a gate-escape prevention mechanism for `totally_unknown_udf` at those positions — a false claim that would lead to incorrect reasoning about regression risk during future refactors; (2) the true walk locks (TM-06/TM-07/TM-10) test different semantics (aggregate PASS gate, not unknown-UDF REJECT gate) — conflating them misleads refactoring review; (3) the 6 affected sites include module docstrings read by adversary and developer tooling, compounding the impact.

**TD-VSDD-059 applicability:** The fix-burst-28 regression-class comment pattern ("walk removal → false Ok") constitutes a truthfulness claim about gate mechanics. A false truthfulness claim in a docstring is equivalent to a false-close mechanism under TD-VSDD-059 (paper-fix detection): the comment asserts behavioral correctness of the walk at that position when the actual correctness mechanism is elsewhere. The independent verification in this pass caught the false claim before it propagated to an adversary's reasoning about gate coverage.

**Closure evidence @0749f16e (fix-burst-29):**

Implementer independently verified the trace (walk path through `collect_unknown_scalars_from_sql_query` / `sql_unknown_names` independence at Positions 4/5). 6 sites corrected:

(1) **Inline regression comments ×2** (Position-4 SQL WHERE arm, Position-5 SqlPipe-head arm in `engine.rs`): Replaced `"walk removal → false Ok (gate escape)"` with accurate `"walk removal → count/sum/avg built-ins no longer filtered from sql_unknown_names → TM-06/TM-07 E-QUERY-001 regression (built-in passes-gate locks)"`.

(2) **Module docstrings ×2** (`check_enrich_udf_availability` Position-4/5 section headers): Replaced walk-observability claim with accurate description of aggregate-gate filtering role and TM citation.

(3) **Module header 1-3-vs-4-5 split:** Section header distinguishing "walk-observable positions (1-3)" from "aggregate-gate positions (4-5)" updated to reflect that the split is about the GATE MECHANISM (E-QUERY-039 via predicate_fncall_names only vs. E-QUERY-039 via sql_unknown_names independently + aggregate filtering via predicate_fncall_names), not about walk-observability in the E-QUERY-039 sense.

(4) **OD-7 docstring:** `5/7-accurate` claim with TM citations — Position-4/5 TM citations corrected to reference TM-06/TM-07/TM-10 (aggregate-passes-gate locks) rather than unknown-UDF-rejects-gate locks.

**Residual walk-observable grep:** After fix-burst-29, grep for `walk removal → false Ok` or `walk-observable.*Position [45]` across `crates/prism-query/src/` yields only genuinely-observable positions (1-3, 6-7). Positions 4-5 now correctly describe TM-06/TM-07/TM-10 as the aggregate-gate lock tests.

**Test count:** 1653/1653 prism-query at 9745372c baseline. Fix-burst-29 is comment/docstring only — no new tests required (the behavioral correctness is already verified by the existing 1653 tests; the finding is a documentation-truthfulness defect, not a code defect). `just check` re-run not required for comment-only change; `cargo nextest run -p prism-query --no-fail-fast` GREEN on 9745372c (pre-existing baseline) carries over.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in `crates/` on HEAD 9745372c. Fix-burst-29 changes are comment/docstring corrections only (no `event_type` emission sites modified). No BC-2.16.002 catalog row required.

---

## Verification Walk

- **Seven-position parity (BC-2.11.019 v1.18 ↔ ADR-048 v1.15 ↔ code ↔ tests):** All three artifacts confirm seven predicate surfaces gated. No residual six-position live-prose claims.
- **`collect_unknown_scalars_from_sql_query` independence:** Both `Ast::Sql(Select)` and `Ast::SqlPipe` arms confirmed to call `collect_unknown_scalars_from_sql_query` via `sql_unknown_names` independently of `predicate_fncall_names`. The independence was verified by tracing the `sql_unknown_names` accumulation path through `engine.rs` — the path does not pass through `predicate_fncall_names` at Positions 4/5.
- **Aggregate-gate filter confirmed correct:** `predicate_fncall_names` at Position 4 filters DataFusion built-in names (`count`, `sum`, `avg`, etc.) from `sql_unknown_names` before the infusion-availability check. `totally_unknown_udf` is not a DataFusion built-in → unaffected by the filter → E-QUERY-039 fires via `sql_unknown_names` regardless.
- **TM-06/TM-07/TM-10 lock tests verified:** `test_BC_2_11_019_sql_where_count_passes_gate`, `test_BC_2_11_019_sql_where_sum_passes_gate` (Position 4), `test_BC_2_11_019_sqlpipe_head_aggregate_passes_gate` (Position 5) — all confirmed GREEN at 9745372c and exercising the aggregate PASS gate. These are the correct walk-observable locks for the `predicate_fncall_names` walk at Positions 4/5.
- **Positions 1-3/6-7 walk-observable status confirmed:** Arms for these positions do NOT call `collect_unknown_scalars_from_sql_query`. The `predicate_fncall_names` walk is the sole E-QUERY-039 gate mechanism at these positions. E-QUERY-001 compound-predicate form tests for each position confirmed present.
- **Fix-burst-28 OD-citation sweep residual grep:** `OD-[1-5]` in `crates/prism-query/src/engine.rs` and test files = 2 legitimate locked-decision references (non-positional context; correct). Unchanged by fix-burst-29.
- **1653/1653 prism-query:** confirmed GREEN at 9745372c baseline; fix-burst-29 is comment-only (no behavioral change; count unchanged).

---

## Status

**CLOSED — fix-burst-29 COMPLETE (D-1755 2026-07-14).**

Finding fully closed:
- **F-PQLFN-P37-MED-001 CLOSED @0749f16e:** 6 sites corrected (inline regression comments ×2, module docstrings ×2, module header 1-3-vs-4-5 split, OD-7 docstring TM citations); residual walk-observable grep = only genuinely-observable positions; 1653/1653 prism-query.

**CASCADE TALLY:** 37 passes / 29 fix-bursts
**STREAK:** 0/3 on new frozen HEAD 0749f16e (DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-29 pushed new commits; streak resets; all findings closed; 0749f16e is local-only)
**NEXT ACTION:** LOCAL pass-38 on frozen 0749f16e (streak 0/3)
