# LOCAL Adversary v2 Passes 9 / 10 / 11 — 3-CLEAN CONVERGENCE

**Story:** S-DEMO-QUERY-PUSHDOWN-001 v2.5  
**Feature HEAD (frozen):** `69aafcc7`  
**Convergence Authority:** BC-5.39.001 (D-779)  
**Passes Covered:** 9, 10, 11 (fresh streak after pass-8 reset)  
**Date:** 2026-06-05  
**Convergence Decision:** D-1015

---

## Streak History (v2 cascade)

| Pass | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak After | Notes |
|------|--------------|-----------------|----------|--------------|-------|
| v2p1 | no | no | CRIT:3, HIGH:2, OBS:2 | 0/3 | dead-code wiring class recurred |
| v2p2 | no | no | CRIT:1, HIGH:1, MED:1, OBS:1 | 0/3 | dead-code at test layer |
| v2p3 | no | yes | MED:1 | 0/3 | SAP-1 catalog row missing (EC-003) |
| v2p4 | no | no | HIGH:2, LOW:2, OBS:1 | 0/3 | name drift + test coverage |
| v2p5 | no | no | HIGH:1 | 0/3 | AC-EQUIV-001 misnamed real-path test |
| v2p6 | no | no | MED:1 | 0/3 | AC-CWS-003 non-load-bearing assertion |
| v2p7 | **yes** | yes | 0 | **1/3** | CLEAN — streak begins |
| v2p8 | no | no | MED:1 | 0/3 | ADV-P08-MED-001 correctness (inclusive-boundary); RESET |
| v2p9 | **yes** | yes | 0 | **1/3** | fresh streak begins; ADV-P08 closure verified |
| v2p10 | **yes** | yes | 0 | **2/3** | full independent re-derivation |
| v2p11 | **yes** | yes | 0 | **3/3 CONVERGED** | convergence emphasis pass; novelty ZERO |

**CORRECTION NOTE (orchestrator-issued):** Pass-10 adversary mis-stated streak as 3/3 converged after pass 10. The correct count is: pass-7 was 1/3 in the prior streak (which pass-8 reset), then pass-9 begins the fresh streak at 1/3, pass-10 is 2/3, and pass-11 is 3/3. Convergence is authoritatively at pass 11 per BC-5.39.001.

---

## Pass 9 — Verdict

**Feature HEAD:** `69aafcc7` (frozen — no code change from pass-8 fix)  
**CLEAN(strict):** yes  
**CLEAN(PR-merge):** yes  
**Findings:** 0  
**Streak:** 0/3 → 1/3

### Focus: ADV-P08-MED-001 closure verification

ADV-P08-MED-001 (inclusive-boundary under-fetch for Ge/Le predicates) was closed at `69aafcc7` with two root-cause fixes:

1. **DTU boundary logic** — `device_in_time_window` / `alert_in_time_window` in `crates/prism-dtu-crowdstrike/src/routes/detections.rs` and `crates/prism-dtu-armis/src/routes/search.rs`: corrected to strict-outside exclusion (`ts < start || ts > end`) so records at exact boundary pass through. DataFusion inclusive semantics then narrows correctly.

2. **RFC3339 Z-normalization** — `pipeline.rs` `to_rfc3339()` produced `+00:00` (ASCII 43); fixture timestamps stored as `Z` (ASCII 90). Lexicographic mismatch caused DataFusion string comparison to drop exact-boundary records. Fixed to `to_rfc3339_opts(SecondsFormat::Secs, true)` which emits Z suffix.

**Closure verification:** Both root-cause fixes confirmed load-bearing via the 2 new Red Gate tests added at `69aafcc7`:
- `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline`
- `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline`

Both tests call `run_materialization_pipeline` (not `PipelineExecutor::execute` directly), assert boundary records ARE present in results, and would fail if the boundary exclusion logic were reverted.

**SAP-1:** PASS — catalog count 71; no unregistered `event_type` emissions in workspace (`rg 'event_type\s*=' crates/ --type rust`).  
**SAP-2:** PASS — CrowdStrike DTU (`detections.rs`) and Armis DTU (`search.rs`) column/type parity with `crowdstrike.sensor.toml` and `armis.sensor.toml` confirmed; boundary fix is behavioral (comparison logic), not schema change.

---

## Pass 10 — Verdict

