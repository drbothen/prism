# LOCAL Adversary Pass 3 (v2.x) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 3 — v2.x re-implementation (third adversary pass against v2.2 code + EC-003 fix)
**Feature HEAD at pass start (frozen):** `4e6dde5c`
**Feature HEAD after fix-burst (EC-003 WARN emission):** `0a93ffef`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): yes**
**Streak after: 0/3**

1 finding total: 1 MEDIUM (ADV-P03-MED-001).

Finding CLOSED by combined code fix-burst (implementer commit `0a93ffef`) + spec fix-burst (BC-2.16.002 v1.65→v1.66, new catalog row 71 `push_down.inverted_time_range`; BC-INDEX v5.85→v5.86 reconciled per D-1010). CLEAN(PR-merge)=yes achieved (zero CRIT/HIGH/MED after fix). Streak remains 0/3 (CLEAN(strict)=no prevents streak advancement; one MED was present before fix). LOCAL pass 4 NEXT.

---

## Pass-2 Closures Verified (Load-Bearing through `run_materialization_pipeline` path)

All pass-2 closures independently re-verified as genuinely load-bearing through the real production path (`run_materialization_pipeline → SpecDrivenSensorAdapter::fetch → FQL/AQL-build → DTU`) at HEAD `4e6dde5c`:

| Finding ID | Pass-2 Closure | Load-Bearing Verification |
|---|---|---|
| ADV-P02-CRIT-001 | 3 new e2e tests via `run_materialization_pipeline` (CrowdStrike FQL, CrowdStrike LIMIT, Armis AQL augmentation) | Tests confirmed to call `run_materialization_pipeline` (not `PipelineExecutor::execute` directly); wire-level DTU assertions via `/dtu/filter-log` and aql-log routes; `filtered_count < unfiltered_count` non-vacuous; confirmed load-bearing under `just check 4024/4024 PASS` |
| ADV-P02-MED-001 | `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` drives `SpecDrivenSensorAdapter::fetch()` | Test calls `fetch()` on a real `SpecDrivenSensorAdapter` instance; asserts DTU aql-log receives `after:<ts>`; augmentation decision branch is the only code path that produces that assertion; confirmed load-bearing |
| ADV-P02-HIGH-001 (=DRIFT-P1-001) | Story v2.1→v2.2: `crates_touched += prism-dtu-crowdstrike`; AC-CWS-DTU-001 added | Story file confirms `prism-dtu-crowdstrike` in `crates_touched`; `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window` present in crate; Red Gate test exercises DTU filter honoring via `/dtu/filter-log` route; load-bearing |
| OBS-NOTE-001 | Resolved benign INFRA-OBS | `just check --no-fail-fast 4024/4024 PASS` at `4e6dde5c` with no test failures; teardown race confirmed non-exploitable and non-flaking at pass-3 check |

**Dead-code defect class confirmed CLOSED through real execution path. The class that recurred in passes 1 and 2 (production path bypassed by tests) is now definitively closed: three test vectors drive `run_materialization_pipeline` end-to-end and produce load-bearing wire-level assertions at the DTU boundary.**

---

## SAP-1 Probe (PG-LP11-001 Tracing Emission Catalog)

**SAP-1 standing probe applied to feature HEAD `4e6dde5c` (pre-fix) per CLAUDE.md.**

Grep: `rg 'event_type\s*=' crates/ --type rust` on feature HEAD.

**Finding:** `event_type = "push_down.inverted_time_range"` emission present at `crates/prism-query/src/pushdown.rs` (`extract_time_window_from_ast` function). No corresponding catalog row existed in BC-2.16.002 Canonical Structured Event Catalog at pass-3 start (BC-2.16.002 was at v1.65; catalog count 70).

**Disposition:** ADV-P03-MED-001 — PG-LP11-001 obligation not met for EC-003 emission. **CLOSED in this burst** (see below). BC-2.16.002 v1.65→v1.66: catalog row 71 `push_down.inverted_time_range` WARN added; BC-INDEX v5.85→v5.86 (collision reconciliation per D-1010). Catalog count 70→71.

**SAP-1 result after fix:** Catalog row 71 present in BC-2.16.002 v1.66. No remaining event_type emissions in `crates/` lacking catalog rows at HEAD `0a93ffef`.

---

## SAP-2 Probe (DTU↔TOML Schema Parity)

**SAP-2 standing probe applied to `crates_touched` sensor specs: prism-dtu-armis, prism-dtu-crowdstrike.**

For each sensor, DTU types.rs and route structs compared against TOML `[[tables]]` column declarations.

