---
document_type: behavioral-contract
level: L3
version: "3.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Pagination & Cache"
capability: "CAP-014"
---

# BC-2.07.004: Cache Invalidation on Write Operations

**Note:** This file previously contained BC-2.07.004 v2.0 "REMOVED -- Cursor State Persisted After Delivery". That contract was removed (persistent cursor model replaced by ephemeral pagination tokens). This file is now repurposed for cache invalidation behavior.

## Preconditions
- A write operation (e.g., `confirm_action` executing a containment, alert acknowledgment, credential mutation) succeeds against a sensor API or credential store
- The response cache (CAP-014) contains one or more entries for the affected `(client_id, sensor_id, source_id)` tuple

## Postconditions
- All cache entries matching the `(client_id, sensor_id, source_id)` tuple of the completed write operation are invalidated **synchronously** before the write response is returned to the caller
- The invalidation occurs after the write succeeds at the sensor/backend but before `confirm_action` returns its success response -- this ordering prevents stale reads after writes
- Subsequent queries for the same tuple will miss the cache and fetch fresh data from the sensor API
- If no cache entries exist for the affected tuple, the invalidation is a no-op (no error)
- Cache invalidation is logged in the AuditEntry for the write operation (number of entries evicted)

## Invariants
- DI-018: Cache bounds (LRU eviction)
- Write-then-read consistency: a query issued after a successful write response will never return pre-write cached data for the affected tuple

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Cache invalidation itself cannot fail — it is an in-memory map deletion | If the cache data structure is poisoned (e.g., mutex panic), the server is in an unrecoverable state and should terminate |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-018 | Agent writes and immediately queries the same data source | Cache was invalidated synchronously; the query hits the sensor API and populates a fresh cache entry |
| EC-07-010 | Write affects a source_id that has no cached entries | Invalidation is a no-op; no error raised |
| EC-07-011 | Concurrent write and read for the same tuple | Synchronization (lock) ensures the read either sees pre-invalidation cached data or misses and fetches fresh; no partial state |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Invariants | DI-018 |
| L2 Edge Cases | DEC-018 |
| Addresses | ADV-5-004 |
| Priority | P1 |
