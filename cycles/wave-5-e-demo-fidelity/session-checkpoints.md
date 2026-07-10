---
document_type: session-checkpoints-archive
cycle: wave-5-e-demo-fidelity
producer: state-manager
---

# Session Checkpoints Archive — wave-5-e-demo-fidelity

Archived session resume checkpoints superseded by newer snapshots.
Current checkpoint lives in `.factory/STATE.md §Session Resume Checkpoint`.

---

## Archived: D-1632 — 2026-07-09; STATE v8.232

**FIX-IEQ-ERRPATH-001 PR #219 OPEN @dacb60fa (base develop). LOCAL cascade CONVERGED (19 passes; streak 3/3 CLEAN(strict) on frozen 35117a38). Reviews completed on 35117a38: pr-reviewer APPROVE (0 blocking; NB-1/NB-2/NB-3 non-blocking); security-reviewer CLEAR (0 CRIT/HIGH/MED; SEC-FIND-001 CWE-117 CONFIRMED GENUINE — CLOSED in fix-burst 51f071ff; SEC-FIND-002 CWE-200 — CLOSED in 51f071ff). CI-FAIL-001 clippy (pre-existing on develop, toolchain drift) — CLOSED in dacb60fa. Fix-burst 51f071ff: sanitize_for_log @3 column_not_found.rejected sites + CWE-117 unit test; SEC-002 comment corrected; audit-script G2/G3/G6/G7/G8 WARN→FAIL; G4 heuristic→canonical anchor. Clippy dacb60fa: 3 match→? infusion/mod.rs. Branch PUSHED @dacb60fa (51f071ff + dacb60fa); just check 5392/5392 GREEN; non-exhaustive 89/89. PR-LEVEL cascade 0/3 on frozen dacb60fa (BC-5.39.001; DRIFT-ORCH-PRLEVEL-PUSH-001 — push reset streak). NB-1 PR-body fix via gh pr edit (no code push).**

