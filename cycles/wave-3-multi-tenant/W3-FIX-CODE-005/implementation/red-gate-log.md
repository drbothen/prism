# Red Gate Log — W3-FIX-CODE-005

**Story:** W3-FIX-CODE-005 — DTU harness + Armis/CrowdStrike: sibling poll-backoff
propagation and missing org-id guards

**Date:** 2026-05-01  
**Agent:** test-writer  
**Status:** RED GATE VERIFIED — all guard-absence tests fail with assertion errors

---

## Test Files Written

| File | Tests | Failing | Passing |
|------|-------|---------|---------|
| `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs` | 8 | 3 | 5 |
| `crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs` | 6 | 2 | 4 |
| **Total** | **14** | **5** | **9** |

---

## Failing Tests (Red Gate Verified)

### CR-017: Armis tag/alert org-id guard absent

**File:** `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs`

```
test_post_device_tag_real_org_absent_header_returns_401 ... FAILED
  expected 401 got 201
  — guard not yet present in tags.rs::post_device_tag

test_delete_device_tag_real_org_absent_header_returns_401 ... FAILED
  expected 401 got 404
  — guard not yet present in tags.rs::delete_device_tag

test_get_alerts_real_org_absent_header_returns_401 ... FAILED
  expected 401 got 200
  — guard not yet present in alerts.rs::get_alerts
```

Each test fails with an assertion error against the live HTTP response —
**not** a build error. The guard simply does not exist yet.

### CR-018: CrowdStrike detection org-id guard absent

**File:** `crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs`

```
test_list_detection_ids_real_org_absent_header_returns_401 ... FAILED
  expected 401 got 200
  — guard not yet present in detections.rs::list_detection_ids

test_get_detection_summaries_real_org_absent_header_returns_401 ... FAILED
  expected 401 got 200
  — guard not yet present in detections.rs::get_detection_summaries
```

---

## Passing Tests (Correct — Pre-Guard State)

These pass because they test backward-compat / positive-path / nil-instance
behavior that is already correct without implementation changes:

| Test | Reason passes |
|------|--------------|
| `test_post_device_tag_real_org_correct_header_returns_201` | Guard irrelevant when header is correct |
| `test_post_device_tag_default_instance_absent_header_returns_201` | Legacy clone skips guard |
| `test_delete_device_tag_default_instance_absent_header_allows_request` | Legacy clone skips guard |
| `test_get_alerts_real_org_correct_header_returns_200` | Guard irrelevant when header is correct |
| `test_get_alerts_default_instance_absent_header_returns_200` | Legacy clone skips guard |
| `test_list_detection_ids_real_org_correct_header_returns_200` | Guard irrelevant when header is correct |
| `test_list_detection_ids_nil_instance_absent_header_returns_200` | Nil-instance clone skips guard |
| `test_get_detection_summaries_real_org_correct_header_returns_200` | Guard irrelevant when header is correct |
| `test_get_detection_summaries_nil_instance_absent_header_returns_200` | Nil-instance clone skips guard |

**Note:** Positive-path and backward-compat tests passing pre-implementation is
expected and correct. The Red Gate requirement is that the _guard-bypass tests_
fail — which they do. These 9 passing tests verify the implementer does not
break existing behavior when adding the guard.

---

## Scope Not Requiring Tests

| Item | Reason |
|------|--------|
| CR-016 (50ms poll cadence) | Pure constant change; no behavioral assertion needed. Verified by `grep -r "from_millis(10)" crates/prism-dtu-harness/src/clones/` post-fix. |
| CR-020 (validate_spec_path comment) | Documentation-only; no test change required per story AC-004. |
| L-50-004 (tech-debt.md entry) | Documentation artifact; no test. |

---

## Baseline Regression Check

All 14 new test binaries compile cleanly:

```
cargo test --features dtu --test cr017_tag_alert_org_id_guard --test cr018_detections_org_id_guard --no-run
Finished `test` profile [unoptimized + debuginfo] target(s) in 23.52s
```

Full workspace compile check:

```
cargo test --workspace --features dtu --no-run
Finished `test` profile [unoptimized + debuginfo] target(s)
```

No existing tests broken.

---

## Implementer Instructions

Make each failing test pass, one at a time, with minimum code:

1. **`tags.rs::post_device_tag`** — insert the `is_real_org` dual-mode guard
   (pattern from `devices.rs:89-94`) after `check_bearer_auth`. Required import:
   `crate::routes::devices::validate_org_id` (or wherever it lives in the crate).

2. **`tags.rs::delete_device_tag`** — same guard, same position.

3. **`alerts.rs::get_alerts`** — same guard, after `check_bearer_auth`.
   Import `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID` if not already visible.

4. **`detections.rs::list_detection_ids`** — insert the nil-instance guard
   (pattern from `hosts.rs:146-150`) after `check_auth`. Requires:
   `use crate::state::validate_org_id` (or wherever the helper lives) and
   `use prism_core::OrgId`.

5. **`detections.rs::get_detection_summaries`** — same nil-instance guard.

**Architecture compliance (do not mix up):**
- Armis: compare against `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID` (non-nil sentinel)
- CrowdStrike: compare against `OrgId::from_uuid(uuid::Uuid::nil())` (nil sentinel)
