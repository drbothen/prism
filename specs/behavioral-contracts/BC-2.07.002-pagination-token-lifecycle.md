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
capability: "CAP-011"
---

# BC-2.07.002: Pagination Token Lifecycle — Forward Progress, Expiry, and Cleanup

**Note:** This file replaces BC-2.07.002 v2.0 "Forward-Only Pagination Within a Single Query Session". The scope has been expanded to cover the full pagination token lifecycle: forward-only progress within a session, token expiry (600s TTL), lazy cleanup, active cursor cap, and cross-client cursor allocation.

## Preconditions
- A multi-page query is in progress for a `(client_id, sensor_id, source_id)` tuple
- The caller provides a pagination cursor from a previous page response, or requests a new paginated query

## Postconditions

### Forward-Only Progress Within a Session
- Each successive page returns records that are forward of the previous page (no going backward)
- The server validates that the pagination cursor references the current query session and represents a valid forward position
- There is no mechanism to "rewind" or paginate backward within a query; the caller must start a new query to re-read earlier data
- If the sensor API itself violates forward progress (returns duplicate or earlier records), Prism deduplicates at the adapter level
- This constraint applies only to cursor-based page traversal within one paginated session, not to independent query invocations
- There is no forward-only progress invariant across separate queries; the same query may be re-issued for fresh results (per DI-001)

### Token Expiry (600s TTL)
- Pagination cursors expire 600 seconds after creation (`expires_at = created_at + 600s`)
- The 600s TTL (vs 300s for confirmation tokens) accounts for the fact that multi-page result navigation is interactive and may involve the analyst reviewing results between page fetches; 300s would be too aggressive for large result sets
- Expired cursors are rejected with `E-STATE-001` and a structured error guiding the agent to start a new query
- The TTL is configurable per deployment but defaults to 600s

### Lazy Cleanup
- Expired cursors are lazily cleaned up on each new cursor creation request
- Before checking the active cursor cap, the cursor store sweeps all expired cursors from memory
- There is no background cleanup thread; cleanup is piggy-backed on cursor creation

### Active Cursor Cap (200 Maximum)
- A maximum of 200 active (non-expired) cursors may exist at any time across all clients and sensors
- New cursor creation beyond this cap (after lazy cleanup of expired cursors) returns `E-STATE-002` with suggestion: "Too many active pagination sessions. Complete or abandon existing pagination sessions before starting new ones." (`E-STATE-002` is distinct from `E-STATE-001` (expired/invalid cursor) because cap-reached is retryable — cursors expire at 600s TTL.)
- This cap prevents unbounded memory growth from abandoned pagination sessions

### Cross-Client Cursor Allocation (DEC-020)
- When the cursor cap is approaching and a cross-client query (`client_id: null`) needs cursors for multiple clients, allocation follows these rules:
  - Clients are processed in alphabetical order by `client_id`
  - The first N clients (within the remaining cap budget) receive cursors; subsequent clients receive a partial failure indicating cursor cap reached
  - This ensures deterministic, fair allocation rather than race-condition-dependent ordering

## Invariants
- Pagination within a single query session is forward-only; there is no constraint across separate queries
- No disk persistence is involved; forward progress is enforced in-memory within the session
- Cursor count never exceeds 200 active (non-expired) cursors at any time
- Expired cursors are always cleaned up before cap enforcement

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STATE-001` | Cursor references an expired or unknown query session | Structured error: suggestion to start a new query |
| `E-STATE-002` | Active cursor cap (200) reached after lazy cleanup | Structured error: suggestion to complete or abandon existing pagination sessions. Retryable: true (cursors expire at 600s TTL). |
| `PrismError::InvalidInput` | Cursor token cannot be decoded | Structured error: suggestion to start a new query without a cursor |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-020 | Sensor API returns duplicate records across pages | Prism adapter deduplicates by record ID within the session |
| EC-07-021 | First page request (no cursor) | Always valid; starts from the beginning of the result set |
| EC-07-022 | Cursor used after 600s TTL expires | `E-STATE-001` returned; agent must re-issue the original query |
| EC-07-023 | 199 active cursors, cross-client query for 3 clients | Lazy cleanup runs first; if only 1 slot remains after cleanup, first client (alphabetically) gets a cursor, remaining 2 receive partial failure |
| DEC-020 | Cross-client query cursor allocation fairness | Alphabetical client_id ordering; first-N-within-cap allocation |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Entity | Cursor (entities.md) |
| Replaces | BC-2.07.002 v2.0 (forward-only pagination only) |
| Addresses | ADV-1-002, ADV-2-005, ADV-7-002, ADV-7-011 |
| Priority | P0 |
