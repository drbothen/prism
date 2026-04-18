---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Adapters"
capability: [CAP-001, CAP-002]
---

# BC-2.01.010: Partial Failure Handling for Paginated and Cross-Client Queries

## Preconditions
- A sensor query (single-client or cross-client) is in progress
- At least one page or one client's query has succeeded before a failure occurs

## Postconditions
- Successfully fetched data is returned to the caller (not discarded)
- Response includes `truncated: true` when pagination was interrupted
- Response includes `truncation_reason` describing the failure (e.g., "sensor_unavailable", "rate_limited", "authentication_expired")
- For cross-client queries, `partial_failures` array lists each failed client with error category and suggestion
- Cursor advances only to the last successfully fetched and delivered page

## Invariants
- DI-001: Cursor advances only for successfully delivered pages (ephemeral in-memory cursor is not advanced beyond the last successful page)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | HTTP 503 mid-pagination | Not a tool-level error; partial results returned with metadata |
| N/A | HTTP 429 after backoff exhaustion | Not a tool-level error; partial results returned with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-001 | HTTP 503 after some pages fetched | Return fetched pages with `truncated: true` and `truncation_reason: "sensor_unavailable"`; cursor at last successful page |
| EC-01-014 | First page fails (no data fetched) | Empty results with full error in metadata; this is still not a tool-level error for cross-client queries |
| EC-01-015 | Network timeout during a single-client query | Return any fetched pages as partial; if no pages fetched, return structured error with timeout details and retry suggestion |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001, CAP-002 |
| L2 Invariants | DI-001 |
| Priority | P0 |
