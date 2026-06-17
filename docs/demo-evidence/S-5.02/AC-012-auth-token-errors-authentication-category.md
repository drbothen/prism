# AC-012 Evidence — AuthTokenExpired/AuthTokenInvalid map to authentication category

**AC:** AC-012 (HIGH-1 — BC-2.10.007 v1.8 §Category decision rule, authentication category)

**Claim:**
- `AuthTokenExpired` maps to code `E-AUTH-010`, `category: "authentication"`, `original_params_valid: true`
- `AuthTokenInvalid` maps to code `E-AUTH-011`, `category: "authentication"`, `original_params_valid: true`

The token is the credential, not the query parameter — the original tool params are
structurally valid; the session credential expired or was revoked.

## Verification method

Tests:
- `test_HIGH_1_auth_token_expired_category_is_authentication`
- `test_HIGH_1_auth_token_invalid_category_is_authentication`

File: `crates/prism-mcp/tests/tool_dispatch_tests.rs`

`test_HIGH_1_auth_token_expired_category_is_authentication` passes `PrismError::AuthTokenExpired`
to `prism_error_to_structured_call_result` and asserts:
- `category = "authentication"` (pre-fix this was `"upstream_error"` — semantically wrong)
- `original_params_valid = true` (token format was valid, credential expired)
- `code = "E-AUTH-010"` (not `"E-INT-001"`)

`test_HIGH_1_auth_token_invalid_category_is_authentication` passes
`PrismError::AuthTokenInvalid { reason: "signature verification failed" }` and asserts:
- `category = "authentication"`
- `original_params_valid = true` (token structurally valid, invalid at runtime)
- `code = "E-AUTH-011"`

API surface: `prism_mcp::error_mapping::prism_error_to_structured_call_result`

## Test execution

```
cargo nextest run -p prism-mcp \
  -E 'test(test_HIGH_1_auth_token_expired_category_is_authentication) or \
      test(test_HIGH_1_auth_token_invalid_category_is_authentication)'
```

Result: PASS (both tests)

## Verdict

PASS
