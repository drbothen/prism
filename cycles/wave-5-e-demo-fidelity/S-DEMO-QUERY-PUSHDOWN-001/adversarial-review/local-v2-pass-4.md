# LOCAL Adversary Pass 4 (v2.x) — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Pass:** LOCAL adversary pass 4 — v2.x re-implementation (fourth adversary pass against v2.3 code + EC-003 closure verified)
**Feature HEAD at pass start (frozen):** `0a93ffef`
**Feature HEAD after fix-burst:** `70ae30d2`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-1 | SAP-2 | CLAUDE.md Canonical Principle | ADR-033 v1.0

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

5 findings total: 2 HIGH + 2 LOW + 1 OBS [process-gap]. All 5 CLOSED (or recorded for codification in the case of OBS-001).

Fix-burst committed at `70ae30d2`. just check 4032/4032 PASS 0 failed. CLEAN(strict)=no (findings were present before fix); streak remains 0/3. LOCAL pass-5 NEXT.

---

## Pass-3 Closure Verified (EC-003 WARN Emission Catalog Row)

ADV-P03-MED-001 closure independently re-verified as genuinely load-bearing at HEAD `0a93ffef`:

| Finding ID | Pass-3 Closure | Load-Bearing Verification |
|---|---|---|
| ADV-P03-MED-001 | BC-2.16.002 v1.66 catalog row 71 `push_down.inverted_time_range` added; implementer commit `0a93ffef` adds 2 unit tests | `test_inverted_time_range_emits_warn` uses a capturing tracing subscriber; directly calls `extract_time_window_from_ast` with `start > end`; asserts `push_down.inverted_time_range` WARN event was recorded in the subscriber. Not a doc-comment fix — genuine tracing subscriber capture. Closure is load-bearing. |

**EC-003 class CONFIRMED CLOSED through load-bearing tracing-subscriber test.**

---

## SAP-1 Probe (PG-LP11-001 Tracing Emission Catalog)

**SAP-1 standing probe applied to feature HEAD `0a93ffef` per CLAUDE.md.**

Grep: `rg 'event_type\s*=' crates/ --type rust` on feature HEAD.

**Result:** All `event_type =` emissions in `crates/` have a corresponding row in BC-2.16.002 Canonical Structured Event Catalog (v1.67 after ADV-P04-LOW-002 prose-fix closure). Catalog count 71. No unregistered emissions found.

**SAP-1 result: PASS.** No new findings from SAP-1 probe.

---

## SAP-2 Probe (DTU↔TOML Schema Parity)

**SAP-2 standing probe applied to `crates_touched` sensor specs: prism-dtu-crowdstrike, prism-dtu-armis.**

For each sensor, DTU `types.rs` and route structs compared against TOML `[[tables]]` column declarations.

- **prism-dtu-crowdstrike:** All columns declared in `crowdstrike.sensor.toml` present in DTU response structs. `created_timestamp` `options=["INDEX"]` confirmed. `parse_fql_time_bounds` function name in `state.rs` confirmed (matches story v2.3 corrected name). No TOML-only columns without DTU equivalents.
- **prism-dtu-armis:** All columns in `armis.sensor.toml` present in DTU response structs. `last_seen` and `created_at` `options=["INDEX"]` confirmed. No TOML-only columns without DTU equivalents.

**SAP-2 result: PASS.** No P1 CRITICAL findings.

---

## Findings

### ADV-P04-HIGH-001 — HIGH — Parser function name drift: story + STORY-INDEX cited `parse_created_timestamp_bounds`; implementation is `parse_fql_time_bounds`

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** ADV-P04-HIGH-001

**Description:** The story spec (v2.2) and STORY-INDEX cited the CrowdStrike DTU parser function as `parse_created_timestamp_bounds`. The actual implementation committed in the DTU (`crates/prism-dtu-crowdstrike/src/state.rs`) uses `parse_fql_time_bounds`. This is a POLICY 22 named-entity existence violation — a function name cited in a spec/index artifact does not exist in the codebase. Any future implementer or test-writer reading the story would be directed to a non-existent function name.

6 story body sites + 1 STORY-INDEX note site cited the phantom name.

**Evidence:** `grep parse_created_timestamp_bounds` across story file and STORY-INDEX returns 7 hits; `grep parse_fql_time_bounds` across `crates/prism-dtu-crowdstrike/` confirms the real function name.

**Fix:** story-writer: story v2.2→v2.3 — 6 body sites updated `parse_created_timestamp_bounds` → `parse_fql_time_bounds`; §Changelog v2.3 row added. STORY-INDEX v2.284→v2.285 — Full Story List row note updated.

