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

# BC-2.01.003: Cursor-Based Forward-Only Pagination

## Preconditions
- A sensor query is initiated for a data source that uses cursor-based pagination (CrowdStrike, Cyberint, Armis)
- If resuming, a stored cursor exists for the `(client_id, sensor_id, source_id)` tuple

## Postconditions
- Each page response includes a `cursor` value in `_meta.pagination` for fetching the next page
- `has_more: true` indicates additional pages are available; `has_more: false` indicates the final page
- The new cursor is strictly >= the previous cursor (forward-only progress)
- Cursor is persisted to FileStore only AFTER successful delivery of the page to the caller

## Invariants
- DI-001: Cursor forward progress -- new cursor >= stored cursor
- DI-009: Persistence before state update -- FileStore.save() succeeds before in-memory cursor advances
- DI-013: Atomic state writes -- cursor persisted via temp-fsync-rename pattern

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::CursorRegression` | New cursor < stored cursor | Fatal error with both cursor values; collection halts for that source |
| `PrismError::Io` | FileStore persistence fails (disk full, permission denied) | Cursor does not advance; error surfaced to caller with suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-003 | First query with no stored cursor | Query starts from the beginning (no cursor parameter sent to sensor API); first page establishes the initial cursor |
| EC-01-004 | Page returns zero records but `has_more: true` | Cursor does not advance; next page requested with same cursor; adapter includes loop-detection counter to prevent infinite empty-page loops |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-009, DI-013 |
| Priority | P0 |