- **prism-dtu-crowdstrike:** All columns declared in `crowdstrike.sensor.toml` `[[tables]]` blocks present in DTU `DetectionListParams` / detection response structs. `created_timestamp` with `options = ["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.
- **prism-dtu-armis:** All columns in `armis.sensor.toml` (devices and alerts tables) present in `SearchQueryParams` + DTU response structs (device and alert route shapes). `last_seen` and `created_at` with `options = ["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.

**SAP-2 result: PASS.** No P1 CRITICAL findings (no TOML column without DTU equivalent).

---

## Finding

### ADV-P03-MED-001 — MEDIUM — EC-003 `push_down.inverted_time_range` WARN emission lacks BC-2.16.002 catalog row (SAP-1 obligation unmet)

**Severity:** MEDIUM
**Confidence:** HIGH
**Finding ID:** ADV-P03-MED-001

**Description:** The code commit `0a93ffef` implements EC-003 of S-DEMO-QUERY-PUSHDOWN-001 v2.2: when `extract_time_window_from_ast` detects an inverted time window (`start_time > end_time`), it emits `tracing::warn!(event_type = "push_down.inverted_time_range", ...)`. This emission is present at `crates/prism-query/src/pushdown.rs`. Per CLAUDE.md SAP-1 and PG-LP11-001, every `event_type =` emission must have a corresponding row in BC-2.16.002 Canonical Structured Event Catalog in the same commit as the implementation.

At pass-3 start (HEAD `4e6dde5c`), the implementation was present (EC-003 WARN emission wired with 2 unit tests) but the BC-2.16.002 catalog row was absent. BC-2.16.002 was at v1.65 (catalog count 70). The observability catalog is out of sync with the implementation.

**Impact:** Observability-only. The correctness of push-down behavior (no incorrect filters sent to sensors for inverted windows, pass-through semantics) is unaffected. However, the SAP-1 standing probe is a POLICY, not a guideline. Missing catalog row = MED finding per PG-LP11-001.

**Evidence:** `rg 'event_type.*push_down' crates/ --type rust` confirms emission at `crates/prism-query/src/pushdown.rs::extract_time_window_from_ast`. BC-2.16.002 at v1.65 has catalog count 70 and no `push_down.inverted_time_range` row.

**Fix:** BC-2.16.002 v1.65→v1.66 — add catalog row 71 `push_down.inverted_time_range` WARN to the Canonical Structured Event Catalog. Fields: `start_time: %display` (ISO8601 lower bound), `end_time: %display` (ISO8601 upper bound). Catalog scope statement extended to include `prism-query` push-down time-window analysis. BC-INDEX v5.85→v5.86 (collision reconciliation per D-1010 critical section).

**Status: CLOSED** — BC-2.16.002 v1.65→v1.66 authored by product-owner (per SAP-1 pattern; BC-amendment is spec-owner scope). Code fix: `0a93ffef` adds 2 unit tests (`test_inverted_time_range_emits_warn` — asserts WARN event emitted; `test_non_inverted_time_range_no_warn` — asserts normal window does not emit). Catalog row 71 confirmed present in BC-2.16.002 v1.66. BC-INDEX v5.86 collision reconciliation complete. just check 4024/4024 PASS 0 failed.

---

## Post-Fix Verification

**Feature HEAD after EC-003 fix-burst:** `0a93ffef`
**just check result:** 4024/4024 PASS 0 failed
**New tests added:** 2 unit tests in `crates/prism-query/src/pushdown.rs` test module:
- `test_inverted_time_range_emits_warn` — asserts `push_down.inverted_time_range` WARN event emitted when `start_time > end_time`; load-bearing
- `test_non_inverted_time_range_no_warn` — asserts no WARN for correctly ordered time window; guards against over-emission

**SAP-1 after fix:** Catalog row 71 present in BC-2.16.002 v1.66. No remaining unregistered event_type emissions in `crates/`.
**SAP-2 after fix:** DTU↔TOML parity confirmed (no code change in DTU types; parity verified unchanged).

---

## Convergence Trajectory (v2.x)

| Pass | HEAD (start) | HEAD (end) | Findings | Streak |
|------|-------------|------------|---------|--------|
| v2-pass-1 | `aec965f9` | `f50061a5` | CRIT:3 HIGH:2 OBS:2 | 0/3 |
| v2-pass-2 | `f50061a5` | `4e6dde5c` | CRIT:1 HIGH:1 MED:1 OBS:1 | 0/3 |
| v2-pass-3 | `4e6dde5c` | `0a93ffef` | MED:1 | 0/3 (CLEAN PR-merge: yes) |

**Trajectory:** 9 → 4 → 1. Finding count collapsing. Dead-code defect class CONFIRMED CLOSED through load-bearing tests. CLEAN(PR-merge) achieved. Remaining gap to CLEAN(strict): pass-4 must return zero findings at all severities.

**Next:** LOCAL pass 4 at HEAD `0a93ffef`. Fresh context. Verify EC-003 WARN emission tests are load-bearing (tracing subscriber captures). Re-run SAP-1 full workspace sweep. Re-run SAP-2. Confirm streak attempt 0/3 → 1/3 on CLEAN(strict).
