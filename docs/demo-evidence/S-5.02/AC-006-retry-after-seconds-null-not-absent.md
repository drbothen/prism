# AC-006 Evidence — retry_after_seconds is null (not absent) for non-rate-limit errors

**AC:** AC-006 (BC-2.10.007 v1.8 postcondition — null-not-absent invariant)

**Claim:** For a non-rate-limit error (e.g., `PrismError::SensorHttpError`) mapped through
`build_structured_error_response`, `structuredContent.error.retry_after_seconds` is present
in the JSON as `null` — not absent from the object.

## Verification method

Test: `test_BC_2_10_007_no_retry_after_produces_null_not_absent`
File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

The test constructs a `StructuredErrorFields` with `retry_after_seconds: None`, calls
`build_structured_error_response`, and then:
1. Verifies `structuredContent.error.retry_after_seconds` IS PRESENT in the JSON object
   (`get("retry_after_seconds").is_some()`)
2. Verifies the value is JSON `null` (`v.is_null()`)

This distinguishes the null-not-absent invariant from a field that might simply be
omitted from serialization. The `SensorRateLimited` variant always carries a required
`u64` `retry_after_ms` — the null case is exercised exclusively by non-rate-limit
error variants where no retry hint exists.

API surface: `prism_mcp::error_mapping::build_structured_error_response`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_007_no_retry_after_produces_null_not_absent)'
```

Result: PASS

## Verdict

PASS
