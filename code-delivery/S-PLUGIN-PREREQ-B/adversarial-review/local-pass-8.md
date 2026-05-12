---
pass: 8
story: S-PLUGIN-PREREQ-B
head_sha: ebd9a3ec
base_sha: 90d7c80f
factory_sha_at_pass: 412d584b
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 3M / 1L / 4O
adversary_run_date: 2026-05-11
novelty_score: 4/4 = 1.0
---

# Adversarial Review — Pass 8 (S-PLUGIN-PREREQ-B)

**Pass:** 8 (LOCAL, 8th in cycle)
**Story:** S-PLUGIN-PREREQ-B (Real PipelineExecutor)
**Worktree HEAD (target):** `ebd9a3ec`
**Base SHA (develop):** `90d7c80f`
**factory-artifacts HEAD:** `412d584b`
**Verdict:** **BLOCKED-soft** — `streak: 0/3` (does NOT advance to 1/3)
**Finding summary:** 0 CRIT / 0 HIGH / **3 MED** / 1 LOW / 4 OBS
**Trajectory:** 20 → 10 → 4 → 7 → 10 → 9 → 8 → 4
**Novelty self-assessment:** 4/4 = 1.0 (all new dimensions or substantive paper-fix detections, none retread prior passes)

## Executive Summary

Three of fix-burst-7's four closures are **paper-fix-detection failures** (TD-VSDD-059). Specifically: F-LP7-MED-001's tracing-event branching has no test asserting the branch fires; F-LP7-MED-003's "partial-record discard" test does not seed any records to discard (so the test trivially passes against pre-fix code). F-LP7-LOW-001's TD-012 doc-comment annotation is correctly filed. Part B finds one MEDIUM new dimension on async error propagation incompleteness in pagination cursor encoding and one LOW on cross-array fan-out semantics. Streak does **not** advance.

---

## Part A — Closure Verification of fix-burst-7

| Finding | Closure status | Notes |
|---------|----------------|-------|
| F-LP7-MED-001 (empty-token audit signal) | **PAPER-FIX** | Code branches correctly but ZERO tests assert the branch (F-LP8-MED-001 below) |
| F-LP7-MED-002 (FailingAuthProvider abort test) | **CLEAN** | `test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately` verified at `pipeline_oauth_retry.rs:284-323` with `.expect(0)` on data endpoint mock. Sibling synced. |
| F-LP7-MED-003 (partial-record discard) | **PAPER-FIX** | Test exists but does not exercise the discard semantic; step-1 produces a scalar (not accumulated records) (F-LP8-MED-002 below) |
| F-LP7-LOW-001 (execute_step doc TD-012) | **CLEAN** | TD-S-PLUGIN-PREREQ-B-012 filed at tech-debt-register v2.12; docstring annotation at `pipeline.rs:436-438` references the TD |

---

## Part B — Findings

### F-LP8-MED-001 — F-LP7-MED-001 closure is paper-fix: empty-token branch is untested

- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Dimension:** P8-D (closure verification) / TD-VSDD-059
- **Location:**
  - `crates/prism-spec-engine/src/pipeline.rs:143-174` (execute branch)
  - `crates/prism-spec-engine/src/pipeline.rs:466-500` (execute_step branch)
  - `crates/prism-spec-engine/tests/` (zero asserting tests)
- **Description:** Fix-burst-7 added a 3-way match (Ok-non-empty → `auth_initial_acquired` info; Ok-empty → `auth_initial_acquired_empty` debug; Err → `auth_initial_failed` error). Grep across the entire test directory finds ZERO occurrences of `auth_initial_acquired_empty` or `auth_initial_acquired` as test-side assertions. Only `auth_initial_acquired` appears in a comment at `pipeline_oauth_retry.rs:326`. The branch is observably correct by inspection, but per TD-VSDD-059 (paper-fix detection): every fix MUST have a test that fails against pre-fix code and passes against post-fix code. A reviewer cannot distinguish "fix is real" from "fix is paper" without that test. If a future refactor merges the two Ok arms back into one info-level emit, no test fails.
- **Evidence:**
  - `Grep('auth_initial_acquired_empty', tests/)` — zero matches
  - `Grep('auth_initial_acquired', tests/)` — one match, a comment (line 326)
  - No test uses `tracing-subscriber` with the empty-token path (the existing tracing-subscriber test at `pipeline_http_integration.rs:1840` asserts `pipeline_truncated`, not auth events)
