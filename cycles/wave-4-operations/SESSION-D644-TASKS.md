---
document_type: session-tasks
version: "1.1"
status: active
related_burst: D-644
predecessor_state: D-643
predecessor_session_tasks: SESSION-D580-TASKS.md (cascade pass-1 through pass-5 era)
timestamp: 2026-05-16T22:30:00Z
---

# Session Task List — D-644 Durable Pre-/Clear Snapshot

This file persists the task list and full cascade state from the session covering D-580 through D-643 (cascade pass-6 through pass-36; ~150 consecutive single-commit bursts; ~12.5M tokens consumed across pass-6 through pass-36 fresh-context cycle).

**Intended audience:** orchestrator at next session start AFTER /clear. Read alongside:
- `.factory/STATE.md` v7.331 (this burst bumps; the §RESUME PROTOCOL section + `current_step` + `prereq_e_adversary_streak` fields)
- `.factory/SESSION-HANDOFF.md` v7.331 (this burst bumps; §POST-D644 DURABLE RESUME SNAPSHOT section added)
- `.factory/cycles/wave-4-operations/S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md` (full cascade history)
- `.factory/cycles/wave-4-operations/SESSION-D580-TASKS.md` (prior session task list, pass-1 through pass-5 era)

## Cascade State Summary (as of D-644)

**Cascade progress:**
- **36 adversary passes** completed (pass-1 through pass-36)
- **27 fix-bursts closed** (FB1 through FB27)
- **FB28 PENDING** — 3 MED findings from pass-36 awaiting dispatch
- **150 consecutive single-commit bursts** TD-VSDD-053 stable (this is the 150th)
- **8 CLEAN passes** of cascade: pass-9, pass-19, pass-23, pass-25, pass-26, pass-29, pass-30, pass-35
- **6 streak resets** (after CLEAN passes 9, 19, 23, 26, 30, 35 — only pass-25→pass-26 successfully advanced 1/3 → 2/3)
- **Current streak:** 0/3 (reset by pass-36)

**Trajectory novel-finding count:** 14→9→8→9→10→10→8→4→0→3→1→1→3→1→3→1→1→1→1→0→2→1→1→0→0→2→1→1→1→0→1→1→1→1→0→3

**Cascade trajectory shorthand:** Documented at length in `.factory/STATE.md` frontmatter `pass_trajectory` field.

## Task Status Table

| # | Status | Description |
|---|--------|-------------|
| Prior | DONE | All FB6 through FB27 closures (see SESSION-D580-TASKS.md for FB1-FB5 era) |
| 82 | **DONE** | **PREREQ-E fix-burst-28 CLOSED** — D-645 combined-burst closed all 3 MED findings; story v1.13; STORY-INDEX v2.117; 151st consecutive single-commit |
| 83 | **READY-FOR-DISPATCH** | PREREQ-E pass-37 (first of NEW 3-CLEAN sequence — 9th attempt; streak 0/3) |

## §FB28 Closure Note (D-645 COMPLETE)

**All 3 in-scope findings closed in combined-burst D-645 (2026-05-16). 151st consecutive single-commit.**

| Finding | Agent | Status | Notes |
|---------|-------|--------|-------|
| F-LP36-MED-001 | product-owner | CLOSED | AC-9 test name canonicalized to `_003_` convention |
| F-LP36-MED-002 | product-owner | CLOSED | Red Gate Tests 6+7 expanded 4-sensor scope Option A; `red_gate_tests:` count 8→11 |
| F-LP36-MED-003 | state-manager | CLOSED | STORY-INDEX col 3 updated; STORY-INDEX v2.116→v2.117 |

**PO-caught observations (not new findings):**
- Task-spec in this file referenced `_003_` naming for the Cyberint/Claroty/Armis rows under Test 7 (`F-LP36-MED-001` specification). Correct namespace is `_002_` per Test 7 convention in the story. PO deferred to file authority (story is canonical).
- `red_gate_tests:` frontmatter count needed sibling-bump 8→11 alongside Red Gate table expansion. Applied in same burst (PO TD-VSDD-060 sibling-catch).

**TD-VSDD-060 sweep (state-manager):** ADR-027 already has SS-07 (prism-query) in `subsystems_affected`. No other forward-prop sites found. All other hits are historical narrative.

**Next action:** Dispatch adversary spec pass-37 (task 83 READY-FOR-DISPATCH).

---

## FB28 Detailed Closure Specification (archived — DONE)

**3 in-scope MEDIUM findings from pass-36 awaiting closure:**

