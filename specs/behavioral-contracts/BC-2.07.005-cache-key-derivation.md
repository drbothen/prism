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

# BC-2.07.005: Cache Key Derivation from Query Parameters

**Note:** This file replaces BC-2.07.005 v2.0 "REMOVED -- Query Fingerprint Computation". That contract was removed (persistent cursor model eliminated query fingerprints). This file now specifies cache key derivation for the response caching subsystem (CAP-014).

## Preconditions
- A sensor query tool is invoked with query parameters (filter, sort, etc.)
- The response cache subsystem needs to compute a cache key for lookup or storage

## Postconditions
- The full cache key is a 4-tuple: `(client_id, sensor_id, source_id, query_hash)`. The first three components are stored as plain values (not hashed), enabling prefix-scan invalidation by `(client_id, sensor_id, source_id)`. The `query_hash` distinguishes different queries within the same source.
- The `query_hash` component is computed as SHA-256 of the canonicalized query parameters
- **First-class key components** (stored as plain values, not part of the hash):
  - `client_id` (tenant scoping)
  - `sensor_id` (sensor scoping)
  - `source_id` (data source scoping, e.g., "alerts", "detections", "hosts")
- **Included in hash computation** (these define the logical query within a source):
  - Filter parameters: `severity`, `status`, `time_range`
  - Sort parameters (if any)
  - `page_size` -- included because the cache stores the paginated response; a request for page_size=10 and page_size=50 produce different response payloads
- **Excluded from hash computation** (these do not change the underlying query):
  - `cursor` -- pagination state changes per page but the underlying query is the same. This exclusion is safe because only first-page (cursor=null) requests are cached; requests with a non-null cursor always bypass the cache (see BC-2.07.003).
  - `force_refresh` -- bypass flag, not a query parameter
- Canonicalization ensures deterministic hashing:
  - Parameters are sorted alphabetically by key name
  - Null/absent parameters are omitted (not hashed as empty string)
  - String values are compared case-sensitively
  - The canonical form is a JSON object with sorted keys, serialized to a UTF-8 byte string, then SHA-256 hashed
- The resulting `query_hash` is a hex-encoded SHA-256 string (64 characters)

## Invariants
- Identical logical queries always produce the same `query_hash` regardless of parameter ordering in the MCP tool input
- Different logical queries always produce different `query_hash` values (SHA-256 collision resistance)
- The full cache key `(client_id, sensor_id, source_id, query_hash)` matches the CacheEntry entity definition in entities.md
- Cache invalidation by `(client_id, sensor_id, source_id)` is a prefix scan over the first three key components — no need to enumerate individual `query_hash` values

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Canonicalization always succeeds for valid tool inputs | Input validation occurs before cache key derivation; invalid inputs are rejected at the MCP tool handler level |

## Query Engine vs Direct Tool Cache Keys

The `query_hash` is derived differently depending on the caller:
- **Direct sensor query tools**: `query_hash` is computed from the tool's MCP input parameters (filter params, sort, page_size) as described above. These entries store a single page of results.
- **Query engine fan-out**: `query_hash` is computed from the sensor-native push-down filter parameters (the translated API params produced during query planning, not the original AxiQL string). These entries store the complete all-pages result set. Because the query engine fetches all pages and the direct tool fetches a single page, their `query_hash` values differ even for semantically overlapping queries — the inputs to the hash function are structurally different (push-down params vs tool params + page_size).

Both key types coexist in the same `(client_id, sensor_id, source_id)` partition and share LRU bounds and TTL.

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-040 | Two queries with same filters but different `page_size` | Different `query_hash` -- `page_size` is included in the hash because the cache stores the paginated response as-is. Each page_size variant is cached independently. |
| EC-07-041 | Query with `force_refresh: true` | `force_refresh` is excluded from hash; the `query_hash` matches the non-forced version. The cache bypass and replacement logic uses this hash to overwrite the existing entry |
| EC-07-042 | Query with all optional filter parameters absent vs. explicitly null | Both produce the same `query_hash` -- absent and null are treated identically (omitted from canonical form) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Entity | CacheEntry (entities.md) |
| Addresses | ADV-6-001, ADV-6-002 |
| Priority | P1 |
