---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Prompt Injection Defense"
capability: "CAP-010"
---

# BC-2.09.008: Response Envelope with Trust Annotations

## Preconditions
- A sensor query tool has produced results ready for MCP response formatting
- Safety scanning (BC-2.09.003) and parallel field generation (BC-2.09.004) are complete

## Postconditions
- Every tool response is wrapped in a consistent envelope structure within `structuredContent`:
  ```
  {
    "_meta": {
      "tool": "<tool_name>",
      "data_source": "<sensor_id>",
      "query_time": "<ISO8601 timestamp>",
      "trust_level": "untrusted_external" | "internal",
      "safety_flags": ["<field_name on item_N>", ...],
      "total_results": <integer>,
      "page": <integer>,
      "has_more": <boolean>,
      "next_cursor": "<cursor_string>" | null
    },
    "results": [...]
  }
  ```
- The `_meta.safety_flags` array is always present (empty array when no flags triggered)
- The `_meta.query_time` reflects when Prism fetched the data from the sensor API
- The `_meta.data_source` identifies the sensor that produced the data
- The `content[].text` prose summary begins with the provenance marker and includes aggregate counts, never individual record field values

## Invariants
- DI-006: Envelope structure enforces separation between metadata (trusted) and results (untrusted)
- DI-004: Audit completeness -- safety_flags from the envelope are also recorded in the AuditEntry

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | Envelope construction is deterministic | No runtime failure possible independent of the underlying query |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-09-018 | Query returns zero results | `_meta.total_results: 0`, `results: []`, `has_more: false`; envelope is still present |
| EC-09-019 | Cross-client query with multiple sensors | `_meta.data_source` is an array of sensor IDs; each result item includes `source_sensor` field |
| EC-09-020 | Response truncated due to sensor unavailability mid-pagination | `_meta` includes `truncated: true`, `truncation_reason: "sensor_unavailable"` alongside normal envelope fields |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-004, DI-006 |
| L2 Edge Cases | DEC-008 |
| L2 Risk | R-005 |
| Priority | P0 |