**STATE v8.232. develop f935edb6 (PUSHED, origin==local; UNCHANGED until PR #219 merges). BC-INDEX v7.73. STORY-INDEX v2.645. ARCH-INDEX v2.174. error-taxonomy v2.34. active_contracts 257. draft_contracts 0. non-exhaustive EXPECTED=89. total_stories 228. workspace_test_count 5392 (fix-branch dacb60fa PUSHED; develop baseline 5319 @f935edb6). bc_count_corrected 266.**

**NOTE: Superseded by D-1633 — PR-LEVEL pass-1 NOT-CLEAN(2 MED+1 OBS) + same-burst closure; PR #219 HEAD @39c8b134; PR-LEVEL streak 0/3 on 39c8b134; workspace_test_count 5392→5397.**

---

## Archived: D-1433 — 2026-06-30; STATE v8.061

**STATE v8.061. SESSION WRAP D-1433. TEST-SPEED INITIATIVE COMPLETE (at time of writing; corrected by D-1435): S-PERF-GATE-004 (PR #209 @e3148007) + S-PERF-GATE-005 (PR #210 @8bc0404e) MERGED. Full nextest ~hours->86.4s. stop() 5.002s->0.019s. develop_head 8bc0404e. BC-INDEX v7.26. STORY-INDEX v2.533. ARCH-INDEX v2.150. error-taxonomy v2.03. active_contracts 254. draft_contracts 0. non-exhaustive EXPECTED=88. total_stories 222.**

**PR #208 STATUS (at time of writing):** S-DEMO-FIDELITY-REMEDIATION-001 Category-2 (BC-2.10.012 pql_hints UDF-discovery hints) implemented. Feature branch rebased LOCAL-ONLY onto develop@8bc0404e at 59474484 (band-aid removed; Category-2 intact). just check UNCONFIRMED (rebase agent killed mid pre-push verify). Origin feature branch + PR #208 still at stale 4a624a08. Prior LOCAL 3-CLEAN (was @481a0484) + PR-LEVEL cascade (was @4a624a08) VOIDED by rebase+Category-2. Streak 0/3.

**NEXT ACTION (at time of writing):** (1) In .worktrees/S-DEMO-FIDELITY-REMEDIATION-001: run `just check` to confirm green. (2) `git push --force-with-lease` to update origin feature branch + PR #208. (3) Fresh full adversarial re-convergence on new pushed HEAD (streak 0/3). (4) Demo evidence refresh (AC-CAT2 + de-pin). (5) PR-LEVEL 3-CLEAN(strict) -> security + pr-reviewer + CI green -> user-authorized squash-merge -> post-merge burst -> S-PRISMQL-CASE-INSENSITIVE-001.

**NOTE: Superseded by D-1435 PIVOT — T-PERF-PROFILE brought forward; PR #208 PARKED; test-speed initiative marked INCOMPLETE. See STATE.md Session Resume Checkpoint (D-1435).**

---

## Archived: D-1429 — 2026-06-30; STATE v8.057

**STATE v8.057. POST-MERGE BURST D-1429. S-PERF-GATE-004 MERGED — PR #209 squash-merged to develop@e3148007 2026-06-30T06:30:27Z. develop_head e3148007. RUSTSEC-2026-0190 closed (anyhow 1.0.102→1.0.103). Test-speed initiative milestone 1 COMPLETE. POL-14 NO-OP (BC-5.39.001 ACTIVE; no product BCs). CR-002 (LOW) deferred to future maintenance (14 crates direct `anyhow="1"`; dep-hygiene only). PR #208 (S-DEMO-FIDELITY-REMEDIATION-001): feature branch HEAD 4a624a08 (pre-rebase onto e3148007); PR #208 OPEN; PR-LEVEL streak 0/3 (passes 1+2 already completed per D-1426/D-1427). BC-INDEX v7.26. STORY-INDEX v2.530. ARCH-INDEX v2.150. error-taxonomy v2.03. active_contracts 254. draft_contracts 0. non-exhaustive EXPECTED=88. total_stories 221.**

**NEXT ACTION:** (1) `git -C .worktrees/S-DEMO-FIDELITY-REMEDIATION-001 rebase origin/develop` (rebase PR #208 HEAD onto e3148007; per orchestrator plan). (2) Revert `now+30` band-aid in DTU test files (stashed cosmetic [RED GATE] annotation cleanups: stash@{0} on #208 worktree needs reconciling post-rebase). (3) Resume PR #208 adversarial re-converge (re-gate on rebased HEAD) → PR-LEVEL 3-CLEAN(strict) → security + pr-reviewer → user-auth squash-merge (NO --admin) → post-merge state burst. (4) After PR #208: deliver S-PRISMQL-CASE-INSENSITIVE-001 (demo-critical; ADR-047).

**TRACK B — DAY-2 MORPH (POST-T14):** `.factory/specs/matured-vision-day2-requirements.md`. Demo target FROZEN. Brief reframe GATED on human sign-off.

**PENDING HUMAN AUTH:** (A) brief reframe sign-off; (B) EC-11 namespace collisions.

**RESUME PROTOCOL (zero prior context):**
0. Read SESSION-HANDOFF.md §RESUME SNAPSHOT D-1429 for full context.
1. `vsdd-factory:factory-worktree-health` (BLOCKING).
2. `git log --oneline -1 origin/develop` → expect `e3148007`.
3. S-DEMO-FIDELITY-REMEDIATION-001 branch HEAD 4a624a08 (pre-rebase onto e3148007); worktree .worktrees/S-DEMO-FIDELITY-REMEDIATION-001; PR #208 OPEN.
4. NEXT: rebase onto e3148007 → revert now+30 → adversarial re-converge → PR-LEVEL 3-CLEAN(strict) → user-auth squash-merge (NO --admin).
5. S-PRISMQL-SQLPIPE-COLUMN-GATE-001 + S-DTU-ARMIS-FIXTURE-VOCAB-001 draft stubs registered (P3; depend on S-DEMO merge).
6. Autonomy D-989 active.

---

## Archived: D-1441 — 2026-07-01; STATE v8.069

**STATE v8.069. D-1441 — S-PERF-GATE-006 LOCAL ADVERSARY PASS-5 FIX-BURST CLOSED. develop_head 8bc0404e. BC-INDEX v7.26. STORY-INDEX v2.540. ARCH-INDEX v2.150. error-taxonomy v2.03. active_contracts 254. draft_contracts 0. non-exhaustive EXPECTED=88. total_stories 224.**

**T-PERF-PROFILE DELIVERY PLAN:** Full optimization BEFORE PR #208 ships (human decision D-1436). Three-story sequence: S-PERF-GATE-006 (Justfile RUSTFLAGS="" fingerprint alignment; ~150s savings; 7 ACs; 1 pt; **draft v1.5**) → S-PERF-GATE-007 (.config/nextest.toml wasm-cap + http-cap groups; ~150-200s savings; 8 ACs; 2 pts; draft v1.0) → S-PERF-GATE-008 (shared wasmtime Engine — pending architect/research consult; NOT yet authored). Baseline: `just check` ~13.3min warm (report: `.factory/research/test-suite-perf-profile-2026-06-30.md`). Combined projected → ~4.5-6.3min.

**S-PERF-GATE-006 STATUS:** LOCAL adversary pass-5 closed (NOT CLEAN — F-006-MED-001 MED causal-model contradiction + OBS-1/OBS-2; all closed). Story v1.4→v1.5; feature HEAD UNCHANGED 089b36df (spec-only fix; no new code commit). Key insight: AC-007 check-fast alignment is load-bearing — fixing only `just check` clippy would move the ~157s penalty to `just clippy`, not eliminate it. 3-CLEAN streak 0/3 on 089b36df. NEXT: LOCAL adversary pass-6 re-gate on frozen 089b36df + story v1.5.

**PR #208 STATUS:** PARKED. Feature branch LOCAL HEAD 0978983f (NOT pushed; origin + PR #208 stale at 4a624a08). Local state: now-10 (band-aid reverted); 4 wall-clock-racy DTU scenario tests QUARANTINED with SID-1 `#[ignore]` (test_BC_2_06_019_armis_primary_device_stage_visibility, test_BPRL_P4_02_armis_alerts_stage_guard_primary_device, test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility, test_BPRL_P4_02_detections_stage_guard_primary_device). Streak 0/3. BLOCKED until S-PERF-GATE-006+007+008 merge.

**PENDING USER-APPROVED WORK:** LOCAL adversary pass-6 on S-PERF-GATE-006 089b36df + story v1.5 → 3-CLEAN → PR → merge → S-PERF-GATE-007 → S-PERF-GATE-008 (gated on architect/research consult). CR-002 (anyhow >=1.0.103 pin in 14 crates) DEFERRED -- non-urgent dep-hygiene.

**RESUME PROTOCOL (zero prior context):**
0. Read SESSION-HANDOFF.md RESUME SNAPSHOT D-1433 for prior context; D-1435 PIVOT + D-1436 delivery plan + D-1437 pass-1 + D-1438 pass-2 + D-1439 pass-3 + D-1440 pass-4 + D-1441 pass-5 fix-bursts recorded in STATE.md Current Phase Steps.
1. Run vsdd-factory:factory-worktree-health (BLOCKING).
2. `git log --oneline -1 origin/develop` → expect `8bc0404e`.
3. S-PERF-GATE-006 LOCAL adversary pass-6 pending on frozen 089b36df + story v1.5 (7 ACs; 3-CLEAN streak 0/3).
4. PR #208 PARKED at LOCAL 0978983f (4 tests quarantined; NOT pushed to origin).
5. NEXT: LOCAL adversary pass-6 on 089b36df + story v1.5 → 3-CLEAN → PR S-PERF-GATE-006 → merge → deliver S-PERF-GATE-007 → S-PERF-GATE-008 (pending arch/research) → rebase PR #208 + un-quarantine 4 tests → ship PR #208.
6. S-PRISMQL-SQLPIPE-COLUMN-GATE-001 + S-DTU-ARMIS-FIXTURE-VOCAB-001 draft stubs registered (P3; depend on S-DEMO merge).
7. Autonomy D-989 active.

---

## Session Resume Checkpoint (D-1464 -- 2026-07-01; STATE v8.084)

**STATE v8.084. D-1462 — S-PERF-GATE-007 pass-8 CLEAN(strict)=YES, CLEAN(PR-merge)=YES (zero findings in per-story perimeter; adversary verified all 9 ACs, all 11 binaries resolve per AC-009, ~190-260s savings consistent, §PR Evidence Framing Note internally consistent, POL-32 ok; REC-4 heading out-of-perimeter, did NOT reset streak; 3-CLEAN streak advanced to 1/3 on 2d11f540); D-1463 — S-PERF-GATE-006 pass-16 FIX-BURST CLOSED (OBS-1 `check` comment "all three non-fmt steps" imprecise; FIXED: implementer cfae9375 comment-only "the three cargo-compilation steps"; HEAD 442911b8→cfae9375; streak 0/3 on cfae9375; adversary noted next pass should be strict-clean); D-1464 — profiling report REC-4 heading corrected ~60-100s→~40-60s. develop_head 8bc0404e. BC-INDEX v7.26. STORY-INDEX v2.548. ARCH-INDEX v2.150. error-taxonomy v2.03. active_contracts 254. draft_contracts 0. non-exhaustive EXPECTED=87. total_stories 224. [PROCESS-GAP]: ~25 combined LOCAL passes; code verified correct throughout; findings consistently prose/doc-consistency on zero-blast-radius tooling stories. Human directive = continue strict 3-CLEAN on both.**

**S-PERF-GATE-006 STATUS:** LOCAL adversary pass-16 CLEAN(PR-merge)=YES / CLEAN(strict)=NO. OBS-1 OBS (`check` recipe comment "all three non-fmt steps" — recipe has 5 non-fmt steps, only 3 are cargo-compilation steps; phrasing invites miscount). FIXED: implementer comment-only (commit cfae9375) — comment updated to "the three cargo-compilation steps". Feature HEAD advanced 442911b8→cfae9375 (comment-only). **3-CLEAN streak 0/3 on cfae9375** (new HEAD per frozen-HEAD rule; pass-17 re-gate pending on cfae9375). Adversary noted: with OBS-1 fixed, next pass should be strict-clean. Story v2.1.

**S-PERF-GATE-007 STATUS:** IMPLEMENTED + VERIFIED + READY. 3-CLEAN LOCAL in progress (1/3 on 2d11f540). **3-CLEAN streak 1/3 on 2d11f540** (pass-8 CLEAN per BC-5.39.001 frozen-HEAD rule). Passes 9+10 must stay on unchanged 2d11f540. Story v1.6.

**PENDING USER-APPROVED WORK:** Human directive = continue strict 3-CLEAN on both. S-PERF-GATE-006 pass-17 on cfae9375 + story v2.1. S-PERF-GATE-007 pass-9 on 2d11f540 + story v1.6. S-PERF-GATE-008 (gated on architect/research consult).

---

## Session Resume Checkpoint (D-1633 -- 2026-07-09; STATE v8.233)

_Archived to session-checkpoints.md by D-1634 burst (state-manager keep-last-1 discipline)._

**FIX-IEQ-ERRPATH-001 PR-LEVEL pass-1 CLOSED (NOT-CLEAN 2 MED+1 OBS; all findings CLOSED same-burst). PR #219 OPEN @39c8b134 (7e23a2c2 + 39c8b134 on top of dacb60fa; just check 5397/5397 GREEN; non-exhaustive 89/89).**

**STATE v8.233. develop f935edb6 (UNCHANGED). BC-INDEX v7.74. STORY-INDEX v2.646. ARCH-INDEX v2.174. error-taxonomy v2.35. active_contracts 257. [Updated D-1633: PR-LEVEL pass-1 NOT-CLEAN + same-burst closure; PR #219 @39c8b134; PR-LEVEL streak 0/3]**

**LAST MERGED:** S-PRISMQL-CASE-INSENSITIVE-001 (PR #217 → develop@f935edb6, 2026-07-08).

**OPEN PRs:** PR #219 FIX-IEQ-ERRPATH-001 (https://github.com/drbothen/prism/pull/219; base develop; HEAD 39c8b134; pr-reviewer APPROVE (was on 35117a38 — re-review needed on 39c8b134); PR-LEVEL cascade 0/3 on frozen 39c8b134).

**OPEN FIX-CASCADE:** FIX-IEQ-ERRPATH-001 — LOCAL strict-3-CLEAN CONVERGED @35117a38 (passes 17/18/19). Pass-1 PR-LEVEL CLOSED (2 MED+1 OBS) @39c8b134; PR-LEVEL streak 0/3 (DRIFT-ORCH-PRLEVEL-PUSH-001 reset on push). At merge: closes DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 + DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001 (audit-script 62→70).

**WORKTREES:** FIX-IEQ-ERRPATH-001 at .worktrees/FIX-IEQ-ERRPATH-001 (ACTIVE; fix-branch 39c8b134 PUSHED; PR #219 OPEN; PR-LEVEL streak 0/3).

**PENDING-HUMAN:** (1) S-3.09 decision: resume vs keep-parked vs abandon (DRIFT-PARKED-S309-001). (2) W3-FIX-S307-001 decision: resume/commit-and-continue vs stash vs abandon (DRIFT-PARKED-W3FIX-S307-001). (3) E-OCSF-005..023 gap triage/prioritization (DRIFT-EOCSF-GAP-005-023-001).

**RESUME PROTOCOL:**
1. Run vsdd-factory:factory-worktree-health (BLOCKING).
2. `git log --oneline -1 origin/develop` → expect `f935edb6`.
3. `git -C .worktrees/FIX-IEQ-ERRPATH-001 log --oneline -1` → expect `39c8b134` (fix-branch pushed, PR-LEVEL pass-1 same-burst closure complete).
4. VERY NEXT ACTION: PR-LEVEL adversary pass 2 on frozen 39c8b134 (BC-5.39.001; streak candidate 1/3).
5. On merge: state-manager post-merge burst (STATE v8.233→v8.234+).
6. Surface PENDING-HUMAN items 1-3.

---

## Session Resume Checkpoint (D-1637 -- 2026-07-09; STATE v8.237)

_Archived to session-checkpoints.md by D-1638 burst (state-manager keep-last-1 discipline)._

**FIX-IEQ-ERRPATH-001 PR-LEVEL pass-5 NOT-CLEAN(3 MED prose-citation) on frozen 8610ecd0 + same-burst spec/story-only closure. PR #219 OPEN @8610ecd0 (base develop). PR HEAD UNCHANGED — no code push. PR-LEVEL streak stays 0/3. ADV-PR-P5-MED-001/002 (BC-2.11.016 §Preconditions.2 position-8 prose + position-11 prose wrong 3 ways) CLOSED PO BC-2.11.016 v1.25. ADV-PR-P5-MED-003 (story AC-M2 recursion claim — bare-Field arm calls extract_column_name_from_field_path directly) CLOSED story-writer. Class-closure sweep: position-4 table (OrderBy::expr→OrderExpr::expr) + 4 BC-2.11.004 §Error Cases fixes folded into v1.30. BC-2.11.017 v1.13 / BC-2.11.020 v1.18 (pin-only). Story pin round v2.44 / v2.20 / v1.29 / v1.54. BC-INDEX v7.76→v7.77; STORY-INDEX v2.648→v2.649. workspace_test_count 5397 UNCHANGED; just check 5397/5397 GREEN; non-exhaustive 89/89.**

**STATE v8.237. develop f935edb6 (PUSHED, origin==local; UNCHANGED until PR #219 merges). BC-INDEX v7.77. STORY-INDEX v2.649. ARCH-INDEX v2.174. error-taxonomy v2.35. active_contracts 257. draft_contracts 0. non-exhaustive EXPECTED=89. total_stories 228. workspace_test_count 5397 (fix-branch 8610ecd0 PUSHED; develop baseline 5319 @f935edb6). bc_count_corrected 266. [Updated D-1637: PR-LEVEL pass-5 NOT-CLEAN(3 MED) + same-burst spec/story-only closure; PR HEAD 8610ecd0 UNCHANGED; streak stays 0/3]**

**LAST MERGED:** S-PRISMQL-CASE-INSENSITIVE-001 (PR #217 → develop@f935edb6, 2026-07-08).

**OPEN PRs:** PR #219 FIX-IEQ-ERRPATH-001 (https://github.com/drbothen/prism/pull/219; base develop; HEAD 8610ecd0; PR-LEVEL streak 0/3 on frozen 8610ecd0).

**OPEN FIX-CASCADE:** FIX-IEQ-ERRPATH-001 — LOCAL strict-3-CLEAN CONVERGED @35117a38 (passes 17/18/19). Pass-1 through pass-5 all done; pass-5 NOT-CLEAN(3 MED); PR HEAD UNCHANGED @8610ecd0; streak stays 0/3.

**RESUME PROTOCOL:**
1. Run vsdd-factory:factory-worktree-health (BLOCKING).
2. `git log --oneline -1 origin/develop` → expect `f935edb6`.
3. `git -C .worktrees/FIX-IEQ-ERRPATH-001 log --oneline -1` → expect `8610ecd0` (fix-branch pushed; PR-LEVEL streak 0/3; streak candidate 1/3 starts at pass 6).
4. VERY NEXT ACTION: PR-LEVEL adversary pass 6 on frozen 8610ecd0 (streak candidate 1/3).

---

## Session Resume Checkpoint (D-1658 -- 2026-07-10; STATE v8.258)

_Archived to session-checkpoints.md by D-1659 burst (state-manager keep-last-1 discipline)._

**EQUERY042 DEFECT CLOSED.** PR #220 squash-merged develop@b9cf3f9b 2026-07-10. Fix: E-QUERY-042 Literal::Timestamp arm in GROUP BY/ORDER BY (ADR-052 §D4 v1.11 arms 6+7); 15 new tests. Full cascade: LOCAL 5-pass (3-CLEAN @7db0b1ba; D-1654) + PR-LEVEL 3-pass ALL CLEAN(strict) on frozen 7db0b1ba. CI PASS; security CLEAR; pr-reviewer APPROVE cycle 1. develop_head 8ea29823→b9cf3f9b.

**CSDEVICES LOCAL cascade in progress.** Pass-6 fix-burst COMPLETE (D-1658). Pass-6 (frozen @30217403): NOT CLEAN(strict) CLEAN(PR-merge) — 1 LOW F-CSD-P6-001 (check_expr_insubquery_projection lacked DML source_select defense-in-depth arm; zero current exploitability, S-3.06 forward risk). All other probes CLEAR (HAVING grammar-unreachable for Expr::InSubquery; BUNDLED_SPEC_SCHEMAS compile-time fresh; DTU body handling in-scope-clear; CWE-209 consistent; version chain intact). Fix-burst: implementer @3d48b6a9 T20 RED→GREEN (DML source_select arm; comment corrected); 20/20 defect tests; 15/15 temporal; 1522/1522 prism-query; just check GREEN; non-exhaustive 89/89. No spec changes (code-only). Streak 0/3. LOCAL pass 7 IN FLIGHT on frozen `3d48b6a9`. `.worktrees/FIX-CSDEVICES-EMPTY-PIPELINE` active on `fix/csdevices-empty-pipeline`.

**STATE v8.258. develop b9cf3f9b (PUSHED, origin==local). BC-INDEX v7.82. STORY-INDEX v2.653. ARCH-INDEX v2.175. error-taxonomy v2.38. active_contracts 257. draft_contracts 0. non-exhaustive EXPECTED=89. total_stories 229. workspace_test_count 5397 ON develop@8ea29823 (+15 new tests from PR #220 unverified on develop@b9cf3f9b). bc_count_corrected 266.**

**LAST MERGED:** DEFECT-EQUERY042-GROUPBY-DEADARM-001 (PR #220 → develop@b9cf3f9b, 2026-07-10).

**OPEN PRs:** NONE (AUDIT-COVERAGE-001 PR not yet opened; CSDEVICES fix-cascade in progress).

**RESUME PROTOCOL:**
1. Run vsdd-factory:factory-worktree-health (BLOCKING).
2. `git log --oneline -1 origin/develop` → expect `b9cf3f9b`.
3. `git -C .factory log -1 --format="%h %s"` → factory-artifacts HEAD (do not hard-code).
4. VERY NEXT ACTION = LOCAL pass 7 result on frozen `3d48b6a9` (`fix/csdevices-empty-pipeline`; streak 0/3).
