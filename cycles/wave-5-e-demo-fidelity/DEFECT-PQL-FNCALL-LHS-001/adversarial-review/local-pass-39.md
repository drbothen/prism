---
pass: 39
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 0/3
streak_after: 1/3
status: CLEAN
---

# LOCAL Adversary Pass 39 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; includes fix-burst-30 corrections)
**CLEAN(strict): YES** (zero findings of any severity)
**CLEAN(PR-merge): YES** (zero findings of CRIT + HIGH + MED)
**Streak: 1/3** (first consecutive CLEAN(strict) pass on frozen 5e4c7ccb; streak ADVANCES 0/3 → 1/3 per BC-5.39.001)

---

## Fix-burst-30 Closure Verification

Fix-burst-30 resolved the findings from pass-38. Verifying closures against frozen 5e4c7ccb:

**F-PQLFN-P38-MED-001** (check_enrich_udf_availability rustdoc false "Positions 1-3/6-7 do not reach E-QUERY-039" — 5 docstring sites): VERIFIED CLOSED at 5e4c7ccb. Function-level rustdoc now reads "All 7 positions reach E-QUERY-039 via the `predicate_fncall_names → sql_unknown_names` fold. Pipe-mode and filter-mode predicates are both collected unconditionally." Sweep grep for "do not reach E-QUERY-039" and "do not feed sql_unknown_names" and "bypass — does not reach": zero residuals at 5e4c7ccb. 1653/1653 prism-query GREEN.

**F-PQLFN-P38-OBS-001** (fn-call LHS BETWEEN/IN/LIKE generic E-QUERY-039 message; BC-2.11.004 §LOW-002 scope limit): VERIFIED ACCEPTED-NO-ACTION at 5e4c7ccb. BC-2.11.004 §LOW-002 accepted-class entry present. Behavior unchanged and on record.

**F-PQLFN-P38-OBS-002** (E-QUERY offset byte-semantics undocumented at namespace level): VERIFIED CLOSED at 5e4c7ccb. `error-taxonomy.md` E-QUERY namespace header contains the byte-offset note per fix-burst-30 (v2.52). Version 2.52 changelog row present.

SAP-1 at 5e4c7ccb: `event_type =` emission sweep across `crates/` — 174 unique sites catalogued in BC-2.16.002 §Postconditions. Fix-burst-30 changes (5 rustdoc corrections + error-taxonomy namespace note) introduced zero net-new `event_type =` emissions. No BC-2.16.002 catalog row required.

---

## Findings

**ZERO findings.** No CRIT, HIGH, MED, LOW, OBS, or PROCESS-GAP issues identified at frozen HEAD 5e4c7ccb.

---

## SAP-1 Result

**PASS.** `crates/` `event_type =` emission sweep at frozen 5e4c7ccb: 174 unique `event_type` values found; all 174 catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog with full field schema, audit role, and recurrence policy. No new emissions introduced since pass-38. No catalog update required.

---

## Verification Walk

**All 7 predicate positions traced arm-by-arm (unified fold verification):**

The `predicate_fncall_names → sql_unknown_names` fold in `check_enrich_udf_availability` is unconditional at 5e4c7ccb. Each position confirmed:

- **(1) Pipe `| where` stage predicate** — `PipeStage::Where(Predicate)` arm: predicate passed to `collect_unknown_scalar_offsets_from_predicate`; fold-only path (no dual-path for positions 1-3/6-7). Verified at 5e4c7ccb.
- **(2) Filter-mode root predicate** — `Ast::Filter(Predicate)` arm: predicate walked by same fold. Fold-only path. Verified at 5e4c7ccb.
- **(3) SqlPipe pipe-stage `| where` predicate** — `Ast::SqlPipe` tail-stage Where arm: predicate folded. Fold-only path. Verified at 5e4c7ccb.
- **(4) SQL WHERE** — `sql_query.where_` fed into `collect_unknown_scalars_from_sql_query`; ALSO fed into `predicate_fncall_names` fold. **Dual-path position** — both call sites verified at 5e4c7ccb: `check_position_reach_e_query_039` exercises both paths; load-bearing tests TM-06 (fold) + TM-14 (SQL walk).
- **(5) SqlPipe-head WHERE** — `ast.head_query.where_` dual-path same as position 4. Both call sites verified at 5e4c7ccb.
- **(6) SQL DML WHERE** — `Ast::Sql(SqlStatement::Dml(dml))` arm explicit: `dml.filter` walked by `check_enrich_udf_availability`'s explicit DML arm (ADR-048 v1.15 §D.7.5 OD-6). Fold-only path for positions 6-7. Load-bearing test `test_f_pqlfn_p7_low_002_delete_where_unknown_udf_fires_e_query_039` GREEN at 5e4c7ccb.
- **(7) INSERT source_select WHERE** — same explicit DML arm: `dml.source_select.where_` walked. Fold-only path. Load-bearing test `test_f_pqlfn_p32_obs_001_insert_source_select_where_aggregate_fires_e_query_001` GREEN at 5e4c7ccb.

