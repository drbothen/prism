---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.011: Graceful Normalization Error Handling (No Silent Data Loss)

## Preconditions
- A sensor record is being normalized to OCSF
- The normalization process encounters an error (type mismatch, encoding failure, missing required context)

## Postconditions
- Missing OCSF fields produce valid OCSF messages with those fields absent (OCSF fields are optional by design)
- Type coercion failures result in the field being placed in `raw_extensions` instead of the OCSF message
- Protobuf encoding failures cause the record to be skipped with a logged error, but do not halt the batch
- A warning-level log entry is emitted for each normalization issue, identifying the record, field, and issue
- The cursor advances past skipped records to prevent re-processing infinite loops

## Invariants
- DI-005: Invalid protobuf messages are never delivered downstream
- No sensor data is silently dropped without a log entry

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | OCSF field mapping produces a type mismatch | Field diverted to `raw_extensions`; warning logged |
| Error (record skipped) | DynamicMessage fails to encode to valid protobuf bytes | Record skipped; error logged with record ID and sensor; cursor advances past it |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-020 | Entire batch of records fails normalization (e.g., sensor API changed response format) | All records skipped with errors logged; empty OCSF result set returned; cursor advances; alert-level log for "all records in batch failed normalization" |
| EC-02-021 | Single field causes normalization to take >1s (deeply nested JSON) | No per-field timeout; but total normalization time is included in response `_meta.query_time` for observability |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
