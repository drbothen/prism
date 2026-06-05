# LOCAL Adversary Pass 1 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Thread QueryParams push-down into PipelineExecutor via FetchContext
**Pass:** LOCAL adversary pass 1
**Feature HEAD at pass:** `19184786`
**Feature HEAD after fix-burst:** `a75fada4`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

8 findings recorded (2 HIGH, 2 MED, 2 LOW, 2 OBS). All 8 CLOSED or ADJUDICATED. Code fix-burst commit `a75fada4` (implementer). Spec fix-burst applied to .factory/ working tree.

---

## Findings

### F-PUSHDOWN-001 — HIGH — Push-down applied to ALL pipeline steps incl entity-fetch step

**Severity:** HIGH
**Category:** contract-semantics / test-coverage gap
**Status:** CLOSED — implementer commit `a75fada4`

**Description:** The `apply_push_down_to_request()` dispatch used `auth_type` as the sole discriminator. CrowdStrike sensor specs have two steps: step-1 (`detection-queries`, `auth_type = Oauth2ClientCredentials`) and step-2 (`detection-entities`, `auth_type = Oauth2ClientCredentials`). Both steps share the same auth_type, so push-down was applied to BOTH steps including the entity-fetch step, which must NOT receive push-down parameters (entity-fetch accepts a list of IDs, not pagination params). A single-step fixture test did not expose this because it never ran the two-step pipeline with both steps present.

**Fix:** Implementer gated push-down on `is_first_step` boolean threaded through `FetchContext`. Step-2 (entity-fetch) receives `is_first_step = false` and passes through `apply_push_down_to_request()` with no-op behavior. Added a two-step CrowdStrike fixture test `test_BC_2_11_007_crowdstrike_two_step_step1_gets_pushdown_step2_does_not` that explicitly asserts step-1 receives `limit=50` and step-2 does NOT receive any push-down params.

**Closure verified:** Load-bearing test present; `just check` 4001/4001 GREEN.

---

### F-PUSHDOWN-002 — MED — AC-005 result-equivalence test only checked param-reaches-wire, not equivalence

**Severity:** MED
**Category:** test coverage / non-load-bearing assertion
**Status:** CLOSED — implementer commit `a75fada4`

**Description:** The Red Gate test `test_BC_2_11_007_push_down_result_equivalence_invariant` asserted that `limit=1` reached the wire (wiremock `query_param("limit", "1")` matcher), verifying the mechanical plumbing precondition. AC-005 states the invariant: "push-down is an optimization only; result must be identical whether push-down params are applied or not." The test only verified the precondition (limit reaches wire), not the actual invariant (two executions — one with push-down, one without — produce identical result sets). A test for the precondition alone is not an ungameable test for the invariant.

**Fix:** Implementer expanded `test_BC_2_11_007_push_down_result_equivalence_invariant` to run the pipeline twice — once with push-down params set, once with `FetchContext::default()` (no push-down) — and assert both produce the same record-batch shape and row count. Wiremock server configured to respond identically to both requests (the constrained and unconstrained form).

**Closure verified:** Test now exercises the equivalence invariant directly.

---

### F-PUSHDOWN-003 — MED — EC-005 warning log absent

**Severity:** MED
**Category:** missing-behavior / spec non-conformance
**Status:** CLOSED — implementer commit `a75fada4`

**Description:** BC-2.11.007 EC-005 specifies: "If `start_time > end_time`, the executor MUST emit a `tracing::warn!` event with `event_type = \"push_down.time_window_inverted\"` and apply no time-window parameters for that step." The implementation silently returned `Ok(req)` with no time-window injection and no warning log for the inverted-window case. No test existed to verify the warning.

**Fix:** Implementer added the warning log in `apply_push_down_to_request()` for the `start_time > end_time` case. Added `test_BC_2_11_007_ec005_inverted_time_window_emits_warning` unit test using `tracing_test::traced_test` to capture the log event and assert `event_type = "push_down.time_window_inverted"` is present. BC-2.16.002 Structured Event Catalog updated with the new `push_down.time_window_inverted` row (SAP-1 compliance).

**Closure verified:** Warning emitted; catalog row present; test load-bearing.

---

### F-PUSHDOWN-004 — LOW — Stale Red Gate stub doc-comments

