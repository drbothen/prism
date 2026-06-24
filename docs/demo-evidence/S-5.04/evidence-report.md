# Evidence Report — S-5.04: prism-mcp Sensor Health Subsystem

**Story:** S-5.04  
**Branch:** `feature/S-5.04`  
**Frozen HEAD at recording:** `4a2b2155` (pre-evidence commit)  
**Recording date:** 2026-06-24  
**Product type:** Rust MCP backend (CLI/terminal — VHS recordings)  
**Recording tool:** VHS 0.11.0 with `cargo nextest run` as the demo mechanism  

## Coverage Matrix

All 12 acceptance criteria are covered. Each recording runs real `cargo nextest run`
against the actual implementation with mock adapters at the adapter boundary (SID-1 in-process
pattern). No evidence is hand-fabricated.

| AC | Artifact (tape / gif / webm) | Tests Demonstrated | BC Traced |
|----|------------------------------|--------------------|-----------|
| AC-1 | `AC-01-connectivity-up-latency.*` | `test_BC_2_08_001_live_probe_200_returns_up_with_latency` (ProbeOutcome.status=Up + latency_ms) + `test_BC_2_08_001_check_one_surfaces_latency_ms_on_consumer_result_F_S504_P1_MED_001` (SensorHealthResult.latency_ms non-zero) | BC-2.08.001 postcondition 1 |
| AC-2 | `AC-02-auth-invalid-401-not-down.*` | `test_BC_2_08_002_live_probe_401_returns_auth_invalid_not_down` (success), `test_BC_2_08_002_connection_refused_returns_down_auth_unknown` (EC-004 error path), `test_BC_2_08_002_live_probe_403_returns_auth_invalid` | BC-2.08.002 postcondition 1 |
| AC-3 | `AC-03-rate-limit-429-retry-after.*` | `test_BC_2_08_003_rate_limit_state_from_http_429_with_header` (is_rate_limited=true, retry_after ~30s), `test_BC_2_08_003_rate_limit_default_60s_when_no_header` (EC-003), `test_BC_2_08_003_live_probe_429_yields_up_and_populates_rate_limit` | BC-2.08.003 postcondition 1 |
| AC-4 | `AC-04-last-successful-query-timestamp.*` | `test_BC_2_08_004_write_and_read_timestamp_roundtrip` (write+read at function level), `test_BC_2_08_004_checker_record_and_read_timestamp` (checker level), `test_BC_2_08_004_read_timestamp_returns_none_before_write` | BC-2.08.004 postcondition 1 |
| AC-5 | `AC-05-timestamp-persists-restart.*` | `test_BC_2_08_004_timestamp_survives_checker_reconstruction` (shared PrismContext), `test_BC_2_08_004_timestamp_survives_context_reconstruction_with_storage` (RocksDB path), `test_BC_2_08_004_checker_last_successful_query_none_before_record` | BC-2.08.004 postcondition 2 |
| AC-6 | `AC-06-partial-aggregate-3up-1down.*` | `test_BC_2_08_007_aggregate_partial_when_some_up_some_down` (3up+1down=Partial), `test_BC_2_08_007_aggregate_healthy_when_all_up`, `test_BC_2_08_007_aggregate_unhealthy_when_all_down`, `test_BC_2_08_007_invariant_partial_is_not_error`, `test_BC_2_08_007_aggregate_all_auth_invalid_is_unhealthy` (F-S504-LP3-HIGH-001) | BC-2.08.007 postcondition 1 |
| AC-7 | `AC-07-live-probe-level.*` | `test_BC_2_08_005_S504_live_probe_sets_probe_level_live` (server-internal test with health_checker wired), `test_BC_2_08_006_sensors_health_resource_live_probe_level`, `test_BC_2_08_006_sensors_health_resource_reachable_is_boolean` (not null), `test_BC_2_08_005_check_sensor_health_returns_spec_only_probe_level` (error path) | BC-2.08.005 postcondition (live-probe anchor) |
| AC-8 | `AC-08-espec026-invalid-probe-table.*` | `test_BC_2_16_009_probe_table_names_missing_table_returns_ESpec026` (error: missing_table), `test_BC_2_16_009_validates_all_4_bundled_specs` (success: all 4 canonical specs pass Rule 8) | BC-2.16.009 Validation Rule 8 / E-SPEC-026 |
| AC-9 | `AC-09-probe-table-routing.*` | `test_BC_2_08_001_check_one_routes_to_probe_table_via_spec_map` (probe_table=Some → routes to sensor_detections), `test_BC_2_08_001_check_one_falls_back_to_first_table_via_spec_map` (probe_table=None → first declared table), `test_BC_2_08_001_check_one_falls_back_to_devices_when_org_not_in_spec_map` (key-miss → sentinel "devices") | BC-2.08.001 postcondition 5 |
| AC-10 | `AC-10-canonical-sensor-probe-tables.*` | `test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_crowdstrike` (→ detections), `test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_cyberint` (→ alerts), `test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_claroty` (→ devices), `test_BC_2_08_001_canonical_sensor_toml_probe_table_declared_armis` (→ devices) | BC-2.08.001 precondition + BC-2.16.009 Rule 8 happy path |
| AC-11 | `AC-11-all-rate-limited-aggregate.*` | `test_BC_2_08_007_EC_007_all_rate_limited_aggregate_yields_rate_limited` (all-429 → RateLimited, not Partial), `test_BC_2_08_007_EC_007_mixed_rate_limited_and_down_is_partial` (error path: mixed → Partial) | BC-2.08.007 §Postconditions EC-08-015 |
| AC-12 | `AC-12-response-shape-summary-counts-suggestion.*` | `test_BC_2_08_007_EC_007_response_shape_overall_status_summary_counts_suggestion` (overall_status + summary_counts fields), `test_BC_2_08_007_EC_08_015_auth_invalid_production_path_suggestion` (verbatim auth-invalid string), `test_BC_2_08_007_EC_08_015_genuine_down_production_path_suggestion_distinct_from_5xx` (verbatim unreachable string) | BC-2.08.007 §Postconditions response shape |

