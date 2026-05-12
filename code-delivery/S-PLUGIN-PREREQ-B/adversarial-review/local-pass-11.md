---
pass: 11
story: S-PLUGIN-PREREQ-B
head_sha: 01df68cd
base_sha: 90d7c80f
factory_sha_at_pass: 33d384ed
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 2M / 1L / 4O
adversary_run_date: 2026-05-11
novelty_score: 7/7 = 1.0
---

# Adversarial Review — LOCAL Pass 11 (S-PLUGIN-PREREQ-B)

## Executive Summary

Fix-burst-10 closures CLEAN and load-bearing per TD-VSDD-059. All four closures (F-LP10-MED-001 truncate_at_char_boundary + BROAD sibling sweep, F-LP10-MED-002 Object-warn, F-LP10-LOW-002 visibility, F-LP10-LOW-001 TD-016 deferral) verified. BROAD sweep was thorough; spec_parser.rs:383 bonus defensive fix legitimate.

Pass-11 surfaces TWO new MEDIUM defects and ONE LOW that block streak:

1. **F-LP11-MED-001 — BC↔impl drift on structured event catalog (recurrence of F-LP9-MED-001 pattern for non-auth events).** BC-2.16.002 v1.7 documents 7 audit events (auth_initial_* and auth_refresh_* families). Implementation emits 11 distinct structured event_types — 4 NOT documented: pipeline_truncated, pagination_cursor_unsupported_type, fanout_invalid_source_type, fanout_ambiguous_multi_array.

2. **F-LP11-MED-002 — auth_initial_* field-schema drift between execute() and execute_step().** Both emit same event_type but execute_step adds `step_name` field that execute does NOT. BC documents only `sensor_id`, `client_id`.

3. **F-LP11-LOW-001 — truncate_at_char_boundary has zero direct unit tests.** Security-relevant helper exercised only through one integration test. Edge cases untested.

Streak does NOT advance. Pass-11 is 0/3 (unchanged).

## Part A — Closure Verification of fix-burst-10

| Finding | Closure status | Load-bearing? | Notes |
|---------|----------------|---------------|-------|
| F-LP10-MED-001 (validation.rs:126 byte-slice) | CLEAN | YES | truncate_at_char_boundary at validation.rs:26-31 via char_indices().nth(max_chars). Test exercises 60×🎯 panic vector. BROAD sibling sweep verified — 7 sites: 2 FIXED + 5 EXEMPT. Extended sweep to split_at/.get(..N)/range patterns — no additional sites. |
| F-LP10-MED-002 (find_fan_out_array Object warn) | CLEAN | YES | pipeline.rs:986-1009 Strategy-A. Test asserts fanout_invalid_source_type structured event. |
| F-LP10-LOW-002 (visibility) | CLEAN | YES | auth_provider.rs:146-228 — token/call_count private with getters. FailingAuthProvider parallel. No test writes .token. |
| F-LP10-LOW-001 (TD-016) | CLEAN | N/A | tech-debt-register v2.13 cites TD-016 P2 with substantial justification. |

**Result:** All four CLEAN, load-bearing, discipline-compliant. Coverage completeness audit verified Value::Bool/Null/Number stringification is intentional scalar interpolation — no missing warn.

## Part B — New Dimension Findings

### F-LP11-MED-001 — BC↔impl drift on structured event catalog

- Severity: MEDIUM | Confidence: HIGH | Dimension: P11-B (audit trail completeness)
- 11 event_types emitted in pipeline.rs; 4 NOT documented in BC-2.16.002 v1.7:
  - pipeline.rs:363 `pipeline_truncated` — NOT documented
  - pipeline.rs:902 `pagination_cursor_unsupported_type` — NOT documented
  - pipeline.rs:999 `fanout_invalid_source_type` — NOT documented (added fix-burst-10)
  - pipeline.rs:1025 `fanout_ambiguous_multi_array` — NOT documented (added fix-burst-9)
- BC v1.7 fixed only auth_initial_* enumeration; non-auth events shipped fix-burst-9/-10 introduced the gap. SIEM/SOC pipeline authors cannot enumerate contract surface from BC alone.
- Same pattern as F-LP9-MED-001 — fix added new event without backfilling BC. Recurrent pattern (N=2) → PG-LP11-001.
- **Recommendation:** Amend BC-2.16.002 v1.7 → v1.8 — add "Structured Event Catalog" enumerating all 11 event_types with fields, levels, trigger conditions.

### F-LP11-MED-002 — auth_initial_* field-schema drift

