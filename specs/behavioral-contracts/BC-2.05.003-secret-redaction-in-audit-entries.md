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

# BC-2.05.003: Credential Values Are Never Present in Audit Entries

## Preconditions
- An MCP tool invocation includes parameters that reference or contain credential material (API keys, tokens, secrets)
- The audit middleware is serializing the `parameters` field of the `AuditEntry`

## Postconditions
- The `parameters` field in the audit entry replaces all credential values with a redaction marker (e.g., `"[REDACTED]"`)
- Credential names (e.g., `"api_key"`, `"client_secret"`) are preserved for traceability -- only the values are redacted
- The redaction applies to all known secret field patterns: fields ending in `_key`, `_secret`, `_token`, `_password`, `_credential`, and any field resolved via the credential store
- No credential value appears in `result_summary`, `capability_checks`, `safety_flags`, or any other audit entry field

## Invariants
- DI-002: Credential isolation per client -- credential values are never included in logs

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Unrecognized secret field | A new tool parameter contains a secret in a field not matching known patterns | The value is logged unredacted; this is a code-level bug caught by review and test. Known patterns must be updated when new secret fields are introduced. |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-004 | Tool parameter contains a nested JSON object with a secret buried in a sub-field | Redaction applies recursively to all fields matching secret patterns at any nesting depth |
| EC-05-005 | Tool parameter value coincidentally looks like a secret pattern name (e.g., a hostname containing `_token`) | Only fields whose names match secret patterns are redacted; values containing pattern substrings are not modified |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-002 |
| Priority | P0 |
