# AC-004 Evidence — Structured error has exactly 9 fields + _meta.trust_level:"internal"

**AC:** AC-004 (BC-2.10.007 v1.5 postcondition — wire shape)

**Claim:** Any MCP tool error response from the structured error builder has `structuredContent.error`
containing exactly the 9 fields: `code`, `message`, `category`, `retryable`, `retry_after_seconds`,
`suggestion`, `source`, `original_params_valid`, `upstream_message` — and
`structuredContent._meta.trust_level` equals `"internal"`.

## Verification method

Test: `test_BC_2_10_007_structured_error_has_nine_fields_and_meta_trust_level`
File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

The test calls `build_structured_error_response` from `prism_mcp::error_mapping` with a
`StructuredErrorFields` struct, then:
1. Verifies `_meta.trust_level = "internal"`
2. Iterates over all 9 required field names and asserts each is present in `structuredContent.error`
3. Asserts exact values for `code`, `category`, `retryable`, `original_params_valid`, `source`
4. Verifies `retry_after_seconds` is present as JSON `null` (not absent)
5. Verifies `upstream_message` is present as JSON `null` (not absent)

API surface: `prism_mcp::error_mapping::build_structured_error_response` +
`prism_mcp::error_mapping::StructuredErrorFields`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_007_structured_error_has_nine_fields_and_meta_trust_level)'
```

Result: PASS

## Verdict

PASS
