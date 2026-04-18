---
document_type: behavioral-contract
level: L3
version: "3.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Adapter Pagination & Response Cache"
capability: "CAP-011"
---

# BC-2.07.001: Internal Ephemeral Pagination Token Structure

**Note:** This file replaces BC-2.07.001 v2.0. Pagination is now entirely internal to the query engine's sensor fetch layer. No pagination tokens are exposed to the MCP agent. The agent uses `limit` and `total_available` on the `query` tool (BC-2.11.001) instead of cursor-based page traversal.

## Preconditions
- The query engine (BC-2.11.005) initiates a multi-page sensor API fetch as part of ephemeral materialization
- The sensor adapter produces records from a data source in pages
- Each page response from the sensor API includes a continuation token or offset for the next page

## Postconditions
- The pagination token is an opaque, ephemeral value held in-memory for the duration of the sensor fetch
- The token encapsulates the sensor-specific pagination state (e.g., CrowdStrike offset string, Claroty page number, Armis AQL cursor)
- The query engine uses the token internally to fetch successive pages from the sensor API until all pages are retrieved (up to the security limit in BC-2.11.006)
- Tokens are never exposed to the MCP agent -- the agent never sees or provides pagination cursors
- Tokens are never persisted to disk. They exist only in the server's in-memory state for the active fetch.
- Token structure is internal to Prism and is not a public API contract

## Invariants
- Pagination tokens are ephemeral (in-memory only, no disk persistence)
- Pagination tokens are internal to the query engine's fetch layer (never in MCP responses)
- Token deserialization failure produces a structured error, not a panic

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Sensor API rejects the pagination token mid-fetch (e.g., server-side cursor expired) | Partial results from pages already fetched are materialized; error reported in `sensor_errors` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-001 | Sensor API returns a cursor type that differs between pages (e.g., numeric then string) | Token encapsulates the raw value; Prism normalizes internally |
| DEC-010 | Claroty returns polymorphic ID (number in one record, string in next) | Both normalize to string within the token; `12345` and `"12345"` are equivalent |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| Replaces | BC-2.07.001 v2.0 (MCP-exposed ephemeral pagination tokens) |
| Priority | P0 |
