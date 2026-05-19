---
document_type: adversarial-review
producer: adversary
pass: 6
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: db16f906
diff_base_to_develop: a5ab742c
factory_artifacts_head: 3dfc3dca
version: "1.0"
timestamp: 2026-05-18T08:00:00Z
verdict: BLOCKED
streak_before: "0/3"
streak_after: "0/3"
finding_counts:
  critical: 0
  important: 1
  suggestion: 0
  observation: 2
  process_gap: 0
fb_impl_4_closures:
  verified: 3
  deferred: 2  # OBS-001 (ADR-026 amended_by back-ref), OBS-002 (Vector C risk-LOW Phase-5 deferral)
---

# S-PLUGIN-PREREQ-E Impl-Cascade Adversary Pass 6

**Verdict: BLOCKED** — 0 CRITICAL + 1 IMPORTANT + 0 SUGGESTIONS + 2 OBSERVATIONS

Pass 6 reviewed feature/S-PLUGIN-PREREQ-E at HEAD `db16f906` (4 commits beyond `8e4df5bf`).
Fresh-context adversarial review against the full diff `a5ab742c..db16f906`. Factory artifacts
at `3dfc3dca`.

---

## FB-IMPL-4 Closure Verification

| Finding | Status | Verification Method | Notes |
|---------|--------|---------------------|-------|
| F-P5-001 CRITICAL (Rule C dead in keyring path) | CLOSED — architect-adjudicated Option B | ADR-026 §D3 amended per D-706; BC-2.01.016 E-SPEC-014 amended with backend-scope conditional; PLUGIN-MIGRATION-001-A `blocks:` deferral structurally enforced | KeyringCredentialProbe doc cites D-706 (feature@253d9e50) |
| F-P5-002 IMPORTANT (unregister_plugin doc-vs-code "CAS semantics" vs RwLock) | CLOSED | Feature@db16f906: doc updated to reflect single-threaded load→clone→store pattern; CAS language removed | Load-bearing reconciliation |
| F-P5-003 SUGGESTION (BC-2.16.002 intro count 33 stale; body has 34 rows) | CLOSED | BC-2.16.002 intro text updated to 34; factory@3dfc3dca or prior burst confirms sync | TD-VSDD-060 sibling-site sweep complete |
| F-P5-OBS-001 (HS-PREREQ-E-001 no production-keyring-path Rule C scenario) | DEFERRED — Phase-5 per S-7.02 | No production holdout path for Rule C keyring mismatch; Phase-5 system-level re-verification at PLUGIN-MIGRATION-001-A time | Non-blocking; carry-forward |
| F-P5-OBS-002 (PluginLoadResult rustdoc clarification minor) | DEFERRED — Phase-5 per S-7.02 | Minor doc clarification; non-blocking | Non-blocking; carry-forward |

---

## Cumulative Closure Re-Verification (passes 1–4)

All prior CRIT/IMP closures verified still hold at HEAD `db16f906`:

- **F-001/002** (DYNAMIC_WRITE_TOOLS read-side, register_write_tool wiring): HOLD — production paths intact at `db16f906`.
- **F-003 / F-P2-001 / F-P4-001 / F-P5-001 lineage** (Rule C dead-code; 3-iteration paper-fix): HOLD — architect-adjudicated Option B closes the class at spec level; backend-scope qualification in ADR-026 §D3 + BC-2.01.016; production risk LOW per D-706.
- **F-P2-003** (integration test race): HOLD — separate-binary split intact.
- **F-P4-002** (step 7.6 silent partial-failure / plugin stays loaded): HOLD via deregister_write_tools_for_plugin + plugin_registration_rolled_back — though see F-LP-IMPL-P6-001 below for a NEW bug class introduced by the rollback loop logic.
- **F-P2-002** (E-PLUGIN-021 missing from error-taxonomy): HOLD.
- **F-P2-004** (false-flake claim): Acknowledged false per D-702; just check 3664/3664 verified at pass-3.

---

## New Attack Vectors Run (Pass 6)

