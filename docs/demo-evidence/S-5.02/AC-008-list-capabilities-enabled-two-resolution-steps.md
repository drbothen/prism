# AC-008 Evidence — list_capabilities enabled capability has two resolution steps

**AC:** AC-008 (BC-2.10.011 v1.5 postcondition — single-client mode, enabled status)

**Claim:** `list_capabilities("acme")` where a capability path has a `[[write_endpoints]]`
entry (compile tier permits) and runtime prism.toml grants it produces
`capabilities["sensor.crowdstrike.containment"].status = "enabled"` with `resolution_chain`
having two steps (`compile_tier → permit` and `runtime_tier → allow`).

## Verification method

Test: `test_BC_2_10_011_enabled_capability_has_two_resolution_steps`
File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

The test constructs a `PrismServer` with a test fixture that registers a capability path
at compile tier (via `WriteEndpointRegistry`) and grants it at runtime tier (via
`FeatureFlagEvaluator`). It calls `list_capabilities` with a valid `client_id`, then
inspects the returned JSON:
- `status = "enabled"` for the registered capability path
- `resolution_chain.len() = 2`
- `resolution_chain[0].level = "compile_tier"` and `result = "permit"`
- `resolution_chain[1].level = "runtime_tier"` and `result = "allow"`

API surface: `prism_mcp::server::PrismServer::list_capabilities` (tri-state handler)

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_011_enabled_capability_has_two_resolution_steps)'
```

Result: PASS

## Verdict

PASS
