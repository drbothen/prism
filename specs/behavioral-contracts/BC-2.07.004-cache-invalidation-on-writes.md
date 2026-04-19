---
document_type: behavioral-contract
level: L3
version: "3.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-07"
capability: "CAP-014"
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

# BC-2.07.004: Cache Invalidation on Write Operations

**Note:** This file previously contained BC-2.07.004 v2.0 "REMOVED -- Cursor State Persisted After Delivery". That contract was removed (persistent cursor model replaced by ephemeral pagination tokens). This file is now repurposed for cache invalidation behavior.

## Preconditions
- A write operation (e.g., `confirm_action` executing a containment, alert acknowledgment, credential mutation) succeeds against a sensor API or credential store
- The response cache (CAP-014) contains one or more entries for the affected `(client_id, sensor_id, source_id)` tuple

## Postconditions
- All cache entries matching the `(client_id, sensor_id, source_id)` prefix of the completed write operation are invalidated **synchronously** before the write response is returned to the caller. Because the cache key is a 4-tuple `(client_id, sensor_id, source_id, query_hash)` with `source_id` as a first-class key component (not hashed), invalidation is a prefix scan on `(client_id, sensor_id, source_id)` that efficiently matches all `query_hash` variants for the affected source.
- The invalidation occurs after the write succeeds at the sensor/backend but before `confirm_action` returns its success response -- this ordering prevents stale reads after writes
- Subsequent queries for the same tuple will miss the cache and fetch fresh data from the sensor API
- If no cache entries exist for the affected tuple, the invalidation is a no-op (no error)
- Cache invalidation is logged in the AuditEntry for the write operation (number of entries evicted)

## Write Tool to source_id Mapping

Each write tool invalidates cache entries for the following source_id(s):

| Write Tool | source_id(s) Invalidated | Rationale |
|------------|--------------------------|-----------|
| `crowdstrike_contain_host` | `crowdstrike_hosts`, `crowdstrike_detections` | Containment changes host state and may affect detection status |
| `crowdstrike_acknowledge_alert` | `crowdstrike_alerts`, `crowdstrike_detections` | Acknowledgment changes alert/detection status |
| `cyberint_acknowledge_alert` | `cyberint_alerts` | Acknowledgment changes alert status |
| `cyberint_close_alert` | `cyberint_alerts` | Closing changes alert status |
| `claroty_resolve_alert` | `claroty_alerts` | Resolution changes alert status |
| `claroty_device_action` | `claroty_devices` | Device action changes device state |
| `armis_update_alert_status` | `armis_alerts` | Status update changes alert state |
| `armis_device_action` | `armis_devices` | Device action changes device state |
| `set_credential` | (none -- credential store, not sensor cache) | Credential mutations do not affect cached sensor query data |
| `delete_credential` | (none -- credential store, not sensor cache) | Credential mutations do not affect cached sensor query data |

This mapping is maintained in the write tool adapter layer. When a new write tool is added, the corresponding source_id invalidation set must be defined. **Each sensor adapter MUST define its invalidation mapping** — omitting a mapping for a write tool is a bug that will cause stale cache reads after writes.

## Invariants
- DI-018: Cache bounds (LRU eviction)
- Write-then-read consistency: a query issued after a successful write response will never return pre-write cached data for the affected tuple

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| E-CACHE-001 | Cache invalidation fails (e.g., mutex poisoned) | Log error, return cache invalidation failure. If the cache data structure is poisoned (e.g., mutex panic), the server is in an unrecoverable state and should terminate |

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
| Addresses | ADV-5-004, ADV-6-001, ADV-6-005 |
| Priority | P1 |
