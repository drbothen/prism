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

# BC-2.07.002: Cursor Regression Is Detected and Produces a Fatal Error

## Preconditions
- A collection cycle has produced a new cursor for a `(client_id, sensor_id, source_id)` tuple
- A stored cursor exists for the same tuple

## Postconditions
- The `ensure_forward_progress()` check verifies `new_cursor >= stored_cursor`
- If the check passes, the new cursor is eligible for persistence
- If the new cursor is strictly less than the stored cursor, `PrismError::CursorRegression` is raised
- The error message includes both cursor values for debugging: "Cursor regression: new cursor {new} <= stored {stored}"
- The collection cycle halts for that source; other sources are unaffected

## Invariants
- DI-001: Cursor forward progress -- cursor regression never occurs under correct operation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::CursorRegression` | New cursor timestamp is earlier than stored cursor timestamp | Fatal error for that source; operator must investigate (possible clock skew, API bug, or data corruption) |
| `PrismError::CursorRegression` | Same timestamp but new record ID sorts before stored record ID | Fatal error; same investigation path |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-001 | New cursor is exactly equal to stored cursor (no new records) | Valid; the cursor does not advance but no regression error is raised; the collection cycle completes with zero new records |
| EC-07-002 | First collection cycle (no stored cursor exists) | Any new cursor is valid; no regression check is performed; the cursor is persisted as the initial state |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-001 |
| Priority | P0 |
