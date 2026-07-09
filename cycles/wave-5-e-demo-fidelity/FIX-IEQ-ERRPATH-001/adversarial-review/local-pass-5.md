---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [5]
feature_head_at_review: ffcdc5fe
fix_burst_head: fdfa78f2
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  OBS: 2
  total: 2
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
adjudication_note: "BACKFILL (D-1617). Adversary emitted CLEAN(strict)=yes — CONTRADICTION: 2 OBS findings listed while asserting zero findings. OBS-1 independently re-classified to CONFIRMED FP-001 VIOLATION by orchestrator empirical verification (D-1615); pass-5 verdict adjudicated NOT CLEAN(strict). Spec closure committed in D-1615 wrap; code closure @fdfa78f2 committed in D-1616."
---

# LOCAL Adversary Pass 5 — FIX-IEQ-ERRPATH-001

> **BACKFILL NOTE:** This report was authored in D-1617 to document pass-5 (taken during D-1615 session). Findings and closure records are drawn from D-1615 orchestrator notes and the L21 lessons entry.

---

## Pass 5 (frozen ffcdc5fe; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result issued by adversary:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES — **CONTRADICTION** (2 OBS listed while asserting zero findings; see L21 verdict-arithmetic invariant)

**Adjudicated result (orchestrator D-1615):** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 2 (OBS-1: FP-001 violation under-classified as LOW-confidence OBS; OBS-2: verdict-arithmetic process gap); 1 code-behavior defect confirmed

**Code HEAD at review:** ffcdc5fe (frozen; post-pass-4 fix-burst HEAD; agg-arg walk positions 11+)

**Fix-burst HEAD:** fdfa78f2 (D-1616: implementer compute_sqlpipe_head_binding() helper; SqlPipe head-projection seeding per BC-2.11.016 v1.13)

