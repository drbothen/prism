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

# BC-2.09.004: Safety Flag Parallel Fields (Flag, Don't Strip)

## Preconditions
- Suspicious pattern detection (BC-2.09.003) has identified one or more matches in sensor record string fields
- The sensor record is being serialized into `structuredContent` for the MCP response

## Postconditions
- For each flagged field, a parallel field named `{field_name}_safety_flag` is added to the same JSON object
- The parallel field value is a string describing the detection: `"SUSPICIOUS: contains potential prompt injection pattern (matches '{pattern_description}')"` 
- The original field value is never modified, stripped, encoded, or truncated due to safety detection
- If a field has no safety flag, the `{field_name}_safety_flag` field is `null` (present but null, enabling the LLM to see the schema is consistent)
- The AuditEntry for the tool invocation includes the list of all safety flags triggered, with field names and pattern categories

## Invariants
- DI-006: Flagging is additive; original data integrity is preserved for forensic analysis

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | This is an additive operation; it cannot fail independently of the response construction |  |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-09-008 | Multiple suspicious patterns match the same field | `_safety_flag` contains all matched patterns separated by semicolons: `"SUSPICIOUS: pattern1; pattern2"` |
| EC-09-009 | A field name already ends with `_safety_flag` in the sensor data | The parallel field is named `{field_name}_safety_flag_safety_flag` (double suffix) to avoid collision |
| EC-09-010 | Sensor record has 50+ string fields, 10 are flagged | All 10 parallel fields added; performance impact is negligible (string operations only) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 |
| L2 Invariants | DI-006 |
| L2 Risk | R-005 |
| Priority | P0 |
