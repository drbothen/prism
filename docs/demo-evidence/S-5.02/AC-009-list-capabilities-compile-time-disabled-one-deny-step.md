# AC-009 Evidence — list_capabilities compile_time_disabled has one deny step

**AC:** AC-009 (BC-2.10.011 v1.5 postcondition — compile_time_disabled status)

**Claim:** `list_capabilities("acme")` where a capability has no `[[write_endpoints]]`
entry in the sensor TOML (compile tier denies) produces `status = "compile_time_disabled"`
and `resolution_chain` with one step (`compile_tier → deny`).

## Verification method

Tests:
- `test_BC_2_10_011_compile_time_disabled_has_one_deny_step`
- `test_F6_compile_absent_paths_produce_compile_time_disabled_via_resolver`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

`test_BC_2_10_011_compile_time_disabled_has_one_deny_step` sets up a fixture with a
capability path that has no corresponding `WriteEndpoint` registration (compile tier
has no permit). It asserts:
- `status = "compile_time_disabled"` for the unregistered capability path
- `resolution_chain.len() = 1`
- `resolution_chain[0].level = "compile_tier"` and `result = "deny"`

`test_F6_compile_absent_paths_produce_compile_time_disabled_via_resolver` verifies
that capability paths enumerated from `all_capability_paths` (the full known set) but
absent from the `WriteEndpointRegistry` are consistently classified as
`compile_time_disabled` by the resolver — not as `runtime_disabled` — regardless of
runtime config state.

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_011_compile_time_disabled_has_one_deny_step) or \
      test(test_F6_compile_absent_paths_produce_compile_time_disabled_via_resolver)'
```

Result: PASS (both tests)

## Verdict

PASS