**Feature HEAD:** `69aafcc7` (unchanged)  
**CLEAN(strict):** yes  
**CLEAN(PR-merge):** yes  
**Findings:** 0  
**Streak:** 1/3 → 2/3  
**Lens:** full independent re-derivation — all 4 sensors, all 16 ACs, all 18 Red Gate tests

### Re-derivation summary

All 4 push-down sensors verified:

- **CrowdStrike FQL time-window:** `parse_fql_time_bounds` extracts `gt`/`lt`/`gte`/`lte` predicates; DTU `detections.rs` applies inclusive boundary via strict-outside exclusion; `test_ac_cws_001_crowdstrike_fql_pushdown_via_run_materialization_pipeline` load-bearing.
- **CrowdStrike LIMIT:** `test_ac_cws_002_crowdstrike_limit_pushdown_via_run_materialization_pipeline` calls production path; DTU filter-log asserts both bounds present; `result.len() <= limit`.
- **CrowdStrike no-filter absence:** `test_ac_cws_003_crowdstrike_no_filter_absence_via_dtu_filter_log` queries DTU `/dtu/filter-log`; asserts `created_timestamp` absent; `result.len() == 50`.
- **Armis AQL time-window:** `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` drives `SpecDrivenSensorAdapter::fetch()` directly; `search.rs` boundary logic confirmed inclusive.
- **AC-EQUIV-001 result equivalence:** `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` in `prism-bin`; subset/no-fabrication invariant load-bearing.
- **Inclusive boundary (CrowdStrike + Armis):** 2 new tests at `69aafcc7` confirmed.

16 ACs accounted for; 18 Red Gate tests per story v2.5 frontmatter confirmed. BC-2.01.013 v1.14 result-equivalence clause satisfied. BC-2.11.007 v1.8 push-down semantics clause satisfied. ADR-033 architecture satisfied.

**SAP-1:** PASS — zero unregistered `event_type` emissions.  
**SAP-2:** PASS — CrowdStrike + Armis DTU↔TOML parity confirmed.

---

## Pass 11 — Convergence Emphasis Verdict

**Feature HEAD:** `69aafcc7` (frozen)  
**CLEAN(strict):** yes  
**CLEAN(PR-merge):** yes  
**Findings:** 0  
**Streak:** 2/3 → **3/3 CONVERGED**  
**Novelty:** ZERO

### Convergence confirmation

Pass 11 applied a fresh-context holistic sweep across:

1. **Result-equivalence re-derivation** — all 4 sensors re-derived from push-down entry points through production path (`run_materialization_pipeline` → `SpecDrivenSensorAdapter::fetch` → DTU). No divergence from spec found.
2. **Spec↔impl↔index drift check** — story v2.5 frontmatter fields (`acceptance_criteria_count: 16`, `red_gate_tests: 18`) match counted AC items and test methods. STORY-INDEX v2.287 row consistent. BC-INDEX v5.87 consistent with BC versions in story `bc_pins`.
3. **BC-2.01.013 v1.14 conformance** — query push-down result-equivalence clause verified satisfied.
4. **BC-2.11.007 v1.8 conformance** — FQL/AQL push-down semantics clause verified satisfied.
5. **EC-009 (inclusive boundary behavioral note)** present in story body; RC3339 Z-normalization rule present.
6. **SAP-1 final sweep:** PASS — catalog 71 rows; no unregistered emissions.
7. **SAP-2 final check:** PASS — no TOML-only columns missing DTU equivalents.

Zero findings. Novelty ZERO. Convergence criterion met per BC-5.39.001 D-779: three consecutive CLEAN(strict)=yes passes (9, 10, 11) on frozen feature HEAD `69aafcc7`.

**just check (at HEAD 69aafcc7):** 4035/4035 PASS, 0 failed.

---

## Convergence Summary

| Metric | Value |
|--------|-------|
| Convergence passes | 9, 10, 11 |
| Convergence HEAD | `69aafcc7` |
| Story version | v2.5 |
| Acceptance criteria | 16 |
| Red Gate tests | 18 |
| just check | 4035/4035 PASS |
| BC-5.39.001 authority | D-779 |
| Decision | D-1015 |

**LOCAL 3-CLEAN CONVERGED per BC-5.39.001 D-779.**  
**NEXT: per-story-delivery Step 5 — demo-recorder (per-AC evidence) → push → pr-manager 9-step PR cycle → PR-LEVEL 3-CLEAN.**