## Recording Method

This is a pure Rust MCP backend with no standalone CLI binary for the health subsystem.
The demo recordings use `cargo nextest run` filtered to AC-specific test names, executed inside
VHS to produce terminal recordings. This is the canonical evidence format for library/backend
features — the test suite with real mock adapters at the SensorAdapter boundary constitutes
the demo. No evidence is hand-fabricated; all test assertions execute against the actual
production code paths.

Mock adapters used (defined in `bc_s_5_04_health_test.rs`):
- `MockAdapterOk` — simulates HTTP 200 (returns `Ok(vec![])`)
- `MockAdapterUnauthorized` — simulates HTTP 401 (`SensorError::HttpError { status: 401 }`)
- `MockAdapterForbidden` — simulates HTTP 403 (`SensorError::HttpError { status: 403 }`)
- `MockAdapterConnectionRefused` — simulates timeout/unreachable (`SensorError::Timeout`)
- `MockAdapterRateLimited` — simulates HTTP 429 (`SensorError::RateLimited { retry_after_ms: 30_000 }`)
- `MockAdapterRateLimitedArmis` — 429 for armis sensor (used in EC-007 all-rate-limited test)
- `MockAdapterHostileBody` — simulates 500 with oversized hostile body (F-S504-P2-008)

## SID-1 DTU-blocked tests

Two tests are `#[ignore]`'d per SID-1 discipline with explicit blocking-dependency comments:
- `test_BC_2_08_001_live_probe_200_returns_up_with_latency_dtu` — requires prism-dtu-crowdstrike
- `test_BC_2_08_002_live_probe_401_returns_auth_invalid_dtu` — requires prism-dtu-crowdstrike

Both have in-process companion tests (listed above in AC-1 and AC-2 rows) that exercise
the same production code paths via the mock adapter boundary.

## Baseline test run

All 366 `prism-mcp` tests pass at frozen HEAD `4a2b2155`:

```
Summary [3.310s] 366 tests run: 366 passed, 3 skipped
```

The 3 skipped are the `#[ignore]`'d DTU integration tests.
