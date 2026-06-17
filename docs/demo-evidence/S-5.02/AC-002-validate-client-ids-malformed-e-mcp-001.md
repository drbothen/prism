# AC-002 Evidence — E-MCP-001 + original_params_valid:false on malformed client_id

**AC:** AC-002 (BC-2.10.004 v2.8 postcondition case (b))

**Claim:** `validate_client_ids` with a client_id containing `/` or `..` (e.g., `"acme/../../etc"`)
returns `E-MCP-001` with `original_params_valid: false` in the structured error envelope.

## Verification method

Tests:
- `test_BC_2_10_004_malformed_client_id_returns_e_mcp_001_prefix`
- `test_BC_2_10_004_path_traversal_client_id_returns_e_mcp_001`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

Both tests construct client_ids that fail the `[a-zA-Z0-9_-]{1,64}` regex (path-traversal
chars `/`, `.`). They assert:
- `code = "E-MCP-001"`
- `message.starts_with("E-MCP-001: invalid client_id format:")`
- `original_params_valid = false`
- Response is `Ok(CallToolResult)` with `is_error = true`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_004_malformed_client_id_returns_e_mcp_001_prefix) or \
      test(test_BC_2_10_004_path_traversal_client_id_returns_e_mcp_001)'
```

Result: PASS (both tests)

## Verdict

PASS
