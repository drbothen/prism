# Lessons Learned — S-DEMO-QUERY-PUSHDOWN-001 (wave-5-e-demo-fidelity)

## Codification Candidates

### [recurrence] ADV-P05-HIGH-001 — Hand-fed-FQL-vs-real-path test anti-pattern: when fixing one hand-fed test, sweep ALL push-down tests across ALL crates (TD-VSDD-060)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1012
**Finding:** ADV-P05-HIGH-001 (v2.x LOCAL pass-5)
**Tags:** [recurrence] [sibling-sweep] [TD-VSDD-060] [dead-code-via-test-layer]
**Classification:** BLOCKING — recurrence of a previously closed defect class; exhaustive sweep is the required response.

**Description:**
Pass 4 caught the hand-fed-FQL + direct-`PipelineExecutor::execute`-boundary test anti-pattern for AC-CWS-002 (in `prism-bin`). The pass-4 fix correctly addressed AC-CWS-002 but did not exhaustively sweep sibling tests across other crates in `crates_touched`. Pass 5 found the same defect class at AC-EQUIV-001 in `prism-spec-engine` — the test claimed "via real materialization path" in its name but bypassed `run_materialization_pipeline`.

This is a **cross-crate sibling-sweep gap**: the pass-4 fix-burst operated within `prism-bin` and did not review the `prism-spec-engine` AC-EQUIV-001 test, which was testing the same behavioral property (result equivalence via real path) but living in a different crate.

**Root cause of recurrence:**
The pass-4 AC-CWS-002 fix-burst swept within `prism-bin` but treated the fix as crate-scoped. The AC-EQUIV-001 test in `prism-spec-engine` existed independently and was NOT reviewed as a sibling despite both tests covering the same "real materialization path" behavioral domain (BC-2.11.007 result-equivalence).

**Correct response (codified rule):**
When fixing a hand-fed-FQL / direct-`PipelineExecutor::execute`-boundary test anti-pattern, the fix-burst MUST:
1. Sweep ALL push-down tests across ALL `crates_touched` (not just the crate where the defect was found).
2. Explicitly list and disposition each test: real-path vs boundary vs unit (per the 32-test sweep pattern used in pass 5).
3. Any test whose NAME claims "via real path" / "via run_materialization_pipeline" / "via production entry point" but whose BODY does NOT call that function = P1 HIGH finding; fix in the same burst.

**Outcome:** Pass-5 fix-burst added exhaustive sibling-sweep of 32 push-down tests across 4 crates (TD-VSDD-060 compliant). All 32 dispositioned. No further misnamed real-path claims remain.

**Codification direction (for future session-reviewer / VSDD process improvement):**
- This pattern should become a standing adversary probe: after any fix involving `run_materialization_pipeline` test rewrites, adversary MUST grep for ALL tests containing `"run_materialization_pipeline"` in their name or doc-comment across the full workspace, then verify each drives the function, not just PipelineExecutor directly.
- Codification could be SAP-3 or an amendment to SAP-1/SAP-2 scope (TD-VSDD-060 sweep extension).

---

### [recurrence][codification-candidate] Test-strength gap class — assertion weaker than AC/doc claim (passes 4/5/6 pattern)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1013
**Findings:** ADV-P04-HIGH-002 (pass 4 — AC-CWS-002), ADV-P05-HIGH-001 (pass 5 — AC-EQUIV-001), ADV-P06-MED-001 (pass 6 — AC-CWS-003)
**Tags:** [recurrence] [test-strength-gap] [wire-level-assertion] [codification-candidate] [dead-code-via-test-layer]
**Classification:** CODIFICATION CANDIDATE — three consecutive passes found the same defect class; systematic test-writing discipline gap.

**Description:**
Three consecutive adversary passes (4, 5, 6) each found a push-down AC test whose assertion was weaker than its AC/doc claim:

