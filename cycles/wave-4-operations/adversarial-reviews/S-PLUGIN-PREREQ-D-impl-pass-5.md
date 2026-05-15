---
pass: impl-pass-5
story: S-PLUGIN-PREREQ-D
target_branch: feature/S-PLUGIN-PREREQ-D
target_sha: e1d83fa4
verdict: BLOCKED
streak_before: 0/3
streak_after: 0/3
findings_total: 3
findings_crit: 0
findings_high: 1
findings_med: 0
findings_low: 2
findings_obs: 1
date: 2026-05-15
recorded_by: D-555
consecutive_single_commit: 60
---

# S-PLUGIN-PREREQ-D Adversary Impl-Pass-5

## Header

- **Pass:** impl-pass-5 (LOCAL adversary cascade per BC-5.39.001)
- **Target:** `feature/S-PLUGIN-PREREQ-D@e1d83fa4`
- **Dispatched:** 2026-05-15
- **Verdict:** BLOCKED
- **3-CLEAN Streak:** 0/3 → 0/3 (BLOCKED; no advance; 5th consecutive blocked pass)
- **Prior passes:** impl-pass-1 BLOCKED (18) → fix-burst-impl-1 CLOSED → impl-pass-2 BLOCKED (12) → fix-burst-impl-2 CLOSED → impl-pass-3 BLOCKED (6) → fix-burst-impl-3 CLOSED → impl-pass-4 BLOCKED (2) → fix-burst-impl-4 CLOSED → **impl-pass-5 BLOCKED (3)**

## Closure Verification — Fix-Burst-Impl-4 Prior Findings

| Finding | Prior Verdict | Pass-5 Verification | Evidence |
|---------|--------------|--------------------|-|
| F-PASS4-HIGH-001 (test paper-fix — 5 inline-replica tests don't dispatch through registered callbacks) | CLOSED (e1d83fa4) | PAPER-FIX REOPENED | New dispatch test at `plugin_integration_tests.rs:1538-1862` satisfies dispatch mechanics (Component Model component + instantiate + .call()) BUT uses `Linker::<HostState>::new(&engine)` (line 1553) instead of `PluginRuntime::build_linker(&engine)` (production builder at mod.rs:168); test registers its OWN simplified callback (lines 1565-1618) writing `Val::U16(response.status)`; a regression of host_functions.rs:452 from `Val::U16(response.status)` to `Val::U32(u32::from(response.status))` inside `register_host_functions` would NOT cause this test to fail — the test's own callback is what gets exercised, not the production one |
| F-PASS4-MED-001 (story §Structured Event Catalog Additions count drift 12→13) | CLOSED (b788d53c) | VERIFIED | Story v1.34 confirmed; §Structured Event Catalog Additions says "13 events" at all swept sites; 13th catalog row `plugin_log_level_unrecognized` present in table |

**Summary:** 1/2 prior closures VERIFIED. 1 PAPER-FIX-REOPENED (F-PASS4-HIGH-001 → F-PASS5-HIGH-001). Production code is verified correct at all prior fix-burst sites. The narrowing gap is exclusively in the test evidence layer: the dispatch test now uses more genuine Component Model mechanics but wires a test-local linker rather than the production linker.

## New Findings

### F-PASS5-HIGH-001 — Test Paper-Fix Recurrence — Test-Local Linker vs Production Linker (5th Cascade; TD-VSDD-059)

**Severity:** HIGH
**Routing:** implementer
**Status:** OPEN

**Location:** `crates/prism-spec-engine/tests/plugin_integration_tests.rs:1538-1862`

**Description:**

The dispatch test added by fix-burst-impl-4 (`test_F_PASS4_HIGH_001_component_model_dispatch_invokes_host_http_request_through_registered_callback`) advances past all prior iterations by using genuine Component Model mechanics: it builds a WAT component, compiles it with `Component::from_binary`, creates a `Linker`, instantiates it, and calls an exported function via `.call()`. This is structurally correct for testing dispatch.

**However, the test uses a test-local linker, not the production linker:**

- Line 1553: `let mut linker = Linker::<HostState>::new(&engine);`
- Lines 1565-1618: The test registers its OWN `host.http-request` callback directly on this test-local linker. The callback is a closure that writes `Val::U16(response.status)`.
- The production builder `PluginRuntime::build_linker(&engine)` (defined at `mod.rs:168`) calls `register_host_functions` which registers the production implementations.
- The test's callback at line 1616 (`Val::U16(response.status)`) is a COPY of what the production code does at `host_functions.rs:452` (`Val::U16(response.status)`).

**The regression-detection gap:**

If a future commit changes `host_functions.rs:452` from `Val::U16(response.status)` to `Val::U32(u32::from(response.status))`, the following occurs:
- Production code is broken (WIT `u16` slot receives `u32` Val variant)
- The dispatch test at `plugin_integration_tests.rs:1538-1862` would PASS because the test-local callback (line 1616) still writes `Val::U16(response.status)` — the test is exercising its own closure, not the production function

The implementer's "sanity-revert demonstration" was performed at line 1616 (the test's own callback registration), not at `host_functions.rs:452` (the production code). This is why the sanity-revert found the expected trap: the test IS load-bearing for its own callback, but not for the production callback.

