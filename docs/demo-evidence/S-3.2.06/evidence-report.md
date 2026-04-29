# Demo Evidence Report — S-3.2.06

**Story:** S-3.2.06 — prism-dtu-pagerduty: Shared-mode OrgId ingress tagging
**Impl SHA:** 138d5a8b
**Date:** 2026-04-29
**Recorder:** Demo Recorder agent

## Coverage Summary

| Recording | ACs / BCs Covered | Result |
|-----------|-------------------|--------|
| AC-001-all-8-tests-green | BC-3.2.004, BC-3.2.005 / AC-001–006 (all 6 story ACs) | 8/8 GREEN |
| AC-002-concurrent-orgid | BC-3.2.004 postcondition 4 / AC-003 | 1/1 GREEN (--nocapture) |

## Recordings

### AC-001 — All 8 org_tagging tests GREEN (BC-3.2.004 + BC-3.2.005)

- Tape: `AC-001-all-8-tests-green.tape`
- GIF: `AC-001-all-8-tests-green.gif` (140 KB)
- WebM: `AC-001-all-8-tests-green.webm` (273 KB)

Command demonstrated:
```
cargo test -p prism-dtu-pagerduty --features dtu --test org_tagging
```

**Result:** 8/8 GREEN.

Tests exercised:

| Test | Story AC | BC |
|------|----------|----|
| test_BC_3_2_004_ac001_org_id_in_incident_record | AC-001 | BC-3.2.004 postcondition 1 |
| test_BC_3_2_004_ac002_dedup_key_not_org_scoped | AC-002 | BC-3.2.004 postcondition 2 |
| test_BC_3_2_004_ac002_org_id_absent_from_routing | AC-002 | BC-3.2.004 postcondition 2 |
| test_BC_3_2_004_ac003_concurrent_incidents_distinguished | AC-003 | BC-3.2.004 postcondition 4 |
| test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results | AC-004 | BC-3.2.004 postcondition 5 |
| test_BC_3_2_005_ac005_pagerduty_dtu_mode_is_shared | AC-005 | BC-3.2.005 postcondition 1 |
| test_BC_3_2_005_ac005_mode_immutable_after_startup | AC-005 | BC-3.2.005 invariant 1 |
| test_BC_3_2_005_ac006_invalid_mode_string_rejected | AC-006 | BC-3.2.005 postcondition 3 |

### AC-002 — Concurrent OrgId tagging: org_A and org_B incidents distinguished

- Tape: `AC-002-concurrent-orgid.tape`
- GIF: `AC-002-concurrent-orgid.gif` (116 KB)
- WebM: `AC-002-concurrent-orgid.webm` (208 KB)

Command demonstrated:
```
cargo test -p prism-dtu-pagerduty --features dtu --test org_tagging \
  test_BC_3_2_004_ac003_concurrent_incidents_distinguished -- --nocapture
```

Demonstrates:
- `test_BC_3_2_004_ac003_concurrent_incidents_distinguished` — org_A and org_B each capture an
  incident concurrently via `capture_incident_tagged`; each resulting `IncidentRecord` contains
  its sender's OrgId UUID independently (AC-003 / BC-3.2.004 postcondition 4).

**Result:** 1/1 GREEN.

## Acceptance Criteria Coverage

| AC | Title | Recorded? | Notes |
|----|-------|-----------|-------|
| AC-001 | OrgId in `custom_details` / IncidentRecord.org_id | Yes (AC-001 tape) | ac001 test |
| AC-002 | OrgId absent from dedup_key and HTTP routing | Yes (AC-001 tape) | ac002 x2 tests |
| AC-003 | Concurrent incidents from different orgs distinguished | Yes (AC-001 + AC-002 tapes) | ac003 test (--nocapture in AC-002) |
| AC-004 | Mode metadata absent from query results | Yes (AC-001 tape) | ac004 test |
| AC-005 | DtuMode::Shared set at startup and immutable | Yes (AC-001 tape) | ac005 x2 tests |
| AC-006 | Invalid mode string rejected | Yes (AC-001 tape) | ac006 test |
