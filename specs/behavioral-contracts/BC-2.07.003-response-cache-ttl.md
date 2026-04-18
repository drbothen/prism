---
document_type: behavioral-contract
level: L3
version: "4.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Adapter Pagination & Response Cache"
capability: "CAP-014"
---

# BC-2.07.003: Query Engine Sensor-Fetch Cache with Configurable TTL

**Note:** This file replaces BC-2.07.003 v3.0. With per-sensor read tools removed, only one cache type exists: the query engine's sensor-fetch cache. There is no "direct tool cache" -- all data access goes through the query engine.

## Preconditions
- The query engine initiates a sensor API fetch as part of ephemeral materialization (BC-2.11.005)
- The response cache subsystem is initialized in memory
- The query's `force_refresh` parameter is `false` (the default)

## Postconditions
- Before issuing sensor API calls, the cache is checked for an entry matching the `(client_id, sensor_id, source_id, push_down_hash)` tuple
- The `push_down_hash` is the canonical hash of the sensor-native push-down filter parameters (the translated API params produced by BC-2.11.007, not the original PrismQL query string)
- Two different PrismQL queries that produce the same sensor-native push-down filters share the same cache entry
- If a cache hit is found and the entry has not exceeded its TTL, the cached sensor response is returned to the query engine without contacting the sensor API
- If no cache entry exists or the TTL has expired, the sensor API is queried (all pages fetched), the complete response is stored in the cache with the configured TTL, and the fresh response is returned
- The cache stores the full result set from the all-pages fan-out fetch (pre-OCSF-normalization sensor records)
- The query engine's OCSF normalization and PrismQL post-filters are applied after cache retrieval, not before -- the cache stores raw sensor responses
- TTL values are configurable per data source type:
  - Alerts / detections: 60 seconds (default) -- high-churn data requiring freshness
  - Devices / hosts / assets: 300 seconds (default) -- lower-churn inventory data
  - Health / status endpoints: not cached (always live)
- When `force_refresh: true` is set on the `query` tool, the cache is bypassed and any existing entry for the tuple is replaced with the fresh response
- Cache hits increment the `hit_count` on the CacheEntry for metrics visibility via `check_sensor_health`

## Invariants
- DI-018: Cache bounds (LRU eviction when entry count exceeds configurable per-client-per-sensor bound)
- The cached response is the exact sensor API response that was fetched -- no transformation applied before caching
- TTL is measured from `created_at` of the CacheEntry, not from last access (TTL, not sliding expiration)
- Only one cache type exists: query engine sensor-fetch cache. There is no separate "direct tool cache."

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Cache miss | Normal path -- fetch from sensor API and populate cache |
| N/A | Cache serialization failure | Log warning, proceed as cache miss, query sensor API directly |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-030 | Two concurrent queries for the same tuple arrive simultaneously, both miss cache | One query populates the cache; the other may also query the sensor API (no request coalescing in v1). Both return correct results. |
| EC-07-031 | TTL expires between cache check and response return | Stale-by-milliseconds response is acceptable; next request will refresh |
| EC-07-032 | `force_refresh: true` with no existing cache entry | Sensor API is queried; result is cached normally |

## Cross-Client Query Cache Interaction

- Cross-client queries (`clients: null`) check and populate per-client cache partitions independently during fan-out
- Each client's cache partition is keyed by `(client_id, sensor_id, source_id, push_down_hash)` -- the cross-client query checks each client's partition separately
- Cache entries populated by cross-client fan-out are reusable by subsequent single-client queries with the same push-down parameters (and vice versa)
- A cross-client query may result in a mix of cache hits (for some clients) and cache misses (for others); this is transparent to the caller

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Invariants | DI-018 |
| Replaces | BC-2.07.003 v3.0 (dual direct-tool + query-engine cache) |
| Addresses | ADV-5-004, ADV-6-001, ADV-7-006 |
| Priority | P1 |
