---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-002"
---

# BC-2.01.002: Cross-Client Fan-Out Query Aggregates Results with Per-Client Attribution

## Preconditions
- `client_id: null` is passed in the MCP tool call (indicating cross-client query)
- At least one client is configured with the target sensor enabled
- The target sensor type (e.g., CrowdStrike) is specified in the query

## Postconditions
- Results from all configured clients for the target sensor are aggregated in a single response
- Each result item includes a `client_id` field identifying its source client
- Response metadata includes `clients_queried` (list of clients that returned results)
- Response metadata includes `clients_skipped` with reasons for any clients not queried
- Pagination: cross-client queries return the first page from each client. Response metadata includes a `cursors` map (`client_id` -> `cursor_token`) for clients with additional pages (`has_more: true`). The agent can follow up with per-client queries (using the specific `client_id` and `cursor` from the map) to fetch subsequent pages. Cross-client responses never auto-paginate beyond the first page per client.

## Invariants
- DI-008: Per-result `client_id` attribution is never absent or incorrect
- DI-004: Exactly one AuditEntry emitted, recording the cross-client nature of the query

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | No clients are configured for the target sensor | Structured error: "No clients configured for sensor '{sensor}'" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-003 | 3 of 5 clients configured but 1 has expired credentials | Return results from 2 successful clients; include `partial_failures` array listing the failed client with error category and suggestion |
| DEC-005 | Cross-client CrowdStrike query but Client B only has Armis | Client B silently excluded; `clients_skipped` includes Client B with reason "sensor not configured" |
| EC-01-002 | All clients fail (e.g., all credentials expired) | Return empty results with all clients listed in `partial_failures`; this is not a tool-level error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
