# LOCAL Adversary Pass 5 (v2.x) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 5 — v2.x cascade (fifth adversary pass against v2.4 code + all pass-4 closures verified)
**Feature HEAD at pass start (frozen):** `70ae30d2`
**Feature HEAD after fix-burst:** `b87d58a4`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

1 finding: ADV-P05-HIGH-001 (HIGH). CLOSED. Fix-burst committed at `b87d58a4`. just check 4033/4033 PASS 0 failed. CLEAN(strict)=no (finding was present before fix); streak remains 0/3. LOCAL pass-6 NEXT.

---

## Pass-4 Closure Verification (4 closures — all LOAD-BEARING)

All 4 pass-4 closures independently re-verified at feature HEAD `70ae30d2`:

| Finding ID | Pass-4 Closure | Load-Bearing Verification at HEAD 70ae30d2 |
|---|---|---|
| ADV-P04-HIGH-001 | story v2.2→v2.3: `parse_created_timestamp_bounds` → `parse_fql_time_bounds` at 6 story sites + 1 STORY-INDEX note | `grep parse_created_timestamp_bounds` across `.factory/stories/` returns ZERO hits; `grep parse_fql_time_bounds crates/prism-dtu-crowdstrike/src/state.rs` returns the production function. Closure is load-bearing: all spec/index citations now agree with the actual implementation name. |
| ADV-P04-HIGH-002 | implementer `70ae30d2`: AC-CWS-002 test rewritten to `run_materialization_pipeline` + both-bounds DTU wire assertion | `grep test_ac_cws_002 crates/prism-bin/` finds the test calling `run_materialization_pipeline`. Test asserts `/dtu/filter-log` body contains `created_timestamp:>=` AND `created_timestamp:<=` in combined form AND `filtered_count < unfiltered_count`. Not a boundary-bypass test. Closure is load-bearing. |
| ADV-P04-LOW-001 | implementer `70ae30d2`: 7 unit tests for `parse_fql_time_bounds` in `prism-dtu-crowdstrike/src/state.rs` | `grep test_ac_cws_dtu_001` and `grep parse_fql_time_bounds` in state.rs test module — 7 test functions present, including the canonical `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window`. All 7 pass. Closure is load-bearing. |
| ADV-P04-LOW-002 | product-owner BC-2.16.002 v1.66→v1.67: catalog row 71 Recurrence description corrected `FetchContext`→`(start_time, end_time) tuple` | BC-2.16.002 row 71 Recurrence field: reads "before returning the `(start_time, end_time)` tuple to the caller." `FetchContext` string absent from this row. `extract_time_window_from_ast` in `crates/prism-query/src/pushdown.rs` returns `(Option<String>, Option<String>)` — matches the corrected prose. Closure is load-bearing. |

**All 4 pass-4 closures CONFIRMED LOAD-BEARING. No regression of any closed finding.**

---

## SAP-1 Probe (PG-LP11-001 Tracing Emission Catalog)

**SAP-1 standing probe applied to feature HEAD `70ae30d2` per CLAUDE.md.**

Grep: `rg 'event_type\s*=' crates/ --type rust` on feature HEAD.

**Result:** All `event_type =` emissions in `crates/` have a corresponding row in BC-2.16.002 Canonical Structured Event Catalog (v1.67). Catalog count 71. `push_down.inverted_time_range` WARN emission (row 71) confirmed present — this is the EC-003 closure from pass 3, verified load-bearing in pass 4, confirmed intact at pass 5. No unregistered emissions found.

**SAP-1 result: PASS.** No new findings from SAP-1 probe.

---

## SAP-2 Probe (DTU↔TOML Schema Parity)

**SAP-2 standing probe applied to `crates_touched` sensor specs: prism-dtu-crowdstrike, prism-dtu-armis.**

For each sensor, DTU `types.rs` and route structs compared against TOML `[[tables]]` column declarations.

