# AC-014 Evidence — SensorNotRegisteredForOrg maps to permission category

**AC:** AC-014 (OBS-1 — BC-2.10.007 v1.8 §Category decision rule, permission category)

**Claim:** `PrismError::SensorNotRegisteredForOrg` (E-QUERY-032) produces
`category: "permission"` and `original_params_valid: true`.

Rationale: the params are structurally valid; the sensor exists but is not in scope for
this org — a cross-org isolation denial, not a malformed request.

## Verification method

Tests:
- `test_BC_2_10_007_sensor_not_registered_for_org_category_is_permission` (unit test in `error_mapping::tests`)
- `test_BC_3_2_001_map_prism_error_sensor_not_registered_for_org_to_32602` (integration test in `tool_dispatch_tests`)

`test_BC_2_10_007_sensor_not_registered_for_org_category_is_permission` in
`crates/prism-mcp/src/error_mapping.rs` passes
`PrismError::SensorNotRegisteredForOrg { sensor_id: "crowdstrike", org_slug: "demo-org" }`
to `prism_error_to_structured_call_result` and asserts:
- `category = "permission"`
- `original_params_valid = true`

`test_BC_3_2_001_map_prism_error_sensor_not_registered_for_org_to_32602` in
`crates/prism-mcp/tests/tool_dispatch_tests.rs` verifies the JSON-RPC error code is
`-32602` (INVALID_PARAMS) and the message contains `"E-QUERY-032:"` with the org_slug
surfaced (permitted per AD-017 — org_slug is not a credential value).

API surface: `prism_mcp::error_mapping::prism_error_to_structured_call_result`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_007_sensor_not_registered_for_org_category_is_permission) or \
      test(test_BC_3_2_001_map_prism_error_sensor_not_registered_for_org_to_32602)'
```

Result: PASS (both tests)

## Verdict

PASS
