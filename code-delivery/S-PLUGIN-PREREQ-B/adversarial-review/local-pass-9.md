---
pass: 9
story: S-PLUGIN-PREREQ-B
head_sha: 411f4cbf
base_sha: 90d7c80f
factory_sha_at_pass: 10a8dd25
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 2M / 1L / 4O
adversary_run_date: 2026-05-11
novelty_score: 3/3 = 1.0
---

# Adversarial Review — Pass 9 (S-PLUGIN-PREREQ-B)

## Executive Summary

Fix-burst-8 closures are functionally CORRECT and the four new/rewritten tests are load-bearing per TD-VSDD-059 (verified by mental Red-Gate inversion). However, two new MEDIUM defects surface that block streak advancement:

1. **F-LP9-MED-001 — BC↔impl drift on auth audit signal.** BC-2.16.002 v1.6 postcondition (line 71) still says the executor emits "one of two" tracing events. The implementation has had THREE branches since fix-burst-7 (`auth_initial_acquired_empty` was added; fix-burst-8 wrote the asserting test). The BC was version-bumped to v1.6 for the partial-record discard postcondition but the v1.5 audit-signal row was NOT amended.

2. **F-LP9-MED-002 — Multi-array validator heuristic has false-negative gap.** The new Category 2b validator only classifies a prior step as "array-valued" if (pagination OR response_path ends with `[*]`). A common pattern — `response_path: "$.ids"` returning `{"ids":[1,2,3]}` without pagination — is NOT classified. A 4-step spec referencing TWO such non-paginated whole-array steps passes validation and hits the exact silent-stringification bug F-LP8-LOW-001 was filed to prevent.

Part A confirms all four fix-burst-8 closures CLEAN and LOAD-BEARING.

Streak does NOT advance. Pass-9 is **0/3** (unchanged).

## Part A — Closure Verification of fix-burst-8

| Finding | Closure status | Load-bearing? | Notes |
|---------|----------------|---------------|-------|
| F-LP8-MED-001 (empty-token tracing branch test) | **CLEAN** | YES | Sub-case (a) asserts `auth_initial_acquired` AND NOT `auth_initial_acquired_empty`. Sub-case (b) asserts `auth_initial_acquired_empty` AND that bare `auth_initial_acquired` does not appear (via replace + contains check). Merging arms would FAIL sub-case (b). Uses `set_default` thread-local subscriber isolation — correct for current_thread runtime. |
| F-LP8-MED-002 (rewritten partial-discard test) | **CLEAN** | YES | CursorToken pagination, `response_path: "$.items"`, is_final_step=true. Page-1 mock returns 2-record array → accumulates. Page-2 returns 500 → Err propagates. `matches!(result, Err(HttpRequestFailed{..}))` + status==500. Changing error path to Ok(records) would FAIL the matches assertion. |
| F-LP8-MED-003 (structured event_type on cursor warn) | **CLEAN** | YES | pipeline.rs:899-906 now has structured event_type/actual_type/cursor_preview. Test asserts the field. All 5 tracing::warn! in pipeline.rs now carry structured event_type. |
| F-LP8-LOW-001 (validator multi-array rejection) | **CLEAN-but-incomplete** | YES (within scope) | Test exercises paginated-array case. Validator heuristic has false-negative gap for non-paginated whole-array — see F-LP9-MED-002. |

All 4 closures CLEAN and LOAD-BEARING per TD-VSDD-059.

## Part B — New Dimension Findings

### F-LP9-MED-001 — BC-2.16.002 v1.6 postcondition row says "ONE OF TWO" tracing events; implementation emits THREE

- Severity: MEDIUM | Confidence: HIGH | Dimension: P9-G (BC↔impl drift)
- BC line 71 says "one of two" events: `auth_initial_acquired` info OR `auth_initial_failed` error.
- Impl since fix-burst-7 has THREE: non-empty → `auth_initial_acquired` info, empty → `auth_initial_acquired_empty` debug, Err → `auth_initial_failed` error.
- BC v1.6 changelog row only mentions partial-record discard, not audit-signal amendment.
- POL-7 violation: BC H1/postconditions are source of truth; impl drift means BC misleads SIEM/SOC authors building alerts.
- Evidence: `Grep('auth_initial_acquired_empty', .factory/specs/)` returns 0 matches; in worktree code returns 7 matches.
- **Recommendation:** Amend BC line 71 to enumerate three branches. Bump BC v1.6 → v1.7 with changelog citing F-LP9-MED-001.

### F-LP9-MED-002 — Multi-array validator heuristic false-negative gap: non-paginated whole-array response_path

