---
pass: 13
story: S-PLUGIN-PREREQ-B
head_sha: c72702cc
base_sha: 90d7c80f
factory_sha_at_pass: fc3d8fb9
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 1M / 1L / 2O (+ 1 process-gap)
adversary_run_date: 2026-05-11
novelty_score: 4/4 = 1.0
---

# Adversarial Review — LOCAL Pass 13 (S-PLUGIN-PREREQ-B)

## Executive Summary

Fix-burst-12 closures verified per TD-VSDD-059/060/091. All three actionable closures CLEAN and load-bearing (F-LP12-MED-001 execute_step tests assert exact event_type + step_name + detail; F-LP12-LOW-001 BC v1.5 pin removed sibling-sweep zero; F-LP12-LOW-002 lessons.md exists at canonical cycles/<cycle>/lessons.md path referenced by state-manager and orchestrator agent prompts).

However: fix-burst-12's "14/14 catalog coverage" claim is OVERSTATED. Fresh-context audit reveals 5 catalog rows (3, 7, 8, 9, 10 = auth_initial_failed exec + auth_refresh_*) lack positive log-buffer assertions. They have path-coverage tests (call counts + error results) but NOT contract-surface tests for event_type strings + field schemas.

This is the 4th occurrence of catalog-amendment-without-test-anchoring (F-LP9-MED-001 → F-LP11-MED-001+MED-002 → F-LP12-MED-001 → F-LP13-MED-001). Each prior fix-burst closed the specific rows it knew about; rows 3/7/8/9/10 pre-date the v1.8 amendment and were never test-anchored.

Pass-13 also surfaces F-LP13-LOW-001 [process-gap]: PG-LP11-001 SOP Layer 1 (implementer self-check) is unwired in the implementer.md engine prompt. Only Layer 3 (adversary) actively enforces. Layers 1, 2 are paper; Layer 4 deferred to TD-VSDD-093.

Streak does NOT advance. Pass-13 is 0/3.

## Part A — Closure Verification of fix-burst-12

### F-LP12-MED-001 (execute_step trio) — CLEAN, load-bearing

3 tests at pipeline.rs:1054-1336 invoke PipelineExecutor::execute_step directly with set_default() tracing-subscriber and assert exact event_type + step_name + detail fields. Test 1 also asserts EMPTY variant is NOT emitted (negative assertion). Test 3 enforces wiremock .expect(0). Load-bearing: refactor removing step_name, merging Ok arms, or removing detail field would FAIL one or more tests.

### F-LP12-LOW-001 (BC v1.5 pin removal) — CLEAN

pipeline.rs:461 reads `(BC-2.16.002 — see Structured Event Catalog)`. Sibling sweep `grep -rn 'BC-2.16.002 v[0-9]' crates/prism-spec-engine` → 0 matches.

### F-LP12-LOW-002 (cycle lessons file) — CLEAN, durably referenced

cycles/wave-4-operations/lessons.md exists (~45 lines). state-manager.md:94/136 and orchestrator.md:408 cite cycles/<cycle>/lessons.md as canonical write target. Referenced from STATE D-421, tech-debt-register, story changelog, STORY-INDEX.

### TD-VSDD-093 — WELL-SCOPED

Filed at P3 with appropriate scope/target.

### 14/14 Catalog Coverage Claim — CHALLENGED

Fix-burst-12 report at fix-burst-12.md:32 claims "11/14 → 14/14 (100%)." False:
- Row 3 (auth_initial_failed exec): pipeline_oauth_retry.rs:284 asserts error variant + call count; NO buffer assert on event_type string
- Rows 7-10 (auth_refresh_triggered/succeeded/failed/double_401): pipeline_oauth_retry.rs:75/158 etc assert state (calls/records/Err) but NEVER capture or assert tracing buffer
- `Grep('contains("auth_refresh', tests/pipeline_oauth_retry.rs)` → ZERO buffer matches

Genuine catalog coverage: 9/14 (the 11 fix-burst-12 claimed minus rows 3+7+8+9+10 plus the 3 execute_step rows fix-burst-12 added = actually 14-5=9 verifiable + 5 missing = 9/14 buffer-asserted).

## Part B — New Dimension Findings

### F-LP13-MED-001 — BC v1.8 catalog rows 3, 7, 8, 9, 10 lack positive log-buffer assertions

