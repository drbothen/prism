---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.003: Ephemeral Cursor-Based Forward-Only Pagination

## Preconditions
- A sensor query is initiated for a data source that uses cursor-based pagination (CrowdStrike, Cyberint, Armis)
- The query is an interactive MCP tool invocation (request/response, not background polling)

## Postconditions
- Each page response includes a `cursor` value in `_meta.pagination` for fetching the next page
- `has_more: true` indicates additional pages are available; `has_more: false` indicates the final page
- The pagination cursor is an opaque, ephemeral token: returned in the response, passed back by the caller for the next page, and discarded when the query session ends (no disk persistence)
- Cursors are held in-memory only for the duration of the query session; they are never written to disk
- The new cursor is strictly forward of the previous cursor within the same query (forward-only progress within a session)

## Invariants
- DI-001: Cursor forward progress -- new page cursor is always forward of the previous within a query session
- Pagination state is ephemeral: no disk I/O, no FileStore, no atomic writes for cursor data

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Caller provides a cursor from a different or expired query session | Structured error: `code: "E-MCP-004"`, suggestion: "Cursor is invalid or expired. Start a new query without a cursor parameter." |
| `PrismError::Sensor` | Sensor API rejects the pagination token (e.g., expired server-side cursor) | Structured error with suggestion to retry without cursor |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-003 | First query with no cursor parameter | Query starts from the beginning; first page establishes the initial pagination cursor |
| EC-01-004 | Page returns zero records but `has_more: true` | Next page requested with same cursor; adapter includes loop-detection counter to prevent infinite empty-page loops |
| EC-01-025 | Caller sends a cursor after a long delay (server restart, etc.) | Cursor is invalid (in-memory only, lost on restart); error returned with suggestion to start a new query |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001 |
| Addresses | ADV-1-002, ADV-1-006, ADV-2-005 |
| Priority | P0 |
