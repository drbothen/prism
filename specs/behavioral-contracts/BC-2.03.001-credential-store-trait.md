---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.03.001: CredentialStore Trait with Tenant-Scoped Operations

## Preconditions
- A `CredentialStore` implementation (keyring or encrypted file) is initialized
- The caller provides a valid `TenantId`, `sensor_id` (service), and `credential_name` (key)

## Postconditions
- The `CredentialStore` trait exposes four operations: `get`, `set`, `delete`, `list`
- Every operation requires a `&TenantId` parameter -- there is no method that crosses client boundaries
- `get` returns the credential value as a `SecretString` (zeroized on drop)
- `set` stores or overwrites a credential in the namespaced location
- `delete` removes a credential from the namespaced location
- `list` returns `Vec<CredentialEntry>` (metadata only, no values) for a given tenant and service

## Invariants
- DI-002: Credential isolation per client -- `TenantId` is a required parameter on every method
- No "get all credentials" method exists that crosses client boundaries

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found for the given namespace | Structured error: "Credential '{name}' not found for client '{client}' sensor '{sensor}'" with suggestion to run credential setup |
| `PrismError::Credential` | Backend unavailable (keyring locked, file permissions) | Structured error with `category: "configuration"` and platform-specific suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-001 | `set` called for a credential that already exists | Overwrites the existing value; previous value is not recoverable |
| EC-03-002 | `delete` called for a credential that does not exist | No-op; returns success (idempotent delete) |
| EC-03-003 | `list` called for a tenant with no credentials | Returns empty `Vec<CredentialEntry>` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