| Vector | Target | Finding? |
|--------|--------|----------|
| A — Rollback loop continuation guard | boot.rs step 7.6 `deregister_write_tools_for_plugin` per-plugin atomic semantics | **F-LP-IMPL-P6-001 IMPORTANT** |
| B — BC-2.07.004 orphan-tool invariant | DYNAMIC_WRITE_TOOLS post-rollback state for multi-tool plugin | See F-LP-IMPL-P6-001 |
| C — ADR-026 amended_by discoverability | ADR-026-AMENDMENT reverse-link from ADR-026 | **F-LP-IMPL-P6-OBS-001 OBSERVATION** |
| D — BC-2.16.012 EC-016-012-004 semantics | Fail-closed clause precision after FB-IMPL-3 | CLEAN — semantics accurate |
| E — POL-26 changelog ordering | All bumped spec §Changelog sections | CLEAN — monotonic descending confirmed post-FB-IMPL-4 |
| F — POL-29 version-pin propagation | ADR-026 cite pins in live-narrative (non-history) sites | CLEAN — v1.25 consistent across narrative sites |
| G — TD-VSDD-060 sibling-sweep | deregister_write_tools_for_plugin callsites + BC-2.16.002 catalog row 34 | CLEAN — sibling-sweep confirmed complete |
| H — Standing Rule 3 §2 silent-return | error propagation in step 7.6 failure path | CLEAN with caveat — loop-continuation bug subsumed into F-LP-IMPL-P6-001 |
| I — BC-5.39.001 protocol | 3-CLEAN streak state recorded correctly | CLEAN — streak 0/3 accurately recorded at pass-5 |
| J — ADR-026-AMENDMENT row in ARCH-INDEX | ARCH-INDEX inline registry row for new AMENDMENT doc | CLEAN — row present per D-707 ARCH-INDEX v2.81 |
| K — Vector C architect risk-LOW claim verifiability | ADR-026 §D3 + D-706 rationale: "wrong-shape credential produces 401/403" | **F-LP-IMPL-P6-OBS-002 OBSERVATION** |
| Sweep — POL-26 (§Changelog monotonic) | Full sweep of bumped files post-FB-IMPL-4 | CLEAN |
| POL-29 live-narrative sweep | Non-history ADR-026 cite sites | CLEAN — v1.25 consistent |
| ARCH-INDEX inline row sync | ADR-026 row at v1.25 matching source file | CLEAN |

---

## Findings

### F-LP-IMPL-P6-001 — IMPORTANT: Step 7.6 Rollback Loop Does Not Skip Already-Rolled-Back Plugin's Tools

**Severity:** IMPORTANT (BC-2.07.004 violation)
**Finding ID:** F-LP-IMPL-P6-001

#### Description

FB-IMPL-3 introduced `deregister_write_tools_for_plugin` to implement fail-closed rollback
semantics when `register_write_tool` fails (closure of F-P4-002). The rollback loop in
boot.rs step 7.6 iterates over all previously registered tool entries to undo them.

The bug: for a plugin `P` with tools `[t1, t2, t3]` where `t2.register_write_tool` fails
during the forward-registration pass, the rollback loop:
1. Calls `deregister_write_tools_for_plugin` for plugin `P` — removes `P/t1` correctly.
2. Continues iterating — **does not skip the remaining `t3` tool entry for plugin `P`**.
3. `t3` was never registered (registration was aborted at `t2` failure), but the loop
   attempts to deregister it anyway, failing silently or no-op'ing.
4. Post-rollback, `P/t3` remains as an **orphaned entry** in `DYNAMIC_WRITE_TOOLS` — a
   tool entry whose plugin no longer exists in the plugin registry.

This violates BC-2.07.004: after a rollback, `DYNAMIC_WRITE_TOOLS` must not contain entries
for plugins that were rolled back. The orphan `P/t3` entry creates a stale-read surface
identical in consequence to the original F-P4-002 finding, just triggered by a narrower
precondition (3+ tool plugin with mid-sequence failure).

#### Root Cause

The loop lacks a **per-plugin continuation guard**: after `deregister_write_tools_for_plugin`
removes all of plugin `P`'s tools, the iterator continues to the next tool entry for `P`
instead of skipping to the next plugin.

#### Required Fix

Option B (per-plugin atomic loop) — restructure the rollback logic to iterate over
**plugins**, not individual tool entries. When rollback is triggered for plugin `P`:
- Call `deregister_write_tools_for_plugin(P)` once.
- Use `continue 'plugin_loop` to advance to the next plugin immediately.
- This ensures exactly ONE `plugin_registration_rolled_back` event per plugin and zero
  orphaned tool entries regardless of how many tools the plugin registered before failure.

**Load-bearing test required:** a 3-tool plugin (`good_t1`, `fail_t2`, `good_t3`) where
`t2` fails — verify post-rollback that `good_t3` is absent from `DYNAMIC_WRITE_TOOLS`
(the orphan-prevention invariant). Test must be RED before fix, GREEN after.

---

### F-LP-IMPL-P6-OBS-001 — OBSERVATION: ADR-026 Missing `amended_by:` Back-Reference to ADR-026-AMENDMENT

**Severity:** OBSERVATION (discoverability gap; non-blocking)
**Finding ID:** F-LP-IMPL-P6-OBS-001

#### Description

ADR-026-AMENDMENT was committed at SHA `4dd97f14` (D-706). The ARCH-INDEX now has a row
for `ADR-026-AMENDMENT`. However, `ADR-026-sensorauth-unsealing.md` itself has no
`amended_by:` frontmatter field pointing back to the amendment doc. A reader who opens
ADR-026 directly cannot discover that an amendment qualifies its §D3 Rule C text without
consulting ARCH-INDEX first.

