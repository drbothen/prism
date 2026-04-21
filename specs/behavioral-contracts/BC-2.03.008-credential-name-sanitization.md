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
input-hash: "85d7741"
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

# BC-2.03.008: Credential Name Sanitization Against Path Traversal

## Description

All credential names are validated against the pattern `[a-zA-Z0-9_\-\.]+` at the `CredentialStore` trait boundary before any backend operation is invoked. This validation prevents path traversal attacks (per HS-007-06) by rejecting names containing path separators (`/`, `\`), null bytes, or other disallowed characters. The error message identifies both the rejected name and the specific invalid characters. Leading dots and consecutive separators are permitted.

## Preconditions
- A credential operation is invoked with a `credential_name` parameter
- Validation occurs at the `CredentialStore` trait boundary, before any backend is invoked

## Postconditions
- `credential_name` is validated against the pattern `[a-zA-Z0-9_\-\.]+`
- Names containing path separators (`/`, `\`), null bytes, or other disallowed characters are rejected
- Validation error includes the rejected name and the allowed pattern
- This validation prevents path traversal attacks (serveMyAPI vulnerability HS-007-06)

## Invariants
- DI-014: Credential name sanitization -- validated before any filesystem or keyring operation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Name contains `/` (e.g., `../../etc/passwd`) | Rejected with pattern and the specific invalid characters identified |
| `PrismError::InvalidInput` | Name contains null byte (`\0`) | Rejected immediately |
| `PrismError::InvalidInput` | Name is empty string | Rejected: "Credential name must be non-empty" |
| `PrismError::InvalidInput` | Name contains spaces | Rejected with the allowed pattern |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-018 | Name with leading dot (e.g., `.hidden_key`) | Valid per pattern (dot is allowed); stored and retrievable |
| EC-03-019 | Name with consecutive dashes/dots (e.g., `key--name..v2`) | Valid per pattern; no normalization applied |
| EC-03-020 | Name at maximum length (256 characters of valid chars) | Accepted; backend-specific limits may apply (keyring services have service name length limits) |
| EC-03-021 | Name exceeds backend limit (e.g., keyring service name > 255 chars) | Backend returns error; mapped to `PrismError::Credential` with suggestion to use a shorter name |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.008-001 | Name `../../etc/passwd` (path traversal) | `PrismError::InvalidInput`; pattern violation; `/` identified as disallowed |
| TV-BC-2.03.008-002 | Name contains null byte `\0` | `PrismError::InvalidInput`; rejected immediately |
| TV-BC-2.03.008-003 | Empty name `""` | `PrismError::InvalidInput`; "Credential name must be non-empty" |
| TV-BC-2.03.008-004 | Valid name `my-api-key.v2` | Accepted; passes to backend |
| TV-BC-2.03.008-005 | Valid name `.hidden_key` (leading dot) | Accepted per pattern |
| TV-BC-2.03.008-006 | Name with spaces | `PrismError::InvalidInput`; allowed pattern shown |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-014 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-011; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
