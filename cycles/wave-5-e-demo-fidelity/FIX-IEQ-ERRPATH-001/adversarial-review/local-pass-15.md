---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [15]
feature_head_at_review: 3f2eddd1
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  med: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 15 — FIX-IEQ-ERRPATH-001

---

## Pass 15 (frozen 3f2eddd1; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 1/3 — RESET)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 1 total (0 CRIT / 0 HIGH / 1 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK RESET: 0/3 → 0/3** (BC-5.39.001; streak was already 0/3 after pass-14; remains 0/3).

**Code HEAD at review:** 3f2eddd1 (frozen; D-1625 fix-burst: EC-11-072 stage-join suspension + EC-11-073 lock; 5373/5373 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** no (1 MED finding present; streak criterion requires ZERO findings of ANY severity)

**CLEAN(PR-merge):** no (MED finding is above PR-merge-gate threshold; PR-merge requires ZERO CRIT + HIGH + MED)

**All 1 finding CLOSED same-burst** (D-1627): BC-2.11.016 v1.20 HEAD-JOIN SUSPENSION RULE + EC-11-074/075 + 4 sibling BC/taxonomy pins + 4 story pin round + test-writer RED @399eba90 + implementer GREEN @3212070c; 5385/5385 GREEN; non-exhaustive 89/89. Streak remains 0/3. VERY NEXT ACTION: freeze 3212070c → LOCAL adversary pass 16 (streak candidate 1/3; fresh context, strict).

---

## Finding ADV-FIX-P15-MED-001 — HEAD-JOIN: positions 1–6 bare unqualified refs checked against FROM-only schema

**Severity:** MED (HIGH confidence; novel angle — head SQL clause positions 1–6 with JOIN, unexplored by passes 1–14)

**Confidence:** HIGH (adversary independently reproduced the false E-QUERY-038 path: `SELECT col FROM crowdstrike_alerts JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id` where `col` exists only in `some_other_table` triggers plan-time ColumnNotFound on fix-branch 3f2eddd1; confirms FP-001 absolute invariant is violated)

**Finding:** `check_query_column_availability` at @3f2eddd1 evaluates head SQL clause positions 1–6 (SELECT, WHERE, GROUP BY, ORDER BY, JOIN ON bare, HAVING) against the FROM table schema only. When a query includes a non-empty JOIN list and a head-clause position references a bare unqualified column that exists only in the JOIN-partner schema (not in the FROM table schema), the gate fires E-QUERY-038 — a false positive.

Root cause: BC-2.11.016 v1.19 §Preconditions.2 enumerated head positions 1–6 for `Ast::Sql` and `Ast::SqlPipe` forms but included no JOIN suspension exception. The FP-001 trigger list (5 triggers as of v1.19) covered the pipe-stage walk (PipeStage::Join via STAGE-JOIN SUSPENSION RULE and star+JOIN via STAR-WITH-JOIN SUSPENSION RULE) but did NOT extend to head SQL positions 1–6.

The same class applies across all head positions:
1. **Position 1 (SELECT):** `SELECT col FROM t1 JOIN t2 ON t1.id = t2.id` where `col` is in t2 only → false E-QUERY-038
2. **Position 2 (WHERE):** `SELECT * FROM t1 JOIN t2 ON t1.id = t2.id WHERE col = 'high'` where `col` is in t2 only → false E-QUERY-038 (note: IEQ in SQL-mode WHERE rejected per BC-2.11.024 §SQL-Mode Rejection; test uses plain `=`)
3. **Position 3 (GROUP BY):** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id GROUP BY col` where `col` is in t2 only → false E-QUERY-038
4. **Position 4 (ORDER BY):** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id ORDER BY col` where `col` is in t2 only → false E-QUERY-038
5. **Position 6 (HAVING):** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id HAVING col > 0` where `col` is in t2 only → false E-QUERY-038
6. **Ast::SqlPipe head form:** Same failure mode in the SqlPipe head variant.

(Position 5 = JOIN ON; the ON condition references the join keys themselves, typically qualified.)

**FP-001 class:** head-join (newly classified at D-1627; distinct from pipe-stage-join (D-1625) and star-with-join (D-1623))

**Closure:**
- product-owner authored BC-2.11.016 v1.20 HEAD-JOIN SUSPENSION RULE: for head SQL clause positions 1–6 of Ast::Sql and Ast::SqlPipe forms, when the query has a non-empty JOIN list, bare unqualified column refs MUST NOT fire E-QUERY-038 (fail-open); FROM-present columns still checked at plan time; qualified refs (segments.len()==2) retain full E-QUERY-038 gate; joinless queries unchanged; both Ast::Sql and Ast::SqlPipe covered; FP-001 trigger list extended 5→6; future-strengthening schema-union note preserved. EC-11-074 (bare col in head SQL position 1–6 with JOIN — fail open, no false E-QUERY-038) + EC-11-075 (Ast::SqlPipe head form with JOIN — same fail-open guarantee). Sibling syncs: BC-2.11.017 v1.8, BC-2.11.020 v1.13, BC-2.11.004 v1.25, error-taxonomy v2.33.
- test-writer RED @399eba90: 12 tests (10 RED gates EC-11-074/075 across positions 1/2/3/4/6 × Sql+SqlPipe; 2 negative controls GREEN: joinless query still fires E-QUERY-038 for genuine typo; FROM-present col with JOIN is checked normally). Grammar note: IEQ in SQL-mode WHERE is rejected per BC-2.11.024 §SQL-Mode Rejection; position-2 test uses `WHERE col = 'high'` (documented in test module header).
- implementer GREEN @3212070c: `head_has_joins` boolean + `bare_head_cols` HashSet collected via new private fns `collect_bare_field_names_from_expr` / `collect_bare_pred_field_names`; gate loop in `check_query_column_availability` swallows ColumnNotFound only for bare head cols when `head_has_joins` is true; qualified refs (segments.len()==2) retain full E-QUERY-038; EC-11-074/075 tests GREEN at @3212070c; full `just check` 5385/5385 GREEN; non-exhaustive 89/89.

**Status:** CLOSED (BC v1.20 + test RED @399eba90 + implementer GREEN @3212070c)

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 3f2eddd1. No new `event_type` assignments introduced at @3f2eddd1 vs prior reviewed HEAD. Five total catalog rows verified (three `column_not_found.rejected` emission sites + two `reload.*` sites). Zero gaps. The fix-burst at @3212070c adds `head_has_joins` + `bare_head_cols` computation and a gate-bypass conditional in the existing loop — no new tracing emission site; SAP-1 coverage unchanged post-fix.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-074 and EC-11-075 added to BC-2.11.016 v1.20 carry full field schema, audit role, and recurrence policy in byte-parity with the canonical EC-11-039..073 body format.

**Audit-script Section G arithmetic:** VERIFIED (pending EC-11-074/075 expansion) — Section G count 70 (as of v1.19 with EC-11-039..073): EC-11-039..073 = 35 entries from D-1625 closure. EC-11-074 + EC-11-075 (2 new entries from D-1627) extend the section G count to 72. Arithmetic correct; no gap or duplicate expected.

**FP-001 probe list (16+ shapes):** PASS on all prior-pass shapes — adversary re-ran the full prior-pass FP-001 probe list at @3f2eddd1; all 16+ shapes from passes 1–14 continue to behave correctly (zero regressions). The new failing shape (bare unqualified head-clause col with JOIN) is novel and restricted to positions 1–6 of Ast::Sql / Ast::SqlPipe head.

**POL-24 (byte-for-byte EC-body):** PASS — EC-11-074/075 bodies match the canonical format.

**POLICY 33 N/A:** CONFIRMED — Policy 33 (multi-tenant isolation gate for cross-tenant data access) is not applicable to this fix-burst. The HEAD-JOIN SUSPENSION RULE applies to plan-time column availability checking only; it does not touch runtime data-path or tenant isolation logic.

---

## Post-Fix Verification

**fix-branch HEAD after fix-burst:** 3212070c

**Test count:** 5385/5385 GREEN (12 new tests from EC-11-074/075 gate: 10 RED gates + 2 negative controls; +12 from 5373)

**Non-exhaustive gate:** 89/89 UNCHANGED (no new public types introduced)

**`just check` result:** 5385/5385 GREEN; non-exhaustive 89/89; fmt + clippy + layout PASS

---

## Convergence Assessment

**Trajectory:** 6 → 3 → 3 → 2 → 1 → [0] → 2 → [0] → 4(low/obs) → 1(med)

**Pattern:** Pass 15 finds 1 MED on frozen 3f2eddd1 (HEAD 3f2eddd1 was the post-pass-14 fix-burst HEAD; passes 13 found 0 findings on a prior frozen HEAD, but pass 14 found 4 and the fix-burst moved HEAD to 3f2eddd1). The finding ADV-FIX-P15-MED-001 is a genuine behavioral gap: positions 1–6 of the head SQL clause were not covered by any suspension rule. Passes 1–14 probed the pipe-stage walk and the SqlPipe head-projection seeding path; pass 15 probed the head clause positions themselves in the JOIN-present context and found the gap.

**Novelty assessment:** MEDIUM — The head-join FP-001 gap (positions 1–6 with JOIN) is a novel angle not surfaced in passes 1–14. Prior passes established STAR-WITH-JOIN suspension (passes 12/13) and STAGE-JOIN suspension (pass 14) in the pipe-stage walk. Pass 15 extended the probe to the head SQL clause positions and found the analogous gap. After D-1627 closure, the FP-001 coverage is more complete: head positions 1–6 now fail-open when JOIN is non-empty (EC-11-074/075), alongside the pipe-stage walk suspension rules (EC-11-070/071/072/073).

**Streak status:** 0/3 (BC-5.39.001). VERY NEXT ACTION: freeze 3212070c → LOCAL adversary pass 16 (fresh context, strict; streak candidate 1/3). Three consecutive CLEAN(strict) passes on unchanged HEAD required (passes 16/17/18) → then push branch + open fix-PR via pr-manager.
