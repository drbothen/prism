---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: vsdd-factory:adversary
timestamp: 2026-07-18T21:45:00Z
phase: 3
inputs:
  - .worktrees/AUDIT-COVERAGE-001/scripts/t13-preflight-audit.py
  - .worktrees/AUDIT-COVERAGE-001/CLAUDE.md
input-hash: "48230cc"
traces_to: stories/AUDIT-COVERAGE-001-t13-preflight-audit-coverage.md
pass: 2
previous_review: adversarial-review/post-rebase-pass-1.md
story_id: AUDIT-COVERAGE-001
scope: LOCAL
feature_head_at_review: 98bb1de2
date: 2026-07-18
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
streak_after: "2/3"
convergence: NOT_CONVERGED
---

# Adversarial Review — F-AUD-R2, LOCAL pass 2

**Perimeter:** LOCAL cascade (script + governance doc)
**Frozen HEAD:** `98bb1de2` (UNCHANGED since pass 1)
**Date:** 2026-07-18
**Scope:** 2 files vs `origin/develop@277b7844` — `scripts/t13-preflight-audit.py` (+4938/−502), `CLAUDE.md` (+30/−2)
**Streak entering:** 1/3

## Top-line counts
CRIT: 0 | HIGH: 0 | MED: 0 | LOW: 0 | OBS: 0 | PROCESS-GAP: 0

**No findings.** Fresh-context re-derivation (not inherited from pass 1) confirms the artifacts hold.

## Per-axis results

**Fail-loud integrity (POL-34) — PASS.** Strict-success predicate at line 5733 gates both DEMO-READY and sys.exit. Every escape hatch forces DEMO-READY: NO + exit 1: _PrismCrashError → results["CRASH"]="FAIL:…" (5460); unexpected exceptions → AUDIT_INTERNAL_ERROR FAIL (5468); INFO-bucket leak → counter-parity fail folds into _has_mismatch (5726); coverage-count drift → hard sys.exit(1) before any check runs (5624–5631). Traced every branch of the summary block — no path reports YES while a behavior is broken; only PASS-writing sites are per-check, each with a non-vacuous assertion.

**Matrix↔results parity — PASS.** Bidirectional gate (5648–5669): _matrix_only and _results_only both set _has_mismatch. ID-grammar regex correctly matches [H14b]/[H13a], excludes synthetic keys (BOOT/CRASH/AUDIT_INTERNAL_ERROR). Matrix count independently verified: A(23)+B(15)+C(8)+D(5)+E(6)+F(6)+G(8, G5 retired)+H(35) = 106 = EXPECTED_COVERAGE_COUNT.

**Error-template grounding vs canonical v2.56 (POL-24) — PASS.** Every anchor matches canonical templates: E-QUERY-032, E-QUERY-037 (both anchors), E-QUERY-038 (incl. structuredContent.error wire assertion at H2/F5), E-QUERY-039, E-QUERY-041, E-QUERY-042 all three arm messages (H5/H5b/H5c), E-QUERY-040, E-QUERY-033 (+ -32602), E-QUERY-043, E-QUERY-003.

**E-SENSOR taxonomy closure — verified HELD.** B14/B15 assert sensor_errors carries E-SENSOR-030 (1812–1828); E-SENSOR-030 now canonically registered (v2.56 line 539, degraded/fan_out, SensorError::AllTargetsFailed); semantics match single-source no-route query. Fail-loud direction safe. Probed the -32000 INTERNAL_ERROR mapping tension vs success-envelope+sensor_errors path: BC-2.01.010 governs the partial-failure array distinctly — both coexist. No defect.

**Rebase-literal integrity vs develop@277b7844 — PASS.** Tool catalog (14 LIVE + 40 NYA, byte-grounded in server.rs with CONSCIOUS-UPDATE notes), crowdstrike_devices in _DATA_GUARANTEED (matches merged CSDEVICES fix), E-QUERY-043 gate (H24), row-shape null-not-absent (H20 asserts threat_score key PRESENT-even-when-null per BC-2.11.001).

**POL-22 entity/citation — PASS.** Grounded in real entities: server.rs constants, resources.rs build_resource_list (3 static), prompts.rs (5 prompts), sensor TOML column types (H5c device_id String), H23 reads canonical runbook via worktree-aware _find_factory_file resolver.

**AD-017 — PASS.** Demo fixture keys only, passed to subprocess env; no print(ENV)/key-logging path; server stderr → PID-suffixed log file.

**POL-21 — PASS.** All CLAUDE.md addition anchors resolve (ADR-052 §D4, BC-2.16.002 §Postconditions, BC-5.39.001, BC-2.11.004 arm(4), BC-2.11.001). v1.16 pin = point-in-time origin reference (BC now v1.22); acceptable origin-pin convention, not TD-VSDD-091 volatile pin.

**Governance coherence (POL-4) — PASS.** Holdout gate slots between LOCAL 3-CLEAN and demo-recording; actor assignments match routing table; streak-reset language consistent with frozen-HEAD rule; upstream-conflict clause correctly extended for SAP-3/SID-2.

**Index policies (POL-25/POL-29) — N/A.** No BC/story/VP/index mutations in branch.

## Fresh-lens results (pass-2 targeted)
- **Subprocess lifecycle — clean.** Popen in guarded try; outer finally (5472–5488) always runs buffer cleanup → stdin.close() → terminate() → wait(5) → kill() → log-fh close. No orphan path. Boot loop polls proc.poll() within 15s budget, 2.0s stability window; send_msg converts BrokenPipe/OSError into structured _PrismCrashError.
- **Determinism — clean.** H21 compares consecutive in-session results via json.dumps(sort_keys=True) — no timing dependency; guarded against vacuous-empty. Only wall-clock gates are A17–A21 hang detectors (elapsed > 3.0s) — generous threshold, regression detector not flaky gate.
- **Environment/portability — clean.** Env-override → XDG → POSIX resolution chains; worktree-aware _find_factory_file; fcntl/select POSIX-only matching demo target.
- **Section D/E/F/G spot-checks (D5, E6, G2, G3, H19b, H20) — clean.** Each check tests its matrix claim; filter-echo membership assertions; OCSF Title-case verification; null-leak guards; data-guaranteed floors. No vacuous-PASS-on-empty.

## Novelty assessment
LOW — no gaps. Assertion discipline uniform and saturated. Fresh-context re-derivation independently re-verified highest-risk surfaces; nothing substantive found. Converged.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 0 |
| PROCESS-GAP | 0 |

**Overall Assessment:** pass
**CLEAN (strict):** yes
**CLEAN (PR-merge):** yes
**Convergence:** NOT_CONVERGED (streak 2/3; 1 more CLEAN(strict) pass required on frozen 98bb1de2)
**Readiness:** continue LOCAL cascade on frozen 98bb1de2; NO pushes mid-streak (DRIFT-ORCH-PRLEVEL-PUSH-001)

Streak advances 1/3 → 2/3 (frozen HEAD 98bb1de2 unchanged).
