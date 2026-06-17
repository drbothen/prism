# AC-011 Evidence — not_registered_tools field name (not not_implemented)

**AC:** AC-011 (BC-2.10.011 v1.5 postcondition — field rename)

**Claim:** The `list_capabilities` response field is named `not_registered_tools` (not
`not_implemented`), and it contains exactly the `NOT_YET_AVAILABLE_TOOLS` constant's entries.

## Verification method

Tests:
- `test_BC_2_10_011_not_registered_tools_field_not_not_implemented`
- `test_F10_not_registered_tools_allocation_optimization_does_not_regress`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

`test_BC_2_10_011_not_registered_tools_field_not_not_implemented` calls `list_capabilities`
with a valid `client_id` and asserts:
- The response JSON contains the key `"not_registered_tools"` (not absent)
- The response JSON does NOT contain the old key `"not_implemented"`
- The value under `"not_registered_tools"` is an array

`test_F10_not_registered_tools_allocation_optimization_does_not_regress` verifies the
field contents match the canonical `NOT_YET_AVAILABLE_TOOLS` constant from
`prism_mcp::server` — entry count and values are stable.

API surface: `prism_mcp::server::PrismServer::list_capabilities` +
`prism_mcp::server::NOT_YET_AVAILABLE_TOOLS`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_011_not_registered_tools_field_not_not_implemented) or \
      test(test_F10_not_registered_tools_allocation_optimization_does_not_regress)'
```

Result: PASS (both tests)

## Verdict

PASS
