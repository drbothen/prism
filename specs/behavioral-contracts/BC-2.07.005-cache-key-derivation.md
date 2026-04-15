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
- The cache key (`query_hash`) is computed as SHA-256 of the canonicalized query parameters
- **Included in hash computation** (these define the logical query):
  - `client_id` (tenant scoping)
  - `sensor_id` (sensor scoping)
  - `source_id` (data source scoping, e.g., "alerts", "detections", "hosts")
  - Filter parameters: `severity`, `status`, `time_range`
  - Sort parameters (if any)
  - `page_size` -- included because the cache stores the paginated response; a request for page_size=10 and page_size=50 produce different response payloads
- **Excluded from hash computation** (these do not change the underlying query):
  - `cursor` -- pagination state changes per page but the underlying query is the same
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
- The `query_hash` derivation matches the CacheEntry entity definition in entities.md

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Canonicalization always succeeds for valid tool inputs | Input validation occurs before cache key derivation; invalid inputs are rejected at the MCP tool handler level |

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
