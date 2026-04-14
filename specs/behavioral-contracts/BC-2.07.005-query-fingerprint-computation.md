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

# BC-2.07.005: Query Fingerprint Is SHA-256 of Sorted Config Fields

## Preconditions
- A data source has query configuration (filter parameters, limit, sort fields)
- The fingerprint is computed at startup and stored alongside the cursor state

## Postconditions
- The fingerprint is computed as: `SHA-256(sorted(config_fields) + limit)`
- `config_fields` is a `BTreeMap<String, String>` of all query-affecting parameters, sorted by key for determinism
- The `limit` value is included because changing it alters cursor behavior
- The fingerprint is hex-encoded (lowercase) for human readability in error messages
- Fields that do not affect query results (e.g., display names, logging levels) are excluded from the fingerprint

## Invariants
- DI-010: Query fingerprint consistency -- fingerprint is computed deterministically from sorted fields

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Fingerprint computation is pure and cannot fail | If the input fields are deterministic, the output is deterministic |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-007 | Two different config states produce the same fingerprint (hash collision) | Astronomically unlikely with SHA-256; treated as a non-concern |
| EC-07-008 | A new config field is added to the query parameters in a Prism upgrade | The fingerprint changes; this triggers a fingerprint mismatch on startup (per BC-2.07.006) |
| EC-07-009 | Config field value changes but the field set remains the same | Fingerprint changes because the value is included in the hash input |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-010 |
| Priority | P0 |
