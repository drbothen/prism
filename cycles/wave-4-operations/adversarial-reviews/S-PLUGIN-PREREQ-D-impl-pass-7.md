# S-PLUGIN-PREREQ-D Adversary Impl-Pass-7 Report

**Date:** 2026-05-15
**Pass:** impl-pass-7 (7th implementation adversarial pass)
**Target branch/commit:** `feature/S-PLUGIN-PREREQ-D@862e721a`
**Verdict:** BLOCKED
**Streak status:** 0/3 (reset — 7th consecutive BLOCKED)
**Decision burst:** D-559

---

## Summary

Adversary impl-pass-7 dispatched against `feature/S-PLUGIN-PREREQ-D@862e721a`
(post fix-burst-impl-6 closure of all 4 impl-pass-6 findings via parallel
split-routing).

**Outcome:** BLOCKED — 1 in-perimeter finding (0 CRIT + 0 HIGH + 1 MED + 0 LOW)
+ 1 process-gap OBS carry-forward.

**LIGHTEST PASS YET:** This is the single-finding pass across the entire 7-pass
arc. Severity decay continues terminal: 0 CRIT, 0 HIGH, 1 MED (spec
documentation gap only), 0 LOW. The production code layer is fully converged.
Adversary forecasts approximately 80% CLEAN probability for impl-pass-8 after
fix-burst-impl-7 closes F-PASS7-MED-001.

---

## Prior Closure Verification

### All 4 impl-pass-6 Findings — HELD

**F-PASS6-MED-001 HELD:** `tests/fixtures/src/component_model_dispatch.wit` +
`component_model_dispatch.core.wat` + `component_model_dispatch.README.md` +
`Justfile` recipe `build-fixture-component_model_dispatch` all present at
`tests/fixtures/src/`. Rebuild verification path documented. Reproducibility
gap CLOSED.

**F-PASS6-LOW-001 HELD:** `tests/fixtures/component_model_dispatch.prx` exists
at the story-mandated path. Old path `crates/prism-spec-engine/fixtures/`
confirmed gone. `Component::from_file` path in test updated.

**F-PASS6-LOW-002 HELD:** `plugin_integration_tests.rs:3` header reads
`//! Traces to: S-PLUGIN-PREREQ-D (v1.35)`. Confirmed.

**F-PASS6-LOW-003 HELD:** Story v1.34 §Changelog Burst column reads
`S-PLUGIN-PREREQ-D-fix-burst-impl-4`. Confirmed.

### Carry-Forward Spot-Checks — All HOLD

**F-PASS5-HIGH-001 (production-linker test) — HOLD CONFIRMED:**
`plugin_integration_tests.rs:2001-2014` loads
`tests/fixtures/component_model_dispatch.prx` and instantiates against
`PluginRuntime::build_linker(&engine)` (production builder). Asserts
`Val::U16(403)`. Load-bearing: a regression of `host_functions.rs:452`
from `Val::U16(response.status)` to `Val::U32(...)` would cause this
test to fail with a wasmtime type-mismatch trap. CONFIRMED.

**F-PASS3-CRIT-001 (boot sequence ordering) — HOLD CONFIRMED:**
`boot.rs:160` calls `plugin_load_step_with_audit` BEFORE `boot.rs:164`
calls `step7_init_storage`. Ordering constraint satisfied. CONFIRMED.

**F-PASS3-CRIT-002 (Val::U16 writeback) — HOLD CONFIRMED:**
`host_functions.rs:452` uses `Val::U16(response.status)`. CONFIRMED.

**BC-2.16.002 v1.17 row 32 — HOLD CONFIRMED:**
`plugin_log_level_unrecognized` present as row 32 with correct fields.

---

## New Findings

### F-PASS7-MED-001 — Fixture Strategy Table Stale; 5th Fixture Unregistered

**Severity:** MEDIUM
**Routing:** story-writer
**Predicted:** Yes — explicitly noted in impl-pass-6 dispatch brief as
"minor scope gap: Fixture Strategy table not extended" and excluded from
fix-burst-impl-6 implementer scope.

**Finding:**
The new `tests/fixtures/component_model_dispatch.prx` committed in
fix-burst-impl-6 (worktree commits b1752cb5 + 862e721a) is the **5th
fixture** in the story's fixture ecosystem. The four existing fixtures are:
- `tests/fixtures/minimal.prx`
- `tests/fixtures/trap_plugin.prx`
- `tests/fixtures/infinite_loop.prx`
- `tests/fixtures/bad_wit.prx`

The new `component_model_dispatch.prx` brings the total to 5. However, the
story's Fixture Strategy section was NOT updated:

**Site A — Task 13, story line 666:**
```
Commit all 4 .prx test fixtures to tests/fixtures/ ...
```
Still says 4. Should be 5.

**Site B — story line 819, Strategy decision header:**
```
Fixture Strategy decision: ... all 4 test fixtures committed ...
```
Still says 4. Should be 5.

**Site C — story lines 831-836, Strategy table:**
The table has exactly 4 rows (minimal, trap_plugin, infinite_loop, bad_wit).
The new `component_model_dispatch.prx` is absent from the table.