**LOCAL 3-CLEAN(strict) streak after pass-5:** 0/3 (NOT CLEAN(strict) per orchestrator adjudication; fix-burst dispatched; RESET by @fdfa78f2 push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P5-OBS-001 — SqlPipe head aliases never seed downstream binding context (FP-001 VIOLATION; under-classified)

**Severity issued by adversary:** OBS (LOW-confidence, "possibly false positive")

**Severity adjudicated:** CONFIRMED FP-001 VIOLATION (MED/HIGH class — false E-QUERY-038 emitted on valid queries)

**Classification:** code-behavior — FP-001 false positive (SqlPipe head-projection context not seeded into downstream pipe-stage binding; aliases and bare field names invisible to pipe stage column walk)

**Affected files:** `crates/prism-query/src/engine.rs` (compute_sqlpipe_head_binding absent; SqlPipe arm in check_pipe_stage_columns invoked with None binding override)

**BC reference:** BC-2.11.016 v1.12 §Preconditions.2 FP-001 invariant; DERIVED-COLUMN BINDING RULE

**Finding:** The adversary observed that SqlPipe queries of the form `SELECT count(*) AS cnt ... GROUP BY severity | sort cnt` and `SELECT severity AS sev FROM t | where sev='High'` may produce false E-QUERY-038 errors when pipe stages reference columns that are outputs of the SqlPipe head (aliases or bare selected names). The adversary classified this as "LOW-confidence possibly FP-001" and nonetheless emitted CLEAN(strict)=yes. The orchestrator constructed and executed both query shapes against the fix-branch (ffcdc5fe) and confirmed: both produce false E-QUERY-038 errors on the fix-branch; both execute cleanly on develop baseline. The finding is a CONFIRMED FP-001 VIOLATION, not a low-confidence observation.

Root cause: `compute_sqlpipe_head_binding()` does not exist in ffcdc5fe. The SqlPipe arm in `check_pipe_stage_columns` calls with `initial_binding_override = None`, causing the pipe-stage walk to use raw source schema only. Head output columns (aliases, bare selected names, GROUP BY names) are absent from the binding context visible to `| where`, `| sort`, and other downstream pipe stages.

**Closure:** CLOSED — BC-2.11.016 v1.12→v1.13 (D-1615 spec layer): SqlPipe HEAD-PROJECTION BINDING RULE added to §Preconditions.2 — FOURTEEN-position gate extended so that SqlPipe head output columns (aliases ∪ bare selected names ∪ GROUP BY refs) are seeded into initial-available binding context before pipe-stage walk; anonymous unaliased non-Field items suspended; EC-11-059 (head alias seeds binding; downstream pipe alias ref valid), EC-11-060 (head bare name seeds binding; downstream pipe ref valid), EC-11-061 (head star expands to schema-derived cols; downstream pipe ref valid). BC-2.11.004 v1.18→v1.19 + BC-2.11.020 v1.6→v1.7 + taxonomy v2.26→v2.27 (POL-25 propagation). BC-INDEX v7.64→v7.65. Code: implementer @fdfa78f2 (D-1616) — `compute_sqlpipe_head_binding()` helper in `engine.rs` ({AS aliases} ∪ {bare un-aliased Field names} ∪ {bare GROUP BY field names}; SELECT * → None → raw-schema fallback; anonymous un-aliased non-Field → suspended fail-open per FP-001); threaded as `initial_binding_override` into `check_pipe_stage_columns` at 2 call sites (Ast::Pipe passes None, Ast::SqlPipe passes computed binding); 31/31 module GREEN; just check 5350/5350 GREEN; non-exhaustive 89/89.

---

## Finding ADV-FIX-P5-OBS-002 — Verdict-arithmetic process gap: CLEAN(strict)=yes asserted while OBS findings present

**Severity:** OBS [process-gap]

**Classification:** adversary process discipline — verdict arithmetic inconsistency (L21 class)

**Finding:** The adversary report for pass-5 asserted `CLEAN(strict)=yes` in the verdict line while simultaneously listing 2 OBS findings in the body. Per BC-5.39.001 and the project-local disambiguation (CLAUDE.md §Strict vs PR-Merge Convergence), CLEAN(strict) requires ZERO findings of ANY severity including OBS and PROCESS-GAP. A report asserting CLEAN(strict)=yes while listing OBS findings is self-contradictory; the orchestrator cannot dispatch on an inconsistent signal. This is the same defect class documented in L21 (FP-001 classification undersells + verdict arithmetic inconsistency). The inconsistency forced an empirical investigation round (orchestrator-driven query execution) to resolve the FP-001 candidate before a fix-burst could be dispatched.

**Closure:** PROCESS — L21 codified the adversary verdict arithmetic invariant and FP-001 empirical verification requirement (D-1615). Pass-5 verdict adjudicated NOT CLEAN(strict). Remediation: BC-2.11.016 v1.13 + implementer @fdfa78f2 (see OBS-001 above).

---

## Fix-burst Summary

**Chain:** ffcdc5fe (frozen pass-5 HEAD — agg-arg walk; FP-001 violation confirmed empirically) → [D-1615 spec layer] BC-2.11.016 v1.12→v1.13 (HEAD-PROJECTION BINDING RULE + EC-11-059/060/061) + BC-2.11.004 v1.18→v1.19 + BC-2.11.020 v1.6→v1.7 + taxonomy v2.26→v2.27 + BC-INDEX v7.64→v7.65 → fdfa78f2 (D-1616 implementer: compute_sqlpipe_head_binding(); 31/31 module GREEN; just check 5350/5350 GREEN; non-exhaustive 89/89)

**Spec closures:**

- BC-2.11.016 v1.12→v1.13 (D-1615): SqlPipe HEAD-PROJECTION BINDING RULE added; EC-11-059/060/061; fourteen-position gate extension
- BC-2.11.004 v1.18→v1.19 (D-1615): POL-25 companion — SqlPipe head output columns note for positions 8/9/10/11/12
- BC-2.11.020 v1.6→v1.7 (D-1615): POL-25 companion — same §Error Cases note + BC anchor pins→v1.13
- error-taxonomy v2.26→v2.27 (D-1615): POL-25 sweep + frontmatter-lag repair
- BC-INDEX v7.64→v7.65 (D-1615)

**Story pins FROZEN** per D-1615 — story-writer pin round deferred to D-1616 pre-pass-6 (before LOCAL adversary pass-6 dispatch).

**LOCAL 3-CLEAN(strict) streak after pass-5:** 0/3 (NOT CLEAN(strict) per orchestrator adjudication). **NEXT:** implementer @fdfa78f2 GREEN → story-writer pin round → state burst D-1616 → freeze fdfa78f2 → LOCAL pass 6.