- **Pass 4 — AC-CWS-002:** Test claimed "time-window both-bounds via run_materialization_pipeline" but hand-fed pre-built FQL to `PipelineExecutor::execute`, bypassing the production entry point. Wire-level DTU filter-log assertion absent.
- **Pass 5 — AC-EQUIV-001:** Test claimed "result equivalence via real materialization path" but called `PipelineExecutor::execute` directly (sibling-sweep miss from the pass-4 fix). Real-path coverage absent for BC-2.11.007 subset/no-fabrication invariant.
- **Pass 6 — AC-CWS-003:** Test name and AC claimed "DTU filter-log shows absence of `created_timestamp` clause when no time filter present, and all 50 records returned." Test only asserted `!is_empty()` and non-zero count. Wire-level absence assertion absent — if production code incorrectly injected a timestamp clause, test would still pass.

In all three cases, production code was correct at discovery time. The test-strength gap meant incorrect production behavior would not have been caught by the named test.

**Common root cause:**
Test-writer and implementer wrote structural proxies (non-empty, count > 0) or boundary-only assertions instead of wire-level evidence (DTU filter-log body, specific field presence/absence, exact count matching the fixture). The proxy assertion passes when production is correct AND when production incorrectly injects/omits the named property — it is not a falsifiable test for the named AC property.

**Codification direction (for future test-writer + implementer disciplines):**

1. **Wire-level assertion requirement for push-down AC tests:** Any AC test for a push-down property MUST assert the named property via DTU wire evidence (e.g., `/dtu/filter-log` body contains or does not contain the specific clause/field), not via structural proxies.
2. **Exact fixture count assertion:** When an AC claims "all N records returned" (AC-CWS-003: 50 records), the test MUST assert `result.len() == N` not `!result.is_empty()`.
3. **Doc-claim-vs-assertion audit at fix-burst time:** When a fix-burst touches any push-down AC test, the implementer MUST audit the assertion against the AC text and test name/doc-comment before declaring the closure load-bearing. This audit should be explicitly documented in the fix-burst commit message.
4. **Adversary standing probe extension (codification candidate):** The exhaustive doc-claim-vs-assertion audit performed in pass 6 (comparing all push-down AC test assertions against their AC text) should become a standing adversary probe for all push-down story passes. Candidate for SAP-3 or SAP-2 extension.

**Outcome (pass 6):** Pass-6 fix-burst added AC-CWS-003 wire-level absence assertion (DTU `/dtu/filter-log` body checked for absence of `created_timestamp` clause; `result.len() == 50` exact count). Exhaustive doc-claim-vs-assertion audit of all push-down AC tests confirmed all others CORRECT/load-bearing after the fix.

---

### [correctness] ADV-P08-MED-001 — Push-down must over-fetch at inclusive boundaries (Ge/Le): DTU filtering must use strict-outside exclusion, never inclusive exclusion

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1014
**Finding:** ADV-P08-MED-001 (v2.x LOCAL pass-8)
**Tags:** [correctness] [push-down] [boundary-semantics] [result-equivalence] [BC-2.11.007]
**Classification:** BLOCKING — correctness defect; BC-2.11.007 result-equivalence invariant violated.

**Description:**
Pass 8 found that inclusive time predicates (`>=`/`<=`, `CompareOp::Ge`/`Le`) caused push-down to UNDER-fetch boundary records. Two root causes:

1. **DTU inclusive-boundary semantics (CrowdStrike `detections.rs` + Armis `search.rs`):** The DTU filtering functions (`device_in_time_window`, `alert_in_time_window`) used inclusive-boundary exclusion — a record with `timestamp == bound` was excluded. DataFusion applies inclusive semantics for `>=`/`<=`. This asymmetry dropped exact-boundary records when push-down was active.

2. **RFC3339 `+00:00` vs `Z` lexicographic comparison bug:** `chrono::to_rfc3339()` emits `+00:00` (ASCII `+` = 43); fixture timestamps stored as `Z` (ASCII `Z` = 90). Lexicographic comparison `+00:00` < `Z` caused DataFusion's string-based comparison to drop the boundary record even when both strings represent the same UTC instant.

