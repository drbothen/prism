---
document_type: behavioral-contract
level: L3
version: "3.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "abc4070"
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

# BC-2.07.006: Cache Memory Bounds and Eviction Policy

**Note:** This file replaces BC-2.07.006 v2.0 "REMOVED -- Fingerprint Mismatch Detection". That contract was removed (persistent cursor model eliminated query fingerprints). This file now specifies cache memory bounds and eviction behavior for the response caching subsystem (CAP-014).

## Description

Each `(client_id, sensor_id)` cache partition is independently bounded at 50 entries by default (configurable via TOML). When a new insertion would exceed the bound, the Least Recently Used entry is synchronously evicted before insertion. LRU ordering is by most-recent access (read or write). For the initial 4-sensor deployment, worst-case cache memory is approximately 100MB, well within the NFR-015 512MB process budget.

## Preconditions
- The response cache contains cached entries and a new entry is about to be inserted
- The cache subsystem has a configurable per-client-per-sensor entry count bound

## Postconditions
- Each `(client_id, sensor_id)` pair has an independent entry count bound (default: 50 entries)
- When a new cache entry would exceed the bound for a given `(client_id, sensor_id)` pair, the Least Recently Used (LRU) entry is evicted before the new entry is inserted
- LRU ordering is determined by the most recent access time (read or write) of each entry
- Eviction is synchronous with the insert operation -- the caller does not observe the eviction
- Evicted entries are immediately freed (no background cleanup needed for bounded caches)
- The entry count bound is configurable via TOML configuration:
  ```toml
  [defaults.cache]
  max_entries_per_sensor = 50  # per (client_id, sensor_id) pair
  ```
- Memory budgeting: each CacheEntry stores a serialized response (`Vec<u8>`) plus metadata. The per-entry memory overhead is approximately 200 bytes of metadata plus the serialized response size. With the default of 50 entries per sensor and an average response size of ~10KB, the worst-case memory per typical deployment is: 50 clients x N sensors x 50 entries x ~10KB. For the initial 4 sensors, this is ~100MB (well within the 512MB NFR-015 memory budget). Operators adding many sensors should monitor cache memory and adjust `max_entries_per_sensor` accordingly.
- The global cache memory is bounded by: `num_clients * num_sensors_per_client * max_entries_per_sensor * avg_entry_size`

## Invariants
- DI-018: Cache entry count never exceeds the configured bound for any `(client_id, sensor_id)` pair
- Eviction is deterministic: the least-recently-used entry is always the eviction target
- Cache operations (insert, lookup, evict) are O(1) amortized (LRU implemented via HashMap + doubly-linked list or equivalent)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Eviction is a normal operational path, not an error | LRU eviction occurs transparently; no error is surfaced to the caller |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-050 | Cache is at capacity and a new entry is inserted | The LRU entry is evicted; the new entry is inserted. No error. |
| EC-07-051 | All entries in a `(client_id, sensor_id)` partition have the same access time | Eviction falls back to insertion order (FIFO) as a tiebreaker |
| EC-07-052 | `max_entries_per_sensor` set to 0 in config | Caching is effectively disabled for that sensor; every query hits the sensor API. No error. |
| EC-07-053 | Cross-client query (`client_id: null`) produces cached results for some clients but not others | Each client's cache partition is independent; cache hits and misses are per-client. Response includes a mix of cached and fresh data, transparent to the agent. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Insert 51st entry into a partition with `max_entries_per_sensor: 50` | LRU entry evicted; new entry inserted; partition count remains 50 | happy-path |
| All entries have same timestamp — evict one | FIFO (insertion order) tiebreaker selects oldest inserted entry | edge-case |
| `max_entries_per_sensor: 0` — query fires | Cache bypassed; sensor API called; no cache entry stored | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | LRU eviction keeps partition count at or below bound | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 |
| L2 Invariants | DI-018 |
| L2 Entity | CacheEntry (entities.md) |
| Addresses | ADV-6-001, ADV-6-002 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 3.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 3.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 3.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 3.0 | Phase 1 | 2026-04-14 | product-owner | Repurposed for cache memory bounds (replaced removed fingerprint-mismatch contract) |
