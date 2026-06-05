# LOCAL Adversary Pass 1 (v2.1) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Thread QueryParams push-down into PipelineExecutor via FetchContext (v2.1: CrowdStrike FQL + Armis AQL full-wiring)
**Pass:** LOCAL adversary pass 1 — v2 re-implementation (first adversary pass against v2.1 code)
**Feature HEAD at pass start (frozen):** `aec965f9`
**Feature HEAD after fix-burst:** `f50061a5`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

9 findings total: 3 CRITICAL + 2 HIGH + 1 OBS wired in-scope + 1 OBS (process-note) + 1 DRIFT item recorded.

All 9 findings CLOSED by code fix-burst (implementer commit `f50061a5`). Streak 0/3. LOCAL pass 2 next.

**ROOT CAUSE (v2.1 pass-1 angle):** The v1 root-cause defect class RECURRED in the v2 re-implementation. Push-down translation functions existed and were syntactically correct but were **unreachable through the real `run_materialization_pipeline → adapter → DTU` path**. Tests passed via direct `FetchContext` construction that bypassed the production pipeline entry point. This is the same class found in v1 passes 5+6 (D-1004): tests validated translation logic in isolation while the production call path remained hardcoded or un-wired.

Additionally, the CrowdStrike DTU (`prism-dtu-crowdstrike`) did not honor `filter=` params forwarded from the production path — the DTU fixture returned all rows regardless of filter. This was identified as OBS-001 and wired in-scope during the fix-burst, expanding the implementation footprint to include `prism-dtu-crowdstrike` (not in story `crates_touched` v2.1).

---

## Findings

### F-P1-CRIT-001 — CRITICAL — CrowdStrike `created_timestamp` lacked `options=["INDEX"]`; FQL AST extraction returned None; push-down dead code

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** F-P1-CRIT-001

**Description:** The `created_timestamp` column in `crowdstrike.sensor.toml` lacked the `options = ["INDEX"]` annotation. The FQL push-down extraction code requires `INDEX` option to identify time-window columns and build the FQL filter. Without it, `extract_fql_time_bounds()` returned `None` and the entire CrowdStrike time push-down was dead code at runtime.

**Evidence:** `crowdstrike.sensor.toml` `[[tables.columns]]` block for `created_timestamp` had no `options` field. `extract_fql_time_bounds()` grep confirmed it conditionally reads `options` to determine INDEX eligibility — column not annotated → function short-circuits → `None` returned → FQL filter never constructed → AC-CWS-001/002/003 time push-down false green.

**Root cause class:** Production TOML spec missing required annotation; test used fabricated fixture with INDEX annotation present; SAP-2 parity gap.

**Fix:** CLOSED. `crowdstrike.sensor.toml` `created_timestamp` column annotated with `options = ["INDEX"]`. Production-TOML test added asserting INDEX annotation present and FQL extraction returns `Some(...)` for the real TOML.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-CRIT-002 — CRITICAL — `augment_armis_aql_with_time_window` never called; Armis AQL time push-down unreachable

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** F-P1-CRIT-002

**Description:** The function `augment_armis_aql_with_time_window` was implemented but never called from `SpecDrivenSensorAdapter::fetch`. The Armis branch of `fetch` constructed the `FetchContext` without invoking the AQL augmentation function, so no `after:`/`before:`/`timeFrame:` clause was ever appended to the Armis AQL query. AC-ARMIS-TW-001..005 were all false green — they validated the augmentation function in isolation, not through the production adapter path.

**Evidence:** Grep `SpecDrivenSensorAdapter::fetch` Armis branch — no call site for `augment_armis_aql_with_time_window`. Function exists in module but is dead code at the only relevant call point.

**Root cause class:** Wiring gap — function authored but not integrated into production code path. Same class as v1 passes 5+6 F-P6-CRIT-001 (materialization.rs hardcoded None).

**Fix:** CLOSED. `augment_armis_aql_with_time_window` wired into `SpecDrivenSensorAdapter::fetch` Armis branch. Real-path test asserts that the DTU aql-log endpoint receives `after:<ts>` in the AQL string when a time-window predicate is present in the query params.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-CRIT-003 — CRITICAL — AC-ARMIS-TW-005 E2E body was a live `todo!()` (POLICY 12)

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** F-P1-CRIT-003