**Codified rules:**

1. **DTU filtering MUST over-fetch at boundaries.** When push-down generates a time-window filter for inclusive predicates (`>=`/`<=`), the DTU clone's filtering function MUST exclude only strictly-outside records (`ts < start` or `ts > end`). Records at the exact boundary (`ts == start` or `ts == end`) MUST pass through the DTU filter. DataFusion's post-filter applies the correct inclusive semantics and narrows the result set. This is the "over-fetch-never-under-fetch" DTU invariant for push-down.

2. **RFC3339 normalization is mandatory.** Any code path that serializes timestamps for push-down into DTU fixture filtering or DataFusion string comparison MUST use `to_rfc3339_opts(SecondsFormat::Secs, true)` to produce `Z`-suffix form. `to_rfc3339()` (`+00:00` form) must NOT be used for DTU push-down filter serialization — it produces a lexicographically-wrong string that drops exact-boundary records at DataFusion's string-comparison layer.

3. **BC-2.11.007 result-equivalence requires boundary Red Gate tests.** Any story implementing time-window push-down for a new sensor MUST include at least one boundary Red Gate test that: (a) uses an exact `>=` or `<=` predicate, (b) runs via `run_materialization_pipeline` (not `PipelineExecutor::execute` directly), and (c) asserts the boundary record IS included in the result (i.e., the push-down result count matches the non-pushed count at the boundary).

**Outcome:** Fix-burst `69aafcc7` corrected both root causes. 2 new boundary Red Gate tests added (CrowdStrike + Armis). Story v2.4→v2.5 (EC-009 + `red_gate_tests` 16→18). just check 4035/4035 PASS.

---

### [correctness] RFC3339 `+00:00` vs `Z` suffix — DTU push-down timestamps must normalize to `Z` form to prevent lexicographic boundary drop

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1014
**Finding:** ADV-P08-MED-001 root cause 2 (v2.x LOCAL pass-8)
**Tags:** [correctness] [rfc3339] [timestamp-normalization] [lexicographic-comparison] [push-down]
**Classification:** BLOCKING (co-root-cause of ADV-P08-MED-001); codified as standing rule.

**Description:**
`chrono::DateTime::to_rfc3339()` produces `2024-01-01T00:00:00+00:00`. Fixture timestamps are stored as `2024-01-01T00:00:00Z`. In lexicographic string ordering, `+` (ASCII 43) < `Z` (ASCII 90). DataFusion uses string-based comparison for timestamp predicates in this push-down path. The result: a `>=` predicate with the `+00:00` form evaluates `"2024-01-01T00:00:00+00:00" >= "2024-01-01T00:00:00Z"` as `false` (since `+` < `Z`), dropping the boundary record.

Both `2024-01-01T00:00:00+00:00` and `2024-01-01T00:00:00Z` represent the identical UTC instant. The bug is purely a string-representation asymmetry.

**Codified rule:**
In all push-down timestamp serialization sites, use `chrono::SecondsFormat::Secs` with `use_z: true`:
```rust
dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)  // → "2024-01-01T00:00:00Z" (CORRECT)
// NOT:
dt.to_rfc3339()  // → "2024-01-01T00:00:00+00:00" (WRONG for string comparison)
```
This rule applies to: DTU fixture comparison sites, DataFusion predicate generation, any push-down filter string serialization in `pipeline.rs` or equivalent.

---

### [process-gap] ADV-P04-OBS-001 — Test-docstring file.rs:NNN line-pin discipline gap (TD-VSDD-091 adjacency)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1011
**Finding:** ADV-P04-OBS-001 (v2.x LOCAL pass-4)
**Tags:** [process-gap] [codification-candidate] [TD-VSDD-091-adjacency]
**Classification:** NOT blocking — process improvement candidate only.

