---
document_type: behavioral-contract
level: L3
version: "4.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "8bd996e"
traces_to: ["CAP-011"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-07"
capability: "CAP-011"
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

# BC-2.07.002: Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup

**Note:** This file replaces BC-2.07.002 v3.0. Pagination is now entirely internal to the query engine's sensor fetch layer. No pagination tokens or cursors are exposed to the MCP agent. The active cursor cap, cross-client cursor allocation, and token expiry semantics are reframed as internal resource management.

## Description

The query engine's pagination lifecycle enforces forward-only progress within a single sensor API fetch, caps all concurrent sensor API fetch operations at 200, and bounds total fetch time to the 30-second query budget. Incomplete fetches due to timeout produce partial results with a `sensor_errors` warning rather than failing the entire query. Cross-client fan-out respects alphabetical client ordering when the 200-fetch cap is reached, ensuring deterministic fairness.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Multi-page fetch completing within 30s | All pages materialized; no `sensor_errors` entry | happy-path |
| Fetch timeout at 30s boundary mid-page | Partial results returned; `sensor_errors` includes truncation notice with pages fetched count | error |
| 201 concurrent fetch operations | 201st is queued; alphabetical client ordering for slot assignment | edge-case |
| Sensor API returns duplicate record IDs across page boundary | Deduplication at adapter level; each record appears once | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-029 | Cursor cap: rejects at 200 active | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Entity | Cursor (entities.md) |
| Replaces | BC-2.07.002 v3.0 (MCP-exposed pagination token lifecycle) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 4.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 4.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 4.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 4.0 | Phase 1 | 2026-04-14 | product-owner | Repurposed: pagination entirely internal; MCP exposure removed |
