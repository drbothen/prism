# LOCAL Adversary Pass 6 (v2.x) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 6 — v2.x cascade (sixth adversary pass; all pass-5 closures verified; one new MED finding CLOSED in-burst)
**Feature HEAD at pass start (frozen):** `b87d58a4`
**Feature HEAD after fix-burst:** `266827e1`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

1 finding: ADV-P06-MED-001 (MED). CLOSED in-burst at `266827e1`. Production code was correct throughout; test-strength gap only. just check 4033/4033 PASS 0 failed. CLEAN(strict)=no (finding was present before fix); streak remains 0/3. LOCAL pass-7 NEXT.

---

## Pass-5 Closure Verification (1 closure — LOAD-BEARING)

Pass-5 closure independently re-verified at feature HEAD `b87d58a4`:

| Finding ID | Pass-5 Closure | Load-Bearing Verification at HEAD b87d58a4 |
|---|---|---|
| ADV-P05-HIGH-001 | implementer `b87d58a4`: NEW `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` drives `run_materialization_pipeline` end-to-end + asserts subset/no-fabrication invariant; old misnamed `prism-spec-engine` test renamed `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary` (honest scope); story v2.4 AC-EQUIV-001 citation updated | `grep test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline crates/prism-bin/` returns the test in `adv_p02_e2e_pushdown_pipeline_test.rs`; test body calls `run_materialization_pipeline(…)` not `PipelineExecutor::execute` directly; asserts `result.len() <= base_result.len()` (subset invariant) and `!result.is_empty()` (no-fabrication). `grep test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary crates/prism-spec-engine/` returns the renamed test — honest boundary scope stated in both function name and doc-comment. All 32 push-down tests from pass-5 TD-VSDD-060 sweep remain correctly classified at HEAD `b87d58a4`. Closure is load-bearing. |

**Pass-5 closure CONFIRMED LOAD-BEARING. No regression.**

---

## SAP-1 Probe (PG-LP11-001 Tracing Emission Catalog)

**SAP-1 standing probe applied to feature HEAD `b87d58a4` per CLAUDE.md.**

Grep: `rg 'event_type\s*=' crates/ --type rust` on feature HEAD.

**Result:** All `event_type =` emissions in `crates/` have a corresponding row in BC-2.16.002 Canonical Structured Event Catalog (v1.67). Catalog count 71. No new emissions added in pass-5 fix-burst `b87d58a4` (implementer only renamed an existing test and added a new test; no new `event_type` sites). `push_down.inverted_time_range` WARN emission (row 71) confirmed intact. No unregistered emissions found.

**SAP-1 result: PASS.** No new findings from SAP-1 probe.

---

## SAP-2 Probe (DTU↔TOML Schema Parity)

**SAP-2 standing probe applied to `crates_touched` sensor specs: prism-dtu-crowdstrike, prism-dtu-armis.**

For each sensor, DTU `types.rs` and route structs compared against TOML `[[tables]]` column declarations.