**5th paper-fix recurrence enumeration:**

| Pass | Paper-fix layer |
|------|----------------|
| impl-pass-1 | Callback bodies not wired to production host_* functions (plain closures returning hardcoded values) |
| impl-pass-2 | run_boot_sequence called from PrismCommand::Start, BUT step7_init_storage todo!() fires before plugin_load_step_with_audit is reached |
| impl-pass-3 | Val-type fixes applied, BUT dispatch test uses `linker.instantiate_pre()` without calling an exported function — no actual dispatch occurs |
| impl-pass-4 | Dispatch test uses `.call()` on exported function, BUT uses `Linker::new` rather than production linker — production callback body not exercised |
| impl-pass-5 | Dispatch test uses genuine Component Model mechanics + `.call()`, BUT registers test-local callback on test-local `Linker::new` — production `register_host_functions` not invoked |

**The iteration pattern:** Each pass the implementer satisfies more of the prior prescription's mechanical requirements while still failing the "would this test catch a production regression?" question.

**Fix prescription — two routes (Route B preferred):**

**Route A (higher-cost): Pre-built `.prx` fixture**
1. Build a Component Model `.prx` artifact that imports `host::http-request` from the Prism host interface (WIT-defined)
2. Load the `.prx` binary in the test via `Component::from_binary`
3. Instantiate using `PluginRuntime::build_linker(&engine)` — the production builder that calls `register_host_functions`
4. Call the exported function that internally invokes `host::http-request`
5. Assert on the Val::U16 status field

**Route B (lower-cost, preferred): `PluginRuntime::build_linker` + Func extraction + direct invocation with synthesized Val params**
1. Call `PluginRuntime::build_linker(&engine)` to get the production-built linker
2. Use `.get_func()` on the linker to extract the registered `host.http-request` function
3. Synthesize `Val` params for method, url, headers, body
4. Call the extracted function via `.call()` with the synthesized params
5. Assert that the returned `Val::U16(...)` status field matches expected value
6. This directly invokes the production function registered by `register_host_functions` at `host_functions.rs:340-470`

**Route B sanity-revert REQUIREMENT for closure:**
- Revert `host_functions.rs:452`: change `Val::U16(response.status)` → `Val::U32(u32::from(response.status))`
- Run the test: it MUST FAIL with a wasmtime type-mismatch trap or assertion error (because production code is now returning wrong Val variant to the registered function interface)
- Revert back immediately
- If the test passes with the bad code, the fix is not load-bearing — do not claim closure

**Carry-forward from prior prescription:** The test may still use WAT + Component Model dispatch if that is the easier route to wire `PluginRuntime::build_linker`. Route B via direct Func extraction is lower-cost and does not require WAT at all.

---

### F-PASS5-LOW-001 — STORY-INDEX Row Attribution Wording Ambiguity

**Severity:** LOW
**Routing:** state-manager (cosmetic wording fix)
**Status:** OPEN

**Location:** `.factory/stories/STORY-INDEX.md` line 394 (approximate)

**Description:**

STORY-INDEX row for S-PLUGIN-PREREQ-D (v2.104) contains the annotation:

> "fix-burst-impl-3 sibling-sweep"

However, the story body sibling-sweep (bumping "12 events" → "13 events" at 4 active-body sites + appending `plugin_log_level_unrecognized` catalog row) was performed by fix-burst-impl-4 via factory commit `b788d53c` (D-554), not by fix-burst-impl-3.

fix-burst-impl-3 is when the 13th catalog row was added to BC-2.16.002 v1.17 (commit `d8f51552`). fix-burst-impl-4 is when the story body sweep happened (commit `b788d53c`).

