---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Cursor State Management"
capability: "CAP-011"
---

# BC-2.07.006: Fingerprint Mismatch at Startup Is a Fatal Error

## Preconditions
- Prism starts up and loads existing cursor state from a state file
- The state file contains a stored `QueryFingerprint`
- The current configuration produces a different fingerprint

## Postconditions
- `PrismError::FingerprintMismatch` is raised as a fatal error
- The error message includes:
  - The stored fingerprint (hex)
  - The current fingerprint (hex)
  - The path to the state file
  - An actionable instruction: "Query configuration has changed since the last run. Delete the state file at '{path}' to reset cursor state and re-fetch from the beginning."
- Prism does NOT silently reset the cursor or delete the state file
- The operator must explicitly acknowledge data re-fetch by deleting the file

## Invariants
- DI-010: Query fingerprint consistency -- mismatch is fatal with actionable message

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::FingerprintMismatch` | Stored fingerprint does not match current fingerprint | Fatal error; Prism exits with non-zero code after printing the actionable message |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-012 | Operator changed filter parameters between runs | Fatal error as specified; operator deletes state file to acknowledge full re-fetch |
| EC-07-010 | State file exists but has no fingerprint field (legacy or corrupted) | Treated as a mismatch; same fatal error with message noting "stored fingerprint: <missing>" |
| EC-07-011 | State file is corrupt JSON | `PrismError::Config` with message: "State file at '{path}' is corrupt. Delete it to reset." |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-010 |
| Priority | P0 |