- **prism-dtu-crowdstrike:** All columns declared in `crowdstrike.sensor.toml` present in DTU response structs. `parse_fql_time_bounds` function name in `state.rs` confirmed (matches story v2.4). `created_timestamp` `options=["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.
- **prism-dtu-armis:** All columns in `armis.sensor.toml` present in DTU response structs. `last_seen` and `created_at` `options=["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.

**SAP-2 result: PASS.** No P1 CRITICAL findings.

---

## Exhaustive Doc-Claim-vs-Assertion Audit (all push-down AC tests)

Per the pass-5 lesson (test-strength gap class: assertion weaker than AC/doc claim), an exhaustive audit of all push-down AC test assertions was conducted against their corresponding AC/doc claims at HEAD `b87d58a4`.

**Method:** For each AC test in `crates_touched`, read (1) the test name + doc-comment claim, (2) the assertions actually present in the test body, (3) the corresponding AC text in the story spec.

| Test | Doc/Name Claim | Assertion Present | AC Match? | Finding? |
|---|---|---|---|---|
| `test_ac_cws_001_fql_pushdown_wired` (prism-bin) | CrowdStrike FQL filter reaches DTU via run_materialization_pipeline | DTU `/dtu/filter-log` contains `fql_filter` param; `filtered < unfiltered` | YES | CORRECT |
| `test_ac_cws_002_time_window_both_bounds` (prism-bin) | Both CrowdStrike time bounds in combined FQL via run_materialization_pipeline | DTU `/dtu/filter-log` contains `created_timestamp:>=` AND `created_timestamp:<=`; `filtered < unfiltered` | YES | CORRECT |
| `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` (prism-bin) | AC-EQUIV-001 result equivalence via run_materialization_pipeline | `result.len() <= base_result.len()` (subset); `!result.is_empty()` (no-fabrication); drives `run_materialization_pipeline` | YES | CORRECT |
| `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary` (prism-spec-engine) | Boundary supplement: PipelineExecutor FQL subset invariant | Calls `PipelineExecutor::execute`; asserts subset; doc-comment clearly states supplementary boundary scope | YES (supplementary, honest scope) | CORRECT |
| `test_ac_armis_aql_001_wired` (prism-bin) | Armis AQL reaches DTU via run_materialization_pipeline | DTU `/api/v1/search` receives AQL param; `filtered < unfiltered` | YES | CORRECT |
| `test_ac_armis_tw_001..005` (prism-bin) | Armis time-window AC-ARMIS-TW-001..005 | Augmented AQL contains `after:T`/`before:T` clauses; DTU filter-log confirms | YES | CORRECT |
| `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window` (prism-dtu-crowdstrike) | CrowdStrike DTU honors combined FQL time bounds | `parse_fql_time_bounds` returns both bounds; no fabrication | YES | CORRECT |
| `test_ac_cws_003_crowdstrike_where_clause_absent_from_dtu_filter_log` (prism-bin) | AC-CWS-003: no `created_timestamp` clause in DTU `/dtu/filter-log` when WHERE pushdown absent; all 50 records returned | Asserted: `/dtu/filter-log` body does NOT contain `created_timestamp`; `result.len() == 50` | **PARTIAL — see ADV-P06-MED-001 below** | **FINDING** |
| All `parse_fql_time_bounds` unit tests (7, prism-dtu-crowdstrike) | Named property boundary (start-only, end-only, both-bounds, inverted, missing, malformed) | Each asserts the named property directly via function return value | YES | CORRECT |

**Audit finding:** AC-CWS-003 assertion was non-load-bearing for the absent-clause aspect. See ADV-P06-MED-001 below.

All other AC tests in the push-down suite: CORRECT. Test assertions match doc/name claims and AC text.

---

## Findings

### ADV-P06-MED-001 (MED) — AC-CWS-003: test asserted only `!is_empty()` + count; did not query DTU `/dtu/filter-log` for ABSENCE of `created_timestamp` clause

**Severity:** MED
**AC:** AC-CWS-003 (story v2.4) — "When a query has no WHERE-clause CrowdStrike time filter, the CrowdStrike DTU does NOT inject a `created_timestamp` FQL predicate into the filter-log entry, and all 50 fixture records are returned."
**Doc claim:** Test name `test_ac_cws_003_crowdstrike_where_clause_absent_from_dtu_filter_log` claims DTU filter-log absence assertion.
**Assertion found at HEAD b87d58a4:** Test called `run_materialization_pipeline` with a no-time-filter query and asserted `!result.is_empty()` and non-zero count. It did NOT query the DTU `/dtu/filter-log` endpoint to verify the `created_timestamp` clause was ABSENT from the actual FQL sent to the DTU.
**Impact:** Test name and AC claim "filter-log absence" as the load-bearing property, but the test only verified non-empty return and count. If the production code incorrectly injected a `created_timestamp` clause on unfiltered queries, the test would still pass (data returns regardless). Production code is correct at this HEAD (no spurious injection); the test-strength gap means the correctness of the no-filter path has no wire-level absence assertion.
**Root cause:** Same test-strength gap class as AC-CWS-002 (pass 4) and AC-EQUIV-001 (pass 5) — assertion weaker than the AC/doc claim. Third recurrence of this class across passes 4/5/6.
**Fix required:** Add DTU `/dtu/filter-log` query in AC-CWS-003 test body to assert absence of `created_timestamp` clause when no time filter is present; assert `result.len() == 50` (exact fixture count, not just non-empty).

**CLOSED at `266827e1`:** implementer `266827e1` — AC-CWS-003 test in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` updated: (1) DTU `/dtu/filter-log` queried; (2) asserts `/dtu/filter-log` body does NOT contain `created_timestamp`; (3) asserts `result.len() == 50` (exact fixture count). Module docstring for `pushdown.rs` reconciled (stale VP-031/S-3.02 reference replaced with correct VP-031/this-story/ADR-033 anchor per story scope). just check 4033/4033 PASS 0 failed.

**CLEAN(strict): no** (finding was open before fix-burst; streak stays 0/3 per BC-5.39.001 D-779).

---

## Module Docstring Fix (non-finding, scope housekeeping)

During the AC-CWS-003 fix-burst, the `pushdown.rs` module-level docstring was found to cite `VP-031/S-3.02` as the origin of the module. Story S-3.02 was the original push-down implementation story; ADR-033 + S-DEMO-QUERY-PUSHDOWN-001 substantially extended this. The docstring was updated in `266827e1` to cite the correct authority chain (VP-031, this story, ADR-033). Not a new finding — routine maintenance discovered during fix scope inspection.

---

## Post-Fix Verification

**Feature HEAD after fix-burst:** `266827e1`
**just check:** 4033/4033 PASS 0 failed
**SAP-1 (post-fix):** PASS — no new `event_type` emissions in `266827e1`; catalog count 71 unchanged
**SAP-2 (post-fix):** PASS — DTU↔TOML parity unchanged by test-only fix-burst

---

## Streak Status

**Streak after pass 6: 0/3**

Pass 6 found 1 MED finding (CLOSED in-burst). CLEAN(strict)=no. Per BC-5.39.001 D-779, streak does not advance. LOCAL pass-7 NEXT (fresh context; verify ADV-P06-MED-001 closure load-bearing — DTU filter-log absence assertion present and asserts absence of `created_timestamp` clause when no time filter; `result.len() == 50`; full SAP-1 + SAP-2; streak attempt 0/3 → 1/3).

**Convergence trajectory (v2.x passes 1–6):** 9→4→1→5→1→1

---

## Lesson Recorded

Test-strength gap class (assertion weaker than AC/doc claim) recurred at AC-CWS-002 (pass 4), AC-EQUIV-001 (pass 5), and AC-CWS-003 (pass 6). Three consecutive passes caught the same defect class. Pass-6 added an exhaustive doc-claim-vs-assertion audit of all push-down AC tests — all others CORRECT/load-bearing after pass-5's exhaustive sibling-sweep. The recurring nature of this class indicates a systematic test-writing discipline gap: test-writer + implementer should assert the named property via wire-level evidence (filter-log, DTU log, specific field presence/absence), not non-empty/structural proxies. Lesson appended to `cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/lessons.md` (D-1013 anchor).