- Severity: MEDIUM | Confidence: HIGH | Dimension: P9-F (sibling-sweep gap on F-LP8-LOW-001)
- validation.rs:259-265 classifies "array-valued" as `(pagination.is_some() || response_path.ends_with("[*]"))`.
- The existing fan-out test (line 775+) uses `pagination: None`, `response_path: "$.ids"` with response `{"ids":[...]}` → runtime fan-out triggers via `find_fan_out_array.is_array()` BUT validator heuristic misses this pattern.
- A 4-step spec with TWO such non-paginated whole-array steps feeding into a step that references both passes validation, then hits silent stringification at runtime — the exact bug F-LP8-LOW-001 was filed to prevent.
- find_fan_out_array (pipeline.rs:957-976) checks `step_vars.get(&key).filter(|v| v.is_array())` — runtime classifier disagrees with validator heuristic.
- Real-world example: CrowdStrike `/devices/queries/devices/v1` returns `{"resources":[...]}` — whole-array, no pagination.
- **Recommendation:** Option (b) preferred — runtime `tracing::warn!` with `event_type = "fanout_ambiguous_multi_array"` at find_fan_out_array when >1 array-valued variable found in step_vars. Add Red Gate test. Restores audit-signal completeness without rejecting legitimate single-array fan-out specs.

### F-LP9-LOW-001 — Dead mock setup in rewritten F-LP8-MED-002 test (cursor="" matcher cannot match)

- Severity: LOW | Confidence: HIGH | Dimension: P9-B (mock hygiene)
- pipeline_http_integration.rs:2072-2082 sets up `query_param("cursor", "")` matcher.
- pipeline.rs:809-817 CursorToken branch only appends `?cursor=c` if Some(c); never produces `?cursor=` empty.
- Mock 1 never matches; Mock 2 (no query matcher, up_to_n_times(1)) serves page-1; Mock 3 (cursor=abc) serves page-2.
- Non-blocking but worth flagging — future readers may misinterpret intent.
- **Recommendation:** Delete Mock 1 or convert to explicit `not(query_param_contains("cursor"))` assertion.

## Observations (non-blocking)

### OBS-LP9-001 — Cancellation safety: execute() is safe today; PREREQ-C parallel dispatch must re-audit

- Dimension: P9-C
- No Drop impl on PipelineExecutor; no tokio::spawn; all work on calling task. AuthToken does NOT zeroize (TD-S-PLUGIN-PREREQ-B-002, P3 acknowledged).
- File TD-S-PLUGIN-PREREQ-B-014 P3 for PREREQ-C cancellation audit.

### OBS-LP9-002 — Concurrent execute() with shared Arc<dyn AuthProvider> has no token-refresh serialization

- Dimension: P9-E (concurrency contract)
- AuthProvider trait does NOT specify single-flight refresh. Production CredentialStoreAuthProvider (PREREQ-D) must implement it.
- File TD-S-PLUGIN-PREREQ-B-015 P2 for PREREQ-D scope.

### OBS-LP9-003 — cursor_preview byte-slicing panics on UTF-8 boundary

- Dimension: P9-K (latent quality)
- pipeline.rs:891-898 — `s[..100]` panics mid-codepoint on multi-byte UTF-8 strings.
- Genuine bug, demoted to OBS because cursor values are typically ASCII and panic surfaces in test not silently.
- **Recommendation:** Bundle into fix-burst-9. Use `s.chars().take(100).collect::<String>()`.

### OBS-LP9-004 — [process-gap] BC changelog amendments do not enforce postcondition-row review

- Dimension: P9-G / [process-gap]
- Same root cause as OBS-LP8-004, different manifestation. Fix-burst-7's code-only fix created BC↔impl drift that survived 2 passes + a BC v1.6 amendment.
- **Recommendation:** Codify product-owner SOP: "audit-signal postcondition rows must enumerate every event_type emitted by matching code site; fix-burst that introduces a new event_type MUST amend the BC in the same burst."

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP9-MED-001 | MEDIUM | P9-G | BC↔impl drift | BC audit-signal row says "two events"; impl emits three | YES |
| F-LP9-MED-002 | MEDIUM | P9-F | sibling-sweep gap on F-LP8-LOW-001 | Validator misses non-paginated whole-array response_path | YES |
| F-LP9-LOW-001 | LOW | P9-B | mock hygiene | Dead Mock 1 in rewritten test | YES |
| OBS-LP9-001 | OBS | P9-C | cancellation safety | Safe today; PREREQ-C must re-audit | NO |
| OBS-LP9-002 | OBS | P9-E | concurrency contract | AuthProvider single-flight; PREREQ-D | NO |
| OBS-LP9-003 | OBS | P9-K | UTF-8 panic | cursor_preview byte-slicing | Bundle if open |
| OBS-LP9-004 | OBS+[process-gap] | P9-G | product-owner SOP | BC enumeration completeness | NO (process) |

## Process-Gap Findings

OBS-LP9-004 tagged [process-gap]. Same lineage as OBS-LP8-004. Codify product-owner SOP.

## Recommendations

Fix-burst-9 scope:
1. F-LP9-MED-001: Amend BC-2.16.002 line 71 to enumerate three audit events. Bump v1.6→v1.7.
2. F-LP9-MED-002: Add runtime warn `fanout_ambiguous_multi_array` at find_fan_out_array. Red Gate test.
3. F-LP9-LOW-001: Delete dead Mock 1 (or convert to explicit assertion).
4. OBS-LP9-003 (bundled): Fix UTF-8 boundary in cursor_preview using `chars().take(100).collect()`. Add multi-byte test.

After fix-burst-9, dispatch pass-10. Target: streak 0/3 → 1/3.

**Verdict: BLOCKED-soft. Streak 0/3. Fix-burst-9 required.**
