# LOCAL Adversary Pass 2 (v2.1) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 2 — v2.1 re-implementation (second adversary pass against v2.1/v2.2 code)
**Feature HEAD at pass start (frozen):** `f50061a5`
**Feature HEAD after fix-burst:** `4e6dde5c`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

4 findings total: 1 CRITICAL + 1 HIGH + 1 MEDIUM + 1 OBS.

3 findings CLOSED by combined code fix-burst (implementer commit `4e6dde5c`) + spec fix-burst (story-writer story v2.1→v2.2, STORY-INDEX v2.283→v2.284). 1 OBS resolved as benign INFRA-OBS (no code change). Streak remains 0/3. LOCAL pass 3 NEXT.

**ROOT CAUSE (v2.1 pass-2 angle):** The dead-code defect class RECURRED AT THE TEST LAYER. Pass-1 closures verified that the production wiring chain (`run_materialization_pipeline → SpecDrivenSensorAdapter::fetch → FQL/AQL build → DTU`) was functionally correct. However, the tests for that chain had been hand-feeding `_fql`/`_aql`/`query.limit` values directly into `PipelineExecutor::execute` rather than calling `run_materialization_pipeline` — the production entry point that actually constructs `QueryParams` from the PrismQL AST. The tests were green but vacuous for the pipeline entry-point path. The pass-1 fix-burst established correct wiring; pass-2 found that the tests still bypassed the production path by seeding pre-built filter values rather than letting the pipeline extract them from the query AST.

Additionally: DRIFT-P1-001 (HIGH) was open from pass 1 — `prism-dtu-crowdstrike` present in the implementation footprint but absent from story `crates_touched`. This is a POLICY 13 spec/impl consistency violation. Story-writer closed it via story v2.1→v2.2 (added `prism-dtu-crowdstrike` to `crates_touched`, added AC-CWS-DTU-001, bumped acceptance_criteria_count 15→16, updated inputs[], File Structure rows, Architecture Compliance row, Library Requirements row). STORY-INDEX v2.283→v2.284.

OBS-NOTE-001 (leaky nextest teardown warnings from pass 1) assessed BENIGN at this pass: per-test ephemeral DTU clones, no cross-test state leakage, no false flake risk under standard `nextest` parallelism. Resolved as INFRA-OBS — no action.

---

## Pass-1 Closures Verified

All pass-1 closures independently re-verified as load-bearing at HEAD `f50061a5`:

| Finding ID | Closure | Verification |
|---|---|---|
| F-P1-CRIT-001 | `created_timestamp` INDEX annotation added | `crowdstrike.sensor.toml` confirms `options = ["INDEX"]`; production-TOML test confirms `extract_fql_time_bounds` returns `Some(...)` |
| F-P1-CRIT-002 | `augment_armis_aql_with_time_window` wired into `SpecDrivenSensorAdapter::fetch` Armis branch | Grep of `spec_driven_adapter.rs` confirms call site present; DTU aql-log test passes asserting `after:<ts>` received |
| F-P1-CRIT-003 | AC-ARMIS-TW-005 `todo!()` replaced with real test body + non-ignored in-process substitute | `test_ac_armis_tw_005_anti_double_filter` has real assertions; non-ignored substitute present in module |
| F-P1-CRIT-004 | `PipelineExecutor::execute_step` seeds `query.limit` from `params.limit` | Test `result.len() <= 3` asserts via real pipeline; LIMIT 3 query reduces result set |
| F-P1-CRIT-005 | AC-EQUIV-001 rewritten to call `run_materialization_pipeline`; `filtered_count < unfiltered_count` load-bearing | Test body confirmed non-vacuous; filter-log wire assertion present |
| F-P1-HIGH-001 | Stale "Red Gate stub" doc comments removed | `rg "Red Gate stub\|TODO: implement"` returns zero hits in push-down test module |
| F-P1-HIGH-002 | Single-column assumption documented; 2 bounded-range tests added | Doc-comment present; `test_bounded_range_*` tests pass |
| F-P1-HIGH-003 | Wire-level CrowdStrike coverage via `/dtu/filter-log` route | Closed by F-P1-OBS-001 fix; `filter-log` route confirmed in `prism-dtu-crowdstrike/src/routes/mod.rs` |
| F-P1-OBS-001 | CrowdStrike DTU `filter=` honoring wired; `/dtu/filter-log` route | `parse_created_timestamp_bounds` present in `state.rs`; `capture_filter` + `filter_log` route confirmed; fixture filtering confirmed active |

**All 9 pass-1 closures LOAD-BEARING. No regression from fix-burst.**

---

## Findings

### ADV-P02-CRIT-001 — CRITICAL — No test drove the real `run_materialization_pipeline → SpecDrivenSensorAdapter::fetch → FQL/AQL-build → DTU` chain with a time predicate

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** ADV-P02-CRIT-001

