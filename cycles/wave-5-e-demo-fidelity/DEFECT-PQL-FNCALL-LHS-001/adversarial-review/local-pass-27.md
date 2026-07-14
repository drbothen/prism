---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [27]
feature_head_at_review: 1a07a5f9
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 6
  crit: 0
  high: 0
  med: 3
  low: 1
  obs: 2
  process_gap: 0
code_behavior_defects: 3
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 27 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 27 (frozen 1a07a5f9; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate + expr_to_sql FuncCall arm + SqlPipe stage span offset translation; LOCAL cascade; streak RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 6 (3 MED, 1 LOW, 2 OBS); SAP-1 PASS

All 6 findings CLOSED by fix-burst-21.

---

## Findings

### F-PQLFN-P27-MED-001 [MED][truthful-offset-drift]

**LOW-006 SqlPipe stage-error offsets are stage-relative (error path returned before span shift)**

The `parse_sqlpipe_internal` function calls `stage_parser.parse(&input[split_offset..])` to parse each `| where` stage substring. When the parser returns an error, the error path is returned BEFORE the span-shift walk (`shift_parse_error_offsets`). This means E-QUERY-001 aggregate-gate errors emitted from `| where` stages report offsets relative to the stage substring, not to the full query string. This violates ADR-048 §D.7.2 truthful-offset. The fix-burst-20 shift_scalar_spans helpers correctly shift SUCCESS-path span fields but the ERROR-path offset goes through `parse_sqlpipe_internal`'s early return, bypassing the shift entirely.

**Status:** CLOSED by fix-burst-21 — `shift_parse_error_offsets` helper introduced in `parse_sqlpipe_internal`; error path now calls shift on the error offset before returning. RED 8→GREEN 45 offset evidence; head-path verified absolute. Code @c3dcad27 (implementer).

---

### F-PQLFN-P27-MED-002 [MED][test-coverage-gap]

**LOW-006 tests covered 3 of 6 shared-parser surfaces**

The fix-burst-20 implementation introduced the reserved-keyword gate across 6 shared-parser call surfaces (Pipe WHERE / Filter / SQL WHERE / SqlPipe head WHERE / SqlPipe where stage / DML WHERE). The test suite for LOW-006 covered the fn_call_comparison production in 3 of these surfaces (Pipe/SQL/SqlPipe-stage) but omitted 3 surfaces: filter-mode direct parse, SQL HAVING (aggregate-position via bc_2_11_004 fn_call_comparison in the HAVING walker), and DML WHERE. Without tests for the 3 missing surfaces the shared-parser claim (that the single production covers all 6) was not mechanically verified.

**Status:** CLOSED by fix-burst-21 — 3 new surface tests added (filter-mode / SQL HAVING / DML WHERE), ALL GREEN on arrival (shared parser confirmed); test-writer @ef8b9bb3. just check 5567/5567 GREEN; prism-query 1631/1631.

---

### F-PQLFN-P27-MED-003 [MED][paper-fix/POL-24]

**Rejection tests asserted variant only, not message text**

The 8 LOW-006 rejection tests (keyword fn-name gate, `NOT(x)` / `AND(x,y)` / etc.) asserted the error variant (`Err(PrismError::QueryParseFailed { .. })`) but did not assert the message text content. Under POL-24 (message-text assertion required on ALL rejection tests to prevent paper-fix regressions), variant-only assertions are insufficient. A rename or stub-out of the E-QUERY-001 message string would not be caught by these tests.

**Status:** CLOSED by fix-burst-21 — all 8 LOW-006 rejection tests now assert message text (POL-24 locks added); test-writer @ef8b9bb3.

---

### F-PQLFN-P27-LOW-001 [LOW]

**Stale "Red Gate pending" markers in BC line 143 + 5 test doc-comments**

BC-2.11.004 §Error Cases LOW-006 body (line ~143) retained a "Red Gate pending" marker from the fix-burst-20 spec authoring round. Five test doc-comments in the LOW-006 test module also carried "Red Gate: awaiting implementation" language. These markers were valid at spec-authoring time but become false at fix-burst-21 completion — the code and tests both ship.

**Status:** CLOSED by fix-burst-21 — BC-2.11.004 v1.42→v1.43: "Red Gate pending" → "Implemented (fix-burst 20)" in §Error Cases LOW-006 body (LOW-001 closure); 5 stale test doc-markers updated in code @c3dcad27 (implementer); POL-23 sweep: S-PRISMQL-CASE-INSENSITIVE-001 v1.67→v1.68 (4 pins); spec @21b63bb8 (PO).

---

### F-PQLFN-P27-OBS-001 [OBS]

**Double-nested E-QUERY-001 Display on Chumsky path**

On the Chumsky parse path (`prism-query ParseError` variant), the Display emits `"E-QUERY-001: parse error at offset N"` (Form B). When this is wrapped into `PrismError::QueryParseFailed`, the outer Display also prepends `"E-QUERY-001: "`, producing `"E-QUERY-001: E-QUERY-001: parse error at offset N"` in the final error message. This is the "double-nested E-QUERY-001" Form B phenomenon documented in error-taxonomy v2.48. The recovery guard (`parse_sqlpipe_internal` retry path) checks for the `"E-QUERY-001:"` prefix to distinguish semantic errors from structural errors — this prefix check is load-bearing for blocking recovery on semantic (fn-call) errors.

**Status:** CLOSED by architect adjudication @9bd637d5 (factory-artifacts): ADR-048 v1.11→v1.12 — OBS-001 option (b) RATIFIED: two-form E-QUERY-001 Display convention is intentional design. Form A = plan-time de-nested (clean single prefix); Form B = Chumsky double-nested (prefix is load-bearing for recovery guard). Both forms normalized in §D.7.2 as canonical. No code change required.

---

### F-PQLFN-P27-OBS-002 [OBS]

**Recovery-guard scope undocumented behavioral change (percentile)**

The `parse_sqlpipe_internal` recovery guard blocks retry when the error contains the `"E-QUERY-001:"` prefix (semantic validation = not a structural parse error). This guard was introduced for fn-call keyword rejection but has a broader effect: any E-QUERY-001 error (including aggregate-position errors for `percentile(x, 0.95)` in SqlPipe WHERE) also blocks recovery. Pre-fix, `percentile` in SqlPipe WHERE would be retried as a structural SqlPipe parse attempt (likely producing a different error or a partial result). Post-fix, it is hard-blocked at the semantic stage. This behavioral change for percentile was not documented in ADR-048 §D.7 or the spec.

**Status:** CLOSED by architect adjudication @9bd637d5 (factory-artifacts): ADR-048 v1.11→v1.12 — OBS-002 option (a) RATIFIED: broad recovery guard is intentional general semantic-error protection. "prefix = semantic validation = blocks recovery" is now §D.7.2 normative prose. Percentile behavior change is intended (percentile in SqlPipe WHERE is an aggregate-position semantic error; blocking recovery is correct). No code change required.

---

## SAP-1 Result

PASS — no new `event_type =` emissions found in crates/ workspace on frozen 1a07a5f9.

---

## Fix-Burst 21 Summary

All 6 findings CLOSED:

| Finding | Severity | Closer | Commit |
|---------|----------|--------|--------|
| MED-001 (truthful-offset-drift) | MED | implementer | @c3dcad27 |
| MED-002 (test-coverage-gap) | MED | test-writer | @ef8b9bb3 |
| MED-003 (paper-fix/POL-24) | MED | test-writer | @ef8b9bb3 |
| LOW-001 (stale Red Gate markers) | LOW | PO + implementer | @21b63bb8 + @c3dcad27 |
| OBS-001 (double-nested Display) | OBS | architect adjudication | @9bd637d5 |
| OBS-002 (recovery-guard scope) | OBS | architect adjudication | @9bd637d5 |

**NEW FROZEN HEAD:** ef8b9bb3 (LOCAL-ONLY NOT pushed)
**CASCADE TALLY:** 27 passes / 21 fix-bursts
**STREAK:** 0/3 on new frozen HEAD ef8b9bb3
**just check:** 5567/5567 GREEN; prism-query 1631/1631; non-exhaustive 91/91
**PQL NEXT:** LOCAL pass 28 on frozen ef8b9bb3
