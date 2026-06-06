---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 5
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "c9b22402"
feature_head_after_fix_burst: "1a8cc8aa"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 5 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head c9b22402 at review)
**Pass:** PR-LEVEL pass 5 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Pass-4 Closure Verification

Pass-4 closures (PR-LEVEL pass 4 was the first CLEAN strict pass — streak 0/3 → 1/3). All prior
closures from passes 1-3 confirmed load-bearing at HEAD c9b22402.

## Adversary Pass 5 Findings

### OBS-P05-001 (LOW) — Stale Test Comment: fixture-specific language

**Finding ID:** OBS-P05-001
**Severity:** LOW
**Category:** Test comment accuracy (TD-VSDD-091 adjacency)

**Description:** A test comment in the push-down test suite used fixture-specific language
(referencing a specific fixture file path or dataset name) that would become inaccurate if fixtures
were reorganized. The comment should use fixture-agnostic language anchored to the behavioral
contract property being tested.

**Root cause:** Test comment authored when fixtures were first created; no sweep was done to
ensure all test comments used fixture-agnostic, contract-anchored language.

**Closure:** CLOSED by implementer fix-burst at HEAD c9b22402 → 1a8cc8aa. Test comment updated
to use fixture-agnostic, contract-anchored language consistent with TD-VSDD-091 discipline.
Feature HEAD c9b22402 → 1a8cc8aa.

### OBS-P05-002 (LOW) — Limit-Clamp Rationale Missing in Code

**Finding ID:** OBS-P05-002
**Severity:** LOW
**Category:** Code documentation accuracy

**Description:** The limit-clamp logic in the push-down implementation lacked an explanatory
comment documenting why the clamp value was chosen (e.g., that it matches the DTU's per-request
cap or an AC-specified maximum). A reader reviewing the code for the first time could not determine
whether the hardcoded clamp value was arbitrary or derived from a spec requirement.

**Root cause:** Implementation focused on correctness; comment discipline for non-obvious numeric
constants was not applied at authorship time.

**Closure:** CLOSED by implementer fix-burst at 1a8cc8aa. Comment added documenting the clamp
rationale referencing the relevant AC and DTU constraint.

### OBS-P05-003 (LOW) — AQL Keyword Bound: Leading-Space Handling Not Evident

**Finding ID:** OBS-P05-003
**Severity:** LOW
**Category:** Input normalization discipline

**Description:** The `extract_aql_keyword_bound` function in `crates/prism-dtu-armis/src/search.rs`
did not visibly trim leading spaces from extracted AQL keyword bounds. The adversary noted that AQL
values with leading spaces (e.g., ` after:2024-01-01T00:00:00`) could bypass the extraction logic or
produce malformed filter strings. While the current call-path may not admit leading spaces, the
absence of an explicit trim and a test covering that case left the function's contract ambiguous.

**Root cause:** Defensive normalization (trim leading whitespace) was not applied when the function
was authored; no test exercised the leading-space case.

**Closure:** CLOSED by implementer fix-burst at 1a8cc8aa. `extract_aql_keyword_bound` updated to
trim leading spaces before extraction; new test added confirming that AQL strings with leading spaces
are normalized correctly.

## Summary

**CLEAN(strict):** no (3 LOW findings — OBS-P05-001, OBS-P05-002, OBS-P05-003; not zero)
**CLEAN(PR-merge):** yes (0 CRIT + 0 HIGH + 0 MED; all LOWs are non-blocking for merge)
**Streak:** 0/3 (streak reset — pass 4 was 1/3 but pass 5 found LOWs; strict criterion requires zero findings)
**Feature HEAD before fix-burst:** c9b22402
**Feature HEAD after fix-burst:** 1a8cc8aa
**Fix-burst summary:** implementer — (1) test comment fixture-agnostic; (2) limit-clamp comment; (3) augment trims leading space + new test
**Next step:** PR-LEVEL pass 6 (fresh streak attempt on 1a8cc8aa)

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | All prior pass closures verified load-bearing; no correctness regression |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: DTU↔TOML parity confirmed |
| Code documentation | LOW → FIXED | OBS-P05-002 limit-clamp comment; OBS-P05-001 fixture-agnostic comment |
| Input normalization | LOW → FIXED | OBS-P05-003 leading-space trim + test |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions |
| Security (CLEAR-TO-MERGE per pass-1 verdict) | PASS | No new security surface added |
| Story spec (v2.5) | PASS | No story-spec changes triggered by this pass |