### F-LP36-MED-001 — AC-9 vs Red Gate Test 8 test-name drift
**Routing:** product-owner
**Files:** `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md`
**Sites:**
- Line 239 AC-9: `test_BC_2_16_012_write_tool_invalidation_runtime_register` (missing `_003_` segment)
- Line 273 Red Gate Test 8: `test_BC_2_16_012_003_write_tool_invalidation_runtime_register` (canonical with `_003_`)
**Fix:** Canonicalize AC-9 test name to Red Gate convention `_003_`. Single-line edit in same file. Story v1.12 → v1.13.

### F-LP36-MED-002 — AC-8 vs Red Gate Tests 6+7 coverage gap
**Routing:** product-owner (requires Option A vs B adjudication)
**Files:** Same story file lines 235 (AC-8), 269 (Red Gate 6), 271 (Red Gate 7)
**Issue:** AC-8 prescribes test covering 4 sensors + novel name; Red Gate has only CrowdStrike-only + novel-name (no Cyberint/Claroty/Armis)
**Options:**
- **Option A:** Expand Red Gate Tests 6+7 to cover all 4 built-in sensors. Adds new test rows for Cyberint/Claroty/Armis.
- **Option B:** Decompose AC-8 into AC-8a (CrowdStrike per Red Gate 7) + AC-8b (novel-name per Red Gate 6). Add 3 more Red Gate tests for Cyberint/Claroty/Armis if 4-sensor breadth is intended.
- **Production-grade default recommendation:** Option A (expand Red Gate to match AC-8's prescribed 4-sensor scope, preserving AC-8 as written)

### F-LP36-MED-003 — Story crates_touched vs STORY-INDEX column drift
**Routing:** state-manager (mechanical column fix + STORY-INDEX bump)
**Files:** `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md`
**Site:** Line 395 column 3: `prism-sensors,prism-spec-engine` (missing `prism-query`)
**Fix:** Add `prism-query` to column 3. Bump STORY-INDEX v2.116 → v2.117 with §Changelog row.

## FB28 Dispatch Recommendation

Single combined-burst (D-629/D-631/D-639/D-640/D-641 precedent):
- PO: F-LP36-MED-001 (single-line test-name fix) + F-LP36-MED-002 (Option A expansion of Red Gate Tests 6+7)
- state-manager: F-LP36-MED-003 (STORY-INDEX column fix) + STATE/HANDOFF/cycle-snapshot/tasks closure

Story v1.12 → v1.13 expected. STORY-INDEX v2.116 → v2.117 expected.

## Strategic Options for Next Session

The cascade has demonstrated 5 prior "first CLEAN → reset" patterns (passes 9, 19, 23, 26, 30, 35 all reset). Only pass-25→pass-26 advanced 1/3 → 2/3. After FB28 closure, the next pass (pass-37) starts the 9th attempt at 3-CLEAN sequence. Strategic options:

### Option 1 — Continue Cascade (production-grade default)
Dispatch FB28 + pass-37. Per BC-5.39.001 + CLAUDE.md Canonical Principle. Expected: ~750k-1.5M tokens to potentially reach 3-CLEAN (assuming pattern of 75% reset rate continues).

### Option 2 — Codify POL-29 mid-cycle then continue
Before FB28, codify POL-29 (FB-introduces-new-defects discipline; comprehensive same-file/cross-file sweep on every fix-burst). This addresses the root cause of the recurring sibling-sweep gap pattern. Then dispatch FB28 with explicit POL-29 enforcement in dispatch prompt. May break the reset pattern.

### Option 3 — Accept Current Spec + Human Architect Review
Pause cascade, dispatch architect for comprehensive human-style review of the spec package, then make architect-judgment call on whether the residual MEDIUM-grade findings warrant continued cascade or graduated approval. Bypasses BC-5.39.001 strict 3-CLEAN protocol; requires explicit user authorization (user_directive_persistent in STATE.md mandates "No pragmatic convergence").

### Option 4 — Pause Cascade + Graduate to Phase 3 Implementation
Accept current spec quality (8 CLEAN passes is unusual statistical evidence of quality). Phase 1d → Phase 2 transition. Dispatch story-writer to begin per-story-delivery cycle. Resume cascade later if implementation surfaces spec gaps.

**Default per "continue cascade" standing directive:** Option 1. User should signal explicit choice if alternative is preferred.

## Standing DO-NOT Directives (carry-forward, all intact)

- DO NOT push `factory-artifacts` to remote (orchestrator policy: local-only; 150+ commit divergence is expected correct state)
- DO NOT use `--no-verify` on any git command (TD-FACTORY-HOOK-BYPASS-001 P0)
- DO NOT add Claude attribution to commits (user explicit directive for prism)
- DO NOT dispatch PLUGIN-MIGRATION-001-A/B/C/D before PREREQ-E Phase 1d converges (3-CLEAN) and implementation lands
- DO NOT add entries to tech-debt-register without explicit human direction + concrete future dependency + specific story anchor (Canonical Principle Rule 3)
- DO NOT introduce the retired two-commit Stage-1/Stage-2/backfill chain (TD-VSDD-053; single-commit-per-burst only)
- DO NOT bypass git hooks or use `--no-verify` (POL-3)
- DO NOT commit files using Python/sed/echo bypass for .factory/ mutations (TD-FACTORY-HOOK-BYPASS-001; Edit/Write tools only)
- DO NOT run adversary passes on S-PLUGIN-PREREQ-D spec (closed; 43 passes converged 2026-05-14)
- DO NOT clean up sibling worktrees (S-3.09 + S-PLUGIN-PREREQ-B + S-PLUGIN-PREREQ-C + W3-FIX-S307-001 remain by design)
- DO NOT directly edit policies.yaml without session-reviewer codification workflow at cycle-close
- DO NOT run PREREQ-E implementation TDD before Phase 1d 3-CLEAN spec convergence
- DO NOT declare convergence without meeting BC-5.39.001 (3 consecutive CLEAN passes required)
- DO NOT merge to develop without explicit user authorization (Standing Rule — user-auth-required-for-merges)

## Pinned Artifact Versions (PREREQ-E 19-artifact set)

| Artifact | Version |
|----------|---------|
| Story | v1.13 (F-LP36-MED-001+002 closed D-645) |
| BC-2.01.016 | v1.5 (modified 2026-05-16) |
| BC-2.16.011 | v1.6 (modified 2026-05-16) |
| BC-2.16.012 | v1.15 (modified 2026-05-16) |
| BC-2.16.002 | v1.20 (catalog row 33 + bullet `(v1.20)`) |
| ADR-026 | v1.12 (D7 pin propagation v1.10 throughout downstream) |
| ADR-027 | v1.6 (D3 dual-file enumeration + SS-07 in subsystems_affected) |
| VP-153 | v0.5 |
| VP-154 | v0.6 |
| VP-155 | v0.5 |
| VP-156 | v0.8 (4 D7 pins at v1.10) |
| HS-PREREQ-E-001 | v1.3 (frontmatter verification_properties: [VP-153]) |
| HS-PREREQ-E-002 | v1.1 (verification_properties: [VP-154, VP-155]) |
| HS-PREREQ-E-003 | v1.5 (verification_properties: [VP-156]; HS-003-04/05 footers cite VP-156) |
| error-taxonomy | v1.30 (E-PIPELINE-001 row at v1.20 pin; E-SPEC-008 RETIRED; E-SPEC-012/013/014 + E-PLUGIN-012/020 active) |
| ARCH-INDEX | v2.55 |
| VP-INDEX | v1.47 (Total 156, P0=122, P1=34) |
| STORY-INDEX | v2.117 (F-LP36-MED-003 CLOSED D-645; prism-query added to PREREQ-E col 3) |
| BC-INDEX | v4.93 |
| verification-architecture | v1.37 (P33 sub-node added; arithmetic synced 156/122/34) |
| verification-coverage-matrix | v1.34 (totals synced 156/122/34) |

## Resume Reading Order (Next Session After /Clear)

1. **`.factory/STATE.md`** (v7.331) — current_step + prereq_e_adversary_streak + RESUME PROTOCOL section
2. **`.factory/SESSION-HANDOFF.md`** (v7.331) — §POST-D644 DURABLE RESUME SNAPSHOT section
3. **`.factory/cycles/wave-4-operations/SESSION-D644-TASKS.md`** — this file (task list + FB28 spec + strategic options)
4. **`.factory/cycles/wave-4-operations/S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md`** — full cascade history through D-643
5. **`.factory/cycles/wave-4-operations/SESSION-D580-TASKS.md`** — prior session task list (pass-1 through pass-5 era; D-580 precedent)
6. **`.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-{1..36}.md`** — per-pass finding context if needed (36 files total)

## Session Continuation Behavior

At next session start after /clear:
1. Orchestrator MUST read STATE.md → SESSION-HANDOFF.md → SESSION-D644-TASKS.md in that order
2. Orchestrator MUST verify SHA chain integrity: HEAD should be D-644 with predecessor D-643 `1f205b69`
3. Orchestrator MUST verify TD-VSDD-053 stable (150+ consecutive single-commit bursts; no backfill/Stage-1/2 in chain)
4. Orchestrator MUST present Strategic Options 1-4 to user and await explicit choice before dispatching FB28 or pass-37
5. Per user_directive_persistent "No pragmatic convergence. Fix all issues before build." — Option 1 is the default if user does not signal otherwise
