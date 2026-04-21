---
document_type: behavioral-contract
level: L3
version: "4.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "67e5667"
traces_to: ["CAP-014"]
extracted_from: ".factory/specs/prd.md"
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

# BC-2.07.005: Cache Key Derivation from Push-Down Parameters

**Note:** This file replaces BC-2.07.005 v3.0. With per-sensor read tools removed, there is only one cache key type: query engine push-down parameter hashes. There is no "direct tool query hash" vs "query engine push-down hash" distinction.

## Description

The full cache key is a 4-tuple `(client_id, sensor_id, source_id, push_down_hash)` where the first three components are stored as plain values enabling prefix-scan invalidation, and `push_down_hash` is the SHA-256 hex of the canonicalized sensor-native push-down filter parameters. The original PrismQL query string, `force_refresh`, and post-filters are excluded from the hash — two PrismQL queries with different syntax but identical push-down parameters share one cache entry. Canonicalization alphabetically sorts parameter keys and omits null/absent values.

## Preconditions
- The query engine has planned a sensor API fetch with push-down filter parameters (BC-2.11.007)
- The response cache subsystem needs to compute a cache key for lookup or storage

## Postconditions
- The full cache key is a 4-tuple: `(client_id, sensor_id, source_id, push_down_hash)`. The first three components are stored as plain values (not hashed), enabling prefix-scan invalidation by `(client_id, sensor_id, source_id)`. The `push_down_hash` distinguishes different queries within the same source.
- The `push_down_hash` component is computed as SHA-256 of the canonicalized sensor-native push-down filter parameters (the translated API params produced during query planning by BC-2.11.007, not the original PrismQL query string)
- **First-class key components** (stored as plain values, not part of the hash):
  - `client_id` (tenant scoping)
  - `sensor_id` (sensor scoping)
  - `source_id` (data source scoping, e.g., "alerts", "detections", "hosts")
- **Included in hash computation** (these define the sensor-native query):
  - Push-down filter parameters: the sensor-native translated filters (e.g., CrowdStrike FQL filter string, Armis AQL WHERE clause, Claroty POST body filters)
  - These are the parameters that actually change what the sensor API returns
- **Excluded from hash computation**:
  - The original PrismQL query string -- two different PrismQL queries that produce the same push-down filters share a cache entry
  - `force_refresh` -- bypass flag, not a query parameter
  - PrismQL post-filters (applied after fetch, not part of sensor API request)
  - `limit` on the `query` tool -- the cache stores the full sensor API response; `limit` is applied after materialization
- Canonicalization ensures deterministic hashing:
  - Parameters are sorted alphabetically by key name
  - Null/absent parameters are omitted (not hashed as empty string)
  - String values are compared case-sensitively
  - The canonical form is a JSON object with sorted keys, serialized to a UTF-8 byte string, then SHA-256 hashed
- The resulting `push_down_hash` is a hex-encoded SHA-256 string (64 characters)

## Invariants
- Identical push-down parameters always produce the same `push_down_hash` regardless of the PrismQL query that generated them
- Different push-down parameters always produce different `push_down_hash` values (SHA-256 collision resistance)
- The full cache key `(client_id, sensor_id, source_id, push_down_hash)` matches the CacheEntry entity definition in entities.md
- Cache invalidation by `(client_id, sensor_id, source_id)` is a prefix scan over the first three key components -- no need to enumerate individual hash values
- Only one cache key type exists: push-down parameter hashes. There is no separate "tool query hash."

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Canonicalization always succeeds for valid push-down parameters | Input validation occurs before cache key derivation; invalid inputs are rejected at the query planning level |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-040 | Two PrismQL queries with different syntax but same push-down filters | Same `push_down_hash` -- cache is shared between them |
| EC-07-041 | Query with `force_refresh: true` | `force_refresh` is excluded from hash; the `push_down_hash` matches the non-forced version. The cache bypass and replacement logic uses this hash to overwrite the existing entry |
| EC-07-042 | Query with all optional filter parameters absent vs. explicitly null | Both produce the same `push_down_hash` -- absent and null are treated identically (omitted from canonical form) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| PrismQL queries with different syntax, same sensor-native push-down filters | Identical `push_down_hash`; shared cache entry | happy-path |
| Same push-down params with keys in different order | Same `push_down_hash` (alphabetical sort) | happy-path |
| `force_refresh: true` vs `force_refresh: false`, same params | Same `push_down_hash`; cache bypass logic uses hash to overwrite entry | edge-case |
| Absent optional param vs explicit null param | Same `push_down_hash` (both omitted from canonical form) | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-025 | Cache key derivation: deterministic | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Entity | CacheEntry (entities.md) |
| Replaces | BC-2.07.005 v3.0 (dual direct-tool + query-engine cache keys) |
| Addresses | ADV-6-001, ADV-6-002 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 4.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 4.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 4.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 4.0 | Phase 1 | 2026-04-14 | product-owner | Repurposed: single cache key type; dual-hash model removed |
