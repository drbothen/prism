---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "dc078d2"
traces_to: ["CAP-010"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-09"
capability: "CAP-010"
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

# BC-2.09.007: OutputSchema for Type-Safe LLM Reasoning

## Description

Every MCP tool defines an `outputSchema` (JSON Schema) describing its successful response structure, derived from Rust types via `JsonSchema` derive at compile time. The schema explicitly declares `_meta` envelope fields (`tool`, `data_source`, `query_time`, `trust_level`, `safety_flags`, pagination) and typed `results` array items with field names, types, and descriptions. Declaring `_meta.safety_flags` as a structured array in the schema makes the centralized flagging pattern visible to the LLM before it sees any data, improving field extraction reliability and reinforcing the data/metadata boundary per DI-006.

## Preconditions
- MCP tools are being registered via `tools/list`
- The MCP protocol version supports `outputSchema` on tool definitions

## Postconditions
- Every MCP tool defines an `outputSchema` (JSON Schema) describing the structure of its successful response
- The `outputSchema` includes the `_meta` envelope fields: `tool`, `data_source`, `query_time`, `trust_level`, `safety_flags`, pagination fields
- The `outputSchema` includes typed definitions for the `results` array items, with field names, types, and descriptions
- The `_meta.safety_flags` array is declared in the schema as `type: "array"` with structured items so the LLM knows where to find safety annotations (centralized, not per-field parallel fields)
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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Inspect `outputSchema` of a CrowdStrike sensor tool | Schema includes `_meta.trust_level`, `_meta.safety_flags` (typed array), `results` (typed array items) | happy-path |
| Response includes `raw_extensions` vendor field | `additionalProperties: true` on `raw_extensions` object in schema | edge-case |
| OCSF query response with mixed event classes | Base OCSF fields typed; class-specific in `additionalProperties` | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Every tool has `outputSchema` including `_meta.safety_flags` typed array | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
