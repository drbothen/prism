---
document_type: behavioral-contract
level: L3
version: "2.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-006"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "412c872"
traces_to: ["CAP-006"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.010: Confirmation Token Consumption via confirm_action

## Description

`confirm_action` is the second step of Prism's irreversible write two-step pattern. It
validates the supplied `token_id` against the in-memory token store: checks that the token
is not expired, not already consumed, and that the `client_id` parameter matches the token's
embedded `client_id` (equality check only — no config lookup). If validation passes, the
token is marked consumed immediately before execution (preventing double-execution on retry),
then the sensor adapter is called directly using the stored `tool_name` and `action_params`.

`confirm_action` validates `client_id` against the token's embedded `client_id`, NOT against
client configuration. This allows the `__global__` sentinel for global-scope operations.

## Preconditions
- The `confirm_action` MCP tool is invoked with a `client_id` and `token_id`
- A matching token exists in the in-memory token store

## Postconditions
- The token is validated: not expired, not already consumed, client_id matches the token's embedded client_id, action_hash matches
- **`confirm_action` validates `client_id` against the token's embedded `client_id`, NOT against client configuration.** This allows the `__global__` sentinel for global-scope operations (e.g., global alias mutations). The `client_id` parameter is only checked for equality with `ConfirmationToken.client_id` — no config lookup occurs.
- If valid, `confirm_action` calls the sensor adapter directly (not the MCP tool handler) using the `tool_name` and `action_params` stored in the ConfirmationToken. This bypasses capability checks, which were already validated at token generation time.
- The token is marked as `consumed: true` immediately before execution (single-use)
- The execution result is returned to the caller
- Exactly 1 AuditEntry is produced per `confirm_action` call, containing sub-fields for both token validation (`token_id`, `token_status`, `client_id_match`) and execution result (`tool_name`, `action_params`, `execution_status`, `sensor_response_summary`)
- Cache invalidation is triggered for the affected `source_id`(s) per the write tool to source_id mapping defined in BC-2.07.004
- If cache invalidation fails after a successful write, `confirm_action` returns `status: 'executed'` with a `_meta.cache_warning` field indicating stale cache data may exist. The write is not rolled back.

## Invariants
- DI-007: Consumed tokens cannot be reused; expired tokens are rejected
- Token is single-use: marked consumed before execution, not after (prevents double-execution on retry)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::TokenExpired` | Token's `expires_at` is in the past | `code: "E-FLAG-003"`, `retryable: false`, suggestion: "Call the original write tool again to generate a new confirmation token" (DEC-009) |
| `PrismError::TokenConsumed` | Token has already been used | `code: "E-FLAG-004"`, `retryable: false`, suggestion: "This token has already been used. Call the original write tool to generate a new token if needed." |
| `PrismError::InvalidInput` | Token ID not found in store | `code: "E-FLAG-008"`, suggestion: "Token not found. It may have expired and been cleaned up." |
| `PrismError::InvalidInput` | Supplied `client_id` does not match the token's embedded `client_id` | `code: "E-MCP-004"`, `retryable: false`, suggestion: "The client_id does not match the token's originating client. Use the same client_id that was used when the token was generated." |
| `PrismError::Sensor` | Token valid but sensor API execution fails | Token is still consumed (cannot retry with same token); error returned; agent must request a new token |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-009 | Token expired at exactly 300s boundary | Expired; boundary is exclusive (>= 300s elapsed means expired) |
| EC-04-020 | Network failure during execution after token consumed | Token consumed; operation may or may not have executed; response indicates uncertainty; agent should verify state |
| EC-04-021 | Concurrent `confirm_action` calls with same token | First call consumes the token; second call gets `E-FLAG-004` error; no double-execution |
| EC-04-022 | `confirm_action` called with `client_id: "__global__"` for a global-scope alias operation | Valid; the `"__global__"` sentinel is accepted as a `client_id` match when the token was generated for a global-scope mutation (aliases, schedules, packs, global-scope rules). The `"__global__"` value is not a real client ID and must not be used for any other purpose. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.010.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Valid consumption | Unexpired, unconsumed token; correct `client_id` | Write executes; token marked consumed; audit entry emitted |
| Expired token | `now >= expires_at` | `E-FLAG-003`; write not executed |
| Already consumed | Token `consumed: true` | `E-FLAG-004`; write not executed |
| Client ID mismatch | `client_id` does not match token's embedded `client_id` | `E-MCP-004`; write not executed |
| Concurrent consumption | Two simultaneous calls with same token | One succeeds; other gets `E-FLAG-004`; no double-execution |

## Verification Properties

- **VP-007** (Confirmation token expiry: expired at boundary inclusive) — verifies expiry check in `confirm_action`.
- **VP-008** (Confirmation token: single-use enforcement) — verifies concurrent consumption produces exactly one execution.
- **VP-009** (Confirmation token: content hash mismatch rejects) — verifies action hash is re-verified at consumption time.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Related BCs | BC-2.07.004 (cache invalidation triggered on write completion) |
| Addresses | ADV-6-007, ADV-6-010 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 2.0 | Phase 1 | 2026-04-14 | product-owner | Rewrite (v2.0) |
