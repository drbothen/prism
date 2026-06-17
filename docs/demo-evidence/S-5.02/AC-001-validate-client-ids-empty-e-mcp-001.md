# AC-001 Evidence — E-MCP-001 prefix on empty client_id

**AC:** AC-001 (BC-2.10.004 v2.8 postcondition case (a))

**Claim:** `validate_client_ids` with an empty string `""` returns an error whose message starts
with `"E-MCP-001: invalid client_id format:"`.

## Verification method

Test: `test_BC_2_10_004_empty_client_id_returns_e_mcp_001_prefix`
File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

The test calls `server.list_capabilities(Parameters(ListCapabilitiesParams::for_client("")))`,
asserts `is_error = true`, inspects `structuredContent.error.code` (must equal `"E-MCP-001"`),
and verifies `message.starts_with("E-MCP-001:")`.

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_004_empty_client_id_returns_e_mcp_001_prefix)'
```

Result: PASS

Additional assertions verified by this test:
- `original_params_valid = false` (format check failed — case (a))
- Message does NOT contain `"E-AUTH-003"` (no namespace collision with sensor-layer auth code)
- Response is `Ok(CallToolResult)` not `Err(ErrorData)` — structured error surfaced as tool result

## Verdict

PASS
