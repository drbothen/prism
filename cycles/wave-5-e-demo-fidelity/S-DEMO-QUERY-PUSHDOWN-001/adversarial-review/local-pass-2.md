# LOCAL Adversary Pass 2 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Thread QueryParams push-down into PipelineExecutor via FetchContext
**Pass:** LOCAL adversary pass 2
**Feature HEAD at pass:** `a75fada4`
**Feature HEAD after fix-burst:** `688f82b5`
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3**

3 findings recorded (1 MED, 1 LOW, 1 OBS). All 3 CLOSED. Code fix-burst commit `688f82b5` (implementer). Spec fix-burst applied to .factory/ working tree (F-PUSHDOWN2-MED-001 status sync).

---

## Pass-1 Closure Re-verification

Fresh-context re-derivation of all 8 pass-1 closures at feature HEAD `a75fada4`. All verified LOAD-BEARING:

| Finding | Closure claim | Re-derivation result |
|---------|--------------|---------------------|
| F-PUSHDOWN-001 (HIGH) `is_first_step` gate | `is_first_step` field present in FetchContext; push-down gated on `is_first_step=true`; two-step CrowdStrike fixture test `test_BC_2_11_007_crowdstrike_two_step_step1_gets_pushdown_step2_does_not` asserts step-1 receives `limit=50` and step-2 does NOT receive any push-down params. | VERIFIED LOAD-BEARING — test is ungameable: requires both steps to execute on different `is_first_step` values. |
| F-PUSHDOWN-002 (MED) AC-005 two-path equivalence | `test_BC_2_11_007_push_down_result_equivalence_invariant` runs pipeline twice: once with push-down params set (limit=1, start_time), once with `FetchContext::default()`; asserts both produce same record-batch shape and row count. Wiremock responds identically to both. | VERIFIED LOAD-BEARING — test exercises the equivalence invariant directly, not merely the precondition. |
| F-PUSHDOWN-003 (MED) EC-005 warning log | `tracing::warn!` emitted for `start_time > end_time` case in `apply_push_down_to_request()`; `push_down.time_window_inverted` row added to BC-2.16.002 Structured Event Catalog (SAP-1); unit test `test_BC_2_11_007_ec005_inverted_time_window_emits_warning` uses `tracing_test::traced_test` to capture and assert the event_type field. | VERIFIED LOAD-BEARING — test fails if warning is removed; catalog row present. |
| F-PUSHDOWN-004 (LOW) Stale stub doc-comments | All `// TODO: implement push-down translation` and `// stub: returns req unchanged` comments removed from production implementation of `apply_push_down_to_request()`. | VERIFIED — zero stale stub markers in function body at a75fada4. |
| F-PUSHDOWN-005 (LOW) Cyberint page_size doc drift | Cyberint push-down handler comment updated: "Cyberint: time-window push-down via from_date/to_date in POST body. page_size is not a Cyberint API parameter." No over-claim present. | VERIFIED — doc claim removed; no behavior gap. |
| F-PUSHDOWN-006 (OBS) VP-031 mis-anchor | Story frontmatter `verification_properties: []`; VP-031 not referenced in story frontmatter or body. | VERIFIED — no VP-031 reference. |
| F-PUSHDOWN-007 (OBS) BC-2.11.005 indirect trace | Body note propagated: "BC-2.11.005: affected indirectly — push-down reduces materialized row count; not directly tested by a dedicated AC in this story." | VERIFIED — note present in Behavioral Contracts table. |
| F-PUSHDOWN-008 (OBS) BC-2.01.013 OUT_OF_SCOPE stale | BC-2.01.013 v1.12 authored by PO: OUT_OF_SCOPE annotation converted to affirmative; EC-01-027 + TV-006 re-cast; BC-INDEX row v5.83. | VERIFIED — v1.12 is current; no OUT_OF_SCOPE clause. |

All 8 pass-1 closures independently verified load-bearing at feature HEAD `a75fada4`.

---

## Findings

### F-PUSHDOWN2-MED-001 — MED — STORY-INDEX / story-file status asymmetry

**Severity:** MED
**Category:** index consistency / POLICY 13 (STORY-INDEX ↔ story-file sync)
**Status:** CLOSED — state-manager (this burst) story v1.2→v1.3; status `ready`→`in_progress`

**Description:** STORY-INDEX (v2.279) was updated by the D-1001 burst to badge S-DEMO-QUERY-PUSHDOWN-001 as `in_progress v1.2`. However the story FILE itself retained `status: ready` in frontmatter and `**Status:** ready` in the body header, as well as the v1.2 changelog entry noting "Body header version/status updated to v1.2/ready." This is a within-burst sibling-sweep asymmetry introduced by the D-1001 spec fix-burst: the index was updated to `in_progress` (correctly, per Source-of-Truth Rule 5 — active LOCAL cascade → `in_progress`) but the file was not. POLICY 13 requires STORY-INDEX ↔ story-file status parity.