**Description:**
Several test function doc-comments in `crates/prism-dtu-crowdstrike/` and `crates/prism-query/src/pushdown.rs` cite source-file locations as `file.rs:NNN` (line-number pins). TD-VSDD-091 (anti-volatile-pin discipline) prohibits line-number pins in narrative spec content because they decay on subsequent diffs. The rule is currently enforced by adversary pass probes for `.factory/` SPEC artifacts.

However, there is no automated enforcement path for test-docstring line-number pins inside `crates/**/*.rs` files. A test's `/// See file.rs:123` style comment is technically in-code narrative — it can decay the same way a spec pin does — but it falls outside the current TD-VSDD-091 adversary probe scope and no pre-commit lint exists for it.

**Codification direction (for future session-reviewer / factory process improvement):**
- Option A: Extend adversary SAP probe to grep `crates/**/*.rs` doc-comments for `\w+\.rs:\d+` pattern and flag as LOW findings.
- Option B: Add a clippy custom lint or a pre-commit hook regex scan for `\w+\.rs:\d+` inside `///` comment lines.
- Option C: Record as CLAUDE.md standing convention and address opportunistically during fix-bursts that touch doc-comments.

**Resolution:** Option C is the minimum bar; Option A is the right permanent fix. Codification decision deferred to session-reviewer at cycle-close. Do NOT add a new story now — this is a process/lint gap, not an unmet user-facing requirement.

**Non-blocking confirmation:** The line-pin sites do not affect test behavior or correctness. Code passes `just check 4032/4032`. This record exists only as a cycle-close codification prompt.

---

### [traceability][codification-candidate][recurrence×2] F-P08-MED-001 + F-P09-MED-001 — Dangling-AC class: code/test cites an AC identifier absent from the story source-of-truth; one-at-a-time fix enables recurrence; requires complete sweep

**Date recorded:** 2026-06-05 (first occurrence D-1018); strengthened 2026-06-06 (second occurrence D-1019)
**D-NNN anchors:** D-1018 (pass-8, AC-INDEX-CWS-001) + D-1019 (pass-9, AC-CWS-WIRE-001)
**Findings:** F-P08-MED-001 (PR-LEVEL pass-8) + F-P09-MED-001 (PR-LEVEL pass-9)
**Tags:** [traceability] [dangling-AC] [story-source-of-truth] [codification-candidate] [acceptance-criteria-count] [recurrence] [complete-sweep]
**Classification:** MEDIUM — spec completeness defect; no code change needed; test passes but AC not formally defined in story. CLASS NOW CLOSED for this story (ZERO remaining dangling ACs after v2.7 fix).

**Description:**
PR-LEVEL adversary pass 8 found 11 code and test sites citing `AC-INDEX-CWS-001` (CrowdStrike
`crowdstrike.sensor.toml` `created_timestamp` declares `options = ["INDEX"]` — required for ADR-033
T1 heuristic to recognize the column as push-down-eligible). However, story v2.5's formal acceptance
criteria list did not define `AC-INDEX-CWS-001`. The story defined `AC-INDEX-001` (the Armis parallel)
but the CrowdStrike equivalent was never formally added when the tests were authored during the v2
LOCAL cascade.

Pass-8 fix was one-at-a-time: story-writer added AC-INDEX-CWS-001 to story v2.6 without performing
an exhaustive sweep of ALL this-story AC identifiers cited in code/tests. PR-LEVEL adversary pass 9
immediately found a second dangling AC: `AC-CWS-WIRE-001` cited 18 times in
`bc_2_11_007_pushdown_test.rs` (test function `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu`
plus 17 doc-comment/assertion string citations). This is the S-7.01 partial-fix miss pattern.

Pass 9 applied the complete sweep (orchestrator-directed). After story-writer added AC-CWS-WIRE-001
to story v2.7, the full sweep confirmed ZERO remaining dangling ACs.

