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

# BC-2.09.001: Structural Separation of Untrusted Data

## Preconditions
- A sensor query tool has fetched records from an external sensor API
- The records contain string fields that may include attacker-controlled content (hostnames, file paths, process names, descriptions)

## Postconditions
- All sensor-originated string values are placed in `structuredContent` JSON fields, never interpolated into `content[].text` prose
- The `content[].text` summary references counts, types, and severity levels but does not embed any sensor field values verbatim
- String values in `structuredContent` are JSON-encoded (string type), preserving their exact content without interpretation
- No string concatenation or interpolation of sensor data into narrative text occurs in the response construction code path

## Invariants
- DI-006: Prompt injection sanitization -- no MCP tool response interpolates untrusted sensor data into prose text

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | This is a construction-time constraint, not a runtime check | Enforced by code structure and integration tests |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-008 | Hostname contains prompt injection payload (`ignore all previous instructions...`) | Hostname placed in `structuredContent.hostname` as JSON string; `content[].text` says "1 detection found" without mentioning the hostname |
| EC-09-001 | Alert description field contains multi-line text with code fences | Description placed in `structuredContent.description` as JSON string; markdown syntax is not rendered |
| EC-09-002 | Sensor record contains an empty string field | Empty string placed normally in `structuredContent`; no special handling |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Priority | P0 |
