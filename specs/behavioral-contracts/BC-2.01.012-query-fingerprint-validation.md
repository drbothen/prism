---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.012: Query Fingerprint Validation at Startup

## Preconditions
- Prism starts up and finds an existing cursor state file for a `(client_id, sensor_id, source_id)` tuple
- The state file contains a stored query fingerprint (SHA-256 hash)

## Postconditions
- The current configuration's query fingerprint is computed from sorted filter fields + limit
- If the stored fingerprint matches the current fingerprint, the cursor is valid and collection resumes
- If the fingerprints do not match, startup fails with a fatal error

## Invariants
- DI-010: Query fingerprint consistency -- mismatch is fatal

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::FingerprintMismatch` | Stored fingerprint != current fingerprint | Fatal error: "Query configuration has changed since the last run. Delete the state file at '{path}' to reset cursor state. Stored fingerprint: {stored}. Current fingerprint: {current}." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-012 | Operator changed filter parameters between runs | Fatal error with actionable message; operator must explicitly delete state file |
| EC-01-018 | State file exists but is empty or corrupted | Treated as corrupted state; fatal error with message to delete and re-initialize |
| EC-01-019 | No state file exists (first run) | No fingerprint to compare; new fingerprint stored alongside cursor after first successful query |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-010 |
| Priority | P0 |