**Description:** The tests for CrowdStrike FQL time push-down and Armis AQL augmentation drove the production function chain by hand-feeding pre-built filter values into `PipelineExecutor::execute` directly — bypassing `run_materialization_pipeline`, which is the sole production callsite that constructs `QueryParams` from the PrismQL AST via ADR-033 Option T1. Tests seeded `_fql`/`_aql`/`query.limit` directly, so the `extract_fql_time_bounds` → `extract_armis_aql` → `QueryParams` construction path was never exercised. If `run_materialization_pipeline` has a bug in AST-to-QueryParams conversion, all affected tests would still pass because they never call that function.

**Evidence:** Test file grep — no call to `run_materialization_pipeline` in any time-window or AQL test. Tests call `PipelineExecutor::execute` with pre-constructed `ExecutionContext` containing pre-seeded filter strings. Production pipeline entry point: `crates/prism-query/src/materialization.rs::run_materialization_pipeline` — not called by any push-down test.

**Root cause class:** Same class as F-P1-CRIT-001..005: tests validated translation logic in isolation while the production pipeline entry point was bypassed. The wiring was already correct (tests passed first-run when real paths were called) — this confirms real end-to-end functionality is working; the tests just did not prove it.

**Fix:** CLOSED. Implementer added 3 new e2e tests via `run_materialization_pipeline` (committed in `4e6dde5c`):
1. `test_adv_p02_e2e_crowdstrike_fql_time_window_via_run_materialization_pipeline` — CrowdStrike FQL: calls `run_materialization_pipeline` with a PrismQL query containing `WHERE created_timestamp >= T`; asserts DTU `/dtu/filter-log` receives a production-built FQL string containing `created_timestamp`.
2. `test_adv_p02_e2e_crowdstrike_limit_via_run_materialization_pipeline` — CrowdStrike LIMIT: calls `run_materialization_pipeline` with `SELECT ... LIMIT 3`; asserts `result.len() <= 3` and `filtered_count < unfiltered_count`.
3. `test_adv_p02_e2e_armis_aql_augmentation_via_run_materialization_pipeline` — Armis AQL: calls `run_materialization_pipeline` with a PrismQL query; asserts DTU aql-log endpoint receives an AQL string with `after:<ts>` clause appended by `augment_armis_aql_with_time_window`.

All three tests passed first-run — confirming the existing wiring is correct. Wire-level assertions confirm DTU received production-built (not hand-fed) filter values. `filtered_count < unfiltered_count` is load-bearing.

**Feature HEAD closure:** `4e6dde5c`

---

### ADV-P02-MED-001 — MEDIUM — AC-ARMIS-TW-005 SID-1 substitutes did not exercise the `fetch()` augmentation decision branch

**Severity:** MEDIUM
**Confidence:** HIGH
**Finding ID:** ADV-P02-MED-001

**Description:** The non-ignored SID-1 substitute tests for AC-ARMIS-TW-005 (anti-double-filter guard) verified that `augment_armis_aql_with_time_window` does not re-append `after:` if already present — but they exercised the augmentation function in isolation. They did not drive the full `SpecDrivenSensorAdapter::fetch()` path to confirm that the anti-double-filter guard is actually invoked when the adapter's `fetch()` receives a `QueryParams` for an Armis sensor with an existing `after:` in the seeded AQL. The decision branch in `fetch()` that calls `augment_armis_aql_with_time_window` was not exercised by the SID-1 substitute.

**Evidence:** SID-1 substitute test body: calls `augment_armis_aql_with_time_window(existing_aql, params)` directly; no call to `SpecDrivenSensorAdapter::fetch()`. The production flow through `fetch()` was not exercised, so a regression where `fetch()` stopped calling `augment_armis_aql_with_time_window` at all would not be caught by this substitute.

**Root cause class:** SID-1 substitute scope too narrow — covered the augmentation function but not the integration point in `fetch()`.

**Fix:** CLOSED. Implementer added `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` in `4e6dde5c` — drives `SpecDrivenSensorAdapter::fetch()` directly with a `QueryParams` containing `start_time: Some(T)` for an Armis sensor; asserts that the DTU aql-log endpoint receives an AQL string with `after:T` appended. This exercises the `fetch()` decision branch (not just the augmentation function in isolation).

**Feature HEAD closure:** `4e6dde5c`

---

### ADV-P02-HIGH-001 (= DRIFT-P1-001) — HIGH — `prism-dtu-crowdstrike` modified but absent from `crates_touched` (POLICY 13)

**Severity:** HIGH (reclassified from DRIFT-P1-001 recorded as process note in pass-1 report)
**Confidence:** HIGH
**Finding ID:** ADV-P02-HIGH-001

**Description:** The pass-1 fix-burst (`f50061a5`) added FQL time-window honoring to `prism-dtu-crowdstrike` (`state.rs`, `routes/detections.rs`, `routes/mod.rs`). However, story `crates_touched` in v2.1 listed only `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]` — `prism-dtu-crowdstrike` was absent. This is a POLICY 13 spec/impl consistency violation: the spec must reflect all crates materially modified by the implementation. Additionally, the story lacked an acceptance criterion for the CrowdStrike DTU filter-honoring behavior (F-P1-OBS-001 wired in-scope but not AC-tracked).

