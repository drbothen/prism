---
document_type: demo-evidence-report
story_id: S-3.6.02
title: "HS-007 Wave 3 anchor refresh — demo evidence"
date: "2026-04-29"
producer: demo-recorder
behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]
---

# Demo Evidence Report — S-3.6.02

**Story:** HS-007 multi-tenant cross-repo failure holdout refresh — re-anchor to Wave 3 BCs
**Branch:** feature/S-3.6.02
**Implementation commit:** 5f2c16c5

## BC Anchors

| BC ID | Title | Status |
|-------|-------|--------|
| BC-3.5.001 | Harness Logical Isolation | Anchored |
| BC-3.5.002 | Harness Network Isolation | Anchored |
| BC-3.6.001 | Per-Org Failure Injection | Anchored |
| BC-3.6.002 | Harness Crash Detection | Anchored |

## Acceptance Criteria Coverage

### AC-001 — All 5 HS-007 anchor tests GREEN

**Criterion:** `cargo test --test hs_007_anchor_test` reports 5/5 passing (no failures, no stubs).

**Recordings:**
- `AC-001-hs-007-anchor-tests-green.gif` — animated terminal capture
- `AC-001-hs-007-anchor-tests-green.webm` — archival video
- `AC-001-hs-007-anchor-tests-green.tape` — VHS script source

**Result:** PASS — all 5 tests green.

```
test test_hs_007_anchored_to_wave_3_bcs ... ok
test test_hs_007_no_legacy_wave_bc_references ... ok
test test_hs_007_no_stub_markers ... ok
test test_hs_007_phase_is_3a ... ok
test test_hs_007_three_sub_scenarios_present ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### AC-002 — File frontmatter snapshot

**Criterion:** `tests/holdout-scenarios/HS-007-cross-repo-failure.md` frontmatter shows
`behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]` and `phase: 3.A`.

**Recordings:**
- `AC-002-frontmatter-anchors.gif` — animated terminal capture
- `AC-002-frontmatter-anchors.webm` — archival video
- `AC-002-frontmatter-anchors.tape` — VHS script source

**Result:** PASS — frontmatter contains all 4 Wave 3 BC anchors with `phase: 3.A` and
`timestamp: "2026-04-29T00:00:00Z"`.

## Sub-Scenario Summary

| Sub-scenario | Description | BC Traced |
|-------------|-------------|-----------|
| HS-007-01 | Per-org failure isolation: `inject_failure(acme-corp, Claroty, AuthReject)` — acme-corp gets HTTP 401; globex unaffected; `clear_failure` restores normal operation | BC-3.6.001 |
| HS-007-02 | Routing bug detection via network mode: wrong-org credentials sent to live clone endpoint assert HTTP 401, proving cross-process boundary enforcement | BC-3.5.002 |
| HS-007-03 | Crash detection: `InternalError` injection causes clone panic; `HarnessError::CloneCrashed` raised for acme-corp within 1s; globex unaffected | BC-3.6.002 |

## Coverage Summary

| AC | Criterion | Recorded | PASS/FAIL |
|----|-----------|----------|-----------|
| AC-001 | 5/5 anchor tests green | Yes | PASS |
| AC-002 | Frontmatter BC anchors visible | Yes | PASS |

**All acceptance criteria demonstrated. Zero stub markers remain. No legacy Wave 1/2 BC references.**
