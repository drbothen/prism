---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Prompt Injection Defense"
capability: "CAP-010"
---

# BC-2.09.004: Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Addresses | ADV-1-008 |
| Priority | P0 |