**Evidence:** `git diff` between `aec965f9` (pass-1 start) and `f50061a5` (pass-1 end) shows `crates/prism-dtu-crowdstrike/src/state.rs`, `crates/prism-dtu-crowdstrike/src/routes/detections.rs`, `crates/prism-dtu-crowdstrike/src/routes/mod.rs` all modified. Story v2.1 `crates_touched` does not include `prism-dtu-crowdstrike`.

**Fix:** CLOSED by story-writer via story v2.1→v2.2. Changes:
- `crates_touched` updated to `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-dtu-crowdstrike, prism-sensors]`
- Added AC-CWS-DTU-001: "CrowdStrike DTU honors FQL time-window filter= param — `filtered_count < unfiltered_count` LOAD-BEARING; Red Gate test: `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window`"
- `acceptance_criteria_count` 15→16; `red_gate_tests` 15→16
- `inputs[]` expanded: added `crates/prism-dtu-crowdstrike/src/state.rs`, `crates/prism-dtu-crowdstrike/src/routes/detections.rs`, `crates/prism-dtu-crowdstrike/src/routes/mod.rs`
- Architecture Compliance Rules: added CrowdStrike DTU FQL-honoring rule row
- Library & Framework Requirements: added `prism-dtu-crowdstrike` row
- File Structure Requirements: added 3 MODIFY rows for prism-dtu-crowdstrike files
- frontmatter `timestamp` updated; `crates_touched` comment updated (v2.2 change record)
- STORY-INDEX v2.283→v2.284 (Full Story List row updated to `in_progress v2.2`; crates_touched column updated)

DRIFT-P1-001 RESOLVED. POLICY 13 compliance restored.

**Spec fix commits:** story-writer (story v2.1→v2.2 + STORY-INDEX v2.283→v2.284)

---

## OBS-NOTE-001 Assessment — Leaky nextest teardown warnings

**Category:** Process note (carry-forward from pass-1 report)
**Confidence:** HIGH (RESOLVED BENIGN)

**Assessment:** Adversary ran `just iter prism-bin --no-fail-fast` and captured teardown warnings. Findings:
- Approximately 3 warnings of the form: `[DTU-THREAD-WARN] prism-dtu-crowdstrike thread did not cleanly join before nextest worker exit`
- Each warning corresponds to an independent per-test DTU clone instance spawned by the test harness
- No test state is shared between tests — each DTU clone is ephemeral (fresh `Arc<Mutex<AppState>>` per test)
- The warnings are exclusively teardown-phase lifecycle noise; all tests pass (4022/4022)
- Under `--no-fail-fast` with parallelism, no test-ordering dependency was observed
- The race is DTU server thread not yet flushed before the nextest worker process exits — benign infrastructure noise, not a production code defect

**Disposition:** RESOLVED as INFRA-OBS. No action required. The class is non-exploitable: per-test isolation ensures no cross-test state corruption. Future harness improvement (graceful DTU thread shutdown) would be a nice-to-have P4 TD, not a correctness issue.

**OBS-NOTE-001 status:** CLOSED — resolved-benign INFRA-OBS.

---

## Streak and Convergence

| Pass | Head Before | Head After | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|-------------|------------|---------------|-----------------|----------|--------|
| v2 pass-1 | aec965f9 | f50061a5 | no | no | 3 CRIT + 2 HIGH + OBS(wired) + OBS(process-note) | 0/3 |
| v2 pass-2 | f50061a5 | 4e6dde5c | no | no | 1 CRIT + 1 HIGH(=DRIFT) + 1 MED + 1 OBS(resolved-benign) | 0/3 |

**Pass trajectory (v2 series):** `v2p1: 9→0(closed); v2p2: 4→0(closed/resolved)`

**DRIFT-P1-001:** RESOLVED — story v2.2 + STORY-INDEX v2.284 (crates_touched + AC-CWS-DTU-001 + inputs + file-structure rows).
**OBS-NOTE-001:** RESOLVED — benign INFRA-OBS (per-test ephemeral DTU clones; no cross-test state; teardown race is non-exploitable).

**Next:** LOCAL adversary pass 3 on HEAD `4e6dde5c`. Fresh-context — re-derive all push-down paths from production TOML + DTU source. Verify 3 new e2e tests via `run_materialization_pipeline` are genuinely load-bearing (not vacuous). Confirm `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` drives `fetch()` decision branch. SAP-1 tracing catalog completeness for any new `event_type` emission sites in `4e6dde5c`. SAP-2 DTU↔TOML parity re-verify on full crates_touched set (now includes `prism-dtu-crowdstrike`).

---

## Build Verification

**`just check` at `4e6dde5c`:** 4022/4022 PASS, 0 failed, 0 ignored (non-`#[ignore]`).
**Crates changed in fix-burst:** prism-spec-engine, prism-bin (new e2e tests via `run_materialization_pipeline`), prism-dtu-armis (SID-1 substitute for `fetch()` augmentation branch).
**just check delta from pass-1:** 4018→4022 tests (4 new tests added by `4e6dde5c`).