**Why this class evades many adversary passes:**

1. **Tests pass.** The cited AC tests exist and pass at every HEAD. No test failure signals the gap.
2. **AC count appears internally consistent.** The story's stated `acceptance_criteria_count` matches
   the number of formally defined ACs in the story text — the count is correct *by the story's own
   (understated) definition*. An adversary checking "does the count field match the count of AC
   sections?" sees no discrepancy.
3. **The behavioral requirements ARE satisfied.** The implementations are correct. The gap is purely
   a traceability formality, not a correctness defect.

These three conditions combine to make the dangling-AC class invisible to most adversary probes:
the code is correct, tests pass, and the story's counts are internally self-consistent. The gap is
only detectable by a cross-reference probe: grep code/test files for AC identifiers and verify
each cited AC ID resolves to a formally defined AC in the story's `## Acceptance Criteria` section.

**Root cause of the recurrence (pass 8→9):**
The pass-8 fix was one-at-a-time — story-writer added only the AC that pass-8 flagged
(AC-INDEX-CWS-001) without sweeping ALL this-story AC identifiers cited in code/tests. The
AC-CWS-WIRE-001 identifier was in the SAME test file as AC-INDEX-CWS-001 and was not discovered
until pass 9.

**Correct fix protocol (codified after pass 9):**
When any dangling-AC finding is closed, the fix MUST use the complete-sweep approach:
```bash
# Find ALL AC identifiers cited in this-story's code/test files:
rg 'AC-[A-Z][A-Z0-9_-]+' crates/**/*.rs | grep -oP 'AC-[A-Z][A-Z0-9_-]+' | sort -u
# For each unique AC ID: verify it appears as "### AC-ID:" in the story file.
# Any unresolved ID = MEDIUM finding (same class). Fix all in the same burst.
```

**Codification direction (cross-reference probe):**

A standing adversary probe should be added to detect this class AT PASS 1 (not pass 8-9):

```
For every PR-LEVEL adversary pass on a story:
1. grep crates/**/*.rs for all strings matching AC-[A-Z][A-Z0-9_-]+ (AC identifier pattern)
2. For each unique AC ID found, verify it appears as a defined AC section (### AC-ID: ...) in the story file
3. Any AC-ID cited in code/tests that is NOT defined in the story = MEDIUM finding (dangling-AC class)
   Exception: historical comments referencing an AC that was subsequently renamed — verify the
   renamed AC covers the same behavioral property.
4. When closing a dangling-AC finding: sweep ALL this-story AC IDs in the same burst (not one-at-a-time).
```

This probe would have caught F-P08-MED-001 + F-P09-MED-001 together at PR-LEVEL pass 1 (or even
during the LOCAL cascade) rather than at passes 8 and 9.

**Codification candidates:**
- SAP-5 standing adversary probe: dangling-AC grep cross-reference check (apply at every pass,
  not just when a finding is suspected)
- CLAUDE.md codification: add to the adversary's mandatory pre-convergence checklist
- Story-writer discipline: when adding any new AC-ID to code/tests, simultaneously add it to the
  story's formal acceptance criteria in the same commit

**Outcome:**
- Pass-8: story-writer added AC-INDEX-CWS-001 to story v2.6 (one-at-a-time). Feature HEAD
  unchanged at 1a8cc8aa. Streak RESET 2/3 → 0/3.
- Pass-9: orchestrator directed complete sweep; story-writer added AC-CWS-WIRE-001 to story v2.7
  (AC count 17→18; red_gate_tests 19→20; STORY-INDEX v2.288→v2.289). Feature HEAD unchanged at
  1a8cc8aa. Streak remains 0/3. Complete sweep confirmed ZERO remaining dangling ACs. Pass 10
  next (fresh streak; code stable 1a8cc8aa + story v2.7; need 3 strict-clean).

---