**Fix:** State-manager status sync in this burst: story frontmatter `status: ready`→`in_progress`; body header `**Status:** ready`→`in_progress`; version bumped v1.2→v1.3; v1.3 changelog row added (monotonic descending per POLICY 32); body H1 label updated v1.2→v1.3. STORY-INDEX already shows `in_progress v1.2` — updated to `in_progress v1.3` (spec_version in sprint-state also updated v1.2→v1.3).

**Closure verified:** Story frontmatter `status: in_progress`; body header `**Status:** in_progress`; changelog monotonic descending (v1.3 > v1.2 > v1.1 > v1.0); STORY-INDEX parity restored (sprint-state spec_version v1.2→v1.3).

---

### OBS-1 — LOW — EC-005 `.expect()` on Option replaced with let-chain

**Severity:** LOW
**Category:** code quality / error-handling discipline
**Status:** CLOSED — implementer commit `688f82b5`

**Description:** In `apply_push_down_to_request()`, the EC-005 inverted-time-window path used `matches!(...).then(|| ...).expect("...")` pattern to extract the warning condition. This `.expect()` on an `Option` is semantically equivalent to an unreachable-assert, but the production-grade default requires explicit `if let` or `let-else` patterns in non-test code paths (CLAUDE.md §Conventions error-handling: no `expect()` on `Result` or `Option` in critical paths).

**Fix:** Implementer replaced with `if let Some(...) = ...` let-chain pattern. Equivalent behavior; no test changes required.

**Closure verified:** No `.expect()` on `Option` in `apply_push_down_to_request()` at commit `688f82b5`. `just check` 4001/4001 GREEN.

---

### OBS-2 — LOW — AC-005 mock-smoke-test rationale comment absent

**Severity:** LOW (OBS)
**Category:** test documentation / SID-1 §4 compliance
**Status:** CLOSED — implementer commit `688f82b5`

**Description:** The AC-005 result-equivalence test uses a Wiremock server (not a live DTU) to satisfy both the push-down and no-push-down execution paths. Per SID-1 §4, an `#[ignore]`'d integration test must include a code comment citing the blocking dependency. The AC-005 test is NOT `#[ignore]`'d (it runs against Wiremock), but the rationale for using Wiremock rather than the live DTU was not documented inline, making it unclear whether this was a deliberate SID-1-compliant decision or an accidental shortcut.

**Fix:** Implementer added a block comment above `test_BC_2_11_007_push_down_result_equivalence_invariant` explaining: "Uses Wiremock to exercise both push-down and no-push-down code paths without external DTU dependency. DTU-ext integration test with live sensor API is out-of-scope for LOCAL cascade; equivalence invariant is fully exercised at the API-request-generation layer. Per SID-1 §4: no `#[ignore]` flag because Wiremock is in-process; this is the production-grade substitute."

**Closure verified:** Comment present at commit `688f82b5`. No behavior change. `just check` 4001/4001 GREEN.

---

## Code Fix-Burst Summary

**Commit:** `688f82b5` on `feature/S-DEMO-QUERY-PUSHDOWN-001`
**Commit message:** `fix(S-DEMO-QUERY-PUSHDOWN-001): OBS-1 replace matches!+expect with if-let; OBS-2 add smoke-test rationale comment`
**just check result:** 4001/4001 GREEN (all tests pass, full workspace)

**Findings closed by code fix-burst:**
- OBS-1: EC-005 `.expect()` on Option → replaced with `if let` let-chain
- OBS-2: AC-005 mock-smoke-test rationale comment added

**Feature HEAD progression:** `a75fada4` → `688f82b5`

---

## Spec Fix-Burst Summary (this .factory/ burst)

**Findings closed by spec fix-burst:**
- F-PUSHDOWN2-MED-001: story status `ready`→`in_progress` (frontmatter + body header); version v1.2→v1.3; v1.3 changelog row added; STORY-INDEX spec_version updated v1.2→v1.3; sprint-state.yaml spec_version updated

---

## Convergence Trajectory

| Pass | Feature HEAD | Findings | Delta | CLEAN(strict) | CLEAN(PR-merge) | Streak |
|------|-------------|----------|-------|--------------|-----------------|--------|
| 1 | 19184786→a75fada4 | 8 (2H+2M+2L+2OBS) | — | no | no | 0/3 |
| 2 | a75fada4→688f82b5 | 3 (1M+1L+1OBS) | -5 | no | no | 0/3 |

**NEXT:** LOCAL adversary pass 3 against feature HEAD `688f82b5`. Pass 3 EXPECTED CLEAN (only LOW/OBS remain; F-PUSHDOWN2-MED-001 closed; OBS-1+OBS-2 closed by code). If CLEAN(strict)=yes → streak 1/3. Requires 2 more consecutive CLEAN passes for LOCAL 3/3 convergence.