- **Recommendation (FIX REQUIRED — blocks streak):** Add `test_BC_2_16_002_auth_initial_acquired_empty_emits_debug_not_info` using the existing `tracing-subscriber` log-buffer harness (same pattern as `pipeline_http_integration.rs:1840`). Run two cases: (a) `MockAuthProvider::new("real-token")` → assert log contains `auth_initial_acquired` and NOT `auth_initial_acquired_empty`; (b) `NullAuthProvider` → assert log contains `auth_initial_acquired_empty` and NOT a bare `auth_initial_acquired` info entry. Without this test, F-LP7-MED-001 is regression-vulnerable.

### F-LP8-MED-002 — F-LP7-MED-003 closure is paper-fix: partial-discard test cannot fail against pre-amendment code

- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Dimension:** P8-D (closure verification) / TD-VSDD-059
- **Location:** `crates/prism-spec-engine/tests/pipeline_http_integration.rs:1869-1957` (`test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500`)
- **Description:** The test's setup is:
  - Step-1: `GET /step1` → 200 with body `{"token":"tok-abc"}`, `response_path: "$.token"` (scalar)
  - Step-2: `GET /step2` → 500

  Tracing the pre-fix execute() logic: step-1's `extract_at_path` returns a scalar (`"tok-abc"`), which the match at `pipeline.rs:345-358` falls into the `scalar` arm where the comment explicitly says "Never added to all_records regardless of step position." Thus `all_records` is **empty before step-2 even runs**. When step-2 returns 500, the pipeline propagates Err, and `all_records` was already `[]`. **The "discard" semantic is never exercised** because there is nothing to discard.

  Per TD-VSDD-059: this test must fail against pre-fix code. There is no "pre-fix code" — the BC v1.5 → v1.6 amendment is a documentation clarification; the code already returned Err on a non-2xx step response (verified at `pipeline.rs:718-725`). The test asserts only `matches!(result, Err(SpecEngineError::HttpRequestFailed { .. }))` which would have passed equally against v1.5 BC and against the pre-amendment code. The test does NOT test "all_records was nonzero and was then discarded" — that scenario is structurally absent.
- **Evidence:**
  - `pipeline_http_integration.rs:1913` — `response_path: "$.token"` produces scalar
  - `pipeline.rs:347-357` — scalar arm skips `all_records.extend`; intermediate-step records are never accumulated anyway
  - Test only asserts Err type (line 1949-1953), with the comment "structurally guaranteed by (a) above" (line 1955) — the author appears to have noticed this themselves
  - The user pass-8 brief explicitly requested: "(a) seeds step-1 with two pages of valid records, (b) makes step-2 return 500 on the second request, (c) asserts the final Vec<Value> is EMPTY". None of (a)/(b)/(c) is satisfied as the brief describes.
- **Recommendation (FIX REQUIRED — blocks streak):** Rewrite the test to actually exercise the discard semantic. Required structure:
  - Make step-1 a 2-page paginated step (CursorToken or OffsetLimit) returning an ARRAY of records — at least 4 total — and mark step-1 as the FINAL step (or use a single-step pipeline with mid-pagination 500).
  - Better: use a 2-step pipeline where step-1 is final and returns `{"items":[{r1},{r2}], "cursor":"x"}` (so `all_records` accumulates 2 records on page 1), then step-1 page-2 returns 500. Assert that `execute()` returns Err AND that no Ok with partial records is constructed.
  - Even better and matching BC text exactly: 2 successful records accumulated, then HTTP failure, assert Err. The current test cannot detect a regression where a developer changes `execute()` to construct `Ok(PipelineResult { records: accumulated_so_far, ... })` on error — because in this test `accumulated_so_far` is always `[]`.

### F-LP8-MED-003 — Cursor pagination cursor is double-extracted from raw body but escape logic skips array-valued cursors silently

- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Dimension:** P8-A (failure-recovery — non-array cursor degradation)
- **Location:** `crates/prism-spec-engine/src/pipeline.rs:874-890` (`extract_cursor`)
- **Description:** The `extract_cursor` helper handles `String`, `Number`, `Null` cleanly. For `Array`/`Object`/`Bool` cursor values it logs a `tracing::warn!` diagnostic and returns `None`, terminating pagination. This is the documented behavior. But the diagnostic warn does NOT include the actual cursor value (only `?other` Debug format, which for non-string types serializes a partial JSON shape), and the pipeline silently stops fetching further pages without emitting an error. If a sensor's API contract evolves and starts returning a structured cursor `{"next":"abc"}` instead of a string, the client will receive only page-1 data without ANY error to the operator. This is a silent data-loss vector.

  Specifically: `tracing::warn!` events without a structured `event_type` field are difficult to grep against the standard audit signal family (`auth_*`, `pipeline_*`). There is no `pagination_terminated_unexpected_type` event_type for SIEM/SOC to alert on.
- **Evidence:** `pipeline.rs:880-888` — warn-only path, no `event_type` field. Compare to `pipeline.rs:362-370` (pipeline_truncated event_type) for the project's established audit-event pattern. The function is reachable when a spec author uses `pagination_cursor_path` that resolves to an unexpected type — a spec-level mistake that should surface louder than a warn.
- **Recommendation (FIX REQUIRED — blocks streak):** Either (a) promote to an Err with `SpecEngineError::HttpRequestFailed{detail="cursor type unsupported: {actual_type}"}` to fail-fast (matches the cursor-non-advance treatment at line 390-395, which is the analogous infinite-loop guard), OR (b) keep the warn but add structured `event_type = "pagination_cursor_unsupported_type"` so audit pipelines can alert. The current bare-warn is inconsistent with the project's audit-signal discipline established in BC v1.5/v1.6 amendments.

### F-LP8-LOW-001 — `find_fan_out_array` returns first array only — multi-array template silently uses one