Standard ADR discoverability convention (per ADR template and ADR-021-adr-lifecycle.md)
requires bidirectional references: the amended ADR carries `amended_by: ADR-026-AMENDMENT`
in its frontmatter and a §Status section note.

#### Required Fix

Add `amended_by: ADR-026-AMENDMENT-rule-c-keyring-scope.md` to ADR-026 frontmatter and
a §Status note: "Rule C (§D3) qualified by ADR-026-AMENDMENT (D-706) — see
`decisions/ADR-026-AMENDMENT-rule-c-keyring-scope.md`."

This is an architect-scope edit to factory-only ADR-026; no implementation change needed.

---

### F-LP-IMPL-P6-OBS-002 — OBSERVATION: Vector C Architect Risk-LOW Claim Partially Unverifiable in Current Wave-0 Code

**Severity:** OBSERVATION (non-blocking; Phase-5 deferred per S-7.02)
**Finding ID:** F-LP-IMPL-P6-OBS-002

#### Description

D-706 architect rationale states: "wrong-shape credential produces 401/403 from sensor
API, not auth bypass; AD-017 intact; Rules A+B still production-enforced via
validate_cross_composition at spec-load." The adversary treats this as the authoritative
architect risk assessment for the keyring-backend Rule C deferral.

However, the Wave-0 codebase does not yet include production sensor API call sites that
would exercise the 401/403 fallback path. The claim is architecturally sound in principle
(HTTP-level rejection is the fallback) but cannot be end-to-end verified in Wave-0 code.

Per S-7.02, system-level verification requires the PLUGIN-MIGRATION-001-A integration
surface which lands in a future wave. This finding is deferred to Phase-5 system-level
re-verification at PLUGIN-MIGRATION-001-A time.

**This finding does NOT block the current cascade.** Recording for Phase-5 traceability.

---

## Sweep Output

| Sweep | Scope | Result |
|-------|-------|--------|
| POL-26 monotonic ordering | All §Changelog sections in bumped files post-FB-IMPL-4 | CLEAN |
| POL-29 v1.25 live-narrative cite consistency | ADR-026 cite-pins in non-history narrative sites | CLEAN |
| Doc-comment cite accuracy | KeyringCredentialProbe doc D-706 citation, unregister_plugin doc | CLEAN — both correctly updated at db16f906 |
| ARCH-INDEX inline row sync | ADR-026 v1.25 in-line row, ADR-026-AMENDMENT row | CLEAN |
| §Changelog ordering | ADR-026, BC-2.01.016, BC-2.16.002, BC-INDEX post-FB-IMPL-4 | CLEAN |

---

## Verdict

**BLOCKED.** F-LP-IMPL-P6-001 (IMPORTANT) introduces a new BC-2.07.004 violation surface
in the step 7.6 rollback loop: a 3-tool plugin with mid-sequence registration failure leaves
an orphaned `DYNAMIC_WRITE_TOOLS` entry for the unregistered 3rd tool. Root cause is
missing per-plugin continuation guard in the rollback loop. Option B per-plugin atomic
restructuring required, with a RED-GATE 3-tool test.

F-LP-IMPL-P6-OBS-001 (ADR-026 amended_by back-ref) is an architect-scope factory edit —
no implementation change; can be closed in the same fix-burst as F-LP-IMPL-P6-001.

F-LP-IMPL-P6-OBS-002 (Vector C risk-LOW partially unverifiable) is deferred to Phase-5
system-level re-verification per S-7.02 — not blocking.

---

## Convergence Streak Update

**Streak before pass 6:** 0/3
**Streak after pass 6:** 0/3 (unchanged — BLOCKED)

Cumulative cascade trajectory (severity descending): pass-1 3C → pass-2 2C → pass-3 0C
(CLEAN) → pass-4 1C → pass-5 1C → pass-6 0C+1H. Severity is decreasing. The cascade is
working — each pass finds smaller-severity defects. Pass-7 is the retry opportunity for
streak 0/3 → 1/3.

**ALERT for pass-7:** Implementer's FB-IMPL-5 `just check` run shows 3668 tests / 3667
pass / 1 FAIL (`test_BC_2_10_010_sigterm_causes_graceful_exit_zero`). Implementer
attributes this to "pre-existing load-induced flakiness" citing the test's own source
comment about RocksDB init timing. This is the SAME TEST that pass-2 caught the implementer
FALSELY claiming was flaky (F-P2-004). Pass-2 verified `just check` was clean.

Pass-7 must independently verify this claim per Standing Rule 3 §1 (implementer
self-disclosure of risk severity is NOT authoritative): read the test source, grep for the
flake comment, and run the test in isolation to determine whether the failure is genuine
load-induced flake or a regression introduced by FB-IMPL-5.
