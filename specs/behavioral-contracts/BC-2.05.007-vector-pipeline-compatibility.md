---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Audit Trail"
capability: "CAP-007"
---

# BC-2.05.007: Audit Entries Are Compatible with the Vector Pipeline

## Preconditions
- The `tracing` subscriber is configured with a JSON output format
- The downstream Vector pipeline expects structured JSON log lines on stderr (or a configured output)

## Postconditions
- Each audit entry is emitted as a single-line JSON object (no multi-line pretty-printing)
- The JSON structure uses `snake_case` field names consistent with convention-reconciliation.md
- Standard fields (`timestamp`, `level`, `target`, `span`, `fields`) follow `tracing-subscriber` JSON layer conventions
- Prism-specific audit fields are nested under a consistent key (e.g., `fields.audit.*` or top-level structured fields) parseable by Vector's JSON parser
- The `timestamp` field uses ISO 8601 format with UTC timezone, parseable by Vector's `parse_timestamp` transform

## Invariants
- DI-004: Audit completeness -- Vector compatibility ensures the pipeline can ingest all entries

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Serialization failure | A field value cannot be serialized to JSON (e.g., non-UTF-8 string) | The field value is replaced with a placeholder string `"<serialization_error>"` and the entry is still emitted |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-011 | Audit entry contains very large `parameters` or `result_summary` (e.g., a bulk query with thousands of filter terms) | The entry is emitted as-is; truncation is not applied at the Prism layer. Vector pipeline configuration handles size limits if needed. |
| EC-05-012 | Tracing subscriber outputs to stderr which is captured by the MCP host process | Audit entries on stderr do not interfere with MCP JSON-RPC on stdout; the two streams are strictly separated |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |
