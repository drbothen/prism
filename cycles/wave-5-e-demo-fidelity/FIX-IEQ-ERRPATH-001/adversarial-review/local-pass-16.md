---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [16]
feature_head_at_review: 3212070c
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

# LOCAL Adversary Pass 16 — FIX-IEQ-ERRPATH-001

---

## Pass 16 (frozen 3212070c; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 1/3 — RESET)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 1 total (0 CRIT / 0 HIGH / 1 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK RESET: 0/3 → 0/3** (BC-5.39.001; streak remains 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001 — fix-burst advanced HEAD 3212070c→35117a38).

**Code HEAD at review:** 3212070c (frozen; D-1627 fix-burst: EC-11-074/075 head-join suspension + bare_head_cols HashSet; 5385/5385 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** no (1 MED finding present; streak criterion requires ZERO findings of ANY severity)

**CLEAN(PR-merge):** no (MED finding is above PR-merge-gate threshold; PR-merge requires ZERO CRIT + HIGH + MED)

**All 1 finding CLOSED same-burst** (D-1628): BC-2.11.016 v1.21 PER-REFERENCE SCOPING + EC-11-076 + 4 sibling BC/taxonomy pins + 4 story pin round + test-writer RED @5ef1a1a8 + implementer GREEN @35117a38; 5391/5391 GREEN; non-exhaustive 89/89. Streak remains 0/3. VERY NEXT ACTION: freeze 35117a38 → LOCAL adversary pass 17 (fresh context, strict; streak candidate 1/3).

---

## Finding ADV-FIX-P16-MED-001 — HEAD-JOIN PER-REFERENCE SCOPING: name-keyed suspension wrongly suppresses qualified refs of same name

**Severity:** MED (HIGH confidence; novel angle — same-name bare+qualified interaction in head positions 1–6, unexplored by passes 1–15)

**Confidence:** HIGH (adversary independently reproduced the false-negative: `SELECT alias.col FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.severity = some_other_table.id WHERE col = 'x'` — qualified `alias.col` is a genuine typo if `alias.col` does not exist in the JOIN-partner schema, but the HEAD-JOIN SUSPENSION at @3212070c suppresses E-QUERY-038 for `alias.col` because bare `col` appears in head positions via the WHERE clause; confirms BC-2.11.016 v1.20 §Preconditions HEAD-JOIN SUSPENSION RULE is missing per-reference scoping language)

**Finding:** `check_query_column_availability` at @3212070c implements the HEAD-JOIN SUSPENSION RULE via `bare_head_cols: HashSet<String>` keyed by column name. The extraction functions `collect_bare_field_names_from_expr` and `collect_bare_pred_field_names` return bare name strings for bare unqualified FieldPath references. When the gate loop encounters a qualified reference (e.g., `alias.col` with `segments.len()==2`), it calls `extract_column_name_from_field_path` which returns the last-segment bare name `col`. The gate then checks `bare_head_cols.contains("col")` — which is true because a bare `col` appears in head positions 1–6 (e.g., `WHERE col = 'x'`) — and suppresses E-QUERY-038 for `alias.col` as well.

Root cause: BC-2.11.016 v1.20 HEAD-JOIN SUSPENSION RULE specifies fail-open for "bare unqualified column refs" when JOIN non-empty, but the implementation tracks suspension by column name rather than by reference bareness. A qualified reference `alias.col` has `segments.len()==2` and should retain the full E-QUERY-038 gate regardless of whether any bare reference to the same name appears elsewhere in head positions.

This is NOT an FP-001 violation in the fail-open direction (qualified refs firing E-QUERY-038 is correct plan-time behavior, not a false positive). It is an unsanctioned false-negative: qualified typos at head positions 1–6 silently receive fail-open treatment they are not entitled to, losing plan-time E-QUERY-038 detection and the associated did_you_mean pedagogical hint.

Concrete failure scenarios:
1. **Qualified SELECT with typo:** `SELECT alias.typo_col FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — `alias.typo_col` should fire E-QUERY-038 (qualified typo); suppressed because bare `col` is in WHERE (head position 2).
2. **Qualified agg-arg with typo:** `SELECT sum(alias.typo_severity) FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — `alias.typo_severity` in agg-arg (head position 1 agg extension) should fire E-QUERY-038; suppressed same reason.
3. **SqlPipe form:** Same suppression applies to Ast::SqlPipe head form.
4. **Bare WHERE ref correctly suspended:** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id WHERE col = 'high'` — bare `col` in WHERE should remain fail-open per EC-11-074; confirmed UNAFFECTED by fix.
5. **Qualified present-col correctly passes gate:** `SELECT alias.severity FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — `alias.severity` exists in schema; should check type normally without suppression; confirmed correct after fix.
6. **Negative control — joinless qualified typo still fires:** `SELECT alias.typo_col FROM crowdstrike_alerts AS alias WHERE col = 'x'` (no JOIN) — should still fire E-QUERY-038 (UNCHANGED by fix).

**FP-001 classification:** PER-REFERENCE scoping clarification — trigger count UNCHANGED at 6; this finding corrects the scoping semantics of trigger 6 (HEAD-JOIN SUSPENSION RULE), not its existence. Fail-open direction unaffected.

**Closure:**
- product-owner authored BC-2.11.016 v1.21 PER-REFERENCE SCOPING sentence added to HEAD-JOIN SUSPENSION RULE ("suspension applies per individual column reference, not per column name; a qualified reference (segments.len()==2) to a name that also appears bare elsewhere in positions 1–6 is NOT suspended, and retains the full E-QUERY-038 gate") + EC-11-076 (mixed same-name bare+qualified: bare WHERE `col` fail-open; qualified SELECT `alias.col` fires E-QUERY-038 on typo; canonical test vector). FP-001 trigger count unchanged (6 — scoping clarification, not new trigger). Origin anchors (v1.15/v1.17/v1.18/v1.19/v1.20 rule-introduction anchors) preserved. Sibling syncs: BC-2.11.017 v1.8→v1.9 (pin-only), BC-2.11.020 v1.13→v1.14 (prose+pin: PER-REFERENCE SCOPING inserted into HEAD-JOIN restatement), BC-2.11.004 v1.25→v1.26 (pin-only), error-taxonomy v2.33→v2.34 (prose+pin).
- story-writer pin round (4 carrier stories, 5 value classes, zero live stale pins remain): S-DEMO-FIDELITY-REMEDIATION-001 v2.38→v2.39 (6 sites); S-DEMO-PRISMQL-ONBOARDING-001-B v2.15→v2.16 (4 sites + L22 Key-Clauses cell: PER-REFERENCE SCOPING appended, EC cite 074/075→074/075/076); S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.24→v1.25 (3 sites); S-PRISMQL-CASE-INSENSITIVE-001 v1.49→v1.50 (5 sites).
- test-writer RED @5ef1a1a8: 6 tests EC-11-076 (4 RED: alias-qualified SELECT, table-qualified SELECT, qualified agg-arg `sum(alias.typo)`, SqlPipe form; 2 negative controls GREEN: bare-only suspension preserved, qualified present-col ok). Zero EC-11-074/075 regressions (bare suspension still operative).
- implementer GREEN @35117a38 (full SHA 35117a38a8615a4c8b30dce7acdb48cc28df2c32): replaced name-keyed `bare_head_cols: HashSet<String>` with per-reference `(name, is_bare)` pairs via new `extract_field_paths_with_bareness` / `extract_predicate_columns_with_bareness` / `collect_predicate_columns_with_bareness`; removed dead `collect_bare_field_names_from_expr` / `collect_bare_pred_field_names` + `bare_head_cols` block; all 6 position extractors + gate loop updated. ec11076 6/6; ec1107 filter 22/22; prism-query 1481/1481; `just check` 5391/5391 (was 5385); non-exhaustive 89/89; SAP-1 zero new emissions.

**Status:** CLOSED (BC v1.21 + test RED @5ef1a1a8 + implementer GREEN @35117a38)

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 3212070c. No new `event_type` assignments introduced at @3212070c vs prior reviewed HEAD. Five total catalog rows verified (three `column_not_found.rejected` emission sites + two `reload.*` sites). Zero gaps. The fix-burst at @35117a38 replaces `HashSet<String>` with per-reference `(name, is_bare)` pairs — no new tracing emission site; SAP-1 coverage unchanged post-fix.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-076 added to BC-2.11.016 v1.21 carries full field schema, audit role, and recurrence policy in byte-parity with the canonical EC-11-039..075 body format.

**Audit-script Section G arithmetic:** VERIFIED — EC-11-076 (1 new entry from D-1628) extends the section G count from 72 (v1.20 with EC-11-039..075) to 73. Arithmetic correct; no gap or duplicate expected. FP-001 trigger count UNCHANGED at 6 (PER-REFERENCE SCOPING is a clarification, not a new trigger shape).

**FP-001 probe list (16+ shapes):** PASS on all prior-pass shapes — adversary re-ran the full prior-pass FP-001 probe list at @3212070c; all shapes from passes 1–15 continue to behave correctly (zero regressions). The new failing shape (qualified ref suppressed by name-keyed bare_head_cols) is novel and distinct from all prior suspension shapes.

**POL-24 (byte-for-byte EC-body):** PASS — EC-11-076 body matches the canonical format.

**POLICY 33 N/A:** CONFIRMED — Policy 33 (multi-tenant isolation gate) is not applicable to this fix-burst. The PER-REFERENCE SCOPING correction applies to plan-time column availability checking only; it does not touch runtime data-path or tenant isolation logic.

---

## Post-Fix Verification

**fix-branch HEAD after fix-burst:** 35117a38 (LOCAL-ONLY)

**Test count:** 5391/5391 GREEN (6 new tests from EC-11-076 gate: 4 RED gates + 2 negative controls; +6 from 5385)

**Non-exhaustive gate:** 89/89 UNCHANGED (no new public types introduced)

**`just check` result:** 5391/5391 GREEN; non-exhaustive 89/89; fmt + clippy + layout PASS

---

## Convergence Assessment

**Trajectory:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)

