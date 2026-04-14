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

# BC-2.04.009: Confirmation Token Generation for Irreversible Write Operations

## Preconditions
- An irreversible write tool is invoked (e.g., `contain_host`, `quarantine_file`)
- The tool is enabled (both compile-time and runtime feature flags pass)
- The caller's parameters are valid

## Postconditions
- The tool does NOT execute the write operation
- Instead, it returns a `ConfirmationToken` containing:
  - `token_id`: cryptographic random string
  - `action_summary`: human-readable description (e.g., "Isolate host abc (10.0.1.5) from network for client acme-corp")
  - `action_hash`: SHA-256 of the action parameters (client_id, tool, params)
  - `expires_at`: `created_at + 300s` (5 minutes)
- The token is stored in Prism's in-memory token store (not persisted to disk)
- The response clearly instructs the agent: "Call `confirm_action(token)` to execute, or let the token expire to cancel"

## Invariants
- DI-007: Token is valid for exactly 300 seconds
- The action is NOT executed during this step

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Action parameters fail validation | Structured error before token generation; no token created |
| `PrismError::Sensor` | Cannot verify the target entity exists (e.g., host ID not found) | Structured error; no token created; prevents generating tokens for nonexistent targets |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-018 | Agent requests a token for the same action twice | Two independent tokens are created; both are valid until consumed or expired |
| EC-04-019 | Token store grows unbounded (many unconsumed tokens) | Expired tokens are lazily cleaned up on each new token request; memory bounded by active token count |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Priority | P1 |
