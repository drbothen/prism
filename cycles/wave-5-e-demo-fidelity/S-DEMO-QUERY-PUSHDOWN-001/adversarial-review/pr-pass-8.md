---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 8
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "1a8cc8aa"
feature_head_after_fix_burst: "1a8cc8aa"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 8 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 1a8cc8aa at review)
**Pass:** PR-LEVEL pass 8 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Pass-7 Closure Verification

Pass-7 was CLEAN(strict)=yes with zero findings. Feature HEAD remains 1a8cc8aa. All pass-5
closures confirmed load-bearing. Streak was 2/3 going into this pass.

## Adversary Pass 8 Findings

### F-P08-MED-001 (MEDIUM) — Dangling AC: AC-INDEX-CWS-001 cited in 11 code/test sites but absent from story source-of-truth

**Finding ID:** F-P08-MED-001
**Severity:** MEDIUM
**Category:** Traceability / AC completeness (story source-of-truth deficiency)

**Description:** A grep of the feature branch reveals 11 code and test sites citing the acceptance
criterion identifier `AC-INDEX-CWS-001`:

- `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs` — test function
  `test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option` (and related)
- Multiple test doc-comments and assertion strings citing `AC-INDEX-CWS-001`
- `crates/prism-sensors/specs/crowdstrike.sensor.toml` — options declaration comment citing
  `AC-INDEX-CWS-001`

However, the story file `S-DEMO-QUERY-PUSHDOWN-001-query-param-push-down-into-pipeline-executor.md`
at version v2.5 does NOT define an AC named `AC-INDEX-CWS-001`. The story defines `AC-INDEX-001`
(the Armis parallel: `armis.sensor.toml` last_seen/created_at declare `options = ["INDEX"]`), but
the CrowdStrike equivalent for `created_timestamp` was never formally added to the story's
acceptance criteria.

**Root cause:** The CrowdStrike `options = ["INDEX"]` requirement was implemented during the
push-down redesign (v2 LOCAL cascade) as part of the ADR-033 T1 heuristic wiring. The tests were
named with the `AC-INDEX-CWS-001` identifier to maintain parallel structure with AC-INDEX-001
(Armis). However, the story-writer did not add a corresponding `AC-INDEX-CWS-001` entry to the
story's formal acceptance criteria when the tests were authored. This created a traceability gap
where the test citation resolves to a non-existent AC in the story's source-of-truth, and the
passing test count implied by the story (16 ACs, 18 Red Gate tests at v2.5) was understated.

**Impact assessment:** This is NOT a code correctness defect. The test `test_ac_index_cws_001_*`
ALREADY EXISTS and ALREADY PASSES at HEAD 1a8cc8aa. The CrowdStrike `crowdstrike.sensor.toml`
`created_timestamp` ALREADY has `options = ["INDEX"]` declared. The behavioral requirement is
satisfied. The gap is purely a story-spec traceability deficiency: 11 code/test sites cite an AC
that the story's formal AC list does not define, making the story's claimed AC count understated.

**Why this is MEDIUM (not HIGH):** The implementation is correct; no code change is needed. The
finding is a spec completeness defect: the story's AC count (16 at v2.5) did not reflect the
CrowdStrike INDEX requirement that was clearly implemented and tested under the AC-INDEX-CWS-001
identifier.

**Closure:** CLOSED by story-writer (spec-only fix; no code change to feature branch):
- Added `AC-INDEX-CWS-001` to the story's acceptance criteria as the CrowdStrike parallel to
  `AC-INDEX-001` (Armis). Both require that the respective sensor TOML declares
  `options = ["INDEX"]` on the push-down-eligible datetime column so that ADR-033 T1 extraction
  correctly recognizes it.
- `acceptance_criteria_count` 16 → 17.
- `red_gate_tests` 18 → 19 (existing test `test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option` added to the formal Red Gate table).
- Story version v2.5 → v2.6.
- STORY-INDEX Full Story List row updated: in_progress v2.5 → in_progress v2.6.
- STORY-INDEX v2.287 → v2.288.
- Feature code HEAD remains 1a8cc8aa (spec-only fix; no code change required).

**Note on streak:** The streak resets 2/3 → 0/3 because this finding is MEDIUM severity, which
means CLEAN(strict)=no. The finding was purely a story-spec traceability deficiency; feature code
is stable and correct at 1a8cc8aa.

## Summary

**CLEAN(strict):** no (1 MEDIUM F-P08-MED-001; not zero)
**CLEAN(PR-merge):** no (MEDIUM finding present; blocks this pass from counting as PR-merge-clean)
**Streak:** 0/3 (streak RESET 2/3 → 0/3 per BC-5.39.001 D-779; MEDIUM finding)
**Feature HEAD:** 1a8cc8aa (UNCHANGED — spec-only fix; no code change)
**Story version after fix:** v2.6
**STORY-INDEX version after fix:** v2.288
**Next step:** PR-LEVEL pass 9 (fresh streak on 1a8cc8aa + fixed story v2.6; need 3 strict-clean)

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | All pass-7 closures confirmed load-bearing; no correctness regression |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: CrowdStrike+Armis DTU↔TOML parity confirmed |
| AC traceability (story source-of-truth completeness) | FAIL → FIXED | F-P08-MED-001: 11 code/test sites cited AC-INDEX-CWS-001 absent from story v2.5; story-writer added AC-INDEX-CWS-001 → story v2.6 |
| Code documentation | PASS | All comments accurate |
| Input normalization | PASS | AQL leading-space trim present; confirmed durable |
| Inclusive-boundary semantics (EC-009) | PASS | Confirmed durable |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions |
| Security | PASS (CLEAR-TO-MERGE per pass-1 verdict) | No new security surface |
| Demo evidence | PASS | Counts accurate |
| pr-reviewer verdict | APPROVE (recorded from pass-1 era; NITs since cleaned at 1a8cc8aa) | pr-reviewer approved on eab62613 (pre-fix-burst); NITs cleaned in c9b22402→1a8cc8aa delta |
| security verdict | SECURITY-CLEAR-TO-MERGE (pass-1) | No new security surface added since pass-1 verdict; still valid |