- Severity: MEDIUM | Confidence: HIGH | Dimension: P11-B (event field-schema consistency)
- pipeline.rs:145-150 (execute): `tracing::info!(event_type = "auth_initial_acquired", sensor_id, client_id, "...")` — 2 fields
- pipeline.rs:468-474 (execute_step): same event_type but ADDS `step_name = %step.name` — 3 fields
- Same divergence for auth_initial_acquired_empty and auth_initial_failed.
- BC v1.7 documents execute()'s field-schema only; execute_step is "mirror entry point" with hidden field-schema diff.
- **Recommendation:** Amend BC v1.8 to document the field-schema difference (lower-risk than removing step_name which would break SIEM consumers).

### F-LP11-LOW-001 — truncate_at_char_boundary has zero direct unit tests

- Severity: LOW | Confidence: HIGH | Dimension: P11-A (helper coverage)
- validation.rs:26-31 helper introduced fix-burst-10. No direct unit tests; only exercised via integration test with max_chars=200 + 60×🎯.
- Untested edges: empty + 0, empty + max>0, ASCII boundary, exactly-max-chars, max > length.
- Code reading confirms correctness, but a future refactor (e.g., `split_at_checked`, `s.get(..idx).unwrap_or(s)`) could silently change behavior.
- **Recommendation:** Add `#[cfg(test)] mod tests` block with 5 unit tests. ~15 lines.

## Observations (non-blocking)

### OBS-LP11-001 — extract_at_path silently accepts malformed paths

- Dimension: P11-I (JSON Pointer edge cases)
- pipeline.rs:845-866 — `$.foo..bar` produces `/foo//bar` (RFC 6901 empty key); `$.foo/bar` produces `/foo~1bar` (literal `/`).
- Validator doesn't catch these; runtime returns "path not found." Validator-side rejection would surface at spec-load.
- **Recommendation:** Bundle into TD-003 scope (existing PREREQ-C TD).

### OBS-LP11-002 — pipeline_http_integration.rs is 2832 lines

- Dimension: P11-G (test discoverability)
- Recommend split into thematic files (audit-log, fanout, pagination, interpolation, error-discard) at PREREQ-D.

### OBS-LP11-003 — public API surface differs by cargo feature

- Dimension: P11-H
- test-helpers feature adds 3 public types. Gating is correct; doc gap. Non-blocking.

### OBS-LP11-004 — worktree drift minimal

- Dimension: P11-D
- prism-spec-engine doesn't directly use prism_core::SensorId. No rebase blockers.

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP11-MED-001 | MEDIUM | P11-B | BC↔impl drift recurrence | 4 events emitted but not in BC | YES |
| F-LP11-MED-002 | MEDIUM | P11-B | field-schema drift | step_name in execute_step but not execute, neither documented | YES |
| F-LP11-LOW-001 | LOW | P11-A | helper coverage | truncate_at_char_boundary zero unit tests | YES |
| OBS-LP11-001 | OBS | P11-I | extract_at_path edges | Bundle into TD-003 | NO |
| OBS-LP11-002 | OBS | P11-G | test file size | Split at PREREQ-D | NO |
| OBS-LP11-003 | OBS | P11-H | feature drift doc | Defer | NO |
| OBS-LP11-004 | OBS | P11-D | worktree drift | No action | NO |

## Process-Gap Findings

### PG-LP11-001 — [process-gap] Pattern: new event_type sites repeatedly miss BC backfill

- Recurrence: 2 (F-LP9-MED-001 closed auth events; F-LP11-MED-001 same pattern for non-auth events).
- **Recommendation:** Codify SOP rule: "Any new `tracing::*!(event_type = ...)` site in pipeline.rs/auth_provider.rs/validation.rs/interpolation.rs MUST trigger a BC-2.16.002 audit-row update in the same burst." Add to burst-closure checklist.

## Recommendations

### Fix-burst-11 scope (REQUIRED)

1. F-LP11-MED-001 + F-LP11-MED-002: Amend BC-2.16.002 v1.7 → v1.8 with comprehensive Structured Event Catalog enumerating all 11 event_types + documenting field-schema differences between execute() and execute_step().
2. F-LP11-LOW-001: Add 5 unit tests for truncate_at_char_boundary in validation.rs.
3. PG-LP11-001 codification: Update project burst SOP / codify in lessons-learned doc.

After fix-burst-11, dispatch pass-12 targeting streak 0/3 → 1/3.

## Novelty Self-Check

7/7 = 1.0. All findings on dimensions never previously examined or extending existing dimensions with new evidence.

**Verdict: BLOCKED-soft. Streak 0/3. Fix-burst-11 required.**
