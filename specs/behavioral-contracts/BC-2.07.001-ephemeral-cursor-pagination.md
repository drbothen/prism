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

# BC-2.07.001: Ephemeral Pagination Token Structure

## Preconditions
- A sensor adapter produces records from a data source in pages
- Each page response from the sensor API includes a continuation token or offset for the next page

## Postconditions
- The pagination token is an opaque, ephemeral value held in-memory for the duration of a query session
- The token encapsulates the sensor-specific pagination state (e.g., CrowdStrike offset string, Claroty page number, Armis AQL cursor)
- The token is serialized as an opaque base64-encoded string in the MCP response `_meta.pagination.cursor` field
- The caller passes the token back on the next page request; the server decodes it to resume pagination
- Tokens are never persisted to disk. They exist only in the server's in-memory state for the active query.
- Token structure is internal to Prism and not a public API contract; the caller treats it as an opaque string

## Invariants
- Pagination tokens are ephemeral (in-memory only, no disk persistence)
- Token deserialization failure produces a structured error, not a panic

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Token cannot be decoded (corrupted, tampered, or from a different server instance) | Structured error: `code: "E-MCP-004"`, suggestion: "Pagination cursor is invalid. Start a new query without a cursor." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-001 | Sensor API returns a cursor type that differs between pages (e.g., numeric then string) | Token encapsulates the raw value; Prism normalizes internally |
| DEC-010 | Claroty returns polymorphic ID (number in one record, string in next) | Both normalize to string within the token; `12345` and `"12345"` are equivalent |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| Replaces | BC-2.07.001 v1.0 (persistent composite cursor) |
| Addresses | ADV-1-002, ADV-1-006, ADV-2-005 |
| Priority | P0 |
