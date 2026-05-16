---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 36
scope: spec
verdict: BLOCKED
total_findings: 3
severity_breakdown:
  critical: 0
  high: 0
  medium: 3
  low: 0
  observation: 0
in_scope_findings: 3
observations_queued: 0
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-28-pending
fix_burst_closed_at: pending
streak_after_pass: "0/3"
streak_before_pass: "1/3"
streak_reset: true
novelty: HIGH (3 NEW defect axes surviving 35 prior passes including 8 CLEAN: test-naming drift + Red Gate coverage gap + STORY-INDEX column drift)
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 36

**Verdict: BLOCKED — 3 in-scope MEDIUM findings. Streak RESETS 1/3 → 0/3 (6th reset).**

Pass-35→pass-36 RESET. 3 NEW defect axes that survived 35 prior passes including 8 CLEAN passes.

## F-LP36-MED-001 — AC-9 ↔ Red Gate Test 8 test-name drift

**Severity:** MEDIUM
**Evidence:**
- Story line 239 AC-9: `test_BC_2_16_012_write_tool_invalidation_runtime_register`
- Story line 273 Red Gate Test 8: `test_BC_2_16_012_003_write_tool_invalidation_runtime_register`

Same test, two names. Rust treats them as different identifiers. Implementer following Red Gate creates `_003_` variant; AC-9 verification fails.

**Fix:** product-owner — canonicalize both names to Red Gate convention (`_003_`).

## F-LP36-MED-002 — AC-8 ↔ Red Gate Tests 6+7 coverage gap

**Severity:** MEDIUM
**Evidence:**
- Story line 235 AC-8: integration test `test_BC_2_16_012_spec_parser_behavioral_equivalence` covers 4 sensors + novel name
- Story line 269 Red Gate Test 6: `test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch` (novel-name only)
- Story line 271 Red Gate Test 7: `test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike` (CrowdStrike only)

No Red Gate covers AC-8's 4-sensor scope. "Tests MUST fail before implementation" rule violated.

**Fix:** product-owner — Option A (expand Red Gate 6+7 to 4 sensors) or Option B (decompose AC-8).

## F-LP36-MED-003 — Story crates_touched ↔ STORY-INDEX column drift

**Severity:** MEDIUM
**Evidence:**
- Story line 22: `crates_touched: [prism-sensors, prism-spec-engine, prism-query]`
- STORY-INDEX line 395 column 3: `prism-sensors,prism-spec-engine` (missing `prism-query`)

**Fix:** state-manager — add `prism-query` to STORY-INDEX line 395 column 3 + STORY-INDEX version bump.

## Trajectory Summary

| Pass | In-Scope | Streak | Note |
|------|----------|--------|------|
| 33 | 1 HIGH | 0/3 | 6th sibling-sweep recurrence |
| 34 | 1 HIGH | 0/3 | 7th sibling-sweep recurrence |
| 35 | 0 | 1/3 ★ | 8th CLEAN |
| 36 | **3 MED** | **0/3 RESET (6th)** | 3 NEW defect axes |

## Next Step

FB28 pending: product-owner (2 findings) + state-manager (1 finding + closure). Single combined-burst recommended.

Pass-36 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-36.md` (this file).
