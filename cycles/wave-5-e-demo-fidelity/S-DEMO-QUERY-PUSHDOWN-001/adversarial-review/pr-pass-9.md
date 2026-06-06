---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 9
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "1a8cc8aa"
feature_head_after_fix_burst: "1a8cc8aa"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-06
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 9 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 1a8cc8aa at review)
**Pass:** PR-LEVEL pass 9 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-06

## Pass-8 Closure Verification

Pass-8 found F-P08-MED-001 (dangling AC-INDEX-CWS-001: 11 code/test sites cited the AC identifier
but story v2.5 did not define it). Story-writer closed it with a spec-only fix: AC-INDEX-CWS-001
added to story v2.6 (CrowdStrike parallel to AC-INDEX-001 Armis; cites existing test
`test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option`).
acceptance_criteria_count 16→17; red_gate_tests 18→19; story v2.5→v2.6; STORY-INDEX v2.287→v2.288.
Feature HEAD unchanged at 1a8cc8aa.

The story-writer fix was verified at story v2.6: AC-INDEX-CWS-001 is now formally defined in the
story's acceptance criteria section and the cited test is correctly referenced.

## Complete Dangling-AC Sweep (Orchestrator-Directed at Pass 9)

The orchestrator directed a complete dangling-AC sweep at pass 9 — rather than finding one dangling
AC per pass, an exhaustive cross-reference was performed:

**Sweep method:** grep `crates/**/*.rs` for all AC identifiers matching the this-story prefix
pattern (`AC-CWS-`, `AC-ARMIS-`, `AC-CYB-`, `AC-CLAR-`, `AC-INDEX-`, `AC-WIRE-`, `AC-EQUIV-`)
then verify each unique AC ID cited in code/tests resolves to a formally defined `### AC-ID:`
heading in the story file.

**Sweep result: ZERO remaining dangling ACs.**

All AC identifiers cited in `crates/**/*.rs` now resolve to formally defined AC sections in story
v2.7 (see closure below):

| AC ID | Status | Story Section |
|-------|--------|---------------|
| AC-CWS-001..005 | Defined | story §Acceptance Criteria |
| AC-CWS-DTU-001 | Defined | story §Acceptance Criteria |
| AC-ARMIS-001..005 | Defined | story §Acceptance Criteria |
| AC-ARMIS-TW-001..005 | Defined | story §Acceptance Criteria |
| AC-INDEX-001 | Defined | story §Acceptance Criteria (Armis) |
| AC-INDEX-CWS-001 | Defined | story §Acceptance Criteria (CrowdStrike — added pass-8) |
| AC-CWS-WIRE-001 | Defined | story §Acceptance Criteria (CrowdStrike — added THIS PASS; see below) |
| AC-EQUIV-001 | Defined | story §Acceptance Criteria |
| AC-WIRE-001b | Sub-test reference (not a separate AC) | Not a dangling AC — sub-test notation |
| AC-001..012, AC-1..16 | Belong to other stories' shared test files | Not this-story ACs |

## Adversary Pass 9 Findings

### F-P09-MED-001 (MEDIUM) — Second dangling AC: AC-CWS-WIRE-001 cited 18× in test file but absent from story source-of-truth

**Finding ID:** F-P09-MED-001
**Severity:** MEDIUM
**Category:** Traceability / AC completeness (story source-of-truth deficiency)

**Description:** The complete dangling-AC sweep found that `bc_2_11_007_pushdown_test.rs` contains
18 citations of the acceptance criterion identifier `AC-CWS-WIRE-001`. Specifically:

- The test function `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu` is named with the
  `AC-CWS-WIRE-001` identifier, directly parallel to the existing `AC-CWS-003` wire-level assertion
  AC (which covers CrowdStrike absence when no time filter is present).
- 17 additional citation sites (doc-comments, assertion strings) in the same test file reference
  `AC-CWS-WIRE-001`.

However, story v2.6 does NOT define an AC named `AC-CWS-WIRE-001`. The story defines AC-CWS-003
(the wire-level assertion that `created_timestamp` is ABSENT when no time filter is applied) but
does not have a corresponding CrowdStrike wire-level assertion AC for the positive case (when a
time filter IS applied and `created_timestamp` IS present in the filter).

**Root cause:** This is the same class as F-P08-MED-001 (dangling AC-INDEX-CWS-001). The pass-8
fix correctly added AC-INDEX-CWS-001 but was a one-at-a-time approach — the story-writer added
only the AC that pass-8 flagged without performing an exhaustive sweep of all this-story AC
identifiers cited in code/tests. The `AC-CWS-WIRE-001` identifier was in the same test file as
`AC-INDEX-CWS-001` but was not swept at pass-8. This is precisely the S-7.01 partial-fix miss
pattern: fix-burst addressed one instance without sweeping all siblings of the same class.

