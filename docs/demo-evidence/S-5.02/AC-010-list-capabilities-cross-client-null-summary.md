# AC-010 Evidence — list_capabilities(null) cross-client summary shape

**AC:** AC-010 (BC-2.10.011 v1.5 postcondition — cross-client summary mode)

**Claim:** `list_capabilities(client_id: null)` returns the shape:
`{client_id: null, clients: {<id>: {client_registered, enabled_count, runtime_disabled_count, compile_time_disabled_count}}, not_registered_tools: [...]}`.

## Verification method

Test: `test_BC_2_10_011_cross_client_null_returns_summary_shape`
File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

The test constructs a `PrismServer` with a multi-client fixture and calls
`list_capabilities` with `client_id = null`. It asserts the returned JSON:
- `client_id` key is present and equals JSON `null`
- `clients` key is present and is an object
- Each entry under `clients` has the fields: `client_registered`, `enabled_count`,
  `runtime_disabled_count`, `compile_time_disabled_count`
- `not_registered_tools` key is present (renamed from `not_implemented`)

API surface: `prism_mcp::server::PrismServer::list_capabilities` (cross-client summary path)

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_011_cross_client_null_returns_summary_shape)'
```

Result: PASS

## Verdict

PASS
