---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 7
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "1a8cc8aa"
feature_head_after_fix_burst: "1a8cc8aa"
clean_strict: true
clean_pr_merge: true
streak_after: "2/3"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 7 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 1a8cc8aa at review)
**Pass:** PR-LEVEL pass 7 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Pass-6 Closure Verification

Pass-6 was CLEAN(strict)=yes with zero findings. All pass-5 closures carried forward and
confirmed durable at HEAD 1a8cc8aa (same feature HEAD; no code changes since pass 6).

## Adversary Pass 7 Findings

**ZERO findings.**

Full adversarial sweep at HEAD 1a8cc8aa produced no findings of any severity.

This is the second consecutive CLEAN(strict) pass on the frozen HEAD 1a8cc8aa. All behavioral
contracts (BC-2.01.013 v1.14, BC-2.11.007 v1.8, BC-2.11.005 v1.6) re-derived independently.
Push-down correctness path (run_materialization_pipeline → adapter → DTU → DataFusion post-filter)
confirmed end-to-end. Inclusive-boundary semantics (EC-009) and RFC3339 Z-normalization confirmed
durable. SAP-1 (catalog 71 rows) and SAP-2 (CrowdStrike+Armis DTU↔TOML parity) both PASS.

## Summary

**CLEAN(strict):** yes (zero findings of any severity)
**CLEAN(PR-merge):** yes (zero findings)
**Streak:** 2/3 (streak 1/3 → 2/3 per BC-5.39.001 D-779; second consecutive strict-clean pass on 1a8cc8aa)
**Feature HEAD:** 1a8cc8aa (unchanged — zero findings, no fix-burst needed)
**Next step:** PR-LEVEL pass 8 (streak 2/3 → 3/3 target for convergence)

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | Full independent re-derivation of all behavioral contracts; all 17 ACs satisfied |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71, no unregistered event_type. SAP-2: CrowdStrike+Armis DTU↔TOML parity confirmed |
| SID-1 (no-ignored-test rationalization) | PASS | All load-bearing tests confirmed; no rationalization |
| Code documentation | PASS | All comments accurate and contract-anchored after pass-5 fixes |
| Input normalization | PASS | AQL leading-space trim present |
| Inclusive-boundary semantics (EC-009) | PASS | DTU over-fetch confirmed; Z-normalization confirmed |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions |
| Security | PASS (CLEAR-TO-MERGE per pass-1 verdict) | No new security surface |
| Demo evidence | PASS | Counts accurate at HEAD 1a8cc8aa |
| Story spec (v2.5) | PASS | No spec drift |
