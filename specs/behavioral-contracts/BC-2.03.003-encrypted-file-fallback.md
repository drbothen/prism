---
document_type: behavioral-contract
level: L3
version: "1.1"
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
- Key derivation uses HKDF-SHA256: the provided key material is passed through HKDF-SHA256 to produce the 256-bit AES key. A fixed application-specific info string (`"prism-credential-v1"`) is used. No salt is needed for HKDF when the input key material has sufficient entropy (environment variable or K8s secret).
- Each encryption operation generates a fresh 96-bit random nonce (one nonce per encryption operation, not per credential lifetime). The nonce is stored prepended to the ciphertext in the `.enc` file: `[12-byte nonce][ciphertext+tag]`.
- The derived encryption key is never stored on disk alongside the encrypted files
- Files are created with mode `0600`; parent directories with mode `0700`
- Credential files use the atomic temp-fsync-rename pattern for crash safety

## Invariants
- DI-002: Credential isolation -- file paths namespaced by `client_id`
- Credentials encrypted at rest (ISO 27001 requirement)
- Nonce uniqueness: a fresh 96-bit random nonce per encryption operation ensures nonce reuse probability is negligible

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Encryption key not provided (env var missing) | `category: "configuration"`, suggestion: "Set PRISM_CREDENTIAL_KEY or PRISM_CREDENTIAL_KEY_FILE environment variable" |
| `PrismError::Credential` | Decryption fails (wrong key, corrupted file, tampered ciphertext) | `category: "data"`, suggestion: "Credential file may be corrupted or encrypted with a different key. Re-create the credential." |
| `PrismError::Io` | Credentials directory not writable | Structured error with the path and permission details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-006 | Encryption key rotated -- existing credentials encrypted with old key | Decryption fails for existing credentials; operator must re-set all credentials with the new key |
| EC-03-007 | File permissions changed by external process (e.g., chmod 777) | Prism detects overly permissive files at startup; warning logged recommending `chmod 600` |
| EC-03-008 | Credential file exists but is zero bytes | Treated as corrupted; `PrismError::Credential` with suggestion to re-create |
| EC-03-009 | Credential file shorter than 12 bytes (nonce incomplete) | Treated as corrupted; nonce cannot be extracted |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Addresses | ADV-1-011, ADV-2-007 |
| Priority | P0 |
