---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [14]
feature_head_at_review: 09ea9979
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 4
  low: 1
  obs: 3
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 14 — FIX-IEQ-ERRPATH-001

---

## Pass 14 (frozen 09ea9979; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 2/3 — RESET)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 4 total (0 CRIT / 0 HIGH / 0 MED / 1 LOW / 3 OBS / 0 PROCESS-GAP)

**STREAK RESET: 1/3 → 0/3** (BC-5.39.001). Fix-burst required.

**Code HEAD at review:** 09ea9979 (frozen; D-1623 fix-burst: EC-11-070/071 star-with-join suspension — BC-2.11.016 v1.18 STAR-WITH-JOIN SUSPENSION RULE + doc-count fix; 5371/5371 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** no (1 LOW + 3 OBS present; streak criterion requires ZERO findings of ANY severity)

**CLEAN(PR-merge):** yes (ZERO findings of CRIT + HIGH + MED severity; PR-merge-gate threshold met)

**All 4 findings CLOSED same-burst** (D-1625): BC-2.11.016 v1.19 STAGE-JOIN SUSPENSION RULE + 4 sibling BC/taxonomy pins + 4 story pin round + test-writer RED @dc81d8b9 + implementer GREEN @3f2eddd1; 5373/5373 GREEN; non-exhaustive 89/89. Streak RESET to 0/3. VERY NEXT ACTION: freeze 3f2eddd1 → LOCAL adversary pass 15 (fresh context, strict).

---

## Finding ADV-FIX-P14-OBS-001 — STAGE-JOIN: PipeStage::Join fell into catch-all with no suspension

**Severity:** LOW (orchestrator-upgraded from OBS: behavioral correctness gap, FP-001 class; ADV-FIX-P14-OBS-001)

**Confidence:** MEDIUM (adversary graded initial confidence LOW; orchestrator confirmed behavioral gap — PipeStage::Join was not an explicit arm in the binding-context walk, meaning it fell into the default catch-all path with NO suspension and NO DERIVED seeding; unqualified downstream references to join-source-only columns falsely fired E-QUERY-038)

**Finding:** The `check_pipe_stage_columns` binding-context walk at @09ea9979 enumerates explicit arms for `PipeStage::Where`, `PipeStage::Stats`, `PipeStage::Fields`, `PipeStage::Enrich`, `PipeStage::Dedup`, `PipeStage::Sort`, and `PipeStage::Limit`. `PipeStage::Join` was not an explicit arm; it fell into the default catch-all. The catch-all comment (at @09ea9979) described this as "fail-open per existing policy," implying Join's fall-through was intentional. However:

1. The catch-all does NOT call `set_suspended(true)` — it simply does not modify the binding context.
2. A downstream `| where some_join_source_col` where `some_join_source_col` is only available from the join partner (not from the FROM table schema) will reach the `is_registered()` gate with the FROM-only schema, find the column absent, and fire `E-QUERY-038` — a false positive.
3. BC-2.11.016 v1.18 §FP-001 invariant mandates that any pipe stage which introduces column aliases or alters the available column set must either seed the binding context or set `suspended:=true`. The Join stage introduces join-partner columns into the logical row but cannot determine which columns come from which partner at plan time without schema-union resolution. FP-001 mandates suspended:=true in this case.

**FP-001 class:** stage-join (newly classified at D-1625)

**Closure:**
- product-owner authored BC-2.11.016 v1.19 STAGE-JOIN SUSPENSION RULE: explicit `PipeStage::Join(_)` arm sets `suspended:=true` for remainder of walk; FP-001 trigger list extended to cover explicit Join stage shape; EC-11-072 + EC-11-073 added; EC-11-072 example corrected to parseable alias-free form (see ADV-FIX-P14-OBS-004 for alias-field background); BC-2.11.017 v1.7 / BC-2.11.020 v1.12 / BC-2.11.004 v1.24 / error-taxonomy v2.32 sibling-synced.
- test-writer RED @dc81d8b9: EC-11-072 red gate test (unqualified join-source downstream ref fires ColumnNotFound on frozen 09ea9979) + EC-11-073 red gate test (MIXED-STAR + head-join same failure mode).
- implementer GREEN @3f2eddd1: explicit `PipeStage::Join(_) => { ctx.suspended = true; }` arm inserted before catch-all; catch-all comment corrected (see ADV-FIX-P14-OBS-002). Both EC-11-072 and EC-11-073 tests GREEN at @3f2eddd1. Full `just check` 5373/5373 GREEN; non-exhaustive 89/89.

**Status:** CLOSED (BC v1.19 + test RED @dc81d8b9 + implementer GREEN @3f2eddd1)

---

## Finding ADV-FIX-P14-OBS-002 — Catch-All Comment Falsely Claimed Join "Fail-Open Per Existing Policy"

**Severity:** OBS (HIGH-confidence; misleading commentary creates spec-to-code misalignment risk for future implementers)

**Confidence:** HIGH (text is present verbatim in the catch-all arm; the claim is demonstrably false — the catch-all does not set suspended, and falls through with unchanged binding context, which is not "fail-open" behavior when join-source columns are referenced downstream)

**Finding:** The default/catch-all arm in `check_pipe_stage_columns` at @09ea9979 carried a comment claiming Join "fail-open per existing policy." This comment was incorrect:

- "Fail-open" in the FP-001 context means the gate suspends and allows downstream references through without triggering E-QUERY-038.
- The catch-all at @09ea9979 does NOT suspend; it leaves the binding context unchanged. A downstream reference to a join-source-only column would reach the `is_registered()` gate unimpeded and produce a false E-QUERY-038.
- "Existing policy" implies a prior intentional decision. No such decision was recorded in BC-2.11.016 or SESSION-HANDOFF decisions log. This appears to be an explanatory comment added during pass-12 fix-burst that misdescribed the behavior.

**Closure:** @3f2eddd1 — `PipeStage::Join` removed from catch-all enumeration; explicit arm `PipeStage::Join(_) => { ctx.suspended = true; }` inserted (closes ADV-FIX-P14-OBS-001 structural gap). Catch-all comment updated to remove the false claim; enumeration updated to accurately reflect which stages reach the catch-all (Limit, tail variants, and any future stages not yet enumerated). Comment-only correction confirms the implemented behavior (remaining catch-all stages: no binding-context mutation, pass-through semantics for stages that do not alter column availability).

**Status:** CLOSED (@3f2eddd1 — comment corrected; Join removed from catch-all)

---

## Finding ADV-FIX-P14-OBS-003 — Coverage Gap: MIXED-STAR + Head-Join Shape Not Covered by Test

**Severity:** OBS (coverage gap; behavioral claim in BC-2.11.016 §FP-001 re: MIXED-STAR + join suspends; no test at @09ea9979)

**Confidence:** MEDIUM (BC-2.11.016 v1.18 documents that branch (c) MIXED-STAR path sets `suspended:=true` when JOIN list is non-empty; EC-11-071 covers this for SELECT * style; a MIXED-STAR head `SELECT *, t.col FROM t JOIN u ON ...` with a downstream join-source reference was not covered by a dedicated test)

**Finding:** EC-11-071 (bare SELECT * with JOIN at @09ea9979) covers the plain-star shape. However, the MIXED-STAR path (branch (c): `SELECT *, expr AS alias, bare_field, ...` with a non-empty JOIN list) lacked a dedicated red-gate test verifying that the STAGE-JOIN suspension applies to this shape as well. Branch (c) sets `suspended:=true` additively alongside DERIVED seeding — but this interaction was untested for the join-present case.

**Closure:** EC-11-073 added to BC-2.11.016 v1.19 as the spec anchor for this shape (MIXED-STAR + head-join; fails open, no false E-QUERY-038). Test-writer RED @dc81d8b9 covers EC-11-073 (MIXED-STAR with explicit Join stage + downstream join-source-only column ref). Implementer GREEN @3f2eddd1 confirms branch (c) + explicit `PipeStage::Join(_)` arm cooperate correctly: `suspended:=true` set before any subsequent DERIVED seeding steps for the mixed-star items.

**Status:** CLOSED (EC-11-073 spec anchor + RED @dc81d8b9 + GREEN @3f2eddd1)

---

## Finding ADV-FIX-P14-OBS-004 — Dispatch-Memo Path Typo (Memo-Only, No Artifact Impact)

**Severity:** OBS (memo-only; no artifact impact; process-hygiene note)

**Confidence:** HIGH (path is wrong; the file exists at the src/tests/ location, not the tests/ location; the behavior tested is identical regardless)

**Finding:** The orchestrator dispatch memo (and prior pass reports in this cascade where the path was carried forward) cited the path for the BC-2.11.019 N1B test file as:

```
crates/prism-query/tests/bc_2_11_019_n1b_test.rs
```

The actual path is:

```
crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs
```

`crates/prism-query/tests/` holds integration tests (external crate tests). `crates/prism-query/src/tests/` holds in-process unit tests. The N1B test file is an in-process test at `src/tests/`. This distinction matters for SID-1 compliance: in-process tests run without subprocess overhead and are the preferred mechanism for behavioral coverage in this cascade.

**Impact:** Memo-only. No factory artifact (BC, story, or pass report) computes behavior from this path citation. No behavioral regression or false-passing test. Pass reports in the cascade that cited the tests/ path (including this correction note) should be read as referencing `src/tests/bc_2_11_019_n1b_test.rs`.

**Closure:** No code or spec change needed. This finding is closed via correction-in-record: the correct path is `crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs`. Future dispatch memos must use the src/tests/ path for in-process test references in prism-query. Correction recorded here and noted in convergence-trajectory.

**Status:** CLOSED (correction-in-record; no artifact change required)

---

## Additional Record: EC-11-072 Example Parseability Correction

During the fix-burst authoring, the test-writer discovered that the initial BC-2.11.016 v1.19 EC-11-072 example used an aliased JOIN form:

```
FROM crowdstrike_alerts | join some_other_table AS u ON severity == u.id | where u.col = 'x'
```

This form is unparseable: the PrismQL pipe-mode JOIN grammar (`PipeStage::Join` in the AST) does not support an alias field. `JoinStage` in the Rust AST has no `alias` field — the join partner is referenced by its bare table name. The test-writer confirmed this by inspecting the `JoinStage` struct definition at @09ea9979.

The product-owner corrected BC-2.11.016 v1.19 at 3 sites (EC-11-072 main example, the test-vector table row, and the canonical-form reference in the story anchor) before commit, replacing with the parseable alias-free form:

```
FROM crowdstrike_alerts | join some_other_table on severity == id | where col = 'x'
```

A grammar note was added to BC-2.11.016 §Grammar-constraints: `JoinStage` carries no alias field; the join partner is referenced by its literal table name at all downstream sites. This is NOT a new behavioral rule — it is a clarification of the existing grammar surface. The test at @3f2eddd1 uses the alias-free form and passes GREEN.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 09ea9979. No new `event_type` assignments introduced at @09ea9979 vs prior reviewed HEAD. Five total catalog rows verified (three `column_not_found.rejected` emission sites + two `reload.*` sites). Zero gaps. The fix-burst at @3f2eddd1 adds only a `PipeStage::Join(_)` arm with `ctx.suspended = true` — no new tracing emission site; SAP-1 coverage unchanged post-fix.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-072 and EC-11-073 added to BC-2.11.016 v1.19 carry full field schema, audit role, and recurrence policy in byte-parity with the canonical EC-11-039..071 body format.

**Audit-script Section G arithmetic:** VERIFIED — Section G count 62 → 70: EC-11-039..071 (33 entries from pass-12 green) + EC-11-072 + EC-11-073 (2 new entries from D-1625) = 35 entries in the EC-11-NNN range from this cascade. Arithmetic correct; no gap or duplicate.

**POLICY 33 N/A:** CONFIRMED — Policy 33 (multi-tenant isolation gate for cross-tenant data access) is not applicable to this fix-burst. The STAGE-JOIN SUSPENSION RULE applies to plan-time binding-context walk only; it does not touch runtime data-path or tenant isolation logic.

---

## Post-Fix Verification

**fix-branch HEAD after fix-burst:** 3f2eddd1

**Test count:** 5373/5373 GREEN (added EC-11-072 + EC-11-073 tests: +2 from 5371)

**Non-exhaustive gate:** 89/89 UNCHANGED (no new public types introduced)

**`just check` result:** 5373/5373 GREEN; non-exhaustive 89/89; fmt + clippy + layout PASS

---

## Convergence Assessment

**Trajectory:** 6 → 3 → 3 → 2 → 1 → [0] → 2 → [0] → 4(low/obs)

**Pattern:** Pass 14 finds 1 LOW + 3 OBS on a frozen HEAD that was CLEAN at pass 13. The low finding (ADV-FIX-P14-OBS-001) is a genuine behavioral gap — PipeStage::Join fell into the catch-all without suspension, causing false E-QUERY-038 on join-source-only downstream column references. This is a rotated-angle find: prior passes covered star+JOIN via the compute_sqlpipe_head_binding head-projection path; pass 14 probed the pipe-stage walk itself for explicit Join stage handling and found the gap.

The two OBS findings (OBS-002 comment + OBS-003 coverage) are structural companions to OBS-001. OBS-004 is a memo-path hygiene note with no behavioral impact. All 4 findings were closed same-burst (D-1625).

**Novelty assessment:** LOW-to-MEDIUM — The PipeStage::Join pipe-stage-walk gap (OBS-001) is a novel angle not surfaced in passes 1–13. The prior passes probed the head-projection binding path; this pass probed the pipe-stage walk itself. The gap was plausible given that head-projection suspension (star+JOIN) and pipe-stage walk are two distinct code paths. OBS-002/003 are structural companions. OBS-004 is process-hygiene. After D-1625 closure, the behavioral surface is more complete: both the head-projection path (EC-11-070/071) and the pipe-stage walk (EC-11-072/073) now correctly suspend on JOIN-bearing shapes.

**Streak status:** 0/3 RESET (BC-5.39.001). VERY NEXT ACTION: freeze 3f2eddd1 → LOCAL adversary pass 15 (fresh context, strict). Three consecutive CLEAN(strict) passes on unchanged HEAD required (passes 15/16/17) → then push branch + open fix-PR via pr-manager.