**Impact:** Low — cosmetic wording ambiguity in STORY-INDEX annotation; no functional impact on spec semantics.

**Fix:** Update STORY-INDEX annotation to accurately attribute the story body sweep to fix-burst-impl-4 (D-554), and the BC-2.16.002 row addition to fix-burst-impl-3 (D-552/d8f51552).

---

### F-PASS5-LOW-002 — Story §Structured Event Catalog Additions `event_type` Field Asymmetry

**Severity:** LOW
**Routing:** story-writer (alignment fix)
**Status:** OPEN

**Location:** `S-PLUGIN-PREREQ-D.md` §Structured Event Catalog Additions, row for `plugin_log_level_unrecognized` (approximate line 809)

**Description:**

The §Structured Event Catalog Additions table row for `plugin_log_level_unrecognized` lists the "Fields" column as:

> `plugin_id, received_name, event_type`

However, BC-2.16.002 v1.17 row 32 lists only:

> `plugin_id, received_name`

`event_type` is the row key (the event type identifier itself), not a payload field. 12 sibling rows in the same §Structured Event Catalog Additions table do NOT include `event_type` in the Fields column — they list only the payload-specific fields. This row is asymmetric with all 12 siblings.

**Impact:** Low — the inconsistency could confuse implementation reading the story table. The BC is authoritative (BC source-of-truth precedence rule); the story is misaligned.

**Fix:** Remove `event_type` from the Fields column of the `plugin_log_level_unrecognized` row. Updated Fields: `plugin_id, received_name`. Bump story version (v1.34 → v1.35 or note as cosmetic patch).

---

## Process-Gap Candidate (Codification Queue 26 → 27)

### PG-IMPL-LP5-001 — Production-Linker vs Test-Linker Boundary Detector

**Pattern observed (5th cascade recurrence, progressively refined):** Each fix-burst iteration satisfies more of the prior prescription's mechanics while maintaining the same fundamental gap: the test exercises a test-local callback, not the production callback registered by `register_host_functions`.

**Proposed codification:** Adversary must verify production-linker vs test-linker boundary for every test claiming to close a TD-VSDD-059 paper-fix HIGH.

Detection heuristic: grep test body for `Linker::new(` / `Linker::<.*>::new(` AND check whether production builder `PluginRuntime::build_linker` / equivalent is also present. If test-local linker is constructed without production-linker exercise, the closure is paper-fix at a refined layer. Re-open the finding.

**Escalation recommendation:** 5th cascade recurrence indicates this anti-pattern is systemic enough to warrant explicit codification as standing dispatch language in the adversary agent prompt — not just a per-burst note. Orchestrator should add to adversary agent prompt: "When verifying closure of any 'test exercises host callback through Component Model dispatch' finding: (1) grep test body for `Linker::new` — if present and production linker not also used, REOPEN; (2) verify sanity-revert targets production code (host_functions.rs), not test-local closure."

**Routing:** session-reviewer at cycle-close adjudication. NOT added to policies.yaml this burst per codification queue routing discipline.

---

## Policy Verification Summary

| Policy | Verdict | Notes |
|--------|---------|-------|
| POL-14 (BC promotion at merge) | PASS | No spec amendments this adversary pass |
| POL-15 (boot-step gate ordering) | PASS | `plugin_load_step_with_audit` before `step7_init_storage` VERIFIED (unchanged from pass-4) |
| POL-18 (required-features test gate) | PASS | No new test blocks added by adversary |
| TD-VSDD-053 (single-commit-per-burst) | PASS | D-555 is 60th consecutive single-commit |
| TD-VSDD-059 (paper-fix detection) | FIRING | F-PASS5-HIGH-001 — 5th paper-fix recurrence at progressively refined layer (test-local Linker::new vs production PluginRuntime::build_linker) |
| TD-VSDD-060 (sibling-sweep on count changes) | PASS | F-PASS4-MED-001 closure verified; story body swept at all 4 sites |
| BC-5.39.001 (3-CLEAN protocol) | 0/3 | BLOCKED; streak does not advance |

---

## Trajectory Analysis

### 5-Pass Arc