- Severity: MEDIUM | Confidence: HIGH | Dimension: P12-B+D continuation (auth_refresh_* + execute auth_initial_failed)
- Category: spec-surface paper-fix (TD-VSDD-059)
- Evidence: pipeline.rs lines 166, 630, 641, 651, 683 emit structured events. Test grep shows ZERO buffer assertions on the event_type strings. Existing tests use path-coverage only.
- Why load-bearing now: BC v1.8 explicitly enumerates these as audit-signal contract surface. Refactor removing step_name from auth_refresh_triggered would NOT fail any test.
- Pattern recurrence: 4 (F-LP9/11/12/13). Each fix-burst closed the rows it knew about; pre-v1.8 rows remained unanchored.
- Recommendation: Add 5 unit tests in pipeline.rs #[cfg(test)] for rows 3, 7, 8, 9, 10. ~150 lines. Genuine 14/14 anchoring.

### F-LP13-LOW-001 [process-gap] — Lesson 1 Layer 1 not wired in implementer prompt

- Severity: LOW | Confidence: HIGH | Dimension: P13-K (extension stability)
- Evidence: lessons.md declares 4 enforcement layers. Grep on implementer.md / state-manager.md engine prompts shows ZERO citations of lessons.md or "Structured Event Catalog" discipline. Only Layer 3 (adversary) actively enforces.
- Net: 1 wired + 1 deferred (TD-093 lefthook) + 2 paper.
- Recommendation: Either (a) update lessons.md "Enforcement layers" to honestly state Layer 1+2 paper, Layer 3 sole-enforcement until TD-093 lands, OR (b) file engine-level TD to extend implementer.md/state-manager.md prompts to cite lessons.md.
- [process-gap] first occurrence; recurrence count 1.

## Observations (non-blocking)

### OBS-LP13-001 — Interpolator not re-exported at crate root

lib.rs:67 exports InterpolationContext + InterpolationError but not Interpolator. Likely intentional; minor inconsistency. Bundle into PREREQ-C public API audit.

### OBS-LP13-002 — BC version stability signal

BC v1.8 NOT amended in fix-burst-12. First non-amendment burst since fix-burst-1. Positive convergence signal — if F-LP13-MED-001 closure adds 5 tests WITHOUT amending BC, stability axis truly converges.

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP13-MED-001 | MEDIUM | P12-B+D continuation | spec-surface paper-fix | BC v1.8 rows 3/7/8/9/10 lack positive log-buffer assertions; "14/14" claim overstated | YES |
| F-LP13-LOW-001 | LOW + [process-gap] | P13-K | SOP enforcement gap | Lesson 1 Layer 1 (impl self-check) not wired into implementer.md; Layer 3 (adversary) sole load-bearing | YES (engine-scope) |
| OBS-LP13-001 | OBS | P13-E | public API freshness | Interpolator not re-exported at crate root | NO |
| OBS-LP13-002 | OBS | P13-K | BC stability signal | v1.8 stable across fix-burst-12 | NO |

## Process-Gap Findings

F-LP13-LOW-001 [process-gap] — Lesson 1 Layer 1 unwired in implementer.md/state-manager.md engine prompts. First occurrence; recurrence count 1. If future pass surfaces SECOND case of "codified-lesson layer-1-unwired", escalates to MEDIUM with pattern flag.

## Recommendations

### Fix-burst-13 scope (REQUIRED)

1. F-LP13-MED-001: Add 5 unit tests in pipeline.rs #[cfg(test)] for catalog rows 3 (auth_initial_failed execute), 7 (auth_refresh_triggered), 8 (auth_refresh_succeeded), 9 (auth_refresh_failed), 10 (auth_refresh_double_401). Each uses setup_log_capture helper + asserts contains("<event_type>") + contains(step_name) + contains("detail") where applicable. ~150 lines; story red_gate_tests 59 → 64.

2. F-LP13-LOW-001 [process-gap]: Honestly update lessons.md to document enforcement reality (Layer 1+2 paper, Layer 3 active, Layer 4 deferred). DO NOT claim layers wired that aren't.

### Deferred
- OBS-LP13-001..002: non-blocking.

After fix-burst-13, dispatch pass-14. Target: streak 0/3 → 1/3.

## Novelty Self-Check

4/4 = 1.0. All findings substantive.

**Verdict: BLOCKED-soft. Streak 0/3. Fix-burst-13 required for genuine 14/14 + honest enforcement-layer documentation.**
