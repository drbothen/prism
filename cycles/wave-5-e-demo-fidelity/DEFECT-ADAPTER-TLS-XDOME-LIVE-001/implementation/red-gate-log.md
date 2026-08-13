# Red Gate Log — DEFECT-ADAPTER-TLS-XDOME-LIVE-001

**Author:** test-writer  
**Date:** 2026-08-13  
**Branch:** `feature/DEFECT-ADAPTER-TLS-XDOME-LIVE-001`  
**Protocol:** BC-5.38.001 Red Gate  

---

## Summary

8 Red Gate tests written. 6 are RED (failing assertion, not compile error). 2 are GREEN-BY-DESIGN (regression guards).

Red Gate PASS per BC-5.38.001: all tests that must be RED before implementation ARE RED. No implementation code written.

---

## Test Inventory

| RG    | Test Name                                                        | File                                      | Status               | Fail Reason (pre-fix)                                                                |
|-------|------------------------------------------------------------------|-------------------------------------------|----------------------|--------------------------------------------------------------------------------------|
| RG-001 | `test_map_error_http_401_maps_to_http_error_not_internal`       | `prism-bin/src/spec_driven_adapter.rs`    | RED (FAIL)           | `map_spec_engine_error_to_sensor_error` returns `SensorError::Internal` for all inputs; assertion `matches!(result, SensorError::HttpError { .. })` panics |
| RG-002 | `test_map_error_status_0_maps_to_internal`                      | `prism-bin/src/spec_driven_adapter.rs`    | GREEN-BY-DESIGN      | `status_code=0` correctly maps to `Internal` today; regression guard for post-fix correctness |
| RG-003 | `test_pipeline_non_2xx_body_in_detail`                          | `prism-spec-engine/src/pipeline.rs`       | RED (FAIL)           | `HttpRequestFailed.detail` is `"HTTP 403 Forbidden"` — no body captured; `detail.contains("forbidden")` panics |
| RG-004 | `test_fanout_all_failed_emits_fan_out_target_failed_warn`       | `prism-sensors/src/fanout.rs`             | RED (FAIL)           | No `tracing::warn!` loop in `AllTargetsFailed` arm; `logs_contain("fan_out_target_failed")` returns false |
| RG-005 | `test_probe_connectivity_403_returns_up_not_down`               | `prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` | RED (FAIL) | `map_spec_engine_error_to_sensor_error` maps to `Internal` → probe catch-all → `Down`; `assert_eq!(status, Up)` panics |
| RG-006 | `test_build_http_client_sends_user_agent_header`                 | `prism-bin/src/spec_driven_adapter.rs`    | RED (FAIL)           | No `.user_agent()` call in `build_http_client_with_custom_timeout`; received UA is empty string |
| RG-007 | `test_sensor_health_wire_shape_403_reachable_auth_invalid`      | `prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` | RED (FAIL) | RG-005 probe returns `Down` → `reachable=false`; `json.contains("\"reachable\":true")` panics |
| RG-008 | `test_reqwest_http2_not_enabled`                                | `prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` | GREEN-BY-DESIGN | reqwest Cargo.lock block correctly has no `h2` dep today; regression guard |

---

## nextest Evidence

### prism-bin inline tests (RG-001, RG-002, RG-006)

```
Summary: 3 tests run: 1 passed, 2 failed
FAIL prism-bin spec_driven_adapter::tests::test_map_error_http_401_maps_to_http_error_not_internal
  → "HttpRequestFailed(status_code=401) must map to SensorError::HttpError, got SensorError::Internal instead."
PASS prism-bin spec_driven_adapter::tests::test_map_error_status_0_maps_to_internal (GREEN-BY-DESIGN)
FAIL prism-bin spec_driven_adapter::tests::test_build_http_client_sends_user_agent_header
  → "User-Agent header must start with 'prism/'; got: \"\""
```

### prism-spec-engine (RG-003)

