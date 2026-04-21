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
input-hash: "8e43eb2"
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

# BC-2.03.003: AES-256-GCM Encrypted File Backend Fallback

## Description

The `EncryptedFileBackend` stores each credential as a separate file at `{credentials_dir}/{client_id}/{sensor_id}/{credential_name}.enc`, encrypted with AES-256-GCM. Key derivation uses HKDF-SHA256 with a random 32-byte salt per credential file and a fixed application info string `"prism-credential-v1"`. Each encryption operation generates a fresh 96-bit random nonce. Files use the atomic temp-fsync-rename pattern for crash safety and are created with mode `0600`. The derived key is never stored on disk.

## Preconditions
- The `EncryptedFileBackend` is selected (explicitly configured or as fallback when keyring is unavailable)
- An encryption key is provided via environment variable or K8s secret mount (`_FILE` suffix)

## Postconditions
- Each credential is stored as a separate file: `{credentials_dir}/{client_id}/{sensor_id}/{credential_name}.enc`
- Key derivation uses HKDF-SHA256: the provided key material is passed through HKDF-SHA256 to produce the 256-bit AES key. A fixed application-specific info string (`"prism-credential-v1"`) is used. A random 32-byte salt is generated per credential file and stored prepended to the ciphertext. This ensures distinct derived keys per credential even if the same master key material is used across deployments.
- Each encryption operation generates a fresh 96-bit random nonce (one nonce per encryption operation, not per credential lifetime). The file format is: `[32-byte salt][12-byte nonce][ciphertext+tag]`.
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
| EC-03-009 | Credential file shorter than 44 bytes (salt + nonce incomplete) | Treated as corrupted; salt and nonce cannot be extracted |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.003-001 | Encrypt and decrypt same credential with same key | Roundtrip produces original plaintext; VP-034 |
| TV-BC-2.03.003-002 | Decrypt with wrong key | `PrismError::Credential` with `category: "data"` and re-create suggestion |
| TV-BC-2.03.003-003 | Missing PRISM_CREDENTIAL_KEY env var | `PrismError::Credential` with env var set suggestion |
| TV-BC-2.03.003-004 | Zero-byte credential file (EC-03-008) | Treated as corrupted; structured error; re-create suggestion |
| TV-BC-2.03.003-005 | File shorter than 44 bytes (EC-03-009) | Treated as corrupted; salt/nonce extraction fails |
| TV-BC-2.03.003-006 | Two separate encryptions of same value | Different salt and nonce each time; ciphertexts differ |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-034 | Encryption round-trip: encrypt then decrypt returns plaintext (proptest) |
| VP-035 | Key derivation: same inputs produce same key (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Addresses | ADV-1-011, ADV-2-007 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-034/VP-035; added ## Changelog. Note: file was previously version 1.1 (pre-existing bump) — no additional version bump needed; Changelog row added only. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
