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

# BC-2.07.007: In-Memory Pagination and Cache State Isolated Per-Client, Per-Sensor

## Preconditions
- Multiple clients are configured, each with one or more sensors and data sources
- Pagination state and response cache are held in-memory

## Postconditions
- In-memory pagination state is scoped by `(client_id, sensor_id, source_id)` per query session
- One client's pagination state is never visible to or affected by another client's queries
- Response cache entries (if subsystem 07 is used for caching) are also keyed by `(client_id, sensor_id, source_id)` plus query parameters
- Cache entries are bounded per client per sensor (see NFR-017 for cache bounds)
- No cross-client state leakage is possible; data structures enforce the tuple key

## Invariants
- DI-008: Client data separation -- pagination and cache state scoped per client
- No disk persistence for pagination state

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Memory pressure from many concurrent queries | Cache entries evicted via LRU; pagination tokens for active queries are not evicted |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-013 | Two clients query the same sensor type simultaneously | State is fully separate; each has its own in-memory entries keyed by client_id |
| EC-07-022 | Server restart clears all in-memory state | All pagination tokens and cache entries are lost; clients must start new queries. This is acceptable for an interactive tool. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-008 |
| Replaces | BC-2.07.007 v1.0 (file-based per-client state isolation) |
| Addresses | ADV-1-002, ADV-2-005 |
| Priority | P0 |