**Severity:** LOW
**Category:** doc-comment drift / TD-VSDD-091
**Status:** CLOSED — implementer commit `a75fada4`

**Description:** The Red Gate stub `apply_push_down_to_request()` was preserved with `// TODO: implement push-down translation` comments after the implementer filled in the real logic. These stale stub markers were factually wrong after implementation.

**Fix:** Implementer removed all `// TODO: implement` and `// stub: returns req unchanged` comments from the production implementation. Doc-comment updated to reflect actual behavior.

**Closure verified:** No stale stub markers remain in `apply_push_down_to_request()` function body.

---

### F-PUSHDOWN-005 — LOW→MED (reclassified LOW) — Cyberint page_size doc drift

**Severity:** LOW (originally noted as LOW→MED; confirmed LOW on close — doc claim only, no behavior gap)
**Category:** doc-comment drift / TD-VSDD-091
**Status:** CLOSED — implementer commit `a75fada4`

**Description:** The Cyberint push-down handler had a doc comment claiming "page_size translation not yet implemented; future story." This claim was incorrect — the story scope covers Cyberint time-window push-down (from_date/to_date injection) but not page_size (which Cyberint POST body does not support directly). The comment overstated a gap.

**Fix:** Implementer removed the "not yet implemented" claim. Comment replaced with: "Cyberint: time-window push-down via from_date/to_date in POST body. page_size is not a Cyberint API parameter."

**Closure verified:** Doc claim removed; no behavior gap.

---

### F-PUSHDOWN-006 — OBS→MED (per POLICY 4 semantic mis-anchor) — VP-031 semantic mis-anchor in story frontmatter

**Severity:** OBS→MED (POLICY 4 — mis-anchor creates downstream traceability gap)
**Category:** traceability / frontmatter error
**Status:** CLOSED — story-writer (VP-031 removed from `verification_properties: []`)

**Description:** Story frontmatter listed `verification_properties: [VP-031]`. VP-031 is the prism-query depth-limit Kani proof in `crates/prism-query/src/proofs/`. This story is entirely in `prism-spec-engine` and `prism-bin` — it does not touch prism-query's query planning layer. VP-031 verifies the *query planner's* push-down depth bound, not the *pipeline executor's* parameter threading. The mis-anchor would cause VP-031 to be counted as a verification property for a story it does not verify, polluting traceability.

**Fix:** Story-writer removed `VP-031` from `verification_properties` frontmatter. Field set to `[]`.

**Adjudication note (per POLICY 4 — Source-of-Truth Rule 1):** This is NOT a §7 code-amendment. VP-031 was never in scope — it was mis-assigned in the story stub. The BC (BC-2.11.007 result-equivalence) is verified behaviorally by the AC-005 two-path subset test. A dedicated push-down correctness VP may be warranted post-merge (see codification-candidate item below).

**Closure verified:** Story frontmatter `verification_properties: []`; no VP-031 reference in story body or frontmatter.

---

### F-PUSHDOWN-007 — OBS — BC-2.11.005 listed but no AC trace

**Severity:** OBS
**Category:** traceability / coverage documentation
**Status:** ADJUDICATED — retained; body note propagated by story-writer

**Description:** Story frontmatter lists `BC-2.11.005` (Ephemeral Materialization) in `behavioral_contracts`. No AC in the story directly traces to BC-2.11.005. Push-down affects materialization behavior (fewer rows collected = smaller ephemeral dataset), but no AC is written to assert this.

**Adjudication (product-owner):** Retain BC-2.11.005 in `behavioral_contracts`. It is "affected-but-indirectly-tested" — the push-down optimization reduces the data collected during materialization, which is the mechanism BC-2.11.005 describes. The test coverage is indirect: if push-down reaches the wire (AC-002/AC-005), materialization necessarily receives fewer rows. PO ruled: add inline body note "BC-2.11.005: affected indirectly — push-down reduces materialized row count; not directly tested by a dedicated AC in this story."

**Closure verified:** Body note propagated by story-writer in v1.2.

---

### F-PUSHDOWN-008 — OBS — BC-2.01.013 v1.11 OUT-OF-SCOPE clause contradicted shipped behavior

**Severity:** OBS (escalated to MED for BC amendment; CLOSED by PO)
**Category:** contract-semantics / spec drift
**Status:** CLOSED — product-owner (BC-2.01.013 v1.11→v1.12; EC-01-027 + TV-006 re-cast; Source-of-Truth Rule 1 anchor)

