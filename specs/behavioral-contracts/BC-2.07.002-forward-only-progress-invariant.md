---
document_type: behavioral-contract
level: L3
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Pagination & Cache"
capability: "CAP-011"
---

# BC-2.07.002: Pagination Moves Forward Within a Query

## Preconditions
- A multi-page query is in progress for a `(client_id, sensor_id, source_id)` tuple
- The caller provides a pagination cursor from a previous page response

## Postconditions
- Each successive page returns records that are forward of the previous page (no going backward)
- The server validates that the pagination cursor references the current query session and represents a valid forward position
- There is no mechanism to "rewind" or paginate backward within a query; the caller must start a new query to re-read earlier data
- If the sensor API itself violates forward progress (returns duplicate or earlier records), Prism deduplicates at the adapter level

## Invariants
- Pagination within a query session is forward-only
- No disk persistence is involved; forward progress is enforced in-memory within the session

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Cursor references an expired or unknown query session | Structured error: suggestion to start a new query |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-020 | Sensor API returns duplicate records across pages | Prism adapter deduplicates by record ID within the session |
| EC-07-021 | First page request (no cursor) | Always valid; starts from the beginning of the result set |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| Replaces | BC-2.07.002 v1.0 (persistent cursor regression detection) |
| Addresses | ADV-1-002, ADV-2-005 |
| Priority | P0 |
