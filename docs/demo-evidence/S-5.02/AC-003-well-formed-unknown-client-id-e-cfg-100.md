# AC-003 Evidence — Well-formed unknown client_id maps to E-CFG-100 / configuration category

**AC:** AC-003 (BC-2.10.004 v2.8 postcondition case (c))

**Claim:** A tool call where `client_id` passes format validation but is not in the runtime
capability registry maps to `PrismError::ClientNotFound` via `error_mapping.rs`, producing
code `E-CFG-100`, `category: "configuration"`, `retryable: false`, `original_params_valid: true`.

## Verification method

Tests:
- `test_BC_2_10_004_well_formed_unknown_client_id_maps_to_e_cfg_100`
- `test_CRIT_A_client_not_found_structured_error_category_configuration_params_valid`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

`test_BC_2_10_004_well_formed_unknown_client_id_maps_to_e_cfg_100` calls `map_prism_error`
with `PrismError::ClientNotFound { client_id: "valid-but-unknown".to_owned() }` and asserts:
- JSON-RPC code `-32602` (INVALID_PARAMS)
- Message string contains `"E-CFG-100:"`

`test_CRIT_A_client_not_found_structured_error_category_configuration_params_valid` exercises
the full structured path via `prism_error_to_structured_call_result`, asserting:
- `category = "configuration"`
- `original_params_valid = true` (format was valid; client simply absent from registry)

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_BC_2_10_004_well_formed_unknown_client_id_maps_to_e_cfg_100) or \
      test(test_CRIT_A_client_not_found_structured_error_category_configuration_params_valid)'
```

Result: PASS

## Verdict

PASS
