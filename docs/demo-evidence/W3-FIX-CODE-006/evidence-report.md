# Demo Evidence — W3-FIX-CODE-006

**Story:** Armis activity/risk endpoint org-id guard test coverage (CR-023 closure)
**Branch:** fix/W3-FIX-CODE-006-armis-activity-risk-test-coverage
**Commit:** e92385be
**Date:** 2026-05-02
**Mode:** Test-only delivery — demo evidence is nextest pass log (POL-010)

---

## Nextest Run: cr023_activity_risk_org_id_guard

**Command:**
```
cargo nextest run -p prism-dtu-armis --all-features --test cr023_activity_risk_org_id_guard
```

**Output:**
```
────────────
 Nextest run ID 003decac-47cb-4908-9835-8be2a52acb76 with nextest profile: default
    Starting 6 tests across 1 binary
        PASS [   0.064s] (1/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_activity_real_org_absent_header_returns_401
        PASS [   0.064s] (2/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_risk_real_org_correct_header_returns_200
        PASS [   0.064s] (3/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_activity_default_instance_absent_header_returns_200
        PASS [   0.064s] (4/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_risk_real_org_absent_header_returns_401
        PASS [   0.064s] (5/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_activity_real_org_correct_header_returns_200
        PASS [   0.064s] (6/6) prism-dtu-armis::cr023_activity_risk_org_id_guard test_get_device_risk_default_instance_absent_header_returns_200
────────────
     Summary [   0.065s] 6 tests run: 6 passed, 0 skipped
```

---

## Per-AC Evidence

| AC | Test Function | Result | Duration |
|----|--------------|--------|----------|
| AC-001 | `test_get_device_activity_real_org_absent_header_returns_401` | PASS | 0.064s |
| AC-002 | `test_get_device_activity_real_org_correct_header_returns_200` | PASS | 0.064s |
| AC-003 | `test_get_device_activity_default_instance_absent_header_returns_200` | PASS | 0.064s |
| AC-004 | `test_get_device_risk_real_org_absent_header_returns_401` | PASS | 0.064s |
| AC-005 | `test_get_device_risk_real_org_correct_header_returns_200` | PASS | 0.064s |
| AC-006 | `test_get_device_risk_default_instance_absent_header_returns_200` | PASS | 0.064s |

**Result: 6/6 PASS — CR-023 closed**
