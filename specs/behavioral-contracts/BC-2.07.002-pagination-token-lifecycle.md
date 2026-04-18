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
capability: "CAP-011"
---

# BC-2.07.002: Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup

**Note:** This file replaces BC-2.07.002 v3.0. Pagination is now entirely internal to the query engine's sensor fetch layer. No pagination tokens or cursors are exposed to the MCP agent. The active cursor cap, cross-client cursor allocation, and token expiry semantics are reframed as internal resource management.

## Preconditions
- The query engine is executing a multi-page sensor API fetch as part of ephemeral materialization (BC-2.11.005)
- The fetch loop uses internal pagination tokens to traverse sensor API pages

## Postconditions

### Forward-Only Progress Within a Fetch
- Each successive page returns records that are forward of the previous page (no going backward)
- There is no mechanism to "rewind" or re-fetch earlier pages within a single fetch operation
- If the sensor API itself violates forward progress (returns duplicate or earlier records), Prism deduplicates at the adapter level

### Fetch Timeout (30s Total Query Budget)
- The entire query (including all sensor fetches) must complete within the 30-second query timeout (BC-2.11.006)
- If the timeout is reached mid-page-fetch, the pages already retrieved are materialized and the query proceeds with partial data
- A per-fetch warning is included in `sensor_errors` when the fetch was truncated by timeout

### Concurrent Fetch Limits
- A maximum of 200 concurrent sensor API fetch operations may be in progress at any time across all active queries
- New fetch operations beyond this cap are queued until existing fetches complete
- This cap prevents unbounded concurrent connections to sensor APIs during large cross-client fan-out queries

### Cross-Client Fetch Ordering (DEC-020)
- When concurrent fetch slots are limited, clients are processed in alphabetical order by `client_id`
- This ensures deterministic, fair allocation rather than race-condition-dependent ordering

## Invariants
- Pagination within a single fetch operation is forward-only
- No disk persistence is involved; forward progress is enforced in-memory
- Concurrent sensor API fetch count never exceeds 200

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Timeout | Query timeout (30s) reached during page fetch | Partial results from pages already fetched are materialized; `sensor_errors` includes truncation notice |
| `PrismError::Sensor` | Sensor API returns error mid-pagination | Partial results from successful pages are materialized; error reported in `sensor_errors` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-020 | Sensor API returns duplicate records across pages | Prism adapter deduplicates by record ID within the fetch |
| EC-07-022 | Sensor API cursor expires server-side during a long multi-page fetch | Partial results from pages already retrieved are used; error in `sensor_errors` |
| EC-07-023 | Cross-client query for 50 clients, each needing multi-page fetches | Fetch operations are queued beyond the 200 concurrent cap; alphabetical client ordering for fairness |
| DEC-020 | Cross-client fetch ordering fairness | Alphabetical client_id ordering; deterministic |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Entity | Cursor (entities.md) |
| Replaces | BC-2.07.002 v3.0 (MCP-exposed pagination token lifecycle) |
| Priority | P0 |