**Load-bearing locks enumerated per position at 5e4c7ccb:**
- Position 1 (pipe | where): TM-02, TM-04
- Position 2 (filter-mode): TM-05
- Position 3 (SqlPipe | where): TM-08
- Position 4 (SQL WHERE dual-path): TM-06, TM-14, TM-16
- Position 5 (SqlPipe-head WHERE dual-path): TM-07, TM-17, TM-18
- Position 6 (DML WHERE): TM-10, p7_low_002
- Position 7 (INSERT source_select WHERE): p32_obs_001
- HAVING exemption lock (p33_med_001): load-bearing `stddev` passes-gate test confirms HAVING predicates are NOT walked into `predicate_fncall_names`; the `stddev` aggregate in HAVING executes correctly without E-QUERY-001 (ADR-048 v1.4 §D.7.1 permit ruling confirmed live).

**Parser gates LOW-004/005/006 verified against BC verbatim:**
- LOW-004 (identifier-class keywords rejected at fn-call position): `fn_call_comparison` parser rejects PrismQL reserved keywords in fn-call-name position; BC-2.11.019 §LOW-004 text confirmed consistent with parser behavior at 5e4c7ccb.
- LOW-005 (aggregate-class names rejected): `AGGREGATE_FUNC_NAMES` blocklist removal confirmed — blocklist approach was superseded by the DataFusion built-in registry check. BC-2.11.019 §LOW-005 text reflects registry-based exclusion at 5e4c7ccb.
- LOW-006 (temporal-class keywords rejected): temporal keyword gate in `fn_call_comparison` verified; BC-2.11.019 §LOW-006 text confirmed consistent at 5e4c7ccb.

**AGGREGATE_FUNC_NAMES blocklist removal confirmed:** The static `AGGREGATE_FUNC_NAMES` blocklist (superseded approach from pre-fix-burst-20 era) is absent at 5e4c7ccb. DataFusion built-in registry check (scalar + aggregate + window via `SessionContext` state) is the live mechanism. No dead-code blocklist present that could cause false E-QUERY-039 fires on DataFusion built-in aggregate names.

**Two-variant walker verified:** `collect_unknown_scalar_from_expr` (SQL scalar walk, no offset tracking) and `collect_unknown_scalar_offsets_from_predicate` (predicate walk with offset tracking) are the two distinct functions at 5e4c7ccb. They serve different scopes: SQL-mode projection/WHERE walk uses `collect_unknown_scalars_from_sql_query` which calls `collect_unknown_scalar_from_expr`; predicate positions 1-7 use `collect_unknown_scalar_offsets_from_predicate`. The two-variant architecture is correct and the fix-burst-30 rustdoc (all 5 corrected sites) now accurately describes this.

**Spec quadrangle version-aligned at 5e4c7ccb:**
- BC-2.11.019 v1.18 (index pin at BC-INDEX v8.18)
- ADR-048 v1.15 (ARCH-INDEX v2.191)
- error-taxonomy v2.52
- BC-2.11.019 §LOW-002 accepted-class for BETWEEN/IN/LIKE LHS: present

**fix-burst-30 rustdoc verified TRUE:** All 5 corrected docstring sites at 5e4c7ccb accurately reflect the unified-fold behavior. Function-level rustdoc, pipe/filter branch docstrings, and position-4/5/7 per-position docstrings all consistent with in-body comment on the `predicate_fncall_names` fold line ("all 7 positions"). No internal contradiction remains. TD-VSDD-059 requirement satisfied (closure load-bearing — behavioral correctness was already right; doc accuracy is now correct).

**1653/1653 prism-query GREEN at 5e4c7ccb.**

---

## Status

```
CLEAN — pass 39 complete. No fix-burst required.

CASCADE TALLY: 39 passes / 30 fix-bursts

STREAK: 1/3 (advances from 0/3; first consecutive CLEAN(strict) on frozen 5e4c7ccb)

FINDINGS BREAKDOWN:
  (none)

CLEAN(strict): YES (zero findings of any severity)
CLEAN(PR-merge): YES (zero findings of CRIT+HIGH+MED)

NEXT ACTION: LOCAL adversary pass 40 on frozen 5e4c7ccb (streak 1/3; two CLEAN(strict) required to advance to 2/3, then 3/3)
```