**Status: CLOSED** — story-writer applied fix (story v2.2→v2.3; STORY-INDEX v2.284→v2.285; 6 story sites + 1 index site corrected; all grep-confirmed clean).

---

### ADV-P04-HIGH-002 — HIGH — AC-CWS-002 named test hand-fed FQL directly; did not call `run_materialization_pipeline`; did not assert both FQL time bounds combined

**Severity:** HIGH
**Confidence:** HIGH
**Finding ID:** ADV-P04-HIGH-002

**Description:** The test named for AC-CWS-002 (in `prism-spec-engine` or `prism-bin`) was found to hand-feed a pre-built FQL string directly into `PipelineExecutor` bypassing `run_materialization_pipeline` (the production entry point per ADR-033 T1). The story's AC-CWS-002 acceptance criterion requires testing the CrowdStrike FQL combined both-bounds form (`created_timestamp:>='start'+created_timestamp:<='end'`). The test only asserted one bound or asserted the final result without a wire-level FQL assertion, making it non-load-bearing for the actual FQL combination logic.

This is the same dead-code-via-test-layer defect class that recurred in passes 1 and 2 at a higher severity, now manifesting at the AC-CWS-002 layer.

**Fix:** implementer commit `70ae30d2`: genuine `run_materialization_pipeline` test in `prism-bin` asserting BOTH bounds — the FQL time-window filter logged at the DTU `/dtu/filter-log` contains `created_timestamp:>=` AND `created_timestamp:<=` in the same combined form; `filtered_count < unfiltered_count` non-vacuous. The misnamed `prism-spec-engine` test renamed honestly and strengthened to assert combined-FQL structure.

**Status: CLOSED** — implementer `70ae30d2`. just check 4032/4032 PASS 0 failed. Wire-level DTU assertion via `/dtu/filter-log` confirms both bounds present in combined FQL form; load-bearing.

---

### ADV-P04-LOW-001 — LOW — `parse_fql_time_bounds` had no in-process unit tests (Armis sibling had 4)

**Severity:** LOW
**Confidence:** HIGH
**Finding ID:** ADV-P04-LOW-001

**Description:** `crates/prism-dtu-crowdstrike/src/state.rs` `parse_fql_time_bounds` function had zero dedicated unit tests at HEAD `0a93ffef`. The comparable Armis DTU time-parsing function had 4 in-process unit tests confirming correct parsing behavior for various inputs. The Armis sibling having more test coverage than CrowdStrike for equivalent functionality is a coverage gap per the production-grade default.

**Fix:** implementer `70ae30d2`: 7 new unit tests in `prism-dtu-crowdstrike/src/state.rs` `#[cfg(test)] mod tests` block, including the story-cited test `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window` which drives `parse_fql_time_bounds` directly with the canonical CrowdStrike `created_timestamp` FQL syntax. Additional tests cover: start-only, end-only, both-bounds, inverted-range pass-through, missing-value graceful parse, and malformed-value handling.

**Status: CLOSED** — implementer `70ae30d2`. 7 unit tests added; all pass.

---

### ADV-P04-LOW-002 — LOW — BC-2.16.002 catalog prose said "before returning the FetchContext to the caller" but function returns tuple

**Severity:** LOW
**Confidence:** HIGH
**Finding ID:** ADV-P04-LOW-002

**Description:** The BC-2.16.002 v1.66 catalog row 71 (`push_down.inverted_time_range`) Recurrence description contained the phrase "before returning the FetchContext to the caller." This is factually wrong: `extract_time_window_from_ast` (`crates/prism-query/src/pushdown.rs`) returns a `(Option<String>, Option<String>)` tuple, not a `FetchContext`. The `FetchContext` is a `prism-spec-engine` type that does not exist in `prism-query`. The error would mislead a reader about the function's return type.

**Fix:** product-owner: BC-2.16.002 v1.66→v1.67 — Recurrence description corrected: "before returning the FetchContext to the caller" → "before returning the `(start_time, end_time)` tuple to the caller". All other catalog row fields (event_type, field schema, level, function, audit role, recurrence semantics) unchanged. No catalog-count change. BC-INDEX v5.86→v5.87 — BC-2.16.002 in-line row updated to v1.67; v5.87 changelog row added.

**Status: CLOSED** — product-owner (BC-2.16.002 v1.66→v1.67; BC-INDEX v5.86→v5.87). Description-only correction; no behavioral change.

---

### ADV-P04-OBS-001 [process-gap] — OBS — Test-docstring file.rs:NNN line-number pins (TD-VSDD-091 adjacency): no lint exists for test-docstring sites

**Severity:** OBS [process-gap]
**Confidence:** MEDIUM
**Finding ID:** ADV-P04-OBS-001

**Description:** Several test function doc-comments in `crates/prism-dtu-crowdstrike/` and `crates/prism-query/src/pushdown.rs` cite source file locations as `file.rs:NNN` (line-number pins). Per TD-VSDD-091, line-number pins in narrative specs are forbidden because they decay on subsequent diffs. The TD-VSDD-091 anti-volatile-pin rule is currently enforced by adversary pass probes for SPEC artifacts, but there is no automated lint or pre-commit gate for test-docstring line-number pins in code files.

**Classification:** [process-gap] / [codification-candidate]. This is a VSDD pipeline/process gap, not a prism implementation defect. The code itself is correct. The gap is that the TD-VSDD-091 anti-volatile-pin discipline has no automated enforcement path for test-docstring sites.

**Disposition:** RECORD as codification candidate. Tag: [process-gap][codification-candidate][TD-VSDD-091-adjacency]. Do NOT block: this is a process discipline gap, not a code or spec defect. Do NOT fix code in this burst. Log in cycle lessons for future SAP/lint/TD consideration.

**Status: RECORDED — not blocking.** Cycle lessons updated. Codification candidate for future session-reviewer / VSDD process improvement.

---

## Fix-Burst Summary

**Feature HEAD after fix-burst:** `70ae30d2`
**just check result:** 4032/4032 PASS 0 failed
**Specialist commits:**

| Specialist | Commit | Work |
|---|---|---|
| story-writer | (in fix-burst `70ae30d2` context) | story v2.2→v2.3 (parse_fql_time_bounds name reconciliation; 6 story body sites + §Changelog v2.3); STORY-INDEX v2.284→v2.285 (Full Story List row note) |
| implementer | `70ae30d2` | AC-CWS-002 test rewritten to `run_materialization_pipeline` + both-bounds DTU wire assertion; 7 unit tests for `parse_fql_time_bounds`; misnamed spec-engine test renamed + strengthened |
| product-owner | (in fix-burst `70ae30d2` context) | BC-2.16.002 v1.66→v1.67 (Recurrence description tuple-return correction); BC-INDEX v5.86→v5.87 |

---

## Post-Fix Verification

- `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window`: passes in `prism-bin` via `run_materialization_pipeline` → wire-level DTU filter-log assertion → BOTH bounds combined in FQL form → load-bearing
- `parse_fql_time_bounds` unit tests (7): all pass in `prism-dtu-crowdstrike/src/state.rs`
- BC-2.16.002 catalog prose corrected: "tuple" not "FetchContext"
- Story + STORY-INDEX: `parse_fql_time_bounds` name consistent with implementation

---

## Convergence Trajectory (v2.x)

| Pass | HEAD (start) | HEAD (end) | Findings | Streak |
|------|-------------|------------|---------|--------|
| v2-pass-1 | `aec965f9` | `f50061a5` | CRIT:3 HIGH:2 OBS:2 | 0/3 |
| v2-pass-2 | `f50061a5` | `4e6dde5c` | CRIT:1 HIGH:1 MED:1 OBS:1 | 0/3 |
| v2-pass-3 | `4e6dde5c` | `0a93ffef` | MED:1 | 0/3 (CLEAN PR-merge: yes) |
| v2-pass-4 | `0a93ffef` | `70ae30d2` | HIGH:2 LOW:2 OBS:1[process-gap] | 0/3 |

**Trajectory:** 9 → 4 → 1 → 5(pre-fix). Pass-4 finding count higher than pass-3 count but these are NEW lens findings (function-name drift + test-coverage gap + catalog-prose correction), not regressions of closed findings from passes 1-3. All pass-1/2/3 closures confirmed LOAD-BEARING and remain closed. Finding class progression: CRIT-class (passes 1-2) → MED-class (pass-3) → HIGH/LOW-class (pass-4: name-drift + test-gap + prose). No REGRESSION of prior-closed findings.

**Next:** LOCAL pass 5 at HEAD `70ae30d2`. Fresh context. Verify ADV-P04-HIGH-002 closure is load-bearing (both-bounds FQL via `run_materialization_pipeline` → wire-level DTU). Full SAP-1 + SAP-2. Streak attempt 0/3 → 1/3 on CLEAN(strict).
