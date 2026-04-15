---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-006"
---

# BC-2.04.010: Confirmation Token Consumption via confirm_action

## Preconditions
- The `confirm_action` MCP tool is invoked with a `client_id` and `token_id`
- A matching token exists in the in-memory token store

## Postconditions
- The token is validated: not expired, not already consumed, client_id matches the token's embedded client_id, action_hash matches
- If valid, the original write operation is executed against the sensor API
- The token is marked as `consumed: true` immediately before execution (single-use)
- The execution result is returned to the caller
- Both the token validation and the execution are audit-logged

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Priority | P1 |