**Impact assessment:** NOT a code correctness defect. The test
`test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu` EXISTS and PASSES at HEAD 1a8cc8aa.
The behavioral requirement (CrowdStrike wire-level assertion that FQL + limit parameters reach the
DTU) is satisfied in the implementation. The gap is purely a story-spec traceability deficiency:
18 code/test sites cite an AC that the story's formal AC list does not define, leaving the story's
acceptance_criteria_count understated.

**Why this class evades pass-by-pass detection (recurrence from F-P08-MED-001 lesson):**
1. Tests pass — no test failure signals the gap.
2. AC count appears internally consistent by the story's own (understated) definition.
3. The behavioral requirement IS satisfied.

**Closure:** CLOSED by story-writer (spec-only fix; no code change to feature branch):
- Added `AC-CWS-WIRE-001` to the story's acceptance criteria as the CrowdStrike wire-level
  assertion AC: CrowdStrike FQL filter and LIMIT parameters reach the DTU (wire-level evidence
  from DTU `/dtu/filter-log`). Cites existing test
  `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu`.
- `acceptance_criteria_count` 17 → 18.
- `red_gate_tests` 19 → 20 (existing test added to the formal Red Gate table).
- Story version v2.6 → v2.7.
- STORY-INDEX Full Story List row updated: in_progress v2.6 → in_progress v2.7.
- STORY-INDEX v2.288 → v2.289.
- Feature code HEAD remains 1a8cc8aa (spec-only fix; no code change required).

## Complete Sweep Confirmation (Post-Fix)

After the story-writer applied the v2.7 fix, the orchestrator re-verified the complete
dangling-AC sweep:

**ZERO remaining dangling ACs.** All AC identifiers cited in `crates/**/*.rs` that match
this-story prefixes now resolve to formally defined AC sections in story v2.7. The dangling-AC
class is definitively closed for this story.

Excluded from sweep (by design):
- `AC-WIRE-001b`: sub-test reference notation used in a doc-comment to distinguish a sub-case
  of the same behavioral property — not a separate AC identifier requiring its own story heading.
- `AC-001..012`, `AC-1..16`: belong to other stories' shared test files; confirmed by searching
  story STORY-INDEX to verify they are claimed by other stories.

## Summary

**CLEAN(strict):** no (1 MEDIUM F-P09-MED-001; not zero)
**CLEAN(PR-merge):** no (MEDIUM finding present; blocks this pass from counting as PR-merge-clean)
**Streak:** 0/3 (streak RESET 0/3 — it was 0/3 entering pass 9; this pass finds another MEDIUM;
   streak remains at 0/3 per BC-5.39.001 D-779; finding was closed before streak can begin)
**Feature HEAD:** 1a8cc8aa (UNCHANGED — spec-only fix; no code change)
**Story version after fix:** v2.7
**STORY-INDEX version after fix:** v2.289
**Next step:** PR-LEVEL pass 10 (fresh streak on 1a8cc8aa + fixed story v2.7; need 3 strict-clean)

## Codification Note (Second Occurrence — Class Now Definitively Closed)

The dangling-AC class has now recurred twice in consecutive passes (F-P08-MED-001 at pass 8,
F-P09-MED-001 at pass 9). The root cause in both cases was one-at-a-time (rather than exhaustive)
AC-ID cross-reference checking. The pass-9 lesson update codifies the complete-sweep approach:

**Required adversary discipline (codification candidate SAP-5):**
At every PR-LEVEL adversary pass for a story, run a complete dangling-AC sweep:
```
rg 'AC-[A-Z][A-Z0-9_-]+' crates/**/*.rs | grep -oP 'AC-[A-Z][A-Z0-9_-]+' | sort -u
```
For each unique AC ID found: verify it appears as `### AC-ID:` in the story file, OR explicitly
document why it belongs to another story. Any unresolved ID = MEDIUM finding (this class).

This probe was applied at pass 9 and confirmed the class is now closed for this story (ZERO
remaining dangling ACs after v2.7 fix).

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | All pass-8 closures confirmed load-bearing; no correctness regression |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: CrowdStrike+Armis DTU↔TOML parity confirmed |
| AC traceability — complete sweep | FAIL → FIXED | F-P09-MED-001: 18 code/test sites cited AC-CWS-WIRE-001 absent from story v2.6; story-writer added AC-CWS-WIRE-001 → story v2.7. Post-fix sweep: ZERO remaining dangling ACs |
| Code documentation | PASS | All comments accurate |
| Input normalization | PASS | AQL leading-space trim present; confirmed durable |
| Inclusive-boundary semantics (EC-009) | PASS | Confirmed durable |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions |
| Security | PASS (CLEAR-TO-MERGE per pass-1 verdict) | No new security surface |
| Demo evidence | PASS | Counts accurate (CrowdStrike=8, Armis=5 per v1.69) |
| pr-reviewer verdict | APPROVE (on eab62613; NITs since cleaned at 1a8cc8aa) | Still valid; no code change since NITs were cleaned |
| security verdict | SECURITY-CLEAR-TO-MERGE (pass-1) | No new security surface added; still valid |