**Pattern:** Pass 16 finds 1 MED on frozen 3212070c (HEAD 3212070c was the post-pass-15 fix-burst HEAD; pass 15 also found 1 MED on its frozen HEAD 3f2eddd1). The finding ADV-FIX-P16-MED-001 is a genuine scoping precision gap: the per-name HashSet<String> mechanism introduced at @3212070c to track bare head refs did not distinguish qualified from bare refs of the same name. Passes 1–15 probed FP-001 violations in the fail-open direction; pass 16 probed in the false-negative-widening direction and found the gap.

**Novelty assessment:** MEDIUM-HIGH — The same-name bare+qualified interaction at head positions 1–6 is a novel angle not surfaced in passes 1–15. Prior passes established the HEAD-JOIN SUSPENSION RULE itself (pass 15), the STAGE-JOIN SUSPENSION RULE (pass 14), and the STAR-WITH-JOIN SUSPENSION RULE (passes 12/13). Pass 16 tested the scoping precision of the pass-15 implementation and found that name-keyed tracking was insufficiently granular.

**Streak status:** 0/3 (BC-5.39.001). VERY NEXT ACTION: freeze 35117a38 → LOCAL adversary pass 17 (fresh context, strict; streak candidate 1/3). Three consecutive CLEAN(strict) passes on unchanged HEAD required (passes 17/18/19) → then push branch + open fix-PR via pr-manager.