```
Summary: 1 test run: 0 passed, 1 failed
FAIL prism-spec-engine pipeline::non_2xx_body_snippet_tests::test_pipeline_non_2xx_body_in_detail
  → "HttpRequestFailed.detail must contain 'forbidden'. Current detail: \"HTTP 403 Forbidden\"."
```

### prism-sensors (RG-004)

```
Summary: 1 test run: 0 passed, 1 failed
FAIL prism-sensors fanout::fan_out_target_failed_warn_tests::test_fanout_all_failed_emits_fan_out_target_failed_warn
  → "No 'fan_out_target_failed' WARN was found in the log output."
```

### prism-bin integration tests (RG-005, RG-007, RG-008)

```
Summary: 3 tests run: 1 passed, 2 failed
FAIL prism-bin::defect_adapter_tls_xdome_live_001 test_probe_connectivity_403_returns_up_not_down
  → "probe must return ConnectivityStatus::Up for 403 response. Got: Down."
FAIL prism-bin::defect_adapter_tls_xdome_live_001 test_sensor_health_wire_shape_403_reachable_auth_invalid
  → "JSON must contain '\"reachable\":true' for 403 response."
PASS prism-bin::defect_adapter_tls_xdome_live_001 test_reqwest_http2_not_enabled (GREEN-BY-DESIGN)
```

---

## Regression Impact

Full crate test suites run after test addition:

| Crate             | Before | After | Delta                                     |
|-------------------|--------|-------|-------------------------------------------|
| prism-spec-engine | 771    | 772   | +1 (RG-003, RED)                          |
| prism-sensors     | 189    | 190   | +1 (RG-004, RED)                          |
| prism-bin         | 173    | 177   | +4 (RG-001 RED, RG-002 GREEN, RG-006 RED, RG-005 RED, RG-007 RED, RG-008 GREEN — integration tests counted separately) |

Zero pre-existing tests broken.

---

## Files Changed

| File                                                                    | Change                                          |
|-------------------------------------------------------------------------|------------------------------------------------|
| `crates/prism-bin/src/spec_driven_adapter.rs`                          | Appended RG-001, RG-002, RG-006 inline tests   |
| `crates/prism-spec-engine/src/pipeline.rs`                             | Appended `non_2xx_body_snippet_tests` module (RG-003) |
| `crates/prism-sensors/src/fanout.rs`                                   | Appended `fan_out_target_failed_warn_tests` module (RG-004) |
| `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`         | Created — RG-005, RG-007, RG-008               |
| `crates/prism-sensors/Cargo.toml`                                      | Added `tracing-test = "0.2"` dev-dep for RG-004 |
| `crates/prism-bin/Cargo.toml`                                          | Added `[[test]]` entry for integration test file |

---

## BC Coverage

| BC            | RG Tests                  |
|---------------|---------------------------|
| BC-2.08.002   | RG-001, RG-004, RG-005, RG-007 |
| BC-2.16.014   | RG-003, RG-006, RG-008    |

---

## Notes

- **RG-002** (GREEN-BY-DESIGN): `status_code=0` → `Internal` is correct pre- and post-fix. Added as regression guard to ensure the fix does NOT map ALL `HttpRequestFailed` to `HttpError` (only `status_code > 0`).
- **RG-008** (GREEN-BY-DESIGN): reqwest has no direct `h2` dependency today (ADR-050 compliant). Regression guard against future `features = ["http2"]` additions.
- **RG-005/007 design note**: Uses `AuthType::CustomViaPlugin` + `MockAuthProvider` instead of `BearerStatic` because `ProbeAuth` cannot be downcast as `BearerStaticSensorAuth` → BearerStatic path returns `Internal` immediately without HTTP call → wrong failure mode. Plugin path makes real HTTP calls through the pipeline.
- **RG-005/007 design note**: Uses HTTP 403 (not 401) because the pipeline's 401-retry loop ends in `AuthRefreshFailed` (not `HttpRequestFailed { status_code: 401 }`).
