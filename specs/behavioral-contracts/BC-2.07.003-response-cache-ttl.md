---
document_type: behavioral-contract
level: L3
version: "4.5"
status: draft
producer: product-owner
timestamp: 2026-04-14T00:00:00Z
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-014"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-07"
capability: "CAP-014"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.003: Query Engine Sensor-Fetch Cache with Configurable TTL

**Note:** This file replaces BC-2.07.003 v3.0. With per-sensor read tools removed, only one cache type exists: the query engine's sensor-fetch cache. There is no "direct tool cache" -- all data access goes through the query engine.

## Description

The query engine maintains a single in-memory sensor-fetch cache keyed by `(client_id, sensor_id, source_id, push_down_hash)`. Cache entries store raw sensor API responses (pre-OCSF normalization) with configurable TTLs by data type: 60 seconds for high-churn alerts/detections, 300 seconds for lower-churn device/asset inventory, and no caching for health/status endpoints. Two distinct PrismQL queries that produce identical sensor-native push-down parameters share the same cache entry. The `force_refresh` parameter bypasses the cache and replaces the existing entry. A forced refresh whose fetch cannot produce a complete replacement invalidates (removes) the existing entry rather than retaining it (architect adjudication D3).

## Preconditions
- The query engine initiates a sensor API fetch as part of ephemeral materialization (BC-2.11.005)
- The response cache subsystem is initialized in memory
- The query's `force_refresh` parameter is `false` (the default)

## Postconditions
- Before issuing sensor API calls, the cache is checked for an entry matching the `(client_id, sensor_id, source_id, push_down_hash)` tuple
- The `push_down_hash` is the canonical hash of the sensor-native push-down filter parameters (the translated API params produced by BC-2.11.007, not the original PrismQL query string)
- Two different PrismQL queries that produce the same sensor-native push-down filters share the same cache entry
- If a cache hit is found and the entry has not exceeded its TTL, the cached sensor response is returned to the query engine without contacting the sensor API
- If no cache entry exists or the TTL has expired, the sensor API is queried (all pages fetched, up to the effective fetch-limit when limit push-down applies — BC-2.01.013 v1.14), and the complete response *for that effective fetch-limit* is stored in the cache with the configured TTL, and the fresh response is returned
- The cache stores the complete result set returned by the fan-out fetch for the effective fetch-limit (pre-OCSF-normalization sensor records)
- Cache keys distinguish effective fetch-limits per BC-2.07.005 v4.4 — entries truncated at different limits never alias
- The query engine's OCSF normalization and PrismQL post-filters are applied after cache retrieval, not before -- the cache stores raw sensor responses
- TTL values are configurable per data source type:
  - Alerts / detections: 60 seconds (default) -- high-churn data requiring freshness
  - Devices / hosts / assets: 300 seconds (default) -- lower-churn inventory data
  - Health / status endpoints: not cached (always live)
- When `force_refresh: true` is set on the `query` tool, the cache is bypassed and any existing entry for the tuple is replaced with the fresh response
- When `force_refresh: true` and the fresh fetch fails to produce a complete response — either all targets failed, or per-target errors made the result partial (partial responses are never cached) — the existing cache entry for the tuple is **invalidated (removed)**, not retained. The fetch failure is surfaced to the forcing caller via the partial-failure envelope (BC-2.11.011). Subsequent non-forced queries for the tuple miss the cache and attempt a fresh fetch. Rationale: `force_refresh` is an explicit analyst distrust signal; retaining the distrusted entry would silently serve it to later queries (architect adjudication D3, `proposals/cache-envelope-adjudication-2026-06-10.md`). A fetch failure on a **non-forced** query never invalidates an existing unexpired entry (availability semantics unchanged on the normal path). Invalidation is per-entry; sibling entries at other fetch-limits age out by TTL.
- Cache hits increment the aggregate `total_hits` counter on `QueryCache` for performance metrics. Per-entry hit counts were considered but rejected in S-3.05 implementation due to clone-on-hit cost (cloning `Vec<serde_json::Value>` on every hit); aggregate visibility is sufficient for `check_sensor_health` health metrics.

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
| EC-07-033 | `force_refresh: true`; fresh fetch fails for all targets | Existing entry for the tuple is invalidated; error surfaced to the forcing caller (BC-2.11.011); subsequent non-forced identical query is a cache miss and re-attempts the fetch |
| EC-07-034 | `force_refresh: true`; fresh fetch returns partial results (some targets errored) | Partial response is NOT cached (complete-responses-only); existing entry is invalidated; partial results + `sensor_errors` returned to the forcing caller |

