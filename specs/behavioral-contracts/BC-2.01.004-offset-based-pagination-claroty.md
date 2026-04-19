---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
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

# BC-2.01.004: Offset-Based Hybrid Pagination for Claroty Audit Logs

## Preconditions
- The query targets the Claroty xDome `audit_logs` data source
- The Claroty adapter is initialized with valid bearer token credentials

## Postconditions
- Pagination uses offset-based mechanics (Claroty audit_log API does not support cursor-based pagination)
- The hybrid cursor combines a timestamp component with an offset count
- Records are returned in the order provided by the Claroty API
- Forward-only progress is maintained via the composite (timestamp, offset) cursor

## Invariants
- DI-001: Cursor forward progress -- composite (timestamp, offset) cursor never regresses

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Claroty API returns HTTP 400 for invalid offset | Structured error with `category: "api_contract"` and the rejected offset value |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-005 | Offset exceeds total record count | Claroty returns empty page; pagination halts with `has_more: false` |
| EC-01-006 | New audit log records inserted during paginated traversal causing offset drift | Accepted as a known limitation of offset pagination; duplicate records possible, deduplicated by record ID at the adapter layer |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001 |
| Priority | P0 |
