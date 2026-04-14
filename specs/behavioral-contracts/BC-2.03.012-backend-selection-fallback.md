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

# BC-2.03.012: Credential Backend Selection and Fallback

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
