---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Credential Management"
capability: "CAP-004"
---

# BC-2.03.007: Secret Redaction in Logs, Errors, and MCP Responses

## Preconditions
- A credential value has been loaded into memory as a `SecretString`

## Postconditions
- `SecretString` implements `Display` as `"[REDACTED]"` -- never exposes the value in format strings
- `SecretString` implements `Debug` as `"SecretString([REDACTED])"` -- safe for debug logging
- Error messages referencing credentials include the credential name and namespace but never the value
- MCP tool responses never include credential values in any field (`results`, `content`, `_meta`)
- Audit log entries record credential access events with namespace but never values
- The `--dry-run` config validator displays credentials as `first2 + "***" + last2` (e.g., `ab***yz`)

## Invariants
- DI-002: Credential values never in logs, MCP responses, or error messages
- SOC 2 / ISO 27001: Credential values encrypted at rest, redacted in transit

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Code accidentally passes `SecretString` to a format macro | `Display` impl returns `"[REDACTED]"` -- defense in depth |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-016 | Stack trace includes function that had `SecretString` parameter | `SecretString` is zeroized on drop; stack traces show the type name, not the value |
| EC-03-017 | `--dry-run` with a 1-character credential | Displayed as `"***"` (no first/last character leak for short values) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
