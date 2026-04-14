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

# BC-2.03.003: AES-256-GCM Encrypted File Backend Fallback

## Preconditions
- The `EncryptedFileBackend` is selected (explicitly configured or as fallback when keyring is unavailable)
- An encryption key is provided via environment variable or K8s secret mount (`_FILE` suffix)

## Postconditions
- Each credential is stored as a separate file: `{credentials_dir}/{client_id}/{sensor_id}/{credential_name}.enc`
- Files are encrypted with AES-256-GCM using a unique random salt (nonce) per credential
- The encryption key is derived from the provided key material (never stored on disk alongside the encrypted files)
- Files are created with mode `0600`; parent directories with mode `0700`
- Credential files use the atomic temp-fsync-rename pattern for crash safety

## Invariants
- DI-002: Credential isolation -- file paths namespaced by `client_id`
- Credentials encrypted at rest (ISO 27001 requirement)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Encryption key not provided (env var missing) | `category: "configuration"`, suggestion: "Set PRISM_CREDENTIAL_KEY or PRISM_CREDENTIAL_KEY_FILE environment variable" |
| `PrismError::Credential` | Decryption fails (wrong key, corrupted file) | `category: "data"`, suggestion: "Credential file may be corrupted or encrypted with a different key. Re-create the credential." |
| `PrismError::Io` | Credentials directory not writable | Structured error with the path and permission details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-006 | Encryption key rotated -- existing credentials encrypted with old key | Decryption fails for existing credentials; operator must re-set all credentials with the new key |
| EC-03-007 | File permissions changed by external process (e.g., chmod 777) | Prism detects overly permissive files at startup; warning logged recommending `chmod 600` |
| EC-03-008 | Credential file exists but is zero bytes | Treated as corrupted; `PrismError::Credential` with suggestion to re-create |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
