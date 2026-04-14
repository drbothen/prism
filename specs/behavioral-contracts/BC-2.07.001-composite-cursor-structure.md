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

# BC-2.07.001: Cursor Is a Composite of Timestamp and RecordID

## Preconditions
- A sensor adapter produces records from a data source
- Each record has at least a timestamp and a record identifier

## Postconditions
- The cursor is a composite value containing at minimum `(Timestamp, RecordID)`
- `Timestamp` is a UTC datetime providing temporal ordering
- `RecordID` is a string or numeric identifier providing tie-breaking within the same timestamp
- The cursor implements `PartialOrd` for comparison: Timestamp is compared first, then RecordID
- Cursor components are extensible: Claroty may use 2-tuple or 3-tuple cursors depending on the data source
- Cursors are serialized as JSON for persistence (field names and types are deterministic)

## Invariants
- DI-001: Cursor forward progress -- a new cursor must be >= the stored cursor

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Record lacks a usable timestamp across all fallback fields | Warning logged; record is included in response but contributes a null cursor (does not advance state) |
| `PrismError::Sensor` | Record lacks a usable record ID | Warning logged; timestamp-only cursor is used with degraded deduplication capability |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-013 | Armis record has no valid timestamp in any of the 1-3 fallback fields | Record included in results but does not advance cursor; if all records in a page lack timestamps, pagination halts to prevent infinite loops |
| DEC-010 | Claroty returns polymorphic ID (number in one record, string in next) | Both normalize to string for cursor comparison; `12345` and `"12345"` are equivalent |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-001 |
| Priority | P0 |