### [process-gap][codification-candidate] Strict-3-CLEAN cosmetic tail — after code convergence, fresh-context passes surface a continuing tail of single cosmetic LOWs; pre-cascade hygiene-sweep gate candidate

**Date recorded:** 2026-06-06
**D-NNN anchor:** D-1020
**Findings:** F-P11-LOW-001 (draft comments), OBS-P13-001 (vacuous assertion), OBS-P13-002 (evidence currency), F-P14-LOW-001 (volatile SHA in evidence), F-P16-LOW-001 (story line-pins)
**Tags:** [process-gap] [codification-candidate] [strict-3-clean-tail] [hygiene-sweep-gate] [PR-LEVEL-cascade-efficiency]
**Classification:** PROCESS-GAP — no correctness defects; pattern-level observation about cascade efficiency.

**Description:**
PR-LEVEL passes 10-16 for S-DEMO-QUERY-PUSHDOWN-001 demonstrate a recurring pattern
in the strict-3-CLEAN tail of the PR-LEVEL cascade:

After all substantive findings (correctness, security, spec-semantics) are closed
and CLEAN(PR-merge) is achieved and held, fresh-context adversary passes continue
surfacing single cosmetic spec/test-hygiene LOWs in each new pass. Key observations:

1. **CLEAN(PR-merge) held for ALL 16 consecutive passes.** Not one MED/HIGH/CRIT
   finding appeared in passes 10-16. The code is production-grade.

2. **CLEAN(strict) achieved only at passes 10, 12, 15** (out of 7 passes in the tail).
   Each LOW finding reset the strict streak.

3. **Each fix created new surface for the next pass.** The pass-13 evidence refresh
   (fixing OBS-P13-002) introduced a new volatile SHA pin (F-P14-LOW-001). The
   pass-14 de-pin then advanced the streak — until pass-16 found story line-pins
   that had persisted through 15 passes.

4. **All hygiene classes were eventually closed via complete sweeps**, not one-at-a-time
   fixes: dangling-AC (pass 9 complete sweep), draft-comment (pass 11 complete sweep),
   evidence-SHA (pass 14 de-pin + v2.7 anchor), story line-pins (pass 16 complete sweep).

5. **The recurring pattern**: cosmetic items not caught during LOCAL cascade or initial
   PR review accumulate silently, then drain one-at-a-time during the strict-3-CLEAN
   phase, each resetting the streak. 7 passes (10-16) were required to advance past
   5 LOWs.

**Codification candidate — Pre-PR Hygiene-Sweep Gate:**

Run a focused hygiene-sweep ONCE before the PR-LEVEL cascade begins, covering:

1. **Line-pin sweep (TD-VSDD-091):** grep story body + spec artifacts for `~\d+`
   and `file.rs:\d+` patterns; anchor to function names.
2. **Draft-comment sweep:** grep all PR diff files for "wait —", "TODO:", "FIXME:",
   "// temp", "draft" in comments; clean all occurrences.
3. **Vacuous-assertion audit:** for each e2e/integration test AC assertion, verify
   the assertion independently falsifies the named property (not just "is_not_empty"
   proxies).
4. **Volatile-SHA sweep in evidence:** grep `docs/demo-evidence/<story>/` for raw
   40-char hex SHAs; replace with stable `PR#/story-version/LOCAL-converged-SHA` refs.
5. **AC traceability sweep (SAP-5):** grep `crates/**/*.rs` for `AC-[A-Z][A-Z0-9_-]+`;
   verify each resolves to `### AC-ID:` in story file.

Running this sweep ONCE before pass-1 would drain all 5 classes found in passes 10-16
upfront, allowing the strict-3-CLEAN cascade to focus on substantive behavioral review
from the first pass.

**Implementation note:** This gate should be a standing pre-PR checklist item in the
pr-manager 9-step cycle, not a separate story. It is a process discipline, not a
code deliverable.

