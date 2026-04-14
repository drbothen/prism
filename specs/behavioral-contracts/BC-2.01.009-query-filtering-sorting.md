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

# BC-2.01.009: Query Filtering and Sorting Parameters

## Preconditions
- A sensor query tool is invoked with optional filter and sort parameters
- Filter parameters may include: severity, status, time_range (start/end), and sensor-specific fields

## Postconditions
- Filters are translated to sensor-native query parameters (e.g., CrowdStrike filter syntax, Armis AQL WHERE clauses, Claroty POST body filters)
- Sort parameters are translated to sensor-native sort directives where supported
- The query fingerprint (SHA-256 of sorted filter fields + limit) is computed and stored alongside the cursor
- Only records matching the specified filters are returned

## Invariants
- DI-010: Query fingerprint consistency -- if filters change between runs, fingerprint mismatch is detected at startup

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Unrecognized filter field name for the target sensor | Structured error listing valid filter fields for that sensor |
| `PrismError::InvalidInput` | Time range `start` is after `end` | Structured error: "Invalid time range: start ({start}) must be before end ({end})" |
| `PrismError::InvalidInput` | Severity value not in valid set | Structured error listing valid severity values for the target sensor |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-012 | Operator changed filter parameters since last run | Fatal `PrismError::FingerprintMismatch` with both fingerprints and message to delete state file |
| EC-01-013 | No filters specified (fetch all) | Valid query; all records returned; fingerprint still computed from empty filter set + limit |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-010 |
| Priority | P0 |
