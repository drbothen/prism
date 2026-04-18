---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flags"
capability: "CAP-006"
---

# BC-2.04.009: Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap)

## Preconditions
- An irreversible write tool is invoked (e.g., `contain_host`, `quarantine_file`)
- The tool is enabled (both compile-time and runtime feature flags pass)
- The caller's parameters are valid

## Postconditions
- The tool does NOT execute the write operation
- Instead, it returns a `ConfirmationToken` containing:
  - `token_id`: cryptographic random string
  - `client_id`: the `TenantId` of the client this action targets (prevents cross-client token replay)
  - `tool_name`: the originating write tool name (e.g., `crowdstrike_contain_host`, `set_credential`)
  - `action_params`: the original tool parameters as `serde_json::Value` (enables `confirm_action` to re-dispatch the write without the caller re-supplying params)
  - `action_summary`: human-readable description (e.g., "Isolate host abc (10.0.1.5) from network for client acme-corp")
  - `action_hash`: SHA-256 of the action parameters (client_id, tool, params)
  - `expires_at`: `created_at + 300s` (5 minutes)
- The token is stored in Prism's in-memory token store (not persisted to disk)
- The response clearly instructs the agent: "Call `confirm_action(token)` to execute, or let the token expire to cancel"
- Hard cap: the token store enforces a maximum of 100 active (non-expired) tokens. If the cap is reached, token creation fails with `E-FLAG-007`.
- Proactive cleanup: on each token creation request, expired tokens are swept from the store before checking the cap.

## Invariants
- DI-007: Token is valid for exactly 300 seconds
- The action is NOT executed during this step
- Token store never exceeds 100 active tokens

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Action parameters fail validation | Structured error before token generation; no token created |
| `PrismError::Sensor` | Cannot verify the target entity exists (e.g., host ID not found) | Structured error; no token created; prevents generating tokens for nonexistent targets |
| `PrismError::Flag` | Token cap reached (100 active tokens after cleanup) | Structured error: `code: "E-FLAG-007"`, `message: "Token store capacity reached (100 active tokens)"`, `suggestion: "Wait for existing tokens to expire or confirm/cancel pending actions before requesting new ones"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-018 | Agent requests a token for the same action twice | Two independent tokens are created; both are valid until consumed or expired (subject to cap) |
| EC-04-019 | Token store at 99 active tokens, new request arrives | Expired tokens cleaned up first; if cleanup frees space, token is created; if still at 100 after cleanup, `E-FLAG-007` |
| EC-04-034 | Server restart loses all in-memory tokens | All pending confirmations are lost; agent must re-request tokens. This is acceptable: tokens are short-lived (5 min) and the agent can retry. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Addresses | ADV-1-003, ADV-2-002, ADV-5-002, ADV-5-008 |
| Priority | P1 |