- **Severity:** LOW
- **Confidence:** HIGH
- **Dimension:** P8-A (failure-recovery / fan-out semantics)
- **Location:** `crates/prism-spec-engine/src/pipeline.rs:937-956` (`find_fan_out_array`)
- **Description:** If a step's template references TWO arrays from prior steps — e.g., `path_template = "/api/${stepA.ids}/details/${stepB.ids}"` where both `stepA.ids` and `stepB.ids` are arrays — `find_fan_out_array` returns only the first match (whichever iterator finds first across path_template then body_template references). The other array gets stringified via `value_to_string` on the JSON Array variant (`"[1,2,3]"`) and percent-encoded. The result is each batch's URL has `/api/{batch-N}/details/%5B1%2C2%2C3%5D`. No error, no warning — but the result is almost certainly not what the spec author intended (which is likely cartesian fan-out or zipped fan-out).

  This is not the same as F-LP2-HIGH-001 (which was about the first array's source key being overridden); this is a NEW dimension where multiple arrays exist simultaneously. The validator at `validation.rs:218-239` checks variable references resolve to known steps but does NOT check the resulting type interaction.
- **Evidence:** `pipeline.rs:946-953` returns at first match. `interpolation.rs:212-218` `value_to_string`: `other => other.to_string()` produces the JSON serialization for Array variants (e.g., `[1,2,3]`).
- **Recommendation:** (a) Add a validator check in `validation.rs` that rejects specs where the same step's templates reference multiple array-valued variables from prior steps; OR (b) emit a `tracing::warn!` with `event_type = "fanout_ambiguous_multi_array"` at runtime when `find_fan_out_array` would have multiple matches. Per user mandate "no defer-with-surfacing without substantial justification" — FIX, not defer.

---

## Observations (non-blocking)

### OBS-LP8-001 — `request_count` increment placement is inconsistent under retry abort

- **Dimension:** P8-C (async error propagation)
- **Location:** `pipeline.rs:623, 677`
- **Description:** `*request_count += 1` is incremented before the status check on both the initial and retry requests, but the function returns Err on certain status checks AFTER incrementing. This is correct (HTTP request was issued, so the count is accurate), but a reader checking BC-2.16.002's v1.5 postcondition ("`request_count` is the number of HTTP requests issued by the pipeline steps") might wonder whether a failed request counts. The BC is silent on this micro-question. No action required; flagged only for spec clarity in PREREQ-C/D.

### OBS-LP8-002 — `boot.rs` consumes `prism-spec-engine` for `ConfigManager` only, not `PipelineExecutor`

- **Dimension:** P8-B (cross-borough boot-sequence interaction / POL-15)
- **Location:** `crates/prism-bin/src/boot.rs:497-530` (step4_load_sensor_specs)
- **Description:** Verified: `prism-bin/src/boot.rs` instantiates `ConfigManager` (via `parse_spec_directory`) but never references `PipelineExecutor`, `AuthProvider`, or any pipeline execution type. POL-15 (`runtime_wiring_required_for_accepted_adrs`) requires that ADR-023 §C2 (accepted) be reachable from a production binary. Currently it is NOT — `PipelineExecutor::execute` has zero production callers. However, the story explicitly defers this to PREREQ-D (story §94-101) and the story `status: draft`. POL-15 does not fire until the story flips to `merged` or `partial-merge`. **Non-blocking** for pass-8 but **must** be re-evaluated at the PR-merge gate. Recommend the wave-gate adversary track this as a "POL-15 pending PREREQ-D" sentinel.

### OBS-LP8-003 — `execute_step` has zero call sites in the entire workspace

- **Dimension:** P8-B / P8-D (dead-write / production residue)
- **Location:** `crates/prism-spec-engine/src/pipeline.rs:453` (definition); ripgrep across workspace shows callers = 0
- **Description:** Confirmed via Grep: `PipelineExecutor::execute_step` is invoked by exactly zero callers (test or production). The function exists, is `pub`, has fully-fleshed implementation with eager-token, but is unreachable. TD-S-PLUGIN-PREREQ-B-012 (P3, filed in burst-7) acknowledges this. The function ALSO contains an unused `request_count` local at `pipeline.rs:501` that is incremented inside `issue_request_with_retry` but never returned to the caller — dead-write of computed state. Per F-LP7-MED-003-equivalent paper-fix concern: nothing exercises the `auth_initial_acquired_empty` branch in `execute_step` either (since nothing exercises `execute_step` at all). Confirmation that fix-burst-6's F-LP6-MED-003 sibling-fix (symmetric eager-token in execute_step) and fix-burst-7's mirrored empty-token branch are correct by inspection but are paper-fix-vulnerable as a category. Non-blocking because TD-012 explicitly accepts this gap.

### OBS-LP8-004 — `[process-gap]` BC v1.6 amendment landed without an enforcing test

- **Dimension:** P8-J (workspace ripple) / `[process-gap]`
- **Location:** `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` v1.6 (changelog entry)
- **Description:** `[process-gap]` BC-2.16.002 was amended at v1.6 to add a "partial-record discard" postcondition. The story changelog and BC changelog both claim "F-LP7-MED-003 closure". But as F-LP8-MED-002 demonstrates, the closing test does not actually exercise the new postcondition — yet the BC was amended and version-bumped. This represents a process gap: the product-owner/state-manager workflow should verify that BC postcondition amendments have at least one Red Gate test that would fail without the postcondition being implemented. Recommend codifying as `product-owner` agent SOP: "when adding/modifying a postcondition, identify the Red Gate test that would fail without it, and cite the test name in the BC changelog row." This would have caught F-LP8-MED-002 at the product-owner step rather than at the adversary pass.

---

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP8-MED-001 | MEDIUM | P8-D | paper-fix-detection (TD-VSDD-059) | F-LP7-MED-001 empty-token branch has zero test coverage | YES |
| F-LP8-MED-002 | MEDIUM | P8-D | paper-fix-detection (TD-VSDD-059) | F-LP7-MED-003 partial-discard test does not seed records to discard | YES |
| F-LP8-MED-003 | MEDIUM | P8-A | audit-signal discipline / silent data loss | extract_cursor non-string types terminate silently without structured event_type | YES |
| F-LP8-LOW-001 | LOW | P8-A | fan-out semantics | Multi-array template silently uses only the first array | YES |
| OBS-LP8-001 | OBS | P8-C | spec clarity | request_count semantics on failed retry unclear in BC | NO |
| OBS-LP8-002 | OBS | P8-B | POL-15 sentinel | boot.rs does not instantiate PipelineExecutor — deferred to PREREQ-D | NO |
| OBS-LP8-003 | OBS | P8-B/D | dead-write | execute_step has zero call sites; accepted via TD-012 | NO |
| OBS-LP8-004 | OBS+`[process-gap]` | P8-J | product-owner SOP | BC postcondition amendment without enforcing test allowed by current workflow | NO (process) |

---

## Process-Gap Findings

OBS-LP8-004 is tagged `[process-gap]`. Recommend orchestrator route to cycle-closing checklist for product-owner agent SOP codification: BC postcondition amendments require a cited Red Gate test that would fail without the amendment.

---

## Novelty Self-Check Table (vs prior passes)

| Prior dimension class | Pass(es) | Pass-8 overlap? |
|----------------------|----------|-----------------|
| P5-A reqwest gzip | 5 | none |
| P5-B audit-log symmetry | 5 | none (F-LP8-MED-003 is novel sub-dimension: structured event_type on warn-only paths) |
| P5-J dollar-dot path | 5 | none |
| P5-LOW-003 lazy-token | 5 | none |
| P6-A NullAuth public-API | 6 | confirmed clean (test-helpers feature gate) |
| P6-J VP-PLUGIN-005 frontmatter | 6 | confirmed clean |
| P6-MED-003 execute_step sibling | 6 | OBS-LP8-003 acknowledges deferred status (TD-012) |
| P7-MED-001 empty-token | 7 | F-LP8-MED-001 NEW paper-fix detection |
| P7-MED-002 FailingAuth abort | 7 | confirmed clean (Part A) |
| P7-MED-003 partial-discard | 7 | F-LP8-MED-002 NEW paper-fix detection |
| P7-LOW-001 execute_step TD-012 | 7 | confirmed clean (Part A) |
| P8-A fan-out multi-array | 8 | F-LP8-LOW-001 NEW |
| P8-A cursor non-string | 8 | F-LP8-MED-003 NEW |

All 4 actionable Part B findings are in fresh dimensions or are paper-fix-detection on fix-burst-7 closures. Zero retread of prior passes. Per TD-VSDD-059 ethos, paper-fix detections are the most-load-bearing class of finding for a "production-grade closure, no shortcuts" project rubric.

---

## Recommendations

### Streak verdict

Streak does NOT advance. Pass-8 is **0/3** (unchanged). Three actionable MED findings block.

### Fix-burst-8 scope (proposed)

1. **F-LP8-MED-001**: Add `test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state` with tracing-subscriber harness asserting both branches (empty-token → `auth_initial_acquired_empty` debug; non-empty → `auth_initial_acquired` info).
2. **F-LP8-MED-002**: Rewrite `test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500` to seed step-1 as the FINAL step with cursor pagination returning 2 valid records on page-1 and HTTP 500 on page-2. Assert `result.is_err()` AND that no `PipelineResult` with `records.len() > 0` is constructible from the error path.
3. **F-LP8-MED-003**: Add structured `event_type = "pagination_cursor_unsupported_type"` field to the bare-warn at `pipeline.rs:880-887` and add a Red Gate test asserting the event fires when cursor resolves to an Array/Object/Bool.
4. **F-LP8-LOW-001**: Fix — either validator rejection or structured warn event_type at runtime. Per user mandate, no defer.

After fix-burst-8, dispatch pass-9 to retest closures and continue toward 3/3 CLEAN streak.

### Sibling-site sweep (TD-VSDD-060) — verified

- F-LP7-MED-001 sibling at `pipeline.rs:481` (execute_step) — present and mirrored. OK by inspection.
- F-LP7-MED-002 — `FailingAuthProvider` is feature-gated under `cfg(any(test, feature = "test-helpers"))` symmetric with `NullAuthProvider`/`MockAuthProvider`. OK.
- F-LP7-MED-003 — BC v1.6 amendment applies workspace-wide; no other BC in `subsystem: SS-16` has a partial-failure semantic that drifts (verified by grep). OK.
- F-LP7-LOW-001 — TD-012 single-site annotation; no other un-wired pub functions in `prism-spec-engine` need the same annotation (PluginRuntime etc. are pre-existing). OK.

**Verdict: BLOCKED-soft. Streak remains 0/3. Fix-burst-8 required before pass-9.**
