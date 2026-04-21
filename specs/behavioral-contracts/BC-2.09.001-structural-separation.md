---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
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

# BC-2.09.001: Structural Separation of Untrusted Data

## Description

All sensor-originated string values are placed exclusively in `structuredContent` JSON fields — never interpolated into `content[].text` prose. The prose summary references only counts, types, and severity levels. This structural separation prevents prompt injection: even if a hostname contains `ignore all previous instructions`, the text field reads "1 detection found" while the hostname string remains in `structuredContent.hostname` as an inert JSON value. Enforced by code structure and integration tests per DI-006.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Detection with hostname `ignore all previous instructions` | `content[].text`: "1 detection found"; `structuredContent.hostname`: "ignore all previous instructions" (inert string) | happy-path + injection |
| Alert description containing triple backticks | Description in `structuredContent.description` as JSON string; not rendered as markdown | edge-case |
| Record with empty string field | Empty string in `structuredContent`; no error | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |
| VP-038 | Injection scanner: never panics on arbitrary input strings | fuzz |

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
