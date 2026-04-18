---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flags"
capability: "CAP-006"
---

# BC-2.04.012: Token Content Hash Verification Prevents Action Tampering

## Preconditions
- A `ConfirmationToken` was created with an `action_hash` (SHA-256 of sorted action parameters)
- `confirm_action` is invoked to execute the operation

## Postconditions
- The `action_hash` stored in the token is compared against the hash that was computed at token creation time
- If the hash does not match (indicating the action parameters were modified), the token is rejected
- This prevents a scenario where an agent could create a token for "contain host A" and then use it to "contain host B"
- The hash includes: `client_id`, tool name, and all action-specific parameters (sorted and serialized to canonical JSON)

## Invariants
- DI-007: The confirmed action must match the original request
- Hash computation is deterministic (sorted keys, canonical JSON serialization)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `action_hash` mismatch | `code: "E-FLAG-005"`, `retryable: false`, suggestion: "The confirmation token does not match the requested action. Request a new token for the intended action." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-025 | Same action parameters produce different hashes due to JSON serialization order | Prevented by sorting keys before hashing; canonical JSON ensures deterministic output |
| EC-04-026 | Token created for `contain_host(host_id: "abc")`, confirmed for `contain_host(host_id: "xyz")` | Hash mismatch; rejected; agent must request a new token for host "xyz" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Priority | P1 |
