---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Audit & Compliance"
capability: "CAP-007"
---

# BC-2.05.010: Confirmation Token Lifecycle Events Are Audit-Logged

## Preconditions
- An irreversible write operation produces a `ConfirmationToken` (issuance) or a `confirm_action` call consumes/rejects a token

## Postconditions
- **Token issuance**: audit entry records `result_summary: "confirmation_token_issued"` with the `action_summary` and token expiry time, but NOT the token ID
- **Token consumption (success)**: audit entry for `confirm_action` records `result_summary: "confirmed_and_executed"` with the action outcome
- **Token rejection (expired)**: audit entry records `result_summary: "token_expired"` with the original action summary
- **Token rejection (consumed)**: audit entry records `result_summary: "token_already_consumed"` with the original action summary
- All token lifecycle events include the `client_id`, `sensor`, and `tool_name` of the original write operation

## Invariants
- DI-007: Confirmation token expiry
- DI-004: Audit completeness

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Invalid token ID | `confirm_action` called with a token ID that does not exist | Audit entry records `result_summary: "token_not_found"` |
| Action hash mismatch | Token's `action_hash` does not match the `confirm_action` parameters | Audit entry records `result_summary: "action_hash_mismatch"` with both hashes for forensic review |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-009 | Token expired at exactly 300 seconds | Token is rejected; audit entry records `result_summary: "token_expired"` |
| EC-05-017 | Token issued but never consumed (analyst abandons the flow) | Only the issuance audit entry exists; no consumption entry; the token expires silently with no additional audit event |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004, DI-007 |
| Priority | P0 |
