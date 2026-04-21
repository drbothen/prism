---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "572c2a9"
traces_to: ["CAP-004"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.03.007: Secret Redaction in Logs, Errors, and MCP Responses

## Description

The `SecretString` type enforces redaction at every output surface. Its `Display` impl returns `"[REDACTED]"` and `Debug` impl returns `"SecretString([REDACTED])"`, making accidental leakage through format strings impossible by construction. Error messages reference credentials by namespace only (name, client, sensor — never value). The `--dry-run` validator displays a partial preview (`first2 + "***" + last2`) to confirm a credential is non-empty without exposing its value. The type zeroizes memory on drop.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.007-001 | `format!("{}", secret)` where secret is a SecretString | Output is `"[REDACTED]"`; never the actual value |
| TV-BC-2.03.007-002 | `format!("{:?}", secret)` (debug format) | Output is `"SecretString([REDACTED])"` |
| TV-BC-2.03.007-003 | `--dry-run` with credential value "abcdefyz" (8 chars) | Displayed as `"ab***yz"` |
| TV-BC-2.03.007-004 | `--dry-run` with 1-character credential | Displayed as `"***"` (no char leakage) |
| TV-BC-2.03.007-005 | MCP response containing a credential operation result | No `results`, `content`, or `_meta` field contains the credential value |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
