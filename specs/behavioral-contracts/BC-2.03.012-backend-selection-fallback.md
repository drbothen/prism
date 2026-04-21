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
input-hash: "365fb25"
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

# BC-2.03.012: Credential Backend Selection and Fallback

## Description

At startup, Prism selects exactly one credential backend for the session. Explicit configuration (`credential_backend = "keyring"` or `"encrypted_file"`) is honored strictly with no fallback. When no backend is specified, keyring is attempted first; if unavailable, encrypted file is used as fallback. The selected backend is logged at startup and is deterministic given the same configuration and environment. Per-credential backend mixing within a session is not permitted.

## Preconditions
- Prism is initializing the credential store
- Configuration may explicitly specify a backend or rely on automatic selection

## Postconditions
- If `credential_backend = "keyring"` is configured, only the keyring backend is used (no fallback)
- If `credential_backend = "encrypted_file"` is configured, only the encrypted file backend is used
- If no backend is specified, keyring is attempted first; if unavailable (platform limitation or locked), encrypted file is used as fallback
- The selected backend is logged at startup (`info` level)
- All credentials for a session use the same backend (no per-credential backend mixing)

## Invariants
- Exactly one backend is active per session
- Backend selection is deterministic given the same configuration and environment

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Explicit `keyring` configured but keyring unavailable | Fatal startup error; no fallback (explicit config is honored strictly) |
| `PrismError::Credential` | Explicit `encrypted_file` configured but encryption key missing | Fatal startup error with suggestion to set the encryption key env var |
| `PrismError::Credential` | Auto-select: both keyring and encrypted file fail | Fatal startup error listing both failure reasons |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-028 | Container deployment with no keyring available | Auto-select falls back to encrypted file; this is the expected container deployment pattern |
| EC-03-029 | Switching backends between sessions (credentials stored in keyring, now using encrypted file) | Credentials from previous backend are not migrated; operator must re-store credentials in the new backend |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.012-001 | Auto-select; keyring available | Keyring selected; logged at info level |
| TV-BC-2.03.012-002 | Auto-select; keyring unavailable; encrypted file available with key | Encrypted file selected; logged at info level |
| TV-BC-2.03.012-003 | Explicit `keyring` config; keyring unavailable | Fatal startup error; no fallback to encrypted file |
| TV-BC-2.03.012-004 | Explicit `encrypted_file` config; no encryption key | Fatal startup error with env var suggestion |
| TV-BC-2.03.012-005 | Container with no keyring (EC-03-028) | Auto-select falls back to encrypted file |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

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
