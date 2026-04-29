# Demo Evidence Report — S-3.2.03

**Story:** prism-dtu-crowdstrike: Multi-tenant state segregation — containment + detection store re-keying (D-048)
**Story ID:** S-3.2.03
**Branch:** feature/S-3.2.03
**Implementation commit:** 186acb4d
**Recorded:** 2026-04-29
**Recorder:** demo-recorder agent
**BC Anchors:** BC-3.2.001, BC-3.2.003

---

## Coverage Summary

| Recording | AC | BC Anchor | Path | Result |
|-----------|-----|-----------|------|--------|
| AC-001-all-14-multi-tenant-tests-green | AC-001 through AC-007 | BC-3.2.001, BC-3.2.003 | success | PASS — 14/14 |
| AC-002-containment-detection-isolation-http-routes | AC-007 (HTTP routes) | BC-3.2.001 invariant 1 | success | PASS — 3/3 |

---

## AC-001 — All 14 multi_tenant tests GREEN

**Acceptance Criterion (AC-001 through AC-007):** `cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant` must exit with 14 passed, 0 failed.

**Test command:**
```
cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant
```

**Observed result:** `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s`

**Tests covered:**
- `test_BC_3_2_001_containment_cross_org_returns_none` (AC-001)
- `test_BC_3_2_001_detection_status_cross_org_returns_none` (AC-002)
- `test_BC_3_2_001_containment_write_does_not_affect_other_org` (AC-003)
- `test_BC_3_2_003_session_registry_not_rekeyed` (AC-004)
- `test_BC_3_2_001_reset_for_removes_only_target_org_containment` (AC-005)
- `test_BC_3_2_001_reset_for_removes_only_target_org_detection_status` (AC-005)
- `test_BC_3_2_001_reset_for_both_stores_atomically` (AC-005)
- `test_AC_007_contain_route_accepts_org_a_containment` (AC-007)
- `test_AC_007_lift_containment_route_uses_org_scoped_key` (AC-007)
- `test_AC_007_patch_detections_route_uses_org_scoped_key` (AC-007)
- `prop_containment_cross_org_isolation` (AC-006, 1000 proptest cases)
- `prop_detection_cross_org_isolation` (AC-006, 1000 proptest cases)
- `prop_reset_for_selectivity` (AC-005, AC-006, 1000 proptest cases)
- `prop_write_isolation_no_cross_org_mutation` (AC-003, AC-006, 1000 proptest cases)

**Recordings:**
- [AC-001-all-14-multi-tenant-tests-green.gif](AC-001-all-14-multi-tenant-tests-green.gif)
- [AC-001-all-14-multi-tenant-tests-green.webm](AC-001-all-14-multi-tenant-tests-green.webm)
- [AC-001-all-14-multi-tenant-tests-green.tape](AC-001-all-14-multi-tenant-tests-green.tape)

---

## AC-002 — Containment + detection isolation: HTTP routes

**Acceptance Criterion (AC-007):** All route handler call sites compile and accept `OrgId`-scoped requests. The three HTTP route tests (`test_AC_007_*`) return HTTP 202/202/200 respectively.

**Test command:**
```
cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant test_AC_007 -- --nocapture
```

**Observed result:** `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.05s`

**Tests covered:**
- `test_AC_007_contain_route_accepts_org_a_containment` — POST contain → HTTP 202
- `test_AC_007_lift_containment_route_uses_org_scoped_key` — POST lift_containment → HTTP 202
- `test_AC_007_patch_detections_route_uses_org_scoped_key` — PATCH detections → HTTP 200

**Recordings:**
- [AC-002-containment-detection-isolation-http-routes.gif](AC-002-containment-detection-isolation-http-routes.gif)
- [AC-002-containment-detection-isolation-http-routes.webm](AC-002-containment-detection-isolation-http-routes.webm)
- [AC-002-containment-detection-isolation-http-routes.tape](AC-002-containment-detection-isolation-http-routes.tape)

---

## Deferred

**D-048 (session_registry non-re-keying):** Covered by `test_BC_3_2_003_session_registry_not_rekeyed` in the full suite (AC-001 recording). Not separately demoed per orchestrator brief — verified as part of the 14-test run.

---

## Toolchain

| Tool | Version | Purpose |
|------|---------|---------|
| VHS | 0.10.0 | Terminal session recording (.gif + .webm) |
| cargo | (workspace) | Test runner |
| FiraCode Nerd Font Mono | installed | Terminal font for recording |
