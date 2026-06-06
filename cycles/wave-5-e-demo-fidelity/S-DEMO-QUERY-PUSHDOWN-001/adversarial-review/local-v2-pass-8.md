# LOCAL Adversary Pass 8 (v2.x) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 8 — v2.x cascade (eighth adversary pass; pass-7 was CLEAN 1/3; pass-8 found a real correctness defect — streak RESETS 1/3→0/3)
**Feature HEAD at pass start (frozen):** `266827e1`
**Feature HEAD after fix-burst:** `69aafcc7`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Operational Note — Parallel 8/9 Attempt Errored; Pass 8 Re-run Sequential

Passes 8 and 9 were initially dispatched in parallel against frozen HEAD `266827e1` (following pass-7 CLEAN 1/3 to accelerate streak advancement). Both returned transient socket errors with no adversary verdict. The parallel attempts are moot — no findings were recorded, no verdicts issued. Pass 8 was re-run sequentially and found ADV-P08-MED-001 (a real correctness defect). The streak reset to 0/3. Pass 9 will be dispatched fresh against the fixed HEAD `69aafcc7` as a new streak attempt.

---

## Pass-7 Verdict (context)

Pass 7 ran against frozen HEAD `266827e1` and returned **CLEAN(strict): yes, CLEAN(PR-merge): yes, Streak 1/3**. Pass-7 verified ADV-P06-MED-001 closure load-bearing (AC-CWS-003 DTU `/dtu/filter-log` absence assertion + `result.len() == 50` confirmed present and asserted correctly). SAP-1 and SAP-2 both passed. Streak advanced from 0/3 to 1/3.

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3** (reset from 1/3)

1 finding: ADV-P08-MED-001 (MED, correctness). CLOSED by fix-burst at `69aafcc7`. Two root causes fixed: (1) DTU inclusive-boundary semantics for `>=`/`<=` predicates (CrowdStrike + Armis); (2) RFC3339 timestamp normalization (`+00:00` vs `Z` suffix). 2 new boundary Red Gate tests added. just check 4035/4035 PASS 0 failed. Feature HEAD `266827e1`→`69aafcc7`. CLEAN(strict)=no (finding was present before fix-burst); streak resets 1/3→0/3 per BC-5.39.001 D-779.

---

## Pass-7 Closure Verification (1 closure — LOAD-BEARING, performed at pass-8 start)

Pass-7 closure independently re-verified at feature HEAD `266827e1`:

| Finding ID | Pass-7 Status | Load-Bearing Verification at HEAD 266827e1 |
|---|---|---|
| ADV-P06-MED-001 | CLEAN(strict)=yes at pass 7; all prior closures confirmed | AC-CWS-003 test (`test_ac_cws_003_crowdstrike_where_clause_absent_from_dtu_filter_log`) confirmed: DTU `/dtu/filter-log` queried in test body; asserts `!body.contains("created_timestamp")`; asserts `result.len() == 50` (exact fixture count, not just `!is_empty()`). Closure is load-bearing. |

**Pass-7 closure CONFIRMED LOAD-BEARING. No regression at HEAD `266827e1`.**

---

## SAP-1 Probe (PG-LP11-001 Tracing Emission Catalog) — HEAD `266827e1`

`rg 'event_type\s*=' crates/ --type rust` applied at frozen HEAD `266827e1`.

All `event_type =` emissions present have a corresponding row in BC-2.16.002 Canonical Structured Event Catalog (v1.67). Catalog count 71. `push_down.inverted_time_range` WARN emission (row 71) confirmed intact. No unregistered emissions found.

**SAP-1 result: PASS.** No new findings from SAP-1 probe.

---

## SAP-2 Probe (DTU↔TOML Schema Parity) — HEAD `266827e1`

SAP-2 probe applied to `crates_touched` sensor specs: `prism-dtu-crowdstrike`, `prism-dtu-armis`.

- **prism-dtu-crowdstrike:** All columns in `crowdstrike.sensor.toml` present in DTU response structs. `created_timestamp` with `options=["INDEX"]` confirmed. `parse_fql_time_bounds` function in `state.rs` present. No TOML-only columns missing DTU equivalents.
- **prism-dtu-armis:** All columns in `armis.sensor.toml` present in DTU response structs. `last_seen` and `created_at` with `options=["INDEX"]` confirmed. No TOML-only columns missing DTU equivalents.

**SAP-2 result: PASS.** No P1 CRITICAL findings.

---

## Findings

### ADV-P08-MED-001 (MED, correctness) — Inclusive time predicates (`>=`/`<=`, `CompareOp::Ge`/`Le`) under-fetch boundary records: push-down applied strict/exclusive semantics while DataFusion applied inclusive

