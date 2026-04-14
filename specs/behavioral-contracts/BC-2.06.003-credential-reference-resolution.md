---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Client Configuration"
capability: "CAP-009"
---

# BC-2.06.003: Credential References in Config Resolve to Credential Store Entries

## Preconditions
- A sensor config entry includes a `credential_ref` field
- The credential store backend (keyring or encrypted file) is accessible

## Postconditions
- The `credential_ref` value is resolved against the credential store using the `(client_id, sensor_id, credential_name)` namespace
- Resolution follows the priority chain: `_FILE` env var suffix > env var > credential store
  - `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL_FILE` reads the secret from a file path (K8s secret mount pattern)
  - `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL` reads the secret from the env var value
  - Fallback: credential store lookup by `(client_id, sensor_id, credential_ref)`
- The resolved credential is available as a `SecretString` that is never logged or serialized

## Invariants
- DI-002: Credential isolation per client -- resolution is always scoped by `client_id`
- DI-014: Credential name sanitization -- `credential_ref` is validated against `[a-zA-Z0-9_\-\.]+`

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found in any resolution tier | Error: "Credential '{ref}' for clients.{id}.sensors.{sensor} not found. Check credential store or set PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL env var." |
| `PrismError::InvalidInput` | `credential_ref` contains characters outside `[a-zA-Z0-9_\-\.]+` | Error: "Invalid credential reference '{ref}': must match [a-zA-Z0-9_\\-\\.]+" |
| `PrismError::Credential` | `_FILE` env var points to a non-existent or unreadable file | Error: "Credential file '{path}' referenced by PRISM_...CREDENTIAL_FILE not found or unreadable" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | OS keyring is locked at startup | `PrismError::Credential` with suggestion to unlock keychain or use encrypted file fallback |
| EC-06-003 | `_FILE` env var is set but file content has trailing newline | File content is trimmed of leading/trailing whitespace before use as credential value |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-002, DI-014 |
| Priority | P0 |
