# AC-005 Evidence — SensorRateLimited retry_after_seconds ms-to-s conversion

**AC:** AC-005 (BC-2.10.007 v1.8 postcondition — 429 wiring)

**Claim:** `PrismError::SensorRateLimited { sensor: "crowdstrike", retry_after_ms: 30_000 }`
mapped through `to_error_data_with_retry` produces `retryable: true` and `retry_after_seconds: 30`
(= `retry_after_ms / 1000`).

## Verification method

Tests:
- `test_BC_2_10_007_sensor_rate_limited_retry_after_seconds_ms_to_s_conversion`
- `test_HIGH_A_sensor_rate_limited_end_to_end_retry_after_seconds`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

`test_BC_2_10_007_sensor_rate_limited_retry_after_seconds_ms_to_s_conversion` calls
`to_error_data_with_retry(SensorRateLimited { sensor: "crowdstrike", retry_after_ms: 30_000 })`
and asserts the returned `u64` equals `30_000` (the raw ms value the builder divides by 1000).

`test_HIGH_A_sensor_rate_limited_end_to_end_retry_after_seconds` exercises the production
path end-to-end via `prism_error_to_structured_call_result`, asserting:
- `retryable = true`
- `retry_after_seconds = 30` in `structuredContent.error`

API surface: `prism_mcp::error_mapping::to_error_data_with_retry` +
`prism_mcp::error_mapping::prism_error_to_structured_call_result`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_007_sensor_rate_limited_retry_after_seconds_ms_to_s_conversion) or \
      test(test_HIGH_A_sensor_rate_limited_end_to_end_retry_after_seconds)'
```

Result: PASS (both tests)

## Verdict

PASS
