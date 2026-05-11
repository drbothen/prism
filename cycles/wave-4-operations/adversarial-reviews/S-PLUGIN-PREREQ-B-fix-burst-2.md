---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: "2026-05-11T20:30:00Z"
phase: 3
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/error.rs"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-B-pass-2.md"
input-hash: "b851051"
traces_to: ".factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md"
pass: 2
previous_review: "S-PLUGIN-PREREQ-B-pass-2.md"
review_level: LOCAL
target_artifact: S-PLUGIN-PREREQ-B
fix_burst_for_pass: 2
target_sha: a6895d7a
base_sha: 7511e749
verdict: CLOSED
finding_summary_closed:
  critical: 0
  high: 2
  medium: 3
  low: 3
  obs_acknowledged: 2
prior_passes: "pass-2 BLOCKED-hard 10 findings (0C+2H+3M+3L+2O)"
---

# Adversarial Review: S-PLUGIN-PREREQ-B Fix-Burst-2 Closure Report (Pass 2 Remediation)

**Verdict: CLOSED** — All 8 actionable findings from LOCAL pass-2 closed at implementer commit
`a6895d7a`. 2 OBS items acknowledged. Streak stays 0/3 (fix-bursts do not advance streak;
adversary pass-3 advances streak). STATE+HANDOFF v7.135→v7.136. Story v1.1→v1.2
(red_gate_tests 16→29). 2 new TDs filed: TD-S-PLUGIN-PREREQ-B-003 P3 +
TD-S-PLUGIN-PREREQ-B-004 P3.

## Finding ID Convention

Finding IDs for this closure report use the pass-2 source format: `F-LP2-<SEV>-<SEQ>`
(LOCAL pass-2 findings from S-PLUGIN-PREREQ-B-pass-2.md). This file records closure
dispositions only — no new adversary findings are raised here.

## Part A — Fix Verification (pass >= 2 only)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP2-HIGH-001 | HIGH | RESOLVED | `find_fan_out_array` now returns `(String, Value)` key+value tuple; fan-out loop overrides `batch_step_vars[source_key]` with batch slice — paper-fix regression eliminated. Paper-fix-proof test `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls` asserts 3 distinct query strings each <700 chars. |
| F-LP2-HIGH-002 | HIGH | RESOLVED | `MAX_PAGES_PER_STEP=1000` at `pipeline.rs:37`; page-cap check at `pipeline.rs:207-218`; prev_cursor non-advance guard at `pipeline.rs:315-322`. New test `test_BC_2_16_002_execute_aborts_on_non_advancing_cursor`. |
| F-LP2-MED-001 | MEDIUM | ACKNOWLEDGED | red_gate_tests count drift corrected in story v1.2 frontmatter: 16→29 canonical-name grep (+ 1 error.rs unit = 30 total). State-manager scope item — no code change required. |
| F-LP2-MED-002 | MEDIUM | RESOLVED | `pipeline.rs:615` — `trimmed.starts_with('[')` added to JSON branch; Content-Type now correctly derived for JSON arrays. New test `test_BC_2_16_002_execute_derives_application_json_for_array_body`. |
| F-LP2-MED-003 | MEDIUM | RESOLVED | `pipeline.rs:703` — `Value::Number(n) => Some(n.to_string())` coercion added; numeric cursors no longer silently fail. New test `test_BC_2_16_002_execute_coerces_numeric_cursor_to_string`. |
| F-LP2-LOW-001 | LOW | RESOLVED | `error.rs:137-158` — `test_auth_acquisition_failed_error_constructs` in `#[cfg(test)] mod tests` exercises variant construction + Display. |
| F-LP2-LOW-002 | LOW | RESOLVED | `pipeline.rs:682-686` — per-segment `~`→`~0`, `/`→`~1` escaping before join. Inline TD-S-PLUGIN-PREREQ-B-003 P3 comment added for PREREQ-C bracket/wildcard scope. |
| F-LP2-LOW-003 | LOW | RESOLVED | `pipeline.rs:19` — `percent_encoding` use hoisted to module top; closure hygiene corrected. |
| OBS-LP2-001 | OBS | ACKNOWLEDGED | Fan-out resource bound: partial coverage via HIGH-002 fix (MAX_PAGES_PER_STEP). Full cumulative bound filed as TD-S-PLUGIN-PREREQ-B-004 P3; defer to PREREQ-D or post-keystone hardening. |
| OBS-LP2-002 | OBS | ACKNOWLEDGED | red_gate count automation process-gap; links to existing PG-LP7-002 codification. No new TD filed. |

## Part B — New Findings (or all findings for pass 1)

No new findings raised in this closure report. This is a fix-burst closure document only.
All findings from pass-2 are dispositioned in Part A above.

### CRITICAL

None.

### HIGH

None.

### MEDIUM

None.

### LOW

None.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** pass — fix-burst closure (no new findings)
**Convergence:** Findings remain (streak 0/3) — pass-3 required to advance streak
**Readiness:** Fix-burst-2 complete; pass-3 dispatch next against HEAD `a6895d7a`

### New TDs Filed This Burst

| TD ID | Priority | Description |
|-------|----------|-------------|
| TD-S-PLUGIN-PREREQ-B-003 | P3 | `extract_at_path` JSON Pointer dot-notation only; bracket/wildcard deferred. PREREQ-C scope. |
| TD-S-PLUGIN-PREREQ-B-004 | P3 | `MAX_REQUESTS_PER_PIPELINE` cumulative bound unimplemented. Per-step cap via HIGH-002 fix. Defer to PREREQ-D or post-keystone hardening. |

### Test Coverage Added

- **New Red Gate tests this burst:** 5
  - `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls` (paper-fix-proof)
  - `test_BC_2_16_002_execute_aborts_on_non_advancing_cursor`
  - `test_BC_2_16_002_execute_derives_application_json_for_array_body`
  - `test_BC_2_16_002_execute_coerces_numeric_cursor_to_string`
  - `test_auth_acquisition_failed_error_constructs` (error.rs unit)
- **Red Gate count:** 16 → 29 canonical-name grep (+ 1 error.rs unit = 30 total)
- **Full crate tests:** 263/263 pass
- **Workspace:** builds clean

### Cascade Trajectory

| Pass | Findings | Status |
|------|---------|--------|
| pass-1 | 20 (4C+6H+5M+2L+3O) | BLOCKED-hard — REMEDIATED at fix-burst-1 (`7511e749`) |
| fix-burst-1 | 12 closed + 2 TDs | CLOSED at `7511e749` |
| pass-2 | 10 (0C+2H+3M+3L+2O) | BLOCKED-hard — REMEDIATED at fix-burst-2 (`a6895d7a`) |
| fix-burst-2 | 8 closed + 2 OBS acknowledged + 2 TDs | **CLOSED at `a6895d7a`** |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 (fix-burst closure) |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | N/A (fix-burst closure report) |
| **Median severity** | N/A |
| **Trajectory** | 20 → 10 (pass-1 → pass-2); fix-burst-2 closes 8 of 10 actionable |
| **Verdict** | FINDINGS_REMAIN — streak 0/3; pass-3 required to advance |
