---
document_type: adversarial-review-pass
pass: 19
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 0863184a
timestamp: 2026-06-13T08:00:00Z
streak_before: 0/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: true
findings_count: 1
finding_ids: [BPRL-P19-01]
closure_burst: D-1125
novelty: LOW
---

# PR-LEVEL Pass 19 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Pass:** 19 | **PR:** #185 | **HEAD:** 5d5484d0 (before closure) → 0863184a (after closure)
**Streak before:** 0/3 | **Streak after:** 0/3
**CLEAN(strict):** NO | **CLEAN(PR-merge):** YES

---

## Summary

Pass 19 ran a z13 evidence-anchor re-audit on ALL 6 tapes and the full evidence-report
(`docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/`). The entire code, spec, BC, story,
and integration-test surface was found clean — all prior BPRL closures confirmed intact.
One finding surfaced:

**BPRL-P19-01 MED (partial-fix regression):** BPRL-P12-01 (D-1118) relocated VP-020-K
(`test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd`) from `prism-dtu-cyberint` to
`prism-dtu-demo-server`. The AC-019 tape command was never updated: it ran only
`-p prism-dtu-cyberint` (covering VP-020-I, VP-020-J, VP-020-L — 3 tests), and the
evidence-report claimed all 4 VP-020 tests passed. VP-020-K was never invoked by the
recorded tape, yet `evidence-report.md` stated 4/4 VP-020 coverage.

This was a coverage overstatement: the tape ran 3 of 4 VP-020 tests and never showed
VP-020-K executing.

BPRL-P19-01 was closed in-burst by demo-recorder (feature-branch commit 0863184a):
AC-019 re-recorded with BOTH commands — `-p prism-dtu-cyberint` (VP-020-I/J/L, 3 PASS)
plus `-p prism-dtu-demo-server -E test(cyberint_alert_cve_resolves_in_nvd)` (VP-020-K,
1 PASS) = 4 total. VHS re-render succeeded; `.webm`/`.gif` show all 4 green. Evidence-report
corrected to accurately reflect the two-crate split: cyberint=3 (VP-020-I/J/L),
demo-server=10 (including VP-020-K). All 6 tape z13 anchor checks clean (no fabricated
names). Feature HEAD advanced to 0863184a = remote after push. Streak remains 0/3.

---

## Finding

### BPRL-P19-01 — MED — AC-019 tape command omitted VP-020-K after test relocation; evidence-report overstated VP-020 coverage

**Severity:** MED
**Classification:** Partial-fix regression (BPRL-P12-01 closed the false-green test but left
the tape command and evidence-report coverage claim unswept)

**Location:**
- `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/AC-019-cyberint-cve-pivot.tape`
- `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/evidence-report.md` (~lines 91-94, 158, 174)

**Description:**

BPRL-P12-01 (D-1118) correctly identified that VP-020-K
(`test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd`) was a false-green in
`prism-dtu-cyberint` (the test never called `NvdState::lookup_and_count`). The fix moved
the genuine integration test to `prism-dtu-demo-server`. That relocation was correct.

However, the AC-019 VHS tape (`AC-019-cyberint-cve-pivot.tape`) was not updated: its
run-command remained `-p prism-dtu-cyberint`. After the relocation, running the tape
executes only `prism-dtu-cyberint` tests — yielding VP-020-I, VP-020-J, and VP-020-L
(3 tests) but NOT VP-020-K (which now lives in `prism-dtu-demo-server`).

The `evidence-report.md` continued to state that all 4 VP-020 tests pass and all 4 are
demonstrated in the AC-019 evidence, but VP-020-K was never reached by the recorded tape
command.

**Impact:** An auditor replaying the AC-019 tape would observe 3 VP-020 tests passing and
see no evidence of the end-to-end CVE pivot test (VP-020-K). The evidence overstates the
demonstrated coverage by 1 test. The underlying test itself is genuine and passing in
`prism-dtu-demo-server` — this is an evidence-recording gap, not a test correctness gap.

**Root cause:** The test-relocation/rename sweep (TD-VSDD-060 sibling sweep) in D-1118
updated story RGT rows and code, but did not extend to demo-evidence `.tape` run-commands
and the corresponding evidence-report corpus tables. Those surfaces were not in the D-1118
sweep scope.

**Z13 evidence-anchor re-audit result (all 6 tapes):**

All 6 tapes and `evidence-report.md` were re-audited for BC-anchor integrity following
lesson z13 (evidence-artifact BC-anchor verification). Results:

- AC-001 through AC-018: all BC identifiers in `.tape` header comments and
  `evidence-report.md` narrative resolve to real anchors in `.factory/specs/` and `crates/`.
  No fabricated names. PASS.
- AC-019 (`AC-019-cyberint-cve-pivot.tape`): BC-anchor identifiers CORRECT after D-1124
  closure (PC-8=scenario catalog / PC-9=baseline namespace; INV-CYBERINT-ALERT-CVE-CORRELATION-001;
  ScenarioEntityCatalog — all confirmed present). Coverage claim INCORRECT (see above).

---

## Convergence-Positive Checks (all PASS before BPRL-P19-01 surfaced)

### Code / Spec / BC axes

All prior convergence-positive checks from passes 13-18 carried forward. Feature HEAD
at 5d5484d0 was code-unchanged relative to 7ddc0a51 (only evidence-prose commits in
5d5484d0 and its predecessors back to 7ddc0a51). No new code in diff:

- BC-2.06.019 v1.7 all Postconditions covered by RGT rows. PASS.
- BC-2.06.020 v1.4 all Postconditions (PC-1 through PC-9) + INV-CYBERINT-ALERT-CVE-CORRELATION-001
  covered by RGT rows VP-020-I through VP-020-L + VP-020-A through VP-020-H. PASS.
- SAP-1 (tracing emission catalog): no new `event_type` values in diff. PASS.
- Forbidden-pattern sweep: no `reqwest::Client::new()` without timeout, no `unwrap()` in
  critical paths, no `println!` in production code. PASS.
- POL-12 zero stub residue: no `todo!()`, `unimplemented!()`. PASS.
- All BPRL-P1 through BPRL-P18 do-not-reflag items confirmed still closed. PASS.

### Evidence file count

19 evidence files present under `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/`.
19/19. PASS.

---

## Closure (D-1125 — demo-recorder, feature-branch commit 0863184a)

AC-019 re-recorded with both required commands:

1. `cargo nextest run -p prism-dtu-cyberint -E 'test(BC_2_06_020)'` →
   VP-020-I (`test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog`),
   VP-020-J (`test_BC_2_06_020_cyberint_scenario_alerts_use_catalog_cves`),
   VP-020-L (`test_BC_2_06_020_cyberint_baseline_cve_format_non_pivotable`): 3 PASS

2. `cargo nextest run -p prism-dtu-demo-server -E 'test(cyberint_alert_cve_resolves_in_nvd)'` →
   VP-020-K (`test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd`): 1 PASS

Total: 4/4 VP-020 tests demonstrated. VHS re-render succeeded: `.webm` and `.gif` both
show all 4 tests green across the two commands.

Evidence-report corrected:
- Lines ~91-94: cyberint command output updated to reflect 3 VP-020 tests (I/J/L)
- Line ~158: demo-server command added showing VP-020-K
- Line ~174: total VP-020 coverage corrected to "4/4 (cyberint: VP-020-I/J/L; demo-server: VP-020-K)"

Two-crate split accurately documented: cyberint=3 (VP-020-I/J/L), demo-server=10 (VP-020-K + 9 others).

Feature HEAD after commit: `0863184a` = remote (pushed).

---

## Do-Not-Reflag Carry Forward

All BPRL-P1 through BPRL-P18 do-not-reflag entries carry forward. BPRL-P19-01 added:

- **BPRL-P19-01 CLOSED (D-1125):** AC-019 re-recorded to cover VP-020-K under
  `prism-dtu-demo-server`; evidence-report accurately reflects two-crate split
  (cyberint=VP-020-I/J/L, demo-server=VP-020-K). Feature HEAD 0863184a.
  **DO NOT re-raise "AC-019 tape command only runs cyberint tests", "VP-020-K not shown in
  evidence", or "evidence-report overstates VP-020 coverage as 4/4 when tape only shows 3"
  — CLOSED.**

---

## Pass Status

```
CLEAN (strict): NO — BPRL-P19-01 MED (AC-019 tape omitted VP-020-K after test relocation)
CLEAN (PR-merge): YES — ZERO findings of CRIT + HIGH + MED in code/spec/BCs/integration tests
Streak: 0/3 → 0/3 (BPRL-P19-01 prevents advancement)
Closure: D-1125 (demo-recorder commit 0863184a; AC-019 re-recorded; VP-020-K coverage confirmed; push complete)
NEXT: PR-LEVEL pass 20 at HEAD 0863184a (diff CHANGED — re-materialize via gh pr diff 185; do NOT reuse stale /tmp diffs)
```