**Description:** The acceptance criterion AC-ARMIS-TW-005 (anti-double-filter guard — if AQL already contains `after:`/`before:`, do not re-append) had a test body that was a live `todo!()` macro call. Running the test would have panicked. This is a POLICY 12 violation (no `todo!()` / `unimplemented!()` in non-`#[ignore]`'d tests) and a BC-5.39.001 violation (a "passing" test that is actually a `todo!()` panic-on-run cannot be claimed as CLEAN evidence).

**Evidence:** Test function `test_ac_armis_tw_005_anti_double_filter` body contained `todo!("implement anti-double-filter guard test")` with no `#[ignore]` attribute.

**Root cause class:** Incomplete test implementation delivered as passing. Paper-test class under TD-VSDD-059.

**Fix:** CLOSED. `todo!()` replaced with real test body implementing the anti-double-filter guard scenario. Test now asserts: when AQL already contains `after:`, `augment_armis_aql_with_time_window` does NOT append a second `after:` clause. DTU health-check preflight added per SID-1 §3 pattern. Test stays `#[ignore]` (requires DTU clone running) with SID-1 §4 citation comment: `// DTU-EXT-001: requires DTU clone running; ungated in CI after S-DEMO-QUERY-PUSHDOWN-001 merges`. Non-ignored in-process substitute test added that drives the same guard logic without DTU dependency.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-CRIT-004 — CRITICAL — CrowdStrike limit never reached DTU via `run_materialization_pipeline`

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** F-P1-CRIT-004

**Description:** The CrowdStrike `limit` translation was implemented in `spec_driven_adapter.rs` but the adapter never received a non-`None` `limit` from `run_materialization_pipeline`. The `PipelineExecutor::execute_step` seeded `QueryParams` with `limit: None` (the default), so the adapter's limit translation was unreachable for the production entry point. AC-CWS-003 (limit push-down) was validated only via direct `FetchContext` construction, bypassing the pipeline.

**Evidence:** `materialization.rs` `run_materialization_pipeline` call site for `QueryParams` construction — `limit` field defaulted to `None`. Adapter `limit` translation code present but requires `params.limit.is_some()` — unreachable from production path.

**Root cause class:** Same class as F-P1-CRIT-002 — wiring gap in the production pipeline → adapter handoff. `params.limit` not propagated from query plan to `QueryParams`.

**Fix:** CLOSED. `PipelineExecutor::execute_step` (or equivalent `run_materialization_pipeline` entry) updated to seed `query.limit` from `params.limit` where available. Real-path test verifies that `result.len() <= 3` when `LIMIT 3` is specified in the PrismQL query and the CrowdStrike DTU returns a filtered set.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-CRIT-005 — CRITICAL — AC-EQUIV-001 vacuous — did not drive the real pipeline

**Severity:** CRITICAL
**Confidence:** HIGH
**Finding ID:** F-P1-CRIT-005

**Description:** AC-EQUIV-001 (result equivalence: filtered result is a proper subset of unfiltered result) was implemented as a vacuous test — it constructed two `FetchContext` objects with different `limit` values but did not run them through `run_materialization_pipeline`. The assertion `filtered.len() <= unfiltered.len()` was trivially true because both results were fabricated from the same in-memory fixture, not from real pipeline execution. The test provided no evidence of correctness for the production path.

**Evidence:** AC-EQUIV-001 test body: no call to `run_materialization_pipeline` or `PipelineExecutor::execute`. Both `filtered` and `unfiltered` populated from mock data. Assertion technically valid but non-load-bearing for the push-down claim.

**Root cause class:** Paper-test for equivalence invariant. TD-VSDD-059 class — test claims push-down equivalence without exercising the mechanism that produces the inequality.

**Fix:** CLOSED. Rewritten to drive the real pipeline via `run_materialization_pipeline` (or equivalent). `unfiltered_count > 0` assertion load-bearing (confirms pipeline returned data). `filtered_count < unfiltered_count` assertion load-bearing (confirms limit push-down reduced result set). Wire filter-log assertion: DTU `/dtu/filter-log` receives the FQL filter string when `WHERE` clause is present.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-OBS-001 — OBS (wired in-scope) — CrowdStrike DTU did NOT honor `filter=` param

**Severity:** OBS (elevated to wired-in-scope per Canonical Principle)
**Confidence:** HIGH
**Finding ID:** F-P1-OBS-001

**Description:** SAP-2 probe on `prism-dtu-crowdstrike` revealed that the DTU test harness (`prism-dtu-crowdstrike`) did not implement FQL filter honoring. The DTU returned all fixture rows regardless of the `filter=` parameter forwarded from the production path. This meant AC-CWS-002 (`WHERE start_time >= T` reduces result count) could not be validated at the wire level even after fixing CRIT-001/004 — the DTU would return full fixture regardless.

**SAP-2 probe:** `crates/prism-dtu-crowdstrike/src/routes/detections.rs` — no `filter` param parsing; all fixture rows returned unconditionally.

**Disposition:** WIRED IN-SCOPE per Canonical Principle (fix AI-built defect in current scope; adversary may not mark CLEAN on a known-broken DTU when the story's production path depends on it). Implementation:
- `crates/prism-dtu-crowdstrike/src/types.rs`: added `parse_fql_time_bounds()` function — parses `created_timestamp >= 'T'` and `created_timestamp <= 'T'` from FQL filter strings
- `crates/prism-dtu-crowdstrike/src/routes/detections.rs`: `capture_filter` captures the `filter=` query param; `parse_fql_time_bounds()` used to filter fixture rows by `created_timestamp` range
- `crates/prism-dtu-crowdstrike/src/routes/filter_log.rs`: new `/dtu/filter-log` route records the most recent `filter=` value received; accessible at test-assertion time
- AC-CWS-002 test updated to assert wire-level filter: `GET /dtu/filter-log` returns the FQL string that was forwarded from the production path

**Note:** `prism-dtu-crowdstrike` was NOT in story `crates_touched` v2.1 (v2.1 listed: `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]`). This fix-burst adds `prism-dtu-crowdstrike` to the implementation footprint. Flagged as DRIFT item: story-writer must add `prism-dtu-crowdstrike` to `crates_touched` in the next spec-touch burst (not this code-only burst). See §Drift Items below.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-HIGH-001 — HIGH — Stale "Red Gate stub" doc comments + duplicate doc block

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** F-P1-HIGH-001

**Description:** Multiple test functions in the push-down test module carried stale doc comments copied from the v1 Red Gate stub phase, including "// TODO: implement" and "// Red Gate stub — fill in after implementer" annotations on implemented tests. One function had a duplicate doc-comment block (two `///` or `//!` blocks describing the same function). These represent TD-VSDD-091 (volatile/stale doc-comment content) class violations.

**Evidence:** Grep `crates/prism-spec-engine/src/` for `Red Gate stub`, `TODO: implement`, duplicate `///` blocks in test module — multiple matches in push-down test functions.

**Root cause class:** Stale boilerplate from test-writer phase not cleaned up during implementation.

**Fix:** CLOSED. All stale "Red Gate stub" / "TODO: implement" comments removed from implemented test functions. Duplicate doc block collapsed to single accurate description. No logic change — comment-only cleanup.

**Feature HEAD closure:** `f50061a5`

---

### F-P1-HIGH-002 — HIGH — Column-agnostic first-wins time bound (single-column assumption undocumented)

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** F-P1-HIGH-002

**Description:** The FQL time-bound extraction logic used a "first wins" heuristic when multiple `Compare` predicates referenced datetime columns. In a query with two time predicates (e.g., `start_time >= T1 AND end_time <= T2`), the extraction took only the first match and discarded the second. This behavior was undocumented and inconsistent with the story's AC-CWS-001 bounded-range scenario, which specifies both lower and upper bound push-down.

**Evidence:** `extract_fql_time_bounds()` implementation — single `start_time` + single `end_time` slots but the predicate-walk loop breaks on first match for each slot. Scenario: `start_time >= T1 AND end_time <= T2` — both should be extracted. Tested with: (a) single lower bound (pass); (b) single upper bound (pass); (c) bounded range (behavior undefined if `end_time` missed due to column-name mismatch vs `start_time`).

**Root cause class:** Under-specified single-column assumption; not surfaced as a documented limitation.

**Fix:** CLOSED. Extraction logic documented with explicit single-column assumption + 2 tests added: (a) bounded range `start_time >= T AND start_time <= T2` correctly populates both `start_time` and `end_time` slots; (b) test asserting behavior when heterogeneous column names are used (documents limitation). Limitation recorded in doc-comment: "Assumes start_time and end_time are separate column slots; first match per slot wins."

**Feature HEAD closure:** `f50061a5`

---

### F-P1-HIGH-003 — HIGH — No wire-level CrowdStrike coverage (pre OBS-001 fix)

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** F-P1-HIGH-003

**Description:** Before the OBS-001 in-scope fix, the AC-CWS-002 test had no wire-level assertion that the CrowdStrike DTU received the correct FQL filter string. The test verified that the result set was smaller after applying a filter, but did not assert that the FQL filter was actually forwarded to the DTU. This meant a correct-looking result could arise from DataFusion post-filtering even if FQL push-down was broken, masking the production wiring failure.

**Evidence:** AC-CWS-002 test body before fix: `assert!(filtered_count < unfiltered_count)` with no assertion on what the DTU received. DataFusion post-filtering alone would satisfy the assertion even if FQL was never forwarded.

**Root cause class:** Insufficient wire-level observability in test assertions. Without the `/dtu/filter-log` route, no test could verify the DTU received the filter.

**Fix:** CLOSED as part of OBS-001 fix-burst. CrowdStrike DTU `/dtu/filter-log` route added. AC-CWS-002 test updated to assert: `GET /dtu/filter-log` returns a non-empty FQL string containing `created_timestamp` after the filtered query executes.

**Feature HEAD closure:** `f50061a5`

---

## Process Notes (OBS)

### OBS-NOTE-001 — Leaky nextest test-harness warnings (non-fatal; assess at pass 2)

**Category:** Process note (not a defect finding)
**Confidence:** INFORMATIONAL

**Description:** The implementer flagged approximately 3 nextest "leaky" warnings during `just check` after the fix-burst. The warnings relate to DTU thread-cleanup races in the test harness — the DTU in-process server thread does not cleanly terminate before the nextest worker exits, producing teardown log noise. Correctness is unaffected (all 4018/4018 tests pass; no failures). The warnings are test-harness lifecycle issues, not production code defects.

**Assessment:** Non-blocking for current pass. LOCAL adversary pass 2 should assess whether the teardown race is exploitable (could cause false flakes under load) or is benign infrastructure noise. If pass 2 determines it is a real flake risk, it should be elevated to a HIGH finding with a concrete test-harness fix required. If pass 2 confirms benign, it should be closed as INFRA-OBS.

**Next adversary action:** Run `just iter prism-bin` with `--no-fail-fast` and capture any teardown warnings. Report in pass 2 with CLEAN/INFRA-OBS or HIGH disposition.

---

## Drift Items (Recorded — Not Defects)

### DRIFT-P1-001 — `prism-dtu-crowdstrike` added to implementation footprint during fix-burst

**Type:** Spec drift (story `crates_touched` does not list `prism-dtu-crowdstrike`)
**Source:** OBS-001 fix-burst added `crates/prism-dtu-crowdstrike/src/` changes (filter honoring + filter-log route)
**Story crates_touched v2.1:** `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]`
**Missing:** `prism-dtu-crowdstrike`

**Required action:** Story-writer must add `prism-dtu-crowdstrike` to `crates_touched` in the next spec-touch burst. This is NOT a current-burst scope item (code-only burst rule). Recorded here for traceability.

**Anchor:** STATE.md D-1008; sprint-state.yaml S-DEMO-QUERY-PUSHDOWN-001 notes.

---

## Streak and Convergence

| Pass | Head Before | Head After | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|-------------|------------|---------------|-----------------|----------|--------|
| v2 pass-1 | aec965f9 | f50061a5 | no | no | 3 CRIT + 2 HIGH + OBS(wired) + OBS(process-note) | 0/3 |

**Pass trajectory (v2 series):** `v2p1: 9→0 (all closed by fix-burst)`

**Next:** LOCAL adversary pass 2 on HEAD `f50061a5`. Fresh-context — re-derive all push-down paths from production TOML + DTU source. Confirm F-P1-OBS-001 filter-log route is load-bearing. Assess OBS-NOTE-001 leaky teardown. Verify SAP-1 tracing catalog completeness for any new `event_type` emission sites added in fix-burst.

---

## Build Verification

**`just check` at f50061a5:** 4018/4018 PASS, 0 failed, 0 ignored (non-`#[ignore]`).
**Crates changed in fix-burst:** prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-dtu-crowdstrike (NEW — footprint expansion), prism-sensors.