- **prism-dtu-crowdstrike:** All columns declared in `crowdstrike.sensor.toml` present in DTU response structs. `parse_fql_time_bounds` function name in `state.rs` confirmed (matches story v2.4). `created_timestamp` `options=["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.
- **prism-dtu-armis:** All columns in `armis.sensor.toml` present in DTU response structs. `last_seen` and `created_at` `options=["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.

**SAP-2 result: PASS.** No P1 CRITICAL findings.

---

## Exhaustive Sibling-Sweep: 32 Push-Down Tests Across 4 Crates (TD-VSDD-060)

Per TD-VSDD-060 sibling-site sweep discipline, the AC-CWS-002 AC-EQUIV-001 fix-burst triggered an exhaustive review of ALL push-down tests across all 4 `crates_touched` that contain tests.

**Total push-down tests reviewed:** 32 across `prism-query`, `prism-spec-engine`, `prism-bin`, `prism-dtu-crowdstrike`

**Disposition of all 32 tests:**

| Category | Count | Disposition |
|---|---|---|
| Real-path tests via `run_materialization_pipeline` | 4 | CORRECT — drive production entry point; assert DTU wire-level or filtered<unfiltered load-bearing outcomes |
| Honestly-named boundary tests via `PipelineExecutor::execute` directly | 21 | CORRECT — test names accurately describe their scope (e.g., `_via_pipeline_executor_boundary`); these test the PipelineExecutor contract layer, not the full materialization path; load-bearing for their named contract surface |
| In-process DTU unit tests (`parse_fql_time_bounds`, AQL augmentation) | 7 | CORRECT — drive production parsing/augmentation functions directly; load-bearing for their named function contract |

**Finding from sweep:** The old prism-spec-engine test named `test_ac_equiv_001_result_equivalence_via_real_materialization_path` (HEAD 70ae30d2, pre-fix) claimed "via real materialization path" in its name but hand-fed `_fql` and called `PipelineExecutor::execute` directly — bypassing `run_materialization_pipeline`. The name was a false claim. The BC-2.11.007 result-equivalence subset/no-fabrication invariant had no real-path coverage (sibling-sweep miss from pass 4's AC-CWS-002 fix). This is **ADV-P05-HIGH-001** (see below).

No other test in the 32 was found to misrepresent its scope. All non-real-path tests are either honestly named or are unit tests whose names accurately bound their scope.

**TD-VSDD-060 sweep: COMPLETE.** All 32 tests dispositioned.

---

## Findings

### ADV-P05-HIGH-001 — HIGH — AC-EQUIV-001 test hand-fed FQL + called PipelineExecutor::execute directly; claimed "via real materialization path" in its name; BC-2.11.007 result-equivalence subset/no-fabrication invariant had no real-path coverage

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** ADV-P05-HIGH-001

**Description:** At feature HEAD `70ae30d2`, the test named `test_ac_equiv_001_result_equivalence_via_real_materialization_path` (in `prism-spec-engine`) hand-fed a pre-built `_fql` string and called `PipelineExecutor::execute` directly. It did not call `run_materialization_pipeline` (the production entry point per ADR-033 T1) despite its name claiming the real materialization path.

This is the same dead-code-via-test-layer defect class that was closed in pass 4 for AC-CWS-002 — it recurred at the AC-EQUIV-001 layer. Pass 4's AC-CWS-002 fix correctly addressed CrowdStrike's `run_materialization_pipeline` path for the CWS-002 acceptance criterion but did not sweep the sibling AC-EQUIV-001 test (which existed in `prism-spec-engine`, a different crate). The sibling sweep was not exhaustive enough to catch the cross-crate sibling at pass 4.

**BC impact:** BC-2.11.007 v1.8 states: "push-down is an optimization only; the query result must be identical whether or not push-down occurs." This invariant requires testing via the REAL materialization path (not a pipeline-executor shortcut). The misnamed test did not satisfy AC-EQUIV-001; the invariant had no real-path coverage.

**Root cause:** Same defect class as AC-CWS-002 pass-4 finding — hand-fed-FQL + direct-PipelineExecutor-boundary bypass — but in a sibling crate (`prism-spec-engine`) that pass 4's fix-burst did not sweep. The false-claim name ("via_real_materialization_path") masked the gap from casual code review.

**Fix:** Two changes in fix-burst `b87d58a4`:
1. **NEW real-path test** (`test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`) added in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`. Drives `run_materialization_pipeline` end-to-end with CrowdStrike DTU. Asserts subset invariant: every record returned WITH push-down filter must appear in the unfiltered result set (no fabrication), AND `filtered_count < unfiltered_count` (non-vacuous). Load-bearing for BC-2.11.007 result-equivalence.
2. **Renamed boundary test** — old `test_ac_equiv_001_result_equivalence_via_real_materialization_path` (false name) renamed to `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary`. New name is accurate: exercises the PipelineExecutor boundary; does NOT satisfy AC-EQUIV-001 alone (explicitly noted in test doc-comment and story §AC-EQUIV-001 supplementary note).

**Story spec update:** AC-EQUIV-001 Red Gate test citation updated from the old misnamed test to the new `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`. Supplementary boundary test noted. Story v2.3→v2.4. STORY-INDEX v2.285→v2.286.

**Status: CLOSED** — implementer `b87d58a4`. just check 4033/4033 PASS 0 failed. Exhaustive sibling-sweep of all 32 push-down tests across 4 crates confirmed no other misnamed real-path claims remain (TD-VSDD-060).

---

## Fix-Burst Summary

**Feature HEAD after fix-burst:** `b87d58a4`
**just check result:** 4033/4033 PASS 0 failed
**Specialist commits:**

| Specialist | Commit | Work |
|---|---|---|
| implementer | `b87d58a4` | NEW `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` in prism-bin (run_materialization_pipeline end-to-end + subset/no-fabrication invariant); misnamed prism-spec-engine test renamed `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary` (honest scope name) |
| story-writer | (in fix-burst context) | story v2.3→v2.4: AC-EQUIV-001 Red Gate test citation updated to new authoritative test name; supplementary boundary test noted. STORY-INDEX v2.285→v2.286: Full Story List row updated in_progress v2.3→in_progress v2.4 |

---

## Post-Fix Verification

- `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`: passes in `prism-bin` via `run_materialization_pipeline` → DTU → subset invariant → non-vacuous → load-bearing for BC-2.11.007
- `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary`: passes in `prism-spec-engine`; name accurately describes its scope (PipelineExecutor-boundary only)
- Story AC-EQUIV-001: Red Gate test citation updated; supplementary boundary test noted; story v2.4
- STORY-INDEX v2.286: Full Story List row in_progress v2.4; §Changelog v2.286 row prepended
- TD-VSDD-060 sibling-sweep: all 32 push-down tests across 4 crates dispositioned; no other misnamed real-path claims
- SAP-1 PASS: catalog count 71; no unregistered event_type emissions
- SAP-2 PASS: CrowdStrike + Armis DTU↔TOML parity confirmed

---

## Lesson Recorded (hand-fed-FQL-vs-real-path anti-pattern)

This finding is the SECOND recurrence of the hand-fed-FQL-via-PipelineExecutor-boundary test anti-pattern in this cascade:
- **Pass 4** caught it for AC-CWS-002 (in `prism-bin`); fix was correct but sibling sweep was incomplete.
- **Pass 5** caught it for AC-EQUIV-001 (in `prism-spec-engine`, a different crate, missed by pass 4 sweep).

The pass-5 fix added an **exhaustive sibling-sweep of all 32 push-down tests** across all 4 crates_touched (TD-VSDD-060). This is the correct response: when one hand-fed test is found, sweep ALL push-down tests, not just the immediate crate.

Lesson recorded in `cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/lessons.md`.

---

## Convergence Trajectory (v2.x)

| Pass | HEAD (start) | HEAD (end) | Findings | Streak |
|------|-------------|------------|---------|--------|
| v2-pass-1 | `aec965f9` | `f50061a5` | CRIT:3 HIGH:2 OBS:2 | 0/3 |
| v2-pass-2 | `f50061a5` | `4e6dde5c` | CRIT:1 HIGH:1 MED:1 OBS:1 | 0/3 |
| v2-pass-3 | `4e6dde5c` | `0a93ffef` | MED:1 | 0/3 (CLEAN PR-merge: yes) |
| v2-pass-4 | `0a93ffef` | `70ae30d2` | HIGH:2 LOW:2 OBS:1[process-gap] | 0/3 |
| v2-pass-5 | `70ae30d2` | `b87d58a4` | HIGH:1 | 0/3 |

**Trajectory:** 9 → 4 → 1 → 5(pre-fix) → 1. Finding count collapsing. All prior-closed findings confirmed LOAD-BEARING and remain closed. Finding class progression: CRIT-class (passes 1-2) → MED-class (pass 3) → HIGH/LOW-class (pass 4: name-drift + test-gap + prose) → HIGH-class (pass 5: sibling-sweep miss from pass 4's AC-CWS-002 fix). No REGRESSION of prior-closed findings.

**Next:** LOCAL pass 6 at HEAD `b87d58a4`. Fresh context. Verify ADV-P05-HIGH-001 closure is load-bearing (AC-EQUIV-001 real-path test drives `run_materialization_pipeline`; subset/no-fabrication invariant confirmed; exhaustive 32-test sibling-sweep confirmed complete). Full SAP-1 + SAP-2. Streak attempt 0/3 → 1/3 on CLEAN(strict).