**Outcome (passes 10-16):**
- Pass-10: CLEAN(strict)=yes. Streak 0/3 → 1/3.
- Pass-11: F-P11-LOW-001 (draft comments). Implementer sweep ac75e84d. Streak → 0/3.
- Pass-12: CLEAN(strict)=yes. Streak 0/3 → 1/3.
- Pass-13: OBS-P13-001+002 (vacuous assert + evidence). Fixed 6583e419. Streak → 0/3.
- Pass-14: F-P14-LOW-001 (volatile SHA). Demo-recorder de-pin 6835e4fa. Streak → 0/3.
- Pass-15: CLEAN(strict)=yes. Streak 0/3 → 1/3.
- Pass-16: F-P16-LOW-001 (story line-pins). Story-writer sweep v2.7→v2.8 6835e4fa (spec-only). Streak → 0/3.
- Feature code HEAD UNCHANGED at 6835e4fa since pass-14 de-pin. ALL 16 passes CLEAN(PR-merge)=yes.
- NEXT: PR-LEVEL pass 17 (fresh streak; need 3 strict-clean on 6835e4fa + story v2.8).
- Passes 17/18/19 CONVERGED (D-1021; BC-5.39.001 D-779). PR #173 squash-merged develop@9447671f (D-1022). Phase B Lane 2 COMPLETE.

---

## Cycle-Close Codification Candidates (D-1022 — S-7.02 Cycle-Closing-Checklist)

Three process-gap candidates surfaced during the S-DEMO-QUERY-PUSHDOWN-001 PR-LEVEL cascade. All three are **JUSTIFIED DEFERRALS** — engine/process scope, not prism product defects. No prism product follow-up story required for any of these.

### (a) OBS-P13-003 — PR-LEVEL Adversary Worktree-Path Probe (engine/dispatch-ergonomics)

**D-NNN anchor:** D-1022 (cycle-close); first occurrence D-1013 (LOCAL pass-13)
**Tags:** [process-gap] [engine-scope] [dispatch-ergonomics] [justified-deferral]
**Classification:** JUSTIFIED DEFERRAL — engine/dispatch-ergonomics scope; no prism product story needed.

**Description:**
PR-LEVEL adversary dispatched into the feature worktree `.worktrees/S-DEMO-QUERY-PUSHDOWN-001` but lacked a `.factory/` mount from that cwd. Relative glob paths for SAP-1 (`rg 'event_type\s*=' crates/`), POLICY-13 checks, and POLICY-32 sweeps resolved against the worktree root rather than the main repo root, causing some probe axes to silently miss `.factory/` artifacts.

**Mitigation applied (passes 15-19):** Orchestrator dispatch instructions now explicitly specify absolute paths: `--cwd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-QUERY-PUSHDOWN-001` with `.factory/` references as `--factory-root /Users/jmagady/Dev/prism/.factory/`. This was applied as in-session SOP from pass 15 onward and all passes 15-19 confirmed CLEAN.

**Required action:** vsdd-factory engine process improvement — adversary dispatch should automatically resolve `.factory/` as an absolute path from the main repo root, not relative to the feature worktree cwd. Track in drbothen/vsdd-factory upstream issue tracker. No prism product story needed.

### (b) Strict-3-CLEAN Cosmetic Tail — Pre-PR Hygiene-Sweep Gate (engine/process)

**D-NNN anchor:** D-1022 (cycle-close); D-1020 (codification candidate first recorded)
**Tags:** [process-gap] [engine-scope] [pre-PR-gate] [hygiene-sweep] [justified-deferral]
**Classification:** JUSTIFIED DEFERRAL — engine/process scope; pre-PR gate is a vsdd-factory pr-manager workflow improvement, not a prism product feature.

