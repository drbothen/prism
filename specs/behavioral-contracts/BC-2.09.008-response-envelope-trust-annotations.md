---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "abc4070"
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

# BC-2.09.008: Response Envelope with Trust Annotations

## Description

Every tool response is wrapped in a consistent `structuredContent` envelope with a `_meta` section and a `results` array. The `_meta` section carries `tool`, `data_source`, `query_time`, `trust_level`, `safety_flags`, `total_results`, `page`, `has_more`, and `next_cursor`. `_meta.safety_flags` is always present (empty array when no flags triggered). The `content[].text` prose summary begins with a provenance marker and contains only aggregate counts, never individual sensor field values, enforcing the DI-004/DI-006 separation between trusted metadata and untrusted results.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| CrowdStrike query returning 5 detections | `_meta.total_results: 5`, `_meta.safety_flags: []`, `results` array with 5 items | happy-path |
| Query returning zero results | `_meta.total_results: 0`, `results: []`, `has_more: false`; envelope present | edge-case |
| Cross-client query from 3 sensors | `_meta.data_source` is array of 3 sensor IDs; each result has `source_sensor` | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-004, DI-006 |
| L2 Edge Cases | DEC-008 |
| L2 Risk | R-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
