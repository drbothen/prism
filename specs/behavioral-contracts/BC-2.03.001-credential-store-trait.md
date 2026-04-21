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

# BC-2.03.001: CredentialStore Trait with Tenant-Scoped Operations

## Description

The `CredentialStore` trait defines the four-operation interface (`get`, `set`, `delete`, `list`) for all credential backends. Every operation requires a `&TenantId` parameter, enforcing at the type level that credential access is always scoped to a specific client. `get` returns a `SecretString` that zeroizes its content on drop; `list` returns metadata only (never values). There is no cross-client enumeration method.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.001-001 | `get(tenant_a, "crowdstrike", "client_secret")` — credential exists | Returns `SecretString` with the value; `SecretString::Display` → `"[REDACTED]"` |
| TV-BC-2.03.001-002 | `get(tenant_a, "crowdstrike", "missing_key")` — credential absent | `PrismError::Credential` with name/client/sensor in message |
| TV-BC-2.03.001-003 | `set` over existing credential | Overwrites; previous value gone |
| TV-BC-2.03.001-004 | `delete` for nonexistent credential | Returns success (idempotent) |
| TV-BC-2.03.001-005 | `list(tenant_a, "crowdstrike")` with no stored credentials | Returns empty `Vec<CredentialEntry>` |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |
| VP-034 | Encryption round-trip: encrypt then decrypt returns plaintext (proptest) |

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
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-011/VP-034 cross-reference; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
