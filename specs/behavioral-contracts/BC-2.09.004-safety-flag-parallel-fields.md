---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
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

# BC-2.09.004: Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)

## Description

Safety detections from BC-2.09.003 are recorded exclusively in the centralized `_meta.safety_flags` array on the response envelope — there are no per-field parallel `{field}_safety_flag` fields. Each entry is a structured object identifying the field name, item index, pattern category, and matched pattern description. The original field value is never modified, stripped, or encoded. An empty array indicates no flags were triggered. The AuditEntry for the invocation also records all flags for forensic traceability.

## Preconditions
- Suspicious pattern detection (BC-2.09.003) has identified one or more matches in sensor record string fields
- The sensor record is being serialized into `structuredContent` for the MCP response

## Postconditions
- All safety detections are recorded in the centralized `_meta.safety_flags` array on the response envelope
- Each entry in the array is a structured object: `{"field": "{field_name}", "index": {item_index}, "category": "{pattern_category}", "pattern": "{matched_pattern_description}"}`
- There are NO per-field parallel `{field_name}_safety_flag` fields. Safety information is centralized in `_meta.safety_flags` only.
- The original field value is never modified, stripped, encoded, or truncated due to safety detection
- If no safety flags are triggered, `_meta.safety_flags` is an empty array
- The AuditEntry for the tool invocation includes the list of all safety flags triggered

## Invariants
- DI-006: Flagging is additive; original data integrity is preserved for forensic analysis
- Safety flags are centralized in `_meta.safety_flags`; no per-field parallel fields

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | This is an additive operation; it cannot fail independently of the response construction |  |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-09-008 | Multiple suspicious patterns match the same field | Multiple entries in `_meta.safety_flags` with the same `field` name but different `pattern` values |
| EC-09-010 | Sensor record has 50+ string fields, 10 are flagged | All 10 detections added to `_meta.safety_flags` array; performance impact is negligible |
| EC-09-012 | LLM agent reads `_meta.safety_flags` to understand risk | The array provides clear, structured context about which fields triggered which patterns, without polluting the data schema with parallel fields |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| One field matches one pattern | `_meta.safety_flags: [{"field": "hostname", "index": 0, "category": "prompt_injection", "pattern": "..."}]` | happy-path |
| One field matches two patterns | Two entries in `_meta.safety_flags` with same `field`, different `pattern` | edge-case |
| No patterns match | `_meta.safety_flags: []` | happy-path |
| 50 string fields, 10 flagged | All 10 flags in array; none in per-field parallel fields | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Addresses | ADV-1-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended Changelog row. |
| 1.1 | (prior) | product-owner | Prior remediation |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
