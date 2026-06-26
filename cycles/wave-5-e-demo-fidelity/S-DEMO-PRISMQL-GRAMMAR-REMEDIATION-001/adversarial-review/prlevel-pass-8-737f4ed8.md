---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pr: "203"
pass_number: 8
frozen_pr_head: 737f4ed8
base: "903c8fcb"
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "NO"
finding_count: 2
timestamp: 2026-06-25T23:35:00Z
---

# PR-LEVEL Adversarial Review Pass 8 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**FROZEN PR HEAD:** `737f4ed8`
**Base:** `develop@903c8fcb`
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO

---

## Findings

### F-P2-MED-001 — inject_now_predicate InSubquery fold gap (MED, BLOCKING)

**Severity:** MED (blocking)
**Area:** Temporal predicate injection / Area B temporal normalization
**Root cause:** `inject_now_predicate` routed `Predicate::InSubquery` through the catch-all `other => other` arm and did NOT recurse into the subquery's WHERE/HAVING clauses. The detection side (`predicate_has_unfolded_temporal`) DID recurse into `Predicate::InSubquery`, creating a fold↔detect asymmetry. Consequence: `NOW()` / `INTERVAL` inside `WHERE x IN (SELECT ... WHERE ts > NOW()-INTERVAL '1h')` was left unfolded, causing `predicate_has_unfolded_temporal` to return `true` (correctly detecting unfold) while `inject_now_predicate` left the NOW literal in place, causing `normalize` to return `None` — the query was wrongly rejected with `E-QUERY-034` / `-32000 INTERNAL_ERROR ("report to support")` instead of executing correctly.

**Status:** CLOSED

**Fix:** Implementer commit `737f4ed8→0cdf24d1`. `inject_now_predicate` now recurses into `Predicate::InSubquery` via `inject_now_sql_query` (mirroring the detection side). A sibling sweep confirmed only `InSubquery` was asymmetric — all other `Predicate` arms were symmetric. The `lib.rs` comment describing the fold behavior was corrected. Load-bearing test `test_f_p2_med_001_inject_now_folds_inside_in_subquery` added: asserts that a pinned ISO timestamp literal is correctly injected inside the IN-subquery's WHERE clause with no residual NOW/INTERVAL/internal-error.

---

### OBS-1 — E-QUERY-037 availability-gate source-extractor InSubquery walk gap (was pre-existing S-3.13 gap; fixed in-scope)

**Severity:** OBS (fixed in-scope)
**Area:** E-QUERY-037 availability gate; `collect_predicate_sources_into_gate`
**Root cause:** The availability-gate source extractor's `Ast::Sql(Select)` and `Ast::SqlPipe` arms did not walk `WHERE InSubquery` sources — only the DML arm contained that logic. The result: a `WHERE x IN (SELECT * FROM sensitive_table WHERE ...)` construct in a `SELECT` or `SqlPipe` query would not register `sensitive_table` as requiring an availability check. No live defect today (the gate skips Internal/Composite source kinds, so CTE and internal tables are not false-gated), but the parity gap was a structural correctness issue.

**Status:** CLOSED (in-scope fix, clean, zero regressions)

**Fix:** Implementer commit `→950c19be`. Both the `Ast::Sql(Select)` arm (`sq.where_`) and `Ast::SqlPipe` arm (`spq.head.where_`) now call `collect_predicate_sources_into_gate` on their WHERE clause, mirroring the DML arm. The gate already correctly skips `SourceKind::Internal` and `SourceKind::Composite` (no false-gating of CTE/internal tables). Two load-bearing tests added. Full `prism-query` suite: 1144/1144 zero regressions.

---

### OBS-2 — D1 message keyword subset (SETTLED, no action)

**Severity:** OBS (settled)
**Status:** NO ACTION — D1 message keyword subset is verbatim per BC-2.11.023 §D1 (POL-24 authoritative). Do not flag.

---

## Post-fix State

**New FROZEN PR HEAD:** `950c19be`
**Pre-push `just check`:** EXIT 0 — full workspace (all 1672s), including all DTU tests (armis `test_BC_2_06_019_armis_primary_device_stage_visibility` confirmed pass via pre-push hook; a prior ad-hoc shell run showed an environmental flake, NOT a real failure).
**Non-exhaustive gate:** 87 (UNCHANGED)
**fmt-canonical:** clean
**SAP-1:** clean
**3-CLEAN streak:** RESET 0/3 on `950c19be`

---

## Environmental Note (armis test flake)

The implementer's ad-hoc `just check` mid-session reported a failure for `test_BC_2_06_019_armis_primary_device_stage_visibility`. This was an environmental flake in the agent shell. The authoritative pre-push `just check` hook ran the FULL workspace on `950c19be` to EXIT 0 (1672s), confirming all DTU tests pass. This is not a real failure and should not be flagged in future passes.

---

## Next Steps

- PR-LEVEL adversary: 3 consecutive CLEAN(strict) on UNCHANGED `950c19be`
- CI green on `950c19be`
- Squash-merge (--admin, D-1337)
- Post-merge POL-14 BC promotion
- Pre-flight demo re-audit
- T13 capstone → T14 recording
