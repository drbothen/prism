# AC-015 Evidence — Watchdog infrastructure failures map to internal category

**AC:** AC-015 (OBS-2 — BC-2.10.007 v1.8 §Category decision rule, internal category, watchdog subcategory)

**Claim:** `PrismError::WatchdogKilled`, `PrismError::HeartbeatMissed`, and
`PrismError::RestartLimitExceeded` all produce `category: "internal"` and
`original_params_valid: true`.

Rationale: watchdog failures are Prism infrastructure events; the original params were
valid — the subsystem failed independently of the request content.

## Verification method

Tests:
- `test_BC_2_10_007_watchdog_killed_category_is_internal` (unit test in `error_mapping::tests`)
- `test_BC_2_10_007_heartbeat_missed_maps_to_internal_category` (unit test in `error_mapping::tests`)
- `test_BC_2_10_007_restart_limit_exceeded_maps_to_internal_category` (unit test in `error_mapping::tests`)

All three tests are in `crates/prism-mcp/src/error_mapping.rs`. Each passes its
respective `PrismError` watchdog variant to `prism_error_to_structured_call_result`
and asserts:
- `category = "internal"`
- `original_params_valid = true`
- `retryable = false` (watchdog infrastructure failures are not retryable on the same call)

Note: The `test_CRIT_B_catch_all_category_is_upstream_error` test in `tool_dispatch_tests.rs`
confirms the catch-all path still emits `"upstream_error"` for genuinely unmapped variants
(e.g., `PrismError::Infusion`) — the watchdog variants now have EXPLICIT arms rather than
falling to the catch-all, which is what this AC verifies (explicit arm, not catch-all).

API surface: `prism_mcp::error_mapping::prism_error_to_structured_call_result`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_007_watchdog_killed_category_is_internal) or \
      test(test_BC_2_10_007_heartbeat_missed_maps_to_internal_category) or \
      test(test_BC_2_10_007_restart_limit_exceeded_maps_to_internal_category)'
```

Result: PASS (all three)

## Verdict

PASS