## Cross-Client Query Cache Interaction

- Cross-client queries (`clients: null`) check and populate per-client cache partitions independently during fan-out
- Each client's cache partition is keyed by `(client_id, sensor_id, source_id, push_down_hash)` -- the cross-client query checks each client's partition separately
- Cache entries populated by cross-client fan-out are reusable by subsequent single-client queries with the same push-down parameters (and vice versa)
- A cross-client query may result in a mix of cache hits (for some clients) and cache misses (for others); this is transparent to the caller

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query with same push-down params as previous query (within TTL) | Cache hit; sensor API not called; `total_hits` aggregate counter incremented | happy-path |
| Query with `force_refresh: true` on cached entry | Cache bypassed; sensor API called; entry replaced with fresh response | happy-path |
| Alert query after TTL of 60s expires | Cache miss; fresh fetch from sensor API; new entry stored | edge-case |
| Two concurrent identical queries, both miss cache | Both return correct results; no coalescing; at most 2 API calls | edge-case |
| `force_refresh: true` on cached entry; fresh fetch fails for all targets | Existing entry invalidated (removed); error surfaced via partial-failure envelope (BC-2.11.011); subsequent non-forced identical query is a cache MISS (EC-07-033) | error |
| `force_refresh: true` on cached entry; fresh fetch partial (some targets errored) | Partial response not cached; existing entry invalidated; partial results + `sensor_errors` returned to forcing caller (EC-07-034) | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-025 | Cache key derivation: deterministic | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Invariants | DI-018 |
| Replaces | BC-2.07.003 v3.0 (dual direct-tool + query-engine cache) |
| Addresses | ADV-5-004, ADV-6-001, ADV-7-006 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 4.5 | QRY cascade pass-1 D1+D3 (review-2026-06-10 PO consolidated amendment burst) | 2026-06-10 | product-owner | Architect adjudication D1+D3 (`proposals/cache-envelope-adjudication-2026-06-10.md`). D1 limit-premise sentences: miss-path postcondition now reads "all pages fetched, up to the effective fetch-limit when limit push-down applies — BC-2.01.013 v1.14" with "the complete response *for that effective fetch-limit*" stored; full-result-set bullet now "the complete result set returned by the fan-out fetch for the effective fetch-limit" (parenthetical preserved byte-identical); cross-reference added: cache keys distinguish effective fetch-limits per BC-2.07.005 v4.4. D3 forced-refresh-failure postcondition: forced refresh that cannot store a complete replacement (all targets failed OR partial — partial responses are never cached) INVALIDATES the existing entry; failure surfaced via BC-2.11.011 envelope; non-forced fetch failures never invalidate unexpired entries; invalidation per-entry, sibling fetch-limit entries age out by TTL. Added EC-07-033/034 + two mirroring canonical test vectors; Description sentence appended. P1-03 constraint respected: all normalization language ("pre-OCSF-normalization", "raw sensor responses", "no transformation applied before caching") byte-frozen pending human adjudication. |
| 4.4 | S-3.05-CR-001 | 2026-05-07 | implementer | CR-001 closure: replaced per-entry `hit_count` on `CacheEntry` with aggregate `total_hits: AtomicU64` on `QueryCache`. Design rationale: per-entry counter required cloning `Vec<serde_json::Value>` on every cache hit; aggregate counter avoids the clone cost while retaining sufficient visibility for `check_sensor_health` metrics (CR-005 design accepted in S-3.05 pass-2). |
| 4.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 4.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 4.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 4.0 | Phase 1 | 2026-04-14 | product-owner | Repurposed: single cache type; dual-cache model removed |