**Description:** BC-2.01.013 v1.11 §Pagination/Push-Down Scope Clause (introduced at S-DEMO-001 D-924) included an `OUT_OF_SCOPE` annotation stating push-down translation from FetchContext fields into sensor-native request parameters was "deferred to follow-up story S-DEMO-QUERY-PUSHDOWN-001." This story IS that follow-up story — it implements the deferred push-down. After this story ships, BC-2.01.013 must reflect that the implementation gap has been closed, per CLAUDE.md Source-of-Truth Rule 1 (story spec supersedes BC for implementation scope, not contract semantics; the contract now reflects delivered behavior).

**Fix:** Product-owner authored BC-2.01.013 v1.12: (1) `OUT_OF_SCOPE` annotation converted to affirmative: "push-down translation IS implemented by S-DEMO-QUERY-PUSHDOWN-001 — see BC-2.11.007 for per-sensor translation rules"; (2) EC-01-027 re-cast from "deferred" to "implemented — see BC-2.11.007 EC-005"; (3) TV-006 re-cast to reflect implemented behavior. BC-INDEX row v1.11→v1.12 (BC-INDEX v5.82→v5.83). OCSF-CLASS-MIGRATION-001 story body pin updated v1.11→v1.12 (per SAP-1 sibling sweep).

**Authority:** Source-of-Truth Rule 1 (story spec + delivered behavior supersedes BC for implementation scope). NOT a §7 code-amendment — the code (this story) is authoritative; the BC is being brought into alignment.

**Closure verified:** BC-2.01.013 v1.12 authored; BC-INDEX v5.82→v5.83; sibling site OCSF-CLASS-MIGRATION-001 swept.

---

## Code Fix-Burst Summary

**Commit:** `a75fada4` on `feature/S-DEMO-QUERY-PUSHDOWN-001`
**Commit message:** `fix(S-DEMO-QUERY-PUSHDOWN-001): close 5 LOCAL adversary pass-1 findings`
**just check result:** 4001/4001 GREEN (all tests pass, full workspace)

**Findings closed by code fix-burst (F-001 through F-005):**
- F-PUSHDOWN-001: `is_first_step` gate added; 2-step CrowdStrike fixture test added
- F-PUSHDOWN-002: AC-005 equivalence invariant test rewritten with two-path comparison
- F-PUSHDOWN-003: EC-005 warning log added; catalog row added; unit test added
- F-PUSHDOWN-004: Stale stub doc-comments removed
- F-PUSHDOWN-005: Cyberint page_size over-claim removed

**Findings closed by spec fix-burst (.factory/ working tree):**
- F-PUSHDOWN-006: VP-031 removed from story frontmatter (story-writer)
- F-PUSHDOWN-007: Body note propagated (story-writer v1.2)
- F-PUSHDOWN-008: BC-2.01.013 v1.11→v1.12 (PO); BC-INDEX v5.82→v5.83; OCSF sibling pin swept

---

## New-VP Follow-Up Note [codification-candidate] [process-gap]

After VP-031 removal, `verification_properties: []` for S-DEMO-QUERY-PUSHDOWN-001. Push-down correctness (AC-002..005, BC-2.11.007 result-equivalence invariant) is currently verified only behaviorally by the AC-005 two-path subset test. A dedicated Kani proof VP verifying the push-down translation monotonicity property (applying push-down never increases row count above the unconstrained case) may be warranted.

**Status:** Deferred. Justified deferral: AC-005 two-path subset test covers the invariant behaviorally; a Kani proof would add formal exhaustiveness. PO/architect adjudication required for VP authorship and proof harness design (push-down proofs need a mock `SensorSpec` type that Kani can enumerate). This is a codification candidate, NOT a blocking issue.

**Anchor:** Tagged [codification-candidate]. Follow-up at cycle-close review. Does NOT block LOCAL pass-2 dispatch or PR delivery.

---

## Convergence Trajectory

| Pass | Findings | Delta | CLEAN(strict) | CLEAN(PR-merge) | Streak |
|------|----------|-------|--------------|-----------------|--------|
| 1 | 8 (2H+2M+2L+2OBS) | — | no | no | 0/3 |

**NEXT:** LOCAL adversary pass 2 against feature HEAD `a75fada4`.
