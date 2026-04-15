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

# BC-2.07.003: Response Cache with Configurable TTL

**Note:** This file replaces BC-2.07.003 v2.0 "REMOVED -- Atomic File Writes for Cursor State". That contract was removed (persistent cursor model replaced by ephemeral pagination tokens). This file now specifies response caching behavior.

## Preconditions
- A sensor query tool is invoked (e.g., `get_crowdstrike_alerts`) with `force_refresh: false` (the default)
- The response cache subsystem is initialized in memory

## Postconditions
- Before issuing a sensor API call, the cache is checked for an entry matching the `(client_id, sensor_id, query_hash)` tuple
- If a cache hit is found and the entry has not exceeded its TTL, the cached response is returned immediately without contacting the sensor API
- If no cache entry exists or the TTL has expired, the sensor API is queried, the response is stored in the cache with the configured TTL, and the fresh response is returned
- TTL values are configurable per data source type:
  - Alerts / detections: 60 seconds (default) -- high-churn data requiring freshness
  - Devices / hosts / assets: 300 seconds (default) -- lower-churn inventory data
  - Health / status endpoints: not cached (always live)
- When `force_refresh: true` is set, the cache is bypassed and any existing entry for the tuple is replaced with the fresh response
- Cache hits increment the `hit_count` on the CacheEntry for metrics visibility via `check_sensor_health`

## Invariants
- DI-018: Cache bounds (LRU eviction when entry count exceeds configurable per-client-per-sensor bound)
- Cached responses are byte-identical to the original sensor API response; no transformation occurs between cache write and cache read
- TTL is measured from `created_at` of the CacheEntry, not from last access (TTL, not sliding expiration)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Cache miss | Normal path -- query sensor API and populate cache |
| N/A | Cache serialization failure | Log warning, proceed as cache miss, query sensor API directly |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-030 | Two concurrent queries for the same tuple arrive simultaneously, both miss cache | One query populates the cache; the other may also query the sensor API (no request coalescing in v1). Both return correct results. |
| EC-07-031 | TTL expires between cache check and response return | Stale-by-milliseconds response is acceptable; next request will refresh |
| EC-07-032 | `force_refresh: true` with no existing cache entry | Sensor API is queried; result is cached normally |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Invariants | DI-018 |
| Addresses | ADV-5-004, ADV-6-001 |
| Priority | P1 |