| Pass | Total | CRIT | HIGH | MED | LOW | Streak | Fix-Burst |
|------|-------|------|------|-----|-----|--------|-----------|
| impl-pass-1 | 18 | 3 | 6 | 7 | 2 | 0/3→0/3 | fix-burst-impl-1 CLOSED 18/18 (D-548) |
| impl-pass-2 | 12 | 2 | 3 | 6 | 1 | 0/3→0/3 | fix-burst-impl-2 CLOSED 12/12 (D-550) |
| impl-pass-3 | 6 | 3 | 1 | 2 | 0 | 0/3→0/3 | fix-burst-impl-3 CLOSED 6/6 (D-552) |
| impl-pass-4 | 2 | 0 | 1 | 1 | 0 | 0/3→0/3 | fix-burst-impl-4 CLOSED 2/2 (D-554) |
| impl-pass-5 | 3 | 0 | 1 | 0 | 2 | 0/3→0/3 | fix-burst-impl-5 NEXT |

**Total findings closed (passes 1-4):** 18+12+6+2 = 38 CLOSED.
**Open (pass-5):** 3 findings (1 HIGH + 2 LOW + 1 [process-gap] OBS).
**Trajectory shorthand:** 18→12→6→2→3

### Severity-Weighted Assessment

Severity-weighted, pass-5 (3 findings: 0C+1H+0M+2L) is lower in total weight than pass-4 (2 findings: 0C+1H+1M+0L): pass-4 had 1H+1M, pass-5 has 1H+2L. The only CRIT-class and MED-class patterns have been genuinely remediated. The persistence of the HIGH is exclusively the test evidence layer for the same root finding.

**Production code is verified correct** across all 5 passes. The adversary has not found a production bug in passes 3-5. The remaining cascade is a test-discipline issue, not a production-code issue.

**Magnitude apparent increase (2→3):** Pass-5 introduces 2 LOW findings (STORY-INDEX wording + story field asymmetry) alongside the 1 HIGH (test-local-linker paper-fix). These 2 LOWs were latent from fix-burst-impl-4 closure; the LOW count increase does not represent regression in production correctness.

---

## Next-Pass Dispatch Template

### fix-burst-impl-5 — Implementer (PRIMARY) + state-manager (LOWs)

**Implementer task (HIGH closure — Route B preferred):**

Route B: `PluginRuntime::build_linker` direct invocation
1. Replace `Linker::<HostState>::new(&engine)` in the existing dispatch test with `PluginRuntime::build_linker(&engine)`
2. Remove the test-local `host.http-request` callback registration (lines 1565-1618)
3. Wire `HostState` with necessary dependencies for the production callback to execute (may require an allowlisted URL in the http_allow_list)
4. Use `linker.get_func()` to extract `host.http-request` OR call through WAT export that triggers the import
5. Call `.call()` with synthesized `Val` params
6. Assert returned `Val::U16(...)` or `Val::Record(...)` status matches expected

**MANDATORY sanity-revert before declaring closure:**
- Change `host_functions.rs:452`: `Val::U16(response.status)` → `Val::U32(u32::from(response.status))`
- Run the test: MUST FAIL with wasmtime type-mismatch trap
- Revert immediately — confirm the fix is load-bearing

**State-manager task (LOW-001 closure — cosmetic wording):**
- Update STORY-INDEX.md annotation for S-PLUGIN-PREREQ-D to correctly attribute the story body sweep to fix-burst-impl-4 (D-554), not fix-burst-impl-3

**Story-writer task (LOW-002 closure — field asymmetry):**
- Remove `event_type` from the Fields column of `plugin_log_level_unrecognized` row in §Structured Event Catalog Additions
- Updated Fields: `plugin_id, received_name` (matching BC-2.16.002 v1.17 row 32 schema + 12 sibling rows)
- Bump story version v1.34 → v1.35 (or v1.34.1 if patch notation is preferred)
- Update STORY-INDEX row to reflect new story version

**After fix-burst-impl-5:** Dispatch adversary impl-pass-6. Target: 0/3 → 1/3 streak advance.

**Adversary impl-pass-6 carry-forward:**
- Re-verify F-PASS5-HIGH-001 closure: does the test use `PluginRuntime::build_linker`? Does the sanity-revert at `host_functions.rs:452` cause the test to fail?
- Re-verify F-PASS5-LOW-001 closure: STORY-INDEX annotation correctly attributes burst source
- Re-verify F-PASS5-LOW-002 closure: `plugin_log_level_unrecognized` Fields = `plugin_id, received_name` (no `event_type`)
- Apply PG-IMPL-LP5-001 production-linker-vs-test-linker boundary check to all new tests
- Scan for net-new findings introduced by fix-burst-impl-5
