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

# BC-2.09.007: OutputSchema for Type-Safe LLM Reasoning

## Preconditions
- MCP tools are being registered via `tools/list`
- The MCP protocol version supports `outputSchema` on tool definitions

## Postconditions
- Every MCP tool defines an `outputSchema` (JSON Schema) describing the structure of its successful response
- The `outputSchema` includes the `_meta` envelope fields: `tool`, `data_source`, `query_time`, `trust_level`, `safety_flags`, pagination fields
- The `outputSchema` includes typed definitions for the `results` array items, with field names, types, and descriptions
- Parallel `_safety_flag` fields are declared in the schema as `type: ["string", "null"]` so the LLM expects them
- Error responses follow a separate schema defined on the error path (not in `outputSchema`)
- The schema enables the LLM to reason about response structure before seeing actual data, improving field extraction reliability

## Invariants
- DI-006: OutputSchema makes the data/metadata boundary explicit to the LLM

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | Static configuration | Schema is derived from Rust types via `JsonSchema` derive at compile time |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-09-016 | Tool response includes dynamic fields not in the schema (e.g., vendor-specific `raw_extensions`) | `raw_extensions` is declared as `type: "object"` with `additionalProperties: true` in the schema |
| EC-09-017 | OCSF fields vary by event class | OCSF portion of schema uses a base set of common fields; class-specific fields are in `additionalProperties` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Priority | P0 |