**Why MED, not LOW:**
The Fixture Strategy table is a traceability artifact — its purpose is to
provide a complete inventory of what fixtures exist and why. An incomplete
table means a future reader of the story spec cannot derive the full fixture
set from the story alone. This is a documentation gap that affects the
auditability of the implementation. It is not blocking production behavior
(the fixture works correctly), but it is a specification accuracy gap under
the production-grade default.

**Fix prescription:**
Three single-line story edits (approximately 5 minutes total):
1. Task 13 count: "4 .prx test fixtures" → "5 .prx test fixtures"
2. Strategy header count: "all 4" → "all 5"
3. Strategy table: append 5th row for `component_model_dispatch.prx`
   - Fixture: `tests/fixtures/component_model_dispatch.prx`
   - WIT world: `prism:dispatch-test@0.1.0`
   - Build tool: wasm-tools 1.248.0 (`component embed + component new`)
   - Purpose: Route A pre-built fixture for production-linker dispatch test
     (F-PASS5-HIGH-001 closure; `PluginRuntime::build_linker` end-to-end)
   - Source: `tests/fixtures/src/component_model_dispatch.wit` +
     `component_model_dispatch.core.wat` + Justfile recipe

Additionally: v1.37 changelog row and STORY-INDEX v2.107 row sync.

---

## Process-Gap OBS

### PG-IMPL-LP6-002 (carry-forward from impl-pass-6 queue — materialized)

**Status:** Already in codification queue (count 30; unchanged this pass)

**Finding:** When a fix-burst commits a new `.prx` to `tests/fixtures/`, the
story spec's Fixture Strategy table + Task 13 enumeration MUST be updated in
the same burst (or the immediately following burst if the fix-burst dispatch
brief explicitly excludes story-file modifications).

This gap was *predicted* by the impl-pass-6 dispatch brief ("Fixture Strategy
table 5th-entry not extended — impl-pass-7 adjudicates") and has now
*materialized* as a confirmed finding (F-PASS7-MED-001). The prediction-to-
materialization pipeline demonstrates the codification queue item is correct
in its behavioral diagnosis.

**Codification target:** Standing dispatch language for story-writer role:
"After any fix-burst that commits a new `.prx` to `tests/fixtures/`, verify
that the story's Fixture Strategy table row count and Task enumeration are
in sync with the actual fixture count. This is a 5-minute check, not a
deferred task."

---

## Convergence Assessment

### 7-Pass Trajectory

| Pass | CRIT | HIGH | MED | LOW | Net | Key Signal |
|------|------|------|-----|-----|-----|------------|
| 1 | 5 | 6 | 4 | 3 | 18 | Initial implementation |
| 2 | 2 | 3 | 4 | 3 | 12 | Paper-fix cascade begins |
| 3 | 2 | 1 | 2 | 1 | 6 | Val types + boot order |
| 4 | 0 | 0 | 2 | 0 | 2 | Test paper-fix narrowed |
| 5 | 0 | 1 | 1 | 1 | 3 | Production-linker gap |
| 6 | 0 | 0 | 1 | 3 | 4 | FIRST ZERO CRIT+HIGH |
| 7 | 0 | 0 | 1 | 0 | 1 | **LIGHTEST PASS YET** |

**CRIT trajectory:** 5 → 2 → 2 → 0 → 0 → 0 → 0 (converged at pass 4)
**HIGH trajectory:** 6 → 3 → 1 → 0 → 1 → 0 → 0 (converged at pass 6)
**MED trajectory:** 4 → 4 → 2 → 2 → 1 → 1 → 1 (one persistent MED; spec-only)
**LOW trajectory:** 3 → 3 → 1 → 0 → 1 → 3 → 0 (fully cleared pass 7)

### Assessment

**Production code layer:** FULLY CONVERGED. No CRIT, no HIGH, no LOW findings
for 3+ consecutive passes. The pass-7 single MED finding is a story spec
documentation gap (table row count), not a production code defect.

**Specification documentation layer:** Single open gap (F-PASS7-MED-001).
Mechanical to close (3 single-line edits). No logic required.

**Forecast for impl-pass-8:**
After fix-burst-impl-7 closes F-PASS7-MED-001 with the prescribed 3-site
story edit:
- 0 predicted CRIT (production code clean for 4 consecutive passes)
- 0 predicted HIGH (production code clean for 2 consecutive passes)
- 0 predicted MED (F-PASS7-MED-001 fully closed by prescribed fix)
- 0 predicted LOW (all LOW sites enumerated and closed)
- CLEAN probability: approximately 80%

The 20% residual uncertainty accounts for:
- Adversary discovering a previously undetected story-spec inconsistency
- A new sibling-sweep gap introduced by the v1.37 changelog row
- A previously undetected carry-forward that becomes visible at lower noise

**BC-5.39.001 streak:** 0/3 (remains; F-PASS7-MED-001 is a blocking finding
regardless of severity). Streak advances only at CLEAN pass.

---

## Summary Verdict

**BLOCKED** — 0 CRIT + 0 HIGH + 1 MED + 0 LOW + 1 OBS carry-forward.

F-PASS7-MED-001 is the sole blocking finding: story Fixture Strategy table
stale after 5th fixture committed in fix-burst-impl-6. Mechanical story-writer
fix. No production code changes required.

**NEXT ACTION:** Dispatch fix-burst-impl-7 to story-writer only. Scope:
3 single-line story edits + v1.37 changelog row + STORY-INDEX v2.107.
After closure, dispatch adversary impl-pass-8.