**Severity:** MED — correctness, not security; only affects exact-boundary queries; production code wrong for this case
**BC:** BC-2.11.007 v1.8 — result-equivalence invariant: push-down result MUST be identical whether or not push-down occurs. `CompareOp::Ge` and `CompareOp::Le` are explicitly within scope (v1.8 covers the full `>=`/`<=` predicate set).
**Root Cause 1 — DTU inclusive-boundary semantics (CrowdStrike `detections.rs` + Armis `search.rs`):**
`device_in_time_window` and `alert_in_time_window` predicates in the DTU filtering functions used strict inequality at boundaries: a record with `timestamp == bound` was excluded. DataFusion, however, applies inclusive semantics for `>=`/`<=` predicates. This asymmetry caused push-down to UNDER-fetch by dropping exact-boundary records — a BC-2.11.007 result-equivalence violation. The fix: DTU filtering must exclude only strictly-outside records (`< start_time` or `> end_time`), so that at the boundary the DTU OVER-fetches and DataFusion narrows to the exact inclusive set. Push-down is defined as an optimization (BC-2.11.007 §Mechanism); it must never produce a strict subset of the non-pushed result.
**Root Cause 2 — RFC3339 timestamp normalization (`pipeline.rs` / `to_rfc3339()`):**
`chrono::DateTime::to_rfc3339()` emits timestamps in `+00:00` form (e.g., `2024-01-01T00:00:00+00:00`). DataFusion's string-based timestamp comparison uses lexicographic ordering. The `+00:00` suffix (ASCII `+` = 43) sorts BEFORE `Z` (ASCII `Z` = 90) lexicographically. When a boundary record's timestamp is stored in the DTU fixture as `2024-01-01T00:00:00Z`, and the push-down filter uses `created_timestamp:>=2024-01-01T00:00:00+00:00`, the comparison `+00:00` < `Z` (string order) means the boundary record is dropped silently even though both forms represent the same UTC instant. Fix: use `to_rfc3339_opts(SecondsFormat::Secs, true)` which emits the `Z` suffix, matching the fixture's canonical form and ensuring lexicographic string comparison at DataFusion's boundary is correct.

**Impact:** Queries with `>=` or `<=` time predicates dropped exact-boundary records when push-down was active. BC-2.11.007 Ge/Le are explicitly in scope. This was untested — no boundary Red Gate test existed for the `>=`/`<=` case (the existing tests used only `>` and `<` strict comparisons). Production code wrong; tests did not exercise this case.

**CLOSED at `69aafcc7`:** implementer `69aafcc7` — two-root-cause fix:
1. `crates/prism-dtu-crowdstrike/src/routes/detections.rs` + `crates/prism-dtu-armis/src/routes/search.rs`: `device_in_time_window`/`alert_in_time_window` changed to exclude only strictly-outside records (`ts < start` or `ts > end`); inclusive boundary (`ts == start` or `ts == end`) now passes through. DTU over-fetches at boundary; DataFusion narrows to exact inclusive set. BC-2.11.007 result-equivalence restored.
2. `pipeline.rs` (or equivalent push-down timestamp serialization site): `to_rfc3339()` → `to_rfc3339_opts(SecondsFormat::Secs, true)` to emit `Z`-suffix RFC3339 form, preventing lexicographic boundary drop at DataFusion string-comparison layer.
3. 2 new boundary Red Gate tests via `run_materialization_pipeline`: `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` + `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`. Each test queries with an exact `>=`/`<=` boundary predicate and asserts the boundary record IS included in the result (result-equivalence verification for inclusive boundary).
4. Story spec updated: `v2.4→v2.5` — EC-009 added (inclusive-boundary edge case: DTU over-fetch at `ts == bound`; DataFusion narrows; `to_rfc3339_opts` Z-normalization note); `red_gate_tests 16→18`.

just check 4035/4035 PASS 0 failed.

**CLEAN(strict): no** (finding was open before fix-burst; streak resets 1/3→0/3 per BC-5.39.001 D-779).

---

## Post-Fix Verification

**Feature HEAD after fix-burst:** `69aafcc7`
**just check:** 4035/4035 PASS 0 failed
**SAP-1 (post-fix):** PASS — no new `event_type` emissions in `69aafcc7`; catalog count 71 unchanged
**SAP-2 (post-fix):** PASS — DTU `detections.rs`/`search.rs` boundary fix is behavioral (comparison logic), not schema; TOML↔DTU column parity unchanged

---

## Streak Status

**Streak after pass 8: 0/3** (reset from 1/3)

Pass 7 was CLEAN(strict)=yes (streak 1/3). Pass 8 found ADV-P08-MED-001 — a real correctness defect (inclusive-boundary under-fetch + RFC3339 Z-normalization). CLEAN(strict)=no. Per BC-5.39.001 D-779, streak resets to 0/3. LOCAL pass-9 NEXT (fresh context; verify ADV-P08-MED-001 closure load-bearing — 2 new boundary Red Gate tests confirm inclusive boundary records are included; `to_rfc3339_opts` Z-suffix confirmed; full SAP-1 + SAP-2; streak attempt 0/3 → 1/3).

**Convergence trajectory (v2.x passes 1–8):** 9→4→1→5→1→1→0→1

---

## Spec Updates (story v2.4→v2.5)

- `EC-009` added: inclusive-boundary push-down behavior for `CompareOp::Ge`/`Le` predicates. DTU filtering excludes only strictly-outside records (over-fetch at boundary); DataFusion post-filter narrows to exact inclusive set. BC-2.11.007 result-equivalence invariant holds. RFC3339 normalization note: push-down must serialize timestamps with `to_rfc3339_opts(SecondsFormat::Secs, true)` (Z-suffix) to ensure correct lexicographic string comparison at DataFusion layer.
- `red_gate_tests`: 16→18 (2 new tests: `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` + `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline`).
- `acceptance_criteria_count`: 16 (unchanged — EC-009 is an edge-case behavioral note; BC-2.11.007 v1.8 already scopes Ge/Le).
- BC array unchanged: BC-2.11.007 v1.8 already covers Ge/Le.
