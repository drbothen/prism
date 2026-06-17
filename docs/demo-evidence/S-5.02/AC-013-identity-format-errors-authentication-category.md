# AC-013 Evidence — Identity-format errors map to authentication category

**AC:** AC-013 (HIGH-1 — BC-2.10.007 v1.8 §Category decision rule, identity-format subcategory)

**Claim:**
- `InvalidOrgSlug` maps to code `E-AUTH-001`, `category: "authentication"`, `original_params_valid: false`
- `InvalidAnalystId` maps to code `E-AUTH-002`, `category: "authentication"`, `original_params_valid: false`
- `InvalidClientId` maps to code `E-AUTH-003`, `category: "authentication"`, `original_params_valid: false`

Identity-format failures mean the caller supplied a structurally invalid identity token;
the params are malformed, not simply rejected at runtime.

## Verification method

Tests:
- `test_HIGH_1_invalid_org_slug_category_is_authentication`
- `test_HIGH_1_invalid_analyst_id_category_is_authentication`
- `test_HIGH_1_invalid_client_id_category_is_authentication`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

Each test passes the corresponding `PrismError` variant to `prism_error_to_structured_call_result`
and asserts:
- `category = "authentication"` (identity FORMAT failure per §Category rule)
- `original_params_valid = false` (malformed identity token — caller's params were invalid)
- `code` starts with `"E-AUTH-"` (E-AUTH-001, E-AUTH-002, E-AUTH-003 respectively)

Note: `InvalidClientId` in this context covers the case where `PrismError::InvalidClientId`
is constructed directly (e.g., from internal identity validation). The MCP `validate_client_ids`
free function DOES NOT route cases (a)/(b) through `PrismError::InvalidClientId` — it stays
on the `E-MCP-001` path (architecture compliance rule 1 and 7). These tests cover the
`PrismError::InvalidClientId` variant as it would arrive from other subsystem paths.

API surface: `prism_mcp::error_mapping::prism_error_to_structured_call_result`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_HIGH_1_invalid_org_slug_category_is_authentication) or \
      test(test_HIGH_1_invalid_analyst_id_category_is_authentication) or \
      test(test_HIGH_1_invalid_client_id_category_is_authentication)'
```

Result: PASS (all three)

## Verdict

PASS