**Description:**
After all code correctness findings were resolved (pass-9 code converged), 7 additional passes (passes 10-16) were required to drain 5 classes of cosmetic spec/evidence artifact LOWs:
- Dangling-AC (pass-9 complete sweep; required after one-at-a-time fix at pass-8 missed AC-CWS-WIRE-001)
- Draft-comment (pass-11 complete sweep; ac75e84d)
- Vacuous-assert (pass-13; 6583e419)
- Volatile-SHA in evidence (pass-14 TD-VSDD-091 de-pin; 6835e4fa)
- Story line-pins (pass-16 TD-VSDD-091 complete sweep; story v2.8)

Each LOW finding reset the strict-3-CLEAN streak, costing one full cascade pass per occurrence (7 passes total for 5 classes). Running ONE upfront hygiene-sweep before the PR-LEVEL cascade would drain all 5 classes from the first pass, allowing all 19 passes to focus on substantive behavioral review.

**Proposed pre-PR hygiene-sweep gate** (to be added as a standing checklist item in the pr-manager 9-step cycle before PR-LEVEL pass 1):
1. Line-pin sweep (TD-VSDD-091): grep story body + spec artifacts for `~\d+` and `file.rs:\d+`; anchor to function names.
2. Draft-comment sweep: grep all PR diff files for "wait —", "TODO:", "FIXME:", "// temp", "draft" in comments.
3. Vacuous-assertion audit: for each e2e/integration test AC assertion, verify it independently falsifies the named property.
4. Volatile-SHA sweep in evidence: grep `docs/demo-evidence/<story>/` for raw 40-char hex SHAs; replace with stable `PR#/story-version/LOCAL-converged-SHA` refs.
5. AC traceability sweep (SAP-5): grep `crates/**/*.rs` for `AC-[A-Z][A-Z0-9_-]+`; verify each resolves to `### AC-ID:` in story file.

**Required action:** vsdd-factory engine process improvement — add the pre-PR hygiene-sweep gate to the pr-manager 9-step cycle template as a mandatory Step 0 before adversary pass 1. This is a process discipline, not a code deliverable. Track in drbothen/vsdd-factory upstream. No prism product story needed.

### (c) Dangling-AC CI-Lint Candidate (engine/CI)

**D-NNN anchor:** D-1022 (cycle-close); D-1018 (first occurrence AC-INDEX-CWS-001) + D-1019 (second occurrence AC-CWS-WIRE-001)
**Tags:** [process-gap] [engine-scope] [CI-lint] [dangling-AC] [traceability] [justified-deferral]
**Classification:** JUSTIFIED DEFERRAL — engine/CI scope; a CI lint rule for the vsdd-factory pr-manager workflow, not a prism product feature.

**Description:**
Two consecutive PR-LEVEL passes (passes 8 and 9) found dangling-AC traceability gaps — test/code files cited AC identifiers (`AC-INDEX-CWS-001` at pass-8; `AC-CWS-WIRE-001` at pass-9) that were absent from the story's formally defined AC headings. Both recurred because the pass-8 fix added only the single AC flagged without sweeping ALL this-story AC identifiers cited in crates.

The complete-sweep approach applied at pass-9 (SAP-5 candidate) definitively closed the class for this story. However, neither the LOCAL adversary cascade nor the pre-PR step had a systematic mechanism to catch dangling ACs early.

**Proposed CI-lint gate:**
```bash
# For each story in delivery:
rg 'AC-[A-Z][A-Z0-9_-]+' crates/ --type rust -o -N | sort -u \
  | while read ac_id; do
      grep -q "### ${ac_id}:" .factory/stories/<story-file>.md || echo "DANGLING: $ac_id"
    done
```
Any dangling AC ID = MEDIUM finding. Run as a CI check or as a mandatory pre-PR step.

**Required action:** vsdd-factory engine process improvement — add a dangling-AC traceability lint to the CI pipeline or to the pr-manager pre-PR checklist. Assert every `AC-<this-story-prefix>` cited in `crates/**/*.rs` resolves to a `### AC-ID:` heading in the story file. Track in drbothen/vsdd-factory upstream. No prism product story needed.
